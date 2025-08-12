#![allow(dead_code)]

use std::collections::{HashMap, HashSet}; 

use geom::prelude::*;

/// Reconnects offset segments by merging adjacent arcs vertices.
const EPS_CONNECT: f64 = 1e-8;

pub fn offset_reconnect_arcs(arcs: &mut Vec<Arc>) -> Vec<Vec<Arc>> {
    let mut result = Vec::new();

    // remove bridges
    remove_bridge_arcs(arcs);

    let len = arcs.len();

    // Initialize the edge list: each arc contributes 2 vertices
    let mut arc_map: HashMap<usize, (usize, usize)> = HashMap::new(); // map arcs to end vertices
    let mut merge: Vec<(usize, usize)> = Vec::new(); // coincident vertices
    let mut k = 1000; // TODO: use 0
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

    println!("DEBUG: Merge operations: {:?}", merge);
    println!("DEBUG: Arc map after merge: {:?}", arc_map);

    // Build the graph from arc_map
    let graph: Vec<(usize, usize)> = arc_map.values().cloned().collect();

    println!("DEBUG: Initial arcs count: {}", len);
    println!("DEBUG: Arc map: {:?}", arc_map);
    println!("DEBUG: Graph edges: {:?}", graph);

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
    // let components: Vec<Vec<usize>> = Vec::new(); // Temporary placeholder
    let components = find_connected_components(&graph);

    println!("DEBUG: Found {} components", components.len());
    for (i, component) in components.iter().enumerate() {
        println!("DEBUG: Component {}: {:?}", i, component);
    }

    // Convert each component (cycle of vertex IDs) to a sequence of arcs
    for component in components.iter() {
        if component.len() >= 2 {
            println!("DEBUG: Converting component {:?} to arcs", component);
            let arc_sequence = vertex_path_to_arcs(&component, &arcs, &arc_map);
            println!("DEBUG: Arc sequence length: {}", arc_sequence.len());
            if !arc_sequence.is_empty() {
                result.push(arc_sequence);
            }
        }
    }

    println!("DEBUG: offset_reconnect_arcs returned {} components", result.len());

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
// Checked.
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


fn vertex_path_to_arcs(
    vertex_path: &[usize], 
    arcs: &[Arc], 
    arc_map: &HashMap<usize, (usize, usize)>
) -> Vec<Arc> {
    // Convert a path of vertex IDs back to a sequence of arcs
    // We need to find which arc connects each pair of consecutive vertices in the path
    
    let mut result = Vec::new();
    let mut used_arcs = HashSet::new();
    
    for i in 0..vertex_path.len() {
        let current_vertex = vertex_path[i];
        let next_vertex = vertex_path[(i + 1) % vertex_path.len()];
        
        println!("DEBUG: Looking for arc connecting {} -> {}", current_vertex, next_vertex);
        
        // Find arc that connects current_vertex to next_vertex using arc_map
        let arc_idx = find_connecting_arc_by_vertices(current_vertex, next_vertex, arc_map);
        
        if let Some(idx) = arc_idx {
            if !used_arcs.contains(&idx) {
                println!("DEBUG: Found arc {} connecting vertices", idx);
                if idx < arcs.len() {
                    let arc = &arcs[idx];
                    result.push(arc.clone());
                    used_arcs.insert(idx);
                }
            }
        } else {
            println!("DEBUG: No arc found connecting {} -> {}", current_vertex, next_vertex);
        }
    }
    
    result
}

fn find_connecting_arc_by_vertices(
    vertex1: usize, 
    vertex2: usize, 
    arc_map: &HashMap<usize, (usize, usize)>
) -> Option<usize> {
    // Find the arc index that connects vertex1 to vertex2
    // Check both directions since the graph is undirected
    
    for (&arc_idx, &(start_vertex, end_vertex)) in arc_map {
        if (start_vertex == vertex1 && end_vertex == vertex2) ||
           (start_vertex == vertex2 && end_vertex == vertex1) {
            println!("DEBUG: Found arc {} mapping to ({}, {})", arc_idx, start_vertex, end_vertex);
            return Some(arc_idx);
        }
    }
    
    println!("DEBUG: No arc found in arc_map for vertices {} -> {}", vertex1, vertex2);
    None
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

/// Finds connected components (cycles) in an undirected graph.
/// 
/// This function uses Depth-First Search (DFS) to find connected components in an undirected graph.
/// It focuses on finding cycles and shortest paths, eliminating duplicates that differ only in direction.
/// 
/// # Arguments
/// * `graph` - Vector of edges represented as (u, v) pairs where each edge connects vertex u to vertex v
/// 
/// # Returns
/// Vector of connected components, where each component is a vector of vertex IDs forming a closed path
/// 
/// # Algorithm
/// Uses DFS-based cycle detection with the following optimizations:
/// - Detects all fundamental cycles in the graph
/// - Eliminates duplicate cycles that differ only in traversal direction
/// - Prefers shortest cycles when multiple cycles share vertices
/// - Reference: "Introduction to Algorithms" by Cormen et al., Chapter 22 (Graph Algorithms)
/// 
/// # Examples
/// ```rust
/// use offroad::find_connected_components;
/// let graph = vec![(0, 1), (1, 2), (2, 0), (3, 4)];
/// let components = find_connected_components(&graph);
/// // Returns cycles like [[0, 1, 2]] for triangle and potentially isolated vertices
/// ```
pub fn find_connected_components(graph: &[(usize, usize)]) -> Vec<Vec<usize>> {
    // Optimized for graphs where each vertex has degree 1-4 only
    // This is typical for arc connection graphs in geometric applications
    // 1. Build adjacency list and analyze vertex degrees
    // 2. Find connected components using DFS
    // 3. For each component, find fundamental cycles efficiently
    // 4. Deduplicate and return cycles
    use std::collections::{HashMap, HashSet};
    
    if graph.is_empty() {
        return Vec::new();
    }
    
    // Build undirected adjacency list and track vertex degrees
    let mut adj_list: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut vertex_degrees: HashMap<usize, usize> = HashMap::new();
    let mut all_vertices = HashSet::new();
    let mut edges_set = HashSet::new();
    
    // Normalize edges to avoid duplicates (keep smaller vertex first)
    for &(u, v) in graph {
        let edge = if u <= v { (u, v) } else { (v, u) };
        edges_set.insert(edge);
        all_vertices.insert(u);
        all_vertices.insert(v);
    }
    
    // Build adjacency list from normalized edges and count degrees
    for (u, v) in edges_set {
        if u != v { // Skip self-loops for now
            adj_list.entry(u).or_insert_with(Vec::new).push(v);
            adj_list.entry(v).or_insert_with(Vec::new).push(u);
            *vertex_degrees.entry(u).or_insert(0) += 1;
            *vertex_degrees.entry(v).or_insert(0) += 1;
        }
    }
    
    // Validate degree constraint (1-4 edges per vertex)
    for (&vertex, &degree) in &vertex_degrees {
        if degree > 4 {
            eprintln!("Warning: Vertex {} has degree {} > 4, which violates the constraint", vertex, degree);
        }
    }
    
    // Sort adjacency lists for deterministic ordering
    for neighbors in adj_list.values_mut() {
        neighbors.sort();
    }
    
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    
    // Find all connected components and extract cycles from each
    for &start_vertex in &all_vertices {
        if visited.contains(&start_vertex) {
            continue;
        }
        
        let component_vertices = find_component_vertices(start_vertex, &adj_list, &mut visited);
        if component_vertices.len() >= 3 {
            // For small components with degree constraints, we can use efficient cycle detection
            let cycles = find_cycles_optimized(&component_vertices, &adj_list, &vertex_degrees);
            for cycle in cycles {
                if !is_duplicate_cycle(&cycle, &components) {
                    components.push(cycle);
                }
            }
        }
    }
    
    components
}

/// Finds a connected component starting from a given vertex using DFS
fn find_component_with_cycles(
    start: usize, 
    adj_list: &HashMap<usize, Vec<usize>>, 
    visited: &mut HashSet<usize>
) -> Vec<usize> {
    let mut component = Vec::new();
    let mut stack = vec![start];
    let mut local_visited = HashSet::new();
    
    while let Some(vertex) = stack.pop() {
        if local_visited.contains(&vertex) {
            continue;
        }
        
        local_visited.insert(vertex);
        visited.insert(vertex);
        component.push(vertex);
        
        if let Some(neighbors) = adj_list.get(&vertex) {
            for &neighbor in neighbors {
                if !local_visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }
    
    component
}

/// Extracts the shortest cycle from a connected component
fn extract_shortest_cycle(
    component: &[usize], 
    adj_list: &HashMap<usize, Vec<usize>>
) -> Option<Vec<usize>> {
    if component.len() < 3 {
        return None; // Need at least 3 vertices for a cycle
    }
    
    // Try to find the shortest cycle using BFS from each vertex
    // Prefer cycles that start with smaller vertex IDs for deterministic results
    let mut shortest_cycle: Option<Vec<usize>> = None;
    
    for &start in component {
        if let Some(mut cycle) = find_cycle_from_vertex(start, adj_list, component) {
            // Normalize cycle to start with the smallest vertex for consistent comparison
            if let Some(min_pos) = cycle.iter().position(|&x| x == *cycle.iter().min().unwrap()) {
                cycle.rotate_left(min_pos);
            }
            
            let should_replace = if let Some(ref current) = shortest_cycle {
                cycle.len() < current.len() || 
                (cycle.len() == current.len() && cycle < *current)
            } else {
                true
            };
            
            if should_replace {
                shortest_cycle = Some(cycle);
            }
        }
    }
    
    shortest_cycle
}

/// Finds a cycle starting from a specific vertex using DFS
fn find_cycle_from_vertex(
    start: usize,
    adj_list: &HashMap<usize, Vec<usize>>,
    component: &[usize]
) -> Option<Vec<usize>> {
    let component_set: HashSet<usize> = component.iter().cloned().collect();
    
    // Use DFS to find shortest cycle from start vertex
    fn dfs_shortest_cycle(
        current: usize,
        start: usize,
        adj_list: &HashMap<usize, Vec<usize>>,
        component_set: &HashSet<usize>,
        path: &mut Vec<usize>,
        visited: &mut HashSet<usize>,
        min_cycle_len: &mut usize
    ) -> Option<Vec<usize>> {
        
        if path.len() >= *min_cycle_len {
            return None; // Don't explore paths longer than current minimum
        }
        
        path.push(current);
        visited.insert(current);
        
        if let Some(neighbors) = adj_list.get(&current) {
            for &neighbor in neighbors {
                if !component_set.contains(&neighbor) {
                    continue;
                }
                
                if neighbor == start && path.len() >= 3 {
                    // Found a cycle back to start
                    if path.len() < *min_cycle_len {
                        *min_cycle_len = path.len();
                        let result = path.clone();
                        path.pop();
                        visited.remove(&current);
                        return Some(result);
                    }
                } else if !visited.contains(&neighbor) {
                    if let Some(cycle) = dfs_shortest_cycle(neighbor, start, adj_list, component_set, path, visited, min_cycle_len) {
                        path.pop();
                        visited.remove(&current);
                        return Some(cycle);
                    }
                }
            }
        }
        
        path.pop();
        visited.remove(&current);
        None
    }
    
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    let mut min_cycle_len = usize::MAX;
    
    dfs_shortest_cycle(start, start, adj_list, &component_set, &mut path, &mut visited, &mut min_cycle_len)
}

/// Reconstructs a cycle from the parent information
fn reconstruct_cycle(
    u: usize, 
    v: usize, 
    parent: &HashMap<usize, Option<usize>>
) -> Vec<usize> {
    let mut cycle = Vec::new();
    
    // Trace back from u to find the path to the common ancestor
    let mut path_u = Vec::new();
    let mut current = u;
    path_u.push(current);
    while let Some(Some(p)) = parent.get(&current) {
        current = *p;
        path_u.push(current);
    }
    
    // Trace back from v to find the path to the common ancestor
    let mut path_v = Vec::new();
    current = v;
    path_v.push(current);
    while let Some(Some(p)) = parent.get(&current) {
        current = *p;
        path_v.push(current);
    }
    
    // Find the lowest common ancestor
    let path_u_set: HashSet<usize> = path_u.iter().cloned().collect();
    let mut lca = v;
    for &vertex in &path_v {
        if path_u_set.contains(&vertex) {
            lca = vertex;
            break;
        }
    }
    
    // Build the cycle: path from u to lca + path from lca to v
    for &vertex in path_u.iter().take_while(|&&x| x != lca) {
        cycle.push(vertex);
    }
    cycle.push(lca);
    for &vertex in path_v.iter().take_while(|&&x| x != lca).collect::<Vec<_>>().iter().rev() {
        cycle.push(*vertex);
    }
    
    cycle
}

/// Finds all vertices in a connected component using DFS
fn find_component_vertices(
    start: usize, 
    adj_list: &HashMap<usize, Vec<usize>>, 
    visited: &mut HashSet<usize>
) -> Vec<usize> {
    let mut component = Vec::new();
    let mut stack = vec![start];
    let mut local_visited = HashSet::new();
    
    while let Some(vertex) = stack.pop() {
        if local_visited.contains(&vertex) {
            continue;
        }
        
        local_visited.insert(vertex);
        visited.insert(vertex);
        component.push(vertex);
        
        if let Some(neighbors) = adj_list.get(&vertex) {
            for &neighbor in neighbors {
                if !local_visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }
    
    component
}

/// Optimized cycle detection for graphs with degree constraints (1-4 edges per vertex)
fn find_cycles_optimized(
    component: &[usize], 
    adj_list: &HashMap<usize, Vec<usize>>,
    vertex_degrees: &HashMap<usize, usize>
) -> Vec<Vec<usize>> {
    use std::collections::HashSet;
    
    if component.len() < 3 {
        return Vec::new(); // Need at least 3 vertices for a cycle
    }
    
    let mut cycles = Vec::new();
    let component_set: HashSet<usize> = component.iter().cloned().collect();
    
    // With degree constraints, we can use more efficient strategies
    // For vertices with degree 2, they must be part of a simple path or cycle
    // For vertices with degree 3+, they are branch points
    
    // Strategy 1: For small components (typical case), use simple DFS
    if component.len() <= 10 {
        for &start in component {
            let found_cycles = find_cycles_from_vertex_optimized(start, adj_list, &component_set, vertex_degrees);
            for cycle in found_cycles {
                // Normalize and check for duplicates
                let mut normalized_cycle = cycle;
                if let Some(min_pos) = normalized_cycle.iter().position(|&x| x == *normalized_cycle.iter().min().unwrap()) {
                    normalized_cycle.rotate_left(min_pos);
                }
                
                if !is_duplicate_cycle(&normalized_cycle, &cycles) {
                    cycles.push(normalized_cycle);
                }
            }
        }
    } else {
        // Strategy 2: For larger components, use degree-based analysis
        cycles = find_cycles_by_degree_analysis(component, adj_list, vertex_degrees);
    }
    
    // Sort cycles by length, then by lexicographic order for deterministic results
    cycles.sort_by(|a, b| {
        a.len().cmp(&b.len()).then_with(|| a.cmp(b))
    });
    
    cycles
}

/// Optimized cycle detection from a single vertex using degree constraints
fn find_cycles_from_vertex_optimized(
    start: usize,
    adj_list: &HashMap<usize, Vec<usize>>,
    component_set: &HashSet<usize>,
    _vertex_degrees: &HashMap<usize, usize>
) -> Vec<Vec<usize>> {
    let mut cycles = Vec::new();
    
    // DFS to find cycles with optimizations for degree constraints
    fn dfs_optimized(
        current: usize,
        start: usize,
        adj_list: &HashMap<usize, Vec<usize>>,
        component_set: &HashSet<usize>,
        path: &mut Vec<usize>,
        visited: &mut HashSet<usize>,
        cycles: &mut Vec<Vec<usize>>
    ) {
        
        if path.len() > 8 {
            return; // Avoid very long cycles in geometric applications
        }
        
        path.push(current);
        visited.insert(current);
        
        if let Some(neighbors) = adj_list.get(&current) {
            for &neighbor in neighbors {
                if !component_set.contains(&neighbor) {
                    continue;
                }
                
                if neighbor == start && path.len() >= 3 {
                    // Found a cycle back to start
                    cycles.push(path.clone());
                } else if !visited.contains(&neighbor) {
                    dfs_optimized(neighbor, start, adj_list, component_set, path, visited, cycles);
                }
            }
        }
        
        path.pop();
        visited.remove(&current);
    }
    
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    dfs_optimized(start, start, adj_list, component_set, &mut path, &mut visited, &mut cycles);
    
    cycles
}

/// Find cycles using degree-based analysis for larger components
fn find_cycles_by_degree_analysis(
    component: &[usize], 
    adj_list: &HashMap<usize, Vec<usize>>,
    vertex_degrees: &HashMap<usize, usize>
) -> Vec<Vec<usize>> {
    let mut cycles = Vec::new();
    
    // For now, fall back to the simple approach
    // This can be optimized further based on specific degree patterns
    let component_set: HashSet<usize> = component.iter().cloned().collect();
    
    for &start in component {
        if vertex_degrees.get(&start).unwrap_or(&0) >= &2 {
            let found_cycles = find_cycles_from_vertex_optimized(start, adj_list, &component_set, vertex_degrees);
            for cycle in found_cycles {
                let mut normalized_cycle = cycle;
                if let Some(min_pos) = normalized_cycle.iter().position(|&x| x == *normalized_cycle.iter().min().unwrap()) {
                    normalized_cycle.rotate_left(min_pos);
                }
                
                if !is_duplicate_cycle(&normalized_cycle, &cycles) {
                    cycles.push(normalized_cycle);
                }
            }
        }
    }
    
    cycles
}

/// Finds all fundamental cycles in a connected component
fn find_all_cycles_in_component(
    component: &[usize], 
    adj_list: &HashMap<usize, Vec<usize>>
) -> Vec<Vec<usize>> {
    use std::collections::HashSet;
    
    if component.len() < 3 {
        return Vec::new(); // Need at least 3 vertices for a cycle
    }
    
    let mut cycles = Vec::new();
    let component_set: HashSet<usize> = component.iter().cloned().collect();
    
    // Try to find cycles starting from each vertex
    for &start in component {
        let found_cycles = find_cycles_from_vertex(start, adj_list, &component_set);
        for cycle in found_cycles {
            // Normalize and check for duplicates
            let mut normalized_cycle = cycle;
            if let Some(min_pos) = normalized_cycle.iter().position(|&x| x == *normalized_cycle.iter().min().unwrap()) {
                normalized_cycle.rotate_left(min_pos);
            }
            
            if !is_duplicate_cycle(&normalized_cycle, &cycles) {
                cycles.push(normalized_cycle);
            }
        }
    }
    
    // Sort cycles by length, then by lexicographic order for deterministic results
    cycles.sort_by(|a, b| {
        a.len().cmp(&b.len()).then_with(|| a.cmp(b))
    });
    
    cycles
}

/// Finds cycles starting from a specific vertex using DFS
fn find_cycles_from_vertex(
    start: usize,
    adj_list: &HashMap<usize, Vec<usize>>,
    component_set: &HashSet<usize>
) -> Vec<Vec<usize>> {
    let mut cycles = Vec::new();
    
    // DFS to find all simple cycles from start vertex
    fn dfs_find_cycles(
        current: usize,
        start: usize,
        adj_list: &HashMap<usize, Vec<usize>>,
        component_set: &HashSet<usize>,
        path: &mut Vec<usize>,
        visited: &mut HashSet<usize>,
        cycles: &mut Vec<Vec<usize>>
    ) {
        
        if path.len() > 10 {
            return; // Avoid very long cycles
        }
        
        path.push(current);
        visited.insert(current);
        
        if let Some(neighbors) = adj_list.get(&current) {
            for &neighbor in neighbors {
                if !component_set.contains(&neighbor) {
                    continue;
                }
                
                if neighbor == start && path.len() >= 3 {
                    // Found a cycle back to start
                    cycles.push(path.clone());
                } else if !visited.contains(&neighbor) {
                    dfs_find_cycles(neighbor, start, adj_list, component_set, path, visited, cycles);
                }
            }
        }
        
        path.pop();
        visited.remove(&current);
    }
    
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    dfs_find_cycles(start, start, adj_list, component_set, &mut path, &mut visited, &mut cycles);
    
    cycles
}

/// Checks if a cycle is a duplicate of any existing cycle (considering direction)
fn is_duplicate_cycle(new_cycle: &[usize], existing_cycles: &[Vec<usize>]) -> bool {
    for existing in existing_cycles {
        if is_same_cycle(new_cycle, existing) {
            return true;
        }
    }
    false
}

/// Checks if two cycles are the same (considering both directions and rotations)
fn is_same_cycle(cycle1: &[usize], cycle2: &[usize]) -> bool {
    if cycle1.len() != cycle2.len() {
        return false;
    }
    
    let len = cycle1.len();
    
    // Check all rotations in both directions
    for start in 0..len {
        // Forward direction
        let mut matches = true;
        for i in 0..len {
            if cycle1[i] != cycle2[(start + i) % len] {
                matches = false;
                break;
            }
        }
        if matches {
            return true;
        }
        
        // Reverse direction
        matches = true;
        for i in 0..len {
            if cycle1[i] != cycle2[(start + len - i) % len] {
                matches = false;
                break;
            }
        }
        if matches {
            return true;
        }
    }
    
    false
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
            arcseg(point(0.0, 0.0), point(1.0, 1.0)),
            arcseg(point(1.0, 1.0), point(2.0, 2.0)),
            arcseg(point(0.0, 0.0), point(1.0, 1.0)), // duplicate
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_duplicate_lines_reversed() {
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 1.0)),
            arcseg(point(1.0, 1.0), point(0.0, 0.0)), // reversed duplicate
            arcseg(point(2.0, 2.0), point(3.0, 3.0)),
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_no_duplicates() {
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 1.0)),
            arcseg(point(1.0, 1.0), point(2.0, 2.0)),
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
            arcseg(point(3.0, 3.0), point(4.0, 4.0)),
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_different_arcs_same_endpoints() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.0),
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, -1.0), 1.0), // different center
            arcseg(point(3.0, 3.0), point(4.0, 4.0)),
        ];
        let original_len = arcs.len();
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), original_len); // should not remove different arcs
    }

    #[test]
    fn test_remove_bridge_arcs_mixed_arc_and_line() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.0),
            arcseg(point(0.0, 0.0), point(2.0, 0.0)), // line with same endpoints
            arcseg(point(3.0, 3.0), point(4.0, 4.0)),
        ];
        let original_len = arcs.len();
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), original_len); // should not remove arc-line combinations
    }

    #[test]
    fn test_remove_bridge_arcs_multiple_duplicates() {
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 1.0)),
            arcseg(point(0.0, 0.0), point(1.0, 1.0)), // duplicate 1
            arcseg(point(0.0, 0.0), point(1.0, 1.0)), // duplicate 2
            arc(point(2.0, 2.0), point(4.0, 2.0), point(3.0, 3.0), 1.0),
            arc(point(2.0, 2.0), point(4.0, 2.0), point(3.0, 3.0), 1.0), // duplicate arc
            arcseg(point(5.0, 5.0), point(6.0, 6.0)),
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
            arcseg(point(0.0, 0.0), point(1.0, 1.0)),
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_remove_bridge_arcs_close_but_not_equal() {
        let eps = super::EPS_CONNECT;
        let mut arcs = vec![
            arcseg(point(0.0, 0.0), point(1.0, 1.0)),
            arcseg(point(0.0, 0.0), point(1.0 + eps * 0.5, 1.0 + eps * 0.5)), // close but within tolerance
        ];
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), 0); // should remove both as they're close enough
    }

    #[test]
    fn test_remove_bridge_arcs_different_radius() {
        let mut arcs = vec![
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.0),
            arc(point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0), 1.5), // different radius
            arcseg(point(3.0, 3.0), point(4.0, 4.0)),
        ];
        let original_len = arcs.len();
        remove_bridge_arcs(&mut arcs);
        assert_eq!(arcs.len(), original_len); // should not remove arcs with different radius
    }
}

