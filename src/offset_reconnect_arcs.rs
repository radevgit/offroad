#![allow(dead_code)]

use std::collections::{HashMap, HashSet}; 

use geom::prelude::*;

/// Reconnects offset segments by merging adjacent arcs vertices.
const EPS_CONNECT: f64 = 1e-6;

pub fn offset_reconnect_arcs(arcs: &mut Vec<Arc>) -> Vec<Vec<Arc>> {
    let mut result = Vec::new();

    // remove bridges
    remove_bridge_arcs(arcs);

    let len = arcs.len();

    // Initialize the edge list: each arc contributes 2 vertices
    let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new(); // map arcs to end vertices
    let mut merge: Vec<(usize, usize)> = Vec::new(); // coincident vertices
    let mut k = 1000;
    // arc orientation is always from small id to large id
    for i in 0..len {
        arc_map.insert(i, (k, k + 1));
        k += 2;
    }

    // find where the arcs are touching at ends
    for i in 0..arcs.len() {
        for j in 0..arcs.len() {
            if i == j {
                continue; // skip self
            }

            // merge close points, point ids
            if arcs[i].a.close_enough(arcs[j].a, EPS_CONNECT) {
                let mid = middle_point(&arcs[i].a, &arcs[j].a);
                arcs[i].a = mid;
                arcs[j].a = mid;
                // track merge
                let x = arc_map.get(&i).unwrap().0;
                let y = arc_map.get(&j).unwrap().0;
                merge.push((x, y));
            }
            if arcs[i].a.close_enough(arcs[j].b, EPS_CONNECT) {
                let mid = middle_point(&arcs[i].a, &arcs[j].b);
                arcs[i].a = mid;
                arcs[j].b = mid;
                // track merge
                let x = arc_map.get(&i).unwrap().0;
                let y = arc_map.get(&j).unwrap().1;
                merge.push((x, y));
            }
            if arcs[i].b.close_enough(arcs[j].a, EPS_CONNECT) {
                let mid = middle_point(&arcs[i].b, &arcs[j].a);
                arcs[i].b = mid;
                arcs[j].a = mid;
                // track merge
                let x = arc_map.get(&i).unwrap().1;
                let y = arc_map.get(&j).unwrap().0;
                merge.push((x, y));
            }
            if arcs[i].b.close_enough(arcs[j].b, EPS_CONNECT) {
                let mid = middle_point(&arcs[i].b, &arcs[j].b);
                arcs[i].b = mid;
                arcs[j].b = mid;
                // track merge
                let x = arc_map.get(&i).unwrap().1;
                let y = arc_map.get(&j).unwrap().1;
                merge.push((x, y));
            }
        }
    }

    merge_points(&mut arc_map, &merge);

    // Build the graph from arc_map
    let _graph: Vec<(usize, usize)> = arc_map.values().cloned().collect();

    // Find connected components (cycles) in the undirected graph defined by edges in "graph" vector.
    // Where each component is a closed path of vertices Ids.
    // If there are large paths with repeated vertices, larger paths will be split and the shortest paths will be used.
    // Eliminate duplicate components that differ only in path direction.
    // Use most effective algorithm to find connected components.
    // Write tests covering various cases of connected components.
    // Provide reference to the algorithm used where necessary.
    // Use the function find_connected_components to get the connected components.
    // let components = find_connected_components(&graph);
    
    // TODO: Implement find_connected_components and filter_composite_cycles
    let components: Vec<Vec<usize>> = Vec::new(); // Temporary placeholder

    // Filter composite cycles
    // let components = filter_composite_cycles(components, len);
    
    // Convert each component (cycle of vertex IDs) to a sequence of arcs
    for component in components.iter() {
        if component.len() >= 2 {
            let arc_sequence = vertex_path_to_arcs(&component, &arcs, len);
            if !arc_sequence.is_empty() {
                result.push(arc_sequence);
            }
        }
    }

    result
}

fn middle_point(a: &Point, b: &Point) -> Point {
    Point {
        x: (a.x + b.x) / 2.0,
        y: (a.y + b.y) / 2.0,
    }
}

