use image::{DynamicImage, GenericImageView};
use bevy::render::mesh::{Mesh, VertexAttribute};
use bevy::render::pipeline::PrimitiveTopology;
use bevy::prelude::*;

static HEIGHTMAP_BYTES: &[u8] = include_bytes!("assets/europe_heightmap.png");

pub struct MapGenerator {
    heightmap: DynamicImage,
    resolution: u32,
}

impl MapGenerator {
    pub fn new() -> Self {
        let image = image::load_from_memory(HEIGHTMAP_BYTES).unwrap();
        MapGenerator {
            heightmap: image,
            resolution: 256,
        }
    }

    #[inline]
    pub fn sample(&self, x: i32, z: i32) -> f32 {
        let (img_width, img_height) = self.heightmap.dimensions();

        let img_x = i32::max(0, ((x as f32 / self.resolution as f32 * img_width as f32) as i32) - 1) as u32;
        let img_z = i32::max(0, ((z as f32 / self.resolution as f32 * img_height as f32) as i32) - 1) as u32;

        let mut red = self.heightmap.get_pixel(img_x, img_z)[0];
        if red > 0 {
            red += 2; // TODO for visibility of land w/o colour
        }

        red as f32 / 255.0 * 20.0
    }

    #[inline]
    pub fn sample_alps(&self, x: i32, z: i32) -> f32 {
        self.sample(x + 1893, z + 1251)
    }

    pub fn create_mesh(&self) -> Mesh {
        let res = self.resolution;
        let res_sq = res * res;
        let mut positions = Vec::with_capacity(res_sq as usize);
        let mut normals = Vec::with_capacity(res_sq as usize);
        let mut indices = Vec::with_capacity(((res - 1) * (res - 1) * 2 * 3) as usize);

        for z in 0..res as i32 {
            for x in 0..res as i32 {
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

        for z in 0..res - 1 {
            for x in 0..res - 1 {
                let top_left = x + z * res;
                let top_right = x + 1 + z * res;
                let bottom_left = x + (z + 1) * res;
                let bottom_right = x + 1 + (z + 1) * res;

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
