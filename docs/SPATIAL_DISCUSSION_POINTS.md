# Spatial Acceleration Research - Summary for Discussion

## Overview

Implemented a **modular spatial index framework** for intersection detection acceleration in `offset_split_arcs` and `offset_prune_invalid`. Ready for integration phase and performance evaluation.

## Deliverables

### 1. Core Implementation (`src/spatial.rs`, 330 lines)

**Generic API** (no togo/Arc dependencies):
- `add(id, min_x, max_x, min_y, max_y)` - Add AABB
- `query(min_x, max_x, min_y, max_y) -> Vec<usize>` - Get overlapping candidates

**Two Backends:**
- **BroadPhaseFlat** - O(n) linear scan (validation, small datasets)
- **BroadPhaseGrid** - O(1) hash + O(k) candidates (production, clustering)

**Helper Functions:**
- `aabb_from_segment()` - Trivial (min/max endpoints)
- `aabb_from_arc()` - Uses arc geometry directly (no bounding circles needed)

### 2. Documentation

- `SPATIAL_ACCELERATION.md` - Algorithm analysis + integration strategy
- `SPATIAL_IMPLEMENTATION_RESEARCH.md` - Implementation details + next steps
- `SPATIAL_ARCHITECTURE_DIAGRAM.md` - Visual guides + decision tree

### 3. Testing

All 91 tests passing:
```
test spatial::tests::test_aabb_overlap ... ok
test spatial::tests::test_broad_phase_flat_query ... ok
test spatial::tests::test_broad_phase_grid_query ... ok
```

## Key Design Decisions

### ✓ Coordinate-Based API
**Why:** Universal across any geometry system, not locked to Arc type
- Can be reused for other projects (physics, CAD, etc.)
- Potential future extraction to standalone `spatial-index` crate
- Type-safe with AABB::new() validation

### ✓ No External Dependencies
**Why:** Per your philosophy + full control
- Only uses std::collections::HashMap
- Togo-only for geometry data
- Can implement advanced structures (BVH, R-tree) as needed

### ✓ Multiple Backends
**Why:** Flexibility for different use cases
- Flat validates correctness first
- Grid handles your expected 3-4 neighbor clustering
- Future: BVH for large/irregular datasets

### ✓ Built-in Statistics
**Why:** Essential for benchmarking effectiveness
- `bbox_tests` - How many AABB tests performed
- `bbox_overlaps` - How many passed AABB filter
- `pruning_ratio` - Effectiveness of spatial filtering

## Addressing Your Requirements

| Requirement | Status | Details |
|---|---|---|
| Generic coordinate API | ✓ | `add(id, min_x, max_x, min_y, max_y)` |
| No external crates | ✓ | Only std, zero dependencies beyond togo |
| Direct arc bounds | ✓ | `aabb_from_arc()` uses arc geometry (no circles) |
| Potential crate separation | ✓ | Module is already decoupled |
| Handle 3-4 neighbors | ✓ | Grid backend designed for this pattern |
| Few hundred to thousands | ✓ | Both implementations scale to this range |
| Universal geometry support | ✓ | Coordinates-only interface |

## Expected Performance Gains

### Theoretical (300 arcs, 3-4 neighbors)

```
Baseline:       300 × 299 / 2 = 44,850 pairs tested
With flat:      300 × 3 = 900 pairs (validation)
With grid:      ~900 pairs (production)

Pair reduction: 50× improvement
Speedup factor: Depends on intersection test cost vs spatial overhead
```

### Measured (offset_pline1 example)

**Still needed:** Profile to determine:
- Actual neighbor count distribution
- Intersection test cost
- Spatial operation overhead
- Optimal grid cell size

## Integration Steps

### Phase 1: Validation (Low Risk)

```rust
use offroad::spatial::{BroadPhaseFlat, aabb_from_arc};

// In offset_split_arcs():
let mut bp = BroadPhaseFlat::new();
for (i, arc) in parts.iter().enumerate() {
    let bbox = aabb_from_arc(arc);
    bp.add(i, bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
}

for arc0 in parts.iter() {
    let bbox = aabb_from_arc(arc0);
    let candidates = bp.query(bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
    
    for id in candidates {
        // Only expensive intersection tests on candidates
        if should_intersect(&arc0, &parts[id]) {
            // Existing split logic...
        }
    }
}
```

### Phase 2: Production (Tuning)

```rust
// Replace BroadPhaseFlat with BroadPhaseGrid
let mut bp = BroadPhaseGrid::new(50.0);  // Cell size = tuning parameter

// Rest of code identical - same API!
let candidates = bp.query(bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
```

### Phase 3: Benchmarking

Profile both variants on representative geometries:
- offset_pline1 (current example)
- Pathological cases (dense clusters, long thin arcs)
- Various offset distances

## Questions for Next Discussion

1. **Profiling:** Can you provide timing breakdown for offset_pline1?
   - Current execution time
   - Estimated % spent in intersection tests
   - Count of actual intersections found

2. **Geometry Characteristics:**
   - Are there representative test cases with 100s-1000s arcs?
   - Do arcs tend to be similar length or highly variable?
   - Any pathological cases (very dense clusters)?

3. **Grid Cell Tuning:**
   - Can we auto-detect optimal cell size from arc statistics?
   - Or specify as parameter to OffsetCfg?

4. **Future Extensions:**
   - Would BVH (O(log n) query) be valuable for future scaling?
   - Any other intersection detection use cases beyond offset?

5. **Integration Timeline:**
   - When would you want to integrate flat version for testing?
   - Separate branch or main?

## Files Modified/Created

- **Created:** `src/spatial.rs` (full implementation)
- **Created:** `docs/SPATIAL_ACCELERATION.md` (algorithm analysis)
- **Created:** `docs/SPATIAL_IMPLEMENTATION_RESEARCH.md` (detailed summary)
- **Created:** `docs/SPATIAL_ARCHITECTURE_DIAGRAM.md` (visual guides)
- **Modified:** `src/lib.rs` (exported spatial module)
- **Created:** `examples/debug_intersections.rs` (instrumentation template)

## Status

- ✓ Design complete
- ✓ Implementation complete
- ✓ Tests passing
- ✓ Documentation comprehensive
- ⏳ Ready for profiling & integration

## Next Steps (Your Decision)

**Option A: Experimental**
- Integrate BroadPhaseFlat into offset_split_arcs
- Profile offset_pline1 to measure effectiveness
- Decide whether to proceed with grid optimization

**Option B: Research Extended**
- Investigate specific problem geometries
- Profile current baseline to identify bottleneck
- Then decide between flat/grid/other approaches

**Option C: Deeper Analysis**
- Implement instrumentation in current code (count pairs tested)
- Characterize intersection test distribution
- Then choose acceleration strategy

## Discussion Points

1. Should we start with profiling current code to establish baseline?
2. Is flat → grid progression the right path, or try grid immediately?
3. Should spatial structure be optional feature in OffsetCfg?
4. Any other algorithms worth exploring (sweep line, etc.)?
5. Timeline for integration?
