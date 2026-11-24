//! The fit of a base tone recipe to a target colour, used by `fit_base_tone` with `#[path]`. It never ships.

use head_paint::delta_e::delta_e_2000;
use head_paint::pigment::{BaseTone, HAEMOGLOBIN_MAX, MELANIN_MAX, PigmentTable};

const SCAN: usize = 160;

pub fn amounts(x: [f64; 2]) -> (f64, f64) {
    (
        x[0].clamp(0.0, 1.0).powi(3) * MELANIN_MAX,
        x[1].clamp(0.0, 1.0).powi(3) * HAEMOGLOBIN_MAX,
    )
}

pub fn nelder_mead(start: &[f64], step: &[f64], cost: &dyn Fn(&[f64]) -> f64) -> Vec<f64> {
    let n = start.len();
    let mut simplex: Vec<Vec<f64>> = vec![start.to_vec()];
    for i in 0..n {
        let mut p = start.to_vec();
        p[i] += step[i];
        simplex.push(p);
    }
    let mut values: Vec<f64> = simplex.iter().map(|p| cost(p)).collect();
    for _ in 0..400 {
        let mut order: Vec<usize> = (0..=n).collect();
        order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
        simplex = order.iter().map(|&i| simplex[i].clone()).collect();
        values = order.iter().map(|&i| values[i]).collect();
        if values[n] - values[0] < 1e-9 {
            break;
        }
        let centre: Vec<f64> = (0..n)
            .map(|k| simplex[..n].iter().map(|p| p[k]).sum::<f64>() / n as f64)
            .collect();
        let along = |t: f64| -> Vec<f64> {
            (0..n)
                .map(|k| centre[k] + t * (simplex[n][k] - centre[k]))
                .collect()
        };
        let reflected = along(-1.0);
        let fr = cost(&reflected);
        if fr < values[0] {
            let expanded = along(-2.0);
            let fe = cost(&expanded);
            if fe < fr {
                simplex[n] = expanded;
                values[n] = fe;
            } else {
                simplex[n] = reflected;
                values[n] = fr;
            }
        } else if fr < values[n - 1] {
            simplex[n] = reflected;
            values[n] = fr;
        } else {
            let contracted = along(0.5);
            let fc = cost(&contracted);
            if fc < values[n] {
                simplex[n] = contracted;
                values[n] = fc;
            } else {
                for i in 1..=n {
                    let first = simplex[0].clone();
                    for (v, f) in simplex[i].iter_mut().zip(first) {
                        *v = f + 0.5 * (*v - f);
                    }
                    values[i] = cost(&simplex[i]);
                }
            }
        }
    }
    let best = (0..=n)
        .min_by(|&a, &b| values[a].total_cmp(&values[b]))
        .unwrap();
    simplex[best].clone()
}

pub fn wrap(degrees: f64) -> f64 {
    let d = degrees.rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

pub struct Fit {
    pub tone: BaseTone,
    pub lab: [f64; 3],
}

/// Which controls a fit may use besides the two pigments.
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Pigments,
    /// The chroma scale and an undertone limited to this many degrees either way.
    Undertone(f64),
    /// As `Undertone`, and the unnatural colour as well.
    Unnatural(f64),
}

const MIN_SCALE: f64 = 0.1;
const MAX_SCALE: f64 = 1.6;
/// The cost, in CIEDE2000 units, of a chroma scale of 1/e or e. It makes the fit leave the scale at 1 when the pigments are enough.
const SCALE_PENALTY: f64 = 2.0;
/// The cost of an undertone at its limit.
const UNDERTONE_PENALTY: f64 = 0.5;
/// The cost per degree of unnatural colour.
const UNNATURAL_PENALTY: f64 = 0.02;

fn tone_of(x: &[f64], mode: Mode) -> BaseTone {
    let (melanin, haemoglobin) = amounts([x[0], x[1]]);
    let mut tone = BaseTone::from_pigments(melanin, haemoglobin);
    if let Mode::Undertone(cap) | Mode::Unnatural(cap) = mode {
        tone.chroma_scale = x[2].exp().clamp(MIN_SCALE, MAX_SCALE);
        tone.undertone_degrees = x[3].clamp(-cap, cap);
    }
    if matches!(mode, Mode::Unnatural(_)) {
        tone.unnatural_degrees = wrap(x[4]);
    }
    tone
}

pub fn fit(table: &PigmentTable, target: [f64; 3], mode: Mode) -> Fit {
    let cost = |x: &[f64]| {
        let out_of_range: f64 = x[..2].iter().map(|v| (v - v.clamp(0.0, 1.0)).abs()).sum();
        let tone = tone_of(x, mode);
        let mut c = delta_e_2000(tone.lab(table), target) + 100.0 * out_of_range;
        if let Mode::Undertone(cap) | Mode::Unnatural(cap) = mode {
            c += SCALE_PENALTY * tone.chroma_scale.ln().powi(2) + UNDERTONE_PENALTY * (tone.undertone_degrees / cap).powi(2);
            c += 100.0 * ((x[2].exp() - x[2].exp().clamp(MIN_SCALE, MAX_SCALE)).abs() + ((x[3] - x[3].clamp(-cap, cap)).abs() / cap));
        }
        if matches!(mode, Mode::Unnatural(_)) {
            c += UNNATURAL_PENALTY * tone.unnatural_degrees.abs();
        }
        c
    };
    let extra = match mode {
        Mode::Pigments => 0,
        Mode::Undertone(_) => 2,
        Mode::Unnatural(_) => 3,
    };
    let mut best = (f64::MAX, vec![0.0; 2 + extra]);
    for i in 0..SCAN {
        for j in 0..SCAN {
            let x = [i as f64 / (SCAN - 1) as f64, j as f64 / (SCAN - 1) as f64];
            let mut point = vec![x[0], x[1]];
            point.resize(2 + extra, 0.0);
            let c = cost(&point);
            if c < best.0 {
                best = (c, point);
            }
        }
    }
    let mut step = vec![0.02, 0.02];
    if extra > 0 {
        step.extend([0.1, 3.0]);
    }
    if extra > 2 {
        step.push(10.0);
    }
    // Restart a few times so the simplex does not stall on the flat ridge between the pigments and the extra controls.
    let mut x = best.1;
    for _ in 0..4 {
        x = nelder_mead(&x, &step, &cost);
    }
    let tone = tone_of(&x, mode);
    Fit { tone, lab: tone.lab(table) }
}

