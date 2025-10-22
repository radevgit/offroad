#![allow(dead_code)]
#![deny(unused_results)]

use togo::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::offset_raw::OffsetRaw;
use crate::spatial::spatial::{aabb_from_arc_loose, aabb_from_segment, BroadPhaseFlat};

// Global stats for profiling
pub struct SplitStats {
    pub spatial_queries: usize,
    pub bbox_tests: usize,
    pub precise_tests: usize,
    pub intersections_found: usize,
}

lazy_static::lazy_static! {
    static ref SPLIT_STATS: Mutex<SplitStats> = Mutex::new(SplitStats {
        spatial_queries: 0,
        bbox_tests: 0,
        precise_tests: 0,
        intersections_found: 0,
    });
}

pub fn reset_stats() {
    let mut stats = SPLIT_STATS.lock().unwrap();
    stats.spatial_queries = 0;
    stats.bbox_tests = 0;
    stats.precise_tests = 0;
    stats.intersections_found = 0;
}

pub fn get_stats() -> (usize, usize, usize, usize) {
    let stats = SPLIT_STATS.lock().unwrap();
    (stats.spatial_queries, stats.bbox_tests, stats.precise_tests, stats.intersections_found)
}

static ZERO: f64 = 0.0;
const EPSILON: f64 = 1e-10;

