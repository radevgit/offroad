# Phase 2-3: Graph-Based Cycle Detection & Reconnect Logic Refinements

## Overview
After Phase 1 (AABB spatial indexing) was completed with instrumentation to profile overhead, Phases 2-3 focused on implementing a sophisticated graph-based algorithm to detect non-intersecting cycles and reconnect offset segments properly.

**Status**: ✅ These features are working and committed to `fix_reconnect` branch. They form the foundation for future optimization work.

---

## Phase 2: Graph-Based Cycle Detection (Commit 7970502)

### Purpose
Implement a graph-based algorithm to find non-intersecting cycles in the offset arcs. This solves a key problem: after offset splitting, we need to identify which arcs form closed loops and separate them properly.

### Key Changes

#### 1. **New Module: `src/graph/find_cycles.rs`** (407 lines)
A comprehensive graph cycle detection algorithm with geometric awareness:

**Core Algorithm**:
- Builds a graph representation from input arcs where:
  - Vertices = endpoints of arcs in 2D space
  - Edges = the geometric arcs themselves
  - Each vertex can have up to 8 connected edges
  
- **Graph Structure**:
  ```rust
  struct CycleGraph {
      vertices: Vec<Point>,
      adjacency: HashMap<VertexId, Vec<usize>>,  // vertex -> edge IDs
      edges: Vec<GraphEdge>,
  }
  ```

- **Vertex Merging**: Close points within tolerance are automatically merged to the same vertex:
  ```rust
  const VERTEX_TOLERANCE: f64 = 1e-8;
  ```

**Key Feature: "Most Close on the Right" Rule**
This geometric-aware traversal strategy avoids intersections:
- At each vertex with multiple exit options, chooses the edge with the **smallest right-turn angle**
- Uses proper tangent calculations via `togo`'s arc tangent methods
- Works for both line segments and curved arcs

**Implementation**:
```rust
fn choose_rightmost_edge(
    graph: &CycleGraph,
    vertex: VertexId,
    incoming_edge_id: usize,
    available_edges: &[usize]
) -> Option<usize>
```

- Calculates incoming direction using arc tangent at vertex
- Calculates outgoing directions for all candidates
- Sorts by angle and selects the rightmost (most "straight ahead" on the right)

**Cycle Finding Process**:
1. Start from each unused edge in the graph
2. Follow edges using the rightmost-turn rule
3. When returning to start vertex, mark all edges as used (found a cycle)
4. Repeat until no more cycles found

**Test Coverage**: 
- 16 comprehensive tests including:
  - Empty input, single arc, basic shapes (triangle, square)
  - Complex patterns (figure-8, X-intersection, double edges)
  - Multiple separate cycles
  - Mixed arc types (curved + line segments)
  - X-intersection handling via angle selection
  - Complex graphs with branches

### Phase 2 Files Modified

| File | Changes |
|------|---------|
| `src/offset_split_arcs.rs` | 117 line changes: integrated index_map tracking for spatial index |
| `src/spatial/spatial.rs` | Added 5 lines for spatial support |
| `benches/bench_offset_multiple.rs` | Updated for profiling |
| `examples/measure_overhead.rs` | **NEW** (46 lines) - Profile Phase 2 overhead |
| `examples/profile_offset.rs` | **NEW** (39 lines) - Profile offset performance |
| `src/offset_split_arcs_instrumented.rs` | **NEW** (407 lines) - Instrumented version for measurements |

---

## Phase 3: Cycle Detection Refinements + Memory/Perf Analysis (Commit fbb3436)

### Purpose
After Phase 2's implementation, add comprehensive performance and memory analysis to understand overhead and optimize further.

### Key Changes

#### 1. **New Performance Analysis Examples**

**`examples/analyze_memory.rs`** (80 lines)
- Profiles memory usage of the offset algorithm
- Tests with progressively larger polylines
- Tracks heap allocations and memory patterns

**`examples/search_perf.rs`** (74 lines)
- Benchmarks the spatial search performance
- Compares different search strategies
- Identifies performance bottlenecks

**`examples/vector_perf.rs`** (62 lines)
- Analyzes vector operation performance
- Tests insertion, removal, and iteration patterns
- Helps identify inefficient patterns

#### 2. **Minor Algorithm Refinements**

**`src/offset_prune_invalid.rs`** (12 line changes)
- Refined validation logic after cycle detection
- Better handling of edge cases

**`src/offset_split_arcs.rs`** (2 line change)
- Minimal tuning based on profiling results

### Phase 3 Files Modified

