#![allow(dead_code)]

use std::fmt::Display;

use crate::Point;



#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PVertex {
    pub p: Point,
    pub g: f64,
}

impl Display for PVertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.p, self.g)
    }
}

impl PVertex {
    #[inline]
    pub fn new(p: Point, g: f64) -> Self {
        PVertex { p, g }
    }
}

#[inline]
pub fn pvertex(p: Point, g: f64) -> PVertex {
    PVertex::new(p, g)
}

pub type Polyline = Vec<PVertex>;

pub fn polyline_reverse(poly: &Polyline) -> Polyline {
    let last = poly.last().unwrap();
    let mut rev = poly.clone();
    rev.reverse();
    let mut res: Polyline = Vec::with_capacity(poly.len());
    for i in 0..rev.len() - 1 {
        let e = pvertex(rev[i].p, -rev[i + 1].g);
        res.push(e);
    }
    let e = pvertex(rev.last().unwrap().p, -last.g);
    res.push(e);

    res
}

pub fn polyline_scale(poly: &Polyline, scale: f64) -> Polyline {
    let mut res: Polyline = Vec::with_capacity(poly.len());
    for e in poly.iter() {
        let e = pvertex(e.p * scale, e.g);
        res.push(e);
    }
    res
}

pub fn polyline_translate(poly: &Polyline, translate: Point) -> Polyline {
    let mut res: Polyline = Vec::with_capacity(poly.len());
    for e in poly.iter() {
        let e = pvertex(e.p + translate, e.g);
        res.push(e);
    }
    res
}

#[cfg(test)]
mod test_pvertex {
    use super::*;
    use crate::point::point;

    #[test]
    fn test_new() {
        let p0 = PVertex::new(point(1.0, 2.0), 5.5);
        let p1 = pvertex(point(1.0, 2.0), 5.5);
        assert_eq!(p0, p1);
    }

    #[test]
    fn test_display() {
        let p = pvertex(point(1.0, 2.0), 5.5);
        
        assert_eq!(
            "[[1.00000000000000000000, 2.00000000000000000000], 5.5]",
            format!("{}", p)
        );
    }
}
