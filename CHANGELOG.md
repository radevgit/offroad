# Changelog

## [0.4.2] - 2025-10-26
- Implement spatial index for the prune phase

## [0.4.1] - 2025-10-25
- fixed offset for arclines
- added benchmarks

## [0.4.0] - 2025-10-25
- Fixed negative bulge arc offsetting bugs
- Improved bulge sign inference from arc connectivity
- Cyclic loop refactoring in poly_to_raws_single and arcs_to_raws_single
- Removed debug print statements

## [0.3.0] - 2025
- Graph-based cycle detection algorithm
- Arc reconnection system for offset segments
- Phase 2-3 refinements

## [0.2.0] - 2025
- Enhanced reconnection logic
- Multiple component handling improvements

## [0.1.0] - 2025-09-24
- Implemented graph-based cycle detection algorithm
- Added arc reconnection system for offset segments
- Migrated to Togo library for improved geometry handling

## [v0.06] - 2025
- Fixed negative bulge arc offsetting bugs
- Improved bulge sign inference from arc connectivity
- Fixed offset negation issue for negative bulges

## [v0.05] - 2024
- Refactored offset configuration with SVG output flags
- Added `offset_arcline` example
- Renamed examples for clarity (`offset_simple` → `offset_polyline`)
- Improved benchmark organization
- Stabilized API for production use

## [v0.0.4] - 2024
- Enhanced arc offset calculations
- Improved numerical precision

## [v0.0.3] - 2024
- Added support for multiple arc configurations
- Improved numerical stability in offset calculations

## [v0.0.2] - 2024
- Initial release with basic offset functionality
- Foundation for polyline and arc offsetting