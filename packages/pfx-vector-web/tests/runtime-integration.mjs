import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { createVectorCore } from "../dist/index.js";

const require = createRequire(import.meta.url);
const wasm = require("../../../crates/pfx-vector-wasm/pkg/pfx_vector_wasm.js");
const core = createVectorCore(wasm);

function near(actual, expected, tolerance = 1e-6) {
  assert.ok(
    Number.isFinite(actual) && Math.abs(actual - expected) <= tolerance,
    `expected ${actual} to be within ${tolerance} of ${expected}`,
  );
}

const line = "M0 0 L3 4";
const square = "M0 0 L100 0 L100 100 L0 100 Z";
const overlap = "M50 50 L150 50 L150 150 L50 150 Z";

assert.match(wasm.core_version(), /^\d+\.\d+\.\d+$/);
assert.equal(core.validatePath(line), true);
assert.equal(core.validatePath("M0 0 L"), false);
near(core.pathLength(line), 5);

const midpoint = core.pointAtLength(line, 2.5);
near(midpoint.x, 1.5);
near(midpoint.y, 2);

const samples = core.pointsAtLengths(line, [0, 2.5, 5]);
assert.equal(samples.length, 3);
near(samples[1].x, 1.5);
const frames = core.framesAtLengths(line, [1, 2]);
assert.equal(frames.length, 2);
near(Math.hypot(frames[0].tangentX, frames[0].tangentY), 1);

const bounds = core.pathBounds(square);
assert.ok(bounds);
near(bounds.minX, 0);
near(bounds.maxY, 100);
assert.equal(core.containsPoint(square, 50, 50), true);
assert.equal(core.containsPoint(square, 150, 150), false);

const transformed = core.transformPath(square, [1, 0, 0, 1, 25, 35]);
const movedBounds = core.pathBounds(transformed);
near(movedBounds.minX, 25);
near(movedBounds.minY, 35);

const split = core.splitSegment("M0 0 L10 0", 0, 0, 0.5);
near(core.pathLength(split), 10);
const sliced = core.sliceContour(square, 0, 75);
near(core.pathLength(sliced), 75, 0.01);

for (const path of [
  core.unionPaths(square, overlap),
  core.intersectPathAreas(square, overlap),
  core.subtractPaths(square, overlap),
  core.xorPaths(square, overlap),
]) {
  assert.equal(core.validatePath(path), true);
  assert.ok(core.pathLength(path) > 0);
}

const mesh = core.tessellateFill(square);
assert.ok(mesh.vertices.length >= 6);
assert.ok(mesh.indices.length >= 3);

assert.throws(() => core.pathLength("M0 0 L"));
for (let i = 0; i < 50; i++) {
  near(core.pathLength(line), 5);
  assert.equal(core.containsPoint(square, 25, 25), true);
}

console.log("TypeScript consumer + real Node WebAssembly integration: PASS");
