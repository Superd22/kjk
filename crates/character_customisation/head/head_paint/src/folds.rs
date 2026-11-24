//! The fold layer: the creases that anatomy places on every face, at any age (paint track, fine detail).
//!
//! A fold is not a wrinkle. A wrinkle comes with age (forehead lines, crow's feet) and lives in `wrinkles.rs`. The four folds here are
//! the nose-to-mouth fold (nasolabial), the mouth corner fold (marionette), the chin crease and the under-eye crease. Their sliders do
//! not depend on age, so a young face can have a marked nose-to-mouth fold.
//!
//! **The path comes from the head.** Each fold's path is computed on the head it is painted on. Its three control points are placed
//! from landmarks (`ANCHORS`: the nose wing, the mouth corner, the middle of the chin crease, the eye's corners and the lower lid), with
//! offsets in millimetres along the face's own outward and up directions, which are also taken from the landmarks. A curve through the
//! control points is laid on the head's surface. Each landmark is three mesh vertices and their weights, so when a slider moves the nose
//! wing or the mouth corner, the fold moves with it, and a small change of the head gives a small change of the path.
//!
//! **What the study of KOTOR's 57 heads showed** (`fold_place`, 2026-11-19). KOTOR's fold runs parallel to this path: along one fold its
//! sideways distance from the path changes by 0.4 to 0.7 mm (standard deviation). Between heads, though, the fold sits 2.2 to 4.9 mm
//! further in or out (standard deviation), and nothing measured on the head predicts that: not the valley of the surface, not the
//! darkest line of the shading layer, not the distance between the nose wing and the mouth corner, nor the mouth's width (all
//! correlations 0.42 or less, the strongest being the mouth corner fold against the shading's darkest line). So each fold has a shift
//! slider, and its default is KOTOR's mean shift.
//!
//! **The cross-section is a step between two tones.** KOTOR's nose-to-mouth fold is not a thin dark line. It is the border between a
//! darker cheek and a lighter area round the mouth, soft and several millimetres wide, and it reads mostly through its light side: over
//! 82 sides, the contrast against the lighter side is 6.8 L* against a depth of 4.0 L*. So a fold here has a darkening that reaches
//! `width_mm` into the outer side, a lightening (`highlight`) in a band half as wide on the inner side, and a border whose softness is
//! `edge_mm` (see `step`). Each falloff is exp(-(d / w)^1.6), scaled to be one half at its width, and fades to zero between 2.5 and 3.5
//! widths, so a fold costs nothing further away.
//!
//! **Determinism.** The same slider values on the same head always give the same pixels. Every yes-or-no choice and every random
//! number comes from a whole-number hash of the seed (`noise::hash3`), never from a clock, a thread or a file.
//!
//! Lengths and widths are in millimetres on the head. Darkness is in L*.
//!
//! # The sliders
//!
//! Each fold has these sliders. Left and right share them by default (`Fold::linked`). The ranges are the ranges the fitted KOTOR heads
//! use, with a margin.
//!
//! | Slider | Range | Default | What it does |
//! |---|---|---|---|
//! | `strength` | 0 to 30 L* | 0 | How much darker the outer side is at the fold's border (the cheek, for the nose-to-mouth fold). |
//! | `highlight` | 0 to 20 L* | 0 | How much lighter the inner side is just inside the border (round the mouth, for the nose-to-mouth fold). With strength and highlight at zero the fold paints nothing. |
//! | `width_mm` | 0.5 to 10 mm | per fold | How far the darkening reaches into the outer side before it falls to half. The light band on the inner side is half as wide. |
//! | `edge_mm` | 0.2 to 5 mm | per fold | How soft the border is: the distance over which the dark side gives way to the light side. |
//! | `start_mm` | 0 to 25 mm | 0 | Where along the path the fold begins, from its first control point. |
//! | `length_mm` | 3 to 60 mm | per fold | How long the fold is, from its start. |
//! | `shift_mm` | -8 to 8 mm | per fold | How far the whole fold sits from its computed path, along the skin: positive is outward, or downward for the chin and under-eye creases. |
//!
//! Each fold also has `variation` (0 to 1, default 0) and `seed` (any whole number). The variation bends the fold by up to 1.5 mm at its
//! middle, changes its length by up to 20%, and makes the two sides differ by up to 1.5 mm in shift and 15% in length, strength and
//! highlight. At 0
//! the two sides are mirror images. The per-fold defaults are in `DEFAULTS`.


