use offroad::graph::merge_ends::merge_close_endpoints;
use togo::prelude::*;

fn main() {
    println!("Testing merge_ends module...");
    
    // Create a simple test case with two close endpoints
    let mut arcs = vec![
        arcseg(point(0.0, 0.0), point(1.0, 0.0)),
        arcseg(point(1.0 + 1e-9, 0.0 + 1e-9), point(2.0, 0.0)), // Very close to (1,0)
    ];
    
    println!("Before merge:");
    println!("  Arc 0: a=({:.10}, {:.10}), b=({:.10}, {:.10})", 
             arcs[0].a.x, arcs[0].a.y, arcs[0].b.x, arcs[0].b.y);
    println!("  Arc 1: a=({:.10}, {:.10}), b=({:.10}, {:.10})", 
             arcs[1].a.x, arcs[1].a.y, arcs[1].b.x, arcs[1].b.y);
    
    let distance_before = (arcs[0].b - arcs[1].a).norm();
    println!("  Distance between endpoints: {:.12}", distance_before);
    
    // Apply merge
    merge_close_endpoints(&mut arcs, 1e-8);
    
    println!("\nAfter merge:");
    println!("  Arc 0: a=({:.10}, {:.10}), b=({:.10}, {:.10})", 
             arcs[0].a.x, arcs[0].a.y, arcs[0].b.x, arcs[0].b.y);
    if arcs.len() > 1 {
        println!("  Arc 1: a=({:.10}, {:.10}), b=({:.10}, {:.10})", 
                 arcs[1].a.x, arcs[1].a.y, arcs[1].b.x, arcs[1].b.y);
        
        let distance_after = (arcs[0].b - arcs[1].a).norm();
        println!("  Distance between endpoints: {:.12}", distance_after);
    } else {
        println!("  Only {} arc(s) remain", arcs.len());
    }
    
    println!("\nTest completed successfully!");
}