//! The construction of one eye, as in the prototype's `curve.rs`. The changes are for speed only: the topology is found once per mesh,
//! the curve's flat outline is computed once per build, and the height field's distance to the curve is only measured where it is used.

use crate::maker::Mesh;
use crate::{V3, add, cross, dot, length, normalize, scale, sub};
use std::collections::{HashMap, HashSet};

/// Merged vertices of one eye: the lid edge loop in order, each loop vertex's share along its lid and whether it is on the upper lid,
/// the vertices inside the loop, and the head's other vertices by their merged-graph distance from the loop.
pub struct Topo {
    pub loop_g: Vec<usize>,
    pub inner_g: usize,
    pub outer_g: usize,
    pub inside: Vec<usize>,
    pub layer: Vec<i32>,
    pub ring_g: Vec<usize>,
    /// The rim's layer as a cycle, in order.
    pub rim: Vec<usize>,
}

pub struct EyeEdges {
    pub edge: Vec<u32>,
    pub inner: u32,
    pub outer: u32,
    pub ring: Vec<u32>,
    /// Per `edge` vertex, whether it is on the upper lid.
    pub upper: Vec<bool>,
}

pub fn topo(mesh: &Mesh, e: &EyeEdges, roll: i32) -> Topo {
    let n = mesh.merged_count();
    let loop_g: Vec<usize> = e.edge.iter().map(|&v| mesh.group[v as usize] as usize).collect();
    let on_loop: HashSet<usize> = loop_g.iter().copied().collect();
    let ring_g: Vec<usize> = e.ring.iter().map(|&v| mesh.group[v as usize] as usize).collect();
    let mut comp = vec![usize::MAX; n];
    let mut sizes = Vec::new();
    for s in 0..n {
        if comp[s] != usize::MAX || on_loop.contains(&s) {
            continue;
        }
        let id = sizes.len();
        let mut stack = vec![s];
        comp[s] = id;
        let mut size = 0;
        while let Some(g) = stack.pop() {
            size += 1;
            for &m in mesh.neighbours_of(g) {
                let m = m as usize;
                if comp[m] == usize::MAX && !on_loop.contains(&m) {
                    comp[m] = id;
                    stack.push(m);
                }
            }
        }
        sizes.push(size);
    }
    let outside = (0..sizes.len()).max_by_key(|&i| sizes[i]).unwrap();
    let touching: HashSet<usize> = loop_g.iter().flat_map(|&g| mesh.neighbours_of(g).iter().map(|&m| comp[m as usize])).filter(|&c| c != usize::MAX && c != outside).collect();
    let inside: Vec<usize> = (0..n).filter(|&g| touching.contains(&comp[g])).collect();
    let mut layer = vec![-1i32; n];
    let mut front: Vec<usize> = loop_g.clone();
    for &g in &front {
        layer[g] = 0;
    }
    let mut d = 0;
    while !front.is_empty() && d < 40 {
        d += 1;
        let mut next = Vec::new();
        for &g in &front {
            for &m in mesh.neighbours_of(g) {
                let m = m as usize;
                if layer[m] < 0 && comp[m] == outside {
                    layer[m] = d;
                    next.push(m);
                }
            }
        }
        front = next;
    }
    let mut t = Topo { loop_g, inner_g: mesh.group[e.inner as usize] as usize, outer_g: mesh.group[e.outer as usize] as usize, inside, layer, ring_g, rim: Vec::new() };
    t.rim = cycle(mesh, &t, roll);
    t
}

fn cycle(mesh: &Mesh, topo: &Topo, l: i32) -> Vec<usize> {
    let vs: Vec<usize> = (0..topo.layer.len()).filter(|&g| topo.layer[g] == l).collect();
    let set: HashSet<usize> = vs.iter().copied().collect();
    let mut out = vec![vs[0]];
    let mut prev = usize::MAX;
    loop {
        let cur = *out.last().unwrap();
        let next = mesh.neighbours_of(cur).iter().map(|&m| m as usize).find(|m| set.contains(m) && *m != prev && !out[1..].contains(m) && (*m != out[0] || out.len() > 2));
        match next {
            Some(m) if m == out[0] => break,
            Some(m) => {
                prev = cur;
                out.push(m);
            }
            None => break,
        }
    }
    out
}

/// Spherical place of an eye-frame point: (angle round the forward axis, angle from straight ahead) in radians, and the radius.
pub fn sph(q: V3) -> (f64, f64, f64) {
    let r = length(q);
    (q[1].atan2(q[0]), (q[0].hypot(q[1])).atan2(q[2]), r)
}

