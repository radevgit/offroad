use togo::prelude::*;
use offroad::prelude::{OffsetCfg, offset_polyline_to_polyline, pline_01};

fn main() {
    let mut cfg = OffsetCfg::default();
    // This will create an SVG file at /tmp/pline1.svg
    let mut svg = SVG::new(300.0, 300.0, Some("/tmp/pline1.svg"));
    cfg.svg = Some(&mut svg);
    cfg.svg_orig = true;
    cfg.svg_final = true;

    let poly_orig = pline_01()[0].clone();
    // Translate to fit in the SVG viewport
    let poly = polyline_translate(&poly_orig, point(100.0, -50.0));

    let offset_polylines = offset_polyline_to_polyline(&poly, 10.0, &mut cfg);
    // Internal offsetting
    let poly = polyline_reverse(&poly);
    let offset_polylines2 = offset_polyline_to_polyline(&poly, 15.5600615, &mut cfg);
    //let _offset_polylines = offset_polyline_to_polyline(&poly, 16.0, &mut cfg);


    if let Some(svg) = cfg.svg.as_mut(){
        // Write svg to file
        svg.write_stroke_width(0.1);
    }

    assert_eq!(offset_polylines.len(), 1, "Expected exactly 1 offset polyline");
    assert_eq!(offset_polylines[0].len(), 27);
    assert_eq!(offset_polylines2.len(), 4, "Expected exactly 1 offset polyline");
    assert_eq!(offset_polylines2[0].len(), 8);
    assert_eq!(offset_polylines2[1].len(), 9);
    assert_eq!(offset_polylines2[2].len(), 7);
    assert_eq!(offset_polylines2[3].len(), 3);
}
