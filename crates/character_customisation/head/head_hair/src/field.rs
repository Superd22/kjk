//! The flow field: a direction at every vertex of a coarse grid around the head, solved from a few constraints (Fu et al. 2007).
//!
//! The solve minimises the sum, over every grid vertex, of the squared Laplacian of its vector, plus `OMEGA` squared times the squared
//! distance between each constrained vertex's vector and its target. The Laplacian is the mean of the 6 neighbours minus the vertex. The
//! normal equations are solved by conjugate gradients with a diagonal preconditioner, for the three components at once. A solve can
//! start from a coarser grid's answer, which cuts the number of iterations.

use crate::v3::{V3, add, lerp, scale};
use std::time::Instant;

/// The constraint weight. Fu et al. use 100.
pub const OMEGA: f64 = 100.0;
/// A small pull of every vector toward zero. Without it the field far from every curve is barely set by the energy, and the solve
/// converges slowly there. With it the field fades over about 10 cells away from the curves, as Fu et al.'s stopping rule expects.
pub const DAMPING: f64 = 1e-4;

/// A box of cubic cells. Vertex (i, j, k) lies at `origin + cell * (i, j, k)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub origin: V3,
    pub cell: f64,
    pub dims: [usize; 3],
}

impl Grid {
    /// A grid that covers the box from `lo` to `hi` with `cells` cells along its longest side.
    pub fn covering(lo: V3, hi: V3, cells: usize) -> Self {
        let size = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
        let cell = size.iter().cloned().fold(0.0, f64::max) / cells.max(1) as f64;
        Grid::with_cell(lo, hi, cell)
    }

    pub fn with_cell(lo: V3, hi: V3, cell: f64) -> Self {
        let dims = [0, 1, 2].map(|k| ((hi[k] - lo[k]) / cell).ceil() as usize + 1);
        Grid { origin: lo, cell, dims }
    }

    pub fn len(&self) -> usize {
        self.dims[0] * self.dims[1] * self.dims[2]
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        (k * self.dims[1] + j) * self.dims[0] + i
    }

    pub fn coords(&self, v: usize) -> [usize; 3] {
        let i = v % self.dims[0];
        let j = (v / self.dims[0]) % self.dims[1];
        [i, j, v / (self.dims[0] * self.dims[1])]
    }

    pub fn position(&self, v: usize) -> V3 {
        let c = self.coords(v);
        [0, 1, 2].map(|k| self.origin[k] + self.cell * c[k] as f64)
    }

    /// The vertices within `radius` of a point, with their distances.
    pub fn near(&self, p: V3, radius: f64, mut f: impl FnMut(usize, f64)) {
        let lo = [0, 1, 2].map(|k| (((p[k] - radius - self.origin[k]) / self.cell).ceil().max(0.0)) as usize);
        let hi = [0, 1, 2].map(|k| ((((p[k] + radius - self.origin[k]) / self.cell).floor()).max(-1.0) as i64).min(self.dims[k] as i64 - 1));
        if hi.iter().any(|&h| h < 0) {
            return;
        }
        for k in lo[2]..=hi[2] as usize {
            for j in lo[1]..=hi[1] as usize {
                for i in lo[0]..=hi[0] as usize {
                    let v = self.index(i, j, k);
                    let q = self.position(v);
                    let d = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)).sqrt();
                    if d <= radius {
                        f(v, d);
                    }
                }
            }
        }
    }

    fn neighbours(&self, v: usize, mut f: impl FnMut(usize)) {
        let [i, j, k] = self.coords(v);
        if i > 0 {
            f(v - 1);
        }
        if i + 1 < self.dims[0] {
            f(v + 1);
        }
        let row = self.dims[0];
        if j > 0 {
            f(v - row);
        }
        if j + 1 < self.dims[1] {
            f(v + row);
        }
        let layer = self.dims[0] * self.dims[1];
        if k > 0 {
            f(v - layer);
        }
        if k + 1 < self.dims[2] {
            f(v + layer);
        }
    }
}

/// One soft constraint: the vector at `vertex` should be `target`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constraint {
    pub vertex: usize,
    pub target: V3,
    pub weight: f64,
}

/// A solved field.
#[derive(Clone, Debug)]
pub struct Field {
    pub grid: Grid,
    pub values: Vec<V3>,
}

/// What a solve cost.
#[derive(Clone, Copy, Debug, Default)]
pub struct SolveStats {
    pub iterations: usize,
    /// The final residual divided by the right-hand side's size.
    pub residual: f64,
    pub seconds: f64,
    pub unknowns: usize,
}

