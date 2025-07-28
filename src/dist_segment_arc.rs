#![allow(dead_code)]

use crate::{
    circle::circle,
    dist_line_circle::{dist_line_circle, DistLineCircleConfig},
    dist_point_arc::{dist_point_arc, dist_point_arc_dist, DistPointArcConfig},
    dist_point_segment::dist_point_segment,
    dist_segment_circle::{dist_segment_circle, DistSegmentCircleConfig},
    int_segment_arc::{int_segment_arc, SegmentArcConfig},
    segment::Segment,
    utils::min_4,
    Arc,
};

pub fn dist_segment_arc(seg: &Segment, arc: &Arc) -> f64 {
    let res = int_segment_arc(seg, arc);
    match res {
        SegmentArcConfig::NoIntersection() => {
            let (dist0, _) = dist_point_segment(&arc.a, seg);
            let (dist1, _) = dist_point_segment(&arc.b, seg);
            let dist2 = dist_point_arc_dist(&seg.a, arc);
            let dist3 = dist_point_arc_dist(&seg.b, arc);
            let dist = min_4(dist0, dist1, dist2, dist3);
            let line = crate::line::line(seg.a, seg.b - seg.a);
            let circle = circle(arc.c, arc.r);
            let res2 = dist_line_circle(&line, &circle);
            let dist4 = match res2 {
                DistLineCircleConfig::OnePair(_, param, closest0, closest1) => {
                    if param >= 0.0 && param <= 1.0 && arc.contains(closest1) {
                        (closest0 - closest1).norm()
                    } else {
                        f64::MAX
                    }
                }
                _ => f64::MAX,
            };
            return f64::min(dist, dist4);
        }
        _ => {
            return 0.0;
        }
    }
}

#[cfg(test)]
mod tests_distance_segment_arc {
    use crate::{arc::arc, point::point, segment::segment};

    #[test]
    fn test_segment_outside_circle_01() {
        let seg = segment(point(-1.0, 2.0), point(1.0, 2.0));
        let arc = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 1.0);
    }

    #[test]
    fn test_segment_outside_circle_02() {
        let seg = segment(point(-1.0, 2.0), point(1.0, 2.0));
        let arc = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 2.0);
    }

    #[test]
    fn test_segment_inside_circle_01() {
        let seg = segment(point(-2.0, 0.0), point(0.5, 0.0));
        let arc = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 0.0);
    }

    #[test]
    fn test_segment_inside_circle_02() {
        let seg = segment(point(-0.5, 0.0), point(2.0, 0.0));
        let arc = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 0.0);
    }

    #[test]
    fn test_segment_inside_circle_03() {
        let seg = segment(point(-2.0, 0.0), point(2.0, 0.0));
        let arc = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 0.0);
    }

    #[test]
    fn test_segment_inside_circle_04() {
        let seg = segment(point(-0.5, 0.0), point(2.0, 0.0));
        let arc = arc(point(0.0, 1.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 0.5);
    }

    #[test]
    fn test_segment_inside_circle_05() {
        let seg = segment(point(-2.0, 0.0), point(0.5, 0.0));
        let arc = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 0.5);
    }

    #[test]
    fn test_segment_inside_circle_06() {
        let seg = segment(point(-2.0, 0.0), point(2.0, 0.0));
        let arc = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 0.0), 2.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 1.0);
    }

    #[test]
    fn test_segment_inside_circle_07() {
        let seg = segment(point(0.0, 0.0), point(2.0, 0.0));
        let arc = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 0.0), 2.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 1.0);
    }

    #[test]
    fn test_segment_inside_circle_08() {
        let seg = segment(point(-2.0, 0.0), point(0.0, 0.0));
        let arc = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 0.0), 2.0);
        let res = super::dist_segment_arc(&seg, &arc);
        assert_eq!(res, 1.0);
    }
}
