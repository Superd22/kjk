//! The eye opening built from a curve on the eyeball. The curve is drawn from ten handles. The lid's rim is laid on it, the layers under
//! the rim go back against the eyeball, and the skin in a ring round the eye is stretched and lowered to meet it. The lid piece is the
//! upper arch swept down past the lower arch about a line through the eyeball's centre.
//!
//! It began on 2026-11-21 as a prototype for viewer 2's preview, and it is now part of how every head is built: `head_sliders` reads
//! `curve::HANDLES` for the opening's ten values, builds each eye through `maker`, and takes the finer triangles below the lower lids
//! from `refine`. It keeps the prototype's sizes (its environment defaults) and its "polar" mode only, and it does not depend on
//! `head_sliders`, so the dependency runs one way.

pub mod curve;
pub mod maker;
pub mod refine;
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
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn length(a: V3) -> f64 {
    dot(a, a).sqrt()
}

pub fn normalize(a: V3) -> V3 {
    scale(a, 1.0 / length(a).max(1e-300))
}
