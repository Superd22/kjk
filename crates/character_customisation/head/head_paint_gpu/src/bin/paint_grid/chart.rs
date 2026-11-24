//! The chart of the paint grid: a map of the skin where hair can grow onto the plane, whose columns and rows are the grid's.
//!
//! The x < 0 half of the mean head is laid flat as rigidly as it can be, so that a step is about 2 mm everywhere, and the x > 0 half is
//! its mirror image. The middle line is the half's edge. Up the face it is the straight line of column 0, so that the two halves meet
//! there without a bend. Over the top of the head and down the back it runs along a few more straight pieces, each turned by the same
//! angle from the last, so that it reaches the back of the neck upright. Where two pieces meet the grid has a cone, a point round which
//! its lines turn a little, instead of the single point where all the columns met before. A line that meets one of the pieces past the
//! face carries on, on the other half, as the line its mirror image meets, so the grid's lines run on smoothly across the middle line
//! everywhere. Holes where hair never grows, such as the mouth, the eyes and the ears, are covered by flat patches for the solve, so the
//! lines run on straight across them.

use head_paint::layout_points::{cross, dot, normalize, sub};
use std::collections::{BTreeMap, HashMap};

/// A triangle mesh in millimetres. `parent` is the layout triangle a triangle lies in, or `None` for the patches laid over holes, and
/// `hole_mm` is the length of the outline of the hole a patch covers.
#[derive(Clone, Default)]
pub struct Mesh {
    pub p: Vec<[f64; 3]>,
    pub t: Vec<[usize; 3]>,
    pub parent: Vec<Option<usize>>,
    pub hole_mm: Vec<f64>,
}

impl Mesh {
    pub fn boundary_loops(&self) -> Vec<Vec<usize>> {
        let mut directed: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for t in &self.t {
            for e in 0..3 {
                *directed.entry((t[e], t[(e + 1) % 3])).or_default() += 1;
            }
        }
        let mut next: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for &(a, b) in directed.keys() {
            if !directed.contains_key(&(b, a)) {
                next.entry(a).or_default().push(b);
            }
        }
        let mut used: BTreeMap<(usize, usize), bool> = BTreeMap::new();
        let mut loops = Vec::new();
        let mut starts: Vec<usize> = next.keys().copied().collect();
        starts.sort();
        for s in starts {
            for &n0 in &next[&s].clone() {
                if used.contains_key(&(s, n0)) {
                    continue;
                }
                let mut lp = vec![s];
                let (mut a, mut b) = (s, n0);
                loop {
                    used.insert((a, b), true);
                    if b == s {
                        break;
                    }
                    lp.push(b);
                    let Some(nb) = next.get(&b).and_then(|v| v.iter().copied().find(|&c| !used.contains_key(&(b, c)))) else { break };
                    a = b;
                    b = nb;
                }
                loops.push(lp);
            }
        }
        loops
    }

    /// Splits every triangle in four through the middles of its edges, which keeps the surface's shape.
    pub fn subdivide(&self) -> Mesh {
        let mut out = Mesh { p: self.p.clone(), ..Mesh::default() };
        let mut mid: HashMap<(usize, usize), usize> = HashMap::new();
        let mut m = |a: usize, b: usize, out: &mut Mesh| -> usize {
            let k = (a.min(b), a.max(b));
            *mid.entry(k).or_insert_with(|| {
                out.p.push([0, 1, 2].map(|c| 0.5 * (out.p[a][c] + out.p[b][c])));
                out.p.len() - 1
            })
        };
        for (ti, t) in self.t.iter().enumerate() {
            let [a, b, c] = *t;
            let ab = m(a, b, &mut out);
            let bc = m(b, c, &mut out);
            let ca = m(c, a, &mut out);
            for q in [[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]] {
                out.t.push(q);
                out.parent.push(self.parent[ti]);
                out.hole_mm.push(self.hole_mm[ti]);
            }
        }
        out
    }
}

