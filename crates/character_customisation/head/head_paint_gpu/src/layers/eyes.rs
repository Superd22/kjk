//! The eye paint (`head_paint::eyes`). The processor finds each eye's lid edges on the head, lays the liner's wing on the skin and works
//! out the colours. The shader computes every mask per pixel from the distances to those lines (`shaders/eyes.wgsl`).
//!
//! The darker skin of the socket, the crease and the under-eye is the skin's own colour scaled down to a lower L*, as in a shadow, so it
//! follows the base tone's pigments. The red of the rims and the lids adds a* to the skin's own colour.
//!
//! Uniform buffer, in `vec4<f32>` rows: nine rows of settings and colours (see `EyesParams` in the shader), then for each eye the centre,
//! the direction toward the temple with the eyeball's radius in w, the direction up, 16 points of the upper lid, 14 of the lower lid and
//! 8 of the wing. The storage buffer is not used.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::eyes::{EyeLines, EyePaint, eye_shapes_on, lch_to_linear};
use head_paint::pigment::{PigmentTable, linear_to_lab};

/// The colour of the lashes, in linear sRGB: a near black with a little brown.
const LASH: [f64; 3] = [0.0045, 0.0036, 0.0031];

#[derive(Default)]
pub struct EyeLayer {
    table: Option<PigmentTable>,
}

fn f(v: f64) -> u32 {
    (v as f32).to_bits()
}

fn row(v: [f64; 4]) -> [u32; 4] {
    v.map(f)
}

/// The natural log, per channel, of how the rims' red changes the skin's colour at full strength: the same lightness with 20 more a*
/// and 3 less b*, a pink red that follows the skin's own colour.
fn red_tint(base: [f64; 3]) -> [f64; 3] {
    let lab = linear_to_lab(base);
    let redder = head_paint::colour::lab_to_srgb([lab[0], lab[1] + 20.0, lab[2] - 3.0]).map(head_paint::pigment::decode);
    [0, 1, 2].map(|k| (redder[k].max(1e-6) / base[k].max(1e-6)).ln())
}

fn eye_rows(lines: Option<&EyeLines>) -> Vec<u32> {
    let mut w = Vec::with_capacity(4 * 41);
    match lines {
        None => w.resize(4 * 41, 0),
        Some(l) => {
            w.extend(row([l.centre[0], l.centre[1], l.centre[2], 1.0]));
            w.extend(row([l.out[0], l.out[1], l.out[2], l.radius]));
            w.extend(row([l.up[0], l.up[1], l.up[2], 0.0]));
            for (list, n) in [(&l.upper, 16), (&l.lower, 14), (&l.wing, 8)] {
                assert_eq!(list.len(), n, "the eye's lines have the wrong number of points");
                for (p, x) in list.iter() {
                    w.extend(row([p[0], p[1], p[2], *x]));
                }
            }
        }
    }
    w
}

impl GpuLayer for EyeLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let paint: &EyePaint = &input.recipe.eyes;
        let detail = input.head.detail.as_ref().and_then(|d| d.folds.as_ref());
        let shapes = eye_shapes_on(&input.head.positions, input.head.eye_edge);
        let on = input.layers.eyes && !paint.is_none() && shapes.is_some() && detail.is_some();
        let table = self.table.get_or_insert_with(PigmentTable::build);
        let base = input.recipe.base_colour(input.layers, table);
        let red = red_tint(base);
        let p = paint;
        let mut w: Vec<u32> = Vec::with_capacity(4 * (9 + 2 * 41));
        w.extend(row([if on { 1.0 } else { 0.0 }, p.upper_lash, p.upper_lash_mm, p.lower_lash]));
        w.extend(row([p.lower_lash_mm, p.liner, p.liner_mm, p.liner_lower]));
        w.extend(row([p.crease, p.crease_mm, p.socket, p.under_eye]));
        w.extend(row([p.shadow, p.shadow_mm, p.rims, p.wing_mm]));
        let lash = if std::env::var("EYES_DEBUG").is_ok() { [0.0, 1.0, 0.0] } else { LASH };
        for c in [lash, lch_to_linear(p.liner_colour), lch_to_linear(p.shadow_colour)] {
            w.extend(row([c[0], c[1], c[2], 0.0]));
        }
        w.extend(row([p.lid_light, p.bags, p.bags_mm, 0.0]));
        w.extend(row([red[0], red[1], red[2], p.lid_red]));
        match (on, shapes, detail) {
            (true, Some(s), Some(d)) => {
                for side in 0..2 {
                    let lines = EyeLines::new(paint, &s[side], side, &d.landmarks.frame);
                    if std::env::var("EYES_DEBUG").is_ok() {
                        let pl = |p: [f64; 3]| { let r = [p[0] - lines.centre[0], p[1] - lines.centre[1], p[2] - lines.centre[2]]; (r[0] * lines.out[0] + r[1] * lines.out[1] + r[2] * lines.out[2], r[0] * lines.up[0] + r[1] * lines.up[1] + r[2] * lines.up[2]) };
                        eprintln!("side {side} upper {:?}", lines.upper.iter().map(|(p, _)| pl(*p)).collect::<Vec<_>>());
                        let g = |p: [f64; 3]| ((p[0] - lines.centre[0]).powi(2) + (p[1] - lines.centre[1]).powi(2) + (p[2] - lines.centre[2]).powi(2)).sqrt() - lines.radius;
                        eprintln!("side {side} upper gap {:?}", lines.upper.iter().map(|(p, _)| (g(*p) * 10.0).round() / 10.0).collect::<Vec<_>>());
                        eprintln!("side {side} lower gap {:?}", lines.lower.iter().map(|(p, _)| (g(*p) * 10.0).round() / 10.0).collect::<Vec<_>>());
                        eprintln!("side {side} wing {:?}", lines.wing);
                    }
                    w.extend(eye_rows(Some(&lines)));
                }
            }
            _ => {
                w.extend(eye_rows(None));
                w.extend(eye_rows(None));
            }
        }
        LayerWords { uniform: w, storage: input.new_head.then(|| vec![0; 4]) }
    }

    fn paints(&self, id: &str) -> bool {
        id.starts_with("eye_")
    }
}
