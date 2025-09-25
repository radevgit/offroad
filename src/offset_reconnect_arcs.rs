#![allow(dead_code)]

use togo::prelude::*;
use crate::graph::{
    merge_ends::merge_close_endpoints_default,
    find_cycles::find_non_intersecting_cycles
};

const EPS_CONNECT: f64 = 1e-7;

/// Reconnects offset segments by merging close endpoints and finding non-intersecting cycles.
/// 
/// This function takes a collection of offset arcs and processes them to:
/// 1. Merge close endpoints that should be connected due to numerical precision issues
/// 2. Find cycles in the resulting graph that don't geometrically intersect
/// 3. Return separate Arclines for each non-intersecting cycle
/// 
/// # Arguments
/// * `arcs` - Input arcline containing offset arcs that may have disconnected endpoints
/// 
/// # Returns
/// Vector of Arclines, each representing a separate non-intersecting cycle
pub fn offset_reconnect_arcs(arcs: Arcline) -> Vec<Arcline> {
    // Use the input arcs directly, no need to clone since we take ownership
    let mut arc_vec: Vec<Arc> = arcs;
    
    // Step 1: Merge close endpoints to connect arcs that should be connected
    // This fixes numerical precision issues from the offset algorithm
    merge_close_endpoints_default(&mut arc_vec);
    
    // Step 2: Find non-intersecting cycles using our tangent-based algorithm
    // This separates the arcs into geometrically non-intersecting components
    let cycles = find_non_intersecting_cycles(&arc_vec);
    
    // Step 3: Each cycle is already a Vec<Arc> which is an Arcline
    let mut result = Vec::new();
    for cycle_arcs in cycles {
        if !cycle_arcs.is_empty() {
            result.push(cycle_arcs);
        }
    }
    
    result
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_offset_reconnect_integration() {
        // Create a simple arcline with disconnected endpoints that should be merged
        let arcs = vec![
            // Create a nearly-closed triangle with small gaps (within merge tolerance)
            arcseg(Point::new(0.0, 0.0), Point::new(10.0, 0.0)),
            arcseg(Point::new(10.0, 1e-9), Point::new(5.0, 8.66)), // tiny gap < 1e-8
            arcseg(Point::new(5.0, 8.66), Point::new(-1e-9, 0.0)), // tiny gap < 1e-8
        ];

        // Run the reconnection algorithm
        let result = offset_reconnect_arcs(arcs);

        // Should find 1 cycle (the merged triangle)
        assert_eq!(result.len(), 1, "Should find exactly one cycle");
        
        // The first result should have 3 arcs (the connected triangle)
        let first_cycle = result.first().unwrap();
        assert_eq!(first_cycle.len(), 3, "Triangle should have 3 segments");

        println!("Integration test passed: found {} cycles", result.len());
    }

    #[test]
    fn test_offset_reconnect_with_arcs() {
        // Create arcline with simple line segments for this test
        let arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(5.0, 0.0)),
            arcseg(Point::new(5.0, 0.0), Point::new(5.0, 5.0)),
            arcseg(Point::new(5.0, 5.0), Point::new(0.0, 5.0)),
            arcseg(Point::new(0.0, 5.0), Point::new(0.0, 0.0)),
        ];

        let result = offset_reconnect_arcs(arcs);

        // Should find exactly 1 cycle (the square)
        assert_eq!(result.len(), 1, "Should find exactly one cycle");
        
        // The square should have 4 arcs
        let first_cycle = result.first().unwrap();
        assert_eq!(first_cycle.len(), 4, "Square should have 4 segments");
        
        println!("Mixed geometry test passed: found {} cycles", result.len());
    }

    #[test]
    fn test_offset_reconnect_with_actual_curved_arcs() {
        // Create test with actual curved arcs - this tests our tangent-based algorithm properly
        let center = Point::new(0.0, 0.0);
        let radius = 5.0;
        
        // Create a circle made of 4 quarter-circle arcs with small gaps (within merge tolerance)
        let arcs = vec![
            // Quarter arc from (5,0) to (0,5) - center at (0,0), radius 5
            arc(Point::new(5.0, 0.0), Point::new(0.0, 5.0), center, radius),
            // Gap + Quarter arc from (0,5.000000001) to (-5,0) 
            arc(Point::new(0.0, 5.0 + 1e-9), Point::new(-5.0, 0.0), center, radius),
            // Gap + Quarter arc from (-5.000000001,0) to (0,-5)
            arc(Point::new(-5.0 - 1e-9, 0.0), Point::new(0.0, -5.0), center, radius),
            // Gap + Quarter arc from (0,-4.999999999) to (5,0) - close the circle with small gap
            arc(Point::new(0.0, -5.0 + 1e-9), Point::new(5.0, 0.0), center, radius),
        ];

        let result = offset_reconnect_arcs(arcs);

        // Should find exactly 1 cycle (the circle)
        assert_eq!(result.len(), 1, "Should find exactly one cycle");
        
        let first_cycle = result.first().unwrap();
        assert_eq!(first_cycle.len(), 4, "Circle should have 4 quarter-arc segments");
        
        // Verify we actually have curved arcs, not line segments
        let has_curved_arcs = first_cycle.iter().any(|arc| arc.r != f64::INFINITY);
        assert!(has_curved_arcs, "Should contain actual curved arcs, not just line segments");

        println!("Curved arcs test passed: found {} cycles with actual curved geometry", result.len());
    }

    #[test]
    fn test_offset_reconnect_mixed_arcs_and_segments() {
        // Test with both curved arcs AND line segments - this is realistic for offset results
        let arcs = vec![
            // Start with a line segment
            arcseg(Point::new(0.0, 0.0), Point::new(3.0, 0.0)),
            // Add a curved arc - quarter circle from (3,0) to (6,3) with center at (6,0)
            arc(Point::new(3.0, 0.0), Point::new(6.0, 3.0), Point::new(6.0, 0.0), 3.0),
            // Another line segment with small gap (within merge tolerance)
            arcseg(Point::new(6.0 + 1e-9, 3.0), Point::new(6.0, 6.0)),
            // Another curved arc - quarter circle from (6,6) to (3,9) with center at (6,9)
            arc(Point::new(6.0, 6.0), Point::new(3.0, 9.0), Point::new(6.0, 9.0), 3.0),
            // Close with line segments
            arcseg(Point::new(3.0, 9.0), Point::new(0.0, 9.0)),
            arcseg(Point::new(-1e-9, 9.0), Point::new(0.0, 0.0)), // small gap to test merging
        ];

        let result = offset_reconnect_arcs(arcs);

        // Should find exactly 1 cycle (the mixed shape)
        assert_eq!(result.len(), 1, "Should find exactly one cycle");
        
        let first_cycle = result.first().unwrap();
        assert_eq!(first_cycle.len(), 6, "Mixed shape should have 6 segments");
        
        // Verify we have both curved arcs and line segments
        let curved_count = first_cycle.iter().filter(|arc| arc.r != f64::INFINITY).count();
        let line_count = first_cycle.iter().filter(|arc| arc.r == f64::INFINITY).count();
        
        assert!(curved_count >= 2, "Should have at least 2 curved arcs");
        assert!(line_count >= 4, "Should have at least 4 line segments");

        println!("Mixed arcs/segments test passed: found {} cycles", result.len());
    }
}


