#![allow(dead_code)]

use crate::int_interval_interval::{intersect_interval_interval, IntervalConfig};
use crate::int_line_line::{intersect_line_line, LineConfig};
use crate::interval::interval;

use crate::{line::line, point::Point, segment::Segment};

#[derive(Debug, PartialEq)]
pub enum SegmentConfig {
    NoIntersection(),
    OnePoint(Point, f64, f64),

    TwoPoints(Point, Point, Point, Point),
}

pub fn intersect_segment_segment(segment0: Segment, segment1: Segment) -> SegmentConfig {
    let (seg0_origin, seg0_direction, seg0_extent) = segment0.get_centered_form();
    let (seg1_origin, seg1_direction, seg1_extent) = segment1.get_centered_form();
    let line0 = line(seg0_origin, seg0_direction);
    let line1 = line(seg1_origin, seg1_direction);

    let ll_result = intersect_line_line(line0, line1);

    match ll_result {
        LineConfig::ParallelDistinct() => return SegmentConfig::NoIntersection(),
        LineConfig::OnePoint(p, s0, s1) => {
            if s0.abs() <= seg0_extent && s1.abs() <= seg1_extent {
                return SegmentConfig::OnePoint(p, s0, s1);
            } else {
                return SegmentConfig::NoIntersection();
            }
        }
        LineConfig::ParallelTheSame() => {
            let diff = seg1_origin - seg0_origin;
            let t = seg0_direction.dot(diff);
            let interval0 = interval(-seg0_extent, seg0_extent);
            let interval1 = interval(t - seg1_extent, t + seg1_extent);

            let ii_result = intersect_interval_interval(interval0, interval1);
            match ii_result {
                IntervalConfig::NoOverlap() => return SegmentConfig::NoIntersection(),
                IntervalConfig::Overlap(_, _) => {
                    let (p0, p1, p2, p3) = Point::sort_parallel_points(
                        segment0.p0,
                        segment0.p1,
                        segment1.p0,
                        segment1.p1,
                    );
                    return SegmentConfig::TwoPoints(p0, p1, p2, p3);
                }
                IntervalConfig::Touching(_) => {
                    return SegmentConfig::NoIntersection();
                }
            }
        }
    }
}

pub fn is_touching_segment_segment(s0: Segment, s1: Segment) -> bool {
    if s0.p0 == s1.p0 || s0.p0 == s1.p1 || s0.p1 == s1.p0 || s0.p1 == s1.p1 {
        true
    } else {
        false
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
            intersect_segment_segment(s0, s1),
            SegmentConfig::NoIntersection()
        );
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
            intersect_segment_segment(s0, s1),
            SegmentConfig::NoIntersection()
        );
    }

    #[test]
    fn test_parallel_overlaping() {
        let ulp = std::f64::EPSILON * 2.0;
        let s0 = segment(point(0.0, 0.0), point(2.0, 2.0));
        let s1 = segment(point(1.0, 1.0), point(3.0, 3.0));
        match intersect_segment_segment(s0, s1) {
            SegmentConfig::TwoPoints(p0, p1, p2, p3) => {
                assert!(p0.close_enough(point(0.0, 0.0), ulp));
                assert!(p1.close_enough(point(1.0, 1.0), ulp));
                assert!(p2.close_enough(point(2.0, 2.0), ulp));
                assert!(p3.close_enough(point(3.0, 3.0), ulp));
            }
            _ => panic!("Unexpected SegmentConfig variant"),
        }
    }

    #[test]
    fn test_parallel_overlaping2() {
        let ulp = std::f64::EPSILON * 3.0;
        let s0 = segment(point(0.0, 0.0), point(2.0, 2.0));
        let s1 = segment(point(4.0, 4.0), point(-4.0, -4.0));
        match intersect_segment_segment(s0, s1) {
            SegmentConfig::TwoPoints(p0, p1, p2, p3) => {
                assert!(p0.close_enough(point(4.0, 4.0), ulp));
                assert!(p1.close_enough(point(2.0, 2.0), ulp));
                assert!(p2.close_enough(point(0.0, 0.0), ulp));
                assert!(p3.close_enough(point(-4.0, -4.0), ulp));
            }
            _ => panic!("Unexpected SegmentConfig variant"),
        }
    }

    #[test]
    fn test_parallel_touching() {
        let s0 = segment(point(0.0, 0.0), point(1.0, 0.0));
        let s1 = segment(point(1.0, 0.0), point(4.0, 0.0));
        assert!(intersect_segment_segment(s0, s1) == SegmentConfig::NoIntersection());
    }

    #[test]
    fn test_touching_at_ends() {
        let sqrt_2 = std::f64::consts::SQRT_2;
        let s0 = segment(point(0.0, 0.0), point(2.0, 2.0));
        let s1 = segment(point(2.0, 2.0), point(4.0, 0.0));
        assert_eq!(
            intersect_segment_segment(s0, s1),
            SegmentConfig::OnePoint(point(2.0, 2.0), sqrt_2, -sqrt_2)
        );
    }
}
