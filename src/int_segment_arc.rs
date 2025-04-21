#![allow(dead_code)]

use crate::{
    circle::circle,
    int_segment_circle::{intersect_segment_circle, SegmentCircleConfig},
    segment::Segment,
    Arc, Point,
};



#[derive(Debug, PartialEq)]
pub enum SegmentArcConfig {
    NoIntersection(),
    OnePoint(Point, f64),
    TwoPoints(Point, Point, f64, f64),
}


pub fn intersect_segment_arc(segment: Segment, arc: &Arc) -> SegmentArcConfig {
    const EPS_CONTAINS: f64 = 1E-10;
    let circle = circle(arc.c, arc.r);
    let sc_res = intersect_segment_circle(segment, circle);
    match sc_res {
        SegmentCircleConfig::NoIntersection() => {
            return SegmentArcConfig::NoIntersection();
        }
        SegmentCircleConfig::OnePoint(p0, t0) => {
            if arc.contains(p0) {
                return SegmentArcConfig::OnePoint(p0, t0);
            } else {
                return SegmentArcConfig::NoIntersection();
            }
        }
        SegmentCircleConfig::TwoPoints(p0, p1, t0, t1) => {
            let b0 = arc.contains(p0);
            let b1 = arc.contains(p1);
            if b0 && b1 {
                return SegmentArcConfig::TwoPoints(p0, p1, t0, t1);
            }
            if b0 {
                return SegmentArcConfig::OnePoint(p0, t0);
            }
            if b1 {
                return SegmentArcConfig::OnePoint(p1, t1);
            }
            return SegmentArcConfig::NoIntersection();
        }
    }
}


pub fn is_touching_segment_arc(s: Segment, a: &Arc) -> bool {
    if s.p0 == a.a || s.p0 == a.b || s.p1 == a.a || s.p1 == a.b {
        true
    } else {
        false
    }
}

#[cfg(test)]
mod test_intersect_segment_arc {
    use crate::{arc::arc_circle_parametrization, point::point, segment::segment, svg::svg};

    use super::*;
    const ONE: f64 = 1f64;
    const ZERO: f64 = 0f64;

    #[test]
    #[ignore = "svg output"]
    fn test_intersect_segment_arc() {
        let mut svg = svg(300.0, 350.0);
        
        let v0 = point(100.0, 100.0);
        let v1 = point(150.0, 150.0);
        let v2 = point(130.0, 200.0);
        let v3 = point(130.0, 0.0);
        let b = -0.5;
        let arc = arc_circle_parametrization(v0, v1, b);
        let segment = segment(v2, v3);
        let res = intersect_segment_arc(segment, &arc);
        let (pc, pd) = match res {
            SegmentArcConfig::NoIntersection() => (point(0.0, 0.0), point(0.0, 0.0)),
            SegmentArcConfig::OnePoint(p, _) => (p, p),
            SegmentArcConfig::TwoPoints(p0, p1, _, _) => (p0, p1),
        };

        svg.arc(&arc, "red");
        svg.line(&segment, "green");
        svg.circle(&circle(pc, 1.0), "black");
        svg.circle(&circle(pd, 1.0), "black");
        svg.write();
        
    }
}


#[cfg(test)]
mod tests_segment_arc {
    use crate::{
        arc::{arc, arc_circle_parametrization},
        point::point,
        segment::segment,
    };

    use super::*;

    #[test]
    fn test_no_intersection() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let s0 = segment(point(0.0, 0.0), point(sgrt_2_2, sgrt_2_2));
        let arc0 =
            arc_circle_parametrization(point(1.0, 0.0), point(2.0, 1.0), -1.0 + f64::EPSILON);
        assert_eq!(
            intersect_segment_arc(s0, &arc0),
            SegmentArcConfig::NoIntersection()
        );
    }

    #[test]
    fn test_no_intersection2() {
        let s0 = segment(point(-0.5, 1.0), point(0.5, 1.0));
        let arc0 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = intersect_segment_arc(s0, &arc0);
        assert_eq!(res, SegmentArcConfig::NoIntersection());
    }

    #[test]
    fn test_no_intersection3() {
        
        let s0 = segment(point(-1.0, 0.5), point(1.0, 0.5));
        let arc0 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = intersect_segment_arc(s0, &arc0);
        assert_eq!(res, SegmentArcConfig::NoIntersection());
    }

    #[test]
    fn test_no_intersection4() {
        
        let s0 = segment(point(-1.0, 1.0), point(-0.0, 1.0));
        let arc0 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = intersect_segment_arc(s0, &arc0);
        assert_eq!(res, SegmentArcConfig::NoIntersection());
    }

    #[test]
    fn test_two_points() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let s0 = segment(point(-1.0, 0.0), point(0.0, 1.0));
        let arc1 = arc(point(1.0, 1.0), point(0.0, 0.0), point(0.5, 0.5), sgrt_2_2);
        let res = intersect_segment_arc(s0, &arc1);
        match res {
            SegmentArcConfig::OnePoint(p0, _) => {
                assert!(p0.close_enough(point(0.0, 1.0), 1E-8));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_one_point() {
        
        let s0 = segment(point(-0.5, 1.0), point(0.5, 1.0));
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = intersect_segment_arc(s0, &arc0);
        assert_eq!(res, SegmentArcConfig::OnePoint(point(0.0, 1.0), 0.0));
    }

    #[test]
    fn test_one_point2() {
        
        let s0 = segment(point(-1.0, 0.0), point(1.0, 0.0));
        let arc0 = arc(point(0.0, -1.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = intersect_segment_arc(s0, &arc0);
        assert_eq!(res, SegmentArcConfig::OnePoint(point(1.0, 0.0), 1.0));
    }

    #[test]
    fn test_one_point3() {
        
        let s0 = segment(point(-2.0, 0.0), point(2.0, 0.0));
        let arc0 = arc(point(0.0, 1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        let res = intersect_segment_arc(s0, &arc0);
        assert_eq!(res, SegmentArcConfig::OnePoint(point(-1.0, 0.0), -1.0));
    }

    #[test]
    fn test_one_point4() {
        
        let s0 = segment(point(-1.0, 1.0), point(-0.0, 1.0));
        let arc0 = arc(point(0.0, 1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        let res = intersect_segment_arc(s0, &arc0);
        assert_eq!(res, SegmentArcConfig::OnePoint(point(0.0, 1.0), 0.5));
    }
}