/// Cuts the mesh along the plane x = 0. Returns the mesh, whose triangles each lie on one side, and each triangle's side (0 for x < 0).
pub fn slice(mesh: &Mesh) -> (Mesh, Vec<usize>) {
    let mut out = Mesh { p: mesh.p.clone(), ..Mesh::default() };
    for p in &mut out.p {
        if p[0].abs() < 0.05 {
            p[0] = 0.0;
        }
    }
    let mut side = Vec::new();
    let mut cut: HashMap<(usize, usize), usize> = HashMap::new();
    let sign = |x: f64| if x < 0.0 { -1 } else if x > 0.0 { 1 } else { 0 };
    for (ti, t) in mesh.t.iter().enumerate() {
        let s = t.map(|v| sign(out.p[v][0]));
        if s.iter().all(|&x| x == 0) {
            continue;
        }
        if s.iter().all(|&x| x <= 0) || s.iter().all(|&x| x >= 0) {
            out.t.push(*t);
            out.parent.push(mesh.parent[ti]);
            out.hole_mm.push(mesh.hole_mm[ti]);
            side.push(usize::from(s.iter().any(|&x| x > 0)));
            continue;
        }
        // The triangle's outline, walked in order, with a new corner where an edge crosses the plane.
        let mut ring: Vec<usize> = Vec::new();
        for e in 0..3 {
            let (a, b) = (t[e], t[(e + 1) % 3]);
            ring.push(a);
            let (sa, sb) = (sign(out.p[a][0]), sign(out.p[b][0]));
            if sa * sb < 0 {
                let k = (a.min(b), a.max(b));
                let v = *cut.entry(k).or_insert_with(|| {
                    let (pa, pb) = (out.p[a], out.p[b]);
                    let f = pa[0] / (pa[0] - pb[0]);
                    out.p.push([0.0, pa[1] + f * (pb[1] - pa[1]), pa[2] + f * (pb[2] - pa[2])]);
                    out.p.len() - 1
                });
                ring.push(v);
            }
        }
        for want in [-1, 1] {
            let poly: Vec<usize> = ring.iter().copied().filter(|&v| { let x = sign(out.p[v][0]); x == want || x == 0 }).collect();
            for k in 1..poly.len().saturating_sub(1) {
                out.t.push([poly[0], poly[k], poly[k + 1]]);
                out.parent.push(mesh.parent[ti]);
                out.hole_mm.push(mesh.hole_mm[ti]);
            out.hole_mm.push(mesh.hole_mm[ti]);
                side.push(usize::from(want > 0));
            }
        }
    }
    (out, side)
}

/// How one coordinate is set: by an unknown, or by a constant.
#[derive(Clone, Copy)]
enum Coord {
    Free(usize),
    Fixed(f64),
}

/// A triangle's gradient basis in its own frame: the gradient of each corner's hat function, and the triangle's area.
fn gradients(p: &[[f64; 3]], t: [usize; 3]) -> Option<([[f64; 2]; 3], f64)> {
    let v = t.map(|i| p[i]);
    let n = cross(sub(v[1], v[0]), sub(v[2], v[0]));
    let area2 = dot(n, n).sqrt();
    if area2 < 1e-12 {
        return None;
    }
    let n = n.map(|x| x / area2);
    let e1 = normalize(sub(v[1], v[0]));
    let e2 = cross(n, e1);
    let q = v.map(|x| [dot(sub(x, v[0]), e1), dot(sub(x, v[0]), e2)]);
    let g = [0, 1, 2].map(|k| {
        let e = [q[(k + 2) % 3][0] - q[(k + 1) % 3][0], q[(k + 2) % 3][1] - q[(k + 1) % 3][1]];
        [-e[1] / area2, e[0] / area2]
    });
    Some((g, 0.5 * area2))
}

type Row = (Vec<(usize, f64)>, f64);

fn push_term(row: &mut Row, c: Coord, coeff: f64) {
    match c {
        Coord::Free(i) => row.0.push((i, coeff)),
        Coord::Fixed(x) => row.1 += coeff * x,
    }
}

/// The rows of the conformal map: per triangle, `grad v - J grad u`, weighted by the square root of its area.
fn conformal_rows(p: &[[f64; 3]], tris: &[[usize; 3]], coords: &[[Coord; 2]]) -> Vec<Row> {
    let mut rows = Vec::new();
    for &t in tris {
        let Some((g, area)) = gradients(p, t) else { continue };
        let w = area.sqrt();
        for axis in 0..2 {
            let mut row: Row = (Vec::new(), 0.0);
            for k in 0..3 {
                let jg = [-g[k][1], g[k][0]];
                push_term(&mut row, coords[t[k]][1], w * g[k][axis]);
                push_term(&mut row, coords[t[k]][0], -w * jg[axis]);
            }
            rows.push(row);
        }
    }
    rows
}

