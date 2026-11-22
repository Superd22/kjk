//! What every feature supplies to the slider kit, and what the kit does the same way for every feature.
//!
//! A feature (the nose, the jaw and chin, and those after them) supplies only what is its own:
//! - its measures: proportions (where it sits and how big it is) and outlines (shapes compared after position and size are taken out),
//!   computed on any triangle mesh in the frame of the eyes, KOTOR's own or ours;
//! - its zone, and the parts of its zone that its sliders may move;
//! - its sliders' names, hints and groups, as `Spec`s;
//! - the window of the face its measure reads, and where its bumps go.
//!
//! The kit does the rest: the slider maker, the fault checks, the optimiser, the needs, the scorecard, the leak table, the samples, the
//! clay pictures, the browser bindings and the page's data.

use crate::V3;
use crate::measure::{Proportion, Surface, Window};
use crate::mesh::HeadMesh;
use crate::zone::Zone;
use serde::Serialize;

/// An outline of a feature. Its points are compared after their position and size are taken out. When `widths` is set, its points are
/// (half width, share of the height) at fixed shares, and only the widths are compared, each divided by the mean width.
pub struct OutlineDef {
    pub key: &'static str,
    pub label: &'static str,
    pub points: usize,
    pub widths: bool,
}

/// One named slider of a feature: the measure it sets, its name and hint for a player, and the parts it may move.
pub struct Spec {
    /// The measure it sets, a key of the feature's proportions.
    pub measure: &'static str,
    pub name: &'static str,
    pub hint: &'static str,
    /// The group the page lists it under.
    pub group: &'static str,
    /// The parts whose bumps it moves.
    pub parts: &'static [&'static str],
    /// More parts it may move where the strict slider cannot go further.
    pub repair_parts: &'static [&'static str],
    /// Measures and outlines it may change freely, because they change with its own.
    pub free: &'static [&'static str],
    /// Measures and outlines it holds at a fifth of the usual strength.
    pub loose: &'static [&'static str],
}

/// A feature measured on one surface. A proportion that cannot be read on this surface (for example a jaw corner under hair) is NaN,
/// and such an outline is empty. Those rows are left out of every score.
#[derive(Clone, Debug, Serialize)]
pub struct Measured {
    pub proportions: Vec<f64>,
    pub outlines: Vec<Vec<[f64; 2]>>,
    /// Landmarks, for drawing.
    pub marks: serde_json::Value,
    /// Cuts through the surface, for drawing.
    pub cuts: Vec<Vec<[f64; 2]>>,
}

impl Measured {
    /// Whether row `r` (proportions, then outlines) could be read.
    pub fn readable(&self, r: usize) -> bool {
        let n = self.proportions.len();
        if r < n { !self.proportions[r].is_nan() } else { !self.outlines[r - n].is_empty() }
    }
}

/// What the KOTOR head's own mesh offers besides its triangles: which vertices are skin (not hair, not an open edge), and the colour of
/// its texture under each triangle.
pub struct KotorExtras<'a> {
    /// The test head's name.
    pub name: Option<&'a str>,
    /// KOTOR's own triangles, so a feature can leave some out.
    pub triangles: Option<&'a [[u32; 3]]>,
    /// The triangles of our head, for the fit's normals.
    pub fit_triangles: Option<&'a [[u32; 3]]>,
    pub skin: Option<&'a [bool]>,
    pub colour: Option<&'a [V3]>,
    /// The fit of our head to this KOTOR head (smoothed), which may only be used to locate a region, never to measure it.
    pub fit: Option<&'a [V3]>,
}

/// Lines of the leak table that a feature reports on their own: groups of vertices that must stay still or move only rigidly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Guard {
    /// The neck's rim and every vertex below it.
    Rim,
    /// The 36 vertices of the lip line.
    LipLine,
    /// The skin of the lips and around them (the mouth zone).
    Lips,
    /// The inside of the mouth.
    MouthBag,
    Ears,
    /// The 28 lid edge vertices and the 8 eyeball centre vertices of each eye (`lid_edges.json`), with their copies.
    Lids,
}

impl Guard {
    pub fn label(&self) -> &'static str {
        match self {
            Guard::Rim => "the neck's rim and below",
            Guard::LipLine => "the lip line",
            Guard::Lips => "the lips and the skin around them",
            Guard::MouthBag => "the inside of the mouth",
            Guard::Ears => "the ears",
            Guard::Lids => "the lid edges and the eyeballs' centres",
        }
    }
}

/// How a feature places the bumps its sliders are made of.
pub struct BumpRule {
    /// One spread of each bump's Gaussian, in mm.
    pub size_mm: f64,
    /// How far apart the bumps' centres are, in mm.
    pub spacing_mm: f64,
    /// The lowest zone weight at a bump's centre.
    pub min_weight: f64,
    /// Whether a place on the base head (frame of the eyes, mm) may hold a bump's centre. Only the right side and the middle are
    /// asked; each bump is mirrored.
    pub allowed: fn(V3) -> bool,
}

