#![allow(dead_code)]

use crate::{arc::arc, Arc, OffsetRaw, Point};

const ZERO: f64 = 0f64;


pub fn offset_connect_raw(raws: &Vec<OffsetRaw>, off: f64) -> Vec<OffsetRaw> {
    let mut res = Vec::with_capacity(2 * raws.len() + 1); 
    let last = raws.len() - 1;
    for i in 0..last {
        
        let old = raws[i].arc;
        let old_next = raws[i + 1].arc;
        let g0 = raws[i].g;
        let g1 = raws[i + 1].g;
        let orig = raws[i].orig;
        let connect = arc_connect(old, old_next, g0, g1, orig, off);
        
        res.push (OffsetRaw {
            arc: connect.0,
            orig,
            g: connect.1,
        });
    }
    
    let last = raws.last().unwrap();
    let old = last.arc;
    let raw_next = raws.first().unwrap();
    let old_next = raw_next.arc;
    let g0 = last.g;
    let g1 = raw_next.g;
    let orig = last.orig;
    let connect = arc_connect(old, old_next, g0, g1, orig, off);
    
    res.push (OffsetRaw {
        arc: connect.0,
        orig,
        g: connect.1,
    });
    res
}

fn arc_connect(old: Arc, old_next: Arc, g0: f64, g1: f64, orig: Point, off: f64) -> (Arc, f64) {
    if g0 >= ZERO && g1 >= ZERO {
        (arc(old.b, old_next.a, orig, off), 0.5) 
    } else if g0 >= ZERO && g1 < ZERO {
        (arc(old.b, old_next.a, orig, off), 0.5) 
    } else if g0 < ZERO && g1 >= ZERO {
        (arc(old_next.a, old.b, orig, off), -0.5)
    } else {
        
        (arc(old_next.a, old.b, orig, off), -0.5)
    }
}

#[cfg(test)]
mod test_offset_connect_raw {
    use crate::{offset_polyline_raw, pline_01, point::point, pvertex::{polyline_translate, pvertex}, svg::svg};

    use super::*;

    #[test]
    
    fn test_offset_connect_segments_arcs_00() {
        let pline = vec![
            pvertex(point(100.0, 100.0), 0.5),   
            pvertex(point(200.0, 200.0), 0.5),
        ];
        let mut svg = svg(400.0, 600.0);
        let pline = polyline_translate(&pline, point(0.0, 100.0));
        svg.polyline(&pline, "grey");

        
        let off: f64 = 52.25;
        let offset_raw1 = offset_polyline_raw(&pline, off);
        let offset_raw2 = offset_connect_raw(&offset_raw1, off);
        let mut offset_raw3 = Vec::new();
        offset_raw3.extend(offset_raw1);
        offset_raw3.extend(offset_raw2);
        
        svg.offset_raws(&offset_raw3, "red");
        svg.write();
    }

    #[test]
    
    fn test_offset_connect_segments_arcs_01() {
        let pline = vec![
            pvertex(point(100.0, 100.0), 0.5),
            pvertex(point(200.0, 100.0), 0.5),
            pvertex(point(300.0, 200.0), -0.5),
            pvertex(point(200.0, 300.0), -0.5),
            pvertex(point(100.0, 300.0), 0.5),
            pvertex(point(0.0, 200.0), 0.5),
        ];
        let mut svg = svg(400.0, 600.0);
        let pline = polyline_translate(&pline, point(10.0, 100.0));
        svg.polyline(&pline, "grey");

        
        let off: f64 = 52.25;
        let offset_raw1 = offset_polyline_raw(&pline, off);
        let offset_raw2 = offset_connect_raw(&offset_raw1, off);
        let mut offset_raw3 = Vec::new();
        offset_raw3.extend(offset_raw1);
        offset_raw3.extend(offset_raw2);
        
        svg.offset_raws(&offset_raw3, "red");
        svg.write();
    }

    #[test]
    
    fn test_offset_connect_segments_lines_01() {
        let pline = vec![
            pvertex(point(100.0, 100.0), 0.0),
            pvertex(point(200.0, 100.0), 0.0),
            pvertex(point(200.0, 200.0), 0.0),
            pvertex(point(100.0, 200.0), 0.0),
        ];
        let mut svg = svg(400.0, 600.0);
        
        svg.polyline(&pline, "grey");

        
        let off: f64 = 52.25;
        let offset_raw1 = offset_polyline_raw(&pline, off);
        let offset_raw2 = offset_connect_raw(&offset_raw1, off);
        let mut offset_raw3 = Vec::new();
        
        offset_raw3.extend(offset_raw2);
        
        svg.offset_raws(&offset_raw3, "red");
        svg.write();
    }

    #[test]
    
    fn test_offset_connect_segments_02() {
        let pline = vec![
            pvertex(point(100.0, 100.0), -0.4),
            pvertex(point(200.0, 100.0), -0.4),
            pvertex(point(200.0, 200.0), -0.4),
            pvertex(point(100.0, 200.0), -0.4),
        ];
        let mut svg = svg(400.0, 600.0);
        let pline = polyline_translate(&pline, point(0.0, 100.0));
        svg.polyline(&pline, "grey");

        
        
        let off: f64 = 62.00;
        let offset_raw1 = offset_polyline_raw(&pline, off);
        let offset_raw2 = offset_connect_raw(&offset_raw1, off);
        let mut offset_raw3 = Vec::new();
        offset_raw3.extend(offset_raw1);
        offset_raw3.extend(offset_raw2);
        
        svg.offset_raws(&offset_raw3, "red");
        svg.write();
    }

    #[test]
    
    fn test_offset_connect_segments_03() {
        let pline = pline_01();
        let mut svg = svg(400.0, 600.0);
        let pline = polyline_translate(&pline, point(150.0, 200.0));
        svg.polyline(&pline, "grey");

        let off: f64 = 62.00;
        let offset_raw1 = offset_polyline_raw(&pline, off);
        let offset_raw2 = offset_connect_raw(&offset_raw1, off);
        let mut offset_raw3 = Vec::new();
        offset_raw3.extend(offset_raw1);
        offset_raw3.extend(offset_raw2);
        
        svg.offset_raws(&offset_raw3, "red");
        svg.write();
    }
}
