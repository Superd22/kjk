//! What a nose is: proportions of the face and outlines, measured on any triangle mesh in the same way, whether it is one of KOTOR's
//! low faceted heads or one of ours.
//!
//! Everything is measured in the frame of the eyes. The origin is midway between the eyes' centres, x runs along the line between
//! them, y points up and z out of the face. Lengths are given in millimetres on a face whose eyes are 63 mm apart, so a head's nose is
//! measured against its own face. Our heads and the KOTOR heads they copy share their eyes, so the frame is the same for both.
//!
//! The nose is read from cuts through the surface: the cut down the middle of the face (the side profile), and level cuts across the
//! nose. Each cut is read as the front-most surface at each place, sampled every 0.25 mm and lightly blurred, so the facets of a low
//! mesh do not move the landmarks.
//!
//! The landmarks are:
//! - the tip: the front-most point of the side profile, at the middle height of the part within 1 mm of the front;
//! - the base (subnasale): where the underside of the nose meets the upper lip, the deepest point below the line from the tip to the
//!   front of the upper lip;
//! - the wings: in each level cut through the wings, the steepest place of each side, where the wing turns away from a viewer in
//!   front. The widest such pair between the base and the tip marks the wings.

use crate::{V3, cross, dot, normalize, scale, sub};

pub const EYES_APART_MM: f64 = 63.0;
pub const STEP: f64 = 0.25;

/// The proportions, in this order. Each is in millimetres on a face whose eyes are 63 mm apart, except the tip's turn, in degrees.
pub const PROPORTIONS: [Proportion; 14] = [
    Proportion {
        key: "width_wings",
        label: "Width at the wings",
        low: "narrow",
        high: "wide",
        unit: "mm",
    },
    Proportion {
        key: "width_bridge",
        label: "Width of the bridge",
        low: "thin bridge",
        high: "broad bridge",
        unit: "mm",
    },
    Proportion {
        key: "width_tip",
        label: "Width of the tip",
        low: "fine tip",
        high: "round tip",
        unit: "mm",
    },
    Proportion {
        key: "length",
        label: "Length (eyes to nose base)",
        low: "short",
        high: "long",
        unit: "mm",
    },
    Proportion {
        key: "projection",
        label: "How far the tip stands out",
        low: "flat",
        high: "projecting",
        unit: "mm",
    },
    Proportion {
        key: "root",
        label: "Height of the bridge between the eyes",
        low: "low bridge",
        high: "high bridge",
        unit: "mm",
    },
    Proportion {
        key: "tip_turn",
        label: "Turn of the tip",
        low: "turned down",
        high: "turned up",
        unit: "degrees",
    },
    Proportion {
        key: "dorsum",
        label: "Line of the bridge",
        low: "scooped",
        high: "humped",
        unit: "mm",
    },
    Proportion {
        key: "flare",
        label: "Flare of the wings",
        low: "wings in line with their base",
        high: "flared wings",
        unit: "mm",
    },
    Proportion {
        key: "bridge_height",
        label: "How far the middle of the bridge stands out",
        low: "flat bridge",
        high: "high bridge",
        unit: "mm",
    },
    Proportion {
        key: "round_profile",
        label: "Roundness of the tip from the side",
        low: "sharp tip",
        high: "round tip",
        unit: "mm",
    },
    Proportion {
        key: "round_below",
        label: "Bluntness of the tip from below",
        low: "pointed tip",
        high: "blunt tip",
        unit: "mm",
    },
    Proportion {
        key: "root_dip",
        label: "Dip between the brow and the bridge",
        low: "straight from brow to bridge",
        high: "deep dip",
        unit: "mm",
    },
    Proportion {
        key: "bend",
        label: "Where the bridge bulges, seen from the side",
        low: "near the eyes",
        high: "near the tip",
        unit: "mm",
    },
];
pub const N_PROPORTIONS: usize = PROPORTIONS.len();

/// The outlines, in this order.
pub const OUTLINES: [Outline; 3] = [
    Outline {
        key: "profile",
        label: "Side profile",
        points: 40,
    },
    Outline {
        key: "front",
        label: "Outline from the front",
        points: 20,
    },
    Outline {
        key: "base",
        label: "Base seen from below",
        points: 31,
    },
];

pub struct Proportion {
    pub key: &'static str,
    pub label: &'static str,
    pub low: &'static str,
    pub high: &'static str,
    pub unit: &'static str,
}

pub struct Outline {
    pub key: &'static str,
    pub label: &'static str,
    pub points: usize,
}

/// The base head's eyeballs' centres, in metres in the fit's frame, the +x eye first.
pub const BASE_EYES_M: [V3; 2] = [
    [0.029543484810493445, 0.7303516981330398, 0.11696164284455376],
    [-0.029459976005412943, 0.7304376213611672, 0.11543812671549593],
];

