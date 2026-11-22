//! The weights rule of a region: a shared fit over all heads, plus a linear correction from the head's own shape. Each head is also
//! fitted alone, pulled towards the shared fit. The differences between those fits and the shared fit are learned as a function of the
//! head's shape, measured on the region's free vertices and reduced to its main components.

use super::regions::Region;
use super::solve::{Equations, Prior, solve, solve_spd};
use crate::V3;
use base64::Engine;
use nalgebra::{DMatrix, SymmetricEigen};
use serde_json::{Value, json};
use std::collections::HashMap;

pub struct Settings {
    pub components: usize,
    pub ridge: f64,
    /// How strongly a head's change from the shared fit is smoothed over the mesh before it is learned.
    pub smoothing: f64,
    /// How strongly a head's own fit is pulled towards the shared fit.
    pub own_pull: f64,
}

pub const SETTINGS: Settings = Settings {
    components: 16,
    ridge: 1.0,
    smoothing: 300.0,
    own_pull: 1.0,
};

/// One head's input to the calibration.
pub struct Sample {
    pub name: String,
    pub pos: Vec<V3>,
    pub equations: Equations,
}

fn sum(samples: &[&Sample]) -> Equations {
    let mut eq = samples[0].equations.clone();
    for s in &samples[1..] {
        eq.add(&s.equations);
    }
    eq
}

fn shared_fit(region: &Region, samples: &[&Sample]) -> Vec<Vec<f64>> {
    solve(
        &sum(samples),
        &region.problem,
        Some(&Prior {
            strength: &region.hint_strength,
            weights: &region.hint,
        }),
    )
}

fn own_fit(region: &Region, sample: &Sample, shared: &[Vec<f64>], pull: f64) -> Vec<Vec<f64>> {
    let strength: Vec<f64> = region.hint_strength.iter().map(|m| m + pull).collect();
    let weights: Vec<Vec<f64>> = (0..shared.len())
        .map(|v| {
            let m = region.hint_strength[v];
            region.hint[v]
                .iter()
                .zip(&shared[v])
                .map(|(h, s)| (m * h + pull * s) / (m + pull).max(1e-12))
                .collect()
        })
        .collect();
    solve(
        &sample.equations,
        &region.problem,
        Some(&Prior {
            strength: &strength,
            weights: &weights,
        }),
    )
}

fn shape(region: &Region, pos: &[V3]) -> Vec<f64> {
    let a = pos[region.anchor];
    region
        .problem
        .free
        .iter()
        .flat_map(|&v| (0..3).map(move |k| pos[v][k] - a[k]))
        .collect()
}

/// Smooths each bone's column of a change over the free vertices, holding it at 0 outside: (I + s Q) d' = d, where Q ties edges.
fn smooth_change(region: &Region, d: &[Vec<f64>], strength: f64) -> Vec<Vec<f64>> {
    let free = &region.problem.free;
    let nb = region.problem.bones.len();
    let mut slot: HashMap<usize, usize> = HashMap::new();
    for (i, &v) in free.iter().enumerate() {
        slot.insert(v, i);
    }
    let mut lower: HashMap<(usize, usize), f64> = (0..free.len()).map(|i| ((i, i), 1.0)).collect();
    for &(a, b, t) in &region.plain_ties {
        let t = strength * t;
        for (p, q) in [(a, b), (b, a)] {
            if let Some(&i) = slot.get(&p) {
                *lower.entry((i, i)).or_default() += t;
                if let Some(&o) = slot.get(&q)
                    && p < q
                {
                    *lower.entry((i.max(o), i.min(o))).or_default() -= t;
                }
            }
        }
    }
    let cols: Vec<Vec<f64>> = (0..nb)
        .map(|j| {
            solve_spd(
                free.len(),
                &lower,
                &d.iter().map(|r| r[j]).collect::<Vec<_>>(),
            )
        })
        .collect();
    (0..free.len())
        .map(|i| (0..nb).map(|j| cols[j][i]).collect())
        .collect()
}

