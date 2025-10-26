use std::time::Instant;
use togo::{poly::arcline500, prelude::*};
use offroad::prelude::*;

fn main() {
    println!("Multi-Offset Benchmark (~500 arcs in double spiral)");
    println!("======================================================");
    
    let mut cfg = OffsetCfg::default();
    cfg.svg_orig = false;
    let arc_orig = arcline500();
    
    let start = Instant::now();
    
    // Forward direction
    for i in 1..100 {
        offset_arcline_to_arcline(&arc_orig, (i as f64)/4.0, &mut cfg);
    }
    
    // Reverse direction
    let arcs_reversed = arcline_reverse(&arc_orig);
    for i in 1..100 {
        offset_arcline_to_arcline(&arcs_reversed, (i as f64)/4.0, &mut cfg);
    }
    
    let total_time = start.elapsed();
    let operations = 99 * 2; // 99 offsets in each direction
    let avg_per_operation = total_time / operations;
    
    println!("Total time for {} offset operations: {:?}", operations, total_time);
    println!("Average time per operation: {:?}", avg_per_operation);
    println!("Operations per second: {:.1}", 1.0 / avg_per_operation.as_secs_f64());
}

/*
> cargo bench --bench bench_offset_multiple500

Total time for 198 offset operations: 14.064167675s
Average time per operation: 71.031149ms
Operations per second: 14.1


*/
