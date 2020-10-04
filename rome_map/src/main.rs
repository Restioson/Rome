use std::time::Instant;
use crate::terrarium_raster::Raster;
use rome_map::{Map, Height};
use geo::prelude::*;
use geo::{Point, Rect, Coordinate};
use bitvec::bitvec;
use rayon::prelude::*;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use bitvec::vec::BitVec;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufReader};
use xz2::bufread::XzDecoder;
use xz2::stream::MtStreamBuilder;
use serde::Serialize;
use std::path::Path;

mod terrarium_raster;
mod osm_water_polygons;

fn main() {
    let now = Instant::now();
    println!("Reading heightmaps");
    let heightmaps = terrarium_raster::read_all();
    let map = combine(heightmaps);
    println!("Compressing and saving map data");
    write_ser(&map, "output/map.mapdat", compress_zstd);
    println!("Done in {:.2}s. Saved to output/map.mapdat", now.elapsed().as_secs_f32());
}

fn compress_zstd(bytes: &[u8]) -> Vec<u8> {
    let mut compressed_bytes = Vec::new();
    let mut zstd_writer = zstd::Encoder::new(&mut compressed_bytes, 6).unwrap();
    println!(" -> Compressing");
    zstd_writer.write_all(bytes).unwrap();
    zstd_writer.finish().unwrap();
    compressed_bytes
}

fn compress_xz(bytes: &[u8]) -> Vec<u8> {
    let mut compressed_bytes = Vec::new();

    let stream = MtStreamBuilder::new()
        .threads(num_cpus::get() as u32)
        .preset(6)
        .encoder()
        .unwrap();

    let mut xz_writer = xz2::write::XzEncoder::new_stream(&mut compressed_bytes, stream);
    println!(" -> Compressing");
    xz_writer.write_all(&bytes).unwrap();
    xz_writer.finish().unwrap();

    compressed_bytes
}

fn write_ser<T: Serialize>(dat: &T, path: impl AsRef<Path>, compress: fn(&[u8]) -> Vec<u8>) {
    let bytes = bincode::serialize(&dat).unwrap();
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .unwrap();
    let mut buf_writer = std::io::BufWriter::new(file);
    let compressed_bytes = compress(&bytes);

    println!(" -> Saving");
    buf_writer.write_all(&compressed_bytes).unwrap();
    buf_writer.flush().unwrap();
}

fn to_lat_long(x: u32, y: u32) -> Coordinate<f64> {
    // 23,3 is top left
    Coordinate {
        x: (x + 23 * 1000) as f64 * 360.0 / 54000.0 - 180.0,
        y: 90.0 - (y + 3 * 1000) as f64 / 27000.0 * 180.0,
    }
}

fn combine(mut heightmaps: Vec<((u32, u32), Raster)>) -> Map {
    let max_x = *heightmaps.iter().map(|((x, _y), _raster)| x).max().unwrap();
    let max_y = *heightmaps.iter().map(|((_x, y), _raster)| y).max().unwrap();

    let (width, height) = (heightmaps[0].1.width, heightmaps[0].1.height);

    // Just to make the output prettier :)
    heightmaps.sort_by_key(|((x, y), _)| -((x + y * height) as i32));

    let cap = ((max_x + 1) as usize * width as usize) * (max_y + 1) as usize * height as usize;
    let mut height_map = vec![Height(0); cap];
    let mut is_water = bitvec![0; cap];

    let water_polygons = rasterize_polygons((width, height), (max_x, max_y));

    println!("Stitching rasters");
    let pb = ProgressBar::new(((max_x + 1) * (max_y + 1)) as u64);
    pb.set_style(ProgressStyle::default_bar().progress_chars("#>-"));

    for ((tile_x, tile_y), raster) in heightmaps {
        assert_eq!((raster.width, raster.height), (width, height));

        let water_map = water_polygons.get(&(tile_x, tile_y)).unwrap();

        for y in 0..height {
            for x in 0..width {
                let global_x = (tile_x * width) + x;
                let global_y = (tile_y * height) + y;
                let idx = (global_x + (global_y * (max_x + 1) * width)) as usize;
                height_map[idx] = Height(raster.get(x, y) as i16);

                let water = *water_map.get((x + y * width) as usize).unwrap();
                *is_water.get_mut(idx).unwrap() = water;
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    
    Map {
        width: ((max_x + 1) * width) as usize,
        height: ((max_y + 1) * height) as usize,
        height_map,
        is_water,
    }
}

fn rasterize_polygons(
    tile_dim: (u32, u32),
    maxes: (u32, u32),
) -> HashMap<(u32, u32), BitVec> {
    let res = File::open("output/water_polygons_rasterised.dat.xz");

    if let Ok(file) = res {
        let reader = XzDecoder::new(BufReader::new(file));
        match bincode::deserialize_from(reader) {
            Ok(polygons) => {
                println!("Found cached rasterized polygons - using those.");
                return polygons
            },
            Err(_) => {
                println!("Found cached rasterized polygons, but they were invalid - re-rasterizing.");
            }
        }
    }

    println!("Reading polygons");
    let water_polygons: Vec<_> = osm_water_polygons::read_all()
        .into_par_iter()
        .map(|poly| (poly.bounding_rect().unwrap(), poly))
        .collect();

    let (width, height) = tile_dim;
    let (max_x, max_y) = maxes;

    let tile_coords: Vec<_> = (0..=max_x).cartesian_product(0..=max_y).collect();

    let pb = ProgressBar::new(((max_x + 1) * (max_y + 1)) as u64);
    pb.set_style(
        ProgressStyle::default_bar().progress_chars("#>-")
    );

    println!("Rasterizing polygons");
    let polygons = tile_coords
        .into_par_iter()
        .progress_with(pb)
        .map(|(tile_x, tile_y)| {
            let mut is_water = bitvec![0; (width * height) as usize];

            let tile_min = to_lat_long(tile_x * width, tile_y * height + height);
            let tile_max = to_lat_long(tile_x * width + width, tile_y * height);

            let tile_rect = Rect::new(tile_min, tile_max);

            let local_polygons: Vec<_> = water_polygons
                .iter()
                .filter(|(bounding, _poly)| {
                    bounding.intersects(&tile_rect) ||
                        bounding.contains(&tile_rect) ||
                        tile_rect.contains(bounding)
                })
                .collect();

            for x in 0..width {
                for y in 0..height {
                    let global_x = (tile_x * width) + x;
                    let global_y = (tile_y * height) + y;

                    let lat_long = to_lat_long(global_x, global_y);
                    let lat_long = Point(lat_long);

                    let idx = (x + (y * width)) as usize;

                    *is_water.get_mut(idx).unwrap() = local_polygons
                        .iter()
                        .filter(|(bounding_rect, _poly)| bounding_rect.contains(&lat_long))
                        .any(|(_bounding_rect, poly)| poly.contains(&lat_long));
                }
            }

            ((tile_x, tile_y), is_water)
        })
        .collect();

    println!("Caching rasterized polygons");
    write_ser(&polygons, "output/water_polygons_rasterised.dat.xz", compress_xz);
    polygons
}