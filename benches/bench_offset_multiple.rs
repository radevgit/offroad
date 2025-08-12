// https://nikolaivazquez.com/blog/divan/
// 

use geom::prelude::*;
use offroad::{offset::{offset_polyline_to_polyline, pline_01, OffsetCfg}};



fn main() {
    // Run registered benchmarks.
    divan::main();
}


#[divan::bench(args = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0], 
    max_time = 1.0, // seconds
    sample_size = 200, // 64 × 84 = 5376
    )]
fn bench_offset_polyline_to_polyline(off: f64) {
    let mut cfg = OffsetCfg::default();

    let mut p = pline_01()[0].clone();
    p = polyline_translate(&p, point(180.0, -60.0));
    p = polyline_scale(&p, 2.5);
    offset_polyline_to_polyline(&p, 16.0, &mut cfg);

    let p2 = polyline_reverse(&p);
    offset_polyline_to_polyline(&p2, 16.0, &mut cfg);
}

/*
> cargo bench

Timer precision: 20 ns
bench_offset_multiple                 fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ bench_offset_polyline_to_polyline                │               │               │               │         │
   ├─ 1                               373.1 µs      │ 446.2 µs      │ 375.3 µs      │ 388.5 µs      │ 13      │ 2600
   ├─ 2                               480 µs        │ 565.5 µs      │ 534.3 µs      │ 529 µs        │ 10      │ 2000
   ├─ 4                               545.9 µs      │ 565 µs        │ 562.8 µs      │ 559.9 µs      │ 9       │ 1800
   ├─ 8                               566.8 µs      │ 628.3 µs      │ 581.6 µs      │ 584.4 µs      │ 9       │ 1800
   ├─ 16                              674.7 µs      │ 784.2 µs      │ 715.3 µs      │ 729.3 µs      │ 7       │ 1400
   ├─ 32                              642.5 µs      │ 917.4 µs      │ 755.8 µs      │ 775.4 µs      │ 7       │ 1400
   ├─ 64                              568.9 µs      │ 623.1 µs      │ 589.6 µs      │ 595 µs        │ 9       │ 1800
   ├─ 128                             566.6 µs      │ 585.9 µs      │ 568.6 µs      │ 571.5 µs      │ 9       │ 1800
   ╰─ 256                             565 µs        │ 585.2 µs      │ 566.5 µs      │ 570.5 µs      │ 9       │ 1800

*/
