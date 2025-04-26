#![allow(dead_code)]

use std::mem::transmute;

const ALMOST_EQUAL_C: u64 = 0x8000_0000_0000_0000 as u64;
const ALMOST_EQUAL_CI: i64 = unsafe { std::mem::transmute::<u64, i64>(ALMOST_EQUAL_C) };

#[inline]
pub fn almost_equal_as_int(a: f64, b: f64, ulps: i64) -> bool {
    debug_assert!(a.is_finite());
    debug_assert!(b.is_finite());
    if a.signum() != b.signum() {
        return a == b;
    }
    let mut a_i: i64 = unsafe { std::mem::transmute::<f64, i64>(a) };
    let mut b_i: i64 = unsafe { std::mem::transmute::<f64, i64>(b) };

    if a_i < 0i64 {
        a_i = ALMOST_EQUAL_CI - a_i;
    }
    if b_i < 0i64 {
        b_i = ALMOST_EQUAL_CI - b_i;
    }

    if (a_i - b_i).abs() <= ulps {
        return true;
    }
    return false;
}

pub fn close_enough(a: f64, b: f64, eps: f64) -> bool {
    return (a - b).abs() < eps;
}

pub fn perturbed_ulps_as_int(f: f64, c: i64) -> f64 {
    debug_assert!(!(f == 0.0 && c == -1));
    let mut f_i: i64 = unsafe { transmute::<f64, i64>(f) };
    f_i += c;
    unsafe { transmute::<i64, f64>(f_i) }
}

#[inline]
pub fn next(ind: usize, size: usize) -> usize {
    if (ind + 1) < size {
        return ind + 1;
    }
    0
}

#[inline]
pub fn prev(ind: usize, size: usize) -> usize {
    if ind > 0 {
        return ind - 1;
    }
    size - 1
}

#[cfg(test)]
mod test_next_prev {
    use super::*;

    #[test]
    fn test_next() {
        assert_eq!(next(0, 3), 1);
        assert_eq!(next(1, 3), 2);
        assert_eq!(next(2, 3), 0);
        assert_eq!(next(3, 3), 0);
    }

    #[test]
    fn test_prev() {
        assert_eq!(prev(2, 3), 1);
        assert_eq!(prev(1, 3), 0);
        assert_eq!(prev(0, 3), 2);
    }
}

#[cfg(test)]
mod test_almost_equal_as_int {
    use super::*;
    use std::mem::transmute;

    #[test]
    fn test_almost_equal_as_int_nearby() {
        let result: bool = almost_equal_as_int(2.0, 1.999999999999999, 10);
        assert_eq!(result, true);
        let result: bool = almost_equal_as_int(-2.0, -1.999999999999999, 10);
        assert_eq!(result, true);

        let result: bool = almost_equal_as_int(2.0, 1.999999999999998, 10);
        assert_eq!(result, true);
        let result: bool = almost_equal_as_int(-2.0, -1.999999999999998, 10);
        assert_eq!(result, true);

        let result: bool = almost_equal_as_int(1.999999999999998, 2.0, 10);
        assert_eq!(result, true);
        let result: bool = almost_equal_as_int(-1.999999999999998, -2.0, 10);
        assert_eq!(result, true);
    }

    #[test]
    fn test_almost_equal_as_int_distant() {
        let result: bool = almost_equal_as_int(2.0, 1.999999999999997, 10);
        assert_eq!(result, false);
        let result: bool = almost_equal_as_int(-2.0, -1.999999999999997, 10);
        assert_eq!(result, false);

        let result: bool = almost_equal_as_int(1.999999999999997, 2.0, 10);
        assert_eq!(result, false);
        let result: bool = almost_equal_as_int(-1.999999999999997, -2.0, 10);
        assert_eq!(result, false);
    }

