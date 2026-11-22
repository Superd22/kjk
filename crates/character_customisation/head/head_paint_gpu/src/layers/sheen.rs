//! The sheen layer: the Rust half gives the three region amounts and the whiteness share. The shader multiplies them by the
//! interpolated region weights and shifts the lightness. The storage buffer is not used, so it holds four unused words.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::sheen::SHEEN_WHITENESS;

#[derive(Default)]
pub struct SheenLayer;

impl GpuLayer for SheenLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let s = input.recipe.sheen;
        let uniform = [s.cheeks, s.nose, s.chin, SHEEN_WHITENESS].map(|v| (v as f32).to_bits()).to_vec();
        LayerWords { uniform, storage: input.new_head.then(|| vec![0; 4]) }
    }

    fn paints(&self, id: &str) -> bool {
        ["sheen_cheeks", "sheen_nose", "sheen_chin"].contains(&id)
    }
}
