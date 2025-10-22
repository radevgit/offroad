#![allow(dead_code)]

use togo::prelude::*;

use crate::offset_raw::OffsetRaw;

const ZERO: f64 = 0f64;

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

pub(crate) fn offset_segment(seg: &Arc, orig: Point, g: f64, off: f64) -> OffsetRaw {
    if seg.is_seg() {
        line_offset(seg, orig, off)
    } else {
        arc_offset(seg, orig, g, off)
    }
}

// Offsets line segment on right side
// #00028
fn line_offset(seg: &Arc, orig: Point, off: f64) -> OffsetRaw {
    // line segment
    let perp = seg.b - seg.a;
    let (perp, _) = point(perp.y, -perp.x).normalize(false);
    let offset_vec = perp * off;
    let mut arc = arcseg(seg.a + offset_vec, seg.b + offset_vec);
    arc.id(seg.id);
    return OffsetRaw {
        arc,
        orig: orig,
        g: ZERO,
    };
}

pub const EPS_COLLAPSED: f64 = 1E-10; // TODO: what should be the exact value.
// Offsets arc on right side
// #00028
fn arc_offset(seg: &Arc, orig: Point, bulge: f64, offset: f64) -> OffsetRaw {
    // Arc is always CCW
    //let seg = arc_circle_parametrization(seg.a, seg.b, bulge);
    let (v0_to_center, _) = (seg.a - seg.c).normalize(false);
    let (v1_to_center, _) = (seg.b - seg.c).normalize(false);

    let off = if bulge < 0.0 { -offset } else { offset };
    let offset_radius = seg.r + off;
    let a = seg.a + v0_to_center * off;
    let b = seg.b + v1_to_center * off;
    if offset_radius < EPS_COLLAPSED || offset_radius.is_nan()
        || a.close_enough(b, EPS_COLLAPSED)
    {
        // Collapsed arc is now line
        let mut arc = arcseg(b, a);
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
    //let last = pline.len() - 1;
    for i in 0..pline.len() - 1 {
        let bulge = pline[i].b;
        let seg = arc_from_bulge(pline[i].p, pline[i + 1].p, bulge);
        let check = seg.is_valid(EPS_COLLAPSED);
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
    // last segment
    let bulge = pline.last().unwrap().b;
    let seg = arc_from_bulge(pline.last().unwrap().p, pline[0].p, bulge);
    let check = seg.is_valid(EPS_COLLAPSED);
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


pub fn arcs_to_raws(arcss: &Vec<Arcline>) -> Vec<Vec<OffsetRaw>> {
    let mut varcs: Vec<Vec<OffsetRaw>> = Vec::new();
    for arcs in arcss {
        varcs.push(arcs_to_raws_single(arcs));
    }
    varcs
}

pub fn arcs_to_raws_single(arcs: &Arcline) -> Vec<OffsetRaw> {
    let mut offs = Vec::with_capacity(arcs.len() + 1);
 
    for i in 0..arcs.len() - 1 {
        let seg = arcs[i];
        let check = seg.is_valid(EPS_COLLAPSED);
        if !check {
            continue;
        }
        let bulge = bulge_from_arc(seg.a, seg.b, seg.c, seg.r);
        let orig = if bulge < ZERO { seg.a } else { seg.b };
        let off = OffsetRaw {
            arc: seg,
            orig: orig,
            g: bulge,
        };
        offs.push(off);
    }
    // last segment
    let seg = arcs.last().unwrap();
    let check = seg.is_valid(EPS_COLLAPSED);
    if check {
        let bulge = bulge_from_arc(seg.a, seg.b, seg.c, seg.r);
        let orig = if bulge < ZERO { seg.a } else { seg.b };
        let off = OffsetRaw {
            arc: *seg,
            orig: orig,
            g: bulge,
        };
        offs.push(off);
    }

    offs
}



#[cfg(test)]
mod test_offset_polyline_raw {
    use togo::prelude::*;

    use crate::offset_raw::offsetraw;

    use super::*;

    #[test]
    fn test_arc_offset_collapsed_arc() {
        // let arc0 = arc();
        // let res = arc_offset(
        //     pvertex(point(0.0, 0.0), -1.0),
        //     pvertex(point(1.0, 0.0), 0.0),
        //     1.0,
        // );
        // assert_eq!(
        //     res,
        //     OffsetRaw {
        //         arc: arcline(point(1.0, 0.0), point(0.0, 0.0)),
        //         orig: point(1.0, 0.0),
        //         g: 0.0
        //     }
        // );
    }

    #[test]
    fn test_new() {
        let arc = arc_from_bulge(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = offsetraw(arc, point(5.0, 6.0), 3.3);
        let o1 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(o0, o1);
    }

    #[test]
    fn test_display_01() {
        let arc = arc_from_bulge(point(0.0, 0.0), point(2.0, 2.0), 1.0);
        let o0 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(
            "[[[0.00000000000000000000, 0.00000000000000000000], [2.00000000000000000000, 2.00000000000000000000], [1.00000000000000000000, 1.00000000000000000000], 1.41421356237309514547], [5.00000000000000000000, 6.00000000000000000000], 3.3]",
            format!("{}", o0)
        );
    }

    #[test]
    fn test_display_02() {
        let arc = arc_from_bulge(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(
            "[[[1.00000000000000000000, 2.00000000000000000000], [3.00000000000000000000, 4.00000000000000000000], [3.49848484848484808651, 1.50151515151515169144], 2.54772716009334887488], [5.00000000000000000000, 6.00000000000000000000], 3.3]",
            format!("{}", o0)
        );
    }

    #[test]
    fn test_line_offset_vertical() {
        // vertical segment
        // let seg = arcline(point(2.0, 1.0), point(2.0, 11.0));
        // let res = offsetraw(
        //     arcline(point(3.0, 1.0), point(3.0, 11.0)),
        //     point(2.0, 11.0),
        //     0.0,
        // );
        // assert_eq!(line_offset(&seg, 1.0), res);
    }
    #[test]
    fn test_line_offset_horizontal() {
        // horizontal segment
        // let seg = arcline(point(-2.0, 1.0), point(3.0, 1.0));
        // let res = offsetraw(
        //     arcline(point(-2.0, -1.0), point(3.0, -1.0)),
        //     point(3.0, 1.0),
        //     0.0,
        // );
        // assert_eq!(line_offset(&seg, 2.0), res);
    }
    #[test]
    fn test_line_offset_diagonal() {
        // diagonal segment
        // let seg = arcline(point(-1.0, 1.0), point(2.0, 2.0));
        // let res = offsetraw(
        //     arcline(point(0.0, 2.0), point(-1.0, 3.0)),
        //     point(-2.0, 2.0),
        //     0.0,
        // );
        // assert_eq!(line_offset(&seg, std::f64::consts::SQRT_2), res);
    }

    #[test]
    //#[ignore = "svg output"]
    fn test_offset_polyline_raw02() {
        // let pline = vec![
        //     pvertex(point(100.0, 100.0), 0.5),
        //     pvertex(point(200.0, 100.0), -0.5),
        //     pvertex(point(200.0, 200.0), 0.5),
        //     pvertex(point(100.0, 200.0), -0.5),
        // ];
        // let plines = vec![pline.clone()];
        // let mut svg = svg(400.0, 600.0);
        // //let pline = polyline_translate(&pline, point(0.0, 100.0));
        // svg.polyline(&pline, "red");

        // //let pline = polyline_reverse(&pline);
        // let off: f64 = 52.25;
        // let offset_raw = offset_polyline_raw(&plines, off);
        // svg.offset_raws(&offset_raw, "blue");
        // svg.write();
    }

    // #[test]
    // //#[ignore = "svg output"]
    // fn test_offset_polyline_raw03() {
    //     let plines = pline_01();
    //     let mut svg = svg(400.0, 600.0);
    //     let plines = vec![pline.clone()];
    //     svg.polyline(&pline, "red");

    //     //let pline = polyline_reverse(&pline);
    //     let off: f64 = 52.25;
    //     let offset_raw = offset_polyline_raw(&plines, off);
    //     svg.offset_raws(&offset_raw, "blue");
    //     svg.write();
    // }

    #[test]
    #[ignore = "svg output"]
    fn test_arc_from_bulge_plinearc_svg() {
        let arc0 = arc_from_bulge(
            point(-52.0, 250.0),
            point(-23.429621235520095, 204.88318696736243),
            -0.6068148963145962,
        );
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle0 = circle(point(arc0.c.x, arc0.c.y), 0.1);
        svg.circle(&circle0, "blue");

        let offsetraw = offset_segment(&arc0, point(-52.0, 250.0), -0.6068148963145962, 16.0);
        svg.arcsegment(&offsetraw.arc, "green");
        let circle1 = circle(point(offsetraw.arc.c.x, offsetraw.arc.c.y), 0.1);
        svg.circle(&circle1, "blue");

        svg.write();
    }
}
