#![allow(dead_code)]

use crate::{segment::Segment, Point};

#[derive(Debug, PartialEq)]
pub enum SegmentPointConfig {
    NoIntersection(),
    OnePoint(Point),
}

pub fn int_segment_point(_segment0: &Segment, _point1: &Point) -> SegmentPointConfig {
    SegmentPointConfig::NoIntersection()
}

#[cfg(test)]
mod test_int_segment_point {
    use crate::point::point;
    use crate::segment::segment;

    use super::*;

    #[test]
    fn test_no_intersection() {
        let s0 = segment(point(0.0, 0.0), point(1.0, 0.0));
        let p1 = point(2.0, 0.0);
        assert_eq!(
            int_segment_point(&s0, &p1),
            SegmentPointConfig::NoIntersection()
        );
    }
}