pub fn dir(theta: f64, phi: f64) -> V3 {
    let (st, ct) = theta.sin_cos();
    let (sp, cp) = phi.sin_cos();
    [sp * ct, sp * st, cp]
}

/// A closed target curve on the unit sphere of one eye's frame, split at its two tips: the upper arch and the lower arch, each from the
/// inner tip to the outer tip, as unit directions with their running arc length (radians).
#[derive(Clone)]
pub struct Target {
    pub upper: Vec<V3>,
    pub lower: Vec<V3>,
    pub up_s: Vec<f64>,
    pub lo_s: Vec<f64>,
}

fn arc(v: &[V3]) -> Vec<f64> {
    let mut s = vec![0.0];
    for w in v.windows(2) {
        let d = dot(w[0], w[1]).clamp(-1.0, 1.0).acos();
        s.push(s.last().unwrap() + d);
    }
    s
}

impl Target {
    /// The direction at a share (0 at the inner tip, 1 at the outer one) along an arch.
    pub fn at(&self, upper: bool, share: f64) -> V3 {
        let (v, s) = if upper { (&self.upper, &self.up_s) } else { (&self.lower, &self.lo_s) };
        let want = share.clamp(0.0, 1.0) * s.last().unwrap();
        for i in 1..v.len() {
            if s[i] >= want {
                let t = if s[i] > s[i - 1] { (want - s[i - 1]) / (s[i] - s[i - 1]) } else { 0.0 };
                return normalize(add(scale(v[i - 1], 1.0 - t), scale(v[i], t)));
            }
        }
        *v.last().unwrap()
    }

    /// The whole closed curve (upper arch, then the lower arch back), as unit directions.
    pub fn closed(&self) -> Vec<V3> {
        let mut c = self.upper.clone();
        c.extend(self.lower.iter().rev().skip(1).take(self.lower.len() - 2).copied());
        c
    }

    /// The curve that the handles `h` describe, on an eyeball of radius `r` (eye frame).
    pub fn from_handles(h: &[f64], r: f64) -> Self {
        let pin = [-h[0], h[1]];
        let pout = [h[2], h[3]];
        let c = [pout[0] - pin[0], pout[1] - pin[1]];
        let l = c[0].hypot(c[1]).max(1e-9);
        let e = [c[0] / l, c[1] / l];
        let nrm = [-e[1], e[0]];
        let (qi, qo) = (shape_q(h[8]), shape_q(h[9]));
        let n = 160;
        let mk = |upper: bool| -> Vec<V3> {
            (0..=n)
                .map(|i| {
                    let s = i as f64 / n as f64;
                    let b = if upper { h[4] * arch(s, h[5], qi, qo) } else { -h[6] * arch(s, h[7], qi, qo) };
                    let p = [pin[0] + s * c[0] + nrm[0] * b, pin[1] + s * c[1] + nrm[1] * b];
                    unflat([p[0] / r, p[1] / r])
                })
                .collect()
        };
        let (upper, lower) = (mk(true), mk(false));
        let (up_s, lo_s) = (arc(&upper), arc(&lower));
        Target { upper, lower, up_s, lo_s }
    }
}

/// The handles of the opening, in this order. Places and heights are mm along the eyeball's surface in the flat map (toward the temple
/// and up); the peak places are shares of the way from the inner tip to the outer one; the shapes go from -1 (pointed) to 1 (blunt).
pub const HANDLES: [&str; 10] = ["Inner corner place", "Inner corner height", "Outer corner place", "Outer corner height", "Upper lid height", "Upper lid peak place", "Lower lid depth", "Lower lid low point place", "Inner corner shape", "Outer corner shape"];

pub fn arch(s: f64, m: f64, q_in: f64, q_out: f64) -> f64 {
    let m = m.clamp(0.05, 0.95);
    let (t, q) = if s < m { (s / m, q_in) } else { ((1.0 - s) / (1.0 - m), q_out) };
    let t = t.clamp(0.0, 1.0);
    (1.0 - (1.0 - t) * (1.0 - t)).powf(q)
}

pub fn shape_q(k: f64) -> f64 {
    (-k * 2.5f64.ln()).exp()
}

/// 2D place of a unit direction: the angle from straight ahead along the direction round the forward axis (azimuthal equidistant).
pub fn flat(d: V3) -> [f64; 2] {
    let (th, ph, _) = sph(d);
    [ph * th.cos(), ph * th.sin()]
}

pub fn unflat(u: [f64; 2]) -> V3 {
    let ph = u[0].hypot(u[1]);
    if ph < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    let th = u[1].atan2(u[0]);
    dir(th, ph)
}

