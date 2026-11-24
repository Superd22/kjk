//! Bald references for KOTOR's heads, and measures of the whole head: the skull, the forehead, where the face sits, the ears and the nape.
//!
//! KOTOR models hair as the outside of the head, with no scalp under it. A reference is KOTOR's real skin with the hair taken off and the
//! hole filled by a skull learned from the KOTOR heads whose skull shows. This is a measure only. It builds no sliders, and nothing the
//! viewer uses reads it.

use crate::V3;
use crate::feature::Shot;
use crate::measure::Frame;

/// The whole head from the side, from in front, three-quarter from behind and from behind.
pub const SHOTS: [Shot; 4] = [
    Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-175.0, 95.0, -200.0, 140.0] },
    Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-105.0, 105.0, -200.0, 140.0] },
    Shot { name: "three-quarter", yaw: 45.0, pitch: 0.0, pivot: 0.0, window: [-120.0, 120.0, -200.0, 140.0] },
    Shot { name: "back", yaw: 180.0, pitch: 0.0, pivot: 0.0, window: [-105.0, 105.0, -200.0, 140.0] },
];

/// A head's KOTOR hair parts, from the prep folder (`<dir>/<name>.bin`, float32 triangle corners in the fit's frame), in the head frame.
pub fn hair_parts(dir: &std::path::Path, name: &str) -> (Vec<V3>, Vec<[u32; 3]>) {
    let frame = Frame::head();
    let raw = std::fs::read(dir.join(format!("{name}.bin"))).unwrap_or_default();
    let v: Vec<V3> = raw
        .chunks_exact(12)
        .map(|c| {
            let f = |k: usize| f32::from_le_bytes([c[k], c[k + 1], c[k + 2], c[k + 3]]) as f64;
            frame.local([f(0), f(4), f(8)])
        })
        .collect();
    let t = (0..v.len() as u32 / 3).map(|k| [3 * k, 3 * k + 1, 3 * k + 2]).collect();
    (v, t)
}


/// The KOTOR heads whose outer surface is a skull: bald, shaven, or with short hair painted on the skull's own shape. Chosen by eye on the
/// sheet of all 57 heads (`bald look`). Heads with a hair volume (a bob, a bun, a crest, a hood), a hairline step or spikes are left out.
/// pfha02 (a bun at the back) and pfhc02 (a raised crown) were moved out on 2026-11-21 after their colour sheets (`skull colour`).
pub const SKULL_SHOWS: [&str; 36] = [
    "comm_a_f", "comm_a_f2", "comm_a_m", "comm_b_f", "comm_b_m", "comm_w_f", "comm_w_m", "mod_kenobi", "mod_kyle_katarn", "mod_layne",
    "mod_narve_canderous", "n_admoffh", "n_dorakh", "n_hussef", "n_jedimalek", "n_mastkavarh", "n_traskh", "n_zezkaiellh", "n_zherronh",
    "old_a_f", "old_a_m", "old_b_f", "old_b_m", "old_w_f", "old_w_m", "p_baodurh", "p_carthh", "p_joleeh", "pfha05", "pfhb04", "pmha01", "pmha02", "pmha04", "pmha05", "pmhc01", "sith_app",
];

/// A triangle soup that rays are cast against, in the head frame (mm).
pub struct Soup {
    pub tris: Vec<[V3; 3]>,
    normals: Vec<V3>,
}

impl Soup {
    pub fn new(tris: Vec<[V3; 3]>) -> Self {
        let normals = tris.iter().map(|t| crate::normalize(crate::cross(crate::sub(t[1], t[0]), crate::sub(t[2], t[0])))).collect();
        Self { tris, normals }
    }

    /// The hit nearest to `p` along the line `p + t d`, with |t| at most `reach`, on a triangle whose normal is within 70 degrees of `d`
    /// either way when `facing` is false, or facing the same way as `d` when it is true. Returns `t` and the triangle.
    pub fn hit(&self, p: V3, d: V3, reach: f64, facing: bool) -> Option<(f64, usize)> {
        let mut best: Option<(f64, usize)> = None;
        for (k, t) in self.tris.iter().enumerate() {
            let c = crate::dot(self.normals[k], d);
            if (facing && c < 0.34) || (!facing && c.abs() < 0.34) {
                continue;
            }
            let e1 = crate::sub(t[1], t[0]);
            let e2 = crate::sub(t[2], t[0]);
            let h = crate::cross(d, e2);
            let a = crate::dot(e1, h);
            if a.abs() < 1e-12 {
                continue;
            }
            let s = crate::sub(p, t[0]);
            let u = crate::dot(s, h) / a;
            if !(-1e-6..=1.0 + 1e-6).contains(&u) {
                continue;
            }
            let q = crate::cross(s, e1);
            let v = crate::dot(d, q) / a;
            if v < -1e-6 || u + v > 1.0 + 1e-6 {
                continue;
            }
            let tt = crate::dot(e2, q) / a;
            if tt.abs() <= reach && best.is_none_or(|(b, _)| tt.abs() < b.abs()) {
                best = Some((tt, k));
            }
        }
        best
    }
}

impl Soup {
    /// The first hit along `p + t d` with t between 0 and `reach`, on any triangle.
    pub fn first(&self, p: V3, d: V3, reach: f64) -> Option<f64> {
        let mut best: Option<f64> = None;
        for t in &self.tris {
            let e1 = crate::sub(t[1], t[0]);
            let e2 = crate::sub(t[2], t[0]);
            let h = crate::cross(d, e2);
            let a = crate::dot(e1, h);
            if a.abs() < 1e-12 {
                continue;
            }
            let s = crate::sub(p, t[0]);
            let u = crate::dot(s, h) / a;
            let q = crate::cross(s, e1);
            let v = crate::dot(d, q) / a;
            if u < 0.0 || v < 0.0 || u + v > 1.0 {
                continue;
            }
            let tt = crate::dot(e2, q) / a;
            if (0.0..=reach).contains(&tt) && best.is_none_or(|b| tt < b) {
                best = Some(tt);
            }
        }
        best
    }
}

/// Outward normals per merged vertex of our mesh at `merged` places.
pub fn merged_normals(mesh: &crate::mesh::HeadMesh, merged: &[V3]) -> Vec<V3> {
    let mut n = vec![[0.0; 3]; merged.len()];
    for t in &mesh.triangles {
        let g = t.map(|i| mesh.group[i as usize] as usize);
        if g[0] == g[1] || g[1] == g[2] || g[0] == g[2] {
            continue;
        }
        let f = crate::cross(crate::sub(merged[g[1]], merged[g[0]]), crate::sub(merged[g[2]], merged[g[0]]));
        for &k in &g {
            n[k] = crate::add(n[k], f);
        }
    }
    n.iter().map(|&v| if crate::length(v) > 0.0 { crate::normalize(v) } else { v }).collect()
}

