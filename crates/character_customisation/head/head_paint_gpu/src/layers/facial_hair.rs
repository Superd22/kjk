//! The facial hair (`head_paint::facial_hair`). The processor writes the hair groups onto the head's paint grid,
//! and the shader draws the shadow and the single hairs per pixel (`shaders/facial_hair.wgsl`, with the shared drawing of
//! `shaders/hair_grid.wgsl`).
//!
//! The layer has no uniform buffer, because a shader stage may bind at most 12 and the other layers use them all. Its storage buffer
//! holds four rows of settings (see `facial_hair.wgsl`), then the face grid with its field, its looks and its lookup, as
//! `FaceGrid::words` lays them out from row 4, then the lower lip's shape, which is drawn over the hairs.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::eyes::lch_to_linear;
use head_paint::hair_groups::{HairGroup, HairGroups};
use head_paint::lips::{LipShape, RADIUS_MM, STEP_MM};
use head_paint::pigment::{linear_to_lab, PigmentTable};

/// The colour of grey hairs as L*, hue in degrees and chroma: a near white with a little warmth, as for the brows.
const GREY_LCH: [f64; 3] = [70.0, 75.0, 5.0];

/// A group's look as nine rows for the shader: the linear colour and its L*, then the thickness in mm, the tone variation, the share of
/// grey hairs and the curl, then the sheen, the random pattern, how far the hairs follow the flow and the locks' wave, then the locks'
/// size in mm, gathering, highlight and softness, then the strands' width in mm, straightness, shading and bands, then their light at
/// the root, their layering, their packing and their shading by lock, then how parallel they run, their fine lines, their feathered edges
/// and how far the hairs hang over the lips, then the clumps' width in mm, their wave, the light zones and the mixed edges, then the combed
/// streaks, their length in mm, their sharpness and a 0.
fn look(g: &HairGroup) -> [f64; 36] {
    let [l, h, c, thickness, tone_var, grey, curl, sheen, seed, lock_mm, gather, light, flow, wave, soft, strand, straight, shade, bands, root_light, layering, packing, lock_tone, parallel, fine_lines, feather, over_lips, clump, clump_wave, light_zones, edge_mix, comb, streak_mm, streak_sharp] = g.look();
    let colour = lch_to_linear([l, h, c]);
    [colour[0], colour[1], colour[2], l, thickness, tone_var, grey, curl, sheen, seed, flow, wave, lock_mm, gather, light, soft, strand, straight, shade, bands, root_light, layering, packing, lock_tone, parallel, fine_lines, feather, over_lips, clump, clump_wave, light_zones, edge_mix, comb, streak_mm, streak_sharp, 0.0]
}

#[derive(Default)]
pub struct FacialHairLayer {
    /// The words of the last repaint, sent again only when they change.
    last: Option<Vec<u32>>,
    table: Option<PigmentTable>,
}

impl GpuLayer for FacialHairLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let head = input.head.detail.as_ref().and_then(|d| d.beard.as_ref());
        let groups: &HairGroups = input.hair_groups;
        let paints = groups.groups.iter().any(|g| !g.hidden) || input.hair_edit.is_some();
        let table = self.table.get_or_insert_with(PigmentTable::build);
        let base_l = linear_to_lab(input.recipe.base_colour(input.layers, table))[0];
        let words: Vec<u32> = match head.filter(|_| input.layers.facial_hair && paints) {
            None => vec![0; 4],
            Some(head) => {
                let grey = lch_to_linear(GREY_LCH);
                let mut w: Vec<f32> = Vec::new();
                // The field's shadow is already in L*, so its strength here is 1, then the most that any group's hairs follow the flow,
                // then the widest painted strand of any group, then the most that any group's strands are layered.
                // Each group's random pattern is in its look, and the lattices' own seed stays 0, as it was for every head before the
                // hair groups took the sliders.
                let flow = groups.groups.iter().filter(|g| !g.hidden && !g.pull).map(|g| g.look()[12]).fold(0.0, f64::max);
                let strand = groups.groups.iter().filter(|g| !g.hidden && !g.pull).map(|g| g.look()[15]).fold(0.0, f64::max);
                let layering = groups.groups.iter().filter(|g| !g.hidden && !g.pull).map(|g| g.look()[20]).fold(0.0, f64::max);
                w.extend([1.0, flow as f32, strand as f32, layering as f32]);
                // Row 1 holds the L* of the head's base skin tone, which says whether dark hair is kept dark against the skin, and the
                // direction the light zones are lit from: from above and 27 degrees to the front.
                let (f, u) = (head.grid.forward, head.grid.up);
                let light = head_paint::layout_points::normalize([0, 1, 2].map(|k| 0.5 * f[k] + u[k]));
                w.extend([base_l, light[0], light[1], light[2]].map(|v| v as f32));
                w.extend([grey[0], grey[1], grey[2], 1.0].map(|v| v as f32));
                // Row 3 holds in y the sign that turns the grid's own normal outward, for the light zones.
                w.extend([0.0, head.grid.normal_sign() as f32, 0.0, 0.0]);
                let mut looks: Vec<[f64; 36]> = groups.groups.iter().map(look).collect();
                if looks.is_empty() {
                    looks.push(look(&HairGroup::preset("stubble").expect("the stubble preset")));
                }
                w.extend(head.words(groups, input.hair_edit, &looks, 4));
                // The lower lip, drawn over the hairs: the lips' frame, then the lower border and the middle of the opening every
                // `lips::STEP_MM` along the mouth, as the lip layer paints them after its sliders. Its first row goes in row 3, z.
                if let Some(lips) = input.head.detail.as_ref().and_then(|d| d.lips.as_ref()) {
                    w[14] = (w.len() / 4) as f32;
                    let shape = LipShape::new(&input.recipe.lips, lips);
                    let fr = &lips.frame;
                    for v in [fr.origin, fr.across, fr.up, fr.forward] {
                        w.extend([v[0], v[1], v[2], 0.0].map(|x| x as f32));
                    }
                    let n = shape.lower.len().min(shape.opening.len());
                    w.extend([shape.u0, STEP_MM, n as f64, RADIUS_MM].map(|x| x as f32));
                    for i in 0..n.div_ceil(2) {
                        let at = |k: usize| (shape.lower[k.min(n - 1)] as f32, shape.opening[k.min(n - 1)] as f32);
                        let (a, b) = (at(2 * i), at(2 * i + 1));
                        w.extend([a.0, a.1, b.0, b.1]);
                    }
                }
                w.into_iter().map(f32::to_bits).collect()
            }
        };
        let storage = if input.new_head || self.last.as_ref() != Some(&words) { Some(words.clone()) } else { None };
        self.last = Some(words);
        LayerWords { uniform: vec![0; 4], storage }
    }

    fn paints(&self, id: &str) -> bool {
        id.starts_with("beard_")
    }

    fn has_uniform(&self) -> bool {
        false
    }
}
