use geom::prelude::*;
use offroad::prelude::{OffsetCfg, offset_polyline_multiple};

fn main() {
    let mut cfg = OffsetCfg::default();
    // This will create an SVG file at /tmp/out3.svg
    let mut svg = SVG::new(300.0, 300.0, "/tmp/out3.svg");
    cfg.svg = Some(&mut svg);
    cfg.debug_orig = true;
    cfg.debug_prune = true;
    let poly = vec![
        pvertex(point(0.0, 0.0), 0.0),
        pvertex(point(100.0, 100.0), 0.5),
        pvertex(point(200.0, 0.0), 1.3),
    ];
    // Translate to fit in the SVG viewport
    let poly = polyline_translate(&poly, point(40.0, 100.0));
    let _offset_polylines = offset_polyline_multiple(&poly, 3.0, 3.0, 30.0, &mut cfg);
    
    if let Some(svg) = cfg.svg.as_deref_mut() {
        // Write svg to file
        svg.write_stroke_width(0.1);
    }
}