/// Spreads values given on `fixed` merged vertices to the others, each the mean of its neighbours (a harmonic fill), by Jacobi steps.
pub fn harmonic(mesh: &crate::mesh::HeadMesh, values: &mut [V3], fixed: &[bool], steps: usize) {
    for _ in 0..steps {
        let prev = values.to_vec();
        for g in 0..values.len() {
            if fixed[g] {
                continue;
            }
            let nb = mesh.neighbours_of(g);
            if nb.is_empty() {
                continue;
            }
            let mut s = [0.0; 3];
            for &j in nb {
                s = crate::add(s, prev[j as usize]);
            }
            values[g] = crate::scale(s, 1.0 / nb.len() as f64);
        }
    }
}

/// A small principal-component model of our merged vertices over a set of heads (the places flattened, mm).
pub struct Model {
    pub mean: Vec<f64>,
    /// Unit columns, one per component.
    pub basis: Vec<Vec<f64>>,
    /// The variance of each component over the heads.
    pub variance: Vec<f64>,
}

impl Model {
    pub fn learn(rows: &[Vec<f64>], components: usize) -> Self {
        let n = rows.len();
        let m = rows[0].len();
        let mean: Vec<f64> = (0..m).map(|j| rows.iter().map(|r| r[j]).sum::<f64>() / n as f64).collect();
        let c: Vec<Vec<f64>> = rows.iter().map(|r| r.iter().zip(&mean).map(|(a, b)| a - b).collect()).collect();
        let gram = nalgebra::DMatrix::from_fn(n, n, |i, j| c[i].iter().zip(&c[j]).map(|(a, b)| a * b).sum::<f64>());
        let eig = nalgebra::SymmetricEigen::new(gram);
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| eig.eigenvalues[b].total_cmp(&eig.eigenvalues[a]));
        let mut basis = Vec::new();
        let mut variance = Vec::new();
        for &k in order.iter().take(components.min(n - 1)) {
            let l = eig.eigenvalues[k];
            if l <= 1e-9 {
                break;
            }
            let mut u = vec![0.0; m];
            for i in 0..n {
                let w = eig.eigenvectors[(i, k)] / l.sqrt();
                for j in 0..m {
                    u[j] += w * c[i][j];
                }
            }
            basis.push(u);
            variance.push(l / (n - 1) as f64);
        }
        Self { mean, basis, variance }
    }

    /// The model's shape that best matches `target` on the entries `known`, with each component held toward 0 by its own variance
    /// against a noise of `noise` mm per entry.
    pub fn complete(&self, target: &[f64], known: &[bool], noise: f64) -> Vec<f64> {
        let k = self.basis.len();
        let mut a = nalgebra::DMatrix::<f64>::zeros(k, k);
        let mut b = nalgebra::DVector::<f64>::zeros(k);
        for j in (0..target.len()).filter(|&j| known[j]) {
            let r = target[j] - self.mean[j];
            for p in 0..k {
                b[p] += self.basis[p][j] * r;
                for q in 0..k {
                    a[(p, q)] += self.basis[p][j] * self.basis[q][j];
                }
            }
        }
        for p in 0..k {
            a[(p, p)] += noise * noise / self.variance[p];
        }
        let c = a.lu().solve(&b).unwrap_or_else(|| nalgebra::DVector::zeros(k));
        let mut out = self.mean.clone();
        for p in 0..k {
            for (j, o) in out.iter_mut().enumerate() {
                *o += c[p] * self.basis[p][j];
            }
        }
        out
    }
}


/// A surface to measure: triangles in the head frame (mm), each marked when it is filled in rather than KOTOR's or ours for real.
pub struct Shape {
    pub tris: Vec<[V3; 3]>,
    pub filled: Vec<bool>,
}

/// The rows of the whole-head measure, in mm or degrees, in the head frame (the eyes' centres at 0, y up, the face along +z).
pub const ROWS: [(&str, &str); 19] = [
    ("top", "Height of the top of the head above the eyes"),
    ("chin", "Depth of the bottom of the chin below the eyes"),
    ("face_height", "Face height: eyes to the bottom of the chin"),
    ("brow_front", "How far the brow (the front of the forehead 10 to 30 mm above the eyes) lies in front of the eyes"),
    ("back", "How far the back of the head lies behind the eyes"),
    ("back_height", "Height of the back of the head's farthest point above the eyes"),
    ("length", "Skull length: brow to the back of the head"),
    ("width", "Skull width, the widest level cut 30 to 90 mm above the eyes"),
    ("length_ratio", "Skull length over face height"),
    ("width_ratio", "Skull width over face height"),
    ("top_ratio", "Height of the top of the head over face height"),
    ("forehead_slope", "Slope of the forehead from upright, brow to 45 mm above it, seen from the side (positive leans back)"),
    ("crown_front", "How far in front of the back of the head the top of the head lies, over the skull length"),
    ("nape_in", "How far the back of the neck, 15 mm below the chin's bottom, lies in front of the back of the head"),
    ("nape_slope", "Slope of the back of the neck from upright, from 5 mm above to 25 mm below the chin's bottom (positive leans forward at the top)"),
    ("crown_curve", "How far the back of the head's outline bulges behind the straight line from the top of the head to the nape"),
    ("face_forward", "How far forward the eyes sit on the skull: the eyes' distance from the back over the skull length"),
    ("neck_depth", "Depth of the neck front to back, 15 mm below the chin's bottom"),
    ("neck_under", "How far the neck's middle, 15 mm below the chin's bottom, lies in front of the skull's middle (between brow and back)"),
];

/// A cut through a shape by a plane across one axis (`axis` 0 for x, 1 for y), as segments in the two other axes: (z, y) for x, (x, z)
/// for y. Each segment says whether its triangle is filled.
pub fn cut(shape: &Shape, axis: usize, at: f64) -> Vec<([[f64; 2]; 2], bool)> {
    let (a, b) = if axis == 0 { (2, 1) } else { (0, 2) };
    let mut out = Vec::new();
    for (t, &f) in shape.tris.iter().zip(&shape.filled) {
        let d = t.map(|p| if p[axis] == at { 1e-9 } else { p[axis] - at });
        let mut ends = Vec::with_capacity(2);
        for (i, j) in [(0, 1), (1, 2), (2, 0)] {
            if (d[i] > 0.0) != (d[j] > 0.0) {
                let s = d[i] / (d[i] - d[j]);
                ends.push([t[i][a] + s * (t[j][a] - t[i][a]), t[i][b] + s * (t[j][b] - t[i][b])]);
            }
        }
        if ends.len() == 2 {
            out.push(([ends[0], ends[1]], f));
        }
    }
    out
}

/// Along a cut, at each level `v` of the segments' second coordinate, the largest and the smallest first coordinate, and whether each
/// lies on a filled triangle. `None` where nothing crosses the level.
pub fn extent(segs: &[([[f64; 2]; 2], bool)], v: f64) -> Option<((f64, bool), (f64, bool))> {
    let mut hi: Option<(f64, bool)> = None;
    let mut lo: Option<(f64, bool)> = None;
    for ([p, q], f) in segs {
        if (p[1] - v) * (q[1] - v) > 0.0 || p[1] == q[1] {
            continue;
        }
        let u = p[0] + (v - p[1]) / (q[1] - p[1]) * (q[0] - p[0]);
        if hi.is_none_or(|h| u > h.0) {
            hi = Some((u, *f));
        }
        if lo.is_none_or(|l| u < l.0) {
            lo = Some((u, *f));
        }
    }
    Some((hi?, lo?))
}