/// The frame a head is measured in: `Frame::head()`, or the frame of the head's own eyes when `SLIDERS_FRAME=eyes` asks for the frame
/// used before stage A of the eyes.
pub fn head_frame(eyes: [V3; 2]) -> Frame {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var("SLIDERS_FRAME").is_ok_and(|v| v == "eyes") {
        return Frame::from_eyes(eyes);
    }
    let _ = eyes;
    Frame::head()
}

/// The frame of the eyes.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub origin: V3,
    pub axes: [V3; 3],
    /// Millimetres on the reference face per metre of the head.
    pub scale: f64,
}

impl Frame {
    /// `eyes` are the eyes' centres in metres, in a frame where y points up and the face looks along +z.
    pub fn from_eyes(eyes: [V3; 2]) -> Self {
        let origin = scale(crate::add(eyes[0], eyes[1]), 0.5);
        let across = sub(eyes[0], eyes[1]);
        let x = normalize(across);
        let up = [0.0, 1.0, 0.0];
        let y = normalize(sub(up, scale(x, dot(up, x))));
        let mut z = cross(x, y);
        if z[2] < 0.0 {
            z = scale(z, -1.0);
        }
        let apart = crate::length(across);
        Self {
            origin,
            axes: [x, y, z],
            scale: 1000.0 * EYES_APART_MM / (apart * 1000.0),
        }
    }

    /// The frame every head is measured in since stage A of the eyes: the base head's frame of the eyes, the same for every head.
    /// KOTOR's heads share one head bone, so their faces sit in one space; reading a frame from skull landmarks on the fits wobbled
    /// more between two fits of one head (1.5 mm) than it varied between heads (1.3 mm). Nothing a slider does moves this frame.
    pub fn head() -> Self {
        Self::from_eyes(BASE_EYES_M)
    }

    pub fn local(&self, p: V3) -> V3 {
        let d = sub(p, self.origin);
        self.axes.map(|a| dot(d, a) * self.scale)
    }

    pub fn world(&self, q: V3) -> V3 {
        let mut p = self.origin;
        for (a, v) in self.axes.iter().zip(q) {
            p = crate::add(p, scale(*a, v / self.scale));
        }
        p
    }
}

/// A box in the frame of the eyes (mm): |x| below `x`, y between `y[0]` and `y[1]`, z above `z`. A surface keeps the triangles that reach
/// into it.
#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub x: f64,
    pub y: [f64; 2],
    pub z: f64,
}

impl Window {
    pub fn contains(&self, p: V3) -> bool {
        p[0].abs() < self.x && p[1] > self.y[0] && p[1] < self.y[1] && p[2] > self.z
    }
}

/// The nose's window.
pub const NOSE_WINDOW: Window = Window { x: 45.0, y: [-110.0, 35.0], z: -25.0 };

/// The part of a surface near a feature, in the frame of the eyes.
pub struct Surface {
    pub triangles: Vec<[V3; 3]>,
    /// For each millimetre of height from `from`, the triangles that reach into it, so a level cut only looks at those.
    bins: Vec<Vec<u32>>,
    from: f64,
    /// The frame's millimetres per metre of the head, when the surface was made from a head and its frame (NaN otherwise). The eyes
    /// use it for the eyeball, whose size is fixed in metres.
    pub scale: f64,
    /// The eyeballs' centres in the frame (mm), the +x eye first, when the surface was made from our head mesh, whose eyeball-centre
    /// rings (`OUR_RINGS`) are in no triangle.
    pub eyes: Option<[V3; 2]>,
    /// Triangles in the frame that hide the eyeball but are not skin (our generated upper lid pieces), read by the eyes' measure only.
    pub lids: Vec<[V3; 3]>,
}

/// The eyeball-centre rings of our head mesh of 4,750 vertices: 8 vertices for the +x eye, then 8 for the -x eye.
pub const OUR_RINGS: [std::ops::Range<usize>; 2] = [4734..4742, 4742..4750];
pub const OUR_VERTEX_COUNT: usize = 4750;

/// The eyeballs' centres of our head (positions in metres), the means of the rings, or None for any other mesh.
pub fn our_eyes(positions: &[V3]) -> Option<[V3; 2]> {
    (positions.len() == OUR_VERTEX_COUNT).then(|| {
        OUR_RINGS.map(|r| {
            let n = r.len() as f64;
            positions[r].iter().fold([0.0; 3], |a, &p| crate::add(a, scale(p, 1.0 / n)))
        })
    })
}

impl Surface {
    /// The surface near the nose.
    pub fn new(positions: &[V3], triangles: &[[u32; 3]], frame: &Frame) -> Self {
        Self::within(positions, triangles, frame, &NOSE_WINDOW)
    }

