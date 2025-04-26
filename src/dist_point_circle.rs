#![allow(dead_code)]

use crate::circle::Circle;
use crate::Point;

pub fn distance_point_circle(point: Point, circle: &Circle) -> (Point, f64) {
    let diff = point - circle.c;
    let length = diff.norm();
    if length > 0f64 {
        let diff = diff / length;
        (circle.c + diff * circle.r, (length - circle.r).abs())
    } else {
        let unit = Point::new(1.0, 0.0);
        (circle.c + unit * circle.r, circle.r)
    }
}

#[cfg(test)]
mod tests_distance_point_circle {

    #[test]
    fn test_distance_point_circle() {}
}