/// The front and the back of a shape at height `y`, read in a band 8 mm either side of the middle (a level cut, so a seam or a gap on
/// the middle line is not missed), with whether each lies on a filled triangle.
pub fn level(shape: &Shape, y: f64) -> Option<((f64, bool), (f64, bool))> {
    let mut hi: Option<(f64, bool)> = None;
    let mut lo: Option<(f64, bool)> = None;
    for ([p, q], f) in cut(shape, 1, y) {
        // the part of the segment (x, z) with |x| at most 8 mm
        let (mut a, mut b) = (0.0f64, 1.0f64);
        let dx = q[0] - p[0];
        for bound in [-8.0, 8.0] {
            if dx.abs() < 1e-12 {
                if p[0].abs() > 8.0 {
                    a = 1.0;
                    b = 0.0;
                }
                continue;
            }
            let t = (bound - p[0]) / dx;
            if (bound > 0.0) == (dx > 0.0) { b = b.min(t) } else { a = a.max(t) }
        }
        if a > b {
            continue;
        }
        for t in [a, b] {
            let z = p[1] + t * (q[1] - p[1]);
            if hi.is_none_or(|h| z > h.0) {
                hi = Some((z, f));
            }
            if lo.is_none_or(|l| z < l.0) {
                lo = Some((z, f));
            }
        }
    }
    Some((hi?, lo?))
}

/// The side outline of a shape (the middle cut): for heights from -130 to 150 mm every 1 mm, the front and the back.
pub fn side_outline(shape: &Shape) -> Vec<(f64, Option<((f64, bool), (f64, bool))>)> {
    let segs = cut(shape, 0, 0.0);
    (0..=280).map(|k| -130.0 + k as f64).map(|y| (y, extent(&segs, y))).collect()
}

/// The front outline (the widest place on each side at each height, every 1 mm from -130 to 150 mm), as (height, left, right).
pub fn front_outline(shape: &Shape) -> Vec<(f64, Option<f64>, Option<f64>)> {
    (0..=280)
        .map(|k| -130.0 + k as f64)
        .map(|y| {
            let segs = cut(shape, 1, y);
            let r = segs.iter().flat_map(|(s, _)| s.iter().map(|p| p[0])).fold(None::<f64>, |m, x| Some(m.map_or(x, |m| m.max(x))));
            let l = segs.iter().flat_map(|(s, _)| s.iter().map(|p| p[0])).fold(None::<f64>, |m, x| Some(m.map_or(x, |m| m.min(x))));
            (y, l, r)
        })
        .collect()
}

/// The whole head measured: each row's value (NaN when it cannot be read) and whether it rests on a filled part.
pub fn measure_head(shape: &Shape) -> Vec<(f64, bool)> {
    let side = side_outline(shape);
    let at = |y: f64| level(shape, y);
    // the top: the highest level the middle cut reaches
    let top = side.iter().rev().find(|(_, e)| e.is_some()).map(|(y, e)| (*y, e.unwrap().0.1 && e.unwrap().1.1));
    // the chin, as the jaw measure reads it: its front is the front-most point of the middle cut 85 to 125 mm below the eyes, and its
    // bottom the lowest point of the cut within 25 mm behind that front
    let mid = cut(shape, 0, 0.0);
    let pts: Vec<([f64; 2], bool)> = mid.iter().flat_map(|(s, f)| s.iter().map(move |p| (*p, *f))).collect();
    let chin_front = pts.iter().filter(|(p, _)| p[1] < -85.0 && p[1] > -125.0).map(|(p, _)| p[0]).fold(f64::NAN, f64::max);
    let chin = pts
        .iter()
        .filter(|(p, _)| p[1] < -60.0 && p[0] > chin_front - 25.0)
        .fold(None::<(f64, bool)>, |m, (p, f)| if m.is_none_or(|m| p[1] < m.0) { Some((p[1], *f)) } else { m });
    let brow = (10..=30).filter_map(|y| at(y as f64)).map(|e| e.0).fold(None::<(f64, bool)>, |m, x| if m.is_none_or(|m| x.0 > m.0) { Some(x) } else { m });
    let brow_y = (10..=30).map(|y| y as f64).filter(|&y| at(y).is_some_and(|e| Some(e.0.0) == brow.map(|b| b.0))).next().unwrap_or(20.0);
    let backs: Vec<(f64, (f64, bool))> = (-40..=150).filter_map(|y| at(y as f64).map(|e| (y as f64, e.1))).collect();
    let back = backs.iter().copied().fold(None::<(f64, (f64, bool))>, |m, x| if m.is_none_or(|m| x.1.0 < m.1.0) { Some(x) } else { m });
    // the height of the back of the head: the middle of the heights whose back lies within 3 mm of the farthest
    let back = back.map(|(_, b)| {
        let near: Vec<f64> = backs.iter().filter(|(_, e)| e.0 < b.0 + 3.0).map(|(y, _)| *y).collect();
        (0.5 * (near[0] + near[near.len() - 1]), b)
    });
    let width = (30..=90)
        .map(|y| {
            let segs = cut(shape, 1, y as f64);
            let mut hi: Option<(f64, bool)> = None;
            let mut lo: Option<(f64, bool)> = None;
            for (s, f) in &segs {
                for p in s {
                    if hi.is_none_or(|h| p[0] > h.0) {
                        hi = Some((p[0], *f));
                    }
                    if lo.is_none_or(|l| p[0] < l.0) {
                        lo = Some((p[0], *f));
                    }
                }
            }
            hi.zip(lo).map(|(h, l)| (h.0 - l.0, h.1 || l.1))
        })
        .fold(None::<(f64, bool)>, |m, x| match x {
            Some(x) if m.is_none_or(|m| x.0 > m.0) => Some(x),
            _ => m,
        });
    let nan = (f64::NAN, false);
    let top = top.unwrap_or(nan);
    let chin = chin.unwrap_or(nan);
    let brow = brow.unwrap_or(nan);
    let (back_y, back) = back.unwrap_or((f64::NAN, nan));
    let _ = &side;
    let width = width.unwrap_or(nan);
    let face = (-chin.0, chin.1);
    let length = (brow.0 - back.0, brow.1 || back.1);
    let forehead = at(brow_y + 45.0).map(|e| (((brow.0 - e.0.0) / 45.0).atan().to_degrees(), brow.1 || e.0.1)).unwrap_or(nan);
    let crown = side.iter().rev().find(|(_, e)| e.is_some()).and_then(|(_, e)| *e).map(|e| 0.5 * (e.0.0 + e.1.0));
    let crown_front = crown.map(|c| ((c - back.0) / length.0, top.1 || back.1)).unwrap_or(nan);
    let neck_y = chin.0 - 15.0;
    let nape = at(neck_y).map(|e| (e.1.0 - back.0, e.1.1 || back.1)).unwrap_or(nan);
    let nape_slope = at(neck_y + 20.0).zip(at(neck_y - 10.0)).map(|(a, b)| (((a.1.0 - b.1.0) / 30.0).atan().to_degrees(), a.1.1 || b.1.1)).unwrap_or(nan);
    // the back outline from the top of the head to the nape: how far it lies behind the straight line between them, at most
    let crown_curve = match (at(top.0 - 3.0), at(neck_y)) {
        (Some(te), Some(ne)) => {
            let (y0, z0, y1, z1) = (top.0 - 3.0, te.1.0, neck_y, ne.1.0);
            let mut best = (0.0f64, false);
            for y in (y1.ceil() as i64)..=(y0.floor() as i64) {
                if let Some(e) = at(y as f64) {
                    let chord = z0 + (y as f64 - y0) / (y1 - y0) * (z1 - z0);
                    if chord - e.1.0 > best.0 {
                        best = (chord - e.1.0, e.1.1);
                    }
                }
            }
            best
        }
        _ => nan,
    };
    vec![
        top,
        chin,
        face,
        brow,
        (-back.0, back.1),
        (back_y, back.1),
        length,
        width,
        (length.0 / face.0, length.1 || face.1),
        (width.0 / face.0, width.1 || face.1),
        (top.0 / face.0, top.1 || face.1),
        forehead,
        crown_front,
        nape,
        nape_slope,
        crown_curve,
        (-back.0 / length.0, length.1),
        at(neck_y).map(|e| (e.0.0 - e.1.0, e.0.1 || e.1.1)).unwrap_or(nan),
        at(neck_y).map(|e| (0.5 * (e.0.0 + e.1.0) - 0.5 * (brow.0 + back.0), length.1 || e.0.1 || e.1.1)).unwrap_or(nan),
    ]
}

