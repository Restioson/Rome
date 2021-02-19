use bevy::render::mesh::{Indices, Mesh};
use bevy::render::pipeline::PrimitiveTopology;
use std::cmp;
use std::collections::HashMap;

#[derive(Default)]
struct MeshBuilder {
    indexer: Indexer<u32>,
    triangle_indices: Vec<u32>,
}

impl MeshBuilder {
    fn push_triangle(&mut self, points: [[f32; 3]; 3]) {
        for point in &points {
            self.triangle_indices.push(self.indexer.index(*point));
        }
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, self.indexer.into_positions());
        mesh.set_indices(Some(Indices::U32(self.triangle_indices)));
        mesh
    }
}

// Adapted from https://github.com/morgan3d/misc/blob/master/terrain/Terrain.cpp
#[rustfmt::skip]
pub fn build_mesh(lod_levels: u8) -> Mesh {
    let mut builder = MeshBuilder::default();

    // How many grid cells to use most-detailed LOD for, divided by two
    let g = 512 / 2; // TODO ?
    let pad = 1; // TODO

    for lod_level in 0..lod_levels {
        let step: usize = 1 << (lod_level as usize);
        let half_step: isize = (step >> 1) as isize;
        // Radius that the LOD takes up
        let lod_radius: isize = step as isize * (g + pad);

        for z in (-lod_radius..lod_radius).step_by(step) {
            for x in (-lod_radius..lod_radius).step_by(step) {
                // Don't draw inside of other LOD's areas
                if cmp::max((x + half_step).abs(), (z + half_step).abs()) >= g * half_step {

                    let (x, z, l, s) = (x as f32, z as f32, lod_level as f32, step as f32);

                    // y is set to LOD level (`l`) for use in the shader
                    //        x       y   z
                    let a = [ x,      l,  z     ];
                    let c = [ x + s,  l,  z     ];
                    let g = [ x,      l,  z + s ];
                    let i = [ x + s,  l,  z + s ];

                    if lod_level > 0 {
                        // Tessellate the square as such:
                        //   A-----B-----C   ^     ^
                        //   | \   |   / |   |   half-step
                        //   |   \ | /   |         |
                        //   D-----E-----F  step   v
                        //   |   / | \   |
                        //   | /   |   \ |   |
                        //   G-----H-----I   v
                        //   <-   step  ->

                        let hs = s / 2.0; // half-step

                        // y is set to LOD level (`l`) for use in the shader
                        //        x        y   z
                        let b = [ x + hs,  l,  z      ];
                        let d = [ x,       l,  z + hs ];
                        let e = [ x + hs,  l,  z + hs ];
                        let f = [ x + s,   l,  z + hs ];
                        let h = [ x + hs,  l,  z + s  ];

                        let (x, z) = (x as isize, z as isize);

                        // Stitch the border into the next level
                        if x == -lod_radius {
                            //   A-----B-----C
                            //   | \   |   / |
                            //   |   \ | /   |
                            //   |     E-----F
                            //   |   / | \   |
                            //   | /   |   \ |
                            //   G-----H-----I
                            builder.push_triangle([e, a, g]);
                        } else {
                            builder.push_triangle([e, a, d]);
                            builder.push_triangle([e, d, g]);
                        }

                        if z == lod_radius - step as isize {
                            builder.push_triangle([e, g, i]);
                        } else {
                            builder.push_triangle([e, g, h]);
                            builder.push_triangle([e, h, i]);
                        }

                        if x == lod_radius - step as isize {
                            builder.push_triangle([e, i, c]);
                        } else {
                            builder.push_triangle([e, i, f]);
                            builder.push_triangle([e, f, c]);
                        }

                        if z == -lod_radius {
                            builder.push_triangle([e, c, a]);
                        } else {
                            builder.push_triangle([e, c, b]);
                            builder.push_triangle([e, b, a]);
                        }
                    } else {
                        builder.push_triangle([i, a, g]);
                        builder.push_triangle([i, c, a]);

                        // TODO LOD stitch
                    }
                }
            }
        }
    }

    builder.build()
}

type HashF32 = ordered_float::OrderedFloat<f32>;

fn hash_f32(f: f32) -> HashF32 {
    ordered_float::OrderedFloat(f)
}

trait Index: Copy {
    fn zero() -> Self;
    fn increment(&mut self);
    fn as_usize(&self) -> usize;
}

impl Index for u16 {
    fn zero() -> Self {
        0
    }

    fn increment(&mut self) {
        *self += 1;
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl Index for u32 {
    fn zero() -> Self {
        0
    }

    fn increment(&mut self) {
        *self += 1;
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }
}

struct Indexer<I: Index> {
    hash: HashMap<[HashF32; 3], I>,
    n: I,
}

impl<I: Index> Default for Indexer<I> {
    fn default() -> Self {
        Indexer {
            hash: HashMap::new(),
            n: I::zero(),
        }
    }
}

impl<I: Index> Indexer<I> {
    fn index(&mut self, i: [f32; 3]) -> I {
        let n = &mut self.n;
        *self
            .hash
            .entry([hash_f32(i[0]), hash_f32(i[1]), hash_f32(i[2])])
            .or_insert_with(|| {
                let old = *n;
                n.increment();
                old
            })
    }

    fn into_positions(self) -> Vec<[f32; 3]> {
        let mut vec = vec![[0.0, 0.0, 0.0]; self.n.as_usize()];
        for (pos, idx) in self.hash.into_iter() {
            vec[idx.as_usize()] = [pos[0].0, pos[1].0, pos[2].0];
        }

        vec
    }
}
