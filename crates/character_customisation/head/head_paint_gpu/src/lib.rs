//! The head paint of `head_paint`, painted on the GPU (paint track, part C, decision 41).
//!
//! The processor does what is not per pixel: the head's per-vertex fields, its landmarks and lines, and the colour of the base tone. The
//! shaders in `shaders/` do everything per pixel. The same crate and the same shaders run natively and in the browser through WebGPU.
//! `painter.rs` describes the passes, and `layers/mod.rs` how a layer is split between the processor and a shader.

pub mod gpu;
pub mod eyeball;
pub mod head;
pub mod layers;
pub mod painter;
pub mod probe;
pub mod tile;
#[cfg(target_arch = "wasm32")]
pub mod web;
