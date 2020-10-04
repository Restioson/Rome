use serde::{Serialize, Deserialize};
use bitvec::vec::BitVec;

/// Height of a point (metres)
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Height(pub i16);

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub height_map: Vec<Height>,
    pub is_water: BitVec,
}

impl Map {
    pub fn get(&self, x: u32, y: u32) -> Pixel {
        Pixel {
            height: self.height_map[x as usize + y as usize * self.width],
            is_water: self.is_water[x as usize + y as usize * self.width],
        }
    }
}

pub struct Pixel {
    pub height: Height,
    pub is_water: bool,
}