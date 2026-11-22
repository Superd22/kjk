//! The slider values whose feature best matches a measured feature, for any feature of the kit.
//!
//! The smoothing is linear, so a built head is its smoothed start plus each value times its slider's smoothed movement. The movements
//! are smoothed once for all heads (`Smoothed`) and the start once per head, and only the vertices in the feature's window are kept,
//! so measuring a new set of values costs about ten milliseconds. The values are found by Levenberg and Marquardt's method over all sliders at once.

use crate::V3;
use crate::feature::{Feature, Measured, outline_gaps};
pub use crate::feature::Scale;
use crate::measure::{Frame, Surface};
use crate::set::{SMOOTHING, SliderSet};
use head_shape::smooth::Smoother;

/// Every slider's movement and every shared field of a set, smoothed once over the whole head. The smoothing is the same for every
/// head, so the 57 heads of a bench share it. A slider is found at its own place in the set, or else by its name, so the same set in
/// another order, or its first sliders, use it too.
pub struct Smoothed {
    names: Vec<String>,
    basis: Vec<Option<Vec<V3>>>,
    fields: std::sync::Arc<Vec<Vec<V3>>>,
}

impl Smoothed {
    pub fn new(set: &SliderSet, smoother: &Smoother) -> Self {
        let n = set.vertex_count;
        let basis = each(set.sliders.len(), |j| {
            let s = &set.sliders[j];
            s.keys.is_empty().then(|| if s.kind == "after" { s.movement(n) } else { smoother.smooth(&s.movement(n), SMOOTHING) })
        });
        let fields = each(set.fields.len(), |f| smoother.smooth(&set.fields[f].movement(n), SMOOTHING));
        Self { names: set.sliders.iter().map(|s| s.name.clone()).collect(), basis, fields: std::sync::Arc::new(fields) }
    }

