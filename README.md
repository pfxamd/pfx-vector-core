# PFx Vector Core

`PFx Vector Core` is a Rust vector-geometry kernel for deterministic 2D geometry, path mathematics, SVG geometry, advanced intersections, and curve-preserving Boolean path operations. Rendering, DOM, UI, scene graphs, and editor state are outside the core.

**Status:** `v0.3.0`.

## v0.3 Boolean operations

The core now exposes:

```rust
boolean_union(a, b, tolerance)
boolean_intersection(a, b, tolerance)
boolean_difference(a, b, tolerance)
boolean_xor(a, b, tolerance)
```

A general API is also available:

```rust
boolean_paths(a, b, BooleanOperation::Union, tolerance)
```

For SVG fill semantics, `boolean_paths_with_fill_rules` accepts independent `NonZero` or `EvenOdd` rules for each input.

The Boolean engine:

- finds intersections with the native curve intersection engine
- splits source segments at their actual parameters
- preserves line, quadratic Bézier, cubic Bézier, and elliptical-arc fragments
- classifies both sides of every fragment against the requested set operation
- removes internal/shared boundaries
- reverses fragments when required by the resulting topology
- stitches retained fragments back into closed directed contours
- represents holes explicitly through contour direction

It does **not** flatten curves into polygons as the Boolean representation.

### Web API

The WebAssembly/TypeScript boundary exposes the same operations for SVG path data:

```text
unionPaths(a, b)
intersectPathAreas(a, b)
subtractPaths(a, b)
xorPaths(a, b)
```

## v0.2 advanced intersections

- quadratic-quadratic intersections
- quadratic-cubic intersections
- cubic-cubic intersections
- Bézier-arc intersections
- arc-arc intersections
- stable parameters on both intersecting segments
- crossing, tangent, and endpoint classification
- equivalent/reversed Bézier overlap detection
- partial overlap ranges for compatible elliptical arcs
- analytical elliptical-arc bounds
- near-tangent false-positive hardening
- deterministic intersection ordering

## v0.1 foundation

- `f64` geometry with explicit `Tolerance`
- points, vectors, angles, affine transforms, and bounds
- line, rectangle, rounded rectangle, circle, ellipse, polyline, and polygon geometry
- quadratic and cubic Bézier curves
- elliptical arcs
- `Path → Subpath → Segment` topology and `PathBuilder`
- path length and point/tangent-at-length queries
- adaptive flattening
- tight curve/arc bounds
- closest-point queries
- `nonzero` / `evenodd` fill hit testing
- stroke styles, dash processing, stroke bounds, and stroke hit testing
- SVG path parsing/normalization/serialization
- SVG transform-list parsing
- `viewBox` / `preserveAspectRatio` mapping
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

The scalar type is `f64`. Approximate operations use explicit tolerance rather than a hidden global epsilon. Invalid non-finite inputs are rejected at public construction/parsing boundaries where applicable.

Some operations are deliberately tolerance-driven:

- Bézier/arc length uses adaptive numerical integration.
- curve and arc flattening is adaptive.
- closest-point-on-curves currently uses the flattened representation as its search basis.
- stroke hit testing uses the flattened centerline representation.
- arbitrary affine transforms of arc segments are flattened when an exact arc representation is not retained.
- advanced curve intersections use bounded subdivision and local numerical refinement.
- Boolean topology uses numerical side probes after exact curve fragmentation to determine set membership around each boundary fragment.

## Validated Boolean scope

Validated cases include overlapping/disjoint/contained/identical closed paths, shared edges, holes, reversed input orientation, `evenodd` inputs, curved circle intersections, tangent circles, large translated coordinates, and set-operation symmetry.

Current limits:

- open subpaths are rejected
- self-intersecting input normalization is not yet guaranteed by the Boolean engine
- differently parameterized coincident Bézier overlaps are limited by current overlap detection
- general path offsets are not yet included

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
