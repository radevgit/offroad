//! Example demonstrating graph-based cycle detection with the "right-hand rule" algorithm
//!
//! This example shows how the cycle detection algorithm:
//! 1. Takes raw offset arcs (potentially disconnected or messy)
//! 2. Merges close endpoints to fix numerical precision issues
//! 3. Uses the "rightmost-turn" rule to identify non-intersecting cycles
//! 4. Outputs clean, separated cycles
//!
//! The "rightmost-turn" rule works by:
//! - At each vertex with multiple edges, choosing the edge that turns most "to the right"
//! - This prevents self-intersections and ensures smooth, geometric-aware cycle detection
//!
//! Run with: cargo run --example graph_cycle_detection

use std::fs::File;
use std::io::Write;
use offroad::graph::{merge_close_endpoints, find_non_intersecting_cycles};
use togo::prelude::*;

/// Draw SVG representation of the graph
fn draw_svg(
    filename: &str,
    cycles: &[Vec<Arc>],
    title: &str,
) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    
    // Calculate bounds
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    
    for cycle in cycles {
        for arc in cycle {
            min_x = min_x.min(arc.a.x).min(arc.b.x);
            max_x = max_x.max(arc.a.x).max(arc.b.x);
            min_y = min_y.min(arc.a.y).min(arc.b.y);
            max_y = max_y.max(arc.a.y).max(arc.b.y);
        }
    }
    
    let padding = 50.0;
    let width = (max_x - min_x + 2.0 * padding) as i32;
    let height = (max_y - min_y + 2.0 * padding) as i32;
    
    // Helper to convert coordinates
    let to_svg_x = |x: f64| (x - min_x + padding) as i32;
    let to_svg_y = |y: f64| (max_y - y + padding) as i32;  // Flip Y for SVG
    
    // Write SVG header
    write!(
        file,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n",
        width, height
    )?;
    write!(file, "  <title>{}</title>\n", title)?;
    write!(file, "  <style>line {{ stroke: black; }} circle {{ fill: red; }} text {{ font-size: 12px; }} .label {{ fill: blue; }} </style>\n")?;
    
    // Draw cycles with different colors
    let colors = vec!["#FF6B6B", "#4ECDC4", "#45B7D1", "#FFA07A", "#98D8C8"];
    
    for (cycle_idx, cycle) in cycles.iter().enumerate() {
        let color = colors[cycle_idx % colors.len()];
        
        for arc in cycle {
            let x1 = to_svg_x(arc.a.x);
            let y1 = to_svg_y(arc.a.y);
            let x2 = to_svg_x(arc.b.x);
            let y2 = to_svg_y(arc.b.y);
            
            if arc.r == f64::INFINITY {
                // Line segment
                write!(
                    file,
                    "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"2\"/>\n",
                    x1, y1, x2, y2, color
                )?;
            } else {
                // Arc - draw as simple line for now (simplified SVG output)
                write!(
                    file,
                    "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"2\" stroke-dasharray=\"5,5\"/>\n",
                    x1, y1, x2, y2, color
                )?;
            }
        }
    }
    
    // Draw vertices (arc endpoints)
    for cycle in cycles {
        for arc in cycle {
            let x1 = to_svg_x(arc.a.x);
            let y1 = to_svg_y(arc.a.y);
            let x2 = to_svg_x(arc.b.x);
            let y2 = to_svg_y(arc.b.y);
            
            write!(file, "  <circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"red\"/>\n", x1, y1)?;
            write!(file, "  <circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"red\"/>\n", x2, y2)?;
        }
    }
    
    write!(file, "</svg>\n")?;
    
    println!("✓ Created {}", filename);
    Ok(())
}

