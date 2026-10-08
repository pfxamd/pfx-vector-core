import init, * as wasm from "../crates/pfx-vector-wasm/pkg/pfx_vector_wasm.js";

const byId = (id) => document.getElementById(id);
const source = byId("path");
const slider = byId("distance");
const status = byId("status");
let engineReady = false;
let totalLength = 0;
let lastValid = "";

function setStatus(message, ready = false) {
  status.textContent = message;
  status.classList.toggle("ready", ready);
}

function updatePosition() {
  if (!engineReady || !lastValid) return;
  const fraction = Number(slider.value) / 1000;
  byId("fraction").textContent = Math.round(fraction * 100) + "%";
  const start = performance.now();
  const data = JSON.parse(wasm.frame_at_length_svg(lastValid, totalLength * fraction));
  const duration = performance.now() - start;
  byId("position").setAttribute("cx", data.x);
  byId("position").setAttribute("cy", data.y);
  byId("normal").setAttribute("x1", data.x);
  byId("normal").setAttribute("y1", data.y);
  byId("normal").setAttribute("x2", data.x + data.normalX * 34);
  byId("normal").setAttribute("y2", data.y + data.normalY * 34);
  byId("coordinates").textContent = data.x.toFixed(1) + ", " + data.y.toFixed(1);
  byId("segment").textContent = String(data.segment);
  byId("latency").textContent = duration.toFixed(2) + " ms";
}

function updatePath() {
  if (!engineReady) return;
  const data = source.value.trim();
  try {
    if (!wasm.validate_path(data)) throw new Error("Invalid SVG path");
    const length = wasm.path_length_svg(data);
    if (!Number.isFinite(length) || length <= 0) throw new Error("Path must have positive finite length");
    const flattened = wasm.flatten_path_svg(data, 0.75);
    byId("original").setAttribute("d", data);
    byId("flattened").setAttribute("d", flattened);
    byId("length").textContent = length.toFixed(2);
    byId("error").textContent = "";
    lastValid = data;
    totalLength = length;
    updatePosition();
  } catch (error) {
    byId("error").textContent = error instanceof Error ? error.message : String(error);
  }
}

for (const button of document.querySelectorAll("[data-path]")) {
  button.addEventListener("click", () => {
    source.value = button.dataset.path;
    updatePath();
  });
}
source.addEventListener("input", updatePath);
slider.addEventListener("input", updatePosition);

try {
  await init();
  engineReady = true;
  setStatus("ENGINE ONLINE", true);
  updatePath();
} catch (error) {
  setStatus("ENGINE UNAVAILABLE");
  byId("error").textContent = "WebAssembly failed to load: " + String(error);
}
