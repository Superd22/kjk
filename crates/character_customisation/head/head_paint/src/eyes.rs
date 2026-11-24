//! The eye paint's geometry: where each eye's lid edge lies on the head (paint track, part C, the eye makeup, liner and shadow step).
//!
//! The lid edge of each eye is a loop of 28 vertices of the 4,750-vertex mesh, written by the shape session in
//! `likeness-fit-data/sliders/lid_edges.json`. The indices are the same on every head of the default mesh. Here each loop is split into
//! its upper lid (16 points from the inner to the outer corner) and its lower lid (14 points, the same way), with the two corners shared.
//! The eyeball's centre is the mean of 8 vertices. Its radius is read from the same 8 vertices (`eyeball_radii`), and it is 12.3 mm on
//! every head while the eyeball's size is not a slider.

/// The vertices of one eye's lid edge and eyeball.
pub struct EyeVertices {
    /// The upper lid edge, from the inner corner to the outer corner.
    pub upper: [usize; 16],
    /// The lower lid edge, from the inner corner to the outer corner.
    pub lower: [usize; 14],
    /// The 8 vertices whose mean is the eyeball's centre.
    pub centre: [usize; 8],
}

/// The two eyes: first the eye on the x < 0 side of the head, then the eye on the x > 0 side, as `folds::FaceFrame::out` numbers them.
pub const EYES: [EyeVertices; 2] = [
    EyeVertices {
        upper: [1273, 1264, 1263, 1253, 1256, 1283, 1285, 1250, 1252, 1262, 1272, 1280, 1246, 1249, 1260, 1270],
        lower: [1273, 1281, 1286, 1268, 1265, 1258, 1241, 1238, 2018, 1245, 1242, 1278, 1275, 1270],
        centre: [4742, 4743, 4744, 4745, 4746, 4747, 4748, 4749],
    },
    EyeVertices {
        upper: [3611, 3601, 3602, 3591, 3592, 3621, 3622, 3588, 3589, 3599, 3609, 3617, 3584, 3585, 3597, 3607],
        lower: [3611, 3619, 3624, 3604, 3603, 3595, 3577, 3576, 4341, 3581, 3580, 3614, 3613, 3607],
        centre: [4734, 4735, 4736, 4737, 4738, 4739, 4740, 4741],
    },
];

/// The rim of each eye when the eye's opening is built on the curve (`head_eye_curve`): the third ring of vertices out from the lid edge
/// loop, which the curve lays on the visible edge of the opening while it puts the loop and the two rings after it under the lid. Upper
/// and lower lid from the inner corner to the outer corner, in the order of `EYES`, split where the curve splits them on the base head.
pub struct RimVertices {
    pub upper: &'static [usize],
    pub lower: &'static [usize],
}

pub const RIMS: [RimVertices; 2] = [
    RimVertices {
        upper: &[80, 76, 75, 68, 64, 60, 54, 2, 1, 1203, 1201, 1199, 1195, 1196, 1237, 1233],
        lower: &[80, 84, 1214, 1213, 1218, 1217, 1222, 2022, 1226, 1225, 1230, 1229, 1234, 1233],
    },
    RimVertices {
        upper: &[2486, 2479, 2475, 2471, 2465, 2413, 2414, 3541, 3539, 3537, 3535, 3534, 3575, 3573],
        lower: &[2486, 2485, 2491, 2495, 3552, 3553, 3556, 3557, 3560, 4345, 3564, 3565, 3568, 3569, 3572, 3573],
    },
];

/// Which vertices the eye paint reads each lid's edge from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LidEdge {
    /// The loop of `lid_edges.json` (`EYES`), the visible edge of the opening on a head built without the curve.
    #[default]
    Loop,
    /// The rim (`RIMS`), the visible edge of the opening on a head built on the curve.
    Rim,
}

/// The eyeball's radius in millimetres when its 8 centre vertices lie `EYEBALL_RING_MM` from their mean, as they do on every head of
/// the default mesh before the eyeball's size is a slider. The eye paint and the eyeball's iris were fitted at this radius.
pub const EYEBALL_RADIUS_MM: f64 = 12.3;

