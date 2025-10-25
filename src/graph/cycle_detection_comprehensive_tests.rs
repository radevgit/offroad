//! Comprehensive code coverage tests for cycle detection algorithm
//! Tests all combinations of arc types, vertex degrees, and edge cases

#[cfg(test)]
mod comprehensive_cycle_tests {
    use crate::graph::{merge_close_endpoints, find_non_intersecting_cycles};
    use togo::prelude::*;

    // ============================================================================
    // LINE SEGMENT ONLY TESTS
    // ============================================================================

    #[test]
    fn test_line_simple_triangle() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(1.0, 0.0), Point::new(0.5, 1.0)),
            arcseg(Point::new(0.5, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 triangle cycle");
        assert_eq!(cycles[0].len(), 3, "Triangle should have 3 arcs");
    }

    #[test]
    fn test_line_square() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(1.0, 0.0), Point::new(1.0, 1.0)),
            arcseg(Point::new(1.0, 1.0), Point::new(0.0, 1.0)),
            arcseg(Point::new(0.0, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 square cycle");
        assert_eq!(cycles[0].len(), 4, "Square should have 4 arcs");
    }

    #[test]
    fn test_line_pentagon() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(1.0, 0.0), Point::new(1.5, 0.8)),
            arcseg(Point::new(1.5, 0.8), Point::new(0.8, 1.5)),
            arcseg(Point::new(0.8, 1.5), Point::new(0.0, 1.0)),
            arcseg(Point::new(0.0, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 pentagon cycle");
        assert_eq!(cycles[0].len(), 5, "Pentagon should have 5 arcs");
    }

    #[test]
    fn test_line_two_separate_triangles() {
        let mut arcs = vec![
            // Triangle 1
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(1.0, 0.0), Point::new(0.5, 1.0)),
            arcseg(Point::new(0.5, 1.0), Point::new(0.0, 0.0)),
            
            // Triangle 2
            arcseg(Point::new(3.0, 0.0), Point::new(4.0, 0.0)),
            arcseg(Point::new(4.0, 0.0), Point::new(3.5, 1.0)),
            arcseg(Point::new(3.5, 1.0), Point::new(3.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 2, "Should find 2 separate cycles");
        assert_eq!(cycles[0].len(), 3, "Each cycle should have 3 arcs");
        assert_eq!(cycles[1].len(), 3, "Each cycle should have 3 arcs");
    }

    // ============================================================================
    // ARC ONLY TESTS
    // ============================================================================

    #[test]
    fn test_arc_full_circle() {
        let center = Point::new(0.0, 0.0);
        let radius = 1.0;
        
        let mut arcs = vec![
            arc(Point::new(1.0, 0.0), Point::new(0.0, 1.0), center, radius),
            arc(Point::new(0.0, 1.0 + 1e-9), Point::new(-1.0, 0.0), center, radius),
            arc(Point::new(-1.0 - 1e-9, 0.0), Point::new(0.0, -1.0), center, radius),
            arc(Point::new(0.0, -1.0 + 1e-9), Point::new(1.0, 0.0), center, radius),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 circle cycle");
        assert_eq!(cycles[0].len(), 4, "Circle should have 4 quarter-arcs");
        
        // Verify all arcs are curved (not lines)
        for arc in &cycles[0] {
            assert!(!arc.r.is_infinite(), "Should be curved arc, not line");
        }
    }

    #[test]
    fn test_arc_two_circles() {
        let center_left = Point::new(-1.0, 0.0);
        let center_right = Point::new(1.0, 0.0);
        let radius = 1.0;
        
        let mut arcs = vec![
            // Left circle
            arc(Point::new(-2.0, 0.0), Point::new(-1.0, 1.0), center_left, radius),
            arc(Point::new(-1.0, 1.0), Point::new(0.0, 0.0), center_left, radius),
            arc(Point::new(0.0, 0.0 + 1e-9), Point::new(-1.0, -1.0), center_left, radius),
            arc(Point::new(-1.0, -1.0), Point::new(-2.0, 0.0), center_left, radius),
            
            // Right circle
            arc(Point::new(0.0, 0.0 - 1e-9), Point::new(1.0, 1.0), center_right, radius),
            arc(Point::new(1.0, 1.0), Point::new(2.0, 0.0), center_right, radius),
            arc(Point::new(2.0, 0.0), Point::new(1.0, -1.0), center_right, radius),
            arc(Point::new(1.0, -1.0), Point::new(0.0, 0.0), center_right, radius),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // Two circles touching at a point: merging endpoints within tolerance 1e-8 connects them
        // into a figure-8 where the touching point connects the two loops
        // The algorithm creates 1 figure-8 cycle as the non-intersecting path
        assert_eq!(cycles.len(), 1, "Two touching circles form a figure-8 (1 cycle)");
        let cycle = &cycles[0];
        assert_eq!(cycle.len(), 8, "Figure-8 should have 8 arcs (4 per loop)");
    }

    #[test]
    fn test_arc_semicircle() {
        let center = Point::new(0.0, 0.0);
        let radius = 2.0;
        
        let mut arcs = vec![
            // Semicircle (top half)
            arc(Point::new(2.0, 0.0), Point::new(0.0, 2.0), center, radius),
            arc(Point::new(0.0, 2.0 + 1e-9), Point::new(-2.0, 0.0), center, radius),
            // Line segment for bottom
            arcseg(Point::new(-2.0, 0.0), Point::new(2.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 semicircle cycle");
        assert_eq!(cycles[0].len(), 3, "Semicircle should have 3 segments");
    }

    #[test]
    fn test_arc_small_radius() {
        let center = Point::new(0.0, 0.0);
        let radius = 0.1;
        
        let mut arcs = vec![
            arc(Point::new(0.1, 0.0), Point::new(0.0, 0.1), center, radius),
            arc(Point::new(0.0, 0.1 + 1e-9), Point::new(-0.1, 0.0), center, radius),
            arc(Point::new(-0.1 - 1e-9, 0.0), Point::new(0.0, -0.1), center, radius),
            arc(Point::new(0.0, -0.1 + 1e-9), Point::new(0.1, 0.0), center, radius),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find small circle");
        assert_eq!(cycles[0].len(), 4, "Should have 4 quarter-arcs");
    }

    #[test]
    fn test_arc_large_radius() {
        let center = Point::new(0.0, 0.0);
        let radius = 100.0;
        
        let mut arcs = vec![
            arc(Point::new(100.0, 0.0), Point::new(0.0, 100.0), center, radius),
            arc(Point::new(0.0, 100.0 + 1e-9), Point::new(-100.0, 0.0), center, radius),
            arc(Point::new(-100.0 - 1e-9, 0.0), Point::new(0.0, -100.0), center, radius),
            arc(Point::new(0.0, -100.0 + 1e-9), Point::new(100.0, 0.0), center, radius),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find large circle");
        assert_eq!(cycles[0].len(), 4, "Should have 4 quarter-arcs");
    }

    // ============================================================================
    // MIXED LINE AND ARC TESTS
    // ============================================================================

    #[test]
    fn test_mixed_square_with_one_arc() {
        let center = Point::new(1.0, 0.0);
        let radius = 1.0;
        
        let mut arcs = vec![
            // Bottom line
            arcseg(Point::new(0.0, 0.0), Point::new(2.0, 0.0)),
            // Right arc (instead of line)
            arc(Point::new(2.0, 0.0), Point::new(2.0, 1.0), center, radius),
            // Top line
            arcseg(Point::new(2.0, 1.0), Point::new(0.0, 1.0)),
            // Left line
            arcseg(Point::new(0.0, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 mixed cycle");
        assert_eq!(cycles[0].len(), 4, "Should have 4 segments");
        
        // Check we have both lines and arcs
        let has_line = cycles[0].iter().any(|a| a.r.is_infinite());
        let has_arc = cycles[0].iter().any(|a| !a.r.is_infinite());
        assert!(has_line, "Should contain line segments");
        assert!(has_arc, "Should contain curved arcs");
    }

    #[test]
    fn test_mixed_square_alternating_arcs_and_lines() {
        let c1 = Point::new(1.0, 0.0);
        let c2 = Point::new(1.0, 1.0);
        let r = 1.0;
        
        let mut arcs = vec![
            // Bottom arc
            arc(Point::new(0.0, 0.0), Point::new(2.0, 0.0), c1, r),
            // Right line
            arcseg(Point::new(2.0, 0.0), Point::new(2.0, 1.0)),
            // Top arc
            arc(Point::new(2.0, 1.0), Point::new(0.0, 1.0), c2, r),
            // Left line
            arcseg(Point::new(0.0, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 cycle");
        assert_eq!(cycles[0].len(), 4, "Should have 4 segments");
    }

    #[test]
    fn test_mixed_complex_shape() {
        let mut arcs = vec![
            // Start with line
            arcseg(Point::new(0.0, 0.0), Point::new(3.0, 0.0)),
            // Arc up
            arc(Point::new(3.0, 0.0), Point::new(6.0, 3.0), Point::new(6.0, 0.0), 3.0),
            // Line right
            arcseg(Point::new(6.0 + 1e-9, 3.0), Point::new(6.0, 6.0)),
            // Arc down
            arc(Point::new(6.0, 6.0), Point::new(3.0, 9.0), Point::new(6.0, 9.0), 3.0),
            // Line down
            arcseg(Point::new(3.0, 9.0), Point::new(0.0, 9.0)),
            // Line left
            arcseg(Point::new(-1e-9, 9.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 cycle");
        assert_eq!(cycles[0].len(), 6, "Should have 6 segments");
        
        let has_line = cycles[0].iter().any(|a| a.r.is_infinite());
        let has_arc = cycles[0].iter().any(|a| !a.r.is_infinite());
        assert!(has_line, "Should contain line segments");
        assert!(has_arc, "Should contain arcs");
    }

    // ============================================================================
    // VERTEX DEGREE TESTS (how many edges at a vertex)
    // ============================================================================

    #[test]
    fn test_vertex_degree_2_simple_path() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(1.0, 0.0), Point::new(1.0, 1.0)),
            arcseg(Point::new(1.0, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // All vertices should have degree 2 in this triangle
        assert_eq!(cycles.len(), 1, "Should find 1 cycle");
    }

    #[test]
    fn test_vertex_degree_3_t_intersection() {
        let mut arcs = vec![
            // Vertical line
            arcseg(Point::new(1.0, 0.0), Point::new(1.0, 2.0)),
            // Horizontal top line
            arcseg(Point::new(0.0, 2.0), Point::new(1.0, 2.0)),
            // Horizontal bottom line
            arcseg(Point::new(1.0, 0.0), Point::new(2.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // This forms incomplete paths, not a cycle
        assert_eq!(cycles.len(), 0, "Should find no complete cycles");
    }

    #[test]
    fn test_vertex_degree_4_cross_intersection() {
        let mut arcs = vec![
            // Outer box
            arcseg(Point::new(0.0, 0.0), Point::new(2.0, 0.0)),
            arcseg(Point::new(2.0, 0.0), Point::new(2.0, 2.0)),
            arcseg(Point::new(2.0, 2.0), Point::new(0.0, 2.0)),
            arcseg(Point::new(0.0, 2.0), Point::new(0.0, 0.0)),
            
            // Internal cross - horizontal
            arcseg(Point::new(0.0, 1.0), Point::new(2.0, 1.0)),
            // Internal cross - vertical
            arcseg(Point::new(1.0, 0.0), Point::new(1.0, 2.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // Center point has degree 4
        assert!(cycles.len() > 0, "Should find cycles");
    }

    // ============================================================================
    // ENDPOINT MERGING TESTS
    // ============================================================================

    #[test]
    fn test_merge_tiny_gaps_1e_9() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(1.0 + 1e-9, 1e-9), Point::new(1.0, 1.0)),
            arcseg(Point::new(1.0, 1.0), Point::new(0.0, 1.0)),
            arcseg(Point::new(-1e-9, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should merge tiny gaps and find cycle");
        assert_eq!(cycles[0].len(), 4, "Should form complete square");
    }

    #[test]
    fn test_merge_small_gaps_1e_7() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(1.0 + 1e-7, 1e-7), Point::new(1.0, 1.0)),
            arcseg(Point::new(1.0, 1.0), Point::new(0.0, 1.0)),
            arcseg(Point::new(-1e-7, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-6);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should merge small gaps");
    }

    #[test]
    fn test_no_merge_large_gaps() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(1.0 + 0.1, 0.1), Point::new(1.0, 1.0)),  // Large gap
            arcseg(Point::new(1.0, 1.0), Point::new(0.0, 1.0)),
            arcseg(Point::new(-0.1, 1.0), Point::new(0.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 0, "Should not merge large gaps");
    }

    // ============================================================================
    // ARC DIRECTION TESTS (rightmost-turn rule)
    // ============================================================================

    #[test]
    fn test_arc_direction_at_vertex_with_tangents() {
        // Create a shape where arc direction matters
        let center = Point::new(0.0, 0.0);
        let radius = 1.0;
        
        let mut arcs = vec![
            // Bottom to right quarter
            arc(Point::new(0.0, -1.0), Point::new(1.0, 0.0), center, radius),
            // Right to top quarter
            arc(Point::new(1.0, 0.0), Point::new(0.0, 1.0), center, radius),
            // Top to left quarter
            arc(Point::new(0.0, 1.0), Point::new(-1.0, 0.0), center, radius),
            // Left to bottom quarter
            arc(Point::new(-1.0, 0.0), Point::new(0.0, -1.0), center, radius),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should respect arc directions");
        assert_eq!(cycles[0].len(), 4, "Should form complete circle");
    }

    #[test]
    fn test_rightmost_turn_rule_at_vertex_degree_3() {
        // Create a T-junction where rightmost-turn matters
        let mut arcs = vec![
            // Vertical line up
            arcseg(Point::new(1.0, 0.0), Point::new(1.0, 1.0)),
            // Turn right
            arcseg(Point::new(1.0, 1.0), Point::new(2.0, 1.0)),
            // Turn down and back
            arcseg(Point::new(2.0, 1.0), Point::new(1.0, 1.0 + 1e-9)),
            // Continue down
            arcseg(Point::new(1.0, 1.0 - 1e-9), Point::new(1.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // The algorithm should use rightmost-turn rule to form cycles
        assert!(!cycles.is_empty(), "Cycles found using turn rule");
    }

    // ============================================================================
    // SPECIAL CASES AND EDGE CONDITIONS
    // ============================================================================

    #[test]
    fn test_empty_input() {
        let mut arcs = vec![];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 0, "Empty input should produce no cycles");
    }

    #[test]
    fn test_single_arc() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 0, "Single arc cannot form cycle");
    }

    #[test]
    fn test_two_arcs_not_connected() {
        let mut arcs = vec![
            arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            arcseg(Point::new(2.0, 0.0), Point::new(3.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 0, "Disconnected arcs cannot form cycle");
    }

    #[test]
    fn test_degenerate_zero_radius_arc() {
        // Arc with very small radius (degenerate case)
        let center = Point::new(0.0, 0.0);
        let radius = 1e-10;  // Very small radius
        
        let mut arcs = vec![
            arc(Point::new(1e-10, 0.0), Point::new(0.0, 1e-10), center, radius),
            arc(Point::new(0.0, 1e-10 + 1e-9), Point::new(-1e-10, 0.0), center, radius),
            arc(Point::new(-1e-10 - 1e-9, 0.0), Point::new(0.0, -1e-10), center, radius),
            arc(Point::new(0.0, -1e-10 + 1e-9), Point::new(1e-10, 0.0), center, radius),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        // Degenerate arcs with endpoints within merge tolerance collapse to a single point.
        // This creates tiny arcs that merge together completely, resulting in 0 cycles.
        // This is correct behavior: arcs with degenerate radii and merged endpoints don't form valid cycles.
        assert_eq!(cycles.len(), 0, "Degenerate arcs with merged endpoints produce no cycles");
    }

    #[test]
    fn test_negative_coordinate_arcs() {
        let center = Point::new(-5.0, -5.0);
        let radius = 1.0;
        
        let mut arcs = vec![
            arc(Point::new(-4.0, -5.0), Point::new(-5.0, -4.0), center, radius),
            arc(Point::new(-5.0, -4.0 + 1e-9), Point::new(-6.0, -5.0), center, radius),
            arc(Point::new(-6.0 - 1e-9, -5.0), Point::new(-5.0, -6.0), center, radius),
            arc(Point::new(-5.0, -6.0 + 1e-9), Point::new(-4.0, -5.0), center, radius),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should handle negative coordinates");
    }

    #[test]
    fn test_mixed_coordinate_quadrants() {
        let center = Point::new(0.0, 0.0);
        let radius = 1.0;
        
        let mut arcs = vec![
            // Q1
            arc(Point::new(1.0, 0.0), Point::new(0.0, 1.0), center, radius),
            // Q2
            arc(Point::new(0.0, 1.0 + 1e-9), Point::new(-1.0, 0.0), center, radius),
            // Q3
            arc(Point::new(-1.0 - 1e-9, 0.0), Point::new(0.0, -1.0), center, radius),
            // Q4
            arc(Point::new(0.0, -1.0 + 1e-9), Point::new(1.0, 0.0), center, radius),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Circle spanning all quadrants");
    }

    // ============================================================================
    // COMPLEX TOPOLOGY TESTS
    // ============================================================================

    #[test]
    fn test_three_circles_cascade() {
        let c1 = Point::new(0.0, 0.0);
        let c2 = Point::new(3.0, 0.0);
        let c3 = Point::new(1.5, 2.5);
        let r = 1.0;
        
        let mut arcs = vec![
            // Circle 1
            arc(Point::new(1.0, 0.0), Point::new(0.0, 1.0), c1, r),
            arc(Point::new(0.0, 1.0 + 1e-9), Point::new(-1.0, 0.0), c1, r),
            arc(Point::new(-1.0 - 1e-9, 0.0), Point::new(0.0, -1.0), c1, r),
            arc(Point::new(0.0, -1.0 + 1e-9), Point::new(1.0, 0.0), c1, r),
            
            // Circle 2
            arc(Point::new(4.0, 0.0), Point::new(3.0, 1.0), c2, r),
            arc(Point::new(3.0, 1.0 + 1e-9), Point::new(2.0, 0.0), c2, r),
            arc(Point::new(2.0 - 1e-9, 0.0), Point::new(3.0, -1.0), c2, r),
            arc(Point::new(3.0, -1.0 + 1e-9), Point::new(4.0, 0.0), c2, r),
            
            // Circle 3
            arc(Point::new(2.5, 2.5), Point::new(1.5, 3.5), c3, r),
            arc(Point::new(1.5, 3.5 + 1e-9), Point::new(0.5, 2.5), c3, r),
            arc(Point::new(0.5 - 1e-9, 2.5), Point::new(1.5, 1.5), c3, r),
            arc(Point::new(1.5, 1.5 + 1e-9), Point::new(2.5, 2.5), c3, r),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 3, "Should find 3 separate circles");
    }

    #[test]
    fn test_spiral_like_pattern() {
        // Create concentric circles
        let center = Point::new(0.0, 0.0);
        
        let mut arcs = vec![];
        
        // Inner circle radius 1.0
        arcs.push(arc(Point::new(1.0, 0.0), Point::new(0.0, 1.0), center, 1.0));
        arcs.push(arc(Point::new(0.0, 1.0 + 1e-9), Point::new(-1.0, 0.0), center, 1.0));
        arcs.push(arc(Point::new(-1.0 - 1e-9, 0.0), Point::new(0.0, -1.0), center, 1.0));
        arcs.push(arc(Point::new(0.0, -1.0 + 1e-9), Point::new(1.0, 0.0), center, 1.0));
        
        // Outer circle radius 2.0
        arcs.push(arc(Point::new(2.0, 0.0), Point::new(0.0, 2.0), center, 2.0));
        arcs.push(arc(Point::new(0.0, 2.0 + 1e-9), Point::new(-2.0, 0.0), center, 2.0));
        arcs.push(arc(Point::new(-2.0 - 1e-9, 0.0), Point::new(0.0, -2.0), center, 2.0));
        arcs.push(arc(Point::new(0.0, -2.0 + 1e-9), Point::new(2.0, 0.0), center, 2.0));
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 2, "Should find 2 concentric circles");
    }

    #[test]
    fn test_many_segment_polygon() {
        // Create a 16-sided polygon
        let mut arcs = vec![];
        let n = 16;
        
        for i in 0..n {
            let angle1 = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            let angle2 = 2.0 * std::f64::consts::PI * ((i + 1) as f64) / (n as f64);
            
            let x1 = angle1.cos();
            let y1 = angle1.sin();
            let x2 = angle2.cos();
            let y2 = angle2.sin();
            
            arcs.push(arcseg(Point::new(x1, y1), Point::new(x2, y2)));
        }
        
        merge_close_endpoints(&mut arcs, 1e-8);
        let cycles = find_non_intersecting_cycles(&arcs);
        
        assert_eq!(cycles.len(), 1, "Should find 1 polygon");
        assert_eq!(cycles[0].len(), 16, "Should have 16 segments");
    }
}
