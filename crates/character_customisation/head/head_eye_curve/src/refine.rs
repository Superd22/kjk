//! Finer triangles in the rows of skin below each lower rim, made at the end of the build. The head's own vertices keep their indices;
//! the new ones come after them. Each triangle of the band is split into `SPLIT` x `SPLIT` triangles, and each triangle next to the
//! band that shares one split edge is split into a fan from its third corner. The new points lie on a smooth height field over the
//! eyeball, not on the flat triangles, so the fine skin follows the eyeball's curve.

use crate::curve::{Topo, flat, from_eye, to_eye, unflat};
use crate::maker::Mesh;
use crate::{V3, length, normalize, scale};
use std::collections::{HashMap, HashSet};

/// The rows below the rim that are split: the triangles between the rim's layer and `ROWS` layers out.
const ROWS: i32 = 3;
const SPLIT: usize = 4;
/// How much of the height field's slopes the new points follow (0 gives the flat triangles' heights, 1 the full quadratic patch).
const CURVED: f64 = 0.75;

/// The refinement of one head mesh: its new vertices and the triangles that replace the split ones.
pub struct Refine {
    /// The head's vertex count: new vertex `j` is vertex `count + j`.
    pub count: usize,
    /// Per new vertex: the three vertices of the head triangle it lies in, and its weights on them.
    pub parents: Vec<([u32; 3], [f64; 3])>,
    /// Per new vertex: the eye whose eyeball its height is measured from.
    pub eye: Vec<usize>,
    /// The triangles that replace the split ones, each with the head triangle it lies in.
    pub triangles: Vec<[u32; 3]>,
    pub parent: Vec<u32>,
    /// The head triangles that are split.
    pub split: Vec<u32>,
    /// Per new vertex, how far it follows the smooth height field rather than its flat triangle: 0 next to the triangles that are not
    /// split whole, so the fans and the band's border stay flat and meet the mesh round them with no kink.
    bend: Vec<f64>,
    /// The merged vertices whose heights and slopes the new points read, per eye.
    corners: [Vec<usize>; 2],
}

/// Whether each merged vertex lies on the lower lid's side of eye `topo`: reached from the lower lid's loop vertices, layer by layer.
fn lower_side(mesh: &Mesh, topo: &Topo, upper: &[bool]) -> Vec<bool> {
    let n = mesh.merged_count();
    let mut votes = vec![(0i32, 0i32); n];
    let mut lower = vec![false; n];
    for (&g, &u) in topo.loop_g.iter().zip(upper) {
        lower[g] = !u;
        votes[g] = if u { (0, 1) } else { (1, 0) };
    }
    for l in 1..=topo.layer.iter().copied().max().unwrap_or(0) {
        for g in (0..n).filter(|&g| topo.layer[g] == l) {
            let (mut lo, mut up) = (0, 0);
            for &m in mesh.neighbours_of(g) {
                if topo.layer[m as usize] == l - 1 {
                    lo += votes[m as usize].0;
                    up += votes[m as usize].1;
                }
            }
            votes[g] = (lo.min(1000), up.min(1000));
            lower[g] = lo > up;
        }
    }
    lower
}