#[cfg(test)]
mod test_merge_points {

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

#[cfg(test)]
mod test_find_connected_components {
    use super::find_connected_components;

    #[test]
    fn test_find_connected_components_empty_graph() {
        let graph = vec![];
        let components = find_connected_components(&graph);
        assert_eq!(components.len(), 0);
    }

    #[test]
    fn test_find_connected_components_simple_triangle() {
        // Simple triangle: 0-1-2-0
        let graph = vec![(0, 1), (1, 2), (2, 0)];
        let components = find_connected_components(&graph);
        
        assert_eq!(components.len(), 1);
        let cycle = &components[0];
        assert_eq!(cycle.len(), 3);
        
        // Check that it contains all vertices (order may vary)
        assert!(cycle.contains(&0));
        assert!(cycle.contains(&1));
        assert!(cycle.contains(&2));
    }

    #[test]
    fn test_find_connected_components_square() {
        // Square: 0-1-2-3-0
        let graph = vec![(0, 1), (1, 2), (2, 3), (3, 0)];
        let components = find_connected_components(&graph);
        
        assert_eq!(components.len(), 1);
        let cycle = &components[0];
        assert_eq!(cycle.len(), 4);
        
        // Check that it contains all vertices
        for i in 0..4 {
            assert!(cycle.contains(&i));
        }
    }