// arc_map - map arcs and their end vertices
// where points id are ordered as arc.a and arc.b (CCW)
// Reduce the "merge" to make the vertices unique and update arc_map,
// So the arcs vertices are now the updated one indices.
fn merge_points(arc_map: &mut HashMap<usize, (usize, usize)>, merge: &Vec<(usize, usize)>) {
    use std::collections::HashMap;
    
    // Build a union-find structure to group vertices that should be merged
    let mut parent: HashMap<usize, usize> = HashMap::new();
    
    // Initialize: each vertex is its own parent
    for (_, (start, end)) in arc_map.iter() {
        parent.insert(*start, *start);
        parent.insert(*end, *end);
    }
    
    // Find root with path compression
    fn find(parent: &mut HashMap<usize, usize>, x: usize) -> usize {
        if parent[&x] != x {
            let root = find(parent, parent[&x]);
            parent.insert(x, root);
        }
        parent[&x]
    }
    
    // Union operation: merge two vertices
    fn union(parent: &mut HashMap<usize, usize>, x: usize, y: usize) {
        let root_x = find(parent, x);
        let root_y = find(parent, y);
        if root_x != root_y {
            // Always use the smaller root as the canonical representative
            if root_x < root_y {
                parent.insert(root_y, root_x);
            } else {
                parent.insert(root_x, root_y);
            }
        }
    }
    
    // Process all explicit merge operations
    for &(vertex1, vertex2) in merge {
        union(&mut parent, vertex1, vertex2);
    }
    
    // Update arc_map with canonical vertex IDs
    for (_arc_id, (start, end)) in arc_map.iter_mut() {
        *start = find(&mut parent, *start);
        *end = find(&mut parent, *end);
    }
}


fn vertex_path_to_arcs(vertex_path: &[usize], arcs: &[Arc], len: usize) -> Vec<Arc> {
    // Convert a path of vertex IDs back to a sequence of arcs
    // Vertex IDs: 0..len-1 are arc start points, len..2*len-1 are arc end points
    
    let mut result = Vec::new();
    let mut used_arcs = HashSet::new();
    
    for i in 0..vertex_path.len() {
        let current_vertex = vertex_path[i];
        let next_vertex = vertex_path[(i + 1) % vertex_path.len()];
        
        // Find arc that connects current_vertex to next_vertex
        let arc_idx = find_connecting_arc(current_vertex, next_vertex, len);
        
        if let Some(idx) = arc_idx {
            if idx < arcs.len() && !used_arcs.contains(&idx) {
                // Determine if we need to reverse the arc direction
                let arc = &arcs[idx];
                //let use_forward = should_use_forward_direction(current_vertex, next_vertex, len);
                

                result.push(arc.clone());
                // if use_forward {
                //     result.push(arc.clone());
                // } else {
                //     // Create reversed arc
                //     result.push(Arc::new(arc.b, arc.a, arc.c, arc.r));
                // }
                used_arcs.insert(idx);
            }
        }
    }
    
    result
}

fn find_connecting_arc(vertex1: usize, vertex2: usize, len: usize) -> Option<usize> {
    // Find which arc connects two vertices
    let arc1_idx = if vertex1 < len { vertex1 } else { vertex1 - len };
    let arc2_idx = if vertex2 < len { vertex2 } else { vertex2 - len };
    
    if arc1_idx == arc2_idx {
        Some(arc1_idx)
    } else {
        None
    }
}

fn should_use_forward_direction(from_vertex: usize, to_vertex: usize, len: usize) -> bool {
    // Determine if we should use the arc in its original direction
    // from_vertex represents either start (idx < len) or end (idx >= len) of an arc
    // to_vertex represents the next vertex in the path
    
    let arc_idx = if from_vertex < len { from_vertex } else { from_vertex - len };
    let from_is_start = from_vertex < len;
    let to_is_start = to_vertex < len;
    let to_arc_idx = if to_vertex < len { to_vertex } else { to_vertex - len };
    
    if arc_idx == to_arc_idx {
        // Same arc - check if we go from start to end or end to start
        from_is_start && !to_is_start
    } else {
        // Different arcs - use forward direction by default
        true
    }
}