use crate::layout_points::{cross, dot, length, normalize, sub};

/// A point of the head given by three mesh vertices and their weights, so it moves with those vertices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub vertices: [u32; 3],
    pub weights: [f64; 3],
}

/// The folds' landmarks. A pair holds the point on the x < 0 side first, then its mirror on the x > 0 side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchors {
    pub nose_wing: [Anchor; 2],
    pub mouth_corner: [Anchor; 2],
    pub chin_middle: Anchor,
    pub lower_lip_middle: Anchor,
    pub eye_inner: [Anchor; 2],
    pub eye_outer: [Anchor; 2],
    pub lower_lid: [Anchor; 2],
    pub nose_tip: Anchor,
}

pub const ANCHORS: Anchors = Anchors {
    nose_wing: [Anchor { vertices: [311, 1813, 366], weights: [0.51203, 0.38339, 0.10458] }, Anchor { vertices: [4145, 4105, 2709], weights: [0.85109, 0.05926, 0.08965] }],
    mouth_corner: [Anchor { vertices: [464, 1750, 1749], weights: [0.29807, 0.43844, 0.26349] }, Anchor { vertices: [2831, 2832, 2828], weights: [0.59915, 0.20779, 0.19306] }],
    chin_middle: Anchor { vertices: [2917, 1164, 1163], weights: [0.04626, 0.41041, 0.54333] },
    lower_lip_middle: Anchor { vertices: [731, 2916, 728], weights: [0.42016, 0.02767, 0.55217] },
    eye_inner: [Anchor { vertices: [83, 1214, 1828], weights: [0.39501, 0.13049, 0.47451] }, Anchor { vertices: [2495, 3552, 3551], weights: [0.49303, 0.42608, 0.08089] }],
    eye_outer: [Anchor { vertices: [964, 93, 94], weights: [0.13317, 0.49616, 0.37066] }, Anchor { vertices: [3318, 2575, 2505], weights: [0.59325, 0.18132, 0.22543] }],
    lower_lid: [Anchor { vertices: [1221, 1239, 2019], weights: [0.23422, 0.74039, 0.02539] }, Anchor { vertices: [3579, 4342, 3576], weights: [0.46671, 0.13547, 0.39782] }],
    nose_tip: Anchor { vertices: [281, 282, 283], weights: [0.88809, 0.07562, 0.03629] },
};

impl Anchors {
    pub fn all(&self) -> Vec<Anchor> {
        let mut v = Vec::new();
        for pair in [self.nose_wing, self.mouth_corner, self.eye_inner, self.eye_outer, self.lower_lid] {
            v.extend(pair);
        }
        v.extend([self.chin_middle, self.lower_lip_middle, self.nose_tip]);
        v
    }

    /// An anchor's point in millimetres, from the head's mesh positions in metres.
    pub fn point(a: &Anchor, positions: &[[f64; 3]]) -> [f64; 3] {
        let mut p = [0.0; 3];
        for (v, w) in a.vertices.iter().zip(a.weights) {
            for k in 0..3 {
                p[k] += 1000.0 * w * positions[*v as usize][k];
            }
        }
        p
    }
}

/// The closest point of the head's surface to a point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceHit {
    pub triangle: usize,
    pub barycentric: [f64; 3],
    pub point: [f64; 3],
    pub normal: [f64; 3],
    pub distance: f64,
}

/// The head's triangles in millimetres, with a grid to find the closest point of the surface.
pub struct SurfaceIndex {
    positions: Vec<[f64; 3]>,
    triangles: Vec<[u32; 3]>,
    cell: f64,
    origin: [f64; 3],
    dims: [usize; 3],
    cells: Vec<Vec<u32>>,
}

const INDEX_CELL_MM: f64 = 4.0;

fn closest_on_triangle(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return [0.0, 1.0, 0.0];
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [1.0 - v, v, 0.0];
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return [0.0, 0.0, 1.0];
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [1.0 - w, 0.0, w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }
    let den = 1.0 / (va + vb + vc);
    let (v, w) = (vb * den, vc * den);
    [1.0 - v - w, v, w]
}

