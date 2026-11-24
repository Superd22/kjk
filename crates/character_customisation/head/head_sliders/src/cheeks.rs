//! The cheekbones and the cheeks as a feature of the slider kit.
//!
//! Everything is measured in the frame of the eyes (`measure::Frame`), on any triangle mesh. The cheekbone's crest is read on upright
//! cuts below the outer half of each eye. The cheek below it is read on the same cuts, against the straight line from the crest down to
//! the level of the mouth. The face's width is read on level cuts, where the face falls 25 mm behind the cheekbone.

use crate::V3;
use crate::brow::{distances, near_eye, level, measure_uncovered, side_at_depth, smoothstep, upright};
use crate::feature::{BumpRule, Feature, Guard, KotorExtras, Measured, OutlineDef, Shot, Spec};
use crate::measure::{Frame, Proportion, Surface, Window, resample};
use crate::mesh::HeadMesh;
use crate::zone::Zone;
use serde_json::json;

pub struct Cheeks;

pub static CHEEKS: Cheeks = Cheeks;

pub const PROPORTIONS: [Proportion; 4] = [
    Proportion { key: "cheek_width", label: "Width of the face at the cheekbones", low: "narrow cheekbones", high: "wide cheekbones", unit: "mm" },
    Proportion { key: "cheekbone_projection", label: "How far the cheekbones stand out to the front", low: "flat cheekbones", high: "prominent cheekbones", unit: "mm" },
    Proportion { key: "cheek_fullness", label: "How far the cheek below the cheekbone lies in front of it or behind it", low: "hollow cheeks", high: "full cheeks", unit: "mm" },
    Proportion { key: "cheek_side", label: "Width of the face below the cheekbones", low: "narrow below the cheekbones", high: "wide below the cheekbones", unit: "mm" },
];

pub const OUTLINES: [OutlineDef; 2] = [
    OutlineDef { key: "cheek_profile", label: "Cheek from the side", points: 16, widths: false },
    OutlineDef { key: "face_front", label: "Face outline from the front", points: 11, widths: true },
];

const GROUPS: [&str; 2] = ["cheekbones", "cheeks"];

const SHOTS: [Shot; 4] = [
    Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-85.0, 85.0, -95.0, 30.0] },
    Shot { name: "three-quarter", yaw: 40.0, pitch: 0.0, pivot: 0.0, window: [-95.0, 75.0, -95.0, 30.0] },
    Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-110.0, 60.0, -95.0, 30.0] },
    Shot { name: "below", yaw: 0.0, pitch: 45.0, pivot: -40.0, window: [-85.0, 85.0, -105.0, 20.0] },
];

/// The window the measure reads.
const WINDOW: Window = Window { x: 95.0, y: [-90.0, 20.0], z: -80.0 };

/// The upright cuts of the cheekbone, in mm from the middle.
const BONE_XS: [f64; 3] = [40.0, 45.0, 50.0];
/// The upright cut of the cheek's profile and fullness, in mm from the middle.
const CHEEK_X: f64 = 40.0;
/// The bottom of the cheek's line, below the eyes.
const CHEEK_BOTTOM: f64 = -70.0;

/// Measures the cheekbones and the cheeks of a surface, or says what could not be found.
pub fn measure(surface: &Surface) -> Result<Measured, &'static str> {
    let mut dots: Vec<V3> = Vec::new();
    // the cheekbone's depth: the upright cuts below the outer half of each eye, 15 to 30 mm below the eyes, both sides averaged; and the
    // cheek's fullness: how far the cheek 36 to 46 mm below the eyes lies in front of the cheekbone (negative when it lies behind)
    let (mut bz, mut drop) = (0.0, 0.0);
    for &x in &BONE_XS {
        for side in [-1.0, 1.0] {
            let c = upright(surface, side * x, -80.0, 10.0);
            let band = |lo: f64, hi: f64| -> Option<f64> {
                let ks: Vec<usize> = (c.index(lo)..=c.index(hi)).filter(|&k| !c.z[k].is_nan()).collect();
                (ks.len() > 20).then(|| ks.iter().map(|&k| c.z[k]).sum::<f64>() / ks.len() as f64)
            };
            let (hi, lo) = (band(-30.0, -15.0).ok_or("no cheekbone")?, band(-46.0, -36.0).ok_or("no cheek")?);
            dots.extend([[side * x, -22.5, hi], [side * x, -41.0, lo]]);
            bz += hi / 6.0;
            drop += (lo - hi) / 6.0;
        }
    }
    let cheekbone_projection = bz;
    // the cheek's profile from the side, 40 mm from the middle, both sides averaged
    let mut profile = vec![[0.0; 2]; OUTLINES[0].points];
    for side in [-1.0, 1.0] {
        let c = upright(surface, side * CHEEK_X, -80.0, 10.0);
        let line: Vec<[f64; 2]> = (c.index(CHEEK_BOTTOM + 2.0)..=c.index(-5.0)).filter(|&k| !c.z[k].is_nan()).map(|k| [c.u(k), c.z[k]]).collect();
        if line.len() < 100 {
            return Err("no cheek profile");
        }
        for (p, q) in profile.iter_mut().zip(resample(&line, OUTLINES[0].points)) {
            p[0] += 0.5 * q[0];
            p[1] += 0.5 * q[1];
        }
    }
    dots.extend(profile.iter().map(|p| [CHEEK_X, p[0], p[1]]));
    // the face's half width where it falls 25 mm behind its depth 40 mm from the middle, both sides averaged
    let half = |y: f64, dots: &mut Vec<V3>| -> Option<f64> {
        let c = level(surface, y);
        let mut w = 0.0;
        for side in [-1.0, 1.0] {
            let h = side_at_depth(&c, side * CHEEK_X, 25.0)?;
            dots.push([side * h, y, c.at(side * CHEEK_X) - 25.0]);
            w += 0.5 * h;
        }
        Some(w)
    };
    let mut cheek_width = 0.0;
    for y in [-15.0, -20.0, -25.0] {
        cheek_width += 2.0 * half(y, &mut dots).ok_or("no cheek width")? / 3.0;
    }
    let mut cheek_side = 0.0;
    for y in [-45.0, -50.0, -55.0] {
        cheek_side += 2.0 * half(y, &mut dots).ok_or("no width below the cheekbones")? / 3.0;
    }
    let mut front = Vec::with_capacity(OUTLINES[1].points);
    for k in 0..OUTLINES[1].points {
        let y = -5.0 * k as f64;
        front.push([half(y, &mut Vec::new()).ok_or("no face outline")?, y]);
    }
    Ok(Measured {
        proportions: vec![cheek_width, cheekbone_projection, drop, cheek_side],
        outlines: vec![profile, front],
        marks: json!({"dots": dots}),
        cuts: Vec::new(),
    })
}

