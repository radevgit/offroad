//! Implementation of the merge_ends functionality for connecting close arc endpoints.
//! This module handles merging endpoints that are within tolerance and eliminating small arcs.

use togo::prelude::*;

/// Tolerance for merging close endpoints (separate from togo's EPS_COLLAPSED)
pub const MERGE_TOLERANCE: f64 = 1e-8;

/// Represents a group of close endpoints that should be merged
#[derive(Debug, Clone)]
struct EndpointGroup {
    /// All points in this group
    points: Vec<Point>,
    /// Indices of arcs that have endpoints in this group
    arc_indices: Vec<(usize, EndpointType)>,
    /// Centroid of the group (merge target)
    centroid: Point,
}

/// Indicates whether an endpoint is the start (a) or end (b) of an arc
#[derive(Debug, Clone, Copy, PartialEq)]
enum EndpointType {
    Start, // Point 'a' of the arc
    End,   // Point 'b' of the arc
}

/// Main function to merge close endpoints and eliminate small arcs
///
/// # Arguments
/// * `arcs` - Mutable vector of arcs to process
/// * `tolerance` - Distance threshold for considering points "close enough"
///
/// # Process
/// 1. Find groups of endpoints that are within tolerance distance
/// 2. Merge each group to its centroid
/// 3. Remove arcs that become too small after merging
/// 4. Adjust remaining arcs to ensure geometric consistency
pub fn merge_close_endpoints(arcs: &mut Vec<Arc>, tolerance: f64) {
    if arcs.is_empty() {
        return;
    }

    // Step 1: Find groups of close endpoints
    let groups = find_endpoint_groups(arcs, tolerance);
    
    // Step 2: Merge endpoints to group centroids
    merge_to_centroids(arcs, &groups);
    
    // Step 3: Eliminate small arcs
    eliminate_small_arcs(arcs, tolerance);
    
    // Step 4: Adjust arcs for consistency
    adjust_arcs_for_consistency(arcs);
}

/// Find groups of endpoints that are within tolerance distance of each other
fn find_endpoint_groups(arcs: &[Arc], tolerance: f64) -> Vec<EndpointGroup> {
    let mut all_endpoints = Vec::new();
    
    // Collect all endpoints with their arc indices
    for (arc_idx, arc) in arcs.iter().enumerate() {
        all_endpoints.push((arc.a, arc_idx, EndpointType::Start));
        all_endpoints.push((arc.b, arc_idx, EndpointType::End));
    }
    
    if all_endpoints.is_empty() {
        return Vec::new();
    }
    
    // Build spatial index of all endpoints
    let mut spatial_index = HilbertRTree::with_capacity(all_endpoints.len());
    for (point, _, _) in &all_endpoints {
        spatial_index.add_point(point.x, point.y);
    }
    spatial_index.build();
    
    let mut groups = Vec::new();
    let mut used = vec![false; all_endpoints.len()];
    
    // For each unused endpoint, start a new group
    for i in 0..all_endpoints.len() {
        if used[i] {
            continue;
        }
        
        let mut group = EndpointGroup {
            points: Vec::new(),
            arc_indices: Vec::new(),
            centroid: Point::new(0.0, 0.0),
        };
        
        // Add the starting point to the group
        let (point_i, arc_i, end_type_i) = all_endpoints[i];
        group.points.push(point_i);
        group.arc_indices.push((arc_i, end_type_i));
        used[i] = true;
        
        // Find all points within tolerance using spatial index (iterative expansion)
        let mut queue = vec![point_i];
        
        while !queue.is_empty() {
            let current_point = queue.pop().unwrap();
            
            // Query spatial index for points within tolerance of current_point
            let mut nearby_indices = Vec::new();
            spatial_index.query_circle(
                current_point.x,
                current_point.y,
                tolerance,
                &mut nearby_indices,
            );
            
            // Process each nearby point
            for idx in nearby_indices {
                if used[idx] {
                    continue;
                }
                
                // Get the actual point data from spatial index
                if let Some((x, y)) = spatial_index.get_point(idx) {
                    let point_j = Point::new(x, y);
                    let (_, arc_j, end_type_j) = all_endpoints[idx];
                    
                    // Verify actual distance (spatial index may be approximate)
                    if (point_j - current_point).norm() <= tolerance {
                        group.points.push(point_j);
                        group.arc_indices.push((arc_j, end_type_j));
                        used[idx] = true;
                        queue.push(point_j); // Add to queue for further expansion
                    }
                }
            }
        }
        
        // Calculate centroid
        group.centroid = calculate_centroid(&group.points);
        
        // Only add groups with more than one point (single points don't need merging)
        if group.points.len() > 1 {
            groups.push(group);
        }
    }
    
    groups
}

