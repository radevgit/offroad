// https://nikolaivazquez.com/blog/divan/
// 

use togo::prelude::*;
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
    offset_polyline_to_polyline(&p, off, &mut cfg);

    let p2 = polyline_reverse(&p);
    offset_polyline_to_polyline(&p2, off, &mut cfg);
}

/*
> cargo bench

Timer precision: 20 ns
bench_offset_multiple                 fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ bench_offset_polyline_to_polyline                │               │               │               │         │
   ├─ 1                               328.7 µs      │ 389.7 µs      │ 330.2 µs      │ 339.9 µs      │ 15      │ 3000
   ├─ 2                               329 µs        │ 332.8 µs      │ 329.9 µs      │ 330.3 µs      │ 16      │ 3200
   ├─ 4                               328.7 µs      │ 332.5 µs      │ 330.2 µs      │ 330.2 µs      │ 16      │ 3200
   ├─ 8                               329.2 µs      │ 342.7 µs      │ 329.9 µs      │ 331.6 µs      │ 16      │ 3200
   ├─ 16                              333 µs        │ 335.7 µs      │ 333.6 µs      │ 333.9 µs      │ 15      │ 3000
   ├─ 32                              346 µs        │ 359.1 µs      │ 347.1 µs      │ 348.2 µs      │ 15      │ 3000
   ├─ 64                              428.8 µs      │ 430.9 µs      │ 429.5 µs      │ 429.7 µs      │ 12      │ 2400
   ├─ 128                             410.3 µs      │ 414.3 µs      │ 411.1 µs      │ 411.5 µs      │ 13      │ 2600
   ╰─ 256                             502.3 µs      │ 505 µs        │ 503.5 µs      │ 503.6 µs      │ 10      │ 2000


*/
