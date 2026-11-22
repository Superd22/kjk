//! The eyebrows (`head_paint::brows`). The processor lays each brow's middle line on the skin from the shape sliders, and the shader
//! draws the shadow, the fill and the hairs per pixel (`shaders/brows.wgsl`).
//!
//! Uniform buffer, in `vec4<f32>` rows: eight rows of settings, colours and the face's middle (see `BrowParams` in the shader), then for
//! each brow 33 points with their distance along the brow, 33 directions up the skin with the half thickness, and one row with the
//! brow's length and whether it is painted. The layer has no storage buffer, so that the facial hair can have one.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::brows::{BrowLine, Brows, POINTS};
use head_paint::eyes::lch_to_linear;

/// The colour of grey hairs as L*, hue in degrees and chroma: a near white with a little warmth.
const GREY_LCH: [f64; 3] = [70.0, 75.0, 5.0];

#[derive(Default)]
pub struct BrowLayer {
    /// The shape sliders and the head that the lines were made from, and the lines.
    key: Option<(u64, [u64; 12])>,
    lines: Option<[BrowLine; 2]>,
    head_count: u64,
}

fn f(v: f64) -> u32 {
    (v as f32).to_bits()
}

fn shape_key(b: &Brows) -> [u64; 12] {
    [b.height_mm, b.tilt_mm, b.arch_mm, b.peak_mm, b.corner_mm, b.corner_at, b.thick_mm, b.taper_mm, b.belly_mm, b.inner_mm, b.outer_mm, b.tail].map(f64::to_bits)
}

impl GpuLayer for BrowLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let b: &Brows = &input.recipe.brows;
        if input.new_head {
            self.head_count += 1;
        }
        let head = input.head.detail.as_ref().and_then(|d| d.brows.as_ref());
        let rows = 8 + 2 * (2 * POINTS + 1);
        let storage = None;
        let Some(head) = head.filter(|_| input.layers.brows && !b.is_none()) else {
            return LayerWords { uniform: vec![0; 4 * rows], storage };
        };
        let key = (self.head_count, shape_key(b));
        if self.key != Some(key) {
            self.lines = Some([head.line(b, 0), head.line(b, 1)]);
            self.key = Some(key);
        }
        let lines = self.lines.as_ref().expect("lines");
        let colour = lch_to_linear([b.lightness, b.hue, b.chroma]);
        let grey = lch_to_linear(GREY_LCH);
        let mut w: Vec<u32> = Vec::with_capacity(4 * rows);
        w.extend([1.0, b.amount.clamp(0.0, 1.0), b.density.clamp(0.0, 1.0), b.length_mm].map(f));
        w.extend([b.hair_mm, b.angle_degrees.to_radians().tan(), b.inner_up, b.messy].map(f));
        w.extend([b.spill_mm, b.trim, b.inner_full, b.fill].map(f));
        w.extend([b.shadow, b.tone_var, b.grey, 0.0].map(f));
        w.extend([colour[0], colour[1], colour[2], 0.0].map(f));
        w.extend([grey[0], grey[1], grey[2], 0.0].map(f));
        w.extend([head.middle[0], head.middle[1], head.middle[2], 0.0].map(f));
        w.extend([head.across[0], head.across[1], head.across[2], 0.0].map(f));
        for line in lines {
            for (p, s) in &line.points {
                w.extend([p[0], p[1], p[2], *s].map(f));
            }
            for (u, h) in &line.across {
                w.extend([u[0], u[1], u[2], *h].map(f));
            }
            w.extend([line.length, 1.0, 0.0, 0.0].map(f));
        }
        LayerWords { uniform: w, storage }
    }

    fn paints(&self, id: &str) -> bool {
        id.starts_with("brow_")
    }

    fn has_storage(&self) -> bool {
        false
    }
}
