#![allow(dead_code)]

use std::collections::{HashMap, HashSet}; 

use geom::prelude::*;
use rand::distr::uniform::SampleBorrow;

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
                let x = arc_map.get(&i).unwrap().0;
                let y = arc_map.get(&j).unwrap().0;
                merge.push((x, y));
            }
        }
    }

    merge_points(&mut arc_map, &merge);

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

// arc_map - map arcs and their end vertices
// where points id are ordered as arc.a and arc.b (CCW)
// Reduce the "merge" to make the vertices unique and update arc_map,
// So the arcs vertices are now the updated one indices.
fn merge_points(arc_map: &mut HashMap<usize, (usize, usize)>, merge: &Vec<(usize, usize)>) {
    
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
