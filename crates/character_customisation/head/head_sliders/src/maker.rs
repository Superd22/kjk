//! The slider maker of the kit: each named slider sets one measure of its feature and holds the others still.
//!
//! A slider's movement is built from smooth bumps placed on some parts of the feature's zone. Each bump is a Gaussian on the base head,
//! mirrored on the other side of the face, and moves the skin across, up or out. Among all mixes of the slider's bumps, the slider takes
//! the smoothest one that changes its own measure by the asked amount and changes every other measure, and the feature's outlines, as
//! little as it can.
//!
//! The mix is found step by step from 0 to each end of the slider's range. At each step the measure's response to every bump is
//! measured again, so the slider follows the feature as it changes, and what the other measures drifted is pulled back. The movement
//! at each step is kept as a key of the slider (`set::Key`), and the slider blends between keys, so it may bend along its range. Each
//! key is checked for folds and turned-over triangles. A step that makes one is tried again stiffer, then with the holds loosened, then
//! with more parts (the repair levels), and a side whose steps keep folding ends there, which caps the slider's range.

use crate::check::Checker;
use crate::feature::{Feature, Measured, Spec, outline_difference, outline_shape_gaps, row_of};
use crate::measure::Frame;
use crate::mesh::HeadMesh;
use crate::set::{Field, Key, SMOOTHING, Slider, SliderSet};
use crate::solve::{Evaluator, Scale};
use crate::zone::Zone;
use crate::{V3, add, length, scale, sub};
use head_shape::smooth::Smoother;
use nalgebra::{DMatrix, DVector};

/// The bumps, each a movement of the whole head in metres, and the part of the feature each belongs to.
pub struct Bumps {
    pub movements: Vec<Vec<V3>>,
    pub parts: Vec<&'static str>,
    pub centres: Vec<V3>,
}

/// Places the feature's bumps on the base head and builds their movements.
pub fn bumps(feature: &dyn Feature, mesh: &HeadMesh, zone: &Zone, base: &Measured) -> Bumps {
    let rule = feature.bumps();
    let frame = Frame::from_eyes(mesh.eyes);
    let local: Vec<V3> = mesh.first.iter().map(|&i| frame.local(mesh.base[i as usize])).collect();
    let mut centres: Vec<V3> = Vec::new();
    for (g, &p) in local.iter().enumerate() {
        if zone.weight[g] < rule.min_weight || mesh.hidden[g] || !(rule.allowed)(p) {
            continue;
        }
        if centres.iter().all(|c| length(sub(*c, p)) >= rule.spacing_mm) {
            centres.push(p);
        }
    }
    let to_world = |d: V3| -> V3 {
        let mut w = [0.0; 3];
        for a in 0..3 {
            w = add(w, scale(frame.axes[a], d[a] / frame.scale));
        }
        w
    };
    let mut movements = Vec::new();
    let mut parts = Vec::new();
    let mut at = Vec::new();
    for &c in &centres {
        let midline = c[0].abs() < 1.0;
        let mirror = [-c[0], c[1], c[2]];
        let g = |p: V3, q: V3| (-0.5 * (length(sub(p, q)) / rule.size_mm).powi(2)).exp();
        for axis in 0..3 {
            if midline && axis == 0 {
                continue;
            }
            let merged: Vec<V3> = local
                .iter()
                .enumerate()
                .map(|(v, &p)| {
                    let (a, b) = (g(p, c), if midline { 0.0 } else { g(p, mirror) });
                    let mut d = [0.0; 3];
                    d[axis] = if axis == 0 { a - b } else { a + b };
                    let d = to_world(scale(d, zone.weight[v]));
                    if length(d) < 1e-7 { [0.0; 3] } else { d }
                })
                .collect();
            let mut moved = mesh.expand(&merged);
            if !feature.carries_hidden() {
                for (v, m) in moved.iter_mut().enumerate() {
                    if mesh.hidden[mesh.group[v] as usize] {
                        *m = [0.0; 3];
                    }
                }
            }
            feature.shape_bump(mesh, &mut moved);
            movements.push(moved);
            parts.push(feature.part_of(c, base));
            at.push(c);
        }
    }
    Bumps { movements, parts, centres: at }
}

