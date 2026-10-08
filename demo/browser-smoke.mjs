import { chromium } from "playwright";
import assert from "node:assert/strict";

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
const errors = [];
page.on("pageerror", (error) => errors.push(error.message));

try {
  await page.goto("http://127.0.0.1:8787/demo/", { waitUntil: "networkidle" });
  await page.locator("#status.ready").waitFor({ timeout: 30000 });
  const firstLength = Number((await page.locator("#length").textContent()).trim());
  assert.ok(firstLength > 0 && Number.isFinite(firstLength));
  await page.locator("#distance").fill("875");
  assert.equal((await page.locator("#fraction").textContent()).trim(), "88%");
  assert.match((await page.locator("#coordinates").textContent()).trim(), /^-?\d+\.\d, -?\d+\.\d$/);

  await page.getByRole("button", { name: "Arcs" }).click();
  const arcLength = Number((await page.locator("#length").textContent()).trim());
  assert.ok(arcLength > 0 && arcLength !== firstLength);
  await page.locator("#path").fill("M0 0 L");
  assert.ok((await page.locator("#error").textContent()).trim().length > 0);
  await page.getByRole("button", { name: "Mixed" }).click();
  assert.equal((await page.locator("#error").textContent()).trim(), "");

  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.locator("#view").isVisible());
  assert.deepEqual(errors, []);
  console.log("Browser geometry integration: PASS");
} finally {
  await browser.close();
}
