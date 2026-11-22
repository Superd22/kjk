//! The paint's controls in the browser, for the pages that build their sliders from them. The paint itself runs in `head_paint_gpu`.

use crate::controls;
use wasm_bindgen::prelude::*;

/// The paint's layers and controls as JSON: `{ "groups": [...], "controls": [...] }`, in the order that `GpuPainter::paint` reads them.
#[wasm_bindgen]
pub fn controls_json() -> String {
    controls::controls_json()
}

/// A recipe's values as JSON, one per control id, for the recipe of `values` (one number per control).
#[wasm_bindgen]
pub fn values_json(values: &[f64]) -> String {
    controls::values_json(&controls::recipe_from(values)).to_string()
}

/// The size of this module's memory in bytes. It only grows, so it is the most the module has held.
#[wasm_bindgen]
pub fn memory_bytes() -> f64 {
    (core::arch::wasm32::memory_size(0) * 65536) as f64
}
