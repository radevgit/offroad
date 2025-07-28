#![allow(dead_code)]

use robust::{orient2d, Coord};

use crate::{
    circle::{circle, Circle},
    point::{point, Point},
    utils::diff_of_prod,
};

use std::{fmt::Display, sync::atomic::AtomicUsize};

pub type Arcline = Vec<Arc>;

static ID_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone)]
pub struct Arc {
    pub a: Point,
    pub b: Point,
    pub c: Point,
    pub r: f64,
    pub id: usize,
}

impl PartialEq for Arc {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c && self.r == other.r
    }
}

impl Display for Arc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}, {:.20}]", self.a, self.b, self.c, self.r)
    }
}

impl Arc {
    #[inline]
    pub fn new(a: Point, b: Point, c: Point, r: f64) -> Self {
        let id = ID_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Arc { a, b, c, r, id }
    }

    #[inline]
    pub fn id(&mut self, id: usize) {
        self.id = id;
    }
}

#[inline]
pub fn arc(a: Point, b: Point, c: Point, r: f64) -> Arc {
    Arc::new(a, b, c, r)
}

#[inline]
pub fn arcline(a: Point, b: Point) -> Arc {
    Arc::new(a, b, point(f64::INFINITY, f64::INFINITY), f64::INFINITY)
}

impl Arc {
    #[inline]
    pub fn is_arc(&self) -> bool {
        self.r != f64::INFINITY
    }
    #[inline]
    pub fn is_line(&self) -> bool {
        self.r == f64::INFINITY
    }
    #[inline]
    pub fn translate(&mut self, point: Point) {
        self.a = self.a + point;
        self.b = self.b + point;
        self.c = self.c + point;
    }

    #[inline]
    pub fn reverse(&self) -> Arc {
        Arc::new(self.b, self.a, self.c, self.r)
    }
}

#[cfg(test)]
mod test_arc {
    use super::*;

    #[test]
    fn test_new() {
        let arc0 = Arc::new(point(1.0, 1.0), point(1.0, 3.0), point(2.0, -1.0), 1.0);
        let arc1 = arc(point(1.0, 1.0), point(1.0, 3.0), point(2.0, -1.0), 1.0);
        assert_eq!(arc0, arc1);
    }

    #[test]
    fn test_display() {
        let arc = arc(point(1.0, 1.0), point(1.0, 3.0), point(2.0, -1.0), 1.0);

        assert_eq!(
            "[[1.00000000000000000000, 1.00000000000000000000], [1.00000000000000000000, 3.00000000000000000000], [2.00000000000000000000, -1.00000000000000000000], 1.00000000000000000000]",
            format!("{}", arc)
        );
    }

    #[test]
    fn test_is_arc() {
        let arc = arcline(point(1.0, 1.0), point(1.0, 3.0));
        assert!(arc.is_line());
        assert!(!arc.is_arc());
    }

    #[test]
    fn test_copy() {
        let arc = arcline(point(1.0, 1.0), point(1.0, 3.0));
        let arc2 = arc;
        assert_eq!(arc, arc2);
    }
}

impl Arc {
    pub fn contains(&self, p: Point) -> bool {
        let pa = Coord {
            x: self.a.x,
            y: self.a.y,
        };
        let pb = Coord {
            x: self.b.x,
            y: self.b.y,
        };
        let pp = Coord { x: p.x, y: p.y };
        let perp = orient2d(pa, pp, pb);

        perp >= 0f64
    }

    fn simple_orient2d(p: Coord<f64>, q: Coord<f64>, r: Coord<f64>) -> f64 {
        (q.x - p.x) * (r.y - q.y) - (q.y - p.y) * (r.x - q.x)
    }

    #[inline]

    pub fn contains_order2d(a: Point, b: Point, p: Point) -> f64 {
        let pa = Coord { x: a.x, y: a.y };
        let pb = Coord { x: b.x, y: b.y };
        let pp = Coord { x: p.x, y: p.y };
        orient2d(pa, pb, pp)
    }
}

const EPS_COLLAPSED: f64 = 1E-8;
pub fn arc_is_collapsed_radius(r: f64) -> bool {
    if r < EPS_COLLAPSED || r.is_nan() {
        return true;
    }
    false
}

pub fn arc_is_collapsed_ends(a: Point, b: Point) -> bool {
    if a.close_enough(b, EPS_COLLAPSED) {
        return true;
    }
    false
}

pub fn arc_check(seg: &Arc) -> bool {
    if arc_is_collapsed_radius(seg.r) || arc_is_collapsed_ends(seg.a, seg.b) {
        return false;
    }
    true
}

#[cfg(test)]
mod test_arc_contains {
    use super::*;