/// The root mean square distance of an eye's 8 centre vertices from their mean, in millimetres, on an eyeball of `EYEBALL_RADIUS_MM`.
/// The shape session's eyeball size slider scales that ring about its mean, in proportion to the eyeball.
pub const EYEBALL_RING_MM: f64 = 6.907;

/// The eyeball's radius of each eye in millimetres, in the order of `EYES`, from the head's positions in metres: `EYEBALL_RADIUS_MM`
/// times the ring's root mean square distance from its mean over `EYEBALL_RING_MM`. A head that is not on the default mesh gets
/// `EYEBALL_RADIUS_MM`.
pub fn eyeball_radii(positions: &[[f64; 3]]) -> [f64; 2] {
    if positions.len() != MESH_VERTICES {
        return [EYEBALL_RADIUS_MM; 2];
    }
    [0, 1].map(|s| {
        let ring = EYES[s].centre.map(|i| positions[i].map(|x| 1000.0 * x));
        let mean = [0, 1, 2].map(|k| ring.iter().map(|p| p[k]).sum::<f64>() / 8.0);
        let rms = (ring.iter().map(|p| dot(sub(*p, mean), sub(*p, mean))).sum::<f64>() / 8.0).sqrt();
        // Rounded to a thousandth of a millimetre, so that the rings of today's heads, which lie within 0.00002 mm of the reference,
        // give exactly the reference radius and paint exactly as before.
        (EYEBALL_RADIUS_MM * rms / EYEBALL_RING_MM * 1000.0).round() / 1000.0
    })
}

/// How many vertices a mesh needs for `EYES` to apply.
pub const MESH_VERTICES: usize = 4750;

/// One eye of one head, in millimetres.
#[derive(Clone, Debug, PartialEq)]
pub struct EyeShape {
    pub upper: Vec<[f64; 3]>,
    pub lower: Vec<[f64; 3]>,
    pub centre: [f64; 3],
    /// The eyeball's radius, from `eyeball_radii`.
    pub radius: f64,
}

impl EyeShape {
    pub fn inner(&self) -> [f64; 3] {
        self.upper[0]
    }
    pub fn outer(&self) -> [f64; 3] {
        *self.upper.last().expect("upper lid")
    }
}

/// The two eyes of a head whose mesh is the default mesh, from its positions in metres, with the lids' edges read from the loop. `None`
/// on another mesh.
pub fn eye_shapes(positions: &[[f64; 3]]) -> Option<[EyeShape; 2]> {
    eye_shapes_on(positions, LidEdge::Loop)
}

/// The points of a polyline at `n` even steps of its length, from its first point to its last.
fn resample(points: &[[f64; 3]], n: usize) -> Vec<[f64; 3]> {
    let run: Vec<f64> = shares(points).iter().map(|s| s.1).collect();
    (0..n)
        .map(|i| {
            let t = i as f64 / (n - 1) as f64;
            let k = run.partition_point(|&r| r < t).clamp(1, points.len() - 1);
            let f = ((t - run[k - 1]) / (run[k] - run[k - 1]).max(1e-12)).clamp(0.0, 1.0);
            [0, 1, 2].map(|a| points[k - 1][a] + f * (points[k][a] - points[k - 1][a]))
        })
        .collect()
}

/// The two eyes as `eye_shapes` gives them, with the lids' edges read from `edge`. The rim is resampled to the loop's 16 points of the
/// upper lid and 14 of the lower lid, which the shader takes.
pub fn eye_shapes_on(positions: &[[f64; 3]], edge: LidEdge) -> Option<[EyeShape; 2]> {
    let mut shapes = eye_shapes_of_loop(positions)?;
    if edge == LidEdge::Rim {
        let mm = |i: &usize| positions[*i].map(|x| 1000.0 * x);
        for (shape, rim) in shapes.iter_mut().zip(&RIMS) {
            shape.upper = resample(&rim.upper.iter().map(mm).collect::<Vec<_>>(), EYES[0].upper.len());
            shape.lower = resample(&rim.lower.iter().map(mm).collect::<Vec<_>>(), EYES[0].lower.len());
        }
    }
    Some(shapes)
}

