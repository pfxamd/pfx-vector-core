# Using PFx Vector Core in another application

PFx Vector Core v1.0.0 is a geometry kernel, **not** a UI or rendering library.
A consumer owns document state, selection, rendering, history, pointer events, and undo/redo.

## Current distribution contract

- Rust crates and the TypeScript adapter live in this repository.
- `@pfxamd/vector-core-web` has `"private": true`: it is **not** installed from npm.
- The GitHub Release includes source archives, but no precompiled WebAssembly assets.
- Consumers must build the WASM bindings and TypeScript adapter locally, or vendor pinned build outputs into their own application. Do not assume `npm install @pfxamd/vector-core-web` works.

Pin an explicit release tag (for example, `v1.0.0`) rather than depending on moving `main` when integrating.

## Build

Prerequisites: Rust stable (minimum Rust 1.85), `wasm32-unknown-unknown` target, `wasm-pack`, Node 22, and npm.

From the repository root:

```sh
rustup target add wasm32-unknown-unknown
wasm-pack build crates/pfx-vector-wasm --release --target web --out-dir pkg
npm install --prefix packages/pfx-vector-web
npm run build:integration --prefix packages/pfx-vector-web
```

For a Node.js consumer, generate Node bindings instead:

```sh
wasm-pack build crates/pfx-vector-wasm --release --target nodejs --out-dir pkg
npm run test:integration --prefix packages/pfx-vector-web
```

The integration test exercises the compiled adapter against real Rust/WASM operations.

## Browser consumer

Copy the generated `crates/pfx-vector-wasm/pkg/` directory and
`packages/pfx-vector-web/dist/index.js` into versioned, application-owned assets.
Keep the JS glue and `.wasm` file together.

```js
import init, * as wasm from "./vendor/pfx-vector-wasm/pfx_vector_wasm.js";
import { createVectorCore } from "./vendor/vector-core-web/index.js";

await init(); // Must finish before any geometry call.
const vector = createVectorCore(wasm);

const path = "M0 0 L100 0 L100 100 L0 100 Z";
const length = vector.pathLength(path);
const bounds = vector.pathBounds(path);
const union = vector.unionPaths(path, "M50 50 L150 50 L150 150 Z");
```

Serve the app over HTTP(S); do not load the module with `file://`.
The host must serve `.wasm` with a compatible MIME type (normally
`application/wasm`). The bundler/server must preserve or correctly resolve
the `wasm-pack` generated module's WASM URL.

The adapter is synchronous **after** the asynchronous browser initialization.
Catch errors from invalid or unsupported geometry instead of assuming every path succeeds.

## Constraints for editor integration

- The geometry API currently reparses SVG path strings for most WASM calls.
  Cache geometry in the host where appropriate; repeated per-frame calls can be costly.
- Path measure indexes are immutable snapshots and must be rebuilt after edits.
  Incremental editor caching/ownership is not supplied by the browser adapter.
- The default SVG serializer precision is six decimal places; nearly degenerate arcs
  may require a higher precision through the Rust SVG API.
- Results are deterministic within the documented numerical tolerance; they are not
  exact symbolic geometry across all pathological inputs.
- Rendering and editor interaction must live outside the core.

## Verification

- `packages/pfx-vector-web/tests/runtime-integration.mjs` tests the compiled
  TypeScript adapter against real Node WebAssembly in GitHub Actions.
- `demo/browser-smoke.mjs` also tests the adapter through real WebAssembly on
  desktop Chromium, Firefox, and WebKit.
- These tests do **not** certify external bundlers, mobile browsers, consumer
  editor workflows, or arbitrary adversarial SVG files.