impl SurfaceIndex {
    /// `positions` holds the head's positions in metres, one per vertex of `triangles`.
    pub fn new(triangles: &[[u32; 3]], positions: &[[f64; 3]]) -> Self {
        let positions: Vec<[f64; 3]> = positions.iter().map(|p| p.map(|x| 1000.0 * x)).collect();
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for t in triangles {
            for &i in t {
                for k in 0..3 {
                    lo[k] = lo[k].min(positions[i as usize][k]);
                    hi[k] = hi[k].max(positions[i as usize][k]);
                }
            }
        }
        let cell = INDEX_CELL_MM;
        let dims = [0, 1, 2].map(|k| (((hi[k] - lo[k]) / cell).floor() as usize + 1).max(1));
        let mut cells = vec![Vec::new(); dims[0] * dims[1] * dims[2]];
        for (n, t) in triangles.iter().enumerate() {
            let c = t.map(|i| positions[i as usize]);
            let a = [0, 1, 2].map(|k| ((c[0][k].min(c[1][k]).min(c[2][k]) - lo[k]) / cell).floor() as usize);
            let b = [0, 1, 2].map(|k| (((c[0][k].max(c[1][k]).max(c[2][k]) - lo[k]) / cell).floor() as usize).min(dims[k] - 1));
            for x in a[0]..=b[0] {
                for y in a[1]..=b[1] {
                    for z in a[2]..=b[2] {
                        cells[(x * dims[1] + y) * dims[2] + z].push(n as u32);
                    }
                }
            }
        }
        SurfaceIndex { positions, triangles: triangles.to_vec(), cell, origin: lo, dims, cells }
    }

    /// The closest point of the surface within 12 mm of `p` (in millimetres).
    pub fn closest(&self, p: [f64; 3]) -> Option<SurfaceHit> {
        let mut best: Option<SurfaceHit> = None;
        let c = [0, 1, 2].map(|k| ((p[k] - self.origin[k]) / self.cell).floor() as i64);
        for ring in 0..3i64 {
            for x in c[0] - ring..=c[0] + ring {
                for y in c[1] - ring..=c[1] + ring {
                    for z in c[2] - ring..=c[2] + ring {
                        if (x - c[0]).abs().max((y - c[1]).abs()).max((z - c[2]).abs()) != ring {
                            continue;
                        }
                        if x < 0 || y < 0 || z < 0 || x as usize >= self.dims[0] || y as usize >= self.dims[1] || z as usize >= self.dims[2] {
                            continue;
                        }
                        for &t in &self.cells[(x as usize * self.dims[1] + y as usize) * self.dims[2] + z as usize] {
                            let [a, b, cc] = self.triangles[t as usize].map(|i| self.positions[i as usize]);
                            let w = closest_on_triangle(p, a, b, cc);
                            let q = [0, 1, 2].map(|k| w[0] * a[k] + w[1] * b[k] + w[2] * cc[k]);
                            let d = length(sub(p, q));
                            if best.is_none_or(|h| d < h.distance || (d == h.distance && (t as usize) < h.triangle)) {
                                best = Some(SurfaceHit { triangle: t as usize, barycentric: w, point: q, normal: normalize(cross(sub(b, a), sub(cc, a))), distance: d });
                            }
                        }
                    }
                }
            }
            // A hit nearer than the cells already searched cannot be beaten by the next ring.
            if best.is_some_and(|h| h.distance <= ring as f64 * self.cell) {
                break;
            }
        }
        best
    }

    /// The layout point (u, v) of a hit, from the layout's points and its triangles (the same triangles the index was built from).
    pub fn uv_of(&self, hit: SurfaceHit, uv: &[[f64; 2]], triangles: &[[u32; 3]]) -> [f64; 2] {
        let t = triangles[hit.triangle].map(|i| uv[i as usize]);
        [0, 1].map(|k| hit.barycentric[0] * t[0][k] + hit.barycentric[1] * t[1][k] + hit.barycentric[2] * t[2][k])
    }
}

/// The four folds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldKind {
    Nasolabial = 0,
    Marionette = 1,
    Chin = 2,
    UnderEye = 3,
}

pub const FOLD_COUNT: usize = 4;
pub const KINDS: [FoldKind; FOLD_COUNT] = [FoldKind::Nasolabial, FoldKind::Marionette, FoldKind::Chin, FoldKind::UnderEye];

impl FoldKind {
    pub fn id(self) -> &'static str {
        ["nasolabial", "marionette", "chin", "under_eye"][self as usize]
    }
    pub fn label(self) -> &'static str {
        ["Nose-to-mouth fold", "Mouth corner fold", "Chin crease", "Under-eye crease"][self as usize]
    }
}