/// Whether a flat place lies inside the flat outline `poly`. `inside(flat(unflat(u)), poly)` is the prototype's `inside_curve`.
fn inside_poly(p: [f64; 2], poly: &[[f64; 2]]) -> bool {
    let mut inside = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < a[0] + (p[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            inside = !inside;
        }
    }
    inside
}

fn inside_curve(u: [f64; 2], poly: &[[f64; 2]]) -> bool {
    inside_poly(flat(unflat(u)), poly)
}

fn seg_dist(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let l2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if l2 > 0.0 { (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0) } else { 0.0 };
    // a plain square root, not `hypot`: the browser's `hypot` is slow, and this runs millions of times per build
    let (x, y) = (p[0] - a[0] - t * ab[0], p[1] - a[1] - t * ab[1]);
    (x * x + y * y).sqrt()
}

fn curve_dist(u: [f64; 2], poly: &[[f64; 2]]) -> f64 {
    let mut m = f64::MAX;
    for k in 0..poly.len() {
        m = m.min(seg_dist(u, poly[k], poly[(k + 1) % poly.len()]));
    }
    m
}

/// The sizes of the construction (mm unless said), the prototype's defaults.
#[derive(Clone, Copy, Debug)]
pub struct Sizes {
    pub gap: f64,
    pub reach: f64,
    pub roll: i32,
    pub sink: f64,
    pub blend: f64,
}

pub const SIZES: Sizes = Sizes { gap: 0.15, reach: 15.0, roll: 3, sink: 1.0, blend: 0.12 };
const UNDER: f64 = 0.03;
/// The region round the eye that the rim's move spreads over: vertices nearer than this (mm) to the live or the new rim.
const SPREAD: f64 = 22.0;
/// The first layer outside the rim leaves the curve by `PIN_STEP` radians outward and `PIN_RISE` mm up, so the lid's edge cannot roll
/// back over the opening.
const PIN_STEP: f64 = 0.03;
const PIN_RISE: f64 = 0.5;
/// The same along the lower lid. A step as high as the upper lid's makes a short wall there that faces the light and shows as a pale
/// line along the lid, so the lower lid's step is lower and wider.
pub const PIN_LOWER: [f64; 2] = [0.04, 0.3];

/// The lower lid's step (radians outward, mm up): `PIN_LOWER`, or `EYE_CURVE_PIN_LOWER="step,rise"` on native builds.
fn pin_lower() -> [f64; 2] {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(v) = std::env::var("EYE_CURVE_PIN_LOWER") {
        let n: Vec<f64> = v.split(',').filter_map(|x| x.parse().ok()).collect();
        if n.len() == 2 {
            return [n[0], n[1]];
        }
    }
    PIN_LOWER
}
/// Within `FLOOR` mm of the curve, the skin rises from it at least `RISE` mm per mm, by a smooth maximum `SOFT` mm wide. Further out
/// the floor stays level, so it lifts only the lid's edge and leaves no line on the cheek.
const RISE: f64 = 0.5;
const SOFT: f64 = 0.3;
const FLOOR: f64 = 1.5;

/// Along the lower lid, the rim and the first layer outside it sink just inside the eyeball, so the eyeball hides the skin's climb
/// from the curve, as KOTOR's skin passes through its eyeball. The rows there are split finely at the end of the build
/// (`crate::refine`), or their long edges would dip into the eyeball between their ends.
#[derive(Clone, Copy, Debug)]
pub struct Sink {
    /// How far the rim goes inside the eyeball (mm).
    pub rim: f64,
    /// How far the rim moves in from the curve, along the eyeball (mm), so the skin crosses the eyeball on the curve.
    pub inward: f64,
    /// The first layer's height above the eyeball (mm, below 0 inside it). It is exempt from the height floor.
    pub first: f64,
    /// The first layer's step outward from the rim (radians).
    pub step: f64,
    /// How much deeper the layers under the rim go than the rim (mm).
    pub under: f64,
    /// The height floor's base along the lower lid (mm), in place of the gap.
    pub floor: f64,
    /// The share of the lower arch at each tip over which the sink fades in.
    pub fade: f64,
}

pub const SINK: Sink = Sink { rim: 0.25, inward: 0.6, first: -0.05, step: 0.02, under: 0.2, floor: 0.0, fade: 0.3 };

pub fn to_eye(c: V3, side: f64, p: V3) -> V3 {
    [(p[0] - c[0]) * side, p[1] - c[1], p[2] - c[2]]
}

pub fn from_eye(c: V3, side: f64, q: V3) -> V3 {
    [q[0] * side + c[0], q[1] + c[1], q[2] + c[2]]
}

