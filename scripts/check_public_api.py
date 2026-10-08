#!/usr/bin/env python3
"""Guard public WASM function signatures against v0.24.0.

This checks normalized Rust exported parameter and return tokens; changes in
runtime semantics and TypeScript structural declarations need separate checks.
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

def wasm_signatures(source: str) -> dict[str, str]:
    signatures = {}
    pattern = r"(?m)^pub fn ([A-Za-z_][A-Za-z_0-9]*)\\s*\\("
    for match in re.finditer(pattern, source):
        start = match.start()
        open_paren = source.index("(", match.start())
        depth = 0
        end = open_paren
        while end < len(source):
            if source[end] == "(":
                depth += 1
            elif source[end] == ")":
                depth -= 1
                if depth == 0:
                    end += 1
                    break
            end += 1
        remaining = source[end:]
        tail = re.match(r"\\s*(?:->\\s*[^\\{]+)?\\{", remaining)
        if tail is None:
            raise ValueError(f"Cannot parse signature for {match.group(1)}")
        value = source[start:end] + remaining[:tail.end() - 1]
        signatures[match.group(1)] = re.sub(r"\\s+", "", value)
    return signatures

def main() -> None:
    missing = []
    for path, pattern in FILES.items():
        before = set(re.findall(pattern, baseline_source(path)))
        after = set(re.findall(pattern, Path(path).read_text()))
        removed = sorted(before - after)
        print(f"{path}: {len(before)} baseline symbols, {len(after)} current symbols")
        missing.extend(f"{path}: {name}" for name in removed)
    path = "crates/pfx-vector-wasm/src/lib.rs"
    previous = wasm_signatures(baseline_source(path))
    current = wasm_signatures(Path(path).read_text())
    changed = sorted(name for name in previous.keys() & current.keys()
                     if previous[name] != current[name])
    missing.extend(f"{path}: changed signature {name}" for name in changed)
    if missing:
        raise SystemExit("Public API symbols removed:\n" + "\n".join(missing))
    print(f"Public API name/signature compatibility with {BASELINE}: PASS")

if __name__ == "__main__":
    main()
