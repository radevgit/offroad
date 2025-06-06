#![allow(dead_code)]

use crate::arc::arc;
use crate::int_circle_circle::CircleConfig;
use crate::{arc::Arc, circle::circle, int_circle_circle::intersect_circle_circle, point::Point};

#[derive(Debug, PartialEq)]
pub enum ArcArcConfig {
    NoIntersection(),
    NonCocircularOnePoint(Point),
    NonCocircularTwoPoints(Point, Point),
    CocircularOnePoint0(Point),
    CocircularOnePoint1(Point),
    CocircularTwoPoints(Point, Point),
    CocircularOnePointOneArc0(Point, Arc),
    CocircularOnePointOneArc1(Point, Arc),
    CocircularOneArc0(Arc),
    CocircularOneArc1(Arc),
    CocircularOneArc2(Arc),
    CocircularOneArc3(Arc),
    CocircularOneArc4(Arc),
    CocircularTwoArcs(Arc, Arc),
}

pub fn intersect_arc_arc(arc0: &Arc, arc1: &Arc) -> ArcArcConfig {
    const EPS_CONTAINS: f64 = 1E-10;
    let circle0 = circle(arc0.c, arc0.r);
    let circle1 = circle(arc1.c, arc1.r);
    let cc_result = intersect_circle_circle(circle0, circle1);

    match cc_result {
        CircleConfig::NoIntersection() => return ArcArcConfig::NoIntersection(),
        CircleConfig::SameCircles() => {
            if arc1.contains(arc0.a) {
                if arc1.contains(arc0.b) {
                    if arc0.contains(arc1.a) && arc0.contains(arc1.b) {
                        if arc0.a == arc1.a && arc0.b == arc1.b {
                            return ArcArcConfig::CocircularOneArc0(arc0.clone());
                        } else {
                            if arc0.a != arc1.b {
                                if arc1.a != arc0.b {
                                    let res_arc0 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
                                    let res_arc1 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
                                    return ArcArcConfig::CocircularTwoArcs(res_arc0, res_arc1);
                                } else {
                                    let res_point0 = arc0.b;
                                    let res_arc0 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
                                    return ArcArcConfig::CocircularOnePointOneArc0(
                                        res_point0, res_arc0,
                                    );
                                }
                            } else {
                                if arc1.a != arc0.b {
                                    let res_point0 = arc0.a;
                                    let res_arc0 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
                                    return ArcArcConfig::CocircularOnePointOneArc1(
                                        res_point0, res_arc0,
                                    );
                                } else {
                                    let res_point0 = arc0.a;
                                    let res_point1 = arc0.b;
                                    return ArcArcConfig::CocircularTwoPoints(
                                        res_point0, res_point1,
                                    );
                                }
                            }
                        }
                    } else {
                        return ArcArcConfig::CocircularOneArc1(arc0.clone());
                    }
                } else {
                    if arc0.a != arc1.b {
                        let res_arc0 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
                        return ArcArcConfig::CocircularOneArc2(res_arc0);
                    } else {
                        let res_point0 = arc0.a;
                        return ArcArcConfig::CocircularOnePoint0(res_point0);
                    }
                }
            }
            if arc1.contains(arc0.b) {
                if arc0.b != arc1.a {
                    let res_arc0 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
                    return ArcArcConfig::CocircularOneArc3(res_arc0);
                } else {
                    let res_point0 = arc1.a;
                    return ArcArcConfig::CocircularOnePoint1(res_point0);
                }
            }

            if arc0.contains(arc1.a) {
                return ArcArcConfig::CocircularOneArc4(*arc1);
            } else {
                return ArcArcConfig::NoIntersection();
            }
        }
        CircleConfig::NoncocircularOnePoint(point0) => {
            if arc0.contains(point0) && arc1.contains(point0) {
                return ArcArcConfig::NonCocircularOnePoint(point0);
            } else {
                return ArcArcConfig::NoIntersection();
            }
        }
        CircleConfig::NoncocircularTwoPoints(point0, point1) => {
            let b0 = arc0.contains(point0) && arc1.contains(point0);
            let b1 = arc0.contains(point1) && arc1.contains(point1);

            if b0 && b1 {
                return ArcArcConfig::NonCocircularTwoPoints(point0, point1);
            }
            if b0 {
                return ArcArcConfig::NonCocircularOnePoint(point0);
            }
            if b1 {
                return ArcArcConfig::NonCocircularOnePoint(point1);
            }
            return ArcArcConfig::NoIntersection();
        }
    }
}

#[cfg(test)]
mod tests_arc_arc {
    use super::*;
    use crate::arc::Arc;
    use crate::point::point;