/// Calculate the centroid (average position) of a group of points
fn calculate_centroid(points: &[Point]) -> Point {
    if points.is_empty() {
        return Point::new(0.0, 0.0);
    }
    
    let sum = points.iter().fold(Point::new(0.0, 0.0), |acc, &p| acc + p);
    sum / points.len() as f64
}

/// Merge endpoints to their group centroids
fn merge_to_centroids(arcs: &mut [Arc], groups: &[EndpointGroup]) {
    for group in groups {
        for &(arc_idx, endpoint_type) in &group.arc_indices {
            match endpoint_type {
                EndpointType::Start => {
                    arcs[arc_idx].a = group.centroid;
                }
                EndpointType::End => {
                    arcs[arc_idx].b = group.centroid;
                }
            }
        }
    }
}

/// Convenience function that uses the default MERGE_TOLERANCE
pub fn merge_close_endpoints_default(arcs: &mut Vec<Arc>) {
    merge_close_endpoints(arcs, MERGE_TOLERANCE);
}

/// Remove arcs that are too small after merging
fn eliminate_small_arcs(arcs: &mut Vec<Arc>, tolerance: f64) {
    arcs.retain(|arc| !is_arc_too_small(arc, tolerance));
}

/// Check if an arc is too small and should be eliminated
fn is_arc_too_small(arc: &Arc, tolerance: f64) -> bool {
    let chord_length = (arc.b - arc.a).norm();
    
    if arc.r == f64::INFINITY {
        // Line segment: check only chord length
        chord_length <= tolerance
    } else {
        // Circular arc: check both chord length and radius
        let radius = arc.r.abs();
        chord_length <= tolerance && radius <= tolerance
    }
}

/// Adjust all arcs to ensure geometric consistency after merging
fn adjust_arcs_for_consistency(arcs: &mut [Arc]) {
    for arc in arcs.iter_mut() {
        arc.make_consistent();
    }
}

#[cfg(test)]
mod tests {
    use togo::prelude::*;
    use super::*;
    
