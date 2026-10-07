# PFx Vector Core

`PFx Vector Core` is a Rust vector-geometry kernel for deterministic 2D geometry, SVG geometry, spatial acceleration, intersections, contour normalization, Boolean operations, offsets, tessellation, path simplification, and curve fitting. Rendering, DOM, UI, scene graphs, and editor state are outside the core.

**Status:** `v0.10.0`.

## v0.10 incremental path spatial engine

For paths that are edited repeatedly, build one reusable index and synchronize new immutable path snapshots into it:

```rust
let mut index = IncrementalPathSpatialIndex::build(&path, tolerance)?;

let sync = index.sync_path(&edited_path)?;

let inside = index.contains_point(point, FillRule::NonZero)?;
let stroke_hit = index.stroke_contains_point(&stroke_style, point)?;
let nearest = index.closest_point(point)?;
```

`sync_path` compares the new path with the previous snapshot. Unchanged geometry is retained. Changed segments are re-flattened and re-indexed individually when possible; local topology changes rebuild only the affected subpath. A full index rebuild is reserved for path-level topology changes such as adding or removing subpaths.

`PathSpatialSync` reports whether a full rebuild occurred and how much indexed geometry changed.

The incremental index also retains the original source-segment parameter range for every flattened edge. Nearest-point results therefore report the original `subpath_index`, `segment_index`, source `t`, and path distance rather than flatten-edge coordinates.

Closed-path semantics remain split intentionally: fill and stroke queries include the implicit closing edge, while nearest-point location follows the explicit source segments used by `closest_point` and `path_length`.

## v0.9 dynamic spatial indexing

For geometry sets that change during editing or interactive processing, the core now provides a mutable broad-phase index with stable caller-owned item IDs:

```rust
let mut index = DynamicSpatialIndex::from_bounds(&bounds)?;

index.update(42, new_bounds)?;
index.insert(5000, added_bounds)?;
index.remove(7)?;

let candidates = index.query_bounds(query_bounds, tolerance.absolute)?;
```

`DynamicSpatialIndex` combines the deterministic immutable sweep index with a bounded mutation overlay. Recent inserts, updates, and removals are visible immediately, while automatic rebuilds compact accumulated changes without forcing a full rebuild after every edit.

The mutable index also provides deterministic self- and cross-index candidate-pair queries. It remains a broad phase only: exact intersections and topology decisions stay in the existing geometry engines.

## v0.8 spatial acceleration

The core now provides a deterministic broad-phase spatial index for large geometry sets:

```rust
let bounds = segments
    .iter()
    .map(|segment| segment.bounds())
    .collect::<Vec<_>>();

let index = SpatialIndex::new(&bounds);
let candidates = index.query_bounds(query_bounds, tolerance.absolute)?;
```

For pairwise geometry work:

```rust
let self_pairs = spatial_self_candidate_pairs(
    &bounds,
    tolerance.absolute,
)?;

let cross_pairs = spatial_cross_candidate_pairs(
    &bounds_a,
    &bounds_b,
    tolerance.absolute,
)?;
```

These functions return only AABB-overlapping candidates. Exact intersections are still resolved by the curve intersection engine.

### Reusable path index

For repeated geometric queries, build a path index once:

```rust
let index = PathSpatialIndex::build(&path, tolerance)?;

let inside = index.contains_point(point, FillRule::NonZero)?;
let stroke_hit = index.stroke_contains_point(&stroke_style, point)?;
let nearest = index.closest_point(point)?;
```

`PathSpatialIndex` performs one adaptive flattening pass and reuses the resulting edge index across queries.

### Integrated acceleration

The broad phase is now used automatically by:

- self-intersection normalization
- Boolean segment pairing
- topology side probes through reusable indexed fill classification
- WebAssembly path intersection queries

The acceleration layer never replaces exact geometry. It only removes pairs or edges whose bounds prove they cannot participate.

## v0.7 self-intersection and contour normalization

