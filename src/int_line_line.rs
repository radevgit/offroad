#![allow(dead_code)]

use crate::{line::Line, point::Point};

#[derive(Debug, PartialEq)]
pub enum LineConfig {
    ParallelDistinct(),
    ParallelTheSame(),
    OnePoint(Point, f64, f64),
}

const ZERO: f64 = 0f64;
pub fn intersect_line_line(line0: Line, line1: Line) -> LineConfig {
    let q = line1.origin - line0.origin;
    let dot_d0_perp_d1 = line0.dir.perp(line1.dir);
    if dot_d0_perp_d1 != ZERO {
        let dot_qperp_d0 = q.perp(line0.dir);
        let dot_qperp_d1 = q.perp(line1.dir);
        let s0 = dot_qperp_d1 / dot_d0_perp_d1;
        let s1 = dot_qperp_d0 / dot_d0_perp_d1;
        let p = line0.origin + line0.dir * s0;
        return LineConfig::OnePoint(p, s0, s1);
    } else {
        let dot_qperp_d1 = q.perp(line1.dir);
        if dot_qperp_d1.abs() != ZERO {
            return LineConfig::ParallelDistinct();
        } else {
            return LineConfig::ParallelTheSame();
        }
    }
}

#[cfg(test)]
mod test_intersect_line_line {
    use super::*;
    use crate::line::line;
    use crate::point::{almost_equal_as_int, point};

    #[test]
    fn test_parallel_distinct() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let l0 = line(point(0.0, 0.0), point(sgrt_2_2, sgrt_2_2));
        let l1 = line(point(f64::EPSILON, 0.0), point(sgrt_2_2, sgrt_2_2));
        assert_eq!(intersect_line_line(l0, l1), LineConfig::ParallelDistinct());
    }

    #[test]
    fn test_parallel_the_same() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let l0 = line(point(0.0, 0.0), point(sgrt_2_2, sgrt_2_2));
        let l1 = line(point(1.0, 1.0), point(sgrt_2_2, sgrt_2_2));
        assert_eq!(intersect_line_line(l0, l1), LineConfig::ParallelTheSame());
    }

    #[test]
    fn test_one_point() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let sgrt_2 = std::f64::consts::SQRT_2;
        let l0 = line(point(0.0, 0.0), point(sgrt_2_2, sgrt_2_2));
        let l1 = line(point(0.0, 2.0), point(sgrt_2_2, -sgrt_2_2));
        let res = intersect_line_line(l0, l1);
        match res {
            LineConfig::OnePoint(p, s0, s1) => {
                assert_eq!(p, point(1.0, 1.0));
                assert!(almost_equal_as_int(s0, sgrt_2, 1));
                assert!(almost_equal_as_int(s1, sgrt_2, 1));
            }
            _ => assert!(false),
        }
    }
}
