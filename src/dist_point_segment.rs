#![allow(dead_code)]

use crate::circle::Circle;
use crate::segment::Segment;
use crate::Point;










pub fn distance_point_segment(point: Point, segment: Segment) -> (Point, f64) {
    
    
    let closest;
    const ZERO: f64 = 0f64;
    const ONE: f64 = 1f64;
    let direction = segment.p1 - segment.p0;
    let mut diff = point - segment.p1;
    let mut t = direction.dot(diff);
    if t >= ZERO {
        closest = segment.p1;
    } else {
        diff = point - segment.p0;
        t = direction.dot(diff);
        if t <= ZERO {
            closest = segment.p0;
        } else {
            let sqr_length = direction.dot(direction);
            if sqr_length > ZERO {
                t = t / sqr_length;
                closest = segment.p0 + direction * t;
            } else {
                closest = segment.p0;
            }
        }
    }

    (closest, (point - closest).norm_imp())
}

#[cfg(test)]
mod tests_distance_point_segment {
    use super::*;

    #[test]
    fn test_distance_point_segment() {}
}
