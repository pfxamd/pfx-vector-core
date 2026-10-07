from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[1]
required=["crates/pfx-vector-core/src/foundation/mod.rs","crates/pfx-vector-core/src/math/mod.rs","crates/pfx-vector-core/src/geometry/mod.rs","crates/pfx-vector-core/src/curves/mod.rs","crates/pfx-vector-core/src/path/mod.rs","crates/pfx-vector-core/src/numeric/mod.rs","crates/pfx-vector-core/src/algorithms/mod.rs","crates/pfx-vector-svg/src/path/parser.rs","crates/pfx-vector-svg/src/path/serializer.rs","crates/pfx-vector-wasm/src/lib.rs","packages/pfx-vector-web/src/index.ts"]
for rel in required:
    if not (ROOT/rel).exists(): print(f"missing: {rel}",file=sys.stderr);sys.exit(1)
for f in (ROOT/"crates/pfx-vector-core/src").rglob("*.rs"):
    t=f.read_text()
    for bad in("wasm_bindgen","web_sys","pfx_vector_svg"):
        if bad in t: print(f"boundary violation {bad}: {f}",file=sys.stderr);sys.exit(1)
print("structure: ok")
