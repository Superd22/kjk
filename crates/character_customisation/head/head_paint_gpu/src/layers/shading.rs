//! The shading layer: the lightness shift of the sky light and the bump at each point. The processor sends the layer's settings and the
//! constants of the shared pattern, and the shader shifts the colour's lightness by the strength times the pattern.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::shading::{BUMP_GAIN, SKY_GAIN, SKY_REFERENCE};

#[derive(Default)]
pub struct ShadingLayer;

impl GpuLayer for ShadingLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let s = &input.recipe.shading;
        let floats = [s.strength, s.highlight_whiteness, SKY_GAIN, SKY_REFERENCE, BUMP_GAIN, 0.0, 0.0, 0.0];
        LayerWords { uniform: floats.iter().map(|&f| (f as f32).to_bits()).collect(), storage: input.new_head.then(|| vec![0; 4]) }
    }

    fn paints(&self, id: &str) -> bool {
        ["shading_strength", "highlight_whiteness"].contains(&id)
    }
}
