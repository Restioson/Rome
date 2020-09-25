use image::{DynamicImage, GenericImageView};
use bevy::render::mesh::{Mesh, VertexAttribute};
use bevy::render::pipeline::PrimitiveTopology;
use bevy::prelude::*;

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
            chunk_size: 32,
            resolution: 4,
        }
    }

    pub fn generate_meshes(
        &self,
        mut meshes: ResMut<Assets<Mesh>>
    ) -> Vec<((u32, u32), Handle<Mesh>)> {
        let (img_width, img_height) = self.heightmap.dimensions();
        let x_tiles = (img_width as f32 / self.chunk_size as f32 / self.resolution as f32).floor() as u32;
        let z_tiles = (img_height as f32 / self.chunk_size as f32 / self.resolution as f32).floor() as u32;

        let mut handles = Vec::with_capacity((x_tiles * z_tiles) as usize);
        for x in 0..x_tiles {
            for z in 0..z_tiles {
                let top_left = (x * self.chunk_size, z * self.chunk_size);
                let res = self.resolution;
                let chunk = self.chunk_size;
                let top_left_px = ((x * res * chunk) as i32, (z * res * chunk) as i32);
                let generator = MeshGenerator {
                    heightmap: &self.heightmap,
                    resolution: self.resolution,
                    chunk_size: self.chunk_size,
                    top_left_px,
                };

                let mesh = generator.create_mesh();
                let handle = meshes.add(mesh);
                handles.push((top_left, handle));
            }
        }

        handles
    }
}

pub struct MeshGenerator<'a> {
    heightmap: &'a DynamicImage,
    chunk_size: u32,
    resolution: u32,
    top_left_px: (i32, i32),
}

impl MeshGenerator<'_> {
    #[inline]
    pub fn sample(&self, x: i32, z: i32) -> f32 {
        let img_x = i32::max(0, x * self.resolution as i32 + self.top_left_px.0 - 1);
        let img_z = i32::max(0, z * self.resolution as i32 + self.top_left_px.1 - 1);

        let mut red = self.heightmap.get_pixel(img_x as u32, img_z as u32)[0];
        if red > 0 {
            red += 10; // TODO for visibility of land w/o colour
        }

        red as f32 / 255.0 * 15.0
    }

    pub fn create_mesh(&self) -> Mesh {
        let chunk = self.chunk_size;
        let chunk_sq = chunk * chunk;
        let mut positions = Vec::with_capacity(chunk_sq as usize);
        let mut normals = Vec::with_capacity(chunk_sq as usize);
        let mut indices = Vec::with_capacity(((chunk - 1) * (chunk - 1) * 2 * 3) as usize);

        for z in 0..chunk as i32 {
            for x in 0..chunk as i32 {
                let top_left = self.sample(x - 1, z - 1);
                let top_right = self.sample(x + 1, z - 1);
                let bottom_left = self.sample(x - 1, z + 1);
                let bottom_right = self.sample(x + 1, z + 1);
                let y = self.sample(x, z);

                let x = x as f32;
                let z = z as f32;

                // Position
                positions.push([x, y, z]);

                // Normal
                let normal_1 = Vec3::new(
                    top_left - top_right,
                    2.0,
                    bottom_left - top_left,
                ).normalize();
                let normal_2 = Vec3::new(
                    bottom_left - bottom_right,
                    2.0,
                    top_right - bottom_right,
                ).normalize();
                let normal = (normal_1 + normal_2).normalize();
                normals.push([normal.x(), normal.y(), normal.z()]);
            }
        }

        for z in 0..chunk - 1 {
            for x in 0..chunk - 1 {
                let top_left = x + z * chunk;
                let top_right = x + 1 + z * chunk;
                let bottom_left = x + (z + 1) * chunk;
                let bottom_right = x + 1 + (z + 1) * chunk;

                indices.push(top_left);
                indices.push(top_right);
                indices.push(bottom_left);

                indices.push(top_right);
                indices.push(bottom_right);
                indices.push(bottom_left);

                // Back faces
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
