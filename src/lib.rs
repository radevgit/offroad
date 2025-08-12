#![doc(html_no_source)]

//! 2D offsetting for arc polylines/polygons.
//!
//! # Examples
//!
//! Check "examples" directory for usage examples.


// Offsetting algorithm components
pub mod offset;
// 
pub mod offset_raw;
// raw offsetting components (lines, arcs)
mod offset_polyline_raw;
// connect raw offsets with arcs
mod offset_connect_raw;
// split raw offsets into segments in intersection points
mod offset_split_arcs;
// prune invalid offsets that are close to original polylines
mod offset_prune_invalid;
// resulting soup of arcs is ordered and reconnected
mod offset_reconnect_arcs;


// Re-export main offsetting functions
// For public API
pub mod prelude {
    pub use crate::offset::{offset_polyline, offset_polyline_multiple, OffsetCfg};
    pub use crate::offset::{pline_01, pline_02, pline_03, pline_04};
}
// For internal use
pub use crate::offset_polyline_raw::{offset_polyline_raw, poly_to_raws};
pub use crate::offset_connect_raw::offset_connect_raw;
pub use crate::offset_split_arcs::offset_split_arcs;
pub use crate::offset_prune_invalid::offset_prune_invalid;
pub use crate::offset_reconnect_arcs::{offset_reconnect_arcs, find_connected_components};

#[cfg(test)]
mod tests;