impl Refine {
    /// `upper` per eye says which of its lid edge loop's vertices are on the upper lid.
    pub fn new(mesh: &Mesh, topos: &[Topo; 2], upper: [&[bool]; 2], hidden_tris: &HashSet<usize>) -> Self {
        let count = mesh.group.len();
        let tris = &mesh.triangles;
        let g = |v: u32| mesh.group[v as usize] as usize;
        let ekey = |a: usize, b: usize| if a < b { (a, b) } else { (b, a) };
        // the band: triangles whose corners all lie on the lower side, from the rim's layer to `ROWS` layers out
        let mut eye_of: HashMap<usize, usize> = HashMap::new();
        for (e, topo) in topos.iter().enumerate() {
            let lower = lower_side(mesh, topo, upper[e]);
            let m = topo.rim.first().map_or(3, |&r| topo.layer[r]);
            for (t, tr) in tris.iter().enumerate() {
                if hidden_tris.contains(&t) {
                    continue;
                }
                if tr.iter().all(|&v| (m..=m + ROWS).contains(&topo.layer[g(v)]) && lower[g(v)]) {
                    eye_of.insert(t, e);
                }
            }
        }
        // a triangle next to the band with two or three split edges is split whole too, until every other triangle has one at most
        let mut split_edges: HashSet<(usize, usize)> = HashSet::new();
        let mut touch: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
        for (t, tr) in tris.iter().enumerate() {
            for k in 0..3 {
                touch.entry(ekey(g(tr[k]), g(tr[(k + 1) % 3]))).or_default().push(t);
            }
        }
        loop {
            split_edges.clear();
            for &t in eye_of.keys() {
                let tr = tris[t];
                for k in 0..3 {
                    split_edges.insert(ekey(g(tr[k]), g(tr[(k + 1) % 3])));
                }
            }
            let mut more = Vec::new();
            for &ek in &split_edges {
                for &t in &touch[&ek] {
                    if eye_of.contains_key(&t) {
                        continue;
                    }
                    let tr = tris[t];
                    let s = (0..3).filter(|&k| split_edges.contains(&ekey(g(tr[k]), g(tr[(k + 1) % 3])))).count();
                    if s >= 2 {
                        let e = touch[&ek].iter().find_map(|o| eye_of.get(o)).copied().unwrap_or(0);
                        more.push((t, e));
                    }
                }
            }
            if more.is_empty() {
                break;
            }
            for (t, e) in more {
                eye_of.insert(t, e);
            }
        }
        let mut r = Refine { count, parents: Vec::new(), eye: Vec::new(), triangles: Vec::new(), parent: Vec::new(), split: Vec::new(), bend: Vec::new(), corners: [Vec::new(), Vec::new()] };
        // the new points on an edge are shared by the triangles on both sides (by their own vertices, so a seam keeps two copies)
        let mut on_edge: HashMap<(u32, u32, usize), u32> = HashMap::new();
        let mut point = |r: &mut Refine, tr: [u32; 3], w: [f64; 3], e: usize, t: usize, i: usize, j: usize| -> u32 {
            let nz: Vec<usize> = (0..3).filter(|&k| w[k] > 1e-12).collect();
            if nz.len() == 1 {
                return tr[nz[0]];
            }
            let key = if nz.len() == 2 {
                let (a, b) = (tr[nz[0]], tr[nz[1]]);
                let (lo, hi, s) = if a < b { (a, b, w[nz[1]]) } else { (b, a, w[nz[0]]) };
                (lo, hi, (s * SPLIT as f64).round() as usize)
            } else {
                (u32::MAX, t as u32, i * (SPLIT + 1) + j)
            };
            *on_edge.entry(key).or_insert_with(|| {
                r.parents.push((tr, w));
                r.eye.push(e);
                (count + r.parents.len() - 1) as u32
            })
        };
        let mut order: Vec<usize> = eye_of.keys().copied().collect();
        order.sort_unstable();
        for &t in &order {
            let e = eye_of[&t];
            let tr = tris[t];
            let k = SPLIT;
            let mut at = vec![vec![0u32; k + 1]; k + 1];
            for i in 0..=k {
                for j in 0..=k - i {
                    let w = [(k - i - j) as f64 / k as f64, i as f64 / k as f64, j as f64 / k as f64];
                    at[i][j] = point(&mut r, tr, w, e, t, i, j);
                }
            }
            for i in 0..k {
                for j in 0..k - i {
                    r.triangles.push([at[i][j], at[i + 1][j], at[i][j + 1]]);
                    r.parent.push(t as u32);
                    if i + j + 1 < k {
                        r.triangles.push([at[i + 1][j], at[i + 1][j + 1], at[i][j + 1]]);
                        r.parent.push(t as u32);
                    }
                }
            }
            r.split.push(t as u32);
        }
        // the fans next to the band
        for (t, tr) in tris.iter().enumerate() {
            if eye_of.contains_key(&t) {
                continue;
            }
            let Some(k) = (0..3).find(|&k| split_edges.contains(&ekey(g(tr[k]), g(tr[(k + 1) % 3])))) else { continue };
            let e = touch[&ekey(g(tr[k]), g(tr[(k + 1) % 3]))].iter().find_map(|o| eye_of.get(o)).copied().unwrap_or(0);
            let (a, b, c) = (k, (k + 1) % 3, (k + 2) % 3);
            let pts: Vec<u32> = (0..=SPLIT)
                .map(|s| {
                    let mut w = [0.0; 3];
                    w[a] = (SPLIT - s) as f64 / SPLIT as f64;
                    w[b] = s as f64 / SPLIT as f64;
                    point(&mut r, *tr, w, e, t, 0, 0)
                })
                .collect();
            for s in 0..SPLIT {
                r.triangles.push([pts[s], pts[s + 1], tr[c]]);
                r.parent.push(t as u32);
            }
            r.split.push(t as u32);
        }
        // a vertex of a triangle that is not split whole is flat, unless that triangle reaches under the rim, where the eyeball hides it
        let mut flat_at: HashSet<usize> = HashSet::new();
        for (t, tr) in tris.iter().enumerate() {
            if eye_of.contains_key(&t) || hidden_tris.contains(&t) {
                continue;
            }
            let under = topos.iter().any(|topo| {
                let m = topo.rim.first().map_or(3, |&x| topo.layer[x]);
                tr.iter().any(|&v| topo.layer[g(v)] >= 0 && topo.layer[g(v)] < m)
            });
            if !under {
                flat_at.extend(tr.iter().map(|&v| g(v)));
            }
        }
        r.bend = r.parents.iter().map(|(tr, w)| (0..3).map(|i| if flat_at.contains(&g(tr[i])) { 0.0 } else { w[i] }).sum()).collect();
        for e in 0..2 {
            let mut c: Vec<usize> = r.parents.iter().zip(&r.eye).filter(|(_, x)| **x == e).flat_map(|(p, _)| p.0.map(g)).collect();
            c.sort_unstable();
            c.dedup();
            r.corners[e] = c;
        }
        r
    }

