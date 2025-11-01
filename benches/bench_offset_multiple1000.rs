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
> 
cargo bench --bench bench_offset_multiple1000

BRUTE-FORCE (USE_BRUTE_FORCE = true):
Total time for 50 offset operations: 19.007039989s
Average time per operation: 380.140799ms
Operations per second: 2.6

SPATIAL INDEX (USE_BRUTE_FORCE = false) - polyarcs only:
Total time for 50 offset operations: 5.05559079s
Average time per operation: 101.111815ms
Operations per second: 9.9

AABB v0.3:
Total time for 50 offset operations: 4.975804519s
Average time per operation: 99.51609ms
Operations per second: 10.0

AABB v0.5:
Total time for 50 offset operations: 4.808297775s
Average time per operation: 96.165955ms
Operations per second: 10.4
_________________________________________________________
Opt 10 - check aabb before split
Total time for 50 offset operations: 3.936534119s
Average time per operation: 78.730682ms
Operations per second: 12.7
_________________________________________________________
Opt 11 - find_endpoint_groups with spatial index (40%)
Total time for 50 offset operations: 2.359176736s
Average time per operation: 47.183534ms
Operations per second: 21.2
_________________________________________________________


*/
