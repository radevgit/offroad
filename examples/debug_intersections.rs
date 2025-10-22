use togo::prelude::*;
use offroad::prelude::{OffsetCfg, offset_polyline_to_polyline, pline_01};

fn main() {
    let mut cfg = OffsetCfg::default();

    let poly_orig = pline_01()[0].clone();
    let poly = polyline_translate(&poly_orig, point(100.0, -50.0));

    println!("=== Offset Intersection Analysis ===");
    println!("Input polyline vertices: {}", poly.len());

    // Internal offsetting
    let poly = polyline_reverse(&poly);
    
    // Instrument offset_split_arcs to track intersections
    let offset_polylines = offset_polyline_to_polyline(&poly, 15.5600615, &mut cfg);
    
    println!("Output polylines: {}", offset_polylines.len());
    let total_vertices: usize = offset_polylines.iter().map(|p| p.len()).sum();
    println!("Total output vertices: {}", total_vertices);
}
