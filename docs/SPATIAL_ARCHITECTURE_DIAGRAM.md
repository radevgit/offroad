# Spatial Acceleration Architecture

## Current O(n²) Approach

```
offset_split_arcs()
├─ Collect all arcs into parts: Vec<Arc>
├─ for each part0 in parts:
│  ├─ for each part1 in remaining parts:  ← O(n²) tests!
│  │  ├─ AABB check: 1 cycle (skipped)
│  │  └─ Intersection test: 1000+ cycles (expensive!)
│  └─ Handle split if found
└─ Return final arcs

Behavior: ~300 arcs → 44,850 intersection tests
```

## Optimized with BroadPhaseFlat (Validation)

```
offset_split_arcs()
├─ Collect all arcs into parts: Vec<Arc>
├─ Create BroadPhaseFlat
│  └─ For each arc: compute AABB, store in flat Vec
├─ for each part0 in parts:
│  ├─ Query spatial: bp.query(bbox) → 3-4 candidates ← Fast!
│  ├─ for each candidate in results:
│  │  ├─ AABB check: 1 cycle (already overlapping)
│  │  └─ Intersection test: 1000+ cycles (only on candidates!)
│  └─ Handle split if found
└─ Return final arcs

Benefit: Validates correctness, identifies neighbor count
Overhead: Linear scan per query (acceptable for <500 items)
```

## Optimized with BroadPhaseGrid (Production)

```
offset_split_arcs()
├─ Collect all arcs into parts: Vec<Arc>
├─ Create BroadPhaseGrid with cell_size=50.0
│  └─ For each arc:
│     ├─ Compute AABB
│     ├─ Find grid cells: (gx_min..gx_max, gy_min..gy_max)
│     └─ Insert into each cell's Vec<AABB>
├─ for each part0 in parts:
│  ├─ Query spatial: bp.query(bbox)
│  │  ├─ Convert AABB to grid cells: O(1)
│  │  ├─ Look up cells in HashMap: O(1) per cell
│  │  └─ Gather AABBs: O(k) where k ≈ 3-4
│  ├─ for each candidate in results:  ← Only 3-4!
│  │  ├─ AABB check: 1 cycle
│  │  └─ Intersection test: 1000+ cycles
│  └─ Handle split if found
└─ Return final arcs

Behavior: 300 arcs with 3-4 neighbors → 1,200 tests (37× speedup)
Memory: O(n) grid cells + O(n) AABB storage (modest)
```

## API Flow

### Adding Geometry

```rust
// User has Arc data from offset calculations
let arc = Arc { a: point(1.0, 2.0), b: point(3.0, 4.0), c: point(2.0, 2.0), r: 1.0 };

// Compute bounds (no circles needed, direct from arc)
let bbox = aabb_from_arc(&arc);  // → AABB { min_x: 1.0, max_x: 3.0, min_y: 1.0, max_y: 4.0 }

// Add to spatial structure (universal coordinates)
bp.add(arc_id, bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);
```

### Querying Candidates

```rust
// Want to find all arcs intersecting with arc0
let bbox = aabb_from_arc(&arc0);

// Query using coordinate-based API (no Arc type needed)
let candidates: Vec<usize> = bp.query(bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y);

// Candidates are IDs of potentially intersecting arcs (~3-4 expected)
for id in candidates {
    let arc1 = &parts[id];
    // Perform precise intersection test only on candidates
    if intersects(&arc0, &arc1) {
        // Handle intersection...
    }
}
```

## Data Structure Comparison

### BroadPhaseFlat

```
items: Vec<(id, AABB)>
┌─────────┬──────────┬──────────┬──────────┐
│ Arc0    │ Arc1     │ Arc2     │ Arc3     │
│ AABB(1) │ AABB(2)  │ AABB(3)  │ AABB(4)  │
└─────────┴──────────┴──────────┴──────────┘

query(bbox) → iterate all, test each AABB overlap
Time: O(n) per query
Best for: Validation, <500 items
```

### BroadPhaseGrid

```
grid: HashMap<(gx, gy), Vec<(id, AABB)>>

    ┌─────────┬──────────┬──────────┐
    │ Cell(0,0) │ Cell(1,0) │ Cell(2,0)│
    │ Arc0 Arc5 │ Arc1 Arc3 │ Arc2 Arc7│
    └─────────┴──────────┴──────────┘
    ┌─────────┬──────────┬──────────┐
    │ Cell(0,1) │ Cell(1,1) │ Cell(2,1)│
    │ Arc4 Arc6 │ Arc9 Arc8 │ Arc10    │
    └─────────┴──────────┴──────────┘

query(bbox) → find cells occupied by bbox, gather AABBs, deduplicate
Time: O(1) cell lookup + O(k) candidates where k ≈ 3-4 per cell
Memory: O(n + c) where c = grid cells
Best for: Production, 100-10k+ items, clustering
```

## Intersection Test Reduction

### Geometry Pattern

```
Original polyline (offset path):
    ┌─────────────┐
    │   Arc0      │
    │ ┌─────────┐ │
    │ │ Arc1    │ │
    │ │  ┌────┐ │ │
    │ │  │Arc2│ │ │
    │ └──┼────┼─┘ │
    │    │Arc3│   │  ← These 3-4 arcs may intersect each other
    └────┴────┴───┘

Without spatial: Test Arc2 vs all 296 other arcs
With spatial:   Test Arc2 vs only Arc1, Arc3, maybe Arc4 (3-4 tests)
```

### Test Count Example

```
300 arcs, expected 3-4 neighbors per arc

Baseline O(n²):
  300 × 299 / 2 = 44,850 pairs tested

With grid (30 neighbors per cell, ~10 cells occupied):
  300 × 3 = 900 pairs tested (3 neighbors per arc average)

Speedup: 44,850 / 900 ≈ 50× on pair tests
(Actual timing benefit depends on intersection test cost vs spatial overhead)
```

## Extension Points for Future Work

### 1. Faster Query (BVH)
```rust
pub trait BroadPhase {
    fn add(&mut self, id: usize, min_x: f64, max_x: f64, min_y: f64, max_y: f64);
    fn query(&mut self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Vec<usize>;
}

// Current implementations
impl BroadPhase for BroadPhaseFlat { /* ... */ }
impl BroadPhase for BroadPhaseGrid { /* ... */ }

// Future implementation
impl BroadPhase for BroadPhaseBVH { /* O(log n) query */ }
```

### 2. Reuse in Other Projects
```rust
// Extract to separate crate: spatial-index
// Used by: offroad, CAD tools, physics engines, game engines
```

### 3. Statistics/Profiling
```rust
let stats = bp.stats();
println!("AABB tests: {}", stats.bbox_tests);
println!("Overlaps: {}", stats.bbox_overlaps);
println!("Pruning ratio: {:.1}%", stats.pruning_ratio * 100.0);
```

## Decision Tree

```
Start: offset_split_arcs has many intersection tests

├─ Is n < 500?
│  └─ Use BroadPhaseFlat
│     └─ Fastest to implement, validates correctness
│        └─ If 10x speedup achieved → Production ready
│           └─ If still too slow → Benchmark grid
│
├─ Is n < 5000 with clustering?
│  └─ Use BroadPhaseGrid
│     └─ Tune cell_size = 2 × typical arc length
│        └─ If 5x+ speedup → Production ready
│           └─ If memory issues → Tune cell_size up
│
└─ Is n > 5000 or non-uniform?
   └─ Benchmark BVH implementation
      └─ O(log n) query beats grid worst-case
         └─ Accept higher build cost for dynamic insert scenarios
```
