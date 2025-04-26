#![allow(dead_code)]

use crate::interval::Interval;

// The possible 'configuration' in Result are listed as an
// enumeration.
// https://github.com/davideberly/GeometricTools/blob/master/GTE/Mathematics/IntrIntervals.h
// https://github.com/davideberly/GeometricTools/blob/4f47cd85ae2711612dc9c166a87fe16a8cb368cf/GTE/Mathematics/IntrIntervals.h#L177
#[derive(Debug, PartialEq)]
pub enum IntervalConfig {
    NoOverlap(),
    Overlap(f64, f64),
    Touching(f64),
}

pub fn intersect_interval_interval(interval0: Interval, interval1: Interval) -> IntervalConfig {
    assert!(interval0.0 <= interval0.1);
    assert!(interval1.0 <= interval1.1);
    if interval0.1 < interval1.0 || interval0.0 > interval1.1 {
        return IntervalConfig::NoOverlap();
    } else if interval0.1 > interval1.0 {
        if interval0.0 < interval1.1 {
            let overlap0 = if interval0.0 < interval1.0 {
                interval1.0
            } else {
                interval0.0
            };
            let overlap1 = if interval0.1 > interval1.1 {
                interval1.1
            } else {
                interval0.1
            };
            if overlap0 < overlap1 {
                return IntervalConfig::Overlap(overlap0, overlap1);
            } else {
                // case with degenerate intervals
                return IntervalConfig::Overlap(overlap0, overlap0);
            }
        } else {
            // interval0[0] == interval1[1]
            return IntervalConfig::Touching(interval0.0);
        }
    } else {
        // interval0[1] == interval1[0]
        return IntervalConfig::Touching(interval0.1);
    }
}

#[cfg(test)]
mod tests_intersect_interval_interval {
    use crate::interval::interval;

    use super::*;

    #[test]
    fn test_no_overlap() {
        let i0 = interval(1.0, 2.0);
        let i1 = interval(3.0, 4.0);
        assert_eq!(
            intersect_interval_interval(i0, i1),
            IntervalConfig::NoOverlap()
        );
    }

    #[test]
    fn test_one_point() {
        let i0 = interval(1.0, 2.0);
        let i1 = interval(2.0, 4.0);
        assert_eq!(
            intersect_interval_interval(i0, i1),
            IntervalConfig::Touching(2.0)
        );
    }

    #[test]
    fn test_one_point2() {
        let i0 = interval(1.0, 2.0);
        let i1 = interval(2.0 + f64::EPSILON, 4.0);
        assert_eq!(
            intersect_interval_interval(i0, i1),
            IntervalConfig::Touching(2.0)
        );
    }

    #[test]
    fn test_one_point_degenerate() {
        let i0 = interval(-0.0, -0.0);
        let i1 = interval(-1.0, 1.0);
        assert_eq!(
            intersect_interval_interval(i0, i1),
            IntervalConfig::Overlap(0.0, 0.0)
        );
    }

    #[test]
    fn test_overlap() {
        let i0 = interval(1.0, 2.0);
        let i1 = interval(2.0 - f64::EPSILON, 4.0);
        assert_eq!(
            intersect_interval_interval(i0, i1),
            IntervalConfig::Overlap(1.9999999999999998, 2.0)
        );
    }

    #[test]
    fn test_degenerate() {
        let i0 = interval(1.0, 2.0);
        let i1 = interval(1.5, 1.5);
        assert_eq!(
            intersect_interval_interval(i0, i1),
            IntervalConfig::Overlap(1.5, 1.5)
        );
    }

    #[test]
    fn test_touching_degenerate() {
        let i0 = interval(1.0, 2.0);
        let i1 = interval(2.0, 2.0);
        assert_eq!(
            intersect_interval_interval(i0, i1),
            IntervalConfig::Touching(2.0)
        );
    }
}