pub struct Rule {
    pub mean: Vec<f64>,
    pub basis: Vec<Vec<f64>>,
    pub scale: Vec<f64>,
    /// The base weights of the free vertices, flattened by vertex then bone.
    pub base: Vec<f64>,
    pub change: Vec<Vec<f64>>,
}

impl Rule {
    pub fn weights(&self, region: &Region, pos: &[V3]) -> Vec<Vec<f64>> {
        let x = shape(region, pos);
        let nb = region.problem.bones.len();
        let z: Vec<f64> = self
            .basis
            .iter()
            .zip(&self.scale)
            .map(|(b, s)| {
                x.iter()
                    .zip(&self.mean)
                    .zip(b)
                    .map(|((xi, mi), bi)| (xi - mi) * bi)
                    .sum::<f64>()
                    / s
            })
            .collect();
        let mut out = region.problem.fixed.clone();
        for (i, &v) in region.problem.free.iter().enumerate() {
            let mut w: Vec<f64> = (0..nb)
                .map(|j| {
                    let k = i * nb + j;
                    let x = self.base[k]
                        + z.iter()
                            .zip(&self.change)
                            .map(|(zc, c)| zc * c[k])
                            .sum::<f64>();
                    if region.problem.allowed[i][j] {
                        x.max(0.0)
                    } else {
                        0.0
                    }
                })
                .collect();
            let s: f64 = w.iter().sum();
            w.iter_mut().for_each(|x| *x /= s.max(1e-9));
            out[v] = w;
        }
        out
    }
}

fn learn(
    region: &Region,
    samples: &[&Sample],
    shared: &[Vec<f64>],
    own: &[&Vec<Vec<f64>>],
    s: &Settings,
) -> Rule {
    let free = &region.problem.free;
    let nb = region.problem.bones.len();
    let x: Vec<Vec<f64>> = samples.iter().map(|q| shape(region, &q.pos)).collect();
    let (h, dim) = (x.len(), x[0].len());
    let mean: Vec<f64> = (0..dim)
        .map(|k| x.iter().map(|r| r[k]).sum::<f64>() / h as f64)
        .collect();
    let xc = DMatrix::from_fn(h, dim, |i, k| x[i][k] - mean[k]);
    let eig = SymmetricEigen::new(&xc * xc.transpose());
    let mut order: Vec<usize> = (0..h).collect();
    order.sort_by(|a, b| eig.eigenvalues[*b].total_cmp(&eig.eigenvalues[*a]));
    let k = s.components.min(h - 1);
    let mut basis = Vec::new();
    let mut scale = Vec::new();
    for &c in &order[..k] {
        let sigma = eig.eigenvalues[c].max(1e-30).sqrt();
        let v = xc.transpose() * eig.eigenvectors.column(c) / sigma;
        basis.push(v.iter().copied().collect::<Vec<f64>>());
        scale.push(sigma / (h as f64).sqrt());
    }
    let z = DMatrix::from_fn(h, k, |i, c| {
        (0..dim)
            .map(|d| (x[i][d] - mean[d]) * basis[c][d])
            .sum::<f64>()
            / scale[c]
    });
    let deltas: Vec<Vec<f64>> = own
        .iter()
        .map(|w| {
            let d: Vec<Vec<f64>> = free
                .iter()
                .map(|&v| (0..nb).map(|j| w[v][j] - shared[v][j]).collect())
                .collect();
            let d = if s.smoothing > 0.0 {
                smooth_change(region, &d, s.smoothing)
            } else {
                d
            };
            d.iter()
                .zip(free)
                .flat_map(|(r, &v)| r.iter().map(move |x| x * region.keep[v]))
                .collect()
        })
        .collect();
    let m = deltas[0].len();
    let dm: Vec<f64> = (0..m)
        .map(|q| deltas.iter().map(|r| r[q]).sum::<f64>() / h as f64)
        .collect();
    let lhs = z.transpose() * &z + DMatrix::identity(k, k) * s.ridge;
    let dl = DMatrix::from_fn(h, m, |i, q| deltas[i][q] - dm[q]);
    let c = lhs.lu().solve(&(z.transpose() * dl)).expect("ridge system");
    let base = free
        .iter()
        .enumerate()
        .flat_map(|(i, &v)| (0..nb).map(move |j| (i, v, j)))
        .map(|(i, v, j)| shared[v][j] + dm[i * nb + j])
        .collect();
    Rule {
        mean,
        basis,
        scale,
        base,
        change: (0..k).map(|r| c.row(r).iter().copied().collect()).collect(),
    }
}