/// The rows of the rigid step: per triangle, the gradients of u and v equal the rows of the rotation `rot[t]` (its cosine and sine).
fn rigid_rows(p: &[[f64; 3]], tris: &[[usize; 3]], coords: &[[Coord; 2]], rot: &[[f64; 2]], weight: &[f64]) -> Vec<Row> {
    let mut rows = Vec::new();
    for (ti, &t) in tris.iter().enumerate() {
        let Some((g, area)) = gradients(p, t) else { continue };
        let w = (area * weight[ti]).sqrt();
        let [c, s] = rot[ti];
        let target = [[c, -s], [s, c]];
        for which in 0..2 {
            for axis in 0..2 {
                let mut row: Row = (Vec::new(), -w * target[which][axis]);
                for k in 0..3 {
                    push_term(&mut row, coords[t[k]][which], w * g[k][axis]);
                }
                rows.push(row);
            }
        }
    }
    rows
}

/// Each triangle's nearest rotation to its present map, as a cosine and a sine.
fn rotations(p: &[[f64; 3]], tris: &[[usize; 3]], coords: &[[Coord; 2]], x: &[f64]) -> Vec<[f64; 2]> {
    tris.iter()
        .map(|&t| {
            let Some((g, _)) = gradients(p, t) else { return [1.0, 0.0] };
            let mut j = [[0.0; 2]; 2];
            for k in 0..3 {
                for which in 0..2 {
                    let val = value(coords[t[k]][which], x);
                    for axis in 0..2 {
                        j[which][axis] += val * g[k][axis];
                    }
                }
            }
            let th = (j[1][0] - j[0][1]).atan2(j[0][0] + j[1][1]);
            [th.cos(), th.sin()]
        })
        .collect()
}

fn value(c: Coord, x: &[f64]) -> f64 {
    match c {
        Coord::Free(i) => x[i],
        Coord::Fixed(f) => f,
    }
}

/// Least squares over the rows, by conjugate gradients on the normal equations, from `x`.
fn least_squares(rows: &[Row], unknowns: usize, mut x: Vec<f64>) -> Vec<f64> {
    let apply = |x: &[f64]| -> Vec<f64> {
        let mut y = vec![0.0; unknowns];
        for (row, _) in rows {
            let r: f64 = row.iter().map(|&(i, a)| a * x[i]).sum();
            for &(i, a) in row {
                y[i] += a * r;
            }
        }
        y
    };
    let mut b = vec![0.0; unknowns];
    let mut diag = vec![0.0; unknowns];
    for (row, c) in rows {
        for &(i, a) in row {
            b[i] -= a * c;
            diag[i] += a * a;
        }
    }
    let inv: Vec<f64> = diag.iter().map(|&d| if d > 0.0 { 1.0 / d } else { 0.0 }).collect();
    let ax = apply(&x);
    let mut r: Vec<f64> = (0..unknowns).map(|i| b[i] - ax[i]).collect();
    let mut z: Vec<f64> = r.iter().zip(&inv).map(|(a, b)| a * b).collect();
    let mut d = z.clone();
    let mut rz: f64 = r.iter().zip(&z).map(|(a, b)| a * b).sum();
    let b_norm = b.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-30);
    for _ in 0..40000 {
        if r.iter().map(|x| x * x).sum::<f64>().sqrt() / b_norm < 1e-11 {
            break;
        }
        let ad = apply(&d);
        let dad: f64 = d.iter().zip(&ad).map(|(a, b)| a * b).sum();
        if dad <= 0.0 {
            break;
        }
        let alpha = rz / dad;
        for i in 0..unknowns {
            x[i] += alpha * d[i];
            r[i] -= alpha * ad[i];
        }
        for i in 0..unknowns {
            z[i] = r[i] * inv[i];
        }
        let rz2: f64 = r.iter().zip(&z).map(|(a, b)| a * b).sum();
        let beta = rz2 / rz;
        rz = rz2;
        for i in 0..unknowns {
            d[i] = z[i] + beta * d[i];
        }
    }
    x
}

