# PFx Vector Core — Contributor Rules

## Naming source

This repository follows **PFx Naming System v3.1.0** in advisory `balanced` mode.

Priority:

1. Explicit project instruction.
2. Correctness.
3. Native language/framework convention.
4. Existing project convention.
5. Clarity and maintainability.
6. PFx fingerprint.

The PFx fingerprint is not a prefix tax. Ordinary domain types, functions, variables, and modules keep natural semantic names.

## Identity

- `pfxamd` — canonical namespace and metadata identity.
- `PFx` — visible project identity.
- `pfx` — lowercase code/package form when a PFx-owned namespace is appropriate.
- `Pfx` — PascalCase form only for genuinely shared PFx-owned primitives where the prefix adds useful ownership context.

## Rust

Use native Rust conventions:

- crates/modules/files: `snake_case` in Rust identifiers; Cargo package names may use kebab-case.
- types/traits/enums: `UpperCamelCase`.
- functions/methods/variables: `snake_case`.
- constants/statics: `SCREAMING_SNAKE_CASE`.

Do not rename ordinary geometry concepts to decorative forms such as `PfxPoint`, `PfxPath`, or `pfx_parse_path`.
The PFx identity is already carried by the repository/crate namespace and metadata.

## TypeScript

Use native TypeScript conventions:

- types/interfaces: `PascalCase`.
- functions/variables: `camelCase`.
- true constants: `UPPER_SNAKE_CASE` when appropriate.
- module files: `kebab-case` when more than one word is needed.

Use `Pfx*` only for a reusable PFx-owned shared primitive. Infrastructure bindings local to this package should remain naturally named.

## Architecture boundaries

Dependency direction is one-way:

`pfx-vector-core <- pfx-vector-svg <- pfx-vector-wasm <- @pfxamd/vector-core-web`

The core must not depend on SVG syntax, WebAssembly bindings, browser APIs, DOM, rendering, or UI state.

## Quality gates

Before merging:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo build -p pfx-vector-wasm --target wasm32-unknown-unknown
```

Do not introduce `unsafe` code into the workspace.
