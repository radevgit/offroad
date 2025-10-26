#![allow(dead_code)]

use togo::prelude::*;
use togo::spatial::HilbertRTree;

use crate::offsetraw::OffsetRaw;

// Prune arcs that are close to any of the arcs in the polyline.
const PRUNE_EPSILON: f64 = 1e-8;

// Set to true to use brute-force algorithm (for testing/comparison)
const USE_BRUTE_FORCE: bool = false;

pub fn offset_prune_invalid(
    polyraws: &Vec<Vec<OffsetRaw>>,
    offsets: &mut Vec<Arc>,
    off: f64,
) -> Vec<Arc> {
    if USE_BRUTE_FORCE {
        offset_prune_invalid_brute_force(polyraws, offsets, off)
    } else {
        offset_prune_invalid_spatial(polyraws, offsets, off)
    }
}

fn offset_prune_invalid_spatial(
    polyraws: &Vec<Vec<OffsetRaw>>,
    offsets: &mut Vec<Arc>,
    off: f64,
) -> Vec<Arc> {
    let mut valid = Vec::new();
    let polyarcs: Vec<Arc> = polyraws
        .iter()
        .flatten()
        .map(|offset_raw| offset_raw.arc.clone())
        .filter(|arc| arc.is_valid(PRUNE_EPSILON))
        .collect();

    // Build spatial index containing both polyarcs and offsets
    let polyarc_count = polyarcs.len();
    let mut spatial_index = HilbertRTree::with_capacity(polyarc_count + offsets.len());
    
    // Add polyarcs to index with offset + epsilon expansion (done once)
    let search_radius = off + PRUNE_EPSILON;
    for arc in polyarcs.iter() {
        let (min_x, max_x, min_y, max_y) = arc_bounds_expanded(arc, search_radius);
        spatial_index.add(min_x, max_x, min_y, max_y);
    }
    
    // Add offsets to index
    for arc in offsets.iter() {
        let (min_x, max_x, min_y, max_y) = arc_bounds(arc);
        spatial_index.add(min_x, max_x, min_y, max_y);
    }
    
    spatial_index.build();

    while offsets.len() > 0 {
        let offset = offsets.pop().unwrap();
        valid.push(offset.clone());

        // Query nearby arcs using spatial index
        let (offset_min_x, offset_max_x, offset_min_y, offset_max_y) =
            arc_bounds(&offset);
        let mut nearby_indices = Vec::new();
        spatial_index.query_intersecting(
            offset_min_x,
            offset_max_x,
            offset_min_y,
            offset_max_y,
            &mut nearby_indices,
        );

        // Check only nearby polyarcs for actual distance
        for idx in nearby_indices {
            if idx < polyarc_count {
                let p = &polyarcs[idx];
                if p.id == offset.id {
                    continue; // skip self offsets
                }
                let dist = distance_element_element(p, &offset);
                if dist < off - PRUNE_EPSILON {
                    valid.pop();
                    break;
                }
            }
        }
    }
    valid
}

fn offset_prune_invalid_brute_force(
    polyraws: &Vec<Vec<OffsetRaw>>,
    offsets: &mut Vec<Arc>,
    off: f64,
) -> Vec<Arc> {
    let mut valid = Vec::new();
    let polyarcs: Vec<Arc> = polyraws
        .iter()
        .flatten()
        .map(|offset_raw| offset_raw.arc.clone())
        .filter(|arc| arc.is_valid(PRUNE_EPSILON))
        .collect();

    while offsets.len() > 0 {
        let offset = offsets.pop().unwrap();
        valid.push(offset.clone());
        for p in polyarcs.iter() {
            if p.id == offset.id {
                continue; // skip self offsets
            }
            let dist = distance_element_element(&p, &offset);
            if dist < off - PRUNE_EPSILON {
                valid.pop();
                break;
            }
        }
    }
    valid
}

/// Get bounding box of an arc
fn arc_bounds(arc: &Arc) -> (f64, f64, f64, f64) {
    if arc.is_seg() {
        // For line segments, just return min/max of endpoints
        let min_x = arc.a.x.min(arc.b.x);
        let max_x = arc.a.x.max(arc.b.x);
        let min_y = arc.a.y.min(arc.b.y);
        let max_y = arc.a.y.max(arc.b.y);
        (min_x, max_x, min_y, max_y)
    } else {
        // For arcs, return the bounding box of the circle (center ± radius)
        let cx = arc.c.x;
        let cy = arc.c.y;
        let r = arc.r;
        (cx - r, cx + r, cy - r, cy + r)
    }
}

/// Get expanded bounding box of an arc (for spatial queries)
fn arc_bounds_expanded(arc: &Arc, expansion: f64) -> (f64, f64, f64, f64) {
    let (min_x, max_x, min_y, max_y) = arc_bounds(arc);
    (
        min_x - expansion,
        max_x + expansion,
        min_y - expansion,
        max_y + expansion,
    )
}

fn distance_element_element(seg0: &Arc, seg1: &Arc) -> f64 {
    let mut dist = std::f64::INFINITY;
    if seg0.is_seg() && seg1.is_seg() {
        dist = dist_segment_segment(&segment(seg0.a, seg0.b), &segment(seg1.a, seg1.b));
    } else if seg0.is_arc() && seg1.is_arc() {
        dist = dist_arc_arc(seg0, seg1);
    } else if seg0.is_seg() && seg1.is_arc() {
        dist = dist_segment_arc(&segment(seg0.a, seg0.b), seg1);
    } else if seg0.is_arc() && seg1.is_seg() {
        dist = dist_segment_arc(&segment(seg1.a, seg1.b), seg0);
    }
    if seg1.id == 0 && dist < 16.0 {
        let _xxx = seg0.id;
        let _yyy = seg1.id;
        return dist;
    }
    return dist;
}
