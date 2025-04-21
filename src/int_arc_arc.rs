#![allow(dead_code)]

use crate::arc::arc;
use crate::int_circle_circle::CircleConfig;
use crate::{arc::Arc, circle::circle, int_circle_circle::intersect_circle_circle, point::Point};




#[derive(Debug, PartialEq)]
pub enum ArcArcConfig {
    NoIntersection(),
    NonCocircularOnePoint(Point),         
    NonCocircularTwoPoints(Point, Point), 
    CocircularOnePoint(Point),            
    CocircularTwoPoints(Point, Point),    
    CocircularOnePointOneArc0(Point, Arc), 
    CocircularOnePointOneArc1(Point, Arc), 
    CocircularOneArc(Arc),                
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
                            
                            return ArcArcConfig::CocircularOneArc(arc0.clone());
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
                                    return ArcArcConfig::CocircularTwoPoints(res_point0, res_point1);
                                }
                            }
                        }
                    } else {
                        
                        return ArcArcConfig::CocircularOneArc(arc0.clone());
                    }
                } else {
                    if arc0.a != arc1.b {
                        
                        let res_arc0 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
                        return ArcArcConfig::CocircularOneArc(res_arc0);
                    } else {
                        
                        
                        let res_point0 = arc0.a;
                        return ArcArcConfig::CocircularOnePoint(res_point0);
                    }
                }
            }
            if arc1.contains(arc0.b) {
                if arc0.b != arc1.a {
                    
                    
                    let res_arc0 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
                    return ArcArcConfig::CocircularOneArc(res_arc0);
                } else {
                    
                    
                    let res_point0 = arc1.a;
                    return ArcArcConfig::CocircularOnePoint(res_point0);
                }
            }

            if arc0.contains(arc1.a) {
                
                return ArcArcConfig::CocircularOneArc(*arc1);
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
    fn test_cocircular_one_arc() {
        
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        assert_eq!(i_arc(&arc0, &arc1), ArcArcConfig::CocircularOneArc(arc0));

        
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(0.0, -1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        assert_eq!(i_arc(&arc0, &arc1), ArcArcConfig::CocircularOneArc(arc0));

        
        let arc0 = arc(point(0.0, -1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(0.0, -1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        assert_eq!(i_arc(&arc0, &arc1), ArcArcConfig::CocircularOneArc(arc0));
    }

    #[test]
    fn test_cocircular_one_arc2() {
        
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 0.0);
        let arc1 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 0.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOneArc(arc0));
    }

    #[test]
    fn test_no_intersection() {
        
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
        assert_eq!(res, ArcArcConfig::CocircularOnePointOneArc0(arc0.a, arc_a));
    }

    #[test]
    fn test_cocircular_one_point_one_arc3() {
        let arc0 = arc(point(0.0, 1.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        let arc_a = arc(arc1.a, arc0.b, arc0.c, arc0.r);
        assert_eq!(res, ArcArcConfig::CocircularOnePointOneArc0(arc0.a, arc_a));
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
    fn test_cocircular_two_points() {
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
        assert_eq!(res, ArcArcConfig::CocircularOnePoint(point(-1.0, 0.0)));
    }

    #[test]
    fn test_cocircular_one_arc3() {
        let arc0 = arc(point(0.0, -1.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        let arc_a = arc(arc0.a, arc1.b, arc0.c, arc0.r);
        assert_eq!(res, ArcArcConfig::CocircularOneArc(arc_a));
    }

    #[test]
    fn test_cocircular_one_point2() {
        let arc0 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-1.0, 0.0), point(1.0, 0.0), point(0.0, 0.0), 1.0);
        let res = i_arc(&arc0, &arc1);
        assert_eq!(res, ArcArcConfig::CocircularOnePoint(point(1.0, 0.0)));
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
