# Changelog

## 0.24.0 - 2026-10-08

### Stabilization and verification

- Fixed measure-index nonconvergence on valid cubic Bézier and elliptical arc inputs, and preserved exact handling of retraced collinear Béziers.
- Replaced repeated prefix integration with local adaptive interval integration, significantly reducing curve and arc index build times without weakening the regression suite.
- Added bounded deterministic load, invalid-input, extreme-coordinate, randomized geometry, minimized-convergence, and SVG-to-core integration tests.
- Added seeded libFuzzer smoke targets for SVG and measure-index behavior, with 10,000 executions per target in CI.
- Added native WebAssembly execution tests in Node.js, in addition to the existing target build and TypeScript type checks.
- Added reproducible Linux release benchmarks recording index construction, queries, peak RSS, and a 1,000-cycle repeated-build soak test.

### Verified scope and limitations

- CI and performance validation cover specified representative workloads, not all possible hostile inputs.
- Fuzz testing is bounded; extended multi-hour campaigns remain appropriate before embedding in high-risk untrusted-file pipelines.
- SVG serialization with the default six decimal places can alter geometry of nearly degenerate arcs. Use a larger `SerializeOptions.precision` where geometric round-trip accuracy is required.
- WebAssembly runtime validation currently targets Node.js; interactive browser/editor validation remains a downstream integration responsibility.
- The TypeScript package remains private and is not published to a package registry.

## 0.23.0 - 2026-10-08

### Added

- Public `SegmentMeasureTable` for reusable per-segment arc-length parameterization.
- Public `PathMeasureIndex` with cumulative source-segment offsets.
- Bidirectional `distance → PathLocation` and `(subpath, segment, t) → path distance` queries.
- Indexed point-at-length and differential-frame queries without repeated adaptive integration after index construction.
- Adaptive cumulative arc-length tables for quadratic Bézier, cubic Bézier, and elliptical-arc segments.
- Exact two-sample measurement tables for line segments.
- Monotone cubic Hermite interpolation and bounded inverse solving within adaptive table intervals.
- WebAssembly batch point/frame queries and reverse path-distance lookup.
- TypeScript `pointsAtLengths`, `framesAtLengths`, and `pathDistanceAt` APIs.
- Regression coverage for curve round-trips, elliptical arcs, retraced collinear curves, large translated coordinates, invalid locations, empty paths, and indexed/direct agreement.
- Criterion benchmark coverage comparing repeated direct and indexed point-at-length queries.

### Changed

- Line-like Bézier measurement classification now uses local geometry scale instead of absolute world-coordinate magnitude, making it translation-invariant.
- Adaptive measurement-table accuracy now respects both the core arc-length tolerance and the meaningful `f64` precision at large coordinate magnitudes.
- One-shot measurement APIs remain unchanged for inexpensive single-query use; reusable indexing is explicit.

### Scope limits

- `PathMeasureIndex` is an immutable geometry snapshot and must be rebuilt after path changes.
- Curve arc-length inversion is bounded numerical interpolation rather than a symbolic closed-form inverse.
- Table accuracy inherits the core numerical arc-length model and floating-point coordinate precision.
- Incremental editor cache ownership and synchronization remain outside the geometry core for this milestone.

## 0.22.0 - 2026-10-08

### Added

- Public `SegmentFrame`, `PathFrame`, and `PathInflection` geometry result types.
- Public `segment_frame_at_t` and `path_frame_at_length` APIs.
- Unified `Segment::derivative_at` and `Segment::second_derivative_at` primitives.
- Analytic second derivatives for quadratic Bézier, cubic Bézier, and elliptical-arc primitives.
- Signed curvature, unit tangent, left-hand normal, and parameter-speed reporting for regular segment points.
- `segment_inflection_parameters` for regular interior cubic Bézier inflections.
- `path_inflections` with original source location and path-distance coordinates.
- WebAssembly `frame_at_length_svg` and `inflection_points_svg`.
- TypeScript `frameAtLength` and `inflectionPoints`.
- Regression coverage for lines, quadratic curvature, circular-arc curvature, reversal semantics, cubic inflections, degenerate tangents, endpoints, path locations, and large translated coordinates.
- Criterion benchmark coverage for differential frames and cubic inflection solving.

### Changed

- The offset engine now consumes the shared segment derivative and second-derivative primitives instead of maintaining duplicate differential formulas.
- `tangent_at_length` delegates to the unified path-frame engine.
- Arc-length inversion now canonicalizes exact endpoints so zero distance returns `t = 0` and the segment total returns `t = 1`.

