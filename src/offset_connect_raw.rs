#![allow(dead_code)]

use robust::{orient2d, Coord};

use crate::{
    arc::{arc, arc_check},
    Arc, OffsetRaw, Point,
};

const ZERO: f64 = 0f64;

pub fn offset_connect_raw(raws: &Vec<Vec<OffsetRaw>>, off: f64) -> Vec<Vec<Arc>> {
    let mut res = Vec::with_capacity(raws.len());
    for raw in raws.iter() {
        res.push(offset_connect_raw_single(raw, off));
    }
    res
}

pub const ID_PADDING: usize = 100;
pub fn offset_connect_raw_single(raws: &Vec<OffsetRaw>, off: f64) -> Vec<Arc> {
    let mut res = Vec::with_capacity(raws.len() + 1);

    let last = raws.len() - 1;
    for i in 0..last {
        let old = raws[i].arc;
        let old_next = raws[i + 1].arc;
        let g0 = raws[i].g;
        let g1 = raws[i + 1].g;
        let orig = raws[i].orig;

        let (mut connect, check) = arc_connect_new(old, old_next, g0, g1, orig, off);
        connect.id(ID_PADDING + old.id);
        if check {
            res.push(connect);
        }
    }

    let last = raws.last().unwrap();
    let old = last.arc;
    let raw_next = raws.first().unwrap();
    let old_next = raw_next.arc;
    let g0 = last.g;
    let g1 = raw_next.g;
    let orig = last.orig;

    let (mut connect, check) = arc_connect_new(old, old_next, g0, g1, orig, off);
    connect.id(ID_PADDING + old.id);
    if check {
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
    if arc_check(&seg) && convex {
        (seg, true)
    } else {
        (seg, false)
    }
}

#[cfg(test)]
mod test_offset_connect_raw {
    use crate::{
        offset_polyline_raw::{offset_polyline_raw, poly_to_raws},
        point::point,
        pvertex::pvertex,
        svg::svg,
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

    fn test_offset_connect_segments_lines_01() {}

    #[test]

    fn test_offset_connect_segments_02() {}

    #[test]

    fn test_offset_connect_segments_03() {}
}
