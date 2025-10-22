# Spatial Acceleration - Quick Reference

## API Cheat Sheet

### Adding Geometry

```rust
use offroad::spatial::{aabb_from_arc, BroadPhaseFlat, BroadPhaseGrid};
use togo::prelude::*;

// Create spatial index
let mut bp = BroadPhaseFlat::new();  // or BroadPhaseGrid::new(50.0)

// Compute bounds from arc
let arc = Arc { /* ... */ };
let bbox = aabb_from_arc(&arc);

// Add to index (coordinate-based API)
bp.add(arc_id, bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);

// Or directly with computed coordinates:
bp.add(arc_id, 1.0, 3.0, 2.0, 4.0);  // (minx, maxx, miny, maxy)
```

### Querying Candidates

```rust
// Query using same coordinate format
let candidates: Vec<usize> = bp.query(
    query_bbox.min_x,
    query_bbox.max_x,
    query_bbox.min_y,
    query_bbox.max_y,
);

// Candidates are IDs, use to lookup original geometry
for id in candidates {
    let arc1 = &all_arcs[id];
    // Perform precise intersection test
}
```

### Statistics (Grid Only)

```rust
let stats = bp.stats();
println!("AABB tests: {}", stats.bbox_tests);
println!("Overlaps: {}", stats.bbox_overlaps);
println!("Intersection tests: {}", stats.precise_tests);
```

## Implementation Comparison

| Aspect | BroadPhaseFlat | BroadPhaseGrid |
|--------|---|---|
| **Query Time** | O(n) | O(1) + O(k) |
| **Memory** | O(n) | O(n + cells) |
| **Tuning** | None | Cell size |
| **Best For** | <500 items, validation | 100-10k items, clustering |
| **Cache** | Good | Excellent |
| **Worst Case** | All pairs overlap | Long thin AABBs |

## Backend Selection

```
Small dataset (<500 arcs)?
└─ Use BroadPhaseFlat

Medium dataset (500-5000) with 3-4 neighbors?
└─ Use BroadPhaseGrid
   Cell size = 2 × typical arc length

Large dataset (>5000) or non-uniform?
└─ Use BroadPhaseGrid with auto-tuning
   Or implement BroadPhaseBVH (future)
```

## Integration Checklist

- [ ] Import `use offroad::spatial::*`
- [ ] Create index: `BroadPhaseFlat::new()` or `BroadPhaseGrid::new(cell_size)`
- [ ] For each arc: compute `aabb_from_arc()`, call `add(id, ...)`
- [ ] In loop: `query()` returns candidate IDs
- [ ] Replace full intersection loop with candidate-only loop
- [ ] Verify results identical (regression test)
- [ ] Benchmark: measure speedup
- [ ] Profile: analyze candidate count & pruning effectiveness

## Tuning BroadPhaseGrid Cell Size

```rust
// Start with estimate based on typical geometry
let avg_arc_length = 50.0;
let cell_size = avg_arc_length * 2.0;  // 100.0
let mut bp = BroadPhaseGrid::new(cell_size);

// Add all geometry...

// Check stats to see if tuning needed
let stats = bp.stats();
let candidates_per_query = stats.bbox_overlaps / (stats.bbox_tests.max(1));

if candidates_per_query > 10.0 {
    // Too many candidates, increase cell size
    cell_size *= 2.0;
} else if candidates_per_query < 2.0 {
    // Too few candidates, may be missing intersections
    cell_size /= 2.0;
}
```

## Common Pitfalls

### ❌ Wrong coordinate order
```rust
// WRONG - will fail assertion
bp.add(id, 3.0, 1.0, 4.0, 2.0);  // min > max!

// RIGHT
bp.add(id, 1.0, 3.0, 2.0, 4.0);  // min < max
```

### ❌ Forgetting to rebuild for each offset iteration
```rust
// If arcs change between operations, rebuild:
bp.clear();
for (i, arc) in new_arcs.iter().enumerate() {
    let bbox = aabb_from_arc(arc);
    bp.add(i, bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
}
```

### ❌ Using old Arc indices after removals
```rust
// IDs are relative to insertion order, not stable references
// Keep external ID mapping if removing arcs:
let mut id_to_arc: HashMap<usize, Arc> = HashMap::new();
for (id, arc) in arcs.iter().enumerate() {
    let bbox = aabb_from_arc(arc);
    bp.add(id, bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
    id_to_arc.insert(id, arc.clone());
}
```

## Performance Tips

### For offset_split_arcs()

1. **Single grid creation** (not per iteration):
   ```rust
   let mut bp = BroadPhaseGrid::new(50.0);
   // Build once at start
   for (i, arc) in parts.iter().enumerate() {
       let bbox = aabb_from_arc(arc);
       bp.add(i, bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
   }
   ```

2. **Reuse query bounds**:
   ```rust
   // Compute once per arc
   let bbox = aabb_from_arc(arc0);
   let candidates = bp.query(bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
   ```

3. **Profile before optimizing**:
   ```rust
   let start = Instant::now();
   let candidates = bp.query(...);
   let query_time = start.elapsed();
   
   let start = Instant::now();
   let result = precise_intersection_test(&arc0, &arc1);
   let test_time = start.elapsed();
   
   // If query_time > test_time/10, something's wrong
   ```

## Expected Results (offset_pline1)

Based on expected geometry (few hundred arcs, 3-4 neighbors):

| Metric | Flat | Grid |
|--------|------|------|
| Pair reduction | 40-50× | 40-50× |
| Candidates per query | ~3-4 | ~3-4 |
| Overhead | Low | Medium |
| **Total speedup** | ~2-5× | ~5-15× |

*Actual depends on intersection test cost vs spatial overhead*

## Debugging

### Validate candidates

```rust
// Ensure candidates include actual intersections
let candidates = bp.query(bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
for id in candidates {
    if let Some(intersection) = precise_test(&arc0, &arcs[id]) {
        // ✓ Good - intersection found in candidate
    }
}

// Sanity check - no missed intersections
for (id, arc1) in all_arcs.iter().enumerate() {
    if let Some(_) = precise_test(&arc0, &arc1) {
        // Must be in candidates!
        assert!(candidates.contains(&id), "Missed intersection!");
    }
}
```

### Monitor stats

```rust
let stats = bp.stats();
eprintln!("AABB tests: {}", stats.bbox_tests);
eprintln!("Overlaps: {:.1}%", (stats.bbox_overlaps as f64 / stats.bbox_tests as f64) * 100.0);
eprintln!("Precise tests: {}", stats.precise_tests);
```

## References

- Implementation: `src/spatial.rs`
- Algorithm analysis: `docs/SPATIAL_ACCELERATION.md`
- Integration guide: `docs/SPATIAL_IMPLEMENTATION_RESEARCH.md`
- Visual guide: `docs/SPATIAL_ARCHITECTURE_DIAGRAM.md`
