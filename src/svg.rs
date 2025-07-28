#![allow(dead_code)]

use std::fmt::Write as _;

use std::{fs::File, io::Write};

use robust::{orient2d, Coord};

use crate::arc::arc_circle_parametrization;
use crate::circle::{circle, Circle};
use crate::offset_polyline_raw::OffsetRaw;
use crate::pvertex::Polyline;
use crate::segment::{segment, Segment};
use crate::{Arc, Point};

pub struct SVG {
    f: File,
    s: String,
    pub xsize: f64,
    pub ysize: f64,
}

impl SVG {
    #[inline]
    pub fn new(xsize: f64, ysize: f64, file: &str) -> Self {
        let f = if file.is_empty() {
            File::create("/tmp/out.svg").expect("creation failed")
        } else {
            File::create(file).expect("creation failed")
        };
        let s = String::new();
        SVG { f, s, xsize, ysize }
    }
}

#[inline]
pub fn svg(xsize: f64, ysize: f64) -> SVG {
    SVG::new(xsize, ysize, "")
}

impl SVG {
    pub fn write(&mut self) {
        let mut header = String::new();
        write!(
            &mut header,
            r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" fill="none" stroke-width="1.0" stroke-linecap="round">"#,
            self.xsize, self.ysize
        ).unwrap();
        write!(
            &mut header,
            "<rect width=\"100%\" height=\"100%\" fill=\"#ffffffff\" />"
        )
        .unwrap();
        header.push_str("\n");

        header.push_str(self.s.as_str());

        let footer = r#"</svg>"#.to_owned();
        header.push_str(footer.as_str());
        self.f.write_all(header.as_bytes()).expect("write failed");
    }

    pub fn circle(&mut self, circle: &Circle, color: &str) {
        let mut s = String::new();
        write!(
            &mut s,
            r#"<circle cx="{}" cy="{}" r="{}" stroke="{}" />"#,
            circle.c.x,
            self.ysize - circle.c.y,
            circle.r,
            color
        )
        .unwrap();
        self.s.push_str(&s);
        self.s.push_str("\n");
    }

    pub fn text(&mut self, p: Point, text: &str, color: &str) {
        let mut s = String::new();
        write!(
            &mut s,
            r#"<text x="{}" y="{}" fill="{}" font-size="2.0">{}</text>"#,
            p.x + 0.0,
            self.ysize - p.y + 0.0,
            color,
            text
        )
        .unwrap();
        self.s.push_str(&s);
        self.s.push_str("\n");
    }

    pub fn line(&mut self, segment: &Segment, color: &str) {
        let mut s = String::new();
        write!(
            &mut s,
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" />"#,
            segment.a.x,
            self.ysize - segment.a.y,
            segment.b.x,
            self.ysize - segment.b.y,
            color
        )
        .unwrap();
        self.s.push_str(&s);
        self.s.push_str("\n");
    }

    pub fn arc(&mut self, arc: &Arc, color: &str) {
        let mut s = String::new();
        let pa = Coord {
            x: arc.a.x,
            y: arc.a.y,
        };
        let pb = Coord {
            x: arc.b.x,
            y: arc.b.y,
        };
        let pc = Coord {
            x: arc.c.x,
            y: arc.c.y,
        };

        let large_arc_flag = if orient2d(pa, pb, pc) < 0.0 { 1 } else { 0 };
        write!(
            &mut s,
            r#"<path d="M {} {} A {} {} {} {} {} {} {}" stroke="{}" />"#,
            arc.a.x,
            self.ysize - arc.a.y,
            arc.r,
            arc.r,
            0,
            large_arc_flag,
            0, // always 0 because arc_circle_parametrization always creates CCW arcs
            arc.b.x,
            self.ysize - arc.b.y,
            color
        )
        .unwrap();
        self.s.push_str(&s);
        self.s.push_str("\n");
    }

    pub fn pvertex(&mut self, p0: Point, p1: Point, g: f64, color: &str) {
        if g == 0f64 {
            // line segment
            let seg = segment(p0, p1);
            self.line(&seg, color);
        } else {
            let arc = arc_circle_parametrization(p0, p1, g);
            self.arc(&arc, color);
        }
        //self.circle(&circle(p0, 0.5), "blue");
        //self.circle(&circle(p1, 0.5), "blue");
    }

    pub fn polyline(&mut self, pline: &Polyline, color: &str) {
        let last = pline.len() - 2;
        for i in 0..=last {
            let p0 = pline[i];
            let p1 = pline[i + 1];
            self.pvertex(p0.p, p1.p, p0.g, color);
        }
        // close pline
        let p0 = pline.last().unwrap();
        self.pvertex(p0.p, pline[0].p, p0.g, color);
    }

    pub fn offset_segment(&mut self, off: &Arc, color: &str) {
        if off.is_line() {
            // line segment
            let seg = segment(off.a, off.b);
            self.line(&seg, color);
        } else {
            self.arc(off, color);
        }
        // self.circle(&circle(off.a, 0.1), "green");
        // self.circle(&circle(off.b, 0.1), "green");
        //self.text(off.a, &off.id.to_string(), color);
    }

    pub fn offset_segments_single(&mut self, offs: &Vec<Arc>, color: &str) {
        for s in offs.iter() {
            self.offset_segment(s, color);
        }
    }

    pub fn offset_segments(&mut self, offs: &Vec<Vec<Arc>>, color: &str) {
        for s in offs.iter() {
            self.offset_segments_single(s, color);
        }
    }

    pub fn polysegment(&mut self, off: &Arc, color: &str) {
        if off.is_line() {
            // line segment
            let seg = segment(off.a, off.b);
            self.line(&seg, color);
        } else {
            self.arc(off, color);
        }
        //self.circle(&circle(off.a, 0.3), "green");
        //self.circle(&circle(off.b, 0.2), "green");
    }

    pub fn polysegments(&mut self, vseg: &Vec<Arc>, color: &str) {
        for s in vseg.iter() {
            self.offset_segment(s, color);
        }
    }

    pub fn polyvsegments(&mut self, vseg: &Vec<Vec<Arc>>, color: &str) {
        for s in vseg.iter() {
            self.polysegments(s, color);
        }
    }

    pub fn offset_raws_single(&mut self, segments: &Vec<OffsetRaw>, color: &str) {
        for s in segments.iter() {
            self.offset_segment(&s.arc, color);
        }
    }

    pub fn offset_raws(&mut self, segments: &Vec<Vec<OffsetRaw>>, color: &str) {
        for s in segments.iter() {
            self.offset_raws_single(s, color);
        }
    }
}

#[cfg(test)]
mod test_svg {

    use super::*;

    #[test]
    fn test_new() {
        let s0 = SVG::new(400.0, 300.0, "");
        let s1 = svg(400.0, 300.0);
        assert_eq!(s0.xsize, s1.xsize);
        assert_eq!(s0.ysize, s1.ysize);
    }
}
