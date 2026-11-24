//! The paint's controls as a flat list, so that a page or an editor can build one slider per control without knowing the recipe
//! (paint track, part C, decision 41).
//!
//! Every number of a `Recipe` is one control. A control has an id, the label a person reads, the layer it belongs to, a range, a step and
//! a default. `values_of` turns a recipe into one number per control, in the order of `controls`, and `recipe_from` turns such a list
//! back into a recipe. Each layer is also a group with an on and off switch, in the order of `GROUPS`, and `layers_from` turns the
//! switches into `Layers`.
//!
//! A new layer's sliders need four changes here, and none in a page:
//! 1. a `Group` in `GROUPS`, and its switch in `layer_switch`;
//! 2. a `Field` for each of its numbers;
//! 3. a row for each field in `controls`;
//! 4. an arm for each field in `get` and `set`.
//!
//! The live painter (`live.rs`) must also learn the new layer, and its test checks that it paints what `paint` paints.

use crate::eyes::EyePaint;
use crate::lines::{ForeheadLines, SLIDERS as LINE_SLIDERS};
use crate::paint::{Layers, Recipe};
use crate::pigment::{BaseTone, HAEMOGLOBIN_MAX, MELANIN_MAX};
use crate::pores::Pores;
use crate::shading::{Shading, TYPICAL_HIGHLIGHT_WHITENESS};
use crate::sheen::Sheen;
use crate::wrinkles::{SETS, Wrinkles};
use serde::Serialize;

/// One paint layer, as a group of controls with its own on and off switch.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Group {
    pub id: &'static str,
    pub label: &'static str,
    /// What the layer paints, in one or two sentences for a person.
    pub note: &'static str,
}

/// The layers in the order in which they are painted.
pub const GROUPS: [Group; 10] = [
    Group { id: "base_tone", label: "Base tone", note: "The skin colour, made from two pigments and turned by three colour controls. When this layer is off, the other layers are painted on a neutral grey skin." },
    Group { id: "shading", label: "Shading", note: "Light from a sky above the head that the face's own shape blocks, and a little more light on the points that stand out. KOTOR paints this light into its textures." },
    Group { id: "sheen", label: "Sheen", note: "A whitish highlight on the cheekbones, along the nose and on the chin." },
    Group { id: "wrinkles", label: "Wrinkles and folds", note: "Nine sets of creases placed on the head, such as crow's feet, nasolabial folds and forehead lines, and a net of fine lines over the skin. Each crease has a dark furrow and lighter ridges beside it. The forehead lines have their own ten sliders. While their amount is above zero, they replace the forehead set." },
    Group { id: "pores", label: "Pores and grain", note: "Pores, and a net of fine furrows that join them, as on real skin seen up close. The nose has the most and largest pores, and the temples and the skin around the eyes the fewest. A faint mottling of broad blotches lies under them." },
    Group { id: "eyes", label: "Eye paint", note: "The paint round the eyes: the lash lines, a liner with a wing, the crease above the upper lid, the darker skin of the socket and under the eye, red lid rims, pale or dark lids, bags under the eyes and eye shadow. Everything follows the edges of the lids. The group also holds the eyeball: the iris, the pupil and the eye white, which are painted whether this layer is on or off. Only the GPU painter paints this layer." },
    Group { id: "lips", label: "Lips", note: "The lips' colour, natural or lipstick, their border with the skin, the dark line where they meet, and the light and shade on them. It follows the lip border of the head it is painted on. Only the GPU painter paints this layer." },
    Group { id: "brows", label: "Eyebrows", note: "The eyebrows, drawn as single hairs that follow a growth direction, over a soft fill and a faint shadow on the skin. The brow's line and thickness are set from the brow ridge of the head it is painted on, so the brows follow the head's shape. Only the GPU painter paints this layer." },
    Group { id: "maps", label: "Light from maps", note: "Lets the light of the viewer, not the paint, make the skin's shine and its fine relief. The painter writes a gloss mask and a normal map beside the colour, and the viewer repeats a small tile of pores and furrows over the face. Each layer then paints less of its own light and shade into the colour. When this layer is off, the colour is exactly the one painted without maps. Only the GPU painter paints this layer." },
    Group { id: "facial_hair", label: "Facial hair", note: "Facial hair drawn as single short hairs over a faint darker shadow on the skin. A head's facial hair is its list of hair groups, each with its own sliders. A stubble covers the beard's regions, each with its own density: the moustache, the soul patch, the chin, the jaw line, the cheeks, the sideburns and the neck. A stroke and a filled area are shaped with control points on the head. The hairs are drawn after the lips, so a moustache can cover the lip's edge. Only the GPU painter paints this layer." },
];

fn layer_switch(layers: &mut Layers, group: usize) -> &mut bool {
    match group {
        0 => &mut layers.base_tone,
        1 => &mut layers.shading,
        2 => &mut layers.sheen,
        3 => &mut layers.wrinkles,
        4 => &mut layers.pores,
        6 => &mut layers.lips,
        7 => &mut layers.brows,
        8 => &mut layers.maps,
        9 => &mut layers.facial_hair,
        _ => &mut layers.eyes,
    }
}

/// The layers that are on, from one switch per group in the order of `GROUPS`. A missing switch is on.
pub fn layers_from(switches: &[bool]) -> Layers {
    let mut layers = Layers::default();
    for (g, &on) in switches.iter().enumerate().take(GROUPS.len()) {
        *layer_switch(&mut layers, g) = on;
    }
    layers
}

