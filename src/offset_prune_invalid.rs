#![allow(dead_code)]

use togo::prelude::*;

use crate::offset_raw::OffsetRaw;
use crate::spatial::spatial::{aabb_from_arc_loose, aabb_from_segment, BroadPhaseFlat};

// Prune arcs that are close to any of the arcs in the polyline.
const PRUNE_EPSILON: f64 = 1e-8;
pub fn offset_prune_invalid(
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

    // Build spatial index for polyline arcs
    let mut spatial = BroadPhaseFlat::new();
    for (idx, arc) in polyarcs.iter().enumerate() {
        let bbox = if arc.is_seg() {
            aabb_from_segment(&arc.a, &arc.b)
        } else {
            aabb_from_arc_loose(arc)
        };
        spatial.add(idx, bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
    }

    while offsets.len() > 0 {
        let offset = offsets.pop().unwrap();
        let offset_bbox = if offset.is_seg() {
            aabb_from_segment(&offset.a, &offset.b)
        } else {
            aabb_from_arc_loose(&offset)
        };

        // Query for overlapping candidates
        let candidates = spatial.query(offset_bbox.min_x, offset_bbox.max_x, 
                                       offset_bbox.min_y, offset_bbox.max_y);

        let mut found_close = false;
        for idx in candidates {
            let p = &polyarcs[idx];
            if p.id == offset.id {
                continue; // skip self offsets
            }
            let dist = distance_element_element(&p, &offset);
            if dist < off - PRUNE_EPSILON {
                found_close = true;
                break;
            }
        }

        if !found_close {
            valid.push(offset);
        }
    }
    valid
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