pub fn offset_split_arcs(row: &Vec<Vec<OffsetRaw>>, connect: &Vec<Vec<Arc>>) -> Vec<Arc> {
    // Merge offsets and offset connections, filter singular arcs
    let mut parts: Vec<Arc> = row
        .iter()
        .flatten()
        .map(|offset_raw| offset_raw.arc.clone())
        .chain(connect.iter().flatten().cloned())
        .filter(|arc| arc.is_valid(EPSILON))
        .collect();

    let mut parts_final = Vec::new();
    let steps = 100000; // TODO: make this configurable

    let mut kk = 0;
    for k in 0..steps {
        let mut j_current = usize::MAX;

        while parts.len() > 0 {
            let part0 = parts.pop().unwrap();
            if parts.len() == 0 {
                // No more parts to check against
                parts_final.push(part0);
                break;
            }

            // Build spatial index for remaining parts (only if > threshold)
            let candidates = if parts.len() > 10 {
                let mut stats = SPLIT_STATS.lock().unwrap();
                stats.spatial_queries += 1;

                let mut spatial = BroadPhaseFlat::new();
                for (idx, part) in parts.iter().enumerate() {
                    if part0.id == part.id {
                        continue;
                    }
                    let bbox = if part.is_seg() {
                        aabb_from_segment(&part.a, &part.b)
                    } else {
                        aabb_from_arc_loose(part)
                    };
                    spatial.add(idx, bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
                }

                // Query candidates overlapping with part0's AABB
                let bbox0 = if part0.is_seg() {
                    aabb_from_segment(&part0.a, &part0.b)
                } else {
                    aabb_from_arc_loose(&part0)
                };
                spatial.query(bbox0.min_x, bbox0.max_x, bbox0.min_y, bbox0.max_y)
            } else {
                // For small lists, iterate all
                (0..parts.len()).collect()
            };

            for j in candidates.iter().rev() {
                j_current = usize::MAX;
                if part0.id == parts[*j].id {
                    // Skip parts coming from the same original arc
                    continue;
                }

                let part1 = parts[*j].clone();

                {
                    let mut stats = SPLIT_STATS.lock().unwrap();
                    stats.precise_tests += 1;
                }

                let (parts_new, _) = if part0.is_seg() && part1.is_seg() {
                    split_line_line(&part0, &part1)
                } else if part0.is_arc() && part1.is_arc() {
                    split_arc_arc(&part0, &part1)
                } else if part0.is_seg() && part1.is_arc() {
                    split_segment_arc(&part0, &part1)
                } else if part0.is_arc() && part1.is_seg() {
                    split_segment_arc(&part1, &part0)
                } else {
                    (Vec::new(), 0)
                };

                if !parts_new.is_empty() {
                    {
                        let mut stats = SPLIT_STATS.lock().unwrap();
                        stats.intersections_found += parts_new.len();
                    }
                    j_current = *j;
                    parts.extend(parts_new);
                    break;
                }
            }
            // this part parts[i] does not intersect with any other part
            if j_current == usize::MAX {
                parts_final.push(part0);
            } else {
                // remove the part1 from the parts
                _ = parts.remove(j_current);
            }
        }

        if parts.is_empty() {
            // No more splits, we are done
            kk = k;
            break;
        }
    }

    let _kkk = kk;
    parts_final
}

// ... rest of the functions remain the same ...
pub fn split_line_line(arc0: &Arc, arc1: &Arc) -> (Vec<Arc>, usize) {
    let mut res = Vec::new();
    let seg0 = segment(arc0.a, arc0.b);
    let seg1 = segment(arc1.a, arc1.b);
    let intersection = int_segment_segment(&seg0, &seg1);
    match intersection {
        SegmentSegmentConfig::NoIntersection()
        | SegmentSegmentConfig::OnePointTouching(_, _, _)
        | SegmentSegmentConfig::TwoPointsTouching(_, _, _, _) => {
            (res, 0)
        }
        SegmentSegmentConfig::OnePoint(sp, _, _) => {
            let mut line00 = arcseg(sp, arc0.a);
            let mut line01 = arcseg(sp, arc0.b);
            let mut line10 = arcseg(sp, arc1.a);
            let mut line11 = arcseg(sp, arc1.b);
            line00.id(arc0.id);
            line01.id(arc0.id);
            line10.id(arc1.id);
            line11.id(arc1.id);
            check_and_push(&mut res, &line00);
            check_and_push(&mut res, &line01);
            check_and_push(&mut res, &line10);
            check_and_push(&mut res, &line11);
            (res, 4)
        }
        SegmentSegmentConfig::TwoPoints(p0, p1, p2, p3) => {
            let mut line00 = arcseg(p0, p1);
            let mut line01 = arcseg(p1, p2);
            let mut line10 = arcseg(p2, p3);
            line00.id(arc0.id);
            line01.id(arc0.id);
            line10.id(arc1.id);
            check_and_push(&mut res, &line00);
            check_and_push(&mut res, &line01);
            check_and_push(&mut res, &line10);
            (res, 3)
        }
    }
}

pub fn split_arc_arc(arc0: &Arc, arc1: &Arc) -> (Vec<Arc>, usize) {
    let mut res = Vec::new();
    let inter = int_arc_arc(&arc0, &arc1);
    match inter {
        ArcArcConfig::NoIntersection()
        | ArcArcConfig::CocircularOnePoint0(_)
        | ArcArcConfig::CocircularOnePoint1(_)
        | ArcArcConfig::CocircularTwoPoints(_, _)
        | ArcArcConfig::NonCocircularOnePointTouching(_)
        | ArcArcConfig::NonCocircularTwoPointsTouching(_, _) => {
            (res, 0)
        }
        ArcArcConfig::NonCocircularOnePoint(p) => {
            let mut arc00 = arc(arc0.a, p, arc0.c, arc0.r);
            let mut arc01 = arc(p, arc0.b, arc0.c, arc0.r);
            let mut arc10 = arc(arc1.a, p, arc1.c, arc1.r);
            let mut arc11 = arc(p, arc1.b, arc1.c, arc1.r);
            arc00.id(arc0.id);
            arc01.id(arc0.id);
            arc10.id(arc1.id);
            arc11.id(arc1.id);
            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            check_and_push(&mut res, &arc10);
            check_and_push(&mut res, &arc11);
            (res, 4)
        }
        ArcArcConfig::NonCocircularTwoPoints(point0, point1) => {
            let mut p0 = point0;
            let mut p1 = point1;
            if points_order(arc0.a, p0, p1) < ZERO {
                (p1, p0) = (p0, p1);
            }
            let mut arc00 = arc(arc0.a, p0, arc0.c, arc0.r);
            let mut arc01 = arc(p0, p1, arc0.c, arc0.r);
            let mut arc02 = arc(p1, arc0.b, arc0.c, arc0.r);
            arc00.id(arc0.id);
            arc01.id(arc0.id);
            arc02.id(arc0.id);

            if points_order(arc1.a, p0, p1) < ZERO {
                (p1, p0) = (p0, p1);
            }
            let mut arc10 = arc(arc1.a, p0, arc1.c, arc1.r);
            let mut arc11 = arc(p0, p1, arc1.c, arc1.r);
            let mut arc12 = arc(p1, arc1.b, arc1.c, arc1.r);
            arc10.id(arc1.id);
            arc11.id(arc1.id);
            arc12.id(arc1.id);

            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            check_and_push(&mut res, &arc02);
            check_and_push(&mut res, &arc10);
            check_and_push(&mut res, &arc11);
            check_and_push(&mut res, &arc12);
            (res, 6)
        }
        ArcArcConfig::CocircularOnePointOneArc0(_, _) => {
            let mut arc00 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
            let mut arc01 = arc(arc1.b, arc0.b, arc0.c, arc0.r);
            let mut arc02 = arc(arc0.b, arc0.a, arc0.c, arc0.r);
            arc00.id(arc0.id);
            arc01.id(arc1.id);
            arc02.id(arc0.id);
            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            check_and_push(&mut res, &arc02);
            (res, 3)
        }
        ArcArcConfig::CocircularOnePointOneArc1(_, _) => {
            let mut arc00 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
            let mut arc01 = arc(arc0.b, arc1.b, arc0.c, arc0.r);
            let mut arc02 = arc(arc1.b, arc1.a, arc0.c, arc0.r);
            arc00.id(arc1.id);
            arc01.id(arc0.id);
            arc02.id(arc1.id);
            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            check_and_push(&mut res, &arc02);
            (res, 3)
        }
        ArcArcConfig::CocircularOneArc0(_) => {
            let mut arc00 = arc(arc0.a, arc0.b, arc0.c, arc0.r);
            arc00.id(arc0.id);
            check_and_push(&mut res, &arc00);
            (res, 1)
        }
        ArcArcConfig::CocircularOneArc1(_) => {
            let mut arc00 = arc(arc1.a, arc0.a, arc0.c, arc0.r);
            let mut arc01 = arc(arc0.a, arc0.b, arc0.c, arc0.r);
            let mut arc02 = arc(arc0.b, arc1.b, arc0.c, arc0.r);
            arc00.id(arc1.id);
            arc01.id(arc0.id);
            arc02.id(arc1.id);
            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            check_and_push(&mut res, &arc02);
            (res, 3)
        }
        ArcArcConfig::CocircularOneArc2(_) => {
            let mut arc00 = arc(arc1.a, arc0.a, arc0.c, arc0.r);
            let mut arc01 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
            let mut arc02 = arc(arc1.b, arc0.b, arc0.c, arc0.r);
            arc00.id(arc1.id);
            arc01.id(arc0.id);
            arc02.id(arc1.id);
            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            check_and_push(&mut res, &arc02);
            (res, 3)
        }
        ArcArcConfig::CocircularOneArc3(_) => {
            let mut arc00 = arc(arc0.a, arc1.a, arc0.c, arc0.r);
            let mut arc01 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
            let mut arc02 = arc(arc0.b, arc1.b, arc0.c, arc0.r);
            arc00.id(arc0.id);
            arc01.id(arc1.id);
            arc02.id(arc0.id);
            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            check_and_push(&mut res, &arc02);
            (res, 3)
        }
        ArcArcConfig::CocircularOneArc4(_) => {
            let mut arc00 = arc(arc0.a, arc1.a, arc0.c, arc0.r);
            let mut arc01 = arc(arc1.a, arc1.b, arc0.c, arc0.r);
            let mut arc02 = arc(arc1.b, arc0.b, arc0.c, arc0.r);
            arc00.id(arc0.id);
            arc01.id(arc1.id);
            arc02.id(arc0.id);
            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            check_and_push(&mut res, &arc02);
            (res, 3)
        }
        ArcArcConfig::CocircularTwoArcs(_, _) => {
            let mut arc00 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
            let mut arc01 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
            arc00.id(arc0.id);
            arc01.id(arc1.id);
            check_and_push(&mut res, &arc00);
            check_and_push(&mut res, &arc01);
            (res, 2)
        }
    }
}

pub fn split_segment_arc(line0: &Arc, arc1: &Arc) -> (Vec<Arc>, usize) {
    debug_assert!(line0.is_seg());
    debug_assert!(arc1.is_arc());
    let mut res = Vec::new();
    let segment = segment(line0.a, line0.b);
    let inter = int_segment_arc(&segment, arc1);
    match inter {
        SegmentArcConfig::NoIntersection()
        | SegmentArcConfig::OnePointTouching(_, _)
        | SegmentArcConfig::TwoPointsTouching(_, _, _, _) => {
            (res, 0)
        }
        SegmentArcConfig::OnePoint(point, _) => {
            let mut line00 = arcseg(line0.a, point);
            let mut line01 = arcseg(point, line0.b);
            let mut arc10 = arc(arc1.a, point, arc1.c, arc1.r);
            let mut arc11 = arc(point, arc1.b, arc1.c, arc1.r);
            line00.id(line0.id);
            line01.id(line0.id);
            arc10.id(arc1.id);
            arc11.id(arc1.id);
            check_and_push(&mut res, &line00);
            check_and_push(&mut res, &line01);
            check_and_push(&mut res, &arc10);
            check_and_push(&mut res, &arc11);
            (res, 4)
        }
        SegmentArcConfig::TwoPoints(point0, point1, _, _) => {
            let mut p0 = point0;
            let mut p1 = point1;
            let mut line00 = arcseg(line0.a, p0);
            let mut line01 = arcseg(p0, p1);
            let mut line02 = arcseg(p1, line0.b);
            if points_order(arc1.a, p0, p1) < ZERO {
                (p1, p0) = (p0, p1);
            }
            let mut arc10 = arc(arc1.a, p0, arc1.c, arc1.r);
            let mut arc11 = arc(p0, p1, arc1.c, arc1.r);
            let mut arc12 = arc(p1, arc1.b, arc1.c, arc1.r);
            line00.id(line0.id);
            line01.id(line0.id);
            line02.id(line0.id);
            arc10.id(arc1.id);
            arc11.id(arc1.id);
            arc12.id(arc1.id);
            check_and_push(&mut res, &line00);
            check_and_push(&mut res, &line01);
            check_and_push(&mut res, &line02);
            check_and_push(&mut res, &arc10);
            check_and_push(&mut res, &arc11);
            check_and_push(&mut res, &arc12);
            (res, 6)
        }
    }
}

fn check_and_push(res: &mut Vec<Arc>, seg: &Arc) {
    let eps = 1e-10;
    if seg.is_valid(eps) {
        res.push(seg.clone())
    }
}

