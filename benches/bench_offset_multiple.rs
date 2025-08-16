// https://nikolaivazquez.com/blog/divan/
//

use geom::prelude::*;
use offroad::offset::{OffsetCfg, example_polyline_01, offset_polyline};

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench(args = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0], max_time = 1.0,  sample_size = 200 )]
fn bench_offset_polyline_to_polyline(off: f64) {
    let mut cfg = OffsetCfg::default();

    let mut p = example_polyline_01();
    p = polyline_translate(&p, point(180.0, -60.0));
    p = polyline_scale(&p, 2.5);
    offset_polyline(&p, off, &mut cfg);

    let p2 = polyline_reverse(&p);
    offset_polyline(&p2, off, &mut cfg);
}

/*
> cargo bench

Timer precision: 20 ns
bench_offset_multiple                 fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ bench_offset_polyline_to_polyline                │               │               │               │         │
   ├─ 1                               368.7 µs      │ 384.5 µs      │ 376.3 µs      │ 376 µs        │ 14      │ 2800
   ├─ 2                               373.6 µs      │ 413.5 µs      │ 380.6 µs      │ 384.6 µs      │ 14      │ 2800
   ├─ 4                               373.7 µs      │ 409.5 µs      │ 375.2 µs      │ 379.3 µs      │ 14      │ 2800
   ├─ 8                               372 µs        │ 440.5 µs      │ 379 µs        │ 383.8 µs      │ 14      │ 2800
   ├─ 16                              439.9 µs      │ 1.033 ms      │ 541.8 µs      │ 606.1 µs      │ 9       │ 1800
   ├─ 32                              628 µs        │ 908 µs        │ 706.4 µs      │ 730.5 µs      │ 7       │ 1400
   ├─ 64                              679.9 µs      │ 708 µs        │ 687.6 µs      │ 689.7 µs      │ 8       │ 1600
   ├─ 128                             614.9 µs      │ 635.9 µs      │ 616 µs        │ 620.7 µs      │ 9       │ 1800
   ╰─ 256                             719.6 µs      │ 742.2 µs      │ 739.8 µs      │ 736.8 µs      │ 7       │ 1400


*/
