#![allow(dead_code)]
#![deny(unused_results)]

use std::vec;

use geom::prelude::*;

use crate::{
    offset_connect_raw::offset_connect_raw,
    offset_prune_invalid::offset_prune_invalid,
    offset_raw::OffsetRaw,
    offset_reconnect_arcs::{offset_reconnect_arcs, remove_bridge_arcs},
    offset_segments_raws::offset_segments_raws,
    offset_split_arcs::offset_split_arcs,
};

const ZERO: f64 = 0.0;
/// Configuration options for offsetting operations.
pub struct OffsetCfg<'a> {
    /// Optional SVG context for rendering
    pub svg: Option<&'a mut SVG>,
    /// Flag to indicate if reconnecting arcs is needed after offsetting
    /// otherwise, a "soup of unordered arcs" is returned
    pub reconnect: bool,
    /// Flag to enable writing in svg original polyline
    pub svg_orig: bool,
    /// Flag to enable writing in svg raw offsets
    pub svg_raw: bool,
    /// Flag to enable writing in svg connect offsets
    pub svg_connect: bool,
    /// Flag to enable writing in svg split offsets
    pub svg_split: bool,
    /// Flag to enable writing in svg pruned offsets
    pub svg_prune: bool,
    /// Flag to enable writing in svg remove bridge offsets
    pub svg_remove_bridges: bool,
    /// Flag to enable writing in svg final offsets
    pub svg_final: bool,
}

impl Default for OffsetCfg<'_> {
    fn default() -> Self {
        OffsetCfg {
            svg: None,
            reconnect: true,
            svg_orig: false,
            svg_raw: false,
            svg_connect: false,
            svg_split: false,
            svg_prune: false,
            svg_remove_bridges: false,
            svg_final: false,
        }
    }
}

/// Computes the offset of a Polyline and returns result as multiple Polylines.
///
/// This is the main entry point for polyline offsetting. It takes an input polyline,
/// applies the specified offset distance, and returns a vector of output polylines.
/// It is expected that the Polyline is a closed shape.
///
/// # Arguments
///
/// * `poly` - The input polyline to offset. Should be a sequence of connected PVertex-es.
/// * `off` - The offset distance. Only positive values offset to the "right" side of the polyline.
/// * `cfg` - Configuration options controlling the offsetting behavior and writing to svg file.
///
/// # Returns
///
/// A vector of polylines representing the offset result. Each Polyline is a sequence of
/// PVertex-es. The number of output polylines depends
/// on the input geometry and offset distance:
/// - Simple cases may produce a single offset polyline
/// - Complex geometries or self-intersecting offsets may produce multiple polylines
/// - Invalid or degenerate cases may produce an empty vector
///
/// # Examples
///
/// ```rust
/// use geom::prelude::*;
/// use offroad::prelude::{OffsetCfg, offset_polyline};
///
/// let mut cfg = OffsetCfg::default();
/// let poly = vec![
///     pvertex(point(0.0, 0.0), 0.0),    // Start point (no arc)
///     pvertex(point(10.0, 0.0), 0.0),   // Line segment
///     pvertex(point(10.0, 10.0), 0.0),  // End point (no arc)
/// ];
///
/// // Offset by 2.0 units
/// let offset_polylines = offset_polyline(&poly, 2.0, &mut cfg);
///
/// println!("Generated {} offset polylines", offset_polylines.len());
/// ```
///
/// # Algorithm Overview
///
/// The offsetting process involves several stages:
/// 1. Convert input polyline to raw offset segments (lines and arcs)
/// 2. Connect adjacent offset segments with transition arcs
/// 3. Split overlapping segments at intersection points
/// 4. Prune invalid segments that are too close to the original
/// 5. Reconnect valid segments into continuous paths
///
/// # Notes
///
/// - The function is intended to handle closed polylines.
/// - Offset direction follows the right-hand rule relative to polyline direction.
pub fn offset_polyline(poly: &Polyline, off: f64, cfg: &mut OffsetCfg) -> Vec<Polyline> {
    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_orig
    {
        svg.polyline(poly, "red");
    }
    let mut offset_arcs = offset_polyline_impl(poly, off, cfg);

    println!(
        "DEBUG: remove_bridge_arcs called with {} arcs",
        offset_arcs.len()
    );

    //remove_bridge_arcs(&mut offset_arcs);

    for arc in &offset_arcs {
        println!(
            "DEBUG: Offset arc: a=({:.2}, {:.2}), b=({:.2}, {:.2}), r={:.2}, id={}",
            arc.a.x, arc.a.y, arc.b.x, arc.b.y, arc.r, arc.id
        );
    }

    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_remove_bridges
    {
        svg.arcline(&offset_arcs, "violet");
        svg.arcline_single_points(&offset_arcs, "green");
    }

    // find_middle_points(&mut offset_arcs);

    // Always reconnect arcs
    let reconnect_arcs = offset_reconnect_arcs(&mut offset_arcs);
    // println!(
    //     "DEBUG: offset_reconnect_arcs returned {} components",
    //     reconnect_arcs.len()
    // );
    // for (i, component) in reconnect_arcs.iter().enumerate() {
    //     println!("DEBUG: Component {}: {} arcs", i, component.len());
    // }

    let final_poly = arclines_to_polylines(&reconnect_arcs);

    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_final
    {
        svg.polylines(&final_poly, "violet");
    }

    final_poly
}

/// Computes the offset of an Arcline and returns result as multiple Arcline-s.
///
/// This function is similar to `offset_polyline` but operates on arclines
/// (sequences of arcs).
/// It is expected that the Arcline is a closed shape.
///
/// # Arguments
///
/// * `arcs` - The input arcline (sequence of arcs) to offset.
/// * `off` - The offset distance. Positive values only, to the "right" side of the Arcline
///   direction.
/// * `cfg` - Configuration options controlling the offsetting behavior and writing to svg file.
///
/// # Returns
///
/// The number of output arclines depends on the input geometry and offset distance:
/// - Simple cases may produce a single offset arcline
/// - Complex geometries or self-intersecting offsets may produce multiple arclines
/// - Invalid or degenerate cases may produce an empty vector
///
/// # Examples
///
/// ```rust
/// use geom::prelude::*;
/// use offroad::prelude::{OffsetCfg, offset_arcline};
///
/// let mut cfg = OffsetCfg::default();
///
/// // Create a simple arcline with line segments
/// let arcline = vec![
///     arcseg(point(0.0, 0.0), point(10.0, 0.0)),   // Line segment  
///     arcseg(point(10.0, 0.0), point(10.0, 10.0)), // Another line segment
/// ];
///
/// // Offset by 2.0 units while preserving arc geometry
/// let offset_arclines = offset_arcline(&arcline, 2.0, &mut cfg);
///
/// println!("Generated {} offset arclines", offset_arclines.len());
/// // Each output arcline maintains the original arc properties
/// ```
///
/// # Algorithm Overview
///
/// The offsetting process follows the same stages as polyline offsetting but preserves
/// arc geometry throughout:
/// 1. Convert input arcline to raw offset segments (lines and arcs)
/// 2. Connect adjacent offset segments with transition arcs
/// 3. Split overlapping segments at intersection points
/// 4. Prune invalid segments that are too close to the original
/// 5. Reconnect valid segments into continuous arc-paths
///
pub fn offset_arcline(arcs: &Arcline, off: f64, cfg: &mut OffsetCfg) -> Vec<Arcline> {
    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_orig
    {
        svg.arcline(arcs, "red");
    }
    let mut offset_arcs = offset_arcline_impl(arcs, off, cfg);

    remove_bridge_arcs(&mut offset_arcs);

    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_remove_bridges
    {
        svg.arcline(&offset_arcs, "violet");
        svg.arcline_single_points(&offset_arcs, "green");
    }

    let mut final_arcs = Vec::new();
    if cfg.reconnect {
        final_arcs = offset_reconnect_arcs(&mut offset_arcs);
        println!(
            "offset_reconnect_arcs returned {} components",
            final_arcs.len()
        );
        for (i, component) in final_arcs.iter().enumerate() {
            println!("  Component {}: {} arcs", i, component.len());
        }
    } else {
        final_arcs.push(offset_arcs);
    }

    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_final
    {
        svg.arclines(&final_arcs, "violet");
    }

    final_arcs
}

