use std::time::Instant;
use togo::{poly::arcline1000, prelude::*};
use offroad::prelude::*;

fn main() {
    println!("Multi-Offset Benchmark (~1000 arcs in double spiral)");
    println!("======================================================");
    
    let mut cfg = OffsetCfg::default();
    cfg.svg_orig = false;
    let arc_orig = arcline1000();
    
    let start = Instant::now();
    
    // Forward direction
    for i in 1..26 {
        offset_arcline_to_arcline(&arc_orig, (i as f64)/4.0, &mut cfg);
    }
    
    // Reverse direction
    let arcs_reversed = arcline_reverse(&arc_orig);
    for i in 1..26 {
        offset_arcline_to_arcline(&arcs_reversed, (i as f64)/4.0, &mut cfg);
    }
    
    let total_time = start.elapsed();
    let operations = 25 * 2; // 25 offsets in each direction
    let avg_per_operation = total_time / operations;
    
    println!("Total time for {} offset operations: {:?}", operations, total_time);
    println!("Average time per operation: {:?}", avg_per_operation);
    println!("Operations per second: {:.1}", 1.0 / avg_per_operation.as_secs_f64());
}

/*
> cargo bench --bench bench_offset_multiple1000

Total time for 50 offset operations: 17.233370346s
Average time per operation: 344.667406ms
Operations per second: 2.9

*/