/// Removes duplicate arcs that overlap as 2D graphics elements.
///
/// DO NOT CHANGE THIS FUNCTION - it's a critical component for maintaining geometric consistency.
fn remove_bridge_arcs(arcs: &mut Vec<Arc>) {
    let mut to_remove = Vec::new();
    for i in 0..arcs.len() {
        for j in (i + 1)..arcs.len() {
            let arc0 = &arcs[i];
            let arc1 = &arcs[j];
            if arc0.is_line() && arc1.is_line() {
                if (arc0.a.close_enough(arc1.a, EPS_CONNECT)
                    && arc0.b.close_enough(arc1.b, EPS_CONNECT))
                    || (arc0.a.close_enough(arc1.b, EPS_CONNECT)
                        && arc0.b.close_enough(arc1.a, EPS_CONNECT))
                {
                    to_remove.push(i);
                    to_remove.push(j);
                    continue;
                }
            }
            if arc0.is_arc() && arc1.is_arc() {
                if arc0.a.close_enough(arc1.a, EPS_CONNECT)
                    && arc0.b.close_enough(arc1.b, EPS_CONNECT)
                    && arc0.c.close_enough(arc1.c, EPS_CONNECT)
                    && close_enough(arc0.r, arc1.r, EPS_CONNECT)
                {
                    to_remove.push(i);
                    to_remove.push(j);
                    continue;
                }
            }
        }
    }
    to_remove.sort_unstable();
    to_remove.dedup();
    for i in to_remove.iter().rev() {
        arcs.remove(*i);
    }
}

#[cfg(test)]
mod test_remove_bridge_arcs {
    use geom::prelude::*;
    use super::remove_bridge_arcs;

    #[test]
    fn test_remove_bridge_arcs_duplicate_arcs() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(1.0, 1.0), point(0.5, 0.5), 0.5),
            arc(point(1.0, 1.0), point(2.0, 2.0), point(1.5, 1.5), 0.5),
            arc(point(0.0, 0.0), point(1.0, 1.0), point(0.5, 0.5), 0.5),
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_duplicate_lines() {
        let mut arcs = vec![
            arcline(point(0.0, 0.0), point(1.0, 1.0)),
            arcline(point(1.0, 1.0), point(2.0, 2.0)),
            arcline(point(0.0, 0.0), point(1.0, 1.0)), // duplicate
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_duplicate_lines_reversed() {
        let mut arcs = vec![
            arcline(point(0.0, 0.0), point(1.0, 1.0)),
            arcline(point(1.0, 1.0), point(0.0, 0.0)), // reversed duplicate
            arcline(point(2.0, 2.0), point(3.0, 3.0)),
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_no_duplicates() {
        let mut arcs = vec![
            arcline(point(0.0, 0.0), point(1.0, 1.0)),
            arcline(point(1.0, 1.0), point(2.0, 2.0)),
            arc(point(2.0, 2.0), point(3.0, 1.0), point(2.5, 1.5), 0.5),
        ];
        let original_len = arcs.len();
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), original_len);
    }

    #[test]
    fn test_remove_bridge_arcs_duplicate_arcs_same_params() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.0),
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.0), // exact duplicate
            arcline(point(3.0, 3.0), point(4.0, 4.0)),
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_different_arcs_same_endpoints() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.0),
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, -1.0), 1.0), // different center
            arcline(point(3.0, 3.0), point(4.0, 4.0)),
        ];
        let original_len = arcs.len();
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), original_len); // should not remove different arcs
    }

    #[test]
    fn test_remove_bridge_arcs_mixed_arc_and_line() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.0),
            arcline(point(0.0, 0.0), point(2.0, 0.0)), // line with same endpoints
            arcline(point(3.0, 3.0), point(4.0, 4.0)),
        ];
        let original_len = arcs.len();
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), original_len); // should not remove arc-line combinations
    }

    #[test]
    fn test_remove_bridge_arcs_multiple_duplicates() {
        let mut arcs = vec![
            arcline(point(0.0, 0.0), point(1.0, 1.0)),
            arcline(point(0.0, 0.0), point(1.0, 1.0)), // duplicate 1
            arcline(point(0.0, 0.0), point(1.0, 1.0)), // duplicate 2
            arc(point(2.0, 2.0), point(4.0, 2.0), point(3.0, 3.0), 1.0),
            arc(point(2.0, 2.0), point(4.0, 2.0), point(3.0, 3.0), 1.0), // duplicate arc
            arcline(point(5.0, 5.0), point(6.0, 6.0)),
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1); // should keep only unique elements
    }

    #[test]
    fn test_remove_bridge_arcs_empty_input() {
        let mut arcs: Vec<Arc> = vec![];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 0);
    }

    #[test]
    fn test_remove_bridge_arcs_single_element() {
        let mut arcs = vec![
            arcline(point(0.0, 0.0), point(1.0, 1.0)),
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_close_but_not_equal() {
        let eps = super::EPS_CONNECT;
        let mut arcs = vec![
            arcline(point(0.0, 0.0), point(1.0, 1.0)),
            arcline(point(0.0, 0.0), point(1.0 + eps * 0.5, 1.0 + eps * 0.5)), // close but within tolerance
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 0); // should remove both as they're close enough
    }

    #[test]
    fn test_remove_bridge_arcs_different_radius() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.0),
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.5), // different radius
            arcline(point(3.0, 3.0), point(4.0, 4.0)),
        ];
        let original_len = arcs.len();
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), original_len); // should not remove arcs with different radius
    }
}