fn offset_polyline_impl(poly: &Polyline, off: f64, cfg: &mut OffsetCfg) -> Vec<Arc> {
    let plines = vec![poly.clone()];
    let poly_raws = polyline_to_raws(&plines);
    offset_algo(&poly_raws, off, cfg)
}

fn offset_arcline_impl(arcs: &Arcline, off: f64, cfg: &mut OffsetCfg) -> Vec<Arc> {
    let alines = vec![arcs.clone()];
    let poly_raws = arclines_to_raws(&alines);
    offset_algo(&poly_raws, off, cfg)
}

#[doc(hidden)]
/// Converts a vector of arcs into a vector of polylines.
#[must_use]
pub fn arclines_to_polylines(reconnect_arcs: &Vec<Arcline>) -> Vec<Polyline> {
    let mut polylines = Vec::with_capacity(reconnect_arcs.len());
    for arcs in reconnect_arcs {
        let polyline = arclines_to_polylines_single(arcs);
        polylines.push(polyline);
    }
    polylines
}

#[doc(hidden)]
/// function to convert from Vec<Arc> to Polyline
/// Note: arcs is a loop of arcs and when converting to PVertex,
/// some Arc can be either "a" to "b" or "b" to "a" oriented
#[must_use]
pub fn arclines_to_polylines_single(arcs: &Arcline) -> Polyline {
    let mut polyline = Vec::new();

    if arcs.is_empty() {
        return polyline;
    }

    println!("DEBUG: Converting {} arcs to polyline", arcs.len());
    for (i, arc) in arcs.iter().enumerate() {
        println!(
            "DEBUG: Arc {}: a=({:.2}, {:.2}), b=({:.2}, {:.2}), r={:.2}",
            i, arc.a.x, arc.a.y, arc.b.x, arc.b.y, arc.r
        );
    }

    // For the first arc, start with its original orientation
    // Convert first arc
    let (start_point, end_point, bulge) = if arcs[0].is_seg() {
        (arcs[0].a, arcs[0].b, 0.0)
    } else {
        let bulge = arc_bulge_from_points(arcs[0].a, arcs[0].b, arcs[0].c, arcs[0].r);
        (arcs[0].a, arcs[0].b, bulge)
    };

    println!(
        "DEBUG: Arc 0 -> polyline vertex: p=({:.2}, {:.2}), bulge={:.6}",
        start_point.x, start_point.y, bulge
    );
    polyline.push(pvertex(start_point, bulge));
    let mut current_end_point = end_point;

    // For subsequent arcs, determine orientation to maintain connectivity
    for i in 1..arcs.len() {
        let arc = &arcs[i];
        let prev_end = current_end_point;

        // Check both possible orientations and choose the one that connects
        let forward_connects = prev_end.close_enough(arc.a, 1e-10);
        let reverse_connects = prev_end.close_enough(arc.b, 1e-10);

        println!(
            "DEBUG: Arc {} connectivity check: prev_end=({:.2}, {:.2}), arc.a=({:.2}, {:.2}), arc.b=({:.2}, {:.2}), forward_connects={}, reverse_connects={}",
            i,
            prev_end.x,
            prev_end.y,
            arc.a.x,
            arc.a.y,
            arc.b.x,
            arc.b.y,
            forward_connects,
            reverse_connects
        );

        let (start_point, end_point, bulge) = if forward_connects {
            // Use arc in forward direction (a -> b)
            if arc.is_seg() {
                (arc.a, arc.b, 0.0)
            } else {
                let bulge = arc_bulge_from_points(arc.a, arc.b, arc.c, arc.r);
                println!("DEBUG: Arc {} forward bulge: {:.6}", i, bulge);
                (arc.a, arc.b, bulge)
            }
        } else if reverse_connects {
            // Use arc in reverse direction (b -> a)
            if arc.is_seg() {
                (arc.b, arc.a, 0.0)
            } else {
                // For reversed arc, we need to negate the bulge
                let forward_bulge = arc_bulge_from_points(arc.a, arc.b, arc.c, arc.r);
                println!(
                    "DEBUG: Arc {} reversed: forward_bulge={:.6}, using={:.6}",
                    i, forward_bulge, -forward_bulge
                );
                (arc.b, arc.a, -forward_bulge)
            }
        } else {
            // No direct connection - this indicates a gap in the arcline
            // For now, use the arc in forward direction and let the gap be visible
            println!(
                "DEBUG: WARNING: Arc {} has no direct connection to previous arc!",
                i
            );
            if arc.is_seg() {
                (arc.a, arc.b, 0.0)
            } else {
                let bulge = arc_bulge_from_points(arc.a, arc.b, arc.c, arc.r);
                (arc.a, arc.b, bulge)
            }
        };

        // Only add vertex if it's different from the previous end point
        if !prev_end.close_enough(start_point, 1e-10) {
            println!(
                "DEBUG: Adding intermediate vertex at ({:.2}, {:.2}) to bridge gap",
                prev_end.x, prev_end.y
            );
            polyline.push(pvertex(prev_end, 0.0)); // Add intermediate vertex with zero bulge
        }

        println!(
            "DEBUG: Arc {} -> polyline vertex: p=({:.2}, {:.2}), bulge={:.6}",
            i, start_point.x, start_point.y, bulge
        );
        polyline.push(pvertex(start_point, bulge));
        current_end_point = end_point;
    }

    println!("DEBUG: Final polyline has {} vertices", polyline.len());
    polyline
}

#[doc(hidden)]
#[must_use]
pub fn polyline_to_raws(plines: &Vec<Polyline>) -> Vec<Vec<OffsetRaw>> {
    let mut varcs: Vec<Vec<OffsetRaw>> = Vec::new();
    for pline in plines {
        let pline = poly_remove_duplicates(pline);
        varcs.push(polyline_to_raws_single(&pline));
    }
    varcs
}

#[doc(hidden)]
#[must_use]
pub fn polyline_to_raws_single(pline: &Polyline) -> Vec<OffsetRaw> {
    let size = pline.len();
    let mut offs = Vec::with_capacity(size + 1);
    for i in 0..size {
        let bulge = pline[i % size].b;
        let arc = arc_circle_parametrization(pline[i % size].p, pline[(i + 1) % size].p, bulge);
        // let check = arc_check(&seg, EPS_COLLAPSED);
        // if !check {
        //     continue;
        // }
        let orig = if bulge < ZERO { arc.a } else { arc.b };
        let off = OffsetRaw {
            arc,
            orig,
            g: bulge,
        };
        offs.push(off);
    }
    offs
}

const EPS_REMOVE_DUPLICATES: f64 = 1e-8;
#[doc(hidden)]
/// Remove consecutive duplicate points from a polyline
#[must_use]
pub fn poly_remove_duplicates(pline: &Polyline) -> Polyline {
    if pline.len() < 2 {
        return pline.clone();
    }

    let mut res: Polyline = Vec::new();
    let size = pline.len();

    let mut old_p: Option<Point> = None;
    let mut flag = false;

    for i in 0..size {
        let cur = i;
        let next = (i + 1) % size;
        let p1 = pline[cur].p;
        let p2 = pline[next].p;
        if p1.close_enough(p2, EPS_REMOVE_DUPLICATES) {
            old_p = Some(p1);
            flag = true;
        } else {
            if flag {
                // If we had a duplicate, add the last unique point
                if let Some(point) = old_p {
                    res.push(pvertex(point, pline[cur].b));
                }
                flag = false;
            } else {
                // If no duplicates, just add the current point
                res.push(pline[cur]);
            }
        }
    }
    res
}

#[must_use]
pub fn arclines_to_raws(arcss: &Vec<Arcline>) -> Vec<Vec<OffsetRaw>> {
    let mut varcs: Vec<Vec<OffsetRaw>> = Vec::new();
    for arcs in arcss {
        varcs.push(arcs_to_raws_single(arcs));
    }
    varcs
}

