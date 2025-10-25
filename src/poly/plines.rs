use togo::prelude::*;

const ZERO: f64 = 0.0;

/// Test polyline for offsetting.
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

/// Test polyline for offsetting.
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
    // let pline2 = polyline_scale(&pline, 1.0);
    // let plines = vec![pline2.clone()];
    return pline;
}

/// Test polyline for offsetting.
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

/// Test polyline for offsetting.
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

/// Gear-shaped polyline with ~500 vertices
/// Creates a gear outline using piecewise line segments approximating tooth geometry
/// Each segment will be converted to arcs by the togo library based on bulge values
pub fn pline_500() -> Polyline {
    let center = Point::new(250.0, 250.0);
    let root_radius = 80.0;       // Inner circle (root of teeth)
    let tip_radius = 150.0;       // Outer circle (tip of teeth)
    let num_teeth = 6;            // 6 teeth around the gear (wider spacing)
    let points_per_tooth = 83;    // ~83 points per tooth ≈ 500 total vertices

    let mut vertices = Vec::new();

    // Create all teeth around the full circle
    for tooth_num in 0..num_teeth {
        let tooth_center_angle = (tooth_num as f64) * 2.0 * std::f64::consts::PI / (num_teeth as f64);
        let tooth_width = 2.0 * std::f64::consts::PI / (num_teeth as f64); // Angular width per tooth
        
        // Each tooth: left flank → tip → right flank
        // We use piecewise line segments and add varying bulges
        for i in 0..points_per_tooth {
            let t = (i as f64) / ((points_per_tooth - 1) as f64); // 0 to 1 within tooth
            
            // Angle: left edge to right edge, with smooth curved transition at base
            // Use sine function to smooth the angle at base (t near 0 and 1) to prevent sharp corners
            let angle_offset = if t < 0.1 || t > 0.9 {
                // Smooth base zones: use sine curve to gradually transition angle
                let base_phase = if t < 0.1 {
                    t / 0.1  // 0 to 1 in first 10%
                } else {
                    (1.0 - t) / 0.1  // 1 to 0 in last 10%
                };
                // Sine easing: smooth transition
                (base_phase * std::f64::consts::PI).sin() * 0.1 * tooth_width
            } else {
                // Linear angle in the middle sections
                (t - 0.5) * tooth_width
            };
            let angle = tooth_center_angle + angle_offset;
            
            // Piecewise line approximation for both outer and inner tooth geometry
            // Create distinct flank segments on both sides
            let radius = if t < 0.2 {
                // Left flank outer: root → tip (steep)
                root_radius + (tip_radius - root_radius) * (t / 0.2)
            } else if t < 0.3 {
                // Left tip zone: stay near tip
                tip_radius - (tip_radius - root_radius) * 0.05 * ((t - 0.2) / 0.1)
            } else if t < 0.7 {
                // Tip plateau: stay at tip
                tip_radius
            } else if t < 0.8 {
                // Right tip zone: begin descent
                tip_radius - (tip_radius - root_radius) * 0.05 * ((t - 0.7) / 0.1)
            } else {
                // Right flank inward: tip → root (steep)
                tip_radius - (tip_radius - root_radius) * ((t - 0.8) / 0.2)
            };
            
            let point = Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            
            // Varying bulges: strong curvature throughout, especially at the base
            // Alternates sign within each section for additional visual complexity
            let bulge = if t < 0.33 {
                // Left flank: strong bulge at base, fades toward tip
                let local_t = t / 0.33;
                // Quadratic shape: strong at start, fades toward end
                let strength = 1.0 - local_t * local_t;
                let base_bulge = 0.7 * strength;
                if ((t * 10.0) as i32) % 2 == 0 {
                    base_bulge
                } else {
                    -base_bulge * 0.4
                }
            } else if t < 0.67 {
                // Tip zone: maintain strong bulge with alternation
                let local_t = (t - 0.33) / 0.34;
                let strength = 0.3 + 0.7 * (1.0 - local_t.powi(2));
                let base_bulge = 0.7 * strength;
                if ((t * 10.0) as i32) % 2 == 0 {
                    base_bulge
                } else {
                    -base_bulge * 0.4
                }
            } else {
                // Right flank: strong bulge at root, fades toward tip
                let local_t = (t - 0.67) / 0.33;
                let strength = 1.0 - local_t * local_t;
                let base_bulge = -0.7 * strength;
                if ((t * 10.0) as i32) % 2 == 0 {
                    base_bulge
                } else {
                    -base_bulge * 0.4
                }
            };
            
            vertices.push(pvertex(point, bulge));
        }
    }

    // Ensure proper closure: add first point at end to close the polyline
    if let Some(first) = vertices.first() {
        vertices.push(*first);
    }

    vertices
}
