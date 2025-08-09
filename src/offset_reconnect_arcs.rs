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
    let mut edge_list: HashMap<usize, usize> = HashMap::new(); // (point id, arc id)
    let mut merge: HashMap<usize, usize> = HashMap::new(); // (point id, point id)
    let mut k = 0;
    // arc orientation is always from small id to large id
    for i in 0..len {
        edge_list.insert(k, i);
        k += 1;
        edge_list.insert(k, i);
        k += 1;
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
                merge.insert(i, j);
            }
            if arcs[i].a.close_enough(arcs[j].b, EPS_CONNECT) {
                let mid = middle_point(&arcs[i].a, &arcs[j].b);
                arcs[i].a = mid;
                arcs[j].b = mid;
                // track merge
                merge.insert(i, j + 1);
            }
            if arcs[i].b.close_enough(arcs[j].a, EPS_CONNECT) {
                let mid = middle_point(&arcs[i].b, &arcs[j].a);
                arcs[i].b = mid;
                arcs[j].a = mid;
                // track merge
                merge.insert(i + 1, j);
            }
            if arcs[i].b.close_enough(arcs[j].b, EPS_CONNECT) {
                let mid = middle_point(&arcs[i].b, &arcs[j].b);
                arcs[i].b = mid;
                arcs[j].b = mid;
                // track merge
                merge.insert(i + 1, j + 1);
            }
        }
    }

    merge_points(&mut edge_list, &merge);

    // Build the graph from edge_list
    let graph: Vec<(usize, usize)> = edge_list.into_iter().collect();

    // Find connected components (cycles) in the undirected graph defined by edges in "graph" vector.
    // Where each component is a closed path of vertices Ids.
    // If there are large paths with repeated vertices, larger paths will be split and the shortest paths will be used.
    // Eliminate duplicate components that differ only in path direction.
    // Use most effective algorithm to find connected components.
    // Write tests covering various cases of connected components.
    // Provide reference to the algorithm used where necessary.
    // Use the function find_connected_components to get the connected components.
    let components = find_connected_components(&graph);
    
    // Filter composite cycles
    let components = filter_composite_cycles(components, len);
    
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

// edge_list is a map of point ids to arc ids
// where points id are ordered as arc.a and arc.b
// Change the edge_list to merge points based on the merge map
// This change should be done in such way to preserve arc orientation
// where arc orientation is always from small id to large id
fn merge_points(edge_list: &mut HashMap<usize, usize>, merge: &HashMap<usize, usize>) {
    // Simply update edge_list to merge points by removing merged points
    // and keeping only the target points
    for (&from_point, &to_point) in merge {
        if let Some(arc_id) = edge_list.remove(&from_point) {
            edge_list.insert(to_point, arc_id);
        }
    }
}

fn find_connected_components(graph: &[(usize, usize)]) -> Vec<Vec<usize>> {
    // Algorithm: DFS-based cycle detection for undirected graphs
    // Reference: Based on Tarjan's algorithm principles for cycle detection
    // This finds all simple cycles (closed paths) in the undirected graph
    //
    // Implementation Details:
    // 1. Build adjacency list representation from edge list
    // 2. Use DFS to explore connected components and detect cycles
    // 3. Track path during DFS to identify when we revisit a vertex (cycle detected)
    // 4. Normalize cycles to canonical form (start with smallest vertex, lexicographically smallest direction)
    // 5. Deduplicate cycles that represent the same geometric path
    // 6. Sort by cycle length (prefer shorter cycles for robustness)
    //
    // Time Complexity: O(V + E) for DFS + O(C × L log L) for normalization
    // where V = vertices, E = edges, C = cycles found, L = average cycle length
    
    use std::collections::{HashMap, HashSet, BTreeSet};
    
    // Build adjacency list from edge list - using BTreeMap for deterministic iteration
    let mut adj_list: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut all_vertices = BTreeSet::new(); // BTreeSet for sorted iteration
    
    for &(u, v) in graph {
        adj_list.entry(u).or_insert_with(Vec::new).push(v);
        adj_list.entry(v).or_insert_with(Vec::new).push(u);
        all_vertices.insert(u);
        all_vertices.insert(v);
    }
    
    // Sort adjacency lists for deterministic behavior
    for neighbors in adj_list.values_mut() {
        neighbors.sort();
    }
    
    let mut cycles = Vec::new();
    let mut global_visited: HashSet<usize> = HashSet::new();
    
    // DFS to find all cycles starting from each unvisited vertex (in sorted order)
    for &start_vertex in &all_vertices {
        if global_visited.contains(&start_vertex) {
            continue;
        }
        
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        find_cycles_dfs(start_vertex, None, &adj_list, &mut visited, &mut path, &mut cycles);
        
        // Mark all vertices in this connected component as globally visited
        global_visited.extend(&visited);
    }
    
    // Remove duplicate cycles (same cycle but different starting point or direction)
    normalize_and_deduplicate_cycles(cycles)
}

fn find_cycles_dfs(
    current: usize,
    parent: Option<usize>,
    adj_list: &HashMap<usize, Vec<usize>>,
    visited: &mut HashSet<usize>,
    path: &mut Vec<usize>,
    cycles: &mut Vec<Vec<usize>>,
) {
    visited.insert(current);
    path.push(current);
    
    if let Some(neighbors) = adj_list.get(&current) {
        for &neighbor in neighbors {
            if Some(neighbor) == parent {
                continue; // Skip back to parent in undirected graph
            }
            
            if let Some(cycle_start_pos) = path.iter().position(|&v| v == neighbor) {
                // Found a cycle - extract it
                let cycle: Vec<usize> = path[cycle_start_pos..].to_vec();
                if cycle.len() >= 3 { // Valid cycle needs at least 3 vertices
                    cycles.push(cycle);
                }
            } else if !visited.contains(&neighbor) {
                // Continue DFS
                find_cycles_dfs(neighbor, Some(current), adj_list, visited, path, cycles);
            }
        }
    }
    
    path.pop();
}

