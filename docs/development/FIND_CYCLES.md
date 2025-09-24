
# Finds cycles in an undirected graph.

Graph detials:
- Graph is unidirected
- Each vertex can have up to 8 edges.
- Edges are part of graph, but they have `real 2D geometry`.
- Some edges can be `double`, meaning a <-> b edge can be present twise, however since
they may have different `real word geometry`, they cannot be merdged. For example, if we 
have a cyrcle and two points on it, then between points a <-> b, there are two arcs (ednges in graph), but they are not the same.
- If edges a part of multiple cycles, we are looking to extract them in such way that the resulting
cycles does not intersect geometricaly.



1. Vertex Representation: Each vertex is the end of an togo::Arc structure. This structure is 2D geometric representation of an arc (also can be line segment when 0 curvature). 
The edges are 2D Arc-s. The Arc is from togo::Arc (togo package). 
One end of arc is `a` and the other `b` point in 2D space. We will first, need an algorithm to identify
what `a` and `b` are close in 2D space, so we can assume they are the same. /src/graph/merge_ends.rs

2. Edge Data Structure: Each edge in the Graph is a Arc in 2D space.

3. Double Edges: Each edge is a Arc, so there can be two Arc-s between point `a` and `b`: Arc1(a, b) Arc2(b,a) for example.

4. Component Priority: This may require more precise definition in the process of the development. 
Larger componenent seems to be larger in area, there are methods for area of closed component in `togo` lib.

5. Component Types: I am looking for cycles only, because this is usualy `tool path` for cutting parts from a sheet of material.

6. Edge Sharing: I may not be correct about this.
The problem is in cases like digital version of `8` we do not want a cycle that self intersects (in 2D geometry), 
but with two `0`-s cycle, or the one that traverses always the outer side.
The togo library have a lot of geometric primitives, however for this task we mayhave to implement some more.

7. Integration: The existing implementation finds a offset of polygons that are composed of togo::Arc-s
On a last step the algorithm have a set of arcs that are part of offset polygon (there can be multiple
offset polygon). For the one polygon case, the things are easy, however for multiple polygons, we need 
to identify the cycles and assign arcs in each cycle to a separate polygon.

The arcs a split parts that due to numerical error may not exactly match in the end points.
Hense the necessitity as a first step to identify the common points (close points in 2D) before finding the cycles.

8. Performance Requirements: Lets start with simple implementation. Latter when we integrate it in
the existing code, use real examples, fix all bugs and and cover with test, we can thinkg of other implementations.

## Q&A Session for Implementation Details

### Questions about `merge_ends.rs`:

**Q1. Tolerance/Epsilon**: What tolerance should we use for considering two points "close enough" to be merged?
**A1**: Related to numerical precision of offset calculations. Use a fixed constant for now, implement more sophisticated solution later.

**Q2. Merging Strategy**: How should we merge close endpoints?
**A2**: Move endpoints to the midpoint of the group (can be many points, but no more than 8). May also need to eliminate very small arcs created during offset process.

**Q3. Arc Modification**: Should we create new Arc instances or modify existing ones?
**A3**: Use togo package function to `adjust` Arcs in place for consistency.

### Questions about `find_cycles.rs`:

**Q4. Geometric Intersection Detection**: How to handle geometric intersections?
**A4**: Arcs should have no 2D intersections (created by offset algorithm), but graph paths can intersect at vertices (like "X" shape). At such vertices, there are 3 possible traversal ways: 2 non-intersecting and 1 intersecting. All equivalent from graph perspective but different in 2D space.

**Q5. Cycle Selection Priority**: How to choose between overlapping cycles?
**A5**: Choose the way that splits cycles into geometrically non-intersecting cycles. Size doesn't matter. Arcs are created so they can always be separated into geometrically non-intersecting cycles.

**Q6. Direction Consistency**: Should cycles have consistent orientation?
**A6**: After cycles are found, create separate algorithm to orient paths CCW. Offset algorithm works by finding "right" offset of CCW polygon.

### Questions about Data Structures:

**Important Note**: There are 2 arcs on a circle between two points. `togo::Arc` works only with CCW arcs. If we have Arc with points a, b, the arc is the CCW one.

**Q7. Graph Representation**: What data structure to use?
**A7**: Up to implementer, whatever best suits the algorithms.

**Q8. Input/Output Format**: What should the interfaces be?
**A8**: 
- Input: `fn merge_close_endpoints(arcs: &mut Vec<Arc>, tolerance: f64) -> ()`
- Output: `fn find_non_intersecting_cycles(arcs: &[Arc]) -> Vec<Vec<Arc>>` is OK, but split into smaller, affordable functions rather than one large function doing all heavy lifting.

## Additional Questions:

**Q9. Small Arc Elimination**: Should this be part of merge_ends.rs and how to define "small"?
**A9**: Should be part of `merge_ends.rs`. Small is relative to `tolerance` parameter. Should be max of (a-b distance, radius of the arc). Can invent more precise definition later.