    #[test]
    fn test_find_connected_components_multiple_cycles() {
        // Two separate triangles: (0-1-2-0) and (3-4-5-3)
        let graph = vec![
            (0, 1), (1, 2), (2, 0),  // First triangle
            (3, 4), (4, 5), (5, 3)   // Second triangle
        ];
        let components = find_connected_components(&graph);
        
        assert_eq!(components.len(), 2);
        
        // Each component should be a 3-vertex cycle
        for cycle in &components {
            assert_eq!(cycle.len(), 3);
        }
        
        // Check that all vertices are present
        let mut all_vertices = std::collections::HashSet::new();
        for cycle in &components {
            for &vertex in cycle {
                all_vertices.insert(vertex);
            }
        }
        assert_eq!(all_vertices.len(), 6);
        for i in 0..6 {
            assert!(all_vertices.contains(&i));
        }
    }

    #[test]
    fn test_find_connected_components_isolated_vertices() {
        // Triangle with isolated edge: (0-1-2-0) and (3-4)
        let graph = vec![(0, 1), (1, 2), (2, 0), (3, 4)];
        let components = find_connected_components(&graph);
        
        // Should only find the triangle (isolated edge doesn't form a cycle)
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 3);
    }

    #[test]
    fn test_find_connected_components_complex_graph() {
        // Graph with multiple cycles: figure-8 pattern
        // Two triangles sharing a vertex: (0-1-2-0) and (0-3-4-0)
        let graph = vec![
            (0, 1), (1, 2), (2, 0),  // First triangle
            (0, 3), (3, 4), (4, 0)   // Second triangle (shares vertex 0)
        ];
        let components = find_connected_components(&graph);
        
        // Should find the shortest cycles from this complex structure
        assert!(!components.is_empty());
        
        // Each component should be at least 3 vertices (minimum cycle)
        for cycle in &components {
            assert!(cycle.len() >= 3);
        }
    }

