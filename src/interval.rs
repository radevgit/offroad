use std::fmt::Display;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Interval(pub f64, pub f64);

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.20}, {:.20}]", self.0, self.1)
    }
}

impl Interval {
    #[inline]
    pub fn new(e0: f64, e1: f64) -> Self {
        Interval(e0, e1)
    }
}

#[inline]
pub fn interval(e0: f64, e1: f64) -> Interval {
    Interval::new(e0, e1)
}

#[cfg(test)]
mod test_interval {
    use super::*;

    #[test]
    fn test_new() {
        let i0 = Interval::new(1.0, 2.0);
        let i1 = interval(1.0, 2.0);
        assert_eq!(i0, i1);
    }

    #[test]
    fn test_display() {
        let i0 = interval(1.0, 2.0);
        assert_eq!(
            "[1.00000000000000000000, 2.00000000000000000000]",
            format!("{}", i0)
        );
    }
}

impl Interval {
    pub fn contains(&self, e: f64) -> bool {
        debug_assert!(self.0 <= self.1);
        if e >= self.0 && e <= self.1 {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test_contains {
    use super::*;

    #[test]
    fn test_contains_fasle() {
        let i0 = Interval::new(1.0, 2.0);
        assert!(!i0.contains(1.0 - f64::EPSILON));
    }

    #[test]
    fn test_contains_true() {
        let i0 = Interval::new(1.0, 2.0);
        assert!(i0.contains(1.0 + f64::EPSILON));
    }
}