    #[test]
    fn test_arc_contains_01() {
        let arc1 = arc(point(2.0, 1.0), point(1.0, 0.0), point(1.0, 1.0), 1.0);
        assert_eq!(arc1.contains(point(0.0, 0.0)), true);
        assert_eq!(arc1.contains(point(-1.0, 1.0)), true);
    }

    #[test]
    fn test_arc_contains_02() {
        let arc1 = arc(point(-1.0, 1.0), point(1.0, 1.0), point(0.0, 1.0), 1.0);
        assert_eq!(arc1.contains(point(0.0, 0.0)), true);
    }

    #[test]
    fn test_arc_contains_large_r() {
        let arc = arc_circle_parametrization(point(1e20, 30.0), point(10.0, 30.0), 1f64);
        assert_eq!(arc.contains(point(1e20 + 1000.0, 30.0)), true);
    }

    #[test]
    fn test_arc_contains_00() {
        let sgrt_2_2 = std::f64::consts::SQRT_2 / 2.0;
        let arc0 = arc(point(1.0, 1.0), point(0.0, 0.0), point(0.5, 0.5), sgrt_2_2);
        assert!(arc0.contains(point(0.0, 1.0)));
    }

    #[test]
    fn test_arc_contains_03() {
        let arc0 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        assert!(arc0.contains(point(0.0, 1.0)));
    }

