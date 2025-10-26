use togo::{poly::arcline200, prelude::*};
use offroad::{offset::offset_arcline_to_arcline, prelude::OffsetCfg};

fn main() {
    let mut cfg = OffsetCfg::default();
    let mut svg = SVG::new(600.0, 600.0, Some("/tmp/debug_multi200.svg"));
    cfg.svg = Some(&mut svg);
    cfg.svg_orig = true;
    cfg.svg_final = true;

    let poly_orig = arcline200();
    println!("Original arcline200 has {} arcs", poly_orig.len());

    let mut total_offsets = vec![];
    
    for i in 1..5 {
        println!("\n=== Offset iteration i={} ===", i);
        let offset_distance = i as f64;
        println!("Calling offset_arcline_to_arcline with offset={}", offset_distance);
        
        let offset = offset_arcline_to_arcline(&poly_orig, offset_distance, &mut cfg);
        println!("Got {} offset arclines for offset {}", offset.len(), offset_distance);
        
        for (idx, arcline) in offset.iter().enumerate() {
            println!("  Arcline {}: {} arcs", idx, arcline.len());
        }
        
        total_offsets.extend(offset);
    }

    println!("\n=== FINAL RESULTS ===");
    println!("Total arclines: {}", total_offsets.len());
    
    if let Some(svg) = cfg.svg.as_mut() {
        svg.write_stroke_width(0.1);
    }
    
    println!("\nExpected: 228 total arclines");
    println!("Got: {} total arclines", total_offsets.len());
}
