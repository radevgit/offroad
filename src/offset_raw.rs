#![allow(dead_code)]

use std::fmt::Display;

use crate::prelude::*;

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

pub fn offset_raws_single(&mut self, segments: &Vec<OffsetRaw>, color: &str) {
    for s in segments.iter() {
        self.offset_segment(&s.arc, color);
    }
}

pub fn offset_raws(&mut self, segments: &Vec<Vec<OffsetRaw>>, color: &str) {
    for s in segments.iter() {
        self.offset_raws_single(s, color);
    }
}

#[cfg(test)]
mod test_offset_raw {
    use crate::prelude::*;

    #[test]
    fn test_new_and_display() {
        let a = arc(point(0.0, 0.0), point(1.0, 0.0), point(0.5, 0.0), 1.0);
        let o = point(0.0, 0.0);
        let off = offsetraw(a, o, 0.25);
        assert_eq!(off.arc, a);
        assert_eq!(off.orig, o);
        assert_eq!(off.g, 0.25);
        // Check Display has brackets and three components
        let s = format!("{}", off);
        assert_eq!(s, format!("[{}, {}, {}]", a, o, 0.25));
    }
}