/// A straight piece of the middle line in the x < 0 half's chart: the points q with `normal · q = offset`. The half lies on the side
/// where `normal · q > offset`.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub normal: [f64; 2],
    pub offset: f64,
    pub ends: [[f64; 2]; 2],
}

impl Segment {
    /// How far a point lies inside the half from the piece's line, in steps. Past the line it is below zero.
    pub fn inside(&self, q: [f64; 2]) -> f64 {
        self.normal[0] * q[0] + self.normal[1] * q[1] - self.offset
    }

    pub fn reflect(&self, q: [f64; 2]) -> [f64; 2] {
        let d = self.inside(q);
        [q[0] - 2.0 * d * self.normal[0], q[1] - 2.0 * d * self.normal[1]]
    }

    /// Whether the nearest point of the piece's line to `q` lies between the piece's ends.
    pub fn beside(&self, q: [f64; 2]) -> bool {
        let d = [self.ends[1][0] - self.ends[0][0], self.ends[1][1] - self.ends[0][1]];
        let f = ((q[0] - self.ends[0][0]) * d[0] + (q[1] - self.ends[0][1]) * d[1]) / (d[0] * d[0] + d[1] * d[1]).max(1e-12);
        (-0.01..=1.01).contains(&f)
    }
}

pub struct Chart {
    pub mesh: Mesh,
    /// Each triangle's half: 0 for x < 0, 1 for x > 0.
    pub side: Vec<usize>,
    /// Each vertex's column and row in each half, or NaN where the vertex is not in that half.
    pub uv: Vec<[[f64; 2]; 2]>,
    /// The middle line's pieces in the x < 0 half, from the face's to the back's.
    pub segments: Vec<Segment>,
}

pub struct Settings<'a> {
    /// The cones, as angles round the ears' axis from the front, upward.
    pub cones_deg: &'a [f64],
    pub axis: [f64; 3],
    /// Row 0 passes through the chin.
    pub chin: [f64; 3],
    pub subdivisions: usize,
    pub rigid_steps: usize,
    /// How much a triangle holds its shape, from its middle and its normal.
    pub weight: &'a dyn Fn([f64; 3], [f64; 3]) -> f64,
    /// The part of the face where a step is made exactly `spacing_mm`.
    pub face: &'a dyn Fn([f64; 3]) -> bool,
    pub spacing_mm: f64,
}

/// The rows that hold the middle line's vertices on their pieces. `offsets[k]` is the offset of piece k.
fn line_rows(chain: &[usize], cones: &[usize], coords: &[[Coord; 2]], offsets: &[Coord], strength: f64) -> Vec<Row> {
    let turn = std::f64::consts::PI / cones.len() as f64;
    let mut rows = Vec::new();
    for (k, &v) in chain.iter().enumerate() {
        let seg = cones.iter().filter(|&&c| k > c).count();
        let mut on = vec![seg];
        if cones.contains(&k) {
            on.push(seg + 1);
        }
        for s in on {
            let phi = std::f64::consts::FRAC_PI_2 + s as f64 * turn;
            let n = [-phi.sin(), phi.cos()];
            let mut row: Row = (Vec::new(), 0.0);
            push_term(&mut row, coords[v][0], strength * n[0]);
            push_term(&mut row, coords[v][1], strength * n[1]);
            push_term(&mut row, offsets[s], -strength);
            rows.push(row);
        }
    }
    rows
}

