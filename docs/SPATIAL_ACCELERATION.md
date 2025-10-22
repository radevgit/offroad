# Spatial Acceleration for Intersection Detection

## Problem Analysis

**Current bottleneck:** `offset_split_arcs` and `offset_prune_invalid` perform O(n²) pairwise intersection tests without spatial filtering.

**Current approach:**
- `offset_split_arcs`: Iterates through all arc pairs, tests for intersection
- `offset_prune_invalid`: Tests each offset arc against all original polyline arcs

**Complexity:** For n arcs, performs n(n-1)/2 expensive intersection computations (arc-arc, segment-arc, segment-segment).

## Available Resources

**Togo library (v0.5.0) provides:**
- Direct arc bounds calculation (no bounding circles required)
- Segment endpoints + arc geometry directly encode bounds
- `dist_arc_arc()`, `dist_segment_arc()` → precise distance calculations

## Implementation Strategy

### Generic API Design
Universal coordinate-based interface (no dependency on Arc type):

```rust
pub fn add(&mut self, id: usize, min_x: f64, max_x: f64, min_y: f64, max_y: f64)
pub fn query(&self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Vec<usize>
```

This enables:
- Use in other projects (non-Offroad geometry systems)
- Potential future separation into standalone crate
- Framework for multiple backends (Flat, Grid, future R-tree)

## Candidate Implementations (In Order of Complexity)

### 1. **BroadPhaseFlat: Simple Linear Scan** ⭐ Start Here
- **Complexity:** O(n) per query, O(n²) worst-case all pairs
- **Implementation:** Store all AABBs in flat Vec, linear AABB-AABB overlap tests
- **Pros:** Minimal code, zero overhead, deterministic, cache-friendly for small n
- **Cons:** No spatial benefit for dense clusters
- **Best for:** <500 arcs, preliminary analysis, fallback

### 2. **BroadPhaseGrid: 2D Spatial Hashing**
- **Complexity:** O(1) cell lookup + O(k) candidates per query, where k = arcs per cell
- **Implementation:** HashMap of grid cells, each cell stores overlapping AABBs
- **Configuration:** Cell size must match typical arc "neighborhood" (3-4 arcs)
- **Pros:** Handles clustering naturally, automatic load balancing, good cache locality
- **Cons:** Pathological cases (very thin/long arcs spanning many cells), tuning required
- **Best for:** 100-10k arcs with expected 3-4 nearest neighbors

### 3. **BroadPhaseBVH: Bounding Volume Hierarchy** (Future)
- **Complexity:** O(log n) query + O(n log n) build
- **Implementation:** Binary tree of AABBs, recursive spatial partitioning
- **Pros:** Optimal asymptotic performance, handles all distributions
- **Cons:** More complex, O(n) memory overhead, rebuild cost
- **Best for:** 10k+ arcs or rebuild-heavy workloads

### 4. **Packed Hilbert R-tree** (Research Phase)
- **Complexity:** O(log n) query, cache-aware traversal
- **Implementation:** Sort AABBs by Hilbert curve, build balanced tree
- **Pros:** Hilbert ordering improves cache hit rate, handles non-uniform data
- **Cons:** Complex to implement, immutable structure (full rebuild on insert)
- **Best for:** Very large datasets with irregular distributions

## Implemented Code Structure

**File:** `src/spatial.rs`

**Core Types:**
- `AABB`: Axis-aligned bounding box (min_x, max_x, min_y, max_y)
- `BroadPhaseFlat`: Flat list with linear queries
- `BroadPhaseGrid`: Grid-based spatial partitioning
- Helper functions: `aabb_from_segment()`, `aabb_from_arc()`

**Key Methods:**
```rust
aabb.overlaps(&other)        // O(1) AABB overlap test
broad_phase.add(id, min_x, max_x, min_y, max_y)
broad_phase.query(min_x, max_x, min_y, max_y) -> Vec<usize>
```

## Integration Strategy

### Phase 1: Profile & Establish Baseline
```rust
// In offset_split_arcs():
let mut bp = BroadPhaseFlat::new();
for (i, arc) in parts.iter().enumerate() {
    let aabb = aabb_from_arc(arc);
    bp.add(i, aabb.min_x, aabb.max_x, aabb.min_y, aabb.max_y);
}

// Replace O(n²) loop:
for arc0 in parts.iter() {
    let bbox = aabb_from_arc(arc0);
    let candidates = bp.query(bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
    for id in candidates {
        // Only test these O(3-4) candidates instead of all n
        if should_test_intersection(&arc0, &parts[id]) {
            // Perform expensive intersection test
        }
    }
}
```

### Phase 2: Evaluate Grid Backend
```rust
// Tune cell_size based on typical arc length / neighborhood size
let mut bp = BroadPhaseGrid::new(50.0);  // Adjust based on geometry scale
// Rest of code identical
```

### Phase 3: Benchmark & Decision
```
Compare:
- Baseline (no spatial structure)
- BroadPhaseFlat (validates correctness)
- BroadPhaseGrid with various cell sizes
- Measure: total time, % of pairs skipped, memory usage
```

## Key Design Decisions

1. **No external crates:** Only togo dependency ✓
2. **Coordinate-based API:** Universal across any geometry system ✓
3. **Separated module:** Future potential as standalone crate ✓
4. **Statistics tracking:** Built-in profiling (bbox_tests, overlaps, etc.) ✓
5. **Multiple backends:** Easy to swap implementations ✓

## Research Questions to Answer via Implementation

1. **Optimal cell size:** How to auto-tune BroadPhaseGrid cell_size?
2. **Overflow cells:** Do some cells accumulate many arcs (pathological)?
3. **AABB tightness:** How much does conservative AABB overhead cost?
4. **Memory vs speed:** When does O(n) grid overhead become worthwhile?
5. **Rebuild cost:** How expensive is rebuilding on each offset iteration?

## Estimated Impact

**Assumption:** 300 arcs with 3-4 neighbors per arc
- **Baseline:** 300 × 299 / 2 ≈ 44,850 intersection tests
- **With spatial:** ~1,200 tests (only 3-4 neighbors per arc)
- **Speedup:** ~37× (if per-test cost is constant)

**Actual speedup depends on:**
- Exact neighbor distribution
- Cost of spatial lookup vs intersection test ratio
- Cache effects (grid might be worse for some patterns)

## Next Steps

1. Run offset_pline1 with BroadPhaseFlat to confirm neighbor count
2. Profile to measure breakdown: spatial overhead vs intersection time
3. Implement BroadPhaseGrid with auto-tuned cell_size
4. Benchmark both on representative test cases
5. Decide: sufficient for near-term, or pursue BVH?
