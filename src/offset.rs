#![allow(dead_code)]
#![deny(unused_results)]

use togo::prelude::*;

use crate::{
    offset_connect_raw::offset_connect_raw,
    offset_polyline_raw::{self, arcs_to_raws, poly_to_raws},
    offset_prune_invalid::offset_prune_invalid,
    offset_raw::OffsetRaw,
    offset_reconnect_arcs::{offset_reconnect_arcs},
    offset_split_arcs::offset_split_arcs
};

/// Configuration options for offsetting operations.
pub struct OffsetCfg<'a> {
    /// Optional SVG context for rendering
    pub svg: Option<&'a mut SVG>, 
    /// Flag to indicate if reconnecting arcs is needed
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
    /// Flag to enable writing in svg final offsets
    pub svg_final: bool,
}

impl<'a> Default for OffsetCfg<'a> {
    fn default() -> Self {
        OffsetCfg {
            svg: None,
            reconnect: true,
            svg_orig: false,
            svg_raw: false,
            svg_connect: false,
            svg_split: false,
            svg_prune: false,
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
/// use togo::prelude::*;
/// use offroad::prelude::*;
///
/// let mut cfg = OffsetCfg::default();
/// let poly = vec![
///     pvertex(point(0.0, 0.0), 0.0),    // Start point (no arc)
///     pvertex(point(10.0, 0.0), 0.0),   // Line segment
///     pvertex(point(10.0, 10.0), 0.0),  // End point (no arc)
/// ];
///
/// // Offset by 2.0 units
/// let offset_polylines = offset_polyline_to_polyline(&poly, 2.0, &mut cfg);
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
pub fn offset_polyline_to_polyline(
    poly: &Polyline,
    off: f64,
    cfg: &mut OffsetCfg,
) -> Vec<Polyline> {
    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_orig
    {
        svg.polyline(poly, "red");
    }
    let offset_arcs = offset_polyline_to_polyline_impl(poly, off, cfg);    

    // Always reconnect arcs
    let reconnect_arcs = offset_reconnect_arcs(offset_arcs);
    // println!(
    //     "DEBUG: offset_reconnect_arcs returned {} components",
    //     reconnect_arcs.len()
    // );
    // for (i, component) in reconnect_arcs.iter().enumerate() {
    //     println!("DEBUG: Component {}: {} arcs", i, component.len());
    // }

    let final_poly = arcs_to_polylines(&reconnect_arcs);

    if let Some(svg) = cfg.svg.as_deref_mut() {
        if cfg.svg_final {
            svg.polylines(&final_poly, "violet");
        }
    }

    final_poly
}

/// Computes the offset of an Arcline and returns result as multiple Arcline-s.
///
/// This function is similar to `offset_polyline_to_polyline` but operates on arclines
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
/// use togo::prelude::*;
/// use offroad::prelude::*;
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
/// let offset_arclines = offset_arcline_to_arcline(&arcline, 2.0, &mut cfg);
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
pub fn offset_arcline_to_arcline(arcs: &Arcline, off: f64, cfg: &mut OffsetCfg) -> Vec<Arcline> {
    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_orig
    {
        svg.arcline(arcs, "red");
    }
    let offset_arcs = offset_arcline_to_arcline_impl(arcs, off, cfg);

    let mut final_arcs = Vec::new();
    if cfg.reconnect {
        final_arcs = offset_reconnect_arcs(offset_arcs);
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

    if let Some(svg) = cfg.svg.as_deref_mut() {
        if cfg.svg_final {
            svg.arclines(&final_arcs, "violet");
        }
    }

    final_arcs
}

fn offset_polyline_to_polyline_impl(poly: &Polyline, off: f64, cfg: &mut OffsetCfg) -> Vec<Arc> {
    let mut plines = Vec::new();
    plines.push(poly.clone());
    let poly_raws = poly_to_raws(&plines);
    let offset_arcs = offset_single(&poly_raws, off, cfg);
    offset_arcs
}

fn offset_arcline_to_arcline_impl(arcs: &Arcline, off: f64, cfg: &mut OffsetCfg) -> Vec<Arc> {
    let mut alines = Vec::new();
    alines.push(arcs.clone());
    let poly_raws = arcs_to_raws(&alines);
    let offset_arcs = offset_single(&poly_raws, off, cfg);
    offset_arcs
}

#[doc(hidden)]
/// Converts a vector of arcs into a vector of polylines.
pub fn arcs_to_polylines(reconnect_arcs: &Vec<Vec<Arc>>) -> Vec<Polyline> {
    let mut polylines = Vec::with_capacity(reconnect_arcs.len());
    for arcs in reconnect_arcs.iter() {
        let polyline = arcs_to_polylines_single(arcs);
        polylines.push(polyline);
    }
    polylines
}

#[doc(hidden)]
/// function to convert from Vec<Arc> to Polyline
/// Note: arcs is a loop of arcs and when converting to PVertex,
/// some Arc can be either "a" to "b" or "b" to "a" oriented
pub fn arcs_to_polylines_single(arcs: &Vec<Arc>) -> Polyline {
    let mut polyline = Vec::new();

    if arcs.is_empty() {
        return polyline;
    }

    // Start with the first arc in its original orientation
    let mut current_end_point = arcs[0].b;

    for (i, arc) in arcs.iter().enumerate() {
        let (start_point, end_point, bulge) = if i == 0 {
            // First arc: use original orientation
            if arc.is_seg() {
                (arc.a, arc.b, 0.0)
            } else {
                let bulge = arc_bulge_from_points(arc.a, arc.b, arc.c, arc.r);
                (arc.a, arc.b, bulge)
            }
        } else {
            // For subsequent arcs, check orientation based on connectivity
            let prev_end = current_end_point;

            // Check if arc.a connects to previous end point
            let use_forward = prev_end.close_enough(arc.a, 1e-10);

            if use_forward {
                // Use arc in forward direction (a -> b)
                if arc.is_seg() {
                    (arc.a, arc.b, 0.0)
                } else {
                    let bulge = arc_bulge_from_points(arc.a, arc.b, arc.c, arc.r);
                    (arc.a, arc.b, bulge)
                }
            } else {
                // Use arc in reverse direction (b -> a)
                if arc.is_seg() {
                    (arc.b, arc.a, 0.0)
                } else {
                    // For reversed arc, we need to negate the bulge
                    let forward_bulge = arc_bulge_from_points(arc.a, arc.b, arc.c, arc.r);
                    (arc.b, arc.a, -forward_bulge)
                }
            }
        };

        polyline.push(pvertex(start_point, bulge));
        current_end_point = end_point;
    }

    polyline
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
        let polyline = arcs_to_polylines_single(&arcs);

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
        let polyline = arcs_to_polylines_single(&arcs);

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
        let polyline = arcs_to_polylines_single(&arcs);
        assert_eq!(polyline.len(), 0);
    }

    #[test]
    fn test_arcs_to_polylines_single_single_arc() {
        let arcs = vec![arcseg(point(0.0, 0.0), point(1.0, 0.0))];

        let polyline = arcs_to_polylines_single(&arcs);

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
        let offset_polylines = offset_polyline_to_polyline(&poly, off, config);
        polylines.extend(offset_polylines);
        off += step;
    }
    polylines
}

fn offset_single(poly_raws: &Vec<Vec<OffsetRaw>>, off: f64, cfg: &mut OffsetCfg) -> Vec<Arc> {
    let offset_raw = offset_polyline_raw::offset_polyline_raw(&poly_raws, off);
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
        // svg.offset_segments_single_points(&offset_split, "violet"); // Method not available in togo
    }

    let offset_prune = offset_prune_invalid(&poly_raws, &mut offset_split, off);

    if let Some(svg) = cfg.svg.as_deref_mut()
        && cfg.svg_prune
    {
        svg.arcline(&offset_prune, "violet");
        // svg.offset_segments_single_points(&offset_prune, "violet"); // Method not available in togo
    }
    offset_prune
}


#[doc(hidden)]
pub fn svg_offset_raws(svg: &mut SVG, offset_raws: &Vec<Vec<OffsetRaw>>, color: &str) {
    for raw in offset_raws {
        for seg in raw {
            if seg.arc.is_seg() {
                let segment = segment(seg.arc.a, seg.arc.b);
                svg.segment(&segment, color);
            } else {
                svg.arc(&seg.arc, color);
            }
        }
    }
}

// pub fn offset_convert_raw_to_arcs(raws: &Vec<OffsetRaw>) -> Vec<Arc> {
//     let mut res = Vec::new();
//     for raw in raws {
//         res.push(raw.arc.clone());
//     }
//     res
// }

/*
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct OffsetSegment {
    pub arc: Arc,
    //pub orig: Point, // original point p0
    pub is_arc: bool,
}

impl Display for OffsetSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.arc, self.is_arc)
    }
}

impl OffsetSegment {
    #[inline]
    pub fn new(arc: Arc, is_arc: bool) -> Self {
        OffsetSegment { arc, is_arc }
    }
}

#[inline]
pub fn offsetsegment(arc: Arc, is_arc: bool) -> OffsetSegment {
    OffsetSegment::new(arc, is_arc)
}
*/

// pub type OffsetSegment = Arc;

// #[derive(Debug, PartialEq)]
// pub struct OffsetRaw {
//     pub arc: Arc,
//     pub orig: Point, // original point p0
//     pub g: f64,
// }

// impl Display for OffsetRaw {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[{}, {}, {}]", self.arc, self.orig, self.g)
//     }
// }

// impl OffsetRaw {
//     #[inline]
//     fn new(arc: Arc, orig: Point, g: f64) -> Self {
//         OffsetRaw { arc, orig, g }
//     }
// }

// #[inline]
// fn offsetraw(arc: Arc, orig: Point, g: f64) -> OffsetRaw {
//     OffsetRaw::new(arc, orig, g)
// }

// #[cfg(test)]
// mod test_offset_raw {
//     use super::*;

//     #[test]
//     fn test_new() {
//         let arc = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
//         let o0 = OffsetRaw::new(arc, point(5.0, 6.0), 3.3);
//         let o1 = offsetraw(arc, point(5.0, 6.0), 3.3);
//         assert_eq!(o0, o1);
//     }

//     #[test]
//     fn test_display() {
//         let arc = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
//         let o0 = OffsetRaw::new(arc, point(5.0, 6.0), 3.3);
//         assert_eq!("[[[1.00000000000000000000, 2.00000000000000000000], [3.00000000000000000000, 4.00000000000000000000], [3.49848484848484808651, 1.50151515151515169144], 2.54772716009334887488], [5.00000000000000000000, 6.00000000000000000000], 3.3]", format!("{}", o0));
//     }
// }

const ZERO: f64 = 0f64;

// // Offsets line segment on right side
// // https://github.com/jbuckmccready/CavalierContours/blob/master/include/cavc/polylineoffset.hpp
// fn line_offset(vertex0: PVertex, vertex1: PVertex, off: f64) -> OffsetRaw {
//     // line segment
//     let edge = vertex1.p - vertex0.p;
//     let edge = point(edge.y, -edge.x).normalize();
//     let offset_v = edge * off;
//     let arc = arcline(vertex0.p + offset_v, vertex1.p + offset_v);
//     return OffsetRaw {
//         arc,
//         orig: vertex0.p,
//         g: ZERO,
//     };
// }

// const EPS_COLLAPSED: f64 = 1E-10; // TODO: what should be the exact value.
//                                   // Offsets arc on right side
//                                   // https://github.com/jbuckmccready/CavalierContours/blob/master/include/cavc/polylineoffset.hpp
//                                   // Offsets arc on right side
// fn arc_offset(v0: PVertex, v1: PVertex, offset: f64) -> OffsetRaw {
//     let bulge = v0.g;
//     // arc is always CCW
//     let param = arc_circle_parametrization(v0.p, v1.p, bulge);
//     let v0_to_center = v0.p - param.c;
//     let v0_to_center = v0_to_center.normalize();
//     let v1_to_center = v1.p - param.c;
//     let v1_to_center = v1_to_center.normalize();

//     let off = if bulge < 0.0 { -offset } else { offset };
//     let offset_radius = (param.r + off).abs();
//     if offset_radius < EPS_COLLAPSED {
//         // Collapsed arc is now line
//         return OffsetRaw {
//             arc: arcline(v0.p + v0_to_center * off, v1.p + v1_to_center * off),
//             orig: v0.p,
//             g: 0f64,
//         };
//     } else {
//         return OffsetRaw {
//             arc: arc(
//                 v0.p + v0_to_center * off,
//                 v1.p + v1_to_center * off,
//                 param.c,
//                 offset_radius,
//             ),
//             orig: v0.p,
//             g: v0.g,
//         };
//     }
// }

// fn segment_offset(vertex0: PVertex, vertex1: PVertex, off: f64) -> OffsetRaw {
//     if vertex0.g == ZERO {
//         line_offset(vertex0, vertex1, off)
//     } else {
//         arc_offset(vertex0, vertex1, off)
//     }
// }

// #[cfg(test)]
// mod test_offset {
//     use crate::pvertex::pvertex;

//     use super::*;
//     const ONE: f64 = 1f64;
//     const ZERO: f64 = 0f64;

//     #[test]
//     fn test_line_offset_vertical() {
//         // vertical segment
//         let v0 = pvertex(point(2.0, 1.0), ZERO);
//         let v1 = pvertex(point(2.0, 11.0), ZERO);
//         let res = offsetraw(
//             arcline(point(3.0, 1.0), point(3.0, 11.0)),
//             point(2.0, 1.0),
//             0.0,
//         );
//         assert_eq!(line_offset(v0, v1, 1.0), res);
//     }
//     #[test]
//     fn test_line_offset_horizontal() {
//         // horizontal segment
//         let v0 = pvertex(point(-2.0, 1.0), ZERO);
//         let v1 = pvertex(point(3.0, 1.0), ZERO);
//         let res = offsetraw(
//             arcline(point(-2.0, -1.0), point(3.0, -1.0)),
//             point(-2.0, 1.0),
//             0.0,
//         );
//         assert_eq!(line_offset(v0, v1, 2.0), res);
//     }
//     #[test]
//     fn test_line_offset_diagonal() {
//         // diagonal segment
//         let v0 = pvertex(point(-1.0, 1.0), ZERO);
//         let v1 = pvertex(point(-2.0, 2.0), ZERO);
//         let res = offsetraw(
//             arcline(point(0.0, 2.0), point(-1.0, 3.0)),
//             point(-1.0, 1.0),
//             0.0,
//         );
//         assert_eq!(line_offset(v0, v1, std::f64::consts::SQRT_2), res);
//     }
// }

// fn offset_polyline_raw(pline: &Polyline, off: f64) -> Vec<OffsetRaw> {
//     let mut result = Vec::with_capacity(pline.len() + 1);
//     let last = pline.len() - 2;
//     for i in 0..=last {
//         let offset = segment_offset(pline[i], pline[i + 1], off);
//         result.push(offset);
//     }
//     // close plyne
//     let offset = segment_offset(*pline.last().unwrap(), pline[0], off);
//     result.push(offset);

//     println!("raw size: {}", result.len());
//     result
// }

/*
pub fn check_if_segments_intersect(off0: OffsetSegment, off1: OffsetSegment) -> bool {
    if !off0.is_arc() {
        if !off1.is_arc() {
            // two segments
            let segment0 = segment(off0.a, off0.b);
            let segment1 = segment(off1.a, off1.b);
            let res = intersect_segment_segment(segment0, segment1);
            match res {
                SegmentConfig::NoIntersection() => {
                    return false;
                }
                _ => {
                    return true;
                }
            }
        } else {
            // segment arc
            let segment0 = segment(off0.a, off0.b);
            let res = intersect_segment_arc(segment0, off1);
            match res {
                SegmentArcConfig::NoIntersection() => {
                    return false;
                }
                _ => {
                    return true;
                }
            }
        }
    } else {
        if !off1.is_arc() {
            // arc and segment
            let segment0 = segment(off1.a, off1.b);
            let res = intersect_segment_arc(segment0, off0);
            match res {
                SegmentArcConfig::NoIntersection() => {
                    return false;
                }
                _ => {
                    return true;
                }
            }
        } else {
            // two arcs
            let res = intersect_arc_arc(off0, off1);
            match res {
                crate::int_arc_arc::ArcConfig::NoIntersection() => {
                    return false;
                }
                _ => {
                    return true;
                }
            }
        }
    }
}
*/

// // Checks, if to create arc connection
// // For two consecutive raw offsets,
// // if end points are close to the other original pline,
// // we skip the connecting arc
// fn offset_if_close_to_pline2(point: Point, seg: &OffsetSegment, off: f64) -> bool {
//     const EPS_IS_CLOSE_PLINE: f64 = 1E-10;
//     let dist = if seg.is_line() {
//         let segment = segment(seg.a, seg.b);
//         let (_, dist) = distance_point_segment(point, segment);
//         dist
//     } else {
//         // is arc
//         let (_, dist) = distance_point_arc(point, &seg);
//         dist
//     };

//     if off - dist > EPS_IS_CLOSE_PLINE {
//         true
//     } else {
//         false
//     }
// }

// pub fn offset_consecutive_offsets_intersect(off0: &OffsetSegment, off1: &OffsetSegment) -> bool {
//     let f0 = off0.is_line();
//     let f1 = off1.is_line();
//     match (f0, f1) {
//         (true, true) => {
//             let seg0 = segment(off0.a, off0.b);
//             let seg1 = segment(off1.a, off1.b);
//             if intersect_segment_segment(seg0, seg1) == SegmentConfig::NoIntersection() {
//                 false
//             } else {
//                 true
//             }
//         }
//         (true, false) => {
//             let seg0 = segment(off0.a, off0.b);
//             if intersect_segment_arc(seg0, off1) == SegmentArcConfig::NoIntersection() {
//                 false
//             } else {
//                 true
//             }
//         }
//         (false, true) => {
//             let seg1 = segment(off1.a, off1.b);
//             if intersect_segment_arc(seg1, off0) == SegmentArcConfig::NoIntersection() {
//                 false
//             } else {
//                 true
//             }
//         }
//         (false, false) => {
//             if intersect_arc_arc(off0, off1) == ArcArcConfig::NoIntersection() {
//                 false
//             } else {
//                 true
//             }
//         }
//     }
// }

// fn offset_connect_segments(
//     oarc: &Vec<OffsetSegment>,
//     raws: &Vec<OffsetRaw>,
//     off: f64,
// ) -> Vec<OffsetSegment> {
//     let flag = false; // to skip close connecting arcs
//     let mut res = Vec::with_capacity(2 * raws.len() + 1); // twise the size
//     let last = raws.len() - 2;
//     for i in 0..=last {
//         // convert offsetRaw to OffsetSegment
//         // make arcs ccw
//         let old = raws[i].arc;
//         let old_next = raws[i + 1].arc;
//         let arc_connect = Arc::new(old.b, old_next.a, raws[i + 1].orig, off);
//         let old_revert;
//         if raws[i].g < ZERO {
//             // revert arc dirrection
//             old_revert = Arc::new(old.b, old.a, old.c, old.r);
//         } else {
//             old_revert = old;
//         }
//         res.push(old_revert);

//         let next_revert;
//         if raws[i + 1].g < ZERO {
//             next_revert = Arc::new(old_next.b, old_next.a, old_next.c, old_next.r);
//         } else {
//             next_revert = old_next;
//         }

//         // Create segment connections
//         let skip0 = offset_consecutive_offsets_intersect(&old_revert, &next_revert);
//         if !skip0 || !flag {
//             res.push(arc_connect);
//         }
//     }
//     // close line
//     let last = raws.last().unwrap();
//     let old = last.arc;
//     let raw_next = raws.first().unwrap();
//     let old_next = raw_next.arc;
//     let arc_connect = Arc::new(old.b, old_next.a, raw_next.orig, off);
//     let old_revert;
//     if last.g < ZERO {
//         old_revert = Arc::new(old.b, old.a, old.c, old.r);
//     } else {
//         old_revert = old;
//     }
//     res.push(old_revert);

//     let next_revert;
//     if raw_next.g < ZERO {
//         next_revert = Arc::new(old_next.b, old_next.a, old_next.c, old_next.r);
//     } else {
//         next_revert = old_next;
//     }

//     // Create segment connections
//     let skip0 = offset_consecutive_offsets_intersect(&old_revert, &next_revert);
//     if !skip0 || !flag {
//         res.push(arc_connect);
//     }

//     println!("connected size: {}", res.len());
//     res
// }

// // Remove line segments with 0 length
// fn offset_remove_degenerate(offs: &mut Vec<OffsetSegment>) {
//     let mut i = 0;
//     while i < offs.len() - 1 {
//         if offs[i].a.x == offs[i].b.x && offs[i].a.y == offs[i].b.y {
//             offs.swap_remove(i);
//         }
//         i = i + 1;
//     }
//     println!("remove degenerate size: {}", offs.len());
// }

// // Split segments at self intersect points
// fn offset_resolve_self_intersect(offs: &mut Vec<OffsetSegment>) {
//     let mut i = 0;
//     let mut j = 1;

//     'LLL: while i < offs.len() - 1 {
//         const EPS_SMALL_SEGMENT: f64 = 1E-8;
//         while j < offs.len() {
//             let (i_new, j_new) = offset_resolve_offseg_offseg(i, j, offs);
//             let offslen = offs.len();

//             if offslen > 2000 {
//                 // TODO
//                 println!("ERROR: {}", offslen);
//                 //debug_assert!(false);
//                 break 'LLL;
//             }
//             i = i_new;
//             j = j_new;
//         }
//         i = i + 1;
//         j = i + 1;
//         print!("{} ", offs.len());
//     }
//     println!("\nself intersect resolve size: {}", offs.len());
// }

// // TODO
// // check segments that start and end point does not coincide
// fn check_add_segment(off: OffsetSegment, segments: &mut Vec<OffsetSegment>) {
//     const EPS_DEGENERATE: f64 = 0.0;
//     if off.a != off.b {
//         let norm = (off.a - off.b).norm();
//         if norm < EPS_DEGENERATE {
//             // small arc is transformed to line
//             let off = arcline(off.a, off.b);
//             print!("{} ", segments.len());
//             segments.push(off);
//         } else {
//             segments.push(off);
//         }
//     }
// }

// fn offset_resolve_line_line(
//     off0: &OffsetSegment,
//     off1: &OffsetSegment,
//     offs: &mut Vec<OffsetSegment>,
//     i: usize,
//     j: usize,
// ) -> (usize, usize) {
//     // two line segments
//     let seg0 = segment(off0.a, off0.b);
//     let seg1 = segment(off1.a, off1.b);
//     if is_touching_segment_segment(seg0, seg1) {
//         return (i, j + 1);
//     }
//     let res = intersect_segment_segment(seg0, seg1);
//     match res {
//         SegmentConfig::NoIntersection() => {
//             return (i, j + 1);
//         }
//         SegmentConfig::OnePoint(sp, _, _) => {
//             // split segments on sp - split point
//             // Important: remove j first, otherwise corner cases
//             offs.swap_remove(j);
//             offs.swap_remove(i);
//             let line00 = arcline(off0.a, sp);
//             let line01 = arcline(sp, off0.b);
//             let line10 = arcline(off1.a, sp);
//             let line11 = arcline(sp, off1.b);
//             check_add_segment(line00, offs);
//             check_add_segment(line01, offs);
//             check_add_segment(line10, offs);
//             check_add_segment(line11, offs);
//             return (i, i + 1);
//         }
//         SegmentConfig::TwoPoints(..) => {
//             return (i, j + 1); // TODO
//         }
//     }
// }

// fn offset_resolve_line_arc(
//     off0: &OffsetSegment,
//     off1: &OffsetSegment,
//     offs: &mut Vec<OffsetSegment>,
//     i: usize,
//     j: usize,
// ) -> (usize, usize) {
//     // line segment arc
//     // Swap line and arc for proper intersect arguments
//     let seg = if off0.is_line() {
//         segment(off0.a, off0.b)
//     } else {
//         segment(off1.a, off1.b)
//     };
//     let arc = if off1.is_arc() { off1 } else { off0 };

//     if is_touching_segment_arc(seg, &arc) {
//         // TODO: have to check for self intersect
//         return (i, j + 1);
//     }

//     let res = intersect_segment_arc(seg, &arc);
//     match res {
//         SegmentArcConfig::NoIntersection() => {
//             return (i, j + 1);
//         }
//         SegmentArcConfig::OnePoint(sp, _) => {
//             // split segment and arc at sp - split point
//             // Important: remove j first, otherwise corner cases
//             if i < j {
//                 offs.swap_remove(j);
//                 offs.swap_remove(i);
//             } else {
//                 offs.swap_remove(i);
//                 offs.swap_remove(j);
//             }
//             let line00 = arcline(seg.p0, sp);
//             let line01 = arcline(sp, seg.p1);
//             let arc10 = Arc::new(arc.a, sp, arc.c, arc.r);
//             let arc11 = Arc::new(sp, arc.b, arc.c, arc.r);
//             check_add_segment(line00, offs);
//             check_add_segment(line01, offs);
//             check_add_segment(arc10, offs);
//             check_add_segment(arc11, offs);
//             return (i, i + 1);
//         }
//         SegmentArcConfig::TwoPoints(..) => {
//             return (i, j + 1); // TODO
//         }
//     }
// }

// fn offset_resolve_arc_arc(
//     off0: &OffsetSegment,
//     off1: &OffsetSegment,
//     offs: &mut Vec<OffsetSegment>,
//     i: usize,
//     j: usize,
// ) -> (usize, usize) {
//     // two arcs
//     if is_touching_arc_arc(&off0, &off1) {
//         return (i, j + 1);
//     }
//     let res = intersect_arc_arc(&off0, &off1);
//     match res {
//         crate::int_arc_arc::ArcArcConfig::NoIntersection() => {
//             return (i, j + 1);
//         }
//         crate::int_arc_arc::ArcArcConfig::NonCocircularOnePoint(sp) => {
//             // split arcs at sp - split point
//             // Important: remove j first, otherwise corner cases
//             offs.swap_remove(j);
//             offs.swap_remove(i);
//             let (arc00, arc01) = split_arc_1point(&off0, sp);
//             let (arc10, arc11) = split_arc_1point(&off1, sp);
//             check_add_segment(arc00, offs);
//             check_add_segment(arc01, offs);
//             check_add_segment(arc10, offs);
//             check_add_segment(arc11, offs);
//             return (i, i + 1);
//         }
//         crate::int_arc_arc::ArcArcConfig::NonCocircularTwoPoints(sp0, sp1) => {
//             // split arcs at sp - split point
//             // Important: remove j first, otherwise corner cases
//             offs.swap_remove(j);
//             offs.swap_remove(i);
//             let (segment00, segment01, segment02) = split_arc_2points(&off0, sp0, sp1);
//             let (segment10, segment11, segment12) = split_arc_2points(&off1, sp0, sp1);
//             check_add_segment(segment00, offs);
//             check_add_segment(segment01, offs);
//             check_add_segment(segment02, offs);
//             check_add_segment(segment10, offs);
//             check_add_segment(segment11, offs);
//             check_add_segment(segment12, offs);
//             return (i, i + 1);
//         }
//         crate::int_arc_arc::ArcArcConfig::CocircularOnePoint(_) => todo!(),
//         crate::int_arc_arc::ArcArcConfig::CocircularTwoPoints(..) => todo!(),
//         crate::int_arc_arc::ArcArcConfig::CocircularOnePointOneArc(..) => todo!(),
//         crate::int_arc_arc::ArcArcConfig::CocircularOneArc(_) => {
//             return (i, j + 1);
//         }
//         crate::int_arc_arc::ArcArcConfig::CocircularTwoArcs(..) => {
//             return (i, j + 1);
//         }
//     }
// }

// fn offset_resolve_offseg_offseg(
//     i: usize,
//     j: usize,
//     offs: &mut Vec<OffsetSegment>,
// ) -> (usize, usize) {
//     // If segments intersect remove from vector and put the splitted parts at the back of vector.
//     // Than adjust the indexes accordingly
//     let off0 = offs[i];
//     let off1 = offs[j];

//     let id0 = off0.id;
//     let id1 = off1.id;

//     let offslen = offs.len();
//     /*if offslen > 1200 {
//         println!(" {} ", offslen);
//     }*/
//     //if offslen > 240 {
//     //    println!("({} {})", id0, id1);
//     // }

//     if off0.is_line() {
//         if off1.is_line() {
//             let res = offset_resolve_line_line(&off0, &off1, offs, i, j);
//             debug_assert!(res.0 < res.1);
//             return res;
//         } else {
//             let res = offset_resolve_line_arc(&off0, &off1, offs, i, j);
//             debug_assert!(res.0 < res.1);
//             return res;
//         }
//     } else {
//         if off1.is_line() {
//             let res = offset_resolve_line_arc(&off1, &off0, offs, i, j);
//             //println!("", res.0, res.1);
//             debug_assert!(res.0 < res.1);
//             return res;
//         } else {
//             let res = offset_resolve_arc_arc(&off0, &off1, offs, i, j);
//             debug_assert!(res.0 < res.1);
//             return res;
//         }
//     }
// }

// fn split_arc_2points(
//     off: &OffsetSegment,
//     sp0: Point,
//     sp1: Point,
// ) -> (OffsetSegment, OffsetSegment, OffsetSegment) {
//     let (p0, p1) = off.order_points_ccw(sp0, sp1);
//     let arc0 = arc(off.a, p0, off.c, off.r);
//     let arc1 = arc(p0, p1, off.c, off.r);
//     let arc2 = arc(p1, off.b, off.c, off.r);
//     (arc0, arc1, arc2)
// }

// fn split_arc_1point(off: &OffsetSegment, sp0: Point) -> (OffsetSegment, OffsetSegment) {
//     let arc0 = arc(off.a, sp0, off.c, off.r);
//     let arc1 = arc(sp0, off.b, off.c, off.r);
//     (arc0, arc1)
// }

// #[cfg(test)]
// mod test_offset_polyline_raw {
//     use std::vec;

//     use crate::{arc::arc_g_from_points, circle::circle, point::point, pvertex::pvertex, svg::svg};

//     use super::*;
//     const ONE: f64 = 1f64;
//     const ZERO: f64 = 0f64;

//     #[test]
//     #[ignore = "svg"]
//     fn test_arc_offset() {
//         let mut svg = svg(300.0, 300.0);
//         let p0 = point(100.0, 100.0);
//         let p1 = point(100.0, 160.0);
//         let p2 = point(120.0, 200.0);
//         let p3 = point(128.0, 192.0);
//         let p4 = point(128.0, 250.0);
//         let p5 = point(0.0, 250.0);
//         let p6 = point(0.0, 100.0);
//         let off = 10.0;
//         let b = 1.5;
//         let pvertex0 = pvertex(p0, b);
//         let pvertex1 = pvertex(p1, 0f64);
//         let offset = segment_offset(pvertex0, pvertex1, off);
//         svg.pvertex(p0, p1, b, "red");
//         svg.segment_raw(&offset, "black");

//         let pvertex2 = pvertex(p2, 0f64);
//         let offset = segment_offset(pvertex1, pvertex2, off);
//         svg.pvertex(p1, p2, 0f64, "red");
//         svg.segment_raw(&offset, "black");

//         let pvertex3 = pvertex(p3, 0f64);
//         let offset = segment_offset(pvertex2, pvertex3, off);
//         svg.pvertex(p2, p3, 0f64, "red");
//         svg.segment_raw(&offset, "black");

//         let pvertex4 = pvertex(p4, 0f64);
//         let offset = segment_offset(pvertex3, pvertex4, off);
//         svg.pvertex(p3, p4, 0f64, "red");
//         svg.segment_raw(&offset, "black");

//         let pvertex5 = pvertex(p5, 0f64);
//         let offset = segment_offset(pvertex4, pvertex5, off);
//         svg.pvertex(p4, p5, 0f64, "red");
//         svg.segment_raw(&offset, "black");

//         let pvertex6 = pvertex(p6, 0f64);
//         let offset = segment_offset(pvertex5, pvertex6, off);
//         svg.pvertex(p5, p6, 0f64, "red");
//         svg.segment_raw(&offset, "black");

//         let offset = segment_offset(pvertex6, pvertex0, off);
//         svg.pvertex(p6, p0, 0f64, "red");
//         svg.segment_raw(&offset, "black");

//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_offset_polyline_raw() {
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
//         let mut svg = svg(300.0, 350.0);
//         svg.polyline(&pline, "red");
//         let pline_offset = offset_polyline_raw(&pline, 10.0);
//         svg.offset_raws(&pline_offset, "black");
//         svg.write();
//     }

//     #[test]
//     #[ignore = "svg output"]
//     fn test_connect_offset_segments() {
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
//         let off = 10.0;
//         let mut svg = svg(300.0, 340.0);
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

// // convert polyline to offsetsegments
// pub fn offset_pline_to_offsetsegments(pline: &Polyline) -> Vec<OffsetSegment> {
//     let mut oarc: Vec<OffsetSegment> = Vec::with_capacity(pline.len());
//     let last = pline.len() - 2;
//     for i in 0..=last {
//         let offseg = arc_circle_parametrization(pline[i].p, pline[i + 1].p, pline[i].g);
//         oarc.push(offseg);
//     }
//     let offseg = arc_circle_parametrization(
//         pline.last().unwrap().p,
//         pline.first().unwrap().p,
//         pline.last().unwrap().g,
//     );
//     oarc.push(offseg);
//     oarc
// }

// // offsets close to pline are removed
// pub fn offset_remove_invalid_offsets(
//     offs: &mut Vec<OffsetSegment>,
//     oarc: &Vec<OffsetSegment>, // original polyline converted to offset segments
//     off: f64,
// ) {
//     const EPS_IVALIID: f64 = 1E-10;
//     let mut i = 0;
//     let mut flag_remove = false;
//     while i < offs.len() {
//         let arc0 = offs[i];
//         for arc1 in oarc.iter() {
//             let dist = distance_offsets(&arc0, arc1);
//             debug_assert!(dist.is_finite());
//             if off - dist > EPS_IVALIID {
//                 // TODO: choose value for comparison
//                 // TODO maybe int diff here?
//                 offs.swap_remove(i);
//                 flag_remove = true;
//                 break;
//             }
//         }
//         if flag_remove {
//             flag_remove = false;
//         } else {
//             i = i + 1;
//         }
//     }
//     println!("remove_invalid_offsets size: {}", offs.len());
// }

// // Calculate distance between offset segment and pline
// fn distance_offsets(off0: &OffsetSegment, off1: &OffsetSegment) -> f64 {
//     if off0.is_line() {
//         let seg0 = segment(off0.a, off0.b);
//         if off1.is_line() {
//             let seg1 = segment(off1.a, off1.b);
//             return distance_segment_segment(seg0, seg1).0;
//         } else {
//             return distance_segment_arc(seg0, &off1).1;
//         }
//     } else {
//         if off1.is_line() {
//             let seg1 = segment(off1.a, off1.b);
//             return distance_segment_arc(seg1, &off0).1;
//         } else {
//             return distance_arc_arc(&off0, &off1).1;
//         }
//     }
// }

/// Test polyline for offseting.
/// Has a mix of positive and negative offsets.
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
    // let pline2 = polyline_scale(&pline, 1.0);
    // let plines = vec![pline2.clone()];
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

// pub fn offset_polyline(pline: &Polyline, off: f64) -> Vec<Arc> {
//     let offset_raw = crate::offset_polyline_raw::offset_polyline_raw(&pline, off);
//     //svg.offset_raws(&offset_raw, "blue");
//     // convert polyline to offsetsegments
//     let mut offset = crate::offset_connect_segments::offset_connect_segments(&offset_raw, off);
//     offset_remove_degenerate(&mut offset);
//     offset_resolve_self_intersect(&mut offset);
//     offset_remove_degenerate(&mut offset);
//     let oarc = offset_pline_to_offsetsegments(&pline);
//     offset_remove_invalid_offsets(&mut offset, &oarc, off);
//     offset
// }

// pub fn offset_center_point(offs: &Vec<OffsetSegment>) -> Point {
//     let mut res = point(0.0, 0.0);
//     let count = offs.len() as f64;
//     for off in offs.iter() {
//         res = res + off.a + off.b;
//     }
//     point(res.x / (2.0 * count), res.y / (2.0 * count))
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
    let mut arcs = Vec::with_capacity(pline.len() + 1);
    let last = pline.len() - 1;
    for i in 0..last {
        let arc = arc_circle_parametrization(pline[i].p, pline[i + 1].p, pline[i].b);
        arcs.push(arc);
    }
    // last segment
    let arc =
        arc_circle_parametrization(pline.last().unwrap().p, pline[0].p, pline.last().unwrap().b);
    arcs.push(arc);
    arcs
}

// pub fn poly_to_raws(plines: &Vec<Polyline>) -> Vec<Vec<OffsetRaw>> {
//     let mut varcs: Vec<Vec<OffsetRaw>> = Vec::new();
//     for pline in plines {
//         varcs.push(poly_to_raws_single(pline));
//     }
//     varcs
// }

// pub fn poly_to_raws_single(pline: &Polyline) -> Vec<OffsetRaw> {
//     let mut offs = Vec::with_capacity(pline.len() + 1);
//     //let last = pline.len() - 1;
//     for i in 0..pline.len() - 1 {
//         let arc = arc_circle_parametrization(pline[i].p, pline[i + 1].p, pline[i].g);
//         let orig = if pline[i].g < ZERO {pline[i].p} else {pline[i + 1].p};
//         let off = OffsetRaw {
//             arc,
//             orig: orig,
//             g: pline[i].g,
//         };
//         offs.push(off);
//     }
//     // last segment
//     let arc = arc_circle_parametrization(pline.last().unwrap().p, pline[0].p, pline.last().unwrap().g);
//     let orig = if pline.last().unwrap().g < ZERO {
//         pline.last().unwrap().p
//     } else {
//         pline[0].p
//     };
//     let off = OffsetRaw {
//         arc,
//         orig: orig,
//         g: pline.last().unwrap().g,
//     };
//     offs.push(off);
//     offs
// }

// pub fn offset_polylines(plines: &Vec<Polyline>, off: f64) -> Vec<Arc> {
//     let offset_raw = offset_polyline_raw(&plines, off);
//     let offset_connect = offset_connect_raw(&offset_raw, off);
//     //let arcs: Vec<Arc> = offset_connect.iter().map(|raw| raw.arc.clone()).collect();
//     let mut offset_split = offset_split_arcs(&offset_connect);
//     let poly_arcs = poly_to_arcs(&plines);
//     let offset_final = offset_prune_invalid_offsets(&poly_arcs, &mut offset_split, off);
//     offset_final
// }

#[cfg(test)]
mod test_offset {

    // use togo::prelude::*;


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
    #[ignore = "svg output"]
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
    #[ignore = "svg output"]
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