The core can now convert closed self-intersecting filled paths into explicit simple boundary contours:

```rust
let normalized = normalize_self_intersections(
    &path,
    FillRule::EvenOdd,
    tolerance,
)?;
```

The normalization pipeline:

- collects explicit and implicit closing segments
- detects intersections between segment pairs
- detects intrinsic loop intersections inside a single cubic Bézier
- splits geometry at stable parameters
- classifies both sides of every fragment using the requested fill rule
- discards fragments that do not separate filled and empty space
- orients retained boundaries with filled space on the left
- walks the resulting planar boundary graph into closed simple contours

The operation is curve-preserving. Lines, quadratic Béziers, cubic Béziers, and elliptical arcs remain native segment types and are only subdivided where topology requires it.

### Integrated engines

Self-intersection normalization now runs automatically before:

- Boolean operations
- filled path offsets
- fill tessellation

This means validated bow-tie shapes, repeated winding, touching lobes, and curve self-crossings can feed those engines directly.

### Web API

```text
normalizeFillContours(data, "nonzero")
normalizeFillContours(data, "evenodd")
```

This is distinct from `normalizePath`: `normalizePath` canonicalizes SVG path syntax, while `normalizeFillContours` resolves fill topology.

## v0.6 cleanup, simplification, and curve fitting

The core now separates three different path-reduction operations:

```rust
let cleaned = cleanup_path(
    &path,
    CleanupOptions::default(),
    tolerance,
)?;

let simplified = simplify_path(
    &path,
    0.1,
    tolerance,
)?;

let fitted = fit_path_curves(
    &path,
    0.1,
    tolerance,
)?;
```

### Cleanup

`cleanup_path` is conservative. It removes duplicate/collinear line noise and converts only demonstrably line-like Bézier segments to lines. Real curves remain curves.

### Simplification

`simplify_path` adaptively flattens source geometry and applies deterministic Ramer-Douglas-Peucker reduction. Its output is intentionally line geometry.

Closed contours are simplified as rings using a deterministic farthest-point split so the implicit close edge remains part of the error model.

### Cubic fitting

`fit_path_curves` reconstructs flattened path samples as cubic Bézier segments using:

- chord-length parameterization
- least-squares control-distance solving
- Newton reparameterization
- recursive error-driven splitting
- explicit recursion limits

It preserves open endpoints and closed contour state while reducing dense sampled paths to far fewer cubic segments.

### Web API

```text
cleanupPath(data, { pointTolerance, collinearTolerance })
simplifyPath(data, maxDeviation)
fitPathCurves(data, maxError)
```

## v0.5 tessellation and triangulation

The core now converts filled paths and expanded strokes into indexed triangle meshes:

```rust
let fill_mesh = tessellate_fill(
    &path,
    FillRule::NonZero,
    tolerance,
)?;

let stroke_mesh = tessellate_stroke(
    &path,
    &stroke_style,
    tolerance,
)?;
```

`Mesh2D` contains:

```text
vertices: Vec<Point2>
indices:  Vec<u32>
```

The triangulation pipeline:

- adaptively flattens curves at the requested geometric tolerance
- resolves actual fill boundaries using `NonZero` or `EvenOdd`
- normalizes contour orientation
- groups holes with their containing outer contours
- bridges holes into triangulable simple polygons
- ear-clips concave polygons
- emits counter-clockwise triangles with deterministic index order

Curve flattening happens only for mesh generation. The original `Line / Quadratic / Cubic / Arc` path remains unchanged.

### Web API

```text
tessellateFill(data, rule, { flatness })
tessellateStroke(data, width, { cap, join, miterLimit, flatness })
```

The web mesh format uses a flat numeric vertex buffer:

```text
vertices = [x0, y0, x1, y1, ...]
indices  = [i0, i1, i2, ...]
```

## v0.4 offset and outline engine

The core now exposes:

