// https://nikolaivazquez.com/blog/divan/
// 

use geom::prelude::*;
use offroad::{offset::pline_01, offset_connect_raw, offset_polyline_raw, offset_prune_invalid, offset_raw::OffsetRaw, offset_split_arcs, poly_to_raws};



fn main() {
    // Run registered benchmarks.
    divan::main();
}

fn offset_multiple_bench(poly_raws: &Vec<Vec<OffsetRaw>>, off: f64) {
    let offset_raw = offset_polyline_raw(&poly_raws, off);

    let offset_connect = offset_connect_raw(&offset_raw, off);

    let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);

    let _offset_final = offset_prune_invalid(&poly_raws, &mut offset_split, off);
}

#[divan::bench(args = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0], 
    max_time = 1.0, // seconds
    sample_size = 200, // 64 × 84 = 5376
    )]
fn bench_offset_multiple_complex_polyline(off: f64) {
    let mut plines: Vec<Vec<crate::PVertex>> = Vec::new();

    let mut p = pline_01()[0].clone();
    p = polyline_translate(&p, point(180.0, -60.0));
    p = polyline_scale(&p, 2.5);
    plines.push(p.clone());
    let poly_raws = poly_to_raws(&plines);
    offset_multiple_bench(&poly_raws, off);

    let p2 = polyline_reverse(&p);
    let poly_raws = poly_to_raws(&vec![p2]);
    offset_multiple_bench(&poly_raws, off);
}

/*
> cargo bench

Timer precision: 20 ns
offset_multiple                            fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ bench_offset_multiple_complex_polyline                │               │               │               │         │
   ├─ 1                                    222.4 µs      │ 233.6 µs      │ 223.6 µs      │ 224.5 µs      │ 23      │ 4600
   ├─ 2                                    222.2 µs      │ 229.8 µs      │ 222.9 µs      │ 223.3 µs      │ 23      │ 4600
   ├─ 4                                    222 µs        │ 233.6 µs      │ 222.6 µs      │ 223.4 µs      │ 23      │ 4600
   ├─ 8                                    222.9 µs      │ 315.2 µs      │ 225.7 µs      │ 247.6 µs      │ 21      │ 4200
   ├─ 16                                   352.1 µs      │ 381.9 µs      │ 368.7 µs      │ 369.8 µs      │ 14      │ 2800
   ├─ 32                                   393.8 µs      │ 410.4 µs      │ 403.3 µs      │ 402.3 µs      │ 13      │ 2600
   ├─ 64                                   566.3 µs      │ 634.4 µs      │ 586 µs        │ 591.9 µs      │ 9       │ 1800
   ├─ 128                                  542.7 µs      │ 582.6 µs      │ 571 µs        │ 568 µs        │ 9       │ 1800
   ╰─ 256                                  650.9 µs      │ 696.5 µs      │ 673.3 µs      │ 670.1 µs      │ 8       │ 1600

*/