    fn i_arc(arc0: &Arc, arc1: &Arc) -> ArcArcConfig {
        intersect_arc_arc(arc0, arc1)
    }

    #[test]
    fn test_no_intersection() {
        let arc0 = arc(point(-2.0, 2.0), point(-2.0, 0.0), point(-2.0, 1.0), 1.0);
        let arc1 = arc(point(2.0, 0.0), point(2.0, 2.0), point(1.0, 1.0), 1.0);
        assert_eq!(i_arc(&arc0, &arc1), ArcArcConfig::NoIntersection());
    }

    #[test]
    fn test_cocircular_one_arc0() {
        let arc0 = arc(point(2.0, 1.0), point(1.0, 0.0), point(1.0, 1.0), 1.0);
        let arc1 = arc(point(2.0, 1.0), point(1.0, 0.0), point(1.0, 1.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOneArc0(arc0));
    }

    #[test]
    fn test_cocircular_one_arc0_2() {
        let arc0 = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOneArc0(arc0));
    }

    #[test]
    fn test_cocircular_two_arc() {
        let arc0 = arc(point(0.0, 0.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(0.0, 2.0), point(1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc00 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
        let arc01 = arc(arc1.a, arc0.b, arc1.c, arc1.r);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularTwoArcs(arc00, arc01));
    }

    #[test]
    fn test_cocircular_one_point_one_arc0() {
        let arc0 = arc(point(0.0, 0.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(-1.0, 1.0), point(1.0, 1.0), point(0.0, 1.0), 1.0);
        let p0 = point(-1.0, 1.0);
        let arc00 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOnePointOneArc0(p0, arc00));
    }

    #[test]
    fn test_cocircular_one_point_one_arc1() {
        let arc0 = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(0.0, 2.0), point(1.0, 1.0), point(0.0, 1.0), 1.0);
        let p0 = point(1.0, 1.0);
        let arc00 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOnePointOneArc1(p0, arc00));
    }

    #[test]
    fn test_cocircular_two_points() {
        let arc0 = arc(point(1.0, 1.0), point(0.0, 2.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(0.0, 2.0), point(1.0, 1.0), point(0.0, 1.0), 1.0);
        let p0 = point(1.0, 1.0);
        let p1 = point(0.0, 2.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularTwoPoints(p0, p1));
    }

    #[test]
    fn test_cocircular_one_arc_1() {
        let arc0 = arc(point(1.0, 1.0), point(0.0, 2.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(0.0, 0.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc00 = arc(arc0.a, arc0.b, arc0.c, arc0.r);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOneArc1(arc00));
    }

    #[test]
    fn test_cocircular_one_arc_2() {
        let arc0 = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(0.0, 0.0), point(0.0, 2.0), point(0.0, 1.0), 1.0);

        let arc00 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOneArc2(arc00));
    }

    #[test]
    fn test_cocircular_one_point_0() {
        let arc0 = arc(point(0.0, 2.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(0.0, 0.0), point(0.0, 2.0), point(0.0, 1.0), 1.0);

        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOnePoint0(arc0.a));
    }

    #[test]
    fn test_cocircular_one_arc_3() {
        let arc0 = arc(point(0.0, 0.0), point(0.0, 2.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc00 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOneArc3(arc00));
    }

    #[test]
    fn test_cocircular_one_point_1() {
        let arc0 = arc(point(0.0, 0.0), point(1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(1.0, 1.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOnePoint1(arc1.a));
    }

    #[test]
    fn test_cocircular_one_arc_4() {
        let arc0 = arc(point(0.0, 0.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(1.0, 1.0), point(0.0, 2.0), point(0.0, 1.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOneArc4(arc1));
    }

    #[test]
    fn test_cocircular_no_intersection() {
        let arc0 = arc(point(0.0, 0.0), point(1.0, 1.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(0.0, 2.0), point(-1.0, 1.0), point(0.0, 1.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NoIntersection());
    }

    #[test]
    fn test_noncircular_one_point() {
        let arc0 = arc(point(0.0, 0.0), point(0.0, 2.0), point(0.0, 1.0), 1.0);
        let arc1 = arc(point(1.0, 1.0), point(2.0, 2.0), point(1.0, 2.0), 1.0);
        let point00 = point(1.0, 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NonCocircularOnePoint(point00));
    }

    #[test]
    fn test_noncircular_two_points_0() {
        let arc0 = arc(point(-0.5, -1.0), point(-0.5, 1.0), point(-0.5, 0.0), 1.0);
        let arc1 = arc(point(0.5, 1.0), point(0.5, -1.0), point(0.5, 0.0), 1.0);
        let point00 = point(0.0, 0.8660254037844386);
        let point01 = point(0.0, -0.8660254037844386);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NonCocircularTwoPoints(point00, point01));
    }

    #[test]
    fn test_noncircular_two_points_1() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.03, 0.03), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NoIntersection());
    }

    #[test]
    fn test_noncircular_two_points_2() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.03), 1.0);
        let point00 = point(0.9998874936711629, 0.015);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NonCocircularOnePoint(point00));
    }