/// Lays the skin flat. `p` and `tris` are the welded mesh in millimetres, and `skin` says which triangles take part.
pub fn solve(p: &[[f64; 3]], tris: &[[usize; 3]], skin: &[bool], s: &Settings) -> Chart {
    let mut mesh = Mesh::default();
    let mut remap: HashMap<usize, usize> = HashMap::new();
    for (ti, w) in tris.iter().enumerate() {
        if !skin[ti] {
            continue;
        }
        let t = w.map(|v| {
            *remap.entry(v).or_insert_with(|| {
                mesh.p.push(p[v]);
                mesh.p.len() - 1
            })
        });
        mesh.t.push(t);
        mesh.parent.push(Some(ti));
        mesh.hole_mm.push(0.0);
    }
    let outward: f64 = mesh
        .t
        .iter()
        .map(|t| {
            let v = t.map(|i| mesh.p[i]);
            let c = [0, 1, 2].map(|a| (v[0][a] + v[1][a] + v[2][a]) / 3.0);
            dot(cross(sub(v[1], v[0]), sub(v[2], v[0])), sub(c, s.axis)).signum()
        })
        .sum();
    if outward < 0.0 {
        for t in &mut mesh.t {
            t.swap(1, 2);
        }
    }
    // Every hole is covered by a fan from its middle; the outline lowest on the head is the neck's cut, which stays open.
    let loops = mesh.boundary_loops();
    let low = |lp: &Vec<usize>| lp.iter().map(|&v| mesh.p[v][1]).sum::<f64>() / lp.len() as f64;
    let outer = (0..loops.len()).min_by(|&a, &b| low(&loops[a]).total_cmp(&low(&loops[b]))).expect("outline");
    for (k, lp) in loops.iter().enumerate() {
        if k == outer {
            continue;
        }
        let c = [0, 1, 2].map(|a| lp.iter().map(|&v| mesh.p[v][a]).sum::<f64>() / lp.len() as f64);
        let around: f64 = (0..lp.len()).map(|e| { let d = sub(mesh.p[lp[e]], mesh.p[lp[(e + 1) % lp.len()]]); dot(d, d).sqrt() }).sum();
        mesh.p.push(c);
        let ci = mesh.p.len() - 1;
        for e in 0..lp.len() {
            mesh.t.push([lp[(e + 1) % lp.len()], lp[e], ci]);
            mesh.parent.push(None);
            mesh.hole_mm.push(around);
        }
    }
    for _ in 0..s.subdivisions {
        mesh = mesh.subdivide();
    }
    let (mesh, side) = slice(&mesh);
    if std::env::var("HOLES").is_ok() {
        for lp in loops.iter() {
            let c = [0, 1, 2].map(|a| lp.iter().map(|&v| mesh.p[v][a]).sum::<f64>() / lp.len() as f64);
            let around: f64 = (0..lp.len()).map(|e| { let d = sub(mesh.p[lp[e]], mesh.p[lp[(e + 1) % lp.len()]]); dot(d, d).sqrt() }).sum();
            println!("hole round ({:.0}, {:.0}, {:.0}), {around:.0} mm round", c[0], c[1], c[2]);
        }
    }
    println!("chart: {} holes covered; {} triangles and {} vertices after {} subdivisions and the cut", loops.len() - 1, mesh.t.len(), mesh.p.len(), s.subdivisions);

    // The middle line: the run of the x < 0 half's outline on the plane x = 0, from its front end at the neck to its back end.
    let ht: Vec<[usize; 3]> = (0..mesh.t.len()).filter(|&k| side[k] == 0).map(|k| mesh.t[k]).collect();
    let half = Mesh { p: mesh.p.clone(), t: ht.clone(), parent: vec![None; ht.len()], hole_mm: vec![0.0; ht.len()] };
    let loops = half.boundary_loops();
    let lp = loops.iter().max_by_key(|l| l.len()).expect("outline");
    let on = |v: usize| mesh.p[v][0] == 0.0;
    let n = lp.len();
    let first = (0..n).find(|&k| on(lp[k]) && !on(lp[(k + n - 1) % n])).expect("middle line");
    let mut chain: Vec<usize> = Vec::new();
    let mut k = first;
    while on(lp[k]) && chain.len() <= n {
        chain.push(lp[k]);
        k = (k + 1) % n;
    }
    if mesh.p[chain[0]][2] < mesh.p[chain[chain.len() - 1]][2] {
        chain.reverse();
    }
    let runs = (0..n).filter(|&k| on(lp[k]) && !on(lp[(k + n - 1) % n])).count();
    let (a, b) = (mesh.p[chain[0]], mesh.p[chain[chain.len() - 1]]);
    println!("chart: the middle line has {} vertices in {runs} runs, from ({:.0}, {:.0}) to ({:.0}, {:.0}) in height and depth; {} outlines", chain.len(), a[1], a[2], b[1], b[2], loops.len());
    let theta = |q: [f64; 3]| (q[1] - s.axis[1]).atan2(q[2] - s.axis[2]).to_degrees();
    let mut ang: Vec<f64> = chain.iter().map(|&v| theta(mesh.p[v])).collect();
    for k in 1..ang.len() {
        while ang[k] - ang[k - 1] > 180.0 {
            ang[k] -= 360.0;
        }
        while ang[k] - ang[k - 1] < -180.0 {
            ang[k] += 360.0;
        }
    }
    let cones: Vec<usize> = s.cones_deg.iter().map(|&d| (0..chain.len()).find(|&k| ang[k] >= d).expect("cone")).collect();
    let dist = |a: [f64; 3], b: [f64; 3]| dot(sub(a, b), sub(a, b)).sqrt();
    let ic = (0..cones[0]).min_by(|&a, &b| dist(mesh.p[chain[a]], s.chin).total_cmp(&dist(mesh.p[chain[b]], s.chin))).expect("chin");
    let arc: f64 = (ic..cones[0]).map(|k| dist(mesh.p[chain[k]], mesh.p[chain[k + 1]])).sum();

    let mut used = vec![false; mesh.p.len()];
    for t in &ht {
        for &v in t {
            used[v] = true;
        }
    }
    let nseg = cones.len() + 1;
    let mut unknowns = nseg - 1;
    let mut offsets: Vec<Coord> = std::iter::once(Coord::Fixed(0.0)).chain((0..nseg - 1).map(Coord::Free)).collect();
    let mut coords = vec![[Coord::Fixed(0.0); 2]; mesh.p.len()];
    for v in 0..mesh.p.len() {
        if !used[v] {
            continue;
        }
        let u = Coord::Free(unknowns);
        unknowns += 1;
        let vv = if v == chain[ic] {
            Coord::Fixed(0.0)
        } else {
            unknowns += 1;
            Coord::Free(unknowns - 1)
        };
        coords[v] = [u, vv];
    }
    let weight: Vec<f64> = ht
        .iter()
        .map(|t| {
            let v = t.map(|i| mesh.p[i]);
            let c = [0, 1, 2].map(|a| (v[0][a] + v[1][a] + v[2][a]) / 3.0);
            (s.weight)(c, normalize(cross(sub(v[1], v[0]), sub(v[2], v[0]))))
        })
        .collect();
    const STRENGTH: f64 = 30.0;
    let rigid = |steps: &[[f64; 3]], offsets: &[Coord], mut x: Vec<f64>| {
        let lines = line_rows(&chain, &cones, &coords, offsets, STRENGTH);
        for _ in 0..s.rigid_steps {
            let rot = rotations(steps, &ht, &coords, &x);
            let mut rows = rigid_rows(steps, &ht, &coords, &rot, &weight);
            rows.extend(lines.iter().cloned());
            x = least_squares(&rows, unknowns, x);
        }
        x
    };

    // First the conformal map, with row 0 at the chin and the first cone as far up as the middle line is long; then the rigid one.
    let steps: Vec<[f64; 3]> = mesh.p.iter().map(|q| q.map(|x| x / s.spacing_mm)).collect();
    let mut rows = conformal_rows(&steps, &ht, &coords);
    rows.extend(line_rows(&chain, &cones, &coords, &offsets, STRENGTH));
    let mut pin: Row = (Vec::new(), -STRENGTH * arc / s.spacing_mm);
    push_term(&mut pin, coords[chain[cones[0]]][1], STRENGTH);
    rows.push(pin);
    let x = least_squares(&rows, unknowns, vec![0.0; unknowns]);
    let mut x = rigid(&steps, &offsets, x);

    // The flat map comes out a little small, because the skin must stretch somewhere to lie flat. It is scaled so that on the face a step
    // is `spacing_mm`; then each piece's line is moved to the nearest place where the mirror image of a whole column and row is whole
    // again, and the map is laid once more with the pieces held there.
    let mut face: Vec<f64> = Vec::new();
    for t in &ht {
        let c = [0, 1, 2].map(|a| t.iter().map(|&v| mesh.p[v][a]).sum::<f64>() / 3.0);
        if !(s.face)(c) {
            continue;
        }
        let Some((g, _)) = gradients(&mesh.p, *t) else { continue };
        for which in 0..2 {
            let gr = [0, 1].map(|a| (0..3).map(|k| value(coords[t[k]][which], &x) * g[k][a]).sum::<f64>());
            face.push(1.0 / gr[0].hypot(gr[1]).max(1e-9));
        }
    }
    face.sort_by(f64::total_cmp);
    let fit = face[face.len() / 2] / s.spacing_mm;
    for xi in x.iter_mut() {
        *xi *= fit;
    }
    let turn = std::f64::consts::PI / cones.len() as f64;
    for (k, o) in offsets.iter_mut().enumerate().skip(1) {
        let Coord::Free(i) = *o else { continue };
        let phi = std::f64::consts::FRAC_PI_2 + k as f64 * turn;
        let diagonal = (phi.sin().abs() - phi.cos().abs()).abs() < 0.5;
        let c = if diagonal { (x[i] * std::f64::consts::SQRT_2).round() / std::f64::consts::SQRT_2 } else { (x[i] * 2.0).round() / 2.0 };
        *o = Coord::Fixed(c);
    }
    let steps: Vec<[f64; 3]> = mesh.p.iter().map(|q| q.map(|v| v * fit / s.spacing_mm)).collect();
    let x = rigid(&steps, &offsets, x);
    println!("chart: the face's step was {:.2} mm before scaling; the middle line's pieces lie at {:?}", s.spacing_mm * fit, offsets.iter().map(|o| (value(*o, &x) * 100.0).round() / 100.0).collect::<Vec<_>>());

    let mut uv = vec![[[f64::NAN; 2]; 2]; mesh.p.len()];
    for v in 0..mesh.p.len() {
        if used[v] {
            uv[v][0] = [value(coords[v][0], &x), value(coords[v][1], &x)];
        }
    }
    // The middle line's vertices are laid exactly on their pieces, and the cones where two pieces meet.
    for (k, &v) in chain.iter().enumerate() {
        let seg = cones.iter().filter(|&&c| k > c).count();
        let pieces: Vec<usize> = if cones.contains(&k) { vec![seg, seg + 1] } else { vec![seg] };
        let normal = |s: usize| { let phi = std::f64::consts::FRAC_PI_2 + s as f64 * turn; [-phi.sin(), phi.cos()] };
        let q = uv[v][0];
        uv[v][0] = if let [a, b] = pieces[..] {
            let (na, nb) = (normal(a), normal(b));
            let (ca, cb) = (value(offsets[a], &x), value(offsets[b], &x));
            let det = na[0] * nb[1] - na[1] * nb[0];
            [(ca * nb[1] - cb * na[1]) / det, (na[0] * cb - nb[0] * ca) / det]
        } else {
            let n = normal(seg);
            let d = n[0] * q[0] + n[1] * q[1] - value(offsets[seg], &x);
            [q[0] - d * n[0], q[1] - d * n[1]]
        };
    }
    let at = |k: usize| uv[chain[k]][0];
    let ends: Vec<usize> = std::iter::once(0).chain(cones.iter().copied()).chain(std::iter::once(chain.len() - 1)).collect();
    let segments: Vec<Segment> = (0..nseg)
        .map(|k| {
            let phi = std::f64::consts::FRAC_PI_2 + k as f64 * turn;
            Segment { normal: [-phi.sin(), phi.cos()], offset: value(offsets[k], &x), ends: [at(ends[k]), at(ends[k + 1])] }
        })
        .collect();

    // The x > 0 half takes the column and row of its mirror image's place on the x < 0 half, with the column's sign turned.
    let lt: Vec<[u32; 3]> = ht.iter().map(|t| t.map(|i| i as u32)).collect();
    let left = head_paint::closest::Bvh::new(mesh.p.clone(), lt);
    for k in 0..mesh.t.len() {
        if side[k] != 1 {
            continue;
        }
        for &v in &mesh.t[k] {
            if uv[v][1][0].is_finite() {
                continue;
            }
            let c = left.closest([-mesh.p[v][0], mesh.p[v][1], mesh.p[v][2]]).expect("x < 0 half");
            let t = ht[c.triangle as usize];
            let q = [0, 1].map(|a| (0..3).map(|e| c.barycentric[e] * uv[t[e]][0][a]).sum::<f64>());
            uv[v][1] = [-q[0], q[1]];
        }
    }
    Chart { mesh, side, uv, segments }
}