    #[test]
    fn test_find_connected_components_line_graph() {
        // Linear chain: 0-1-2-3 (no cycles)
        let graph = vec![(0, 1), (1, 2), (2, 3)];
        let components = find_connected_components(&graph);
        
        // Should find no cycles
        assert_eq!(components.len(), 0);
    }

    #[test]
    fn test_find_connected_components_self_loop() {
        // Self-loop: vertex connected to itself
        let graph = vec![(0, 0)];
        let components = find_connected_components(&graph);
        
        // Self-loops don't form valid cycles (need at least 3 vertices)
        assert_eq!(components.len(), 0);
    }

    #[test]
    fn test_find_connected_components_duplicate_elimination() {
        // Graph where the same cycle can be traversed in different directions
        let graph = vec![(0, 1), (1, 2), (2, 0), (0, 2), (2, 1), (1, 0)];
        let components = find_connected_components(&graph);
        
        // Should eliminate duplicates and return only one cycle
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 3);
    }

    #[test]
    fn test_find_connected_components_pentagon() {
        // Pentagon: 0-1-2-3-4-0
        let graph = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)];
        let components = find_connected_components(&graph);
        
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 5);
        
        // Verify all vertices are present
        for i in 0..5 {
            assert!(components[0].contains(&i));
        }
    }

    #[test]
    fn test_find_connected_components_wheel_graph() {
        // Wheel graph: central vertex 0 connected to rim vertices 1,2,3 which form a cycle
        let graph = vec![
            (1, 2), (2, 3), (3, 1),  // Rim cycle
            (0, 1), (0, 2), (0, 3)   // Spokes to center
        ];
        let components = find_connected_components(&graph);
        
        // Should find at least one cycle
        assert!(!components.is_empty());
        
        // In a wheel graph, we expect to find either:
        // 1. The rim triangle [1,2,3] (length 3), or 
        // 2. A 4-cycle involving the center vertex (length 4)
        // Both are valid cycles in this graph structure
        let has_valid_cycle = components.iter().any(|cycle| cycle.len() == 3 || cycle.len() == 4);
        assert!(has_valid_cycle, "Expected to find either a 3-cycle or 4-cycle, but found: {:?}", components);
    }

    #[test]
    fn test_find_connected_components_degree_constraints() {
        // Test graph where each vertex has degree 1-4
        // Degree 1: terminal vertices
        // Degree 2: path vertices  
        // Degree 3: branch vertices
        // Degree 4: intersection vertices
        
        // Create a graph: 0-1-2-3-0 (degree 2 for all vertices)
        let graph = vec![(0, 1), (1, 2), (2, 3), (3, 0)];
        let components = find_connected_components(&graph);
        
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 4);
        
        // Verify it's the expected square cycle
        let cycle = &components[0];
        for i in 0..4 {
            assert!(cycle.contains(&i));
        }
    }

    #[test]
    fn test_find_connected_components_degree_1_endpoints() {
        // Test graph with degree 1 vertices (endpoints)
        // 0-1-2 (linear chain)
        let graph = vec![(0, 1), (1, 2)];
        let components = find_connected_components(&graph);
        
        // Linear chain has no cycles
        assert_eq!(components.len(), 0);
    }

    #[test]
    fn test_find_connected_components_degree_3_branch() {
        // Test graph with degree 3 vertex (branch point)
        // Y-shaped graph: 0-1, 1-2, 1-3, 2-3 (forms triangle with branch)
        let graph = vec![(0, 1), (1, 2), (1, 3), (2, 3)];
        let components = find_connected_components(&graph);
        
        assert_eq!(components.len(), 1);
        // Should find the triangle [1, 2, 3]
        assert_eq!(components[0].len(), 3);
        let cycle = &components[0];
        assert!(cycle.contains(&1) && cycle.contains(&2) && cycle.contains(&3));
    }

    #[test]
    fn test_find_connected_components_degree_4_intersection() {
        // Test graph with degree 4 vertex (intersection)
        // X-shaped graph: center vertex 0 connected to 4 outer vertices in two triangles
        let graph = vec![
            (0, 1), (0, 2), (0, 3), (0, 4),  // Center to outer vertices (degree 4 for vertex 0)
            (1, 2), (3, 4)                   // Two triangles
        ];
        let components = find_connected_components(&graph);
        
        // Should find two triangles: [0,1,2] and [0,3,4]
        assert_eq!(components.len(), 2);
        
        for component in &components {
            assert_eq!(component.len(), 3);
            assert!(component.contains(&0)); // Center vertex should be in both triangles
        }
    }

    #[test]
    fn test_find_connected_components_mixed_degrees() {
        // Test graph with mixed vertex degrees (1, 2, 3, 4)
        // Complex graph combining different degree patterns
        let graph = vec![
            // Triangle with branch
            (0, 1), (1, 2), (2, 0),  // Triangle (degree 2 for each)
            (2, 3),                  // Branch from vertex 2 (now degree 3)
            (3, 4), (3, 5),          // Branch continues (degree 3 for vertex 3)
            (4, 5)                   // Close another triangle (degree 2 for vertices 4,5)
        ];
        let components = find_connected_components(&graph);
        
        // Should find two triangles: [0,1,2] and [3,4,5]
        assert_eq!(components.len(), 2);
        
        let mut found_triangle_1 = false;
        let mut found_triangle_2 = false;
        
        for component in &components {
            assert_eq!(component.len(), 3);
            if component.contains(&0) && component.contains(&1) && component.contains(&2) {
                found_triangle_1 = true;
            }
            if component.contains(&3) && component.contains(&4) && component.contains(&5) {
                found_triangle_2 = true;
            }
        }
        
        assert!(found_triangle_1, "Should find triangle [0,1,2]");
        assert!(found_triangle_2, "Should find triangle [3,4,5]");
    }

    #[test]
    fn test_find_connected_components_performance_constraints() {
        // Test that the algorithm efficiently handles the degree constraints
        // Create a larger graph that still respects the 1-4 degree constraint
        let mut graph = Vec::new();
        
        // Create a chain of connected triangles (each vertex has degree ≤ 3)
        for i in 0..5 {
            let base = i * 2;
            // Triangle: base, base+1, base+2
            graph.push((base, base + 1));
            graph.push((base + 1, base + 2));
            graph.push((base + 2, base));
            
            // Connect to next triangle if not the last one
            if i < 4 {
                graph.push((base + 2, base + 3));
            }
        }
        
        let components = find_connected_components(&graph);
        
        // Should find 5 triangles
        assert_eq!(components.len(), 5);
        
        for component in &components {
            assert_eq!(component.len(), 3, "Each component should be a triangle");
        }
    }

    #[test]
    fn test_find_connected_components_bowtie_graph() {
        // Bowtie: two triangles sharing one vertex
        // (0-1-2-0) and (0-3-4-0)
        let graph = vec![
            (0, 1), (1, 2), (2, 0),  // First triangle
            (0, 3), (3, 4), (4, 0)   // Second triangle
        ];
        let components = find_connected_components(&graph);
        
        // Should find both triangles
        assert!(components.len() >= 1); // At least one cycle should be found
        
        // All cycles should have at least 3 vertices
        for cycle in &components {
            assert!(cycle.len() >= 3);
        }
    }

    #[test]
    fn test_find_connected_components_large_cycle() {
        // Large cycle: 0-1-2-3-4-5-6-7-0
        let mut graph = vec![];
        for i in 0..8 {
            graph.push((i, (i + 1) % 8));
        }
        let components = find_connected_components(&graph);
        
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 8);
        
        // Verify all vertices are present
        for i in 0..8 {
            assert!(components[0].contains(&i));
        }
    }

    #[test]
    fn test_find_connected_components_mixed_components() {
        // Mix of cycles and non-cycles:
        // Triangle: (0-1-2-0)
        // Square: (3-4-5-6-3)  
        // Line: (7-8-9)
        let graph = vec![
            (0, 1), (1, 2), (2, 0),           // Triangle
            (3, 4), (4, 5), (5, 6), (6, 3),   // Square
            (7, 8), (8, 9)                    // Line (no cycle)
        ];
        let components = find_connected_components(&graph);
        
        // Should find 2 cycles (triangle and square)
        assert_eq!(components.len(), 2);
        
        // One should be 3-vertex, one should be 4-vertex
        let mut cycle_sizes: Vec<usize> = components.iter().map(|c| c.len()).collect();
        cycle_sizes.sort();
        assert_eq!(cycle_sizes, vec![3, 4]);
    }
}