### Scope limits

- Frames require a regular non-degenerate tangent; zero or tolerance-degenerate speed returns `DegenerateOperation`.
- Inflection reporting currently covers regular interior sign-changing curvature roots of cubic Bézier segments.
- The reported normal is the left-hand perpendicular of the oriented tangent; signed curvature follows the same path orientation convention.
- Path-distance coordinates retain the existing tolerance-controlled numerical arc-length model.

## 0.21.0 - 2026-10-07

### Added

- Coincident `Line ↔ Quadratic Bézier` and `Line ↔ Cubic Bézier` overlap detection.
- Multiple overlap ranges for collinear Béziers that reverse direction and retrace a finite line interval.
- Intrinsic overlap split parameters for multi-revolution elliptical arcs.
- Intrinsic split parameters for line-like quadratic/cubic Béziers with repeated projected positions.
- Geometry-based straight-fragment deduplication independent of Bézier parameterization.
- Exact total-variation arc length for collinear quadratic/cubic Béziers, split at derivative roots.
- Regression coverage for line/curve argument symmetry, retraced cubic overlap, near-collinear rejection, multi-turn `NonZero / EvenOdd` normalization, line-vs-nonlinearly-parameterized Bézier Boolean boundaries, retraced cubic topology cleanup, and spatial-index construction over retraced curves.
- Criterion benchmark coverage for retraced line/cubic overlap queries.

### Changed

- `normalize_self_intersections` now applies intrinsic self-overlap splitting before pairwise segment intersection splitting.
- Multi-revolution arcs can normalize repeated winding directly instead of remaining outside guaranteed contour-normalization scope.
- Straight Bézier fragments with matching directed endpoints can deduplicate even when their internal parameterizations differ.
- `segment_length` avoids adaptive integration for line-like Béziers and computes exact one-dimensional total variation instead.
- `PathSpatialIndex::build` therefore no longer fails on validated retraced collinear Béziers while computing segment distance coordinates.

### Scope limits

- Intrinsic self-overlap support covers multi-revolution elliptical arcs and line-like Bézier retraces; arbitrary non-collinear coincident subranges within a single Bézier are not claimed.
- Existing analytic intrinsic cubic self-crossing support remains separate from coincident self-retrace handling.
- Periodic intrinsic arc splitting is bounded and returns `IterationLimit` for pathological revolution counts.
- Point-collapsed degenerate primitives remain governed by existing degenerate-operation behavior.

## 0.20.0 - 2026-10-07

### Added

- Partial overlap detection for quadratic and cubic Bézier segments.
- Overlap detection when neither Bézier segment fully contains the other.
- Reversed partial Bézier overlap support with deterministic parameter ranges.
- Quadratic-to-cubic degree-elevation equivalence for coincident overlap detection.
- Multi-range periodic overlap reporting for multi-revolution elliptical arcs.
- Canonical circle handling across different stored rotations.
- Equivalent ellipse handling across swapped radii and corresponding quarter-turn rotation changes.
- Bounded computed periodic arc alignment search replacing the previous fixed turn window.
- Regression coverage for overlap symmetry, large translated coordinates, full circles with different seams, swapped ellipse axes, reversed multi-revolution arcs, and Boolean topology with partially shared curved boundaries.
- Criterion benchmark coverage for partial cubic overlap queries.

### Changed

- `intersect_segments` may return multiple `Intersection::Overlap` entries when one periodic arc range maps to multiple source-parameter intervals on the other arc.
- Coincident Bézier detection no longer requires identical full control polygons.
- Boolean and contour-normalization splitting automatically consume the stronger overlap ranges without an API migration.
- Near-endpoint overlap parameters are canonicalized through the existing intersection parameter snapping rules.

### Scope limits

- Bézier overlap recognition covers common polynomial equivalence under affine subranges, reversal, and exact quadratic degree elevation; arbitrary non-affine reparameterizations are not claimed.
- Intrinsic self-retrace inside one single Bézier or elliptical-arc segment remains outside the pairwise overlap API.
- Arc overlap alignment is explicitly bounded at 4096 periodic alignments and returns `IterationLimit` beyond that bound.
- Line-versus-curved coincident overlap is not expanded by this milestone.

## 0.19.0 - 2026-10-07

### Added