/// The switches of some layers, one per group in the order of `GROUPS`.
pub fn switches_of(layers: &Layers) -> Vec<bool> {
    let mut l = *layers;
    (0..GROUPS.len()).map(|g| *layer_switch(&mut l, g)).collect()
}

/// Which number of the recipe a control moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Melanin,
    Haemoglobin,
    ChromaScale,
    Undertone,
    Unnatural,
    ShadingStrength,
    HighlightWhiteness,
    SheenCheeks,
    SheenNose,
    SheenChin,
    Age,
    WrinkleDepth,
    WrinkleWidth,
    Wobble,
    FineLines,
    WrinkleSeed,
    SetStrength(usize),
    SetWidth(usize),
    SetLength(usize),
    SetCount(usize),
    SetShift(usize),
    LinesAmount,
    LinesSpacing,
    LinesContinuity,
    LinesWaviness,
    LinesVariation,
    LinesDepth,
    LinesSharpness,
    LinesRidge,
    LinesDip,
    LinesReach,
    PoreDensity,
    PoreGradient,
    PoreSize,
    PoreDepth,
    Grain,
    GrainSize,
    Mottling,
    PoreSeed,
    GrainDirection,
    PoreStretch,
    Glint,
    GlintMm,
    GlintPoints,
    GlintVariation,
    GlintFollow,
    PoreCrater,
    PoreScale,
    PoreGleam,
    EyeUpperLash,
    EyeUpperLashMm,
    EyeLowerLash,
    EyeLowerLashMm,
    EyeLiner,
    EyeLinerMm,
    EyeWingMm,
    EyeWingDegrees,
    EyeLinerLower,
    EyeLinerColour(usize),
    EyeCrease,
    EyeCreaseMm,
    EyeSocket,
    EyeUnderEye,
    EyeRims,
    EyeLidRed,
    EyeShadow,
    EyeShadowMm,
    EyeShadowColour(usize),
    EyeLidLight,
    EyeBags,
    EyeBagsMm,
    /// A lip slider, by its place in `lips::SLIDERS`.
    Lip(usize),
    /// An eyeball slider, by its place in `eyeball::SLIDERS`.
    Ball(usize),
    /// A brow slider, by its place in `brows::SLIDERS`.
    Brow(usize),
    /// A slider of the light from maps, by its place in `maps::SLIDERS`.
    Map(usize),
    /// A slider of the skin's light and colour zones, by its place in `skin::SLIDERS`.
    Skin(usize),
}

/// One control of the paint.
#[derive(Clone, Debug, Serialize)]
pub struct Control {
    pub id: String,
    pub label: String,
    /// The id of the control's group in `GROUPS`.
    pub group: &'static str,
    /// A smaller heading inside the group, such as the name of a wrinkle set. Empty when the control has none.
    pub part: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub default: f64,
    /// The unit shown after the value, such as "mm" or "degrees". Empty for a plain number.
    pub unit: &'static str,
    /// What the control does, in one or two sentences for a person.
    pub help: &'static str,
    /// True for a control that only the GPU painter paints.
    pub gpu_only: bool,
    #[serde(skip)]
    pub field: Field,
}

/// The pore layer's defaults, set by eye on 2026-11-20 for the GPU's furrow net. The old defaults were the median of a fit on the noise of
/// the 512-pixel textures, with pores lighter than the skin and none on the cheeks. The light grains' defaults were set by eye on the
/// afternoon of 2026-11-20, between the values set by hand on Male B1, Female B4, Male A4 and Canderous of the mod.
pub const DEFAULT_PORES: Pores = Pores { density: 100.0, t_zone: 1.0, size_mm: 0.24, depth: 9.0, grain: 4.0, grain_mm: 0.4, mottling: 2.5, seed: 5, gradient: 1.0, grain_direction: 0.8, stretch: 0.5, glint: 2.5, glint_mm: 0.45, glint_points: 1.2, glint_variation: 0.5, glint_follow: 1.0, crater: 0.0, crater_scale: 1.0, gleam: 0.0 };

/// The recipe whose values are the controls' defaults. Its base tone and shading strength are close to the median of the 57 fitted KOTOR
/// heads of 2026-11-19, its pores are `DEFAULT_PORES`, and its wrinkles are every set at its own default for a 50-year-old, and it has no forehead lines.
pub fn default_recipe() -> Recipe {
    Recipe {
        base_tone: BaseTone { melanin: 0.06, haemoglobin: 0.05, chroma_scale: 0.9, undertone_degrees: -5.0, unnatural_degrees: 0.0 },
        shading: Shading { strength: 0.85, highlight_whiteness: TYPICAL_HIGHLIGHT_WHITENESS },
        sheen: Sheen::NONE,
        wrinkles: Wrinkles::at_age(50.0),
        forehead: ForeheadLines::NONE, lips: crate::lips::Lips::DEFAULT, brows: crate::brows::Brows::DEFAULT,
        pores: DEFAULT_PORES,
        eyes: EyePaint::DEFAULT,
        maps: crate::maps::LightMaps::DEFAULT,
        skin: crate::skin::SkinColour::NONE,
    }
}

#[allow(clippy::too_many_arguments)]
fn row(field: Field, id: String, label: String, group: &'static str, part: String, min: f64, max: f64, step: f64, unit: &'static str, help: &'static str) -> Control {
    let default = get(&default_recipe(), field).clamp(min, max);
    Control { id, label, group, part, min, max, step, default, unit, help, gpu_only: false, field }
}

