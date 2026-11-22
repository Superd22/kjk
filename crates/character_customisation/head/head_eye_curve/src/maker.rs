//! The head's mesh and frame as the construction needs them, and `Maker`, which builds a whole head (both eyes) from the live head and
//! the ten handles.

use crate::curve::{self, EyeEdges, SIZES, Target, Topo};
use crate::refine::Refine;
use crate::{V3, add, cross, dot, length, normalize, scale, sub};
use std::collections::HashSet;

/// The shared head mesh: triangles, each vertex's merged vertex, one vertex per merged vertex, the hidden merged vertices (the insides
/// of the nostrils and the mouth) and the merged neighbours.
pub struct Mesh {
    pub triangles: Vec<[u32; 3]>,
    pub group: Vec<u32>,
    pub first: Vec<u32>,
    pub hidden: Vec<bool>,
    offsets: Vec<u32>,
    neighbours: Vec<u32>,
}

impl Mesh {
    pub fn new(triangles: Vec<[u32; 3]>, group: Vec<u32>, first: Vec<u32>, hidden: Vec<bool>) -> Self {
        let n = first.len();
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for t in &triangles {
            let g = t.map(|v| group[v as usize]);
            for (a, b) in [(g[0], g[1]), (g[1], g[2]), (g[2], g[0])] {
                if a != b {
                    edges.push((a, b));
                    edges.push((b, a));
                }
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
        Self { triangles, group, first, hidden, offsets, neighbours }
    }

    pub fn merged_count(&self) -> usize {
        self.first.len()
    }

    pub fn neighbours_of(&self, g: usize) -> &[u32] {
        &self.neighbours[self.offsets[g] as usize..self.offsets[g + 1] as usize]
    }
}

/// `head_sliders::measure::Frame::head()`: the base head's frame of the eyes, the same for every head (mm in the frame).
#[derive(Clone, Copy)]
pub struct Frame {
    origin: V3,
    axes: [V3; 3],
    scale: f64,
}

const BASE_EYES_M: [V3; 2] = [[0.029543484810493445, 0.7303516981330398, 0.11696164284455376], [-0.029459976005412943, 0.7304376213611672, 0.11543812671549593]];
const EYES_APART_MM: f64 = 63.0;
const BALL_MM: f64 = 12.3;
const RING_MM: f64 = 6.907;

impl Frame {
    pub fn head() -> Self {
        let eyes = BASE_EYES_M;
        let origin = scale(add(eyes[0], eyes[1]), 0.5);
        let across = sub(eyes[0], eyes[1]);
        let x = normalize(across);
        let up = [0.0, 1.0, 0.0];
        let y = normalize(sub(up, scale(x, dot(up, x))));
        let mut z = cross(x, y);
        if z[2] < 0.0 {
            z = scale(z, -1.0);
        }
        Self { origin, axes: [x, y, z], scale: 1000.0 * EYES_APART_MM / (length(across) * 1000.0) }
    }

    pub fn local(&self, p: V3) -> V3 {
        let d = sub(p, self.origin);
        self.axes.map(|a| dot(d, a) * self.scale)
    }

    pub fn world(&self, q: V3) -> V3 {
        let mut p = self.origin;
        for (a, v) in self.axes.iter().zip(q) {
            p = add(p, scale(*a, v / self.scale));
        }
        p
    }

    fn world_dir(&self, q: V3) -> V3 {
        sub(self.world(q), self.origin)
    }
}

const LID_EDGES: &str = include_str!("lid_edges.json");

/// Both eyes from `lid_edges.json`, "L" first.
pub fn edges() -> [EyeEdges; 2] {
    let v: serde_json::Value = serde_json::from_str(LID_EDGES).expect("lid_edges.json");
    let u = |x: &serde_json::Value| x.as_u64().unwrap() as u32;
    ["L", "R"].map(|k| {
        let e = &v["eyes"][k];
        EyeEdges {
            edge: e["edge"].as_array().unwrap().iter().map(u).collect(),
            inner: u(&e["inner_corner"]),
            outer: u(&e["outer_corner"]),
            ring: e["eyeball_centre_vertices"].as_array().unwrap().iter().map(u).collect(),
            upper: e["lid"].as_array().unwrap().iter().map(|x| x.as_str() == Some("upper")).collect(),
        }
    })
}

/// An eyeball's centre (metres) and radius (mm on the base head's scale) from its ring of 8 vertices, as `eye_curves::ball`.
fn ball(p: &[V3], ring: &[u32]) -> (V3, f64) {
    let mut c = [0.0; 3];
    for &v in ring {
        c = add(c, p[v as usize]);
    }
    c = scale(c, 1.0 / ring.len() as f64);
    let ms = ring.iter().map(|&v| length(sub(p[v as usize], c)).powi(2)).sum::<f64>() / ring.len() as f64;
    (c, BALL_MM * ms.sqrt() * 1000.0 / RING_MM)
}

/// One eye's lid piece, ready to draw: the open piece's points (metres, relative to `centre`), its triangles, and the turn that
/// closes it: `angle` degrees about `axis` (a unit direction in the head's space, by the right-hand rule) through `centre`.
pub struct Lid {
    pub points: Vec<V3>,
    pub triangles: Vec<[u32; 3]>,
    pub centre: V3,
    pub axis: V3,
    pub angle: f64,
    /// The curve's upper and lower arches, from the inner tip to the outer one, as points on the eyeball's surface (metres, the head's
    /// space). The lid piece of `head_eyes` is built on them.
    pub upper: Vec<V3>,
    pub lower: Vec<V3>,
}

pub struct Maker {
    mesh: Mesh,
    edges: [EyeEdges; 2],
    topos: [Topo; 2],
    keep: [HashSet<usize>; 2],
    frame: Frame,
    pub lids: Vec<Lid>,
    pub refine: Refine,
    /// The new vertices of the last build (`refine`), metres, in the fit's frame.
    pub extra: Vec<V3>,
}

const LID_LIFT: f64 = 0.08;
const LID_MARGIN: f64 = 6.0;

fn rotate(d: V3, k: V3, deg: f64) -> V3 {
    let (s, c) = deg.to_radians().sin_cos();
    add(add(scale(d, c), scale(cross(k, d), s)), scale(k, dot(k, d) * (1.0 - c)))
}

impl Maker {
    pub fn new(mesh: Mesh) -> Self {
        let edges = edges();
        let topos = [0, 1].map(|k| curve::topo(&mesh, &edges[k], SIZES.roll));
        let keep = [0, 1].map(|k| {
            let o = &topos[1 - k];
            topos[k].ring_g.iter().chain(&o.loop_g).chain(&o.inside).chain(&o.ring_g).copied().collect()
        });
        let hidden: HashSet<usize> = (0..mesh.triangles.len()).filter(|&t| mesh.triangles[t].iter().any(|&v| mesh.hidden[mesh.group[v as usize] as usize])).collect();
        let refine = Refine::new(&mesh, &topos, [&edges[0].upper, &edges[1].upper], &hidden);
        Self { mesh, edges, topos, keep, frame: Frame::head(), lids: Vec::new(), refine, extra: Vec::new() }
    }

    /// The number of head vertices the mesh has (before merging).
    pub fn vertex_count(&self) -> usize {
        self.mesh.group.len()
    }

    /// The head `built` (every vertex, metres, the fit's frame) with each eye's opening built on the curve of the handles `h`.
    pub fn build(&mut self, built: &[V3], h: &[f64]) -> Vec<V3> {
        let frame = self.frame;
        let mut p: Vec<V3> = self.mesh.first.iter().map(|&i| frame.local(built[i as usize])).collect();
        let balls = [0, 1].map(|k| {
            let (c, r) = ball(built, &self.edges[k].ring);
            (frame.local(c), r * frame.scale / 1000.0)
        });
        self.lids.clear();
        let mut eyes = [([0.0; 3], 1.0, 1.0); 2];
        for k in 0..2 {
            let (c, r) = balls[k];
            let side = if c[0] >= 0.0 { 1.0 } else { -1.0 };
            let t = Target::from_handles(h, r);
            curve::build_eye(&self.mesh, &self.topos[k], &mut p, c, side, r, &t, SIZES, &self.keep[k]);
            self.lids.push(self.lid(&t, c, side, r));
            eyes[k] = (c, side, r);
        }
        self.extra = self.refine.positions(&self.mesh, &self.topos, &p, &eyes).into_iter().map(|q| frame.world(q)).collect();
        self.mesh.group.iter().map(|&g| frame.world(p[g as usize])).collect()
    }

    fn lid(&self, t: &Target, c: V3, side: f64, r: f64) -> Lid {
        let frame = self.frame;
        let (pts, cols, rows, ax, theta) = curve::lid_piece(t, r, LID_LIFT, LID_MARGIN);
        let centre = frame.world(c);
        let to_world = |q: V3| sub(frame.world(curve::from_eye(c, side, q)), centre);
        let points: Vec<V3> = pts.iter().map(|&q| to_world(q)).collect();
        let mut triangles = Vec::new();
        for i in 0..cols - 1 {
            for j in 0..rows - 1 {
                let at = |i: usize, j: usize| (i * rows + j) as u32;
                for tr in [[at(i, j), at(i + 1, j), at(i + 1, j + 1)], [at(i, j), at(i + 1, j + 1), at(i, j + 1)]] {
                    let q = tr.map(|v| points[v as usize]);
                    let nn = cross(sub(q[1], q[0]), sub(q[2], q[0]));
                    triangles.push(if dot(nn, add(q[0], q[1])) < 0.0 { [tr[0], tr[2], tr[1]] } else { tr });
                }
            }
        }
        // the axis in the head's space: the eye frame is mirrored for one eye and the head frame may be, so the turn's sense is
        // found by turning one point and comparing it with the prototype's closed piece
        let axis = normalize(frame.world_dir([ax[0] * side, ax[1], ax[2]]));
        let probe = pts[(cols / 2) * rows];
        let closed = to_world(rotate(probe, ax, theta));
        let open = to_world(probe);
        let axis = if length(sub(rotate(open, axis, theta), closed)) <= length(sub(rotate(open, scale(axis, -1.0), theta), closed)) { axis } else { scale(axis, -1.0) };
        let on_ball = |d: &V3| frame.world(curve::from_eye(c, side, scale(*d, r)));
        Lid { points, triangles, centre, axis, angle: theta, upper: t.upper.iter().map(on_ball).collect(), lower: t.lower.iter().map(on_ball).collect() }
    }
}
