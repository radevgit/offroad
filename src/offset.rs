#![allow(dead_code)]
#![deny(unused_results)]

use crate::{
    arc::arc_circle_parametrization,
    point::point,
    pvertex::{polyline_reverse, polyline_scale, pvertex, Polyline},
    Arc,
};

const ZERO: f64 = 0f64;
pub fn pline_01() -> Vec<Polyline> {
    let pline = vec![
        pvertex(point(100.0, 100.0), 1.5),
        pvertex(point(100.0, 160.0), ZERO),
        pvertex(point(120.0, 200.0), ZERO),
        pvertex(point(128.0, 192.0), ZERO),
        pvertex(point(128.0, 205.0), ZERO),
        pvertex(point(136.0, 197.0), ZERO),
        pvertex(point(136.0, 245.0), -1.0),
        pvertex(point(131.0, 250.0), ZERO),
        pvertex(point(110.0, 250.0), -1.0),
        pvertex(point(78.0, 250.0), ZERO),
        pvertex(point(50.0, 250.0), -1.0),
        pvertex(point(38.0, 250.0), ZERO),
        pvertex(point(0.001, 250.0), 100000.0),
        pvertex(point(0.0, 250.0), ZERO),
        pvertex(point(-52.0, 250.0), ZERO),
        pvertex(
            point(-23.429621235520095, 204.88318696736243),
            -0.6068148963145962,
        ),
        pvertex(point(82.0, 150.0), 0f64),
        pvertex(point(50.0, 150.0), 1.0),
        pvertex(point(-20.0, 150.0), ZERO),
        pvertex(point(0.0, 100.0), ZERO),
    ];
    let pline2 = polyline_scale(&pline, 1.0);
    let plines = vec![pline2.clone()];
    return plines;
}

pub fn pline_02() -> Polyline {
    let pline = vec![
        pvertex(point(50.0, 50.0), ZERO),
        pvertex(point(200.0, 50.0), ZERO),
        pvertex(point(180.0, 55.0), ZERO),
        pvertex(point(160.0, 65.0), ZERO),
        pvertex(point(140.0, 80.0), ZERO),
        pvertex(point(120.0, 100.0), ZERO),
        pvertex(point(100.0, 125.0), ZERO),
        pvertex(point(120.0, 150.0), ZERO),
        pvertex(point(140.0, 170.0), ZERO),
        pvertex(point(160.0, 185.0), ZERO),
        pvertex(point(180.0, 195.0), ZERO),
        pvertex(point(200.0, 200.0), ZERO),
        pvertex(point(-50.0, 200.0), ZERO),
        pvertex(point(-30.0, 195.0), ZERO),
        pvertex(point(-10.0, 185.0), ZERO),
        pvertex(point(10.0, 170.0), ZERO),
        pvertex(point(30.0, 150.0), ZERO),
        pvertex(point(50.0, 125.0), ZERO),
        pvertex(point(30.0, 100.0), ZERO),
        pvertex(point(10.0, 80.0), ZERO),
        pvertex(point(-10.0, 65.0), ZERO),
        pvertex(point(-30.0, 55.0), ZERO),
        pvertex(point(-50.0, 50.0), ZERO),
        pvertex(point(50.0, 50.0), ZERO),
    ];

    return pline;
}

pub fn pline_03() -> Vec<Polyline> {
    let pline = vec![
        pvertex(point(0.0, 0.0), ZERO),
        pvertex(point(200.0, 0.0), ZERO),
        pvertex(point(200.0, 100.0), ZERO),
        pvertex(point(100.0, 100.0), ZERO),
        pvertex(point(100.0, 200.0), ZERO),
        pvertex(point(0.0, 200.0), ZERO),
    ];
    let pline2 = polyline_scale(&pline, 1.0);
    let plines = vec![pline2.clone()];
    return plines;
}

pub fn poly_to_arcs(plines: &Vec<Polyline>) -> Vec<Vec<Arc>> {
    let mut varcs: Vec<Vec<Arc>> = Vec::new();
    for pline in plines {
        varcs.push(poly_to_arcs_single(pline));
    }
    varcs
}

fn poly_to_arcs_single(pline: &Polyline) -> Vec<Arc> {
    let mut arcs = Vec::with_capacity(pline.len() + 1);
    let last = pline.len() - 1;
    for i in 0..last {
        let arc = arc_circle_parametrization(pline[i].p, pline[i + 1].p, pline[i].g);
        arcs.push(arc);
    }

    let arc =
        arc_circle_parametrization(pline.last().unwrap().p, pline[0].p, pline.last().unwrap().g);
    arcs.push(arc);
    arcs
}

#[cfg(test)]
mod test_offset {