pub mod tools {
    use super::*;
    use crate::mesh::input::Input;
    use crate::render::{clay, label, sheet};
    use std::path::{Path, PathBuf};

    pub fn names(input: &Input) -> Vec<String> {
        let only: Option<Vec<String>> = std::env::var("BALD_HEADS").ok().map(|s| s.split(',').map(String::from).collect());
        input.manifest["heads"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["name"].as_str().unwrap().to_string())
            .filter(|n| only.as_ref().is_none_or(|o| o.contains(n)))
            .collect()
    }

    /// KOTOR's whole model of one head in the head frame: the face mesh, which of its vertices are skin by the texture's colour, and
    /// the hair parts appended after it.
    pub struct Kotor {
        pub v: Vec<V3>,
        pub f: Vec<[u32; 3]>,
        /// Per vertex of `v`, for the face mesh only.
        pub skin: Vec<bool>,
        /// How many of `v` and `f` are the face mesh. The rest are the hair parts.
        pub face_v: usize,
        pub face_f: usize,
    }

    pub fn kotor(input: &Input, hair: &Path, name: &str) -> Kotor {
        let frame = Frame::head();
        let mut v: Vec<V3> = input.points(&format!("heads/{name}/kotor_v")).iter().map(|&p| frame.local(p)).collect();
        let mut f = input.triangles(&format!("heads/{name}/kotor_f"));
        let key = format!("extra/heads/{name}/kotor_skin");
        let skin: Vec<bool> = if input.has(&key) { input.u8s(&key).iter().map(|&s| s != 0).collect() } else { vec![true; v.len()] };
        let (face_v, face_f) = (v.len(), f.len());
        let (hv, hf) = hair_parts(hair, name);
        let o = v.len() as u32;
        v.extend(hv);
        f.extend(hf.iter().map(|t| t.map(|i| i + o)));
        Kotor { v, f, skin, face_v, face_f }
    }

    /// Which of a head's KOTOR triangles are hair: every triangle of the hair parts, and on pmhc04 the face mesh's dark brown triangles
    /// (texture lightness under 45, or under 52 with little red), except inside the face and the beard in front of the ears, which lie
    /// on the face's own shape.
    pub fn hair_faces(input: &Input, name: &str, k: &Kotor) -> Vec<bool> {
        let mut out: Vec<bool> = (0..k.f.len()).map(|i| i >= k.face_f).collect();
        if FACE_HAIR.contains(&name) {
            let painted = painted_hair(input, name, k);
            for (o, p) in out.iter_mut().zip(painted) {
                *o |= p;
            }
        }
        if name == "pmhc04" {
            let lab = input.f32s(&format!("extra/heads/{name}/kotor_colour"));
            for (i, t) in k.f.iter().take(k.face_f).enumerate() {
                let (l, a) = (lab[3 * i], lab[3 * i + 1]);
                let c = crate::scale(crate::add(crate::add(k.v[t[0] as usize], k.v[t[1] as usize]), k.v[t[2] as usize]), 1.0 / 3.0);
                let dark = l < 45.0 || (l < 52.0 && a < 8.0);
                // inside the face (between the brows, the eyes' outer corners and the chin) nothing is hair
                let face = c[0].abs() < 48.0 && c[1] < 32.0 && c[2] > -15.0;
                let beard = c[1] < -8.0 && c[2] > EAR_FRONT_Z;
                out[i] = dark && !face && !beard;
            }
        }
        out
    }

    /// The heads whose face mesh carries a hair volume or a hood, found by `painted_hair`. pmhc04 has its own rule in `hair_faces`.
    pub const FACE_HAIR: [&str; 12] = [
        "mod_exile_canon", "mod_hp_pfha01", "mod_keanu", "n_daraalah", "n_terenaadareh", "p_handmaidenbb", "p_kreiah", "pfha02", "pfha06",
        "pfhc02", "pmha06", "pmhc03",
    ];