/// A clay picture's camera: turned by `yaw` degrees about the up axis, then by `pitch` degrees about the across axis through the height
/// `pivot`, then seen straight on through `window` (left, right, bottom, top, in mm after turning).
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub name: &'static str,
    pub yaw: f64,
    pub pitch: f64,
    pub pivot: f64,
    pub window: [f64; 4],
}

pub trait Feature: Sync + Send {
    /// A short key for files and the page ("nose", "jaw").
    fn key(&self) -> &'static str;
    /// The feature's name for a person.
    fn title(&self) -> &'static str;
    fn proportions(&self) -> &'static [Proportion];
    fn outlines(&self) -> &'static [OutlineDef];
    fn specs(&self) -> &'static [Spec];
    /// The groups of sliders, in the page's order.
    fn groups(&self) -> &'static [&'static str];
    /// The part of the face the measure reads.
    fn window(&self) -> Window;
    /// The feature itself, smaller than the window, where check 3 compares copies of a head.
    fn region(&self) -> Window;
    fn measure(&self, surface: &Surface) -> Result<Measured, &'static str>;
    /// The measure on a KOTOR head's own mesh, where some rows may be unreadable because hair or cloth covers them.
    fn measure_kotor(&self, surface: &Surface, _extras: &KotorExtras, _frame: &crate::measure::Frame, _positions: &[V3]) -> Result<Measured, &'static str> {
        self.measure(surface)
    }
    /// The feature's zone on the base head, from the base head's measure.
    fn zone(&self, mesh: &HeadMesh, base: &Measured) -> Zone;
    /// The part a place on the base head belongs to, in the frame of its eyes.
    fn part_of(&self, p: V3, base: &Measured) -> &'static str;
    fn bumps(&self) -> BumpRule;
    /// A sentence saying what a miss of `g` on proportion `i` gets wrong.
    fn miss_words(&self, i: usize, g: f64) -> String;
    /// Whether the hidden vertices (the insides of the nostrils and of the mouth) move with the skin nearest to them. The nose's
    /// nostrils do; the jaw leaves the inside of the mouth where it is, so a jaw slider never bends it.
    fn carries_hidden(&self) -> bool {
        true
    }
    /// The step of a drawn slider (`set::DrawnStep`), when the slider that sets `measure` is one: a shape too fine for bumps, found on
    /// each head as it is built (the brow's edge). The kit calibrates its amount on the base head (`maker::make_drawn`).
    fn drawn_step(&self, _measure: &str) -> Option<crate::set::DrawnStep> {
        None
    }
    /// A last change to each bump's movement (metres, every vertex), after the kit has built it. The eyes use it to make the lids and
    /// their lining slide on the eyeball. Most features leave it as it is.
    fn shape_bump(&self, _mesh: &HeadMesh, _movement: &mut [V3]) {}
    /// Which share of the heads' needs a slider's range covers on each side: 1 for the farthest head, less to leave out the few
    /// heads whose measure the coarse KOTOR mesh may misread.
    fn span_quantile(&self) -> f64 {
        1.0
    }
    /// The same share for the slider of one measure, for a slider that must reach every head while the others leave a few out.
    fn span_quantile_of(&self, _measure: &str) -> f64 {
        self.span_quantile()
    }
    /// The most a slider's range may reach on each side, in KOTOR spreads of its measure.
    fn span_cap_of(&self, _measure: &str) -> f64 {
        4.0
    }
    /// How much the optimiser weighs a measure's gap, against 1 for the others.
    fn gap_weight(&self, _measure: &str) -> f64 {
        1.0
    }
    /// Whether the optimiser aims at a quarter of KOTOR's spread rather than at the pass marks. The pass marks of a measure that
    /// wobbles on KOTOR's coarse meshes are so loose that the optimiser would not bother to move the sliders.
    fn solve_by_spread(&self) -> bool {
        false
    }
    /// The groups of vertices whose movement the leak table lists on their own lines.
    fn guards(&self) -> &'static [Guard] {
        &[]
    }
    /// Where check 1 puts its small bump, on a part of the face the measure should not see (frame of the eyes, mm).
    fn check_bump(&self) -> V3;
    /// The clay pictures of a slider's ends.
    fn shots(&self) -> &'static [Shot];
    /// The data files the kit writes for this feature, relative to the data folder.
    fn files(&self) -> Files {
        let k = self.key();
        Files {
            sliders: format!("web/{k}/sliders.json"),
            page: format!("web/{k}/page.json"),
            made: format!("web/{k}/made"),
            report: format!("{k}/report.json"),
            named: format!("{k}/named.json"),
            marks: format!("{k}/marks.json"),
            checks: format!("{k}/checks.json"),
            starts: format!("web/{k}/starts.bin"),
        }
    }
}

pub struct Files {
    pub sliders: String,
    pub page: String,
    /// Without its extension: `.bin` and `.json` are added.
    pub made: String,
    pub report: String,
    pub named: String,
    pub marks: String,
    pub checks: String,
    /// The 57 stand-ins before smoothing, as 32-bit floats, for the page.
    pub starts: String,
}

