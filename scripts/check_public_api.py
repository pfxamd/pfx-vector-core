#!/usr/bin/env python3
"""Guard exported API names against the published v0.24.0 baseline.

This is a removal guard, not a full semantic-versioning or signature-diff proof.
"""
import re
import subprocess
from pathlib import Path

BASELINE = "v0.24.0"
FILES = {
    "crates/pfx-vector-wasm/src/lib.rs": r"(?m)^pub fn ([A-Za-z_][A-Za-z_0-9]*)\s*\(",
    "packages/pfx-vector-web/src/index.ts": r"(?m)^export (?:interface|type|function) ([A-Za-z_][A-Za-z_0-9]*)\b",
}

def baseline_source(path: str) -> str:
    return subprocess.check_output(
        ["git", "show", f"{BASELINE}:{path}"], text=True
    )

def main() -> None:
    missing = []
    for path, pattern in FILES.items():
        before = set(re.findall(pattern, baseline_source(path)))
        after = set(re.findall(pattern, Path(path).read_text()))
        removed = sorted(before - after)
        print(f"{path}: {len(before)} baseline symbols, {len(after)} current symbols")
        missing.extend(f"{path}: {name}" for name in removed)
    if missing:
        raise SystemExit("Public API symbols removed:\n" + "\n".join(missing))
    print(f"Public API name compatibility with {BASELINE}: PASS")

if __name__ == "__main__":
    main()
