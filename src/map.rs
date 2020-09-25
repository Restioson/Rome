use bevy::prelude::*;
use bevy::render::mesh::{Mesh, VertexAttribute};
use bevy::render::pipeline::PrimitiveTopology;
use image::{DynamicImage, GenericImageView};
use std::fmt::{self, Debug, Formatter};

static HEIGHTMAP_BYTES: &[u8] = include_bytes!("assets/europe_heightmap.png");

pub struct MapGenerator {
    heightmap: DynamicImage,
    resolution: u32,
    chunk_size: u32,
}

impl MapGenerator {
    pub fn new() -> Self {
        let image = image::load_from_memory(HEIGHTMAP_BYTES).unwrap();
        MapGenerator {
            heightmap: image,
            /// Must be one of 1, 2, 4, 5, 8, 10, 16, 20, 32, 40, 64, 80, 128, 160, 320, 640
            chunk_size: 64,
            resolution: 320,
        }
    }

    pub fn generate_meshes(&self, meshes: &mut Assets<Mesh>) -> Vec<((u32, u32), Handle<Mesh>)> {
        let (img_width, img_height) = self.heightmap.dimensions();
        let scale = 16;
        let x_tiles = img_width / self.chunk_size / scale;
        let z_tiles = img_height / self.chunk_size / scale;

        println!("{}x{}", x_tiles, z_tiles);

        let mut handles = Vec::with_capacity((x_tiles * z_tiles) as usize);
        for x in 0..x_tiles {
            for z in 0..z_tiles {
                let top_left = (x, z);
                let generator = MeshGenerator {
                    heightmap: &self.heightmap,
                    resolution: self.resolution as i32,
                    side_length: self.chunk_size as i32,
                    scale,
                    top_left_px: ((x * self.chunk_size * scale) as i32, (z * self.chunk_size * scale) as i32),
                };

                let coords = (top_left.0 * self.chunk_size, top_left.1 * self.chunk_size);
                let mesh = generator.create_mesh();
                let handle = meshes.add(mesh);
                handles.push((coords, handle));
            }
        }

        handles
    }
}

pub struct MeshGenerator<'a> {
    heightmap: &'a DynamicImage,
    side_length: i32,
    resolution: i32,
    scale: u32,
    top_left_px: (i32, i32),
}

impl Debug for MeshGenerator<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeshGenerator")
            .field("side_length", &self.side_length)
            .field("resolution", &self.resolution)
            .field("top_left_px", &self.top_left_px)
            .finish()
    }
}

impl MeshGenerator<'_> {
    #[inline]
    pub fn sample(&self, x: i32, z: i32) -> f32 {
        let dim = self.heightmap.dimensions();
        let max = (dim.0 as i32 - 1, dim.1 as i32 - 1);
        let to_img = |n, top_left| {
            let in_chunk = (n as f32 / self.resolution as f32 * self.side_length as f32 * self.scale as f32).floor();
            in_chunk as i32 + top_left
        };
        let img_x = i32::min(i32::max(0, to_img(x, self.top_left_px.0)), max.0);
        let img_z = i32::min(i32::max(0, to_img(z, self.top_left_px.1)), max.1);

        let mut red = self.heightmap.get_pixel(img_x as u32, img_z as u32)[0];
        if red > 0 {
            red += 10; // TODO for visibility of land w/o colour
        }

        red as f32 / 255.0 * 15.0
    }

    pub fn create_mesh(&self) -> Mesh {
        let res = self.resolution;
        let res_plus_1_sq = (res + 1) * (res + 1);
        let mut positions = Vec::with_capacity(res_plus_1_sq as usize);
        let mut normals = Vec::with_capacity(res_plus_1_sq as usize);
        let mut indices = Vec::with_capacity((res * res * 2 * 3) as usize);

        for z in 0..res + 1 {
            for x in 0..res + 1 {
                let top_left = self.sample(x - 1, z - 1);
                let top_right = self.sample(x + 1, z - 1);
                let bottom_left = self.sample(x - 1, z + 1);
                let bottom_right = self.sample(x + 1, z + 1);
                let y = self.sample(x, z);

                let x = x as f32 / self.resolution as f32 * self.side_length as f32;
                let z = z as f32 / self.resolution as f32 * self.side_length as f32;

                // Position
                positions.push([x, y, z]);

                // Normal
                let normal_1 =
                    Vec3::new(top_left - top_right, 2.0, bottom_left - top_left).normalize();
                let normal_2 = Vec3::new(bottom_left - bottom_right, 2.0, top_right - bottom_right)
                    .normalize();
                let normal = (normal_1 + normal_2).normalize();
                normals.push([normal.x(), normal.y(), normal.z()]);
            }
        }

        for z in 0..res as u32 {
            for x in 0..res as u32 {
                let top_left = x + z * res as u32;
                let top_right = x + 1 + z * res as u32;
                let bottom_left = x + (z + 1) * res as u32;
                let bottom_right = x + 1 + (z + 1) * res as u32;

                indices.push(bottom_left);
                indices.push(top_right);
                indices.push(top_left);

                indices.push(bottom_left);
                indices.push(bottom_right);
                indices.push(top_right);
            }
        }

        Mesh {
            primitive_topology: PrimitiveTopology::TriangleList,
            attributes: vec![
                VertexAttribute::position(positions),
                VertexAttribute::normal(normals),
            ],
            indices: Some(indices),
        }
    }
}