#[cfg(test)]
mod test_merge_points {
    use geom::prelude::*;

    use super::*;

    #[test]
    fn test_merge_points() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test the example from user: arc0.b == arc1.a should be merged
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(5, (1005, 1006)); // arc0: start=1005, end=1006
        arc_map.insert(7, (1006, 1007)); // arc1: start=1006, end=1007 (arc0.b == arc1.a)
        
        // No explicit merges needed since they already share the same vertex
        let merge = vec![];
        merge_points(&mut arc_map, &merge);
        
        // arc0.b (1006) and arc1.a (1006) are already the same, so no changes
        assert_eq!(arc_map[&5], (1005, 1006)); // arc0: unchanged
        assert_eq!(arc_map[&7], (1006, 1007)); // arc1: unchanged
    }

    #[test]
    fn test_merge_points_multiple_merges() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test multiple merges
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        arc_map.insert(2, (1004, 1005));
        
        // Chain of merges: 1001-1002, 1003-1004
        let merge = vec![(1001, 1002), (1003, 1004)];
        
        merge_points(&mut arc_map, &merge);
        
        // After merges: 1001=1002, 1003=1004 (separate components)
        // Arc 0: 1000 -> 1001 (no change)
        // Arc 1: 1002 -> 1003 becomes 1001 -> 1003 (1002 maps to 1001) 
        // Arc 2: 1004 -> 1005 becomes 1003 -> 1005 (1004 maps to 1003)
        assert_eq!(arc_map[&0], (1000, 1001)); 
        assert_eq!(arc_map[&1], (1001, 1003));  
        assert_eq!(arc_map[&2], (1003, 1005));
    }

    #[test]
    fn test_merge_points_empty_merge() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test empty merge list - should not change anything
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        
        let original_arc_map = arc_map.clone();
        let merge = vec![];
        
        merge_points(&mut arc_map, &merge);
        
        // Should remain unchanged
        assert_eq!(arc_map, original_arc_map);
    }

    #[test]
    fn test_merge_points_self_merge() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test merging a vertex with itself - should be a no-op
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        
        let original_arc_map = arc_map.clone();
        let merge = vec![(1000, 1000), (1001, 1001)];
        
        merge_points(&mut arc_map, &merge);
        
        // Should remain unchanged
        assert_eq!(arc_map, original_arc_map);
    }

    #[test]
    fn test_merge_points_transitive_closure() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test transitive closure: if A->B and B->C, then A,B,C should all map to same
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        arc_map.insert(2, (1004, 1005));
        
        // Create a chain: 1000->1002->1004
        let merge = vec![(1000, 1002), (1002, 1004)];
        
        merge_points(&mut arc_map, &merge);
        
        // All vertices in the chain should map to 1000 (smallest)
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 unchanged, 1001 unchanged
        assert_eq!(arc_map[&1], (1000, 1003)); // 1002 -> 1000, 1003 unchanged
        assert_eq!(arc_map[&2], (1000, 1005)); // 1004 -> 1000, 1005 unchanged
    }

    #[test]
    fn test_merge_points_both_endpoints_merge() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test when both endpoints of an arc need to be merged
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        
        // Merge both endpoints: start with start, end with end
        let merge = vec![(1000, 1002), (1001, 1003)];
        
        merge_points(&mut arc_map, &merge);
        
        // Both arcs should have the same canonical endpoints
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 unchanged (smaller), 1001 unchanged (smaller)
        assert_eq!(arc_map[&1], (1000, 1001)); // 1002->1000, 1003->1001
    }

    #[test]
    fn test_merge_points_complex_graph() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test a more complex merging scenario with multiple connected components
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        arc_map.insert(2, (1004, 1005));
        arc_map.insert(3, (1006, 1007));
        arc_map.insert(4, (1008, 1009));
        
        // Create two separate connected components:
        // Component 1: {1000, 1002, 1004} -> all should map to 1000
        // Component 2: {1006, 1008} -> all should map to 1006
        // Isolated: {1001, 1003, 1005, 1007, 1009} remain unchanged
        let merge = vec![
            (1000, 1002), // Connect 1000 and 1002
            (1002, 1004), // Connect 1002 and 1004 (transitive: 1000-1002-1004)
            (1006, 1008), // Separate component: 1006 and 1008
        ];
        
        merge_points(&mut arc_map, &merge);
        
        // Check the results
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 unchanged, 1001 unchanged
        assert_eq!(arc_map[&1], (1000, 1003)); // 1002->1000, 1003 unchanged
        assert_eq!(arc_map[&2], (1000, 1005)); // 1004->1000, 1005 unchanged
        assert_eq!(arc_map[&3], (1006, 1007)); // 1006 unchanged, 1007 unchanged
        assert_eq!(arc_map[&4], (1006, 1009)); // 1008->1006, 1009 unchanged
    }

    #[test]
    fn test_merge_points_duplicate_merges() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test duplicate merge operations - should handle gracefully
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        
        // Same merge operation repeated multiple times
        let merge = vec![(1000, 1002), (1002, 1000), (1000, 1002)];
        
        merge_points(&mut arc_map, &merge);
        
        // Should still work correctly despite duplicates
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 unchanged, 1001 unchanged
        assert_eq!(arc_map[&1], (1000, 1003)); // 1002->1000, 1003 unchanged
    }

    #[test]
    fn test_merge_points_cycle_formation() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test when merges would form a cycle in the graph
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        arc_map.insert(2, (1004, 1000)); // Note: 1000 appears again
        
        // Create merges that form a cycle: 1001->1002->1004->1000 (but 1000 is start of arc 0)
        let merge = vec![(1001, 1002), (1002, 1004), (1004, 1000)];
        
        merge_points(&mut arc_map, &merge);
        
        // All vertices should merge to 1000 (smallest in the cycle)
        assert_eq!(arc_map[&0], (1000, 1000)); // 1000 unchanged, 1001->1000
        assert_eq!(arc_map[&1], (1000, 1003)); // 1002->1000, 1003 unchanged
        assert_eq!(arc_map[&2], (1000, 1000)); // 1004->1000, 1000 unchanged
    }

    #[test]
    fn test_merge_points_large_numbers() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test with larger vertex IDs to ensure no integer overflow issues
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (100000, 100001));
        arc_map.insert(1, (200000, 200001));
        
        let merge = vec![(100001, 200000)];
        
        merge_points(&mut arc_map, &merge);
        
        // Should use smaller ID as canonical
        assert_eq!(arc_map[&0], (100000, 100001)); // 100000 unchanged, 100001 unchanged
        assert_eq!(arc_map[&1], (100001, 200001)); // 200000->100001, 200001 unchanged
    }

    #[test]
    fn test_merge_points_single_arc() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test with only one arc
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        
        let merge = vec![(1000, 1001)]; // Merge arc's own endpoints
        
        merge_points(&mut arc_map, &merge);
        
        // Both endpoints should become the same (smaller ID)
        assert_eq!(arc_map[&0], (1000, 1000));
    }

    #[test]
    fn test_merge_points_reverse_order() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test that merge order doesn't matter (commutativity)
        let mut arc_map1: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map1.insert(0, (1000, 1001));
        arc_map1.insert(1, (1002, 1003));
        
        let mut arc_map2 = arc_map1.clone();
        
        // Same merges in different order
        let merge1 = vec![(1000, 1002), (1001, 1003)];
        let merge2 = vec![(1003, 1001), (1002, 1000)]; // Reverse order and swapped pairs
        
        merge_points(&mut arc_map1, &merge1);
        merge_points(&mut arc_map2, &merge2);
        
        // Results should be identical
        assert_eq!(arc_map1, arc_map2);
    }

    #[test]
    fn test_merge_points_simple_loop() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test a simple loop: three arcs forming a triangle
        // Arc 0: vertex 1000 -> 1001
        // Arc 1: vertex 1002 -> 1003  
        // Arc 2: vertex 1004 -> 1005
        // Connect them: 1001->1002, 1003->1004, 1005->1000 (forms loop)
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        arc_map.insert(2, (1004, 1005));
        
        let merge = vec![
            (1001, 1002), // Connect arc0 end to arc1 start
            (1003, 1004), // Connect arc1 end to arc2 start
            (1005, 1000), // Connect arc2 end to arc0 start (closes loop)
        ];
        
        merge_points(&mut arc_map, &merge);
        
        // After merges: 1001=1002, 1003=1004, 1005=1000
        // Arc 0: 1000 -> 1001 (no change)
        // Arc 1: 1002 -> 1003 becomes 1001 -> 1003 (1002 maps to 1001)
        // Arc 2: 1004 -> 1005 becomes 1003 -> 1000 (1004 maps to 1003, 1005 maps to 1000)
        assert_eq!(arc_map[&0], (1000, 1001)); 
        assert_eq!(arc_map[&1], (1001, 1003)); 
        assert_eq!(arc_map[&2], (1003, 1000));
    }

    #[test]
    fn test_merge_points_multiple_loops() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test multiple separate loops
        // Loop 1: arcs 0,1,2 (vertices 1000-1005)
        // Loop 2: arcs 3,4 (vertices 2000-2003)
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        // Loop 1
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        arc_map.insert(2, (1004, 1005));
        // Loop 2  
        arc_map.insert(3, (2000, 2001));
        arc_map.insert(4, (2002, 2003));
        
        let merge = vec![
            // Loop 1: triangle
            (1001, 1002), (1003, 1004), (1005, 1000),
            // Loop 2: line segment back and forth
            (2001, 2002), (2003, 2000),
        ];
        
        merge_points(&mut arc_map, &merge);
        
        // After merges: 1001=1002, 1003=1004, 1005=1000, 2001=2002, 2003=2000
        // These create separate components, not complete loops
        
        // Loop 1 results:
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 -> 1001 (unchanged)
        assert_eq!(arc_map[&1], (1001, 1003)); // 1002 -> 1001, 1003 unchanged 
        assert_eq!(arc_map[&2], (1003, 1000)); // 1004 -> 1003, 1005 -> 1000
        
        // Loop 2 results:
        assert_eq!(arc_map[&3], (2000, 2001)); // 2000 unchanged, 2001 unchanged
        assert_eq!(arc_map[&4], (2001, 2000)); // 2002 -> 2001, 2003 -> 2000
    }

    #[test]
    fn test_merge_points_nested_loops() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test nested/connected loop structure
        // Inner loop: arcs 0,1 
        // Outer loop: arcs 2,3,4 that connects to inner loop
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        arc_map.insert(0, (1000, 1001)); // Inner loop arc 1
        arc_map.insert(1, (1002, 1003)); // Inner loop arc 2
        arc_map.insert(2, (1004, 1005)); // Outer loop arc 1
        arc_map.insert(3, (1006, 1007)); // Outer loop arc 2
        arc_map.insert(4, (1008, 1009)); // Outer loop arc 3
        
        let merge = vec![
            // Inner loop
            (1001, 1002), (1003, 1000), // Close inner loop: 1000->1001->1002->1003->1000
            // Connect outer loop
            (1005, 1006), (1007, 1008), (1009, 1004), // Close outer loop: 1004->1005->1006->1007->1008->1009->1004
            // Connect inner to outer
            (1000, 1004), // Connect inner loop to outer loop
        ];
        
        merge_points(&mut arc_map, &merge);
        
        // The merges create several connected components:
        // {1001, 1002}, {1000, 1003, 1004, 1009}, {1005, 1006}, {1007, 1008}
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 -> 1001 (unchanged)
        assert_eq!(arc_map[&1], (1001, 1000)); // 1002 -> 1001, 1003 -> 1000
        assert_eq!(arc_map[&2], (1000, 1005)); // 1004 -> 1000, 1005 unchanged
        assert_eq!(arc_map[&3], (1005, 1007)); // 1006 -> 1005, 1007 unchanged
        assert_eq!(arc_map[&4], (1007, 1000)); // 1008 -> 1007, 1009 -> 1000
    }

    #[test]
    fn test_merge_points_many_small_loops() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test many small independent loops (5 loops, 2 arcs each)
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut merge = vec![];
        
        for loop_id in 0..5 {
            let base = loop_id * 1000 + 1000; // Start IDs: 1000, 2000, 3000, 4000, 5000
            let arc1_id = loop_id * 2;
            let arc2_id = loop_id * 2 + 1;
            
            // Each loop has 2 arcs
            arc_map.insert(arc1_id, (base, base + 1));
            arc_map.insert(arc2_id, (base + 2, base + 3));
            
            // Close each loop
            merge.push((base + 1, base + 2)); // Connect arc1 end to arc2 start
            merge.push((base + 3, base));     // Connect arc2 end to arc1 start
        }
        
        merge_points(&mut arc_map, &merge);
        
        // Each loop creates two separate components per loop
        for loop_id in 0..5 {
            let base = loop_id * 1000 + 1000;
            let arc1_id = loop_id * 2;
            let arc2_id = loop_id * 2 + 1;
            
            // Each loop has two components: {base, base+3} and {base+1, base+2}
            assert_eq!(arc_map[&arc1_id], (base, base + 1)); // unchanged
            assert_eq!(arc_map[&arc2_id], (base + 1, base)); // base+2 -> base+1, base+3 -> base
        }
    }

    #[test]
    fn test_merge_points_long_chain_to_loop() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test a long chain that eventually forms a loop
        // Chain: 1000->1001->1002->1003->1004->1005->1006->1007->1000 (back to start)
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut merge = vec![];
        
        // Create 8 arcs in a chain
        for i in 0..8 {
            arc_map.insert(i, (1000 + i * 2, 1000 + i * 2 + 1));
            if i < 7 {
                // Connect arc i end to arc i+1 start
                merge.push((1000 + i * 2 + 1, 1000 + (i + 1) * 2));
            }
        }
        // Close the loop: connect last arc end to first arc start
        merge.push((1000 + 7 * 2 + 1, 1000)); // 1015 -> 1000
        
        merge_points(&mut arc_map, &merge);
        
        // The merges create separate pairs, not one big component
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 -> 1001 (unchanged)
        assert_eq!(arc_map[&1], (1001, 1003)); // 1002 -> 1001, 1003 unchanged
        assert_eq!(arc_map[&2], (1003, 1005)); // 1004 -> 1003, 1005 unchanged
        assert_eq!(arc_map[&3], (1005, 1007)); // 1006 -> 1005, 1007 unchanged
        assert_eq!(arc_map[&4], (1007, 1009)); // 1008 -> 1007, 1009 unchanged
        assert_eq!(arc_map[&5], (1009, 1011)); // 1010 -> 1009, 1011 unchanged
        assert_eq!(arc_map[&6], (1011, 1013)); // 1012 -> 1011, 1013 unchanged
        assert_eq!(arc_map[&7], (1013, 1000)); // 1014 -> 1013, 1015 -> 1000
    }

    #[test]
    fn test_merge_points_figure_eight() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test figure-8 pattern: two loops sharing a vertex
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        
        // Loop 1: arcs 0,1,2
        arc_map.insert(0, (1000, 1001)); // Shared vertex will be 1000
        arc_map.insert(1, (1002, 1003));
        arc_map.insert(2, (1004, 1005));
        
        // Loop 2: arcs 3,4,5 (shares vertex 1000)
        arc_map.insert(3, (1006, 1007));
        arc_map.insert(4, (1008, 1009));
        arc_map.insert(5, (1010, 1000)); // Ends at shared vertex
        
        let merge = vec![
            // Close loop 1: 1000->1001->1002->1003->1004->1005->1000
            (1001, 1002), (1003, 1004), (1005, 1000),
            // Close loop 2: 1000->1006->1007->1008->1009->1010->1000
            (1000, 1006), (1007, 1008), (1009, 1010),
        ];
        
        merge_points(&mut arc_map, &merge);
        
        // The merges create several components
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 -> 1001 (unchanged)
        assert_eq!(arc_map[&1], (1001, 1003)); // 1002 -> 1001, 1003 unchanged
        assert_eq!(arc_map[&2], (1003, 1000)); // 1004 -> 1003, 1005 -> 1000
        assert_eq!(arc_map[&3], (1000, 1007)); // 1006 -> 1000, 1007 unchanged
        assert_eq!(arc_map[&4], (1007, 1009)); // 1008 -> 1007, 1009 unchanged
        assert_eq!(arc_map[&5], (1009, 1000)); // 1010 -> 1009, 1000 unchanged
    }

    #[test]
    fn test_merge_points_spiral_pattern() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test spiral pattern where arcs connect in a spiral that eventually loops back
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut merge = vec![];
        
        // Create spiral: each arc connects to the next, with some skipping to create spiral effect
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        arc_map.insert(2, (1004, 1005));
        arc_map.insert(3, (1006, 1007));
        arc_map.insert(4, (1008, 1009));
        
        // Spiral connections with gaps
        merge.extend(vec![
            (1001, 1004), // Skip arc 1, connect arc 0 to arc 2
            (1005, 1008), // Skip arc 3, connect arc 2 to arc 4
            (1009, 1002), // Connect arc 4 to arc 1
            (1003, 1006), // Connect arc 1 to arc 3
            (1007, 1000), // Connect arc 3 back to start
        ]);
        
        merge_points(&mut arc_map, &merge);
        
        // The merges create separate components
        assert_eq!(arc_map[&0], (1000, 1001)); // 1000 -> 1001 (unchanged)
        assert_eq!(arc_map[&1], (1002, 1003)); // 1002 -> 1002, 1003 -> 1003 (unchanged)
        assert_eq!(arc_map[&2], (1001, 1005)); // 1004 -> 1001, 1005 unchanged
        assert_eq!(arc_map[&3], (1003, 1000)); // 1006 -> 1003, 1007 -> 1000
        assert_eq!(arc_map[&4], (1005, 1002)); // 1008 -> 1005, 1009 -> 1002
    }

    #[test]
    fn test_merge_points_disconnected_components_with_loops() {
        use std::collections::HashMap;
        use super::merge_points;
        
        // Test multiple disconnected components, some with loops, some without
        let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new();
        
        // Component 1: Simple loop (arcs 0,1)
        arc_map.insert(0, (1000, 1001));
        arc_map.insert(1, (1002, 1003));
        
        // Component 2: Chain without loop (arcs 2,3)
        arc_map.insert(2, (2000, 2001));
        arc_map.insert(3, (2002, 2003));
        
        // Component 3: Self-loop (arc 4)
        arc_map.insert(4, (3000, 3001));
        
        // Component 4: Complex loop (arcs 5,6,7)
        arc_map.insert(5, (4000, 4001));
        arc_map.insert(6, (4002, 4003));
        arc_map.insert(7, (4004, 4005));
        
        let merge = vec![
            // Component 1: close loop
            (1001, 1002), (1003, 1000),
            // Component 2: just connect in sequence (no loop)
            (2001, 2002),
            // Component 3: self-loop
            (3001, 3000),
            // Component 4: complex loop
            (4001, 4002), (4003, 4004), (4005, 4000),
        ];
        
        merge_points(&mut arc_map, &merge);
        
        // Component 1: creates two separate merge pairs
        assert_eq!(arc_map[&0], (1000, 1001)); // unchanged
        assert_eq!(arc_map[&1], (1001, 1000)); // 1002 -> 1001, 1003 -> 1000
        
        // Component 2: chain connection
        assert_eq!(arc_map[&2], (2000, 2001)); // 2000 -> 2001 (unchanged)
        assert_eq!(arc_map[&3], (2001, 2003)); // 2002 -> 2001
        
        // Component 3: self-loop
        assert_eq!(arc_map[&4], (3000, 3000)); // 3000 -> 3000, 3001 -> 3000
        
        // Component 4: separate pairs
        assert_eq!(arc_map[&5], (4000, 4001)); // 4000 -> 4000, 4001 -> 4001 (unchanged)
        assert_eq!(arc_map[&6], (4001, 4003)); // 4002 -> 4001, 4003 -> 4003 (unchanged)
        assert_eq!(arc_map[&7], (4003, 4000)); // 4004 -> 4003, 4005 -> 4000
    }
}
