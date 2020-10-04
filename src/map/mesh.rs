use bevy::prelude::*;
use bevy::render::mesh::{Mesh, VertexAttribute};
use bevy::render::pipeline::PrimitiveTopology;
use std::fmt::{self, Debug, Formatter};
use bevy::render::mesh;
use itertools::Itertools;
use rayon::prelude::*;

pub struct MapGenerator {
    resolution: u32,
    pub chunk_size: u32,
}

impl MapGenerator {
    pub fn new() -> Self {
        MapGenerator {
            chunk_size: 50,
            /// resolution <= 255
            resolution: 150,
        }
    }

    pub fn generate_meshes(&self, map: &rome_map::Map, meshes: &mut Assets<Mesh>) -> Vec<((u32, u32), Handle<Mesh>)> {
        let mut handles = Vec::new();

        let (img_width, img_height) = (map.width, map.height);
        let scale = 10;
        let x_tiles = img_width as u32 / self.chunk_size / scale;
        let z_tiles = img_height as u32 / self.chunk_size / scale;

        let generated: Vec<_> = (0..x_tiles)
            .cartesian_product(0..z_tiles)
            .par_bridge()
            .map(|(x, z)| {
                let top_left = (x * self.chunk_size, z * self.chunk_size);
                let generator = ChunkGenerator {
                    map: &map,
                    resolution: self.resolution,
                    side_length: self.chunk_size,
                    scale,
                    top_left_px: (top_left.0 * scale, top_left.1 * scale),
                };

                (top_left, generator.create_mesh())
            })
            .collect();

        for (coord, mesh) in generated {
            handles.push((coord, meshes.add(mesh)));
        }

        handles
    }
}

pub struct ChunkGenerator<'a> {
    map: &'a rome_map::Map,
    side_length: u32,
    resolution: u32,
    scale: u32,
    top_left_px: (u32, u32),
}

impl Debug for ChunkGenerator<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeshGenerator")
            .field("side_length", &self.side_length)
            .field("resolution", &self.resolution)
            .field("top_left_px", &self.top_left_px)
            .finish()
    }
}

pub struct Pixel {
    height: f32,
    is_water: bool,
}

impl ChunkGenerator<'_> {
    /// Sample an (x, z) on the raster directly, with no conversions or anti-aliasing
    fn sample_raw(&self, x: i32, z: i32) -> rome_map::Pixel {
        let max = (self.map.width as i32 - 1, self.map.height as i32 - 1);

        let img_x = i32::min(i32::max(0, x), max.0);
        let img_z = i32::min(i32::max(0, z), max.1);

        self.map.get(img_x as u32, img_z as u32)
    }

    pub fn sample(&self, x: i32, z: i32) -> Pixel {
        let to_img = |n, top_left| {
            let in_chunk =
                (n as f32 / self.resolution as f32 * self.side_length as f32 * self.scale as f32)
                    .floor();
            in_chunk as i32 + top_left as i32
        };

        let (centre_x, centre_z) = (to_img(x, self.top_left_px.0), to_img(z, self.top_left_px.1));

        let (mut total_height, mut total_is_water) = (0i32, 0u16);

        for kernel_x in -2..=2 {
            for kernel_z in -2..=2 {
                let (x, z) = (centre_x + kernel_x, centre_z + kernel_z);
                let raw_px = self.sample_raw(x, z);
                total_height += raw_px.height.0 as i32;
                if kernel_x.abs() < 2 && kernel_z.abs() < 2 {
                    total_is_water += raw_px.is_water as u16;
                }
            }
        }

        Pixel {
            height: f32::max(total_height as f32 / 25.0 / 750.0, 0.0),
            is_water: total_is_water / 9 >= 1,
        }
    }

    pub fn create_mesh(&self) -> Mesh {
        let res = self.resolution;
        let res_plus_1_sq = (res + 1) * (res + 1);
        let mut positions = Vec::with_capacity(res_plus_1_sq as usize);
        let mut normals = Vec::with_capacity(res_plus_1_sq as usize);
        let mut uvs = Vec::with_capacity(res_plus_1_sq as usize);

        for z in 0..res + 1 {
            for x in 0..res + 1 {
                let (x, z) = (x as i32, z as i32);
                let top_left = self.sample(x - 1, z - 1);
                let top_right = self.sample(x + 1, z - 1);
                let bottom_left = self.sample(x - 1, z + 1);
                let bottom_right = self.sample(x + 1, z + 1);
                let centre = self.sample(x, z);

                let x = x as f32 / self.resolution as f32 * self.side_length as f32;
                let z = z as f32 / self.resolution as f32 * self.side_length as f32;

                // Position
                positions.push([x, centre.height, z]);

                // UV - not actually UVs, just using it to store this data...
                if centre.is_water && top_left.is_water && top_right.is_water && bottom_right.is_water {
                    uvs.push([2.0, 0.0]) // Ocean
                } else if centre.is_water {
                    uvs.push([1.0, 0.0]) // Beach
                } else {
                    uvs.push([0.0, 0.0]) // Land
                }

                // Normal
                let normal_1 = Vec3::new(top_left.height - top_right.height , 2.0, bottom_left.height - top_left.height)
                    .normalize();
                let normal_2 = Vec3::new(bottom_left.height  - bottom_right.height, 2.0, top_right.height - bottom_right.height)
                    .normalize();
                let normal = (normal_1 + normal_2).normalize();
                normals.push([normal.x(), normal.y(), normal.z()]);
            }
        }

        let indices = if res_plus_1_sq >= 1 << 16 {
            mesh::Indices::U32(create_indices::<u32>(res))
        } else {
            mesh::Indices::U16(create_indices::<u16>(res))
        };

        Mesh {
            primitive_topology: PrimitiveTopology::TriangleList,
            attributes: vec![
                VertexAttribute::position(positions),
                VertexAttribute::normal(normals),
                VertexAttribute::uv(uvs),
            ],
            indices: Some(indices),
        }
    }
}

trait Index {
    fn from_u32(idx: u32) -> Self;
}

impl Index for u32 {
    fn from_u32(idx: u32) -> u32 {
        idx
    }
}

impl Index for u16 {
    fn from_u32(idx: u32) -> Self {
        idx as u16
    }
}

fn create_indices<I: Index>(res: u32) -> Vec<I> {
    let mut indices = Vec::with_capacity(res as usize * res as usize * 6);
    for z in 0..res {
        for x in 0..res as u32 {
            let top_left = x + z * (res + 1);
            let top_right = x + 1 + z * (res + 1);
            let bottom_left = x + (z + 1) * (res + 1);
            let bottom_right = x + 1 + (z + 1) * (res + 1);

            indices.push(I::from_u32(bottom_left));
            indices.push(I::from_u32(top_right));
            indices.push(I::from_u32(top_left));

            indices.push(I::from_u32(bottom_left));
            indices.push(I::from_u32(bottom_right));
            indices.push(I::from_u32(top_right));
        }
    }

    indices
}