fn eye_shapes_of_loop(positions: &[[f64; 3]]) -> Option<[EyeShape; 2]> {
    if positions.len() != MESH_VERTICES {
        return None;
    }
    let mm = |i: usize| positions[i].map(|x| 1000.0 * x);
    let radii = eyeball_radii(positions);
    Some([0, 1].map(|s| {
        let e = &EYES[s];
        let mut centre = [0.0; 3];
        for &i in &e.centre {
            let p = mm(i);
            for k in 0..3 {
                centre[k] += p[k] / 8.0;
            }
        }
        EyeShape { upper: e.upper.iter().map(|&i| mm(i)).collect(), lower: e.lower.iter().map(|&i| mm(i)).collect(), centre, radius: radii[s] }
    }))
}

/// The settings of the eye paint. Sizes are in millimetres on the head, darkness in L*, and colours as a lightness (L*), a hue in degrees
/// and a chroma. Both eyes take the same settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EyePaint {
    /// How dark the band of upper lashes along the upper lid edge is, from 0 (none) to 1 (fully the lash colour).
    pub upper_lash: f64,
    /// The width of that band above the edge at its widest, in its outer half.
    pub upper_lash_mm: f64,
    /// How dark the band of lower lashes along the lower lid edge is, from 0 to 1.
    pub lower_lash: f64,
    /// The width of that band below the edge at its widest.
    pub lower_lash_mm: f64,
    /// How opaque the liner is, from 0 (none) to 1.
    pub liner: f64,
    /// The width of the liner above the upper lid edge, at the outer corner.
    pub liner_mm: f64,
    /// How far the liner's wing reaches past the outer corner.
    pub wing_mm: f64,
    /// The wing's angle above the line of the upper lid at the outer corner.
    pub wing_degrees: f64,
    /// How much the liner also lines the lower lid, from 0 to 1.
    pub liner_lower: f64,
    pub liner_colour: [f64; 3],
    /// How much darker the crease above the upper lid is, at its middle.
    pub crease: f64,
    /// How far above the upper lid edge the crease runs.
    pub crease_mm: f64,
    /// How much darker the socket under the brow is, next to the upper lid.
    pub socket: f64,
    /// How much darker the skin under the eye is, at its darkest.
    pub under_eye: f64,
    /// How red the rims of the lids are, mostly the lower lid's, from 0 to 1.
    pub rims: f64,
    /// How red the skin of both lids is, from 0 to 1.
    pub lid_red: f64,
    /// How opaque the eye shadow is, from 0 (none) to 1.
    pub shadow: f64,
    /// How far above the upper lid edge the eye shadow reaches.
    pub shadow_mm: f64,
    pub shadow_colour: [f64; 3],
    /// How much lighter the skin of the lids is, in L*, negative for darker lids.
    pub lid_light: f64,
    /// How deep the crease under the bags below the eye is, in L*.
    pub bags: f64,
    /// How far below the lower lid's edge that crease runs, in millimetres.
    pub bags_mm: f64,
    /// The eyeball's iris, pupil and eye white (`eyeball.rs`). They are painted whether the rest of the eye paint is on or not.
    pub ball: crate::eyeball::Eyeball,
}

impl EyePaint {
    /// No eye paint at all.
    pub const NONE: EyePaint = EyePaint {
        upper_lash: 0.0,
        upper_lash_mm: 1.0,
        lower_lash: 0.0,
        lower_lash_mm: 0.6,
        liner: 0.0,
        liner_mm: 1.5,
        wing_mm: 0.0,
        wing_degrees: 15.0,
        liner_lower: 0.0,
        liner_colour: [12.0, 30.0, 4.0],
        crease: 0.0,
        crease_mm: 7.0,
        socket: 0.0,
        under_eye: 0.0,
        rims: 0.0,
        lid_red: 0.0,
        shadow: 0.0,
        shadow_mm: 7.0,
        shadow_colour: [35.0, 320.0, 15.0],
        lid_light: 0.0,
        bags: 0.0,
        bags_mm: 7.0,
        ball: crate::eyeball::Eyeball::DEFAULT,
    };

