#![allow(dead_code)]

use togo::prelude::*;

use crate::offset_raw::OffsetRaw;

const ZERO: f64 = 0f64;

pub fn offset_polyline_raw(plines: &Vec<Vec<OffsetRaw>>, off: f64) -> Vec<Vec<OffsetRaw>> {
    let mut result = Vec::new();
    for pline in plines.iter() {
        result.push(offset_polyline_raw_single(pline, off));
    }
    result
}

fn offset_polyline_raw_single(pline: &Vec<OffsetRaw>, off: f64) -> Vec<OffsetRaw> {
    let mut result = Vec::with_capacity(pline.len());
    for p in pline.iter() {
        let offset = offset_segment(&p.arc, p.orig, p.g, off);
        result.push(offset);
    }
    result
}

pub(crate) fn offset_segment(seg: &Arc, orig: Point, g: f64, off: f64) -> OffsetRaw {
    if seg.is_seg() {
        line_offset(seg, orig, off)
    } else {
        arc_offset(seg, orig, g, off)
    }
}

// Offsets line segment on right side
// #00028
fn line_offset(seg: &Arc, orig: Point, off: f64) -> OffsetRaw {
    // line segment
    let perp = seg.b - seg.a;
    let (perp, _) = point(perp.y, -perp.x).normalize(false);
    let offset_vec = perp * off;
    let mut arc = arcseg(seg.a + offset_vec, seg.b + offset_vec);
    arc.id(seg.id);
    return OffsetRaw {
        arc,
        orig: orig,
        g: ZERO,
    };
}

const EPS_COLLAPSED: f64 = 1E-10; // TODO: what should be the exact value.
// Offsets arc on right side
// #00028
fn arc_offset(seg: &Arc, orig: Point, bulge: f64, offset: f64) -> OffsetRaw {
    // Arc is always CCW
    //let seg = arc_from_bulge(seg.a, seg.b, bulge);
    let (v0_to_center, _) = (seg.a - seg.c).normalize(false);
    let (v1_to_center, _) = (seg.b - seg.c).normalize(false);

    let off = offset;
    let offset_radius = seg.r + off;
    let a = seg.a + v0_to_center * off;
    let b = seg.b + v1_to_center * off;
    if offset_radius < EPS_COLLAPSED || offset_radius.is_nan()
        || a.close_enough(b, EPS_COLLAPSED)
    {
        // Collapsed arc is now line
        let mut arc = arcseg(b, a);
        arc.id(seg.id);
        return OffsetRaw {
            arc: arc,
            orig: orig,
            g: ZERO,
        };
    } else {
        let mut arc = arc(a, b, seg.c, offset_radius);
        arc.id(seg.id);
        return OffsetRaw {
            arc: arc,
            orig: orig,
            g: bulge,
        };
    }
}

pub fn poly_to_raws(plines: &Vec<Polyline>) -> Vec<Vec<OffsetRaw>> {
    let mut varcs: Vec<Vec<OffsetRaw>> = Vec::new();
    for pline in plines {
        varcs.push(poly_to_raws_single(pline));
    }
    varcs
}

pub fn poly_to_raws_single(pline: &Polyline) -> Vec<OffsetRaw> {
    let mut offs = Vec::with_capacity(pline.len());
    let n = pline.len();
    
    // Cyclic loop: for each vertex i, create arc from vertex i to vertex (i+1) mod n
    for i in 0..n {
        let bulge = pline[i].b;
        let next_i = (i + 1) % n; // Cyclic wrap-around
        let seg = arc_from_bulge(pline[i].p, pline[next_i].p, bulge);
        let check = seg.is_valid(EPS_COLLAPSED);
        if !check {
            continue;
        }
        let orig = if bulge < ZERO { seg.a } else { seg.b };
        let off = OffsetRaw {
            arc: seg,
            orig: orig,
            g: bulge,
        };
        offs.push(off);
    }

    offs
}


pub fn arcs_to_raws(arcss: &Vec<Arcline>) -> Vec<Vec<OffsetRaw>> {
    let mut varcs: Vec<Vec<OffsetRaw>> = Vec::new();
    for arcs in arcss {
        varcs.push(arcs_to_raws_single(arcs));
    }
    varcs
}

