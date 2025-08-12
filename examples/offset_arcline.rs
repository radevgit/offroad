use geom::prelude::*;
use offroad::{offset::offset_arcline_to_arcline, prelude::OffsetCfg};

fn main() {
    // Configuration for offsetting
    let mut cfg = OffsetCfg::default();
    let mut svg = SVG::new(300.0, 300.0, "/tmp/arcline.svg");
    cfg.svg = Some(&mut svg);
    // Show original arcline in SVG output
    cfg.debug_orig = true;
    // Show final offset arclines in SVG output
    cfg.debug_final = true;

    let arc0 = arc_circle_parametrization(point(0.0, 0.0), point(100.0, 100.0), 0.0);
    let arc1 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 0.0), 0.5);
    let arc2 = arc_circle_parametrization(point(200.0, 0.0), point(0.0, 0.0), 1.3);
    let arcs_orig = vec![arc0, arc1, arc2];

    // Translate to fit in the SVG viewport
    //let poly = polyline_translate(&arcs_orig, point(40.0, 100.0));

    // Internal offsetting
    //let poly = polyline_reverse(&poly);
    let offset_arclines = offset_arcline_to_arcline(&arcs_orig, 15.0, &mut cfg);

    println!("Input arcline has {} vertices", arcs_orig.len());
    println!("Output has {} arclines", offset_arclines.len());
    for (i, arcline) in offset_arclines.iter().enumerate() {
        println!("Arcline {}: {} vertices", i, arcline.len());
    }

    if let Some(svg) = cfg.svg.as_deref_mut() {
        // Write svg to file
        svg.write_stroke_width(0.1);
    }
    assert!(
        offset_arclines.len() == 1,
        "Wrong number of offset arclines generated"
    );
}