**Q10. X-Vertex Traversal**: How to decide which traversal option to choose at X-intersection?
**A10**: When following one edge toward the vertex, take the most close on the right to exit the vertex - this way we don't intersect.

**Q11. togo::Arc Interface**: What does the Arc structure look like?
**A11**: Available at: https://github.com/radevgit/togo/blob/main/src/arc.rs

**Q12. Function Decomposition**: Good decomposition for cycle finding?
**A12**: Yes, something like the proposed structure.

**Q13. Testing Strategy**: Should we create test cases?
**A13**: Yes, definitely. Use Arc methods to create from:
- Points coordinates and radius
- Two points and bulge
- `arcseg()` for line arcs
- `is_valid()` exists but may change, not reliable now
- Helpful functions: `arc_bulge_from_points` and `arc_circle_parametrization` (reverse functions)

**Q14. Arc Adjustment**: Is `make_consistent()` the function to use for adjusting arcs?
**A14**: Yes, `make_consistent()` can adjust a little the arc to make it consistent.

**Q15. Tolerance Value**: Should we use `EPS_COLLAPSED = 1E-8` as default tolerance?
**A15**: For now use 1E-8 and make different parameter than EPS_COLLAPSED. Can investigate later what are proper values.

**Q16. Small Arc Definition**: Should we check `max(chord_length, radius)` for small arc detection?
**A16**: Yes, right. Arc can be small in both a-b distance and radius size. For line segments (`arc.r == f64::INFINITY`) check a-b distance only. Merge algorithm for points maybe tricky.

## Implementation Status:

### Completed Features:

#### /src/graph/merge_ends.rs
- ✅ **IMPLEMENTED**: `merge_close_endpoints(arcs: &mut Vec<Arc>, tolerance: f64)` - Algorithm for finding and merging close endpoints of Arcs
- ✅ **IMPLEMENTED**: `merge_close_endpoints_default()` - Convenience function using MERGE_TOLERANCE constant (1E-8)
- ✅ **IMPLEMENTED**: Small arc elimination based on max(chord_length, radius) criterion
- ✅ **TESTED**: Comprehensive test coverage for point merging and small arc removal

#### /src/graph/find_cycles.rs
- ✅ **IMPLEMENTED**: `find_non_intersecting_cycles(arcs: &[Arc]) -> Vec<Vec<Arc>>` - Main cycle detection algorithm
- ✅ **IMPLEMENTED**: Geometric "rightmost" edge selection using proper tangent calculations via `togo::Arc::tangents()`
- ✅ **IMPLEMENTED**: `get_arc_direction_at_vertex()` - Proper tangent-based direction calculation for both arcs and line segments
- ✅ **IMPLEMENTED**: Deferred edge marking to prevent premature cycle truncation
- ✅ **FIXED**: Critical bug where edges were marked as used before cycle completion, causing incomplete cycles
- ✅ **TESTED**: Extensive test suite including:
  - Basic cycle detection with line segments
  - Mixed arc/segment cycles with proper tangent-based selection
  - Edge cases with close endpoints requiring merging
  - Integration tests combining merge_ends and find_cycles functionality

#### /src/graph/find_cycles_tangent_tests_simple.rs
- ✅ **CREATED**: Focused test suite specifically for tangent-based functionality
- ✅ **TESTED**: Real circular arc geometry tests using proper `arc(center, radius)` API
- ✅ **VALIDATED**: Tangent-based rightmost selection with actual geometric verification
- ✅ **COVERAGE**: 9 comprehensive test cases covering various arc configurations

### Key Technical Achievements:

1. **Tangent-Based Direction Calculation**: Replaced flawed endpoint-based calculations with proper tangent vectors using `togo::Arc::tangents()` method, which returns `[Point; 2]` representing entry and exit tangent vectors.

2. **Geometric Correctness**: The "rightmost" edge selection now uses actual geometric tangent directions rather than endpoint positions, ensuring correct traversal paths that avoid geometric intersections.

3. **Robust Cycle Detection**: Fixed critical algorithm bug where edges were marked as used during cycle search instead of after cycle completion, preventing incomplete cycle detection.

4. **Comprehensive Testing**: Created focused test framework using real geometric arcs with `togo::arc()` and `togo::arcseg()` convenience functions, validating both algorithmic correctness and geometric behavior.

5. **API Integration**: Properly integrated with `togo` library using:
   - `arc.tangents()` for direction calculations
   - `arcseg(p1, p2)` for line segment creation  
   - `arc(center, radius)` for circular arc creation
   - Standard `make_consistent()` for arc adjustments

### Development Notes:

- The algorithm successfully handles mixed arc/line segment graphs
- Tangent-based calculations work correctly for both finite-radius arcs and infinite-radius line segments
- All tests pass (36 total: 27 for general cycles, 9 for tangent-specific functionality)
- Code compiles cleanly with no warnings
- Integration between merge_ends and find_cycles works seamlessly

### Previous Experimental Work:
- `/src/offset_reconnect_arcs.rs` contains earlier experiments but is not used in the final implementation
- Main development focused in `graph` directory as specified 

