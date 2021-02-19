use serde::{Serialize, Deserialize};
use bitvec::vec::BitVec;
use std::ops::{Add, Div};

/// Height of a point (metres)
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Height(pub i16);

impl Add<Height> for Height {
    type Output = Height;

    fn add(self, rhs: Height) -> Height {
        Height(self.0 + rhs.0)
    }
}

impl Div<i16> for Height {
    type Output = Height;

    fn div(self, rhs: i16) -> Self::Output {
        Height(self.0 / rhs)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub height_map: Vec<Height>,
    pub is_water: BitVec,
}

impl Map {
    pub fn get(&self, x: u32, y: u32) -> Pixel {
        let x = usize::min(x as usize, self.width - 1);
        let y = usize::min(y as usize, self.height - 1);

        Pixel {
            height: self.height_map[x + y * self.width],
            is_water: self.is_water[x + y * self.width],
        }
    }
}

pub struct Pixel {
    pub height: Height,
    pub is_water: bool,
}