#![allow(dead_code)]

use crate::point::Point;
use std::fmt::Display;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Line {
    pub origin: Point,
    pub dir: Point,
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.origin, self.dir)
    }
}

impl Line {
    #[inline]
    pub fn new(origin: Point, dir: Point) -> Self {
        Line { origin, dir }
    }
}

#[inline]
pub fn line(origin: Point, dir: Point) -> Line {
    Line::new(origin, dir)
}

#[cfg(test)]
mod test_line {
    use super::*;
    use crate::point::point;

    #[test]
    fn test_new() {
        let l0 = Line::new(point(1.0, 2.0), point(3.0, 4.0));
        let l1 = line(point(1.0, 2.0), point(3.0, 4.0));
        assert_eq!(l0, l1);
    }

    #[test]
    fn test_display() {
        let s0 = Line::new(point(1.0, 2.0), point(3.0, 4.0));
        assert_eq!("[[1.00000000000000000000, 2.00000000000000000000], [3.00000000000000000000, 4.00000000000000000000]]", format!("{}", s0));
    }
}
