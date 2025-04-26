#![allow(dead_code)]

use crate::Point;

#[derive(Debug, PartialEq)]
pub enum DistSegmentCircleConfig {
    NoIntersection(),
    OnePoint(Point, f64),
    TwoPoints(Point, Point, f64, f64),
}

#[cfg(test)]
mod tests_dist_segment_circle {

    #[test]
    fn test_dist_segment_circle() {}
}
