#![allow(dead_code)]

use crate::point::point;
use crate::utils::diff_of_prod;
use crate::{circle::Circle, point::Point};

#[derive(Debug, PartialEq)]
pub enum CircleConfig {
    NoIntersection(),
    NoncocircularOnePoint(Point),
    NoncocircularTwoPoints(Point, Point),
    SameCircles(),
}

pub fn int_circle_circle(circle0: Circle, circle1: Circle) -> CircleConfig {
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
            let s = 0.5 * (diff_of_prod(r0, r0, r1, r1) / usqr_len + 1.0);

            let mut discr = diff_of_prod(r0 / usqr_len, r0, s, s);

            if discr < ZERO {
                discr = ZERO;
            }
            let t = discr.sqrt();
            let v = point(u.y, -u.x);
            let tmp = circle0.c + u * s;
            let p0 = tmp - v * t;
            let p1 = tmp + v * t;
            if t > 0f64 {
                return CircleConfig::NoncocircularTwoPoints(p0, p1);
            } else {
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

    fn ff(circle0: Circle, circle1: Circle) -> CircleConfig {
        int_circle_circle(circle0, circle1)
    }

    #[test]
    fn test_same_circles_01() {
        let circle0 = circle(point(100.0, -100.0), 1.0);
        let circle1 = circle(point(100.0, -100.0), 1.0);
        assert_eq!(ff(circle0, circle1), CircleConfig::SameCircles());
    }

    #[test]
    fn test_same_non_intersection_01() {
        let circle0 = circle(point(1000.0, -1000.0), 1.01);
        let circle1 = circle(point(1000.0, -1000.0), 1.0);
        assert_eq!(ff(circle0, circle1), CircleConfig::NoIntersection());
    }

    #[test]
    fn test_same_non_intersection_02() {
        let circle0 = circle(point(1000.0, -1000.0), 1.0);
        let circle1 = circle(point(1002.0, -1002.0), 1.0);
        assert_eq!(ff(circle0, circle1), CircleConfig::NoIntersection());
    }

    #[test]
    fn test_noncircular_two_points() {
        let eps = f64::EPSILON * 10.0;
        let circle0 = circle(point(10.0, -10.0), 1.0);
        let circle1 = circle(point(10.0, -12.0 + eps), 1.0);
        let point0 = point(10.000000042146848, -11.0);
        let point1 = point(9.999999957853152, -11.0);
        let res = ff(circle0, circle1);
        assert_eq!(res, CircleConfig::NoncocircularTwoPoints(point0, point1));
    }

    #[test]
    fn test_noncircular_one_point_01() {
        let eps = f64::EPSILON * 2.0;
        let circle0 = circle(point(10.0, -10.0), 1.0);
        let circle1 = circle(point(10.0, -12.0 + eps), 1.0);
        let point0 = point(10.0, -11.0);
        let res = ff(circle0, circle1);
        assert_eq!(res, CircleConfig::NoncocircularOnePoint(point0));
    }

    #[test]
    fn test_noncircular_one_point_02() {
        let circle0 = circle(point(10.0, -10.0), 1.0);
        let circle1 = circle(point(10.0, -10.5), 0.5);
        let point0 = point(10.0, -11.0);
        let res = ff(circle0, circle1);
        assert_eq!(res, CircleConfig::NoncocircularOnePoint(point0));
    }

    #[test]
    fn test_noncircular_two_points_1() {
        let eps = f64::EPSILON * 5.0;
        let circle0 = circle(point(10.0, -10.0), 1.0);
        let circle1 = circle(point(10.0, -10.5 - eps), 0.5);
        let point0 = point(10.000000059604645, -10.999999999999998);
        let point1 = point(9.999999940395355, -10.999999999999998);
        let res = ff(circle0, circle1);
        assert_eq!(res, CircleConfig::NoncocircularTwoPoints(point0, point1));
    }

    #[test]

    fn test_noncircular_one_point_03() {
        let eps = f64::EPSILON * 2.0;
        let circle0 = circle(point(1000.0, -1000.0), 100.0);
        let circle1 = circle(point(1000.0, -1200.0 + eps), 100.0);
        let point0 = point(1000.0, -1100.0);
        let res = ff(circle0, circle1);
        assert_eq!(res, CircleConfig::NoncocircularOnePoint(point0));
    }
}

#[cfg(test)]
mod tests_circle_old {
    use rand::rand_core::le;

    use super::*;
    use crate::{circle::circle, utils::perturbed_ulps_as_int};

    fn ff(circle0: Circle, circle1: Circle) -> CircleConfig {
        int_circle_circle(circle0, circle1)
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
        let res = int_circle_circle(circle0, circle1);
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

    use crate::svg::svg;
    #[test]
    fn test_intersection_issue_01() {
        let mut svg = svg(150.0, 200.0);
        let c0 = circle(point(100.0, 130.0), 20.0);
        let c1 = circle(point(75.0, 40.0), 85.0);
        svg.circle(&c0, "red");
        svg.circle(&c1, "blue");
        let p0 = point(113.87064429562277, 115.59148769566033);
        let p1 = point(80.68522962987866, 124.80965843614482);

        svg.circle(&circle(p0, 1.0), "red");
        svg.write();
        let res = ff(c0, c1);
        assert_eq!(res, CircleConfig::NoncocircularTwoPoints(p0, p1));
    }
}