/// The parts of the face that hair, a beard or cloth may cover on a KOTOR head, and the rows each carries.
fn bone_box(p: V3) -> bool {
    p[0].abs() > 30.0 && p[0].abs() < 62.0 && p[1] > -42.0 && p[1] < -5.0 && p[2] > -40.0
}
fn cheek_box(p: V3) -> bool {
    p[0].abs() > 25.0 && p[0].abs() < 62.0 && p[1] > -72.0 && p[1] < -35.0 && p[2] > -40.0
}
fn side_box(p: V3) -> bool {
    p[0].abs() > 55.0 && p[0].abs() < 85.0 && p[1] > -60.0 && p[1] < 5.0 && p[2] > -45.0
}
const BONE_ROWS: [&str; 1] = ["cheekbone_projection"];
const CHEEK_ROWS: [&str; 2] = ["cheek_fullness", "cheek_profile"];
const SIDE_ROWS: [&str; 3] = ["cheek_width", "cheek_side", "face_front"];

fn allowed(_p: V3) -> bool {
    true
}

impl Feature for Cheeks {
    fn key(&self) -> &'static str {
        "cheeks"
    }
    fn title(&self) -> &'static str {
        "Cheekbones and cheeks"
    }
    fn proportions(&self) -> &'static [Proportion] {
        &PROPORTIONS
    }
    fn outlines(&self) -> &'static [OutlineDef] {
        &OUTLINES
    }
    fn specs(&self) -> &'static [Spec] {
        &SPECS
    }
    fn groups(&self) -> &'static [&'static str] {
        &GROUPS
    }
    fn window(&self) -> Window {
        WINDOW
    }
    fn region(&self) -> Window {
        Window { x: 75.0, y: [-70.0, 0.0], z: -40.0 }
    }
    fn measure(&self, surface: &Surface) -> Result<Measured, &'static str> {
        measure(surface)
    }
    fn measure_kotor(&self, surface: &Surface, extras: &KotorExtras, frame: &Frame, positions: &[V3]) -> Result<Measured, &'static str> {
        measure_uncovered(self, surface, extras, frame, positions, &[(bone_box, &BONE_ROWS), (cheek_box, &CHEEK_ROWS), (side_box, &SIDE_ROWS)])
    }
    /// The cheekbones and the cheeks: 1 inside, fading to 0 over 6 mm toward the skin round the eyes (the zone "around the eyes",
    /// which the eye sliders own), the nose and the lips, from 50 to 68 mm below the eyes toward the jaw, above 0 to 10 mm below the
    /// eyes toward the temples, from 30 to 55 mm behind the eyes toward the sides of the head, and over 10 mm toward the ears. Every
    /// vertex the smoothing holds still has 0.
    fn zone(&self, mesh: &HeadMesh, _base: &Measured) -> Zone {
        let frame = Frame::from_eyes(mesh.eyes);
        let local: Vec<V3> = mesh.first.iter().map(|&i| frame.local(mesh.base[i as usize])).collect();
        let n = mesh.merged_count();
        let zone_of = |name: &str| 1 + crate::mesh::ZONES.iter().position(|z| *z == name).unwrap() as u8;
        let (eyes, nose, mouth) = (zone_of("around the eyes"), zone_of("nose"), zone_of("mouth"));
        let is = |g: usize, z: u8| mesh.zone[mesh.first[g] as usize] == z;
        let held: Vec<bool> = (0..n).map(|g| mesh.keep[mesh.first[g] as usize] && !mesh.hidden[g]).collect();
        let socket: Vec<bool> = (0..n).map(|g| is(g, eyes) || (held[g] && local[g][0].abs() < 60.0 && local[g][1].abs() < 20.0)).collect();
        let noses: Vec<bool> = (0..n).map(|g| is(g, nose)).collect();
        let lips: Vec<bool> = (0..n).map(|g| is(g, mouth)).collect();
        let ears: Vec<bool> = (0..n).map(|g| held[g] && local[g][0].abs() >= 60.0).collect();
        let (ds, dn, dl, de) = (distances(&local, &socket), distances(&local, &noses), distances(&local, &lips), distances(&local, &ears));
        let weight = (0..n)
            .map(|g| {
                let p = local[g];
                if held[g] || socket[g] || noses[g] || lips[g] || near_eye(p) {
                    return 0.0;
                }
                smoothstep(ds[g] / 6.0)
                    * smoothstep(dn[g] / 6.0)
                    * smoothstep(dl[g] / 6.0)
                    * smoothstep(de[g] / 10.0)
                    * smoothstep((p[1] + 68.0) / 18.0)
                    * smoothstep((10.0 - p[1]) / 10.0)
                    * smoothstep((p[2] + 55.0) / 25.0)
            })
            .collect();
        Zone { weight }
    }
    /// The parts on the base head: the sides (more than 58 mm from the middle, or more than 25 mm behind the eyes), the cheekbones
    /// (above 38 mm below the eyes) and the cheeks (the rest).
    fn part_of(&self, p: V3, _base: &Measured) -> &'static str {
        if p[0].abs() > 58.0 || p[2] < -25.0 {
            "sides"
        } else if p[1] > -38.0 {
            "cheekbones"
        } else {
            "cheeks"
        }
    }
    fn bumps(&self) -> BumpRule {
        BumpRule { size_mm: 8.0, spacing_mm: 6.0, min_weight: 0.6, allowed }
    }
    fn miss_words(&self, i: usize, g: f64) -> String {
        let a = g.abs();
        let (more, less) = match PROPORTIONS[i].key {
            "cheek_width" => ("The face is too wide at the cheekbones", "The face is too narrow at the cheekbones"),
            "cheekbone_projection" => ("The cheekbones stand out too far", "The cheekbones are too flat"),
            "cheek_fullness" => ("The cheeks are too full", "The cheeks are too hollow"),
            "cheek_side" => ("The face is too wide below the cheekbones", "The face is too narrow below the cheekbones"),
            _ => ("Too much", "Too little"),
        };
        format!("{} by {a:.1} mm.", if g > 0.0 { more } else { less })
    }
    fn carries_hidden(&self) -> bool {
        false
    }
    fn span_quantile(&self) -> f64 {
        0.95
    }
    fn solve_by_spread(&self) -> bool {
        true
    }
    fn guards(&self) -> &'static [Guard] {
        &[Guard::LipLine, Guard::Lips, Guard::MouthBag, Guard::Ears, Guard::Lids]
    }
    fn check_bump(&self) -> V3 {
        [0.0, -35.0, 40.0]
    }
    fn shots(&self) -> &'static [Shot] {
        &SHOTS
    }
}