    /// The eye paint of a head whose sliders were not set: dark lash lines, a faint crease and socket, no makeup.
    pub const DEFAULT: EyePaint = EyePaint { upper_lash: 0.85, upper_lash_mm: 1.0, lower_lash: 0.35, lower_lash_mm: 0.6, crease: 4.0, socket: 6.0, under_eye: 4.0, rims: 0.3, ..EyePaint::NONE };

    pub fn is_none(&self) -> bool {
        self.upper_lash <= 0.0 && self.lower_lash <= 0.0 && self.liner <= 0.0 && self.crease <= 0.0 && self.socket <= 0.0 && self.under_eye <= 0.0 && self.rims <= 0.0 && self.lid_red <= 0.0 && self.shadow <= 0.0 && self.lid_light == 0.0 && self.bags <= 0.0
    }
}

/// A colour given as lightness, hue in degrees and chroma, in linear sRGB.
pub fn lch_to_linear(lch: [f64; 3]) -> [f64; 3] {
    let h = lch[1].to_radians();
    let srgb = crate::colour::lab_to_srgb([lch[0], lch[2] * h.cos(), lch[2] * h.sin()]);
    srgb.map(crate::pigment::decode)
}

/// What the eye paint's shader reads of one eye: the lid edges with the share along each lid, and the liner's wing with its half width,
/// all in millimetres.
#[derive(Clone, Debug, PartialEq)]
pub struct EyeLines {
    pub centre: [f64; 3],
    /// The eyeball's radius.
    pub radius: f64,
    /// The direction toward the temple, in the face's plane.
    pub out: [f64; 3],
    pub up: [f64; 3],
    /// The upper lid edge from the inner to the outer corner: a point and its share of the lid's length.
    pub upper: Vec<([f64; 3], f64)>,
    pub lower: Vec<([f64; 3], f64)>,
    /// The wing's centre line from the outer corner to the tip, as seen from the front: a point of the face's plane (across, up, 0) in
    /// millimetres from the eyeball's centre, and the half width there. Every half width is zero when there is no wing.
    pub wing: Vec<([f64; 3], f64)>,
}

/// How many points the wing's centre line has.
pub const WING_POINTS: usize = 8;

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn shares(points: &[[f64; 3]]) -> Vec<([f64; 3], f64)> {
    let mut run = vec![0.0];
    for w in points.windows(2) {
        run.push(run.last().unwrap() + dot(sub(w[1], w[0]), sub(w[1], w[0])).sqrt());
    }
    let total = run.last().copied().unwrap_or(1.0).max(1e-9);
    points.iter().zip(run).map(|(p, r)| (*p, r / total)).collect()
}

impl EyeLines {
    /// The lines of one eye. `side` is 0 for the eye on the x < 0 side, and `frame` is the face's frame.
    pub fn new(paint: &EyePaint, eye: &EyeShape, side: usize, frame: &crate::folds::FaceFrame) -> Self {
        let out = frame.out(side);
        let up = frame.up;
        let mut wing = vec![([0.0; 3], 0.0); WING_POINTS];
        if paint.liner > 0.0 && paint.wing_mm > 0.0 && paint.liner_mm > 0.0 {
            // The wing is drawn as a viewer in front of the face sees it: a straight line in the face's plane that leaves the outer corner
            // at its angle above the line from the inner to the outer corner. Its points are given in the plane, as (across, up, 0).
            let plane = |p: [f64; 3]| (dot(sub(p, eye.centre), out), dot(sub(p, eye.centre), up));
            let axis = sub(eye.outer(), eye.inner());
            let base = dot(axis, up).atan2(dot(axis, out)) + paint.wing_degrees.to_radians();
            // It starts 2 mm before the corner, so that it joins the liner over the fold of the corner.
            let (ox, oy) = plane(eye.outer());
            let (cx, cy) = (ox - 2.0 * base.cos(), oy - 2.0 * base.sin());
            let width = 0.55 * paint.liner_mm;
            for (k, w) in wing.iter_mut().enumerate() {
                let t = k as f64 / (WING_POINTS - 1) as f64;
                let x = cx + t * (paint.wing_mm + 2.0) * base.cos();
                let y = cy + 0.5 * width + t * (paint.wing_mm + 2.0) * base.sin();
                *w = ([x, y, 0.0], width * (1.0 - t) + 0.08 * t);
            }
        }
        EyeLines { centre: eye.centre, radius: eye.radius, out, up, upper: shares(&eye.upper), lower: shares(&eye.lower), wing }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::folds::FaceFrame;

    const FRAME: FaceFrame = FaceFrame { across: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], forward: [0.0, 0.0, 1.0] };

