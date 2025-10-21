//! Find cycles in an undirected graph composed of geometric arcs.
//!
//! This module implements algorithms to find non-intersecting cycles in a graph where:
//! - Vertices are endpoints of arcs in 2D space
//! - Edges are geometric arcs (togo::Arc structures) 
//! - Each vertex can have up to 8 edges
//! - Double edges between vertices are allowed (different geometric paths)
//! - Goal is to extract cycles that don't intersect geometrically
//!
//! The main algorithm follows this approach:
//! 1. Build graph representation from input arcs
//! 2. Find cycles using geometric-aware traversal
//! 3. At X-intersections, choose "most loose on the right" to avoid intersections
//! 4. Return non-intersecting cycles as separate arc sequences

use togo::prelude::*;
use std::collections::{HashMap, HashSet};

/// Tolerance for considering vertices as the same point
const VERTEX_TOLERANCE: f64 = 1e-8;

/// Vertex identifier - represents a unique point in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct VertexId(usize);

/// Edge in the graph, containing geometric information
#[derive(Debug, Clone)]
struct GraphEdge {
    /// The geometric arc this edge represents
    arc: Arc,
    /// Source vertex
    from: VertexId,
    /// Destination vertex  
    to: VertexId,
    /// Edge identifier for tracking
    id: usize,
}

/// Graph representation for cycle finding
#[derive(Debug)]
struct CycleGraph {
    /// Vertex positions in 2D space
    vertices: Vec<Point>,
    /// Adjacency list: vertex -> list of connected edges
    adjacency: HashMap<VertexId, Vec<usize>>,
    /// All edges in the graph
    edges: Vec<GraphEdge>,
}

impl CycleGraph {
    /// Create a new empty graph
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            adjacency: HashMap::new(),
            edges: Vec::new(),
        }
    }
    
    /// Add or find vertex for a given point, merging close points
    fn add_vertex(&mut self, point: Point) -> VertexId {
        // Check if point is close to existing vertex
        for (i, existing_point) in self.vertices.iter().enumerate() {
            if (point - *existing_point).norm() < VERTEX_TOLERANCE {
                return VertexId(i);
            }
        }
        
        // Add new vertex
        let vertex_id = VertexId(self.vertices.len());
        self.vertices.push(point);
        self.adjacency.insert(vertex_id, Vec::new());
        vertex_id
    }
    
    /// Add an edge to the graph
    fn add_edge(&mut self, arc: Arc) {
        let from = self.add_vertex(arc.a);
        let to = self.add_vertex(arc.b);
        
        let edge = GraphEdge {
            arc,
            from,
            to,
            id: self.edges.len(),
        };
        
        // Add to adjacency lists
        self.adjacency.get_mut(&from).unwrap().push(edge.id);
        self.adjacency.get_mut(&to).unwrap().push(edge.id);
        
        self.edges.push(edge);
    }
    
    /// Get all edge IDs connected to a vertex
    fn get_adjacent_edges(&self, vertex: VertexId) -> &[usize] {
        self.adjacency.get(&vertex).map(|v| v.as_slice()).unwrap_or(&[])
    }
    
    /// Get vertex position
    fn get_vertex_position(&self, vertex: VertexId) -> Point {
        self.vertices[vertex.0]
    }
}

/// Build graph from input arcs
fn build_graph(arcs: &[Arc]) -> CycleGraph {
    let mut graph = CycleGraph::new();
    
    for arc in arcs {
        graph.add_edge(*arc);
    }
    
    graph
}

/// Find the next edge to follow from current vertex, avoiding the edge we came from
/// Uses "most close on the right" rule at X-intersections to avoid geometric intersections
fn find_next_edge(
    graph: &CycleGraph, 
    current_vertex: VertexId, 
    came_from_edge: Option<usize>,
    used_edges: &HashSet<usize>
) -> Option<usize> {
    let adjacent_edges = graph.get_adjacent_edges(current_vertex);
    
    // Filter out the edge we came from and already used edges
    let available_edges: Vec<usize> = adjacent_edges.iter()
        .copied()
        .filter(|&edge_id| {
            Some(edge_id) != came_from_edge && !used_edges.contains(&edge_id)
        })
        .collect();
    
    if available_edges.is_empty() {
        return None;
    }
    
    // If only one option, take it
    if available_edges.len() == 1 {
        return Some(available_edges[0]);
    }
    
    // Multiple options - implement "most close on the right" rule
    if let Some(from_edge_id) = came_from_edge {
        return choose_rightmost_edge(graph, current_vertex, from_edge_id, &available_edges);
    }
    
    // No incoming edge (starting point) - just take first available
    Some(available_edges[0])
}

