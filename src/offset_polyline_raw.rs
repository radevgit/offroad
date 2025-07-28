#![allow(dead_code)]

use std::fmt::Display;

use crate::{
    arc::{
        arc, arc_check, arc_circle_parametrization, arc_is_collapsed_ends, arc_is_collapsed_radius,
        arcline,
    },
    point::point,
    Arc, Point, Polyline,
};

const ZERO: f64 = 0f64;

#[derive(Debug, PartialEq, Clone)]
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

pub fn offset_polyline_raw(plines: &Vec<Vec<OffsetRaw>>, off: f64) -> Vec<Vec<OffsetRaw>> {
    let mut result = Vec::new();
    for pline in plines.iter() {
        result.push(offset_polyline_raw_single(pline, off));
    }
    result
}

fn offset_polyline_raw_single(pline: &Vec<OffsetRaw>, off: f64) -> Vec<OffsetRaw> {
    let mut result = Vec::with_capacity(pline.len());
    for p in pline.iter() {
        let offset = offset_segment(&p.arc, p.orig, p.g, off);
        result.push(offset);
    }
    result
}

pub fn offset_segment(seg: &Arc, orig: Point, g: f64, off: f64) -> OffsetRaw {
    if seg.is_line() {
        line_offset(seg, orig, off)
    } else {
        arc_offset(seg, orig, g, off)
    }
}

fn line_offset(seg: &Arc, orig: Point, off: f64) -> OffsetRaw {
    let perp = seg.b - seg.a;
    let (perp, _) = point(perp.y, -perp.x).normalize();
    let offset_vec = perp * off;
    let mut arc = arcline(seg.a + offset_vec, seg.b + offset_vec);
    arc.id(seg.id);
    return OffsetRaw {
        arc,
        orig: orig,
        g: ZERO,
    };
}

const EPS_COLLAPSED: f64 = 1E-10;

pub fn arc_offset(seg: &Arc, orig: Point, bulge: f64, offset: f64) -> OffsetRaw {
    let (v0_to_center, _) = (seg.a - seg.c).normalize();
    let (v1_to_center, _) = (seg.b - seg.c).normalize();

    let off = if bulge < 0.0 { -offset } else { offset };
    let offset_radius = seg.r + off;
    let a = seg.a + v0_to_center * off;
    let b = seg.b + v1_to_center * off;
    if arc_is_collapsed_radius(offset_radius) || arc_is_collapsed_ends(a, b) {
        let mut arc = arcline(b, a);
        arc.id(seg.id);
        return OffsetRaw {
            arc: arc,
            orig: orig,
            g: ZERO,
        };
    } else {
        let mut arc = arc(a, b, seg.c, offset_radius);
        arc.id(seg.id);
        return OffsetRaw {
            arc: arc,
            orig: orig,
            g: bulge,
        };
    }
}

pub fn poly_to_raws(plines: &Vec<Polyline>) -> Vec<Vec<OffsetRaw>> {
    let mut varcs: Vec<Vec<OffsetRaw>> = Vec::new();
    for pline in plines {
        varcs.push(poly_to_raws_single(pline));
    }
    varcs
}

pub fn poly_to_raws_single(pline: &Polyline) -> Vec<OffsetRaw> {
    let mut offs = Vec::with_capacity(pline.len() + 1);

    for i in 0..pline.len() - 1 {
        let bulge = pline[i].g;
        let seg = arc_circle_parametrization(pline[i].p, pline[i + 1].p, bulge);
        let check = arc_check(&seg);
        if !check {
            continue;
        }
        let orig = if bulge < ZERO { seg.a } else { seg.b };
        let off = OffsetRaw {
            arc: seg,
            orig: orig,
            g: bulge,
        };
        offs.push(off);
    }

    let bulge = pline.last().unwrap().g;
    let seg = arc_circle_parametrization(pline.last().unwrap().p, pline[0].p, bulge);
    let check = arc_check(&seg);
    if check {
        let orig = if bulge < ZERO { seg.a } else { seg.b };
        let off = OffsetRaw {
            arc: seg,
            orig: orig,
            g: bulge,
        };
        offs.push(off);
    }

    offs
}

#[cfg(test)]
mod test_offset_polyline_raw {
    use crate::{circle::circle, svg::svg};

    use super::*;

    #[test]
    fn test_arc_offset_collapsed_arc() {}

    #[test]
    fn test_new() {
        let arc = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = OffsetRaw::new(arc, point(5.0, 6.0), 3.3);
        let o1 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(o0, o1);
    }

    #[test]
    fn test_display_01() {
        let arc = arc_circle_parametrization(point(0.0, 0.0), point(2.0, 2.0), 1.0);
        let o0 = OffsetRaw::new(arc, point(5.0, 6.0), 3.3);
        assert_eq!(
            "[[[0.00000000000000000000, 0.00000000000000000000], [2.00000000000000000000, 2.00000000000000000000], [1.00000000000000000000, 1.00000000000000000000], 1.41421356237309514547], [5.00000000000000000000, 6.00000000000000000000], 3.3]",
            format!("{}", o0)
        );
    }

    #[test]
    fn test_display_02() {
        let arc = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = OffsetRaw::new(arc, point(5.0, 6.0), 3.3);
        assert_eq!(
            "[[[1.00000000000000000000, 2.00000000000000000000], [3.00000000000000000000, 4.00000000000000000000], [3.49848484848484808651, 1.50151515151515169144], 2.54772716009334887488], [5.00000000000000000000, 6.00000000000000000000], 3.3]",
            format!("{}", o0)
        );
    }

    #[test]
    fn test_line_offset_vertical() {}
    #[test]
    fn test_line_offset_horizontal() {}
    #[test]
    fn test_line_offset_diagonal() {}

    #[test]

    fn test_offset_polyline_raw02() {}

    #[test]
    #[ignore = "svg output"]
    fn test_arc_circle_parametrization_plinearc_svg() {
        let arc0 = arc_circle_parametrization(
            point(-52.0, 250.0),
            point(-23.429621235520095, 204.88318696736243),
            -0.6068148963145962,
        );
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle0 = circle(point(arc0.c.x, arc0.c.y), 0.1);
        svg.circle(&circle0, "blue");

        let offsetraw = offset_segment(&arc0, point(-52.0, 250.0), -0.6068148963145962, 16.0);
        svg.offset_segment(&offsetraw.arc, "green");
        let circle1 = circle(point(offsetraw.arc.c.x, offsetraw.arc.c.y), 0.1);
        svg.circle(&circle1, "blue");

        svg.write();
    }
}