    #[test]
    fn test_almost_equal_as_int_limits() {
        let mut f_u: u64 = f64::MAX.to_bits();
        f_u -= 2;
        let f_f = f64::from_bits(f_u);
        let result: bool = almost_equal_as_int(f64::MAX, f_f, 3);
        assert_eq!(result, true);

        let mut f_u: u64 = f64::MAX.to_bits();
        f_u -= 4;
        let f_f = f64::from_bits(f_u);
        let result: bool = almost_equal_as_int(f64::MAX, f_f, 3);
        assert_eq!(result, false);

        let mut f_u: u64 = f64::MIN.to_bits();
        f_u -= 2;
        let f_f = f64::from_bits(f_u);
        let result: bool = almost_equal_as_int(f64::MIN, f_f, 3);
        assert_eq!(result, true);

        let mut f_u: u64 = f64::MIN.to_bits();
        f_u -= 4;
        let f_f = f64::from_bits(f_u);
        let result: bool = almost_equal_as_int(f64::MIN, f_f, 3);
        assert_eq!(result, false);
    }

    #[test]
    fn test_almost_equal_as_int_some_numbers() {
        let result: bool = almost_equal_as_int(100.0, -300.0, 10);
        assert_eq!(result, false);
    }

    #[test]
    fn test_print() {
        print_numbers();
        assert!(true);
    }

    fn print_number(f: f64, o: i64) {
        let mut f_i: i64 = unsafe { transmute::<f64, i64>(f) };
        f_i += o;
        println!("{:.20} Ox{:X} {:.}", f, f_i, f_i);
    }

    pub fn print_numbers() {
        let f: f64 = 2.0f64;
        print_number(f, 0);
        println!("");
        let f: f64 = 1.999999999999998;
        for i in -10..=10i64 {
            print_number(f, i);
        }
        println!("");

        let f: f64 = 0.0;
        print_number(f, 0 as i64);
        let o: i64 = unsafe { transmute::<u64, i64>(0x8000_0000_0000_0000 as u64) };
        print_number(f, o);
        println!("");

        for i in 0..=3i64 {
            print_number(f, i);
        }
        println!("");

        let c_i: i64 = unsafe { transmute::<u64, i64>(0x8000_0000_0000_0000 as u64) };
        for i in 0..=3i64 {
            print_number(f, i + c_i);
        }
        println!("");
    }

    #[test]
    fn test_perturbed_ulps_as_int() {
        let t = 1.0;
        let tt = perturbed_ulps_as_int(t, -1);
        let res = almost_equal_as_int(t, tt, 1);

        assert_eq!(res, true);

        let t = 1.0;
        let tt = perturbed_ulps_as_int(t, -1000);
        let res = almost_equal_as_int(t, tt, 1000);
        assert_eq!(res, true);

        let t = f64::MAX;
        let tt = perturbed_ulps_as_int(t, -1000);
        let res = almost_equal_as_int(t, tt, 1000);
        assert_eq!(res, true);

        let t = f64::MAX;
        let tt = perturbed_ulps_as_int(t, -1000000000);
        let res = almost_equal_as_int(t, tt, 1000000000);
        assert_eq!(res, true);
    }

    #[test]
    fn test_positive_negative_zero() {
        assert!(almost_equal_as_int(-0f64, 0f64, 0));
    }
}

#[cfg(test)]
mod test_root {}

#[inline]
pub fn diff_of_prod(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let cd = c * d;
    let err = (-c).mul_add(d, cd);
    let dop = a.mul_add(b, -cd);
    return dop + err;
}
#[inline]
pub fn sum_of_prod(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let cd = c * d;
    let err = c.mul_add(d, -cd);
    let sop = a.mul_add(b, cd);
    return sop + err;
}

#[cfg(test)]
mod test_diff_of_prod {
    use crate::point::point;

    use super::*;

    const _0: f64 = 0f64;
    const _1: f64 = 0f64;
    const _2: f64 = 0f64;

    #[test]
    fn test_diff_of_prod0() {
        let p0 = point(10000.0, 10000.0);
        let p1 = point(-10001.0, -10000.0);
        let res0 = p0.perp(p1);
        let res1 = diff_of_prod(p0.x, p1.y, p0.y, p1.x);
        assert_eq!(res0, res1);
    }

    #[test]
    fn test_diff_of_prod1() {
        let p0 = point(100000.0, 100000.0);
        let p1 = point(-100001.0, -100000.0);
        let res0 = p0.perp(p1);
        let res1 = diff_of_prod(p0.x, p1.y, p0.y, p1.x);
        assert_eq!(res0, res1);
    }
}