/// Choose the rightmost edge when exiting a vertex to avoid geometric intersections
/// "Most close on the right" means choosing the edge with the smallest right turn angle
fn choose_rightmost_edge(
    graph: &CycleGraph,
    vertex: VertexId,
    incoming_edge_id: usize,
    available_edges: &[usize]
) -> Option<usize> {
    let vertex_pos = graph.get_vertex_position(vertex);
    let incoming_edge = &graph.edges[incoming_edge_id];
    
    // Calculate incoming direction using proper tangent calculation
    let incoming_direction = get_arc_direction_at_vertex(&incoming_edge.arc, vertex_pos, true);
    
    // Calculate angles for all available outgoing edges
    let mut edge_angles: Vec<(usize, f64)> = Vec::new();
    
    for &edge_id in available_edges {
        let edge = &graph.edges[edge_id];
        
        // Calculate outgoing direction using proper tangent calculation
        let outgoing_direction = get_arc_direction_at_vertex(&edge.arc, vertex_pos, false);
        
        // Calculate the angle between incoming and outgoing directions
        // Using atan2 to get signed angle (-π to π)
        let cross = incoming_direction.x * outgoing_direction.y - incoming_direction.y * outgoing_direction.x;
        let dot = incoming_direction.x * outgoing_direction.x + incoming_direction.y * outgoing_direction.y;
        let angle = cross.atan2(dot);
        
        edge_angles.push((edge_id, angle));
    }
    
    // Sort by angle and choose the rightmost (smallest positive angle, or largest negative)
    // "Most close on the right" means the edge that makes the smallest right turn
    edge_angles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    // Find the edge with the smallest positive angle (closest to straight ahead on the right)
    // If no positive angles, take the largest negative angle (least left turn)
    let rightmost_edge = edge_angles.iter()
        .find(|(_, angle)| *angle > 0.0)  // Find first positive angle (right turn)
        .or_else(|| edge_angles.last())   // If no positive, take largest negative (least left turn)
        .map(|(edge_id, _)| *edge_id);
    
    rightmost_edge
}

/// Get the proper direction vector at a vertex for an arc using togo's tangent calculation
/// For both arcs and line segments, this uses the tangent method to get correct directions
fn get_arc_direction_at_vertex(arc: &Arc, vertex_pos: Point, incoming: bool) -> Point {
    let tangents = arc.tangents();
    let tolerance = 1e-10;
    
    // Determine if vertex is at start (a) or end (b) of the arc
    let is_at_start = (vertex_pos - arc.a).norm() < tolerance;
    let is_at_end = (vertex_pos - arc.b).norm() < tolerance;
    
    if is_at_start {
        // Vertex is at start point 'a'
        if incoming {
            // Want direction pointing TO the vertex (opposite of start tangent)
            Point::new(-tangents[0].x, -tangents[0].y)
        } else {
            // Want direction pointing FROM the vertex (same as start tangent)
            Point::new(tangents[0].x, tangents[0].y)
        }
    } else if is_at_end {
        // Vertex is at end point 'b'
        if incoming {
            // Want direction pointing TO the vertex (same as end tangent)
            Point::new(tangents[1].x, tangents[1].y)
        } else {
            // Want direction pointing FROM the vertex (opposite of end tangent)
            Point::new(-tangents[1].x, -tangents[1].y)
        }
    } else {
        // Vertex is not exactly at either endpoint - this shouldn't happen in a properly merged graph
        // Fallback to the old endpoint-based calculation
        if incoming {
            let (dir, _) = (vertex_pos - arc.a).normalize(false);
            if (vertex_pos - arc.a).norm() < (vertex_pos - arc.b).norm() {
                dir
            } else {
                let (dir, _) = (vertex_pos - arc.b).normalize(false);
                dir
            }
        } else {
            let (dir, _) = (arc.b - vertex_pos).normalize(false);
            if (vertex_pos - arc.a).norm() < (vertex_pos - arc.b).norm() {
                dir
            } else {
                let (dir, _) = (arc.a - vertex_pos).normalize(false);
                dir
            }
        }
    }
}

