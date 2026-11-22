//! The layers of the paint, each split in two: a Rust half that runs on the processor and a WGSL half that runs per pixel.
//!
//! Each layer owns one file here and one shader in `shaders/`. Its Rust half implements `GpuLayer`: before every repaint it gives the
//! words of its uniform buffer, and, when they changed, the words of its storage buffer. Its shader declares those two buffers at its
//! own bindings and defines the one function the paint pass calls. A layer reads nothing of another layer.
//!
//! | Layer    | Rust          | Shader          | Bindings | Function the paint pass calls                     |
//! |----------|---------------|-----------------|----------|---------------------------------------------------|
//! | shading  | `shading.rs`  | `shading.wgsl`  | 1 and 2  | `shading_apply(c: vec3<f32>, pt: Point) -> vec3<f32>` |
//! | sheen    | `sheen.rs`    | `sheen.wgsl`    | 3 and 4  | `sheen_apply(c: vec3<f32>, pt: Point) -> vec3<f32>`   |
//! | wrinkles | `wrinkles.rs` | `wrinkles.wgsl` | 5 and 6  | `wrinkle_shift(pt: Point) -> f32`                     |
//! | pores    | `pores.rs`    | `pores.wgsl`    | 7 and 8  | `pore_shift(pt: Point) -> f32`                        |
//! | lines    | `lines.rs`    | `lines.wgsl`    | 10 and 11 | `lines_shift(pt: Point) -> f32`                      |
//! | eyes     | `eyes.rs`     | `eyes.wgsl`     | 12 and 13 | `eyes_apply(c: vec3<f32>, pt: Point) -> vec3<f32>`   |
//! | lips     | `lips.rs`     | `lips.wgsl`     | 14 and 15 | `lips_apply(c: vec3<f32>, pt: Point) -> vec3<f32>`   |
//! | brows    | `brows.rs`    | `brows.wgsl`    | 16 and 17 | `brows_apply(c: vec3<f32>, pt: Point) -> vec3<f32>`  |
//! | maps     | `maps.rs`     | `maps.wgsl`     | 18        | none: the other layers and the paint pass read it      |
//! | skin     | `skin.rs`     | `skin.wgsl`     | 20        | `skin_base(pt: Point) -> vec3<f32>`; the shading, the sheen and the paint pass call `skin_shift` |
//! | facial hair | `facial_hair.rs` | `facial_hair.wgsl`, with `hair_grid.wgsl` | 23 | `facial_hair_apply(c: vec3<f32>, pt: Point) -> vec3<f32>` |
//!
//! The forehead lines (`head_paint::lines`) are a layer of their own here, though they share the wrinkles' switch. The paint pass adds their change of lightness to the wrinkles' while the wrinkle layer is on.
//!
//! To add a layer: write its two files, add its module below and its `GpuLayer` to `all`, add its shader to `PAINT_SHADER` before
//! `paint.wgsl`, give it the next two bindings in `GpuPainter::new` (the bind group layout takes them in the order of `all`, skipping 9),
//! and call its function from `fs_main` in `paint.wgsl`.
//!
//! The paint pass calls a layer's function only while the layer is on, and the base tone is the colour `Frame.base`, which the processor
//! computes. Binding 0 is the frame's settings, in `common.wgsl`, with the shared helpers every layer may call.
//! The eye layer is the exception: the paint pass always calls `eyes_apply`, last, and the layer's own uniform says whether it is on,
//! because `Frame.on` has no fifth slot. The brow layer does the same, and is called after it, so the hairs lie on top of everything.
//! The maps layer is the second exception. It paints nothing by itself: while its switch is on, the other layers read its sliders, paint
//! less of their own light, and leave their height, gloss and tile mask in its private numbers, which the paint pass writes to a second
//! texture. While it is off, every layer paints exactly what it painted before the maps existed.

pub mod brows;
pub mod eyes;
pub mod facial_hair;
pub mod lines;
pub mod lips;
pub mod maps;
pub mod pores;
pub mod shading;
pub mod sheen;
pub mod skin;
pub mod wrinkles;

use crate::head::PreparedHead;
use head_paint::paint::{Layers, Recipe};