    /// The face mesh's triangles that the texture's colour calls not skin (`kotor_skin`), except inside the face, at the temples in
    /// front of the ears, in the beard and on the neck's front and sides, which lie on the head's own shape. Behind the ears the cut
    /// reaches down the neck, where long hair hangs. Kreia's hood and the Handmaiden's white hair are found by their grey, and Terena's crest by its dark brown,
    /// because the skin test keeps parts of both.
    pub fn painted_hair(input: &Input, name: &str, k: &Kotor) -> Vec<bool> {
        let lab = input.f32s(&format!("extra/heads/{name}/kotor_colour"));
        (0..k.f.len())
            .map(|i| {
                if i >= k.face_f {
                    return false;
                }
                let t = k.f[i];
                let c = crate::scale(crate::add(crate::add(k.v[t[0] as usize], k.v[t[1] as usize]), k.v[t[2] as usize]), 1.0 / 3.0);
                // pfha01's fringe hangs over the forehead down to its brows: its face starts below the fringe
                let face_top = if name == "mod_hp_pfha01" { 18.0 } else { 32.0 };
                let face = c[0].abs() < 48.0 && c[1] < face_top && c[2] > -15.0;
                let temple = c[1] < 20.0 && c[2] > -45.0;
                let beard = c[1] < -8.0 && c[2] > EAR_FRONT_Z;
                let neck = if c[2] > EAR_FRONT_Z { c[1] < NECK_Y } else { c[1] < NAPE_Y };
                let (l, a, b) = (lab[3 * i], lab[3 * i + 1], lab[3 * i + 2]);
                let not_skin = match name {
                    "p_kreiah" | "p_handmaidenbb" => a.abs() < 6.0 && b.abs() < 12.0,
                    "n_terenaadareh" => l < 45.0 || t.iter().any(|&j| !k.skin[j as usize]),
                    _ => t.iter().any(|&j| !k.skin[j as usize]),
                };
                // Kreia's hood and pfha06's bob hang beside the face and down the neck
                let hood = name == "p_kreiah" || name == "pfha06";
                not_skin && !face && (hood || (!temple && !beard && !neck))
            })
            .collect()
    }

    /// In front of the ears, below this height (mm under the eyes) the colour test never calls the face mesh hair: it is the neck.
    const NECK_Y: f64 = -45.0;
    /// Behind the ears, the same for the nape, lower down.
    const NAPE_Y: f64 = -100.0;

    /// How far back (mm, head frame) the front of the ear lies; the beard rule of pmhc04 keeps dark skin in front of it.
    const EAR_FRONT_Z: f64 = -62.0;

    /// Our merged vertices of the ear area (`extra/ears`).
    pub fn ear_area(input: &Input, mesh: &crate::mesh::HeadMesh) -> Vec<bool> {
        let ears = input.u8s("extra/ears");
        mesh.first.iter().map(|&i| ears[i as usize] != 0).collect()
    }

    /// Our merged vertices the skull model covers: skin above the neck's cut (`extra/below` is the neck and shoulders under it).
    pub fn region(input: &Input, mesh: &crate::mesh::HeadMesh) -> Vec<bool> {
        let below = input.u8s("extra/below");
        (0..mesh.merged_count()).map(|g| !mesh.hidden[g] && below[mesh.first[g] as usize] == 0).collect()
    }

    /// Our mesh laid on a KOTOR head: each merged vertex of the head's fit (`fit_smooth`, head frame, mm) moved along its normal onto
    /// KOTOR's triangles that are not hair, within REACH. Returns the places (the fit's own where nothing was hit) and which were hit.
    pub fn sample(input: &Input, mesh: &crate::mesh::HeadMesh, name: &str, k: &Kotor, hair: &[bool], region: &[bool]) -> (Vec<V3>, Vec<bool>) {
        let frame = Frame::head();
        let fit: Vec<V3> = input.points(&format!("heads/{name}/fit_smooth")).iter().map(|&p| frame.local(p)).collect();
        let carrier = mesh.merge(&fit);
        let normals = merged_normals(mesh, &carrier);
        let soup = Soup::new(k.f.iter().zip(hair).filter(|(_, h)| !**h).map(|(t, _)| t.map(|i| k.v[i as usize])).collect());
        let mut pos = carrier.clone();
        let mut known = vec![false; carrier.len()];
        for g in 0..carrier.len() {
            if !region[g] {
                continue;
            }
            if let Some((t, _)) = soup.hit(carrier[g], normals[g], REACH, false) {
                pos[g] = crate::add(carrier[g], crate::scale(normals[g], t));
                known[g] = true;
            }
        }
        (pos, known)
    }

    /// How far along its normal a vertex of our fit may move to reach KOTOR's surface, in mm.
    const REACH: f64 = 20.0;

    fn flat(p: &[V3]) -> Vec<f64> {
        p.iter().flat_map(|v| v.iter().copied()).collect()
    }

    fn unflat(f: &[f64]) -> Vec<V3> {
        f.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
    }

    /// The bald reference of one head: the places KOTOR's skin gives, and the rest from the skull model joined to them by a harmonic
    /// fill of the gap, then kept `thickness` mm under the hair. Returns the places and which are real.
    #[allow(clippy::too_many_arguments)]
    pub fn reference(
        mesh: &crate::mesh::HeadMesh,
        model: &Model,
        region: &[bool],
        sampled: &[V3],
        known: &[bool],
        ear: &[bool],
        hair_soup: Option<&Soup>,
        thickness: f64,
    ) -> Vec<V3> {
        // the ear is left to KOTOR's own ear mesh: our ear area is closed smoothly from the skin around it
        let known: Vec<bool> = known.iter().zip(ear).map(|(&k, &e)| k && !e).collect();
        let known = &known[..];
        let known3: Vec<bool> = known.iter().flat_map(|&k| [k; 3]).collect();
        let guess = unflat(&model.complete(&flat(sampled), &known3, NOISE));
        // the gap between the model and the real skin, carried over the filled part so the two join without a step
        let mut gap: Vec<V3> = (0..sampled.len()).map(|g| if known[g] { crate::sub(sampled[g], guess[g]) } else { [0.0; 3] }).collect();
        harmonic(mesh, &mut gap, known, 400);
        let mut out: Vec<V3> = (0..sampled.len()).map(|g| if known[g] || !region[g] { sampled[g] } else { crate::add(guess[g], gap[g]) }).collect();
        let not_ear: Vec<bool> = ear.iter().map(|e| !e).collect();
        harmonic(mesh, &mut out, &not_ear, 200);
        if let Some(soup) = hair_soup {
            for _ in 0..3 {
                let normals = merged_normals(mesh, &out);
                let mut push: Vec<V3> = vec![[0.0; 3]; out.len()];
                let mut fixed = known.to_vec();
                for g in 0..out.len() {
                    if known[g] || !region[g] {
                        continue;
                    }
                    // the hair's surface along the normal, looking from 10 mm inside the vertex to 60 mm outside it
                    if let Some(t) = soup.first(crate::sub(out[g], crate::scale(normals[g], 10.0)), normals[g], 70.0) {
                        let room = t - 10.0;
                        if room < thickness {
                            push[g] = crate::scale(normals[g], room - thickness);
                            fixed[g] = true;
                        }
                    }
                }
                harmonic(mesh, &mut push, &fixed, 60);
                for g in 0..out.len() {
                    if !known[g] && region[g] {
                        out[g] = crate::add(out[g], push[g]);
                    }
                }
            }
        }
        out
    }

    /// The noise the skull model allows on each known coordinate, in mm.
    const NOISE: f64 = 1.5;
    /// Components of the skull model.
    const COMPONENTS: usize = 12;

