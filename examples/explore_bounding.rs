// Exploration of TOGO bounding box methods
// Compare aabb_from_arc (fast/loose) vs arc_bounding_circle (tight/slow)

use togo::prelude::*;

fn main() {
    println!("TOGO Bounding Box Method Comparison");
    println!("===================================\n");

    // Test arc 1: Small quarter circle
    let arc1 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
    compare_bounding_methods(&arc1, "Small quarter circle");

    // Test arc 2: Large arc with small radius
    let arc2 = arc(point(0.0, 0.0), point(10.0, 0.0), point(5.0, -1.0), 5.01);
    compare_bounding_methods(&arc2, "Large arc with small radius");

    // Test arc 3: Full circle arc (almost)
    let arc3 = arc(point(1.0, 0.0), point(0.999, 0.01), point(0.0, 0.0), 1.0);
    compare_bounding_methods(&arc3, "Nearly-full circle arc");

    // Test arc 4: Tight arc (wide radius)
    let arc4 = arc(point(0.0, 0.0), point(10.0, 0.0), point(5.0, 100.0), 100.1);
    compare_bounding_methods(&arc4, "Tight arc with wide radius");
}

fn compare_bounding_methods(arc: &Arc, description: &str) {
    println!("Test: {}", description);
    println!("Arc: center={:?}, radius={}, a={:?}, b={:?}", 
             arc.c, arc.r, arc.a, arc.b);

    // Method 1: Arc bounding circle (tight but potentially slower with tan calculations)
    let bounding_circle = arc_bounding_circle(arc);
    let circle_aabb = circle_to_aabb(&bounding_circle);
    let circle_area = (bounding_circle.r * 2.0) * (bounding_circle.r * 2.0);

    // Method 2: Simple AABB from arc endpoints and circle bounds (fast but loose)
    let loose_aabb = simple_aabb_from_arc(arc);
    let loose_area = (loose_aabb.1 - loose_aabb.0) * (loose_aabb.3 - loose_aabb.2);

    // Method 3: Arc radius-aware bounds (middle ground)
    let arc_aware_aabb = arc_aware_aabb_from_arc(arc);
    let arc_aware_area = (arc_aware_aabb.1 - arc_aware_aabb.0) * (arc_aware_aabb.3 - arc_aware_aabb.2);

    println!("  Tight (arc_bounding_circle):");
    println!("    Circle center: {:?}, radius: {}", bounding_circle.c, bounding_circle.r);
    println!("    AABB: x=[{:.4}, {:.4}], y=[{:.4}, {:.4}]", 
             circle_aabb.0, circle_aabb.1, circle_aabb.2, circle_aabb.3);
    println!("    Area: {:.4}", circle_area);

    println!("  Loose (endpoints + circle bounds):");
    println!("    AABB: x=[{:.4}, {:.4}], y=[{:.4}, {:.4}]", 
             loose_aabb.0, loose_aabb.1, loose_aabb.2, loose_aabb.3);
    println!("    Area: {:.4}", loose_area);

    println!("  Arc-aware (simplified bounds):");
    println!("    AABB: x=[{:.4}, {:.4}], y=[{:.4}, {:.4}]", 
             arc_aware_aabb.0, arc_aware_aabb.1, arc_aware_aabb.2, arc_aware_aabb.3);
    println!("    Area: {:.4}", arc_aware_area);

    let overhead = ((loose_area - circle_area) / circle_area) * 100.0;
    let overhead_aware = ((arc_aware_area - circle_area) / circle_area) * 100.0;
    println!("  Overhead vs tight: {:.1}% (loose), {:.1}% (aware)\n", overhead, overhead_aware);
}

/// Convert circle to AABB
fn circle_to_aabb(c: &Circle) -> (f64, f64, f64, f64) {
    let r = c.r;
    (c.c.x - r, c.c.x + r, c.c.y - r, c.c.y + r)
}

/// Simple loose AABB: just endpoints + circle bounds
fn simple_aabb_from_arc(arc: &Arc) -> (f64, f64, f64, f64) {
    let mut min_x = arc.a.x.min(arc.b.x);
    let mut max_x = arc.a.x.max(arc.b.x);
    let mut min_y = arc.a.y.min(arc.b.y);
    let mut max_y = arc.a.y.max(arc.b.y);

    // Include full circle bounds (loose but very fast)
    let r = arc.r;
    let cx = arc.c.x;
    let cy = arc.c.y;

    min_x = min_x.min(cx - r);
    max_x = max_x.max(cx + r);
    min_y = min_y.min(cy - r);
    max_y = max_y.max(cy + r);

    (min_x, max_x, min_y, max_y)
}

/// Arc-aware AABB: similar to simple but annotated
fn arc_aware_aabb_from_arc(arc: &Arc) -> (f64, f64, f64, f64) {
    // This is the same as simple_aabb_from_arc for now
    // Could be optimized by checking if arc spans cardinal directions
    simple_aabb_from_arc(arc)
}
