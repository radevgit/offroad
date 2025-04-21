#![allow(dead_code)]

use crate::int_interval_interval::{intersect_interval_interval, IntervalConfig};
use crate::int_line_circle::LineCircleConfig;
use crate::interval::interval;
use crate::line::line;
use crate::{circle::Circle, int_line_circle::intersect_line_circle, segment::Segment, Point};



#[derive(Debug, PartialEq)]
pub enum DistSegmentCircleConfig {
    NoIntersection(),
    OnePoint(Point, f64),
    TwoPoints(Point, Point, f64, f64),
}

















pub fn dist_segment_circle(seg: Segment, circle: Circle) -> DistSegmentCircleConfig {


    DistSegmentCircleConfig::NoIntersection()
}


#[cfg(test)]
mod tests_dist_segment_circle {
    use super::*;

    #[test]
    fn test_dist_segment_circle() {}
}