- Source-native fill and winding classification for line, quadratic Bézier, cubic Bézier, and elliptical-arc segments.
- Native boundary refinement through `closest_point_on_segment` instead of flattened-edge boundary checks.
- Direct quadratic and cubic y-root solving for horizontal-ray crossings.
- Native elliptical-arc ray solving over the actual source sweep.
- Tolerance-scaled ray perturbation for stable vertex and horizontal-tangency handling.
- Strict source-parameter root ownership that prevents near-endpoint numerical roots outside `[0, 1]` from creating false crossings.
- Conservative native-fill broad phases in `PathSpatialIndex` and `IncrementalPathSpatialIndex`.
- Regression coverage for coarse flatness, curve boundaries, tangencies, full-circle seams, vertex-aligned rays, large coordinates, indexed refinement, incremental synchronization, curved Booleans, and offset/outline cleanup.
- Criterion benchmark coverage for direct and indexed native elliptical-arc fill classification.

### Changed

- `classify_point` and `contains_point` no longer flatten source curves for final fill classification.
- `NonZero` and `EvenOdd` winding decisions are now independent of `Tolerance.flatness`.
- Spatial fill indexes use bounds only as a broad phase and share the same final native classifier as direct queries.
- Boolean side probes, contour normalization, filled offset cleanup, and stroke-component fill tests inherit source-native fill semantics without an API migration.

### Scope limits

- Native curve crossings remain bounded floating-point numerical geometry rather than symbolic exact arithmetic.
- `Tolerance` still controls boundary proximity and numerical robustness.
- Open subpaths retain their existing open-fill semantics; only closed contours receive an implicit closing edge.
- Pathological retraces and self-intersection topology remain governed by the existing normalization/intersection scope limits.

## 0.18.0 - 2026-10-07

### Added

- Curvature-aware adaptive offset fitting for quadratic Bézier, cubic Bézier, and non-circular elliptical-arc geometry.
- Analytic offset derivatives derived from source tangent, second derivative, and signed curvature.
- Detection and bounded splitting around offset singularities where the offset speed factor changes sign or approaches zero.
- Denser in-interval offset validation before accepting fitted cubic segments.
- Precision regression tests measured against source-native offset points through the v0.17 nearest-point engine.
- Large-coordinate curved-offset regression coverage.
- Criterion benchmark coverage for precision elliptical-arc offsets.

### Changed

- Adaptive curve offsets no longer estimate offset derivatives with finite differences.
- Offset fitting now uses native source derivatives and curvature to place cubic handles.
- Cubic and elliptical offsets subdivide more conservatively around high-curvature regions and singularity candidates.
- `outline_path`, stroke geometry, and open-path offsets inherit the improved curve-offset precision automatically without a Web API change.

### Scope limits

- General Bézier and non-circular elliptical offsets remain bounded cubic approximations rather than exact algebraic offset curves.
- Source curves with genuinely degenerate tangents may still use the existing stable-tangent fallback or return a defined geometry/tolerance failure.
- Open offset centerlines remain unnormalized for self-intersections; closed filled offsets continue through the existing Boolean cleanup path.

## 0.17.0 - 2026-10-07

### Added

- Public `SegmentClosestPoint` and `closest_point_on_segment` APIs for source-native nearest-point queries.
- Native nearest-point solving for line, quadratic Bézier, cubic Bézier, and elliptical-arc segments.
- Polynomial stationary-point solving for quadratic and cubic Bézier distance minimization.
- Native elliptical-arc stationary-point solving over the selected arc sweep.
- Source-segment spatial broad phases in `PathSpatialIndex` and `IncrementalPathSpatialIndex`.
- WebAssembly `closest_point_svg` and TypeScript `closestPoint`, returning geometry distance, source subpath/segment, source parameter `t`, and path-distance coordinate.
- Robustness coverage for coarse flatness, large coordinates, non-finite queries, indexed refinement, and incremental-index synchronization.
- Criterion benchmark coverage for precise cubic nearest-point queries.

### Changed

- `closest_point` now evaluates the original path segments instead of returning the nearest point on adaptively flattened edges.
- `PathSpatialIndex::closest_point` and `IncrementalPathSpatialIndex::closest_point` now use source-segment bounds only as a broad phase and refine against native source geometry.
- Nearest-point result geometry and source parameter are no longer controlled by `Tolerance.flatness`; tolerance still participates in numerical classification and arc-length reporting.

### Scope limits

- Bézier and elliptical nearest-point solving is numerical and bounded; it does not claim symbolic closed-form output for general cubic geometry.
- `PathLocation.distance` continues to use the existing tolerance-controlled numerical arc-length integration.
- Spatial nearest queries remain CPU/single-process geometry operations; scene selection policy and snapping UX remain outside the core.

