use bevy::prelude::*;
use bevy::render::mesh::{Mesh, VertexAttribute};
use bevy::render::pipeline::PrimitiveTopology;
use std::fmt::{self, Debug, Formatter};
use crate::map::terrarium_raster::Raster;
use crate::HeightMap;
use bevy::render::mesh;


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

    pub fn generate_meshes(&self, heightmap: &HeightMap, meshes: &mut Assets<Mesh>) -> Vec<((u32, u32), Handle<Mesh>)> {
        let mut handles = Vec::new();

        let (img_width, img_height) = (heightmap.raster.width, heightmap.raster.height);
        let scale = 10;
        let x_tiles = img_width / self.chunk_size / scale;
        let z_tiles = img_height / self.chunk_size / scale;
        let grid_pos = heightmap.grid_pos;
        let offset = (grid_pos.0 * self.chunk_size * x_tiles, grid_pos.1 * self.chunk_size * z_tiles);

        for z in 0..z_tiles {
            for x in 0..x_tiles {
                let top_left = (self.chunk_size * x + offset.0, self.chunk_size * z + offset.1);
                let generator = ChunkGenerator {
                    heightmap: &heightmap.raster,
                    resolution: self.resolution,
                    side_length: self.chunk_size,
                    scale,
                    top_left_px: (
                        x * self.chunk_size * scale,
                        z * self.chunk_size * scale,
                    ),
                };

                let mesh = generator.create_mesh();
                let handle = meshes.add(mesh);
                handles.push((top_left, handle));
            }
        }

        handles
    }
}

pub struct ChunkGenerator<'a> {
    heightmap: &'a Raster,
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

impl ChunkGenerator<'_> {
    #[inline]
    pub fn sample(&self, x: i32, z: i32) -> f32 {
        let max = (self.heightmap.width as i32 - 1, self.heightmap.height as i32 - 1);
        let to_img = |n, top_left| {
            let in_chunk =
                (n as f32 / self.resolution as f32 * self.side_length as f32 * self.scale as f32)
                    .floor();
            in_chunk as i32 + top_left as i32
        };
        let img_x = i32::min(i32::max(0, to_img(x, self.top_left_px.0)), max.0);
        let img_z = i32::min(i32::max(0, to_img(z, self.top_left_px.1)), max.1);

        let mut height = self.heightmap.get(img_x as u32, img_z as u32);
        if height > 0 {
            height += 200; // For visibility of low-lying land
        }

        height as f32 / 750.0
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
                let y = self.sample(x, z);

                let x = x as f32 / self.resolution as f32 * self.side_length as f32;
                let z = z as f32 / self.resolution as f32 * self.side_length as f32;

                // Position
                positions.push([x, f32::max(y, 0.0), z]);

                // UV
                if y <= 0.0 && top_left <= 0.0 && top_right <= 0.0 && bottom_right <= 0.0{
                    uvs.push([2.0, 0.0]) // Ocean
                } else if y <= 0.0 {
                    uvs.push([1.0, 0.0]) // Beach
                } else {
                    uvs.push([0.0, 0.0]) // Land
                }

                // Normal
                let normal_1 =
                    Vec3::new(top_left - top_right, 2.0, bottom_left - top_left).normalize();
                let normal_2 = Vec3::new(bottom_left - bottom_right, 2.0, top_right - bottom_right)
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