    #[test]
    fn test_noncircular_two_points_3() {
        let arc0 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.03), 1.0);
        let arc1 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let point00 = point(0.9998874936711629, 0.015);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NonCocircularOnePoint(point00));
    }

    #[test]
    fn test_cocircular_one_arc() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        assert_eq!(i_arc(&arc0, &arc1), ArcArcConfig::CocircularOneArc0(arc0));

        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(0.0, -1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        assert_eq!(i_arc(&arc0, &arc1), ArcArcConfig::CocircularOneArc1(arc0));

        let arc0 = arc(point(0.0, -1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(0.0, -1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        assert_eq!(i_arc(&arc0, &arc1), ArcArcConfig::CocircularOneArc0(arc0));
    }

    #[test]
    fn test_cocircular_one_arc2() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 0.0);
        let arc1 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 0.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOneArc0(arc0));
    }

    #[test]
    fn test_no_intersection111() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), f64::MAX);
        let arc1 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 0.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NoIntersection());
    }

    #[test]
    fn test_no_cocircular_two_arcs() {
        let arc0 = arc(point(1.0, 0.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        let arc_a = arc(arc0.a, arc1.b, arc0.c, arc0.r);
        let arc_b = arc(arc1.a, arc0.b, arc1.c, arc1.r);
        assert_eq!(res, ArcArcConfig::CocircularTwoArcs(arc_a, arc_b));
    }

    #[test]
    fn test_cocircular_one_point_one_arc() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        let arc_a = arc(arc0.a, arc1.b, arc0.c, arc0.r);
        assert_eq!(res, ArcArcConfig::CocircularOnePointOneArc0(arc0.b, arc_a));
    }

    #[test]
    fn test_cocircular_one_point_one_arc2() {
        let arc1 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc0 = arc(point(-1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        let arc_a = arc(arc1.a, arc0.b, arc0.c, arc0.r);
        assert_eq!(res, ArcArcConfig::CocircularOnePointOneArc1(arc0.a, arc_a));
    }

    #[test]
    fn test_cocircular_one_point_one_arc3() {
        let arc0 = arc(point(0.0, 1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        let arc_a = arc(arc1.a, arc0.b, arc0.c, arc0.r);
        assert_eq!(res, ArcArcConfig::CocircularOnePointOneArc1(arc0.a, arc_a));
    }

    #[test]
    fn test_cocircular_one_point_one_arc4() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        let arc_a = arc(arc0.a, arc1.b, arc0.c, arc0.r);
        assert_eq!(res, ArcArcConfig::CocircularOnePointOneArc0(arc0.b, arc_a));
    }

    #[test]
    fn test_cocircular_two_points_02() {
        let arc0 = arc(point(0.0, 1.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(
            res,
            ArcArcConfig::CocircularTwoPoints(point(0.0, 1.0), point(-1.0, 0.0))
        );
    }

    #[test]
    fn test_cocircular_one_point() {
        let arc0 = arc(point(0.0, 1.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOnePoint1(point(-1.0, 0.0)));
    }

    #[test]
    fn test_cocircular_one_arc3() {
        let arc0 = arc(point(0.0, -1.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        let arc_a = arc(arc0.a, arc1.b, arc0.c, arc0.r);
        assert_eq!(res, ArcArcConfig::CocircularOneArc2(arc_a));
    }

    #[test]
    fn test_cocircular_one_point2() {
        let arc0 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOnePoint0(point(1.0, 0.0)));
    }

    #[test]
    fn test_one_point() {
        let arc0 = arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 0.0), 1.0);
        let arc1 = arc(point(0.0, 0.0), point(-2.0, 0.0), point(-1.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NonCocircularOnePoint(point(0.0, 0.0)));
    }

    #[test]
    fn test_no_intersection2() {
        let arc0 = arc(point(1.0, -1.0), point(1.0, 1.0), point(1.0, 0.0), 1.0);
        let arc1 = arc(point(0.0, 0.0), point(-2.0, 0.0), point(-1.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::NoIntersection());
    }
}