    pub fn within(positions: &[V3], triangles: &[[u32; 3]], frame: &Frame, window: &Window) -> Self {
        let local: Vec<V3> = positions.iter().map(|&p| frame.local(p)).collect();
        let triangles = triangles
            .iter()
            .map(|t| t.map(|i| local[i as usize]))
            .filter(|t| t.iter().any(|&p| window.contains(p)))
            .collect();
        let eyes = our_eyes(positions).map(|e| e.map(|p| frame.local(p)));
        Self { scale: frame.scale, eyes, ..Self::from_local_in(triangles, window) }
    }

    /// A surface near the nose given by its triangles in the frame of the eyes.
    pub fn from_local(triangles: Vec<[V3; 3]>) -> Self {
        Self::from_local_in(triangles, &NOSE_WINDOW)
    }

    /// A surface given by its triangles in the frame of the eyes, binned by height over `window` and 10 mm more on each side.
    pub fn from_local_in(triangles: Vec<[V3; 3]>, window: &Window) -> Self {
        let from = window.y[0] - 10.0;
        let count = (window.y[1] - window.y[0]) as usize + 25;
        let mut bins = vec![Vec::new(); count];
        for (k, t) in triangles.iter().enumerate() {
            let lo = t.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
            let hi = t.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
            let a = ((lo - from).floor().max(0.0) as usize).min(count - 1);
            let b = ((hi - from).floor().max(0.0) as usize).min(count - 1);
            for bin in &mut bins[a..=b] {
                bin.push(k as u32);
            }
        }
        Self { triangles, bins, from, scale: f64::NAN, eyes: None, lids: Vec::new() }
    }

    /// The cut by the plane where coordinate `axis` equals `at`, as segments in the coordinate `along` and depth (z).
    pub fn cut(&self, axis: usize, at: f64, along: usize) -> Vec<[[f64; 2]; 2]> {
        let mut out = Vec::new();
        let mut visit = |t: &[V3; 3]| {
            let d = t.map(|p| {
                let v = p[axis] - at;
                if v == 0.0 { 1e-9 } else { v }
            });
            let mut ends = Vec::with_capacity(2);
            for (i, j) in [(0, 1), (1, 2), (2, 0)] {
                if (d[i] > 0.0) != (d[j] > 0.0) {
                    let s = d[i] / (d[i] - d[j]);
                    ends.push([
                        t[i][along] + s * (t[j][along] - t[i][along]),
                        t[i][2] + s * (t[j][2] - t[i][2]),
                    ]);
                }
            }
            if ends.len() == 2 {
                out.push([ends[0], ends[1]]);
            }
        };
        if axis == 1 {
            for &k in &self.bins[((at - self.from).floor().max(0.0) as usize).min(self.bins.len() - 1)] {
                visit(&self.triangles[k as usize]);
            }
        } else {
            self.triangles.iter().for_each(visit);
        }
        out
    }

    /// The front-most depth of the cut at each sample from `from` in steps of 0.25 mm, NaN where the cut has no surface.
    pub fn front(&self, axis: usize, at: f64, along: usize, from: f64, count: usize) -> Vec<f64> {
        let mut z = vec![f64::NAN; count];
        for [a, b] in self.cut(axis, at, along) {
            let (lo, hi) = if a[0] < b[0] { (a, b) } else { (b, a) };
            let k0 = ((lo[0] - from) / STEP).ceil().max(0.0) as usize;
            let k1 = (((hi[0] - from) / STEP).floor() as isize).min(count as isize - 1);
            if k1 < 0 {
                continue;
            }
            for (k, zk) in z.iter_mut().enumerate().take(k1 as usize + 1).skip(k0) {
                let u = from + k as f64 * STEP;
                let s = if hi[0] > lo[0] {
                    (u - lo[0]) / (hi[0] - lo[0])
                } else {
                    0.0
                };
                let v = lo[1] + s * (hi[1] - lo[1]);
                if zk.is_nan() || v > *zk {
                    *zk = v;
                }
            }
        }
        z
    }
}

/// A sampled cut: the depth every 0.25 mm from `from`.
pub struct Cut {
    pub from: f64,
    pub z: Vec<f64>,
}

impl Cut {
    pub fn at(&self, u: f64) -> f64 {
        let k = (u - self.from) / STEP;
        if k < 0.0 || k > (self.z.len() - 1) as f64 {
            return f64::NAN;
        }
        let i = k.floor() as usize;
        let j = (i + 1).min(self.z.len() - 1);
        let s = k - i as f64;
        self.z[i] * (1.0 - s) + self.z[j] * s
    }

    pub fn u(&self, k: usize) -> f64 {
        self.from + k as f64 * STEP
    }

    pub fn index(&self, u: f64) -> usize {
        (((u - self.from) / STEP).round().max(0.0) as usize).min(self.z.len() - 1)
    }

