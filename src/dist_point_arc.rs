#![allow(dead_code)]

use crate::{circle::circle, dist_point_circle::distance_point_circle, Arc, Point};

pub fn distance_point_arc(point: Point, arc: &Arc) -> (Point, f64) {
    let circle = circle(arc.c, arc.r);
    let pc_result = distance_point_circle(point, &circle);
    if circle.r != pc_result.1 {
        if arc.contains(pc_result.0) {
            (pc_result.0, pc_result.1)
        } else {
            let diff0 = arc.a - point;
            let diff1 = arc.b - point;
            let sqr_length0 = diff0.dot(diff0);
            let sqr_length1 = diff1.dot(diff1);
            if sqr_length0 <= sqr_length1 {
                (arc.a, sqr_length0.sqrt())
            } else {
                (arc.b, sqr_length1.sqrt())
            }
        }
    } else {
        (arc.a, arc.r)
    }
}

#[cfg(test)]
mod tests_dist_point_circle {
    #[test]
    fn test_dist_point_circle() {}
}
