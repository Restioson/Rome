use std::io::{self, Read, Error, Cursor};
use byteorder::{BigEndian, ReadBytesExt};
use xz2::bufread::XzDecoder;

const SIGNATURE: &[u8] = b"TERRARIUM/RASTER";

#[derive(Debug)]
pub enum RasterDecodeError {
    IoError(io::Error),
    InvalidSignature([u8; SIGNATURE.len()]),
    UnknownVersion(u8),
    UnknownRasterFormat(u8),
}

impl From<io::Error> for RasterDecodeError {
    fn from(err: Error) -> Self {
        RasterDecodeError::IoError(err)
    }
}

pub struct Raster {
    format: RasterFormat,
    pub width: u32,
    pub height: u32,
}

impl Raster {
    fn new(format: RasterFormat, width: u32, height: u32) -> Self {
        Raster { format, width, height }
    }

    pub fn get(&self, x: u32, y: u32) -> i32 {
        let index = (x + y * self.height) as usize;
        match &self.format {
            RasterFormat::UByte(vec) => vec[index] as i32,
            RasterFormat::Byte(vec) => vec[index] as i32,
            RasterFormat::Short(vec) => vec[index] as i32,
        }
    }

    fn zero(&mut self) {
        let size = (self.width * self.height) as usize;
        match &mut self.format {
            RasterFormat::UByte(vec) => *vec = vec![0; size],
            RasterFormat::Byte(vec) => *vec = vec![0; size],
            RasterFormat::Short(vec) => *vec = vec![0; size],
        }
    }

    fn set(&mut self, x: u32, y: u32, val: i32) {
        let index = (x + y * self.height) as usize;
        match &mut self.format {
            RasterFormat::UByte(vec) => vec[index] = val as u8,
            RasterFormat::Byte(vec) => vec[index] = val as i8,
            RasterFormat::Short(vec) => vec[index] = val as i16,
        }
    }

    fn copy(src: &mut Self, src_view: DataView, dst: &mut Self, dst_view: DataView) {
        let min_x = i32::max(0, dst_view.x as i32 - src_view.x as i32) as u32;
        let min_y = i32::max(0, dst_view.y as i32 - src_view.y as i32) as u32;
        let max_x = i32::min(src_view.width as i32, (dst_view.x + dst_view.width) as i32 - src_view.x as i32) as u32;
        let max_y = i32::min(src_view.height as i32, (dst_view.y + dst_view.height) as i32 - src_view.y as i32) as u32;

        for local_y in min_y..max_y {
            let result_y = local_y + src_view.y - dst_view.y;

            let local_x = min_x;
            let result_x = local_x + src_view.x - dst_view.x;

            let source_index = local_x + local_y * src.width;
            let result_index = result_x + result_y * dst.width;

            RasterFormat::copy(
                &mut src.format,
                source_index as usize,
                &mut dst.format,
                result_index as usize,
                (max_x - min_x) as usize,
            );
        }
    }

    fn read<R: Read>(&self, width: u32, height: u32, mut reader: R) -> io::Result<Raster> {
        let format = match self.format {
            RasterFormat::UByte(_) => {
                let mut vec = Vec::with_capacity((width * height) as usize);
                for _ in 0..width * height {
                    vec.push(reader.read_u8()?);
                }
                RasterFormat::UByte(vec)
            },
            RasterFormat::Byte(_) => {
                let mut vec = Vec::with_capacity((width * height) as usize);
                for _ in 0..width * height {
                    vec.push(reader.read_i8()?);
                }
                RasterFormat::Byte(vec)
            }
            RasterFormat::Short(_) => {
                let mut vec = Vec::with_capacity((width * height) as usize);
                for _ in 0..width * height {
                    vec.push(reader.read_i16::<BigEndian>()?);
                }
                RasterFormat::Short(vec)
            }
        };

        Ok(Raster::new(format, width, height))
    }
}

#[derive(Clone, Debug)]
enum RasterFormat {
    UByte(Vec<u8>),
    Byte(Vec<i8>),
    Short(Vec<i16>),
}