    #[test]
    fn test_arc_not_contains() {
        let arc = arc(point(0.0, -1.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        let p = point(-1.0, 0.0);
        assert_eq!(arc.contains(p), false);
    }
}

const ZERO: f64 = 0f64;
const MIN_BULGE: f64 = 1E-8;
pub fn arc_circle_parametrization(pp1: Point, pp2: Point, bulge: f64) -> Arc {
    let mut p1 = pp1;
    let mut p2 = pp2;
    let mut bulge = bulge;
    if bulge.abs() < MIN_BULGE || p1.close_enough(p2, EPS_COLLAPSED) {
        return arcline(pp1, pp2);
    }
    if bulge < 0f64 {
        p1 = pp2;
        p2 = pp1;
        bulge = -bulge;
    }

    let t2 = (p2 - p1).norm();
    let dt2 = (1.0 + bulge) * (1.0 - bulge) / (4.0 * bulge);
    let cx = (0.5 * p1.x + 0.5 * p2.x) + dt2 * (p1.y - p2.y);
    let cy = (0.5 * p1.y + 0.5 * p2.y) + dt2 * (p2.x - p1.x);
    let r = 0.25 * t2 * (1.0 / bulge + bulge).abs();
    arc(p1, p2, point(cx, cy), r)
}

pub fn arc_circle_parametrization_xx(pp1: Point, pp2: Point, bulge: f64) -> Arc {
    let mut p1 = pp1;
    let mut p2 = pp2;
    let mut bulge = bulge;
    if bulge.abs() < MIN_BULGE || p1.close_enough(p2, EPS_COLLAPSED) {
        return arcline(pp1, pp2);
    }
    if bulge < ZERO {
        p1 = pp2;
        p2 = pp1;
        bulge = -bulge;
    }

    let mid_point = Point {
        x: (p1.x + p2.x) / 2.0,
        y: (p1.y + p2.y) / 2.0,
    };

    let d = p2 - p1;
    let chord_length = d.norm();

    let radius = (chord_length * (1.0 + bulge * bulge) / (4.0 * bulge)).abs();

    let distance_to_center = diff_of_prod(radius, radius, chord_length, chord_length / 4.0).sqrt();

    let perp_dx = -d.y / chord_length;
    let perp_dy = d.x / chord_length;

    let direction = if bulge < 1.0 { 1.0 } else { -1.0 };

    let center = Point {
        x: mid_point.x + perp_dx * distance_to_center * direction,
        y: mid_point.y + perp_dy * distance_to_center * direction,
    };
    arc(p1, p2, center, radius)
}

#[cfg(test)]
mod test_arc_circle_parametrization {
    use std::f64::consts::SQRT_2;

    use crate::svg::svg;

    use super::*;

    const _0: f64 = 0f64;
    const _1: f64 = 0f64;
    const _2: f64 = 0f64;

    #[test]
    fn test_arc_circle_parametrization_01() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 200.0), 0.5);
        assert_eq!(
            arc0,
            arc(
                point(100.0, 100.0),
                point(200.0, 200.0),
                point(112.5, 187.5),
                88.38834764831844,
            ),
        );
    }

    #[test]
    #[ignore = "svg output"]
    fn test_arc_circle_parametrization_01_svg() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 200.0), 0.5);
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle = circle(point(arc0.c.x, arc0.c.y), 3.0);
        svg.circle(&circle, "blue");
        svg.write();
    }

    #[test]
    fn test_arc_circle_parametrization_02() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 200.0), 1.5);
        assert_eq!(
            arc0,
            arc(
                point(100.0, 100.0),
                point(200.0, 200.0),
                point(170.83333333333334, 129.16666666666666),
                76.60323462854265,
            ),
        );
    }

    #[test]
    #[ignore = "svg output"]
    fn test_arc_circle_parametrization_02_svg() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 200.0), 1.5);
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle = circle(point(arc0.c.x, arc0.c.y), 3.0);
        svg.circle(&circle, "blue");
        svg.write();
    }

    #[test]
    fn test_arc_circle_parametrization_03() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 200.0), -0.5);
        assert_eq!(
            arc0,
            arc(
                point(200.0, 200.0),
                point(100.0, 100.0),
                point(187.5, 112.5),
                88.38834764831844,
            ),
        );
    }

    #[test]
    #[ignore = "svg output"]
    fn test_arc_circle_parametrization_03_svg() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 200.0), -0.5);
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle = circle(point(arc0.c.x, arc0.c.y), 3.0);
        svg.circle(&circle, "blue");
        svg.write();
    }

    #[test]
    fn test_arc_circle_parametrization_04() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 200.0), -1.5);
        assert_eq!(
            arc0,
            arc(
                point(200.0, 200.0),
                point(100.0, 100.0),
                point(129.16666666666666, 170.83333333333334),
                76.60323462854265,
            ),
        );
    }

    #[test]
    #[ignore = "svg output"]
    fn test_arc_circle_parametrization_04_svg() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(200.0, 200.0), -1.5);
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle = circle(point(arc0.c.x, arc0.c.y), 3.0);
        svg.circle(&circle, "blue");
        svg.write();
    }

    #[test]
    fn test_arc_circle_parametrization_05() {
        let arc0 =
            arc_circle_parametrization(point(1.0, 0.0), point(2.0, 1.0), -1.0 + f64::EPSILON);
        assert_eq!(
            arc0,
            arc(
                point(2.0, 1.0),
                point(1.0, 0.0),
                point(1.5000000000000002, 0.4999999999999999),
                SQRT_2 / 2.0,
            ),
        );
    }

    #[test]
    #[ignore = "svg output"]
    fn test_arc_circle_parametrization_05_svg() {
        let arc0 =
            arc_circle_parametrization(point(1.0, 0.0), point(2.0, 1.0), -1.0 + f64::EPSILON);
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle = circle(point(arc0.c.x, arc0.c.y), 0.1);
        svg.circle(&circle, "blue");
        svg.write();
    }

    #[test]
    fn test_display_01() {
        let arc0 = arc_circle_parametrization(point(1.0, 2.0), point(3.0, 4.0), 3.3);
        assert_eq!(
            "[[1.00000000000000000000, 2.00000000000000000000], [3.00000000000000000000, 4.00000000000000000000], [3.49848484848484808651, 1.50151515151515169144], 2.54772716009334887488]",
            format!("{}", arc0)
        );
    }

    #[test]
    fn test_arc_circle_parametrization_bulge_zero() {
        let arc0 = arc_circle_parametrization(point(1.0, 0.0), point(2.0, 1.0), 0.0);
        assert_eq!(arc0, arcline(point(1.0, 0.0), point(2.0, 1.0),),);
    }

    #[test]
    fn test_arc_circle_parametrization_the_same_points() {
        let arc0 = arc_circle_parametrization(point(2.0, 1.0), point(2.0, 1.0), 1.0);
        assert_eq!(arc0, arcline(point(2.0, 1.0), point(2.0, 1.0),),);
    }

    #[test]
    fn test_arc_circle_parametrization_06() {
        let arc0 = arc_circle_parametrization(
            point(200.0, -200.0),
            point(-200.0, 200.0),
            -1.0 + f64::EPSILON,
        );
        assert_eq!(
            arc0,
            arc(
                point(-200.0, 200.0),
                point(200.0, -200.0),
                point(4.4408920985006274e-14, 4.4408920985006274e-14),
                SQRT_2 * 200.0
            ),
        );
    }

    #[test]
    fn test_arc_circle_parametrization_07() {
        let mut arc0 = arc_circle_parametrization(
            point(200.0, -200.0),
            point(-200.0, 200.0),
            -1.0 + f64::EPSILON,
        );
        arc0.translate(point(200.0, 200.0));
        assert_eq!(
            arc0,
            arc(
                point(0.0, 400.0),
                point(400.0, 0.0),
                point(200.00000000000006, 200.00000000000006),
                SQRT_2 * 200.0,
            ),
        );
    }

    #[test]
    #[ignore = "svg output"]
    fn test_arc_circle_parametrization_07_svg() {
        let mut arc0 = arc_circle_parametrization(point(200.0, -200.0), point(-200.0, 200.0), -1.0);
        arc0.translate(point(200.0, 200.0));
        let mut svg = svg(400.0, 600.0);
        svg.arc(&arc0, "red");
        let circle = circle(point(arc0.c.x, arc0.c.y), 2.0);
        svg.circle(&circle, "blue");
        svg.write();
    }

    #[test]
    fn test_arc_circle_parametrization_08() {
        let arc0 = arc_circle_parametrization_xx(point(1.0, 0.0), point(-1.0, 0.0), 0.00000001);
        assert_eq!(
            arc0,
            arc(
                point(1.0, 0.0),
                point(-1.0, 0.0),
                point(0.0, -49999999.99999999),
                50000000.0,
            ),
        );
    }

    #[test]
    fn test_arc_circle_parametrization_line() {
        let line0 = arc_circle_parametrization(point(100.0, 100.0), point(300.0, 100.0), 0.0);
        assert_eq!(line0, arcline(point(100.0, 100.0), point(300.0, 100.0)));
    }
}