/// The bending of each mix of bumps: Q[a][b] is the sum over the head's vertices of the Laplacian of bump a times that of bump b, in
/// mm, scaled so the mean of the diagonal is 1.
pub fn bending(mesh: &HeadMesh, bumps: &Bumps) -> DMatrix<f64> {
    let lap: Vec<Vec<f64>> = bumps
        .movements
        .iter()
        .map(|m| {
            let u = mesh.merge(m);
            let mut out = Vec::with_capacity(u.len() * 3);
            for g in 0..u.len() {
                let nb = mesh.neighbours_of(g);
                for c in 0..3 {
                    let mean = if nb.is_empty() { u[g][c] } else { nb.iter().map(|&j| u[j as usize][c]).sum::<f64>() / nb.len() as f64 };
                    out.push((u[g][c] - mean) * 1000.0);
                }
            }
            out
        })
        .collect();
    let n = lap.len();
    let mut q = DMatrix::zeros(n, n);
    for a in 0..n {
        for b in a..n {
            let v: f64 = lap[a].iter().zip(&lap[b]).map(|(x, y)| x * y).sum();
            q[(a, b)] = v;
            q[(b, a)] = v;
        }
    }
    let mean = (0..n).map(|a| q[(a, a)]).sum::<f64>() / n as f64;
    q / mean
}

/// How strongly the other measures are held (each measured in pass marks), against the bending.
pub const HOLD_PROPORTIONS: f64 = 30.0;
pub const HOLD_OUTLINES: f64 = 6.0;
/// How much weaker a loose hold is, and how much weaker every hold is in the repair steps.
pub const LOOSE: f64 = 0.2;
/// The finite difference step of the measures' response, in mm of a bump. The measures read the surface every 0.25 mm, so a smaller
/// step sees their steps instead of their slope.
const JACOBIAN_MM: f64 = 1.0;
/// How much of the held rows' drift a step pulls back. A full pull back makes large moves where the measure is not linear.
const PULL: f64 = 0.3;
/// A small pull toward small mixes, so bumps no measure sees stay still.
const STILL: f64 = 1e-3;

pub struct Maker<'a> {
    pub feature: &'static dyn Feature,
    pub mesh: &'a HeadMesh,
    pub smoother: &'a Smoother,
    pub bumps: &'a Bumps,
    pub bending: &'a DMatrix<f64>,
    pub base_nose: &'a Measured,
    pub scale: &'a Scale,
    ev: Evaluator,
    checker: Checker,
}

/// A slider made by `Maker::make`, and what happened at each key.
pub struct Made {
    pub slider: Slider,
    /// Per key: its value, how it was made (0 strict, 1 with the holds loosened, 2 with more parts as well), the change of every
    /// proportion there (b minus base) and each outline's difference.
    pub keys: Vec<serde_json::Value>,
    /// Why each end of the range stops: "span" when it reached the asked span, otherwise the reason.
    pub ends: [String; 2],
}

/// The ways a step may be made, from the strictest: (loosened holds, extra parts).
const LEVELS: [(bool, bool); 3] = [(false, false), (true, false), (true, true)];

impl<'a> Maker<'a> {
    pub fn new(
        feature: &'static dyn Feature,
        mesh: &'a HeadMesh,
        smoother: &'a Smoother,
        bumps: &'a Bumps,
        bending: &'a DMatrix<f64>,
        base_nose: &'a Measured,
        scale: &'a Scale,
    ) -> Self {
        let set = SliderSet {
            vertex_count: mesh.base.len(),
            sliders: bumps.movements.iter().map(|m| Slider::from_movement("bump", "bump", "", m)).collect(),
            fields: Vec::new(),
        };
        let ev = Evaluator::new(feature, &set, smoother, &mesh.base, &mesh.triangles, Frame::from_eyes(mesh.eyes));
        let base_built = smoother.smooth(&mesh.base, SMOOTHING);
        let checker = Checker::new(mesh, &base_built, &ev.vertices);
        Self { feature, mesh, smoother, bumps, bending, base_nose, scale, ev, checker }
    }