const EPS_COLLAPSED: f64 = 1E-8; // TODO: what should be the exact value.
#[must_use]
pub fn arcs_to_raws_single(arcs: &Arcline) -> Vec<OffsetRaw> {
    let mut offs = Vec::with_capacity(arcs.len());

    for arc in arcs {
        // let check = arc_check(&seg, EPS_COLLAPSED);
        // if !check {
        //     continue;
        // }
        let bulge = arc_bulge_from_points(arc.a, arc.b, arc.c, arc.r);
        let orig = if bulge < ZERO { arc.a } else { arc.b };
        let off = OffsetRaw {
            arc: *arc,
            orig,
            g: bulge,
        };
        offs.push(off);
    }

    offs
}

#[cfg(test)]
mod test_arcs_to_polylines {
    use super::*;

    #[test]
    fn test_arcs_to_polylines_single_simple_loop() {
        // Create a simple test case with a loop of arcs
        let arcs = vec![
            // First arc: from (0,0) to (1,0) - line segment
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            // Second arc: from (1,0) to (0,1) - quarter circle
            arc_circle_parametrization(point(1.0, 0.0), point(0.0, 1.0), 1.0),
            // Third arc: from (0,1) to (0,0) - line segment (completing the loop)
            arcseg(point(0.0, 1.0), point(0.0, 0.0)),
        ];

        // Convert to polyline
        let polyline = arclines_to_polylines_single(&arcs);

        // Should have 3 vertices (one for each arc start point)
        assert_eq!(polyline.len(), 3);

        // First vertex should be at (0,0) with bulge 0 (line segment)
        assert_eq!(polyline[0].p, point(0.0, 0.0));
        assert_eq!(polyline[0].b, 0.0);

        // Second vertex should be at (1,0) with positive bulge (arc)
        assert_eq!(polyline[1].p, point(1.0, 0.0));
        assert!(polyline[1].b > 0.0); // Should be positive for CCW arc

        // Third vertex should be at (0,1) with bulge 0 (line segment)
        assert_eq!(polyline[2].p, point(0.0, 1.0));
        assert_eq!(polyline[2].b, 0.0);
    }

    #[test]
    fn test_arcs_to_polylines_single_mixed_orientation() {
        // Test with mixed arc orientations that need correction
        let arcs = vec![
            // First arc: from (0,0) to (1,0) - line segment
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            // Second arc: reversed orientation (from (0,1) to (1,0) instead of (1,0) to (0,1))
            // This should be detected and corrected
            arc_circle_parametrization(point(0.0, 1.0), point(1.0, 0.0), 1.0),
        ];

        // Convert to polyline
        let polyline = arclines_to_polylines_single(&arcs);

        // Should have 2 vertices
        assert_eq!(polyline.len(), 2);

        // First vertex should be at (0,0) with bulge 0 (line segment)
        assert_eq!(polyline[0].p, point(0.0, 0.0));
        assert_eq!(polyline[0].b, 0.0);

        // Second vertex should be corrected - the arc should be reversed to maintain continuity
        // Since the arc doesn't connect to (1,0) -> (0,1), it should be used as (1,0) -> (0,1)
        assert_eq!(polyline[1].p, point(1.0, 0.0));
        // The bulge should be negative since we're using the reversed arc
        assert!(polyline[1].b < 0.0);
    }

    #[test]
    fn test_arcs_to_polylines_single_empty() {
        let arcs = vec![];
        let polyline = arclines_to_polylines_single(&arcs);
        assert_eq!(polyline.len(), 0);
    }

    #[test]
    fn test_arcs_to_polylines_single_single_arc() {
        let arcs = vec![arcseg(point(0.0, 0.0), point(1.0, 0.0))];

        let polyline = arclines_to_polylines_single(&arcs);

        // Should have 1 vertex
        assert_eq!(polyline.len(), 1);
        assert_eq!(polyline[0].p, point(0.0, 0.0));
        assert_eq!(polyline[0].b, 0.0);
    }
}

#[doc(hidden)]
pub fn offset_polyline_multiple(
    poly: &Polyline,
    step: f64,
    start: f64,
    end: f64,
    config: &mut OffsetCfg,
) -> Vec<Polyline> {
    let mut off = start;
    let mut polylines = Vec::new();
    while off < end {
        let offset_polylines = offset_polyline(poly, off, config);
        polylines.extend(offset_polylines);
        off += step;
    }
    polylines
}

#[doc(hidden)]
/// Offset algorithm:
/// 1. Compute the raw offset polylines
/// 2. Connect the offset segments
/// 3. Split the arcs
/// 4. Prune invalid segments
pub fn offset_algo(raws: &Vec<Vec<OffsetRaw>>, off: f64, cfg: &mut OffsetCfg) -> Vec<Arc> {
    // TODO: What to return on off == 0.0
    if off < ZERO {
        // Negative offset
        return vec![];
    }
    let offset_raw = offset_segments_raws(raws, off);
    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_raw
    {
        svg_offset_raws(svg, &offset_raw, "blue");
    }

    let offset_connect = offset_connect_raw(&offset_raw, off);
    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_connect
    {
        svg.arclines(&offset_connect, "violet");
    }

    let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);
    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_split
    {
        svg.arcline(&offset_split, "violet");
        svg.arcline_single_points(&offset_split, "violet");
    }

    let offset_prune = offset_prune_invalid(raws, &mut offset_split, off);

    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_prune
    {
        svg.arcline(&offset_prune, "violet");
        svg.arcline_single_points(&offset_prune, "violet");
    }
    offset_prune
}

#[doc(hidden)]
pub fn svg_offset_raws(svg: &mut SVG, offset_raws: &Vec<Vec<OffsetRaw>>, color: &str) {
    for raw in offset_raws {
        for seg in raw {
            if seg.arc.is_seg() {
                let mut segment = segment(seg.arc.a, seg.arc.b);
                segment.id(seg.arc.id);
                svg.segment(&segment, color);
            } else {
                svg.arc(&seg.arc, color);
            }
        }
    }
}

//     #[test]
//     #[ignore = "svg output"]
//     fn test_connect_offset_segments2() {
//         let pline = vec![
//             pvertex(point(100.0, 100.0), 1.5),
//             pvertex(point(100.0, 160.0), ZERO),
//             pvertex(point(120.0, 200.0), ZERO),
//             pvertex(point(128.0, 192.0), ZERO),
//             pvertex(point(128.0, 205.0), ZERO),
//             pvertex(point(136.0, 197.0), ZERO),
//             pvertex(point(136.0, 250.0), ZERO),
//             pvertex(point(0.0, 250.0), ZERO),
//             pvertex(point(0.0, 100.0), ZERO),
//         ];
//         let off = 10.0;
//         let mut svg = svg(300.0, 300.0);
//         svg.polyline(&pline, "red");
//         let pline_offset = offset_polyline_raw(&pline, off);
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let pline_offset2 = offset_connect_segments(&oarc, &pline_offset, off);
//         svg.offset_segments(&pline_offset2, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_connect_offset_segments3() {
//         let pline = vec![
//             pvertex(point(100.0, 100.0), 1.5),
//             pvertex(point(100.0, 160.0), ZERO),
//             pvertex(point(120.0, 200.0), ZERO),
//             pvertex(point(128.0, 192.0), ZERO),
//             pvertex(point(128.0, 205.0), ZERO),
//             pvertex(point(136.0, 197.0), ZERO),
//             pvertex(point(136.0, 250.0), ZERO),
//             pvertex(point(110.0, 250.0), -1.0), // zero radius after offset
//             pvertex(point(78.0, 250.0), ZERO),
//             pvertex(point(50.0, 250.0), -1.0), // zero radius after offset
//             pvertex(point(38.0, 250.0), ZERO),
//             pvertex(point(-52.0, 250.0), ZERO),
//             //pvertex(point(-52.0, 150.0), -1.0),
//             pvertex(
//                 point(-18.499999999999986, 208.0237020535574),
//                 -0.5773502691896256,
//             ),
//             pvertex(point(82.0, 150.0), 0f64),
//             pvertex(point(50.0, 150.0), 1.0),
//             pvertex(point(-20.0, 150.0), ZERO),
//             pvertex(point(0.0, 100.0), ZERO),
//         ];
//         let off = 16.0;
//         let mut svg = svg(300.0, 300.0);
//         svg.polyline(&pline, "red");
//         let pline_offset = offset_polyline_raw(&pline, off);
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let pline_offset2 = offset_connect_segments(&oarc, &pline_offset, off);
//         svg.offset_segments(&pline_offset2, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "Generate coordinate for tests"]
//     fn test_connect_offset_segments4() {
//         // Calculates parametric point in the arc
//         let arc = arc_circle_parametrization(point(50.0, 150.0), point(-20.0, 150.0), 1.0);
//         let offset = arc_offset(pvertex(arc.a, 1.0), pvertex(arc.b, 1.0), 32.0);

