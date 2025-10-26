use togo::{poly::{arcline200}, prelude::*};
use offroad::prelude::*;

fn main() {
    let mut cfg = OffsetCfg::default();
    let mut svg = SVG::new(800.0, 800.0, Some("/tmp/arcline200.svg"));
    cfg.svg = Some(&mut svg);
    cfg.svg_orig = true;
    cfg.svg_raw = true;

    let poly = arcline200();
    let _offset_polylines = offset_arcline_to_arcline(&poly, 4.0, &mut cfg);

    if let Some(svg) = cfg.svg.as_mut(){
        // Write svg to file
        svg.write_stroke_width(0.1);
    }
}