    /// The residual rows: each proportion's change from the base nose over its pass mark, then each outline's change of shape (with
    /// each axis's position and size taken out) per point, over its pass mark and the square root of its point count.
    fn residual(&self, mix: &[f64]) -> Option<(Vec<f64>, Measured)> {
        let nose = self.ev.measure(mix).ok()?;
        let np = self.feature.proportions().len();
        let mut r: Vec<f64> =
            (0..np).map(|i| (nose.proportions[i] - self.base_nose.proportions[i]) / self.scale.proportions[i]).collect();
        if r.iter().any(|x| x.is_nan()) {
            return None;
        }
        for (o, outline) in self.feature.outlines().iter().enumerate() {
            if nose.outlines[o].is_empty() {
                return None;
            }
            let g = outline_shape_gaps(outline, &self.base_nose.outlines[o], &nose.outlines[o], self.scale.references[o]);
            let k = 1.0 / (self.scale.outlines[o] * (outline.points as f64).sqrt());
            r.extend(g.iter().map(|x| x * k));
        }
        Some((r, nose))
    }

    /// Which measure or outline each residual row belongs to: proportions by their index, outlines after them.
    fn row_owner(&self, r: usize) -> usize {
        let np = self.feature.proportions().len();
        if r < np {
            return r;
        }
        let mut at = np;
        for (o, outline) in self.feature.outlines().iter().enumerate() {
            at += 2 * outline.points;
            if r < at {
                return np + o;
            }
        }
        unreachable!()
    }

    /// The response of every residual row to each allowed bump, by central differences of `JACOBIAN_MM`.
    fn jacobian(&self, mix: &[f64], allowed: &[usize], rows: usize) -> DMatrix<f64> {
        let mut j = DMatrix::zeros(rows, allowed.len());
        let h = JACOBIAN_MM;
        for (col, &b) in allowed.iter().enumerate() {
            let mut up = mix.to_vec();
            up[b] += h;
            let mut down = mix.to_vec();
            down[b] -= h;
            if let (Some((ru, _)), Some((rd, _))) = (self.residual(&up), self.residual(&down)) {
                for r in 0..rows.min(ru.len()).min(rd.len()) {
                    j[(r, col)] = (ru[r] - rd[r]) / (2.0 * h);
                }
            }
        }
        j
    }

    pub fn built(&self, mix: &[f64]) -> Vec<V3> {
        let mut p = self.mesh.base.clone();
        for (m, &w) in self.bumps.movements.iter().zip(mix) {
            if w != 0.0 {
                for (q, d) in p.iter_mut().zip(m) {
                    *q = add(*q, scale(*d, w));
                }
            }
        }
        self.smoother.smooth(&p, SMOOTHING)
    }

    /// One step from `mix` toward `goal` units of measure `k`: the smoothest change of the allowed bumps that reaches the goal in
    /// the linear model, with the other rows held by `weight`, and the drift of the held rows pulled back.
    fn step(&self, mix: &[f64], k: usize, goal: f64, allowed: &[usize], weight: &[f64], stiffness: f64, pull: f64) -> Option<Vec<f64>> {
        let (r, _) = self.residual(mix)?;
        let rows = r.len();
        let j = self.jacobian(mix, allowed, rows);
        let q: DMatrix<f64> = DMatrix::from_fn(allowed.len(), allowed.len(), |a, b| self.bending[(allowed[a], allowed[b])]);
        let mut h = q * stiffness;
        let mut g = DVector::zeros(allowed.len());
        for row in 0..rows {
            if weight[row] == 0.0 {
                continue;
            }
            let jr = j.row(row);
            h += jr.transpose() * jr * weight[row];
            g += jr.transpose() * (weight[row] * pull * r[row]);
        }
        for d in 0..allowed.len() {
            h[(d, d)] += STILL;
        }
        let a = j.row(k).transpose().into_owned();
        let d = goal / self.scale.proportions[k] - r[k];
        let chol = h.cholesky()?;
        let hia = chol.solve(&a);
        let hig = chol.solve(&g);
        let denom = a.dot(&hia);
        if denom.abs() < 1e-12 {
            return None;
        }
        let nu = (d + a.dot(&hig)) / denom;
        let delta = hia * nu - hig;
        let mut next = mix.to_vec();
        for (col, &b) in allowed.iter().enumerate() {
            next[b] += delta[col];
        }
        Some(next)
    }

