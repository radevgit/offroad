#![allow(dead_code)]

use geom::prelude::*;

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
        .filter(|arc| arc_check(arc, PRUNE_EPSILON))
        .collect();
    let _zzz = polyarcs.len();

    while offsets.len() > 0 {
        let offset = offsets.pop().unwrap();
        valid.push(offset.clone());
        for p in polyarcs.iter() {
            if p.id == offset.id {
                continue; // skip self ofsets
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
    if seg0.is_line() && seg1.is_line() {
        dist = dist_segment_segment(&segment(seg0.a, seg0.b), &segment(seg1.a, seg1.b));
    } else if seg0.is_arc() && seg1.is_arc() {
        dist = dist_arc_arc(seg0, seg1);
    } else if seg0.is_line() && seg1.is_arc() {
        dist = dist_segment_arc(&segment(seg0.a, seg0.b), seg1);
    } else if seg0.is_arc() && seg1.is_line() {
        dist = dist_segment_arc(&segment(seg1.a, seg1.b), seg0);
    }
    if seg1.id == 0 && dist < 16.0 {
        let _xxx = seg0.id;
        let _yyy = seg1.id;
        return dist;
    }
    return dist;
}