/// What a layer reads before a repaint.
pub struct LayerInput<'a> {
    pub recipe: &'a Recipe,
    pub layers: &'a Layers,
    pub head: &'a PreparedHead,
    /// True on the first repaint after a new head was set, so that the layer rebuilds what it keeps of the head.
    pub new_head: bool,
    /// Values written straight onto the facial hair's grid, as the grid editor will write them, laid over the sliders' preset.
    pub hair_edit: Option<&'a head_paint::hair_grid::GridEdit>,
    /// The facial hair's hair groups, written over the sliders' preset before the grid edit.
    pub hair_groups: &'a head_paint::hair_groups::HairGroups,
}

/// A layer's buffers for one repaint, as 32-bit words (an `f32` goes in with `f32::to_bits`).
pub struct LayerWords {
    /// The uniform buffer. Its length must be a multiple of 4 words and must match the layer's WGSL struct.
    pub uniform: Vec<u32>,
    /// The storage buffer, or `None` when the words of the last repaint still hold.
    pub storage: Option<Vec<u32>>,
}

/// The processor's half of a layer.
pub trait GpuLayer {
    /// The words of the layer's buffers for this repaint. It is called before every repaint, even while the layer is off.
    fn words(&mut self, input: &LayerInput) -> LayerWords;

    /// Whether this layer paints the control with this id (an id of `head_paint::controls`). A control that no layer paints is shown
    /// on the page as one the GPU painter ignores.
    fn paints(&self, _id: &str) -> bool {
        false
    }

    /// Whether the layer has a storage buffer. A fragment shader may bind at most 8, so a layer that needs none binds none, and its
    /// second binding stays free.
    fn has_storage(&self) -> bool {
        true
    }

    /// Whether the layer has a uniform buffer. A fragment shader may bind at most 12, and the frame, the landmarks and the first ten
    /// layers use them all, so a later layer keeps its settings in its storage buffer instead.
    fn has_uniform(&self) -> bool {
        true
    }
}

/// The controls of the base tone, which the processor turns into `Frame.base`.
pub const BASE_TONE_CONTROLS: [&str; 5] = ["melanin", "haemoglobin", "chroma_scale", "undertone_degrees", "unnatural_degrees"];

/// The ids of the controls that no layer of the GPU painter paints yet.
pub fn unpainted_controls() -> Vec<String> {
    let layers = all();
    head_paint::controls::controls().into_iter().map(|c| c.id).filter(|id| !BASE_TONE_CONTROLS.contains(&id.as_str()) && !layers.iter().any(|l| l.paints(id))).collect()
}

/// The shader sources in the order they are joined into one module: the shared helpers, then each layer, then the paint pass.
pub const PAINT_SHADER: [&str; 14] = [
    include_str!("../../shaders/common.wgsl"),
    include_str!("../../shaders/maps.wgsl"),
    include_str!("../../shaders/shading.wgsl"),
    include_str!("../../shaders/sheen.wgsl"),
    include_str!("../../shaders/wrinkles.wgsl"),
    include_str!("../../shaders/pores.wgsl"),
    include_str!("../../shaders/lines.wgsl"),
    include_str!("../../shaders/eyes.wgsl"),
    include_str!("../../shaders/lips.wgsl"),
    include_str!("../../shaders/brows.wgsl"),
    include_str!("../../shaders/skin.wgsl"),
    include_str!("../../shaders/hair_grid.wgsl"),
    include_str!("../../shaders/facial_hair.wgsl"),
    include_str!("../../shaders/paint.wgsl"),
];

/// Every layer, in the order of their bindings: layer k has bindings 1 + 2k and 2 + 2k, except that binding 9 is the shared landmarks, so
/// the fifth layer takes 10 and 11 and the sixth 12 and 13.
pub fn all() -> Vec<Box<dyn GpuLayer>> {
    vec![Box::new(shading::ShadingLayer::default()), Box::new(sheen::SheenLayer::default()), Box::new(wrinkles::WrinkleLayer::default()), Box::new(pores::PoreLayer::default()), Box::new(lines::LineLayer::default()), Box::new(eyes::EyeLayer::default()), Box::new(lips::LipLayer::default()), Box::new(brows::BrowLayer::default()), Box::new(maps::MapsLayer), Box::new(skin::SkinLayer::default()), Box::new(facial_hair::FacialHairLayer::default())]
}
