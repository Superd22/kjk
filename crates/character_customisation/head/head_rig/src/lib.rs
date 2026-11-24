//! The rig of a customized head (roadmap A1). KOTOR's clips move a head through its face bones, so every edited head needs those bones
//! in the right places, and teeth that the jaw carries. This crate places them from the head's own vertices:
//! - `regressor`: each bone and each teeth point is a fixed weighted sum of the head's vertices, learned from KOTOR's 57 test heads;
//! - `bones`: the placed bones turned into the local places that KOTOR's clips add their offsets to, and the scale of those offsets;
//! - `teeth`: an upper and a lower teeth strip through the placed teeth points, with a texture of our own;
//! - `weights`: the skin weights of the mouth, the brows and the cheeks, computed from the head's own shape;
//! - `lips`: the lip line opened, so the upper lip's curl no longer hangs below the lower lip at rest;
//! - `shade`: how dark the inside of the mouth is painted, from its depth behind the lips;
//! - `tongue`: a tongue from the tongue bone to behind the lower teeth, carried by the jaw and the tongue bone;
//! - `rig`: all of the above for one head in one call, which `mouth_rig` and the browser both use;
//! - `calibrate`: learning the weights rules from our fitted heads and KOTOR's motion.
//!
//! The findings behind it are in `docs/research/2026-11-19-head-roadmap.md`, under "The mouth when it talks (A1)".

pub mod bones;
#[cfg(not(target_arch = "wasm32"))]
pub mod calibrate;
pub mod lips;
pub mod regressor;
pub mod rig;
pub mod shade;
pub mod teeth;
pub mod tongue;
#[cfg(target_arch = "wasm32")]
pub mod web;
pub mod weights;

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

pub fn length(a: V3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