```rust
offset_path(path, distance, OffsetStyle::default(), tolerance)
outline_path(path, &stroke_style, tolerance)
stroke_to_path(path, &stroke_style, tolerance)
```

For compound SVG fill semantics:

```rust
offset_path_with_fill_rule(
    path,
    FillRule::EvenOdd,
    distance,
    OffsetStyle::default(),
    tolerance,
)
```

Offset semantics are set-oriented for closed paths:

- positive distance expands the filled area
- negative distance contracts the filled area
- holes shrink on outward offsets and expand on inward offsets
- complete inward collapse returns an empty path

Outline construction supports:

- `Butt / Round / Square` caps
- `Miter / Round / Bevel` joins
- miter-limit fallback
- open and closed source paths
- Boolean cleanup of overlapping segment ribbons, joins, caps, and crossing centerlines

### Curve representation

Offsets are not flattened into a fixed polygonal approximation.

- lines stay exact lines
- circular arcs stay exact circular arcs when the offset radius remains regular
- quadratic/cubic Bézier offsets use adaptive cubic fitting
- general elliptical-arc offsets use adaptive cubic fitting
- adaptive fitting is checked against the requested geometric tolerance

This distinction is intentional: a true parallel curve of a general Bézier or ellipse is not itself exactly representable by the same primitive type.

### Web API

The TypeScript wrapper now provides:

```text
offsetPath(data, distance, { join, miterLimit })
outlinePath(data, width, { cap, join, miterLimit })
```

## v0.3 Boolean operations

- `Union / Intersection / Difference / XOR`
- native curve fragmentation at actual intersection parameters
- line, quadratic, cubic, and arc preservation
- `NonZero / EvenOdd` input semantics
- shared-edge removal
- directed output contours and holes
- WebAssembly and TypeScript Boolean APIs

## v0.2 advanced intersections

- quadratic-quadratic
- quadratic-cubic
- cubic-cubic
- Bézier-arc
- arc-arc
- crossing/tangent/endpoint classification
- stable parameters on both segments
- overlap support for validated coincident cases
- analytical elliptical-arc bounds

## v0.1 foundation

- `f64` geometry with explicit `Tolerance`
- points, vectors, angles, affine transforms, and bounds
- primitive shapes, Bézier curves, and elliptical arcs
- `Path → Subpath → Segment` topology
- measurement, adaptive flattening, nearest-point queries
- fill/stroke hit testing
- SVG path parsing/normalization/serialization
- transform and `viewBox` handling
- WebAssembly bridge and TypeScript wrapper

## Workspace

```text
crates/pfx-vector-core   math, geometry, paths, numeric algorithms
crates/pfx-vector-svg    SVG geometry syntax boundary
crates/pfx-vector-wasm   coarse-grained WebAssembly bridge
packages/pfx-vector-web  TypeScript wrapper
corpus                   adversarial and regression inputs
fuzz                     parser/geometry fuzz target
```

Dependency direction is one-way:

```text
pfx-vector-core <- pfx-vector-svg <- pfx-vector-wasm <- @pfxamd/vector-core-web
```

The geometry core has no runtime third-party dependencies and the Rust workspace forbids `unsafe` code.

## Numerical model

The scalar type is `f64`. Approximate operations use explicit tolerance rather than a hidden global epsilon.

Tolerance-sensitive operations include:

- Bézier/arc length via adaptive numerical integration
- deterministic AABB spatial broad-phase pruning
- reusable indexed path queries
- adaptive curve flattening
- closest-point and stroke hit-testing approximations
- advanced curve intersections via bounded subdivision and local refinement
- general curve offsets via adaptive cubic fitting
- self-intersection normalization through geometric splitting, fill-side classification, and face walking
- Boolean/offset topology through explicit geometric side classification and cleanup
- tessellation through adaptive flattening, contour topology, hole bridging, and ear clipping
- path simplification through explicit geometric deviation thresholds
- cubic fitting through least-squares approximation and iterative reparameterization

## Validated spatial-acceleration scope