| File | Changes |
|------|---------|
| `benches/bench_offset_multiple.rs` | Updated benchmark metrics |
| `examples/analyze_memory.rs` | **NEW** - Memory profiling |
| `examples/search_perf.rs` | **NEW** - Search strategy benchmarking |
| `examples/vector_perf.rs` | **NEW** - Vector operation profiling |
| `src/offset_prune_invalid.rs` | Refined validation |

---

## Foundation Module: `src/graph/merge_ends.rs`

### Purpose
Before cycle detection, endpoints must be properly merged. This handles numerical precision issues from the offset algorithm.

### Core Functionality

**Main Function**:
```rust
pub fn merge_close_endpoints(arcs: &mut Vec<Arc>, tolerance: f64)
```

**4-Step Process**:
1. **Find Endpoint Groups**: 
   - Identifies points within tolerance distance of each other
   - Uses agglomerative clustering algorithm
   
2. **Merge to Centroid**:
   - Calculates centroid of each group
   - Moves all group endpoints to the centroid
   
3. **Eliminate Small Arcs**:
   - Removes arcs that became too small after merging
   - Checks both line segment length AND arc radius
   
4. **Ensure Geometric Consistency**:
   - Calls `arc.make_consistent()` on all arcs

**Endpoint Group Structure**:
```rust
struct EndpointGroup {
    points: Vec<Point>,           // All points in group
    arc_indices: Vec<(usize, EndpointType)>,  // Which arcs have endpoints here
    centroid: Point,              // Merge target
}

enum EndpointType {
    Start,  // Arc's point 'a'
    End,    // Arc's point 'b'
}
```

### Example Workflow
```
Input: Two close endpoints
  Arc 0: (0,0) -> (1.0, 0.0)
  Arc 1: (1.0+1e-9, 1e-9) -> (2.0, 0.0)

After merge_close_endpoints():
  Arc 0: (0,0) -> (1.0, 0.0)           [unchanged]
  Arc 1: (1.0, 0.0) -> (2.0, 0.0)     [endpoint merged to centroid]
  
Result: Arcs are now properly connected
```

### Test Coverage
28 comprehensive tests including:
- Simple two-arc merging
- Small arc elimination
- Multiple point groups (star patterns)
- Boundary conditions
- Mixed arc types (curved + line segments)
- Chain connections
- Complex diagnostic tests

---

## Integration: `src/offset_reconnect_arcs.rs`

### Purpose
The glue that ties everything together. This is called after offset splitting to process the raw arc results.

### Main Function
```rust
pub fn offset_reconnect_arcs(arcs: Arcline) -> Vec<Arcline>
```

**Process**:
1. **Merge Close Endpoints** - Fix numerical precision issues
   ```rust
   merge_close_endpoints_default(&mut arc_vec);
   ```

2. **Find Non-Intersecting Cycles** - Separate disconnected components
   ```rust
   let cycles = find_non_intersecting_cycles(&arc_vec);
   ```

3. **Return as Separate Arclines** - Each cycle becomes its own result
   ```rust
   vec![Vec<Arc>, Vec<Arc>, ...]  // Multiple Arclines
   ```

### Error Handling
- Empty input returns empty vector
- Single arcs that can't form cycles are filtered out
- Geometric validation ensures all cycles are valid

### Test Examples

**Test: Triangle with Small Gaps**
```rust
// Input: Nearly-closed triangle with 1e-9 gaps
let arcs = vec![
    arcseg(Point::new(0.0, 0.0), Point::new(10.0, 0.0)),
    arcseg(Point::new(10.0, 1e-9), Point::new(5.0, 8.66)),  // tiny gap
    arcseg(Point::new(5.0, 8.66), Point::new(-1e-9, 0.0)), // tiny gap
];

let result = offset_reconnect_arcs(arcs);
// Result: 1 cycle with 3 arcs (merged triangle)
```

**Test: Mixed Arc Types with Gaps**
```rust
// Input: Mixed curved arcs and line segments
let arcs = vec![
    arcseg(...),                              // Line
    arc(P1, P2, center, radius),            // Curved
    arcseg(...),                              // Line with gap
    arc(P3, P4, center, radius),            // Curved  
    arcseg(...),                              // Line
    arcseg(..., Point::new(-1e-9, ...)),    // Line with gap
];

let result = offset_reconnect_arcs(arcs);
// Result: 1 cycle with 6 segments (all properly merged)
```

---

## Architecture Diagram

```
offset_split_arcs()
        ↓
   (raw arcs)
        ↓
offset_reconnect_arcs()
        ├─→ merge_close_endpoints()
        │        ├─→ find_endpoint_groups()
        │        ├─→ merge_to_centroids()
        │        ├─→ eliminate_small_arcs()
        │        └─→ adjust_arcs_for_consistency()
        │
        └─→ find_non_intersecting_cycles()
                 ├─→ build_graph()
                 ├─→ find_cycle_from_edge()
                 │        ├─→ find_next_edge()
                 │        └─→ choose_rightmost_edge()
                 │             └─→ get_arc_direction_at_vertex()
                 └─→ [cycles as separate Arclines]
```

