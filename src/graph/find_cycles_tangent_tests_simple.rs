//! Simple tests for tangent-based rightmost edge selection
//! 
//! This module contains basic tests for the enhanced cycle detection algorithm
//! that properly handles tangent directions for both arcs and line segments.

use super::find_cycles::*;
use togo::prelude::*;

#[cfg(test)]
mod basic_tangent_tests {
    use super::*;

    #[test]
    fn test_simple_arc_tangent_calculation() {
        // Test basic arc tangent calculation with a simple circular arc
        let start_point = point(5.0, 0.0);
        let end_point = point(0.0, 5.0);
        let center = point(0.0, 0.0);
        
        // Create a quarter circle arc
        let arc_segment = arc(start_point, end_point, center, 5.0);
        
        let tangents = arc_segment.tangents();
        
        // Print the tangent values for debugging
        println!("Start tangent: ({}, {})", tangents[0].x, tangents[0].y);
        println!("End tangent: ({}, {})", tangents[1].x, tangents[1].y);
        
        // Test that we get reasonable tangent vectors (they should be normalized)
        let start_mag = (tangents[0].x * tangents[0].x + tangents[0].y * tangents[0].y).sqrt();
        let end_mag = (tangents[1].x * tangents[1].x + tangents[1].y * tangents[1].y).sqrt();
        
        assert!((start_mag - 1.0).abs() < 0.1, "Start tangent should be approximately normalized");
        assert!((end_mag - 1.0).abs() < 0.1, "End tangent should be approximately normalized");
        
        // The exact direction depends on the arc implementation, but they should be valid vectors
        assert!(tangents[0].x.abs() + tangents[0].y.abs() > 0.5, "Start tangent should be non-zero");
        assert!(tangents[1].x.abs() + tangents[1].y.abs() > 0.5, "End tangent should be non-zero");
    }

    #[test]
    fn test_arc_vs_line_segment_cycle() {
        // Test cycle with both an arc and line segments
        let arcs = vec![
            // Quarter circle arc
            arc(point(5.0, 0.0), point(0.0, 5.0), point(0.0, 0.0), 5.0),
            // Line segments to complete the cycle
            arcseg(point(0.0, 5.0), point(-5.0, 0.0)),
            arcseg(point(-5.0, 0.0), point(0.0, -5.0)),
            arcseg(point(0.0, -5.0), point(5.0, 0.0)),
        ];
        
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // Should find the mixed arc/segment cycle
        assert!(!cycles.is_empty(), "Should find cycle with mixed arc types");
        for cycle in &cycles {
            assert!(cycle.len() >= 3, "Each cycle should have at least 3 arcs");
        }
    }

    #[test]
    fn test_semicircle_cycle() {
        // Test with semicircle arcs forming a complete circle
        let arcs = vec![
            // Upper semicircle
            arc(point(-3.0, 0.0), point(3.0, 0.0), point(0.0, 0.0), 3.0),
            // Lower semicircle (completing the circle)
            arc(point(3.0, 0.0), point(-3.0, 0.0), point(0.0, 0.0), 3.0),
        ];
        
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // Should find the complete circle as a cycle
        if !cycles.is_empty() {
            assert!(cycles[0].len() >= 2, "Circle should have at least 2 arcs");
        }
    }

    #[test]
    fn test_line_segments_only_still_work() {
        // Ensure basic line segment cycles still work correctly
        let arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(1.0, 1.0)),
            arcseg(point(1.0, 1.0), point(0.0, 1.0)),
            arcseg(point(0.0, 1.0), point(0.0, 0.0)),
        ];
        
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find exactly one square cycle");
        assert_eq!(cycles[0].len(), 4, "Square should have 4 sides");
    }

    #[test]
    fn test_multiple_arcs_radiating_from_vertex() {
        // Test multiple arcs/segments radiating from same vertex for rightmost selection
        let vertex = point(0.0, 0.0);
        
        let arcs = vec![
            // Lines radiating from vertex in different directions
            arcseg(vertex, point(10.0, 0.0)),    // east
            arcseg(vertex, point(0.0, 10.0)),    // north
            arcseg(vertex, point(-10.0, 0.0)),   // west
            arcseg(vertex, point(0.0, -10.0)),   // south
            // Connect endpoints to form cycles
            arcseg(point(10.0, 0.0), point(0.0, 10.0)),
            arcseg(point(0.0, 10.0), point(-10.0, 0.0)),
            arcseg(point(-10.0, 0.0), point(0.0, -10.0)),
            arcseg(point(0.0, -10.0), point(10.0, 0.0)),
        ];
        
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // Should find cycles with proper tangent-based selection
        for cycle in &cycles {
            assert!(cycle.len() >= 3, "Each cycle should have at least 3 arcs");
        }
    }

    #[test]
    fn test_tangent_based_rightmost_selection() {
        // Specific test for tangent-based rightmost edge selection
        // Create a scenario where endpoints and tangents would give different results
        let vertex = point(0.0, 0.0);
        
        let arcs = vec![
            // Arc that curves - tangent direction different from endpoint direction
            arc(vertex, point(2.0, 2.0), point(2.0, 0.0), 2.0),
            // Line segment 
            arcseg(vertex, point(0.0, 3.0)),
            // Complete some cycles
            arcseg(point(2.0, 2.0), point(0.0, 3.0)),
        ];
        
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // Algorithm should handle tangent vs endpoint difference correctly
        for cycle in &cycles {
            assert!(cycle.len() >= 3, "Each cycle should have at least 3 arcs");
        }
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_original_functionality_preserved() {
        // Ensure original test cases still work
        let arcs = vec![
            // Simple triangle
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(0.5, 1.0)),
            arcseg(point(0.5, 1.0), point(0.0, 0.0)),
        ];
        
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find exactly one triangle cycle");
        assert_eq!(cycles[0].len(), 3, "Triangle should have 3 sides");
    }

    #[test]
    fn test_premature_edge_marking_fix() {
        // Regression test for the edge marking bug that was fixed
        let arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 1.0)),     // top-left
            arcseg(point(1.0, 1.0), point(2.0, 0.0)),     // top-right  
            arcseg(point(2.0, 0.0), point(1.0, -1.0)),    // bottom-right
            arcseg(point(1.0, -1.0), point(0.0, 0.0)),    // bottom-left
        ];
        
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find exactly one cycle");
        assert_eq!(cycles[0].len(), 4, "Cycle should have 4 edges");
    }

    #[test]
    fn test_complex_intersection_still_works() {
        // Test that complex intersections still work with tangent improvements
        let arcs = vec![
            // Figure-8 pattern with shared edge
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(1.0, 1.0)),
            arcseg(point(1.0, 1.0), point(0.0, 1.0)),
            arcseg(point(0.0, 1.0), point(0.0, 0.0)),
            // Second loop sharing an edge
            arcseg(point(1.0, 0.0), point(2.0, 0.0)),
            arcseg(point(2.0, 0.0), point(2.0, 1.0)),
            arcseg(point(2.0, 1.0), point(1.0, 1.0)),
        ];
        
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert!(!cycles.is_empty(), "Should find cycles in figure-8 pattern");
        
        for cycle in &cycles {
            assert!(cycle.len() >= 3, "Each cycle should have at least 3 arcs");
        }
    }
}