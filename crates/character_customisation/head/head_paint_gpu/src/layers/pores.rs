//! The pore layer: the furrow net, the pores on its corners, the light grains with dark points between them, and the mottling
//! (`shaders/pores.wgsl`).
//!
//! The processor's half turns the recipe into the shader's numbers and builds the pore map's regions from the head's landmarks
//! (`head_paint::pores::pore_regions`). The shader does the work of every pixel.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::pores::{PORE_BACKGROUND, pore_regions};

/// The feature size of the mottling on the GPU, in millimetres.
pub const MOTTLING_MM: f64 = 10.0;

#[derive(Default)]
pub struct PoreLayer;

fn f(x: f64) -> u32 {
    (x as f32).to_bits()
}

/// The net's squeeze along the flow for a "Grain direction" from 0 to 1: distances along the flow count from 1 down to 0.5 times.
pub fn squeeze(direction: f64) -> f64 {
    1.0 - 0.5 * direction.clamp(0.0, 1.0)
}

/// A pore's mean long-to-short axis ratio for a "Pore stretch" from 0 to 1.
pub fn axis_ratio(stretch: f64) -> f64 {
    1.0 + 1.5 * stretch.clamp(0.0, 1.0)
}

impl GpuLayer for PoreLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let p = &input.recipe.pores;
        let on = input.layers.pores;
        let pores_on = on && p.density > 0.0 && p.depth != 0.0;
        let grain_on = on && p.grain != 0.0;
        let mottling_on = on && p.mottling != 0.0;
        let glint_on = on && (p.glint != 0.0 || p.glint_points != 0.0);
        let seed = |constant: u64| {
            let s = p.seed ^ constant;
            [s as u32, (s >> 32) as u32]
        };
        let (net, mottling) = (seed(0x2E7A), seed(0x3D17));
        let cell = p.grain_mm.clamp(0.2, 2.0);
        let regions = input.head.detail.as_ref().and_then(|d| d.folds.as_ref().map(|fold| pore_regions(&d.landmarks, &fold.landmarks, input.recipe.wrinkles.age))).unwrap_or_default();
        let mut storage = Vec::with_capacity(12 * regions.len() + 4);
        for r in &regions {
            storage.extend([r.a[0], r.a[1], r.a[2], r.reach_mm, r.b[0], r.b[1], r.b[2], 0.0, r.density, r.size, 0.0, 0.0].map(f));
        }
        if storage.is_empty() {
            storage.extend([0; 4]);
        }
        let uniform = vec![
            f(cell),
            f(p.size_mm),
            f(p.depth),
            f(p.grain),
            f(p.mottling),
            f(MOTTLING_MM),
            f(squeeze(p.grain_direction)),
            f(axis_ratio(p.stretch)),
            f(p.gradient.max(0.0)),
            f(p.density / 100.0),
            f(regions.len() as f64),
            0,
            f(PORE_BACKGROUND[0]),
            f(PORE_BACKGROUND[1]),
            f(PORE_BACKGROUND[2]),
            0,
            net[0],
            net[1],
            mottling[0],
            mottling[1],
            pores_on as u32,
            grain_on as u32,
            mottling_on as u32,
            0,
            f(if glint_on { p.glint } else { 0.0 }),
            f(p.glint_mm.clamp(0.2, 2.0)),
            f(if glint_on { p.glint_points } else { 0.0 }),
            f(p.glint_variation.clamp(0.0, 1.0)),
            f(p.glint_follow.clamp(0.0, 1.0)),
            f(p.crater.clamp(0.0, 1.0)),
            f(p.crater_scale.clamp(1.0, 3.0)),
            f(p.gleam.clamp(0.0, 1.0)),
        ];
        LayerWords { uniform, storage: Some(storage) }
    }

    fn paints(&self, id: &str) -> bool {
        ["pore_density", "pore_gradient", "pore_size", "pore_depth", "pore_stretch", "pore_seed", "grain", "grain_size", "grain_direction", "mottling", "grain_light", "grain_light_size", "grain_dark_points", "grain_light_variation", "grain_light_follow", "pore_crater", "pore_scale", "pore_gleam"].contains(&id)
    }
}
