//! Some of this module (meshing and shader code, etc) is based on code by Morgan McGuire, which
//! appeared in his blog Casual Effects
//! [here](http://casual-effects.blogspot.com/2014/04/fast-terrain-rendering-with-continuous.html).
//! The original source is available here: https://github.com/morgan3d/misc/tree/master/terrain
//!
//! The license of the original source follows:
//!
//! ```
//! MIT License
//!
//! Copyright (c) 2016 morgan3d
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//! ```
use crate::{AppState, STATE_STAGE};
use bevy::app::{AppBuilder, Plugin};
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::tasks::AsyncComputeTaskPool;
use std::future::Future;
use std::io::Read;
use std::pin::Pin;
use bevy::render::texture::{Extent3d, TextureFormat, AddressMode, SamplerDescriptor, TextureDimension};
use itertools::Itertools;
use byteorder::WriteBytesExt;
use std::cmp;
use ordered_float::OrderedFloat;
use crate::map::shader::MapMaterial;
use crate::map::mipmap::HeightmapMipMap;
use crate::loading::time;

pub mod mesh;
pub mod shader;
pub mod mipmap;

pub struct RomeMapPlugin;

impl Plugin for RomeMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<HeightMap>()
            .add_asset::<MapMaterial>()
            .add_startup_system(shader::setup.system())
            .on_state_update(
                STATE_STAGE,
                AppState::InGame,
                shader::update_time.system(),
            )
            .on_state_update(
                STATE_STAGE,
                AppState::InGame,
                translate_meshes.system(),
            );
    }
}

fn translate_meshes(camera: Query<&goshawk::RtsCamera>, mut meshes: Query<(&mut Transform, &Handle<MapMaterial>)>) {
    for (mut mesh, _material) in meshes.iter_mut() {
        if let Some(camera) = camera.iter().next() {
            mesh.translation = camera.looking_at;
        }
    }
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct HeightMapLoader {
    pub task_pool: AsyncComputeTaskPool,
}

impl AssetLoader for HeightMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        ctx: &'a mut LoadContext,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            // TODO no block
            let asset = self.task_pool.scope(|scope| {
                scope.spawn(async {
                    time("Loading heightmap", || {
                        println!("Unzipping");

                        let mut decoder = zstd::Decoder::new(&*bytes).unwrap();
                        let mut unzipped = Vec::new();
                        decoder.read_to_end(&mut unzipped).unwrap();

                        println!("Deserializing");
                        let map: rome_map::Map = bincode::deserialize(&unzipped).unwrap();

                        dbg!(map.height, map.width);
                        println!("Done loading heightmap");

                        HeightMap(map)
                    })
                })
            });

            ctx.set_default_asset(LoadedAsset::new(asset.into_iter().next().unwrap()));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mapdat"]
    }
}

#[derive(TypeUuid, Clone)]
#[uuid = "7b7c08b3-986e-49d8-85da-107024f177f1"]
pub struct HeightMap(pub rome_map::Map);

fn clamp(a: i32, max: u32) -> u32 {
    cmp::max(cmp::min(a, max as i32), 0) as u32
}

const Y_SCALE: f32 = 0.2;
const XYZ_SCALE: f32 = 1.0 / 8.0;
pub const LIGHT_POS: [f32; 3] = [-1.0, 0.2, -0.3];

impl HeightMap {
    fn sample_height_water(&self, x: i32, y: i32) -> (u16, bool) {
        let px = self.0.get(clamp(x, self.0.width as u32), clamp(y, self.0.height as u32));
        if px.is_water {
            (0, true)
        } else {
            (i16::max(0, px.height.0) as u16, false)
        }
    }

    fn sample_water(&self, x: i32, y: i32) -> bool {
        let px = self.0.get(clamp(x, self.0.width as u32), clamp(y, self.0.height as u32));
        px.is_water
    }

    fn sample_vec3(&self, x: i32, y: i32, height_factor: f32) -> Vec3 {
        Vec3::new(x as f32, self.sample_height_water(x, y).0 as f32 * height_factor * Y_SCALE * XYZ_SCALE, y as f32)
    }

    fn sample_normal(&self, x: i32, y: i32, height_factor: f32) -> Vec3 {
        let top_left = self.sample_vec3(x, y, height_factor);
        let bottom_left = self.sample_vec3(x, y + 1, height_factor);
        let bottom_right = self.sample_vec3(x + 1, y + 1, height_factor);

        (bottom_right - bottom_left).cross(top_left - bottom_left).normalize()
    }
}