/// Every control of the paint, grouped by layer in the order of `GROUPS`. Each range holds the values of all 57 fitted KOTOR heads.
pub fn controls() -> Vec<Control> {
    use Field::*;
    let plain = |field: Field, id: &str, label: &str, group: &'static str, range: (f64, f64, f64), unit: &'static str, help: &'static str| {
        row(field, id.to_string(), label.to_string(), group, String::new(), range.0, range.1, range.2, unit, help)
    };
    let gpu = |field: Field, id: &str, label: &str, group: &'static str, range: (f64, f64, f64), unit: &'static str, help: &'static str| Control { gpu_only: true, ..plain(field, id, label, group, range, unit, help) };
    let mut list = vec![
        plain(Melanin, "melanin", "Melanin", "base_tone", (0.0, MELANIN_MAX, 0.001), "", "The amount of brown pigment in the skin's top layer. More melanin makes the skin darker and browner."),
        plain(Haemoglobin, "haemoglobin", "Blood", "base_tone", (0.0, HAEMOGLOBIN_MAX, 0.0005), "", "The amount of blood in the skin's lower layer. More blood makes the skin redder."),
        plain(ChromaScale, "chroma_scale", "Colour strength", "base_tone", (0.0, 2.0, 0.01), "", "How strong the skin's colour is. At 1 the colour is the pigments' own. At 0 the skin is grey."),
        plain(Undertone, "undertone_degrees", "Undertone", "base_tone", (-45.0, 45.0, 0.5), "degrees", "Turns the skin's hue a little. A negative value turns the skin redder, and a positive value turns it more yellow."),
        plain(Unnatural, "unnatural_degrees", "Unnatural colour", "base_tone", (-180.0, 180.0, 1.0), "degrees", "Turns the skin's hue around the whole colour wheel, for colours real skin never has. At +80 degrees the skin is grey-green, and at -60 degrees it is purple."),
        plain(ShadingStrength, "shading_strength", "Strength", "shading", (0.0, 3.0, 0.01), "", "How strongly the painted light and shade show. At 1 a head gets the shared pattern of the KOTOR heads, and at 0 it gets none."),
        plain(HighlightWhiteness, "highlight_whiteness", "Highlight whiteness", "shading", (0.0, 1.0, 0.01), "", "How much of the lightening goes toward white instead of a brighter skin colour. At 0 the light keeps the skin's colour."),
        plain(SheenCheeks, "sheen_cheeks", "Cheekbones", "sheen", (0.0, 40.0, 0.1), "L*", "How much lighter the skin is along each cheekbone, at the middle of the highlight."),
        plain(SheenNose, "sheen_nose", "Nose", "sheen", (0.0, 40.0, 0.1), "L*", "How much lighter the skin is along the ridge of the nose, at the middle of the highlight."),
        plain(SheenChin, "sheen_chin", "Chin", "sheen", (0.0, 40.0, 0.1), "L*", "How much lighter the skin is on the front of the chin, at the middle of the highlight."),
        plain(Age, "wrinkle_age", "Age", "wrinkles", (15.0, 90.0, 0.5), "years", "The age sets which crease sets show and how deep they are. Crow's feet start at 25 years, and cheek folds start last, at 45 years."),
        plain(WrinkleDepth, "wrinkle_depth", "Depth of every set", "wrinkles", (0.0, 3.0, 0.01), "", "Multiplies the depth of every crease set. At 0 no set is painted."),
        plain(WrinkleWidth, "wrinkle_width", "Width of every set", "wrinkles", (0.5, 3.0, 0.01), "", "Multiplies the width of every crease's furrow."),
        plain(Wobble, "wrinkle_wobble", "Wobble", "wrinkles", (0.0, 1.5, 0.01), "mm", "How far a crease wanders sideways along its length, so that it does not look drawn with a ruler."),
        plain(FineLines, "fine_lines", "Fine lines", "wrinkles", (0.0, 10.0, 0.05), "L*", "How much darker the furrows of the net of fine lines are. The fine lines are short thin creases over the whole skin, as on old skin."),
        plain(WrinkleSeed, "wrinkle_seed", "Random pattern", "wrinkles", (0.0, 20.0, 1.0), "", "Picks another random wobble, another strength for each crease and another net of fine lines."),
    ];
    for (s, set) in SETS.iter().enumerate() {
        let lines = set.lines.len() as f64;
        let part = |field: Field, key: &str, label: &str, range: (f64, f64, f64), unit: &'static str, help: &'static str| {
            row(field, format!("{}_{key}", set.id), label.to_string(), "wrinkles", set.label.to_string(), range.0, range.1, range.2, unit, help)
        };
        list.push(part(SetStrength(s), "strength", "Strength", (0.0, 3.0, 0.01), "", "How deep this set's creases are, against the set's usual depth at this age. At 1 the set has its usual depth."));
        list.push(part(SetWidth(s), "width", "Width", (0.5, 2.0, 0.01), "", "How wide this set's furrows are, against the set's usual width."));
        list.push(part(SetLength(s), "length", "Length", (0.3, 1.0, 0.01), "", "The share of each crease that is drawn, starting from its anchor, such as the corner of the eye for crow's feet."));
        list.push(part(SetCount(s), "count", "Number of lines", (0.0, lines, 0.05), "", "How many of this set's creases are drawn, the most important first. A fraction draws the last crease fainter."));
        list.push(part(SetShift(s), "shift", "Sideways shift", (-4.0, 4.0, 0.1), "mm", "Moves this set's creases sideways along the skin. The creases on the other side of the face move the mirrored way."));
    }
    // The forehead lines' ranges and help sentences are those of `lines::SLIDERS`, which are in the same order as these fields.
    let line_fields = [LinesAmount, LinesSpacing, LinesContinuity, LinesWaviness, LinesVariation, LinesDepth, LinesSharpness, LinesRidge, LinesDip, LinesReach];
    let line_rows: [(&str, &str, f64, &'static str); 10] = [
        ("amount", "Amount", 0.01, ""),
        ("spacing", "Spacing", 0.01, ""),
        ("continuity", "Broken lines", 0.01, ""),
        ("waviness", "Waviness", 0.01, ""),
        ("variation", "Other arrangement", 1.0, ""),
        ("depth", "Depth", 0.1, "L*"),
        ("sharpness", "Sharpness", 0.01, ""),
        ("ridge", "Light under the lines", 0.01, ""),
        ("dip_mm", "Dip between the brows", 0.1, "mm"),
        ("reach", "Spread up the forehead", 0.01, ""),
    ];
    for ((field, (key, label, step, unit)), slider) in line_fields.into_iter().zip(line_rows).zip(LINE_SLIDERS.iter()) {
        assert_eq!(slider.id, key, "the forehead lines' sliders are in another order");
        list.push(row(field, format!("lines_{key}"), label.to_string(), "wrinkles", "Forehead lines".to_string(), slider.min, slider.max, step, unit, slider.help));
    }
    list.extend([
        plain(PoreDensity, "pore_density", "Pores per cm²", "pores", (0.0, 150.0, 1.0), "", "How many pores show on each square centimetre of skin on the nose. The rest of the face has fewer, as \"Pores away from the nose\" sets. Real faces have 10 to 90 visible pores per cm² on the cheek."),
        gpu(PoreGradient, "pore_gradient", "Pores away from the nose", "pores", (0.0, 2.0, 0.01), "", "How much fewer and smaller the pores are away from the nose. At 0 the whole face has the nose's pores. At 1 the face follows the measured map: the cheek beside the nose has 80% of the nose's pores, the middle of the forehead 70%, the middle of the cheek 50%, and the skin around the eyes 10%. At 2 the fall is twice as steep."),
        plain(PoreSize, "pore_size", "Pore size", "pores", (0.1, 0.6, 0.01), "mm", "The mean radius of a pore on the nose. Pores elsewhere are smaller, down to 40% of this size at the temples. Real visible pores are 0.10 to 0.26 mm in radius."),
        plain(PoreDepth, "pore_depth", "Pore darkness", "pores", (-30.0, 30.0, 0.1), "L*", "How much darker the middle of a pore is than the skin around it. A negative value makes the pores lighter than the skin."),
        gpu(PoreStretch, "pore_stretch", "Pore stretch", "pores", (0.0, 1.0, 0.01), "", "How long the pores are along the skin's grain. At 0 they are round. At 1 a pore is 2.5 times as long as it is wide. Real cheek pores are 1.7 to 2.4 times as long as they are wide."),
        plain(Grain, "grain", "Grain", "pores", (0.0, 15.0, 0.05), "L*", "How much darker the fine furrows of the skin are. The furrows join the pores into a net of small fields."),
        plain(GrainSize, "grain_size", "Grain size", "pores", (0.3, 1.0, 0.01), "mm", "How far apart the corners of the furrow net are. Real cheek skin has furrows 0.25 to 0.4 mm apart, which is too fine to draw at this texture's size."),
        gpu(GrainDirection, "grain_direction", "Grain direction", "pores", (0.0, 1.0, 0.01), "", "How much the furrows run along one direction: across the forehead, out from the eyes and down the cheeks. At 0 they run every way. Older skin has a stronger direction."),
        gpu(Glint, "grain_light", "Light grains", "pores", (0.0, 25.0, 0.1), "L*", "How bright the small grains of skin between the furrows are, as lit bumps catch the light. The number is how much a grain's top lightens skin of middle lightness. The grains add light, so they show more on dark skin than on pale skin, as on KOTOR's high-detail heads."),
        gpu(GlintMm, "grain_light_size", "Light grain size", "pores", (0.25, 1.2, 0.01), "mm", "How far apart the light grains are. KOTOR's high-detail heads show grains about 0.3 to 0.5 mm across, packed side by side."),
        gpu(GlintPoints, "grain_dark_points", "Dark points between grains", "pores", (0.0, 20.0, 0.1), "L*", "How much darker the small points where three grains meet are, on skin of middle lightness. They darken by a share of the light, so they show more on pale skin."),
        gpu(GlintVariation, "grain_light_variation", "Grain to grain variation", "pores", (0.0, 1.0, 0.01), "", "How much the grains differ in brightness. At 0 every grain is as bright as the others. At 1 some grains are barely there."),
        gpu(GlintFollow, "grain_light_follow", "Grains follow the light", "pores", (0.0, 1.0, 0.01), "", "How much the grains and their dark points follow the light painted by the shading and the sheen. At 0 they are as strong in the shade as in the light. At 1 they are gone where the paint is 5 L* darker than the skin, and grow up to 1.6 times as strong where it is 3 L* lighter or more."),
        gpu(PoreCrater, "pore_crater", "Pore craters", "pores", (0.0, 1.0, 0.01), "", "How much each pore looks like a small dent lit from above. The edge of the pore nearest the light falls into shadow, and the far edge catches a thin rim of light. At 0 a pore is a dark dot with a soft edge, the same on every side."),
        gpu(PoreScale, "pore_scale", "Stylised pore size", "pores", (1.0, 3.0, 0.05), "×", "How many times larger than their real size the pores are drawn, so that each pore covers enough pixels to show its shaded and lit sides. The number of pores per cm² is divided by the square of this number, so the pores cover the same share of the skin and it does not look pitted. At 1 the pores have their real size."),
        gpu(PoreGleam, "pore_gleam", "Pore gleam", "pores", (0.0, 1.0, 0.01), "", "How much each pore crater gleams, as oily skin does. A small, sharp, nearly white glint sits on the lit lower rim of each crater. It is strongest on the oily skin of the forehead's middle, the nose, the inner cheeks and the chin, and it follows the painted light as the light grains do. The skin around gives up as much light as the glints add, so the skin's tone stays the same at a distance. It shows only while \"Pore craters\" is above 0."),
        plain(Mottling, "mottling", "Mottling", "pores", (0.0, 20.0, 0.05), "L*", "How strong the broad blotches of lighter and darker skin are. They are about 10 mm across on the GPU and 8 mm on the processor."),
        plain(PoreSeed, "pore_seed", "Random pattern", "pores", (0.0, 20.0, 1.0), "", "Picks another random scatter of pores, furrows and mottling."),
    ]);
    let eye = |field: Field, id: &str, label: &str, part: &str, range: (f64, f64, f64), unit: &'static str, help: &'static str| {
        Control { gpu_only: true, ..row(field, format!("eye_{id}"), label.to_string(), "eyes", part.to_string(), range.0, range.1, range.2, unit, help) }
    };
    list.extend([
        eye(EyeUpperLash, "upper_lash", "Darkness", "Upper lash line", (0.0, 1.0, 0.01), "", "How dark the band of upper lashes along the edge of the upper lid is. At 1 the band is almost black."),
        eye(EyeUpperLashMm, "upper_lash_mm", "Width", "Upper lash line", (0.2, 3.0, 0.05), "mm", "How far the band of upper lashes reaches above the lid's edge, over the outer half of the eye. It is a third as wide at the inner corner."),
        eye(EyeLowerLash, "lower_lash", "Darkness", "Lower lash line", (0.0, 1.0, 0.01), "", "How dark the band of lower lashes along the edge of the lower lid is. It is fainter toward the inner corner."),
        eye(EyeLowerLashMm, "lower_lash_mm", "Width", "Lower lash line", (0.2, 2.5, 0.05), "mm", "How far the band of lower lashes reaches below the lid's edge, near the outer corner."),
        eye(EyeLiner, "liner", "Amount", "Liner", (0.0, 1.0, 0.01), "", "How opaque the liner is. The liner is makeup painted along the upper lid's edge, in its own colour."),
        eye(EyeLinerMm, "liner_mm", "Width", "Liner", (0.3, 5.0, 0.05), "mm", "How far the liner reaches above the upper lid's edge at the outer corner. It is a third as wide at the inner corner."),
        eye(EyeWingMm, "wing_mm", "Wing length", "Liner", (0.0, 20.0, 0.1), "mm", "How far the liner's wing reaches past the outer corner of the eye. At 0 the liner has no wing."),
        eye(EyeWingDegrees, "wing_degrees", "Wing angle", "Liner", (-20.0, 60.0, 0.5), "degrees", "The wing's angle above the line from the inner to the outer corner of the eye. A higher angle points the wing toward the end of the brow."),
        eye(EyeLinerLower, "liner_lower", "Under the eye", "Liner", (0.0, 1.0, 0.01), "", "How much of the liner also lines the lower lid. At 1 the lower line is as opaque as the upper one and 60% as wide."),
        eye(EyeLinerColour(0), "liner_lightness", "Lightness", "Liner", (0.0, 100.0, 0.5), "L*", "How light the liner's colour is. Black liner is about 10."),
        eye(EyeLinerColour(1), "liner_hue", "Hue", "Liner", (0.0, 360.0, 1.0), "degrees", "The liner's hue around the colour wheel. Red is near 30 degrees, yellow near 90, green near 160, blue near 260 and purple near 320."),
        eye(EyeLinerColour(2), "liner_chroma", "Colour strength", "Liner", (0.0, 80.0, 0.5), "", "How strong the liner's colour is. At 0 the liner is grey or black."),
        eye(EyeCrease, "crease", "Darkness", "Crease and socket", (0.0, 25.0, 0.1), "L*", "How much darker the crease above the upper lid is than the skin around it, at its middle."),
        eye(EyeCreaseMm, "crease_mm", "Crease height", "Crease and socket", (2.0, 14.0, 0.1), "mm", "How far above the upper lid's edge the crease runs. Real creases lie 4 to 9 mm above the lashes."),
        eye(EyeSocket, "socket", "Socket darkness", "Crease and socket", (0.0, 30.0, 0.1), "L*", "How much darker the skin of the socket is right above the upper lid. It fades out a little above the crease. The darker skin is made with more melanin, so it keeps the skin's own colour."),
        eye(EyeUnderEye, "under_eye", "Under the eye", "Crease and socket", (0.0, 30.0, 0.1), "L*", "How much darker the skin under the lower lid is, at its darkest, about 3 mm below the lid. The inner half is darker than the outer half."),
        eye(EyeRims, "rims", "Red rims", "Crease and socket", (0.0, 1.0, 0.01), "", "How red the rims of the lids are. The lower lid's rim gets most of it. The redness is made with more blood in the skin."),
        eye(EyeLidRed, "lid_red", "Red lids", "Crease and socket", (0.0, 1.0, 0.01), "", "How red the skin of the lids is: the upper lid up to the crease and the lower lid down to about 5 mm. Tired, old or sore eyes have redder lids."),
        eye(EyeShadow, "shadow", "Amount", "Eye shadow", (0.0, 1.0, 0.01), "", "How opaque the eye shadow is. Eye shadow is makeup on the upper lid, from the lashes up to its height."),
        eye(EyeShadowMm, "shadow_mm", "Height", "Eye shadow", (2.0, 22.0, 0.1), "mm", "How far above the upper lid's edge the eye shadow reaches, over the middle of the eye. It reaches a little higher at the outer end."),
        eye(EyeShadowColour(0), "shadow_lightness", "Lightness", "Eye shadow", (0.0, 100.0, 0.5), "L*", "How light the eye shadow's colour is."),
        eye(EyeShadowColour(1), "shadow_hue", "Hue", "Eye shadow", (0.0, 360.0, 1.0), "degrees", "The eye shadow's hue around the colour wheel. Red is near 30 degrees, yellow near 90, green near 160, blue near 260 and purple near 320."),
        eye(EyeShadowColour(2), "shadow_chroma", "Colour strength", "Eye shadow", (0.0, 80.0, 0.5), "", "How strong the eye shadow's colour is. At 0 the shadow is grey."),
        eye(EyeLidLight, "lid_light", "Lid lightness", "Crease and socket", (-15.0, 20.0, 0.1), "L*", "How much lighter the skin of the lids is than the face, in L*: the upper lid up to the crease and the lower lid down to about 4 mm. Old eyes often have pale, thin lid skin. A negative value makes the lids darker."),
        eye(EyeBags, "bags", "Bags", "Bags", (0.0, 25.0, 0.1), "L*", "How deep the soft crease under the bags below the eye is, in L*. The puffy skin above the crease is painted a little lighter. At 0 there are no bags."),
        eye(EyeBagsMm, "bags_mm", "Bag height", "Bags", (3.0, 14.0, 0.1), "mm", "How far below the lower lid's edge the crease under the bags runs, at the middle of the eye. It is a quarter closer at both ends."),
    ]);
    for (k, s) in crate::eyeball::SLIDERS.iter().enumerate() {
        list.push(Control { gpu_only: true, ..row(Ball(k), format!("eye_{}", s.id), s.label.to_string(), "eyes", crate::eyeball::part(k).to_string(), s.min, s.max, s.step, s.unit, s.help) });
    }
    for (k, s) in crate::lips::SLIDERS.iter().enumerate() {
        list.push(Control { gpu_only: true, ..row(Lip(k), format!("lips_{}", s.id), s.label.to_string(), "lips", String::new(), s.min, s.max, s.step, s.unit, s.help) });
    }
    for (k, s) in crate::brows::SLIDERS.iter().enumerate() {
        list.push(Control { gpu_only: true, ..row(Brow(k), format!("brow_{}", s.id), s.label.to_string(), "brows", s.part.to_string(), s.min, s.max, s.step, s.unit, s.help) });
    }
    // The maps' controls come last, so that a page that leaves them out still sends every other control at its place.
    for (k, (s, part)) in crate::maps::SLIDERS.iter().enumerate() {
        list.push(Control { gpu_only: true, ..row(Map(k), format!("maps_{}", s.id), s.label.to_string(), "maps", part.to_string(), s.min, s.max, s.step, s.unit, s.help) });
    }
    // The skin's light and colour zones come after the maps' for the same reason, though they sit in the groups of the shading and the
    // base tone. Their ids already start with "skin_".
    for (k, (s, group, part)) in crate::skin::SLIDERS.iter().enumerate() {
        list.push(Control { gpu_only: true, ..row(Skin(k), s.id.to_string(), s.label.to_string(), *group, part.to_string(), s.min, s.max, s.step, s.unit, s.help) });
    }
    list
}