## 0.16.0 - 2026-10-07

### Added

- Public `transform_elliptical_arc` for native affine transformation of elliptical arcs.
- Native `EllipticalArc` preservation in `transform_path` for non-singular affine transforms.
- Exact geometry preservation across translation, rotation, non-uniform scaling, skew, and reflection.
- Reflection-aware sweep reversal so transformed parameter direction remains geometrically correct.
- Regression coverage for skewed/non-uniform transforms, reflections, full ellipse sweeps, singular collapse, and non-finite matrices.
- Criterion benchmark coverage for affine transformation of elliptical arcs.

### Changed

- `transformPath` through WebAssembly/TypeScript now preserves non-degenerate elliptical arcs instead of flattening them to line segments.
- Singular affine transforms retain deterministic fallback behavior by flattening only when the transformed ellipse collapses and cannot be represented as a valid `EllipticalArc`.

### Scope limits

- Near-singular transforms whose minor radius falls within the configured tolerance are treated as singular and use the flattening fallback.
- Path transforms remain immutable geometry snapshots; editor history, selection, and scene state remain outside the core.

## 0.15.0 - 2026-10-07

### Added

- Public immutable path-editing API based on `PathEditResult` and `PathEditReport`.
- `SegmentAddress` and explicit `SubpathEndpoint` addressing types.
- Native segment splitting by parameter and by arc length.
- Anchor insertion while preserving line, quadratic Bézier, cubic Bézier, and elliptical-arc primitives.
- Segment and subpath extraction APIs.
- Segment and subpath replacement APIs.
- Segment and subpath removal APIs.
- Subpath reversal and open/closed state editing.
- Endpoint-aware joining of open subpaths, including deterministic source reversal when requested.
- `subpath_delta` and `segment_delta` edit-report fields for deterministic index-shift handling.
- `IncrementalPathSpatialIndex::sync_edit` for direct synchronization from edit results.
- WebAssembly and TypeScript APIs for split, insert, extract, replace, remove, reverse, open/close, join, and path transforms.
- TypeScript `transformPath` exposure for the existing core `Transform2D` path transform.
- Robustness tests for native curve splitting, closed-contour edge removal, endpoint joins, replacement validation, edit reports, incremental spatial synchronization, invalid addresses, and large coordinates.
- Criterion benchmarks for segment editing and edit-plus-incremental-index synchronization on 2000-segment paths.

### Changed

- Closed-contour segment removal now preserves the former implicit closing edge when opening the contour at the removed edge.
- Path edits remain immutable snapshots instead of mutating shared editor state.
- Topology-changing edits report deterministic count deltas rather than promising persistent object IDs.

### Scope limits

- Segment and subpath addresses are positional indices; topology edits can shift later indices.
- Replacement segments must keep the original endpoints within tolerance.
- Joining open subpaths requires selected endpoints to match within tolerance; arbitrary-gap auto-bridging is not performed.
- Undo/redo, selection state, and persistent editor IDs remain outside the geometry core.
- The existing `transform_path` behavior preserves lines/Béziers but flattens elliptical arcs according to the supplied tolerance.

## 0.14.0 - 2026-10-07

### Added

- Public `StrokeHitIndex` for reusable, style-specific stroke hit testing.
- Geometry-backed stroke hit testing using the same segment ribbons, joins, caps, and dash fragments as the outline engine.
- Exact stroke semantics for `Butt`, `Round`, and `Square` caps.
- Exact stroke semantics for `Miter`, `Round`, and `Bevel` joins with miter-limit fallback.
- Dashed stroke hit testing with dash gaps and dash endpoint caps.
- Conservative cap/join-aware broad-phase pruning in `PathSpatialIndex` and `IncrementalPathSpatialIndex`.
- Geometry-derived `stroke_bounds` that includes cap and join reach.
- WebAssembly `hit_test_stroke_svg`.
- TypeScript `hitStroke` with cap, join, miter, dash-array, and dash-offset options.
- Robustness coverage for cap differences, join differences, dashed gaps, cached/indexed equivalence, large coordinates, and overflow rejection.
- Criterion benchmarks comparing per-query construction with reusable `StrokeHitIndex`.

### Changed