    /// The cut blurred by a Gaussian of `sigma` mm, skipping places with no surface.
    pub fn blurred(&self, sigma: f64) -> Cut {
        let r = (3.0 * sigma / STEP).ceil() as isize;
        let w: Vec<f64> = (-r..=r)
            .map(|k| (-0.5 * (k as f64 * STEP / sigma).powi(2)).exp())
            .collect();
        let n = self.z.len() as isize;
        let z = (0..n)
            .map(|i| {
                if self.z[i as usize].is_nan() {
                    return f64::NAN;
                }
                let (mut s, mut ws) = (0.0, 0.0);
                for k in -r..=r {
                    let j = i + k;
                    if j >= 0 && j < n && !self.z[j as usize].is_nan() {
                        s += w[(k + r) as usize] * self.z[j as usize];
                        ws += w[(k + r) as usize];
                    }
                }
                s / ws
            })
            .collect();
        Cut { from: self.from, z }
    }

    pub fn slope(&self, k: usize) -> f64 {
        if k == 0 || k + 1 >= self.z.len() {
            return f64::NAN;
        }
        (self.z[k + 1] - self.z[k - 1]) / (2.0 * STEP)
    }
}

/// One nose, measured.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Nose {
    pub proportions: [f64; N_PROPORTIONS],
    /// The outlines in the frame of the eyes (mm): the profile as (y, z) from 4 mm above the eyes to the base; the front as (half the
    /// width 4 mm behind the front, share of the height) from 15% of the way from the eyes to the tip down to 3 mm above the tip; the
    /// base seen from below as (x, z) across the wings.
    pub outlines: [Vec<[f64; 2]>; 3],
    pub marks: Marks,
    /// The cut down the middle as (y, z) and the level cut through the wings as (x, z), every 0.5 mm, for drawing.
    pub cuts: [Vec<[f64; 2]>; 2],
}

/// The landmarks, in the frame of the eyes (mm), for drawing.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Marks {
    pub tip: V3,
    pub base: V3,
    pub wings: [V3; 2],
    pub bridge: [V3; 2],
}

// A wing's side is steeper than 1 in 1 (45 degrees) where it turns away from a viewer in front; lower slopes are cheek or lip.
const WING_STEEPNESS: f64 = 1.0;
// The widths of the bridge, the tip and the outline from the front are taken this far behind the front (mm).
const DEPTH: f64 = 4.0;
// The tip's roundness and bluntness are read on the part of the tip within this depth of its front (mm).
const CAP_DEPTH: f64 = 2.5;
// The nose's base is looked for at most this far below the tip (mm). Farther down lies the lip, and a face whose upper lip stands
// out had its lip taken for the base.
const BASE_REACH: f64 = 25.0;
// The brow is read on the profile this far above the eyes (mm).
const BROW_Y: f64 = 6.0;
// A parabola's width at half the depth is this share of its width at the full depth.
const CAP_RATIO: f64 = std::f64::consts::FRAC_1_SQRT_2;
const MIDLINE_FROM: f64 = -100.0;
const MIDLINE_COUNT: usize = 500;
const ACROSS_FROM: f64 = -38.0;
const ACROSS_COUNT: usize = 305;

fn across(surface: &Surface, y: f64) -> Cut {
    Cut {
        from: ACROSS_FROM,
        z: surface.front(1, y, 0, ACROSS_FROM, ACROSS_COUNT),
    }
}

/// The steepest place of each side of the nose in a level cut, as (x, z), searching from 1 mm to `reach` mm from the middle. Seen
/// from the front, this is where the side of the nose turns away from the viewer, the edge of the nose's outline. None when a side's
/// steepest slope is below `steep` (depth per width).
fn sides(cut: &Cut, reach: f64, steep: f64) -> Option<[[f64; 2]; 2]> {
    let c = cut.blurred(1.0);
    let mut out = [[0.0; 2]; 2];
    for (side, sign) in [(0, -1.0), (1, 1.0)] {
        let mut best = (f64::NEG_INFINITY, 0usize);
        for k in 1..c.z.len() - 1 {
            let x = c.u(k) * sign;
            if !(1.0..=reach).contains(&x) {
                continue;
            }
            let s = -c.slope(k) * sign;
            if s.is_nan() {
                return None;
            }
            if s > best.0 {
                best = (s, k);
            }
        }
        if best.0 < steep {
            return None;
        }
        out[side] = [c.u(best.1), c.z[best.1]];
    }
    Some(out)
}

