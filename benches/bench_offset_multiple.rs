use offroad::prelude::*;
use std::time::Instant;
use togo::prelude::*;

fn main() {
    println!("Multi-Offset Benchmark (based on offset_multi example)");
    println!("======================================================");

    let mut cfg = OffsetCfg::default();
    cfg.svg_orig = false;
    let poly_orig = pline_01()[0].clone();
    let poly = polyline_translate(&poly_orig, point(250.0, 100.0));

    let start = Instant::now();

    for _ in 0..100 {
        // Forward direction
        for i in 0..100 {
            offset_polyline_to_polyline(&poly, (i as f64) / 5.0, &mut cfg);
        }

        // Reverse direction
        let poly = polyline_reverse(&poly);
        for i in 0..100 {
            offset_polyline_to_polyline(&poly, (i as f64) / 5.0, &mut cfg);
        }
    }

    let total_time = start.elapsed();
    let operations = 100 * 100 * 2; // 100 external iterations × 100 offsets forward × 100 offsets reverse
    let avg_per_operation = total_time / operations as u32;

    println!(
        "Total time for {} offset operations: {:?}",
        operations, total_time
    );
    println!("Average time per operation: {:?}", avg_per_operation);
    println!(
        "Operations per second: {:.1}",
        1.0 / avg_per_operation.as_secs_f64()
    );
}

/*
>
cargo bench --bench bench_offset_multiple

Base:
Total time for 298 offset operations: 196.589137ms
Average time per operation: 659.695µs
Operations per second: 1515.9

Spatial index:
Total time for 298 offset operations: 190.66736ms
Average time per operation: 639.823µs
Operations per second: 1562.9

AABB v0.3:
Total time for 298 offset operations: 185.851519ms
Average time per operation: 623.662µs
Operations per second: 1603.4
_____________________________________________________
Opt 10 - using aabb check before split
Total time for 298 offset operations: 143.272175ms
Average time per operation: 480.779µs
Operations per second: 2080.0
_____________________________________________________
Opt 11 - however the benchmars was changed to 0..100 from 0..500 and external loop added
Total time for 20000 offset operations: 1.407260869s
Average time per operation: 70.363µs
Operations per second: 14212.0
_____________________________________________________

*/
