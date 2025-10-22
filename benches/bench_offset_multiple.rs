use std::time::Instant;
use togo::prelude::*;
use offroad::prelude::*;

fn main() {
    println!("Multi-Offset Benchmark (based on offset_multi example)");
    println!("======================================================");
    
    let mut cfg = OffsetCfg::default();
    cfg.svg_orig = false;
    let poly_orig = pline_01()[0].clone();
    let poly = polyline_translate(&poly_orig, point(250.0, 100.0));
    
    let start = Instant::now();
    
    // Forward direction
    for i in 1..500 {
        offset_polyline_to_polyline(&poly, (i as f64)/5.0, &mut cfg);
    }
    
    // Reverse direction
    let poly = polyline_reverse(&poly);
    for i in 1..500 {
        offset_polyline_to_polyline(&poly, (i as f64)/5.0, &mut cfg);
    }
    
    let total_time = start.elapsed();
    let operations = 149 * 2; // 149 offsets in each direction
    let avg_per_operation = total_time / operations;
    
    println!("Total time for {} offset operations: {:?}", operations, total_time);
    println!("Average time per operation: {:?}", avg_per_operation);
    println!("Operations per second: {:.1}", 1.0 / avg_per_operation.as_secs_f64());
}

/*
> cargo bench

BEFORE (without spatial index):
Total time for 298 offset operations: 196.589137ms
Average time per operation: 659.695µs
Operations per second: 1515.9

AFTER (with BroadPhaseFlat + aabb_from_arc_loose):
Total time for 298 offset operations: 180.781472ms
Average time per operation: 606.649µs
Operations per second: 1648.4

Improvement: 8.0% faster (196.6ms → 180.8ms)

*/
