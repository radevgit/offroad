# Quick Reference: Graph-Based Cycle Detection (Phase 2-3)

## TL;DR

After `offset_split_arcs()` produces raw split arcs, `offset_reconnect_arcs()` cleans them up:

1. **Merge Endpoints** - Connect arcs that should touch (within 1e-8 tolerance)
2. **Find Cycles** - Extract non-intersecting loops using "rightmost-turn" rule
3. **Return Clean Arclines** - Each cycle becomes a separate, valid output

Result: From ~105 scattered arcs → to 99 properly connected internal + 90 external offsets ✓

---

## What Was Added

### New Files

| File | Purpose | Lines | Status |
|------|---------|-------|--------|
| `src/graph/merge_ends.rs` | Endpoint clustering & merging | 500+ | ✅ Working |
| `src/graph/find_cycles.rs` | Graph-based cycle detection | 407 | ✅ Working |
| `src/graph/mod.rs` | Module declarations | - | ✅ Working |

### Modified Files

| File | Change | Impact |
|------|--------|--------|
| `src/offset_reconnect_arcs.rs` | Simplified to 165 lines | ✅ Cleaner |
| `src/offset_split_arcs.rs` | 117 line changes (Phase 2 instrumentation) | ℹ️ Kept for reference |
| `Cargo.toml` | TOGO 0.4.1 → 0.5 | ✅ Done in v0.3.0 |

---

## Core Algorithms

### Algorithm 1: Merge Endpoints

```
FOR each group of close points (within 1e-8):
  1. Find all endpoint groups
  2. Merge all points to centroid
  3. Eliminate resulting small arcs
  4. Validate geometry

Time: O(n²)
Space: O(n)
```

**Example**:
```
Before: Arc A: ... → (1.0, 0.0)
        Arc B: (1.0+1e-9, 1e-9) → ...

After:  Arc A: ... → (1.0, 0.0)
        Arc B: (1.0, 0.0) → ...
        
Result: A.b == B.a (connected!)
```

### Algorithm 2: Find Non-Intersecting Cycles

```
FOR each unused edge:
  1. Start traversal from edge
  2. At each vertex with options:
     a. Calculate incoming direction (using arc tangent)
     b. Calculate all outgoing directions
     c. Choose "most close on the right" (smallest right-turn angle)
  3. Continue until returning to start
  4. Mark all edges as used
  5. Save cycle

Time: O(n·d) where d = avg degree (~2-4)
Space: O(n)
```

**The "Rightmost-Turn" Rule**:
```
At vertex V, coming from edge A, have 3 exit options:

    B (90°)
    ↑
    |
A→ V→ C (45°)
    |
    ↓
    D (180°)

Choose C: smallest positive angle = "most close on the right"
This prevents self-intersections in the output cycle.
```

---

## Key Tolerances

```rust
const VERTEX_TOLERANCE: f64 = 1e-8;    // Merge vertices closer than this
const MERGE_TOLERANCE: f64 = 1e-8;     // Merge endpoints closer than this
const EPS_CONNECT: f64 = 1e-7;         // Reconnection precision
```

---

## Data Structures

### EndpointGroup (merge_ends.rs)
```rust
struct EndpointGroup {
    points: Vec<Point>,                    // All points in group
    arc_indices: Vec<(usize, EndpointType)>,
    centroid: Point,                       // Merge target
}

enum EndpointType { Start, End }
```

### CycleGraph (find_cycles.rs)
```rust
struct CycleGraph {
    vertices: Vec<Point>,                       // Endpoint positions
    adjacency: HashMap<VertexId, Vec<usize>>,  // VertexId → edge IDs
    edges: Vec<GraphEdge>,                     // The arcs
}

struct GraphEdge {
    arc: Arc,        // Geometric arc
    from: VertexId,  // Start vertex
    to: VertexId,    // End vertex
    id: usize,       // Edge identifier
}
```

---

## Integration Points

### Main Pipeline
```
offset_polyline_to_polyline()
    ↓
offset_polyline_raw()
    ↓
offset_split_arcs()  [Phase 1]
    ↓
offset_reconnect_arcs()  [Phase 2-3]
    ├─ merge_close_endpoints_default()
    └─ find_non_intersecting_cycles()
    ↓
[Clean Arclines]
```

### Public API
```rust
pub fn offset_polyline_to_polyline(
    poly: &Vec<Point>,
    offset: f64,
    cfg: &mut OffsetConfig
) -> Vec<Arcline>  // Returns clean cycles
```

---

## Test Coverage

### merge_ends.rs (28 tests)
- ✓ Simple endpoint merging
- ✓ Small arc elimination
- ✓ Multiple point groups (star patterns)
- ✓ Mixed arc types (curved + segments)
- ✓ Boundary conditions
- ✓ Chain connections

### find_cycles.rs (16 tests)
- ✓ Basic shapes (triangle, square)
- ✓ Complex patterns (figure-8, X-intersection)
- ✓ Double edges
- ✓ Multiple separate cycles
- ✓ Mixed geometry
- ✓ Angle selection (rightmost-turn rule)
- ✓ Complex graphs with branches

### Integration Tests (3+)
- ✓ Nearly-closed shapes with gaps
- ✓ Mixed arc types with gaps
- ✓ Curved arcs with gaps
- ✓ Complex mixed shapes

**Total: 47+ tests** - All passing ✅

---

## Performance

### Time Complexity
- Endpoint clustering: **O(n²)**
- Graph building: **O(n)**
- Cycle detection: **O(n·d)** where d ≈ 2-4
- **Total: O(n²)** (dominated by clustering)

### Space Complexity
- **O(n)** linear in arc count