- `stroke_contains_point` no longer uses flattened centerline distance as the final hit criterion.
- Stroke hit testing now indexes independent stroke components instead of Boolean-unioning a complete outline for every query.
- `outline_path` and hit testing share one internal stroke-component construction path.
- Dense-path stroke hit queries no longer depend on a full Boolean outline union and avoid the previous `NonConvergent` failure in spatial equivalence tests.
- Stroke validation rejects finite inputs whose combined width/miter reach would overflow.

### Scope limits

- Curved component point classification inherits the configured `Tolerance` and adaptive flattening used by fill classification.
- Severe cusps or degenerate tangents may still return a defined geometry/tolerance failure during component construction.
- Source-path spatial indexes provide broad-phase pruning but do not cache style-specific stroke components; repeated queries should reuse `StrokeHitIndex`.

## 0.13.0 - 2026-10-07

### Added

- One-sided offset support for open paths through the existing `offset_path` API.
- Signed open-offset semantics: positive distances offset left of path direction and negative distances offset right.
- Exact line-line miter intersections for compatible inner and outer corners.
- Open-path `Miter`, `Bevel`, and `Round` outer joins with miter-limit fallback.
- Native circular-arc round joins for open offsets.
- Native circular-arc preservation for circular source arcs and adaptive cubic offsets for curved source geometry.
- Mixed open/closed path handling while retaining filled-area semantics for closed contours.
- Direct compatibility with open paths produced by the v0.12 contour trim/slice engine.
- Robustness tests for signed sides, inner/outer miter joins, bevel joins, round joins, miter fallback, cubic offsets, circular arcs, trimmed contours, mixed paths, zero distance, and large translated coordinates.
- Criterion benchmark coverage for open cubic path offsets.

### Changed

- `offset_path` and the existing WebAssembly/TypeScript `offsetPath` API no longer reject open paths.
- Closed-path offset behavior and fill-rule handling remain unchanged.
- Offset/outline documentation now distinguishes closed filled-area offsets from open one-sided centerline offsets.

### Scope limits

- Open offset centerlines are not Boolean-normalized after construction.
- Exact line-line joins are retained where possible; nonlinear corners may use connector geometry between adaptive offset pieces.
- Exact 180-degree reversals use a deterministic direct connector.
- Severe cusps or degenerate tangents may return a defined failure rather than unstable geometry.

## 0.12.0 - 2026-10-07

### Added

- Public `ContourSliceMode` with clamped and closed-contour wrapping semantics.
- Public `contour_length` that includes the implicit closing edge of closed contours.
- Public `slice_contour` for extracting an arc-length interval from one contour.
- Public `split_contour_at_length` for complementary contour pieces around a split distance.
- Native line, quadratic Bézier, cubic Bézier, and elliptical-arc preservation during trimming.
- Closed-contour seam wrapping for intervals that cross the implicit close edge.
- WebAssembly `contour_length_svg` and `slice_contour_svg`.
- TypeScript `contourLength`, `sliceContour`, and `splitContour` APIs with subpath selection.
- Robustness tests covering implicit close length, reversed clamp intervals, native curve preservation, seam wrapping, full cycles, open-wrap rejection, complementary splits, zero-length slices, non-finite inputs, and large translated coordinates.
- Criterion benchmark coverage for cubic contour slicing.

### Changed

- Dash expansion and manual contour slicing now share one internal arc-length slicing implementation.
- README spatial-acceleration limits now reflect the dynamic and incremental indexes introduced in v0.9 and v0.10.

### Scope limits

- Slicing addresses one selected contour/subpath at a time.
- Wrap mode applies only to closed contours.
- Equal start and end distances produce an empty slice; a full cycle is represented by an interval spanning the contour length.
- Partial slices are open contours. Complete contour slices preserve the source closed state.
- Slice placement inherits numerical integration and inversion tolerance.

## 0.11.0 - 2026-10-07

### Added

- Public `dash_path` geometry expansion for stroke dash patterns.
- Odd-length dash-array normalization, dash offset/phase handling, and deterministic per-subpath pattern restart.
- Arc-length-based dash splitting with the public `segment_parameter_at_length` helper.
- Native curve preservation while slicing lines, quadratic Béziers, cubic Béziers, and elliptical arcs.
- Closed-path dash wrapping across the start seam without introducing artificial caps.
- Dashed stroke support in `outline_path`, `stroke_to_path`, and `tessellate_stroke`.
- Dashed stroke semantics in stroke bounds and direct hit testing.
- Dashed hit-query support for `PathSpatialIndex` and `IncrementalPathSpatialIndex`.
- WebAssembly `dash_path_svg` and dashed outline support.
- TypeScript `dashPath` plus `dashArray` / `dashOffset` options on outline and stroke tessellation APIs.
- Robustness tests for phase offsets, closed seams, native cubic/arc preservation, indexed hit testing, large translated coordinates, and dashed tessellation area.
- Criterion benchmark coverage for dash expansion and dashed cubic outlining.

