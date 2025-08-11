#![allow(dead_code)]

use std::fmt::Display;

use geom::prelude::*;

/// This is used in offsetting operations (offroad package).
#[derive(Debug, PartialEq, Clone)]
pub struct OffsetRaw {
    /// The arc being offset.
    pub arc: Arc,
    /// The original point before offsetting.
    pub orig: Point,
    /// The distance of the offset.
    pub g: f64,
}

impl Display for OffsetRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.arc, self.orig, self.g)
    }
}

impl OffsetRaw {
    /// Creates a new OffsetRaw instance.
    #[inline]
    pub fn new(arc: Arc, orig: Point, g: f64) -> Self {
        OffsetRaw { arc, orig, g }
    }
}

/// Creates a new OffsetRaw instance.
#[inline]
pub fn offsetraw(arc: Arc, orig: Point, g: f64) -> OffsetRaw {
    OffsetRaw::new(arc, orig, g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offsetraw() {
        let a = arc(point(0.0, 0.0), point(1.0, 0.0), point(0.5, 0.0), 1.0);
        let o = point(0.0, 0.0);
        let off = offsetraw(a, o, 0.25);
        assert_eq!(off.g, 0.25);
    }
}
