// Loads a Rust crate built for the browser by ~/.claude/tools/likeness-fit/sliders/build_wasm.sh, and moves typed arrays in and out.
// A page served at http://127.0.0.1:8765/ imports it as a module:
//   import { loadCrate, fetchTyped, fromBase64 } from "/wasm/load.js";
//   const m = await loadCrate("head_sliders", "/wasm", "2"); // the crate's exports, ready to call; bump "2" after a rebuild
//   const base = await fetchTyped("data/base.bin", Float32Array);
// wasm-bindgen copies a typed array passed to a Rust slice (&[f32] takes a Float32Array, &[u32] a Uint32Array, &[u8] a Uint8Array,
// &[f64] a Float64Array) and returns a Vec<f32> as a new Float32Array. JSON goes across as strings.

// `version` is added to the files' addresses, so a rebuilt module is not taken from the browser's cache.
export async function loadCrate(crate, root = "/wasm", version = "") {
  const t0 = performance.now();
  const q = version ? `?v=${version}` : "";
  const m = await import(`${root}/${crate}/${crate}.js${q}`);
  await m.default({ module_or_path: `${root}/${crate}/${crate}_bg.wasm${q}` });
  return { ...m, loadMs: performance.now() - t0 };
}

export async function fetchTyped(url, Type) {
  const r = await fetch(url);
  if (!r.ok) throw new Error(`${url}: ${r.status}`);
  return new Type(await r.arrayBuffer());
}

export function fromBase64(b64, Type) {
  const s = atob(b64);
  const u = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) u[i] = s.charCodeAt(i);
  return new Type(u.buffer);
}

// Runs fn and returns [its result, the milliseconds it took].
export function timed(fn) {
  const t0 = performance.now();
  const r = fn();
  return [r, performance.now() - t0];
}