### Changed

- Non-empty dash arrays no longer return `UnsupportedCase` from the outline engine.
- Stroke tessellation now expands dash patterns before outline triangulation.
- `PathSpatialIndex` retains the source path snapshot needed to preserve dash semantics during indexed stroke queries.
- The web tessellation wrapper composes curve-preserving dash expansion with the existing stroke tessellation boundary.

### Scope limits

- Dash expansion is bounded; pathological patterns that would require excessive fragments return `IterationLimit`.
- Empty and all-zero dash arrays retain solid-stroke behavior.
- Dash placement follows numerical arc-length integration and inherits the supplied `Tolerance`.
- Existing stroke hit testing remains a flattened centerline-distance query; exact cap/join boundary classification is outside this release.

## 0.10.0 - 2026-10-07

### Added

- Public `IncrementalPathSpatialIndex` for repeated spatial queries on paths that change over time.
- `sync_path` diffing against the previous path snapshot, with segment-level updates when topology stays compatible.
- Subpath-local rebuilds when a subpath changes segment count, start point, or closure state.
- Automatic full rebuild only when the path's subpath count changes.
- `PathSpatialSync` reports for rebuild scope, changed subpaths/segments, and inserted/removed flattened edges.
- Source-segment parameter tracking during adaptive flattening so indexed nearest-point results retain the original `subpath_index`, `segment_index`, and source `t`.
- Incremental length metadata for correct `PathLocation.distance` after path edits.
- Differential tests against full fill, stroke, nearest-point, and path-length algorithms before and after incremental synchronization.
- Robustness coverage for repeated edit cycles, local topology changes, full rebuild fallback, and large translated coordinates.
- Criterion benchmark targets comparing one-segment incremental synchronization with a full path spatial rebuild on a 2000-segment workload.
- `IncrementalPathSpatialIndex` in the core prelude.

### Changed

- Segment flattening now has an internal parameter-preserving edge representation used by the incremental path index.
- Incremental nearest-point queries ignore implicit closing edges for path-location reporting, matching the explicit-segment semantics of `closest_point` and `path_length`.
- Fill and stroke queries still include implicit closing edges for closed subpaths.

### Scope limits

- `sync_path` compares immutable `Path` snapshots; it does not introduce mutable editor state into the geometry model.
- Segment-local reuse applies while the number of subpaths is unchanged. Adding or removing subpaths currently falls back to a full rebuild.
- A changed subpath with different local topology is rebuilt as one unit, while unchanged subpaths remain indexed.
- The index remains a CPU geometry acceleration structure and is not a scene graph, renderer cache, or concurrent editing model.

## 0.9.0 - 2026-10-07

### Added

- Public `DynamicSpatialIndex` for mutable broad-phase geometry workloads.
- Stable caller-provided item IDs across inserts, updates, removals, rebuilds, and queries.
- Deterministic `query_bounds`, `self_candidate_pairs`, and `cross_candidate_pairs` operations for changing geometry sets.
- A bounded mutation overlay that keeps recent changes queryable without rebuilding the immutable sweep index after every edit.
- Automatic deterministic rebuilds after enough distinct items have changed, plus an explicit `rebuild` operation.
- Mutation-focused differential tests against brute-force queries and pair generation, including sparse IDs, repeated updates, removals, and large translated coordinates.
- Criterion benchmark coverage for dynamic queries and update-plus-query workloads on 10000 indexed bounds.
- `DynamicSpatialIndex` in the core prelude.

### Changed

- Spatial acceleration now supports both immutable bulk indexing through `SpatialIndex` and mutation-heavy workloads through `DynamicSpatialIndex`.
- Dynamic query results and candidate pairs are normalized to stable item-index order regardless of mutation history.

### Scope limits

- The dynamic index is a CPU broad phase; exact geometric decisions remain in the existing intersection and topology engines.
- Indexed bounds must be finite and non-empty. Removing an item is explicit rather than represented by an empty bound.
- Mutation tracking is single-process state and is not a concurrent scene graph or editor-state system.
- Automatic rebuild thresholds are an internal performance policy and are not part of the public correctness contract.

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