pub fn proportion_index(f: &dyn Feature, key: &str) -> usize {
    f.proportions().iter().position(|p| p.key == key).unwrap_or_else(|| panic!("{} has no measure {key}", f.key()))
}

/// A row index for a measure or outline key: proportions by their index, outlines after them.
pub fn row_of(f: &dyn Feature, key: &str) -> Option<usize> {
    let n = f.proportions().len();
    f.proportions().iter().position(|p| p.key == key).or_else(|| f.outlines().iter().position(|o| o.key == key).map(|o| n + o))
}

/// An outline with its position and size taken out: moved so its centre is at the origin and scaled so its points lie at 1 from it
/// on average (root mean square). Its turn is kept.
pub fn normalized(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    crate::measure::normalized(points)
}

/// The size of an outline: the root mean square distance of its points from their centre, in mm; for an outline of widths, its mean
/// half width.
pub fn outline_size(def: &OutlineDef, points: &[[f64; 2]]) -> f64 {
    if def.widths {
        return points.iter().map(|p| p[0]).sum::<f64>() / points.len() as f64;
    }
    let n = points.len() as f64;
    let c = [points.iter().map(|p| p[0]).sum::<f64>() / n, points.iter().map(|p| p[1]).sum::<f64>() / n];
    (points.iter().map(|p| (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)).sum::<f64>() / n).sqrt()
}

/// The point by point gaps between two copies of an outline after position and size are taken out, in mm on an outline of `reference`
/// mm, two per point.
pub fn outline_gaps(def: &OutlineDef, a: &[[f64; 2]], b: &[[f64; 2]], reference: f64) -> Vec<f64> {
    if def.widths {
        let mean = |l: &[[f64; 2]]| (l.iter().map(|p| p[0]).sum::<f64>() / l.len() as f64).max(1e-9);
        let (ma, mb) = (mean(a), mean(b));
        return a.iter().zip(b).flat_map(|(p, q)| [(p[0] / ma - q[0] / mb) * reference, 0.0]).collect();
    }
    let (a, b) = (normalized(a), normalized(b));
    a.iter().zip(&b).flat_map(|(p, q)| [(p[0] - q[0]) * reference, (p[1] - q[1]) * reference]).collect()
}

/// Like `outline_gaps`, but with each axis's position and size taken out on its own, so stretching an outline along one axis costs
/// nothing and only a change of its shape counts. The named sliders hold the outlines this way.
pub fn outline_shape_gaps(def: &OutlineDef, a: &[[f64; 2]], b: &[[f64; 2]], reference: f64) -> Vec<f64> {
    if def.widths {
        return outline_gaps(def, a, b, reference);
    }
    let axes = |l: &[[f64; 2]]| -> Vec<[f64; 2]> {
        let n = l.len() as f64;
        let mut out = l.to_vec();
        for c in 0..2 {
            let m = l.iter().map(|p| p[c]).sum::<f64>() / n;
            let s = (l.iter().map(|p| (p[c] - m).powi(2)).sum::<f64>() / n).sqrt().max(1e-9);
            for (q, p) in out.iter_mut().zip(l) {
                q[c] = (p[c] - m) / s;
            }
        }
        out
    };
    let (a, b) = (axes(a), axes(b));
    let k = reference / std::f64::consts::SQRT_2;
    a.iter().zip(&b).flat_map(|(p, q)| [(p[0] - q[0]) * k, (p[1] - q[1]) * k]).collect()
}

/// The difference between two copies of an outline: the root mean square distance between matching points, after position and size
/// are taken out, in mm on an outline of `reference` mm.
pub fn outline_difference(def: &OutlineDef, a: &[[f64; 2]], b: &[[f64; 2]], reference: f64) -> f64 {
    let g = outline_gaps(def, a, b, reference);
    (g.as_chunks::<2>().0.iter().map(|c| c[0] * c[0] + c[1] * c[1]).sum::<f64>() / (g.len() / 2) as f64).sqrt()
}

/// How the gaps between two measured features count: each proportion's gap divided by its pass mark, and each outline's root mean
/// square gap divided by its pass mark.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Scale {
    pub proportions: Vec<f64>,
    pub outlines: Vec<f64>,
    /// The outlines' reference sizes in mm.
    pub references: Vec<f64>,
}

/// The features the kit knows, by key.
pub fn by_key(key: &str) -> Option<&'static dyn Feature> {
    match key {
        "nose" => Some(&crate::nose::NOSE),
        "jaw" => Some(&crate::jaw::JAW),
        "brow" => Some(&crate::brow::BROW),
        "cheeks" => Some(&crate::cheeks::CHEEKS),
        "eyes" => Some(&crate::eyes::EYES),
        _ => None,
    }
}

pub fn all() -> [&'static dyn Feature; 5] {
    [&crate::nose::NOSE, &crate::jaw::JAW, &crate::brow::BROW, &crate::cheeks::CHEEKS, &crate::eyes::EYES]
}
