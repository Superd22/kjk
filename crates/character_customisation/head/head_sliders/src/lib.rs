//! Shape sliders for customized heads (roadmap B5, decisions 46 to 52 of `docs/research/2026-11-19-head-roadmap.md`).
//!
//! A head is nothing but its slider values: the base head, plus each slider's value times that slider's fixed movement of the
//! vertices, then the smoothing that every built head gets (`head_shape`). The same value gives the same movement on every head, and
//! edits add up in any order.
//!
//! The crate is a kit: a feature (`nose`, `jaw`) supplies only its measure, zone, parts and slider specs (`feature::Feature`), and the
//! rest is shared:
//! - `mesh`: the head mesh every head shares, and the inputs the tools read;
//! - `set`: a slider set as plain data, and building a head from slider values, for one feature or several at once;
//! - `zone`: a feature's zone with a soft edge, and the part of a head's shape that belongs to the feature alone;
//! - `measure`: the frame of the eyes, surfaces and cuts, and the nose's measure; `jaw`: the jaw and chin's measure;
//! - `maker`: the named sliders, each setting one measure and holding the others; `check`: folds and turned-over triangles;
//! - `solve`: the slider values whose feature best matches a measured one; `verdict`: a made feature against its target, in words;
//! - `checks`, `heads`, `bench`, `render` (native only): the checks of a measure, the KOTOR heads and their stand-ins, the tool's
//!   commands and the clay pictures;
//! - `round1`: the nose's first sliders, learned from the KOTOR heads, kept for `nose_bench`;
//! - `web` (WebAssembly only): building a head and measuring its features in the browser.

#[cfg(not(target_arch = "wasm32"))]
pub mod bald;
#[cfg(not(target_arch = "wasm32"))]
pub mod bench;
pub mod brow;
pub mod check;
pub mod cheeks;
#[cfg(not(target_arch = "wasm32"))]
pub mod checks;
pub mod eye_curves;
pub mod eyes;
pub mod feature;
#[cfg(not(target_arch = "wasm32"))]
pub mod heads;
pub mod jaw;
pub mod jaw_edge;
pub mod maker;
pub mod measure;
pub mod mesh;
pub mod nose;
pub mod nose_profile;
#[cfg(not(target_arch = "wasm32"))]
pub mod nose_base;
#[cfg(not(target_arch = "wasm32"))]
pub mod render;
pub mod round1;
pub mod set;
#[cfg(not(target_arch = "wasm32"))]
pub mod skull;
pub mod solve;
pub mod throat_corner;
pub mod chin_neck;
#[cfg(not(target_arch = "wasm32"))]
pub mod jaw_skull;
#[cfg(not(target_arch = "wasm32"))]
pub mod lips;
pub mod under_jaw;
pub mod verdict;
pub mod zone;

#[cfg(target_arch = "wasm32")]
pub mod web;

pub type V3 = [f64; 3];

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn scale(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn length(a: V3) -> f64 {
    dot(a, a).sqrt()
}

pub fn normalize(a: V3) -> V3 {
    scale(a, 1.0 / length(a).max(1e-300))
}
