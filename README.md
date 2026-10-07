# PFx Vector Core

`PFx Vector Core` is a Rust vector-geometry kernel for deterministic 2D geometry, path mathematics, and an SVG geometry boundary. Rendering, DOM, UI, scene graphs, and editor state are intentionally outside the core.

**Status:** `v0.1.0`.

## v0.1 scope

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
- basic intersections: line-line, line-quadratic, line-cubic, and line-arc
- SVG path parsing and normalization for `M L H V C S Q T A Z`
- SVG path serialization
- SVG transform-list parsing
- `viewBox` / `preserveAspectRatio` transform calculation
- WebAssembly bridge and a thin TypeScript wrapper

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

The scalar type is `f64`. Approximate operations use explicit tolerance rather than a single hidden global epsilon. Invalid non-finite inputs are rejected at public construction/parsing boundaries where applicable. Topological results distinguish normal absence of a result from invalid geometry or numerical failure.

### Approximate operations in v0.1

Some operations are deliberately tolerance-driven:

- Bézier/arc length uses adaptive numerical integration.
- curve and arc flattening is adaptive.
- closest-point-on-curves currently uses the flattened representation as its search basis.
- stroke hit testing uses the flattened centerline representation.
- arbitrary affine transforms of arc segments are flattened when an exact arc representation is not retained.

These are documented behavior, not hidden precision claims.

## Deliberately outside v0.1

- Boolean path operations
- general path offsets
- tessellation / triangulation
- GPU or raster rendering
- scene graph / document model
- editor selection, history, snapping, and guides
- CSS cascade, DOM, filters, animation, scripting, and text shaping
- full curve-curve / arc-arc intersection engine

## Naming

The repository follows **PFx Naming System v3.1.0** in advisory `balanced` mode.

- `pfxamd` is the canonical metadata/namespace identity.
- `PFx` is the visible identity.
- Rust keeps native `snake_case`, `UpperCamelCase`, and `SCREAMING_SNAKE_CASE` conventions.
- TypeScript keeps natural `camelCase` / `PascalCase` semantics.
- PFx prefixes are not added decoratively to ordinary geometry types or local identifiers.

See [`AGENTS.md`](AGENTS.md) for the local contributor rules.

## Validation

The repository contains unit tests, property tests, SVG specification-oriented cases, regression tests, a fuzz target, adversarial corpora, and Criterion benchmarks.

```bash
python3 scripts/verify_structure.py
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
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

Rust `1.85` is the declared minimum version because the workspace uses Edition 2024. Current stable Rust is recommended.\n\n## License\n\nApache-2.0.