pub fn arc_g_from_points(a: Point, b: Point, c: Point, r: f64) -> f64 {
    let dist = (b - a).norm();
    if dist < 1E-10 {
        return 0f64;
    }

    let pa = Coord { x: a.x, y: a.y };
    let pb = Coord { x: b.x, y: b.y };
    let pc = Coord { x: c.x, y: c.y };
    let perp = orient2d(pa, pb, pc);
    let ddd = (4.0 * r * r) - dist * dist;
    if perp <= 0f64 {
        let seg = r - (0.5 * ddd.sqrt());
        return 2.0 * seg / dist;
    } else {
        let seg = r + (0.5 * ddd.sqrt());
        return 2.0 * seg / dist;
    }
}

#[cfg(test)]
mod test_arc_g_from_points {
    use super::*;

    #[test]
    fn test_a_b_are_close() {
        let a = point(114.31083505599867, 152.84458247200070);
        let b = point(114.31083505599865, 152.84458247200067);
        let arc = arc_circle_parametrization(a, b, 16.0);
        assert_eq!(arc_g_from_points(a, b, arc.c, arc.r), 0.0);
    }

    #[test]
    fn test_a_b_are_the_same() {
        let a = point(114.31083505599865, 152.84458247200067);
        let b = point(114.31083505599865, 152.84458247200067);
        let arc = arc_circle_parametrization(a, b, 16.0);
        assert_eq!(arc_g_from_points(a, b, arc.c, arc.r), 0.0);
    }
}

#[cfg(test)]
mod test_geom_arc_g_from_pt {}

pub fn arc_bound_circle(a: Point, b: Point, g: f64) -> Circle {
    let cx = 0.5 * a.x + 0.5 * b.x;
    let cy = 0.5 * a.y + 0.5 * b.y;
    if g.abs() <= 1f64 {
        let r = 0.5 * (b - a).norm();
        circle(point(cx, cy), r)
    } else {
        let t2 = (b - a).norm();
        let dt2 = (1f64 + g) * (1f64 - g) / (4f64 * g);
        let cx = cx + dt2 * (a.y - b.y);
        let cy = cy + dt2 * (b.x - a.x);

        let r = 0.25 * t2 * (1. / g + g);
        circle(point(cx, cy), r)
    }
}

#[cfg(test)]
mod test_arc_bound_circle {
    use super::*;
    const ONE: f64 = 1f64;
    const ZERO: f64 = 0f64;

    #[test]
    fn test_g_less_1() {
        let v0 = point(-2.0, 1.0);
        let v1 = point(2.0, 1.0);
        let res = circle(point(0f64, 1f64), 2f64);
        assert_eq!(arc_bound_circle(v0, v1, 0.0), res);

        let v0 = point(1.0, 1.0);
        let v1 = point(1.0, 3.0);
        let res = circle(point(1f64, 2f64), 1f64);
        assert_eq!(arc_bound_circle(v0, v1, 1.0), res);
    }

    #[test]
    fn test_g_greater_1() {
        let res = circle(point(0.0, -0.5), 2.5);
        assert_eq!(
            arc_bound_circle(point(-2.0, 1.0), point(2.0, 1.0), 2.0),
            res
        );

        let res = circle(point(5.999999969612645, 1.000000005), 4.999999969612645);
        assert_eq!(
            arc_bound_circle(point(1.0, 1.0), point(1.0, 1.00000001), 2000000000.0),
            res
        );
    }
}

#[derive(Debug, PartialEq)]
pub enum CCWConfig {
    CCW,
    CW,
    COLINEAR,
}

impl Arc {}

impl Arc {}

#[cfg(test)]
mod test_order_points_ccw {

    const ONE: f64 = 1f64;
    const ZERO: f64 = 0f64;
}