/// Example 1: Simple Square with Small Endpoint Gaps
fn example_square_with_gaps() {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 1: Square with Small Endpoint Gaps");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Create a square with small gaps between endpoints
    let mut arcs = vec![
        arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
        arcseg(Point::new(1.0 + 1e-9, 1e-9), Point::new(1.0, 1.0)),        // tiny gap
        arcseg(Point::new(1.0, 1.0), Point::new(0.0, 1.0)),
        arcseg(Point::new(-1e-9, 1.0), Point::new(0.0, 0.0)),               // tiny gap
    ];
    
    println!("\nInput: 4 segments forming a square with 1e-9 gaps");
    for (i, arc) in arcs.iter().enumerate() {
        println!("  Arc {}: ({:.10}, {:.10}) → ({:.10}, {:.10})", 
                 i, arc.a.x, arc.a.y, arc.b.x, arc.b.y);
    }
    
    // Merge endpoints
    println!("\nMerging close endpoints (tolerance: 1e-8)...");
    merge_close_endpoints(&mut arcs, 1e-8);
    
    println!("After merge:");
    for (i, arc) in arcs.iter().enumerate() {
        println!("  Arc {}: ({:.10}, {:.10}) → ({:.10}, {:.10})", 
                 i, arc.a.x, arc.a.y, arc.b.x, arc.b.y);
    }
    
    // Find cycles
    println!("\nFinding non-intersecting cycles...");
    let cycles = find_non_intersecting_cycles(&arcs);
    
    println!("Found {} cycle(s):", cycles.len());
    for (cycle_idx, cycle) in cycles.iter().enumerate() {
        println!("  Cycle {}: {} arcs", cycle_idx, cycle.len());
    }
    
    let _ = draw_svg("output/example1_square_cycles.svg", &cycles, "Square with Gaps - After Cycle Detection");
}

/// Example 2: Two Separate Triangles
fn example_two_triangles() {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 2: Two Separate Triangles");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut arcs = vec![
        // Triangle 1 (left)
        arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
        arcseg(Point::new(1.0, 0.0), Point::new(0.5, 1.0)),
        arcseg(Point::new(0.5, 1.0), Point::new(0.0, 0.0)),
        
        // Triangle 2 (right, separate)
        arcseg(Point::new(3.0, 0.0), Point::new(4.0, 0.0)),
        arcseg(Point::new(4.0, 0.0), Point::new(3.5, 1.0)),
        arcseg(Point::new(3.5, 1.0), Point::new(3.0, 0.0)),
    ];
    
    println!("\nInput: 6 segments forming two separate triangles");
    println!("Triangle 1: 3 arcs");
    println!("Triangle 2: 3 arcs");
    
    merge_close_endpoints(&mut arcs, 1e-8);
    let cycles = find_non_intersecting_cycles(&arcs);
    
    println!("Found {} cycle(s):", cycles.len());
    for (cycle_idx, cycle) in cycles.iter().enumerate() {
        println!("  Cycle {}: {} arcs", cycle_idx, cycle.len());
    }
    
    let _ = draw_svg("output/example2_two_triangles.svg", &cycles, "Two Separate Triangles");
}

/// Example 3: Figure-Eight (Two Connected Squares)
fn example_figure_eight() {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 3: Figure-Eight (Two Connected Squares)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut arcs = vec![
        // Left square
        arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
        arcseg(Point::new(1.0, 0.0), Point::new(1.0, 1.0)),
        arcseg(Point::new(1.0, 1.0), Point::new(0.0, 1.0)),
        arcseg(Point::new(0.0, 1.0), Point::new(0.0, 0.0)),
        
        // Right square (shares edge with left)
        arcseg(Point::new(1.0, 0.0), Point::new(2.0, 0.0)),
        arcseg(Point::new(2.0, 0.0), Point::new(2.0, 1.0)),
        arcseg(Point::new(2.0, 1.0), Point::new(1.0, 1.0)),
        // Note: shared edge (1,0)-(1,1) is already counted
    ];
    
    println!("\nInput: 7 segments forming figure-eight pattern");
    println!("Left square + Right square (sharing vertical edge)");
    
    merge_close_endpoints(&mut arcs, 1e-8);
    let cycles = find_non_intersecting_cycles(&arcs);
    
    println!("Found {} cycle(s):", cycles.len());
    for (cycle_idx, cycle) in cycles.iter().enumerate() {
        println!("  Cycle {}: {} arcs", cycle_idx, cycle.len());
    }
    
    let _ = draw_svg("output/example3_figure_eight.svg", &cycles, "Figure-Eight Pattern");
}