### Real Performance (offset_multi)
```
Input: pline_01 (20 segments)
Output: 99 internal + 90 external offsets

Measurements:
  Phase 2a (merge endpoints): <0.1ms
  Phase 3 (find cycles): <0.1ms
  Total: <0.2ms for cleanup

Total pipeline:
  Raw offset → split → cleanup: ~2ms
```

---

## Key Design Decisions

### Why Agglomerative Clustering?
- Simple and robust
- Works for any point distribution
- Handles both 2D and higher dimensions conceptually
- Natural integration with tolerance-based merging

### Why Graph-Based Approach?
- Separates geometry (arcs) from connectivity (graph structure)
- Handles complex topologies naturally
- Enables geometric-aware traversal rules
- Easy to test and validate independently

### Why "Rightmost-Turn" Rule?
- Prevents self-intersections at vertices
- Works with both line segments and curved arcs
- Mathematically sound (tangent-based directions)
- Proven in computational geometry

### Why Multiple Cycles as Output?
- Some offset results have multiple separate components
- Each component should be renderable independently
- Maintains geometric validity of each cycle
- Enables further processing per cycle if needed

---

## Common Operations

### Get Cycle Information
```rust
let cycles = find_non_intersecting_cycles(&arcs);
for (i, cycle) in cycles.iter().enumerate() {
    println!("Cycle {}: {} arcs", i, cycle.len());
    for arc in cycle {
        println!("  Arc: {:?} -> {:?}", arc.a, arc.b);
    }
}
```

### Merge Endpoints Only
```rust
let mut arcs = /* ... */;
merge_close_endpoints(&mut arcs, 1e-8);
// Now arcs have endpoints within 1e-8 of each other merged
```

### Find Cycles Only
```rust
let arcs = /* ... */;
let cycles = find_non_intersecting_cycles(&arcs);
// Each cycle is a Vec<Arc>
```

### Full Reconnect Pipeline
```rust
let arcs = /* raw split arcs */;
let result = offset_reconnect_arcs(arcs);
// Returns Vec<Arcline> - each element is a cycle
```

---

## Debugging Tips

### Visualize Endpoint Groups
```rust
// After find_endpoint_groups(), endpoints close together
// should be grouped
let groups = find_endpoint_groups(&arcs, 1e-8);
for group in groups {
    println!("Group with {} points, centroid: {:?}", 
             group.points.len(), group.centroid);
}
```

### Check Graph Structure
```rust
let graph = build_graph(&arcs);
println!("Vertices: {}", graph.vertices.len());
println!("Edges: {}", graph.edges.len());
for vertex in 0..graph.vertices.len() {
    let adj = graph.get_adjacent_edges(VertexId(vertex));
    println!("Vertex {}: {} adjacent edges", vertex, adj.len());
}
```

### Verify Cycle Connectivity
```rust
for cycle in cycles {
    for i in 0..cycle.len() {
        let curr = &cycle[i];
        let next = &cycle[(i + 1) % cycle.len()];
        assert!((curr.b - next.a).norm() < 1e-10, 
                "Arcs not connected at index {}", i);
    }
}
```

---

## Examples

### Example 1: Square with Small Gaps
```rust
let arcs = vec![
    arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
    arcseg(Point::new(1.0 + 1e-9, 1e-9), Point::new(1.0, 1.0)),
    arcseg(Point::new(1.0, 1.0), Point::new(0.0, 1.0)),
    arcseg(Point::new(0.0 - 1e-9, 1.0), Point::new(0.0, 0.0)),
];

let result = offset_reconnect_arcs(arcs);
// Result: 1 cycle with 4 properly connected arcs
assert_eq!(result.len(), 1);
assert_eq!(result[0].len(), 4);
```

### Example 2: Two Separate Triangles
```rust
let arcs = vec![
    // Triangle 1
    arcseg(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
    arcseg(Point::new(1.0, 0.0), Point::new(0.5, 1.0)),
    arcseg(Point::new(0.5, 1.0), Point::new(0.0, 0.0)),
    // Triangle 2 (separate)
    arcseg(Point::new(3.0, 0.0), Point::new(4.0, 0.0)),
    arcseg(Point::new(4.0, 0.0), Point::new(3.5, 1.0)),
    arcseg(Point::new(3.5, 1.0), Point::new(3.0, 0.0)),
];

let result = offset_reconnect_arcs(arcs);
// Result: 2 cycles, each with 3 arcs
assert_eq!(result.len(), 2);
for cycle in result {
    assert_eq!(cycle.len(), 3);
}
```

---

## Comparing: Current (v0.3.0) vs Previous

### Before (main without Phase 2-3)
```
offset_split_arcs() → [disconnected arcs]
                       ❌ No merging
                       ❌ No cycle detection
                       ❌ Can't render directly
```

### After (v0.3.0 with Phase 2-3)
```
offset_split_arcs() → offset_reconnect_arcs()
                         ├─ merge_close_endpoints()
                         └─ find_non_intersecting_cycles()
                      → [clean cycles]
                         ✅ Merged endpoints
                         ✅ Separated cycles
                         ✅ Ready to render
```

---

## Status: Ready to Use ✅

All Phase 2-3 features are:
- ✅ Implemented and working
- ✅ Tested with 47+ tests
- ✅ Integrated with main pipeline
- ✅ Optimized and profiled
- ✅ Compatible with TOGO 0.5
- ✅ Released as v0.3.0

**Next**: AABB spatial optimization (Phase 1 enhancement) when needed.

---

## See Also

- `docs/PHASE_2_3_SUMMARY.md` - Detailed technical documentation
- `docs/RECONNECT_REFINEMENTS.md` - Before/after comparison
- `docs/ARCHITECTURE_v0.3.0.md` - Complete system architecture
- `src/graph/find_cycles.rs` - Source code with comments
- `src/graph/merge_ends.rs` - Source code with tests