Validated cases include exact agreement with brute-force candidate pairing, deterministic query ordering, empty bounds, padded queries, 5000 sparse bounds without quadratic candidate growth, large translated coordinates, fill-classification equivalence, stroke-hit equivalence, and nearest-point equivalence on dense paths.

Current limits:

- indexes are immutable snapshots and must be rebuilt after geometry changes
- `PathSpatialIndex` uses flattened edges and inherits the configured flatness tolerance
- the spatial layer is deliberately broad-phase only; it does not approximate exact curve intersections
- no dynamic tree mutation, multithreaded index updates, or GPU spatial structures are part of this release

## Validated contour-normalization scope

Validated cases include bow-tie crossings, multiple simple lobes sharing one node, repeated same-direction winding, opposite-winding cancellation, `NonZero / EvenOdd` differences, nested contours, reversed input orientation, large translated coordinates, native circle arcs, intrinsic cubic Bézier loops, idempotent renormalization, and direct use by Boolean, Offset, and Tessellation.

Current limits:

- inputs must be closed fill contours
- intrinsic cubic self-crossings are supported analytically
- multi-revolution or retraced single elliptical arcs are outside the guaranteed scope
- pathological coincident curves remain subject to the overlap capabilities of the intersection engine
- severe degenerate cusps may return a defined failure instead of unstable topology

## Validated cleanup and fitting scope

Validated cases include duplicate and collinear cleanup, nearly linear Bézier conversion, preservation of real curves, open and closed simplification, endpoint preservation, monotonic simplification under larger tolerances, sharp corners, sampled cubic reconstruction, closed circular samples, reversed point order, large translated coordinates, deterministic output, and dense-wave segment reduction.

Current limits:

- simplification does not promise topology preservation for arbitrary self-intersections
- simplification outputs lines
- fitting outputs cubic Bézier segments rather than rediscovering original primitive types
- fitting error is numerical and sample-driven, with explicit tolerance and recursion bounds

## Validated tessellation scope

Validated cases include convex and concave polygons, multiple disjoint contours, holes, multiple holes, nested islands, `NonZero` and `EvenOdd` semantics, curved paths, stroke expansion, reversed contour orientation, large translated coordinates, and positive triangle winding.

Current tessellation limits:

- fill tessellation requires closed subpaths
- validated self-intersecting fills are normalized before triangulation
- mesh output contains geometry only; renderer/GPU concerns remain outside the core
- curve tessellation accuracy follows the explicit flatness tolerance

## Validated offset/outline scope

Validated cases include outward/inward rectangle offsets, circular offsets, closed outline rings, holes, `EvenOdd` holes, cubic and elliptical outlines, all cap types, all join modes, miter fallback, crossing open centerlines, large translated coordinates, and complete inward collapse.

Current limits:

- offset inputs must be closed
- self-intersecting filled contours are normalized before offset construction
- dashed outline expansion is deferred
- severe cusp/degenerate tangent cases may return a defined failure

## Naming

The repository follows **PFx Naming System v3.1.0** in advisory `balanced` mode.

- `pfxamd` is the canonical metadata/namespace identity.
- `PFx` is the visible identity.
- Rust keeps native `snake_case`, `UpperCamelCase`, and `SCREAMING_SNAKE_CASE` conventions.
- TypeScript keeps natural `camelCase` / `PascalCase` semantics.
- PFx prefixes are not added decoratively to ordinary geometry types or local identifiers.

See [`AGENTS.md`](AGENTS.md) for the local contributor rules.

## Validation

```bash
python3 scripts/verify_structure.py
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo bench -p pfx-vector-core --no-run
cargo build -p pfx-vector-wasm --target wasm32-unknown-unknown
```

For the TypeScript wrapper:

```bash
cd packages/pfx-vector-web
npm install
npm run typecheck
```

Rust `1.85` is the declared minimum version because the workspace uses Edition 2024. Current stable Rust is recommended.

## License

Apache-2.0.
