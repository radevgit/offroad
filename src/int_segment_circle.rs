#![allow(dead_code)]

use crate::int_interval_interval::{intersect_interval_interval, IntervalConfig};
use crate::int_line_circle::LineCircleConfig;
use crate::interval::interval;
use crate::line::line;
use crate::{circle::Circle, int_line_circle::intersect_line_circle, segment::Segment, Point};

#[derive(Debug, PartialEq)]
pub enum SegmentCircleConfig {
    NoIntersection(),
    OnePoint(Point, f64),
    TwoPoints(Point, Point, f64, f64),
}


pub fn intersect_segment_circle(seg: Segment, circle: Circle) -> SegmentCircleConfig {
    let (seg_origin, seg_direction, seg_extent) = seg.get_centered_form();
    let lc_res = intersect_line_circle(line(seg_origin, seg_direction), circle);
    match lc_res {
        LineCircleConfig::NoIntersection() => return SegmentCircleConfig::NoIntersection(),

        LineCircleConfig::OnePoint(p0, param0) => {
            let seg_interval = interval(-seg_extent, seg_extent);
            if seg_interval.contains(param0) {
                return SegmentCircleConfig::OnePoint(p0, param0);
            } else {
                return SegmentCircleConfig::NoIntersection();
            }
        }

        LineCircleConfig::TwoPoints(p0, p1, param0, param1) => {
            let seg_interval = interval(-seg_extent, seg_extent);
            let b0 = seg_interval.contains(param0);
            let b1 = seg_interval.contains(param1);
            if b0 && b1 {
                return SegmentCircleConfig::TwoPoints(p0, p1, param0, param1);
            }
            if b0 {
                return SegmentCircleConfig::OnePoint(p0, param0);
            }
            if b1 {
                return SegmentCircleConfig::OnePoint(p1, param1);
            }
            return SegmentCircleConfig::NoIntersection();
        }
    }
}

#[cfg(test)]
mod tests_segment_circle {
    use crate::{circle::circle, point::point, segment::segment, utils::perturbed_ulps_as_int};

    use super::*;

    #[test]
    fn test_no_intersection() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let s0 = segment(point(0.0, 0.0), point(sgrt_2_2, sgrt_2_2));
        let c0 = circle(point(3.0, 1.0), 1.0);
        assert_eq!(
            intersect_segment_circle(s0, c0),
            SegmentCircleConfig::NoIntersection()
        );
    }

    #[test]
    fn test_interval_degenerate() {
        let s0 = segment(point(-1.0, 1.0), point(1.0, 1.0));
        let c0 = circle(point(0.0, 0.0), 1.0);
        assert_eq!(
            intersect_segment_circle(s0, c0),
            SegmentCircleConfig::OnePoint(point(0.0, 1.0), 0.0)
        );
    }

    #[test]
    fn test_one_point() {
        let s0 = segment(point(-1.0, 1.0), point(-0.0, 1.0));
        let c0 = circle(point(0.0, 0.0), 1.0);
        assert_eq!(
            intersect_segment_circle(s0, c0),
            SegmentCircleConfig::OnePoint(point(0.0, 1.0), 0.5)
        );
    }

    #[test]
    fn test_one_point2() {
        // Segment touches circle.
        let s0 = segment(point(-2.0, 0.0), point(-1.0, 0.0));
        let c0 = circle(point(0.0, 0.0), 1.0);
        let res = intersect_segment_circle(s0, c0);
        assert_eq!(res, SegmentCircleConfig::OnePoint(point(-1.0, 0.0), 0.5)); // TODO it should be param: 1.0?
    }

    #[test]
    fn test_two_points() {
        let _1_eps = perturbed_ulps_as_int(1.0, -1);
        let s0 = segment(point(-1.0, _1_eps), point(1.0, _1_eps));
        let c0 = circle(point(0.0, 0.0), 1.0);
        let res = intersect_segment_circle(s0, c0);
        if let SegmentCircleConfig::TwoPoints(p0, p1, t0, t1) = res {
            assert_eq!(p0.y, _1_eps);
            assert_eq!(p1.y, _1_eps);
            assert_eq!(p0.x + p1.x, 0.0);
            assert_eq!(t0 + t1, 0.0);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_two_points_issue() {
        let s0 = segment(point(144.0, 192.0), point(144.0, 205.0));
        let c0 = circle(point(136.0, 197.0), 16.0);
        let res = intersect_segment_circle(s0, c0);
        assert_eq!(res, SegmentCircleConfig::NoIntersection());
    }
}