/// The landmarks a control point is measured from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    NoseWing,
    MouthCorner,
    ChinMiddle,
    EyeInner,
    LowerLid,
    EyeOuter,
    /// Halfway between the first and the last control point.
    Middle,
}

/// One control point of a fold's path: a landmark and an offset from it in millimetres, outward (away from the face's middle line, on the
/// fold's side) and up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Control {
    pub from: Mark,
    pub out_mm: f64,
    pub up_mm: f64,
}

/// A fold's path: three control points, and how far it goes on past the last one, along its direction there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathShape {
    pub controls: [Control; 3],
    pub extend_mm: f64,
}

/// The directions of the face, from its landmarks: across from the x < 0 side to the x > 0 side, up, and forward.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceFrame {
    pub across: [f64; 3],
    pub up: [f64; 3],
    pub forward: [f64; 3],
}

impl FaceFrame {
    /// The direction away from the face's middle line on a side (0 for x < 0, 1 for x > 0).
    pub fn out(&self, side: usize) -> [f64; 3] {
        if side == 0 { self.across.map(|x| -x) } else { self.across }
    }
}

/// The landmarks of one head, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Landmarks {
    pub nose_wing: [[f64; 3]; 2],
    pub mouth_corner: [[f64; 3]; 2],
    pub chin_middle: [f64; 3],
    pub lower_lip_middle: [f64; 3],
    pub eye_inner: [[f64; 3]; 2],
    pub eye_outer: [[f64; 3]; 2],
    pub lower_lid: [[f64; 3]; 2],
    pub nose_tip: [f64; 3],
    pub frame: FaceFrame,
}

fn mid(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|k| 0.5 * (a[k] + b[k]))
}

fn add_scaled(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}

impl Landmarks {
    /// The landmarks of a head, from its mesh positions in metres.
    pub fn new(positions: &[[f64; 3]]) -> Self {
        let a = &ANCHORS;
        let pt = |x: &Anchor| Anchors::point(x, positions);
        let pair = |x: &[Anchor; 2]| [pt(&x[0]), pt(&x[1])];
        let (eye_inner, eye_outer) = (pair(&a.eye_inner), pair(&a.eye_outer));
        let mouth_corner = pair(&a.mouth_corner);
        let eyes = [mid(eye_inner[0], eye_outer[0]), mid(eye_inner[1], eye_outer[1])];
        let across = normalize(sub(eyes[1], eyes[0]));
        let rise = sub(mid(eyes[0], eyes[1]), mid(mouth_corner[0], mouth_corner[1]));
        let up = normalize(add_scaled(rise, across, -dot(rise, across)));
        let forward = cross(across, up);
        Landmarks {
            nose_wing: pair(&a.nose_wing),
            mouth_corner,
            chin_middle: pt(&a.chin_middle),
            lower_lip_middle: pt(&a.lower_lip_middle),
            eye_inner,
            eye_outer,
            lower_lid: pair(&a.lower_lid),
            nose_tip: pt(&a.nose_tip),
            frame: FaceFrame { across, up, forward },
        }
    }

    fn landmark(&self, from: Mark, side: usize) -> [f64; 3] {
        match from {
            Mark::NoseWing => self.nose_wing[side],
            Mark::MouthCorner => self.mouth_corner[side],
            Mark::ChinMiddle => self.chin_middle,
            Mark::EyeInner => self.eye_inner[side],
            Mark::LowerLid => self.lower_lid[side],
            Mark::EyeOuter => self.eye_outer[side],
            Mark::Middle => unreachable!("the middle is placed from the other control points"),
        }
    }

    /// The control points of a path on one side, in millimetres, before they are laid on the surface.
    pub fn controls(&self, shape: &PathShape, side: usize) -> [[f64; 3]; 3] {
        let out = self.frame.out(side);
        let place = |c: &Control, base: [f64; 3]| add_scaled(add_scaled(base, out, c.out_mm), self.frame.up, c.up_mm);
        let first = place(&shape.controls[0], self.landmark(shape.controls[0].from, side));
        let last = place(&shape.controls[2], self.landmark(shape.controls[2].from, side));
        let middle = match shape.controls[1].from {
            Mark::Middle => place(&shape.controls[1], mid(first, last)),
            f => place(&shape.controls[1], self.landmark(f, side)),
        };
        [first, middle, last]
    }
}

