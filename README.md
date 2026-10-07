# PFx Vector Core

`PFx Vector Core` is a Rust vector-geometry kernel for deterministic 2D geometry, SVG geometry, advanced intersections, Boolean operations, and offset/outline construction, and 2D tessellation. Rendering, DOM, UI, scene graphs, and editor state are outside the core.

**Status:** `v0.5.0`.

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
- adaptive curve flattening
- closest-point and stroke hit-testing approximations
- advanced curve intersections via bounded subdivision and local refinement
- general curve offsets via adaptive cubic fitting
- Boolean/offset topology through explicit geometric side classification and cleanup
- tessellation through adaptive flattening, contour topology, hole bridging, and ear clipping

## Validated tessellation scope

Validated cases include convex and concave polygons, multiple disjoint contours, holes, multiple holes, nested islands, `NonZero` and `EvenOdd` semantics, curved paths, stroke expansion, reversed contour orientation, large translated coordinates, and positive triangle winding.

Current tessellation limits:

- fill tessellation requires closed subpaths
- self-intersecting contour normalization is not guaranteed
- mesh output contains geometry only; renderer/GPU concerns remain outside the core
- curve tessellation accuracy follows the explicit flatness tolerance

## Validated offset/outline scope

Validated cases include outward/inward rectangle offsets, circular offsets, closed outline rings, holes, `EvenOdd` holes, cubic and elliptical outlines, all cap types, all join modes, miter fallback, crossing open centerlines, large translated coordinates, and complete inward collapse.

Current limits:

- offset inputs must be closed
- dashed outline expansion is deferred
- severe cusp/degenerate tangent cases may return a defined failure
- general self-intersecting filled-contour offset normalization is not guaranteed

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