impl Into<(Texture, u16)> for &HeightMap {
    fn into(self) -> (Texture, u16) {
        const HEIGHT_BITS: u8 = 8;
        const LIGHT_BITS: u8 = 8;
        const MAX_LIGHT_LEVEL: u8 = ((1u16 << LIGHT_BITS) - 1) as u8;
        const AMBIENT_LIGHT_STRENGTH: OrderedFloat<f32> = OrderedFloat(0.1);

        let light_pos: Vec3 = LIGHT_POS.into();
        let light_pos = light_pos.normalize();

        let mut max = 0;
        for (y, x) in (0..self.0.height).cartesian_product(0..self.0.width) {
            let h = self.0.get(x as u32, y as u32).height.0;
            if h > max {
                max = h;
            }
        }

        let factor = ((1 << HEIGHT_BITS) - 1) as f32 / max as f32;

        let mut bytes = Vec::with_capacity(self.0.height * self.0.width * 2);

        for (y, x) in (0..(self.0.height as i32)).cartesian_product(0..(self.0.width as i32)) {
            let normal = self.sample_normal(x, y, factor);

            let diffuse = cmp::max(OrderedFloat(normal.dot(light_pos)), OrderedFloat(0.0));
            let brightness = cmp::min(OrderedFloat(1.0), diffuse + AMBIENT_LIGHT_STRENGTH);

            let brightness_level = (brightness.0 as f32 * MAX_LIGHT_LEVEL as f32).round() as u8;
            let (height, water) = self.sample_height_water(x, y);
            let height = (height as f32 * factor).round() as u8;

            let terrain_type = if !water {
                // TODO do better
                let adjacent_water = self.sample_water(x + 1, y) ||
                    self.sample_water(x - 1, y) ||
                    self.sample_water(x, y + 1) ||
                    self.sample_water(x, y - 1) ||
                    self.sample_water(x + 1, y + 1) ||
                    self.sample_water(x + 1, y - 1) ||
                    self.sample_water(x - 1, y + 1) ||
                    self.sample_water(x - 1, y - 1);

                if adjacent_water {
                    2
                } else {
                    0
                }
            } else {
                1
            };


            bytes.write_u8(height).unwrap(); // R channel = height
            bytes.write_u8(brightness_level).unwrap(); // G channel = brightness level
            bytes.write_u8(terrain_type).unwrap(); // B channel = terrain type
            bytes.write_u8(0).unwrap(); // A channel is unused
        }

        let texture = Texture {
            data: bytes,
            size: Extent3d::new(self.0.width as u32, self.0.height as u32, 1),
            format: TextureFormat::Rgba8Uint,
            dimension: TextureDimension::D2,
            sampler: SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                ..Default::default()
            },
        };

        (texture, max as u16)
    }
}

impl HeightmapMipMap {
    pub fn to_texture(&self, max_y: u16) -> Texture {
        const HEIGHT_BITS: u8 = 8;

        let factor = ((1 << HEIGHT_BITS) - 1) as f32 / max_y as f32;
        let mut bytes = Vec::with_capacity(self.height * self.width);

        for (y, x) in (0..self.height).cartesian_product(0..self.width) {
            bytes.write_u8((self.get(x as u32, y as u32).0 as f32 * factor) as u8).unwrap();
        }

        Texture {
            data: bytes,
            size: Extent3d::new(self.width as u32, self.height as u32, 1),
            format: TextureFormat::R8Uint,
            dimension: TextureDimension::D2,
            sampler: SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                ..Default::default()
            },
        }
    }
}

const ZOOM: u8 = 3;
const THREE_POW_ZOOM: f32 = 3.0 * 3.0 * 3.0;

pub const TOP_LEFT_TILE: TileCoord = TileCoord { x: 23.0, y: 3.0 };
pub const TOP_LEFT_LAT_LONG: LatLong = LatLong { latitude: 70.0, longitude: -26.6666 };

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LatLong {
    pub latitude: f32,
    pub longitude: f32,
}

impl LatLong {
    pub fn to_tile_coord(self) -> TileCoord {
        TileCoord {
            x: (self.longitude + 180.0) / 360.0 * 2.0 * THREE_POW_ZOOM,
            y: (90.0 - self.latitude) / 180.0 * THREE_POW_ZOOM,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TileCoord {
   pub x: f32,
   pub y: f32,
}

impl TileCoord {
    pub fn to_lat_long(self) -> LatLong {
        LatLong {
            latitude: 90.0 - ((180.0 * self.y) / THREE_POW_ZOOM),
            longitude: ((180.0 * self.x) / THREE_POW_ZOOM) - 180.0,
        }
    }

    pub fn to_world_space(self) -> Vec2 {
        Vec2 {
            x: (self.x - TOP_LEFT_TILE.x) * 1000.0 * XYZ_SCALE,
            y: (self.y - TOP_LEFT_TILE.y) * 1000.0 * XYZ_SCALE,
        }
    }

    pub fn to_world_space_0y(self) -> Vec3 {
        let v2 = self.to_world_space();
        Vec3::new(v2.x, 0.0, v2.y)
    }
}

