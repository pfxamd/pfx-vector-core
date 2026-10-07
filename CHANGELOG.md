# Changelog

## 0.8.0 - 2026-10-07

### Added

- Deterministic AABB spatial indexing through public `SpatialIndex`.
- Prefix-max sweep indexing for bounded queries without third-party runtime dependencies.
- Public `spatial_self_candidate_pairs` and `spatial_cross_candidate_pairs` helpers for broad-phase geometry pruning.
- Reusable `PathSpatialIndex` built from one adaptive flattening pass.
- Indexed fill classification and containment queries on `PathSpatialIndex`.
- Indexed stroke hit testing using local bounding-box candidate queries.
- Indexed closest-point queries with expanding-radius broad phase and exact line-edge refinement.
- New `Bounds::intersects`, `Bounds::expanded`, and `Bounds::distance_squared_to_point` helpers.
- Robustness tests comparing spatial candidate results to brute force, including 5000 sparse bounds and large translated coordinates.
- Differential tests matching indexed fill, stroke, and nearest queries against the existing algorithms.
- Criterion benchmark coverage for 10000-bound index construction, sparse self-candidate pruning, and indexed nearest queries.

### Changed

- Self-intersection normalization now prunes segment-pair tests by exact segment bounds before invoking the advanced intersection engine.
- Boolean operations now prune cross-path segment pairs by AABB overlap before exact curve intersection.
- Boolean and normalization side probes reuse a single `PathSpatialIndex` instead of repeatedly flattening and scanning the whole path.
- WebAssembly `intersectPaths` now applies the same broad-phase pruning before exact segment intersection.
- Spatial candidate ordering is normalized to deterministic item-index order.

### Scope limits

- `SpatialIndex` is immutable; callers rebuild it after geometry changes.
- `PathSpatialIndex` indexes adaptively flattened edges, so fill/stroke/nearest accuracy follows the supplied `Tolerance.flatness`.
- Spatial pruning is a broad phase only; exact geometric decisions remain in the existing intersection and topology engines.
- The current index is optimized for deterministic CPU geometry workloads, not concurrent mutation or GPU-resident scene structures.

## 0.7.0 - 2026-10-07

### Added

- Public self-intersection normalization through `normalize_self_intersections`.
- Pairwise boundary splitting across line, quadratic Bézier, cubic Bézier, and elliptical-arc segments using the existing advanced intersection engine.
- Analytical detection and parameter splitting for intrinsic self-intersections inside a single cubic Bézier segment.
- Fill-rule-aware topology resolution for both `NonZero` and `EvenOdd`.
- Directed boundary reconstruction with filled space consistently kept on the left side of emitted contours.
- Face-walking continuation at multi-edge intersection nodes, preventing a normalized contour from simply recreating the original crossing.
- Exact overlap cleanup for validated coincident fragments, including repeated and oppositely wound contours.
- Stable handling of bow-tie contours, double winding, touching lobes, nested contours, large translated coordinates, reversed input orientation, and intrinsic cubic loops.
- Automatic contour normalization before Boolean operations, filled offsets, and fill tessellation.
- WebAssembly `normalize_fill_contours_svg` export.
- TypeScript `normalizeFillContours` API.
- Criterion benchmark coverage for self-intersecting contour normalization.

### Changed

- Boolean inputs are normalized to explicit NonZero-oriented boundary contours before cross-path Boolean fragmentation.
- Filled offsets normalize their source fill topology before outline construction and set operations.
- Fill tessellation now consumes normalized simple contours instead of rejecting validated self-intersecting fills.
- Native curve primitives are preserved through normalization; curves are split into subsegments rather than flattened.

### Scope limits

- Normalization currently requires closed fill subpaths.
- Proper intrinsic self-intersection is analytically handled for cubic Bézier segments; multi-revolution or retraced single elliptical arcs remain outside the guaranteed normalization scope.
- General partial overlap detection for differently parameterized but geometrically identical Bézier curves remains limited by the intersection engine.
- Extremely degenerate cusp/retrace configurations can return an explicit numerical or unsupported-case error rather than inventing topology.
- Open stroke topology is not normalized by this fill-contour API.

