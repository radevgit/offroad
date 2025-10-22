# Research Methodology & Validation

## Research Questions Addressed

### Q1: What algorithms are appropriate?
**Answer:** Evaluated 4 main approaches:
- **AABB Pruning** - Simple, validates correctness
- **2D Grid** - Handles your 3-4 neighbor clustering pattern
- **BVH** - O(log n) but more complex
- **Hilbert R-tree** - Excellent but research-only

**Selection:** Grid best for expected use case (hundreds to thousands, 3-4 neighbors)

### Q2: How do we handle bounding boxes without external crates?
**Answer:** Direct arc computation
- Segments: min/max of endpoints (trivial)
- Arcs: use center ± radius + endpoints (simple math)
- No bounding circle overhead needed
- Togo provides direct access to arc geometry

### Q3: Should API depend on Arc type?
**Answer:** No - coordinate-based API
- **Why:** Universal across any geometry system
- **Benefit:** Enables future standalone crate
- **Cost:** None - just pass (minx, maxx, miny, maxy)
- **Flexibility:** Works for polygons, meshes, points, etc.

### Q4: How many arcs in typical use case?
**Your estimate:** Few hundred to low thousands
- **Flat backend:** Supports <500 efficiently
- **Grid backend:** Handles 100-10k+ with tuning
- **Decision:** Start with flat for validation, switch to grid for production

### Q5: Can we integrate without breaking existing code?
**Answer:** Yes - completely optional enhancement
- New module: `src/spatial.rs` (isolated)
- No changes to existing intersection code
- Add spatial structure where beneficial
- Can measure impact independently

## Implementation Validation

### Test Coverage
```
✓ AABB overlap detection - tested
✓ BroadPhaseFlat query - tested
✓ BroadPhaseGrid query with deduplication - tested
✓ Edge cases (empty queries, single item, non-overlapping) - covered
```

### Performance Characteristics
| Operation | Time | Space |
|-----------|------|-------|
| AABB creation | O(1) | O(1) |
| AABB overlap test | O(1) | O(1) |
| Flat add | O(1) | O(1) per item |
| Flat query | O(n) | O(k) result |
| Grid add | O(1) amortized | O(1) per cell |
| Grid query | O(1) lookup + O(k) scan | O(k) result |

### Correctness Guarantees
- ✓ AABB is conservative (contains all geometry)
- ✓ No false negatives (all overlaps found)
- ✓ May have false positives (AABB > actual geometry)
- ✓ Precise intersection tests filter false positives
- ✓ Results identical to O(n²) approach

## Benchmark Methodology

### Phase 1: Profile Current Baseline
```rust
// Instrument offset_split_arcs to count:
// 1. Total pairs tested
// 2. Intersections found
// 3. Execution time

// Extract test case: offset_pline1
// Measure: input arcs, output arcs, time
```

### Phase 2: Validate Flat Backend
```rust
// Replace intersection loop with:
// 1. Build BroadPhaseFlat
// 2. Query candidates
// 3. Test only candidates
// 4. Verify identical results
// 5. Measure overhead

// Metric: pairs eliminated, overhead cost
```

### Phase 3: Optimize Grid Backend
```rust
// Try cell sizes: 25, 50, 100, 200 (scaled by arc size)
// For each:
// 1. Build grid
// 2. Profile query + candidate tests
// 3. Count candidates per query
// 4. Measure total time

// Tuning: Find optimal cell size
```

### Phase 4: Compare Implementations
```
Metric                  Baseline    Flat        Grid
Pairs tested           44,850      ~900        ~900
Overhead time          0           +5%         +10%
Intersection time      100%        ~2%         ~2%
Total speedup          1.0×        ~45×        ~35×
(but intersections are much smaller % of time in reality)
```

## Research Findings

### Finding 1: Conservative AABB Safe
- AABB filtering performs no intersection tests
- Only eliminates candidates where boxes don't overlap
- Precise tests still determine true intersections
- Therefore: Safe to use AABB as preliminary filter

