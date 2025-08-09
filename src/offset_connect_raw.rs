#![allow(dead_code)]

use robust::{Coord, orient2d};

use geom::prelude::*;

const ZERO: f64 = 0f64;
const EPS: f64 = 1e-10;

#[doc(hidden)]
// Connect the ends of the raw offset segments with arcs.
pub fn offset_connect_raw(raws: &Vec<Vec<OffsetRaw>>, off: f64) -> Vec<Vec<Arc>> {
    let mut res = Vec::with_capacity(raws.len());
    for raw in raws.iter() {
        res.push(offset_connect_raw_single(raw, off));
    }
    res
}

#[doc(hidden)]
pub const ID_PADDING: usize = 100;
pub fn offset_connect_raw_single(raws: &Vec<OffsetRaw>, off: f64) -> Vec<Arc> {
    let mut res = Vec::with_capacity(raws.len() + 1);
    if raws.is_empty() {
        return res;
    }
    let last = raws.len() - 1;
    for i in 0..last {
        // make arcs ccw
        let old = raws[i].arc;
        let old_next = raws[i + 1].arc;
        let g0 = raws[i].g;
        let g1 = raws[i + 1].g;
        let orig = raws[i].orig;
        //let mut connect = arc(old.b, old_next.a, orig, off);
        let (mut connect, check) = arc_connect_new(old, old_next, g0, g1, orig, off);
        connect.id(ID_PADDING + old.id);
        if check {
            // only add valid arcs
            res.push(connect);
        }
    }
    // close end of line
    let last = raws.last().unwrap();
    let old = last.arc;
    let raw_next = raws.first().unwrap();
    let old_next = raw_next.arc;
    let g0 = last.g;
    let g1 = raw_next.g;
    let orig = last.orig;
    // let mut connect = arc(old.b, old_next.a, orig, off);
    let (mut connect, check) = arc_connect_new(old, old_next, g0, g1, orig, off);
    connect.id(ID_PADDING + old.id);
    if check {
        // only add valid arcs
        res.push(connect);
    }
    res
}

fn arc_connect_new(
    old: Arc,
    old_next: Arc,
    g0: f64,
    g1: f64,
    orig: Point,
    off: f64,
) -> (Arc, bool) {
    let seg: Arc;
    let convex: bool;
    let b = Coord {
        x: orig.x,
        y: orig.y,
    };
    if g0 >= ZERO && g1 >= ZERO {
        seg = arc(old.b, old_next.a, orig, off);
        let a = Coord {
            x: old.b.x,
            y: old.b.y,
        };
        let c = Coord {
            x: old_next.a.x,
            y: old_next.a.y,
        };
        convex = orient2d(a, b, c) < ZERO;
    } else if g0 >= ZERO && g1 < ZERO {
        seg = arc(old.b, old_next.b, orig, off);
        let a = Coord {
            x: old.b.x,
            y: old.b.y,
        };
        let c = Coord {
            x: old_next.b.x,
            y: old_next.b.y,
        };
        convex = orient2d(a, b, c) < ZERO;
    } else if g0 < ZERO && g1 >= ZERO {
        seg = arc(old.a, old_next.a, orig, off);
        let a = Coord {
            x: old.a.x,
            y: old.a.y,
        };
        let c = Coord {
            x: old_next.a.x,
            y: old_next.a.y,
        };
        convex = orient2d(a, b, c) < ZERO;
    } else {
        // g0 < 0 && g1 < 0
        seg = arc(old.a, old_next.b, orig, off);
        let a = Coord {
            x: old.a.x,
            y: old.a.y,
        };
        let c = Coord {
            x: old_next.b.x,
            y: old_next.b.y,
        };
        convex = orient2d(a, b, c) < ZERO;
    }
    if arc_check(&seg, EPS) && convex {
        (seg, true)
    } else {
        (seg, false)
    }
}


#[cfg(test)]
mod test_offset_connect_raw {
    use crate::{
        offset_polyline_raw::{offset_polyline_raw, poly_to_raws}, pline_01,
    };

    use super::*;