/// The width of a level cut `depth` mm behind its front-most point near the middle, and the two ends.
pub fn width_at_depth(cut: &Cut, depth: f64) -> Option<(f64, [f64; 2])> {
    let c = cut.blurred(0.5);
    let mid = c.index(0.0);
    let reach = (4.0 / STEP) as usize;
    let top = (mid.saturating_sub(reach)..=(mid + reach).min(c.z.len() - 1))
        .filter(|&k| !c.z[k].is_nan())
        .max_by(|&a, &b| c.z[a].total_cmp(&c.z[b]))?;
    let level = c.z[top] - depth;
    let mut ends = [0.0; 2];
    for (side, dir) in [(0usize, -1isize), (1, 1)] {
        let mut k = top as isize;
        loop {
            let next = k + dir;
            if next < 0 || next >= c.z.len() as isize || c.z[next as usize].is_nan() {
                return None;
            }
            if c.z[next as usize] < level {
                let (a, b) = (c.z[k as usize], c.z[next as usize]);
                let s = (a - level) / (a - b);
                ends[side] = c.u(k as usize) + s * dir as f64 * STEP;
                break;
            }
            k = next;
        }
    }
    Some((ends[1] - ends[0], ends))
}

/// `points` evenly spaced along a polyline.
pub fn resample(line: &[[f64; 2]], points: usize) -> Vec<[f64; 2]> {
    let mut acc = vec![0.0];
    for w in line.windows(2) {
        acc.push(
            acc.last().unwrap()
                + ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt(),
        );
    }
    let total = *acc.last().unwrap();
    let mut out = Vec::with_capacity(points);
    let mut j = 0;
    for i in 0..points {
        let s = total * i as f64 / (points - 1) as f64;
        while j + 2 < acc.len() && acc[j + 1] < s {
            j += 1;
        }
        let span = (acc[j + 1] - acc[j]).max(1e-12);
        let t = ((s - acc[j]) / span).clamp(0.0, 1.0);
        out.push([
            line[j][0] + t * (line[j + 1][0] - line[j][0]),
            line[j][1] + t * (line[j + 1][1] - line[j][1]),
        ]);
    }
    out
}