/// A fold's path laid on the head's surface: points about `PATH_STEP_MM` apart, with the surface's normal, the direction across the path
/// that points outward (away from the face's middle, or down for the chin crease), and the distance along the path.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SurfacePath {
    pub points: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub across: Vec<[f64; 3]>,
    pub along_mm: Vec<f64>,
    /// The distance along the path of the last control point. Beyond it lies the extension.
    pub main_mm: f64,
}

pub const PATH_STEP_MM: f64 = 0.25;

/// A quadratic curve through three points: it passes through the first at t = 0, the second at t = 0.5 and the third at t = 1.
fn through_three(c: &[[f64; 3]; 3], t: f64) -> [f64; 3] {
    let ctrl = [0, 1, 2].map(|k| 2.0 * c[1][k] - 0.5 * (c[0][k] + c[2][k]));
    let u = 1.0 - t;
    [0, 1, 2].map(|k| u * u * c[0][k] + 2.0 * u * t * ctrl[k] + t * t * c[2][k])
}

impl SurfacePath {
    /// Lays a path on the surface: the curve through the control points, then `extend_mm` more along its direction at the end. Each point
    /// is moved to the closest point of the surface. The `across` vectors lie along the surface, square to the path, and lean toward
    /// `outward`. They use the surface's normal averaged over 2 mm either way along the path, so one steep triangle beside the nose wing
    /// does not turn them.
    pub fn lay(index: &SurfaceIndex, controls: &[[f64; 3]; 3], extend_mm: f64, outward: [f64; 3]) -> Self {
        let mut raw: Vec<[f64; 3]> = (0..=400).map(|i| through_three(controls, f64::from(i) / 400.0)).collect();
        let mut main = 0.0;
        for w in raw.windows(2) {
            main += length(sub(w[1], w[0]));
        }
        let end = raw[raw.len() - 1];
        let dir = normalize(sub(end, through_three(controls, 0.99)));
        let steps = (extend_mm / 0.1).ceil() as usize;
        for i in 1..=steps {
            raw.push(add_scaled(end, dir, extend_mm * i as f64 / steps as f64));
        }
        let mut path = SurfacePath::default();
        let mut along = 0.0;
        let mut last: Option<[f64; 3]> = None;
        let mut main_mm = None;
        let main_count = 401;
        for (i, p) in raw.iter().enumerate() {
            let Some(hit) = index.closest(*p) else { continue };
            if let Some(prev) = last {
                let step = length(sub(hit.point, prev));
                if step < PATH_STEP_MM && i + 1 != raw.len() && i + 1 != main_count {
                    continue;
                }
                along += step;
            }
            last = Some(hit.point);
            path.points.push(hit.point);
            path.normals.push(hit.normal);
            path.along_mm.push(along);
            if i + 1 == main_count {
                main_mm = Some(along);
            }
        }
        path.main_mm = main_mm.unwrap_or(main.min(along));
        let n = path.points.len();
        for i in 0..n {
            let t = normalize(sub(path.points[(i + 1).min(n - 1)], path.points[i.saturating_sub(1)]));
            let window = (2.0 / PATH_STEP_MM) as usize;
            let mut normal = [0.0; 3];
            for j in i.saturating_sub(window)..(i + window + 1).min(n) {
                normal = add_scaled(normal, path.normals[j], 1.0);
            }
            let mut a = normalize(cross(normalize(normal), t));
            if dot(a, outward) < 0.0 {
                a = a.map(|x| -x);
            }
            path.across.push(a);
        }
        path
    }

    pub fn total_mm(&self) -> f64 {
        *self.along_mm.last().unwrap_or(&0.0)
    }

    /// The point, normal and across vector at a distance along the path, read between the two nearest points.
    pub fn at(&self, s: f64) -> ([f64; 3], [f64; 3], [f64; 3]) {
        let n = self.points.len();
        let i = self.along_mm.partition_point(|&a| a <= s).clamp(1, n - 1);
        let (a, b) = (self.along_mm[i - 1], self.along_mm[i]);
        let t = if b > a { ((s - a) / (b - a)).clamp(0.0, 1.0) } else { 0.0 };
        let lerp = |p: [f64; 3], q: [f64; 3]| [0, 1, 2].map(|k| p[k] + t * (q[k] - p[k]));
        (lerp(self.points[i - 1], self.points[i]), normalize(lerp(self.normals[i - 1], self.normals[i])), normalize(lerp(self.across[i - 1], self.across[i])))
    }
}

/// What the fold layer needs of a whole head: its landmarks and its surface.
pub struct FoldHead {
    pub landmarks: Landmarks,
    pub index: SurfaceIndex,
}