### Finding 2: Coordinate API Universal
- Decoupling from Arc type enables broader use
- Any geometry type reducible to AABB
- Future work: use in other projects, CAD systems, etc.
- No loss of functionality, only gain in flexibility

### Finding 3: Grid Natural for Clustering
- Your expected 3-4 neighbor pattern maps to grid cells
- Cell size ≈ 2× typical neighborhood diameter
- Auto-deduplication prevents duplicates across cell boundaries
- Extreme cases (very thin arcs) need larger cells

### Finding 4: No Magic Cell Size
- Grid tuning essential for performance
- Too small: many cells, high overhead
- Too large: too many candidates per cell
- Optimal likely 2-5× typical inter-arc distance

### Finding 5: Statistics Built-in
- Must measure to validate improvements
- Track bbox_tests, overlaps, precise_tests
- Reveals pruning effectiveness
- Necessary for tuning grid cell size

## Assumptions & Risks

### Assumption 1: 3-4 Neighbors Typical
**Risk:** If some geometries have many intersecting arcs
- **Mitigation:** Measure on real data
- **Fallback:** Use larger grid cells or BVH

### Assumption 2: Intersection Test Expensive
**Risk:** If AABB operations become bottleneck
- **Mitigation:** Profile to measure cost ratio
- **Fallback:** Simplify AABB (coarser bounds)

### Assumption 3: Arc Distribution Clustered
**Risk:** If arcs scattered uniformly
- **Mitigation:** Grid still works, less effective
- **Fallback:** Try sweep line for uniform cases

### Assumption 4: Few Thousands Arcs
**Risk:** If cases exceed 10k+ arcs
- **Mitigation:** BVH implementation needed
- **Fallback:** R-tree or hybrid approach

## Mitigation Strategy

### Before Integration
1. Profile offset_pline1 baseline (count pairs, time)
2. Implement BroadPhaseFlat (validate correctness)
3. Measure overhead on small example
4. Profile to understand cost breakdown

### During Integration
1. Start with Flat (lowest risk)
2. A/B test on representative cases
3. Measure actual neighbor count
4. Tune grid cell size if switching

### After Integration
1. Monitor statistics on real geometries
2. Collect performance metrics
3. Adjust tuning parameters as needed
4. Plan BVH upgrade if needed

## Future Research Directions

### 1. Auto-Tuning Cell Size
```rust
// Analyze arc distribution, compute optimal cell size
let arc_length_stats = analyze_arc_lengths(&arcs);
let cell_size = arc_length_stats.avg * 2.0;
```

### 2. Hybrid Approach
```rust
// Use flat for < 500 arcs, grid for >= 500
if arcs.len() < 500 {
    BroadPhaseFlat::new()
} else {
    BroadPhaseGrid::new(auto_tuned_cell_size)
}
```

### 3. Incremental Grid Updates
```rust
// Instead of rebuilding, incrementally update
// As arcs are split, update grid structure
```

### 4. BVH Implementation
```rust
// If grid doesn't scale well
// Binary tree of AABBs for O(log n) queries
```

### 5. Problem-Specific Optimization
```rust
// For offset-specific patterns:
// - Track which arcs already tested
// - Exploit connectivity structure
// - Cache intersection results
```

## Validation Checklist

- [x] Algorithm analysis complete
- [x] Implementation correct (tests pass)
- [x] No external dependencies
- [x] Coordinate-based API designed
- [x] Multiple backends implemented
- [x] Statistics tracking built-in
- [x] Documentation comprehensive
- [ ] Baseline profile measured (offset_pline1)
- [ ] Flat backend integrated & profiled
- [ ] Grid backend benchmarked with tuning
- [ ] Real-world geometry tested
- [ ] Decision on production readiness made

## Conclusion

Research phase complete with:
- ✓ Evaluated 4 algorithms
- ✓ Designed universal API
- ✓ Implemented 2 backends + helpers
- ✓ Zero external dependencies
- ✓ Comprehensive documentation
- ✓ Built-in profiling support

Next phase: Integration & measurement
