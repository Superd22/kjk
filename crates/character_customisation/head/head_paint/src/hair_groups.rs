//! Hair groups (paint track, part C, decisions 61 and 63). A head's facial hair is its list of hair groups. A group is a stroke, a
//! filled area or a stubble, with its own sliders: the length, the density, the thickness, the curl, the colour, the shadow on the skin,
//! the patchiness and the greying of its hairs.
//!
//! - A stroke is a curve through its control points with a width that changes along it, and its hairs grow along the curve, from its
//!   first point toward its last.
//! - A filled area is the inside of a closed outline through its control points, and its hairs grow the way the grid's zones set by
//!   default, such as back over the scalp and down the nape. A buzz cut is a filled area over the scalp.
//! - A stubble has no control points. It covers the beard's seven regions (`facial_hair::REGIONS`), each with its own density slider,
//!   inside the cheek line, the neck line and the sideburns' top, which are sliders too. These were the facial hair's global sliders
//!   until 2026-11-21.
//!
//! The curve between two control points is a cubic through both. A point is smooth, where the curve goes through it without a corner,
//! or a corner. Each piece of the curve between two points can be bent: its `bend` moves the piece's middle by that many columns and
//! rows, through the tangents at its two ends, without adding a point.
//!
//! A preset, such as a moustache, is a saved group. The presets with control points are made with the grid by the tool `paint_grid` of
//! `head_paint_gpu` and kept in `data/hair_presets.json`. The stubble presets are made here from their slider values.
//!
//! The groups sit in a list, the top one first. They are laid from the bottom of the list up, and each one covers what lies under it, so
//! a grid point takes its hair from the top group that covers it. At a group's soft edge its hair thins into the hair under it, so that a
//! moustache fades into the stubble round it. A group with points is mirrored to the other side of the face by default: column `i`
//! becomes column `cols - 1 - i`.
//!
//! A group's widths are in millimetres on the mean of the 57 test heads, where a grid step is about 2 mm, and its cover is worked out
//! in grid steps. So a group covers the same grid points, the same parts of the face, on every head, and its hairs keep their real size
//! in millimetres.

use crate::face_grid::FaceGrid;
use crate::facial_hair::FacialHair;
use crate::hair_grid::HairField;
use crate::layout_points::dot;
use serde::{Deserialize, Deserializer, Serialize};

/// A grid step is about this many millimetres on the mean of the 57 test heads.
pub const MM_PER_STEP: f64 = 2.0;
/// A group's edge thins out over this many millimetres on each side of its width.
const SOFT_MM: f64 = 1.0;
/// The curve is drawn as straight pieces, this many between two control points.
const PIECES: usize = 12;
/// A stroke's two end points closer than this many grid steps are one point when the stroke is closed or joined. An end point dropped on
/// another snaps to the grid point under the pointer, up to about 0.7 steps from it.
const SAME_POINT_STEPS: f64 = 1.5;
/// A control point's width stays between these many millimetres.
pub const WIDTH_MM: [f64; 2] = [1.0, 120.0];

pub struct GroupSlider {
    pub id: &'static str,
    pub label: &'static str,
    /// A smaller heading of the group's sliders.
    pub part: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step: f64,
    pub unit: &'static str,
    pub help: &'static str,
    /// True for a slider that only a stubble has.
    pub stubble_only: bool,
    /// True for the slider that only a pull has. A pull has no other slider.
    pub pull_only: bool,
    /// True for a slider that only a filled area has.
    pub area_only: bool,
}

const fn slider(id: &'static str, label: &'static str, part: &'static str, range: [f64; 4], unit: &'static str, help: &'static str, stubble_only: bool) -> GroupSlider {
    GroupSlider { id, label, part, min: range[0], max: range[1], default: range[2], step: range[3], unit, help, stubble_only, pull_only: false, area_only: false }
}

const fn pull_slider(id: &'static str, label: &'static str, range: [f64; 4], help: &'static str) -> GroupSlider {
    GroupSlider { id, label, part: "Pull", min: range[0], max: range[1], default: range[2], step: range[3], unit: "", help, stubble_only: false, pull_only: true, area_only: false }
}

const fn area_slider(id: &'static str, label: &'static str, range: [f64; 4], unit: &'static str, help: &'static str) -> GroupSlider {
    GroupSlider { id, label, part: "Edges", min: range[0], max: range[1], default: range[2], step: range[3], unit, help, stubble_only: false, pull_only: false, area_only: true }
}