impl Field {
    /// The vector at a point, by trilinear interpolation, or `None` outside the grid.
    pub fn at(&self, p: V3) -> Option<V3> {
        let g = &self.grid;
        let mut base = [0usize; 3];
        let mut frac = [0.0; 3];
        for k in 0..3 {
            let x = (p[k] - g.origin[k]) / g.cell;
            if !(0.0..=(g.dims[k] - 1) as f64).contains(&x) {
                return None;
            }
            let b = (x.floor() as usize).min(g.dims[k] - 2);
            base[k] = b;
            frac[k] = x - b as f64;
        }
        let mut out = [0.0; 3];
        for c in 0..8 {
            let (di, dj, dk) = (c & 1, (c >> 1) & 1, (c >> 2) & 1);
            let w = (if di == 1 { frac[0] } else { 1.0 - frac[0] }) * (if dj == 1 { frac[1] } else { 1.0 - frac[1] }) * (if dk == 1 { frac[2] } else { 1.0 - frac[2] });
            if w > 0.0 {
                out = add(out, scale(self.values[g.index(base[0] + di, base[1] + dj, base[2] + dk)], w));
            }
        }
        Some(out)
    }

    /// The field read at every vertex of another grid, as the start of a finer solve. Vertices outside this field's grid get zero.
    pub fn resample(&self, grid: &Grid) -> Vec<V3> {
        (0..grid.len()).map(|v| self.at(grid.position(v)).unwrap_or([0.0; 3])).collect()
    }
}

/// The 6 neighbours of every vertex, with `u32::MAX` where the grid ends, and one over their count.
struct Stencil {
    neighbours: Vec<[u32; 6]>,
    inverse_count: Vec<f64>,
}

impl Stencil {
    fn new(grid: &Grid) -> Self {
        let mut neighbours = vec![[u32::MAX; 6]; grid.len()];
        let mut inverse_count = vec![0.0; grid.len()];
        for v in 0..grid.len() {
            let mut k = 0;
            grid.neighbours(v, |n| {
                neighbours[v][k] = n as u32;
                k += 1;
            });
            inverse_count[v] = 1.0 / k as f64;
        }
        Stencil { neighbours, inverse_count }
    }
}

/// The Laplacian of every vertex: the mean of its neighbours minus itself.
fn laplacian(st: &Stencil, t: &[V3], out: &mut [V3]) {
    for (v, o) in out.iter_mut().enumerate() {
        let mut sum = [0.0; 3];
        for &n in &st.neighbours[v] {
            if n != u32::MAX {
                sum = add(sum, t[n as usize]);
            }
        }
        let inv = st.inverse_count[v];
        *o = [sum[0] * inv - t[v][0], sum[1] * inv - t[v][1], sum[2] * inv - t[v][2]];
    }
}

/// The transpose of `laplacian`. `scaled` is scratch space.
fn laplacian_transposed(st: &Stencil, u: &[V3], scaled: &mut [V3], out: &mut [V3]) {
    for (v, s) in scaled.iter_mut().enumerate() {
        *s = scale(u[v], st.inverse_count[v]);
    }
    for (v, o) in out.iter_mut().enumerate() {
        let mut sum = [0.0; 3];
        for &n in &st.neighbours[v] {
            if n != u32::MAX {
                sum = add(sum, scaled[n as usize]);
            }
        }
        *o = [sum[0] - u[v][0], sum[1] - u[v][1], sum[2] - u[v][2]];
    }
}

fn dot_all(a: &[V3], b: &[V3]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x[0] * y[0] + x[1] * y[1] + x[2] * y[2]).sum()
}

