use std::time::Instant;
use togo::{poly::arcline200, prelude::*};
use offroad::prelude::*;

fn main() {
    println!("Multi-Offset Benchmark (~200 arcs in double spiral)");
    println!("======================================================");
    
    let mut cfg = OffsetCfg::default();
    cfg.svg_orig = false;
    let arc_orig = arcline200();
    
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
> cargo bench --bench bench_offset_multiple200

Base
BRUTE-FORCE (USE_BRUTE_FORCE = true):
Total time for 198 offset operations: 3.431741022s
Average time per operation: 17.332025ms
Operations per second: 57.7

SPATIAL INDEX (USE_BRUTE_FORCE = false):
Total time for 198 offset operations: 1.314992506s
Average time per operation: 6.641376ms
Operations per second: 150.6

SPEEDUP: 2.61x faster with spatial index
*/
