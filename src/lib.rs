
mod arc_string;

mod arc;
pub use crate::arc::Arc;


mod circle;
pub use crate::circle::Circle;

mod dist_arc_arc;
mod dist_line_circle;
mod dist_point_arc;
mod dist_point_circle;
mod dist_point_segment;
mod dist_segment_arc;
mod dist_segment_circle;
mod dist_segment_segment;

mod int_arc_arc;
mod int_circle_circle;
mod int_interval_interval;
mod int_line_arc;
mod int_line_circle;
mod int_line_line;

mod int_segment_point;
mod int_segment_arc;
mod int_segment_circle;
mod int_segment_segment;
mod intersect;

mod interval;
mod line;

mod offset;
pub use crate::offset::pline_01;

mod offset_connect_raw;
pub use crate::offset_connect_raw::offset_connect_raw;

mod offset_polyline_raw;
pub use crate::offset_polyline_raw::OffsetRaw;
pub use crate::offset_polyline_raw::offset_polyline_raw;
pub use crate::offset_polyline_raw::offsetraw;

mod offset_split_arcs;
pub use crate::offset_split_arcs::offset_split_arcs;


mod offset_prune_invalid_offsets;

mod point;
pub use crate::point::Point;

mod pvertex;
pub use crate::pvertex::PVertex;
pub use crate::pvertex::Polyline;

mod segment;

mod svg;
mod utils;

#[cfg(test)]
mod tests;


#[macro_export]
macro_rules! expect_float_absolute_eq {
    
    ($a:expr, $b:expr, $epsilon:expr) => {{
        let (a, b, eps) = ($a, $b, $epsilon);
        let r = $crate::afe_is_absolute_eq!(a, b, eps);
        let e = $crate::AbsoluteEqError::new(a, b, eps);
        $crate::bool_to_result(r, e)
    }};
    
    ($a:expr, $b:expr) => {
        $crate::expect_float_absolute_eq!($a, $b, 1.0e-6)
    };
}

