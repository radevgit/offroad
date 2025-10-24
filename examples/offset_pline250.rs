use togo::prelude::*;
use offroad::prelude::{OffsetCfg, offset_polyline_to_polyline, pline_250};

fn main() {
    let mut cfg = OffsetCfg::default();
    // This will create an SVG file at /tmp/pline250.svg
    let mut svg = SVG::new(600.0, 600.0, Some("/tmp/pline250.svg"));
    cfg.svg = Some(&mut svg);
    cfg.svg_orig = true;
    cfg.svg_raw = true;

    let poly_orig = pline_250();
    // Translate to fit in the SVG viewport
    let poly = polyline_translate(&poly_orig, point(-250.0, -250.0));

    // Offset the polyline
    // let offset_polylines = offset_polyline_to_polyline(&poly, 15.0, &mut cfg);
    // for offset_poly in offset_polylines {
    //     svg.polyline(&offset_poly, "grey");
    // }

    // Internal offsetting
    //let poly = polyline_reverse(&poly);
    let _offset_polylines = offset_polyline_to_polyline(&poly, 16.0, &mut cfg);

    if let Some(svg) = cfg.svg.as_mut(){
        // Write svg to file
        svg.write_stroke_width(0.1);
    }
}