    /// Makes the slider of `spec`, with keys every `step` units of its measure out to `spans[0]` units below the base nose and
    /// `spans[1]` units above it.
    pub fn make(&self, spec: &Spec, spans: [f64; 2], step: f64) -> Made {
        let feature = self.feature;
        let np = feature.proportions().len();
        let k = crate::feature::proportion_index(feature, spec.measure);
        let nb = self.bumps.parts.len();
        let free: Vec<usize> = spec.free.iter().filter_map(|f| row_of(feature, f)).collect();
        let loose: Vec<usize> = spec.loose.iter().filter_map(|f| row_of(feature, f)).collect();
        let (r0, _) = self.residual(&vec![0.0; nb]).expect("the base nose");
        let rows = r0.len();
        let weights = |loosened: bool| -> Vec<f64> {
            (0..rows)
                .map(|r| {
                    let owner = self.row_owner(r);
                    if owner == k || free.contains(&owner) {
                        return 0.0;
                    }
                    let w = if owner < np { HOLD_PROPORTIONS } else { HOLD_OUTLINES };
                    let w = if loose.contains(&owner) { w * LOOSE } else { w };
                    if loosened { w * LOOSE } else { w }
                })
                .collect()
        };
        let strict = weights(false);
        let loosened = weights(true);
        let allowed_of = |more: bool| -> Vec<usize> {
            (0..nb)
                .filter(|&b| spec.parts.contains(&self.bumps.parts[b]) || (more && spec.repair_parts.contains(&self.bumps.parts[b])))
                .collect()
        };
        let allowed = [allowed_of(false), allowed_of(true)];
        let mut keys_out: Vec<(f64, Vec<f64>, usize)> = vec![(0.0, vec![0.0; nb], 0)];
        let mut ends = [String::new(), String::new()];
        for (side, sign) in [(0usize, -1.0), (1, 1.0)] {
            let span = spans[side];
            let mut mix = vec![0.0; nb];
            let mut reached: f64 = 0.0;
            let mut why = "span".to_string();
            while reached.abs() < span - 0.05 * step {
                let mut accepted = None;
                let mut reasons: std::collections::BTreeMap<String, usize> = Default::default();
                'sizes: for size in [step, 0.5 * step, 0.25 * step] {
                    let goal = sign * (reached.abs() + size).min(span);
                    for (level, &(loosen, more)) in LEVELS.iter().enumerate() {
                        if more && spec.repair_parts.is_empty() {
                            continue;
                        }
                        let weight = if loosen { &loosened } else { &strict };
                        for (stiffness, pull) in [(1.0, PULL), (4.0, PULL), (16.0, 0.0)] {
                            let Some(next) = self.step(&mix, k, goal, &allowed[more as usize], weight, stiffness, pull) else {
                                *reasons.entry("the measure stopped answering".into()).or_default() += 1;
                                continue;
                            };
                            let Some((r, _)) = self.residual(&next) else {
                                *reasons.entry("the measure could not read the nose".into()).or_default() += 1;
                                continue;
                            };
                            let got = r[k] * self.scale.proportions[k];
                            if std::env::var("SLIDERS_DEBUG").is_ok() {
                                eprintln!("{} side {side} level {level} size {size:.2} stiffness {stiffness}: asked {goal:.2}, got {got:.2} from {reached:.2}", spec.measure);
                            }
                            if (got - reached) * sign <= 0.25 * size {
                                *reasons.entry("the measure stopped growing".into()).or_default() += 1;
                                continue;
                            }
                            let faults = self.checker.faults(&self.built(&next));
                            if faults.total() > 0 && std::env::var("NOSE_DEBUG").is_ok() {
                                let f = Frame::from_eyes(self.mesh.eyes);
                                let at: Vec<String> = faults.at.iter().take(4).map(|p| { let q = f.local(*p); format!("({:.0},{:.0},{:.0})", q[0], q[1], q[2]) }).collect();
                                eprintln!("{} faults {} {} {} at {}", spec.measure, faults.flipped, faults.folded, faults.nostrils_flipped, at.join(" "));
                            }
                            if faults.total() > 0 {
                                let kind = if faults.nostrils_flipped > 0 {
                                    "a nostril turned over"
                                } else if faults.flipped > 0 {
                                    "skin turned over"
                                } else {
                                    "a fold"
                                };
                                *reasons.entry(kind.into()).or_default() += 1;
                                continue;
                            }
                            accepted = Some((got, next, level));
                            break 'sizes;
                        }
                    }
                }
                let Some((got, next, level)) = accepted else {
                    why = reasons.iter().map(|(r, n)| format!("{r} ({n})")).collect::<Vec<_>>().join(", ");
                    break;
                };
                reached = got;
                mix = next;
                keys_out.push((got, mix.clone(), level));
                why = "span".into();
            }
            ends[side] = why;
        }
        keys_out.sort_by(|a, b| a.0.total_cmp(&b.0));
        let keys: Vec<Key> = keys_out
            .iter()
            .map(|(v, mix, _)| {
                let (fields, weights): (Vec<u32>, Vec<f32>) =
                    (0..nb).filter(|&b| mix[b] != 0.0).map(|b| (b as u32, mix[b] as f32)).unzip();
                Key { value: *v, fields, weights }
            })
            .collect();
        let p = &feature.proportions()[k];
        let mut slider = Slider::from_movement(spec.name, "named", &format!("1 {} of {}", p.unit, p.label.to_lowercase()), &[]);
        slider.zone = feature.key().into();
        slider.hint = spec.hint.into();
        slider.measure = spec.measure.into();
        slider.group = spec.group.into();
        slider.low = keys.first().map(|k| k.value).unwrap_or(0.0);
        slider.high = keys.last().map(|k| k.value).unwrap_or(0.0);
        slider.keys = keys;
        let described = keys_out
            .iter()
            .map(|(v, mix, level)| {
                let mut d = self.describe(mix);
                d["value"] = serde_json::json!(v);
                d["level"] = serde_json::json!(level);
                d
            })
            .collect();
        Made { slider, keys: described, ends }
    }

    /// What a mix does: each proportion's change from the base feature, each outline's difference, and its faults.
    pub fn describe(&self, mix: &[f64]) -> serde_json::Value {
        match self.ev.measure(mix) {
            Ok(n) => {
                let p: Vec<f64> = (0..n.proportions.len()).map(|i| n.proportions[i] - self.base_nose.proportions[i]).collect();
                let o: Vec<f64> = self
                    .feature
                    .outlines()
                    .iter()
                    .enumerate()
                    .map(|(o, def)| outline_difference(def, &self.base_nose.outlines[o], &n.outlines[o], self.scale.references[o]))
                    .collect();
                serde_json::json!({"proportions": p, "outlines": o, "marks": n.marks})
            }
            Err(e) => serde_json::json!({ "error": e }),
        }
    }
}