fn seed(v: f64) -> u64 {
    v.round().max(0.0) as u64
}

/// The value of one field of a recipe.
pub fn get(r: &Recipe, field: Field) -> f64 {
    use Field::*;
    match field {
        Melanin => r.base_tone.melanin,
        Haemoglobin => r.base_tone.haemoglobin,
        ChromaScale => r.base_tone.chroma_scale,
        Undertone => r.base_tone.undertone_degrees,
        Unnatural => r.base_tone.unnatural_degrees,
        ShadingStrength => r.shading.strength,
        HighlightWhiteness => r.shading.highlight_whiteness,
        SheenCheeks => r.sheen.cheeks,
        SheenNose => r.sheen.nose,
        SheenChin => r.sheen.chin,
        Age => r.wrinkles.age,
        WrinkleDepth => r.wrinkles.depth,
        WrinkleWidth => r.wrinkles.width,
        Wobble => r.wrinkles.wobble_mm,
        FineLines => r.wrinkles.fine,
        WrinkleSeed => r.wrinkles.seed as f64,
        SetStrength(s) => r.wrinkles.sets[s].strength,
        SetWidth(s) => r.wrinkles.sets[s].width,
        SetLength(s) => r.wrinkles.sets[s].length,
        // A count above the set's number of lines draws every line, so it reads as that number.
        SetCount(s) => r.wrinkles.sets[s].count.min(SETS[s].lines.len() as f64),
        SetShift(s) => r.wrinkles.sets[s].shift_mm,
        LinesAmount => r.forehead.amount,
        LinesSpacing => r.forehead.spacing,
        LinesContinuity => r.forehead.continuity,
        LinesWaviness => r.forehead.waviness,
        LinesVariation => f64::from(r.forehead.variation),
        LinesDepth => r.forehead.depth,
        LinesSharpness => r.forehead.sharpness,
        LinesRidge => r.forehead.ridge,
        LinesDip => r.forehead.dip_mm,
        LinesReach => r.forehead.reach,
        PoreDensity => r.pores.density,
        PoreGradient => r.pores.gradient,
        GrainDirection => r.pores.grain_direction,
        PoreStretch => r.pores.stretch,
        Glint => r.pores.glint,
        GlintMm => r.pores.glint_mm,
        GlintPoints => r.pores.glint_points,
        GlintVariation => r.pores.glint_variation,
        GlintFollow => r.pores.glint_follow,
        PoreCrater => r.pores.crater,
        PoreScale => r.pores.crater_scale,
        PoreGleam => r.pores.gleam,
        PoreSize => r.pores.size_mm,
        PoreDepth => r.pores.depth,
        Grain => r.pores.grain,
        GrainSize => r.pores.grain_mm,
        Mottling => r.pores.mottling,
        PoreSeed => r.pores.seed as f64,
        EyeUpperLash => r.eyes.upper_lash,
        EyeUpperLashMm => r.eyes.upper_lash_mm,
        EyeLowerLash => r.eyes.lower_lash,
        EyeLowerLashMm => r.eyes.lower_lash_mm,
        EyeLiner => r.eyes.liner,
        EyeLinerMm => r.eyes.liner_mm,
        EyeWingMm => r.eyes.wing_mm,
        EyeWingDegrees => r.eyes.wing_degrees,
        EyeLinerLower => r.eyes.liner_lower,
        EyeLinerColour(k) => r.eyes.liner_colour[k],
        EyeCrease => r.eyes.crease,
        EyeCreaseMm => r.eyes.crease_mm,
        EyeSocket => r.eyes.socket,
        EyeUnderEye => r.eyes.under_eye,
        EyeRims => r.eyes.rims,
        EyeLidRed => r.eyes.lid_red,
        EyeShadow => r.eyes.shadow,
        EyeShadowMm => r.eyes.shadow_mm,
        EyeShadowColour(k) => r.eyes.shadow_colour[k],
        EyeLidLight => r.eyes.lid_light,
        EyeBags => r.eyes.bags,
        EyeBagsMm => r.eyes.bags_mm,
        Lip(k) => r.lips.values()[k],
        Ball(k) => r.eyes.ball.get(k),
        Brow(k) => r.brows.values()[k],
        Map(k) => r.maps.values()[k],
        Skin(k) => r.skin.values()[k],
    }
}