    pub fn run(command: &str, args: &[String], data: &Path, input: &Input) -> bool {
        let _ = data;
        let hair = PathBuf::from(std::env::var("BALD_HAIR").expect("BALD_HAIR: the folder of the hair parts"));
        match command {
            // KOTOR's heads whole, with colour-found hair in blue and hair parts in orange: `look <data> <out prefix> [per]`
            "look" => {
                let prefix = args.get(4).cloned().unwrap_or_else(|| "look".into());
                let per: usize = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(8);
                let names = names(input);
                for (page, chunk) in names.chunks(per).enumerate() {
                    let mut tiles = Vec::new();
                    for name in chunk {
                        let k = kotor(input, &hair, name);
                        let tint: Vec<Option<[f64; 3]>> = k
                            .f
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                if i >= k.face_f {
                                    Some([230.0, 140.0, 90.0])
                                } else if t.iter().any(|&j| !k.skin[j as usize]) {
                                    Some([90.0, 140.0, 230.0])
                                } else {
                                    None
                                }
                            })
                            .collect();
                        for (s, shot) in [SHOTS[0], SHOTS[2], SHOTS[3]].iter().enumerate() {
                            let mut img = clay(&k.v, &k.f, shot, &[], &tint);
                            if s == 0 {
                                label(&mut img, name);
                            }
                            tiles.push(img);
                        }
                    }
                    let out = format!("{prefix}_{page}.png");
                    sheet(&tiles, 6).save(&out).unwrap();
                    eprintln!("wrote {out}");
                }
                true
            }
            // How thick KOTOR's hair parts are where they lie over a real skull: for each hair vertex above the eyes, the distance to
            // the face mesh toward the middle of the head: `thickness <data>`
            "thickness" => {
                let centre = [0.0, 20.0, -45.0];
                for name in names(input) {
                    let k = kotor(input, &hair, &name);
                    if k.v.len() == k.face_v {
                        continue;
                    }
                    let face = Soup::new(k.f[..k.face_f].iter().map(|t| t.map(|i| k.v[i as usize])).collect());
                    let mut d: Vec<f64> = k.v[k.face_v..]
                        .iter()
                        .filter(|p| p[1] > 15.0)
                        .filter_map(|&p| {
                            let dir = crate::normalize(crate::sub(centre, p));
                            face.first(p, dir, 60.0)
                        })
                        .collect();
                    d.sort_by(f64::total_cmp);
                    let q = |f: f64| if d.is_empty() { f64::NAN } else { d[((d.len() - 1) as f64 * f) as usize] };
                    let above = k.v[k.face_v..].iter().filter(|p| p[1] > 15.0).count();
                    println!("{name:24} hair points above the eyes {above:6}, reach the skull {:6}, distance 10% {:5.1} 25% {:5.1} median {:5.1} 75% {:5.1}", d.len(), q(0.1), q(0.25), q(0.5), q(0.75));
                }
                true
            }
            // The skull model, its test on the training heads, and the references: `fill <data> <out dir> <thickness mm> [head...]`
            "fill" => {
                let out = PathBuf::from(args.get(4).cloned().unwrap_or_else(|| "bald_out".into()));
                std::fs::create_dir_all(&out).unwrap();
                let thickness: f64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(5.0);
                let targets: Vec<String> = if args.len() > 6 { args[6..].to_vec() } else { vec!["pmhc04".into(), "p_bastilah".into()] };
                let mesh = input.head_mesh();
                let region = region(input, &mesh);
                let ear = ear_area(input, &mesh);
                let all = names(input);
                let train: Vec<String> = SKULL_SHOWS.iter().map(|s| s.to_string()).filter(|n| all.contains(n)).collect();
                let sampled: Vec<(Vec<V3>, Vec<bool>)> = crate::heads::parallel(train.len(), crate::heads::threads(), |i| {
                    let k = kotor(input, &hair, &train[i]);
                    let hf = hair_faces(input, &train[i], &k);
                    sample(input, &mesh, &train[i], &k, &hf, &region)
                });
                let rows: Vec<Vec<f64>> = sampled.iter().map(|(p, _)| flat(p)).collect();
                for (n, (_, kn)) in train.iter().zip(&sampled) {
                    let cover = kn.iter().zip(&region).filter(|(k, r)| **k && **r).count() as f64 / region.iter().filter(|r| **r).count() as f64;
                    eprintln!("{n:24} our skin laid on KOTOR's: {:.0}%", 100.0 * cover);
                }
                // the targets' own holes
                let tk: Vec<(Kotor, Vec<bool>, Vec<V3>, Vec<bool>)> = targets
                    .iter()
                    .map(|n| {
                        let k = kotor(input, &hair, n);
                        let hf = hair_faces(input, n, &k);
                        let (p, kn) = sample(input, &mesh, n, &k, &hf, &region);
                        (k, hf, p, kn)
                    })
                    .collect();
                // leave-one-out: each training head with a target's hole cut in it, filled by the model of the others
                let mut test = serde_json::Map::new();
                for (ti, tname) in targets.iter().enumerate() {
                    let hole: Vec<bool> = (0..region.len()).map(|g| region[g] && !ear[g] && !tk[ti].3[g]).collect();
                    let errs: Vec<(String, f64, f64)> = crate::heads::parallel(train.len(), crate::heads::threads(), |i| {
                        let others: Vec<Vec<f64>> = rows.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, r)| r.clone()).collect();
                        let m = Model::learn(&others, COMPONENTS);
                        let known: Vec<bool> = (0..region.len()).map(|g| region[g] && !hole[g]).collect();
                        let got = reference(&mesh, &m, &region, &sampled[i].0, &known, &ear, None, thickness);
                        let normals = merged_normals(&mesh, &sampled[i].0);
                        let d: Vec<f64> = (0..region.len())
                            .filter(|&g| hole[g] && sampled[i].1[g])
                            .map(|g| crate::dot(crate::sub(got[g], sampled[i].0[g]), normals[g]))
                            .collect();
                        let rms = (d.iter().map(|x| x * x).sum::<f64>() / d.len().max(1) as f64).sqrt();
                        let mut a: Vec<f64> = d.iter().map(|x| x.abs()).collect();
                        a.sort_by(f64::total_cmp);
                        (train[i].clone(), rms, a.get(a.len() * 95 / 100).copied().unwrap_or(f64::NAN))
                    });
                    let mut r: Vec<f64> = errs.iter().map(|e| e.1).collect();
                    r.sort_by(f64::total_cmp);
                    eprintln!("{tname}'s hole cut in the training heads: rms gap along the normal, median {:.2} mm, worst {:.2} mm", r[r.len() / 2], r[r.len() - 1]);
                    for e in &errs {
                        eprintln!("   {:24} rms {:5.2} p95 {:5.2}", e.0, e.1, e.2);
                    }
                    test.insert(tname.clone(), serde_json::json!(errs.iter().map(|e| serde_json::json!({"head": e.0, "rms": e.1, "p95": e.2})).collect::<Vec<_>>()));
                }
                let model = Model::learn(&rows, COMPONENTS);
                eprintln!("skull model: {} components, sd {:?}", model.variance.len(), model.variance.iter().map(|v| (v.sqrt() * 10.0).round() / 10.0).collect::<Vec<_>>());
                let mut summary = serde_json::Map::new();
                for (ti, tname) in targets.iter().enumerate() {
                    let (k, hf, p, kn) = &tk[ti];
                    let hair_soup = Soup::new(k.f.iter().zip(hf).filter(|(_, h)| **h).map(|(t, _)| t.map(|i| k.v[i as usize])).collect());
                    let r = reference(&mesh, &model, &region, p, kn, &ear, Some(&hair_soup), thickness);
                    let kn: Vec<bool> = kn.iter().zip(&ear).map(|(&a, &e)| a && !e).collect();
                    let real = kn.iter().zip(&region).filter(|(a, b)| **a && **b).count();
                    let filled = region.iter().filter(|r| **r).count() - real;
                    eprintln!("{tname}: {real} merged vertices are KOTOR's real skin, {filled} are filled");
                    let bytes: Vec<u8> = r.iter().flat_map(|v| v.iter().flat_map(|x| x.to_le_bytes())).collect();
                    std::fs::write(out.join(format!("{tname}_ref.bin")), bytes).unwrap();
                    std::fs::write(out.join(format!("{tname}_real.bin")), kn.iter().map(|&b| b as u8).collect::<Vec<u8>>()).unwrap();
                    summary.insert(tname.clone(), serde_json::json!({"real": real, "filled": filled}));
                }
                summary.insert("test".into(), serde_json::Value::Object(test));
                summary.insert("training".into(), serde_json::json!(train));
                summary.insert("thickness".into(), serde_json::json!(thickness));
                std::fs::write(out.join("fill.json"), serde_json::to_string_pretty(&serde_json::Value::Object(summary)).unwrap()).unwrap();
                true
            }
            // The references drawn: KOTOR's real skin in clay, the filled skull in orange, then the same with KOTOR's hair in blue, from
            // the side, the front, three-quarter and behind: `show <data> <ref dir> <out.png> [head...]`
            "show" => {
                let dir = PathBuf::from(&args[4]);
                let out = args[5].clone();
                let targets: Vec<String> = if args.len() > 6 { args[6..].to_vec() } else { vec!["pmhc04".into(), "p_bastilah".into()] };
                let mesh = input.head_mesh();
                let region = region(input, &mesh);
                let mut tiles = Vec::new();
                for name in &targets {
                    let r = load_reference(&dir, name, &mesh, &region);
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    for with_hair in [false, true] {
                        let (v, f, tint) = reference_mesh(&k, &hf, &r, &mesh, with_hair);
                        for (s, shot) in SHOTS.iter().enumerate() {
                            let mut img = clay(&v, &f, shot, &[], &tint);
                            if s == 0 {
                                label(&mut img, &format!("{name} {}", if with_hair { "hair on" } else { "hair off" }));
                            }
                            tiles.push(img);
                        }
                    }
                }
                sheet(&tiles, 4).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // The whole head measured on every head: KOTOR's mesh without its hair parts, the same subdivided twice, viewer 2's live
            // head and the FLAME fit, and on the references: `measure <data> <ref dir> <out.json>`
            "measure" => {
                let dir = PathBuf::from(&args[4]);
                let out = args[5].clone();
                let mesh = input.head_mesh();
                let region = region(input, &mesh);
                let heads = crate::jaw_edge::tools::Heads::load(input, data);
                let frame = Frame::head();
                let names = names(input);
                let ours = |p: &[V3]| our_shape(&mesh, &p.iter().map(|&q| frame.local(q)).collect::<Vec<_>>());
                let as_json = |m: Vec<(f64, bool)>| serde_json::json!(m.iter().map(|(v, f)| serde_json::json!([if v.is_nan() { serde_json::Value::Null } else { serde_json::json!(v) }, f])).collect::<Vec<_>>());
                let rows: Vec<serde_json::Value> = crate::heads::parallel(names.len(), crate::heads::threads(), |i| {
                    let name = &names[i];
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    let ks = kotor_shape(&k, &hf);
                    let (sv, sf) = subdivided(&k, &hf);
                    let sub = Shape { filled: vec![false; sf.len()], tris: sf.iter().map(|t| t.map(|j| sv[j as usize])).collect() };
                    let (_, live) = heads.pair(input, name);
                    let fit = input.points(&format!("heads/{name}/fit_smooth"));
                    let mut row = serde_json::json!({
                        "name": name,
                        "skull_shows": SKULL_SHOWS.contains(&name.as_str()),
                        "kotor": as_json(measure_head(&ks)),
                        "subdivided": as_json(measure_head(&sub)),
                        "live": as_json(measure_head(&ours(&live))),
                        "fit": as_json(measure_head(&ours(&fit))),
                    });
                    if dir.join(format!("{name}_ref.bin")).exists() {
                        let r = load_reference(&dir, name, &mesh, &region);
                        let rs = reference_shape(&k, &hf, &r, &mesh);
                        row["reference"] = as_json(measure_head(&rs));
                        let all = Shape { tris: k.f.iter().map(|t| t.map(|j| k.v[j as usize])).collect(), filled: vec![false; k.f.len()] };
                        row["kotor_with_hair"] = as_json(measure_head(&all));
                    }
                    row
                });
                let keys: Vec<&str> = ROWS.iter().map(|r| r.0).collect();
                std::fs::write(&out, serde_json::to_string(&serde_json::json!({"rows": keys, "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // Our live head's and FLAME fit's outlines over KOTOR's reference, from the side and the front, in the same camera:
            // `overlay <data> <ref dir> <out prefix> [head...]`
            "overlay" => {
                let dir = PathBuf::from(&args[4]);
                let prefix = args[5].clone();
                let targets: Vec<String> = if args.len() > 6 { args[6..].to_vec() } else { vec!["pmhc04".into(), "p_bastilah".into()] };
                let mesh = input.head_mesh();
                let region = region(input, &mesh);
                let heads = crate::jaw_edge::tools::Heads::load(input, data);
                let frame = Frame::head();
                for name in &targets {
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    let r = load_reference(&dir, name, &mesh, &region);
                    let (v, f, tint) = reference_mesh(&k, &hf, &r, &mesh, false);
                    let (_, live) = heads.pair(input, name);
                    let live = our_shape(&mesh, &live.iter().map(|&q| frame.local(q)).collect::<Vec<_>>());
                    let fit = input.points(&format!("heads/{name}/fit_smooth"));
                    let fit = our_shape(&mesh, &fit.iter().map(|&q| frame.local(q)).collect::<Vec<_>>());
                    let mut tiles = Vec::new();
                    for shot in [SHOTS[0], SHOTS[1]] {
                        for with_hair in [false, true] {
                            let (v, f, tint) = if with_hair { reference_mesh(&k, &hf, &r, &mesh, true) } else { (v.clone(), f.clone(), tint.clone()) };
                            let mut img = clay(&v, &f, &shot, &[], &tint);
                            for (sh, colour) in [(&fit, [80u8, 220, 240]), (&live, [255u8, 240, 60])] {
                                draw_outline(&mut img, &shot, sh, colour);
                            }
                            label(&mut img, &format!("{name} {} {}", shot.name, if with_hair { "hair on" } else { "hair off" }));
                            tiles.push(img);
                        }
                    }
                    let out = format!("{prefix}_{name}.png");
                    sheet(&tiles, 4).save(&out).unwrap();
                    eprintln!("wrote {out}");
                }
                true
            }
            _ => false,
        }
    }

    /// KOTOR's face mesh without hair, all real.
    pub fn kotor_shape(k: &Kotor, hair: &[bool]) -> Shape {
        let tris: Vec<[V3; 3]> = k.f.iter().zip(hair).filter(|(_, h)| !**h).map(|(t, _)| t.map(|i| k.v[i as usize])).collect();
        Shape { filled: vec![false; tris.len()], tris }
    }

    /// KOTOR's face mesh without hair, subdivided twice.
    pub fn subdivided(k: &Kotor, hair: &[bool]) -> (Vec<V3>, Vec<[u32; 3]>) {
        let f: Vec<[u32; 3]> = k.f.iter().zip(hair).filter(|(_, h)| !**h).map(|(t, _)| *t).collect();
        let (v, f, _) = crate::checks::butterfly(&k.v, &f);
        let (v, f, _) = crate::checks::butterfly(&v, &f);
        (v, f)
    }

    /// Our head at `local` (head frame, mm), without the hidden parts (the mouth's inside, the eyeballs).
    pub fn our_shape(mesh: &crate::mesh::HeadMesh, local: &[V3]) -> Shape {
        let tris: Vec<[V3; 3]> = mesh
            .triangles
            .iter()
            .filter(|t| t.iter().all(|&i| !mesh.hidden[mesh.group[i as usize] as usize]))
            .map(|t| t.map(|i| local[i as usize]))
            .collect();
        Shape { filled: vec![false; tris.len()], tris }
    }

    /// The reference as a shape, its filled triangles marked.
    pub fn reference_shape(k: &Kotor, hair: &[bool], r: &Reference, mesh: &crate::mesh::HeadMesh) -> Shape {
        let (v, f, tint) = reference_mesh(k, hair, r, mesh, false);
        Shape { tris: f.iter().map(|t| t.map(|i| v[i as usize])).collect(), filled: tint.iter().map(|t| t.is_some()).collect() }
    }

    /// Draws a shape's outline on a picture: the middle cut from the side, or the widest place at each height from the front.
    pub fn draw_outline(img: &mut image::RgbImage, shot: &Shot, shape: &Shape, colour: [u8; 3]) {
        let px: f64 = std::env::var("SLIDERS_PX").ok().and_then(|v| v.parse().ok()).unwrap_or(2.5);
        let w = shot.window;
        let mut put = |h: f64, y: f64| {
            let (x, y) = (((h - w[0]) * px) as i64, ((w[3] - y) * px) as i64);
            for dy in -1..=0 {
                for dx in -1..=0 {
                    let (i, j) = (x + dx, y + dy);
                    if i >= 0 && j >= 0 && (i as u32) < img.width() && (j as u32) < img.height() {
                        img.put_pixel(i as u32, j as u32, image::Rgb(colour));
                    }
                }
            }
        };
        if shot.yaw == 90.0 {
            for (s, _) in cut(shape, 0, 0.0) {
                let n = (((s[1][0] - s[0][0]).hypot(s[1][1] - s[0][1])) * px).ceil().max(1.0) as usize;
                for k in 0..=n {
                    let f = k as f64 / n as f64;
                    put(s[0][0] + f * (s[1][0] - s[0][0]), s[0][1] + f * (s[1][1] - s[0][1]));
                }
            }
        } else {
            for (y, l, r) in front_outline(shape) {
                for v in [l, r].into_iter().flatten() {
                    for k in 0..4 {
                        put(v, y + k as f64 * 0.25);
                    }
                }
            }
        }
    }

    /// A reference written by `fill`: our merged places (head frame, mm) and which are KOTOR's real skin.
    pub struct Reference {
        pub merged: Vec<V3>,
        pub real: Vec<bool>,
        pub region: Vec<bool>,
    }

    pub fn load_reference(dir: &Path, name: &str, mesh: &crate::mesh::HeadMesh, region: &[bool]) -> Reference {
        let raw = std::fs::read(dir.join(format!("{name}_ref.bin"))).unwrap();
        let merged: Vec<V3> = raw.chunks_exact(24).map(|c| std::array::from_fn(|k| f64::from_le_bytes(c[8 * k..8 * k + 8].try_into().unwrap()))).collect();
        let real: Vec<bool> = std::fs::read(dir.join(format!("{name}_real.bin"))).unwrap().iter().map(|&b| b != 0).collect();
        assert_eq!(merged.len(), mesh.merged_count());
        Reference { merged, real, region: region.to_vec() }
    }

    /// The reference as one mesh: KOTOR's triangles that are not hair (real), then our triangles that touch a filled vertex (filled,
    /// tinted orange), then KOTOR's hair (tinted blue) when asked.
    pub fn reference_mesh(k: &Kotor, hair: &[bool], r: &Reference, mesh: &crate::mesh::HeadMesh, with_hair: bool) -> (Vec<V3>, Vec<[u32; 3]>, Vec<Option<[f64; 3]>>) {
        let mut v = k.v.clone();
        let mut f = Vec::new();
        let mut tint = Vec::new();
        for (t, h) in k.f.iter().zip(hair) {
            if !h {
                f.push(*t);
                tint.push(None);
            }
        }
        let o = v.len() as u32;
        v.extend(r.merged.iter().copied());
        for t in &mesh.triangles {
            let g = t.map(|i| mesh.group[i as usize]);
            let p = g.map(|j| r.merged[j as usize]);
            let n = crate::cross(crate::sub(p[1], p[0]), crate::sub(p[2], p[0]));
            let c = crate::scale(crate::add(crate::add(p[0], p[1]), p[2]), 1.0 / 3.0);
            // the ear area is closed flat, so the back of our ear flap now faces into the head: it is not drawn
            let inward = crate::dot(n, crate::sub(c, [0.0, 0.0, -40.0])) < 0.0;
            // the closed ear area leaves the ear flap's triangles nearly flat, and our eye area is no part of the reference
            let tiny = 0.5 * crate::length(n) < 1.5;
            let at_eye = [29.5, -29.5].iter().any(|&x| crate::length(crate::sub(c, [x, 0.0, 0.0])) < 24.0);
            if g.iter().any(|&j| !r.real[j as usize]) && g.iter().all(|&j| r.region[j as usize]) && !inward && !tiny && !at_eye {
                f.push(g.map(|j| j + o));
                tint.push(Some([235.0, 150.0, 80.0]));
            }
        }
        if with_hair {
            for (t, h) in k.f.iter().zip(hair) {
                if *h {
                    f.push(*t);
                    tint.push(Some([90.0, 140.0, 230.0]));
                }
            }
        }
        (v, f, tint)
    }
}