impl FoldHead {
    /// `layout_triangles` and `layout_to_mesh` describe the layout, and `positions` holds the head's mesh positions in metres. It is
    /// `None` for a mesh that does not hold the landmarks' vertices.
    pub fn new(layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize], positions: &[[f64; 3]]) -> Option<Self> {
        if ANCHORS.all().iter().any(|a| a.vertices.iter().any(|&v| v as usize >= positions.len())) {
            return None;
        }
        let per_layout: Vec<[f64; 3]> = layout_to_mesh.iter().map(|&m| positions[m]).collect();
        Some(FoldHead { landmarks: Landmarks::new(positions), index: SurfaceIndex::new(layout_triangles, &per_layout) })
    }

    /// The direction a fold's across vectors lean toward on a side: outward, or down for the chin crease.
    pub fn outward(&self, kind: FoldKind, side: usize) -> [f64; 3] {
        match kind {
            FoldKind::Chin | FoldKind::UnderEye => self.landmarks.frame.up.map(|x| -x),
            _ => self.landmarks.frame.out(side),
        }
    }

    pub fn lay(&self, kind: FoldKind, shape: &PathShape, side: usize) -> SurfacePath {
        SurfacePath::lay(&self.index, &self.landmarks.controls(shape, side), shape.extend_mm, self.outward(kind, side))
    }
}

const fn c(from: Mark, out_mm: f64, up_mm: f64) -> Control {
    Control { from, out_mm, up_mm }
}

/// The folds' paths, placed from the landmarks. The fold measure searches around them, and the generator paints along them, moved
/// sideways by each fold's shift. They are drawn long, so that the measure can find where KOTOR's fold starts and ends and the length
/// slider has room.
pub const PATHS: [PathShape; FOLD_COUNT] = [
    PathShape { controls: [c(Mark::NoseWing, 1.0, 2.0), c(Mark::Middle, 2.0, 0.0), c(Mark::MouthCorner, 8.0, -2.0)], extend_mm: 12.0 },
    PathShape { controls: [c(Mark::MouthCorner, 1.0, -1.0), c(Mark::Middle, 0.5, 0.0), c(Mark::MouthCorner, 3.0, -20.0)], extend_mm: 6.0 },
    PathShape { controls: [c(Mark::ChinMiddle, 0.0, 0.0), c(Mark::Middle, 0.0, -0.5), c(Mark::ChinMiddle, 14.0, 2.0)], extend_mm: 4.0 },
    PathShape { controls: [c(Mark::EyeInner, 1.0, -4.5), c(Mark::LowerLid, 0.0, -5.5), c(Mark::EyeOuter, -1.0, -4.5)], extend_mm: 3.0 },
];


/// The sliders of one fold on one side of the face. See the table at the top of this file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoldSide {
    pub strength: f64,
    pub highlight: f64,
    pub width_mm: f64,
    pub edge_mm: f64,
    pub start_mm: f64,
    pub length_mm: f64,
    pub shift_mm: f64,
}

/// The sliders of one fold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fold {
    /// The sliders of the x < 0 side, and of the x > 0 side. When `linked` is true both sides use the first.
    pub sides: [FoldSide; 2],
    pub linked: bool,
    pub variation: f64,
    pub seed: u32,
}

/// The sliders of every fold, in the order of `KINDS`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Folds {
    pub folds: [Fold; FOLD_COUNT],
}

/// The default sliders of each fold at strength 0, which paints nothing. The nose-to-mouth fold's values are those that made the five
/// hand-set test heads read like KOTOR's (`fold_hand`).
pub const DEFAULTS: [FoldSide; FOLD_COUNT] = [
    FoldSide { strength: 0.0, highlight: 0.0, width_mm: 5.0, edge_mm: 1.2, start_mm: 0.0, length_mm: 32.0, shift_mm: 3.0 },
    FoldSide { strength: 0.0, highlight: 0.0, width_mm: 2.2, edge_mm: 2.0, start_mm: 6.0, length_mm: 12.0, shift_mm: 1.4 },
    FoldSide { strength: 0.0, highlight: 0.0, width_mm: 2.6, edge_mm: 2.6, start_mm: 0.0, length_mm: 12.0, shift_mm: 1.4 },
    FoldSide { strength: 0.0, highlight: 0.0, width_mm: 2.0, edge_mm: 1.8, start_mm: 4.0, length_mm: 15.0, shift_mm: 2.1 },
];