    /// The new vertices' places for the merged head `p` (the eye frame's mm, as `Maker::build` holds it), with the eyeballs `balls`
    /// (centre, side, radius). In each eye's frame a point's direction is its corners' mean in the flat map, and its height above the
    /// eyeball follows the corners' heights and slopes (a Phong patch), held between the lowest and the highest corner.
    pub fn positions(&self, mesh: &Mesh, topos: &[Topo; 2], p: &[V3], balls: &[(V3, f64, f64); 2]) -> Vec<V3> {
        let g = |v: u32| mesh.group[v as usize] as usize;
        let mut chart: [HashMap<usize, ([f64; 2], f64, [f64; 2])>; 2] = [HashMap::new(), HashMap::new()];
        for e in 0..2 {
            let (c, side, r) = balls[e];
            let at = |m: usize| {
                let q = to_eye(c, side, p[m]);
                (flat(normalize(q)), length(q) - r)
            };
            let rim_layer = topos[e].rim.first().map_or(3, |&x| topos[e].layer[x]);
            for &v in &self.corners[e] {
                let (u, h) = at(v);
                // the slope: least squares over the neighbours outside the layers under the rim, which fold under the lid
                let (mut sxx, mut sxy, mut syy, mut sxh, mut syh) = (0.0, 0.0, 0.0, 0.0, 0.0);
                for &m in mesh.neighbours_of(v) {
                    let m = m as usize;
                    if topos[e].layer[m] >= 0 && topos[e].layer[m] < rim_layer {
                        continue;
                    }
                    let (um, hm) = at(m);
                    let (dx, dy, dh) = (um[0] - u[0], um[1] - u[1], hm - h);
                    sxx += dx * dx;
                    sxy += dx * dy;
                    syy += dy * dy;
                    sxh += dx * dh;
                    syh += dy * dh;
                }
                let det = sxx * syy - sxy * sxy;
                let grad = if det.abs() > 1e-18 { [(syy * sxh - sxy * syh) / det, (sxx * syh - sxy * sxh) / det] } else { [0.0, 0.0] };
                chart[e].insert(v, (u, h, grad));
            }
        }
        self.parents
            .iter()
            .zip(&self.eye)
            .zip(&self.bend)
            .map(|(((tr, w), &e), &bend)| {
                let (c, side, r) = balls[e];
                let k = tr.map(|v| chart[e][&g(v)]);
                let u = [0, 1].map(|d| (0..3).map(|i| w[i] * k[i].0[d]).sum::<f64>());
                let flat_h: f64 = (0..3).map(|i| w[i] * k[i].1).sum();
                let patch: f64 = (0..3).map(|i| w[i] * (k[i].1 + k[i].2[0] * (u[0] - k[i].0[0]) + k[i].2[1] * (u[1] - k[i].0[1]))).sum();
                let (lo, hi) = (0..3).filter(|&i| w[i] > 0.0).fold((f64::MAX, f64::MIN), |(lo, hi), i| (lo.min(k[i].1), hi.max(k[i].1)));
                let h = (flat_h + CURVED * (patch - flat_h)).clamp(lo, hi);
                let curved = from_eye(c, side, scale(unflat(u), r + h));
                let plain = (0..3).fold([0.0; 3], |a, i| crate::add(a, scale(p[g(tr[i])], w[i])));
                crate::add(scale(plain, 1.0 - bend), scale(curved, bend))
            })
            .collect()
    }
}
