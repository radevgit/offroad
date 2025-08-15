#![allow(dead_code)]

use geom::prelude::*;

use crate::offset_raw::OffsetRaw;

const ZERO: f64 = 0f64;

#[doc(hidden)]
/// Offsets a `vec<vec<OffsetRaw>>`
///
/// Not intended for direct use
pub fn offset_segments_raws(plines: &Vec<Vec<OffsetRaw>>, off: f64) -> Vec<Vec<OffsetRaw>> {
    let mut result = Vec::new();
    for pline in plines {
        result.push(offset_raws_single(pline, off));
    }
    result
}

#[doc(hidden)]
/// Offsets a single `Vec<OffsetRaw>`
pub fn offset_raws_single(raws: &Vec<OffsetRaw>, off: f64) -> Vec<OffsetRaw> {
    let mut result = Vec::with_capacity(raws.len());
    for raw in raws {
        let offset = offset_arc_segment(&raw.arc, raw.orig, raw.g, off);
        result.push(offset);
    }
    result
}

#[doc(hidden)]
/// Offsets single Arc segment
pub fn offset_arc_segment(seg: &Arc, orig: Point, g: f64, off: f64) -> OffsetRaw {
    if seg.is_line() {
        seg_offset(seg, orig, off)
    } else {
        arc_offset(seg, orig, g, off)
    }
}

// #00028
#[doc(hidden)]
/// Offsets line segment on right side
fn seg_offset(seg: &Arc, orig: Point, off: f64) -> OffsetRaw {
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

const EPS_COLLAPSED: f64 = 1E-8; // TODO: what should be the exact value.
// #00028
/// Offsets arc on right side
#[doc(hidden)]
pub fn arc_offset(seg: &Arc, orig: Point, bulge: f64, offset: f64) -> OffsetRaw {
    let (v0_to_center, _) = (seg.a - seg.c).normalize(false);
    let (v1_to_center, _) = (seg.b - seg.c).normalize(false);

    let off = if bulge < 0.0 { -offset } else { offset };
    let offset_radius = seg.r + off;
    let a = seg.a + v0_to_center * off;
    let b = seg.b + v1_to_center * off;
    if arc_check(seg, EPS_COLLAPSED) {
        let mut arc = arc(a, b, seg.c, offset_radius);
        arc.id(seg.id);
        return OffsetRaw {
            arc: arc,
            orig: orig,
            g: bulge,
        };
    } else {
        // Collapsed arc is now line
        let mut arc = arcseg(b, a);
        arc.id(seg.id);
        return OffsetRaw {
            arc: arc,
            orig: orig,
            g: ZERO,
        };
    }
}


#[cfg(test)]
mod test_offset_polyline_raw {
    use geom::prelude::*;

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
        let arc = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        let o0 = offsetraw(arc, point(5.0, 6.0), 3.3);
        let o1 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(o0, o1);
    }

    #[test]
    fn test_display_01() {
        let arc = arc_circle_parametrization(point(0.0, 0.0), point(2.0, 2.0), 1.0);
        let o0 = offsetraw(arc, point(5.0, 6.0), 3.3);
        assert_eq!(
            "[[[0.00000000000000000000, 0.00000000000000000000], [2.00000000000000000000, 2.00000000000000000000], [1.00000000000000000000, 1.00000000000000000000], 1.41421356237309514547], [5.00000000000000000000, 6.00000000000000000000], 3.3]",
            format!("{}", o0)
        );
    }

    #[test]
    fn test_display_02() {
        let arc = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
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

        let offsetraw = offset_arc_segment(&arc0, point(-52.0, 250.0), -0.6068148963145962, 16.0);
        svg.arcsegment(&offsetraw.arc, "green");
        let circle1 = circle(point(offsetraw.arc.c.x, offsetraw.arc.c.y), 0.1);
        svg.circle(&circle1, "blue");

        svg.write();
    }
}