    #[test]
    fn test_merge_close_endpoints_simple() {
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0 + 1e-9, 0.0 + 1e-9), point(2.0, 0.0)), // Very close to (1,0)
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        
        // After merging, the endpoints should be exactly the same
        assert!((arcs[0].b - arcs[1].a).norm() < 1e-10);
    }
    
    #[test]
    fn test_eliminate_small_line_segment() {
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(1.0 + 1e-10, 0.0)), // Very small segment
            arcseg(point(1.0, 0.0), point(2.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        
        // The small segment should be eliminated
        assert_eq!(arcs.len(), 2);
    }
    
    #[test]
    fn test_eliminate_small_arc() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(1.0, 0.0), point(0.5, 0.5), 1.0),
            arc(point(1.0, 0.0), point(1.0 + 1e-10, 1e-10), point(1.0, 1e-10), 1e-10), // Very small arc
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        
        // The small arc should be eliminated
        assert_eq!(arcs.len(), 1);
    }
    
    #[test]
    fn test_no_merge_needed() {
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(2.0, 0.0), point(3.0, 0.0)), // Far apart
        ];
        
        let original_arcs = arcs.clone();
        merge_close_endpoints(&mut arcs, 1e-8);
        
        // Nothing should change
        assert_eq!(arcs.len(), 2);
        for (i, arc) in arcs.iter().enumerate() {
            assert!((arc.a - original_arcs[i].a).norm() < 1e-10);
            assert!((arc.b - original_arcs[i].b).norm() < 1e-10);
        }
    }
    
    #[test]
    fn test_multiple_point_group() {
        // Three arcs meeting at nearly the same point
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0 + 1e-9, 1e-9), point(1.0, 1.0)),
            arcseg(point(1.0 - 1e-9, -1e-9), point(2.0, 0.0)),
        ];
        
        merge_close_endpoints(&mut arcs, 1e-8);
        
        // All three arcs should meet at exactly the same point
        let meeting_point = arcs[0].b;
        assert!((arcs[1].a - meeting_point).norm() < 1e-10);
        assert!((arcs[2].a - meeting_point).norm() < 1e-10);
    }
    
    #[test]
    fn test_centroid_calculation() {
        let points = vec![
            point(0.0, 0.0),
            point(2.0, 0.0),
            point(1.0, 2.0),
        ];
        
        let centroid = calculate_centroid(&points);
        
        // Centroid should be at (1, 2/3)
        assert!((centroid.x - 1.0).abs() < 1e-10);
        assert!((centroid.y - 2.0/3.0).abs() < 1e-10);
    }

    #[test]
    fn test_four_arcs_common_point() {
        // Test 4 arcs all converging to nearly the same point
        let tolerance = 1e-8;
        let common_point = point(5.0, 5.0);
        let mut arcs = vec![
            // Arc 1: ends at common point
            arcseg(point(0.0, 5.0), common_point),
            // Arc 2: starts from common point with tiny offset
            arcseg(point(5.0 + 5e-9, 5.0 + 3e-9), point(10.0, 5.0)),
            // Arc 3: ends at common point with tiny offset
            arcseg(point(5.0, 0.0), point(5.0 - 2e-9, 5.0 + 1e-9)),
            // Arc 4: starts from common point with tiny offset
            arcseg(point(5.0 + 1e-9, 5.0 - 4e-9), point(5.0, 10.0)),
        ];

        merge_close_endpoints(&mut arcs, tolerance);

        // All arcs should now have exactly the same endpoint/startpoint at the centroid
        let centroid_x = (5.0 + (5.0 + 5e-9) + (5.0 - 2e-9) + (5.0 + 1e-9)) / 4.0;
        let centroid_y = (5.0 + (5.0 + 3e-9) + (5.0 + 1e-9) + (5.0 - 4e-9)) / 4.0;

        assert!((arcs[0].b.x - centroid_x).abs() < 1e-15);
        assert!((arcs[0].b.y - centroid_y).abs() < 1e-15);
        assert!((arcs[1].a.x - centroid_x).abs() < 1e-15);
        assert!((arcs[1].a.y - centroid_y).abs() < 1e-15);
        assert!((arcs[2].b.x - centroid_x).abs() < 1e-15);
        assert!((arcs[2].b.y - centroid_y).abs() < 1e-15);
        assert!((arcs[3].a.x - centroid_x).abs() < 1e-15);
        assert!((arcs[3].a.y - centroid_y).abs() < 1e-15);
    }

    #[test]
    fn test_small_arc_elimination_in_group() {
        let tolerance = 1e-8;
        let mut arcs = vec![
            // Normal arc 1
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            // Very small arc that should be eliminated (connects to point 1,0)
            arcseg(point(1.0 + 2e-9, 1e-9), point(1.0 + 1e-10, 2e-10)),
            // Normal arc 2 that connects to the group
            arcseg(point(1.0 - 3e-9, 1e-9), point(2.0, 0.0)),
            // Another normal arc far away
            arcseg(point(10.0, 10.0), point(15.0, 10.0)),
        ];

        let original_count = arcs.len();
        merge_close_endpoints(&mut arcs, tolerance);

        // The small arc should be eliminated
        assert_eq!(arcs.len(), original_count - 1);
        
        // Verify the remaining arcs are properly connected
        assert!((arcs[0].b - arcs[1].a).norm() < 1e-15);
    }

    #[test]
    fn test_multiple_separate_groups() {
        let tolerance = 1e-8;
        let mut arcs = vec![
            // Group 1: around point (1, 0)
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0 + 2e-9, 1e-9), point(1.5, 0.5)),
            arcseg(point(1.0 - 1e-9, -2e-9), point(1.5, -0.5)),
            
            // Group 2: around point (5, 5) 
            arcseg(point(4.0, 5.0), point(5.0, 5.0)),
            arcseg(point(5.0 + 3e-9, 5.0 - 1e-9), point(6.0, 5.0)),
            
            // Isolated arc (no merging needed)
            arcseg(point(10.0, 10.0), point(15.0, 15.0)),
        ];

        merge_close_endpoints(&mut arcs, tolerance);

        // Group 1: Check that all endpoints around (1,0) are merged to same centroid
        let group1_centroid_x = (1.0 + (1.0 + 2e-9) + (1.0 - 1e-9)) / 3.0;
        let group1_centroid_y = (0.0 + 1e-9 + (-2e-9)) / 3.0;
        
        assert!((arcs[0].b.x - group1_centroid_x).abs() < 1e-15);
        assert!((arcs[0].b.y - group1_centroid_y).abs() < 1e-15);
        assert!((arcs[1].a.x - group1_centroid_x).abs() < 1e-15);
        assert!((arcs[1].a.y - group1_centroid_y).abs() < 1e-15);
        assert!((arcs[2].a.x - group1_centroid_x).abs() < 1e-15);
        assert!((arcs[2].a.y - group1_centroid_y).abs() < 1e-15);

        // Group 2: Check that endpoints around (5,5) are merged
        let group2_centroid_x = (5.0 + (5.0 + 3e-9)) / 2.0;
        let group2_centroid_y = (5.0 + (5.0 - 1e-9)) / 2.0;
        
        assert!((arcs[3].b.x - group2_centroid_x).abs() < 1e-15);
        assert!((arcs[3].b.y - group2_centroid_y).abs() < 1e-15);
        assert!((arcs[4].a.x - group2_centroid_x).abs() < 1e-15);
        assert!((arcs[4].a.y - group2_centroid_y).abs() < 1e-15);

        // Isolated arc should be unchanged
        assert!((arcs[5].a.x - 10.0).abs() < 1e-15);
        assert!((arcs[5].a.y - 10.0).abs() < 1e-15);
        assert!((arcs[5].b.x - 15.0).abs() < 1e-15);
        assert!((arcs[5].b.y - 15.0).abs() < 1e-15);
    }

    #[test]
    fn test_arc_types_mixed() {
        // Test with both line segments and actual arcs
        let tolerance = 1e-8;
        let mut arcs = vec![
            // Line segment
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            // Arc ending at nearly same point
            arc(point(1.0 + 2e-9, 1e-9), point(1.5, 0.5), point(1.25, 0.25), 0.35),
            // Another line segment starting from nearly same point
            arcseg(point(1.0 - 1e-9, -1e-9), point(2.0, 0.0)),
        ];

        merge_close_endpoints(&mut arcs, tolerance);

        // Check that all three arcs connect at the merged point
        let merged_point = arcs[0].b; // Should be the centroid
        assert!((arcs[1].a - merged_point).norm() < 1e-15);
        assert!((arcs[2].a - merged_point).norm() < 1e-15);
        
        // Verify arc properties are preserved
        assert!(arcs[0].is_seg());
        assert!(arcs[1].is_arc());
        assert!(arcs[2].is_seg());
    }

    #[test]
    fn test_edge_case_same_point_exactly() {
        // Test when points are already exactly the same
        let tolerance = 1e-8;
        let exact_point = point(3.0, 4.0);
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), exact_point),
            arcseg(exact_point, point(5.0, 0.0)),
            arcseg(point(2.0, 8.0), exact_point),
        ];

        merge_close_endpoints(&mut arcs, tolerance);

        // All should still connect at the exact same point
        assert!((arcs[0].b - exact_point).norm() < 1e-15);
        assert!((arcs[1].a - exact_point).norm() < 1e-15);
        assert!((arcs[2].b - exact_point).norm() < 1e-15);
    }

    #[test]
    fn test_tolerance_boundary() {
        // Test points at tolerance boundary
        let tolerance = 1e-3;
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(0.0, 0.0), point(0.0, 1.0)),
            arcseg(point(tolerance * 0.8, 0.0), point(2.0, 0.0)), // Close to first point
            arcseg(point(tolerance * 2.0, 0.0), point(3.0, 0.0)),  // Far from first point
        ];

        merge_close_endpoints(&mut arcs, tolerance);

        // Just check that the algorithm runs without panicking
        // The exact behavior depends on implementation details
        assert_eq!(arcs.len(), 4); // All arcs preserved
    }

    #[test]
    fn test_very_small_arcs_elimination() {
        let tolerance = 1e-8;
        let mut arcs = vec![
            // Normal arc
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            // Tiny arc (much smaller than tolerance)
            arcseg(point(1.0, 0.0), point(1.0 + 1e-12, 1e-12)),
            // Another tiny arc 
            arc(point(1.0 + 1e-12, 1e-12), point(1.0 + 2e-12, 0.0), point(1.0 + 1e-12, 1e-13), 1e-12),
            // Normal continuation
            arcseg(point(1.0 + 2e-12, 0.0), point(2.0, 0.0)),
        ];

        let original_count = arcs.len();
        merge_close_endpoints(&mut arcs, tolerance);

        // Small arcs should be eliminated
        assert!(arcs.len() < original_count);
        
        // Remaining arcs should be properly connected
        assert!((arcs[0].b - arcs[arcs.len()-1].a).norm() < 1e-15);
    }

    #[test]
    fn test_complex_star_pattern() {
        // Test a complex star pattern with many arcs converging to center
        let tolerance = 1e-6;
        let mut arcs = vec![];
        
        // Create 8 arcs radiating from slightly offset center points
        for i in 0..8 {
            let angle = (i as f64) * std::f64::consts::PI / 4.0;
            let radius = 2.0;
            let end_point = point(
                radius * angle.cos(),
                radius * angle.sin()
            );
            
            // Add small random offset to center point (within tolerance)
            let offset_center = point(
                (i as f64) * tolerance * 0.1 * ((i % 3) as f64 - 1.0),
                (i as f64) * tolerance * 0.1 * ((i % 5) as f64 - 2.0)
            );
            
            arcs.push(arcseg(offset_center, end_point));
        }

        merge_close_endpoints(&mut arcs, tolerance);

        // Just check that the algorithm runs without panicking
        // and that we still have all arcs
        assert_eq!(arcs.len(), 8);
        
        // Check that the merge did something by verifying first few arcs
        // have endpoints closer together than before
        let first_point = arcs[0].a;
        let mut close_count = 0;
        for arc in &arcs {
            if (arc.a - first_point).norm() < tolerance {
                close_count += 1;
            }
        }
        // At least some arcs should be close to the first one
        assert!(close_count >= 2);
    }

    #[test]
    fn test_chain_of_connections() {
        // Test a chain where each arc connects to the next with small gaps
        let tolerance = 1e-8;
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0 + 2e-9, 1e-9), point(2.0, 0.0)),
            arcseg(point(2.0 - 1e-9, 3e-9), point(3.0, 0.0)),
            arcseg(point(3.0 + 5e-9, -2e-9), point(4.0, 0.0)),
        ];

        merge_close_endpoints(&mut arcs, tolerance);

        // Verify the chain is properly connected
        for i in 0..arcs.len()-1 {
            assert!((arcs[i].b - arcs[i+1].a).norm() < 1e-15);
        }
    }

    #[test]
    fn test_merge_endpoints_diagnostic() {
        // Comprehensive diagnostic test (moved from examples/test_merge_ends.rs)
        println!("Testing merge_ends module...");
        
        // Create a simple test case with two close endpoints
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0 + 1e-9, 0.0 + 1e-9), point(2.0, 0.0)), // Very close to (1,0)
        ];
        
        println!("Before merge:");
        println!("  Arc 0: a=({:.10}, {:.10}), b=({:.10}, {:.10})", 
                 arcs[0].a.x, arcs[0].a.y, arcs[0].b.x, arcs[0].b.y);
        println!("  Arc 1: a=({:.10}, {:.10}), b=({:.10}, {:.10})", 
                 arcs[1].a.x, arcs[1].a.y, arcs[1].b.x, arcs[1].b.y);
        
        let distance_before = (arcs[0].b - arcs[1].a).norm();
        println!("  Distance between endpoints: {:.12}", distance_before);
        
        // Apply merge
        merge_close_endpoints(&mut arcs, 1e-8);
        
        println!("\nAfter merge:");
        println!("  Arc 0: a=({:.10}, {:.10}), b=({:.10}, {:.10})", 
                 arcs[0].a.x, arcs[0].a.y, arcs[0].b.x, arcs[0].b.y);
        if arcs.len() > 1 {
            println!("  Arc 1: a=({:.10}, {:.10}), b=({:.10}, {:.10})", 
                     arcs[1].a.x, arcs[1].a.y, arcs[1].b.x, arcs[1].b.y);
            
            let distance_after = (arcs[0].b - arcs[1].a).norm();
            println!("  Distance between endpoints: {:.12}", distance_after);
            
            // Verify the merge was successful
            assert!(distance_after < 1e-10, "Endpoints should be very close after merge");
        } else {
            println!("  Only {} arc(s) remain", arcs.len());
        }
        
        println!("\nDiagnostic test completed successfully!");
    }
}
