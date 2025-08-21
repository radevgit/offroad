#![allow(dead_code)]

use geom::prelude::*;

use crate::offset_raw::OffsetRaw;

const PRUNE_EPSILON: f64 = 1e-9;
#[doc(hidden)]
/// Prune arcs that are close to any of the arcs in the original polyline.
#[must_use]
pub fn offset_prune_invalid(
    polyraws: &Vec<Vec<OffsetRaw>>,
    offsets: &mut Vec<Arc>,
    off: f64,
) -> Vec<Arc> {
    let mut valid = Vec::new();
    let arcs: Vec<Arc> = polyraws
        .iter()
        .flatten()
        .map(|offset_raw| offset_raw.arc)
        //.filter(|arc| arc_check(arc, PRUNE_EPSILON))
        .collect();

    while offsets.len() > 0 {
        let offset = offsets.pop().unwrap();
        valid.push(offset.clone());
        for p in arcs.iter() {
            if p.id == offset.id {
                // skip self offsets
                // self offsets are always on `off` distance
                continue;
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
    dist
}
