#![allow(dead_code)]

use crate::circle::Circle;
use crate::line::Line;
use crate::point::point;
use crate::Point;

#[derive(Debug, PartialEq)]
pub enum DistLineCircleConfig {
    OnePair(f64, f64, Point, Point),
    TwoPairs(f64, f64, f64, Point, Point, Point, Point),
}

pub fn dist_line_circle(line: &Line, circle: &Circle) -> DistLineCircleConfig {
    let mut parameter: [f64; 2] = [0.0; 2];
    let mut closest: [[Point; 2]; 2] = [[point(0.0, 0.0); 2]; 2];
    let num_closest_pairs;

    let delta = line.origin - circle.c;

    const ZERO: f64 = 0.0;
    let direction = line.dir;
    let radius = circle.r;

    let dot_dir_dir = direction.dot(direction);
    let dot_dir_del = direction.dot(delta);
    let dot_perp_dir_del = direction.perp(delta);
    let r_sqr = radius * radius;

    let test = dot_perp_dir_del * dot_perp_dir_del - r_sqr * dot_dir_dir;
    if test >= ZERO {
        num_closest_pairs = 1;
        parameter[0] = -dot_dir_del / dot_dir_dir;
        closest[0][0] = delta + direction * parameter[0];
        closest[0][1] = closest[0][0];

        if test > ZERO {
            let (closestn, _) = closest[0][1].normalize();
            closest[0][1] = closestn * radius;
        }
    } else {
        let a0 = delta.dot(delta) - radius * radius;
        let a1 = dot_dir_del;
        let a2 = dot_dir_dir;
        let discr = f64::max(a1 * a1 - a0 * a2, ZERO);
        let sqrt_discr = discr.sqrt();

        let temp = -dot_dir_del
            + if dot_dir_del > ZERO {
                -sqrt_discr
            } else {
                sqrt_discr
            };
        num_closest_pairs = 2;
        parameter[0] = temp / dot_dir_dir;
        parameter[1] = a0 / temp;
        if parameter[0] > parameter[1] {
            (parameter[1], parameter[0]) = (parameter[0], parameter[1]);
        }

        closest[0][0] = delta + direction * parameter[0];
        closest[0][1] = closest[0][0];
        closest[1][0] = delta + direction * parameter[1];
        closest[1][1] = closest[1][0];
    }

    for j in 0..num_closest_pairs {
        for i in 0..2 {
            closest[j][i] = closest[j][i] + circle.c;
        }
    }

    let dist = (closest[0][0] - closest[1][0]).norm();

    if num_closest_pairs == 1 {
        DistLineCircleConfig::OnePair(dist, parameter[0], closest[0][0], closest[0][1])
    } else {
        DistLineCircleConfig::TwoPairs(
            dist,
            parameter[0],
            parameter[1],
            closest[0][0],
            closest[0][1],
            closest[1][0],
            closest[1][1],
        )
    }
}

#[cfg(test)]
mod test_dist_line_circle {
    use crate::circle::circle;
    use crate::dist_line_circle::DistLineCircleConfig;
    use crate::line::{line, Line};
    use crate::point::point;
    use crate::segment::segment;
    use crate::svg::svg;

    fn rev(line: Line) -> Line {
        Line::new(line.origin, -line.dir)
    }

    #[test]
    #[ignore = "reason"]
    fn test_circle_touching_line() {
        let line = line(point(0.0, 0.0), point(1.0, 0.0));
        let circle = circle(point(1.0, 1.0), 1.0);
        let res = super::dist_line_circle(&line, &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::OnePair(0.0, 1.0, point(1.0, 0.0), point(1.0, 0.0))
        );
        let res = super::dist_line_circle(&rev(line), &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::OnePair(0.0, -1.0, point(1.0, 0.0), point(1.0, 0.0))
        );
    }

    #[test]
    #[ignore = "reason"]
    fn test_circle_not_intersecting_line() {
        let eps = f64::EPSILON;
        let line = line(point(0.0, 0.0), point(1.0, 0.0));
        let circle = circle(point(1.0, 1.0), 1.0 - eps);
        let res = super::dist_line_circle(&line, &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::OnePair(eps, 1.0, point(1.0, 0.0), point(1.0, eps))
        );
        let res = super::dist_line_circle(&rev(line), &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::OnePair(eps, -1.0, point(1.0, 0.0), point(1.0, eps))
        );
    }

    #[test]
    fn test_circle_not_intersecting_line_02() {
        let seg = segment(point(-3.0, 1.5), point(-1.0, 1.5));
        let circle = circle(point(0.0, 0.0), 1.0);
        let line = line(seg.a, seg.b - seg.a);
        let res = super::dist_line_circle(&line, &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::OnePair(1.5, 1.5, point(0.0, 1.5), point(0.0, 1.0))
        );
        let res = super::dist_line_circle(&rev(line), &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::OnePair(1.5, -1.5, point(0.0, 1.5), point(0.0, 1.0))
        );
    }

    #[test]
    #[ignore = "reason"]
    fn test_circle_intersecting_line() {
        let eps = f64::EPSILON;
        let line = line(point(0.0, 0.0), point(1.0, 0.0));
        let circle = circle(point(1.0, 1.0 - eps), 1.0);
        let res = super::dist_line_circle(&line, &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::TwoPairs(
                0.0,
                0.9999999789265757,
                1.0000000210734243,
                point(0.9999999789265757, 0.0),
                point(0.9999999789265757, 0.0),
                point(1.0000000210734243, 0.0),
                point(1.0000000210734243, 0.0)
            )
        );
        let res = super::dist_line_circle(&rev(line), &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::TwoPairs(
                0.0,
                -1.0000000210734243,
                -0.9999999789265757,
                point(1.0000000210734243, 0.0),
                point(1.0000000210734243, 0.0),
                point(0.9999999789265757, 0.0),
                point(0.9999999789265757, 0.0),
            )
        );
    }

    #[test]
    #[ignore = "reason"]
    fn test_circle_intersecting_line_02() {
        let (dir, _) = point(0.0, -100.0).normalize();
        let line = line(point(1.0, 5.0), dir);
        let circle = circle(point(0.0, 0.0), 2.0);
        let res = super::dist_line_circle(&line, &circle);
        assert_eq!(
            res,
            DistLineCircleConfig::TwoPairs(
                0.0,
                -6.732050807568877,
                -3.267949192431123,
                point(-0.16552312575627393, -1.993138754537643),
                point(-0.16552312575627393, -1.993138754537643),
                point(0.4898474500805984, 1.9390847004835905),
                point(0.4898474500805984, 1.9390847004835905),
            )
        );
    }
}
