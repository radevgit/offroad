#![allow(dead_code)]

use crate::{
    point::point,
    pvertex::{pvertex, Polyline},
};

const ZERO: f64 = 0f64;

pub fn pline_01() -> Polyline {
    let pline = vec![
        pvertex(point(100.0, 100.0), 1.5),
        pvertex(point(100.0, 160.0), ZERO),
        pvertex(point(120.0, 200.0), ZERO),
        pvertex(point(128.0, 192.0), ZERO),
        pvertex(point(128.0, 205.0), ZERO),
        pvertex(point(136.0, 197.0), ZERO),
        pvertex(point(136.0, 250.0), ZERO),
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
    return pline;
}