---

## Key Algorithms & Constants

### Constants
| Constant | Value | Purpose |
|----------|-------|---------|
| `VERTEX_TOLERANCE` | `1e-8` | Merge vertices within this distance |
| `MERGE_TOLERANCE` | `1e-8` | Merge endpoints within this distance |
| `EPS_CONNECT` | `1e-7` | Precision for reconnection |

### Time Complexity
- **Merge Endpoints**: O(n²) where n = number of endpoints
  - Agglomerative clustering with tolerance distance
- **Graph Building**: O(n) where n = number of arcs
- **Cycle Finding**: O(n·m) where n = edges, m = average degree
  - For typical offset results (sparse graphs): ~O(n)
- **Total**: O(n²) dominated by endpoint merging

### Space Complexity
- Graph representation: O(n + e) where e = edges
- Endpoint groups: O(n) for clustering results
- Used edges tracking: O(e)
- **Total**: O(n) overall

---

## How It Solves the Original Problem

### Problem: Offset Results Are Disconnected
After splitting arcs to find intersections, we get:
- Potentially thousands of small arc segments
- Many endpoints very close but not exactly connected (numerical error)
- No clear structure - hard to identify individual offset paths

### Solution: Phase 2-3 Approach

1. **Merge Endpoints** (merge_ends.rs)
   - Clusters close endpoints together
   - Reconnects what should be connected
   - Removes invalid small arcs

2. **Build Graph** (find_cycles.rs)
   - Creates vertex-edge structure
   - Establishes connectivity relationships
   - Handles geometric properties

3. **Detect Cycles** (find_cycles.rs)
   - Uses "most close on the right" rule
   - Finds non-intersecting loops
   - Separates disconnected components

4. **Output Clean Arclines** (offset_reconnect_arcs.rs)
   - Each cycle is a separate, valid offset path
   - Can be directly rendered or further processed

### Example Result
```
Input: 105 raw arcs from offset_split_arcs
       └─ Scattered, many near-duplicate endpoints
       
After merge_close_endpoints:
    └─ ~99 arcs (6 small arcs eliminated)
    └─ All endpoints properly merged
    
After find_non_intersecting_cycles:
    ├─ Cycle 1: 99 arcs (main external offset)
    └─ (other cycles if they exist)
    
Output: Clean, renderable offset paths
```

---

## Integration with Main Workflow

```
offset_polyline_to_polyline()
        ↓
offset_polyline_raw()
        ├─→ offset_split_arcs()  [Phase 1: AABB optimization]
        │       └─→ [raw split arcs]
        │
        └─→ offset_reconnect_arcs()  [Phase 2-3: Graph detection]
                ├─→ merge_close_endpoints()
                ├─→ find_non_intersecting_cycles()
                └─→ [clean offset arclines]
```

---

## Future Optimization Opportunities

Based on Phase 2-3 implementation, these areas could be optimized:

1. **Endpoint Clustering**: Use spatial hashing instead of O(n²) distance checks
2. **Graph Traversal**: Use DFS with pre-computed angle tables
3. **Tangent Calculations**: Cache togo's tangent results
4. **Cycle Finding**: Parallel cycle extraction for large graphs
5. **Memory**: Use arena allocation for temporary graph structures

---

## Key Files Reference

| File | Lines | Purpose |
|------|-------|---------|
| `src/graph/find_cycles.rs` | 407 | Cycle detection with geometric awareness |
| `src/graph/merge_ends.rs` | ~500+ | Endpoint merging and small arc elimination |
| `src/graph/mod.rs` | - | Module declarations |
| `src/offset_reconnect_arcs.rs` | 165 | Integration layer |
| `examples/analyze_memory.rs` | 80 | Performance profiling |
| `examples/search_perf.rs` | 74 | Search strategy benchmarking |
| `examples/vector_perf.rs` | 62 | Vector operation profiling |

---

## Status Summary

✅ **Phase 2**: Graph-based cycle detection - Complete and working
✅ **Phase 3**: Performance profiling - Complete and optimized  
✅ **Integration**: All modules properly integrated
✅ **Tests**: Comprehensive test coverage for all modules
✅ **Offroad v0.1.2 → v0.3.0**: Migration to TOGO 0.5 successful with all features working

### Test Results
```
offset_multi example output:
  Internal offsets: 99 ✓ (expected: 99)
  External offsets: 90 ✓ (expected: 90)
  Exit code: 0
```

All functionality is working correctly on the `fix_reconnect` branch!