/// Sets one field of a recipe.
pub fn set(r: &mut Recipe, field: Field, v: f64) {
    use Field::*;
    match field {
        Melanin => r.base_tone.melanin = v,
        Haemoglobin => r.base_tone.haemoglobin = v,
        ChromaScale => r.base_tone.chroma_scale = v,
        Undertone => r.base_tone.undertone_degrees = v,
        Unnatural => r.base_tone.unnatural_degrees = v,
        ShadingStrength => r.shading.strength = v,
        HighlightWhiteness => r.shading.highlight_whiteness = v,
        SheenCheeks => r.sheen.cheeks = v,
        SheenNose => r.sheen.nose = v,
        SheenChin => r.sheen.chin = v,
        Age => r.wrinkles.age = v,
        WrinkleDepth => r.wrinkles.depth = v,
        WrinkleWidth => r.wrinkles.width = v,
        Wobble => r.wrinkles.wobble_mm = v,
        FineLines => r.wrinkles.fine = v,
        WrinkleSeed => r.wrinkles.seed = seed(v),
        SetStrength(s) => r.wrinkles.sets[s].strength = v,
        SetWidth(s) => r.wrinkles.sets[s].width = v,
        SetLength(s) => r.wrinkles.sets[s].length = v,
        SetCount(s) => r.wrinkles.sets[s].count = v,
        SetShift(s) => r.wrinkles.sets[s].shift_mm = v,
        LinesAmount => r.forehead.amount = v,
        LinesSpacing => r.forehead.spacing = v,
        LinesContinuity => r.forehead.continuity = v,
        LinesWaviness => r.forehead.waviness = v,
        LinesVariation => r.forehead.variation = v.round().max(0.0) as u32,
        LinesDepth => r.forehead.depth = v,
        LinesSharpness => r.forehead.sharpness = v,
        LinesRidge => r.forehead.ridge = v,
        LinesDip => r.forehead.dip_mm = v,
        LinesReach => r.forehead.reach = v,
        PoreDensity => r.pores.density = v,
        PoreGradient => r.pores.gradient = v,
        GrainDirection => r.pores.grain_direction = v,
        PoreStretch => r.pores.stretch = v,
        Glint => r.pores.glint = v,
        GlintMm => r.pores.glint_mm = v,
        GlintPoints => r.pores.glint_points = v,
        GlintVariation => r.pores.glint_variation = v,
        GlintFollow => r.pores.glint_follow = v,
        PoreCrater => r.pores.crater = v,
        PoreScale => r.pores.crater_scale = v,
        PoreGleam => r.pores.gleam = v,
        PoreSize => r.pores.size_mm = v,
        PoreDepth => r.pores.depth = v,
        Grain => r.pores.grain = v,
        GrainSize => r.pores.grain_mm = v,
        Mottling => r.pores.mottling = v,
        PoreSeed => r.pores.seed = seed(v),
        EyeUpperLash => r.eyes.upper_lash = v,
        EyeUpperLashMm => r.eyes.upper_lash_mm = v,
        EyeLowerLash => r.eyes.lower_lash = v,
        EyeLowerLashMm => r.eyes.lower_lash_mm = v,
        EyeLiner => r.eyes.liner = v,
        EyeLinerMm => r.eyes.liner_mm = v,
        EyeWingMm => r.eyes.wing_mm = v,
        EyeWingDegrees => r.eyes.wing_degrees = v,
        EyeLinerLower => r.eyes.liner_lower = v,
        EyeLinerColour(k) => r.eyes.liner_colour[k] = v,
        EyeCrease => r.eyes.crease = v,
        EyeCreaseMm => r.eyes.crease_mm = v,
        EyeSocket => r.eyes.socket = v,
        EyeUnderEye => r.eyes.under_eye = v,
        EyeRims => r.eyes.rims = v,
        EyeLidRed => r.eyes.lid_red = v,
        EyeShadow => r.eyes.shadow = v,
        EyeShadowMm => r.eyes.shadow_mm = v,
        EyeShadowColour(k) => r.eyes.shadow_colour[k] = v,
        EyeLidLight => r.eyes.lid_light = v,
        EyeBags => r.eyes.bags = v,
        EyeBagsMm => r.eyes.bags_mm = v,
        Lip(k) => *r.lips.field(k) = v,
        Ball(k) => *r.eyes.ball.field(k) = v,
        Brow(k) => *r.brows.field(k) = v,
        Map(k) => *r.maps.field(k) = v,
        Skin(k) => *r.skin.field(k) = v,
    }
}

