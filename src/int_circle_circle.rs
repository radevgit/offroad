#![allow(dead_code)]

use crate::point::point;
use crate::{circle::Circle, point::Point};

#[derive(Debug, PartialEq)]
pub enum CircleConfig {
    NoIntersection(),
    NoncocircularOnePoint(Point),         
    NoncocircularTwoPoints(Point, Point),
    SameCircles(),
}

pub fn intersect_circle_circle(circle0: Circle, circle1: Circle) -> CircleConfig {
    const ZERO: f64 = 0f64;
    debug_assert!(circle0.r.is_finite());
    debug_assert!(circle1.r.is_finite());

    let u = circle1.c - circle0.c;
    let usqr_len = u.dot(u);
    let r0 = circle0.r;
    let r1 = circle1.r;
    let r0_m_r1 = r0 - r1;

    if usqr_len == ZERO && r0_m_r1 == ZERO {
        return CircleConfig::SameCircles();
    }

    let r0_m_r1_sqr = r0_m_r1 * r0_m_r1;
    if usqr_len < r0_m_r1_sqr {
        return CircleConfig::NoIntersection();
    }

    let r0_p_r1 = r0 + r1;
    let r0_p_r1_sqr = r0_p_r1 * r0_p_r1;
    if usqr_len > r0_p_r1_sqr {
        return CircleConfig::NoIntersection();
    }

    if usqr_len < r0_p_r1_sqr {
        if r0_m_r1_sqr < usqr_len {
            let inv_usqr_len = 1.0 / usqr_len;
            let s = 0.5 * ((r0 * r0 - r1 * r1) * inv_usqr_len + 1.0);
            let mut discr = r0 * r0 * inv_usqr_len - s * s;
            if discr < 0f64 {
                discr = 0f64;
            }
            let t = discr.sqrt();
            let v = point(u.y, -u.x);
            let tmp = circle0.c + u * s;
            let p0 = tmp - v * t;
            let p1 = tmp + v * t;
            if t > 0f64 {
                return CircleConfig::NoncocircularTwoPoints(p0, p1);
            } else {
                // t==0.0
                return CircleConfig::NoncocircularOnePoint(p0);
            }
        } else {
            let p0 = circle0.c + u * (r0 / r0_m_r1);
            return CircleConfig::NoncocircularOnePoint(p0);
        }
    } else {
        let p0 = circle0.c + u * (r0 / r0_p_r1);
        return CircleConfig::NoncocircularOnePoint(p0);
    }
}


#[cfg(test)]
mod tests_circle {
    use super::*;
    use crate::circle::circle;
    use crate::utils::perturbed_ulps_as_int;

    fn ff(circle0: Circle, circle1: Circle) -> CircleConfig {
        intersect_circle_circle(circle0, circle1)
    }

    #[test]
    fn test_same_circles01() {
        let circle0 = circle(point(0.0, 0.0), 1.0);
        let circle1 = circle(point(0.0, 0.0), 1.0);
        assert_eq!(ff(circle0, circle1), CircleConfig::SameCircles());
    }
    #[test]
    fn test_same_circles02() {
        let circle0 = circle(point(0.0, 0.0), 0.0);
        let circle1 = circle(point(0.0, 0.0), 0.0);
        assert_eq!(ff(circle0, circle1), CircleConfig::SameCircles());
    }
    #[test]
    fn test_same_circles03() {
        let circle0 = circle(point(0.0, 0.0), f64::MAX);
        let circle1 = circle(point(0.0, 0.0), f64::MAX);
        assert_eq!(ff(circle0, circle1), CircleConfig::SameCircles());
    }
    #[test]
    fn test_same_circles04() {
        let circle0 = circle(point(f64::MAX, f64::MAX), f64::MAX);
        let circle1 = circle(point(f64::MAX, f64::MAX), f64::MAX);
        assert_eq!(ff(circle0, circle1), CircleConfig::SameCircles());
    }

    #[test]
    fn test_donot_intersect01() {
        let r = perturbed_ulps_as_int(1.0, -2);
        let circle0 = circle(point(-1.0, 0.0), r);
        let circle1 = circle(point(1.0, 0.0), 1.0);
        assert_eq!(ff(circle0, circle1), CircleConfig::NoIntersection());
    }
    #[test]
    fn test_donot_intersect02() {
        let x = perturbed_ulps_as_int(1.0, 2);
        let circle0 = circle(point(-1.0, 0.0), 1.0);
        let circle1 = circle(point(x, 0.0), 1.0);
        assert_eq!(ff(circle0, circle1), CircleConfig::NoIntersection());
    }

    #[test]
    fn test_tangent01() {
        let x = perturbed_ulps_as_int(1.0, 1);
        let circle0 = circle(point(-1.0, 0.0), 1.0);
        let circle1 = circle(point(x, 0.0), 1.0);
        assert_eq!(
            ff(circle0, circle1),
            CircleConfig::NoncocircularOnePoint(point(0.0, 0.0))
        );
    }

    #[test]
    fn test_tangent02() {
        let circle0 = circle(point(1.0, 0.0), 1.0);
        let circle1 = circle(point(1.0 + f64::EPSILON, 0.0), 1.0);
        let res = intersect_circle_circle(circle0, circle1);
        assert_eq!(
            res,
            CircleConfig::NoncocircularTwoPoints(point(1.0, 1.0), point(1.0, -1.0))
        );
    }

    #[test]
    fn test_tangent03() {
        let _0 = perturbed_ulps_as_int(0.0, 1);
        let _1 = perturbed_ulps_as_int(1.0, -1);
        let circle0 = circle(point(0.0, 0.0), 1.0);
        let circle1 = circle(point(_0, 0.0), 1.0);
        let res = ff(circle0, circle1);
        assert_eq!(res, CircleConfig::SameCircles());
    }

    #[test]
    fn test_tangent04() {
        let _1 = perturbed_ulps_as_int(1.0, -1);
        let circle0 = circle(point(0.0, 0.0), 1.0);
        let circle1 = circle(point(0.0, 0.0), _1);
        let res = ff(circle0, circle1);
        assert_eq!(res, CircleConfig::NoIntersection());
    }

    #[test]
    fn test_tangent05() {
        let _1m = perturbed_ulps_as_int(1.0, -1);
        let _1p = perturbed_ulps_as_int(1.0, 1);
        let circle0 = circle(point(1.0, 0.0), 1.0);
        let circle1 = circle(point(_1p, 0.0), _1m);
        let res = ff(circle0, circle1);
        assert_eq!(
            res,
            CircleConfig::NoncocircularTwoPoints(
                point(1.5, 0.8660254037844386),
                point(1.5, -0.8660254037844386)
            )
        );
    }

    #[test]
    fn test_tangent06() {
        let _1m = perturbed_ulps_as_int(1.0, -2);
        let _1p = perturbed_ulps_as_int(1.0, 1);
        let circle0 = circle(point(1.0, 0.0), 1.0);
        let circle1 = circle(point(_1p, 0.0), _1m);
        let res = ff(circle0, circle1);
        assert_eq!(res, CircleConfig::NoncocircularOnePoint(point(2.0, 0.0)));
    }

    #[test]
    fn test_no_intersection2() {
        let c0 = circle(point(0.5, 0.0), 0.5);
        let c1 = circle(point(-1.0, 0.0), 1.0);
        let res = ff(c0, c1);
        assert_eq!(res, CircleConfig::NoncocircularOnePoint(point(0.0, 0.0)));
    }
}
