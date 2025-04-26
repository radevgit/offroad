#![allow(dead_code)]

use crate::int_line_arc::LineArcConfig;
use crate::line::line;
use crate::{
    dist_point_arc::distance_point_arc, int_arc_arc::intersect_arc_arc,
    int_line_arc::intersect_line_arc, point::point, Arc, Point,
};

pub fn distance_arc_arc(arc0: &Arc, arc1: &Arc) -> (Point, f64) {
    let inter = intersect_arc_arc(arc0, arc1);
    match inter {
        crate::int_arc_arc::ArcArcConfig::NoIntersection() => {
            let mut v: Vec<f64> = Vec::new();
            v.push(distance_point_arc(arc0.a, arc1).1);
            v.push(distance_point_arc(arc0.b, arc1).1);
            v.push(distance_point_arc(arc1.a, arc0).1);
            v.push(distance_point_arc(arc1.b, arc0).1);

            let la0 = intersect_line_arc(line(arc0.c, arc1.c), arc0);
            let la1 = intersect_line_arc(line(arc0.c, arc1.c), arc1);
            match (la0, la1) {
                (LineArcConfig::OnePoint(p0, _), LineArcConfig::OnePoint(p1, _)) => {
                    v.push((p0 - p1).norm());
                }
                _ => {}
            }

            let mm;
            let m = v.into_iter().min_by(|a, b| a.partial_cmp(b).unwrap());
            match m {
                Some(d) => {
                    mm = d;
                }
                None => {
                    mm = f64::INFINITY;
                }
            }
            return (point(0.0, 0.0), mm);
        }
        _ => return (point(0.0, 0.0), 0f64),
    }
}

#[cfg(test)]
mod tests_distance_arc_arc {

    #[test]
    fn test_distance_arc_arc() {}
}
