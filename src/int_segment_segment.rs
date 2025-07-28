#![allow(dead_code)]

use crate::int_interval_interval::{int_interval_interval, IntervalConfig};
use crate::int_line_line::{int_line_line, LineConfig};
use crate::interval::interval;

use crate::{line::line, point::Point, segment::Segment};

#[derive(Debug, PartialEq)]
pub enum SegmentSegmentConfig {
    NoIntersection(),
    OnePoint(Point, f64, f64),
    OnePointTouching(Point, f64, f64),
    TwoPoints(Point, Point, Point, Point),
    TwoPointsTouching(Point, Point, Point, Point),
}

const ZERO: f64 = 0f64;
pub fn int_segment_segment(segment0: &Segment, segment1: &Segment) -> SegmentSegmentConfig {
    let (seg0_origin, seg0_direction, seg0_extent) = segment0.get_centered_form();
    let (seg1_origin, seg1_direction, seg1_extent) = segment1.get_centered_form();

    let line0 = line(seg0_origin, seg0_direction);
    let line1 = line(seg1_origin, seg1_direction);

    let ll_result = int_line_line(&line0, &line1);

    match ll_result {
        LineConfig::ParallelDistinct() => return SegmentSegmentConfig::NoIntersection(),
        LineConfig::OnePoint(p, s0, s1) => {
            if s0.abs() <= seg0_extent && s1.abs() <= seg1_extent {
                if are_ends_towching(&segment0, &segment1) {
                    return SegmentSegmentConfig::OnePointTouching(p, s0, s1);
                } else {
                    return SegmentSegmentConfig::OnePoint(p, s0, s1);
                }
            } else {
                return SegmentSegmentConfig::NoIntersection();
            }
        }
        LineConfig::ParallelTheSame() => {
            let diff = seg1_origin - seg0_origin;
            let t = seg0_direction.dot(diff);
            let interval0 = interval(-seg0_extent, seg0_extent);
            let interval1 = interval(t - seg1_extent, t + seg1_extent);

            let ii_result = int_interval_interval(interval0, interval1);
            match ii_result {
                IntervalConfig::NoOverlap() => return SegmentSegmentConfig::NoIntersection(),
                IntervalConfig::Overlap(_, _) => {
                    let (p0, p1, p2, p3) =
                        Point::sort_parallel_points(segment0.a, segment0.b, segment1.a, segment1.b);
                    if are_both_ends_towching(&segment0, &segment1) {
                        return SegmentSegmentConfig::TwoPointsTouching(
                            segment0.a, segment0.b, segment1.a, segment1.b,
                        );
                    }
                    if are_ends_towching(&segment0, &segment1) {
                        return SegmentSegmentConfig::NoIntersection();
                    } else {
                        return SegmentSegmentConfig::TwoPoints(p0, p1, p2, p3);
                    }
                }
                IntervalConfig::Touching(_) => {
                    return SegmentSegmentConfig::NoIntersection();
                }
            }
        }
    }
}

fn are_ends_towching(segment0: &Segment, segment1: &Segment) -> bool {
    if segment0.a == segment1.a
        || segment0.a == segment1.b
        || segment0.b == segment1.a
        || segment0.b == segment1.b
    {
        true
    } else {
        false
    }
}

fn are_both_ends_towching(segment0: &Segment, segment1: &Segment) -> bool {
    (segment0.a == segment1.a && segment0.b == segment1.b)
        || (segment0.b == segment1.a && segment0.a == segment1.b)
}

pub fn if_really_intersecting_segment_segment(part0: &Segment, part1: &Segment) -> bool {
    match int_segment_segment(&part0, &part1) {
        SegmentSegmentConfig::NoIntersection() => false,
        SegmentSegmentConfig::OnePoint(_, _, _) => true,
        SegmentSegmentConfig::OnePointTouching(_, _, _) => false,
        SegmentSegmentConfig::TwoPoints(_, _, _, _) => true,
        SegmentSegmentConfig::TwoPointsTouching(_, _, _, _) => false,
    }
}

#[cfg(test)]
mod test_int_segment_segment {
    use crate::point::point;
    use crate::segment::segment;

    use super::*;

    #[test]
    fn test_no_intersection() {
        let s0 = segment(point(0.0, 0.0), point(2.0, 2.0));
        let s1 = segment(point(2.0, 1.0), point(4.0, -1.0));
        assert_eq!(
            int_segment_segment(&s0, &s1),
            SegmentSegmentConfig::NoIntersection()
        );
        assert!(if_really_intersecting_segment_segment(&s0, &s1) == false);
    }

    #[test]
    fn test_no_intersection_parallel() {
        let s0 = segment(point(0.0, 0.0), point(0.0, 2.0));
        let s1 = segment(point(1.0, 0.0), point(1.0, 2.0));
        assert_eq!(
            int_segment_segment(&s0, &s1),
            SegmentSegmentConfig::NoIntersection()
        );
        assert!(if_really_intersecting_segment_segment(&s0, &s1) == false);
    }

