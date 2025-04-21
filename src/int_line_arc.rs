#![allow(dead_code)]

use crate::{
    arc::Arc, circle::circle, int_line_circle::intersect_line_circle, line::Line, point::Point,
};



#[derive(Debug, PartialEq)]
pub enum LineArcConfig {
    NoIntersection(),
    OnePoint(Point, f64),
    TwoPoints(Point, Point, f64, f64),
}

pub fn intersect_line_arc(line: Line, arc: &Arc) -> LineArcConfig {
    let circle = circle(arc.c, arc.r);
    let lc_result = intersect_line_circle(line, circle);
    match lc_result {
        crate::int_line_circle::LineCircleConfig::NoIntersection() => {
            return LineArcConfig::NoIntersection()
        }
        crate::int_line_circle::LineCircleConfig::OnePoint(p0, t0) => {
            if arc.contains(p0) {
                return LineArcConfig::OnePoint(p0, t0);
            } else {
                return LineArcConfig::NoIntersection();
            }
        }
        crate::int_line_circle::LineCircleConfig::TwoPoints(p0, p1, t0, t1) => {
            let b0 = arc.contains(p0); 
            let b1 = arc.contains(p1);
            if b0 && b1 {
                return LineArcConfig::TwoPoints(p0, p1, t0, t1);
            }
            if b0 {
                return LineArcConfig::OnePoint(p0, t0);
            }
            if b1 {
                return LineArcConfig::OnePoint(p1, t1);
            }
            return LineArcConfig::NoIntersection();
        }
    }
}


#[cfg(test)]
mod tests_line_arc {
    use crate::{
        arc::{arc, arc_circle_parametrization},
        line::line,
        point::point,
    };

    use super::*;

    #[test]
    fn test_no_intersection() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let l0 = line(point(0.0, 0.0), point(sgrt_2_2, sgrt_2_2));
        let arc0 =
            arc_circle_parametrization(point(1.0, 0.0), point(2.0, 1.0), -1.0 + f64::EPSILON);
        assert_eq!(
            intersect_line_arc(l0, &arc0),
            LineArcConfig::NoIntersection()
        );
    }

    #[test]
    fn test_no_intersection2() {
        let l0 = Line::new(point(-0.5, 1.0), point(1.0, 0.0));
        let arc0 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = intersect_line_arc(l0, &arc0);
        assert_eq!(res, LineArcConfig::NoIntersection());
    }

    #[test]
    fn test_no_intersection3() {
        
        let l0 = Line::new(point(-1.0, 0.5), point(1.0, 0.0));
        let arc0 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = intersect_line_arc(l0, &arc0);
        assert_eq!(res, LineArcConfig::NoIntersection());
    }

    #[test]
    fn test_two_points() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let l0 = line(point(-1.0, 0.0), point(sgrt_2_2, sgrt_2_2));
        let arc1 = arc(point(1.0, 1.0), point(0.0, 0.0), point(0.5, 0.5), sgrt_2_2);
        let res = intersect_line_arc(l0, &arc1);
        match res {
            LineArcConfig::TwoPoints(p0, p1, _, _) => {
                assert!(p0.close_enough(point(0.0, 1.0), 1E-7));
                assert!(p1.close_enough(point(0.0, 1.0), 1E-7));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_one_point() {
        let l0 = Line::new(point(-0.5, 1.0), point(1.0, 0.0));
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = intersect_line_arc(l0, &arc0);
        assert_eq!(res, LineArcConfig::OnePoint(point(0.0, 1.0), 0.5));
    }

    #[test]
    fn test_one_point2() {
        
        let l0 = Line::new(point(-1.0, 0.0), point(1.0, 0.0));
        let arc0 = arc(point(0.0, -1.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = intersect_line_arc(l0, &arc0);
        assert_eq!(res, LineArcConfig::OnePoint(point(1.0, 0.0), 2.0));
    }

    #[test]
    fn test_one_point3() {
        
        let l0 = Line::new(point(-2.0, 0.0), point(1.0, 0.0));
        let arc0 = arc(point(0.0, 1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        let res = intersect_line_arc(l0, &arc0);
        assert_eq!(res, LineArcConfig::OnePoint(point(-1.0, 0.0), 1.0));
    }
}
