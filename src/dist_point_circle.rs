#![allow(dead_code)]

use crate::circle::Circle;
use crate::point::{point, Point};

const ZERO: f64 = 0.0;
pub fn dist_point_circle(p: &Point, circle: &Circle) -> (f64, Point, bool) {
    let diff = p - circle.c;
    let length = diff.dot(diff);
    if length > ZERO {
        let length = length.sqrt();
        let diff = diff / length;
        ((length - circle.r).abs(), circle.c + diff * circle.r, false)
    } else {
        let unit = point(1.0, 0.0);
        (circle.r, circle.c + unit * circle.r, true)
    }
}

#[cfg(test)]
mod test_dist_point_circle {
    use crate::{circle::circle, point::point};

    #[test]
    fn test_point_outside_circle() {
        let c = circle(point(1.0, 1.0), 1.0);
        let p = point(3.0, 1.0);
        let (dist, closest, equidistant) = super::dist_point_circle(&p, &c);
        assert_eq!(dist, 1.0);
        assert_eq!(closest, point(2.0, 1.0));
        assert_eq!(equidistant, false);
    }

    #[test]
    fn test_point_on_circle() {
        let c = circle(point(1.0, 1.0), 1.0);
        let p = point(2.0, 1.0);
        let (dist, closest, equidistant) = super::dist_point_circle(&p, &c);
        assert_eq!(dist, 0.0);
        assert_eq!(closest, point(2.0, 1.0));
        assert_eq!(equidistant, false);
    }

    #[test]
    fn test_point_inside_circle() {
        let c = circle(point(1.0, 1.0), 1.0);
        let p = point(1.5, 1.0);
        let (dist, closest, equidistant) = super::dist_point_circle(&p, &c);
        assert_eq!(dist, 0.5);
        assert_eq!(closest, point(2.0, 1.0));
        assert_eq!(equidistant, false);
    }

    #[test]
    fn test_point_in_circle_center() {
        let c = circle(point(1.0, 1.0), 1.0);
        let p = point(1.0, 1.0);
        let (dist, closest, equidistant) = super::dist_point_circle(&p, &c);
        assert_eq!(dist, 1.0);
        assert_eq!(closest, point(2.0, 1.0));
        assert_eq!(equidistant, true);
    }
}
