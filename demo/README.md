# Geometry Lab — browser integration demo

This is an independent test surface, not a vector editor. The browser imports the **real Rust geometry kernel** compiled to WebAssembly; it does not approximate paths using browser geometry APIs.

## Run locally

From repository root, with Rust, `wasm-pack`, and Python 3 installed:

```sh
wasm-pack build crates/pfx-vector-wasm --release --target web --out-dir pkg
python3 -m http.server 8787
```

Open `http://127.0.0.1:8787/demo/`. Do not open `index.html` via `file://` because the browser must fetch a WASM module.

## Covered operations

- Parse and validate user-provided SVG path data.
- Measure arc length using Rust.
- Query differential frames and original segment locations along the contour.
- Generate the flattened geometry from Rust.
- Verify invalid-input error handling and curve/arc/mixed paths.
- Responsive interaction at narrow screen widths.

The automated browser check is defined in `demo/browser-smoke.mjs` and the GitHub Actions workflow `.github/workflows/browser-demo.yml`. The workflow compiles the WASM browser target and drives actual Chromium.

## Limits

This is deliberately not an editor. It does not test every WASM API, every mobile browser, multi-megabyte inputs, or unbounded interactive performance. Query latency reflects a browser-to-WASM API call that currently reparses path data; it is not a cached-index benchmark.