    #[test]
    fn test_no_intersection2() {
        let sqrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let p0 = point(0.0, 0.0);
        let p1 = point(sqrt_2_2, sqrt_2_2);
        let delta = point(f64::EPSILON, 0.0);
        let s0 = segment(p0, p1);
        let s1 = segment(p0 + delta, p1 + delta);
        assert_eq!(
            int_segment_segment(&s0, &s1),
            SegmentSegmentConfig::NoIntersection()
        );
        assert!(if_really_intersecting_segment_segment(&s0, &s1) == false);
    }

    #[test]
    fn test_parallel_overlaping() {
        let ulp = std::f64::EPSILON * 2.0;
        let s0 = segment(point(0.0, 0.0), point(2.0, 2.0));
        let s1 = segment(point(1.0, 1.0), point(3.0, 3.0));
        match int_segment_segment(&s0, &s1) {
            SegmentSegmentConfig::TwoPoints(p0, p1, p2, p3) => {
                assert!(p0.close_enough(point(0.0, 0.0), ulp));
                assert!(p1.close_enough(point(1.0, 1.0), ulp));
                assert!(p2.close_enough(point(2.0, 2.0), ulp));
                assert!(p3.close_enough(point(3.0, 3.0), ulp));
                assert!(if_really_intersecting_segment_segment(&s0, &s1) == true);
            }
            _ => panic!("Unexpected SegmentConfig variant"),
        }
    }

    #[test]
    fn test_parallel_overlaping2() {
        let ulp = std::f64::EPSILON * 3.0;
        let s0 = segment(point(0.0, 0.0), point(2.0, 2.0));
        let s1 = segment(point(4.0, 4.0), point(-4.0, -4.0));
        match int_segment_segment(&s0, &s1) {
            SegmentSegmentConfig::TwoPoints(p0, p1, p2, p3) => {
                assert!(p0.close_enough(point(4.0, 4.0), ulp));
                assert!(p1.close_enough(point(2.0, 2.0), ulp));
                assert!(p2.close_enough(point(0.0, 0.0), ulp));
                assert!(p3.close_enough(point(-4.0, -4.0), ulp));
                assert!(if_really_intersecting_segment_segment(&s0, &s1) == true);
            }
            _ => panic!("Unexpected SegmentConfig variant"),
        }
    }

    #[test]
    fn test_parallel_touching() {
        let s0 = segment(point(0.0, 0.0), point(1.0, 0.0));
        let s1 = segment(point(1.0, 0.0), point(4.0, 0.0));
        assert!(int_segment_segment(&s0, &s1) == SegmentSegmentConfig::NoIntersection());
        assert!(if_really_intersecting_segment_segment(&s0, &s1) == false);
    }

    #[test]
    fn test_touching_at_ends() {
        let sqrt_2 = std::f64::consts::SQRT_2;
        let s0 = segment(point(0.0, 0.0), point(2.0, 2.0));
        let s1 = segment(point(2.0, 2.0), point(4.0, 0.0));
        assert_eq!(
            int_segment_segment(&s0, &s1),
            SegmentSegmentConfig::OnePointTouching(point(2.0, 2.0), sqrt_2, -sqrt_2)
        );
        assert!(if_really_intersecting_segment_segment(&s0, &s1) == false);
    }

    #[test]
    #[ignore = "reason"]
    fn test_zero_size_segment_outside_segment() {
        let s0 = segment(point(2.0, 0.0), point(2.0, 0.0));
        let s1 = segment(point(0.0, 0.0), point(1.0, 0.0));
        assert_eq!(
            int_segment_segment(&s0, &s1),
            SegmentSegmentConfig::NoIntersection()
        );
        assert_eq!(
            int_segment_segment(&s1, &s0),
            SegmentSegmentConfig::NoIntersection()
        );
    }

    #[test]
    #[ignore = "reason"]
    fn test_zero_size_segment_inside_segment() {
        let s0 = segment(point(1.0, 0.0), point(1.0, 0.0));
        let s1 = segment(point(0.0, 0.0), point(2.0, 0.0));
        assert_eq!(
            int_segment_segment(&s0, &s1),
            SegmentSegmentConfig::OnePoint(point(1.0, 0.0), ZERO, ZERO)
        );
        assert_eq!(
            int_segment_segment(&s1, &s0),
            SegmentSegmentConfig::OnePoint(point(1.0, 0.0), ZERO, ZERO)
        );
    }

    #[test]
    fn test_both_zero_size_segments_outside() {
        let s0 = segment(point(2.0, 0.0), point(2.0, 0.0));
        let s1 = segment(point(1.0, 0.0), point(1.0, 0.0));
        assert_eq!(
            int_segment_segment(&s0, &s1),
            SegmentSegmentConfig::NoIntersection()
        );
        assert_eq!(
            int_segment_segment(&s1, &s0),
            SegmentSegmentConfig::NoIntersection()
        );
    }
}
