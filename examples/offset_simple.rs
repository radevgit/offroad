use geom::prelude::*;
use offroad::{OffsetCfg, offset_polyline};

fn main() {
    let mut cfg = OffsetCfg::default();
    let mut svg = SVG::new(300.0, 300.0, "/tmp/out2.svg");
    cfg.svg = Some(&mut svg);
    cfg.reconnect = false;
    cfg.debug_orig = true;
    cfg.debug_prune = true;

    let poly_orig = vec![
        pvertex(point(0.0, 0.0), 0.0),
        pvertex(point(100.0, 100.0), 0.5),
        pvertex(point(200.0, 0.0), 1.3),
    ];
    // Translate to fit in the SVG viewport
    let poly = polyline_translate(&poly_orig, point(40.0, 100.0));

    // Internal offsetting
    let poly = polyline_reverse(&poly);
    let _offset_polylines = offset_polyline(&poly, 15.0, &mut cfg);
    
    if let Some(svg) = cfg.svg.as_deref_mut() {
        // Write svg to file
        svg.write_stroke_width(0.1);
    }

}
