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
//!
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
use byteorder::{NativeEndian, WriteBytesExt};
use std::cmp;
use ordered_float::OrderedFloat;

pub mod mesh;
pub mod shader;

pub struct RomeMapPlugin;

impl Plugin for RomeMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<HeightMap>()
            .add_asset::<shader::MapMaterial>()
            .add_startup_system(shader::setup.system())
            .on_state_update(
                STATE_STAGE,
                AppState::InGame,
                shader::update_time.system(),
            );
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
                    println!("Unzipping");

                    let mut decoder = zstd::Decoder::new(&*bytes).unwrap();
                    let mut unzipped = Vec::new();
                    decoder.read_to_end(&mut unzipped).unwrap();

                    println!("Deserializing");
                    let map: rome_map::Map = bincode::deserialize(&unzipped).unwrap();

                    println!("Done loading heightmap");

                    HeightMap(map)
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

#[derive(TypeUuid)]
#[uuid = "7b7c08b3-986e-49d8-85da-107024f177f1"]
pub struct HeightMap(pub rome_map::Map);

fn clamp(a: i32, max: u32) -> u32 {
    cmp::max(cmp::min(a, max as i32), 0) as u32
}

const F: f32 = 0.01;

impl HeightMap {
    fn sample_height(&self, x: i32, y: i32) -> u16 {
        let px = self.0.get(clamp(x, self.0.width as u32), clamp(y, self.0.height as u32));
        if px.is_water || px.height.0 < 0 {
            0
        } else {
            px.height.0 as u16
        }
    }

    fn sample_vec3(&self, x: i32, y: i32, height_factor: f32) -> Vec3 {
        Vec3::new(x as f32, self.sample_height(x, y) as f32 * height_factor * F, y as f32)
    }

    fn sample_normal(&self, x: i32, y: i32, height_factor: f32) -> Vec3 {
        let top_left = self.sample_vec3(x, y, height_factor);
        let bottom_left = self.sample_vec3(x, y + 1, height_factor);
        let bottom_right = self.sample_vec3(x + 1, y + 1, height_factor);

        (bottom_right - bottom_left).cross(top_left - bottom_left).normalize()
    }
}

impl Into<Texture> for &HeightMap {
    fn into(self) -> Texture {
        const HEIGHT_BITS: u8 = 11;
        const LIGHT_BITS: u8 = 16 - HEIGHT_BITS;
        const MAX_LIGHT_LEVEL: u8 = ((1u16 << LIGHT_BITS) - 1) as u8;
        const AMBIENT_LIGHT_STRENGTH: OrderedFloat<f32> = OrderedFloat(0.1);

        let light_pos = Vec3::new(-1.0, 0.5, -0.3).normalize();

        let mut max = 0;
        for (y, x) in (0..self.0.height).cartesian_product(0..self.0.width) {
            let h = self.0.get(x as u32, y as u32).height.0;
            if h > max {
                max = h;
            }
        }

        let factor = ((1 << HEIGHT_BITS) - 1) as f32 / max as f32;

        let mut bytes = Vec::with_capacity(1024 * 1024 * 2);

        for (y, x) in (0..1024).cartesian_product(0..1024) {
            let normal = self.sample_normal(x, y, factor);

            let diffuse = cmp::max(OrderedFloat(normal.dot(light_pos)), OrderedFloat(0.0));
            let brightness = cmp::min(OrderedFloat(1.0), diffuse + AMBIENT_LIGHT_STRENGTH);

            let brightness_level = (brightness.0 as f32 * MAX_LIGHT_LEVEL as f32).round() as u16;
            let height = (self.sample_height(x, y) as f32 * factor).round() as u16;

            let packed = brightness_level | (height << LIGHT_BITS);
            bytes.write_u16::<NativeEndian>(packed).unwrap();
        }

        Texture {
            data: bytes,
            size: Extent3d::new(1024, 1024, 1),
            format: TextureFormat::R16Uint,
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