/// The slider ranges, as (lowest, highest), in the order of the table at the top of this file: strength, highlight, width, edge, start,
/// length and shift.
pub const RANGES: [(f64, f64); 7] = [(0.0, 30.0), (0.0, 20.0), (0.5, 10.0), (0.2, 5.0), (0.0, 25.0), (3.0, 60.0), (-8.0, 8.0)];

impl FoldSide {
    /// The sliders with every value inside its range.
    pub fn clamped(&self) -> Self {
        let r = RANGES;
        FoldSide {
            strength: self.strength.clamp(r[0].0, r[0].1),
            highlight: self.highlight.clamp(r[1].0, r[1].1),
            width_mm: self.width_mm.clamp(r[2].0, r[2].1),
            edge_mm: self.edge_mm.clamp(r[3].0, r[3].1),
            start_mm: self.start_mm.clamp(r[4].0, r[4].1),
            length_mm: self.length_mm.clamp(r[5].0, r[5].1),
            shift_mm: self.shift_mm.clamp(r[6].0, r[6].1),
        }
    }

    /// Whether the side paints anything.
    pub fn paints(&self) -> bool {
        self.strength != 0.0 || self.highlight != 0.0
    }
}

impl Fold {
    pub fn default_of(kind: FoldKind) -> Self {
        Fold { sides: [DEFAULTS[kind as usize]; 2], linked: true, variation: 0.0, seed: 1 }
    }

    /// The sliders a side is painted with, before the variation.
    pub fn side(&self, side: usize) -> FoldSide {
        if self.linked { self.sides[0] } else { self.sides[side] }
    }
}

impl Folds {
    /// Every fold at its default, which paints nothing.
    pub const NONE: Folds = Folds {
        folds: [
            Fold { sides: [DEFAULTS[0]; 2], linked: true, variation: 0.0, seed: 1 },
            Fold { sides: [DEFAULTS[1]; 2], linked: true, variation: 0.0, seed: 1 },
            Fold { sides: [DEFAULTS[2]; 2], linked: true, variation: 0.0, seed: 1 },
            Fold { sides: [DEFAULTS[3]; 2], linked: true, variation: 0.0, seed: 1 },
        ],
    };

    /// Whether no fold paints anything.
    pub fn is_none(&self) -> bool {
        self.folds.iter().all(|f| (0..2).all(|s| !f.side(s).paints()))
    }
}

/// The exponent of the profile's falloff.
pub const PROFILE_POWER: f64 = 1.6;
/// The profile fades to zero between these numbers of half widths.
const PROFILE_FADE: [f64; 2] = [2.5, 3.5];