/// A drawn slider (`Feature::drawn_step`): its keys hold the step's amount every `step` units of its measure out to `spans`, each the
/// amount that changes the measure by that much on the median head among `starts` (the heads before smoothing; the base head and the
/// stand-ins). The step's effect differs from head to head (a soft brow edge sharpens more than one that is already sharp), and the
/// median head stands for a typical one better than the base head, whose brow edge is softer than most. A side ends early where the
/// measure stops answering, or where a key folds or turns over a triangle on more than a twentieth of the heads.
#[allow(clippy::too_many_arguments)]
pub fn make_drawn(
    feature: &dyn Feature,
    mesh: &HeadMesh,
    smoother: &Smoother,
    starts: &[&[V3]],
    spec: &Spec,
    spans: [f64; 2],
    step: f64,
    apply: crate::set::DrawnStep,
) -> Made {
    let k = crate::feature::proportion_index(feature, spec.measure);
    let frame = Frame::from_eyes(mesh.eyes);
    let all: Vec<u32> = (0..mesh.base.len() as u32).collect();
    let heads: Vec<(Vec<V3>, Checker, f64)> = starts
        .iter()
        .filter_map(|s| {
            let p = smoother.smooth(s, SMOOTHING);
            let m = feature.measure(&crate::measure::Surface::within(&p, &mesh.triangles, &frame, &feature.window())).ok()?;
            let at0 = m.proportions[k];
            at0.is_finite().then(|| (p.clone(), Checker::new(mesh, &p, &all), at0))
        })
        .collect();
    let built = |h: &(Vec<V3>, Checker, f64), a: f64| -> Vec<V3> {
        let mut p = h.0.clone();
        apply(&mut p, &all, &mesh.triangles, a);
        p
    };
    let change = |a: f64| -> Option<f64> {
        let mut c: Vec<f64> = crate::solve::each(heads.len(), |i| {
            let h = &heads[i];
            feature
                .measure(&crate::measure::Surface::within(&built(h, a), &mesh.triangles, &frame, &feature.window()))
                .map_or(f64::NAN, |m| m.proportions[k] - h.2)
        })
        .into_iter()
        .filter(|v| v.is_finite())
        .collect();
        if c.len() < heads.len().div_ceil(2) {
            return None;
        }
        c.sort_by(f64::total_cmp);
        Some(c[c.len() / 2])
    };
    let folding = |a: f64| -> (usize, usize) {
        let f: Vec<usize> = crate::solve::each(heads.len(), |i| heads[i].1.faults(&built(&heads[i], a)).total());
        (f.iter().filter(|&&n| n > 0).count(), f.iter().sum())
    };
    let mut keys_out: Vec<(f64, f64)> = vec![(0.0, 0.0)];
    let mut ends = [String::new(), String::new()];
    // the amount grows 0.05 at a time, 0.05 mm of the step's largest movement
    const DA: f64 = 0.05;
    for (side, sign) in [(0usize, -1.0), (1, 1.0)] {
        let (mut a, mut got) = (0.0, 0.0);
        let mut next = step;
        ends[side] = "span".into();
        while next <= spans[side] + 0.5 * step {
            let a1 = a + sign * DA;
            let Some(g1) = change(a1) else {
                ends[side] = "the measure fails".into();
                break;
            };
            if sign * g1 < sign * got - 0.05 * step {
                ends[side] = "the measure stops answering".into();
                break;
            }
            if sign * g1 >= next {
                let at = a + (a1 - a) * (next - sign * got) / (sign * g1 - sign * got).max(1e-9);
                let (faulty, faults) = folding(at);
                if faulty * 20 > heads.len() {
                    ends[side] = format!("{faults} faulty triangles on {faulty} heads");
                    break;
                }
                keys_out.push((sign * next, at));
                next += step;
                continue;
            }
            if a1.abs() > 40.0 {
                ends[side] = "the measure barely answers".into();
                break;
            }
            a = a1;
            got = g1;
        }
    }
    keys_out.sort_by(|a, b| a.0.total_cmp(&b.0));
    let p = &feature.proportions()[k];
    let mut slider = Slider::from_movement(spec.name, "drawn", &format!("1 {} of {}", p.unit, p.label.to_lowercase()), &[]);
    slider.zone = feature.key().into();
    slider.hint = spec.hint.into();
    slider.measure = spec.measure.into();
    slider.group = spec.group.into();
    slider.keys = keys_out.iter().map(|&(v, a)| Key { value: v, fields: Vec::new(), weights: vec![a as f32] }).collect();
    slider.low = keys_out.first().map_or(0.0, |k| k.0);
    slider.high = keys_out.last().map_or(0.0, |k| k.0);
    let keys = keys_out.iter().map(|&(v, a)| serde_json::json!({"value": v, "amount": a})).collect();
    Made { slider, keys, ends }
}

/// The shared fields of a set made of these bumps.
pub fn fields(bumps: &Bumps) -> Vec<Field> {
    bumps.movements.iter().map(|m| Field::from_movement(m)).collect()
}