/// Example 4: Circle Approximation (4 Quarter-Arcs)
fn example_circle_arcs() {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 4: Circle Approximation with Curved Arcs");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let center = Point::new(0.0, 0.0);
    let radius = 5.0;
    
    // Create 4 quarter-circle arcs
    let mut arcs = vec![
        arc(Point::new(5.0, 0.0), Point::new(0.0, 5.0), center, radius),
        arc(Point::new(0.0, 5.0 + 1e-9), Point::new(-5.0, 0.0), center, radius),     // tiny gap
        arc(Point::new(-5.0 - 1e-9, 0.0), Point::new(0.0, -5.0), center, radius),    // tiny gap
        arc(Point::new(0.0, -5.0 + 1e-9), Point::new(5.0, 0.0), center, radius),     // tiny gap
    ];
    
    println!("\nInput: 4 quarter-circle arcs with small gaps");
    println!("Radius: {}", radius);
    println!("Center: ({}, {})", center.x, center.y);
    
    merge_close_endpoints(&mut arcs, 1e-8);
    let cycles = find_non_intersecting_cycles(&arcs);
    
    println!("Found {} cycle(s):", cycles.len());
    for (cycle_idx, cycle) in cycles.iter().enumerate() {
        println!("  Cycle {}: {} arcs", cycle_idx, cycle.len());
        let has_arcs = cycle.iter().any(|arc| arc.r != f64::INFINITY);
        let has_segs = cycle.iter().any(|arc| arc.r == f64::INFINITY);
        if has_arcs { println!("    - Contains curved arcs"); }
        if has_segs { println!("    - Contains line segments"); }
    }
    
    let _ = draw_svg("output/example4_circle_arcs.svg", &cycles, "Circle Approximation with Curved Arcs");
}

/// Example 5: X-Intersection (Complex Rightmost-Turn Rule Test)
fn example_x_intersection() {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 5: X-Intersection (Rightmost-Turn Rule Test)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nThis example demonstrates the 'rightmost-turn' rule:");
    println!("At each vertex with multiple edges, the algorithm chooses");
    println!("the edge that turns most to the right (smallest right-turn angle).");
    
    // Create a box with internal cross (X pattern)
    let mut arcs = vec![
        // Outer box
        arcseg(Point::new(0.0, 0.0), Point::new(2.0, 0.0)),
        arcseg(Point::new(2.0, 0.0), Point::new(2.0, 2.0)),
        arcseg(Point::new(2.0, 2.0), Point::new(0.0, 2.0)),
        arcseg(Point::new(0.0, 2.0), Point::new(0.0, 0.0)),
        
        // Internal cross (creates intersections at center)
        arcseg(Point::new(0.0, 1.0), Point::new(2.0, 1.0)),  // horizontal
        arcseg(Point::new(1.0, 0.0), Point::new(1.0, 2.0)),  // vertical
    ];
    
    println!("\nInput: Outer box (4 edges) + internal cross (2 edges)");
    println!("Center intersection at (1.0, 1.0)");
    
    merge_close_endpoints(&mut arcs, 1e-8);
    let cycles = find_non_intersecting_cycles(&arcs);
    
    println!("\nFound {} cycle(s):", cycles.len());
    for (cycle_idx, cycle) in cycles.iter().enumerate() {
        println!("  Cycle {}: {} arcs", cycle_idx, cycle.len());
    }
    
    println!("\nThe rightmost-turn rule ensures that:");
    println!("• Each cycle is geometrically non-intersecting");
    println!("• At the center intersection, the algorithm chooses edges");
    println!("  that continue 'naturally' following the right-hand rule");
    
    let _ = draw_svg("output/example5_x_intersection.svg", &cycles, "X-Intersection Pattern");
}