/// Solves the field on a grid. `start` is the first guess, such as a coarser solve resampled; without it the solve starts from zero.
/// It stops when the residual falls below `tolerance` times the right-hand side, or after `max_iterations`.
pub fn solve(grid: &Grid, constraints: &[Constraint], start: Option<Vec<V3>>, tolerance: f64, max_iterations: usize) -> (Field, SolveStats) {
    let started = Instant::now();
    let n = grid.len();
    let stencil = Stencil::new(grid);
    let w2 = OMEGA * OMEGA;
    let mut weight = vec![0.0; n];
    let mut rhs = vec![[0.0; 3]; n];
    for c in constraints {
        weight[c.vertex] += w2 * c.weight;
        rhs[c.vertex] = add(rhs[c.vertex], scale(c.target, w2 * c.weight));
    }
    let mut diagonal = vec![0.0; n];
    for v in 0..n {
        let mut d = 1.0;
        grid.neighbours(v, |m| d += stencil.inverse_count[m] * stencil.inverse_count[m]);
        diagonal[v] = d + weight[v] + DAMPING;
    }
    let mut scratch = vec![[0.0; 3]; 2 * n];
    let apply = |t: &[V3], out: &mut [V3], scratch: &mut [V3]| {
        let (lt, scaled) = scratch.split_at_mut(n);
        laplacian(&stencil, t, lt);
        laplacian_transposed(&stencil, lt, scaled, out);
        for v in 0..n {
            out[v] = add(out[v], scale(t[v], weight[v] + DAMPING));
        }
    };
    let mut x = start.filter(|s| s.len() == n).unwrap_or_else(|| vec![[0.0; 3]; n]);
    let mut ax = vec![[0.0; 3]; n];
    apply(&x, &mut ax, &mut scratch);
    let mut r: Vec<V3> = rhs.iter().zip(&ax).map(|(b, a)| [b[0] - a[0], b[1] - a[1], b[2] - a[2]]).collect();
    let norm_b = dot_all(&rhs, &rhs).sqrt().max(1e-30);
    let mut z: Vec<V3> = r.iter().zip(&diagonal).map(|(r, d)| scale(*r, 1.0 / d)).collect();
    let mut p = z.clone();
    let mut rz = dot_all(&r, &z);
    let mut ap = vec![[0.0; 3]; n];
    let mut stats = SolveStats { unknowns: 3 * n, ..SolveStats::default() };
    for it in 0..max_iterations {
        let res = dot_all(&r, &r).sqrt() / norm_b;
        stats.residual = res;
        stats.iterations = it;
        if res < tolerance {
            break;
        }
        apply(&p, &mut ap, &mut scratch);
        let alpha = rz / dot_all(&p, &ap).max(1e-300);
        for v in 0..n {
            x[v] = add(x[v], scale(p[v], alpha));
            r[v] = add(r[v], scale(ap[v], -alpha));
            z[v] = scale(r[v], 1.0 / diagonal[v]);
        }
        let rz_new = dot_all(&r, &z);
        let beta = rz_new / rz.max(1e-300);
        rz = rz_new;
        for v in 0..n {
            p[v] = add(z[v], scale(p[v], beta));
        }
        stats.iterations = it + 1;
    }
    stats.residual = dot_all(&r, &r).sqrt() / norm_b;
    stats.seconds = started.elapsed().as_secs_f64();
    (Field { grid: *grid, values: x }, stats)
}

/// Solves on grids of 4 and 2 times the cell size first, and starts each finer solve from the coarser answer. `build` makes the
/// constraints for a grid. Returns the field on `fine` and the cost of every level, coarsest first.
pub fn solve_multilevel(fine: &Grid, build: impl Fn(&Grid) -> Vec<Constraint>, tolerance: f64) -> (Field, Vec<SolveStats>) {
    let hi = [0, 1, 2].map(|k| fine.origin[k] + fine.cell * (fine.dims[k] - 1) as f64);
    let mut start: Option<Field> = None;
    let mut all = Vec::new();
    for factor in [4.0, 2.0, 1.0] {
        let grid = if factor == 1.0 { *fine } else { Grid::with_cell(fine.origin, hi, fine.cell * factor) };
        let guess = start.as_ref().map(|f| f.resample(&grid));
        let (field, stats) = solve(&grid, &build(&grid), guess, tolerance, 4000);
        all.push(stats);
        start = Some(field);
    }
    (start.unwrap_or(Field { grid: *fine, values: vec![[0.0; 3]; fine.len()] }), all)
}

/// A polyline with a target magnitude at each point, to be turned into constraints.
#[derive(Clone, Debug)]
pub struct Guide {
    pub points: Vec<V3>,
    pub magnitudes: Vec<f64>,
}

