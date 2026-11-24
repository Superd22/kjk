//! The weights fit. Each free vertex's weights on the fitted bones are chosen so that the vertex moves as KOTOR's matched point moves
//! over many poses. A smoothness term ties the weights of vertices joined by an edge, so the skin does not crease where the weights
//! change. Each vertex's weights are kept from going negative and are pulled to sum to 1.
//!
//! The data term of a vertex is quadratic in its weights w: the sum over poses of |Y w - t|^2, where column j of Y is how far bone j
//! alone moves the vertex and t is how far KOTOR's point moves. It is kept as its normal equations: A = sum Y^T Y, b = sum Y^T t, and
//! c = sum |t|^2, in millimetres.

use super::data::Head;
use super::kotor::{Matches, kotor_posed, matched_points};
use super::pose::{Pose, apply, skin_matrices};
use crate::V3;
use crate::bones::Bone;
use faer::Side;
use faer::prelude::Solve;
use faer::sparse::{SparseColMat, Triplet};
use std::collections::HashMap;

/// How strongly each vertex's weights are pulled to sum to 1.
const SUM_TO_ONE: f64 = 1e3;

pub struct Problem {
    /// The fitted bones, as numbers of `common.json`'s bones.
    pub bones: Vec<usize>,
    pub free: Vec<usize>,
    /// Each vertex's weights on the fitted bones, kept on the vertices that are not free.
    pub fixed: Vec<Vec<f64>>,
    /// For each free vertex, which fitted bones may carry it.
    pub allowed: Vec<Vec<bool>>,
    /// Pairs of vertices whose weights are tied, with the strength of the tie: the mesh's edges and the copies of one place.
    pub ties: Vec<(usize, usize, f64)>,
}

#[derive(Clone)]
pub struct Equations {
    pub bones: usize,
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    /// The number of poses that gave the vertex a target, 0 when no KOTOR point lies near it.
    pub count: Vec<f64>,
    pub c: Vec<f64>,
}

impl Equations {
    pub fn zeros(vertices: usize, bones: usize) -> Equations {
        Equations {
            bones,
            a: vec![0.0; vertices * bones * bones],
            b: vec![0.0; vertices * bones],
            count: vec![0.0; vertices],
            c: vec![0.0; vertices],
        }
    }

    pub fn add(&mut self, o: &Equations) {
        for (x, y) in [
            (&mut self.a, &o.a),
            (&mut self.b, &o.b),
            (&mut self.count, &o.count),
            (&mut self.c, &o.c),
        ] {
            x.iter_mut().zip(y).for_each(|(p, q)| *p += q);
        }
    }

    /// The root mean square distance, in millimetres, between the free vertices' motion with weights `w` and KOTOR's.
    pub fn error(&self, w: &[Vec<f64>]) -> f64 {
        let nb = self.bones;
        let (mut total, mut n) = (0.0, 0);
        for (i, wi) in w.iter().enumerate() {
            if self.count[i] == 0.0 {
                continue;
            }
            let a = &self.a[i * nb * nb..(i + 1) * nb * nb];
            let mut e = self.c[i];
            for j in 0..nb {
                e -= 2.0 * self.b[i * nb + j] * wi[j];
                for k in 0..nb {
                    e += wi[j] * a[j * nb + k] * wi[k];
                }
            }
            total += e / self.count[i];
            n += 1;
        }
        (total / n.max(1) as f64).sqrt()
    }
}

/// The normal equations of one head's free vertices. A vertex only gets a target where a KOTOR face point lies within `reach`.
pub fn normal_equations(
    head: &Head,
    bones: &[Bone],
    key_scale: [f64; 3],
    poses: &[Pose],
    problem: &Problem,
    matches: &Matches,
    reach: f64,
) -> Equations {
    let nb = problem.bones.len();
    let mut eq = Equations::zeros(problem.free.len(), nb);
    let rest = matched_points(
        &kotor_posed(head, &vec![(None, None); bones.len()]),
        matches,
    );
    let ok: Vec<bool> = problem
        .free
        .iter()
        .map(|&v| matches.distance[v] < reach)
        .collect();
    for pose in poses {
        let mats = skin_matrices(bones, key_scale, pose);
        let target = matched_points(&kotor_posed(head, pose), matches);
        for (i, &v) in problem.free.iter().enumerate() {
            if !ok[i] {
                continue;
            }
            let x = head.pos[v];
            let y: Vec<V3> = problem
                .bones
                .iter()
                .map(|&j| {
                    let p = apply(&mats[j], x);
                    std::array::from_fn(|d| (p[d] - x[d]) * 1000.0)
                })
                .collect();
            let t: V3 = std::array::from_fn(|d| (target[v][d] - rest[v][d]) * 1000.0);
            for j in 0..nb {
                for k in 0..nb {
                    eq.a[(i * nb + j) * nb + k] += (0..3).map(|d| y[j][d] * y[k][d]).sum::<f64>();
                }
                eq.b[i * nb + j] += (0..3).map(|d| y[j][d] * t[d]).sum::<f64>();
            }
            eq.c[i] += (0..3).map(|d| t[d] * t[d]).sum::<f64>();
            eq.count[i] += 1.0;
        }
    }
    eq
}

/// A pull of each vertex's weights towards given weights, with a strength per vertex.
pub struct Prior<'a> {
    pub strength: &'a [f64],
    pub weights: &'a [Vec<f64>],
}

