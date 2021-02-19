use rome_map::{Height, Map};

pub struct HeightmapMipMap {
    pub width: usize,
    pub height: usize,
    pub height_map: Vec<Height>,
}

impl HeightmapMipMap {
    pub fn get(&self, x: u32, y: u32) -> Height {
        self.height_map[x as usize + y as usize * self.width]
    }
}

pub fn generate_mipmaps(map: &Map, levels: usize) -> Vec<HeightmapMipMap> {
    assert!(levels > 0, "Mipmap levels must be greater than zero!");
    let mut mipmaps = Vec::with_capacity(levels);
    mipmaps.push(generate_mipmap((map.width, map.height), &map.height_map));

    let (mut current_heightmap, mut width, mut height) = {
        let m = &mipmaps[mipmaps.len() - 1];
        (&m.height_map, m.width, m.height)
    };

    for _ in 0..levels - 1 {
        let current_mipmap = generate_mipmap((width, height), current_heightmap);
        mipmaps.push(current_mipmap);
        let m = &mipmaps[mipmaps.len() - 1];
        current_heightmap = &m.height_map;
        width = m.width;
        height = m.height
    }

    mipmaps
}

fn generate_mipmap((orig_width, orig_height): (usize, usize), orig_map: &[Height]) -> HeightmapMipMap {
    let (target_width, target_height) = (orig_width / 2, orig_height / 2);
    let mut mipmap = Vec::with_capacity(target_width * target_height);

    for target_z in 0..target_height {
        for target_x in 0..target_width {
            let (orig_x, orig_z) = (target_x * 2, target_z * 2);
            let height_sum = sample((orig_x, orig_z), orig_width, orig_map) +
                sample((orig_x + 1, orig_z), orig_width, orig_map) +
                sample((orig_x, orig_z + 1), orig_width, orig_map) +
                sample((orig_x + 1, orig_z + 1), orig_width, orig_map);
            mipmap.push(Height((height_sum / 4) as i16));
        }
    }

    HeightmapMipMap {
        width: target_width,
        height: target_height,
        height_map: mipmap,
    }
}

fn sample((x, y): (usize, usize), width: usize, map: &[Height]) -> i64 {
    map[x + y * width].0 as i64
}