    use std::vec;

    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        offset::{pline_02, pline_03},
        offset_connect_raw,
        offset_polyline_raw::{offset_polyline_raw, poly_to_raws},
        offset_prune_invalid_offsets::offset_prune_invalid_offsets,
        offset_split_arcs, pline_01,
        point::point,
        pvertex::{
            polyline_reverse, polyline_scale, polyline_translate, polylines_reverse, pvertex,
        },
        svg::{svg, SVG},
        utils::random_arc,
        OffsetRaw,
    };

    const ZERO: f64 = 0f64;
    #[test]
    #[ignore = "svg output"]
    fn test_self_intersect_issue() {
        let pline = vec![vec![
            pvertex(point(100.0, 160.0), ZERO),
            pvertex(point(120.0, 200.0), ZERO),
            pvertex(point(128.0, 192.0), ZERO),
            pvertex(point(128.0, 205.0), ZERO),
            pvertex(point(136.0, 197.0), ZERO),
            pvertex(point(136.0, 250.0), ZERO),
        ]];
        let pliner = polylines_reverse(&pline);
        let poly_raws = poly_to_raws(&pliner);
        let mut svg = svg(300.0, 350.0);
        svg.offset_raws(&poly_raws, "black");

        let off = 5.0;

        let offset_raw = offset_polyline_raw(&poly_raws, off);
        svg.offset_raws(&offset_raw, "blue");

        let offset_connect = offset_connect_raw(&offset_raw, off);
        svg.offset_segments(&offset_connect, "violet");

        let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);

        let offset_final = offset_prune_invalid_offsets(&poly_raws, &mut offset_split, off);
        svg.offset_segments_single(&offset_final, "black");

        svg.write();
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_complex_polyline() {
        let plines = &pline_01();
        let poly_raws = poly_to_raws(&plines);
        let mut svg = svg(300.0, 350.0);
        svg.offset_raws(&poly_raws, "red");

        let off = 16.0;

        let offset_raw = offset_polyline_raw(&poly_raws, off);

        let offset_connect = offset_connect_raw(&offset_raw, off);

        let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);

        let offset_final = offset_prune_invalid_offsets(&poly_raws, &mut offset_split, off);
        svg.offset_segments_single(&offset_final, "black");

        svg.write();
    }

    fn offset_multiple(poly_raws: &Vec<Vec<OffsetRaw>>, svg: &mut crate::svg::SVG) {
        svg.offset_raws(&poly_raws, "red");

        let mut off = 10.0;
        while off < 256.0 {
            let offset_raw = offset_polyline_raw(&poly_raws, off);

            let offset_connect = offset_connect_raw(&offset_raw, off);

            let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);

            let offset_final = offset_prune_invalid_offsets(&poly_raws, &mut offset_split, off);
            svg.offset_segments_single(&offset_final, "blue");
            off += 10.0;
        }
    }

    fn offset_single(poly_raws: &Vec<Vec<OffsetRaw>>, off: f64, svg: &mut crate::svg::SVG) {
        let offset_raw = offset_polyline_raw(&poly_raws, off);

        let offset_connect = offset_connect_raw(&offset_raw, off);

        let mut offset_split = offset_split_arcs(&offset_raw, &offset_connect);

        let offset_final = offset_prune_invalid_offsets(&poly_raws, &mut offset_split, off);
        svg.offset_segments_single(&offset_final, "black");
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_multiple_complex_polyline() {
        let mut svg = svg(1280.0, 640.0);
        let mut plines: Vec<Vec<crate::PVertex>> = Vec::new();

        let mut p = pline_01()[0].clone();
        p = polyline_translate(&p, point(180.0, -60.0));
        p = polyline_scale(&p, 2.5);
        plines.push(p.clone());
        let poly_raws = poly_to_raws(&plines);
        offset_multiple(&poly_raws, &mut svg);

        let p2 = polyline_reverse(&p);
        let poly_raws = poly_to_raws(&vec![p2]);
        offset_multiple(&poly_raws, &mut svg);

        svg.write();
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_complex_polyline_2() {
        let p = pline_02();
        let mut plines = Vec::new();
        plines.push(p.clone());
        let p2 = polyline_translate(&p, point(50.0, 50.0));
        plines.push(p2);
        let poly_raws = poly_to_raws(&plines);
        let mut svg = svg(250.0, 350.0);
        svg.offset_raws(&poly_raws, "red");

        offset_multiple(&poly_raws, &mut svg);

        svg.write();
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_arcs_issue() {
        let mut svg = svg(500.0, 400.0);

        let p = vec![
            pvertex(point(50.0, 50.0), 0.2),
            pvertex(point(100.0, 50.0), -0.5),
            pvertex(point(100.0, 100.0), 0.2),
            pvertex(point(50.0, 100.0), -0.5),
        ];
        let mut plines = Vec::new();
        plines.push(p.clone());

        let p2 = polyline_translate(&p, point(50.0, 30.0));
        plines.push(p2);
        let poly_raws = poly_to_raws(&plines);

        offset_multiple(&poly_raws, &mut svg);

        svg.write();
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_new_connect() {
        let plines = pline_03();
        let poly_raws = poly_to_raws(&plines);
        let mut svg = svg(250.0, 350.0);
        svg.offset_raws(&poly_raws, "red");

        let off = 40.0;

        let offset_raw = offset_polyline_raw(&poly_raws, off);
        svg.offset_raws(&offset_raw, "blue");

        let offset_connect = offset_connect_raw(&offset_raw, off);
        svg.offset_segments(&offset_connect, "violet");

        svg.write();
    }

    #[test]
    #[ignore = "svg output"]
    fn test_offset_complex_line_bug() {}

    #[test]
    #[ignore = "svg output"]
    fn test_offset_complex_line_bug_2() {}
}