/// A falloff from 1 at `d` = 0 to 0 far away, at a distance `d` in millimetres, which is one half at `half_mm`.
pub fn profile(d: f64, half_mm: f64) -> f64 {
    let x = d.abs() / half_mm;
    if x >= PROFILE_FADE[1] {
        return 0.0;
    }
    let w = std::f64::consts::LN_2.powf(1.0 / PROFILE_POWER);
    let v = (-(x * w).powf(PROFILE_POWER)).exp();
    if x <= PROFILE_FADE[0] { v } else { v * crate::wrinkles::smoothstep(PROFILE_FADE[1], PROFILE_FADE[0], x) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small head: a flat face in the plane z = 100 mm, 200 mm wide, with every landmark on a mesh vertex of its own. Returns the
    /// mesh positions in metres and the triangles, which serve as both the mesh and the layout.
    fn flat_face() -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
        let n = 81;
        let mut positions = Vec::new();
        for j in 0..n {
            for i in 0..n {
                positions.push([(-100.0 + 2.5 * i as f64) / 1000.0, (600.0 + 2.5 * j as f64) / 1000.0, 0.1]);
            }
        }
        let mut triangles = Vec::new();
        for j in 0..n - 1 {
            for i in 0..n - 1 {
                let a = (j * n + i) as u32;
                triangles.push([a, a + 1, a + n as u32 + 1]);
                triangles.push([a, a + n as u32 + 1, a + n as u32]);
            }
        }
        (positions, triangles)
    }

    /// Puts every anchor's three vertices at a landmark's place on the flat face: the eyes high up, the nose wings and mouth corners
    /// below them.
    fn place_landmarks(positions: &mut [[f64; 3]]) {
        let a = &ANCHORS;
        let set = |positions: &mut [[f64; 3]], anchor: &Anchor, x: f64, y: f64| {
            for &v in &anchor.vertices {
                positions[v as usize] = [x / 1000.0, y / 1000.0, 0.1];
            }
        };
        for (side, sign) in [(0, -1.0), (1, 1.0)] {
            set(positions, &a.eye_inner[side], sign * 15.0, 760.0);
            set(positions, &a.eye_outer[side], sign * 45.0, 762.0);
            set(positions, &a.lower_lid[side], sign * 30.0, 752.0);
            set(positions, &a.nose_wing[side], sign * 17.0, 712.0);
            set(positions, &a.mouth_corner[side], sign * 25.0, 680.0);
        }
        set(positions, &a.chin_middle, 0.0, 655.0);
        set(positions, &a.lower_lip_middle, 0.0, 668.0);
        set(positions, &a.nose_tip, 0.0, 715.0);
    }

    fn head_with(positions: &[[f64; 3]], triangles: &[[u32; 3]]) -> FoldHead {
        let identity: Vec<usize> = (0..positions.len()).collect();
        FoldHead::new(triangles, &identity, positions).expect("the test head holds the landmarks")
    }

    /// A mesh big enough for the anchors' vertex numbers, holding the flat face's grid and, for every other vertex, a copy of the grid's
    /// first point, which no triangle uses.
    fn test_head() -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
        let (grid, triangles) = flat_face();
        let top = ANCHORS.all().iter().flat_map(|a| a.vertices).max().unwrap() as usize + 1;
        let mut positions = vec![grid[0]; top.max(grid.len())];
        positions[..grid.len()].copy_from_slice(&grid);
        // The anchors' vertices are moved to the landmarks. Those inside the grid would tear it, so the grid is placed after them.
        let offset = top as u32;
        let mut all = positions.clone();
        all.extend_from_slice(&grid);
        let tris: Vec<[u32; 3]> = triangles.iter().map(|t| t.map(|i| i + offset)).collect();
        place_landmarks(&mut all);
        (all, tris)
    }

    #[test]
    fn the_profile_is_one_at_the_centre_half_at_the_half_width_and_zero_far_away() {
        assert_eq!(profile(0.0, 2.0), 1.0);
        assert!((profile(2.0, 2.0) - 0.5).abs() < 1e-12);
        assert!((profile(-2.0, 2.0) - 0.5).abs() < 1e-12);
        assert_eq!(profile(7.0, 2.0), 0.0);
        assert!(profile(1.0, 2.0) > profile(1.5, 2.0));
    }

    #[test]
    fn moving_the_nose_wing_and_mouth_corner_moves_the_path_with_them() {
        let (positions, tris) = test_head();
        let head = head_with(&positions, &tris);
        let before = head.lay(FoldKind::Nasolabial, &PATHS[0], 0);
        // Move the nose wing and the mouth corner on the x < 0 side 3 mm outward and 2 mm down.
        let mut moved = positions.clone();
        for anchor in [ANCHORS.nose_wing[0], ANCHORS.mouth_corner[0]] {
            for &v in &anchor.vertices {
                moved[v as usize][0] -= 0.003;
                moved[v as usize][1] -= 0.002;
            }
        }
        let after = head_with(&moved, &tris).lay(FoldKind::Nasolabial, &PATHS[0], 0);
        let s = before.main_mm.min(after.main_mm);
        for t in [0.0, 0.5, 1.0] {
            let (p, _, _) = before.at(t * s);
            let (q, _, _) = after.at(t * s * after.main_mm / before.main_mm);
            assert!((q[0] - p[0] + 3.0).abs() < 0.3, "x moved by {} at {t}", q[0] - p[0]);
            assert!((q[1] - p[1] + 2.0).abs() < 0.3, "y moved by {} at {t}", q[1] - p[1]);
        }
        // A small move of the landmarks gives a small move of the path.
        let mut nudged = positions.clone();
        for &v in &ANCHORS.mouth_corner[0].vertices {
            nudged[v as usize][0] -= 0.0001;
        }
        let small = head_with(&nudged, &tris).lay(FoldKind::Nasolabial, &PATHS[0], 0);
        for t in [0.2, 0.6, 0.9] {
            let (p, _, _) = before.at(t * before.main_mm);
            let (q, _, _) = small.at(t * small.main_mm);
            assert!(length(sub(p, q)) < 0.2, "a 0.1 mm nudge moved the path by {}", length(sub(p, q)));
        }
    }
}
