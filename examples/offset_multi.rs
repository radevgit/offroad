use togo::prelude::*;
use offroad::prelude::{OffsetCfg, offset_polyline_to_polyline, pline_01};

fn main() {
    let mut cfg = OffsetCfg::default();
    // This will create an SVG file at /tmp/pline1.svg
    let mut svg = SVG::new(600.0, 600.0, Some("/tmp/multi.svg"));
    cfg.svg = Some(&mut svg);
    cfg.svg_orig = true;
    cfg.svg_final = true;

    let poly_orig = pline_01()[0].clone();
    // Translate to fit in the SVG viewport
    let poly = polyline_translate(&poly_orig, point(250.0, 100.0));

    let mut offset_external = vec![];
    for i in 1..100 {
        let offset = offset_polyline_to_polyline(&poly, (i as f64)/2.0, &mut cfg);
        offset_external.extend(offset);
    }

    let poly = polyline_reverse(&poly);
    
    let mut offset_internal = vec![];
    for i in 1..100 {
        let offset = offset_polyline_to_polyline(&poly, (i as f64)/2.0, &mut cfg);
        offset_internal.extend(offset);
    }

    if let Some(svg) = cfg.svg.as_mut(){
        // Write svg to file
        svg.write_stroke_width(0.1);
    }

    assert!(
        offset_external.len() == 99,
        "Wrong number of external offset arclines generated: expected 99, got {}",
        offset_external.len()
    );
    assert!(
        offset_internal.len() == 181,
        "Wrong number of internal offset arclines generated: expected 181, got {}",
        offset_internal.len()
    );
}
