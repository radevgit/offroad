/// Test polylines for offsetting with various complexity levels
use togo::prelude::*;

const ZERO: f64 = 0f64;

/// Simple test polyline with mixed curvatures and edge cases.
pub fn pline_01() -> Vec<Polyline> {
    let pline = vec![
        pvertex(point(100.0, 100.0), 1.5),
        pvertex(point(100.0, 160.0), ZERO),
        pvertex(point(120.0, 200.0), ZERO),
        pvertex(point(128.0, 192.0), ZERO),
        pvertex(point(128.0, 205.0), ZERO),
        pvertex(point(136.0, 197.0), ZERO),
        pvertex(point(136.0, 245.0), -1.0), // zero radius after offset
        pvertex(point(131.0, 250.0), ZERO),
        pvertex(point(110.0, 250.0), -1.0), // zero radius after offset
        pvertex(point(78.0, 250.0), ZERO),
        pvertex(point(50.0, 250.0), -1.0), // zero radius after offset
        pvertex(point(38.0, 250.0), ZERO),
        pvertex(point(0.001, 250.0), 100000.0), // almost circle
        pvertex(point(0.0, 250.0), ZERO),
        pvertex(point(-52.0, 250.0), ZERO),
        //pvertex(point(-52.0, 150.0), -1.0),
        pvertex(
            point(-23.429621235520095, 204.88318696736243),
            -0.6068148963145962,
        ),
        pvertex(point(82.0, 150.0), 0f64),
        pvertex(point(50.0, 150.0), 1.0),
        pvertex(point(-20.0, 150.0), ZERO),
        pvertex(point(0.0, 100.0), ZERO),
    ];
    let pline2 = polyline_scale(&pline, 1.0);
    let plines = vec![pline2.clone()];
    return plines;
}

/// Test polyline for offseting.
pub fn pline_02() -> Polyline {
    let pline = vec![
        pvertex(point(50.0, 50.0), ZERO),
        pvertex(point(200.0, 50.0), ZERO),
        pvertex(point(180.0, 55.0), ZERO),
        pvertex(point(160.0, 65.0), ZERO),
        pvertex(point(140.0, 80.0), ZERO),
        pvertex(point(120.0, 100.0), ZERO),
        pvertex(point(100.0, 125.0), ZERO),
        pvertex(point(120.0, 150.0), ZERO),
        pvertex(point(140.0, 170.0), ZERO),
        pvertex(point(160.0, 185.0), ZERO),
        pvertex(point(180.0, 195.0), ZERO),
        pvertex(point(200.0, 200.0), ZERO),
        pvertex(point(-50.0, 200.0), ZERO),
        pvertex(point(-30.0, 195.0), ZERO),
        pvertex(point(-10.0, 185.0), ZERO),
        pvertex(point(10.0, 170.0), ZERO),
        pvertex(point(30.0, 150.0), ZERO),
        pvertex(point(50.0, 125.0), ZERO),
        pvertex(point(30.0, 100.0), ZERO),
        pvertex(point(10.0, 80.0), ZERO),
        pvertex(point(-10.0, 65.0), ZERO),
        pvertex(point(-30.0, 55.0), ZERO),
        pvertex(point(-50.0, 50.0), ZERO),
        pvertex(point(50.0, 50.0), ZERO),
    ];
    return pline;
}

/// Test polyline for offseting.
pub fn pline_03() -> Vec<Polyline> {
    let pline = vec![
        pvertex(point(0.0, 0.0), ZERO),
        pvertex(point(200.0, 0.0), ZERO),
        pvertex(point(200.0, 100.0), ZERO),
        pvertex(point(100.0, 100.0), ZERO),
        pvertex(point(100.0, 200.0), ZERO),
        pvertex(point(0.0, 200.0), ZERO),
    ];
    let pline2 = polyline_scale(&pline, 1.0);
    let plines = vec![pline2.clone()];
    return plines;
}

/// Test polyline for offseting.
pub fn pline_04() -> Vec<Polyline> {
    let outer = vec![
        pvertex(point(50.0, 50.0), 0.2),
        pvertex(point(100.0, 50.0), -0.5),
        pvertex(point(100.0, 100.0), 0.2),
        pvertex(point(50.0, 100.0), -0.5),
    ];
    let inner = vec![
        pvertex(point(75.0, 60.0), ZERO),
        pvertex(point(80.0, 75.0), ZERO),
        pvertex(point(75.0, 80.0), ZERO),
        pvertex(point(70.0, 75.0), ZERO),
    ];
    let inner = polyline_reverse(&inner);
    let mut plines = Vec::new();
    plines.push(outer);
    plines.push(inner);
    return plines;
}

/// Large test polyline with 250 segments - a non-self-intersecting rounded star polygon.
/// Mix of arcs and straight line segments forming a star-like pattern with varying radius.
pub fn pline_250() -> Polyline {
    let mut vertices = Vec::new();
    
    // Generate a rounded star polygon with 250 segments
    // Using a star pattern with alternating radius to avoid self-intersection
    let center = point(500.0, 500.0);
    let num_segments = 250;
    let num_points = 25; // 25 star points, each with 10 segments
    
    for i in 0..num_segments {
        let point_index = i / (num_segments / num_points);
        let segment_in_point = i % (num_segments / num_points);
        let progress = (segment_in_point as f64) / ((num_segments / num_points) as f64);
        
        // Alternating radius: outer and inner radii for star effect
        let is_outer = point_index % 2 == 0;
        let base_radius = if is_outer { 250.0 } else { 150.0 };
        
        // Smooth interpolation between points
        let angle = (point_index as f64 * 6.28318530718 / num_points as f64)
            + progress * (6.28318530718 / num_points as f64);
        
        let x = center.x + base_radius * angle.cos();
        let y = center.y + base_radius * angle.sin();
        
        // Alternate between arcs and lines
        let bulge = if i % 4 == 0 {
            0.2  // gentle arc
        } else if i % 4 == 1 {
            -0.15
        } else if i % 4 == 2 {
            0.1
        } else {
            ZERO
        };
        
        vertices.push(pvertex(point(x, y), bulge));
    }
    
    polyline_scale(&vertices, 1.0)
}
