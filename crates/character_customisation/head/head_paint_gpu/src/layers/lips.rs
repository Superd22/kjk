//! The lips (`head_paint::lips`). The processor lays out the head's lip curves after the sliders' changes of shape (`LipShape`), and
//! the shader paints the lip colour, the border and the light and shade per pixel (`shaders/lips.wgsl`).
//!
//! Uniform buffer, twelve `vec4<f32>`:
//! 0. the mouth frame's origin in millimetres, and the cylinder's radius;
//! 1. `across`, and the `u` of the curves' first sample;
//! 2. `up`, and the curves' step in millimetres;
//! 3. `forward`, and the number of samples of each curve;
//! 4. the front map's lowest `u` and `v`, its cell in millimetres, and its number of columns;
//! 5. its number of rows, 1 when the lips paint, the border softness in millimetres, and the lipstick's amount;
//! 6. the natural lip colour's change from the skin in Lab, and the darker upper lip in L*;
//! 7. the lipstick's colour in Lab, and the lower lip's light in L*;
//! 8. the mouth line, the liner, the light rim and the lip lines, in L*;
//! 9. the two corners' (u, v);
//! 10. the half widths of the x < 0 and the x > 0 side, and two unused numbers;
//! 11. the gloss, the shadow under the lower lip in L*, and two unused numbers.
//!
//! Storage buffer: the upper border's `v` at each sample, then the lower border's, then the opening's, then the front map's `w`, row by
//! row from the lowest `v`, with NaN where the head has no surface.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::lips::{FRONT_CELL_MM, FRONT_U_MM, FRONT_V_MM, LipHead, LipShape, Lips, RADIUS_MM, STEP_MM};
use head_paint::pigment::SpectralModel;

#[derive(Default)]
pub struct LipLayer {
    model: Option<SpectralModel>,
    /// What the storage buffer was made from: the head's curves and the shape sliders.
    key: Option<(Vec<u64>, [u64; 5])>,
    shape: Option<LipShape>,
}

fn f(v: f64) -> u32 {
    (v as f32).to_bits()
}

fn head_key(h: &LipHead) -> Vec<u64> {
    h.upper.iter().chain(&h.lower).chain(&h.opening).flat_map(|p| [p[0].to_bits(), p[1].to_bits()]).collect()
}

impl GpuLayer for LipLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let lips: &Lips = &input.recipe.lips;
        let head = input.head.detail.as_ref().and_then(|d| d.lips.as_ref());
        let Some(head) = head.filter(|_| input.layers.lips && !lips.is_none()) else {
            let storage = (self.key.is_some() || input.new_head).then(|| vec![0; 4]);
            self.key = None;
            return LayerWords { uniform: vec![0; 48], storage };
        };
        let key = (head_key(head), [lips.upper_mm, lips.lower_mm, lips.width_mm, lips.bow_mm, lips.lopsided_mm].map(f64::to_bits));
        let mut storage = None;
        if input.new_head || self.key.as_ref() != Some(&key) {
            let shape = LipShape::new(lips, head);
            let mut words: Vec<u32> = shape.upper.iter().chain(&shape.lower).chain(&shape.opening).map(|&v| f(v)).collect();
            words.extend(head.front.iter().map(|w| w.to_bits()));
            while words.len() % 4 != 0 {
                words.push(0);
            }
            storage = Some(words);
            self.shape = Some(shape);
            self.key = Some(key);
        }
        let shape = self.shape.as_ref().expect("shape");
        let model = self.model.get_or_insert_with(SpectralModel::new);
        let natural = lips.natural_shift(&input.recipe.base_tone, model);
        let stick = lips.stick_lab();
        let fr = &head.frame;
        let n = shape.upper.len();
        let c = shape.corners;
        let uniform = vec![
            f(fr.origin[0]), f(fr.origin[1]), f(fr.origin[2]), f(RADIUS_MM),
            f(fr.across[0]), f(fr.across[1]), f(fr.across[2]), f(shape.u0),
            f(fr.up[0]), f(fr.up[1]), f(fr.up[2]), f(STEP_MM),
            f(fr.forward[0]), f(fr.forward[1]), f(fr.forward[2]), f(n as f64),
            f(-FRONT_U_MM), f(FRONT_V_MM[0]), f(FRONT_CELL_MM), f(head.front_dims[0] as f64),
            f(head.front_dims[1] as f64), f(1.0), f(lips.softness_mm.max(0.05)), f(lips.stick.clamp(0.0, 1.0)),
            f(natural[0]), f(natural[1]), f(natural[2]), f(lips.upper_shade),
            f(stick[0]), f(stick[1]), f(stick[2]), f(lips.highlight),
            f(lips.mouth_line), f(lips.liner), f(lips.white_roll), f(lips.lines),
            f(c[0][0]), f(c[0][1]), f(c[1][0]), f(c[1][1]),
            f(-c[0][0]), f(c[1][0]), 0, 0,
            f(lips.gloss.clamp(0.0, 1.0)), f(lips.shadow), 0, 0,
        ];
        LayerWords { uniform, storage }
    }

    fn paints(&self, id: &str) -> bool {
        id.starts_with("lips_")
    }
}
