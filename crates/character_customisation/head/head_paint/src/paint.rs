//! The recipe of every paint layer and the switches of which layers are painted (paint track, part C).
//!
//! The GPU painter in `head_paint_gpu` turns a recipe into a texture.

use crate::pigment::{BaseTone, PigmentTable, decode, encode};
use crate::shading::Shading;
use crate::sheen::Sheen;
use crate::layout_points::LayoutLocator;
use crate::pores::{PoreLandmarks, Pores};
use crate::wrinkles::{HeadLines, Wrinkles};
use crate::folds::FoldHead;
use crate::lines::{ForeheadHead, ForeheadLines};
use crate::lips::{LipHead, Lips};

/// The encoded colour of a texture with no paint at all, as the viewer data draws an unpainted point: linear 0.2, encoded with gamma 2.2.
pub const NEUTRAL: [u8; 3] = [123, 123, 123];
/// How many pixels the paint grows past the edges of the layout's patches.
pub const SEAM_GROWTH: f64 = 8.0;

/// The settings of every paint layer. Later layers (regional tints, beard shadow, lips) are added here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Recipe {
    pub base_tone: BaseTone,
    pub shading: Shading,
    pub sheen: Sheen,
    /// The wrinkles and folds (`wrinkles.rs`), painted after the sheen.
    pub wrinkles: Wrinkles,
    /// The forehead lines (`lines.rs`). They are painted with the wrinkles and share their switch. At an amount of zero they paint nothing.
    pub forehead: ForeheadLines,
    /// The pores, fine grain and mottling (`pores.rs`), painted last.
    pub pores: Pores,
    /// The eye paint (`eyes.rs`). Only the GPU painter paints it.
    pub eyes: crate::eyes::EyePaint,
    /// The lips (`lips.rs`). Only the GPU painter paints them, after the shading and the sheen and before the fine detail.
    pub lips: Lips,
    /// The eyebrows (`brows.rs`). Only the GPU painter paints them, on top of every other layer.
    pub brows: crate::brows::Brows,
    /// The light from maps (`maps.rs`): the gloss mask, the normal map and the detail tile's mask that the GPU painter writes beside the
    /// colour, and how much light each layer keeps in the colour while they are on.
    pub maps: crate::maps::LightMaps,
    /// The skin's colour under light and its colour zones (`skin.rs`). Only the GPU painter paints them, and at their neutral values they
    /// change nothing.
    pub skin: crate::skin::SkinColour,
}

/// Which layers of a recipe are painted. A layer that is off adds nothing to the texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layers {
    pub base_tone: bool,
    pub shading: bool,
    pub sheen: bool,
    pub wrinkles: bool,
    pub pores: bool,
    /// The eye paint. Only the GPU painter paints it.
    pub eyes: bool,
    /// The lips, which only the GPU painter paints.
    pub lips: bool,
    /// The eyebrows, which only the GPU painter paints.
    pub brows: bool,
    /// The facial hair, which only the GPU painter paints.
    pub facial_hair: bool,
    /// The light from maps. Only the GPU painter writes the maps. It is off by default, and while it is off the colour is what it was
    /// before the maps existed.
    pub maps: bool,
}

impl Default for Layers {
    fn default() -> Self {
        Layers { base_tone: true, shading: true, sheen: true, wrinkles: true, pores: true, eyes: true, lips: true, brows: true, facial_hair: true, maps: false }
    }
}

/// What the fine detail layers need of a whole head: its wrinkle lines, its pore landmarks and what the folds are placed from.
pub struct HeadDetail {
    pub lines: HeadLines,
    pub landmarks: PoreLandmarks,
    /// `None` on a mesh that does not hold the folds' landmarks.
    pub folds: Option<FoldHead>,
    /// `None` on a mesh that does not hold the forehead lines' landmarks.
    pub forehead: Option<ForeheadHead>,
    /// The lips' curves and front map. `None` on a mesh that does not hold the lip anchors.
    pub lips: Option<LipHead>,
    /// The brows' landmarks and the skin near them. `None` on a mesh that does not hold the brow anchors.
    pub brows: Option<crate::brows::BrowHead>,
    /// The facial hair's grid and regions. `None` on a mesh that does not hold their landmarks.
    pub beard: Option<crate::facial_hair::BeardHead>,
}

impl HeadDetail {
    /// Places the wrinkle lines and the pore landmarks on a head. It is `None` if a landmark falls outside the layout.
    pub fn new(uv: &[[f64; 2]], layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize], positions: &[[f64; 3]]) -> Option<Self> {
        let locator = LayoutLocator::new(uv, layout_triangles);
        let per_layout: Vec<[f64; 3]> = layout_to_mesh.iter().map(|&m| positions[m]).collect();
        let lips = LipHead::new(positions, layout_triangles, layout_to_mesh);
        let beard = crate::facial_hair::BeardHead::new(positions, uv, layout_triangles, layout_to_mesh, lips.as_ref());
        Some(HeadDetail { lines: HeadLines::new(&locator, &per_layout), landmarks: PoreLandmarks::new(&locator, &per_layout)?, folds: FoldHead::new(layout_triangles, layout_to_mesh, positions), forehead: ForeheadHead::new(positions), lips, brows: crate::brows::BrowHead::new(positions, layout_triangles, layout_to_mesh), beard })
    }
}

/// An encoded sRGB channel from 0 to 1 as a byte.
fn byte(encoded: f64) -> u8 {
    (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// A linear sRGB colour as encoded bytes.
pub fn srgb8(linear: [f64; 3]) -> [u8; 3] {
    linear.map(|c| byte(encode(c)))
}

impl Recipe {
    /// The colour under every other layer, in linear sRGB: the base tone when it is on, and the neutral grey when it is off.
    pub fn base_colour(&self, layers: &Layers, table: &PigmentTable) -> [f64; 3] {
        if layers.base_tone { self.base_tone.linear_rgb(table) } else { NEUTRAL.map(|c| decode(f64::from(c) / 255.0)) }
    }
}