/// Measures the nose of a surface, or says which landmark could not be found.
pub fn measure(surface: &Surface) -> Result<Nose, &'static str> {
    let raw = Cut {
        from: MIDLINE_FROM,
        z: surface.front(0, 0.0, 1, MIDLINE_FROM, MIDLINE_COUNT),
    };
    let mid = raw.blurred(0.75);

    // the tip
    let tip_k = (mid.index(-85.0)..=mid.index(-20.0))
        .filter(|&k| !mid.z[k].is_nan())
        .max_by(|&a, &b| mid.z[a].total_cmp(&mid.z[b]))
        .ok_or("no tip")?;
    // a round tip's front-most point slides along it, so the tip's height is the middle of the part within 1 mm of the front
    let (mut wy, mut ws) = (0.0, 0.0);
    for k in mid.index(-85.0)..=mid.index(-20.0) {
        let w = mid.z[k] - (mid.z[tip_k] - 1.0);
        if w > 0.0 {
            wy += w * mid.u(k);
            ws += w;
        }
    }
    let tip = [0.0, wy / ws, mid.z[tip_k]];

    // the base: below the tip, the first hollow, then the front of the upper lip, then the deepest point under the line between
    // the tip and the lip's front
    let lowest = mid.index(tip[1] - BASE_REACH);
    // going down from the tip, the lowest point before the profile rises again by more than half a millimetre
    let mut hollow = tip_k;
    let mut k = tip_k;
    while k > lowest + 1 && !mid.z[k - 1].is_nan() {
        k -= 1;
        if mid.z[k] < mid.z[hollow] {
            hollow = k;
        } else if mid.z[k] > mid.z[hollow] + 0.5 {
            break;
        }
    }
    let lip_from = hollow.saturating_sub((20.0 / STEP) as usize).max(lowest);
    let lip = (lip_from..=hollow)
        .filter(|&k| !mid.z[k].is_nan())
        .max_by(|&a, &b| mid.z[a].total_cmp(&mid.z[b]))
        .ok_or("no upper lip")?;
    let base_k = if lip < hollow {
        let (a, b) = ([mid.u(tip_k), mid.z[tip_k]], [mid.u(lip), mid.z[lip]]);
        let (dy, dz) = (b[0] - a[0], b[1] - a[1]);
        let len = (dy * dy + dz * dz).sqrt();
        (lip..=tip_k)
            .filter(|&k| !mid.z[k].is_nan())
            .max_by(|&p, &q| {
                let behind = |k: usize| -((mid.u(k) - a[0]) * dz - (mid.z[k] - a[1]) * dy) / len;
                behind(p).total_cmp(&behind(q))
            })
            .unwrap_or(hollow)
    } else {
        hollow
    };
    let base = [0.0, mid.u(base_k), mid.z[base_k]];
    if tip[1] - base[1] < 2.0 {
        return Err("base too close to the tip");
    }

    // the wings: the widest pair of steepest sides between the base and just above the tip
    let mut rows: Vec<(f64, [[f64; 2]; 2])> = Vec::new();
    let mut y = base[1] + 1.5;
    while y <= tip[1] + 4.0 {
        if let Some(c) = sides(&across(surface, y), 26.0, WING_STEEPNESS) {
            rows.push((y, c));
        }
        y += 0.5;
    }
    if rows.len() < 3 {
        return Err("no wings");
    }
    let widths: Vec<f64> = rows.iter().map(|r| r.1[1][0] - r.1[0][0]).collect();
    let blurred: Vec<f64> = (0..widths.len())
        .map(|i| {
            let lo = i.saturating_sub(2);
            let hi = (i + 2).min(widths.len() - 1);
            widths[lo..=hi].iter().sum::<f64>() / (hi - lo + 1) as f64
        })
        .collect();
    let wi = (0..blurred.len())
        .max_by(|&a, &b| blurred[a].total_cmp(&blurred[b]))
        .ok_or("no wings")?;
    let (wy, wc) = rows[wi];
    let wings = [[wc[0][0], wy, wc[0][1]], [wc[1][0], wy, wc[1][1]]];
    let width_wings = blurred[wi];
    // the cheek just beside each wing, 3 mm out from its side
    // (read halfway between the tip and the base, because the widest row of the wings can jump between rows far apart on a flat nose)
    let wing_cut = across(surface, 0.5 * (tip[1] + base[1])).blurred(1.0);
    let cheek = 0.5 * (wing_cut.at(-0.5 * width_wings - 3.0) + wing_cut.at(0.5 * width_wings + 3.0));
    if cheek.is_nan() {
        return Err("no cheek beside the wings");
    }

    // the base seen from below: at each place across, the front-most surface of the lower nose, from its base to 2 mm above the tip,
    // between 2 mm outside each wing
    let mut below = Cut {
        from: ACROSS_FROM,
        z: vec![f64::NAN; ACROSS_COUNT],
    };
    let mut y = base[1] + 0.5;
    while y <= tip[1] + 2.0 {
        for (k, z) in across(surface, y).z.into_iter().enumerate() {
            if !z.is_nan() && (below.z[k].is_nan() || z > below.z[k]) {
                below.z[k] = z;
            }
        }
        y += 0.5;
    }
    let below = below.blurred(0.5);
    // the tip's width: the lower nose seen from below, 5 mm behind its front
    let (width_tip, _) = width_at_depth(&below, 5.0).ok_or("no tip width")?;
    // the tip's bluntness from below: the width of the lower nose seen from below 2.5 mm behind its front, against the width 5 mm
    // behind. A tip shaped like a parabola scores 0, a blunt tip more and a pointed tip less.
    let (w_near, _) = width_at_depth(&below, CAP_DEPTH).ok_or("no tip front from below")?;
    let round_below = w_near - CAP_RATIO * width_tip;
    // the tip's roundness from the side: how tall the part of the profile within 2.5 mm of the tip's front is, against how tall the
    // part within 5 mm is
    let cap_height = |depth: f64| -> Option<f64> {
        let level = mid.z[tip_k] - depth;
        let edge = |dir: isize| -> Option<f64> {
            let mut k = tip_k as isize;
            loop {
                let next = k + dir;
                if next < 0 || next >= mid.z.len() as isize || mid.z[next as usize].is_nan() {
                    return None;
                }
                if mid.z[next as usize] < level {
                    let (a, b) = (mid.z[k as usize], mid.z[next as usize]);
                    return Some(mid.u(k as usize) + (a - level) / (a - b) * dir as f64 * STEP);
                }
                k = next;
            }
        };
        Some(edge(1)? - edge(-1)?)
    };
    let round_profile = cap_height(CAP_DEPTH).zip(cap_height(2.0 * CAP_DEPTH)).map(|(a, b)| a - CAP_RATIO * b).ok_or("no tip front from the side")?;
    // widths 4 mm behind the front, each averaged over three heights 2 mm apart: the bridge a third of the way from the eyes to the
    // tip, the tip at its own height
    let bridge_y = 0.35 * tip[1];
    let mut width_bridge = 0.0;
    let mut bridge_ends = [0.0; 2];
    for dy in [-2.0, 0.0, 2.0] {
        let (w, e) =
            width_at_depth(&across(surface, bridge_y + dy), DEPTH).ok_or("no bridge width")?;
        width_bridge += w / 3.0;
        if dy == 0.0 {
            bridge_ends = e;
        }
    }
    let length = -base[1];
    let projection = tip[2] - cheek;
    let root = (mid.at(-2.0) + mid.at(0.0) + mid.at(2.0)) / 3.0;
    if root.is_nan() {
        return Err("no bridge between the eyes");
    }
    let tip_turn = (tip[1] - base[1]).atan2(tip[2] - base[2]).to_degrees();

    // the bridge's line: how far the profile lies in front of the line from a quarter of the way down to the tip, on average over
    // the middle 80% of that line
    let top_y = 0.25 * tip[1];
    let a = [top_y, mid.at(top_y)];
    let (dy, dz) = (tip[1] - a[0], tip[2] - a[1]);
    let len = (dy * dy + dz * dz).sqrt();
    let (mut sum, mut count) = (0.0, 0);
    let (mut halves, mut half_counts) = ([0.0; 2], [0usize; 2]);
    for k in mid.index(tip[1])..=mid.index(top_y) {
        let s = ((mid.u(k) - a[0]) * dy + (mid.z[k] - a[1]) * dz) / (len * len);
        if (0.1..=0.9).contains(&s) && !mid.z[k].is_nan() {
            let d = ((mid.u(k) - a[0]) * dz - (mid.z[k] - a[1]) * dy) / len;
            sum += d;
            count += 1;
            let half = (s > 0.5) as usize;
            halves[half] += d;
            half_counts[half] += 1;
        }
    }
    // the bridge's bend: how much further out the lower half of the bridge stands from that line than the upper half
    if half_counts.contains(&0) {
        return Err("no bridge line");
    }
    let bend = halves[1] / half_counts[1] as f64 - halves[0] / half_counts[0] as f64;
    let dorsum = if count > 0 {
        sum / count as f64
    } else {
        return Err("no bridge line");
    };

    // the wings' flare: seen from the front, how far each wing stands out to the side beyond the sides of the nose above it, up to
    // 4 mm above the tip, on average
    let above = &widths[wi..];
    let flare = 0.5 * (width_wings - above.iter().sum::<f64>() / above.len() as f64);
    // how far the bridge stands out: the profile's depth halfway from the eyes to the tip, in front of the eyes' centres
    let mid_y = 0.5 * tip[1];
    let bridge_height = mid.at(mid_y);
    if bridge_height.is_nan() {
        return Err("no middle of the bridge");
    }

    // the dip between the brow and the bridge: how far the profile between the eyes lies behind the line from the brow, 6 mm above
    // the eyes, to the middle of the bridge
    let brow = [BROW_Y, mid.at(BROW_Y)];
    let (dy, dz) = (mid_y - brow[0], bridge_height - brow[1]);
    let len = (dy * dy + dz * dz).sqrt();
    let root_dip = -((0.0 - brow[0]) * dz - (root - brow[1]) * dy) / len;
    if root_dip.is_nan() {
        return Err("no brow above the nose");
    }

    // the outlines
    let profile_line: Vec<[f64; 2]> = (base_k..=mid.index(4.0))
        .rev()
        .filter(|&k| !mid.z[k].is_nan())
        .map(|k| [mid.u(k), mid.z[k]])
        .collect();
    if profile_line.len() < 4 {
        return Err("no profile");
    }
    let profile = resample(&profile_line, OUTLINES[0].points);
    let n_front = OUTLINES[1].points;
    let mut front = Vec::with_capacity(n_front);
    let (top, bottom) = (0.15 * tip[1], tip[1] + 3.0);
    for i in 0..n_front {
        let t = i as f64 / (n_front - 1) as f64;
        let (w, _) = width_at_depth(&across(surface, top + (bottom - top) * t), DEPTH).ok_or("no front outline")?;
        front.push([w / 2.0, t]);
    }
    let n_base = OUTLINES[2].points;
    let ends = [wings[0][0] - 2.0, wings[1][0] + 2.0];
    let mut base_line = Vec::with_capacity(n_base);
    for i in 0..n_base {
        let x = ends[0] + (ends[1] - ends[0]) * i as f64 / (n_base - 1) as f64;
        let z = below.at(x);
        if z.is_nan() {
            return Err("a gap in the base");
        }
        base_line.push([x, z]);
    }

    let drawn = |c: &Cut| {
        (0..c.z.len())
            .step_by(2)
            .filter(|&k| !c.z[k].is_nan())
            .map(|k| [c.u(k), c.z[k]])
            .collect()
    };
    Ok(Nose {
        proportions: [
            width_wings,
            width_bridge,
            width_tip,
            length,
            projection,
            root,
            tip_turn,
            dorsum,
            flare,
            bridge_height,
            round_profile,
            round_below,
            root_dip,
            bend,
        ],
        outlines: [profile, front, base_line],
        marks: Marks {
            tip,
            base,
            wings,
            bridge: [
                [bridge_ends[0], bridge_y, 0.0],
                [bridge_ends[1], bridge_y, 0.0],
            ],
        },
        cuts: [drawn(&raw), drawn(&across(surface, wy))],
    })
}

