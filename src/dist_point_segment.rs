#![allow(dead_code)]

use crate::segment::Segment;
use crate::Point;

pub fn dist_point_segment(point: &Point, segment: &Segment) -> (f64, Point) {
    let closest;
    const ZERO: f64 = 0f64;
    const ONE: f64 = 1f64;
    let direction = segment.b - segment.a;
    let mut diff = point - segment.b;
    let mut t = direction.dot(diff);
    if t >= ZERO {
        closest = segment.b;
    } else {
        diff = point - segment.a;
        t = direction.dot(diff);
        if t <= ZERO {
            closest = segment.a;
        } else {
            let sqr_length = direction.dot(direction);
            if sqr_length > ZERO {
                t = t / sqr_length;
                closest = segment.a + direction * t;
            } else {
                closest = segment.a;
            }
        }
    }

    ((point - closest).norm(), closest)
}

#[cfg(test)]
mod test_dist_point_segment {
    use crate::{point::point, segment::segment};

    #[test]
    fn test_point_at_end_01() {
        let p = point(0.0, 0.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 0.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, 0.0);
        assert_eq!(closest, point(0.0, 0.0));
    }

    #[test]
    fn test_point_at_end_02() {
        let p = point(1.0, 0.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 0.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, 0.0);
        assert_eq!(closest, point(1.0, 0.0));
    }

    #[test]
    fn test_point_inside_segment() {
        let p = point(0.5, 0.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 0.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, 0.0);
        assert_eq!(closest, point(0.5, 0.0));
    }

    #[test]
    fn test_point_segment_01() {
        let p = point(0.0, 1.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 1.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, std::f64::consts::SQRT_2 / 2.0);
        assert_eq!(closest, point(0.5, 0.5));
    }

    #[test]
    fn test_point_close_to_a_01() {
        let p = point(-1.0, 1.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 1.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, std::f64::consts::SQRT_2);
        assert_eq!(closest, point(0.0, 0.0));
    }

    #[test]
    fn test_point_close_to_a_02() {
        let p = point(1.0, -1.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 1.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, std::f64::consts::SQRT_2);
        assert_eq!(closest, point(0.0, 0.0));
    }

    #[test]
    fn test_point_close_to_a_03() {
        let p = point(-1.0, -1.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 1.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, std::f64::consts::SQRT_2);
        assert_eq!(closest, point(0.0, 0.0));
    }

    #[test]
    fn test_point_close_to_b_01() {
        let p = point(0.0, 2.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 1.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, std::f64::consts::SQRT_2);
        assert_eq!(closest, point(1.0, 1.0));
    }

    #[test]
    fn test_point_close_to_b_02() {
        let p = point(2.0, 0.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 1.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, std::f64::consts::SQRT_2);
        assert_eq!(closest, point(1.0, 1.0));
    }

    #[test]
    fn test_point_close_to_b_03() {
        let p = point(2.0, 2.0);
        let seg = segment(point(0.0, 0.0), point(1.0, 1.0));
        let (dist, closest) = super::dist_point_segment(&p, &seg);
        assert_eq!(dist, std::f64::consts::SQRT_2);
        assert_eq!(closest, point(1.0, 1.0));
    }
}
