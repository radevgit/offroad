#![allow(dead_code)]

use togo::prelude::*;

/// Statistics from spatial acceleration structure queries
#[derive(Debug, Clone, Default)]
pub struct SpatialStats {
    pub total_pairs_tested: usize,
    pub bbox_tests: usize,
    pub bbox_overlaps: usize,
    pub precise_tests: usize,
    pub intersections_found: usize,
    pub pruning_ratio: f64,
}

/// Generic 2D AABB (Axis-Aligned Bounding Box)
/// Layout: (min_x, max_x, min_y, max_y)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl AABB {
    /// Create new AABB from coordinates
    pub fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        debug_assert!(min_x <= max_x, "min_x must be <= max_x");
        debug_assert!(min_y <= max_y, "min_y must be <= max_y");
        AABB { min_x, max_x, min_y, max_y }
    }

    /// Check if two AABBs overlap (inclusive)
    pub fn overlaps(&self, other: &AABB) -> bool {
        !(self.max_x < other.min_x || self.min_x > other.max_x ||
          self.max_y < other.min_y || self.min_y > other.max_y)
    }

    /// Check if point is inside AABB
    pub fn contains_point(&self, p: &Point) -> bool {
        p.x >= self.min_x && p.x <= self.max_x &&
        p.y >= self.min_y && p.y <= self.max_y
    }

    /// Expand AABB to include another AABB
    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min_x: self.min_x.min(other.min_x),
            max_x: self.max_x.max(other.max_x),
            min_y: self.min_y.min(other.min_y),
            max_y: self.max_y.max(other.max_y),
        }
    }

    /// Expand AABB by epsilon on all sides
    pub fn expand(&self, epsilon: f64) -> AABB {
        AABB {
            min_x: self.min_x - epsilon,
            max_x: self.max_x + epsilon,
            min_y: self.min_y - epsilon,
            max_y: self.max_y + epsilon,
        }
    }
}

/// Compute AABB for line segment
pub fn aabb_from_segment(a: &Point, b: &Point) -> AABB {
    AABB {
        min_x: a.x.min(b.x),
        max_x: a.x.max(b.x),
        min_y: a.y.min(b.y),
        max_y: a.y.max(b.y),
    }
}

/// Compute loose AABB for arc (fast, non-tight)
/// Includes full circle bounds, so may be 40x larger than tight bounds
/// Trade-off: Fast O(1) computation vs. loose pruning candidates
/// Good for: Fast culling, when geometry is sparse or well-distributed
pub fn aabb_from_arc_loose(arc: &Arc) -> AABB {
    // Start with endpoints
    let mut min_x = arc.a.x.min(arc.b.x);
    let mut max_x = arc.a.x.max(arc.b.x);
    let mut min_y = arc.a.y.min(arc.b.y);
    let mut max_y = arc.a.y.max(arc.b.y);

    // Expand bounds to include full circle (very loose)
    let radius = arc.r;
    let cx = arc.c.x;
    let cy = arc.c.y;

    min_x = min_x.min(cx - radius);
    max_x = max_x.max(cx + radius);
    min_y = min_y.min(cy - radius);
    max_y = max_y.max(cy + radius);

    AABB { min_x, max_x, min_y, max_y }
}

/// Compute tight AABB for arc (slow, uses bounding circle algorithm)
/// Uses togo's arc_bounding_circle which calculates the minimal enclosing circle
/// Trade-off: Slower O(tan calc) but much tighter bounds (1-4x area vs full circle)
/// Good for: Dense geometry where better pruning matters; arc validation against many arcs
pub fn aabb_from_arc_tight(arc: &Arc) -> AABB {
    let bc = arc_bounding_circle(arc);
    let r = bc.r;
    AABB {
        min_x: bc.c.x - r,
        max_x: bc.c.x + r,
        min_y: bc.c.y - r,
        max_y: bc.c.y + r,
    }
}

/// Default: use loose AABB for maximum speed
/// Use tight variant explicitly if needed for specific geometry
pub fn aabb_from_arc(arc: &Arc) -> AABB {
    aabb_from_arc_loose(arc)
}

/// Simple flat list with AABB lookup - O(n) query but minimal overhead
/// Good for small to medium datasets (hundreds to low thousands)
pub struct BroadPhaseFlat {
    items: Vec<(usize, AABB)>,  // (id, bbox)
}

impl BroadPhaseFlat {
    pub fn new() -> Self {
        BroadPhaseFlat { items: Vec::new() }
    }