/// An outline with its position and size taken out: moved so its centre is at the origin and scaled so its points lie at 1 from it
/// on average (root mean square). Its turn is kept, because how a nose is turned in the face matters.
pub fn normalized(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let n = points.len() as f64;
    let c = [
        points.iter().map(|p| p[0]).sum::<f64>() / n,
        points.iter().map(|p| p[1]).sum::<f64>() / n,
    ];
    let size = (points
        .iter()
        .map(|p| (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2))
        .sum::<f64>()
        / n)
        .sqrt()
        .max(1e-9);
    points
        .iter()
        .map(|p| [(p[0] - c[0]) / size, (p[1] - c[1]) / size])
        .collect()
}

/// The size of outline `o`: the root mean square distance of its points from their centre, in mm; for the outline from the front,
/// its mean half width.
pub fn size(o: usize, points: &[[f64; 2]]) -> f64 {
    if o == FRONT {
        return points.iter().map(|p| p[0]).sum::<f64>() / points.len() as f64;
    }
    let n = points.len() as f64;
    let c = [points.iter().map(|p| p[0]).sum::<f64>() / n, points.iter().map(|p| p[1]).sum::<f64>() / n];
    (points.iter().map(|p| (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)).sum::<f64>() / n).sqrt()
}

/// The outline from the front, whose points are half widths at fixed shares of the nose's height.
pub const FRONT: usize = 1;