    /// An eye 26 mm wide on the x > 0 side, with lids that arch 4 mm up and 2.5 mm down.
    fn eye(shift: f64) -> EyeShape {
        let arc = |n: usize, h: f64| (0..n).map(|i| {
            let t = i as f64 / (n - 1) as f64;
            [32.0 - 13.0 + 26.0 * t + shift, h * (std::f64::consts::PI * t).sin(), 10.0]
        }).collect::<Vec<_>>();
        EyeShape { upper: arc(16, 4.0), lower: arc(14, -2.5), centre: [32.0, 0.0, 0.0], radius: EYEBALL_RADIUS_MM }
    }

    /// A head whose eyes have their centre vertices `EYEBALL_RING_MM` times `scale` from each eyeball's centre, and every lid vertex
    /// 0.5 mm plus `EYEBALL_RADIUS_MM` times `scale` from it, in metres.
    fn head_with_balls(scale: f64) -> Vec<[f64; 3]> {
        let mut positions = vec![[0.0; 3]; MESH_VERTICES];
        for (s, e) in EYES.iter().enumerate() {
            let centre = [if s == 0 { -32.0 } else { 32.0 }, 40.0, 60.0];
            let at = |dir: [f64; 3], mm: f64| {
                let l = dot(dir, dir).sqrt();
                [0, 1, 2].map(|k| (centre[k] + mm * dir[k] / l) / 1000.0)
            };
            for (j, &i) in e.centre.iter().enumerate() {
                let dir = [if j & 1 == 0 { 1.0 } else { -1.0 }, if j & 2 == 0 { 1.0 } else { -1.0 }, if j & 4 == 0 { 1.0 } else { -1.0 }];
                positions[i] = at(dir, EYEBALL_RING_MM * scale);
            }
            for (j, &i) in e.upper.iter().chain(&e.lower).enumerate() {
                let t = j as f64 / 30.0;
                positions[i] = at([(t * 6.0).cos(), (t * 6.0).sin(), 1.6], 0.5 + EYEBALL_RADIUS_MM * scale);
            }
        }
        positions
    }

