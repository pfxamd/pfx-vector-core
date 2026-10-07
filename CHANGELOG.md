# Changelog

## 0.2.0 - 2026-10-07

### Added

- Advanced intersections for quadratic-quadratic, quadratic-cubic, cubic-cubic, Bézier-arc, and arc-arc pairs.
- Stable intersection parameters on both input segments with deterministic result ordering.
- Crossing, tangent, and endpoint classification for curve intersections.
- Full overlap detection for equivalent/reversed quadratic and cubic Bézier segments.
- Partial overlap ranges for compatible elliptical arcs sharing the same ellipse geometry.
- Analytical elliptical-arc bounds using actual extrema instead of fixed-angle sampling.
- Robustness tests for non-dyadic tangencies, near-tangent misses, argument-order symmetry, and large translated coordinates.
- Advanced intersection benchmark coverage.

### Changed

- Line-arc intersections now use the same parameter-space intersection engine instead of flattened line approximations.
- Curve-pair search uses recursive parameter subdivision and tight bounds, followed by local Newton refinement.
- Intersection acceptance uses strict numerical residual tolerance separately from broader search tolerance, preventing flatness settings from turning near misses into intersections.
- Tangent neighborhoods are coalesced around confirmed multiple-root events to avoid duplicate near-tangent candidates.

### Scope limits

- General partial overlap detection for differently parameterized Bézier curves remains outside this release.
- Boolean operations, general path offsets, tessellation, rendering, and scene-graph/editor concerns remain deferred.

## 0.1.0 - 2026-10-07

### Added

- Safe-Rust vector geometry kernel with explicit tolerance handling.
- Point/vector/matrix primitives and affine transforms.
- Basic geometry, Bézier curves, elliptical arcs, bounds, and paths.
- Measurement, flattening, nearest-point, fill, stroke, hit-testing, and basic intersection engines.
- SVG path parser, normalization, serialization, transform parsing, and viewBox mapping.
- WebAssembly bridge and TypeScript wrapper.
- Property tests, regression/specification tests, fuzzing corpus/target, and benchmarks.
- Project-local PFx Naming System guidance for Rust and TypeScript contributors.

### Scope limits

- Boolean operations, offsets, tessellation, rendering, scene graphs, and advanced curve-curve intersections are deferred beyond v0.1.
