//! The light from maps (`shaders/maps.wgsl`): the processor's half gives the maps' sliders and the shares each layer keeps in the colour.
//! The layers read them from the shader's uniform while they paint, and leave their height, gloss and tile mask for the paint pass. It
//! has no storage buffer, because the fragment stage has room for only 8.

use super::{GpuLayer, LayerInput, LayerWords};

/// The size of the skin a texel covers at 2,048 pixels, in mm: the median over the face of the layout, measured on Bastila, Carth and
/// Kreia on 2026-11-20 (0.108 to 0.114 mm). Half the face's texels lie within 15% of it. A viewer repeats the detail tile over the
/// texture so that its millimetres match the skin's at this size.
pub const TILE_TEXEL_MM: f64 = 0.11;
/// The share of the gloss kept under the eyebrows' hairs and fill, so that the brows do not catch the skin's highlight.
pub const BROW_GLOSS: f64 = 0.2;
/// The skin grain, in L*, at which "Dry skin" takes its whole share of the gloss.
pub const FULL_GRAIN: f64 = 12.0;

#[derive(Default)]
pub struct MapsLayer;

fn f(x: f64) -> u32 {
    (x as f32).to_bits()
}

impl GpuLayer for MapsLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let m = &input.recipe.maps;
        let on = input.layers.maps;
        let grain = if input.layers.pores { (input.recipe.pores.grain / FULL_GRAIN).clamp(0.0, 1.0) } else { 0.0 };
        let uniform = vec![
            f(if on { 1.0 } else { 0.0 }),
            f(grain),
            0,
            0,
            f(m.skin_gloss),
            f(m.t_zone_gloss),
            f(m.sheen_gloss),
            f(m.lip_gloss),
            f(m.wet_rims),
            f(m.dry),
            f(BROW_GLOSS),
            0,
            f(m.sheen_kept),
            f(m.lip_shine_kept),
            f(m.light_kept),
            f(m.dark_kept),
            f(m.relief_mm / 10.0),
            f(m.wrinkle_relief),
            f(m.lip_relief),
            f(m.eye_relief),
            f(m.pore_rim_kept),
            f(m.grain_kept),
            f(m.grain_relief_mm),
            0,
        ];
        LayerWords { uniform, storage: None }
    }

    fn paints(&self, id: &str) -> bool {
        id.starts_with("maps_")
    }

    fn has_storage(&self) -> bool {
        false
    }
}
