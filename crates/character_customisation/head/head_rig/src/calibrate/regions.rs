//! Which vertices each fit moves, which bones it may give them, and the hints it starts from. All of it is set on the base head, so it
//! is the same for every head.

use super::data::{Common, Template};
use super::smooth;
use super::solve::Problem;
use std::collections::HashMap;

/// How strongly the copies of one place, on the seams of the texture layout, are tied together.
const COPIES: f64 = 1e4;

pub struct Region {
    pub problem: Problem,
    /// A pull towards `hint`, per vertex, and the weights it pulls towards.
    pub hint_strength: Vec<f64>,
    pub hint: Vec<Vec<f64>>,
    /// Where the correction from the head's shape may change the weights, from 0 to 1 per vertex.
    pub keep: Vec<f64>,
    /// The vertex the head's shape is measured from.
    pub anchor: usize,
    /// The edges tied by one over their length squared, for smoothing a head's change from the shared fit.
    pub plain_ties: Vec<(usize, usize, f64)>,
}

/// The mesh's edges, each tied by the mean of `field` at its two ends over its length squared in millimetres, and the copies of one place.
pub fn ties(common: &Common, template: &Template, field: &[f64]) -> Vec<(usize, usize, f64)> {
    let p = &template.pos;
    let mut out: Vec<(usize, usize, f64)> = common
        .edges
        .iter()
        .filter_map(|&[a, b]| {
            let l = crate::length(crate::sub(p[a], p[b])) * 1000.0;
            (l > 1e-6).then(|| (a, b, (field[a] + field[b]) / 2.0 / (l * l)))
        })
        .collect();
    let mut first: HashMap<[i64; 3], usize> = HashMap::new();
    for (v, q) in p.iter().enumerate() {
        let key = q.map(|x| (x * 1e5).round() as i64);
        let f = *first.entry(key).or_insert(v);
        if f != v {
            out.push((f, v, COPIES));
        }
    }
    out
}

fn dense_on(common: &Common, bones: &[usize]) -> (Vec<Vec<f64>>, Vec<f64>) {
    let fixed = common
        .base_skin
        .iter()
        .map(|w| {
            let r: Vec<f64> = bones.iter().map(|&b| w[b]).collect();
            let s: f64 = r.iter().sum();
            r.iter().map(|x| x / s.max(1e-9)).collect()
        })
        .collect();
    let outside = common
        .base_skin
        .iter()
        .map(|w| w.iter().sum::<f64>() - bones.iter().map(|&b| w[b]).sum::<f64>())
        .collect();
    (fixed, outside)
}

/// 1 on the lips and their curl into the mouth, 0 on the skin around them.
pub fn lipness(t: &Template) -> Vec<f64> {
    let cx = t.landmarks["cornerL"][0];
    t.pos
        .iter()
        .map(|p| {
            (1.0 - smooth(0.005, 0.009, (p[1] - t.mouth_y).abs()))
                * (1.0 - smooth(cx - 0.004, cx + 0.002, p[0].abs()))
        })
        .collect()
}

/// 1 around the mouth corners, where the two lips join, 0 elsewhere.
pub fn cornerness(t: &Template) -> Vec<f64> {
    let c = t.landmarks["cornerL"];
    t.pos
        .iter()
        .map(|p| {
            let d =
                (((p[0].abs() - c[0]) / 0.012).powi(2) + ((p[1] - c[1]) / 0.010).powi(2)).sqrt();
            1.0 - smooth(0.6, 1.2, d)
        })
        .collect()
}

pub const MOUTH_BONES: [&str; 10] = [
    "head_g", "f_um_g", "f_lmc_g", "f_rmc_g", "f_jaw_g", "f_llm_g", "f_rlm_g", "f_lns_g",
    "f_rns_g", "neck_g",
];

pub const BROW_BONES: [&str; 5] = [
    "f_mdbrw_g",
    "f_lbrw_g",
    "f_rbrw_g",
    "f_llweye_g",
    "f_rlweye_g",
];

/// How the upper face is fitted, when it is.
#[derive(Clone, Copy, Debug)]
pub struct Brows {
    /// The smoothness over the upper face, against the mouth's 10.
    pub smoothness: f64,
}

/// How the neck is fitted, when it is. KOTOR's head model only covers the neck down to about 80 to 120 mm below the lip line, and its
/// body carries the rest, so below that our neck is pulled onto the lower neck bone, which the clips never move. At the back, the
/// neck reaches up to the base of the skull.
#[derive(Clone, Copy, Debug)]
pub struct Neck {
    /// The smoothness over the neck, against the mouth's 10.
    pub smoothness: f64,
}

