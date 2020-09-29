use shapefile::{Shape, Point};
use shapefile::record::PolygonRing;
use rayon::prelude::*;
use geo::{LineString, Coordinate, Polygon};

fn main() {
    let shapes = shapefile::read("data/water_polygons.shp").unwrap();

    let count = shapes
        .into_par_iter()
        .filter_map(|shape| {
            match shape {
                Shape::Polygon(poly) => Some(poly),
                _ => None,
            }
        })
        .map(|poly| {
            let mut rings = poly.into_inner();
            let outer_ring_idx = rings
                .iter()
                .position(|r| matches!(r, PolygonRing::Outer(_)))
                .unwrap();

            let outer_ring = shapefile_to_geo_ring(rings.remove(outer_ring_idx));
            let inner_rings = rings.into_iter().map(shapefile_to_geo_ring).collect();

            Polygon::new(outer_ring, inner_rings)
        })
        .count();

    println!("{}", count);
}

fn shapefile_to_geo_ring(ring: PolygonRing<Point>) -> LineString<f64> {
    ring
        .into_inner()
        .into_iter()
        .map(|point| Coordinate { x: point.x, y: point.y })
        .collect()
}