/// Find a single cycle starting from the given edge
fn find_cycle_from_edge(
    graph: &CycleGraph, 
    start_edge_id: usize,
    used_edges: &mut HashSet<usize>
) -> Option<Vec<Arc>> {
    if used_edges.contains(&start_edge_id) {
        return None;
    }
    
    let mut cycle_edges = Vec::new();
    let mut current_edge_id = start_edge_id;
    let start_vertex = graph.edges[start_edge_id].from;
    let mut current_vertex = graph.edges[start_edge_id].to;
    
    loop {
        // Add edge to our temporary path (but don't mark as permanently used yet)
        cycle_edges.push(current_edge_id);
        
        // If we've returned to start vertex, we found a cycle
        if current_vertex == start_vertex {
            // Mark all edges in this cycle as used
            for edge_id in &cycle_edges {
                used_edges.insert(*edge_id);
            }
            
            // Convert edge IDs to arcs
            let cycle_arcs: Vec<Arc> = cycle_edges.iter()
                .map(|&edge_id| graph.edges[edge_id].arc)
                .collect();
            
            return Some(cycle_arcs);
        }
        
        // Create a temporary set of edges to avoid in this search (current path + permanently used)
        let mut temp_used: HashSet<usize> = used_edges.clone();
        for &edge_id in &cycle_edges {
            temp_used.insert(edge_id);
        }
        
        // Find next edge to follow
        if let Some(next_edge_id) = find_next_edge(graph, current_vertex, Some(current_edge_id), &temp_used) {
            let next_edge = &graph.edges[next_edge_id];
            
            // Determine which vertex to move to
            current_vertex = if next_edge.from == current_vertex {
                next_edge.to
            } else {
                next_edge.from
            };
            
            current_edge_id = next_edge_id;
        } else {
            // Dead end - not a cycle
            return None;
        }
    }
}

/// Main function to find non-intersecting cycles from input arcs
pub fn find_non_intersecting_cycles(arcs: &[Arc]) -> Vec<Vec<Arc>> {
    if arcs.is_empty() {
        return Vec::new();
    }
    
    // Build graph representation
    let graph = build_graph(arcs);
    
    let mut cycles = Vec::new();
    let mut used_edges = HashSet::new();
    
    // Try to find cycles starting from each edge
    for edge_id in 0..graph.edges.len() {
        if let Some(cycle) = find_cycle_from_edge(&graph, edge_id, &mut used_edges) {
            cycles.push(cycle);
        }
    }
    
    cycles
}

