//! Smoothing of a head's final vertex positions.
//!
//! The head's vertices are first merged where the texture layout split them,
//! so smoothing never tears a seam. Each merged vertex gets a weight between
//! 0 and 1. A vertex marked to keep has weight 0 and does not move. The weight
//! rises smoothly to 1 at `ramp` metres from the nearest kept vertex, so the
//! smoothing fades in around the kept parts instead of leaving a step.
//!
//! Two methods are offered.
//! - Taubin's method alternates a shrinking and an inflating step, so it
//!   removes fine ripples without shrinking the head as plain averaging does.
//! - The implicit method finds the surface that stays closest to the input
//!   while bending least, by solving
//!   `(diag(a) + strength * LᵀL) x = diag(a) p`, where `L` is the mesh's
//!   Laplacian and `a = 1 / weight`. It removes detail finer than a length that
//!   grows with the strength, and leaves broad shapes alone.

use std::collections::HashMap;

/// How the positions are smoothed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Method {
    /// `rounds` rounds of Taubin's method, with the usual factors 0.5 and -0.53.
    Taubin { rounds: u32 },
    /// The implicit method with this strength.
    Implicit { strength: f64 },
}

/// A head mesh made ready for smoothing: merged vertices, their neighbours and their weights.
pub struct Smoother {
    group: Vec<u32>,
    first: Vec<u32>,
    offsets: Vec<u32>,
    neighbours: Vec<u32>,
    weight: Vec<f64>,
}

const TAUBIN_LAMBDA: f64 = 0.5;
const TAUBIN_MU: f64 = -0.53;
// Merged vertices are those within the same 0.1 mm cell, as the Python tools merge them.
const WELD_CELL: f64 = 1e-4;
const MIN_WEIGHT: f64 = 1e-4;

impl Smoother {
    /// Prepares a mesh. `base` is the base head's positions in metres, `triangles` its triangles, `keep` marks the vertices that must
    /// not move, and `ramp` is the distance in metres over which the smoothing fades in around them. A merged vertex is kept when any of
    /// its copies is.
    pub fn new(base: &[[f64; 3]], triangles: &[[u32; 3]], keep: &[bool], ramp: f64) -> Self {
        let mut cells: HashMap<[i64; 3], u32> = HashMap::new();
        let mut group = Vec::with_capacity(base.len());
        let mut first = Vec::new();
        for (i, p) in base.iter().enumerate() {
            let key = p.map(|c| (c / WELD_CELL).round() as i64);
            let g = *cells.entry(key).or_insert_with(|| {
                first.push(i as u32);
                (first.len() - 1) as u32
            });
            group.push(g);
        }
        let n = first.len();
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for t in triangles {
            let g = t.map(|v| group[v as usize]);
            if g[0] == g[1] || g[1] == g[2] || g[0] == g[2] {
                continue;
            }
            for (a, b) in [(g[0], g[1]), (g[1], g[2]), (g[2], g[0])] {
                edges.push((a, b));
                edges.push((b, a));
            }
        }
        edges.sort_unstable();
        edges.dedup();
        let mut offsets = vec![0u32; n + 1];
        for &(a, _) in &edges {
            offsets[a as usize + 1] += 1;
        }
        for i in 0..n {
            offsets[i + 1] += offsets[i];
        }
        let neighbours = edges.iter().map(|&(_, b)| b).collect();

        let mut kept = vec![false; n];
        for (i, &k) in keep.iter().enumerate() {
            if k {
                kept[group[i] as usize] = true;
            }
        }
        let anchors: Vec<[f64; 3]> = (0..n).filter(|&g| kept[g]).map(|g| base[first[g] as usize]).collect();
        let weight = (0..n)
            .map(|g| {
                if kept[g] {
                    return 0.0;
                }
                if ramp <= 0.0 || anchors.is_empty() {
                    return 1.0;
                }
                let p = base[first[g] as usize];
                let d2 = anchors.iter().map(|a| dist2(*a, p)).fold(f64::INFINITY, f64::min);
                let t = (d2.sqrt() / ramp).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            })
            .collect();
        Self { group, first, offsets, neighbours, weight }
    }

    /// The number of merged vertices.
    pub fn merged_count(&self) -> usize {
        self.first.len()
    }

    /// Each merged vertex's weight, from 0 (kept) to 1 (smoothed fully).
    pub fn weights(&self) -> &[f64] {
        &self.weight
    }

    /// For each vertex of the mesh, its merged vertex.
    pub fn groups(&self) -> &[u32] {
        &self.group
    }

