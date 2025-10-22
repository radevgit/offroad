use std::time::Instant;
use togo::prelude::*;
use offroad::prelude::*;

fn main() {
    println!("Profiling offset_split_arcs performance");
    println!("======================================\n");
    
    let mut cfg = OffsetCfg::default();
    cfg.svg_orig = false;
    
    // Use pline_01 - same as benchmark
    let poly_orig = pline_01()[0].clone();
    let poly = polyline_translate(&poly_orig, point(250.0, 100.0));
    
    // Do a few offset operations and measure
    println!("Offset operations on pline_01:");
    println!("Total parts created, Time per operation\n");
    
    for offset_amount in [1.0, 5.0, 10.0, 20.0, 30.0, 50.0, 100.0].iter() {
        let start = Instant::now();
        let result = offset_polyline_to_polyline(&poly, *offset_amount, &mut cfg);
        let elapsed = start.elapsed();
        
        let total_arcs: usize = result.iter().map(|line| line.len()).sum();
        println!("Offset {:5.1}: {} arcs created in {:.3}ms", 
                 offset_amount, total_arcs, elapsed.as_secs_f64() * 1000.0);
    }
    
    println!("\n\nLarge scale test:");
    let start = Instant::now();
    for i in 1..100 {
        offset_polyline_to_polyline(&poly, (i as f64)/5.0, &mut cfg);
    }
    let elapsed = start.elapsed();
    println!("99 offset operations: {:.3}ms total, {:.2}ms avg", 
             elapsed.as_secs_f64() * 1000.0,
             elapsed.as_secs_f64() * 1000.0 / 99.0);
}