    fn movement(&self, index: usize, name: &str) -> Option<&Vec<V3>> {
        let j = if self.names.get(index).is_some_and(|n| n == name) {
            index
        } else {
            self.names.iter().position(|n| n == name).unwrap_or_else(|| panic!("slider {name} was not smoothed"))
        };
        self.basis[j].as_ref()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn each<T: Send>(n: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    crate::heads::parallel(n, crate::heads::threads(), f)
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn each<T: Send>(n: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    (0..n).map(f).collect()
}

/// A head's start and its sliders' movements, smoothed and cut down to the vertices in the feature's window.
pub struct Evaluator {
    pub feature: &'static dyn Feature,
    pub frame: Frame,
    pub vertices: Vec<u32>,
    triangles: Vec<[u32; 3]>,
    start: Vec<V3>,
    /// Per slider, its smoothed movement when it has no keys.
    basis: Vec<Option<Vec<V3>>>,
    /// The set's shared fields, smoothed over the whole head; only the window's vertices are read.
    fields: std::sync::Arc<Vec<Vec<V3>>>,
    set: SliderSet,
    /// For a set with eye sliders: the press of the lids' edges onto the eyeballs, and each head vertex's place in `vertices`
    /// (`u32::MAX` when it is not kept). The measure then reads the pressed head, the head the page shows.
    press: Option<(&'static crate::eye_curves::Press, std::sync::Arc<Vec<u32>>)>,
    /// For the eyes: the places in `vertices` of the eyeballs' centre rings, 8 for the +x eye then 8 for the -x eye.
    rings: Vec<u32>,
    /// For the eyes, when the lid pieces and caruncles are measured: the places of the sliders Upper lid cover and Caruncle width in the
    /// set (`usize::MAX` without one).
    cover: Option<[usize; 2]>,
    /// The set's drawn sliders (`set::DrawnStep`), by their place in the set, applied after the press as `SliderSet::build` applies
    /// them. `vertices` then holds the eyeballs' centre rings too, which their steps read.
    drawn: Vec<(usize, crate::set::DrawnStep)>,
}

impl Evaluator {
    /// One head, with the set's movements smoothed for it alone.
    pub fn new(
        feature: &'static dyn Feature,
        set: &SliderSet,
        smoother: &Smoother,
        start: &[V3],
        triangles: &[[u32; 3]],
        frame: Frame,
    ) -> Self {
        Self::with(feature, set, &Smoothed::new(set, smoother), smoother, start, triangles, frame)
    }

    /// One head, with movements smoothed once for many heads.
    pub fn with(
        feature: &'static dyn Feature,
        set: &SliderSet,
        smoothed: &Smoothed,
        smoother: &Smoother,
        start: &[V3],
        triangles: &[[u32; 3]],
        frame: Frame,
    ) -> Self {
        let smoothed_start = smoother.smooth(start, SMOOTHING);
        let local: Vec<V3> = smoothed_start.iter().map(|&p| frame.local(p)).collect();
        let window = feature.window();
        let near = |p: V3| window.contains(p);
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
        // the eyes' measure places each eyeball at its centre ring's mean, as `Surface::within` does on a whole head
        let rings: Vec<u32> = if feature.key() == "eyes" && start.len() == crate::measure::OUR_VERTEX_COUNT {
            crate::measure::OUR_RINGS.iter().flat_map(|r| r.clone().map(|i| i as u32)).collect()
        } else {
            Vec::new()
        };
        for &i in &rings {
            if index[i as usize] == u32::MAX {
                index[i as usize] = vertices.len() as u32;
                vertices.push(i);
            }
        }
        let rings: Vec<u32> = rings.iter().map(|&i| index[i as usize]).collect();
        let drawn: Vec<(usize, crate::set::DrawnStep)> = set
            .sliders
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == "drawn")
            .filter_map(|(j, s)| Some((j, crate::set::drawn_step(&s.measure)?)))
            .collect();
        if !drawn.is_empty() && start.len() == crate::measure::OUR_VERTEX_COUNT {
            for i in crate::measure::OUR_RINGS.iter().flat_map(|r| r.clone()) {
                if index[i] == u32::MAX {
                    index[i] = vertices.len() as u32;
                    vertices.push(i as u32);
                }
            }
        }
        // the eyes' measure sees the generated upper lid pieces too, made from the whole head
        let pieces = feature.key() == "eyes" && start.len() == crate::measure::OUR_VERTEX_COUNT && crate::eye_curves::has_lid_rig();
        if pieces {
            for i in 0..start.len() {
                if index[i] == u32::MAX {
                    index[i] = vertices.len() as u32;
                    vertices.push(i as u32);
                }
            }
        }
        let cover = pieces.then(|| {
            [crate::eyes::COVER_SLIDER, "Caruncle width"].map(|n| set.sliders.iter().position(|s| s.name == n).unwrap_or(usize::MAX))
        });
        let press = crate::eye_curves::prepared().filter(|_| set.sliders.iter().any(|s| s.zone == "eyes")).map(|pr| {
            for i in pr.read_vertices() {
                if index[i as usize] == u32::MAX {
                    index[i as usize] = vertices.len() as u32;
                    vertices.push(i);
                }
            }
            (pr, std::sync::Arc::new(index.clone()))
        });
        let pick = |p: &[V3]| vertices.iter().map(|&i| p[i as usize]).collect::<Vec<V3>>();
        let basis = set
            .sliders
            .iter()
            .enumerate()
            .map(|(j, s)| if s.keys.is_empty() { smoothed.movement(j, &s.name).map(|m| pick(m)) } else { None })
            .collect();
        let set = SliderSet { vertex_count: set.vertex_count, sliders: set.sliders.clone(), fields: Vec::new() };
        Self { feature, frame, start: pick(&smoothed_start), vertices, triangles, basis, fields: smoothed.fields.clone(), set, press, rings, cover, drawn }
    }

    /// The same head with only its first `count` sliders.
    pub fn first(&self, count: usize) -> Evaluator {
        let count = count.min(self.basis.len());
        Evaluator {
            feature: self.feature,
            frame: self.frame,
            vertices: self.vertices.clone(),
            triangles: self.triangles.clone(),
            start: self.start.clone(),
            basis: self.basis[..count].to_vec(),
            fields: self.fields.clone(),
            set: SliderSet { vertex_count: self.set.vertex_count, sliders: self.set.sliders[..count].to_vec(), fields: Vec::new() },
            press: self.press.clone(),
            rings: self.rings.clone(),
            cover: self.cover,
            drawn: self.drawn.iter().copied().filter(|(j, _)| *j < count).collect(),
        }
    }

    pub fn slider_count(&self) -> usize {
        self.basis.len()
    }

    /// The built head's vertices near the feature, in the order of `vertices`, pressed as `SliderSet::build` presses them.
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
                    for (q, &i) in p.iter_mut().zip(&self.vertices) {
                        let d = f[i as usize];
                        for c in 0..3 {
                            q[c] += v * d[c];
                        }
                    }
                }
            }
        }
        if let Some((pr, slot)) = &self.press {
            pr.apply_some(&mut p, slot);
        }
        for &(j, step) in &self.drawn {
            let amount = self.set.sliders[j].drawn_amount(values.get(j).copied().unwrap_or(0.0));
            if amount != 0.0 {
                step(&mut p, &self.vertices, &self.triangles, amount);
            }
        }
        p
    }

    pub fn measure(&self, values: &[f64]) -> Result<Measured, &'static str> {
        self.feature.measure(&self.surface(values))
    }

    /// The surface that `measure` reads.
    pub fn surface(&self, values: &[f64]) -> Surface {
        let p = self.positions(values);
        let tris = self.triangles.iter().map(|t| t.map(|i| self.frame.local(p[i as usize]))).collect();
        let mut surface = Surface::from_local_in(tris, &self.feature.window());
        if self.rings.len() == 16 {
            let centre = |r: &[u32]| {
                let sum = r.iter().fold([0.0; 3], |a, &i| crate::add(a, p[i as usize]));
                self.frame.local(crate::scale(sum, 1.0 / r.len() as f64))
            };
            surface.eyes = Some([centre(&self.rings[..8]), centre(&self.rings[8..])]);
            surface.scale = self.frame.scale;
        }
        if let Some(c) = self.cover {
            let mut full = vec![[0.0; 3]; self.set.vertex_count];
            for (q, &i) in p.iter().zip(&self.vertices) {
                full[i as usize] = *q;
            }
            surface.lids = crate::eyes::piece_triangles(&full, &self.frame, c.map(|j| values.get(j).copied().unwrap_or(0.0)));
        }
        surface
    }
}