impl Guide {
    /// A stream curve resampled every `spacing`, and extended past its end by `extend` with magnitudes that fall from 1 to 0, so that
    /// hair stops near the curve's end instead of running on.
    pub fn stream(points: &[V3], spacing: f64, extend: f64) -> Self {
        let mut out = resample_polyline(points, spacing);
        let mut magnitudes = vec![1.0; out.len()];
        if out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            let t = crate::v3::normalize(crate::v3::sub(b, a));
            let steps = (extend / spacing).round() as usize;
            for s in 1..=steps {
                out.push(crate::v3::mad(b, t, spacing * s as f64));
                magnitudes.push(1.0 - s as f64 / (steps + 1) as f64);
            }
        }
        Guide { points: out, magnitudes }
    }

    /// Constraints on the vertices within `reach` of the guide: each takes the tangent of its nearest guide point, scaled by that point's
    /// magnitude. The weight falls from 1 at the curve to 0.25 at the reach.
    pub fn constraints(&self, grid: &Grid, reach: f64) -> Vec<Constraint> {
        let mut best: std::collections::HashMap<usize, (f64, V3)> = std::collections::HashMap::new();
        let n = self.points.len();
        if n < 2 {
            return Vec::new();
        }
        for i in 0..n {
            let (a, b) = if i + 1 < n { (self.points[i], self.points[i + 1]) } else { (self.points[i - 1], self.points[i]) };
            let t = scale(crate::v3::normalize(crate::v3::sub(b, a)), self.magnitudes[i]);
            grid.near(self.points[i], reach, |v, d| {
                let e = best.entry(v).or_insert((f64::INFINITY, [0.0; 3]));
                if d < e.0 {
                    *e = (d, t);
                }
            });
        }
        let mut out: Vec<Constraint> = best.into_iter().map(|(vertex, (d, target))| Constraint { vertex, target, weight: 1.0 - 0.75 * d / reach }).collect();
        out.sort_by_key(|c| c.vertex);
        out
    }
}

/// Points along a polyline every `spacing`, from its first point to its last.
pub fn resample_polyline(points: &[V3], spacing: f64) -> Vec<V3> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let mut out = vec![points[0]];
    let mut carried = 0.0;
    for w in points.windows(2) {
        let seg = crate::v3::distance(w[0], w[1]);
        let mut s = spacing - carried;
        while s <= seg {
            out.push(lerp(w[0], w[1], s / seg));
            s += spacing;
        }
        carried = seg - (s - spacing);
    }
    if crate::v3::distance(*out.last().unwrap_or(&points[0]), points[points.len() - 1]) > spacing * 0.25 {
        out.push(points[points.len() - 1]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v3::{dot, length, normalize};

    #[test]
    fn the_field_follows_a_single_straight_constraint() {
        let grid = Grid::covering([0.0; 3], [1.0, 1.0, 1.0], 12);
        let line = [[0.2, 0.5, 0.5], [0.8, 0.5, 0.5]];
        let guide = Guide::stream(&line, grid.cell * 0.25, 0.0);
        let constraints = guide.constraints(&grid, grid.cell);
        assert!(!constraints.is_empty());
        let (field, stats) = solve(&grid, &constraints, None, 1e-6, 5000);
        assert!(stats.residual < 1e-6, "{stats:?}");
        // On the curve the field points along it, at nearly unit length.
        for x in [0.3, 0.5, 0.7] {
            let v = field.at([x, 0.5, 0.5]).expect("inside");
            assert!(dot(normalize(v), [1.0, 0.0, 0.0]) > 0.999, "{v:?}");
            assert!((length(v) - 1.0).abs() < 0.05, "{v:?}");
        }
        // Away from the curve it still points the same way, weaker.
        let v = field.at([0.5, 0.8, 0.3]).expect("inside");
        assert!(dot(normalize(v), [1.0, 0.0, 0.0]) > 0.99, "{v:?}");
    }

    #[test]
    fn a_coarse_start_gives_the_same_answer_with_fewer_iterations() {
        let grid = Grid::covering([0.0; 3], [1.0, 1.0, 1.0], 24);
        let line = [[0.1, 0.3, 0.5], [0.5, 0.6, 0.5], [0.9, 0.4, 0.6]];
        let build = |g: &Grid| Guide::stream(&line, g.cell * 0.25, g.cell * 2.0).constraints(g, g.cell);
        let (plain, one) = solve(&grid, &build(&grid), None, 1e-8, 20000);
        let (multi, levels) = solve_multilevel(&grid, build, 1e-8);
        assert!(levels[2].iterations <= one.iterations, "{levels:?} against {one:?}");
        let worst = plain.values.iter().zip(&multi.values).map(|(a, b)| length(crate::v3::sub(*a, *b))).fold(0.0, f64::max);
        assert!(worst < 0.02, "{worst} {one:?} {levels:?}");
    }

    #[test]
    fn resampling_keeps_both_ends() {
        let r = resample_polyline(&[[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]], 0.3);
        assert_eq!(r[0], [0.0; 3]);
        assert_eq!(*r.last().expect("points"), [1.0, 1.0, 0.0]);
        assert!(r.windows(2).all(|w| crate::v3::distance(w[0], w[1]) <= 0.3 + 1e-9));
    }
}