    /// Add item with AABB (minx, maxx, miny, maxy)
    pub fn add(&mut self, id: usize, min_x: f64, max_x: f64, min_y: f64, max_y: f64) {
        self.items.push((id, AABB::new(min_x, max_x, min_y, max_y)));
    }

    /// Get all candidates overlapping with given AABB
    pub fn query(&self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Vec<usize> {
        let query_box = AABB::new(min_x, max_x, min_y, max_y);
        self.items
            .iter()
            .filter(|(_, bbox)| bbox.overlaps(&query_box))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// 2D Grid spatial structure - excellent for uniform distributions
/// Trades memory (grid cells) for query speed O(1) per cell
pub struct BroadPhaseGrid {
    cell_size: f64,
    grid: std::collections::HashMap<(i32, i32), Vec<(usize, AABB)>>,  // (grid_x, grid_y) -> items
    stats: SpatialStats,
}

impl BroadPhaseGrid {
    /// Create grid with specified cell size
    pub fn new(cell_size: f64) -> Self {
        assert!(cell_size > 0.0, "cell_size must be positive");
        BroadPhaseGrid {
            cell_size,
            grid: std::collections::HashMap::new(),
            stats: SpatialStats::default(),
        }
    }

    /// Convert world coordinates to grid coordinates
    fn world_to_grid(&self, coord: f64) -> i32 {
        (coord / self.cell_size).floor() as i32
    }

    /// Get all grid cells occupied by AABB
    fn get_cells(&self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Vec<(i32, i32)> {
        let gx_min = self.world_to_grid(min_x);
        let gx_max = self.world_to_grid(max_x);
        let gy_min = self.world_to_grid(min_y);
        let gy_max = self.world_to_grid(max_y);

        let mut cells = Vec::new();
        for gx in gx_min..=gx_max {
            for gy in gy_min..=gy_max {
                cells.push((gx, gy));
            }
        }
        cells
    }

    /// Add item with AABB (minx, maxx, miny, maxy)
    pub fn add(&mut self, id: usize, min_x: f64, max_x: f64, min_y: f64, max_y: f64) {
        let bbox = AABB::new(min_x, max_x, min_y, max_y);
        let cells = self.get_cells(min_x, max_x, min_y, max_y);
        
        for cell in cells {
            self.grid.entry(cell).or_insert_with(Vec::new).push((id, bbox));
        }
    }

    /// Get all candidates overlapping with given AABB (with deduplication)
    pub fn query(&mut self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Vec<usize> {
        let query_box = AABB::new(min_x, max_x, min_y, max_y);
        let cells = self.get_cells(min_x, max_x, min_y, max_y);
        
        let mut candidates = std::collections::HashSet::new();
        for cell in cells {
            if let Some(items) = self.grid.get(&cell) {
                for (id, bbox) in items {
                    self.stats.bbox_tests += 1;
                    if bbox.overlaps(&query_box) {
                        candidates.insert(*id);
                        self.stats.bbox_overlaps += 1;
                    }
                }
            }
        }
        
        candidates.into_iter().collect()
    }

    pub fn len(&self) -> usize {
        self.grid.values().map(|v| v.len()).sum()
    }

    pub fn clear(&mut self) {
        self.grid.clear();
        self.stats = SpatialStats::default();
    }

    pub fn stats(&self) -> &SpatialStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_overlap() {
        let a = AABB::new(0.0, 2.0, 0.0, 2.0);
        let b = AABB::new(1.0, 3.0, 1.0, 3.0);
        assert!(a.overlaps(&b));

        let c = AABB::new(3.0, 4.0, 0.0, 2.0);
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn test_broad_phase_flat_query() {
        let mut bp = BroadPhaseFlat::new();
        bp.add(1, 0.0, 2.0, 0.0, 2.0);
        bp.add(2, 1.0, 3.0, 1.0, 3.0);
        bp.add(3, 5.0, 6.0, 5.0, 6.0);

        let candidates = bp.query(0.5, 2.5, 0.5, 2.5);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&1));
        assert!(candidates.contains(&2));
    }

    #[test]
    fn test_broad_phase_grid_query() {
        let mut bp = BroadPhaseGrid::new(1.0);
        bp.add(1, 0.0, 0.5, 0.0, 0.5);
        bp.add(2, 0.8, 1.5, 0.8, 1.5);
        bp.add(3, 5.0, 6.0, 5.0, 6.0);

        let candidates = bp.query(0.2, 1.2, 0.2, 1.2);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&1));
        assert!(candidates.contains(&2));
    }
}