    #[test]
    fn the_eyeball_radius_follows_the_ring() {
        assert_eq!(eyeball_radii(&head_with_balls(1.0)), [EYEBALL_RADIUS_MM; 2]);
        assert_eq!(eyeball_radii(&vec![[0.0; 3]; 100]), [EYEBALL_RADIUS_MM; 2]);
        // The eyeball size slider scales the ring about its mean, and moves the skin round the eye outward with the eyeball's surface.
        let scale = 13.5 / EYEBALL_RADIUS_MM;
        let (small, large) = (eye_shapes(&head_with_balls(1.0)).unwrap(), eye_shapes(&head_with_balls(scale)).unwrap());
        assert_eq!(eyeball_radii(&head_with_balls(scale)), [13.5; 2]);
        let frame = crate::folds::FaceFrame { across: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], forward: [0.0, 0.0, 1.0] };
        for side in 0..2 {
            let a = EyeLines::new(&EyePaint::DEFAULT, &small[side], side, &frame);
            let b = EyeLines::new(&EyePaint::DEFAULT, &large[side], side, &frame);
            assert!(dot(sub(a.centre, b.centre), sub(a.centre, b.centre)).sqrt() < 1e-9);
            assert_eq!((a.radius, b.radius), (EYEBALL_RADIUS_MM, 13.5));
            // What the eye paint measures from the eyeball's surface: each lid point's height above it, the same 0.5 mm on both heads.
            for lines in [&a, &b] {
                for (p, _) in lines.upper.iter().chain(&lines.lower) {
                    let gap = dot(sub(*p, lines.centre), sub(*p, lines.centre)).sqrt() - lines.radius;
                    assert!((gap - 0.5).abs() < 1e-6, "{gap}");
                }
            }
        }
    }

    #[test]
    fn the_rim_keeps_the_loop_counts_and_its_own_ends() {
        let positions: Vec<[f64; 3]> = (0..MESH_VERTICES).map(|i| [i as f64 * 1e-5, (i % 7) as f64 * 1e-4, 0.0]).collect();
        let (looped, rim) = (eye_shapes(&positions).unwrap(), eye_shapes_on(&positions, LidEdge::Rim).unwrap());
        for s in 0..2 {
            assert_eq!((rim[s].upper.len(), rim[s].lower.len()), (looped[s].upper.len(), looped[s].lower.len()));
            assert_eq!((rim[s].centre, rim[s].radius), (looped[s].centre, looped[s].radius));
            let mm = |i: usize| positions[i].map(|x| 1000.0 * x);
            let r = &RIMS[s];
            for (got, want) in [(rim[s].upper[0], r.upper[0]), (*rim[s].upper.last().unwrap(), *r.upper.last().unwrap()), (rim[s].lower[0], r.lower[0]), (*rim[s].lower.last().unwrap(), *r.lower.last().unwrap())] {
                assert!(dot(sub(got, mm(want)), sub(got, mm(want))).sqrt() < 1e-9);
            }
            assert_eq!((r.upper[0], r.upper.last()), (r.lower[0], r.lower.last()));
        }
    }

    #[test]
    fn another_mesh_has_no_eyes() {
        assert!(eye_shapes(&vec![[0.0; 3]; 100]).is_none());
        assert!(eye_shapes(&vec![[0.0; 3]; MESH_VERTICES]).is_some());
    }

    #[test]
    fn no_liner_draws_no_wing() {
        let lines = EyeLines::new(&EyePaint { liner: 0.0, wing_mm: 10.0, ..EyePaint::DEFAULT }, &eye(0.0), 1, &FRAME);
        assert!(lines.wing.iter().all(|w| w.1 == 0.0));
        let lines = EyeLines::new(&EyePaint { liner: 1.0, wing_mm: 10.0, ..EyePaint::DEFAULT }, &eye(0.0), 1, &FRAME);
        assert!(lines.wing.iter().all(|w| w.1 > 0.0));
    }

    #[test]
    fn a_small_change_of_the_head_gives_a_small_change_of_the_lines() {
        let paint = EyePaint { liner: 1.0, wing_mm: 10.0, ..EyePaint::DEFAULT };
        let a = EyeLines::new(&paint, &eye(0.0), 1, &FRAME);
        let b = EyeLines::new(&paint, &eye(0.1), 1, &FRAME);
        for (x, y) in a.upper.iter().zip(&b.upper).chain(a.lower.iter().zip(&b.lower)) {
            assert!(dot(sub(x.0, y.0), sub(x.0, y.0)).sqrt() < 0.2 && (x.1 - y.1).abs() < 1e-9);
        }
        for (x, y) in a.wing.iter().zip(&b.wing) {
            assert!(dot(sub(x.0, y.0), sub(x.0, y.0)).sqrt() < 0.2);
        }
    }

    #[test]
    fn the_wing_leaves_the_corner_at_its_angle() {
        let lines = EyeLines::new(&EyePaint { liner: 1.0, wing_mm: 10.0, wing_degrees: 30.0, ..EyePaint::DEFAULT }, &eye(0.0), 1, &FRAME);
        let (a, b) = (lines.wing[0].0, lines.wing[WING_POINTS - 1].0);
        let angle = (b[1] - a[1]).atan2(b[0] - a[0]).to_degrees();
        assert!((angle - 30.0).abs() < 1e-6, "{angle}");
    }
}