/// Builds one eye's opening on curve `t` (eye frame). `p` are the merged vertices in the head frame (mm), changed in place. `c`, `side`
/// and `r` give the eyeball. `keep` are the vertices that must not move (the eyeball's ring and the other eye).
#[allow(clippy::too_many_arguments)]
pub fn build_eye(mesh: &Mesh, topo: &Topo, p: &mut [V3], c: V3, side: f64, r: f64, t: &Target, sz: Sizes, keep: &HashSet<usize>) {
    let n = p.len();
    let q0: Vec<V3> = p.iter().map(|&x| to_eye(c, side, x)).collect();
    let fl0: Vec<[f64; 2]> = q0.iter().map(|&q| flat(normalize(q))).collect();
    let h0: Vec<f64> = q0.iter().map(|&q| length(q) - r).collect();
    let inside: HashSet<usize> = topo.inside.iter().copied().collect();
    let m_layer = sz.roll;
    let mut rim = topo.rim.clone();
    let near_of = |rim: &[usize], g: usize| (0..rim.len()).min_by(|&i, &j| length(sub(q0[rim[i]], q0[g])).total_cmp(&length(sub(q0[rim[j]], q0[g])))).unwrap();
    let i_in = near_of(&rim, topo.inner_g);
    rim.rotate_left(i_in);
    let i_out = near_of(&rim, topo.outer_g);
    let path_a: Vec<usize> = rim[..=i_out].to_vec();
    let mut path_b: Vec<usize> = rim[i_out..].to_vec();
    path_b.push(rim[0]);
    path_b.reverse();
    let mean_y = |v: &[usize]| v.iter().map(|&g| q0[g][1]).sum::<f64>() / v.len() as f64;
    let (up_path, lo_path) = if mean_y(&path_a) > mean_y(&path_b) { (path_a, path_b) } else { (path_b, path_a) };
    let mut tgt: HashMap<usize, V3> = HashMap::new();
    for (path, upper) in [(&up_path, true), (&lo_path, false)] {
        let mut acc = vec![0.0];
        for w in path.windows(2) {
            acc.push(acc.last().unwrap() + length(sub(q0[w[1]], q0[w[0]])));
        }
        let total = *acc.last().unwrap();
        for (i, &g) in path.iter().enumerate() {
            tgt.insert(g, t.at(upper, acc[i] / total));
        }
    }
    let poly: Vec<[f64; 2]> = t.closed().iter().map(|&d| flat(d)).collect();
    let centroid = {
        let k = poly.len() as f64;
        [poly.iter().map(|q| q[0]).sum::<f64>() / k, poly.iter().map(|q| q[1]).sum::<f64>() / k]
    };
    let outward = |u: [f64; 2]| -> [f64; 2] {
        let k = (0..poly.len())
            .min_by(|&i, &j| {
                let d = |q: [f64; 2]| (q[0] - u[0]).hypot(q[1] - u[1]);
                d(poly[i]).total_cmp(&d(poly[j]))
            })
            .unwrap();
        let (a, b) = (poly[(k + poly.len() - 1) % poly.len()], poly[(k + 1) % poly.len()]);
        let tg = [b[0] - a[0], b[1] - a[1]];
        let mut nn = [tg[1], -tg[0]];
        if nn[0] * (u[0] - centroid[0]) + nn[1] * (u[1] - centroid[1]) < 0.0 {
            nn = [-nn[0], -nn[1]];
        }
        let l = nn[0].hypot(nn[1]).max(1e-12);
        [nn[0] / l, nn[1] / l]
    };
    let sk = SINK;
    // the sink's weight per rim vertex: 1 along the lower lid, fading to 0 toward the tips, 0 on the upper lid
    let mut wsink: HashMap<usize, f64> = HashMap::new();
    {
        let mut acc = vec![0.0];
        for w in lo_path.windows(2) {
            acc.push(acc.last().unwrap() + length(sub(q0[w[1]], q0[w[0]])));
        }
        let total = *acc.last().unwrap();
        for (i, &g) in lo_path.iter().enumerate() {
            let s = acc[i] / total;
            let x = (s.min(1.0 - s) / sk.fade.max(1e-6)).clamp(0.0, 1.0);
            wsink.insert(g, x * x * (3.0 - 2.0 * x));
        }
        for &g in &up_path {
            wsink.insert(g, 0.0);
        }
    }
    let ws = |g: usize| wsink.get(&g).copied().unwrap_or(0.0);
    let mut fl = fl0.clone();
    let mut hh = h0.clone();
    for (&g, &d) in &tgt {
        fl[g] = flat(d);
        hh[g] = sz.gap + ws(g) * (-sk.rim - sz.gap);
        if ws(g) > 0.0 {
            let o = outward(fl[g]);
            let k = ws(g) * sk.inward / r;
            fl[g] = [fl[g][0] - o[0] * k, fl[g][1] - o[1] * k];
        }
    }
    let rim_closed: Vec<usize> = {
        let mut v = up_path.clone();
        v.extend(lo_path.iter().rev().skip(1).take(lo_path.len() - 2));
        v
    };
    let push_out = |u: [f64; 2], m: f64| -> [f64; 2] {
        let mut best = (f64::MAX, [0.0; 2], [0.0; 2]);
        for k in 0..poly.len() {
            let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
            let ab = [b[0] - a[0], b[1] - a[1]];
            let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-12);
            let t = (((u[0] - a[0]) * ab[0] + (u[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0);
            let q = [a[0] + t * ab[0], a[1] + t * ab[1]];
            let d = ((u[0] - q[0]) * (u[0] - q[0]) + (u[1] - q[1]) * (u[1] - q[1])).sqrt();
            if d < best.0 {
                best = (d, q, ab);
            }
        }
        let (d, q, ab) = best;
        let inside = inside_curve(u, &poly);
        if !inside && d >= m {
            return u;
        }
        let mut nn = if !inside && d > 1e-9 {
            [(u[0] - q[0]) / d, (u[1] - q[1]) / d]
        } else {
            let l = ab[0].hypot(ab[1]).max(1e-12);
            let c1 = [-ab[1] / l, ab[0] / l];
            let probe = [q[0] + c1[0] * 1e-4, q[1] + c1[1] * 1e-4];
            if inside_curve(probe, &poly) { [-c1[0], -c1[1]] } else { c1 }
        };
        let l = nn[0].hypot(nn[1]).max(1e-12);
        nn = [nn[0] / l, nn[1] / l];
        [q[0] + nn[0] * m, q[1] + nn[1] * m]
    };
    // 1. the layers between the loop and the rim: under the lid, against the eyeball
    let under: Vec<usize> = (0..n).filter(|&g| topo.layer[g] >= 0 && topo.layer[g] < m_layer && !keep.contains(&g)).collect();
    let nr = rim_closed.len();
    let mut along: HashMap<usize, [f64; 2]> = HashMap::new();
    for (i, &g) in rim_closed.iter().enumerate() {
        let a = std::f64::consts::TAU * i as f64 / nr as f64;
        along.insert(g, [a.cos(), a.sin()]);
    }
    for l in (0..m_layer).rev() {
        for &g in under.iter().filter(|&&g| topo.layer[g] == l) {
            let mut v = [0.0, 0.0];
            for &m in mesh.neighbours_of(g) {
                if topo.layer[m as usize] == l + 1 {
                    if let Some(a) = along.get(&(m as usize)) {
                        v[0] += a[0];
                        v[1] += a[1];
                    }
                }
            }
            if v[0].hypot(v[1]) < 1e-9 {
                for &m in mesh.neighbours_of(g) {
                    if let Some(a) = along.get(&(m as usize)) {
                        v[0] += a[0];
                        v[1] += a[1];
                    }
                }
            }
            let lv = v[0].hypot(v[1]).max(1e-12);
            along.insert(g, [v[0] / lv, v[1] / lv]);
        }
    }
    let on_curve = |a: [f64; 2]| -> [f64; 2] {
        let x = a[1].atan2(a[0]).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU * nr as f64;
        let i = (x.floor() as usize) % nr;
        let f = x - x.floor();
        let (d0, d1) = (tgt[&rim_closed[i]], tgt[&rim_closed[(i + 1) % nr]]);
        flat(normalize(add(scale(d0, 1.0 - f), scale(d1, f))))
    };
    for &g in &under {
        let u = on_curve(along[&g]);
        let o = outward(u);
        let depth = (m_layer - topo.layer[g]) as f64 * UNDER;
        fl[g] = push_out([u[0] + o[0] * depth, u[1] + o[1] * depth], 0.5 * depth);
        hh[g] = 0.4 * sz.gap;
        let a = along[&g];
        let x = a[1].atan2(a[0]).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU * nr as f64;
        let i = (x.floor() as usize) % nr;
        let f = x - x.floor();
        let w = ws(rim_closed[i]) * (1.0 - f) + ws(rim_closed[(i + 1) % nr]) * f;
        hh[g] += w * (-sk.rim - sk.under - hh[g]);
    }
    // 2. the skin round the eye: the rim's move (onto the curve, down to `gap`) spreads over the region as a smooth biharmonic move in
    // 3D, which is nothing at the region's edge, so the skin blends into the face with no step
    let rim_new: Vec<V3> = rim.iter().map(|&l| scale(tgt[&l], r + sz.gap)).collect();
    let dist: Vec<f64> = (0..n).map(|g| rim.iter().map(|&l| length(sub(q0[g], q0[l]))).chain(rim_new.iter().map(|&x| length(sub(q0[g], x)))).fold(f64::MAX, f64::min)).collect();
    let region: Vec<usize> = (0..n).filter(|&g| topo.layer[g] > m_layer && dist[g] < SPREAD && !keep.contains(&g)).collect();
    let mut known: HashMap<usize, V3> = HashMap::new();
    for &g in &rim {
        known.insert(g, sub(scale(unflat(fl[g]), r + hh[g]), q0[g]));
    }
    // the first layer outside the rim sits outward of the mean of its rim neighbours' places on the curve; along the lower lid (every
    // rim neighbour on the lower arch, the tips excepted) by the lower lid's own step
    let lower_rim: HashSet<usize> = lo_path[1..lo_path.len() - 1].iter().copied().collect();
    let [step_lo, rise_lo] = pin_lower();
    // the first layer along the lower lid keeps its sunk height: the floor below would lift it back out of the eyeball
    let mut sunk: HashSet<usize> = HashSet::new();
    for &g in region.iter().filter(|&&g| topo.layer[g] == m_layer + 1) {
        let rim_nb: Vec<usize> = mesh.neighbours_of(g).iter().map(|&m| m as usize).filter(|m| known.contains_key(m) && topo.layer[*m] == m_layer).collect();
        if rim_nb.is_empty() {
            continue;
        }
        let (mut step, rise) = if rim_nb.iter().all(|m| lower_rim.contains(m)) { (step_lo, rise_lo) } else { (PIN_STEP, PIN_RISE) };
        let w = rim_nb.iter().map(|&m| ws(m)).sum::<f64>() / rim_nb.len() as f64;
        let h1 = sz.gap + rise + w * (sk.first - sz.gap - rise);
        step += w * (sk.step - step);
        if w > 0.0 {
            sunk.insert(g);
        }
        let u = flat(normalize(rim_nb.iter().fold([0.0; 3], |a, &m| add(a, unflat(fl[m])))));
        let o = outward(u);
        let v = push_out([u[0] + o[0] * step, u[1] + o[1] * step], 0.75 * step);
        known.insert(g, sub(scale(unflat(v), r + h1), q0[g]));
    }
    let skip: HashSet<usize> = under.iter().chain(rim.iter()).copied().collect();
    let free: Vec<usize> = region.iter().copied().filter(|g| !known.contains_key(g)).collect();
    let mv = smooth_move(mesh, &free, &known, &skip, &q0);
    let mut moved: Vec<(usize, V3)> = region.iter().filter_map(|&g| known.get(&g).map(|&k| (g, k))).collect();
    moved.extend(free.iter().zip(&mv).map(|(&g, &m)| (g, m)));
    for (g, m) in moved {
        let q = add(q0[g], m);
        fl[g] = flat(normalize(q));
        let h = length(q) - r;
        if sunk.contains(&g) {
            hh[g] = h;
            continue;
        }
        // near the curve the skin rises from it at least `RISE` mm per mm, by a smooth maximum, so it never sinks toward the eyeball;
        // along the lower lid it rises from the sink's floor
        let d = curve_dist(fl[g], &poly) * r;
        let d = if d < FLOOR { d - d * d * d / (4.0 * FLOOR * FLOOR) } else { 0.75 * FLOOR };
        let near = |l: &usize| (fl[*l][0] - fl[g][0]).powi(2) + (fl[*l][1] - fl[g][1]).powi(2);
        let base = rim.iter().min_by(|a, b| near(a).total_cmp(&near(b))).map_or(sz.gap, |&l| sz.gap + ws(l) * (sk.floor - sz.gap));
        let hmin = base + RISE * d;
        let k = ((h - hmin) / SOFT * 0.5 + 0.5).clamp(0.0, 1.0);
        hh[g] = hmin + (h - hmin) * k + SOFT * k * (1.0 - k);
    }
    // back to the head
    for &g in region.iter().chain(rim.iter()).chain(under.iter()) {
        p[g] = from_eye(c, side, scale(unflat(fl[g]), r + hh[g]));
    }
    // inside the loop: each vertex follows the loop vertex it is reached from, fading over 6 steps, and goes under the eyeball where
    // it lies in front of 120 degrees
    let mut from: HashMap<usize, (usize, i32)> = HashMap::new();
    let mut front: Vec<usize> = topo.loop_g.clone();
    for &g in &front {
        from.insert(g, (g, 0));
    }
    while !front.is_empty() {
        let mut next = Vec::new();
        for &g in &front {
            let (src, d) = from[&g];
            for &m in mesh.neighbours_of(g) {
                let m = m as usize;
                if inside.contains(&m) && !from.contains_key(&m) {
                    from.insert(m, (src, d + 1));
                    next.push(m);
                }
            }
        }
        front = next;
    }
    for &g in &topo.inside {
        if keep.contains(&g) {
            continue;
        }
        let Some(&(l, d)) = from.get(&g) else { continue };
        let w = (1.0 - d as f64 / 6.0).clamp(0.0, 1.0);
        let f = [fl0[g][0] + w * (fl[l][0] - fl0[l][0]), fl0[g][1] + w * (fl[l][1] - fl0[l][1])];
        let ph = f[0].hypot(f[1]).to_degrees();
        let rad = if ph < 120.0 { (r + h0[g]).min(r - sz.sink) } else { r + h0[g] };
        p[g] = from_eye(c, side, scale(unflat(f), rad));
    }
}

/// The lid piece built from the curve, as one rigid piece. The upper arch, a little longer at each tip, is swept about a line through
/// the eyeball's centre parallel to the line through the two tips, at `r + lift`. Turned by `theta` about the returned axis, the piece
/// covers the prototype's closed lid. Returns the open piece's points (eye frame) as `cols` columns along the arch of `rows` points
/// each, the axis and the closing angle (degrees).
pub fn lid_piece(t: &Target, r: f64, lift: f64, margin: f64) -> (Vec<V3>, usize, usize, V3, f64) {
    let (ti, to) = (t.upper[0], *t.upper.last().unwrap());
    let ax = normalize(sub(to, ti));
    let turn = |d: V3, a: f64| -> V3 {
        let (s, co) = a.to_radians().sin_cos();
        let kd = dot(ax, d);
        let kx = cross(ax, d);
        add(add(scale(d, co), scale(kx, s)), scale(ax, kd * (1.0 - co)))
    };
    let e1 = {
        let up = [0.0, 1.0, 0.0];
        normalize(sub(up, scale(ax, dot(up, ax))))
    };
    let e2 = cross(ax, e1);
    let ang = |d: V3| dot(d, e2).atan2(dot(d, e1)).to_degrees();
    let sgn = {
        let d = turn(e1, 10.0);
        if ang(d) > 0.0 { 1.0 } else { -1.0 }
    };
    let ext = |a: V3, b: V3, mm: f64| {
        let dir = normalize(sub(b, a));
        normalize(add(b, scale(dir, mm / r)))
    };
    let n = t.upper.len();
    let mut arch = vec![ext(t.upper[1], t.upper[0], 4.0), ext(t.upper[1], t.upper[0], 2.0)];
    arch.extend(t.upper.iter().copied());
    arch.push(ext(t.upper[n - 2], t.upper[n - 1], 2.0));
    arch.push(ext(t.upper[n - 2], t.upper[n - 1], 4.0));
    let along = |d: V3| dot(d, ax);
    let mut need: f64 = 10.0;
    for &l in &t.lower {
        let a = along(l);
        let u = arch.windows(2).find_map(|w| {
            let (a0, a1) = (along(w[0]), along(w[1]));
            if (a0 - a) * (a1 - a) <= 0.0 && (a1 - a0).abs() > 1e-9 {
                let s = (a - a0) / (a1 - a0);
                Some(normalize(add(scale(w[0], 1.0 - s), scale(w[1], s))))
            } else {
                None
            }
        });
        if let Some(u) = u {
            let mut d = (ang(l) - ang(u)) * sgn;
            if d < -180.0 {
                d += 360.0
            }
            if d > 180.0 {
                d -= 360.0
            }
            need = need.max(d);
        }
    }
    let theta = need + margin;
    let steps = 24;
    let mut pts = Vec::with_capacity(arch.len() * (steps + 1));
    for &a in &arch {
        for j in 0..=steps {
            pts.push(scale(turn(a, sgn * theta * (j as f64 / steps as f64 - 1.0)), r + lift));
        }
    }
    (pts, arch.len(), steps + 1, scale(ax, sgn), theta)
}

/// The smooth move of the `free` vertices: the biharmonic field (mean value weights from the live mesh `q0`) that takes the moves `known`
/// at their vertices and nothing at every other vertex round the free ones. The Laplacian is held small at the free vertices and at their
/// neighbours that are neither known nor in `skip_rows`, so the move fades out with no step at the region's edge. Solved by conjugate
/// gradients on the normal equations.
fn smooth_move(mesh: &Mesh, free: &[usize], known: &HashMap<usize, V3>, skip_rows: &HashSet<usize>, q0: &[V3]) -> Vec<V3> {
    let idx: HashMap<usize, usize> = free.iter().enumerate().map(|(i, &g)| (g, i)).collect();
    let n = free.len();
    let mut rows: Vec<usize> = free.to_vec();
    let mut seen: HashSet<usize> = free.iter().copied().collect();
    for &g in free {
        for &m in mesh.neighbours_of(g) {
            let m = m as usize;
            if !skip_rows.contains(&m) && !known.contains_key(&m) && seen.insert(m) {
                rows.push(m);
            }
        }
    }
    // a row's weights: per neighbour, the tangents of the half angles it makes with the neighbours it shares a triangle with, over its
    // length
    let weights = |g: usize| -> Vec<(usize, f64)> {
        let nb = mesh.neighbours_of(g);
        let mut w: Vec<(usize, f64)> = nb
            .iter()
            .map(|&m| {
                let m = m as usize;
                let e = sub(q0[m], q0[g]);
                let l = length(e).max(1e-6);
                let mut t = 0.0;
                let mut shared = 0;
                for &o in nb {
                    if o as usize == m || !mesh.neighbours_of(m).contains(&o) {
                        continue;
                    }
                    let f = sub(q0[o as usize], q0[g]);
                    t += (0.5 * (dot(e, f) / (l * length(f).max(1e-6))).clamp(-1.0, 1.0).acos()).tan();
                    shared += 1;
                }
                (m, if shared > 0 { t / l } else { 1.0 / l })
            })
            .collect();
        let total: f64 = w.iter().map(|x| x.1).sum();
        w.iter_mut().for_each(|x| x.1 /= total);
        w
    };
    let mut a: Vec<Vec<(usize, f64)>> = Vec::with_capacity(rows.len());
    let mut b: Vec<V3> = Vec::with_capacity(rows.len());
    for &g in &rows {
        let mut row = Vec::new();
        let mut rhs = [0.0; 3];
        for (v, c) in std::iter::once((g, 1.0)).chain(weights(g).into_iter().map(|(m, w)| (m, -w))) {
            if let Some(&i) = idx.get(&v) {
                row.push((i, c));
            } else if let Some(x) = known.get(&v) {
                rhs = sub(rhs, scale(*x, c));
            }
        }
        a.push(row);
        b.push(rhs);
    }
    let apply = |x: &[V3]| -> Vec<V3> {
        let mut out = vec![[0.0; 3]; n];
        for row in &a {
            let s = row.iter().fold([0.0; 3], |s, &(i, c)| add(s, scale(x[i], c)));
            for &(i, c) in row {
                out[i] = add(out[i], scale(s, c));
            }
        }
        out
    };
    let mut diag = vec![1e-12; n];
    let mut rhs = vec![[0.0; 3]; n];
    for (row, bb) in a.iter().zip(&b) {
        for &(i, c) in row {
            diag[i] += c * c;
            rhs[i] = add(rhs[i], scale(*bb, c));
        }
    }
    let dot_d = |u: &[V3], v: &[V3], d: usize| -> f64 { u.iter().zip(v).map(|(a, b)| a[d] * b[d]).sum() };
    let mut x = vec![[0.0; 3]; n];
    let mut res = rhs;
    let mut z: Vec<V3> = res.iter().zip(&diag).map(|(r, &q)| scale(*r, 1.0 / q)).collect();
    let mut p = z.clone();
    let mut rz = [0, 1, 2].map(|d| dot_d(&res, &z, d));
    let r0 = [0, 1, 2].map(|d| dot_d(&res, &res, d).sqrt().max(1e-30));
    for _ in 0..2000 {
        let ap = apply(&p);
        let mut done = true;
        for d in 0..3 {
            let pap = dot_d(&p, &ap, d);
            if pap.abs() < 1e-300 {
                continue;
            }
            let al = rz[d] / pap;
            for i in 0..n {
                x[i][d] += al * p[i][d];
                res[i][d] -= al * ap[i][d];
            }
            if dot_d(&res, &res, d).sqrt() > 1e-7 * r0[d] {
                done = false;
            }
        }
        if done {
            break;
        }
        z = res.iter().zip(&diag).map(|(r, &q)| scale(*r, 1.0 / q)).collect();
        for d in 0..3 {
            let rz1 = dot_d(&res, &z, d);
            let be = if rz[d].abs() > 1e-300 { rz1 / rz[d] } else { 0.0 };
            rz[d] = rz1;
            for i in 0..n {
                p[i][d] = z[i][d] + be * p[i][d];
            }
        }
    }
    x
}
