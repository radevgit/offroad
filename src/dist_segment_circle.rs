#![allow(dead_code)]

use crate::{
    dist_line_circle::{dist_line_circle, DistLineCircleConfig},
    dist_point_circle::dist_point_circle,
    line::line,
    segment::Segment,
    Circle, Point,
};

#[derive(Debug, PartialEq)]
pub enum DistSegmentCircleConfig {
    OnePoint(f64, Point),
    TwoPoints(f64, Point, Point),
}

const ZERO: f64 = 0.0;
const ONE: f64 = 1.0;

pub fn dist_segment_circle(seg: &Segment, circle: &Circle) -> DistSegmentCircleConfig {
    let (dir, _) = (seg.b - seg.a).normalize();
    let line = line(seg.a, seg.b - seg.a);
    let dlc = dist_line_circle(&line, &circle);
    match dlc {
        DistLineCircleConfig::TwoPairs(
            _,
            param0,
            param1,
            closest00,
            closest01,
            closest10,
            closest11,
        ) => {
            if param0 > ONE && param1 > ONE {
                let (dist, p0, _) = dist_to_circle(seg, circle);
                DistSegmentCircleConfig::OnePoint(dist, p0)
            } else if param0 >= ZERO && param0 <= ONE && param1 > ONE {
                return DistSegmentCircleConfig::OnePoint(ZERO, closest01);
            } else if param0 >= ZERO && param0 <= ONE && param1 >= ZERO && param1 <= ONE {
                return DistSegmentCircleConfig::TwoPoints(ZERO, closest01, closest11);
            } else if param0 < ZERO && param1 > ONE {
                let (dist, p0, _) = dist_to_circle(seg, circle);
                return DistSegmentCircleConfig::OnePoint(dist, p0);
            } else if param0 < ZERO && param1 >= ZERO && param1 <= ONE {
                return DistSegmentCircleConfig::OnePoint(ZERO, closest11);
            } else {
                let (dist, p0, _) = dist_to_circle(seg, circle);
                DistSegmentCircleConfig::OnePoint(dist, p0)
            }
        }
        DistLineCircleConfig::OnePair(d, param, closest0, closest1) => {
            if param < ZERO {
                let (dist, closest, _) = dist_point_circle(&seg.a, &circle);
                DistSegmentCircleConfig::OnePoint(dist, closest)
            } else if param > ONE {
                let (dist0, closest0, _) = dist_point_circle(&seg.a, &circle);
                let (dist1, closest1, _) = dist_point_circle(&seg.b, &circle);
                if dist0 <= dist1 {
                    DistSegmentCircleConfig::OnePoint(dist0, closest0)
                } else {
                    DistSegmentCircleConfig::OnePoint(dist1, closest1)
                }
            } else {
                DistSegmentCircleConfig::OnePoint(d, closest1)
            }
        }
    }
}

fn dist_to_circle(seg: &Segment, circle: &Circle) -> (f64, Point, Point) {
    let (dist0, p0, _) = dist_point_circle(&seg.a, &circle);
    let (dist1, p1, _) = dist_point_circle(&seg.b, &circle);
    if dist0 <= dist1 {
        (dist0, p0, p1)
    } else {
        (dist1, p1, p0)
    }
}

#[cfg(test)]
mod test_dist_segment_circle {
    use std::f64::{consts::SQRT_2, EPSILON};

    use crate::{
        circle::circle,
        dist_segment_circle::DistSegmentCircleConfig,
        point::point,
        segment::{segment, Segment},
    };

    fn rev(seg: Segment) -> Segment {
        segment(seg.b, seg.a)
    }

    #[test]
    fn test_p0_p1_outside_segment_outside_circle_01() {
        let c = circle(point(0.0, 0.0), 2.0);
        let seg = segment(point(-5.0, 1.0), point(-4.0, 1.0));
        let dist = super::dist_segment_circle(&seg, &c);
        assert_eq!(
            dist,
            DistSegmentCircleConfig::OnePoint(
                2.1231056256176606,
                point(-1.9402850002906638, 0.48507125007266594)
            )
        );
        let seg = rev(seg);
        let dist = super::dist_segment_circle(&seg, &c);
        assert_eq!(
            dist,
            DistSegmentCircleConfig::OnePoint(
                2.1231056256176606,
                point(-1.9402850002906638, 0.48507125007266594)
            )
        );
    }

    #[test]
    fn test_p0_outside_p1_inside_circle() {
        let c = circle(point(0.0, 0.0), 2.0);
        let seg = segment(point(-3.0, 1.0), point(1.0, 1.0));
        let dist = super::dist_segment_circle(&seg, &c);
        assert_eq!(
            dist,
            DistSegmentCircleConfig::OnePoint(0.0, point(-1.7320508075688772, 1.0))
        );
        let seg = rev(seg);
        let dist = super::dist_segment_circle(&seg, &c);
        assert_eq!(
            dist,
            DistSegmentCircleConfig::OnePoint(0.0, point(-1.7320508075688772, 1.0))
        );
    }

    #[test]
    fn test_p0_outside_p1_outside_segment_inside_circle() {
        let c = circle(point(0.0, 0.0), 2.0);
        let seg = segment(point(-3.0, 1.0), point(3.0, 1.0));
        let dist = super::dist_segment_circle(&seg, &c);
        assert_eq!(
            dist,
            DistSegmentCircleConfig::TwoPoints(
                0.0,
                point(-1.7320508075688774, 1.0),
                point(1.7320508075688767, 1.0)
            )
        );
        let seg = rev(seg);
        let dist = super::dist_segment_circle(&seg, &c);
        assert_eq!(
            dist,
            DistSegmentCircleConfig::TwoPoints(
                0.0,
                point(1.7320508075688774, 1.0),
                point(-1.7320508075688767, 1.0)
            )
        );
    }

    #[test]
    fn test_p0_inside_p1_inside_circle() {
        let c = circle(point(0.0, 0.0), 2.0);
        let seg = segment(point(-1.0, 1.0), point(1.0, 1.0));
        let dist = super::dist_segment_circle(&seg, &c);
        assert_eq!(
            dist,
            DistSegmentCircleConfig::OnePoint(
                0.5857864376269049,
                point(-1.414213562373095, 1.414213562373095)
            )
        );
        let seg = rev(seg);
        let dist = super::dist_segment_circle(&seg, &c);
        assert_eq!(
            dist,
            DistSegmentCircleConfig::OnePoint(
                0.5857864376269049,
                point(1.414213562373095, 1.414213562373095)
            )
        );
    }
}