/// One number per control, in the order of `controls`.
pub fn values_of(recipe: &Recipe) -> Vec<f64> {
    controls().iter().map(|c| get(recipe, c.field)).collect()
}

/// The recipe of one number per control, in the order of `controls`. Missing numbers keep the default recipe's values.
pub fn recipe_from(values: &[f64]) -> Recipe {
    let mut r = default_recipe();
    for (c, &v) in controls().iter().zip(values) {
        set(&mut r, c.field, v);
    }
    r
}

/// The groups and the controls as JSON, for a page that builds its sliders from them.
pub fn controls_json() -> String {
    serde_json::json!({ "groups": GROUPS, "controls": controls() }).to_string()
}

/// A recipe as JSON: one value per control id.
pub fn values_json(recipe: &Recipe) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = controls().iter().map(|c| (c.id.clone(), serde_json::json!(get(recipe, c.field)))).collect();
    serde_json::Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrinkles::SET_COUNT;

    #[test]
    fn every_number_of_the_recipe_has_one_control_with_a_unique_id() {
        let list = controls();
        assert_eq!(list.len(), 5 + 2 + 3 + 6 + 5 * SET_COUNT + 10 + 18 + 26 + crate::eyeball::SLIDERS.len() + crate::lips::SLIDERS.len() + crate::brows::SLIDERS.len() + crate::maps::SLIDERS.len() + crate::skin::SLIDERS.len());
        let mut ids: Vec<&str> = list.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), list.len());
        assert!(list.iter().all(|c| GROUPS.iter().any(|g| g.id == c.group)));
    }

    #[test]
    fn a_recipe_survives_its_values() {
        let mut r = default_recipe();
        r.wrinkles.sets[2].shift_mm = -2.0;
        r.pores.seed = 9;
        r.base_tone.unnatural_degrees = 80.0;
        let back = recipe_from(&values_of(&r));
        let mut expected = r;
        for (s, set) in expected.wrinkles.sets.iter_mut().enumerate() {
            set.count = set.count.min(SETS[s].lines.len() as f64);
        }
        assert_eq!(back, expected);
    }

    #[test]
    fn each_control_moves_its_own_number_only() {
        let base = default_recipe();
        let start = values_of(&base);
        for (k, c) in controls().iter().enumerate() {
            let mut v = start.clone();
            v[k] = if (c.max - v[k]).abs() > 1e-9 { c.max } else { c.min };
            let moved = values_of(&recipe_from(&v));
            for (j, (a, b)) in moved.iter().zip(&start).enumerate() {
                assert_eq!(a != b, j == k, "{} moved control {j}", c.id);
            }
        }
    }

    #[test]
    fn defaults_lie_in_their_ranges() {
        for c in controls() {
            assert!(c.min <= c.default && c.default <= c.max && c.min < c.max && c.step > 0.0, "{}", c.id);
        }
    }

    #[test]
    fn the_switches_map_to_the_layers() {
        let layers = layers_from(&[true, false, true, false, true]);
        assert_eq!(layers, Layers { base_tone: true, shading: false, sheen: true, wrinkles: false, pores: true, eyes: true, lips: true, brows: true, facial_hair: true, maps: false });
        assert_eq!(switches_of(&layers), vec![true, false, true, false, true, true, true, true, false, true]);
        assert_eq!(layers_from(&[]), Layers::default());
        // A page that sends one switch per group finds the maps' switch last.
        assert!(layers_from(&[true; 9]).maps);
        assert!(!layers_from(&[true; 8]).maps);
    }
}
