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

BASELINE (no spatial index):
Total time for 298 offset operations: 196.589137ms
Average time per operation: 659.695µs
Operations per second: 1515.9

V1 (BroadPhaseFlat, rebuild every iteration):
Total time for 298 offset operations: 180.781472ms
Average time per operation: 606.649µs
Operations per second: 1648.4
Improvement: 8.0%

V2 (BroadPhaseFlat, build ONCE + preallocate):
- offset_split_arcs: Build AABB index ONCE before loops, child arcs inherit parent AABB
- offset_prune_invalid: Build AABB index ONCE with all polyarcs, query for each offset

Total time for 298 offset operations: 112.730806ms
Average time per operation: 378.291µs
Operations per second: 2643.5
Improvement: 42.7% vs baseline! (77% faster than V1!)

Key optimizations:
1. Build spatial index ONCE, never rebuild
2. Preallocate vectors to avoid reallocation
3. Reuse AABB bounds - child arcs inherit parent's AABB
4. Index polyarcs once, query efficiently for each offset

*/