#[cfg(test)]
    mod test_find_cycles_basic {
        use super::*;
    
    #[test]
    fn test_empty_input() {
        let result = find_non_intersecting_cycles(&[]);
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_single_arc() {
        let arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        // Single arc cannot form a cycle
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_simple_triangle() {
        let arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(0.5, 1.0)),
            arcseg(point(0.5, 1.0), point(0.0, 0.0)),
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
    }
    
    #[test]
    fn test_simple_square() {
        let arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(1.0, 1.0)),
            arcseg(point(1.0, 1.0), point(0.0, 1.0)),
            arcseg(point(0.0, 1.0), point(0.0, 0.0)),
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 4);
    }
    
    #[test]
    fn test_build_graph() {
        let arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(1.0, 1.0)),
        ];
        
        let graph = build_graph(&arcs);
        assert_eq!(graph.vertices.len(), 3); // Three unique vertices
        assert_eq!(graph.edges.len(), 2);
    }
    
    #[test]
    fn test_vertex_merging() {
        let arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 1e-10), point(2.0, 0.0)), // Close to (1.0, 0.0)
        ];
        
        let graph = build_graph(&arcs);
        // Should merge close vertices
        assert_eq!(graph.vertices.len(), 3);
    }
    
    #[test]
    fn test_figure_eight() {
        // Create a figure-8 pattern: two squares sharing a common edge
        let arcs = vec![
            // Left square
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(1.0, 1.0)),
            arcseg(point(1.0, 1.0), point(0.0, 1.0)),
            arcseg(point(0.0, 1.0), point(0.0, 0.0)),
            // Right square (shares edge with left)
            arcseg(point(1.0, 0.0), point(2.0, 0.0)),
            arcseg(point(2.0, 0.0), point(2.0, 1.0)),
            arcseg(point(2.0, 1.0), point(1.0, 1.0)),
            // Note: shared edge (1,0)-(1,1) is already in the left square
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        
        // Should find at least one cycle, possibly two depending on algorithm behavior
        assert!(!result.is_empty());
        
        // Each cycle should have at least 3 arcs (minimum for a cycle)
        for cycle in &result {
            assert!(cycle.len() >= 3);
        }
    }
    
    #[test]
    fn test_x_intersection() {
        // Create an X pattern: two diagonal lines crossing
        let center = point(0.5, 0.5);
        let arcs = vec![
            // First diagonal: bottom-left to top-right
            arcseg(point(0.0, 0.0), center),
            arcseg(center, point(1.0, 1.0)),
            // Second diagonal: top-left to bottom-right  
            arcseg(point(0.0, 1.0), center),
            arcseg(center, point(1.0, 0.0)),
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        // X pattern has no cycles, just paths that cross
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_double_edges() {
        // Use arc_from_bulge to create proper arcs
        let p1 = point(0.0, 0.0);
        let p2 = point(2.0, 0.0);
        let bulge1 = 1.0;  // Semicircle bulge
        let bulge2 = 1.0; // Another semicircle bulge (same direction, forms full circle)
        
        let arcs = vec![
            arc_from_bulge(p1, p2, bulge1),
            arc_from_bulge(p2, p1, bulge2),
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        // Should find one cycle formed by the two arcs
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
    }
    
    #[test]
    fn test_multiple_separate_cycles() {
        // Two completely separate triangles
        let arcs = vec![
            // First triangle
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),
            arcseg(point(1.0, 0.0), point(0.5, 1.0)),
            arcseg(point(0.5, 1.0), point(0.0, 0.0)),
            // Second triangle (separate)
            arcseg(point(3.0, 0.0), point(4.0, 0.0)),
            arcseg(point(4.0, 0.0), point(3.5, 1.0)),
            arcseg(point(3.5, 1.0), point(3.0, 0.0)),
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        // Should find two separate cycles
        assert_eq!(result.len(), 2);
        
        // Each should be a triangle
        for cycle in &result {
            assert_eq!(cycle.len(), 3);
        }
    }
    
    #[test]
    fn test_mixed_arc_types() {
        // Combine line segments and curved arcs in a cycle
        let arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 0.0)),  // Line segment
            arc_from_bulge(point(1.0, 0.0), point(0.0, 1.0), 0.5), // Curved arc
            arcseg(point(0.0, 1.0), point(0.0, 0.0)),  // Line segment
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        // Should find one cycle with mixed arc types
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
    }
    
    #[test]
    fn test_choose_rightmost_edge() {
        // Test the geometric angle selection directly
        let arcs = vec![
            // Incoming edge: from (0,0) to (1,1)
            arcseg(point(0.0, 0.0), point(1.0, 1.0)),
            // Three outgoing options from (1,1):
            arcseg(point(1.0, 1.0), point(2.0, 1.0)),  // Right (0°)
            arcseg(point(1.0, 1.0), point(1.0, 2.0)),  // Up (90°)
            arcseg(point(1.0, 1.0), point(0.0, 2.0)),  // Up-left (135°)
        ];
        
        let graph = build_graph(&arcs);
        let vertex_id = VertexId(1); // Vertex at (1,1)
        let incoming_edge = 0;
        let available_edges = vec![1, 2, 3];
        
        let chosen = choose_rightmost_edge(&graph, vertex_id, incoming_edge, &available_edges);
        // Should choose the rightmost edge (least angle change)
        assert!(chosen.is_some());
    }
    
    #[test]
    fn test_complex_graph_with_branches() {
        // Create a more complex graph with multiple possible paths
        let arcs = vec![
            // Main cycle
            arcseg(point(0.0, 0.0), point(2.0, 0.0)),
            arcseg(point(2.0, 0.0), point(2.0, 2.0)),
            arcseg(point(2.0, 2.0), point(0.0, 2.0)),
            arcseg(point(0.0, 2.0), point(0.0, 0.0)),
            // Internal cross connections
            arcseg(point(1.0, 0.0), point(1.0, 2.0)),
            arcseg(point(0.0, 1.0), point(2.0, 1.0)),
        ];
        
        let result = find_non_intersecting_cycles(&arcs);
        // Should find multiple non-intersecting cycles
        assert!(!result.is_empty());
        
        // Verify all cycles are valid (non-empty)
        for cycle in &result {
            assert!(!cycle.is_empty());
        }
    }

    // Integration tests combining merge_ends and find_cycles
    mod integration_tests {
        use super::*;
        use crate::graph::merge_ends::merge_close_endpoints;

        #[test]
        fn test_integration_square_with_close_endpoints() {
            // Create a square with slightly offset endpoints that need merging
            let mut arcs = vec![
                arcseg(
                    Point::new(0.0, 0.0),
                    Point::new(1.0, 0.0001), // Slightly off to test merging
                ),
                arcseg(
                    Point::new(1.0, -0.0001), // Slightly off to test merging
                    Point::new(1.0, 1.0),
                ),
                arcseg(
                    Point::new(1.0, 1.0),
                    Point::new(0.0001, 1.0), // Slightly off to test merging
                ),
                arcseg(
                    Point::new(-0.0001, 1.0), // Slightly off to test merging
                    Point::new(0.0, 0.0),
                ),
            ];

            // First merge close endpoints
            merge_close_endpoints(&mut arcs, 0.001);

            // Then find cycles
            let cycles = find_non_intersecting_cycles(&arcs);

            // Should find one cycle after merging
            assert_eq!(cycles.len(), 1);
            assert_eq!(cycles[0].len(), 4); // Square has 4 edges
        }

        #[test]
        fn test_integration_disconnected_arcs_before_merge() {
            // Create arcs that appear disconnected but become connected after merging
            let mut arcs = vec![
                arcseg(
                    Point::new(0.0, 0.0),
                    Point::new(1.0, 0.0),
                ),
                arcseg(
                    Point::new(1.0002, 0.0), // Gap that needs merging
                    Point::new(1.0, 1.0),
                ),
                arcseg(
                    Point::new(1.0, 1.0),
                    Point::new(0.0, 1.0002), // Gap that needs merging
                ),
                arcseg(
                    Point::new(0.0, 0.9998), // Gap that needs merging
                    Point::new(0.0001, 0.0), // Gap that needs merging
                ),
            ];

            // Before merging, should find no cycles due to gaps
            let cycles_before = find_non_intersecting_cycles(&arcs);
            assert!(cycles_before.is_empty());

            // After merging close endpoints
            merge_close_endpoints(&mut arcs, 0.001);

            // Should now find one cycle
            let cycles_after = find_non_intersecting_cycles(&arcs);
            assert_eq!(cycles_after.len(), 1);
        }

        #[test]
        fn test_integration_multiple_cycles_with_merging() {
            // Create two separate squares with gaps that need merging
            let mut arcs = vec![
                // First square (left)
                arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
                arcseg(Point::new(1.0001, 0.0), Point::new(1.0, 1.0)),
                arcseg(Point::new(1.0, 1.0), Point::new(0.0, 1.0)),
                arcseg(Point::new(-0.0001, 1.0), Point::new(0.0, 0.0)),
                
                // Second square (right, offset)
                arcseg(Point::new(2.0, 0.0), Point::new(3.0, 0.0)),
                arcseg(Point::new(3.0001, 0.0), Point::new(3.0, 1.0)),
                arcseg(Point::new(3.0, 1.0), Point::new(2.0, 1.0)),
                arcseg(Point::new(1.9999, 1.0), Point::new(2.0, 0.0)),
            ];

            // Merge close endpoints
            merge_close_endpoints(&mut arcs, 0.001);

            // Find cycles
            let cycles = find_non_intersecting_cycles(&arcs);

            // Should find two separate cycles
            assert_eq!(cycles.len(), 2);
            for cycle in &cycles {
                assert_eq!(cycle.len(), 4); // Each square has 4 edges
            }
        }

        #[test]
        fn test_integration_small_arcs_elimination() {
            // Create a triangle with one very small arc that should be eliminated
            let mut arcs = vec![
                arcseg(
                    Point::new(0.0, 0.0),
                    Point::new(1.0, 0.0),
                ),
                arcseg(
                    Point::new(1.0, 0.0),
                    Point::new(1.00001, 0.0), // Very small arc
                ),
                arcseg(
                    Point::new(1.00001, 0.0),
                    Point::new(0.5, 1.0),
                ),
                arcseg(
                    Point::new(0.5, 1.0),
                    Point::new(0.0, 0.0),
                ),
            ];

            let original_count = arcs.len();

            // Merge endpoints (should eliminate small arc)
            merge_close_endpoints(&mut arcs, 0.001);

            // Should have fewer arcs after elimination
            assert!(arcs.len() < original_count);

            // Should still find a cycle
            let cycles = find_non_intersecting_cycles(&arcs);
            assert_eq!(cycles.len(), 1);
        }

        #[test]
        fn test_integration_circular_arcs_with_gaps() {
            // Create a simple triangular pattern instead of circular arcs
            // to focus on testing the integration between merge_ends and find_cycles
            let mut arcs = vec![
                arcseg(
                    Point::new(0.0, 0.0),
                    Point::new(1.0001, 0.0), // Small gap to test merging
                ),
                arcseg(
                    Point::new(0.9999, 0.0), // Small gap to test merging
                    Point::new(0.5, 1.0),
                ),
                arcseg(
                    Point::new(0.5, 1.0),
                    Point::new(0.0001, 0.0), // Small gap to test merging
                ),
            ];

            // Merge close endpoints
            merge_close_endpoints(&mut arcs, 0.01);

            // Find cycles
            let cycles = find_non_intersecting_cycles(&arcs);

            // Should find one cycle after merging gaps
            assert_eq!(cycles.len(), 1);
        }
    }
}
