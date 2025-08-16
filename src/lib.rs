#![doc(html_no_source)]

//! 2D offsetting for arc polylines/polygons.
//!
//! # Examples
//!
//! ```rust
//! use geom::prelude::*;
//! use offroad::prelude::{OffsetCfg, offset_polyline};
//!
//! // Create a simple L-shaped polyline
//! let poly = vec![
//!     pvertex(point(0.0, 0.0), 0.0),    // Start point
//!     pvertex(point(10.0, 0.0), 0.0),   // Horizontal line
//!     pvertex(point(10.0, 10.0), 0.0),  // Vertical line
//! ];
//!
//! // Create default configuration
//! let mut cfg = OffsetCfg::default();
//!
//! // Offset by 2.0 units to the right
//! let offset_polylines = offset_polyline(&poly, 2.0, &mut cfg);
//!
//! // The function returns a vector of polylines
//! //assert!(offset_polylines.len() == 1);
//! //assert!(offset_polylines[0].len() == 6);
//! ```
//!
//! Check "examples" directory for more usage examples.

// Offsetting algorithm components
pub mod offset;
#[doc(hidden)]
// connect raw offsets with arcs
mod offset_connect_raw;
#[doc(hidden)]
// prune invalid offsets that are close to original polylines
mod offset_prune_invalid;
#[doc(hidden)]
pub mod offset_raw;
#[doc(hidden)]
// resulting soup of arcs is ordered and reconnected
mod offset_reconnect_arcs;
#[doc(hidden)]
// raw offsetting components (lines, arcs)
mod offset_segments_raws;
#[doc(hidden)]
// split raw offsets into segments in intersection points
mod offset_split_arcs;

// Re-export main offsetting functions
// For public API
pub mod prelude {
    pub use crate::offset::{OffsetCfg, offset_arcline, offset_polyline};
    pub use crate::offset::{
        example_polyline_01, example_polyline_02, example_polyline_03, example_polylines_04,
    };
}
// For internal use
// pub use crate::offset_polyline_raw::{offset_polyline_raw, poly_to_raws};
// pub use crate::offset_connect_raw::offset_connect_raw;
// pub use crate::offset_split_arcs::offset_split_arcs;
// pub use crate::offset_prune_invalid::offset_prune_invalid;
// pub use crate::offset_reconnect_arcs::{offset_reconnect_arcs, find_connected_components, find_middle_points, remove_bridge_arcs};

#[cfg(test)]
mod tests;
