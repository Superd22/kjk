//! The slider values whose nose best matches a measured nose.
//!
//! The smoothing is linear, so a built head is its smoothed start plus each value times its slider's smoothed movement. Both are
//! computed once per head, and only the vertices near the nose are kept, so measuring a new set of values costs about a millisecond.
//! The values are found by Levenberg and Marquardt's method over all sliders at once, from each proportion's own difference (the
//! proportion sliders are one millimetre of their proportion per unit on the base head).

use crate::V3;
use crate::measure::{Frame, N_PROPORTIONS, Nose, OUTLINES, Surface, measure, outline_gaps};
use crate::set::{SMOOTHING, SliderSet};
use head_shape::smooth::Smoother;

/// How the gaps between two noses count: each proportion's gap divided by its pass mark, and each outline's root mean square gap
/// divided by its pass mark.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Scale {
    pub proportions: [f64; N_PROPORTIONS],
    pub outlines: [f64; 3],
    /// The outlines' reference sizes in mm.
    pub references: [f64; 3],
}

/// A head's start and its sliders' movements, smoothed and cut down to the vertices near the nose.
pub struct Evaluator {
    pub frame: Frame,
    pub vertices: Vec<u32>,
    triangles: Vec<[u32; 3]>,
    start: Vec<V3>,
    /// Per slider, its smoothed movement when it has no keys.
    basis: Vec<Option<Vec<V3>>>,
    /// The set's shared fields, smoothed, and the sliders whose keys mix them.
    fields: std::sync::Arc<Vec<Vec<V3>>>,
    set: SliderSet,
}

impl Evaluator {
    pub fn new(set: &SliderSet, smoother: &Smoother, start: &[V3], triangles: &[[u32; 3]], frame: Frame) -> Self {
        let smoothed = smoother.smooth(start, SMOOTHING);
        let local: Vec<V3> = smoothed.iter().map(|&p| frame.local(p)).collect();
        let near = |p: V3| p[0].abs() < 45.0 && p[1] > -110.0 && p[1] < 35.0 && p[2] > -25.0;
        let kept: Vec<[u32; 3]> =
            triangles.iter().copied().filter(|t| t.iter().any(|&i| near(local[i as usize]))).collect();
        let mut index = vec![u32::MAX; start.len()];
        let mut vertices = Vec::new();
        for t in &kept {
            for &i in t {
                if index[i as usize] == u32::MAX {
                    index[i as usize] = vertices.len() as u32;
                    vertices.push(i);
                }
            }
        }
        let triangles = kept.iter().map(|t| t.map(|i| index[i as usize])).collect();
        let pick = |p: &[V3]| vertices.iter().map(|&i| p[i as usize]).collect::<Vec<V3>>();
        let basis = set
            .sliders
            .iter()
            .map(|s| s.keys.is_empty().then(|| pick(&smoother.smooth(&s.movement(start.len()), SMOOTHING))))
            .collect();
        let fields = set.fields.iter().map(|f| pick(&smoother.smooth(&f.movement(start.len()), SMOOTHING))).collect();
        let set = SliderSet { vertex_count: set.vertex_count, sliders: set.sliders.clone(), fields: Vec::new() };
        Self { frame, start: pick(&smoothed), vertices, triangles, basis, fields: std::sync::Arc::new(fields), set }
    }

    /// The same head with only its first `count` sliders.
    pub fn first(&self, count: usize) -> Evaluator {
        let count = count.min(self.basis.len());
        Evaluator {
            frame: self.frame,
            vertices: self.vertices.clone(),
            triangles: self.triangles.clone(),
            start: self.start.clone(),
            basis: self.basis[..count].to_vec(),
            fields: self.fields.clone(),
            set: SliderSet { vertex_count: self.set.vertex_count, sliders: self.set.sliders[..count].to_vec(), fields: Vec::new() },
        }
    }

    pub fn slider_count(&self) -> usize {
        self.basis.len()
    }

    /// The built head's vertices near the nose, in the order of `vertices`.
    pub fn positions(&self, values: &[f64]) -> Vec<V3> {
        let mut p = self.start.clone();
        let add = |p: &mut Vec<V3>, b: &[V3], v: f64| {
            for (q, d) in p.iter_mut().zip(b) {
                for c in 0..3 {
                    q[c] += v * d[c];
                }
            }
        };
        for (b, &v) in self.basis.iter().zip(values) {
            if let Some(b) = b
                && v != 0.0
            {
                add(&mut p, b, v);
            }
        }
        if !self.fields.is_empty() {
            let mut w = vec![0.0; self.fields.len()];
            for (s, &v) in self.set.sliders.iter().zip(values) {
                s.add_key_weights(v, &mut w);
            }
            for (f, &v) in self.fields.iter().zip(&w) {
                if v != 0.0 {
                    add(&mut p, f, v);
                }
            }
        }
        p
    }

    pub fn measure(&self, values: &[f64]) -> Result<Nose, &'static str> {
        let p = self.positions(values);
        let tris = self
            .triangles
            .iter()
            .map(|t| t.map(|i| self.frame.local(p[i as usize])))
            .collect();
        measure(&Surface::from_local(tris))
    }
}