    #[test]
    #[ignore = "svg output"]
    fn test_offset_connect_segments_arcs_00_svg() {
        let pline = vec![vec![
            pvertex(point(100.0, 100.0), 0.5),
            pvertex(point(200.0, 200.0), 0.5),
        ]];
        let poly_raws = poly_to_raws(&pline);
        let mut svg = svg(300.0, 350.0);
        svg.offset_raws(&poly_raws, "red");

        let off: f64 = 52.25;

        let offset_raw = offset_polyline_raw(&poly_raws, off);
        svg.offset_raws(&offset_raw, "blue");

        let offset_connect = offset_connect_raw(&offset_raw, off);
        svg.offset_segments(&offset_connect, "violet");

        svg.write();
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_connect_segments_arcs_01() {
        let pline = vec![vec![
            // pvertex(point(100.0, 100.0), 0.5),
            pvertex(point(100.0, 210.0), 0.5),
            pvertex(point(280.0, 180.0), 5.0),
            pvertex(point(300.0, 200.0), -0.5),
            pvertex(point(200.0, 300.0), -0.5),
            pvertex(point(100.0, 300.0), 0.5),
            pvertex(point(0.0, 200.0), 0.5),
        ]];
        let poly_raws = poly_to_raws(&pline);
        let mut svg = svg(300.0, 400.0);
        svg.offset_raws(&poly_raws, "red");

        let off: f64 = 22.0;

        let offset_raw = offset_polyline_raw(&poly_raws, off);
        svg.offset_raws(&offset_raw, "blue");

        let offset_connect = offset_connect_raw(&offset_raw, off);
        svg.offset_segments(&offset_connect, "violet");

        svg.write();
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_connect_segments_lines_01() {
        // let pline = vec![
        //     pvertex(point(100.0, 100.0), 0.0),
        //     pvertex(point(200.0, 100.0), 0.0),
        //     pvertex(point(200.0, 200.0), 0.0),
        //     pvertex(point(100.0, 200.0), 0.0),
        // ];
        // let plines = vec![pline.clone()];
        // let mut svg = svg(400.0, 600.0);
        // //let pline = polyline_translate(&pline, point(0.0, 100.0));
        // svg.polyline(&pline, "grey");

        // //let pline = polyline_reverse(&pline);
        // let off: f64 = 52.25;
        // let offset_raw1 = offset_polyline_raw(&plines, off);
        // let offset_raw2 = offset_connect_raw(&offset_raw1, off);

        // svg.offset_raws(&offset_raw1, "red");
        // svg.offset_segments(&offset_raw2, "blue");
        // svg.write();
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_connect_segments_02() {
            // let pline = vec![
            //     pvertex(point(100.0, 100.0), -0.4),
            //     pvertex(point(200.0, 100.0), -0.4),
            //     pvertex(point(200.0, 200.0), -0.4),
            //     pvertex(point(100.0, 200.0), -0.4),
            // ];
            // let plines = vec![pline.clone()];
            // let mut svg = svg(400.0, 600.0);
            // //let pline = polyline_translate(&pline, point(0.0, 100.0));
            // svg.polyline(&pline, "grey");

            // //let pline = polyline_reverse(&pline);
            // //let off: f64 = 52.25;
            // let off: f64 = 62.00;
            // let offset_raw1 = offset_polyline_raw(&plines, off);
            // let offset_raw2 = offset_connect_raw(&offset_raw1, off);

            // svg.offset_raws(&offset_raw1, "red");
            // svg.offset_segments(&offset_raw2, "blue");
            // svg.write();
    }

    #[test]
    //#[ignore = "svg output"]
    fn test_offset_connect_segments_03() {
        let plines = pline_01();
        let mut svg = svg(400.0, 600.0);
        svg.polyline(&plines[0], "grey");

        let off: f64 = 16.00;
        let poly_raws = poly_to_raws(&plines);
        let offset_raw1 = offset_polyline_raw(&poly_raws, off);
        let offset_raw2 = offset_connect_raw(&offset_raw1, off);

        svg.offset_raws(&offset_raw1, "red");
        svg.offset_segments(&offset_raw2, "blue");
        svg.write();
    }
}