/// No value goes past this many spreads from 0, so the search cannot run off to a feature the measure misreads.
pub const BOUND: f64 = 5.0;

/// The gaps between a measured feature and its target, each divided by its pass mark: one per proportion, then two per outline point
/// divided by the square root of the outline's point count, so their squares add up to the outline's squared difference over its mark.
/// A row the target does not have (hidden on KOTOR's head) counts 0.
pub fn gaps(feature: &dyn Feature, m: &Measured, target: &Measured, scale: &Scale) -> Vec<f64> {
    let mut r: Vec<f64> = (0..feature.proportions().len())
        .map(|i| {
            let d = feature.gap_weight(feature.proportions()[i].key) * (m.proportions[i] - target.proportions[i]) / scale.proportions[i];
            if d.is_nan() { 0.0 } else { d }
        })
        .collect();
    for (o, outline) in feature.outlines().iter().enumerate() {
        if target.outlines[o].is_empty() || m.outlines[o].is_empty() {
            r.extend(std::iter::repeat_n(0.0, 2 * outline.points));
            continue;
        }
        let g = outline_gaps(outline, &target.outlines[o], &m.outlines[o], scale.references[o]);
        let k = 1.0 / (scale.outlines[o] * (outline.points as f64).sqrt());
        r.extend(g.iter().map(|x| x * k));
    }
    r
}

pub struct Solution {
    pub values: Vec<f64>,
    pub measured: Option<Measured>,
    pub cost: f64,
    pub rounds: usize,
}

/// Finds the slider values whose nose best matches `target`. `spread` is each slider's spread over the KOTOR heads, used for the
/// finite differences and for a weak pull toward 0 (`prior` times the squared values in spreads), which keeps the values off
/// directions that change nothing measured. `start` is where the search begins.
pub fn solve(
    ev: &Evaluator,
    target: &Measured,
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
    target: &Measured,
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
    let residual = |v: &[f64]| -> Option<(Vec<f64>, Measured)> {
        let nose = ev.measure(v).ok()?;
        let mut r = gaps(ev.feature, &nose, target, scale);
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
            measured: None,
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
        measured: Some(nose),
        cost: c,
        rounds,
    }
}