impl RasterFormat {
    fn copy(src: &mut Self, src_idx: usize, dst: &mut Self, dst_idx: usize, len: usize) {
        assert_eq!(std::mem::discriminant(&src), std::mem::discriminant(&dst));

        match src {
            RasterFormat::UByte(src_vec) => {
                let dst_vec = match dst {
                    RasterFormat::UByte(vec) => vec,
                    _ => unreachable!(),
                };

                for i in 0..len {
                    dst_vec[i + dst_idx] = src_vec[i + src_idx];
                }
            },
            RasterFormat::Byte(src_vec) => {
                let dst_vec = match dst {
                    RasterFormat::Byte(vec) => vec,
                    _ => unreachable!(),
                };

                for i in 0..len {
                    dst_vec[i + dst_idx] = src_vec[i + src_idx];
                }
            },
            RasterFormat::Short(src_vec) => {
                let dst_vec = match dst {
                    RasterFormat::Short(vec) => vec,
                    _ => unreachable!(),
                };

                for i in 0..len {
                    dst_vec[i + dst_idx] = src_vec[i + src_idx];
                }
            }
        }
    }
}

#[derive(Debug)]
enum RasterFilter {
    None,
    Left,
    Up,
    Average,
    Paeth,
}

impl RasterFilter {
    fn apply(&self, x: i32, a: i32, b: i32, c: i32) -> i32 {
        match self {
            RasterFilter::None => x,
            RasterFilter::Left => x + a,
            RasterFilter::Up => x + b,
            RasterFilter::Average => x + (a + b) / 2,
            RasterFilter::Paeth => {
                let p = a + b - c;
                let delta_a = (a - p).abs();
                let delta_b = (b - p).abs();
                let delta_c = (c - p).abs();

                if delta_a < delta_b && delta_a < delta_c {
                    x + a
                } else if delta_b < delta_c {
                    x + b
                } else {
                    x + c
                }
            }
        }
    }

    fn apply_to_raster(&self, input: &mut Raster, output: &mut Raster) {
        for y in 0..input.height {
            for x in 0..input.width {
                let value = input.get(x, y);
                let a = if x > 0 {
                    output.get(x - 1, y)
                } else {
                    0
                };

                let b = if y > 0 {
                    output.get(x, y - 1)
                } else {
                    0
                };

                let c = if x > 0 && y > 0 {
                    output.get(x - 1, y - 1)
                } else {
                    0
                };

                output.set(x, y, self.apply(value, a, b, c));
            }
        }
    }
}

struct DataView {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

pub fn read<R: Read>(mut reader: R) -> Result<Raster, RasterDecodeError> {
    let mut signature_bytes = [0; SIGNATURE.len()];
    reader.read_exact(&mut signature_bytes)?;

    if signature_bytes != SIGNATURE {
        return Err(RasterDecodeError::InvalidSignature(signature_bytes))
    }

    let version = reader.read_u8()?;
    if version != 0 {
        return Err(RasterDecodeError::UnknownVersion(version));
    }

    let (raster_width, raster_height) = (reader.read_u32::<BigEndian>()?, reader.read_u32::<BigEndian>()?);

    let raster_format = match reader.read_u8()? {
        0 => RasterFormat::UByte(vec![]),
        1 => RasterFormat::Byte(vec![]),
        2 => RasterFormat::Short(vec![]),
        other => return Err(RasterDecodeError::UnknownRasterFormat(other)),
    };

    let mut raster = Raster::new(raster_format.clone(), raster_width, raster_height);
    raster.zero();

    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    let len = data.len();
    let mut file_cursor = Cursor::new(&mut data);

    while file_cursor.position() < len as u64 {
        let chunk_length = file_cursor.read_u32::<BigEndian>()? as usize;
        let mut chunk_bytes = vec![0; chunk_length];
        file_cursor.read_exact(&mut chunk_bytes)?;
        let mut chunk_cursor = Cursor::new(&mut chunk_bytes);

        let x = chunk_cursor.read_u32::<BigEndian>()?;
        let y = chunk_cursor.read_u32::<BigEndian>()?;
        let width = chunk_cursor.read_u32::<BigEndian>()?;
        let height = chunk_cursor.read_u32::<BigEndian>()?;
        let filter_id = chunk_cursor.read_u8()?;

        let filter = match filter_id {
            1 => RasterFilter::Left,
            2 => RasterFilter::Up,
            3 => RasterFilter::Average,
            4 => RasterFilter::Paeth,
            _ => RasterFilter::None,
        };

        let src_view = DataView { x, y, width, height };
        let dst_view = DataView {
            x: 0,
            y: 0,
            width: raster_width,
            height: raster_height
        };

        let mut raw_raster = raster.read(width, height, XzDecoder::new(&mut chunk_cursor))?;
        let mut filtered_raster = Raster::new(raster_format.clone(), width, height);
        filtered_raster.zero();
        filter.apply_to_raster(&mut raw_raster, &mut filtered_raster);
        Raster::copy(&mut filtered_raster, src_view, &mut raster, dst_view)
    }

    Ok(raster)
}