fn on_free(region: &Region, w: &[Vec<f64>]) -> Vec<Vec<f64>> {
    region.problem.free.iter().map(|&v| w[v].clone()).collect()
}

/// The rule of a region learned from all heads, with a five-fold test on heads left out of the learning.
pub fn calibrate(region: &Region, samples: &[Sample], s: &Settings) -> (Rule, String) {
    let all: Vec<&Sample> = samples.iter().collect();
    let shared = shared_fit(region, &all);
    let own: Vec<Vec<Vec<f64>>> = samples
        .iter()
        .map(|q| own_fit(region, q, &shared, s.own_pull))
        .collect();
    let fold = |i: usize| (i * 7 + 3) % 5;
    let (mut e_shared, mut e_rule, mut better) = (0.0, 0.0, 0);
    for f in 0..5 {
        let train: Vec<usize> = (0..samples.len()).filter(|&i| fold(i) != f).collect();
        let tr: Vec<&Sample> = train.iter().map(|&i| &samples[i]).collect();
        let sh = shared_fit(region, &tr);
        let ow: Vec<&Vec<Vec<f64>>> = train.iter().map(|&i| &own[i]).collect();
        let rule = learn(region, &tr, &sh, &ow, s);
        for i in (0..samples.len()).filter(|&i| fold(i) == f) {
            let a = samples[i].equations.error(&on_free(region, &sh));
            let b = samples[i]
                .equations
                .error(&on_free(region, &rule.weights(region, &samples[i].pos)));
            e_shared += a;
            e_rule += b;
            if b < a {
                better += 1;
            }
        }
    }
    let n = samples.len() as f64;
    let report = format!(
        "on heads left out: shared fit {:.3} mm, with the shape correction {:.3} mm, better on {better} of {}",
        e_shared / n,
        e_rule / n,
        samples.len()
    );
    let own_refs: Vec<&Vec<Vec<f64>>> = own.iter().collect();
    (learn(region, &all, &shared, &own_refs, s), report)
}

fn b64(v: impl Iterator<Item = f64>) -> String {
    base64::engine::general_purpose::STANDARD.encode(
        v.flat_map(|x| (x as f32).to_le_bytes())
            .collect::<Vec<u8>>(),
    )
}

/// The rule in the file format of `crate::weights::ModelFile`.
pub fn to_file(region: &Region, rule: &Rule, n: usize, bone_names: &[String]) -> Value {
    let free = &region.problem.free;
    json!({
        "n": n,
        "bones": region.problem.bones.iter().map(|&b| bone_names[b].clone()).collect::<Vec<_>>(),
        "lipLine": region.anchor,
        "shapeVertices": free,
        "outputs": free.iter().enumerate().map(|(i, &v)| (v, i)).collect::<Vec<_>>(),
        "components": rule.basis.len(),
        "mean": b64(rule.mean.iter().copied()),
        "basis": b64(rule.basis.iter().flatten().copied()),
        "scale": b64(rule.scale.iter().copied()),
        "base": b64(rule.base.iter().copied()),
        "change": b64(rule.change.iter().flatten().copied()),
        "allowed": base64::engine::general_purpose::STANDARD.encode(
            region.problem.allowed.iter().flatten().map(|&a| a as u8).collect::<Vec<u8>>()
        ),
    })
}
