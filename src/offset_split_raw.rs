#![allow(dead_code)]

use crate::{
    arc::{arc, arc_is_collapsed_ends, arc_is_collapsed_radius, arcline},
    int_arc_arc::{intersect_arc_arc, ArcArcConfig},
    int_segment_segment::{intersect_segment_segment, SegmentConfig},
    segment::segment,
    Arc, OffsetRaw,
};

static ZERO: f64 = 0.0;

pub fn offset_split_raw(raws: &Vec<OffsetRaw>) -> Vec<OffsetRaw> {
    let res = Vec::with_capacity(2 * raws.len() + 1);

    res
}

fn split_line_line(arc0: &Arc, arc1: &Arc) -> (Vec<Arc>, i64) {
    let mut res = Vec::new();
    let mut count = 0;

    let seg0 = segment(arc0.a, arc0.b);
    let seg1 = segment(arc1.a, arc1.b);
    let inter = intersect_segment_segment(seg0, seg1);
    match inter {
        SegmentConfig::NoIntersection() => {
            check_and_push(&mut res, arc0.clone());
            check_and_push(&mut res, arc1.clone());
        }
        SegmentConfig::OnePoint(sp, _, _) => {
            let line00 = arcline(sp, arc0.a);
            let line01 = arcline(sp, arc0.b);
            let line10 = arcline(sp, arc1.a);
            let line11 = arcline(sp, arc1.b);
            check_and_push(&mut res, line00);
            check_and_push(&mut res, line01);
            check_and_push(&mut res, line10);
            check_and_push(&mut res, line11);
            count = 4;
        }
        SegmentConfig::TwoPoints(p0, p1, p2, p3) => {
            let line00 = arcline(p0, p1);
            let line01 = arcline(p1, p2);
            let line10 = arcline(p2, p3);
            check_and_push(&mut res, line00);
            check_and_push(&mut res, line01);
            check_and_push(&mut res, line10);
            count = 3;
        }
    }

    (res, count)
}

fn split_arc_arc(arc0: &Arc, arc1: &Arc) -> (Vec<Arc>, i64) {
    let mut res = Vec::new();
    let mut count = 0;
    let inter = intersect_arc_arc(&arc0, &arc1);
    match inter {
        ArcArcConfig::NoIntersection()
        | ArcArcConfig::CocircularOnePoint0(_)
        | ArcArcConfig::CocircularOnePoint1(_)
        | ArcArcConfig::CocircularTwoPoints(_, _) => {
            check_and_push(&mut res, arc0.clone());
            check_and_push(&mut res, arc1.clone());
        }
        ArcArcConfig::NonCocircularOnePoint(p) => {
            let arc00 = arc(arc0.a, p, arc0.c, arc0.r);
            let arc01 = arc(p, arc0.b, arc0.c, arc0.r);
            let arc10 = arc(arc1.a, p, arc1.c, arc1.r);
            let arc11 = arc(p, arc1.b, arc1.c, arc1.r);
            check_and_push(&mut res, arc00);
            check_and_push(&mut res, arc01);
            check_and_push(&mut res, arc10);
            check_and_push(&mut res, arc11);
            count = 4;
        }
        ArcArcConfig::NonCocircularTwoPoints(point0, point1) => {
            let mut p0 = point0;
            let mut p1 = point1;
            if Arc::contains_order2d(arc0.a, p0, p1) < ZERO {
                (p1, p0) = (p0, p1);
            }
            let arc00 = arc(arc0.a, p0, arc0.c, arc0.r);
            let arc01 = arc(p0, p1, arc0.c, arc0.r);
            let arc02 = arc(p1, arc0.b, arc0.c, arc0.r);

            if Arc::contains_order2d(arc1.a, p0, p1) < ZERO {
                (p1, p0) = (p0, p1);
            }
            let arc10 = arc(arc1.a, p0, arc1.c, arc1.r);
            let arc11 = arc(p0, p1, arc1.c, arc1.r);
            let arc12 = arc(p1, arc1.b, arc1.c, arc1.r);

            check_and_push(&mut res, arc00);
            check_and_push(&mut res, arc01);
            check_and_push(&mut res, arc02);
            check_and_push(&mut res, arc10);
            check_and_push(&mut res, arc11);
            check_and_push(&mut res, arc12);
            count = 6;
        }
        ArcArcConfig::CocircularOnePointOneArc0(_, _) => {
            let arc00 = arc(arc0.a, arc1.b, arc0.c, arc0.r);
            let arc01 = arc(arc1.b, arc0.b, arc0.c, arc0.r);
            let arc02 = arc(arc0.b, arc0.a, arc0.c, arc0.r);
            check_and_push(&mut res, arc00);
            check_and_push(&mut res, arc01);
            check_and_push(&mut res, arc02);
            count = 3;
        }
        ArcArcConfig::CocircularOnePointOneArc1(_, _) => {
            let arc00 = arc(arc1.a, arc0.b, arc0.c, arc0.r);
            let arc01 = arc(arc0.b, arc1.b, arc0.c, arc0.r);
            let arc02 = arc(arc1.b, arc1.a, arc0.c, arc0.r);
            check_and_push(&mut res, arc00);
            check_and_push(&mut res, arc01);
            check_and_push(&mut res, arc02);
            count = 3;
        }
        ArcArcConfig::CocircularOneArc0(_) => {
            let arc00 = arc(arc0.a, arc0.b, arc0.c, arc0.r);
            check_and_push(&mut res, arc00);
            count = 1;
        }
        ArcArcConfig::CocircularOneArc1(_) => {
            let arc00 = arc(arc1.a, arc0.a, arc0.c, arc0.r);
            let arc01 = arc(arc0.a, arc0.b, arc0.c, arc0.r);
            let arc02 = arc(arc0.b, arc1.b, arc0.c, arc0.r);
            check_and_push(&mut res, arc00);
            check_and_push(&mut res, arc01);
            check_and_push(&mut res, arc02);
            count = 3;
        }
        ArcArcConfig::CocircularOneArc2(_) => todo!(),
        ArcArcConfig::CocircularOneArc3(_) => todo!(),
        ArcArcConfig::CocircularOneArc4(_) => todo!(),
        ArcArcConfig::CocircularTwoArcs(_, _) => todo!(),
    }
    (res, count)
}

fn check_and_push(res: &mut Vec<Arc>, arc: Arc) {
    if arc.a == arc.b {
        return;
    }
    if arc_is_collapsed_radius(arc.r) || arc_is_collapsed_ends(arc.a, arc.b) {
        let line = arcline(arc.a, arc.b);
        res.push(line);
    }
    res.push(arc);
}

#[cfg(test)]
mod test_offset_split_raw {

    use crate::{offsetraw, point::point, svg::svg};

    use super::*;

    #[test]

    fn test_offset_split_arcs_01() {
        let arc0 = arcline(point(100.0, 100.0), point(200.0, 100.0));
        let arc1 = arcline(point(150.0, 100.0), point(300.0, 100.0));
        let raw0 = offsetraw(arc0.clone(), point(200.0, 100.0), 3.3);
        let raw1 = offsetraw(arc1.clone(), point(200.0, 100.0), 3.3);

        split_line_line(&arc0, &arc1);

        let mut svg = svg(400.0, 600.0);
        let mut offset_raw = Vec::new();
        offset_raw.push(raw0);
        offset_raw.push(raw1);

        svg.offset_raws(&offset_raw, "red");
        svg.write();
    }
}