/// The point by point gaps between two copies of outline `o` after position and size are taken out, in mm on an outline of
/// `reference` mm.
///
/// The outline from the front is a width at fixed shares of the nose's height, so only its widths are compared, each divided by the
/// outline's mean width. Its gaps come in pairs like the others', the second always 0.
pub fn outline_gaps(o: usize, a: &[[f64; 2]], b: &[[f64; 2]], reference: f64) -> Vec<f64> {
    if o == FRONT {
        let mean = |l: &[[f64; 2]]| (l.iter().map(|p| p[0]).sum::<f64>() / l.len() as f64).max(1e-9);
        let (ma, mb) = (mean(a), mean(b));
        return a.iter().zip(b).flat_map(|(p, q)| [(p[0] / ma - q[0] / mb) * reference, 0.0]).collect();
    }
    let (a, b) = (normalized(a), normalized(b));
    a.iter().zip(&b).flat_map(|(p, q)| [(p[0] - q[0]) * reference, (p[1] - q[1]) * reference]).collect()
}

/// Like `outline_gaps`, but with each axis's position and size taken out on its own, so stretching an outline along one axis costs
/// nothing and only a change of its shape counts. The named sliders hold the outlines this way, because a longer or wider nose may
/// keep its shape.
pub fn outline_shape_gaps(o: usize, a: &[[f64; 2]], b: &[[f64; 2]], reference: f64) -> Vec<f64> {
    if o == FRONT {
        return outline_gaps(o, a, b, reference);
    }
    let axes = |l: &[[f64; 2]]| -> Vec<[f64; 2]> {
        let n = l.len() as f64;
        let mut out = l.to_vec();
        for c in 0..2 {
            let m = l.iter().map(|p| p[c]).sum::<f64>() / n;
            let s = (l.iter().map(|p| (p[c] - m).powi(2)).sum::<f64>() / n).sqrt().max(1e-9);
            for (q, p) in out.iter_mut().zip(l) {
                q[c] = (p[c] - m) / s;
            }
        }
        out
    };
    let (a, b) = (axes(a), axes(b));
    let k = reference / std::f64::consts::SQRT_2;
    a.iter().zip(&b).flat_map(|(p, q)| [(p[0] - q[0]) * k, (p[1] - q[1]) * k]).collect()
}

/// The difference between two copies of outline `o`: the root mean square distance between matching points, after position and
/// size are taken out, in mm on an outline of `reference` mm.
pub fn outline_difference(o: usize, a: &[[f64; 2]], b: &[[f64; 2]], reference: f64) -> f64 {
    let g = outline_gaps(o, a, b, reference);
    (g.as_chunks::<2>().0.iter().map(|c| c[0] * c[0] + c[1] * c[1]).sum::<f64>() / (g.len() / 2) as f64).sqrt()
}

/// How two noses differ: each proportion (b minus a) and each outline's difference.
pub fn compare(a: &Nose, b: &Nose, references: &[f64; 3]) -> ([f64; N_PROPORTIONS], [f64; 3]) {
    let p = std::array::from_fn(|i| b.proportions[i] - a.proportions[i]);
    let o = std::array::from_fn(|i| outline_difference(i, &a.outlines[i], &b.outlines[i], references[i]));
    (p, o)
}

/// The pass marks: a nose is made when every proportion and every outline is within its mark of the target's.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PassMarks {
    pub proportions: Vec<f64>,
    pub outlines: Vec<f64>,
    /// The outlines' reference sizes in mm, the mean size of the KOTOR noses' outlines.
    pub references: Vec<f64>,
    pub provisional: bool,
}