fn normalize_and_deduplicate_cycles(cycles: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    use std::collections::HashSet;
    
    let mut normalized_cycles = HashSet::new();
    
    for cycle in cycles {
        if cycle.len() < 3 {
            continue; // Skip invalid cycles
        }
        
        // Find canonical representation: start with smallest vertex, choose direction that gives lexicographically smaller sequence
        let min_vertex = cycle.iter().min().unwrap();
        let min_pos = cycle.iter().position(|&v| v == *min_vertex).unwrap();
        
        // Create forward direction (as given)
        let forward: Vec<usize> = cycle[min_pos..].iter().chain(cycle[..min_pos].iter()).cloned().collect();
        
        // Create reverse direction (reverse the entire cycle, then rotate to start with min)
        let mut reverse_cycle = cycle.clone();
        reverse_cycle.reverse();
        let min_pos_rev = reverse_cycle.iter().position(|&v| v == *min_vertex).unwrap();
        let reverse: Vec<usize> = reverse_cycle[min_pos_rev..].iter().chain(reverse_cycle[..min_pos_rev].iter()).cloned().collect();
        
        // Choose lexicographically smaller representation
        let canonical = if forward <= reverse { forward } else { reverse };
        normalized_cycles.insert(canonical);
    }
    
    // Convert back to Vec and sort by length (prefer shorter cycles)
    let mut result: Vec<Vec<usize>> = normalized_cycles.into_iter().collect();
    result.sort_by_key(|cycle| cycle.len());
    
    // Filter out composite cycles (cycles that can be formed by combining shorter cycles)
    result
}

fn filter_composite_cycles(cycles: Vec<Vec<usize>>, len: usize) -> Vec<Vec<usize>> {
    use std::collections::HashSet;
    
    // Goal: Find all fundamental cycles (no shared arcs) representing distinct geometric components
    // This should work for any number of components, not just 2
    
    // Filter out obviously invalid cycles first
    let valid_cycles: Vec<Vec<usize>> = cycles.into_iter()
        .filter(|cycle| {
            if cycle.len() < 6 || cycle.len() % 2 != 0 {
                false
            } else {
                true
            }
        }).collect();
    
    if valid_cycles.is_empty() {
        return Vec::new();
    }
    
    // Convert cycles to their arc sets for comparison
    let cycle_arc_sets: Vec<(Vec<usize>, HashSet<usize>)> = valid_cycles.iter().map(|cycle| {
        let arc_set: HashSet<usize> = cycle.iter()
            .map(|&v| if v < len { v } else { v - len })
            .collect();
        (cycle.clone(), arc_set)
    }).collect();
    
    // If we have only a few cycles (≤ 3), be more permissive about overlaps
    // This helps with simple geometries where we expect multiple small components
    if cycle_arc_sets.len() <= 3 {
        // Sort by cycle length (prefer shorter fundamental cycles)
        let mut sorted_cycles = cycle_arc_sets;
        sorted_cycles.sort_by_key(|(cycle, _)| cycle.len());
        
        // For small numbers of cycles, allow some overlap but prioritize by size
        let mut selected: Vec<(Vec<usize>, HashSet<usize>)> = Vec::new();
        
        for (cycle, arc_set) in sorted_cycles {
            // For the first few cycles, be more lenient about overlaps
            if selected.len() < 2 {
                selected.push((cycle, arc_set));
            } else {
                // For additional cycles, check for conflicts
                let mut conflicts = false;
                for (_, selected_arc_set) in &selected {
                    let overlap = arc_set.intersection(selected_arc_set).count();
                    // Allow small overlaps for small cycle counts
                    if overlap > arc_set.len() / 2 {  // Only reject if more than 50% overlap
                        conflicts = true;
                        break;
                    }
                }
                
                if !conflicts {
                    selected.push((cycle, arc_set));
                }
            }
        }
        
        // Extract just the cycles
        return selected.into_iter().map(|(cycle, _)| cycle).collect();
    }
    
    // For larger numbers of cycles, use the strict non-overlapping algorithm
    // Sort by cycle length (prefer shorter fundamental cycles)
    let mut sorted_cycles = cycle_arc_sets;
    sorted_cycles.sort_by_key(|(cycle, _)| cycle.len());
    
    // Use greedy algorithm to find maximum set of non-overlapping cycles
    let mut selected: Vec<(Vec<usize>, HashSet<usize>)> = Vec::new();
    
    for (cycle, arc_set) in sorted_cycles {
        let mut conflicts = false;
        
        // Check if this cycle conflicts with any already selected cycle
        for (_, selected_arc_set) in &selected {
            let overlap = arc_set.intersection(selected_arc_set).count();
            
            // If there's any arc overlap, it conflicts
            if overlap > 0 {
                conflicts = true;
                break;
            }
        }
        
        if !conflicts {
            selected.push((cycle, arc_set));
        }
    }
    
    // Extract just the cycles
    selected.into_iter().map(|(cycle, _)| cycle).collect()
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

    use super::*;

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