/// The sliders of a hair group, in the order of `HairGroup::values`. The first nine were the only ones before 2026-11-21, and a group
/// saved then reads the rest at their defaults.
pub const GROUP_SLIDERS: [GroupSlider; 52] = [
    slider("length_mm", "Length", "Hair", [0.2, 20.0, 3.0, 0.05], "mm", "The typical length of the group's hairs, in millimetres along the skin. Each hair is between 60% and 140% of it. A day's stubble is about 0.5 mm, and a week's about 3 mm.", false),
    slider("density", "Density", "Hair", [0.0, 1.0, 0.9, 0.01], "", "How dense the group's hair is. At 1 there are two hairs per square millimetre, a full beard's density. On a stubble it multiplies the density of every region, and at 0 the stubble is gone.", false),
    slider("thickness_mm", "Thickness", "Hair", [0.04, 0.25, 0.12, 0.005], "mm", "How thick each hair is at its root, in millimetres. Real beard hairs are about 0.1 mm thick. Each hair thins toward its tip.", false),
    slider("curl", "Curl", "Hair", [0.0, 1.0, 0.15, 0.01], "", "How much each hair bends and waves. At 0 the hairs are almost straight.", false),
    slider("lightness", "Lightness", "Colour", [3.0, 95.0, 20.0, 0.5], "L*", "How light the hairs are, from black at 3 to white at 95. On a head with dark skin, dark hair is also kept at most 22% as light as the skin under it, so that it shows. On a head whose base skin tone is lighter than 40 L*, the hair keeps its own colour. Hair lighter than 50 keeps its own colour.", false),
    slider("hue", "Hue", "Colour", [0.0, 360.0, 55.0, 1.0], "degrees", "The hairs' hue around the colour wheel. Brown and red hair is between 30 and 70 degrees.", false),
    slider("chroma", "Colour strength", "Colour", [0.0, 60.0, 14.0, 0.5], "", "How strong the hairs' colour is. At 0 they are black, grey or white.", false),
    slider("grey", "Grey hairs", "Colour", [0.0, 1.0, 0.0, 0.01], "", "The share of the group's hairs that are grey or white. At 1 every hair is white.", false),
    slider("droop", "Lean down", "Hair", [0.0, 1.0, 0.5, 0.01], "", "How far the hairs turn from their own direction toward the ground, when the head is upright. A stroke's own direction is along its curve, and a filled area's or a stubble's is the zones' default. At 0 the hairs keep their own direction, and at 1 they hang straight down. Where the skin faces the ground, as under the jaw, the hairs keep more of their own direction.", false),
    slider("shadow", "Shadow", "Hair", [0.0, 30.0, 6.0, 0.1], "L*", "How much darker the skin is under the group's hair, in L* at full density, and how much it takes the hair's colour, as the hairs under the skin and between the stubble show through. This is the beard shadow of a shaved face.", false),
    slider("patchy", "Patchiness", "Hair", [0.0, 1.0, 0.3, 0.01], "", "How unevenly the hair grows. At 0 the density is smooth. At 1 broad patches of about a centimetre have much less hair.", false),
    slider("seed", "Random pattern", "Hair", [0.0, 20.0, 0.0, 1.0], "", "Picks another set of single hairs and another pattern of patches for this group.", false),
    slider("tone_var", "Tone variation", "Colour", [0.0, 1.0, 0.4, 0.01], "", "How much the hairs differ in lightness from each other. At 0 every hair has the same colour.", false),
    slider("sheen", "Sheen", "Colour", [0.0, 1.0, 0.2, 0.01], "", "How much light each hair catches along its middle, as a warm grey of about 39 L*. It gives a beard texture on dark skin, where black hairs alone hardly show, and hardly changes a beard on pale skin.", false),
    slider("moustache", "Moustache", "Where it grows", [0.0, 1.0, 0.85, 0.01], "", "The density of the hair on the upper lip, between the nose and the lip. At 1 there are two hairs per square millimetre, a full beard's density.", true),
    slider("soul_patch", "Soul patch", "Where it grows", [0.0, 1.0, 0.6, 0.01], "", "The density of the hair in the small patch under the middle of the lower lip.", true),
    slider("chin", "Chin", "Where it grows", [0.0, 1.0, 0.85, 0.01], "", "The density of the hair on the front of the chin, below the soul patch and above the jaw's edge.", true),
    slider("jaw", "Jaw line", "Where it grows", [0.0, 1.0, 0.75, 0.01], "", "The density of the hair in a band about 2 cm wide along the lower edge of the jaw, from under the chin to below the ear.", true),
    slider("cheeks", "Cheeks", "Where it grows", [0.0, 1.0, 0.55, 0.01], "", "The density of the hair on the cheeks, between the mouth, the jaw line and the cheek line.", true),
    slider("sideburns", "Sideburns", "Where it grows", [0.0, 1.0, 0.7, 0.01], "", "The density of the hair in front of the ears, from the top of the sideburn down to the jaw.", true),
    slider("neck", "Neck", "Where it grows", [0.0, 1.0, 0.4, 0.01], "", "The density of the hair under the jaw and the chin and on the front of the neck, down to the neck line.", true),
    slider("cheek_line_mm", "Cheek line", "Edges", [-15.0, 8.0, 0.0, 0.1], "mm", "Moves the upper edge of the hair on the cheeks up, or down for a negative value, in millimetres. The cheek line runs from the side of the nose to the top of the sideburn.", true),
    slider("neck_line_mm", "Neck line", "Edges", [-20.0, 20.0, 0.0, 0.1], "mm", "Moves the lower edge of the hair on the neck down, or up for a negative value, in millimetres. By default it runs 45 mm below the jaw's edge, measured along the skin.", true),
    slider("sideburn_top_mm", "Sideburn height", "Edges", [-20.0, 8.0, 0.0, 0.1], "mm", "Moves the top of the sideburns up, or down for a negative value, in millimetres. By default the sideburns reach the temple, 16 mm above the front of the ear's middle. The back of the cheek line follows them.", true),
    slider("edge_mm", "Edge softness", "Edges", [1.0, 12.0, 4.0, 0.1], "mm", "Over how many millimetres the hair thins out at its edges. A small value gives a trimmed, sharp edge.", true),
    pull_slider("pull", "Strength", [-1.0, 1.0, 0.6, 0.01], "How strongly the hairs under the pull turn toward its direction, the way a comb lays them. The pull is strongest on its curve and fades to nothing at the edge of its width. A pull with one point turns the hairs toward that point. A negative value turns the hairs the other way, or away from the point."),
    area_slider("area_edge_mm", "Edge softness", [0.5, 12.0, 1.5, 0.1], "mm", "Over how many millimetres the hair thins out on each side of the filled area's outline. A small value gives a trimmed, sharp edge. A large value feathers the edge, so that the hair thins out slowly and a little of it grows past the outline."),
    slider("lock_mm", "Lock size", "Locks", [0.0, 10.0, 0.0, 0.1], "mm", "How wide a lock of hair is at its roots, in millimetres. A lock is a bundle of neighbouring hairs that run together. At 0 the hairs grow one by one and the other lock sliders do nothing.", false),
    slider("lock_gather", "Gathering", "Locks", [0.0, 1.0, 0.0, 0.01], "", "How tightly the hairs of a lock gather toward its middle along their length. At 0 each hair keeps its own path. At 1 the hairs of a lock meet at their tips, and bare skin or the hair under them shows between the locks.", false),
    slider("lock_light", "Lock highlight", "Locks", [0.0, 1.0, 0.0, 0.01], "", "How much each lock catches the light. The hairs along a lock's middle are lighter, its edges are darker, and each lock has a shade of its own. At 0 every lock has the same shade.", false),
    slider("flow", "Follow the flow", "Hair", [0.0, 1.0, 0.0, 0.01], "", "How far each hair bends to follow the direction of growth along its length, as a comb or a pull turns it. At 0 each hair is a nearly straight line, and at 1 it curves with the flow, so that hairs fan round the chin and sweep along the jaw.", false),
    slider("lock_wave", "Wave", "Locks", [0.0, 1.0, 0.0, 0.01], "", "How much each lock sways from side to side along its length, so that neighbouring locks wave and cross. At 1 a lock sways by about 1.2 millimetres each way. At 0 the locks do not sway.", false),
    slider("lock_soft", "Lock softness", "Locks", [0.0, 1.0, 0.0, 0.01], "", "How broad and soft each lock's light and shade are. At 0 the light falls on a narrow streak along the lock's middle and the lock's edges are dark. At 1 the light spreads over most of the lock's width and fades slowly into its darker edges, so that the locks read as soft bands rather than bright lines.", false),
    slider("strand_mm", "Strand width", "Painted strands", [0.0, 1.5, 0.0, 0.01], "mm", "How wide each strand is, in millimetres, when it is wider than the Thickness. A strand is a broad brush stroke that stands for many hairs, as in a painted beard. It narrows at both ends and fades out where the group's hair thins out, as at a bare patch or the lips. Wider strands are fewer, so that the beard covers the skin about as much as before. At 0 the hairs keep the Thickness.", false),
    slider("straight", "Straightness", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How straight the strands run along the flow. At 1 the random lean, bend and wave of each hair and the random lean and bend of each lock are gone, so the hairs follow only the flow lines and the flow. A painted strand is always a straight stroke, and the strands of a lock all run in the direction at the lock's middle. At 0 each hair keeps its own random path.", false),
    slider("strand_shade", "Strand shading", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How much each strand is shaded across its width, like a small cylinder. Its middle is lighter, its edges are darker, and the skin between the strands lies in their shadow. At 1 a strand's middle is about 1.7 times as light as its colour and its edges are about 4 times darker. At 0 a strand has one shade across its width.", false),
    slider("bands", "Painted bands", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How much the light and dark of the strands fall into a few bold steps, like the bands of a painted texture, instead of many faint ones. At 1 each step is about 60% lighter than the one below it. At 0 the light and dark change smoothly.", false),
    slider("root_light", "Light at the root", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How much lighter each strand is near its root than near its tip, as in a painted beard, where each lock is lit where it starts and darkens where it goes under the next one. At 1 a strand's root is about 1.6 times as light as its colour and its tip about 2 times darker. At 0 a strand has the same lightness along its length.", false),
    slider("layering", "Layering", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How strictly the strands rooted further along the flow lie on top of the strands above them, like the rows of a thatched roof. With Light at the root, the strand on top is then the lit start of a lock. At 0 the strands lie over each other in a random order.", false),
    slider("packing", "Packing", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How tightly the painted strands pack. At 1 there are 3.5 times as many strands, each one fully opaque, so that they overlap like shingles, and the skin under the beard is painted over in the hair's own colour, about 25% darker, so that no skin shows between the strands. The under colour thins out with the hair at the beard's edges and at bare patches. At 0 the strands are as many as their width gives and the skin shows between them.", false),
    slider("lock_tone", "Shading by lock", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How much a strand takes its light and dark from broad bands instead of from itself. The bands are soft streaks that run along the flow, each about as long as two locks and a quarter of a lock wide, so that one band spans several strands. Neighbouring strands then differ only by their own shading across their width. Tone variation sets how strong the bands are, and Lock size sets how large they are. At 0 each strand has a shade of its own. It needs a Lock size above 0.", false),
    slider("parallel", "Parallel strands", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How strictly the strands run side by side along the flow. At 1 each strand takes its direction from the flow at its own root, so that neighbouring strands are parallel and strands of neighbouring locks do not cross, and the strands of a lock no longer gather toward its middle. At 0 the strands of a lock all take the direction at the lock's middle and gather as Gathering says.", false),
    slider("fine_lines", "Fine lines", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How strongly each strand shows fine, parallel light lines along its length, like combed hairs inside a painted lock. The lines are about 0.15 mm wide and 0.5 mm apart. At 1 a line is about 1.8 times as light as the strand's colour and the strand between the lines is about 25% darker. The broad light and dark bands stay as they are. At 0 a strand has no lines.", false),
    slider("feather", "Feathered edges", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How softly the strands and their under colour fade out where the hair thins out, as at a bare patch or the beard's edge. At 1 the strands grow see-through over the whole edge and each strand ends at a place of its own, so that the edge is a soft fringe. At 0 the strands stop sharply where the hair thins out.", false),
    slider("over_lips", "Over the lips", "Hair", [0.0, 1.0, 0.0, 0.01], "", "How far the group's hairs may hang over the lips. At 0 they reach about 3 grid points, roughly 5 mm, past the lips' border, which leaves the edge of the upper lip bare under a long moustache. At 1 they reach 2 points further, so that a full moustache covers the upper lip down to the mouth. The hairs never grow from the lips.", false),
    slider("clump_mm", "Clump width", "Painted strands", [0.0, 6.0, 0.0, 0.05], "mm", "How wide a clump of strands is, in millimetres. The strands whose roots lie in one clump are shaded together, as one piece, from light along its middle to dark at its edges, and they share one shade. Strand shading then shades the whole clump instead of each strand, and the fine lines run across the whole clump. At 0 each strand is shaded on its own.", false),
    slider("clump_wave", "Clump wave", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How much each clump of strands waves gently from side to side along the flow. All the strands of a clump wave together. At 1 a clump sways by about 0.8 millimetres each way, with a wavelength of 9 to 13 millimetres. At 0 the clumps run straight. It needs a Clump width above 0.", false),
    slider("light_zones", "Light zones", "Colour", [0.0, 1.0, 0.0, 0.01], "", "How much the hair is lit in broad, soft zones that follow the face's shape, as in a painted texture. Hair on skin that faces up or forward, as on the cheeks and the front of the chin, is lighter. Hair in hollows, as under the lower lip and at the mouth's corners, and on skin that faces down, as under the jaw, is darker. At 1 the lightest zones are about 1.5 times as light as the hair's colour and the darkest about 2 times darker. At 0 the hair has the same lightness everywhere.", false),
    slider("edge_mix", "Mixed edges", "Hair", [0.0, 1.0, 0.0, 0.01], "", "How much the hairs mix where this group meets another group. Each hair rooted near the meeting line takes the look of a grid point up to about 4 millimetres to one side, chosen at random, and a painted strand also takes its direction from there, so that the two groups fray into each other instead of meeting in a straight line. At 0 each hair takes the look of the grid point nearest its root.", false),
    slider("comb", "Combed streaks", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How much the strands take their light and dark from long, combed streaks instead of from each strand. A streak runs unbroken along the flow lines, as a long lock of a painted beard does, so that the light and dark run on from the cheek and the chin down to the beard's edge. Inside the beard no strand's own start or tip shows, and only the strands at the beard's edges and at bare patches show their tips. At 1 the strands' own shading, their clumps' shading, their fine lines, their light at the root and their locks' highlight are gone, and only the streaks shade the hair. At 0 each strand is shaded on its own.", false),
    slider("streak_mm", "Streak length", "Painted strands", [4.0, 30.0, 18.0, 0.5], "mm", "How long the combed streaks are, in millimetres along the flow. A streak's light or dark runs on over about this length before it fades into another one. It needs Combed streaks above 0.", false),
    slider("streak_sharp", "Sharp streaks", "Painted strands", [0.0, 1.0, 0.0, 0.01], "", "How sharp and how few the light combed streaks are. At 0 the streaks go softly from dark to light, from about 2.2 times darker to 2.2 times as light as the hair's colour, and half of the hair is lighter than its colour. At 1 the hair between the streaks is 25% darker than its colour, and only about one streak in five is light, up to about 2.3 times as light as that darker base. It needs Combed streaks above 0.", false),
];

/// The place of a slider in `GROUP_SLIDERS` by its id. It panics on an unknown id, which is a mistake in this file.
const fn at(id: &str) -> usize {
    let mut k = 0;
    while k < GROUP_SLIDERS.len() {
        let (a, b) = (GROUP_SLIDERS[k].id.as_bytes(), id.as_bytes());
        if a.len() == b.len() {
            let mut same = true;
            let mut c = 0;
            while c < a.len() {
                same &= a[c] == b[c];
                c += 1;
            }
            if same {
                return k;
            }
        }
        k += 1;
    }
    panic!("no such group slider")
}

const LENGTH: usize = at("length_mm");
const DENSITY: usize = at("density");
const THICKNESS: usize = at("thickness_mm");
const CURL: usize = at("curl");
const LIGHTNESS: usize = at("lightness");
const HUE: usize = at("hue");
const CHROMA: usize = at("chroma");
const GREY: usize = at("grey");
const DROOP: usize = at("droop");
const SHADOW: usize = at("shadow");
const PATCHY: usize = at("patchy");
const SEED: usize = at("seed");
const TONE_VAR: usize = at("tone_var");
const SHEEN: usize = at("sheen");
const REGION_DENSITY: usize = at("moustache");
const CHEEK_LINE: usize = at("cheek_line_mm");
const NECK_LINE: usize = at("neck_line_mm");
const SIDEBURN_TOP: usize = at("sideburn_top_mm");
const EDGE: usize = at("edge_mm");
const PULL: usize = at("pull");
const AREA_EDGE: usize = at("area_edge_mm");
const LOCK: usize = at("lock_mm");
const LOCK_GATHER: usize = at("lock_gather");
const LOCK_LIGHT: usize = at("lock_light");
const FLOW: usize = at("flow");
const LOCK_WAVE: usize = at("lock_wave");
const LOCK_SOFT: usize = at("lock_soft");
const STRAND: usize = at("strand_mm");
const STRAIGHT: usize = at("straight");
const STRAND_SHADE: usize = at("strand_shade");
const BANDS: usize = at("bands");
const ROOT_LIGHT: usize = at("root_light");
const LAYERING: usize = at("layering");
const PACKING: usize = at("packing");
const LOCK_TONE: usize = at("lock_tone");
const PARALLEL: usize = at("parallel");
const FINE_LINES: usize = at("fine_lines");
const FEATHER: usize = at("feather");
const OVER_LIPS: usize = at("over_lips");
const CLUMP: usize = at("clump_mm");
const CLUMP_WAVE: usize = at("clump_wave");
const LIGHT_ZONES: usize = at("light_zones");
const EDGE_MIX: usize = at("edge_mix");
const COMB: usize = at("comb");
const STREAK: usize = at("streak_mm");
const STREAK_SHARP: usize = at("streak_sharp");

/// Every slider at its default.
pub fn default_values() -> Vec<f64> {
    GROUP_SLIDERS.iter().map(|s| s.default).collect()
}

fn values_with_defaults<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<f64>, D::Error> {
    let mut v = Vec::<f64>::deserialize(d)?;
    let all = default_values();
    if v.len() < all.len() {
        v.extend_from_slice(&all[v.len()..]);
    }
    v.truncate(all.len());
    Ok(v)
}

/// Whether a group is a stroke along its curve, the filled inside of its closed outline, or a stubble over the beard's regions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupKind {
    #[default]
    Stroke,
    Area,
    Stubble,
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_zero(v: &[f64; 2]) -> bool {
    v[0] == 0.0 && v[1] == 0.0
}

/// A control point of a group's curve, at a column `i` and row `j` of the grid, with the curve's width there in millimetres. `bend`
/// moves the middle of the piece of curve from this point to the next by that many columns and rows. A `corner` point lets the curve
/// turn sharply there.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlPoint {
    pub i: f64,
    pub j: f64,
    pub width_mm: f64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bend: [f64; 2],
    #[serde(default, skip_serializing_if = "is_false")]
    pub corner: bool,
}

impl ControlPoint {
    pub fn new(i: f64, j: f64, width_mm: f64) -> Self {
        ControlPoint { i, j, width_mm, bend: [0.0; 2], corner: false }
    }
}

/// A flow line of a group: a curve through its control points that sets the direction of the group's hairs, from its first point
/// toward its last. Its points' widths paint nothing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FlowLine {
    #[serde(default)]
    pub points: Vec<ControlPoint>,
}

/// Near a flow line, its own direction wins over the other lines'. A line's weight is `1 / (d² + FLOW_NEAR_MM²)²` at `d` millimetres
/// from it, so a line 3 mm away weighs about 4 times less than one on the spot, and one 10 mm away about 100 times less.
const FLOW_NEAR_MM: f64 = 3.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HairGroup {
    pub name: String,
    #[serde(default)]
    pub kind: GroupKind,
    #[serde(default)]
    pub points: Vec<ControlPoint>,
    #[serde(default)]
    pub mirror: bool,
    /// The values of `GROUP_SLIDERS`, in their order.
    #[serde(deserialize_with = "values_with_defaults")]
    pub values: Vec<f64>,
    /// A hidden group paints nothing.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
    /// A pull is a stroke that grows no hair. It turns the hairs of every group under its width toward its own direction.
    #[serde(default, skip_serializing_if = "is_false")]
    pub pull: bool,
    /// The group's flow lines. With at least one, every hair of the group takes its direction from them instead of from its curve or
    /// the zones. They are mirrored with the group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flow: Vec<FlowLine>,
}

/// The hair groups of a layer, the top one first.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HairGroups {
    pub groups: Vec<HairGroup>,
}

/// The presets, by id and label. The label is the group's name, and the name in `data/hair_presets.json` for a preset with points. A
/// preset of several groups is listed by its label under `sets` in that file, with the names of its groups, the top one first.
pub const PRESETS: [(&str, &str); 6] = [("stubble", "Stubble"), ("short_beard", "Short beard"), ("moustache", "Moustache"), ("sideburn", "Sideburn"), ("buzz_cut", "Buzz cut"), ("full_beard", "Full beard")];

const PRESET_GROUPS: &str = include_str!("data/hair_presets.json");


/// The names of the groups of the preset `label` when it has several, the top one first.
fn preset_set(label: &str) -> Option<Vec<String>> {
    let v: serde_json::Value = serde_json::from_str(PRESET_GROUPS).ok()?;
    serde_json::from_value(v["sets"][label].clone()).ok()
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// What a stubble lays on the grid, at each point: how much it covers the point, from 0 to 1, and its density in hairs per square
/// millimetre and its shadow from 0 to 1, both already multiplied by the cover.
pub struct StubbleLayer {
    pub cover: Vec<f64>,
    pub density: Vec<f64>,
    pub shadow: Vec<f64>,
}

/// The flow lines of a group: their direction at each grid point, and their editing through the same handles as a stroke.
impl HairGroup {
    /// Flow line `n` as a stroke of its own, mirrored with the group, so that it is drawn and edited as a stroke is.
    pub fn flow_stroke(&self, n: usize) -> Option<HairGroup> {
        let line = self.flow.get(n)?;
        Some(HairGroup { name: format!("{} flow {}", self.name, n + 1), kind: GroupKind::Stroke, points: line.points.clone(), mirror: self.mirror, values: default_values(), hidden: false, pull: false, flow: Vec::new() })
    }

    /// Edits flow line `n` as a stroke (`flow_stroke`) and keeps its new points. `None` when there is no such line.
    pub fn edit_flow<T>(&mut self, n: usize, f: impl FnOnce(&mut HairGroup) -> T) -> Option<T> {
        let mut s = self.flow_stroke(n)?;
        let r = f(&mut s);
        self.flow[n].points = s.points;
        Some(r)
    }

    /// Adds an empty flow line after the others. Its points are added by clicking on the head. Returns its number.
    pub fn add_flow(&mut self) -> usize {
        self.flow.push(FlowLine::default());
        self.flow.len() - 1
    }

    pub fn remove_flow(&mut self, n: usize) {
        if n < self.flow.len() {
            self.flow.remove(n);
        }
    }

    /// The flow lines' curves, without their mirror images, as columns and rows. A line of fewer than two points has no direction and
    /// is left out.
    fn flow_curves(&self) -> Vec<Vec<[f64; 2]>> {
        (0..self.flow.len()).filter_map(|n| self.flow_stroke(n)).filter(|s| s.points.len() >= 2).map(|s| s.curve().iter().map(|q| [q[0], q[1]]).collect()).collect()
    }

    /// The direction of the flow lines at column `i` and row `j`, in columns and rows of unit length, or `None` for a group with no
    /// flow line of two points. Each line gives its direction at its nearest place, and the directions are blended by distance, so that
    /// the nearest line wins near it and the direction turns smoothly between two lines. A mirrored group's lines are drawn on one side
    /// of the face, and the other side takes their mirror image, so that the two halves meet in a seam down the middle.
    pub fn flow_direction(&self, cols: usize, i: f64, j: f64) -> Option<[f64; 2]> {
        self.flow_field(cols).map(|f| f.at(i, j))
    }

    /// The flow lines ready to give their direction at many grid points (`flow_direction`), or `None` without a line of two points.
    pub fn flow_field(&self, cols: usize) -> Option<FlowField> {
        let curves = self.flow_curves();
        if curves.is_empty() {
            return None;
        }
        let mid = (cols - 1) as f64 / 2.0;
        let side: f64 = curves.iter().flatten().map(|q| q[0] - mid).sum();
        Some(FlowField { curves, cols, side: if self.mirror { side } else { 0.0 } })
    }
}

/// A group's flow lines, as `HairGroup::flow_field` makes them.
pub struct FlowField {
    curves: Vec<Vec<[f64; 2]>>,
    cols: usize,
    /// Which side of the middle column the lines lie on, or 0 when the group is not mirrored.
    side: f64,
}

impl FlowField {
    /// The direction at column `i` and row `j` (`HairGroup::flow_direction`).
    pub fn at(&self, i: f64, j: f64) -> [f64; 2] {
        let last = (self.cols - 1) as f64;
        let flip = self.side * (i - 0.5 * last) < 0.0;
        let t = Self::blend(&self.curves, if flip { last - i } else { i }, j);
        if flip { [-t[0], t[1]] } else { t }
    }

    fn blend(curves: &[Vec<[f64; 2]>], i: f64, j: f64) -> [f64; 2] {
        let mut sum = [0.0; 2];
        for c in curves {
            let mut best = (f64::MAX, [0.0; 2]);
            for w in c.windows(2) {
                let e = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
                let len2 = e[0] * e[0] + e[1] * e[1];
                if len2 < 1e-12 {
                    continue;
                }
                let t = (((i - w[0][0]) * e[0] + (j - w[0][1]) * e[1]) / len2).clamp(0.0, 1.0);
                let d = (w[0][0] + t * e[0] - i).hypot(w[0][1] + t * e[1] - j) * MM_PER_STEP;
                if d < best.0 {
                    let l = len2.sqrt();
                    best = (d, [e[0] / l, e[1] / l]);
                }
            }
            if best.0 == f64::MAX {
                continue;
            }
            let w = 1.0 / (best.0 * best.0 + FLOW_NEAR_MM * FLOW_NEAR_MM).powi(2);
            sum = [sum[0] + w * best.1[0], sum[1] + w * best.1[1]];
        }
        let l = sum[0].hypot(sum[1]);
        if l > 1e-12 { [sum[0] / l, sum[1] / l] } else { [0.0; 2] }
    }
}

impl HairGroup {

    /// The flow lines on the skin of `grid`, and their mirror images, for a viewer, as rows of six numbers like `curve_on_skin`. Each
    /// line ends in an arrow head that shows which way the hairs grow.
    pub fn flow_on_skin(&self, grid: &FaceGrid) -> Vec<f32> {
        let mut out = Vec::new();
        for n in 0..self.flow.len() {
            let Some(s) = self.flow_stroke(n) else { continue };
            out.extend(s.curve_on_skin(grid));
            if s.points.len() < 2 {
                continue;
            }
            for c in s.sides(grid.cols) {
                let (a, b) = (c[c.len() - 2], c[c.len() - 1]);
                let e = [b[0] - a[0], b[1] - a[1]];
                let l = e[0].hypot(e[1]).max(1e-9);
                let e = [e[0] / l, e[1] / l];
                for turn in [0.5f64, -0.5] {
                    let (cs, sn) = (turn.cos(), turn.sin());
                    let back = [-(cs * e[0] - sn * e[1]), -(sn * e[0] + cs * e[1])];
                    for q in [[b[0] + 1.2 * back[0], b[1] + 1.2 * back[1]], [b[0], b[1]]] {
                        match grid.place(q[0], q[1]) {
                            Some((at, nn)) => {
                                out.extend(at.map(|x| (x / 1000.0) as f32));
                                out.extend(nn.map(|x| x as f32));
                            }
                            None => out.extend([f32::NAN; 6]),
                        }
                    }
                    out.extend([f32::NAN; 6]);
                }
            }
        }
        out
    }
}

impl HairGroup {
    /// A preset of one group by its id in `PRESETS`, as laid on the baked paint grid. A preset of several groups is `HairGroups::preset`.
    pub fn preset(id: &str) -> Option<Self> {
        let label = PRESETS.iter().find(|p| p.0 == id)?.1;
        if preset_set(label).is_some() {
            return None;
        }
        match id {
            "stubble" => Some(Self::stubble(label, &FacialHair { amount: 1.0, ..FacialHair::DEFAULT })),
            // The beard set by hand on Carth on 2026-11-20.
            "short_beard" => Some(Self::stubble(label, &FacialHair { amount: 1.0, length_mm: 2.0, hair_mm: 0.17, shadow: 26.0, lightness: 14.0, hue: 45.0, chroma: 22.0, density: [1.0, 1.0, 1.0, 1.0, 0.4, 0.7, 0.8], patchy: 0.3, ..FacialHair::DEFAULT })),
            _ => HairGroups::from_json(PRESET_GROUPS)?.groups.into_iter().find(|g| g.name == label),
        }
    }

    /// A stubble with the values of the facial hair's old global sliders.
    pub fn stubble(name: &str, fh: &FacialHair) -> Self {
        let mut v = default_values();
        v[DENSITY] = fh.amount;
        v[LENGTH] = fh.length_mm;
        v[THICKNESS] = fh.hair_mm;
        v[CURL] = 0.0;
        v[LIGHTNESS] = fh.lightness;
        v[HUE] = fh.hue;
        v[CHROMA] = fh.chroma;
        v[GREY] = fh.grey;
        v[DROOP] = 0.0;
        v[SHADOW] = fh.shadow;
        v[PATCHY] = fh.patchy;
        v[SEED] = fh.seed;
        v[TONE_VAR] = fh.tone_var;
        v[SHEEN] = fh.sheen;
        v[REGION_DENSITY..REGION_DENSITY + 7].copy_from_slice(&fh.density);
        v[CHEEK_LINE] = fh.cheek_line_mm;
        v[NECK_LINE] = fh.neck_line_mm;
        v[SIDEBURN_TOP] = fh.sideburn_top_mm;
        v[EDGE] = fh.edge_mm;
        HairGroup { name: name.to_string(), kind: GroupKind::Stubble, points: Vec::new(), mirror: false, values: v, hidden: false, pull: false, flow: Vec::new() }
    }

    /// A stubble from the facial hair's old global sliders, given by their control ids ("beard_amount", "beard_cheeks" and so on) as a
    /// head's recipe kept them. A slider that is not given takes its old default. `None` when the stubble's amount is 0.
    pub fn stubble_from_old_sliders(get: impl Fn(&str) -> Option<f64>) -> Option<Self> {
        let d = FacialHair::DEFAULT;
        let v = |id: &str, def: f64| get(&format!("beard_{id}")).unwrap_or(def);
        let names = ["moustache", "soul_patch", "chin", "jaw", "cheeks", "sideburns", "neck"];
        let fh = FacialHair {
            amount: v("amount", d.amount),
            length_mm: v("length_mm", d.length_mm),
            hair_mm: v("hair_mm", d.hair_mm),
            shadow: v("shadow", d.shadow),
            patchy: v("patchy", d.patchy),
            seed: v("seed", d.seed),
            density: [0, 1, 2, 3, 4, 5, 6].map(|k| v(names[k], d.density[k])),
            cheek_line_mm: v("cheek_line_mm", d.cheek_line_mm),
            neck_line_mm: v("neck_line_mm", d.neck_line_mm),
            sideburn_top_mm: v("sideburn_top_mm", d.sideburn_top_mm),
            edge_mm: v("edge_mm", d.edge_mm),
            lightness: v("lightness", d.lightness),
            hue: v("hue", d.hue),
            chroma: v("chroma", d.chroma),
            tone_var: v("tone_var", d.tone_var),
            grey: v("grey", d.grey),
            sheen: v("sheen", d.sheen),
        };
        (fh.amount > 0.0).then(|| Self::stubble("Stubble", &fh))
    }

    /// A stubble's values as the stubble's own settings.
    pub fn facial_hair(&self) -> FacialHair {
        let v = &self.values;
        let mut density = [0.0; 7];
        density.copy_from_slice(&v[REGION_DENSITY..REGION_DENSITY + 7]);
        FacialHair {
            amount: v[DENSITY],
            length_mm: v[LENGTH],
            hair_mm: v[THICKNESS],
            shadow: v[SHADOW],
            patchy: v[PATCHY],
            seed: v[SEED],
            density,
            cheek_line_mm: v[CHEEK_LINE],
            neck_line_mm: v[NECK_LINE],
            sideburn_top_mm: v[SIDEBURN_TOP],
            edge_mm: v[EDGE],
            lightness: v[LIGHTNESS],
            hue: v[HUE],
            chroma: v[CHROMA],
            tone_var: v[TONE_VAR],
            grey: v[GREY],
            sheen: v[SHEEN],
        }
    }

    /// The look of the group's hairs for the shader: its colour as L*, hue and chroma, its thickness in mm, its tone variation, its
    /// share of grey hairs, its curl, its sheen, its random pattern, its locks' size in mm, gathering and highlight, how far its hairs
    /// follow the flow, its locks' wave and its locks' softness, then its strands' width in mm, straightness, shading, bands, light at the root, layering, packing,
    /// shading by lock, parallel strands, fine lines and feathered edges, how far its hairs hang over the lips, its clumps' width in mm
    /// and wave, its light zones, how much its hairs mix with another group's where they meet, and its combed streaks, their length in
    /// mm and their sharpness.
    pub fn look(&self) -> [f64; 34] {
        let v = &self.values;
        [
            v[LIGHTNESS],
            v[HUE],
            v[CHROMA],
            v[THICKNESS],
            v[TONE_VAR],
            v[GREY],
            v[CURL],
            v[SHEEN],
            v[SEED].round().max(0.0),
            v[LOCK],
            v[LOCK_GATHER],
            v[LOCK_LIGHT],
            v[FLOW],
            v[LOCK_WAVE],
            v[LOCK_SOFT],
            v[STRAND],
            v[STRAIGHT],
            v[STRAND_SHADE],
            v[BANDS],
            v[ROOT_LIGHT],
            v[LAYERING],
            v[PACKING],
            v[LOCK_TONE],
            v[PARALLEL],
            v[FINE_LINES],
            v[FEATHER],
            v[OVER_LIPS],
            v[CLUMP],
            v[CLUMP_WAVE],
            v[LIGHT_ZONES],
            v[EDGE_MIX],
            v[COMB],
            v[STREAK],
            v[STREAK_SHARP],
        ]
    }

    /// The value of a slider of `GROUP_SLIDERS` by its id.
    pub fn value_mut(&mut self, id: &str) -> Option<&mut f64> {
        GROUP_SLIDERS.iter().position(|s| s.id == id).map(|k| &mut self.values[k])
    }

    fn closed(&self) -> bool {
        self.kind == GroupKind::Area && self.points.len() >= 3
    }

    /// The number of pieces of curve between two control points.
    pub fn segments(&self) -> usize {
        match self.points.len() {
            0 | 1 => 0,
            n if self.closed() => n,
            n => n - 1,
        }
    }

    /// The tangents at both ends of piece `s`, in columns and rows per piece. Without bends and corners they are a Catmull-Rom spline's,
    /// which passes smoothly through every control point. A bend adds 4 times itself to the start's tangent and takes it from the end's,
    /// which moves the piece's middle by the bend. A smooth point shares one tangent between the pieces on its two sides. A corner
    /// point's tangent on each side starts from the straight line to its neighbour on that side.
    fn tangents(&self, s: usize) -> ([f64; 2], [f64; 2]) {
        let p = &self.points;
        let n = p.len();
        let closed = self.closed();
        let segs = self.segments();
        let idx = |k: i64| if closed { k.rem_euclid(n as i64) as usize } else { k.clamp(0, n as i64 - 1) as usize };
        let xy = |k: usize| [p[k].i, p[k].j];
        // The bend of piece `t`, or nothing where there is no such piece.
        let bend = |t: i64| if (0..segs as i64).contains(&t) || closed { p[idx(t)].bend } else { [0.0; 2] };
        let tangent = |k: usize, out: bool| -> [f64; 2] {
            let kk = k as i64;
            let (b_out, b_in) = (bend(kk), bend(kk - 1));
            if p[k].corner {
                if out {
                    let c = xy(idx(kk + 1));
                    [c[0] - p[k].i + 4.0 * b_out[0], c[1] - p[k].j + 4.0 * b_out[1]]
                } else {
                    let a = xy(idx(kk - 1));
                    [p[k].i - a[0] - 4.0 * b_in[0], p[k].j - a[1] - 4.0 * b_in[1]]
                }
            } else {
                let (a, c) = (xy(idx(kk - 1)), xy(idx(kk + 1)));
                let (b_out, b_in) = (if !closed && k + 1 == n { [0.0; 2] } else { b_out }, if !closed && k == 0 { [0.0; 2] } else { b_in });
                [0, 1].map(|x| 0.5 * (c[x] - a[x]) + 4.0 * (b_out[x] - b_in[x]))
            }
        };
        let e = idx(s as i64 + 1);
        (tangent(s, true), tangent(e, false))
    }

    /// The place of piece `s` at `t` from 0 to 1, as a column, a row and the width in mm.
    fn piece_at(&self, s: usize, t: f64) -> [f64; 3] {
        let n = self.points.len();
        let (b, c) = (self.points[s], self.points[(s + 1) % n]);
        let (m0, m1) = self.tangents(s);
        let (t2, t3) = (t * t, t * t * t);
        let (h00, h10, h01, h11) = (2.0 * t3 - 3.0 * t2 + 1.0, t3 - 2.0 * t2 + t, -2.0 * t3 + 3.0 * t2, t3 - t2);
        let q = [h00 * b.i + h10 * m0[0] + h01 * c.i + h11 * m1[0], h00 * b.j + h10 * m0[1] + h01 * c.j + h11 * m1[1]];
        [q[0], q[1], b.width_mm + t * (c.width_mm - b.width_mm)]
    }

    /// The curve through the control points, as (column, row, width in mm) every `1 / PIECES` of the way between two points. A filled
    /// area's outline closes from its last point back to its first, and ends where it starts. A stubble has no curve.
    pub fn curve(&self) -> Vec<[f64; 3]> {
        if self.kind == GroupKind::Stubble {
            return Vec::new();
        }
        let p: Vec<[f64; 3]> = self.points.iter().map(|c| [c.i, c.j, c.width_mm]).collect();
        if p.len() < 2 {
            return p;
        }
        let mut out = Vec::new();
        for s in 0..self.segments() {
            for k in 0..PIECES {
                out.push(self.piece_at(s, k as f64 / PIECES as f64));
            }
        }
        out.push(if self.closed() { p[0] } else { p[p.len() - 1] });
        out
    }

    /// The middle of piece `s`, as a column and a row.
    pub fn middle(&self, s: usize) -> [f64; 2] {
        let q = self.piece_at(s, 0.5);
        [q[0], q[1]]
    }

    /// How much a filled area covers the grid point at column `i` and row `j`, from 0 to 1: fully inside its outline or its mirror
    /// image, thinning out over `soft_mm` on each side of the outline.
    pub fn area_cover(sides: &[Vec<[f64; 3]>], i: f64, j: f64, soft_mm: f64) -> f64 {
        let mut best = f64::MAX;
        for c in sides {
            if c.len() < 3 {
                continue;
            }
            let poly: Vec<[f64; 2]> = c.iter().map(|q| [q[0], q[1]]).collect();
            best = best.min(crate::facial_hair::polygon_distance([i, j], &poly) * MM_PER_STEP);
        }
        smooth(soft_mm, -soft_mm, best)
    }

    /// The curve and, when the group is mirrored, its mirror image on a grid of `cols` columns.
    pub fn sides(&self, cols: usize) -> Vec<Vec<[f64; 3]>> {
        let c = self.curve();
        let mut out = vec![c.clone()];
        if self.mirror {
            out.push(c.iter().map(|q| [(cols - 1) as f64 - q[0], q[1], q[2]]).collect());
        }
        out
    }

    /// How much the group covers the grid point at column `i` and row `j`, from 0 to 1, the curve's direction at the nearest place, in
    /// columns and rows per step, and how far the point lies from the curve as a share of half the width there.
    pub fn cover(sides: &[Vec<[f64; 3]>], i: f64, j: f64) -> (f64, [f64; 2], f64) {
        let mut best = (f64::MAX, [0.0, 0.0], 1.0);
        for c in sides {
            if c.len() == 1 {
                let r = (c[0][0] - i).hypot(c[0][1] - j) * MM_PER_STEP;
                let d = r - 0.5 * c[0][2];
                if d < best.0 {
                    best = (d, [0.0, 0.0], r / (0.5 * c[0][2]).max(0.1));
                }
            }
            for w in c.windows(2) {
                let e = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
                let len2 = e[0] * e[0] + e[1] * e[1];
                if len2 < 1e-12 {
                    continue;
                }
                let t = (((i - w[0][0]) * e[0] + (j - w[0][1]) * e[1]) / len2).clamp(0.0, 1.0);
                let q = [w[0][0] + t * e[0], w[0][1] + t * e[1]];
                let r = (q[0] - i).hypot(q[1] - j) * MM_PER_STEP;
                let half = 0.5 * (w[0][2] + t * (w[1][2] - w[0][2]));
                if r - half < best.0 {
                    let l = len2.sqrt();
                    best = (r - half, [e[0] / l, e[1] / l], r / half.max(0.1));
                }
            }
        }
        (smooth(SOFT_MM, -SOFT_MM, best.0), best.1, best.2.min(1.5))
    }
}

impl HairGroups {
    /// Groups from JSON. A group saved before 2026-11-21 has fewer slider values, and reads the missing ones at their defaults.
    pub fn from_json(text: &str) -> Option<Self> {
        serde_json::from_str(text).ok()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("json")
    }

    /// The look of each group for the shader, in the list's order (`HairGroup::look`).
    pub fn looks(&self) -> Vec<[f64; 34]> {
        self.groups.iter().map(HairGroup::look).collect()
    }

    /// Whether a group that paints lets its hairs hang further over the lips than the grid's usual reach.
    pub fn over_lips(&self) -> bool {
        self.groups.iter().any(|g| !g.hidden && !g.pull && g.values[OVER_LIPS] > 0.0)
    }

    /// The shadow in L* of the bottom group that paints, or the slider's default without one.
    pub fn bottom_shadow(&self) -> (Option<usize>, f64) {
        let n = self.groups.iter().rposition(|g| !g.hidden && !g.pull);
        (n, n.map_or(GROUP_SLIDERS[SHADOW].default, |n| self.groups[n].values[SHADOW]))
    }

    /// Writes the groups over a field on `grid`, from the bottom of the list up. The field starts with the length and the look of the
    /// bottom group that paints, and with its own directions, `base_direction`. A covered point takes the hair of the group over it,
    /// thinned into the hair under it at the group's soft edge, and the look `first_look + n` of the group `n`. A density of 1 is
    /// `full_density` hairs per square millimetre, and the shadow is in L*. `stubble` lays a stubble on the grid.
    pub fn write(&self, grid: &FaceGrid, field: &mut HairField, full_density: f64, first_look: u32, base_direction: &[[f64; 3]], stubble: &dyn Fn(&FacialHair) -> StubbleLayer) {
        let painted: Vec<usize> = (0..self.groups.len()).rev().filter(|&n| !self.groups[n].hidden && !self.groups[n].pull).collect();
        if let Some(&n) = painted.first() {
            field.length_mm.iter_mut().for_each(|l| *l = self.groups[n].values[LENGTH]);
            field.look.iter_mut().for_each(|l| *l = first_look + n as u32);
        }
        for n in painted {
            let g = &self.groups[n];
            let v = &g.values;
            let look = first_look + n as u32;
            if g.kind == GroupKind::Stubble {
                let s = stubble(&g.facial_hair());
                let flow = g.flow_field(grid.cols);
                for (k, &own) in base_direction.iter().enumerate() {
                    let cov = s.cover[k];
                    if cov <= 0.0 && s.shadow[k] <= 0.0 {
                        continue;
                    }
                    field.density[k] = s.density[k] + (1.0 - cov) * field.density[k];
                    field.shadow[k] = s.shadow[k] * v[SHADOW] + (1.0 - cov) * field.shadow[k];
                    if cov <= 0.0 {
                        continue;
                    }
                    field.length_mm[k] = v[LENGTH];
                    field.look[k] = look;
                    let droop = v[DROOP];
                    let own = flow.as_ref().map_or(own, |f| grid.along(k, f.at((k % grid.cols) as f64, (k / grid.cols) as f64)));
                    let d = if droop > 0.0 {
                        let down = grid.downhill(k);
                        [0, 1, 2].map(|a| (1.0 - droop) * own[a] + droop * down[a])
                    } else {
                        own
                    };
                    if dot(d, d) > 1e-12 {
                        field.direction[k] = crate::layout_points::normalize(d);
                    }
                }
                continue;
            }
            let sides = g.sides(grid.cols);
            let flow = g.flow_field(grid.cols);
            for (k, p) in grid.points.iter().enumerate() {
                if !p.valid {
                    continue;
                }
                let (i, j) = ((k % grid.cols) as f64, (k / grid.cols) as f64);
                let (cov, t, across) = match g.kind {
                    GroupKind::Stroke => HairGroup::cover(&sides, i, j),
                    _ => (HairGroup::area_cover(&sides, i, j, v[AREA_EDGE]), [0.0, 0.0], 0.0),
                };
                if cov <= 0.0 {
                    continue;
                }
                // The hair thins toward the group's edges and in broad patches of about a centimetre, so that it is not an even mat.
                let patch = crate::noise::fractal_noise(p.position, 9.0, 2, 0.5, 7151 + 101 * v[SEED].round().max(0.0) as u64);
                let share = v[DENSITY] * cov * (1.0 - 0.35 * across.min(1.0).powi(2)) * (1.0 + v[PATCHY] * patch).clamp(0.5, 1.0 + v[PATCHY]);
                field.density[k] = full_density * share + (1.0 - cov) * field.density[k];
                field.shadow[k] = share * (0.4 + 0.6 * share.min(1.0)) * v[SHADOW] + (1.0 - cov) * field.shadow[k];
                field.length_mm[k] = v[LENGTH];
                field.look[k] = look;
                let droop = v[DROOP];
                let own = match (flow.as_ref().map(|f| f.at(i, j)), g.kind) {
                    (Some(f), _) => grid.along(k, f),
                    (None, GroupKind::Stroke) => grid.along(k, t),
                    _ => field.direction[k],
                };
                let down = grid.downhill(k);
                let d = [0, 1, 2].map(|a| (1.0 - droop) * own[a] + droop * down[a]);
                if dot(d, d) > 1e-12 {
                    field.direction[k] = crate::layout_points::normalize(d);
                }
            }
        }
        self.pull(grid, field);
    }

    /// Turns the field's directions under each pull that shows, from the bottom of the list up. A hair under a pull's curve turns toward
    /// the curve's direction by the pull's strength, and the turn fades to nothing at the edge of the pull's width. Under a pull with one
    /// point, the hairs turn toward the point.
    fn pull(&self, grid: &FaceGrid, field: &mut HairField) {
        for g in self.groups.iter().rev().filter(|g| g.pull && !g.hidden && !g.points.is_empty()) {
            let strength = g.values[PULL];
            let sides = g.sides(grid.cols);
            for (k, p) in grid.points.iter().enumerate() {
                if !p.valid {
                    continue;
                }
                let (i, j) = ((k % grid.cols) as f64, (k / grid.cols) as f64);
                let (w, t) = if sides[0].len() == 1 {
                    let Some(c) = sides.iter().map(|c| c[0]).min_by(|a, b| (a[0] - i).hypot(a[1] - j).total_cmp(&(b[0] - i).hypot(b[1] - j))) else { continue };
                    let r = (c[0] - i).hypot(c[1] - j) * MM_PER_STEP;
                    (smooth(0.5 * c[2], 0.0, r), [c[0] - i, c[1] - j])
                } else {
                    let (_, t, across) = HairGroup::cover(&sides, i, j);
                    (smooth(1.0, 0.0, across), t)
                };
                let w = w * strength.abs();
                if w <= 0.0 {
                    continue;
                }
                let to = grid.along(k, t.map(|x| x * strength.signum()));
                let own = field.direction[k];
                let d = [0, 1, 2].map(|a| (1.0 - w) * own[a] + w * to[a]);
                if dot(d, d) > 1e-12 {
                    field.direction[k] = crate::layout_points::normalize(d);
                }
            }
        }
    }
}

/// Editing a group, as the viewer's handles do. Every change keeps at least one control point, and a filled area keeps three.
impl HairGroup {
    /// The column of `i` on the side of the group's own points, when the group is mirrored and `i` lies on the other side.
    fn own_side(&self, cols: usize, i: f64) -> f64 {
        let mid = (cols - 1) as f64 / 2.0;
        let side: f64 = self.points.iter().map(|p| p.i - mid).sum();
        if self.mirror && side * (i - mid) < 0.0 { (cols - 1) as f64 - i } else { i }
    }

    /// Moves control point `k` to a column and row. A point picked on the mirror image moves its own point to the mirrored place.
    pub fn move_point(&mut self, k: usize, i: f64, j: f64, on_mirror: bool, cols: usize) {
        if let Some(p) = self.points.get_mut(k) {
            p.i = if on_mirror { (cols - 1) as f64 - i } else { i };
            p.j = j;
        }
    }

    /// Sets the width of control point `k` in millimetres. Without a point, it scales every point's width so that the widest is `mm`,
    /// which keeps the group's taper. The widths stay within `WIDTH_MM`.
    pub fn set_width(&mut self, k: Option<usize>, mm: f64) {
        let mm = mm.clamp(WIDTH_MM[0], WIDTH_MM[1]);
        match k {
            Some(k) => {
                if let Some(p) = self.points.get_mut(k) {
                    p.width_mm = mm;
                }
            }
            None => {
                let widest = self.points.iter().map(|p| p.width_mm).fold(0.0, f64::max);
                if widest > 0.0 {
                    for p in &mut self.points {
                        p.width_mm = (p.width_mm * mm / widest).clamp(WIDTH_MM[0], WIDTH_MM[1]);
                    }
                }
            }
        }
    }

    /// Bends piece `s` so that its middle lies at a column and row, without moving its control points. A middle picked on the mirror
    /// image bends the piece toward the mirrored place.
    pub fn bend_segment(&mut self, s: usize, i: f64, j: f64, on_mirror: bool, cols: usize) {
        if s >= self.segments() {
            return;
        }
        let target = [if on_mirror { (cols - 1) as f64 - i } else { i }, j];
        let m = self.middle(s);
        let b = &mut self.points[s].bend;
        b[0] += target[0] - m[0];
        b[1] += target[1] - m[1];
    }

    /// Takes the bend out of piece `s`.
    pub fn straighten_segment(&mut self, s: usize) {
        if s < self.segments() {
            self.points[s].bend = [0.0; 2];
        }
    }

    /// Makes control point `k` a corner, where the curve may turn sharply, or a smooth point.
    pub fn set_corner(&mut self, k: usize, corner: bool) {
        if let Some(p) = self.points.get_mut(k) {
            p.corner = corner;
        }
    }

    /// Adds a control point at a column and row, in the curve's order: between the two points of the nearest piece of the curve, or
    /// before the first or after the last point when it lies past an end. Its width is the curve's width there. Returns its number.
    pub fn insert_point(&mut self, i: f64, j: f64, cols: usize) -> usize {
        let i = self.own_side(cols, i);
        let n = self.points.len();
        if n == 0 {
            self.points.push(ControlPoint::new(i, j, 6.0));
            return 0;
        }
        let curve = self.curve();
        let mut best = (f64::MAX, n, self.points[n - 1].width_mm);
        for (q, w) in curve.windows(2).enumerate() {
            let s = q / PIECES;
            let e = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
            let len2 = (e[0] * e[0] + e[1] * e[1]).max(1e-12);
            let t = (((i - w[0][0]) * e[0] + (j - w[0][1]) * e[1]) / len2).clamp(0.0, 1.0);
            let d = (w[0][0] + t * e[0] - i).hypot(w[0][1] + t * e[1] - j);
            let first = q == 0 && t <= 0.0;
            let last = q + 2 == curve.len() && t >= 1.0;
            let at = if self.closed() { s + 1 } else if first { 0 } else if last { n } else { s + 1 };
            if d < best.0 {
                best = (d, at, w[0][2] + t * (w[1][2] - w[0][2]));
            }
        }
        if n == 1 {
            best = (0.0, 1, self.points[0].width_mm);
        }
        // The new point splits a bent piece in two, each keeping half of the bend.
        if best.1 > 0 && best.1 <= n && best.1 - 1 < self.segments() {
            let b = &mut self.points[best.1 - 1].bend;
            *b = b.map(|x| 0.5 * x);
            let half = *b;
            self.points.insert(best.1, ControlPoint { bend: half, ..ControlPoint::new(i, j, best.2) });
        } else {
            self.points.insert(best.1, ControlPoint::new(i, j, best.2));
        }
        best.1
    }

    /// Adds a control point after the last one, or before the first with `at_start`, at a column and row, with the width of the end
    /// it extends. Returns its number.
    pub fn extend(&mut self, i: f64, j: f64, at_start: bool, cols: usize) -> usize {
        let i = self.own_side(cols, i);
        let width = (if at_start { self.points.first() } else { self.points.last() }).map_or(6.0, |p| p.width_mm);
        if at_start {
            self.points.insert(0, ControlPoint::new(i, j, width));
            0
        } else {
            self.points.push(ControlPoint::new(i, j, width));
            self.points.len() - 1
        }
    }

    /// Removes control point `k`, unless it is the last one left, or one of a filled area's last three.
    pub fn remove_point(&mut self, k: usize) {
        let least = if self.kind == GroupKind::Area { 3 } else { 1 };
        if self.points.len() > least && k < self.points.len() {
            self.points.remove(k);
        }
    }

    /// The points in the other order, with each piece's bend kept on the same piece.
    pub fn reverse(&mut self) {
        let n = self.points.len();
        if n < 2 {
            return;
        }
        let closed = self.closed();
        let bends: Vec<[f64; 2]> = self.points.iter().map(|p| p.bend).collect();
        self.points.reverse();
        for k in 0..n {
            // The piece from new point k to k + 1 was the piece from old point n - 2 - k to n - 1 - k.
            let old = if closed { (2 * n - 2 - k) % n } else { n.wrapping_sub(2 + k) };
            self.points[k].bend = if old < n { bends[old] } else { [0.0; 2] };
        }
    }

    /// Closes a stroke into a filled area through the same points. Its two end points become one when they lie on each other. It needs
    /// three points.
    pub fn close(&mut self) -> bool {
        if self.kind != GroupKind::Stroke {
            return false;
        }
        let n = self.points.len();
        if n >= 4 && (self.points[0].i - self.points[n - 1].i).hypot(self.points[0].j - self.points[n - 1].j) < SAME_POINT_STEPS {
            self.points.pop();
        }
        if self.points.len() < 3 {
            return false;
        }
        self.kind = GroupKind::Area;
        true
    }

    /// Opens a filled area into a stroke at piece `s`, which is taken out: the stroke runs from the point after it round to the point
    /// before it. Without `s`, the piece from the last point back to the first is taken out.
    pub fn open(&mut self, s: Option<usize>) -> bool {
        if self.kind != GroupKind::Area {
            return false;
        }
        let n = self.points.len();
        let s = s.unwrap_or(n.saturating_sub(1)).min(n.saturating_sub(1));
        self.points.rotate_left((s + 1) % n.max(1));
        if let Some(last) = self.points.last_mut() {
            last.bend = [0.0; 2];
        }
        self.kind = GroupKind::Stroke;
        true
    }

    /// Splits a stroke into two at control point `k`, which both halves keep, or opens a filled area there into a stroke that starts
    /// and ends at it. Returns the second half of a stroke. A stroke's end point does not split it.
    pub fn split_at_point(&mut self, k: usize) -> Option<HairGroup> {
        let n = self.points.len();
        match self.kind {
            GroupKind::Area if k < n => {
                self.points.rotate_left(k);
                let mut first = self.points[0];
                first.bend = [0.0; 2];
                self.points.push(first);
                self.kind = GroupKind::Stroke;
                None
            }
            GroupKind::Stroke if k > 0 && k + 1 < n => {
                let rest = self.points.split_off(k);
                self.points.push(ControlPoint { bend: [0.0; 2], ..rest[0] });
                Some(HairGroup { name: format!("{} 2", self.name), points: rest, ..self.clone() })
            }
            _ => None,
        }
    }

    /// Splits a stroke into two by taking out piece `s`, or opens a filled area there. Returns the second half of a stroke.
    pub fn split_at_segment(&mut self, s: usize) -> Option<HairGroup> {
        if s >= self.segments() {
            return None;
        }
        match self.kind {
            GroupKind::Area => {
                self.open(Some(s));
                None
            }
            GroupKind::Stroke => {
                let rest = self.points.split_off(s + 1);
                if let Some(last) = self.points.last_mut() {
                    last.bend = [0.0; 2];
                }
                Some(HairGroup { name: format!("{} 2", self.name), points: rest, ..self.clone() })
            }
            GroupKind::Stubble => None,
        }
    }

    /// Joins another stroke onto this one, end to end: this stroke's last point (or its first, with `own_start`) to the other's first
    /// point (or its last, with `other_end`). Two end points that lie on each other become one. The joined stroke keeps this group's
    /// name and sliders.
    pub fn join(&mut self, own_start: bool, mut other: HairGroup, other_end: bool) -> bool {
        if self.kind != GroupKind::Stroke || other.kind != GroupKind::Stroke || self.points.is_empty() || other.points.is_empty() {
            return false;
        }
        if own_start {
            self.reverse();
        }
        if other_end {
            other.reverse();
        }
        let (a, b) = (self.points[self.points.len() - 1], other.points[0]);
        let mut rest = other.points;
        if (a.i - b.i).hypot(a.j - b.j) < SAME_POINT_STEPS {
            let n = self.points.len();
            self.points[n - 1].bend = rest[0].bend;
            rest.remove(0);
        }
        self.points.extend(rest);
        true
    }
}

/// The control points on the skin of `grid`, for a viewer to draw as handles.
impl HairGroup {
    /// For each control point, and for its mirror image when the group is mirrored, nine numbers: its place and its skin's normal in
    /// metres, its number, 1 on the mirror image or 0, and 0 for a smooth point or 1 for a corner. Then the same for the middle of each
    /// piece of curve, with the piece's number and 2 last.
    pub fn handles(&self, grid: &FaceGrid) -> Vec<f32> {
        let mut out = Vec::new();
        let mut put = |i: f64, j: f64, k: usize, mirrored: bool, kind: f32| {
            let i = if mirrored { (grid.cols - 1) as f64 - i } else { i };
            if let Some((at, n)) = grid.place(i, j) {
                out.extend(at.map(|x| (x / 1000.0) as f32));
                out.extend(n.map(|x| x as f32));
                out.extend([k as f32, f32::from(u8::from(mirrored)), kind]);
            }
        };
        for mirrored in [false, true] {
            if mirrored && !self.mirror {
                continue;
            }
            for (k, p) in self.points.iter().enumerate() {
                put(p.i, p.j, k, mirrored, if p.corner { 1.0 } else { 0.0 });
            }
            for s in 0..self.segments() {
                let m = self.middle(s);
                put(m[0], m[1], s, mirrored, 2.0);
            }
        }
        out
    }

    /// The curve on the skin of `grid`, and its mirror image, for a viewer: six numbers per place, its place and its skin's normal in
    /// metres, with a row of six NaN between the two sides and where the curve leaves the known skin.
    pub fn curve_on_skin(&self, grid: &FaceGrid) -> Vec<f32> {
        let mut out = Vec::new();
        for side in self.sides(grid.cols) {
            for q in side {
                match grid.place(q[0], q[1]) {
                    Some((at, n)) => {
                        out.extend(at.map(|x| (x / 1000.0) as f32));
                        out.extend(n.map(|x| x as f32));
                    }
                    None => out.extend([f32::NAN; 6]),
                }
            }
            out.extend([f32::NAN; 6]);
        }
        out
    }

    /// The edges of a stroke's width, and of its mirror image's, as lines of columns and rows on a grid of `cols` columns: the two sides
    /// of the curve at half its width, or a circle round a stroke of one point. Nothing for a filled area or a stubble, whose width
    /// paints nothing.
    pub fn width_edges(&self, cols: usize) -> Vec<Vec<[f64; 2]>> {
        let mut out = Vec::new();
        if self.kind != GroupKind::Stroke {
            return out;
        }
        for c in self.sides(cols) {
            if c.len() == 1 {
                let r = 0.5 * c[0][2] / MM_PER_STEP;
                out.push((0..=48).map(|k| f64::from(k) * std::f64::consts::TAU / 48.0).map(|a| [c[0][0] + r * a.cos(), c[0][1] + r * a.sin()]).collect());
                continue;
            }
            for side in [1.0, -1.0] {
                out.push(
                    (0..c.len())
                        .map(|k| {
                            let (a, b) = (c[k.saturating_sub(1)], c[(k + 1).min(c.len() - 1)]);
                            let e = [b[0] - a[0], b[1] - a[1]];
                            let l = e[0].hypot(e[1]).max(1e-9);
                            let h = side * 0.5 * c[k][2] / MM_PER_STEP;
                            [c[k][0] - h * e[1] / l, c[k][1] + h * e[0] / l]
                        })
                        .collect(),
                );
            }
        }
        out
    }

    /// The edges of the width (`width_edges`) on the skin of `grid`, for a viewer to draw, as rows of six numbers like `curve_on_skin`.
    pub fn widths_on_skin(&self, grid: &FaceGrid) -> Vec<f32> {
        let mut out = Vec::new();
        for line in self.width_edges(grid.cols) {
            for q in line {
                match grid.place(q[0], q[1]) {
                    Some((at, n)) => {
                        out.extend(at.map(|x| (x / 1000.0) as f32));
                        out.extend(n.map(|x| x as f32));
                    }
                    None => out.extend([f32::NAN; 6]),
                }
            }
            out.extend([f32::NAN; 6]);
        }
        out
    }

    /// The two end points of a stroke on the skin of `grid`, and their mirror images, for a viewer to offer for joining: for each, its
    /// place and its skin's normal in metres, 0 for the first point or 1 for the last, and 1 on the mirror image or 0. Nothing for a
    /// filled area, a stubble or a stroke of one point.
    pub fn ends_on_skin(&self, grid: &FaceGrid) -> Vec<f32> {
        let mut out = Vec::new();
        if self.kind != GroupKind::Stroke || self.points.len() < 2 {
            return out;
        }
        for mirrored in [false, true] {
            if mirrored && !self.mirror {
                continue;
            }
            for (end, p) in [(0.0, self.points[0]), (1.0, self.points[self.points.len() - 1])] {
                let i = if mirrored { (grid.cols - 1) as f64 - p.i } else { p.i };
                if let Some((at, n)) = grid.place(i, p.j) {
                    out.extend(at.map(|x| (x / 1000.0) as f32));
                    out.extend(n.map(|x| x as f32));
                    out.extend([end, f32::from(u8::from(mirrored))]);
                }
            }
        }
        out
    }
}

/// Editing the list of groups. The top group is number 0.
impl HairGroups {
    /// Adds a group at the top of the list. Returns its number, 0.
    pub fn add(&mut self, g: HairGroup) -> usize {
        self.groups.insert(0, g);
        0
    }

    /// The groups of a preset by its id in `PRESETS`, the top one first: one group, or several for a preset listed under `sets` in
    /// `data/hair_presets.json`.
    pub fn preset(id: &str) -> Option<Vec<HairGroup>> {
        let label = PRESETS.iter().find(|p| p.0 == id)?.1;
        match preset_set(label) {
            Some(names) => {
                let all = HairGroups::from_json(PRESET_GROUPS)?.groups;
                names.iter().map(|n| all.iter().find(|g| &g.name == n).cloned()).collect()
            }
            None => HairGroup::preset(id).map(|g| vec![g]),
        }
    }

    /// Adds the groups of a preset at the top of the list, in the preset's order. Returns false when there is no such preset.
    pub fn add_preset(&mut self, id: &str) -> bool {
        let Some(gs) = HairGroups::preset(id) else { return false };
        self.groups.splice(0..0, gs);
        true
    }

    /// A new empty group: a stroke, or a filled area with `area`. Its points are added by clicking on the head.
    pub fn add_empty(&mut self, area: bool) -> usize {
        let kind = if area { GroupKind::Area } else { GroupKind::Stroke };
        let n = self.groups.iter().filter(|g| g.kind == kind).count() + 1;
        let name = if area { format!("Area {n}") } else { format!("Stroke {n}") };
        self.add(HairGroup { name, kind, points: Vec::new(), mirror: true, values: default_values(), hidden: false, pull: false, flow: Vec::new() })
    }

    /// Adds an empty pull at the top of the list. Its points are added by clicking on the head. Returns its number.
    pub fn add_pull(&mut self) -> usize {
        let n = self.groups.iter().filter(|g| g.pull).count() + 1;
        let mut values = default_values();
        values[DROOP] = 0.0;
        self.add(HairGroup { name: format!("Pull {n}"), kind: GroupKind::Stroke, points: Vec::new(), mirror: true, values, hidden: false, pull: true, flow: Vec::new() })
    }

    /// Copies group `n` just above itself. Returns the copy's number.
    pub fn duplicate(&mut self, n: usize) -> Option<usize> {
        let mut g = self.groups.get(n)?.clone();
        g.name = format!("{} copy", g.name);
        self.groups.insert(n, g);
        Some(n)
    }

    pub fn remove(&mut self, n: usize) {
        if n < self.groups.len() {
            self.groups.remove(n);
        }
    }

    /// Moves group `n` one place up the list, toward the top, or down. Returns its new number.
    pub fn shift(&mut self, n: usize, up: bool) -> usize {
        if n >= self.groups.len() {
            return n;
        }
        let to = if up { n.saturating_sub(1) } else { (n + 1).min(self.groups.len() - 1) };
        self.groups.swap(n, to);
        to
    }

    /// Splits group `n` at its control point `k`, or at its piece `k` with `segment`. A stroke's second half goes just under it in the
    /// list. Returns whether anything changed.
    pub fn split(&mut self, n: usize, k: usize, segment: bool) -> bool {
        let Some(g) = self.groups.get_mut(n) else { return false };
        let before = g.clone();
        let rest = if segment { g.split_at_segment(k) } else { g.split_at_point(k) };
        let changed = rest.is_some() || *g != before;
        if let Some(r) = rest {
            self.groups.insert(n + 1, r);
        }
        changed
    }

    /// Joins stroke `b` onto stroke `a`, end to end (`HairGroup::join`), and removes `b`. Joining a stroke's two ends to each other
    /// closes it into a filled area. Returns the joined group's number, or `None` when nothing was joined.
    pub fn join(&mut self, a: usize, a_start: bool, b: usize, b_end: bool) -> Option<usize> {
        if a >= self.groups.len() || b >= self.groups.len() {
            return None;
        }
        if a == b {
            return (a_start == b_end && self.groups[a].close()).then_some(a);
        }
        let other = self.groups[b].clone();
        if !self.groups[a].join(a_start, other, b_end) {
            return None;
        }
        self.groups.remove(b);
        Some(if b < a { a - 1 } else { a })
    }
}

/// The steps of undo and redo for a list of hair groups. A step is the whole list as it was before one edit.
#[derive(Clone, Debug, Default)]
pub struct History {
    past: Vec<HairGroups>,
    future: Vec<HairGroups>,
}

/// At most this many steps of undo are kept.
const HISTORY_STEPS: usize = 200;

impl History {
    /// Keeps the groups as they are before an edit. A step that is the same as the last one kept is not kept twice.
    pub fn checkpoint(&mut self, now: &HairGroups) {
        if self.past.last() != Some(now) {
            self.past.push(now.clone());
            if self.past.len() > HISTORY_STEPS {
                self.past.remove(0);
            }
        }
        self.future.clear();
    }

    /// Goes back one step. Returns false when there is none.
    pub fn undo(&mut self, now: &mut HairGroups) -> bool {
        while let Some(before) = self.past.pop() {
            if before != *now {
                self.future.push(std::mem::replace(now, before));
                return true;
            }
        }
        false
    }

    /// Goes forward one step that was undone. Returns false when there is none.
    pub fn redo(&mut self, now: &mut HairGroups) -> bool {
        let Some(after) = self.future.pop() else { return false };
        self.past.push(std::mem::replace(now, after));
        true
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The paint grid's size and the moustache preset's first point, the middle of the upper lip.
    fn grid() -> (usize, f64, f64) {
        let b = crate::face_grid::FaceBinding::baked();
        let g = HairGroup::preset("moustache").expect("preset");
        (b.cols, g.points[0].i, g.points[0].j)
    }

    fn square(kind: GroupKind) -> HairGroup {
        HairGroup { name: "Patch".into(), kind, points: [(10.0, 10.0), (30.0, 10.0), (30.0, 30.0), (10.0, 30.0)].map(|(i, j)| ControlPoint::new(i, j, 4.0)).to_vec(), mirror: true, values: default_values(), hidden: false, pull: false, flow: Vec::new() }
    }

    /// The Catmull-Rom spline the curve was before bends and corners, written out as it was.
    fn catmull_rom(g: &HairGroup) -> Vec<[f64; 3]> {
        let p: Vec<[f64; 3]> = g.points.iter().map(|c| [c.i, c.j, c.width_mm]).collect();
        let closed = g.kind == GroupKind::Area && p.len() >= 3;
        let n = p.len() as i64;
        let at = |k: i64| if closed { p[k.rem_euclid(n) as usize] } else { p[k.clamp(0, n - 1) as usize] };
        let pieces = if closed { p.len() } else { p.len() - 1 };
        let mut out = Vec::new();
        for s in 0..pieces as i64 {
            let (a, b, c, d) = (at(s - 1), at(s), at(s + 1), at(s + 2));
            for k in 0..PIECES {
                let t = k as f64 / PIECES as f64;
                let (t2, t3) = (t * t, t * t * t);
                let q = [0, 1].map(|x| 0.5 * (2.0 * b[x] + (c[x] - a[x]) * t + (2.0 * a[x] - 5.0 * b[x] + 4.0 * c[x] - d[x]) * t2 + (3.0 * b[x] - a[x] - 3.0 * c[x] + d[x]) * t3));
                out.push([q[0], q[1], b[2] + t * (c[2] - b[2])]);
            }
        }
        out.push(if closed { p[0] } else { p[p.len() - 1] });
        out
    }

    #[test]
    fn without_bends_the_curve_is_the_old_spline() {
        for g in [HairGroup::preset("moustache").expect("preset"), HairGroup::preset("buzz_cut").expect("preset"), square(GroupKind::Area)] {
            let (a, b) = (g.curve(), catmull_rom(&g));
            assert_eq!(a.len(), b.len());
            for (p, q) in a.iter().zip(&b) {
                assert!((p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9 && (p[2] - q[2]).abs() < 1e-12, "{p:?} {q:?}");
            }
        }
    }

    #[test]
    fn the_curve_passes_through_its_points() {
        let g = HairGroup::preset("moustache").expect("preset");
        let c = g.curve();
        for p in &g.points {
            assert!(c.iter().any(|q| (q[0] - p.i).abs() < 1e-9 && (q[1] - p.j).abs() < 1e-9 && (q[2] - p.width_mm).abs() < 1e-9));
        }
    }

    #[test]
    fn a_bend_moves_the_middle_to_the_spot_and_keeps_the_points() {
        let (cols, ..) = grid();
        for kind in [GroupKind::Stroke, GroupKind::Area] {
            let mut g = square(kind);
            g.set_corner(2, kind == GroupKind::Area);
            let m = g.middle(1);
            g.bend_segment(1, m[0] + 3.0, m[1] - 1.0, false, cols);
            let after = g.middle(1);
            assert!((after[0] - m[0] - 3.0).abs() < 1e-9 && (after[1] - m[1] + 1.0).abs() < 1e-9, "{after:?}");
            let c = g.curve();
            for p in &g.points {
                assert!(c.iter().any(|q| (q[0] - p.i).abs() < 1e-9 && (q[1] - p.j).abs() < 1e-9));
            }
            g.straighten_segment(1);
            assert_eq!(g.points[1].bend, [0.0; 2]);
        }
    }

    #[test]
    fn a_bend_on_the_mirror_image_bends_the_own_side() {
        let (cols, ..) = grid();
        let mut g = square(GroupKind::Stroke);
        let m = g.middle(0);
        g.bend_segment(0, (cols - 1) as f64 - m[0], m[1] + 2.0, true, cols);
        let after = g.middle(0);
        assert!((after[0] - m[0]).abs() < 1e-9 && (after[1] - m[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn a_smooth_point_has_no_corner_and_a_corner_point_may() {
        let mut g = square(GroupKind::Stroke);
        for (k, p) in g.points.iter_mut().enumerate() {
            (p.i, p.j) = (10.0 + 10.0 * k as f64, 10.0);
        }
        let m = g.middle(0);
        g.bend_segment(0, m[0], m[1] + 4.0, false, 200);
        let turn = |g: &HairGroup| {
            let c = g.curve();
            let k = PIECES;
            let a = [c[k][0] - c[k - 1][0], c[k][1] - c[k - 1][1]];
            let b = [c[k + 1][0] - c[k][0], c[k + 1][1] - c[k][1]];
            (a[0] * b[1] - a[1] * b[0]).atan2(a[0] * b[0] + a[1] * b[1]).abs()
        };
        assert!(turn(&g) < 0.15, "{}", turn(&g));
        g.set_corner(1, true);
        assert!(turn(&g) > 0.5, "{}", turn(&g));
    }

    #[test]
    fn a_mirrored_group_covers_both_sides_alike() {
        let (cols, mi, mj) = grid();
        let g = HairGroup::preset("moustache").expect("preset");
        let sides = g.sides(cols);
        for (di, dj) in [(-5.0, -2.0), (-9.0, -5.0), (-1.0, 0.0)] {
            let (i, j) = (mi + di, mj + dj);
            let (a, ta, _) = HairGroup::cover(&sides, i, j);
            let (b, tb, _) = HairGroup::cover(&sides, (cols - 1) as f64 - i, j);
            assert!(a > 0.5 && (a - b).abs() < 1e-9, "{a} {b}");
            assert!((ta[0] + tb[0]).abs() < 1e-9 && (ta[1] - tb[1]).abs() < 1e-9);
        }
        assert_eq!(HairGroup::cover(&sides, mi, mj - 20.0).0, 0.0);
    }

    #[test]
    fn a_point_is_inserted_between_its_neighbours() {
        let (cols, mi, mj) = grid();
        let mut g = HairGroup::preset("moustache").expect("preset");
        let k = g.insert_point(mi - 2.5, mj - 0.5, cols);
        assert_eq!(k, 1);
        assert_eq!(g.points.len(), 5);
        let k = g.insert_point((cols - 1) as f64 - (mi - 17.0), mj - 7.0, cols);
        assert_eq!((k, g.points[k].i), (5, mi - 17.0));
        g.remove_point(0);
        assert_eq!(g.points.len(), 5);
    }

    #[test]
    fn a_filled_area_covers_its_inside_and_its_mirror() {
        let (cols, ..) = grid();
        let g = square(GroupKind::Area);
        let sides = g.sides(cols);
        assert_eq!(HairGroup::area_cover(&sides, 20.0, 20.0, 1.5), 1.0);
        assert_eq!(HairGroup::area_cover(&sides, (cols - 1) as f64 - 20.0, 20.0, 1.5), 1.0);
        assert_eq!(HairGroup::area_cover(&sides, 40.0, 20.0, 1.5), 0.0);
        let mut h = g.clone();
        assert_eq!(h.insert_point(20.0, 9.0, cols), 1);
        assert_eq!(h.insert_point(9.0, 20.0, cols), 5);
        h.points.truncate(3);
        h.remove_point(0);
        assert_eq!(h.points.len(), 3);
    }

    #[test]
    fn a_stroke_splits_at_a_point_and_joins_back() {
        let mut gs = HairGroups { groups: vec![square(GroupKind::Stroke)] };
        let whole = gs.groups[0].clone();
        assert!(!gs.split(0, 0, false), "an end point does not split");
        assert!(gs.split(0, 1, false));
        assert_eq!(gs.groups.len(), 2);
        assert_eq!(gs.groups[0].points.len(), 2);
        assert_eq!(gs.groups[1].points.len(), 3);
        assert_eq!(gs.join(0, false, 1, false), Some(0));
        assert_eq!(gs.groups.len(), 1);
        assert_eq!(gs.groups[0].points, whole.points);
    }

    #[test]
    fn a_stroke_splits_at_a_piece_and_joins_reversed() {
        let mut gs = HairGroups { groups: vec![square(GroupKind::Stroke)] };
        gs.groups[0].points[1].bend = [0.0, 2.0];
        assert!(gs.split(0, 1, true));
        assert_eq!((gs.groups[0].points.len(), gs.groups[1].points.len()), (2, 2));
        assert_eq!(gs.groups[0].points[1].bend, [0.0; 2]);
        // The second half's last point joins the first half's first point.
        assert_eq!(gs.join(0, true, 1, true), Some(0));
        let p: Vec<(f64, f64)> = gs.groups[0].points.iter().map(|p| (p.i, p.j)).collect();
        assert_eq!(p, vec![(30.0, 10.0), (10.0, 10.0), (10.0, 30.0), (30.0, 30.0)]);
    }

    #[test]
    fn a_stroke_closes_into_an_area_and_opens_back() {
        let mut g = square(GroupKind::Stroke);
        g.points[3].bend = [1.0, 0.0];
        assert!(g.close());
        assert_eq!((g.kind, g.segments()), (GroupKind::Area, 4));
        assert_eq!(g.curve().len(), 4 * PIECES + 1);
        assert!(g.open(Some(1)));
        assert_eq!(g.kind, GroupKind::Stroke);
        let p: Vec<(f64, f64)> = g.points.iter().map(|p| (p.i, p.j)).collect();
        assert_eq!(p, vec![(30.0, 30.0), (10.0, 30.0), (10.0, 10.0), (30.0, 10.0)]);
        assert_eq!(g.points[1].bend, [1.0, 0.0]);
        let mut a = square(GroupKind::Area);
        assert_eq!(a.split_at_point(2), None);
        let p: Vec<(f64, f64)> = a.points.iter().map(|p| (p.i, p.j)).collect();
        assert_eq!(p, vec![(30.0, 30.0), (10.0, 30.0), (10.0, 10.0), (30.0, 10.0), (30.0, 30.0)]);
        let mut gs = HairGroups { groups: vec![a] };
        assert_eq!(gs.join(0, true, 0, true), Some(0));
        assert_eq!((gs.groups[0].kind, gs.groups[0].points.len()), (GroupKind::Area, 4));
    }

    #[test]
    fn reversing_keeps_each_bend_on_its_piece() {
        for kind in [GroupKind::Stroke, GroupKind::Area] {
            let mut g = square(kind);
            g.points[0].bend = [1.0, 0.0];
            g.points[2].bend = [0.0, 2.0];
            let before = g.curve();
            g.reverse();
            let mut after = g.curve();
            after.reverse();
            if kind == GroupKind::Area {
                after.pop();
                after.rotate_left(PIECES);
                after.push(after[0]);
            }
            for (p, q) in before.iter().zip(&after) {
                assert!((p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9, "{kind:?} {p:?} {q:?}");
            }
        }
    }

    #[test]
    fn the_list_adds_moves_copies_and_removes() {
        let mut gs = HairGroups::default();
        gs.add(HairGroup::preset("stubble").expect("preset"));
        assert_eq!(gs.add_empty(false), 0);
        assert_eq!(gs.groups[0].name, "Stroke 1");
        assert_eq!(gs.duplicate(1), Some(1));
        assert_eq!(gs.groups[1].name, "Stubble copy");
        assert_eq!(gs.shift(2, true), 1);
        assert_eq!(gs.groups[1].name, "Stubble");
        gs.remove(0);
        assert_eq!(gs.groups.len(), 2);
    }

    #[test]
    fn undo_and_redo_step_through_the_edits() {
        let mut gs = HairGroups::default();
        let mut h = History::default();
        h.checkpoint(&gs);
        gs.add(HairGroup::preset("moustache").expect("preset"));
        h.checkpoint(&gs);
        gs.groups[0].points[0].i += 1.0;
        let moved = gs.clone();
        assert!(h.undo(&mut gs));
        assert_eq!(gs.groups[0], HairGroup::preset("moustache").expect("preset"));
        assert!(h.undo(&mut gs));
        assert!(gs.groups.is_empty());
        assert!(!h.undo(&mut gs));
        assert!(h.redo(&mut gs) && h.redo(&mut gs));
        assert_eq!(gs, moved);
    }

    #[test]
    fn every_preset_is_baked_and_reads_back_from_json() {
        for (id, label) in PRESETS {
            let gs = HairGroups::preset(id).unwrap_or_else(|| panic!("{id}"));
            if gs.len() == 1 {
                assert_eq!(gs[0].name, label);
            }
            assert!(gs.iter().all(|g| g.values.len() == GROUP_SLIDERS.len()));
            assert_eq!(HairGroup::preset(id).is_some(), gs.len() == 1);
        }
        let gs = HairGroups { groups: vec![HairGroup::preset("buzz_cut").expect("preset"), HairGroup::preset("stubble").expect("preset")] };
        assert_eq!(HairGroups::from_json(&gs.to_json()), Some(gs));
        let old = r#"{"groups":[{"name":"Moustache","points":[{"i":1.0,"j":2.0,"width_mm":3.0}],"mirror":true,"values":[3.0,0.9,0.12,0.15,20.0,55.0,14.0,0.0,0.5]}]}"#;
        let g = &HairGroups::from_json(old).expect("old file").groups[0];
        assert_eq!(g.kind, GroupKind::Stroke);
        assert_eq!(g.values[..9], [3.0, 0.9, 0.12, 0.15, 20.0, 55.0, 14.0, 0.0, 0.5]);
        assert_eq!(g.values[9..], default_values()[9..]);
    }

    #[test]
    fn a_stubble_keeps_the_old_sliders() {
        let fh = FacialHair { amount: 0.7, length_mm: 2.0, cheek_line_mm: -3.0, density: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7], sheen: 0.5, ..FacialHair::DEFAULT };
        assert_eq!(HairGroup::stubble("S", &fh).facial_hair(), fh);
        let old = |id: &str| match id {
            "beard_amount" => Some(1.0),
            "beard_cheeks" => Some(0.15),
            _ => None,
        };
        let g = HairGroup::stubble_from_old_sliders(old).expect("a stubble");
        assert_eq!(g.facial_hair(), FacialHair { amount: 1.0, density: [0.85, 0.6, 0.85, 0.75, 0.15, 0.7, 0.4], ..FacialHair::DEFAULT });
        assert!(HairGroup::stubble_from_old_sliders(|_| None).is_none());
    }

    #[test]
    fn the_full_beard_is_split_into_areas_with_their_own_flow_lines() {
        let gs = HairGroups::preset("full_beard").expect("preset");
        let names: Vec<&str> = gs.iter().map(|g| g.name.as_str()).collect();
        assert_eq!(names, ["Hanging moustache", "Bare skin beside the soul patch", "Beard moustache", "Under the lip", "Chin", "Jaw", "Cheeks"]);
        assert!(gs.iter().all(|g| !g.pull && g.mirror));
        assert_eq!(gs[0].kind, GroupKind::Stroke);
        let (patch, beard) = (&gs[1], &gs[6]);
        assert_eq!((patch.kind, patch.values[DENSITY]), (GroupKind::Area, 0.0));
        assert!(gs[2..].iter().all(|g| g.kind == GroupKind::Area && !g.flow.is_empty()));
        // The chin's two halves meet in a mirrored seam: below the soul patch, both sides grow down and toward the middle.
        let cols = grid().0;
        let mid = (cols - 1) as f64 / 2.0;
        let (a, b) = (gs[4].flow_direction(cols, mid + 6.0, 30.0).expect("a direction"), gs[4].flow_direction(cols, mid - 6.0, 30.0).expect("a direction"));
        assert!(a[0] < -0.3 && b[0] > 0.3 && a[1] < -0.3 && b[1] < -0.3, "{a:?} {b:?}");
        // The cheeks grow down and back, toward the ear.
        let c = gs[6].flow_direction(cols, mid + 30.0, 48.0).expect("a direction");
        assert!(c[0] > 0.3 && c[1] < -0.3, "{c:?}");
        // The bare patches lie inside the beard's outline, so that the beard is bare there.
        let sides = beard.sides(grid().0);
        for p in &patch.points {
            assert_eq!(HairGroup::area_cover(&sides, p.i, p.j, beard.values[AREA_EDGE]), 1.0);
        }
        let mut list = HairGroups { groups: vec![HairGroup::preset("moustache").expect("preset")] };
        assert!(list.add_preset("full_beard"));
        assert_eq!(list.groups[..7], gs[..]);
        assert_eq!(list.groups[7].name, "Moustache");
        assert!(!list.add_preset("no such preset"));
    }

    #[test]
    fn a_filled_area_edge_keeps_1_5_mm_unless_set() {
        let old = r#"{"groups":[{"name":"Patch","kind":"area","points":[],"values":[3.0,0.9,0.12,0.15,20.0,55.0,14.0,0.0,0.5]}]}"#;
        assert_eq!(HairGroups::from_json(old).expect("old file").groups[0].values[AREA_EDGE], 1.5);
        let g = square(GroupKind::Area);
        let sides = g.sides(grid().0);
        // Across the outline, a feathered edge has more hair than a sharp one outside and less inside, and some hair where the sharp
        // edge has none.
        let across: Vec<(f64, f64)> = (0..=40).map(|k| 2.0 + 0.25 * f64::from(k)).map(|i| (HairGroup::area_cover(&sides, i, 20.0, 1.5), HairGroup::area_cover(&sides, i, 20.0, 6.0))).collect();
        assert!(across.iter().all(|(sharp, soft)| if *sharp < 0.5 { soft >= sharp } else { soft <= sharp }));
        assert!(across.iter().any(|(sharp, soft)| *sharp == 0.0 && *soft > 0.2));
        assert_eq!(HairGroup::area_cover(&sides, 20.0, 20.0, 6.0), 1.0);
    }

    #[test]
    fn a_point_width_is_set_alone_or_all_widths_scale_together() {
        let mut g = HairGroup::preset("moustache").expect("preset");
        let before: Vec<f64> = g.points.iter().map(|p| p.width_mm).collect();
        g.set_width(Some(1), 25.0);
        assert_eq!(g.points[1].width_mm, 25.0);
        assert_eq!(g.points[0].width_mm, before[0]);
        g.set_width(Some(2), 500.0);
        assert_eq!(g.points[2].width_mm, WIDTH_MM[1]);
        g.set_width(Some(9), 5.0);
        let mut g = HairGroup::preset("moustache").expect("preset");
        g.set_width(None, 2.0 * before[0]);
        for (p, w) in g.points.iter().zip(&before) {
            assert!((p.width_mm - 2.0 * w).abs() < 1e-9);
        }
    }

    #[test]
    fn the_width_edges_lie_half_the_width_from_the_curve() {
        let cols = grid().0;
        let mut g = HairGroup { name: "Line".into(), kind: GroupKind::Stroke, points: vec![ControlPoint::new(50.0, 40.0, 8.0), ControlPoint::new(70.0, 40.0, 8.0)], mirror: true, values: default_values(), hidden: false, pull: false, flow: Vec::new() };
        let edges = g.width_edges(cols);
        assert_eq!(edges.len(), 4);
        for q in edges[0].iter().chain(&edges[1]) {
            assert!(((q[1] - 40.0).abs() - 4.0 / MM_PER_STEP).abs() < 1e-9);
        }
        g.points.truncate(1);
        let circle = &g.width_edges(cols)[0];
        assert!(circle.iter().all(|q| ((q[0] - 50.0).hypot(q[1] - 40.0) - 4.0 / MM_PER_STEP).abs() < 1e-9));
        g.kind = GroupKind::Area;
        assert!(g.width_edges(cols).is_empty());
    }

    #[test]
    fn flow_lines_set_the_direction_and_blend_between_them() {
        let cols = grid().0;
        let mut g = square(GroupKind::Area);
        assert_eq!(g.flow_direction(cols, 20.0, 20.0), None);
        // A line of one point has no direction yet.
        let n = g.add_flow();
        g.edit_flow(n, |s| s.extend(15.0, 12.0, false, cols));
        assert_eq!(g.flow_direction(cols, 20.0, 20.0), None);
        // A line running down the rows: every point of the group takes its direction, near it and far from it.
        g.edit_flow(n, |s| s.extend(15.0, 28.0, false, cols));
        for (i, j) in [(15.0, 20.0), (28.0, 11.0), (12.0, 29.0)] {
            let d = g.flow_direction(cols, i, j).expect("a direction");
            assert!(d[0].abs() < 1e-9 && (d[1] - 1.0).abs() < 1e-9, "{d:?}");
        }
        // A second line across the columns, further on: each line wins near itself, and the direction turns smoothly between them.
        let m = g.add_flow();
        g.edit_flow(m, |s| {
            s.extend(26.0, 10.0, false, cols);
            s.extend(26.0, 30.0, false, cols);
            s.points[0].i = 22.0;
            s.points[1].i = 30.0;
            s.points[1].j = 10.0;
        });
        let near_first = g.flow_direction(cols, 15.5, 20.0).expect("a direction");
        let near_second = g.flow_direction(cols, 26.0, 10.5).expect("a direction");
        assert!(near_first[1] > 0.95 && near_second[0] > 0.95, "{near_first:?} {near_second:?}");
        let mut last = f64::MAX;
        for k in 0..=10 {
            let i = 15.0 + 1.1 * f64::from(k);
            let d = g.flow_direction(cols, i, 10.0 + 0.5 * f64::from(10 - k)).expect("a direction");
            let angle = d[1].atan2(d[0]);
            assert!(angle <= last + 1e-9, "the turn goes back at {i}");
            last = angle;
        }
        // The mirror image of a mirrored group flows the mirrored way, so the two halves meet in a seam down the middle.
        let d = g.flow_direction(cols, 26.0, 10.5).expect("a direction");
        let e = g.flow_direction(cols, (cols - 1) as f64 - 26.0, 10.5).expect("a direction");
        assert!((d[0] + e[0]).abs() < 1e-9 && (d[1] - e[1]).abs() < 1e-9);
        g.mirror = false;
        let e = g.flow_direction(cols, (cols - 1) as f64 - 26.0, 10.5).expect("a direction");
        assert!(e[0] > 0.5, "an unmirrored group flows the same way on both sides: {e:?}");
        // Editing a line keeps its points, and removing a line removes its direction.
        g.edit_flow(m, |s| s.move_point(0, 21.0, 9.0, false, cols));
        assert_eq!((g.flow[m].points[0].i, g.flow[m].points[0].j), (21.0, 9.0));
        assert_eq!(g.edit_flow(5, |_| ()), None);
        g.remove_flow(m);
        g.remove_flow(n);
        assert_eq!(g.flow_direction(cols, 20.0, 20.0), None);
    }

    #[test]
    fn a_group_without_flow_lines_reads_and_saves_as_before() {
        let old = r#"{"groups":[{"name":"Moustache","points":[{"i":1.0,"j":2.0,"width_mm":3.0}],"mirror":true,"values":[3.0,0.9,0.12,0.15,20.0,55.0,14.0,0.0,0.5]}]}"#;
        let gs = HairGroups::from_json(old).expect("an old file");
        assert!(gs.groups[0].flow.is_empty());
        assert!(!gs.to_json().contains("flow\""));
        let mut g = square(GroupKind::Area);
        g.add_flow();
        g.edit_flow(0, |s| s.extend(12.0, 12.0, false, 200));
        let back = HairGroups::from_json(&HairGroups { groups: vec![g.clone()] }.to_json()).expect("json");
        assert_eq!(back.groups[0], g);
    }

    #[test]
    fn locks_flow_and_strands_are_off_by_default_and_on_in_the_full_beard() {
        let (size, gather, light) = (at("lock_mm"), at("lock_gather"), at("lock_light"));
        let strands = [at("strand_mm"), at("straight"), at("strand_shade"), at("bands"), at("root_light"), at("layering"), at("packing"), at("lock_tone"), at("parallel"), at("fine_lines"), at("feather"), at("over_lips"), at("clump_mm"), at("clump_wave"), at("light_zones"), at("edge_mix"), at("comb"), at("streak_sharp")];
        assert!([size, gather, light, at("flow"), at("lock_wave"), at("lock_soft")].iter().chain(&strands).all(|&k| GROUP_SLIDERS[k].default == 0.0));
        let old = r#"{"groups":[{"name":"Moustache","points":[{"i":1.0,"j":2.0,"width_mm":3.0}],"mirror":true,"values":[3.0,0.9,0.12,0.15,20.0,55.0,14.0,0.0,0.5,6.0,0.3,0.0,0.4,0.2,0.85,0.6,0.85,0.75,0.55,0.7,0.4,0.0,0.0,0.0,4.0,0.6,1.5]}]}"#;
        let g = &HairGroups::from_json(old).expect("a file from round 12").groups[0];
        assert_eq!(g.look()[9..31], [0.0; 22]);
        assert_eq!(g.look()[31..], [0.0, 18.0, 0.0]);
        let beard = HairGroups::preset("full_beard").expect("the full beard");
        let cheeks = beard.iter().find(|g| g.name == "Cheeks").expect("the beard's cheeks");
        assert_eq!(cheeks.look()[9..19], [10.0, 0.25, 0.3, 1.0, 0.0, 0.0, 1.2, 1.0, 0.6, 1.0]);
        assert_eq!(cheeks.look()[21..], [1.0, 0.5, 1.0, 0.5, 1.0, 0.0, 3.0, 0.4, 1.0, 1.0, 0.7, 11.0, 0.7]);
        assert!(HairGroups { groups: beard.clone() }.over_lips());
        assert!(!HairGroups::from_json(old).expect("a file from round 12").over_lips());
    }
}