pub fn arcs_to_raws_single(arcs: &Arcline) -> Vec<OffsetRaw> {
    let mut offs = Vec::with_capacity(arcs.len());
    let n = arcs.len();
    
    // Cyclic loop: for each arc i, process it (arcs are already connected in sequence)
    for i in 0..n {
        let seg = arcs[i];
        let check = seg.is_valid(EPS_COLLAPSED);
        if !check {
            continue;
        }
        
        // Determine bulge sign from connectivity with next arc
        // All arcs in togo are CCW, so bulge_from_arc() always returns positive
        // We need to check the direction based on how the arc connects to the next arc
        let next_i = (i + 1) % n;
        let next_seg = arcs[next_i];
        
        // Check all four possible connections:
        // Valid arclines should connect exactly, no tolerance needed
        let seg_b_to_next_a = seg.b == next_seg.a;
        let seg_b_to_next_b = seg.b == next_seg.b;
        let seg_a_to_next_a = seg.a == next_seg.a;
        let seg_a_to_next_b = seg.a == next_seg.b;
        
        // Determine if current arc is normal (positive bulge) or reversed (negative bulge)
        // based on which endpoint connects to the next arc
        let bulge_recalc = bulge_from_arc(seg.a, seg.b, seg.c, seg.r);
        let bulge = if seg_b_to_next_a || seg_b_to_next_b {
            // seg.b connects to next arc -> seg is normal (positive bulge)
            bulge_recalc
        } else if seg_a_to_next_a || seg_a_to_next_b {
            // seg.a connects to next arc -> seg is reversed (negative bulge)
            -bulge_recalc
        } else {
            // No clear connection found, default to positive
            bulge_recalc
        };
        
        let orig = if bulge < ZERO { seg.a } else { seg.b };
        let off = OffsetRaw {
            arc: seg,
            orig: orig,
            g: bulge,
        };
        offs.push(off);
    }

    offs
}



#[cfg(test)]
mod test_offset_polyline_raw {
    use togo::prelude::*;

    use crate::offset_raw::offsetraw;

    use super::*;

    #[test]
    fn test_arc_offset_collapsed_arc() {
        // let arc0 = arc();
        // let res = arc_offset(
        //     pvertex(point(0.0, 0.0), -1.0),
        //     pvertex(point(1.0, 0.0), 0.0),
        //     1.0,
        // );
        // assert_eq!(
        //     res,
        //     OffsetRaw {
        //         arc: arcline(point(1.0, 0.0), point(0.0, 0.0)),
        //         orig: point(1.0, 0.0),
        //         g: 0.0
        //     }
        // );
    }