    /// Smooths a head's positions, which must have the base head's vertex count and order. Each copy of a merged vertex moves by the
    /// same amount, so seams stay closed.
    pub fn smooth(&self, positions: &[[f64; 3]], method: Method) -> Vec<[f64; 3]> {
        let p: Vec<[f64; 3]> = self.first.iter().map(|&i| positions[i as usize]).collect();
        let q = match method {
            Method::Taubin { rounds } => self.taubin(&p, rounds),
            Method::Implicit { strength } => self.implicit(&p, strength),
        };
        positions
            .iter()
            .zip(&self.group)
            .map(|(x, &g)| {
                let g = g as usize;
                std::array::from_fn(|c| x[c] + q[g][c] - p[g][c])
            })
            .collect()
    }

    fn neighbours_of(&self, g: usize) -> &[u32] {
        &self.neighbours[self.offsets[g] as usize..self.offsets[g + 1] as usize]
    }

    fn mean_of_neighbours(&self, x: &[[f64; 3]], g: usize) -> [f64; 3] {
        let nb = self.neighbours_of(g);
        if nb.is_empty() {
            return x[g];
        }
        let mut s = [0.0; 3];
        for &j in nb {
            for c in 0..3 {
                s[c] += x[j as usize][c];
            }
        }
        s.map(|v| v / nb.len() as f64)
    }

    fn taubin(&self, p: &[[f64; 3]], rounds: u32) -> Vec<[f64; 3]> {
        let mut x = p.to_vec();
        for _ in 0..rounds {
            for f in [TAUBIN_LAMBDA, TAUBIN_MU] {
                x = (0..x.len())
                    .map(|g| {
                        let m = self.mean_of_neighbours(&x, g);
                        let w = f * self.weight[g];
                        std::array::from_fn(|c| x[g][c] + w * (m[c] - x[g][c]))
                    })
                    .collect();
            }
        }
        x
    }

    /// `L x`, with `L = I - D⁻¹A`.
    fn laplacian(&self, x: &[f64]) -> Vec<f64> {
        (0..x.len())
            .map(|g| {
                let nb = self.neighbours_of(g);
                if nb.is_empty() {
                    return 0.0;
                }
                x[g] - nb.iter().map(|&j| x[j as usize]).sum::<f64>() / nb.len() as f64
            })
            .collect()
    }

    /// `Lᵀ y = y - A D⁻¹ y`, because the adjacency is symmetric.
    fn laplacian_t(&self, y: &[f64]) -> Vec<f64> {
        (0..y.len())
            .map(|g| {
                y[g] - self
                    .neighbours_of(g)
                    .iter()
                    .map(|&j| {
                        let d = self.neighbours_of(j as usize).len() as f64;
                        y[j as usize] / d
                    })
                    .sum::<f64>()
            })
            .collect()
    }

