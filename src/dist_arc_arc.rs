#![allow(dead_code)]

use crate::{
    dist_point_arc::{dist_point_arc, DistPointArcConfig},
    int_arc_arc::{int_arc_arc, ArcArcConfig},
    int_line_arc::int_line_arc,
    line::line,
    utils::min_4,
    Arc,
};

pub fn dist_arc_arc(arc0: &Arc, arc1: &Arc) -> f64 {
    let res = int_arc_arc(arc0, arc1);
    match res {
        ArcArcConfig::NoIntersection() => {}
        _ => {
            return 0.0;
        }
    }

    let dist0 = match dist_point_arc(&arc0.a, arc1) {
        DistPointArcConfig::OnePoint(dist, _) | DistPointArcConfig::Equidistant(dist, _) => dist,
    };
    let dist1 = match dist_point_arc(&arc0.b, arc1) {
        DistPointArcConfig::OnePoint(dist, _) | DistPointArcConfig::Equidistant(dist, _) => dist,
    };
    let dist2 = match dist_point_arc(&arc1.a, arc0) {
        DistPointArcConfig::OnePoint(dist, _) | DistPointArcConfig::Equidistant(dist, _) => dist,
    };
    let dist3 = match dist_point_arc(&arc1.b, arc0) {
        DistPointArcConfig::OnePoint(dist, _) | DistPointArcConfig::Equidistant(dist, _) => dist,
    };
    let mut min_dist = min_4(dist0, dist1, dist2, dist3);

    if arc0.c.close_enough(arc1.c, 10E-10) {
        return min_dist;
    }

    let line_aa = line(arc0.c, arc1.c - arc0.c);
    let res0 = int_line_arc(&line_aa, arc0);
    let res1 = int_line_arc(&line_aa, arc1);
    match (res0, res1) {
        (
            crate::int_line_arc::LineArcConfig::OnePoint(p0, _),
            crate::int_line_arc::LineArcConfig::OnePoint(p1, _),
        ) => {
            let dist = (p0 - p1).norm();
            if dist < min_dist {
                min_dist = dist;
            }
        }
        (
            crate::int_line_arc::LineArcConfig::TwoPoints(p0, p1, _, _),
            crate::int_line_arc::LineArcConfig::OnePoint(p2, _),
        ) => {
            let dists = [(p0 - p2).norm(), (p1 - p2).norm()];
            for &dist in &dists {
                if dist < min_dist {
                    min_dist = dist;
                }
            }
        }
        (
            crate::int_line_arc::LineArcConfig::OnePoint(p0, _),
            crate::int_line_arc::LineArcConfig::TwoPoints(p1, p2, _, _),
        ) => {
            let dists = [(p0 - p1).norm(), (p0 - p2).norm()];
            for &dist in &dists {
                if dist < min_dist {
                    min_dist = dist;
                }
            }
        }
        (
            crate::int_line_arc::LineArcConfig::TwoPoints(p0, p1, _, _),
            crate::int_line_arc::LineArcConfig::TwoPoints(p2, p3, _, _),
        ) => {
            let dists = [
                (p0 - p2).norm(),
                (p0 - p3).norm(),
                (p1 - p2).norm(),
                (p1 - p3).norm(),
            ];
            for &dist in &dists {
                if dist < min_dist {
                    min_dist = dist;
                }
            }
        }
        _ => {}
    }

    min_dist
}

#[cfg(test)]
mod test_dist_arc_arc {
    use core::f64;

    use crate::{arc::arc, dist_arc_arc::dist_arc_arc, point::point};

    #[test]
    fn test_intersected_arc_arc_0() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(2.0, 0.0), point(0.0, 0.0), point(1.0, 1.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 0.0);
    }

    #[test]
    fn test_intersected_arc_arc_1() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(3.0, 0.0), point(-1.0, 0.0), point(1.0, 1.0), 2.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 0.0);
    }

    #[test]
    fn test_two_equidistant_points_0() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 0.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_two_almost_equidistant_points_1() {
        let e = 1e-10;
        let arc0 = arc(
            point(1.0 + e, 0.0),
            point(-1.0 + e, 0.0),
            point(0.0 + e, 0.0),
            1.0,
        );
        let arc1 = arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 0.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0 - e);
    }

    #[test]
    fn test_two_equidistant_points_2() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(-2.0, 0.0), point(0.0, 0.0), point(-1.0, 0.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_arc_endpoints_0() {
        let arc0 = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(0.1, 0.0), point(2.1, 0.0), point(1.0, 0.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 0.9);
    }

    #[test]
    fn test_interior_points_no_intersection_0() {
        let arc0 = arc(point(1.0, -0.5), point(1.0, 0.5), point(0.0, 0.0), 1.5);
        let arc1 = arc(point(-1.0, 1.5), point(-1.0, 0.5), point(0.0, 1.0), 1.5);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 2.0);
    }

    #[test]
    fn test_interior_points_no_intersection_1() {
        let arc0 = arc(point(1.0, 0.5), point(1.0, -0.5), point(1.0, 0.0), 0.5);
        let arc1 = arc(point(-1.0, -0.5), point(-1.0, 0.5), point(-1.0, 0.0), 0.5);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_interior_points_one_intersection_0() {
        let arc0 = arc(point(1.5, 1.0), point(1.5, -1.0), point(1.5, 0.0), 1.0);
        let arc1 = arc(point(-1.5, -1.0), point(-1.5, 1.0), point(-1.5, 0.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_interior_points_one_and_two_intersection_0() {
        let arc0 = arc(point(1.5, 1.0), point(1.5, -1.0), point(1.5, 0.0), 1.0);
        let arc1 = arc(point(-1.5, -1.0), point(-2.5, 0.0), point(-1.5, 0.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_interior_points_two_and_one_intersection_0() {
        let arc0 = arc(point(2.5, 0.0), point(1.5, -1.0), point(1.5, 0.0), 1.0);
        let arc1 = arc(point(-1.5, -1.0), point(-1.5, 1.0), point(-1.5, 0.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_interior_points_two_and_two_intersection_0() {
        let arc0 = arc(point(2.5, 0.0), point(1.5, -1.0), point(1.5, 0.0), 1.0);
        let arc1 = arc(point(-1.5, -1.0), point(-2.5, 0.0), point(-1.5, 0.0), 1.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_cocircular_arcs_01() {
        let arc0 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(point(0.0, -2.0), point(0.0, 2.0), point(0.0, 0.0), 2.0);
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_cocircular_arcs_02() {
        let eps = f64::EPSILON;
        let arc0 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let arc1 = arc(
            point(0.0, -2.0),
            point(0.0, 2.0),
            point(0.0 - eps, 0.0 + eps),
            2.0,
        );
        let dist = dist_arc_arc(&arc0, &arc1);
        assert_eq!(dist, 0.9999999999999998);
    }
}