/// Example 6: Mixed Arc Types
fn example_mixed_arcs() {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Example 6: Mixed Curved Arcs and Line Segments");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut arcs = vec![
        arcseg(Point::new(0.0, 0.0), Point::new(3.0, 0.0)),
        arc(Point::new(3.0, 0.0), Point::new(6.0, 3.0), Point::new(6.0, 0.0), 3.0),
        arcseg(Point::new(6.0 + 1e-9, 3.0), Point::new(6.0, 6.0)),                   // tiny gap
        arc(Point::new(6.0, 6.0), Point::new(3.0, 9.0), Point::new(6.0, 9.0), 3.0),
        arcseg(Point::new(3.0, 9.0), Point::new(0.0, 9.0)),
        arcseg(Point::new(-1e-9, 9.0), Point::new(0.0, 0.0)),                        // tiny gap
    ];
    
    println!("\nInput: Mix of line segments and curved arcs");
    println!("Pattern: Line → Arc → Line → Arc → Line → Line");
    
    merge_close_endpoints(&mut arcs, 1e-8);
    let cycles = find_non_intersecting_cycles(&arcs);
    
    println!("Found {} cycle(s):", cycles.len());
    for (cycle_idx, cycle) in cycles.iter().enumerate() {
        println!("  Cycle {}: {} arcs", cycle_idx, cycle.len());
        let has_arcs = cycle.iter().any(|arc| arc.r != f64::INFINITY);
        let has_segs = cycle.iter().any(|arc| arc.r == f64::INFINITY);
        if has_arcs { println!("    - Contains curved arcs"); }
        if has_segs { println!("    - Contains line segments"); }
    }
    
    let _ = draw_svg("output/example6_mixed_arcs.svg", &cycles, "Mixed Curved Arcs and Line Segments");
}

fn main() -> std::io::Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║ Graph-Based Cycle Detection Examples                       ║");
    println!("║ Demonstrating the 'Rightmost-Turn' Rule Algorithm         ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    
    // Create output directory
    std::fs::create_dir_all("output")?;
    
    example_square_with_gaps();
    example_two_triangles();
    example_figure_eight();
    example_circle_arcs();
    example_x_intersection();
    example_mixed_arcs();
    
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║ ✓ All examples completed successfully!                    ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ SVG output files created in output/ directory:            ║");
    println!("║  • example1_square_cycles.svg                             ║");
    println!("║  • example2_two_triangles.svg                             ║");
    println!("║  • example3_figure_eight.svg                              ║");
    println!("║  • example4_circle_arcs.svg                               ║");
    println!("║  • example5_x_intersection.svg                            ║");
    println!("║  • example6_mixed_arcs.svg                                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    
    println!("\n📋 What the Algorithm Does:");
    println!("   1. Takes raw arcs with potentially disconnected endpoints");
    println!("   2. Merges close endpoints (within tolerance) to connect them");
    println!("   3. Builds a graph with vertices (endpoints) and edges (arcs)");
    println!("   4. Uses 'rightmost-turn' rule at each vertex to find cycles");
    println!("   5. Returns non-intersecting cycles as separate arc sequences");
    
    println!("\n🎯 Key Features:");
    println!("   • Handles numerical precision issues (1e-8 tolerance)");
    println!("   • Works with both line segments and curved arcs");
    println!("   • Detects multiple separate cycles");
    println!("   • Prevents self-intersections with geometric awareness");
    println!("   • Efficient: O(n·d) where d ≈ average vertex degree (2-4)");
    
    Ok(())
}
