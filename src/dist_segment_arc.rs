#![allow(dead_code)]

use crate::{
    dist_point_arc::distance_point_arc, dist_point_segment::distance_point_segment,
    int_segment_arc::intersect_segment_arc, point::point, segment::Segment, Arc, Point,
};

pub fn distance_segment_arc(seg: Segment, arc: &Arc) -> (Point, f64) {
    let inter = intersect_segment_arc(seg, arc);
    match inter {
        crate::int_segment_arc::SegmentArcConfig::NoIntersection() => {
            let mut v: Vec<f64> = Vec::new();
            v.push(distance_point_arc(seg.p0, arc).1);
            v.push(distance_point_arc(seg.p1, arc).1);
            v.push(distance_point_segment(arc.a, seg).1);
            v.push(distance_point_segment(arc.b, seg).1);
            let mm;
            let m = v.into_iter().min_by(|a, b| a.partial_cmp(b).unwrap());
            match m {
                Some(d) => {
                    mm = d;
                }
                None => {
                    mm = f64::INFINITY;
                }
            }
            return (point(0.0, 0.0), mm);
        }
        _ => return (point(0.0, 0.0), 0f64),
    }
}

#[cfg(test)]
mod tests_distance_segment_arc {

    #[test]
    fn test_distance_segment_arc() {}
}
