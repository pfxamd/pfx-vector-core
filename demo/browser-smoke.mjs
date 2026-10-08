import { chromium, firefox, webkit } from "playwright";
import assert from "node:assert/strict";

for (const [browserName, browserType] of [["chromium", chromium], ["firefox", firefox], ["webkit", webkit]]) {
const browser = await browserType.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
const errors = [];
page.on("pageerror", (error) => errors.push(error.message));

try {
  await page.goto("http://127.0.0.1:8787/demo/", { waitUntil: "networkidle" });
  await page.locator("#status.ready").waitFor({ timeout: 30000 });
  const firstLength = Number((await page.locator("#length").textContent()).trim());
  assert.ok(firstLength > 0 && Number.isFinite(firstLength));
  await page.locator("#distance").evaluate((element) => {
    element.value = "875";
    element.dispatchEvent(new Event("input", { bubbles: true }));
  });
  assert.equal((await page.locator("#fraction").textContent()).trim(), "88%");
  assert.match((await page.locator("#coordinates").textContent()).trim(), /^-?\d+\.\d, -?\d+\.\d$/);

  await page.getByRole("button", { name: "Arcs" }).click();
  const arcLength = Number((await page.locator("#length").textContent()).trim());
  assert.ok(arcLength > 0 && arcLength !== firstLength);
  await page.locator("#path").fill("M0 0 L");
  assert.ok((await page.locator("#error").textContent()).trim().length > 0);
  await page.getByRole("button", { name: "Mixed" }).click();
  assert.equal((await page.locator("#error").textContent()).trim(), "");


  // Call real Rust/WASM exports from Chromium and validate geometric invariants.
  const kernel = await page.evaluate(async () => {
    const wasm = await import("/crates/pfx-vector-wasm/pkg/pfx_vector_wasm.js");
    const square = "M 0 0 L 100 0 L 100 100 L 0 100 Z";
    const overlap = "M 50 50 L 150 50 L 150 150 L 50 150 Z";
    const union = wasm.boolean_union_svg(square, overlap);
    const intersection = wasm.boolean_intersection_svg(square, overlap);
    const subtraction = wasm.boolean_difference_svg(square, overlap);
    const xor = wasm.boolean_xor_svg(square, overlap);
    const translated = wasm.transform_path_svg(square, "1,0,0,1,25,35", 0.0001);
    const flattened = wasm.flatten_path_svg("M0 0 C40 90 80 -90 120 0", 0.5);
    const sliced = wasm.slice_contour_svg(square, 0, 0, 75, false);
    return {
      union: wasm.path_length_svg(union),
      intersection: wasm.path_length_svg(intersection),
      subtraction: wasm.path_length_svg(subtraction),
      xor: wasm.path_length_svg(xor),
      translated: JSON.parse(wasm.path_bounds_svg(translated)),
      flattened: wasm.path_length_svg(flattened),
      sliced: wasm.path_length_svg(sliced),
      crossed: wasm.intersect_paths_svg("M0 0 L100 100", "M0 100 L100 0"),
    };
  });
  for (const name of ["union", "intersection", "subtraction", "xor", "flattened", "sliced"]) {
    assert.ok(Number.isFinite(kernel[name]) && kernel[name] > 0, name);
  }
  assert.ok(Math.abs(kernel.sliced - 75) < 0.01);
  assert.ok(Math.abs(kernel.translated.minX - 25) < 0.01);
  assert.ok(Math.abs(kernel.translated.minY - 35) < 0.01);
  assert.ok(JSON.parse(kernel.crossed).length >= 1);

  // Verify the real consumer-facing TypeScript adapter in each browser.
  const consumer = await page.evaluate(async () => {
    const wasm = await import("/crates/pfx-vector-wasm/pkg/pfx_vector_wasm.js");
    const { createVectorCore } = await import("/packages/pfx-vector-web/dist/index.js");
    const core = createVectorCore(wasm);
    const path = "M0 0 L100 0 L100 100 L0 100 Z";
    let invalidInputRejected = false;
    try {
      core.pathLength("M0 0 L");
    } catch {
      invalidInputRejected = true;
    }
    return {
      length: core.pathLength("M0 0 L3 4"),
      bounds: core.pathBounds(path),
      inside: core.containsPoint(path, 50, 50),
      outside: core.containsPoint(path, 200, 200),
      midpoint: core.pointAtLength("M0 0 L3 4", 2.5),
      invalidInputRejected,
    };
  });
  assert.ok(Math.abs(consumer.length - 5) < 1e-9);
  assert.ok(Math.abs(consumer.bounds.maxX - 100) < 1e-6);
  assert.equal(consumer.inside, true);
  assert.equal(consumer.outside, false);
  assert.ok(Math.abs(consumer.midpoint.x - 1.5) < 1e-6);
  assert.equal(consumer.invalidInputRejected, true);

  // Browser-side large geometry workload with actual WASM, not simulated shapes.
  const stress = await page.evaluate(async () => {
    const wasm = await import("/crates/pfx-vector-wasm/pkg/pfx_vector_wasm.js");
    let d = "M0 0";
    for (let i = 0; i < 160; i++) {
      const x = i * 3;
      d += ` C${x + 1} 70 ${x + 2} -70 ${x + 3} 0`;
    }
    const t0 = performance.now();
    const length = wasm.path_length_svg(d);
    const measuredMs = performance.now() - t0;
    const flat = wasm.flatten_path_svg(d, 1);
    return {
      length,
      measuredMs,
      valid: wasm.validate_path(flat),
      size: d.length
    };
  });
  assert.ok(stress.size > 3000);
  assert.ok(Number.isFinite(stress.length) && stress.length > 0);
  assert.equal(stress.valid, true);
  assert.ok(stress.measuredMs < 10000, `160-segment browser workload took ${stress.measuredMs}ms`);

  // Repeated editing exercises browser-to-WASM calls without hiding exceptions.
  for (let i = 0; i < 60; i++) {
    await page.locator("#distance").evaluate((element, step) => {
      element.value = String(step * 16);
      element.dispatchEvent(new Event("input", { bubbles: true }));
    }, i);
  }

  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.locator("#view").isVisible());
  assert.deepEqual(errors, []);
  console.log(`Browser geometry integration (${browserName}): PASS`);
} finally {
  await browser.close();
}
}
