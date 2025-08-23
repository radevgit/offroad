use geom::prelude::*;
use offroad::prelude::{OffsetCfg, example_polyline_01, offset_polyline};

fn main() {
    let mut cfg = OffsetCfg::default();
    // Prints SVG output to stdout
    let mut svg = SVG::new(300.0, 300.0,  Some("/tmp/polyline.svg"));
    cfg.svg = Some(&mut svg);
    cfg.svg_orig = false;
    //cfg.svg_remove_bridges = true;
    cfg.svg_final = true;

    // Translate to fit in the SVG viewport
    let poly = polyline_translate(&example_polyline_01(), point(100.0, -50.0));

    // Offset the polyline
    // let offset_polylines = offset_polyline_to_polyline(&poly, 15.0, &mut cfg);
    // for offset_poly in offset_polylines {
    //     svg.polyline(&offset_poly, "grey");
    // }

    // Internal offsetting
    // let poly = polyline_reverse(&poly);
    // let _offset_polylines = offset_polyline_to_polyline(&poly, 15.5600615, &mut cfg);
    let _offset_polylines = offset_polyline(&poly, 16.0, &mut cfg);

    if let Some(svg) = cfg.svg.as_deref_mut() {
        // Write svg to file
        svg.write_stroke_width(0.2);
    }
}
