#![allow(dead_code)]

use crate::{circle::circle, dist_point_circle::dist_point_circle, Arc, Point};

#[derive(Debug, PartialEq)]
pub enum DistPointArcConfig {
    OnePoint(f64, Point),
    Equidistant(f64, Point),
}

pub fn dist_point_arc(p: &Point, arc: &Arc) -> DistPointArcConfig {
    let circle = circle(arc.c, arc.r);
    let (dist, closest, equidistant) = dist_point_circle(p, &circle);
    if !equidistant {
        if arc.contains(closest) {
            DistPointArcConfig::OnePoint(dist, closest)
        } else {
            let length0 = (arc.a - p).norm();
            let length1 = (arc.b - p).norm();
            if length0 <= length1 {
                DistPointArcConfig::OnePoint(length0, arc.a)
            } else {
                DistPointArcConfig::OnePoint(length1, arc.b)
            }
        }
    } else {
        DistPointArcConfig::Equidistant(arc.r, arc.a)
    }
}

pub fn dist_point_arc_dist(p: &Point, arc: &Arc) -> f64 {
    match dist_point_arc(p, arc) {
        DistPointArcConfig::OnePoint(dist, _) => dist,
        DistPointArcConfig::Equidistant(dist, _) => dist,
    }
}

#[cfg(test)]
mod test_dist_point_arc {

    use core::f64;

    use crate::{arc::arc, dist_point_arc::DistPointArcConfig, point::point, utils::close_enough};

    use super::dist_point_arc;

    #[test]
    fn test_point_is_on_arc() {
        let arc = arc(point(1.0, 0.0), point(1.0, 2.0), point(1.0, 1.0), 1.0);
        let p = point(2.0, 1.0);
        let res = dist_point_arc(&p, &arc);
        assert_eq!(
            res,
            super::DistPointArcConfig::OnePoint(0.0, point(2.0, 1.0))
        );
    }

    #[test]
    fn test_point_is_inside_arc() {
        let arc = arc(point(1.0, 0.0), point(1.0, 2.0), point(1.0, 1.0), 1.0);
        let p = point(1.5, 1.0);
        let res = dist_point_arc(&p, &arc);
        assert_eq!(
            res,
            super::DistPointArcConfig::OnePoint(0.5, point(2.0, 1.0))
        );
    }

    #[test]
    fn test_point_is_outside_arc() {
        let arc = arc(point(1.0, 0.0), point(1.0, 2.0), point(1.0, 1.0), 1.0);
        let p = point(3.0, 1.0);
        let res = dist_point_arc(&p, &arc);
        assert_eq!(
            res,
            super::DistPointArcConfig::OnePoint(1.0, point(2.0, 1.0))
        );
    }

    #[test]
    fn test_point_on_circle_outside_arc_01() {
        let arc = arc(point(0.0, -1.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let p = point(-1.0, 0.0);
        let res = dist_point_arc(&p, &arc);
        assert_eq!(
            res,
            super::DistPointArcConfig::OnePoint(std::f64::consts::SQRT_2, point(0.0, -1.0))
        );
    }

    #[test]
    fn test_point_on_circle_outside_arc_02() {
        let arc = arc(point(0.0, -1.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let p = point(-1.0, f64::EPSILON);
        let res = dist_point_arc(&p, &arc);
        match res {
            DistPointArcConfig::OnePoint(dist, closest) => {
                assert!(close_enough(
                    dist,
                    std::f64::consts::SQRT_2,
                    2.0 * f64::EPSILON
                ));
                assert_eq!(closest, point(0.0, 1.0));
            }
            _ => panic!("Expected OnePoint result"),
        }
    }

    #[test]
    fn test_point_on_arc_center() {
        let arc = arc(point(1.0, 0.0), point(1.0, 2.0), point(1.0, 1.0), 1.0);
        let p = point(1.0, 1.0);
        let res = dist_point_arc(&p, &arc);
        assert_eq!(res, DistPointArcConfig::Equidistant(1.0, point(1.0, 0.0)));
    }
}
