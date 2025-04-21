#![allow(dead_code)]

use std::fmt::Display;

use crate::{
    arc::{arc, arc_circle_parametrization, arc_is_collapsed_ends, arc_is_collapsed_radius, arcline},
    point::point,
    Arc, PVertex, Point, Polyline,
}; 

const ZERO: f64 = 0f64;

#[derive(Debug, PartialEq)]
pub struct OffsetRaw {
    pub arc: Arc,
    pub orig: Point, 
    pub g: f64,
}

impl Display for OffsetRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.arc, self.orig, self.g)
    }
}

impl OffsetRaw {
    #[inline]
    fn new(arc: Arc, orig: Point, g: f64) -> Self {
        OffsetRaw { arc, orig, g }
    }
}

#[inline]
pub fn offsetraw(arc: Arc, orig: Point, g: f64) -> OffsetRaw {
    OffsetRaw::new(arc, orig, g)
}

pub fn offset_polyline_raw(pline: &Polyline, off: f64) -> Vec<OffsetRaw> {
    let mut result = Vec::with_capacity(pline.len() + 1);
    let last = pline.len() - 1;
    for i in 0..last {
        let offset = offset_segment(pline[i], pline[i + 1], off);
        result.push(offset);
    }
    
    let offset = offset_segment(*pline.last().unwrap(), pline[0], off);
    result.push(offset);
    result
}

fn offset_segment(vertex0: PVertex, vertex1: PVertex, off: f64) -> OffsetRaw {
    if vertex0.g == ZERO {
        line_offset(vertex0, vertex1, off)
    } else {
        arc_offset(vertex0, vertex1, off)
    }
}



fn line_offset(v0: PVertex, v1: PVertex, off: f64) -> OffsetRaw {
    
    let perp = v1.p - v0.p;
    let (perp, _) = point(perp.y, -perp.x).normalize();
    let offset_vec = perp * off;
    let arc = arcline(v0.p + offset_vec, v1.p + offset_vec);
    return OffsetRaw {
        arc,
        orig: v1.p,
        g: ZERO,
    };
}

const EPS_COLLAPSED: f64 = 1E-10; 
                                  
                                  
                                  
fn arc_offset(v0: PVertex, v1: PVertex, offset: f64) -> OffsetRaw {
    let bulge = v0.g;
    
    let param = arc_circle_parametrization(v0.p, v1.p, bulge);
    let (v0_to_center, _) = (v0.p - param.c).normalize();
    let (v1_to_center, _) = (v1.p - param.c).normalize();

    let off = if bulge < 0.0 { -offset } else { offset };
    let offset_radius = param.r + off;
    let a = v0.p + v0_to_center * off;
    let b = v1.p + v1_to_center * off;
    if arc_is_collapsed_radius(offset_radius) || arc_is_collapsed_ends(a, b) {
        
        return OffsetRaw {
            arc: arcline(a, b),
            orig: v1.p,
            g: ZERO,
        };
    } else {
        return OffsetRaw {
            arc: arc(a, b, param.c, offset_radius),
            orig: v1.p,
            g: bulge,
        };
    }
}

#[cfg(test)]
mod test_offset_polyline_raw {
    use crate::{
        pline_01,
        pvertex::{polyline_translate, pvertex},
        svg::svg,
    };

    use super::*;

    #[test]
    fn test_new() {
        let arc = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = OffsetRaw::new(arc, point(5.0, 6.0), 3.3);
        let o1 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(o0, o1);
    }

    #[test]
    fn test_display() {
        let arc = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = OffsetRaw::new(arc, point(5.0, 6.0), 3.3);
        assert_eq!("[[[1.00000000000000000000, 2.00000000000000000000], [3.00000000000000000000, 4.00000000000000000000], [3.49848484848484808651, 1.50151515151515169144], 2.54772716009334887488], [5.00000000000000000000, 6.00000000000000000000], 3.3]", format!("{}", o0));
    }

    #[test]
    fn test_line_offset_vertical() {
        
        let v0 = pvertex(point(2.0, 1.0), ZERO);
        let v1 = pvertex(point(2.0, 11.0), ZERO);
        let res = offsetraw(
            arcline(point(3.0, 1.0), point(3.0, 11.0)),
            point(2.0, 11.0),
            0.0,
        );
        assert_eq!(line_offset(v0, v1, 1.0), res);
    }
    #[test]
    fn test_line_offset_horizontal() {
        
        let v0 = pvertex(point(-2.0, 1.0), ZERO);
        let v1 = pvertex(point(3.0, 1.0), ZERO);
        let res = offsetraw(
            arcline(point(-2.0, -1.0), point(3.0, -1.0)),
            point(3.0, 1.0),
            0.0,
        );
        assert_eq!(line_offset(v0, v1, 2.0), res);
    }
    #[test]
    fn test_line_offset_diagonal() {
        
        let v0 = pvertex(point(-1.0, 1.0), ZERO);
        let v1 = pvertex(point(-2.0, 2.0), ZERO);
        let res = offsetraw(
            arcline(point(0.0, 2.0), point(-1.0, 3.0)),
            point(-2.0, 2.0),
            0.0,
        );
        assert_eq!(line_offset(v0, v1, std::f64::consts::SQRT_2), res);
    }

    #[test]
    
    fn test_offset_polyline_raw02() {
        let pline = vec![
            pvertex(point(100.0, 100.0), 0.5),
            pvertex(point(200.0, 100.0), -0.5),
            pvertex(point(200.0, 200.0), 0.5),
            pvertex(point(100.0, 200.0), -0.5),
        ];
        let mut svg = svg(400.0, 600.0);
        
        svg.polyline(&pline, "red");

        
        let off: f64 = 52.25;
        let offset_raw = offset_polyline_raw(&pline, off);
        svg.offset_raws(&offset_raw, "blue");
        svg.write();
    }

    #[test]
    
    fn test_offset_polyline_raw03() {
        let pline = pline_01();
        let mut svg = svg(400.0, 600.0);
        let pline = polyline_translate(&pline, point(100.0, 200.0));
        svg.polyline(&pline, "red");

        
        let off: f64 = 52.25;
        let offset_raw = offset_polyline_raw(&pline, off);
        svg.offset_raws(&offset_raw, "blue");
        svg.write();
    }
}