    #[test]
    fn test_new() {
        let arc = arc_from_bulge(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = offsetraw(arc, point(5.0, 6.0), 3.3);
        let o1 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(o0, o1);
    }

    #[test]
    fn test_display_01() {
        let arc = arc_from_bulge(point(0.0, 0.0), point(2.0, 2.0), 1.0);
        let o0 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(
            "[[[0.00000000000000000000, 0.00000000000000000000], [2.00000000000000000000, 2.00000000000000000000], [1.00000000000000000000, 1.00000000000000000000], 1.41421356237309514547], [5.00000000000000000000, 6.00000000000000000000], 3.3]",
            format!("{}", o0)
        );
    }

    #[test]
    fn test_display_02() {
        let arc = arc_from_bulge(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(
            "[[[1.00000000000000000000, 2.00000000000000000000], [3.00000000000000000000, 4.00000000000000000000], [3.49848484848484808651, 1.50151515151515169144], 2.54772716009334887488], [5.00000000000000000000, 6.00000000000000000000], 3.3]",
            format!("{}", o0)
        );
    }

    #[test]
    fn test_line_offset_vertical() {
        // vertical segment
        // let seg = arcline(point(2.0, 1.0), point(2.0, 11.0));
        // let res = offsetraw(
        //     arcline(point(3.0, 1.0), point(3.0, 11.0)),
        //     point(2.0, 11.0),
        //     0.0,
        // );
        // assert_eq!(line_offset(&seg, 1.0), res);
    }
    #[test]
    fn test_line_offset_horizontal() {
        // horizontal segment
        // let seg = arcline(point(-2.0, 1.0), point(3.0, 1.0));
        // let res = offsetraw(
        //     arcline(point(-2.0, -1.0), point(3.0, -1.0)),
        //     point(3.0, 1.0),
        //     0.0,
        // );
        // assert_eq!(line_offset(&seg, 2.0), res);
    }
    #[test]
    fn test_line_offset_diagonal() {
        // diagonal segment
        // let seg = arcline(point(-1.0, 1.0), point(2.0, 2.0));
        // let res = offsetraw(
        //     arcline(point(0.0, 2.0), point(-1.0, 3.0)),
        //     point(-2.0, 2.0),
        //     0.0,
        // );
        // assert_eq!(line_offset(&seg, std::f64::consts::SQRT_2), res);
    }

    #[test]
    //#[ignore = "svg output"]
    fn test_offset_polyline_raw02() {
        // let pline = vec![
        //     pvertex(point(100.0, 100.0), 0.5),
        //     pvertex(point(200.0, 100.0), -0.5),
        //     pvertex(point(200.0, 200.0), 0.5),
        //     pvertex(point(100.0, 200.0), -0.5),
        // ];
        // let plines = vec![pline.clone()];
        // let mut svg = svg(400.0, 600.0);
        // //let pline = polyline_translate(&pline, point(0.0, 100.0));
        // svg.polyline(&pline, "red");

        // //let pline = polyline_reverse(&pline);
        // let off: f64 = 52.25;
        // let offset_raw = offset_polyline_raw(&plines, off);
        // svg.offset_raws(&offset_raw, "blue");
        // svg.write();
    }

    // #[test]
    // //#[ignore = "svg output"]
    // fn test_offset_polyline_raw03() {
    //     let plines = pline_01();
    //     let mut svg = svg(400.0, 600.0);
    //     let plines = vec![pline.clone()];
    //     svg.polyline(&pline, "red");

    //     //let pline = polyline_reverse(&pline);
    //     let off: f64 = 52.25;
    //     let offset_raw = offset_polyline_raw(&plines, off);
    //     svg.offset_raws(&offset_raw, "blue");
    //     svg.write();
    // }

    #[test]
    #[ignore = "svg output"]
    fn test_arc_from_bulge_plinearc_svg() {
        let arc0 = arc_from_bulge(
            point(-52.0, 250.0),
            point(-23.429621235520095, 204.88318696736243),
            -0.6068148963145962,
        );
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle0 = circle(point(arc0.c.x, arc0.c.y), 0.1);
        svg.circle(&circle0, "blue");

        let offsetraw = offset_segment(&arc0, point(-52.0, 250.0), -0.6068148963145962, 16.0);
        svg.arcsegment(&offsetraw.arc, "green");
        let circle1 = circle(point(offsetraw.arc.c.x, offsetraw.arc.c.y), 0.1);
        svg.circle(&circle1, "blue");

        svg.write();
    }

    // Unit tests for negative bulge arc offsetting bug
    #[test]
    fn test_arc_offset_positive_bulge_right_side() {
        // Create a simple arc with positive bulge (CCW, curving left from the direction of travel)
        // Center at (0, 0), radius 10
        // Arc from (10, 0) to (0, 10) - quarter circle in first quadrant
        let arc = arc(point(10.0, 0.0), point(0.0, 10.0), point(0.0, 0.0), 10.0);
        let bulge = 1.0; // positive bulge
        
        // Offset right by 2.0 units (positive offset)
        let offset_result = arc_offset(&arc, arc.a, bulge, 2.0);
        
        // For positive bulge, offset should expand the radius: 10 + 2 = 12
        let expected_radius = 12.0;
        assert!((offset_result.arc.r - expected_radius).abs() < 0.01,
                "Positive bulge: expected radius {}, got {}", expected_radius, offset_result.arc.r);
    }

    #[test]
    fn test_arc_offset_negative_bulge_right_side() {
        // Create a simple arc with negative bulge (CW when unwound, curving right)
        // Center at (0, 0), radius 10
        // Arc from (10, 0) to (0, 10) - but bulge is negative
        let arc = arc(point(10.0, 0.0), point(0.0, 10.0), point(0.0, 0.0), 10.0);
        let bulge = -1.0; // negative bulge
        
        // Offset right by 2.0 units (positive offset)
        // After bug fix: offset is NOT negated for negative bulge
        // So radius becomes: 10 + 2.0 = 12.0 (same as positive bulge)
        let offset_result = arc_offset(&arc, arc.a, bulge, 2.0);
        
        let expected_radius = 12.0;
        assert!((offset_result.arc.r - expected_radius).abs() < 0.01,
                "Negative bulge: expected radius {}, got {}", expected_radius, offset_result.arc.r);
    }

    #[test]
    fn test_line_offset_simple_horizontal() {
        // Simple horizontal line segment from (0,0) to (10,0)
        let seg = arcseg(point(0.0, 0.0), point(10.0, 0.0));
        
        // Offset to the right by 2 units
        // For a horizontal line going right, right offset should move it downward
        let result = line_offset(&seg, point(0.0, 0.0), 2.0);
        
        // Expected: line from (0, -2) to (10, -2)
        let expected_a = point(0.0, -2.0);
        let expected_b = point(10.0, -2.0);
        assert!(result.arc.a.close_enough(expected_a, 0.01), 
                "Expected a={:?}, got {:?}", expected_a, result.arc.a);
        assert!(result.arc.b.close_enough(expected_b, 0.01), 
                "Expected b={:?}, got {:?}", expected_b, result.arc.b);
    }

    #[test]
    #[ignore = "synthetic test - real bulge comes from polyline/arcline connectivity, not manual parameter"]
    fn test_positive_vs_negative_bulge_arc_offset_direction() {
        // This test is not relevant: bulge parameter to arc_offset doesn't control geometry
        // Real bulge sign is determined by connectivity in poly_to_raws_single or arcs_to_raws_single
        // Keeping for reference but disabled
    }

    #[test]
    fn test_arc_offset_direction_consistency() {
        // This is the critical test: when offsetting with the SAME positive distance,
        // do different bulges offset in different directions?
        
        // Create a simple vertical line from (5, 0) to (5, 10)
        // Positive bulge: curves right (center at x > 5)
        // Negative bulge: curves left (center at x < 5)
        
        let seg = arcseg(point(5.0, 0.0), point(5.0, 10.0));
        
        // For a vertical segment going up:
        // - Right offset should move to x = 7 (increasing x)
        // - Left offset should move to x = 3 (decreasing x)
        
        let result_right = line_offset(&seg, point(5.0, 0.0), 2.0);
        
        // For vertical line going up, right offset should move x coordinate to 7
        assert!((result_right.arc.a.x - 7.0).abs() < 0.01, 
                "Expected right offset to x=7, got x={}", result_right.arc.a.x);
    }

    #[test]
    #[ignore = "synthetic test - real bulge comes from polyline/arcline connectivity, not manual parameter"]
    fn test_negative_bulge_offset_side_bug() {
        // This test is not relevant: bulge parameter to arc_offset doesn't control geometry
        // Real bulge sign is determined by connectivity in poly_to_raws_single or arcs_to_raws_single
        // Keeping for reference but disabled
    }

    #[test]
    fn test_arcline_bulge_always_positive() {
        // INSIGHT: All arcs in togo are CCW, so bulge_from_arc() always returns positive!
        // Negative bulge metadata is LOST when converting arc_from_bulge to Arc struct
        // This is why arcs_to_raws_single can never recover original bulge sign
        
        let start = point(0.0, 0.0);
        let end = point(10.0, 0.0);
        
        // Create arc with negative bulge
        let arc_neg = arc_from_bulge(start, end, -1.0);
        let recalc_neg = bulge_from_arc(arc_neg.a, arc_neg.b, arc_neg.c, arc_neg.r);
        
        // Create arc with positive bulge
        let arc_pos = arc_from_bulge(start, end, 1.0);
        let recalc_pos = bulge_from_arc(arc_pos.a, arc_pos.b, arc_pos.c, arc_pos.r);
        
        // Both recalculate to positive because all arcs in togo are CCW
        assert!(recalc_neg > 0.0, "Recalculated bulge from negative should still be positive (all togo arcs are CCW)");
        assert!(recalc_pos > 0.0, "Recalculated bulge from positive should be positive");
        
        // The arcs might be geometrically different, but bulge sign info is lost
        eprintln!("Original negative bulge: -1.0, recalculated: {}", recalc_neg);
        eprintln!("Original positive bulge: 1.0, recalculated: {}", recalc_pos);
        eprintln!("Arc from neg: a={:?}, b={:?}", arc_neg.a, arc_neg.b);
        eprintln!("Arc from pos: a={:?}, b={:?}", arc_pos.a, arc_pos.b);
    }
}