//         let theta = 125_f64.to_radians();
//         let nr = arc.r + 32.0;
//         let x = arc.c.x + nr * theta.cos();
//         let y = arc.c.y + nr * theta.sin();
//         println!("{} {}", x, y);
//         let g = arc_g_from_points(offset.arc.a, point(x, y), offset.arc.c, offset.arc.r);
//         println!("{}", g);
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_resolve_self_intersect() {
//         let pline = vec![
//             pvertex(point(100.0, 100.0), 1.5),
//             pvertex(point(100.0, 160.0), ZERO),
//             pvertex(point(120.0, 200.0), ZERO),
//             pvertex(point(128.0, 192.0), ZERO),
//             pvertex(point(128.0, 205.0), ZERO),
//             pvertex(point(136.0, 197.0), ZERO),
//             pvertex(point(136.0, 250.0), ZERO),
//             pvertex(point(110.0, 250.0), -1.0), // zero radius after offset
//             pvertex(point(78.0, 250.0), ZERO),
//             pvertex(point(50.0, 250.0), -1.0), // zero radius after offset
//             pvertex(point(38.0, 250.0), ZERO),
//             pvertex(point(0.0001, 250.0), 1000000.0), // almost circle
//             pvertex(point(0.0, 250.0), ZERO),
//             pvertex(point(-52.0, 250.0), ZERO),
//             //pvertex(point(-52.0, 150.0), -1.0),
//             pvertex(
//                 point(-23.429621235520095, 204.88318696736243),
//                 -0.6068148963145962,
//             ),
//             pvertex(point(82.0, 150.0), 0f64),
//             pvertex(point(50.0, 150.0), 1.0),
//             pvertex(point(-20.0, 150.0), ZERO),
//             pvertex(point(0.0, 100.0), ZERO),
//         ];
//         let off = 16.0;
//         let mut svg = svg(300.0, 350.0);
//         svg.polyline(&pline, "red");

//         let pline_offset = offset_polyline_raw(&pline, off);
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut pline_offset2 = offset_connect_segments(&oarc, &pline_offset, off);
//         offset_remove_degenerate(&mut pline_offset2);
//         offset_resolve_self_intersect(&mut pline_offset2);

//         svg.offset_segments(&pline_offset2, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_resolve_self_intersect02() {
//         // triangle
//         let pline = vec![
//             pvertex(point(150.0, 50.0), ZERO),
//             pvertex(point(100.0, 100.0), ZERO),
//             pvertex(point(50.0, 50.0), ZERO),
//         ];
//         let off = 16.0;
//         let mut svg = svg(300.0, 350.0);
//         svg.polyline(&pline, "red");

//         let pline_offset = offset_polyline_raw(&pline, off);
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut pline_offset2 = offset_connect_segments(&oarc, &pline_offset, off);
//         offset_remove_degenerate(&mut pline_offset2);
//         offset_resolve_self_intersect(&mut pline_offset2);

//         svg.offset_segments(&pline_offset2, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_resolve_self_intersect03() {
//         // two lines and arc inside offset
//         let pline = vec![
//             pvertex(point(150.0, 50.0), ZERO),
//             pvertex(point(100.0, 100.0), -1.0),
//             pvertex(point(150.0, 150.0), ZERO),
//         ];
//         let off = 15.0;
//         let mut svg = svg(300.0, 350.0);
//         svg.polyline(&pline, "red");

//         let pline_offset = offset_polyline_raw(&pline, off);
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut pline_offset2 = offset_connect_segments(&oarc, &pline_offset, off);
//         offset_remove_degenerate(&mut pline_offset2);
//         offset_resolve_self_intersect(&mut pline_offset2);
//         offset_remove_invalid_offsets(&mut pline_offset2, &oarc, off);

//         svg.offset_segments(&pline_offset2, "black");
//         svg.write();
//     }

//     #[test]
//     fn test_almost_circle() {
//         let mut svg = svg(300.0, 350.0);
//         let pline = vec![
//             pvertex(point(38.0, 250.0), ZERO),
//             pvertex(point(0.0001, 250.0), 1000000.0), // almost circle
//             pvertex(point(0.0, 250.0), ZERO),
//             pvertex(point(-52.0, 250.0), ZERO),
//             pvertex(point(-7.0, 150.0), ZERO),
//         ];
//         svg.polyline(&pline, "red");
//         let off = 16.0;
//         let pline_offset = offset_polyline_raw(&pline, off);
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut pline_offset2 = offset_connect_segments(&oarc, &pline_offset, off);
//         offset_remove_degenerate(&mut pline_offset2);
//         offset_resolve_self_intersect(&mut pline_offset2);

//         svg.offset_segments(&pline_offset2, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_circle_with_two_close_lines() {
//         let mut svg = svg(300.0, 350.0);
//         let bul = 1.6;
//         let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(100.0, 160.0), bul);
//         let arc1 = arc_circle_parametrization(point(90.0, 130.0), point(150.0, 190.0), 0f64);
//         let arc2 = arc_circle_parametrization(point(94.0, 130.0), point(154.0, 190.0), 0f64);
//         let mut offsets = vec![arc0, arc1, arc2];
//         svg.offset_segments(&offsets, "red");
//         let arc = arc_circle_parametrization(point(100.0, 100.0), point(100.0, 160.0), bul);
//         svg.circle(&circle(arc.c, 0.5), "red");

//         offset_resolve_self_intersect(&mut offsets);

//         svg.offset_segments(&offsets, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_two_circles_one_point() {
//         let mut svg = svg(300.0, 350.0);
//         let bul = -1.6;
//         let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 100.0), bul);
//         let arc1 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 100.0), bul);
//         let mut offsets = vec![arc0, arc1];
//         svg.offset_segments(&offsets, "red");
//         //let arc = arc_circle_parametrization(point(100.0, 100.0), point(100.0, 160.0), bul);
//         offset_resolve_self_intersect(&mut offsets);
//         svg.offset_segments(&offsets, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_zero_offset_arc() {
//         let mut svg = svg(300.0, 350.0);

//         let off = 80.0;
//         let pline = vec![
//             pvertex(point(78.0, 250.0), ZERO),
//             pvertex(point(50.0, 250.0), -1.0), // zero radius after offset
//             pvertex(point(38.0, 250.0), ZERO),
//             pvertex(point(0.0, 250.0), ZERO),
//             pvertex(point(44.0, 200.0), ZERO),
//         ];
//         svg.polyline(&pline, "red");

//         let pline_offset = offset_polyline_raw(&pline, off);
//         svg.offset_raws(&pline_offset, "black");
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         //let mut pline_offset2 = offset_connect_segments(&oarc, &pline_offset, off);
//         //svg.offset_segments(&pline_offset2, "black");
//         //offset_remove_degenerate(&mut pline_offset2);
//         //offset_resolve_self_intersect(&mut pline_offset2);

//         //svg.offset_segments(&pline_offset2, "black");

//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_two_arcs_two_points() {
//         let mut svg = svg(300.0, 350.0);
//         let p00 = point(100.0, 100.0);
//         let p01 = point(100.0, 200.0);
//         let p10 = point(200.0, 100.0);
//         let p11 = point(200.0, 200.0);
//         let arc0 = arc_circle_parametrization(p00, p01, 1.5);
//         let arc1 = arc_circle_parametrization(p10, p11, -1.5);

//         let mut offset = vec![arc0, arc1];
//         svg.offset_segments(&offset, "red");
//         offset_resolve_self_intersect(&mut offset);
//         svg.offset_segments(&offset, "black");

//         svg.write();
//     }
// }

/// Test polyline for offseting.
/// Has a mix of positive and negative offsets.
#[must_use]
pub fn example_polyline_01() -> Polyline {
    vec![
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
        pvertex(point(0.001, 250.0), 100_000.0), // almost circle
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
    ]
}

/// Test polyline for offseting.
#[must_use]
pub fn example_polyline_02() -> Polyline {
    vec![
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
    ]
}

/// Test polyline for offseting.
#[must_use]
pub fn example_polyline_03() -> Polyline {
    vec![
        pvertex(point(0.0, 0.0), ZERO),
        pvertex(point(200.0, 0.0), ZERO),
        pvertex(point(200.0, 100.0), ZERO),
        pvertex(point(100.0, 100.0), ZERO),
        pvertex(point(100.0, 200.0), ZERO),
        pvertex(point(0.0, 200.0), ZERO),
    ]
}

/// Test polyline for offseting.
#[must_use]
pub fn example_polylines_04() -> Vec<Polyline> {
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
    vec![outer, inner]
}

// #[cfg(test)]
// mod test_remove_invalid_offsets {
//     use std::vec;

//     use crate::{circle::circle, point::point, pvertex::pvertex, svg::svg};

//     use super::*;
//     const ONE: f64 = 1f64;
//     const ZERO: f64 = 0f64;

//     #[test]
//     #[ignore = "svg output"]
//     fn test_remove_invalid_offsets() {
//         let pline = vec![
//             pvertex(point(100.0, 100.0), 1.5),
//             pvertex(point(100.0, 160.0), ZERO),
//             pvertex(point(120.0, 200.0), ZERO),
//             pvertex(point(128.0, 192.0), ZERO),
//             pvertex(point(128.0, 205.0), ZERO),
//             pvertex(point(136.0, 197.0), ZERO),
//             pvertex(point(136.0, 250.0), ZERO),
//             pvertex(point(110.0, 250.0), -1.0), // zero radius after offset
//             pvertex(point(78.0, 250.0), ZERO),
//             pvertex(point(50.0, 250.0), -1.0), // zero radius after offset
//             pvertex(point(38.0, 250.0), ZERO),
//             pvertex(point(0.0001, 250.0), 1000000.0), // almost circle
//             pvertex(point(0.0, 250.0), ZERO),
//             pvertex(point(-52.0, 250.0), ZERO),
//             //pvertex(point(-52.0, 150.0), -1.0),
//             pvertex(
//                 point(-18.499999999999986, 208.0237020535574),
//                 -0.5773502691896256,
//             ),
//             pvertex(point(82.0, 150.0), 0f64),
//             pvertex(point(50.0, 150.0), 1.0),
//             pvertex(point(-20.0, 150.0), ZERO),
//             pvertex(point(0.0, 100.0), ZERO),
//         ];
//         let off = 16.0;
//         let mut svg = svg(300.0, 350.0);
//         svg.polyline(&pline, "red");

//         let offset_raw = offset_polyline_raw(&pline, off);
//         //svg.offset_raws(&offset_raw, "blue");
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut offset = offset_connect_segments(&oarc, &offset_raw, off);
//         offset_remove_degenerate(&mut offset);
//         offset_resolve_self_intersect(&mut offset);
//         offset_remove_degenerate(&mut offset);
//         offset_remove_invalid_offsets(&mut offset, &oarc, off);

//         svg.offset_segments(&offset, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_self_intersect_issue() {
//         let pline = vec![
//             //pvertex(point(100.0, 160.0), ZERO),
//             pvertex(point(120.0, 200.0), ZERO),
//             pvertex(point(128.0, 192.0), ZERO),
//             pvertex(point(128.0, 205.0), ZERO),
//             pvertex(point(136.0, 197.0), ZERO),
//             pvertex(point(136.0, 250.0), ZERO),
//         ];
//         let off = 16.0;
//         let mut svg = svg(300.0, 350.0);
//         svg.polyline(&pline, "red");

//         let offset_raw = offset_polyline_raw(&pline, off);
//         //svg.offset_raws(&offset_raw, "blue");
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut offset = offset_connect_segments(&oarc, &offset_raw, off);
//         offset_remove_degenerate(&mut offset);
//         offset_resolve_self_intersect(&mut offset);
//         offset_remove_degenerate(&mut offset);
//         offset_remove_invalid_offsets(&mut offset, &oarc, off);

//         svg.offset_segments(&offset, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_self_intersect_issue_small() {
//         let mut svg = svg(300.0, 350.0);
//         let mut offs = Vec::new();

//         let arc0 = arcline(point(144.0, 192.0), point(144.0, 205.0));

//         let arc1 = arc(
//             point(124.68629150101523, 185.68629150101523),
//             point(152.0, 197.0),
//             point(136.0, 197.0),
//             16.0,
//         );
//         offs.push(arc0);
//         offs.push(arc1);
//         svg.offset_segments(&offs, "red");

//         offset_resolve_line_arc(&arc0, &arc1, &mut offs, 0, 1);

//         svg.offset_segments(&offs, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_remove_invalid_offsets02() {
//         let pline = pline_01();
//         let off = 16.0;
//         let mut svg = svg(300.0, 350.0);
//         svg.polyline(&pline, "red");

//         let offset_raw = offset_polyline_raw(&pline, off);
//         //svg.offset_raws(&offset_raw, "blue");
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut offset = offset_connect_segments(&oarc, &offset_raw, off);
//         offset_remove_degenerate(&mut offset);
//         offset_resolve_self_intersect(&mut offset);
//         offset_remove_degenerate(&mut offset);
//         offset_remove_invalid_offsets(&mut offset, &oarc, off);

//         svg.offset_segments(&offset, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_parallel_lines() {
//         let pline = vec![
//             pvertex(point(100.0, 100.0), -2.5),
//             pvertex(point(150.0, 200.0), ZERO),
//             pvertex(point(250.0, 200.0), -2.5),
//             pvertex(point(200.0, 100.0), ZERO),
//         ];
//         let off = 50.0;
//         let mut svg = svg(300.0, 350.0);
//         svg.polyline(&pline, "red");

//         let offset_raw = offset_polyline_raw(&pline, off);
//         //svg.offset_raws(&offset_raw, "blue");
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut offset = offset_connect_segments(&oarc, &offset_raw, off);
//         offset_remove_degenerate(&mut offset);
//         offset_resolve_self_intersect(&mut offset);
//         offset_remove_degenerate(&mut offset);
//         offset_remove_invalid_offsets(&mut offset, &oarc, off);

//         svg.offset_segments(&offset, "black");
//         svg.write();
//     }
// }

// #[cfg(test)]
// mod test_remove_invalid_offsets02 {
//     use std::vec;

//     use crate::{
//         circle::{circle, Circle},
//         point::point,
//         pvertex::{polyline_reverse, polyline_scale, polyline_translate, pvertex},
//         svg::svg,
//     };

//     use super::*;
//     const ONE: f64 = 1f64;
//     const ZERO: f64 = 0f64;

//     #[test]
//     #[ignore = "svg output"]
//     fn test_remove_invalid_offsets_too_many_intersections() {
//         let pline = pline_01();
//         let mut svg = svg(300.0, 350.0);

//         let pline = polyline_scale(&pline, 0.5);
//         svg.polyline(&pline, "red");
//         // off is 76
//         let offset = offset_polyline(&pline, (76 as f64) / 2.0);
//         svg.offset_segments(&offset, "black");

//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_remove_invalid_offsets_too_many_intersections2() {
//         let pline = pline_01();
//         let mut svg = svg(300.0, 350.0);

//         let pline = polyline_scale(&pline, 0.5);
//         let pline = polyline_translate(&pline, point(0.0, 100.0));
//         svg.polyline(&pline, "red");
//         // 265
//         println!("off:{}", 265);
//         let offset = offset_polyline(&pline, (265 as f64) / 4.0);
//         svg.offset_segments(&offset, "black");

//         let cp = offset_center_point(&offset);
//         let c0 = circle(cp, 1.0);
//         svg.circle(&c0, "purple");

//         svg.write();
//     }

//     #[test]
//     //#[ignore = "svg output"]
//     fn test_remove_invalid_offsets_too_many_intersections3() {
//         let pline = pline_01();
//         let mut svg = svg(300.0, 400.0);

//         svg.polyline(&pline, "red");
//         // 128

//         println!("off:{}", 128);
//         let offset = offset_polyline(&pline, (128 as f64) / 8.0);
//         svg.offset_segments(&offset, "black");

//         let cp = offset_center_point(&offset);
//         let c0 = circle(cp, 1.0);
//         svg.circle(&c0, "red");
//         svg.write();
//     }

//     #[test]
//     //#[ignore = "svg output"]
//     fn test_remove_invalid_offsets02() {
//         let pline = pline_01();
//         let mut svg = svg(300.0, 400.0);

//         //let pline = polyline_scale(&pline, 0.5);
//         //let pline = polyline_translate(&pline, point(0.0, 100.0));
//         svg.polyline(&pline, "red");
//         // 128
//         // for off in (0..2000).step_by(4) {
//         //     println!("off:{}", off);
//         //     let offset = offset_polyline(&pline, (off as f64) / 16.0);
//         //     svg.offset_segments(&offset, "black");
//         // }

//         let pline = polyline_reverse(&pline);
//         svg.polyline(&pline, "red");
//         // 32, 20
//         for off in (0..2000).step_by(4) {
//             println!("off:{}", off);
//             let offset = offset_polyline(&pline, (off as f64) / 16.0);
//             svg.offset_segments(&offset, "black");
//         }

//         svg.write();
//     }

//     #[test]
//     //#[ignore = "svg output"]
//     fn test_tree_repeating_arcs() {
//         let p0 = point(114.96042364631987, 163.00791527073602);
//         let p1 = point(7.8492830595646268, 159.14213389567942);
//         let p2 = point(101.74318153326465, 148.55518203471888);
//         let p3 = point(106.56759783189042, 184.48433521948056);
//         let pc = point(60.0, 200.0);
//         let r = 66.25;

//         let arc0 = arc(p0, p1, pc, r);
//         let arc1 = arc(p2, p0, pc, r);
//         let arc2 = arc(p1, p2, p1, r);
//         let arc3 = arc(p2, p3, point(41.0, 175.0), 66.25);

//         let mut svg = svg(300.0, 400.0);
//         svg.arc(&arc0, "red");
//         svg.arc(&arc1, "green");
//         svg.arc(&arc2, "blue");
//         svg.arc(&arc3, "purple");
//         svg.write();
//     }

//     #[test]
//     //#[ignore = "svg output"]
//     fn test_remove_missing_offset01() {
//         let pline = pline_01();
//         let mut svg = svg(400.0, 600.0);
//         let pline = polyline_translate(&pline, point(0.0, 110.0));
//         svg.polyline(&pline, "red");
//         // 100, 200

//         //let pline = polyline_reverse(&pline);
//         let off: f64 = 52.25;
//         let offset_raw = offset_polyline_raw(&pline, off);
//         //svg.offset_raws(&offset_raw, "blue");
//         // convert polyline to offsetsegments
//         let oarc = offset_pline_to_offsetsegments(&pline);
//         let mut offset = offset_connect_segments(&oarc, &offset_raw, off);
//         offset_remove_degenerate(&mut offset);
//         offset_resolve_self_intersect(&mut offset);
//         offset_remove_degenerate(&mut offset);
//         offset_remove_invalid_offsets(&mut offset, &oarc, off);

//         svg.offset_segments(&offset, "black");

//         //svg.offset_raws(&offset_raw, "black");
//         svg.write();
//     }

// }

#[doc(hidden)]
pub fn polyline_to_arcs(plines: &Vec<Polyline>) -> Vec<Vec<Arc>> {
    let mut varcs: Vec<Vec<Arc>> = Vec::new();
    for pline in plines {
        varcs.push(polyline_to_arcs_single(pline));
    }
    varcs
}

#[doc(hidden)]
fn polyline_to_arcs_single(pline: &Polyline) -> Vec<Arc> {
    let size = pline.len();
    let mut arcs = Vec::with_capacity(size - 1);
    for i in 0..size {
        let p1 = pline[i % size].p;
        let p2 = pline[(i + 1) % size].p;
        let arc = arc_circle_parametrization(p1, p2, pline[i].b);
        arcs.push(arc);
    }
    arcs
}

#[cfg(test)]
mod test_offset {

    // use geom::prelude::*;

    const ZERO: f64 = 0f64;
    // #[test]
    // #[ignore = "svg output"]
    // fn test_self_intersect_issue() {
    //     let pline = vec![vec![
    //         pvertex(point(100.0, 160.0), ZERO),
    //         pvertex(point(120.0, 200.0), ZERO),
    //         pvertex(point(128.0, 192.0), ZERO),
    //         pvertex(point(128.0, 205.0), ZERO),
    //         pvertex(point(136.0, 197.0), ZERO),
    //         pvertex(point(136.0, 250.0), ZERO),
    //     ]];
    //     let pliner = polylines_reverse(&pline);
    //     let poly_raws = poly_to_raws(&pliner);
    //     let mut svg = svg(300.0, 350.0);
    //     svg_offset_raws(&mut svg, &poly_raws, "black");

    //     let off = 5.0;

    //     let offset_raw = offset_polyline_raw(&poly_raws, off);
    //     svg_offset_raws(&mut svg, &offset_raw, "blue");

    //     let offset_connect = offset_connect_raw(&offset_raw, off);
    //     svg.arclines(&offset_connect, "violet");

    //     let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);
    //     //svg.offset_segments_single(&offset_split, "violet");

    //     let offset_final = offset_prune_invalid(&poly_raws, &mut offset_split, off);
    //     svg.arcline(&offset_final, "black");

    //     svg.write();
    // }

    // #[test]
    // #[ignore = "svg output"]
    // fn test_offset_complex_polyline() {
    //     //let plines = polylines_reverse(&pline_01());
    //     let plines = &pline_01();
    //     let poly_raws = poly_to_raws(&plines);
    //     let mut svg = svg(300.0, 350.0);
    //     svg_offset_raws(&mut svg, &poly_raws, "red");

    //     let off = 16.0;

    //     let offset_raw = offset_polyline_raw(&poly_raws, off);
    //     // svg.offset_raws(&offset_raw, "blue");

    //     let offset_connect = offset_connect_raw(&offset_raw, off);
    //     // svg.offset_segments(&offset_connect, "violet");

    //     let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);
    //     // svg.offset_segments_single(&offset_split, "violet");

    //     let offset_final = offset_prune_invalid(&poly_raws, &mut offset_split, off);
    //     svg.arcline(&offset_final, "black");

    //     svg.write();
    // }

    // #[test]
    // #[ignore = "svg output"]
    // fn test_offset_multiple_complex_polyline() {
    //     let mut p = pline_01()[0].clone();
    //     p = polyline_translate(&p, point(180.0, -60.0));
    //     p = polyline_scale(&p, 2.5);

    //     let mut cfg = crate::offset::OffsetCfg::default();
    //     let mut svg = svg(500.0, 400.0);
    //     cfg.svg = Some(&mut svg);
    //     let _ = offset_polyline_multiple(&p, 1.0, 1.0, 100.0, &mut cfg);
    // }

    // #[test]
    // #[ignore = "svg output"]
    // fn test_offset_complex_polyline_2() {
    //     let p = pline_02();
    //     let p2 = polyline_translate(&p, point(50.0, 50.0));
    //     let mut cfg = crate::offset::OffsetCfg::default();
    //     let mut svg = svg(250.0, 350.0);
    //     cfg.svg = Some(&mut svg);
    //     let _ = offset_polyline_multiple(&p2, 1.0, 1.0, 100.0, &mut cfg);
    // }

    // #[test]
    // #[ignore = "svg output"]
    // fn test_offset_arcs_issue() {
    //     let p = vec![
    //         pvertex(point(50.0, 50.0), 0.2),
    //         pvertex(point(100.0, 50.0), -0.5),
    //         pvertex(point(100.0, 100.0), 0.2),
    //         pvertex(point(50.0, 100.0), -0.5),
    //     ];
    //     let mut plines = Vec::new();
    //     plines.push(p.clone());

    //     let p2 = polyline_translate(&p, point(50.0, 30.0));

    //     let mut cfg = crate::offset::OffsetCfg::default();
    //     let mut svg = svg(250.0, 350.0);
    //     cfg.svg = Some(&mut svg);
    //     let _ = offset_polyline_multiple(&p2, 1.0, 1.0, 100.0, &mut cfg);
    // }

    // #[test]
    // #[ignore = "svg output"]
    // fn test_offset_04() {
    //     let pline1 = pline_04();
    //     let p_outer = polyline_reverse(&pline1[0].clone());
    //     //let p_inner = polyline_reverse(&pline1[1].clone());

    //     let mut cfg = crate::offset::OffsetCfg::default();
    //     let mut svg = svg(250.0, 350.0);
    //     cfg.svg = Some(&mut svg);
    //     let _ = offset_polyline_multiple(&p_outer, 1.0, 1.0, 100.0, &mut cfg);
    // }

    // #[test]
    // #[ignore = "svg output"]
    // fn test_offset_new_connect() {
    //     let plines = pline_03();
    //     let poly_raws = poly_to_raws(&plines);
    //     let mut svg = svg(250.0, 350.0);
    //     svg_offset_raws(&mut svg, &poly_raws, "red");

    //     let off = 40.0;

    //     let offset_raw = offset_polyline_raw(&poly_raws, off);
    //     svg_offset_raws(&mut svg, &offset_raw, "blue");

    //     let offset_connect = offset_connect_raw(&offset_raw, off);
    //     svg.arclines(&offset_connect, "violet");

    //     //let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);
    //     //svg.offset_segments_single(&offset_split, "violet");

    //     // let offset_final = offset_prune_invalid_offsets(&poly_raws, &mut offset_split, off);
    //     // svg.offset_segments_single(&offset_final, "black");

    //     svg.write();
    // }

    #[test]
    fn test_offset_complex_line_bug() {
        // let pline = vec![
        //     // pvertex(point(100.0, 100.0), 1.5),
        //     // pvertex(point(100.0, 160.0), ZERO),
        //     // pvertex(point(120.0, 200.0), ZERO),
        //     // pvertex(point(128.0, 192.0), ZERO),
        //     // pvertex(point(128.0, 205.0), ZERO),
        //     // pvertex(point(136.0, 197.0), ZERO),
        //     // pvertex(point(136.0, 245.0), -1.0), // zero radius after offset
        //     // pvertex(point(131.0, 250.0), ZERO),
        //     // pvertex(point(110.0, 250.0), -1.0), // zero radius after offset
        //     // pvertex(point(78.0, 250.0), ZERO),
        //     // pvertex(point(50.0, 250.0), -1.0), // zero radius after offset
        //     // pvertex(point(38.0, 250.0), ZERO),
        //     // pvertex(point(0.001, 250.0), 100000.0), // almost circle
        //     pvertex(point(0.0, 250.0), ZERO),
        //     pvertex(point(-52.0, 250.0), ZERO),
        //     pvertex(
        //         point(-23.429621235520095, 204.88318696736243),
        //         -0.6068148963145962,
        //     ),
        //     pvertex(point(82.0, 150.0), ZERO),
        //     pvertex(point(50.0, 150.0), 1.0),
        //     // pvertex(point(-20.0, 150.0), ZERO),
        //     pvertex(point(0.0, 100.0), ZERO),
        // ];
        // let plines = vec![pline.clone()];
        // let off = 16.0;
        // let mut svg = svg(300.0, 350.0);
        // svg.polyline(&plines[0], "red");

        // let offset_raw = offset_polyline_raw(&plines, off);
        // //svg.offset_raws(&offset_raw, "red");

        // let offset_connect = offset_connect_raw(&offset_raw, off);
        // //svg.offset_segments(&offset_connect, "blue");
        // //svg.circle(&circle(point(-24.166752022892602, 117.33556160015547), 1.0), "red");

        // let mut offset_split = offset_split_arcs(&offset_connect);
        // //svg.offset_segments(&offset_split, "blue");

        // let poly_arcs = poly_to_arcs(&plines);
        // let offset_final = offset_prune_invalid_offsets(&poly_arcs, &mut offset_split, off);
        // svg.offset_segments(&offset_final, "blue");

        // svg.write();
    }

    #[test]
    fn test_offset_complex_line_bug_2() {
        // let pline = vec![
        //     //pvertex(point(100.0, 100.0), 1.5),
        //     pvertex(point(100.0, 160.0), ZERO),
        //     // pvertex(point(120.0, 200.0), ZERO),
        //     // pvertex(point(128.0, 192.0), ZERO),
        //     // pvertex(point(128.0, 205.0), ZERO),
        //     // pvertex(point(136.0, 197.0), ZERO),
        //     // pvertex(point(136.0, 245.0), -1.0), // zero radius after offset
        //     pvertex(point(131.0, 250.0), ZERO),
        //     // pvertex(point(110.0, 250.0), -1.0), // zero radius after offset
        //     //pvertex(point(78.0, 250.0), ZERO),
        //     // pvertex(point(50.0, 250.0), -1.0), // zero radius after offset
        //     //pvertex(point(38.0, 250.0), ZERO),
        //     // pvertex(point(0.001, 250.0), 100000.0), // almost circle
        //     pvertex(point(0.0, 250.0), ZERO),
        //     pvertex(point(-52.0, 250.0), ZERO),
        //     pvertex(
        //         point(-23.429621235520095, 204.88318696736243),
        //         -0.6068148963145962,
        //     ),
        //     pvertex(point(82.0, 150.0), ZERO),
        //     pvertex(point(50.0, 150.0), 1.0),
        //     pvertex(point(-20.0, 150.0), ZERO),
        //     pvertex(point(0.0, 100.0), ZERO),
        // ];
        // let plines = vec![pline.clone()];
        // let off = 16.0;
        // let mut svg = svg(300.0, 350.0);
        // svg.polyline(&plines[0], "red");

        // let offset_raw = offset_polyline_raw(&plines, off);
        // //svg.offset_raws(&offset_raw, "red");

        // let offset_connect = offset_connect_raw(&offset_raw, off);
        // svg.offset_segments(&offset_connect, "blue");
        // svg.circle(
        //     &circle(point(-24.166752022892602, 117.33556160015547), 1.0),
        //     "red",
        // );

        // let mut offset_split = offset_split_arcs(&offset_connect);
        // svg.offset_segments(&offset_split, "blue");

        // // let poly_arcs = poly_to_arcs(&plines);
        // // let offset_final = offset_prune_invalid_offsets(&poly_arcs, &mut offset_split, off);
        // // svg.offset_segments(&offset_final, "blue");

        // svg.write();
    }
}

#[cfg(test)]
mod test_poly_remove_duplicates {
    use super::*;

    #[test]
    fn test_empty_polyline() {
        let pline: Polyline = vec![];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_single_vertex() {
        let pline = vec![pvertex(point(1.0, 2.0), 0.0)];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].p, point(1.0, 2.0));
        assert_eq!(result[0].b, 0.0);
    }

    #[test]
    fn test_no_duplicates() {
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(1.0, 0.0), 0.5),
            pvertex(point(1.0, 1.0), -0.2),
            pvertex(point(0.0, 1.0), 0.0),
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 4);
        // Should be unchanged
        assert_eq!(result[0].p, point(0.0, 0.0));
        assert_eq!(result[1].p, point(1.0, 0.0));
        assert_eq!(result[2].p, point(1.0, 1.0));
        assert_eq!(result[3].p, point(0.0, 1.0));
    }

    #[test]
    fn test_consecutive_duplicates() {
        // Two consecutive vertices with same point (within tolerance)
        let eps = EPS_REMOVE_DUPLICATES / 2.0; // Half the tolerance - should be considered duplicate
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(1.0, 0.0), 0.5),
            pvertex(point(1.0 + eps, 0.0), -0.2), // Very close to previous point
            pvertex(point(0.0, 1.0), 0.0),
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 3); // One vertex should be removed
        assert_eq!(result[0].p, point(0.0, 0.0));
        assert_eq!(result[1].p, point(1.0, 0.0)); // First point should be kept
        assert_eq!(result[2].p, point(0.0, 1.0));
    }

    #[test]
    fn test_exact_duplicates() {
        // Exactly identical consecutive points
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(1.0, 0.0), 0.5),
            pvertex(point(1.0, 0.0), -0.2), // Exact duplicate
            pvertex(point(0.0, 1.0), 0.0),
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 3); // One vertex should be removed
        assert_eq!(result[0].p, point(0.0, 0.0));
        assert_eq!(result[1].p, point(1.0, 0.0)); // First point should be kept
        assert_eq!(result[2].p, point(0.0, 1.0));
    }

    #[test]
    fn test_closing_duplicates() {
        // Last and first vertices are duplicates (polyline closure)
        let eps = EPS_REMOVE_DUPLICATES / 2.0;
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(1.0, 0.0), 0.5),
            pvertex(point(1.0, 1.0), -0.2),
            pvertex(point(eps, eps), 0.0), // Very close to first point
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 3); // Last vertex should be removed
        assert_eq!(result[0].p, point(0.0, 0.0));
        assert_eq!(result[1].p, point(1.0, 0.0));
        assert_eq!(result[2].p, point(1.0, 1.0));
    }

    #[test]
    fn test_multiple_consecutive_duplicates() {
        // Multiple consecutive duplicates
        let eps = EPS_REMOVE_DUPLICATES / 3.0;
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(1.0, 0.0), 0.5),
            pvertex(point(1.0 + eps, eps), 0.1), // Close to previous
            pvertex(point(1.0 - eps, -eps), -0.2), // Close to previous
            pvertex(point(0.0, 1.0), 0.0),
        ];
        let result = poly_remove_duplicates(&pline);
        // Should remove duplicates iteratively
        assert!(result.len() == 3);
        assert_eq!(result[1].b, -0.2);
    }

    #[test]
    fn test_just_outside_tolerance() {
        // Points just outside the tolerance - should NOT be removed
        let eps = EPS_REMOVE_DUPLICATES * 2.0; // Double the tolerance
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(1.0, 0.0), 0.5),
            pvertex(point(1.0 + eps, 0.0), -0.2), // Just outside tolerance
            pvertex(point(0.0, 1.0), 0.0),
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 4); // No vertices should be removed
        assert_eq!(result[0].p, point(0.0, 0.0));
        assert_eq!(result[1].p, point(1.0, 0.0));
        assert_eq!(result[2].p, point(1.0 + eps, 0.0));
        assert_eq!(result[3].p, point(0.0, 1.0));
    }

    #[test]
    fn test_preserves_bulge_values() {
        // Ensure bulge values are preserved correctly when removing duplicates
        let eps = EPS_REMOVE_DUPLICATES / 2.0;
        let pline = vec![
            pvertex(point(0.0, 0.0), 1.5),
            pvertex(point(1.0, 0.0), 0.5),
            pvertex(point(1.0 + eps, 0.0), -0.8), // Duplicate point, different bulge
            pvertex(point(0.0, 1.0), -2.0),
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].b, 1.5);
        assert_eq!(result[1].b, -0.8); // Original bulge should be kept
        assert_eq!(result[2].b, -2.0);
    }

    #[test]
    fn test_all_duplicates_except_one() {
        // All vertices are duplicates of the first one
        let eps = EPS_REMOVE_DUPLICATES / 3.0;
        let pline = vec![
            pvertex(point(5.0, 5.0), 0.0),
            pvertex(point(5.0 + eps, 5.0), 0.5),
            pvertex(point(5.0, 5.0 + eps), -0.2),
            pvertex(point(5.0 - eps, 5.0 - eps), 1.0),
        ];
        let result = poly_remove_duplicates(&pline);
        assert!(result.len() == 0);
    }

    #[test]
    fn test_alternating_close_points() {
        // Points that are close to non-consecutive vertices (should not be removed)
        let eps = EPS_REMOVE_DUPLICATES / 2.0;
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(10.0, 0.0), 0.5),
            pvertex(point(eps, eps), -0.2), // Close to first, but not consecutive
            pvertex(point(10.0 + eps, eps), 0.3), // Close to second, but not consecutive
        ];
        let result = poly_remove_duplicates(&pline);
        // Only consecutive duplicates should be removed
        assert_eq!(result.len(), 4); // No removals as these aren't consecutive
    }

    #[test]
    fn test_very_small_coordinates() {
        // Test with very small coordinate values
        let tiny = 1e-12;
        let pline = vec![
            pvertex(point(tiny, tiny), 0.0),
            pvertex(point(tiny * 2.0, tiny), 0.5),
            pvertex(point(tiny * 2.0 + EPS_REMOVE_DUPLICATES / 2.0, tiny), -0.2),
            pvertex(point(tiny, tiny * 3.0), 0.0),
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_large_coordinates() {
        // Test with large coordinate values
        let large = 1e6;
        let eps = EPS_REMOVE_DUPLICATES / 2.0;
        let pline = vec![
            pvertex(point(large, large), 0.0),
            pvertex(point(large + 100.0, large), 0.5),
            pvertex(point(large + 100.0 + eps, large), -0.2), // Duplicate
            pvertex(point(large, large + 100.0), 0.0),
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 3); // One duplicate should be removed
    }

    #[test]
    fn test_triangle_with_duplicate_vertex() {
        // Realistic triangle case with one duplicate vertex
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(3.0, 0.0), 0.0),
            pvertex(point(3.0, 0.0), 0.0), // Exact duplicate
            pvertex(point(1.5, 2.6), 0.0), // Triangle apex
        ];
        let result = poly_remove_duplicates(&pline);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].p, point(0.0, 0.0));
        assert_eq!(result[1].p, point(3.0, 0.0));
        assert_eq!(result[2].p, point(1.5, 2.6));
    }

    #[test]
    fn test_square_with_mixed_duplicates() {
        // Square with some duplicate vertices
        let eps = EPS_REMOVE_DUPLICATES / 2.0;
        let pline = vec![
            pvertex(point(0.0, 0.0), 0.0),
            pvertex(point(1.0, 0.0), 0.0),
            pvertex(point(1.0 + eps, eps), 0.0), // Duplicate of corner
            pvertex(point(1.0, 1.0), 0.0),
            pvertex(point(0.0, 1.0), 0.0),
            pvertex(point(eps, 1.0 + eps), 0.0), // Close to previous
        ];
        let result = poly_remove_duplicates(&pline);
        assert!(result.len() < pline.len()); // Some duplicates should be removed
        assert!(result.len() >= 4); // Should have at least the 4 corners of square
    }

    #[test]
    fn test_some_random_pline() {
        let mut cfg = OffsetCfg::default();
        // Prints SVG output to stdout
        let mut svg = SVG::new(1280.0, 640.0, Some("/tmp/polyline.svg"));
        cfg.svg = Some(&mut svg);
        cfg.svg_orig = false;
        //cfg.svg_remove_bridges = true;
        cfg.svg_raw = true;
        cfg.svg_connect = true;

        // Translate to fit in the SVG viewport
        let poly = polyline_translate(&example_polyline_01(), point(550.0, 120.0));
        let arcline = polyline_to_arcs_single(&poly);

        for i in 42..=42 {
            let _offset_polylines = offset_polyline(&poly, i as f64, &mut cfg);
        }

        if let Some(svg) = cfg.svg.as_deref_mut() {
            // Write svg to file
            svg.write_stroke_width(0.2);
        }
    }
}