/// Solves a sparse symmetric positive definite system, given its lower triangle.
pub fn solve_spd(n: usize, lower: &HashMap<(usize, usize), f64>, rhs: &[f64]) -> Vec<f64> {
    let triplets: Vec<Triplet<usize, usize, f64>> = lower
        .iter()
        .map(|(&(r, c), &v)| Triplet::new(r, c, v))
        .collect();
    let m = SparseColMat::<usize, f64>::try_new_from_triplets(n, n, &triplets).expect("matrix");
    let b = faer::Mat::<f64>::from_fn(n, 1, |i, _| rhs[i]);
    let llt = m
        .sp_cholesky(Side::Lower)
        .expect("a positive definite system");
    let x = llt.solve(&b);
    (0..n).map(|i| x[(i, 0)]).collect()
}

/// The weights of every vertex on the fitted bones: the fit on the free vertices, and the fixed weights elsewhere.
pub fn solve(eq: &Equations, problem: &Problem, prior: Option<&Prior>) -> Vec<Vec<f64>> {
    let nb = problem.bones.len();
    let n = problem.fixed.len();
    let mut slot = vec![usize::MAX; n];
    for (i, &v) in problem.free.iter().enumerate() {
        slot[v] = i;
    }
    let mut allowed = problem.allowed.clone();
    let mut w = vec![vec![0.0; nb]; problem.free.len()];
    for _ in 0..8 {
        let mut var = vec![usize::MAX; problem.free.len() * nb];
        let mut count = 0;
        for (i, row) in allowed.iter().enumerate() {
            for j in 0..nb {
                if row[j] {
                    var[i * nb + j] = count;
                    count += 1;
                }
            }
        }
        let mut lower: HashMap<(usize, usize), f64> = HashMap::new();
        let mut put = |r: usize, c: usize, x: f64| {
            if r != usize::MAX && c != usize::MAX {
                *lower.entry((r.max(c), r.min(c))).or_default() += x;
            }
        };
        let mut rhs = vec![0.0; count];
        for (i, &v) in problem.free.iter().enumerate() {
            let s = 1.0 / eq.count[i].max(1.0);
            let mu = prior.map_or(0.0, |p| p.strength[v]);
            for j in 0..nb {
                let r = var[i * nb + j];
                if r == usize::MAX {
                    continue;
                }
                for k in 0..=j {
                    let mut x = s * eq.a[(i * nb + j) * nb + k] + SUM_TO_ONE;
                    if j == k {
                        x += mu;
                    }
                    put(r, var[i * nb + k], x);
                }
                rhs[r] +=
                    s * eq.b[i * nb + j] + SUM_TO_ONE + prior.map_or(0.0, |p| mu * p.weights[v][j]);
            }
        }
        for &(a, b, t) in &problem.ties {
            for (p, q) in [(a, b), (b, a)] {
                let i = slot[p];
                if i == usize::MAX {
                    continue;
                }
                for j in 0..nb {
                    let r = var[i * nb + j];
                    if r == usize::MAX {
                        continue;
                    }
                    put(r, r, t);
                    match slot[q] {
                        usize::MAX => rhs[r] += t * problem.fixed[q][j],
                        o if p < q => put(r, var[o * nb + j], -t),
                        _ => {}
                    }
                }
            }
        }
        let x = solve_spd(count, &lower, &rhs);
        let mut negative = false;
        for (i, row) in w.iter_mut().enumerate() {
            for j in 0..nb {
                let r = var[i * nb + j];
                row[j] = if r == usize::MAX { 0.0 } else { x[r] };
                if r != usize::MAX && row[j] < -1e-4 {
                    allowed[i][j] = false;
                    negative = true;
                }
            }
        }
        if !negative {
            break;
        }
    }
    let mut out = problem.fixed.clone();
    for (i, &v) in problem.free.iter().enumerate() {
        let s: f64 = w[i].iter().map(|x| x.max(0.0)).sum();
        out[v] = w[i].iter().map(|x| x.max(0.0) / s.max(1e-9)).collect();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three vertices in a row with two bones. The middle one is free, and its data asks for bone 0 alone.
    fn problem() -> Problem {
        Problem {
            bones: vec![0, 1],
            free: vec![1],
            fixed: vec![vec![1.0, 0.0], vec![0.5, 0.5], vec![0.0, 1.0]],
            allowed: vec![vec![true, true]],
            ties: vec![(0, 1, 1.0), (1, 2, 1.0)],
        }
    }

    fn data(scale: f64) -> Equations {
        let mut eq = Equations::zeros(1, 2);
        eq.a = vec![scale, 0.0, 0.0, scale];
        eq.b = vec![scale, 0.0];
        eq.count = vec![1.0];
        eq.c = vec![scale];
        eq
    }

    #[test]
    fn strong_data_wins_over_smoothness() {
        let w = solve(&data(1e6), &problem(), None);
        assert!(w[1][0] > 0.99, "{:?}", w[1]);
    }

    #[test]
    fn without_data_the_vertex_takes_the_middle_of_its_neighbours() {
        let w = solve(&data(0.0), &problem(), None);
        assert!((w[1][0] - 0.5).abs() < 1e-6, "{:?}", w[1]);
    }

    #[test]
    fn weights_never_go_negative_and_sum_to_one() {
        let mut eq = data(1e6);
        eq.b = vec![2e6, -1e6];
        let w = solve(&eq, &problem(), None);
        assert!(w[1].iter().all(|x| *x >= 0.0));
        assert!((w[1].iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_error_is_zero_for_the_weights_the_data_asks_for() {
        assert!(data(4.0).error(&[vec![1.0, 0.0]]).abs() < 1e-9);
    }
}
