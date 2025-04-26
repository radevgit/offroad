#![allow(dead_code)]

use robust::{orient2d, Coord};

use crate::{
    circle::{circle, Circle},
    point::{point, Point},
};

use std::fmt::Display;

pub type Arcline = Vec<Arc>;

static mut ID_COUNT: u32 = 0;

#[derive(Debug, Copy, Clone)]
pub struct Arc {
    pub a: Point,
    pub b: Point,
    pub c: Point,
    pub r: f64,
    pub id: u32,
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
        let id = unsafe { ID_COUNT };
        unsafe { ID_COUNT = ID_COUNT + 1 };
        Arc { a, b, c, r, id }
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
    #[inline]

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

const EPS_COLLAPSED: f64 = 1E-10;
pub fn arc_is_collapsed_radius(r: f64) -> bool {
    if r.abs() < EPS_COLLAPSED {
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

#[cfg(test)]
mod test_arc_contains {
    use super::*;

    #[test]
    fn test_arc_contains() {
        let arc1 = arc(point(2.0, 1.0), point(1.0, 0.0), point(1.0, 1.0), 1.0);
        assert_eq!(arc1.contains(point(0.0, 0.0)), true);
        assert_eq!(arc1.contains(point(-1.0, 1.0)), true);
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
    fn test_arc_contains_01() {
        let arc0 = arc(point(1.0, 0.0), point(0.0, 1.0), point(0.0, 0.0), 1.0);
        assert!(arc0.contains(point(0.0, 1.0)));
    }
}

pub fn arc_circle_parametrization(aa: Point, bb: Point, gg: f64) -> Arc {
    let mut a = aa;
    let mut b = bb;
    let mut g = gg;
    if g < 0f64 {
        a = bb;
        b = aa;
        g = -gg;
    }
    if g == 0f64 {
        return arcline(a, b);
    }
    let t2 = (b - a).norm();
    let dt2 = (1.0 + g) * (1.0 - g) / (4.0 * g);
    let cx = (0.5 * a.x + 0.5 * b.x) + dt2 * (a.y - b.y);
    let cy = (0.5 * a.y + 0.5 * b.y) + dt2 * (b.x - a.x);
    let r = 0.25 * t2 * (1.0 / g + g).abs();
    arc(a, b, point(cx, cy), r)
}

#[cfg(test)]
mod test_arc_circle_parametrization {
    use super::*;

    const _0: f64 = 0f64;
    const _1: f64 = 0f64;
    const _2: f64 = 0f64;

    #[test]
    fn test_arc_circle_parametrization() {
        let arc0 = arc_circle_parametrization(point(100.0, 100.0), point(300.0, 100.0), -1.0);
        assert_eq!(
            arc0,
            arc(
                point(300.0, 100.0),
                point(100.0, 100.0),
                point(200.0, 100.0),
                100.0,
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

fn qsolve(neg: bool, a: f64, b: f64) -> f64 {
    if 0.0 == a {
        return 0.0;
    }
    if 0.0 == b {
        return 1.0;
    }
    let r;
    if b.abs() > a.abs() {
        r = a / b;
        if neg {
            return r / (1. + (1. + r * r).sqrt());
        } else {
            return -r / (1. + (1. - r * r).sqrt());
        }
    } else {
        r = b / a;
        if !neg {
            return -r;
        } else {
            if r > 0.0 {
                return 1. / (r + (1. + r * r).sqrt());
            } else {
                return 1. / (r - (1. + r * r).sqrt());
            }
        }
    }
}

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
