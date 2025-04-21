#![allow(dead_code)]

use robust::{orient2d, Coord};

use crate::{
    circle::{circle, Circle},
    point::{almost_equal_as_int, point, Point},
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
    pub fn contains_ulps(self: Self, p: Point, ulps: i64) -> bool {
        let length = (p - self.c).norm_imp();
        if almost_equal_as_int(length, self.r, ulps) {
            let diff_pa = p - self.a;
            let diff_ba = self.b - self.a;
            let perp = diff_pa.perp_imp(diff_ba);
            return perp >= 0f64;
        } else {
            return false;
        }
    }

    pub fn contains_eps(self: Self, p: Point, eps: f64) -> bool {
        let length = (p - self.c).norm_imp();
        if (length - self.r).abs() <= eps {
            let diff_pa = p - self.a;
            let diff_ba = self.b - self.a;
            let perp = diff_pa.perp_imp(diff_ba);
            return perp >= 0f64;
        } else {
            return false;
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        let diff_pa = p - self.a;
        let diff_ba = self.b - self.a;
        let perp = diff_pa.perp_imp(diff_ba);
        return perp >= 0f64;
    }
}

#[inline]

pub fn contains_order2d(a: Point, b: Point, p: Point) -> f64 {
    let pa = Coord { x: a.x, y: a.y };
    let pb = Coord { x: b.x, y: b.y };
    let pp = Coord { x: p.x, y: p.y };
    orient2d(pa, pb, pp)
}

pub fn sort_points_on_arc(a: Point, b: Point, x0: Point, x1: Point) -> (Point, Point, Point, Point) {
    let pa = Coord { x: a.x, y: a.y };
    let px0 = Coord { x: x0.x, y: x0.y };
    let px1 = Coord { x: x1.x, y: x1.y };
    let pb = Coord { x: b.x, y: b.y };
    let mut tt = (pa, px0, px1, pb);
    let orient = orient2d(tt.0, tt.1, tt.2);
    if orient < 0.0 {
        tt = (tt.0, tt.2, tt.1, tt.3)
    }
    let rx0 = point(tt.1.x, tt.1.y);
    let rx1 = point(tt.2.x, tt.2.y);
    (a, rx0, rx1, b)
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
    fn test_arc_contains_issue() {
        let arc = arc(
            point(1591.8964578782, 30.0),
            point(8.1035421218, 30.0),
            point(800.0, -200.0),
            824.62112512355623,
        );
        assert_eq!(arc.contains_ulps(point(1560.6068185945, 30.0), 5), false);
    }

    #[test]
    fn test_point_on_arc() {
        let arc = arc(point(1.0, 0.0), point(-1.0, 0.0), point(0.0, 0.0), 1.0);
        assert_eq!(arc.contains_ulps(point(0.0, 1.0), 5), true);
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

fn arc_circle_parametrization_full(aa: Point, bb: Point, gg: f64) -> (Arc, f64, f64) {
    let mut a = aa;
    let mut b = bb;
    let mut g = gg;
    if g < 0f64 {
        
        a = bb;
        b = aa;
        g = -gg;
    }
    if g == 0f64 {
        
        let arc = arcline(a, b);
        return (arc, f64::INFINITY, f64::INFINITY);
    }
    let dist = (b - a).norm_imp();
    let dt2 = (1.0 + g) * (1.0 - g) / (4.0 * g);
    let cx = (0.5 * a.x + 0.5 * b.x) + dt2 * (a.y - b.y);
    let cy = (0.5 * a.y + 0.5 * b.y) + dt2 * (b.x - a.x);
    let r = 0.25 * dist * (1.0 / g + g).abs();
    let theta0 = (a.y - cy).atan2(a.x - cx);
    let theta1 = (b.y - cy).atan2(b.x - cx);
    let arc = arc(a, b, point(cx, cy), r);
    (arc, theta0, theta1)
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
    let t2 = (b - a).norm_imp();
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
    fn test_arc_circle_parametrization_full() {
        
        let (arc0, theta0, theta1) =
            arc_circle_parametrization_full(point(100.0, 100.0), point(300.0, 100.0), -1.0);
        assert_eq!(
            arc0,
            arc(
                point(300.0, 100.0),
                point(100.0, 100.0),
                point(200.0, 100.0),
                100.0,
            ),
        );
        assert_eq!(theta0, 0.0);
        assert_eq!(theta1, std::f64::consts::PI);
    }

    #[test]
    fn test_arc_circle_parametrization_full2() {
        
        let (arc0, theta0, theta1) =
            arc_circle_parametrization_full(point(100.0, 100.0), point(300.0, 100.0), 0.0);
        assert_eq!(arc0, arcline(point(100.0, 100.0), point(300.0, 100.0)),);
        assert_eq!(theta0, f64::INFINITY);
        assert_eq!(theta1, f64::INFINITY);
    }

    #[test]
    fn test_arc_circle_parametrization_line() {
        
        let line0 = arc_circle_parametrization(point(100.0, 100.0), point(300.0, 100.0), 0.0);
        assert_eq!(line0, arcline(point(100.0, 100.0), point(300.0, 100.0)));
    }
}

pub fn arc_g_from_points(a: Point, b: Point, c: Point, r: f64) -> f64 {
    let dist = (b - a).norm_imp();
    if dist < 1E-10 {
        
        
        return 0f64;
    }
    
    let diff_pa = c - a;
    let diff_ba = b - a;
    let perp = diff_pa.perp_imp(diff_ba);
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
mod test_geom_arc_g_from_pt {
   
}


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
        
        let r = 0.5 * (b - a).norm_imp();
        circle(point(cx, cy), r)
    } else {
        let t2 = (b - a).norm_imp();
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

impl Arc {
    
    fn order_ccw(&self, b: Point, c: Point) -> (Point, Point) {
        let zero: f64 = 0f64;
        let cc = Coord {
            x: self.c.x,
            y: self.c.y,
        };
        let pa = Coord {
            x: self.a.x,
            y: self.a.y,
        };
        let pb = Coord { x: b.x, y: b.y };
        let pc = Coord { x: c.x, y: c.y };
        let cab = orient2d(cc, pa, pb);
        let cac = orient2d(cc, pa, pc);
        let cbc = orient2d(cc, pb, pc);
        
        if (cab > zero && cac > zero) || (cab < zero && cac < zero) {
            if cbc > zero {
                return (b, c); 
            } else {
                if cbc < zero {
                    return (c, b); 
                } else {
                    return (b, c); 
                }
            }
        }
        if cab >= zero && cac < zero {
            return (b, c); 
        }
        if cab < zero && cac >= zero {
            return (c, b); 
        }
        if (cab == zero && cac > zero) || (cac == zero && cab > zero) {
            if cbc > zero {
                return (b, c); 
            }
            if cbc < zero {
                return (c, b); 
            }
        }
        if cab == zero && cac == zero {
            
            return (b, c); 
        }

        
        return todo!();
    }
}



impl Arc {
    
    pub fn order_points_ccw(&self, p0: Point, p1: Point) -> (Point, Point) {
        let (t0, t1) = self.order_points_ccw_int(p0, p1);
        let (tt0, tt1) = self.sort_points_on_circle(p0, p1);
        let (ttt0, ttt1) = self.order_ccw(p0, p1);
       
        return (ttt0, ttt1);
    }

    pub fn order_points_ccw_int(&self, p0: Point, p1: Point) -> (Point, Point) {
        if p0 == p1 {
            return (p0, p1);
        }
        let arc = arc(self.a, p1, self.c, self.r);
        if arc.contains(p0) {
            return (p0, p1);
        } else {
            return (p1, p0);
        }
    }

    
    
    
    
    fn sort_points_on_circle(&self, p0: Point, p1: Point) -> (Point, Point) {
        let d = self.a - self.c;
        let d_perp = point(-d.y, d.x);

        
        
        
        let v = point(p0.x - self.c.x, p0.y - self.c.y);
        let w_point = point(d.x * v.x + d.y * v.y, d_perp.x * v.x + d_perp.y * v.y);
        let w1 = w_point;
        let v = point(p1.x - self.c.x, p1.y - self.c.y);
        let w_point = point(d.x * v.x + d.y * v.y, d_perp.x * v.x + d_perp.y * v.y);
        let w2 = w_point;
        if less_than_by_geometry(w1, w2) {
            (p0, p1)
        } else {
            (p1, p0)
        }
    }
}

fn less_than_by_geometry(w0: Point, w1: Point) -> bool {
    let x0 = w0.x;
    let y0 = w0.y;
    let x1 = w1.x;
    let y1 = w1.y;
    const ZERO: f64 = 0f64;

    if y0 < ZERO && y1 >= ZERO {
        return true;
    }
    if y1 < ZERO && y0 >= ZERO {
        return false;
    }
    if y0 > ZERO && y1 == ZERO {
        return x1 < ZERO;
    }
    if y1 > ZERO && y0 == ZERO {
        return x0 > ZERO;
    }
    if y0 == ZERO && y1 == ZERO {
        return (x1 < ZERO && x1 < x0) || (x0 > ZERO && x1 > x0);
    }
    let c = x0 * y1 - x1 * y0;
    if c > ZERO {
        return true;
    }
    if c < ZERO {
        return false;
    }
    return (x0 - x1) * (x0 + x1) < (y1 - y0) * (y1 + y0); 
}

#[cfg(test)]
mod test_order_points_ccw {
    use crate::point::point;

    use crate::arc::arc;

    const ONE: f64 = 1f64;
    const ZERO: f64 = 0f64;

    #[test]
    #[ignore]
    fn test_ccw() {
        let arc = arc(point(1.0, 0.0), point(0.0, -1.0), point(0.0, 0.0), 1.0);
        
        assert_eq!(
            arc.order_points_ccw(point(0.0, 1.0), point(0.0, 1.0)),
            (point(0.0, 1.0), point(0.0, 1.0))
        );
        
        assert_eq!(
            arc.order_points_ccw(point(1.0, 0.0), point(1.0, 0.0)),
            (point(1.0, 0.0), point(1.0, 0.0))
        );
        
        assert_eq!(
            arc.order_points_ccw(point(0.0, 1.0), point(-1.0, 0.0)),
            (point(0.0, 1.0), point(-1.0, 0.0))
        );
        
        assert_eq!(
            arc.order_points_ccw(point(-1.0, 0.0), point(0.0, 1.0)),
            (point(0.0, 1.0), point(-1.0, 0.0))
        );
    }
}