pub const SPECS: [Spec; 4] = [
    Spec {
        measure: "cheek_width",
        name: "Cheekbone width",
        hint: "How wide the face is at the cheekbones, seen from the front.",
        group: "cheekbones",
        parts: &["sides", "cheekbones"],
        repair_parts: &[],
        free: &["face_front"],
        loose: &["cheek_side"],
    },
    Spec {
        measure: "cheekbone_projection",
        name: "Cheekbone prominence",
        hint: "How far the cheekbones stand out to the front, seen from the side or from below. The cheeks below follow part of the way.",
        group: "cheekbones",
        parts: &["cheekbones"],
        repair_parts: &["sides"],
        free: &["cheek_profile"],
        loose: &["cheek_fullness"],
    },
    Spec {
        measure: "cheek_fullness",
        name: "Cheek fullness",
        hint: "Whether the cheeks below the cheekbones are hollow or full, seen from the side or at three-quarter.",
        group: "cheeks",
        parts: &["cheeks"],
        repair_parts: &[],
        free: &["cheek_profile"],
        loose: &[],
    },
    Spec {
        measure: "cheek_side",
        name: "Lower cheek width",
        hint: "How wide the face is below the cheekbones, seen from the front. Narrow gives sunken sides, and wide gives a round face.",
        group: "cheeks",
        parts: &["cheeks", "sides"],
        repair_parts: &[],
        free: &["face_front"],
        loose: &["cheek_width"],
    },
];