/// No value goes past this many spreads from 0, so the search cannot run off to a nose the measure misreads.
pub const BOUND: f64 = 5.0;

/// The gaps between a nose and a target nose, each divided by its pass mark: one per proportion, then two per outline point divided
/// by the square root of the outline's point count, so their squares add up to the outline's squared difference over its mark.
pub fn gaps(nose: &Nose, target: &Nose, scale: &Scale) -> Vec<f64> {
    let mut r: Vec<f64> = (0..N_PROPORTIONS)
        .map(|i| (nose.proportions[i] - target.proportions[i]) / scale.proportions[i])
        .collect();
    for (o, outline) in OUTLINES.iter().enumerate() {
        let g = outline_gaps(o, &target.outlines[o], &nose.outlines[o], scale.references[o]);
        let k = 1.0 / (scale.outlines[o] * (outline.points as f64).sqrt());
        r.extend(g.iter().map(|x| x * k));
    }
    r
}

pub struct Solution {
    pub values: Vec<f64>,
    pub nose: Option<Nose>,
    pub cost: f64,
    pub rounds: usize,
}

/// Finds the slider values whose nose best matches `target`. `spread` is each slider's spread over the KOTOR heads, used for the
/// finite differences and for a weak pull toward 0 (`prior` times the squared values in spreads), which keeps the values off
/// directions that change nothing measured. `start` is where the search begins.
pub fn solve(
    ev: &Evaluator,
    target: &Nose,
    scale: &Scale,
    spread: &[f64],
    prior: f64,
    start: &[f64],
) -> Solution {
    let bounds: Vec<[f64; 2]> = spread.iter().map(|s| [-BOUND * s, BOUND * s]).collect();
    solve_within(ev, target, scale, spread, prior, start, &bounds)
}

/// Like `solve`, with each value kept between its own `bounds`.
pub fn solve_within(
    ev: &Evaluator,
    target: &Nose,
    scale: &Scale,
    spread: &[f64],
    prior: f64,
    start: &[f64],
    bounds: &[[f64; 2]],
) -> Solution {
    let k = ev.slider_count();
    let mut v: Vec<f64> = (0..k)
        .map(|i| start.get(i).copied().unwrap_or(0.0).clamp(bounds[i][0], bounds[i][1]))
        .collect();
    let residual = |v: &[f64]| -> Option<(Vec<f64>, Nose)> {
        let nose = ev.measure(v).ok()?;
        let mut r = gaps(&nose, target, scale);
        r.extend(v.iter().zip(spread).map(|(x, s)| prior.sqrt() * x / s));
        Some((r, nose))
    };
    let cost = |r: &[f64]| r.iter().map(|x| x * x).sum::<f64>();
    let Some((mut r, mut nose)) = residual(&v).or_else(|| {
        v.iter_mut().for_each(|x| *x = 0.0);
        residual(&v)
    }) else {
        return Solution {
            values: v,
            nose: None,
            cost: f64::INFINITY,
            rounds: 0,
        };
    };
    let mut c = cost(&r);
    let mut mu = 1e-2;
    let mut rounds = 0;
    for _ in 0..30 {
        rounds += 1;
        let m = r.len();
        let mut jac = vec![vec![0.0; k]; m];
        for j in 0..k {
            let h = 0.05 * spread[j];
            let h = if v[j] + h > bounds[j][1] { -h } else { h };
            let mut w = v.clone();
            w[j] += h;
            if let Some((rj, _)) = residual(&w) {
                for i in 0..m.min(rj.len()) {
                    jac[i][j] = (rj[i] - r[i]) / h;
                }
            }
        }
        let mut jtj = nalgebra::DMatrix::<f64>::zeros(k, k);
        let mut jtr = nalgebra::DVector::<f64>::zeros(k);
        for i in 0..m {
            for a in 0..k {
                jtr[a] += jac[i][a] * r[i];
                for b in 0..k {
                    jtj[(a, b)] += jac[i][a] * jac[i][b];
                }
            }
        }
        let mut improved = false;
        for _ in 0..8 {
            let mut a = jtj.clone();
            for d in 0..k {
                a[(d, d)] += mu * (jtj[(d, d)] + 1e-9);
            }
            let Some(step) = a.cholesky().map(|ch| ch.solve(&(-&jtr))) else {
                mu *= 10.0;
                continue;
            };
            let w: Vec<f64> = v
                .iter()
                .zip(step.iter())
                .zip(bounds)
                .map(|((x, s), b)| (x + s).clamp(b[0], b[1]))
                .collect();
            if let Some((rw, nw)) = residual(&w) {
                let cw = cost(&rw);
                if cw < c {
                    let gain = c - cw;
                    v = w;
                    r = rw;
                    nose = nw;
                    c = cw;
                    mu = (mu * 0.3).max(1e-6);
                    improved = gain > 1e-6 * c.max(1e-9);
                    break;
                }
            }
            mu *= 10.0;
        }
        if !improved {
            break;
        }
    }
    Solution {
        values: v,
        nose: Some(nose),
        cost: c,
        rounds,
    }
}