/// The lower face, as Mouth v2 fitted it on 2026-11-19, and the brows and the cheeks under the eyes when `brows` is given. Each lip is
/// pulled onto its own bones, fading out 4 to 12 mm before the corners, and the smoothness is 30 times stronger at the corners. The
/// whole face is one fit, so the weights do not step where the mouth's part ends.
pub fn face(common: &Common, t: &Template, brows: Option<Brows>, neck: Option<Neck>) -> Region {
    let names: Vec<&str> = MOUTH_BONES
        .iter()
        .chain(if brows.is_some() {
            &BROW_BONES[..]
        } else {
            &[]
        })
        .chain(if neck.is_some() {
            &["necklwr_g"][..]
        } else {
            &[]
        })
        .copied()
        .collect();
    let bones: Vec<usize> = names.iter().map(|b| common.bone(b)).collect();
    let (fixed, outside) = dense_on(common, &bones);
    let my = t.mouth_y;
    let top = if brows.is_some() {
        my + 0.13
    } else {
        my + 0.03
    };
    // The underside of the brow carries a little of the old lid weight. Viewer 2 keeps the lid bones at rest and the upper lid becomes
    // a piece of its own (decision 59), so with the brows that share is given to the fit, up to 0.2.
    let lids: Vec<f64> = common
        .base_skin
        .iter()
        .map(|w| w[common.bone("eyellid")] + w[common.bone("eyerlid")])
        .collect();
    let inside: std::collections::HashSet<usize> = t.mouth_inside.iter().copied().collect();
    let free: Vec<usize> = (0..common.n)
        .filter(|&v| {
            let p = t.pos[v];
            let other = if brows.is_some() && lids[v] < 0.2 {
                outside[v] - lids[v]
            } else {
                outside[v]
            };
            let in_face = p[1] > my - 0.06 && p[1] < top && p[2] > 0.07;
            let in_neck = neck.is_some()
                && (p[1] < my - 0.02 || (p[2] < 0.06 && p[1] < my + 0.03))
                && !inside.contains(&v);
            (in_face || in_neck) && other < 0.01
        })
        .collect();
    let upper: Vec<bool> = t
        .upper
        .iter()
        .map(|&u| smooth(0.3, 0.7, u) >= 0.5)
        .collect();
    let side: Vec<bool> = t.side.iter().map(|&k| k > 0.5).collect();
    let allowed = free
        .iter()
        .map(|&v| {
            let [x, y, z] = t.pos[v];
            // The face's own bones only carry the front of the head, and the jaw only its front half, so that a bone the clips
            // barely move cannot stand in for the lower neck bone at the back.
            let front = z > 0.07;
            let below_nose = y < my + 0.045 && y > my - 0.06 && front;
            names
                .iter()
                .map(|&b| match b {
                    "f_lmc_g" => x > -0.008 && below_nose,
                    "f_rmc_g" => x < 0.008 && below_nose,
                    "f_lns_g" => x > -0.008 && x < 0.05 && y > my - 0.01 && front,
                    "f_rns_g" => x < 0.008 && x > -0.05 && y > my - 0.01 && front,
                    "f_llm_g" => x > -0.008 && !(side[v] && upper[v]) && below_nose,
                    "f_rlm_g" => x < 0.008 && !(side[v] && upper[v]) && below_nose,
                    "f_jaw_g" => {
                        !(side[v] && upper[v]) && y < my + 0.045 && y > my - 0.08 && z > 0.03
                    }
                    "necklwr_g" => y < my - 0.04 || (z < 0.09 && y < my + 0.03),
                    "f_um_g" => !(side[v] && !upper[v]) && below_nose,
                    "neck_g" => y < my - 0.025 || (z < 0.09 && y < my + 0.03),
                    "f_mdbrw_g" => x.abs() < 0.04 && y > my + 0.02 && front,
                    "f_lbrw_g" | "f_llweye_g" => x > -0.008 && y > my + 0.02 && front,
                    "f_rbrw_g" | "f_rlweye_g" => x < 0.008 && y > my + 0.02 && front,
                    _ => true,
                })
                .collect()
        })
        .collect();
    let lip = lipness(t);
    let corner = cornerness(t);
    let field: Vec<f64> = (0..common.n)
        .map(|v| {
            let upper_face = brows.map_or(10.0, |b| {
                10.0 + (b.smoothness - 10.0) * smooth(my + 0.02, my + 0.04, t.pos[v][1])
            });
            let lower = neck.map_or(0.0, |n| {
                (n.smoothness - 10.0) * (1.0 - smooth(my - 0.06, my - 0.04, t.pos[v][1]))
            });
            upper_face + lower + 290.0 * corner[v]
        })
        .collect();
    let cx = t.landmarks["cornerL"][0];
    let bottom: Vec<f64> = (0..common.n)
        .map(|v| {
            if neck.is_some() {
                1.0 - smooth(my - 0.10, my - 0.08, t.pos[v][1])
            } else {
                0.0
            }
        })
        .collect();
    let hint_strength = (0..common.n)
        .map(|v| {
            let x = t.pos[v][0].abs();
            10.0 * lip[v]
                * (1.0 - smooth(cx - 0.012, cx - 0.004, x))
                * if side[v] { 1.0 } else { 0.0 }
                + 10.0 * bottom[v]
        })
        .collect();
    let mut hint = fixed.clone();
    let j = |b: &str| names.iter().position(|x| *x == b).unwrap();
    for &v in &free {
        if bottom[v] > 0.0 {
            let mut w = vec![0.0; names.len()];
            w[j("necklwr_g")] = 1.0;
            hint[v] = w;
            continue;
        }
        if !side[v] {
            continue;
        }
        let mut w = vec![0.0; names.len()];
        if upper[v] {
            w[j("f_um_g")] = 1.0;
        } else {
            let sx = smooth(-0.006, 0.006, t.pos[v][0]);
            w[j("f_llm_g")] = sx;
            w[j("f_rlm_g")] = 1.0 - sx;
        }
        hint[v] = w;
    }
    let keep = (0..common.n)
        .map(|v| (1.0 - corner[v]) * (1.0 - lip[v]))
        .collect();
    Region {
        problem: Problem {
            bones,
            free,
            fixed,
            allowed,
            ties: ties(common, t, &field),
        },
        hint_strength,
        hint,
        keep,
        anchor: common.lip_line,
        plain_ties: ties(common, t, &vec![1.0; common.n]),
    }
}
