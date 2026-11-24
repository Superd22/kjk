//! The skin's colour under light and its colour zones (`head_paint::skin`, `shaders/skin.wgsl`).
//!
//! The processor's half gives the sliders, the zones' centres in millimetres from the head's landmarks, and a small table of colours of
//! the base tone with its melanin and blood changed (`SkinColour::table`). The shader reads the zones' amounts at each pixel and the
//! colour from the table. It has no storage buffer, because the fragment stage has room for only 8.
//!
//! Uniform buffer, as `vec4<f32>` rows:
//! 0. whether the light sliders act, whether the zones act, and two unused numbers;
//! 1. the four light sliders;
//! 2. and 3. the eight zone amounts from "Blood in the cheeks" to "Darker round the mouth";
//! 4. "Lighter middle of the face", "Zone size", "Light from the front" (0 on a mesh without the landmarks), and one unused number;
//! 5. to 22. the points: the two cheeks, the two ears, the chin, the forehead, the nose's root and tip, the two mouth corners, the inner
//!    and outer corner of the first eye, then of the second, the middle of the eyes, and the face's across, up and forward directions;
//! 23. onward: the table, `TABLE_U` colours per row of blood, `TABLE_B` rows.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::pigment::{PigmentTable, SpectralModel};
use head_paint::skin::{SkinColour, TABLE_B, TABLE_U};

#[derive(Default)]
pub struct SkinLayer {
    pigments: Option<(PigmentTable, SpectralModel)>,
    /// The points of the last head, which change only with the head.
    points: Option<Vec<[f64; 3]>>,
}

fn f(x: f64) -> u32 {
    (x as f32).to_bits()
}

fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn mid(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|k| 0.5 * (a[k] + b[k]))
}
fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(sub(a, b), sub(a, b)).sqrt()
}

/// The zones' points on a head, in millimetres, in the order of the uniform's rows 5 to 22. `None` on a mesh without the landmarks.
pub fn zone_points(input: &LayerInput) -> Option<Vec<[f64; 3]>> {
    let detail = input.head.detail.as_ref()?;
    let l = detail.folds.as_ref()?.landmarks;
    let m = detail.landmarks;
    let f = l.frame;
    let mm: Vec<[f64; 3]> = input.head.positions.iter().map(|p| p.map(|x| 1000.0 * x)).collect();
    let eyes = mid(mid(l.eye_inner[0], l.eye_outer[0]), mid(l.eye_inner[1], l.eye_outer[1]));
    // A centre placed by offsets from the landmarks can lie in front of the skin, so it is moved to the nearest vertex of the head.
    let snap = |c: [f64; 3]| mm.iter().copied().min_by(|a, b| dist(*a, c).total_cmp(&dist(*b, c))).unwrap_or(c);
    let cheeks = [0, 1].map(|s| snap(add(add(l.lower_lid[s], f.up, -17.0), f.out(s), 4.0)));
    // An ear's centre: the outermost vertex 0 to 40 mm below the eyes and 55 to 110 mm behind them, moved 6 mm inward.
    let ears = [0, 1].map(|side| {
        let out = f.out(side);
        let mut best = (f64::MIN, eyes);
        for &p in &mm {
            let r = sub(p, eyes);
            let (y, z) = (dot(r, f.up), dot(r, f.forward));
            if (-40.0..=0.0).contains(&y) && (-110.0..=-55.0).contains(&z) && dot(r, out) > best.0 {
                best = (dot(r, out), p);
            }
        }
        add(best.1, out, -6.0)
    });
    let chin = snap(add(l.chin_middle, f.up, -8.0));
    Some(vec![
        cheeks[0], cheeks[1], ears[0], ears[1], chin, m.forehead, m.nose[0], m.nose[1], l.mouth_corner[0], l.mouth_corner[1],
        l.eye_inner[0], l.eye_outer[0], l.eye_inner[1], l.eye_outer[1], eyes, f.across, f.up, f.forward,
    ])
}

impl GpuLayer for SkinLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let s: &SkinColour = &input.recipe.skin;
        if input.new_head || self.points.is_none() {
            self.points = zone_points(input);
        }
        let light_on = !s.light_is_none();
        let zones_on = input.layers.base_tone && !s.zones_are_none() && self.points.is_some();
        let v = s.values();
        let mut words = vec![u32::from(light_on), u32::from(zones_on), 0, 0];
        words.extend(v[0..4].iter().map(|&x| f(x)));
        words.extend(v[4..12].iter().map(|&x| f(x)));
        words.extend([f(v[12]), f(v[13].max(0.05)), f(if self.points.is_some() { v[14] } else { 0.0 }), 0]);
        let points = self.points.clone().unwrap_or_else(|| vec![[0.0; 3]; 18]);
        for p in &points {
            words.extend([f(p[0]), f(p[1]), f(p[2]), 0]);
        }
        if zones_on {
            let (pigments, model) = self.pigments.get_or_insert_with(|| (PigmentTable::build(), SpectralModel::new()));
            for c in SkinColour::table(&input.recipe.base_tone, pigments, model) {
                words.extend([f(c[0]), f(c[1]), f(c[2]), 0]);
            }
        } else {
            words.extend(std::iter::repeat_n(0, 4 * TABLE_U * TABLE_B));
        }
        LayerWords { uniform: words, storage: None }
    }

    fn paints(&self, id: &str) -> bool {
        id.starts_with("skin_")
    }

    fn has_storage(&self) -> bool {
        false
    }
}
