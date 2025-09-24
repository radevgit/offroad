//! Graph algorithms for finding connected components and cycles in arc graphs.
//!
//! This module provides algorithms for:
//! - Merging close endpoints of arcs to handle numerical precision issues
//! - Finding non-intersecting cycles in graphs of connected arcs
//! - Handling geometric constraints for tool path generation

pub mod merge_ends;
pub mod find_cycles;

#[cfg(test)]
mod find_cycles_tangent_tests_simple;

// Re-export main functions
pub use merge_ends::merge_close_endpoints;
pub use find_cycles::find_non_intersecting_cycles;