## 0.6.0 - 2026-10-07

### Added

- Path cleanup through `cleanup_path` with explicit `CleanupOptions` for point and collinearity tolerances.
- Duplicate-point removal and collinear line-chain reduction for open and closed polylines.
- Conversion of nearly linear quadratic and cubic Bézier segments into exact line segments when their deviation is within the requested cleanup tolerance.
- Path simplification through `simplify_path` using deterministic Ramer-Douglas-Peucker reduction.
- Closed-contour simplification by splitting the ring across a deterministic farthest-point pair instead of treating closure as an ordinary duplicated endpoint.
- Cubic Bézier fitting through `fit_polyline_cubics` and path-level `fit_path_curves`.
- Chord-length parameterization, least-squares control-distance solving, Newton reparameterization, and recursive split fitting.
- Defined recursion limits and explicit failure instead of unbounded fitting.
- Stable fitting for translated large coordinates, reversed input samples, sharp corners, closed contours, and dense sampled curves.
- WebAssembly exports for cleanup, simplification, and cubic curve fitting.
- TypeScript `cleanupPath`, `simplifyPath`, and `fitPathCurves` APIs.
- Criterion benchmarks for dense polyline cleanup, simplification, and cubic fitting.

### Changed

- Cleanup preserves existing non-degenerate curve primitives unless they are demonstrably line-like within the requested tolerance.
- Simplification is explicitly a polyline operation: source curves are adaptively flattened before point reduction.
- Curve fitting is explicitly approximate: source paths are sampled through adaptive flattening, then reconstructed as cubic Bézier segments within the requested fitting error.
- Numerical distance tests use local geometry rather than world-coordinate magnitude where translation should not affect the result.

### Scope limits

- General topology-preserving simplification of arbitrary self-intersecting contours is not guaranteed.
- `simplify_path` returns line geometry rather than attempting to preserve original Bézier or arc segment types.
- `fit_path_curves` outputs cubic Bézier geometry; exact circles and elliptical arcs are not rediscovered as arc primitives.
- The fitting error contract is validated against sampled source geometry; it is not a symbolic proof of Hausdorff distance between arbitrary analytical curves.
- Smoothing/beautification rules that intentionally alter corners are outside the cleanup API.

## 0.5.0 - 2026-10-07

### Added

- Native fill tessellation through `tessellate_fill`, producing indexed `Mesh2D` triangle data.
- Stroke tessellation through `tessellate_stroke`, reusing the v0.4 outline engine before triangulation.
- Concave polygon triangulation through deterministic ear clipping.
- Hole elimination by explicit contour grouping and bridge construction before triangulation.
- `NonZero` and `EvenOdd` fill semantics, including redundant nested contours, multiple holes, and islands inside holes.
- Positive-winding triangle output with validated index ranges.
- Local-coordinate area predicates to remain stable under large world-coordinate translations.
- Adaptive curve flattening at the tessellation boundary using the existing `Tolerance.flatness` contract.
- WebAssembly mesh exports for SVG fill and stroke tessellation.
- TypeScript `tessellateFill` and `tessellateStroke` APIs returning flat vertex buffers and `u32`-style index arrays.
- Criterion benchmarks for rectangle fill, curved fill, and stroke tessellation.

### Changed

- Tessellation deliberately converts curves to piecewise-linear contours only at the mesh-generation boundary; source path geometry remains unchanged.
- Boundary orientation is derived from actual fill semantics instead of trusting source contour winding alone.
- Ear validation treats points on candidate ear diagonals as blockers, preventing invalid clipping in concave polygons.

### Scope limits

- Fill tessellation currently requires closed subpaths.
- General self-intersecting contour normalization is not guaranteed; ambiguous contour-side classification can return `UnsupportedCase`.
- Mesh output is 2D indexed triangles only; GPU buffers, shaders, antialiasing, fringe geometry, and renderer-specific vertex attributes are outside the core.
- Tessellation accuracy for curves is controlled by flattening tolerance and is therefore approximate by design.
- Dashed stroke tessellation remains limited by the current outline engine, which defers dashed expansion.

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
