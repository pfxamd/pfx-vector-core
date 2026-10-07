# Changelog

## 0.4.0 - 2026-10-07

### Added

- Closed-path geometric offsets through `offset_path` with positive outward and negative inward distances.
- Explicit fill-rule offsets through `offset_path_with_fill_rule`, including validated `EvenOdd` hole behavior.
- Stroke-to-filled-path conversion through `outline_path` and `stroke_to_path`.
- `OffsetStyle` with miter, round, and bevel join selection plus miter limits.
- Butt, round, and square cap geometry for open-path outlines.
- Exact line offsets and exact circular-arc offsets.
- Adaptive cubic approximation for quadratic Bézier, cubic Bézier, and non-circular elliptical-arc offsets.
- Boolean cleanup of per-segment ribbons and join/cap components, allowing crossing open centerlines to resolve into clean filled outline areas.
- WebAssembly exports for SVG path offset and outline operations.
- TypeScript `offsetPath` and `outlinePath` APIs with typed cap/join options.
- Criterion benchmarks for rectangle offset and cubic-path outlining.
- Robustness coverage for holes, `EvenOdd` contours, large coordinates, inward collapse, curved geometry, miter fallback, closed rings, and crossing centerlines.

### Changed

- Offset area construction reuses the Boolean topology engine from v0.3 for outward union and inward difference.
- Full circular stroke outlines retain native arc geometry when possible.
- General curve offsets are fitted adaptively against geometric error tolerance instead of using a fixed segment count.

### Scope limits

- `offset_path` requires closed input subpaths.
- Dashed stroke expansion is not part of v0.4; non-empty dash arrays return `UnsupportedCase` from the outline engine.
- Mathematical offsets of Bézier curves and general ellipses are not themselves Bézier/ellipse primitives, so v0.4 represents them as adaptive cubic approximations within tolerance.
- Severe cusps or degenerate tangents may return `DegenerateOperation` or `ToleranceNotMet` rather than invent unstable geometry.
- General self-intersecting filled-contour normalization remains outside the guaranteed offset scope.
- Tessellation, rendering, and scene-graph/editor concerns remain deferred.

## 0.3.0 - 2026-10-07

### Added

- Curve-preserving Boolean operations: `Union`, `Intersection`, `Difference`, and `XOR`.
- Public Rust APIs: `boolean_union`, `boolean_intersection`, `boolean_difference`, `boolean_xor`, and the general `boolean_paths` entry point.
- Explicit fill-rule Boolean API through `boolean_paths_with_fill_rules`.
- Path fragmentation at exact intersection parameters while retaining line, quadratic Bézier, cubic Bézier, and elliptical-arc segment types.
- Boundary-side topology classification: each fragment is tested on both sides and retained only when the Boolean set membership changes across it.
- Directed contour reconstruction with output interior consistently oriented to the left of retained fragments.
- Coincident-fragment deduplication and shared-edge removal.
- WebAssembly Boolean exports for SVG path strings.
- TypeScript wrapper methods for union, area intersection, subtraction, and XOR.
- Criterion benchmark coverage for Boolean union.
- Robustness tests covering shared boundaries, containment holes, identical/disjoint inputs, reversed orientation, `evenodd` holes, tangent circles, large translated coordinates, and commutative operations.

### Changed

- Boolean topology builds directly on the advanced intersection engine from v0.2 rather than flattening curves into polygonal approximations.
- Closed-path implicit closing edges participate in Boolean intersection/splitting even when they are not stored as explicit line segments.
- Boolean output converts source fill semantics into explicit directed contours that can be consumed with the standard nonzero rule.

### Scope limits

- Boolean operations currently require closed input subpaths; open paths return `UnsupportedCase`.
- General normalization of self-intersecting input contours is not yet part of the validated Boolean scope.
- General partial overlap detection for differently parameterized but geometrically identical Bézier curves remains limited by the v0.2 intersection overlap model.
- General offsets, tessellation, rendering, and scene-graph/editor concerns remain deferred.

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