    fn implicit(&self, p: &[[f64; 3]], strength: f64) -> Vec<[f64; 3]> {
        let n = p.len();
        let alpha: Vec<f64> = self.weight.iter().map(|w| 1.0 / w.max(MIN_WEIGHT)).collect();
        // The diagonal of LᵀL, for a Jacobi preconditioner.
        let diag: Vec<f64> = (0..n)
            .map(|g| {
                let own = if self.neighbours_of(g).is_empty() { 0.0 } else { 1.0 };
                let others: f64 = self
                    .neighbours_of(g)
                    .iter()
                    .map(|&j| {
                        let d = self.neighbours_of(j as usize).len() as f64;
                        1.0 / (d * d)
                    })
                    .sum();
                alpha[g] + strength * (own + others)
            })
            .collect();
        let free: Vec<bool> = self.weight.iter().map(|&w| w > 0.0).collect();
        // Kept vertices are fixed values, so only the free ones are solved for.
        let apply = |x: &[f64]| -> Vec<f64> {
            let x: Vec<f64> = (0..n).map(|g| if free[g] { x[g] } else { 0.0 }).collect();
            let lx = self.laplacian(&x);
            let ltlx = self.laplacian_t(&lx);
            (0..n).map(|g| if free[g] { alpha[g] * x[g] + strength * ltlx[g] } else { 0.0 }).collect()
        };
        let mut out = p.to_vec();
        for c in 0..3 {
            let fixed: Vec<f64> = (0..n).map(|g| if free[g] { 0.0 } else { p[g][c] }).collect();
            let lf = self.laplacian(&fixed);
            let ltlf = self.laplacian_t(&lf);
            let b: Vec<f64> = (0..n).map(|g| if free[g] { alpha[g] * p[g][c] - strength * ltlf[g] } else { 0.0 }).collect();
            let x0: Vec<f64> = (0..n).map(|g| if free[g] { p[g][c] } else { 0.0 }).collect();
            let x = conjugate_gradient(&apply, &b, x0, &diag, 1e-12, 4000);
            for g in 0..n {
                out[g][c] = if free[g] { x[g] } else { p[g][c] };
            }
        }
        out
    }
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|c| (a[c] - b[c]).powi(2)).sum()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Solves `A x = b` for a symmetric positive definite `A`, given as a function, with a diagonal preconditioner. It stops when the
/// residual's norm falls below `tolerance` times the norm of `b`.
fn conjugate_gradient(apply: &dyn Fn(&[f64]) -> Vec<f64>, b: &[f64], mut x: Vec<f64>, diag: &[f64], tolerance: f64, max_rounds: usize) -> Vec<f64> {
    let ax = apply(&x);
    let mut r: Vec<f64> = b.iter().zip(&ax).map(|(b, a)| b - a).collect();
    let mut z: Vec<f64> = r.iter().zip(diag).map(|(r, d)| r / d).collect();
    let mut d = z.clone();
    let mut rz = dot(&r, &z);
    let stop = tolerance * dot(b, b).sqrt();
    for _ in 0..max_rounds {
        if dot(&r, &r).sqrt() <= stop {
            break;
        }
        let ad = apply(&d);
        let step = rz / dot(&d, &ad);
        for i in 0..x.len() {
            x[i] += step * d[i];
            r[i] -= step * ad[i];
        }
        z = r.iter().zip(diag).map(|(r, d)| r / d).collect();
        let rz_new = dot(&r, &z);
        let beta = rz_new / rz;
        rz = rz_new;
        for i in 0..d.len() {
            d[i] = z[i] + beta * d[i];
        }
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square grid of `n` by `n` vertices 2.5 mm apart, as two triangles per cell.
    fn grid(n: u32) -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
        let h = 0.0025;
        let v = (0..n * n).map(|i| [(i % n) as f64 * h, (i / n) as f64 * h, 0.0]).collect();
        let mut t = Vec::new();
        for y in 0..n - 1 {
            for x in 0..n - 1 {
                let a = y * n + x;
                t.push([a, a + 1, a + n]);
                t.push([a + 1, a + n + 1, a + n]);
            }
        }
        (v, t)
    }

    fn rippled(v: &[[f64; 3]]) -> Vec<[f64; 3]> {
        v.iter().map(|p| [p[0], p[1], 0.0003 * ((p[0] / 0.0025) as i64 % 2) as f64]).collect()
    }

    fn rms_z(v: &[[f64; 3]], inner: &dyn Fn(usize) -> bool) -> f64 {
        let z: Vec<f64> = v.iter().enumerate().filter(|(i, _)| inner(*i)).map(|(_, p)| p[2]).collect();
        let m = z.iter().sum::<f64>() / z.len() as f64;
        (z.iter().map(|x| (x - m).powi(2)).sum::<f64>() / z.len() as f64).sqrt()
    }

    #[test]
    fn both_methods_remove_a_fine_ripple() {
        let (v, t) = grid(30);
        let s = Smoother::new(&v, &t, &vec![false; v.len()], 0.0);
        let p = rippled(&v);
        let inner = |i: usize| (5..25).contains(&(i % 30)) && (5..25).contains(&(i / 30));
        let before = rms_z(&p, &inner);
        for m in [Method::Taubin { rounds: 5 }, Method::Implicit { strength: 3.0 }] {
            let after = rms_z(&s.smooth(&p, m), &inner);
            assert!(after < before * 0.2, "{m:?}: {before} to {after}");
        }
    }

    #[test]
    fn kept_vertices_do_not_move() {
        let (v, t) = grid(20);
        let keep: Vec<bool> = (0..v.len()).map(|i| i % 20 < 4).collect();
        let s = Smoother::new(&v, &t, &keep, 0.005);
        let p = rippled(&v);
        for m in [Method::Taubin { rounds: 5 }, Method::Implicit { strength: 3.0 }] {
            let q = s.smooth(&p, m);
            for i in (0..v.len()).filter(|&i| keep[i]) {
                assert!(dist2(p[i], q[i]).sqrt() < 1e-7, "{m:?} moved kept vertex {i}");
            }
        }
    }

    #[test]
    fn split_copies_move_together() {
        let (mut v, mut t) = grid(10);
        let copy = v[55];
        v.push(copy);
        t[100] = t[100].map(|x| if x == 55 { 100 } else { x });
        let s = Smoother::new(&v, &t, &vec![false; v.len()], 0.0);
        assert_eq!(s.merged_count(), 100);
        let q = s.smooth(&rippled(&v), Method::Implicit { strength: 3.0 });
        assert!(dist2(q[55], q[100]) < 1e-20);
    }

    #[test]
    fn a_flat_grid_stays_flat() {
        let (v, t) = grid(12);
        let s = Smoother::new(&v, &t, &vec![false; v.len()], 0.0);
        for m in [Method::Taubin { rounds: 10 }, Method::Implicit { strength: 10.0 }] {
            let q = s.smooth(&v, m);
            assert!(q.iter().all(|p| p[2].abs() < 1e-12));
        }
    }
}
