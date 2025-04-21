use std::fmt::Display;

use crate::point::Point;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Segment {
    pub p0: Point,
    pub p1: Point,
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.p0, self.p1)
    }
}

impl Segment {
    #[inline]
    pub fn new(p0: Point, p1: Point) -> Self {
        Segment { p0, p1 }
    }
}

#[inline]
pub fn segment(p0: Point, p1: Point) -> Segment {
    Segment::new(p0, p1)
}

impl Segment {
    pub fn get_centered_form(&self) -> (Point, Point, f64) {
        let center = (self.p0 + self.p1) * 0.5;
        let dir = self.p1 - self.p0;
        let (dirn, norm) = dir.normalize();
        let extent = norm * 0.5;
        (center, dirn, extent)
    }
}

#[cfg(test)]
mod test_segment {
    use crate::point::point;

    use super::*;

    #[test]
    fn test_new() {
        let s0 = Segment::new(point(1.0, 2.0), point(3.0, 4.0));
        let s1 = segment(point(1.0, 2.0), point(3.0, 4.0));
        assert_eq!(s0, s1);
    }

    #[test]
    fn test_display() {
        let s0 = Segment::new(point(1.0, 2.0), point(3.0, 4.0));
        assert_eq!("[[1.00000000000000000000, 2.00000000000000000000], [3.00000000000000000000, 4.00000000000000000000]]", format!("{}", s0));
    }

    #[test]
    fn test_get_centered_form() {
        let s0 = Segment::new(point(1.0, 1.0), point(3.0, 3.0));
        let (center, dir, extent) = s0.get_centered_form();
        assert_eq!(center, point(2.0, 2.0));
        assert_eq!(dir, point(2.0, 2.0));
        assert_eq!(extent, 1.4142135623730951);
    }
}
