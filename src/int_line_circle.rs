#![allow(dead_code)]

use crate::{circle::Circle, line::Line, point::Point};

#[derive(Debug, PartialEq)]
pub enum LineCircleConfig {
    NoIntersection(),
    OnePoint(Point, f64),
    TwoPoints(Point, Point, f64, f64),
}

const ZERO: f64 = 0f64;

pub fn intersect_line_circle(line: Line, circle: Circle) -> LineCircleConfig {
    let diff = line.origin - circle.c;
    let a0 = diff.dot_imp(diff) - circle.r * circle.r;
    let a1 = line.dir.dot_imp(diff);
    let discr = a1.mul_add(a1, -a0);
    if discr > ZERO {
        let root = discr.sqrt();
        let parameter0 = -a1 - root;
        let parameter1 = -a1 + root;
        let point0 = line.origin + line.dir * parameter0;
        let point1 = line.origin + line.dir * parameter1;
        return LineCircleConfig::TwoPoints(point0, point1, parameter0, parameter1);
    } else if discr < ZERO {
        return LineCircleConfig::NoIntersection();
    } else {
        let parameter0 = -a1;
        let point0 = line.origin + line.dir * parameter0;
        return LineCircleConfig::OnePoint(point0, parameter0);
    }
}

#[cfg(test)]
mod test_intersect_line_circle {
    use crate::{circle::circle, point::point, utils::perturbed_ulps_as_int};

    use super::*;

    #[test]
    fn test_no_intersection() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let l0 = Line::new(point(0.0, 0.0), point(sgrt_2_2, sgrt_2_2));
        let c0 = circle(point(3.0, 1.0), 1.0);
        assert_eq!(
            intersect_line_circle(l0, c0),
            LineCircleConfig::NoIntersection()
        );
    }

    #[test]
    fn test_one_point() {
        let l0 = Line::new(point(0.0, 1.0), point(1.0, 0.0));
        let c0 = circle(point(0.0, 0.0), 1.0);
        assert_eq!(
            intersect_line_circle(l0, c0),
            LineCircleConfig::OnePoint(point(0.0, 1.0), 0.0)
        );
    }

    #[test]
    fn test_two_points() {
        let _1_eps = perturbed_ulps_as_int(1.0, -1);
        let l0 = Line::new(point(0.0, _1_eps), point(1.0, 0.0));
        let c0 = circle(point(0.0, 0.0), 1.0);
        let res = intersect_line_circle(l0, c0);
        match res {
            LineCircleConfig::TwoPoints(p0, p1, t0, t1) => {
                assert_eq!(p0.y, _1_eps);
                assert_eq!(p1.y, _1_eps);
                assert_eq!(p0.x + p1.x, 0.0);
                assert_eq!(t0 + t1, 0.0);
            }
            _ => assert!(false),
        }
    }
}
