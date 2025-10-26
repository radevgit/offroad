use togo::{poly::arcline200, prelude::*};
use offroad::{offset::offset_arcline_to_arcline, prelude::{offset_polyline_to_polyline, pline_01, OffsetCfg}};

fn main() {
    let mut cfg = OffsetCfg::default();
    // This will create an SVG file at /tmp/pline1.svg
    let mut svg = SVG::new(800.0, 800.0, Some("/tmp/multi200.svg"));
    cfg.svg = Some(&mut svg);
    cfg.svg_orig = true;
    cfg.svg_raw = true;

    let poly_orig = arcline200();
    // Translate to fit in the SVG viewport
    //let poly = polyline_translate(&poly_orig, point(250.0, 100.0));

    let mut offset_external = vec![];
    for i in 1..5 {
        let offset = offset_arcline_to_arcline(&poly_orig, (i as f64), &mut cfg);
        offset_external.extend(offset);
    }

    if let Some(svg) = cfg.svg.as_mut(){
        // Write svg to file
        svg.write_stroke_width(0.1);
    }

    // assert!(
    //     offset_external.len() == 228,
    //     "Wrong number of offset arclines generated. Expected 228, got {}", offset_external.len()
    // );
}
