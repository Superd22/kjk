//! The jaw and chin as a feature of the slider kit.
//!
//! Everything is measured in the frame of the eyes (`measure::Frame`), on any triangle mesh.
//!
//! The jaw is read from a depth picture of the lower face seen from the front, every 0.35 mm. The lower face is the part of that picture
//! that can be reached from the front of the chin without crossing a step steeper than 5 in 1 (about 79 degrees): where the jaw hides
//! the neck behind it, or turns away to the side, the step is steeper. Its outline is read along rays from a point 60 mm below the eyes,
//! every 5 degrees from straight down to straight out, as the last place on each ray that is still lower face. The two sides are
//! averaged. Each outline point also keeps the depth of the face there, so the same points give the jawline seen from the side.
//!
//! The chin's profile is the cut down the middle of the face, read as the front-most surface every 0.25 mm and lightly blurred, as for
//! the nose. The underside of the chin is the lowest surface of that cut at each depth.

use crate::V3;
use crate::feature::{BumpRule, Feature, Guard, KotorExtras, Measured, OutlineDef, Shot, Spec};
use crate::measure::{Cut, Frame, Proportion, STEP, Surface, Window, resample, width_at_depth};
use crate::mesh::HeadMesh;
use crate::zone::Zone;
use serde_json::json;

pub struct Jaw;

pub static JAW: Jaw = Jaw;

pub const PROPORTIONS: [Proportion; 7] = [
    Proportion { key: "jaw_width", label: "Width of the jaw below the mouth", low: "narrow jaw", high: "wide jaw", unit: "mm" },
    Proportion { key: "jaw_low", label: "Width of the jaw near the chin", low: "tapering jaw", high: "square jaw", unit: "mm" },
    Proportion { key: "jaw_slope", label: "Slope of the jawline, seen from the side", low: "level jawline", high: "steep jawline", unit: "degrees" },
    Proportion { key: "chin_bottom", label: "How far below the eyes the chin reaches", low: "short chin", high: "long chin", unit: "mm" },
    Proportion { key: "chin_width", label: "Width of the chin", low: "narrow chin", high: "broad chin", unit: "mm" },
    Proportion { key: "chin_projection", label: "How far the chin stands out", low: "receding chin", high: "jutting chin", unit: "mm" },
    Proportion { key: "under_angle", label: "Angle under the chin", low: "level under the chin", high: "sloping under the chin", unit: "degrees" },
];

pub const OUTLINES: [OutlineDef; 3] = [
    OutlineDef { key: "front", label: "Jawline from the front", points: 19, widths: false },
    OutlineDef { key: "side", label: "Jawline from the side", points: 12, widths: false },
    OutlineDef { key: "profile", label: "Chin profile", points: 20, widths: false },
];

const GROUPS: [&str; 2] = ["jaw", "chin"];

const SHOTS: [Shot; 4] = [
    Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-80.0, 80.0, -150.0, -30.0] },
    Shot { name: "three-quarter", yaw: 35.0, pitch: 0.0, pivot: 0.0, window: [-80.0, 80.0, -150.0, -30.0] },
    Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-110.0, 50.0, -150.0, -30.0] },
    Shot { name: "below", yaw: 0.0, pitch: 60.0, pivot: -100.0, window: [-80.0, 80.0, -160.0, -40.0] },
];

/// The window the measure reads.
const WINDOW: Window = Window { x: 90.0, y: [-175.0, -20.0], z: -110.0 };
/// The depth picture's step, in mm.
const PIXEL: f64 = 0.35;
/// The steepest step, in depth per width, that still joins the lower face.
const STEEP: f64 = 25.0;
/// The point the outline's rays start from, below the eyes.
const CENTRE_Y: f64 = -60.0;
/// The outline's rays, every 5 degrees from straight down (0) to straight out (90).
const RAYS: usize = 19;
/// The jawline seen from the side runs over the outline's first rays, to 55 degrees.
const SIDE_RAYS: usize = 11;
/// The height of the jaw's width, below the eyes.
const JAW_Y: f64 = -80.0;
/// The jaw's width leaves out what lies further back than this (ears, hair), in mm behind the eyes.
const BEHIND: f64 = -75.0;
/// The height of the jaw's width near the chin, and how far back it reads.
const LOW_Y: f64 = -100.0;
const LOW_BEHIND: f64 = -40.0;

/// The angle of the line fitted to points (depth, height), in degrees: positive when the height rises as the depth goes back.
fn line_angle(p: &[[f64; 2]]) -> f64 {
    let n = p.len() as f64;
    let (mz, my) = (p.iter().map(|q| q[0]).sum::<f64>() / n, p.iter().map(|q| q[1]).sum::<f64>() / n);
    let (szz, szy) = p.iter().fold((0.0, 0.0), |(a, b), q| (a + (q[0] - mz).powi(2), b + (q[0] - mz) * (q[1] - my)));
    (-szy / szz.max(1e-9)).atan().to_degrees()
}

/// The tilt of the camera the lower face is seen from: in front, and this many degrees above.
const TILT: f64 = 40.0;

/// The front-most depth of a surface seen from in front and `TILT` degrees above, every `PIXEL` mm, NaN where there is none. From that
/// camera the jaw hides the skin under it, so the edge of the lower face is where the jaw turns to face more than `TILT` degrees down.
/// Across the picture is x; up the picture is `y cos t - z sin t`; the depth toward the camera is `y sin t + z cos t`.
struct Picture {
    x0: f64,
    v0: f64,
    w: usize,
    h: usize,
    z: Vec<f64>,
}

const V_RANGE: [f64; 2] = [-165.0, 45.0];

fn tilted(p: V3) -> V3 {
    let (s, c) = TILT.to_radians().sin_cos();
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c]
}

fn untilted(q: V3) -> V3 {
    let (s, c) = TILT.to_radians().sin_cos();
    [q[0], q[1] * c + q[2] * s, -q[1] * s + q[2] * c]
}

impl Picture {
    fn new(surface: &Surface) -> Self {
        let (x0, v0) = (-WINDOW.x, V_RANGE[0]);
        let w = (2.0 * WINDOW.x / PIXEL) as usize;
        let h = ((V_RANGE[1] - V_RANGE[0]) / PIXEL) as usize;
        let mut z = vec![f64::NAN; w * h];
        for t in &surface.triangles {
            let q = t.map(|p| {
                let r = tilted(p);
                [(r[0] - x0) / PIXEL, (r[1] - v0) / PIXEL, r[2]]
            });
            let area = (q[1][0] - q[0][0]) * (q[2][1] - q[0][1]) - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1]);
            if area.abs() < 1e-12 {
                continue;
            }
            let lo = |c: usize| q.iter().map(|p| p[c]).fold(f64::MAX, f64::min);
            let hi = |c: usize| q.iter().map(|p| p[c]).fold(f64::MIN, f64::max);
            let (i0, i1) = ((lo(0) - 0.5).ceil().max(0.0) as i64, ((hi(0) - 0.5).floor() as i64).min(w as i64 - 1));
            let (j0, j1) = ((lo(1) - 0.5).ceil().max(0.0) as i64, ((hi(1) - 0.5).floor() as i64).min(h as i64 - 1));
            for j in j0..=j1 {
                for i in i0..=i1 {
                    let (px, py) = (i as f64 + 0.5, j as f64 + 0.5);
                    let b0 = ((q[1][0] - px) * (q[2][1] - py) - (q[2][0] - px) * (q[1][1] - py)) / area;
                    let b1 = ((q[2][0] - px) * (q[0][1] - py) - (q[0][0] - px) * (q[2][1] - py)) / area;
                    let b2 = 1.0 - b0 - b1;
                    if b0 < -1e-9 || b1 < -1e-9 || b2 < -1e-9 {
                        continue;
                    }
                    let d = b0 * q[0][2] + b1 * q[1][2] + b2 * q[2][2];
                    let at = j as usize * w + i as usize;
                    if z[at].is_nan() || d > z[at] {
                        z[at] = d;
                    }
                }
            }
        }
        Self { x0, v0, w, h, z }
    }

    fn pixel(&self, x: f64, v: f64) -> Option<usize> {
        let (i, j) = (((x - self.x0) / PIXEL).floor(), ((v - self.v0) / PIXEL).floor());
        (i >= 0.0 && j >= 0.0 && (i as usize) < self.w && (j as usize) < self.h).then(|| j as usize * self.w + i as usize)
    }

    /// The pixels joined to `seed` by steps no steeper than `STEEP`.
    fn region(&self, seed: usize) -> Vec<bool> {
        let mut inside = vec![false; self.z.len()];
        let mut stack = vec![seed];
        inside[seed] = true;
        let limit = STEEP * PIXEL;
        while let Some(at) = stack.pop() {
            let (i, j) = (at % self.w, at / self.w);
            let here = self.z[at];
            let mut visit = |n: usize| {
                if !inside[n] && !self.z[n].is_nan() && (self.z[n] - here).abs() <= limit {
                    inside[n] = true;
                    stack.push(n);
                }
            };
            if i > 0 {
                visit(at - 1);
            }
            if i + 1 < self.w {
                visit(at + 1);
            }
            if j > 0 {
                visit(at - self.w);
            }
            if j + 1 < self.h {
                visit(at + self.w);
            }
        }
        inside
    }
}

/// The last place along a ray in the picture from `centre` at `angle` degrees from straight down, toward `side` (1 or -1), that is
/// still in the region, as a place in the frame of the eyes.
fn last_inside(p: &Picture, region: &[bool], centre: f64, angle: f64, side: f64) -> Option<V3> {
    let (s, c) = angle.to_radians().sin_cos();
    let step = 0.5 * PIXEL;
    let mut best = None;
    let mut r = 0.0;
    while r < 130.0 {
        let (x, v) = (side * s * r, centre - c * r);
        if let Some(at) = p.pixel(x, v)
            && region[at]
        {
            best = Some(untilted([x, v, p.z[at]]));
        }
        r += step;
    }
    best
}

/// The lowest surface of the cut down the middle of the face at each depth from `from`, every 0.25 mm, NaN where there is none.
pub fn underside(surface: &Surface, from: f64, count: usize) -> Vec<f64> {
    let mut y = vec![f64::NAN; count];
    for t in &surface.triangles {
        let d = t.map(|p| if p[0] == 0.0 { 1e-9 } else { p[0] });
        let mut ends = Vec::with_capacity(2);
        for (i, j) in [(0, 1), (1, 2), (2, 0)] {
            if (d[i] > 0.0) != (d[j] > 0.0) {
                let s = d[i] / (d[i] - d[j]);
                ends.push([t[i][2] + s * (t[j][2] - t[i][2]), t[i][1] + s * (t[j][1] - t[i][1])]);
            }
        }
        if ends.len() != 2 {
            continue;
        }
        let (a, b) = if ends[0][0] < ends[1][0] { (ends[0], ends[1]) } else { (ends[1], ends[0]) };
        let k0 = ((a[0] - from) / STEP).ceil().max(0.0) as usize;
        let k1 = (((b[0] - from) / STEP).floor() as isize).min(count as isize - 1);
        if k1 < 0 {
            continue;
        }
        for (k, yk) in y.iter_mut().enumerate().take(k1 as usize + 1).skip(k0) {
            let u = from + k as f64 * STEP;
            let s = if b[0] > a[0] { (u - a[0]) / (b[0] - a[0]) } else { 0.0 };
            let v = a[1] + s * (b[1] - a[1]);
            if yk.is_nan() || v < *yk {
                *yk = v;
            }
        }
    }
    y
}

/// The width of a closed outline given as points on one side from the bottom (`points`, x positive), at height `y`.
fn half_width_at(points: &[[f64; 3]], y: f64) -> Option<f64> {
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        if (a[1] - y) * (b[1] - y) <= 0.0 && a[1] != b[1] {
            let s = (y - a[1]) / (b[1] - a[1]);
            return Some(a[0] + s * (b[0] - a[0]));
        }
    }
    None
}

/// Measures the jaw and chin of a surface, or says what could not be found.
pub fn measure(surface: &Surface) -> Result<Measured, &'static str> {
    let pic = Picture::new(surface);
    // the seed: the place nearest the camera within 2 mm of the middle, between 80 and 125 mm below the eyes
    let mut seed = None;
    let mut y = -125.0;
    while y <= -80.0 {
        for x in [-1.5, -0.5, 0.5, 1.5] {
            // the front of the chin lies about 15 mm in front of the eyes
            let v = tilted([x, y, 15.0])[1];
            if let Some(at) = pic.pixel(x, v)
                && !pic.z[at].is_nan()
                && seed.is_none_or(|(_, z)| pic.z[at] > z)
            {
                seed = Some((at, pic.z[at]));
            }
        }
        y += PIXEL;
    }
    let (seed, _) = seed.ok_or("no chin")?;
    let region = pic.region(seed);
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(path) = std::env::var("JAW_DEBUG_PICTURE") {
        let (lo, hi) = pic.z.iter().filter(|z| !z.is_nan()).fold((f64::MAX, f64::MIN), |(a, b), &z| (a.min(z), b.max(z)));
        let img = image::RgbImage::from_fn(pic.w as u32, pic.h as u32, |i, j| {
            let at = (pic.h - 1 - j as usize) * pic.w + i as usize;
            let z = pic.z[at];
            if z.is_nan() {
                return image::Rgb([0, 0, 0]);
            }
            let g = (40.0 + 200.0 * (z - lo) / (hi - lo)) as u8;
            if region[at] { image::Rgb([g, g / 2, g / 2]) } else { image::Rgb([g, g, g]) }
        });
        img.save(path).unwrap();
    }
    // the edge of the lower face along rays in the picture from the place 60 mm below the eyes and 20 mm in front, every 2.5 degrees
    // from straight down to 60 degrees above straight out (in the picture, the sides of the face lie lower than in front), both sides
    // averaged, as places in the frame of the eyes on the right side
    let centre = tilted([0.0, CENTRE_Y, 20.0])[1];
    let mut edge: Vec<[f64; 3]> = Vec::new();
    for k in 0..=60 {
        let a = 2.5 * k as f64;
        let (Some(l), Some(r)) = (last_inside(&pic, &region, centre, a, -1.0), last_inside(&pic, &region, centre, a, 1.0)) else {
            return Err("a gap in the jawline");
        };
        edge.push([0.5 * (r[0] - l[0]), 0.5 * (l[1] + r[1]), 0.5 * (l[2] + r[2])]);
    }
    // the outline from the front, every 5 degrees from straight down to straight out around the place 60 mm below the eyes
    let angle_of = |p: &[f64; 3]| p[0].atan2(CENTRE_Y - p[1]).to_degrees();
    let mut outline: Vec<[f64; 3]> = Vec::with_capacity(RAYS);
    for k in 0..RAYS {
        let a = 5.0 * k as f64;
        let Some(i) = (0..edge.len() - 1).find(|&i| angle_of(&edge[i + 1]) >= a) else {
            return Err("the jawline does not reach the sides");
        };
        let (p, q) = (edge[i], edge[i + 1]);
        let (ap, aq) = (angle_of(&p), angle_of(&q));
        let s = if (aq - ap).abs() > 1e-9 { ((a - ap) / (aq - ap)).clamp(0.0, 1.0) } else { 0.0 };
        outline.push(std::array::from_fn(|c| p[c] + s * (q[c] - p[c])));
    }
    let bottom = outline[0];
    let chin_bottom = -bottom[1];
    // the jaw's width seen from the front: the face's outline 80 to 90 mm below the eyes, just under the mouth's corners, in front of
    // the ears; and the same outline's width as the lower face's outline reads it
    let mut jaw_width = 0.0;
    for y in [JAW_Y - 2.0, JAW_Y - 5.0, JAW_Y - 8.0] {
        let (mut lo, mut hi) = (0.0f64, 0.0f64);
        for [a, b] in surface.cut(1, y, 0) {
            for p in [a, b] {
                if p[1] > BEHIND {
                    lo = lo.min(p[0]);
                    hi = hi.max(p[0]);
                }
            }
        }
        if lo >= -10.0 || hi <= 10.0 {
            return Err("no jaw width");
        }
        jaw_width += (hi - lo) / 3.0;
    }
    let w80 = half_width_at(&outline, JAW_Y).ok_or("no jaw side")?;
    // how square the jaw is: halfway down from there to the chin's bottom, how far the outline stands out beyond the straight line
    // from its side 80 mm below the eyes to the middle of the chin's bottom
    let mid_y = 0.5 * (JAW_Y + bottom[1]);
    let _ = (mid_y, w80);
    // the jaw's width near the chin: the face's outline 98 to 104 mm below the eyes, in front of 40 mm behind the eyes so the neck is
    // left out
    let mut jaw_low = 0.0;
    for y in [LOW_Y + 2.0, LOW_Y - 1.0, LOW_Y - 4.0] {
        let (mut lo, mut hi) = (0.0f64, 0.0f64);
        for [a, b] in surface.cut(1, y, 0) {
            for p in [a, b] {
                if p[1] > LOW_BEHIND {
                    lo = lo.min(p[0]);
                    hi = hi.max(p[0]);
                }
            }
        }
        if lo >= -5.0 || hi <= 5.0 {
            return Err("no width near the chin");
        }
        jaw_low += (hi - lo) / 3.0;
    }
    // the jawline seen from the side: the outline's points from the chin's bottom to 55 degrees, as (depth, height); its slope is the
    // line fitted to them, in degrees, rising as it goes back from the chin
    let side: Vec<[f64; 2]> = outline[..=SIDE_RAYS].iter().map(|o| [o[2], o[1]]).collect();
    let jaw_slope = line_angle(&side);
    // the profile down the middle
    let from = -160.0;
    let mid = Cut { from, z: surface.front(0, 0.0, 1, from, 560) }.blurred(0.75);
    let (pk, pogonion) = (mid.index(bottom[1])..=mid.index(bottom[1] + 18.0))
        .filter(|&k| !mid.z[k].is_nan())
        .map(|k| (k, mid.z[k]))
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .ok_or("no front of the chin")?;
    let chin_projection = pogonion;
    // the profile runs from the hollow under the lower lip (the deepest place between 3 and 18 mm above the chin's front, so the lip
    // itself is left out) down to where it turns under the chin: where it falls back faster than 3 mm per mm of height, or 1 mm above
    // the chin's bottom
    let mut low = pk;
    while low > mid.index(bottom[1] + 1.0) && !mid.z[low - 1].is_nan() && mid.z[low] - mid.z[low - 1] < 3.0 * STEP {
        low -= 1;
    }
    let hollow = (mid.index(mid.u(pk) + 3.0)..=mid.index(mid.u(pk) + 18.0))
        .filter(|&k| !mid.z[k].is_nan())
        .min_by(|&a, &b| mid.z[a].total_cmp(&mid.z[b]))
        .ok_or("no hollow under the lip")?;
    let profile_line: Vec<[f64; 2]> = (low..=hollow).rev().filter(|&k| !mid.z[k].is_nan()).map(|k| [mid.u(k), mid.z[k]]).collect();
    if profile_line.len() < 8 {
        return Err("no chin profile");
    }
    let profile = resample(&profile_line, OUTLINES[2].points);
    // the chin across: level cuts from 6 to 16 mm above the chin's bottom, read as the front-most surface across; the width of the
    // part within 6 mm of the chin's front is the chin's width
    let mut chin_width = 0.0;
    for k in 0..5 {
        let y = bottom[1] + 6.0 + 2.5 * k as f64;
        let cut = Cut { from: -60.0, z: surface.front(1, y, 0, -60.0, 481) };
        let (w, _) = width_at_depth(&cut, 6.0).ok_or("no chin width")?;
        chin_width += w / 5.0;
    }
    // the underside of the chin: the lowest surface down the middle from 6 to 22 mm behind the chin's front, as a line
    let ufrom = pogonion - 22.0;
    let under = underside(surface, ufrom, 65);
    let pts: Vec<[f64; 2]> =
        (0..under.len()).filter(|&k| !under[k].is_nan()).map(|k| [ufrom + k as f64 * STEP, under[k]]).collect();
    if pts.len() < 20 {
        return Err("no underside of the chin");
    }
    let under_angle = line_angle(&pts);
    let front: Vec<[f64; 2]> = outline.iter().map(|o| [o[0], o[1]]).collect();
    // points to draw on pictures: the outline on both sides, the profile, the underside and the landmarks
    let mut dots: Vec<[f64; 3]> = outline.iter().flat_map(|o| [*o, [-o[0], o[1], o[2]]]).collect();
    dots.extend(profile.iter().map(|p| [0.0, p[0], p[1]]));
    dots.extend(pts.iter().step_by(8).map(|p| [0.0, p[1], p[0]]));

    Ok(Measured {
        proportions: vec![jaw_width, jaw_low, jaw_slope, chin_bottom, chin_width, chin_projection, under_angle],
        outlines: vec![front, side, profile],
        marks: json!({"bottom": bottom, "pogonion": [0.0, mid.u(pk), pogonion], "dots": dots}),
        cuts: vec![(0..mid.z.len()).step_by(2).filter(|&k| !mid.z[k].is_nan()).map(|k| [mid.u(k), mid.z[k]]).collect()],
    })
}

fn allowed(_p: V3) -> bool {
    true
}

fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The distance from each merged vertex of the base head to the nearest vertex of `fence`, in mm on the reference face.
fn distances(local: &[V3], fence: &[bool]) -> Vec<f64> {
    let f: Vec<V3> = local.iter().zip(fence).filter(|(_, x)| **x).map(|(p, _)| *p).collect();
    local.iter().map(|p| f.iter().map(|q| crate::length(crate::sub(*p, *q))).fold(f64::INFINITY, f64::min)).collect()
}

/// How a KOTOR head's measure is cut down where its own mesh carries hair or cloth over the jaw: the rows each part of the face carries,
/// and that part of the face, in the frame of the eyes.
const SIDES: Window = Window { x: 90.0, y: [-120.0, -55.0], z: -110.0 };
const CHIN: Window = Window { x: 30.0, y: [-140.0, -80.0], z: -60.0 };
/// The rows read on the sides of the jaw, and on the chin (proportions, then outlines after them).
const SIDE_ROWS: [&str; 5] = ["jaw_width", "jaw_low", "jaw_slope", "front", "side"];
const CHIN_ROWS: [&str; 5] = ["chin_bottom", "chin_width", "chin_projection", "under_angle", "profile"];
/// A KOTOR vertex is hair or cloth when the hairless wrap did not take it for skin and it stands more than this far outside our fit of
/// the head, in mm.
const COVER_MM: f64 = 6.0;

/// Which of KOTOR's triangles are hair or cloth over the face: every corner is not skin and stands out from our fit of the head. The
/// fit only locates the skin; nothing is measured on it.
pub fn cover(extras: &KotorExtras, frame: &Frame, positions: &[V3], triangles: &[[u32; 3]], fit_triangles: Option<&[[u32; 3]]>) -> Vec<bool> {
    let (Some(skin), Some(fit)) = (extras.skin, extras.fit) else { return vec![false; triangles.len()] };
    let fit_local: Vec<V3> = fit.iter().map(|&p| frame.local(p)).collect();
    let mut normals = vec![[0.0; 3]; fit.len()];
    if let Some(ft) = fit_triangles {
        for t in ft {
            let n = crate::cross(crate::sub(fit_local[t[1] as usize], fit_local[t[0] as usize]), crate::sub(fit_local[t[2] as usize], fit_local[t[0] as usize]));
            for &i in t {
                normals[i as usize] = crate::add(normals[i as usize], n);
            }
        }
    }
    let outside: Vec<bool> = positions
        .iter()
        .zip(skin)
        .map(|(&p, &is_skin)| {
            if is_skin {
                return false;
            }
            let q = frame.local(p);
            let (k, _) = fit_local
                .iter()
                .enumerate()
                .map(|(k, f)| (k, crate::length(crate::sub(q, *f))))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();
            crate::dot(crate::sub(q, fit_local[k]), crate::normalize(normals[k])) > COVER_MM
        })
        .collect();
    triangles.iter().map(|t| t.iter().all(|&i| outside.get(i as usize).copied().unwrap_or(false))).collect()
}


impl Feature for Jaw {
    fn key(&self) -> &'static str {
        "jaw"
    }
    fn title(&self) -> &'static str {
        "Jaw and chin"
    }
    fn proportions(&self) -> &'static [Proportion] {
        &PROPORTIONS
    }
    fn outlines(&self) -> &'static [OutlineDef] {
        &OUTLINES
    }
    fn specs(&self) -> &'static [Spec] {
        &SPECS
    }
    fn groups(&self) -> &'static [&'static str] {
        &GROUPS
    }
    fn window(&self) -> Window {
        WINDOW
    }
    fn region(&self) -> Window {
        Window { x: 70.0, y: [-135.0, -60.0], z: -60.0 }
    }
    fn measure(&self, surface: &Surface) -> Result<Measured, &'static str> {
        measure(surface)
    }
    fn measure_kotor(&self, surface: &Surface, extras: &KotorExtras, frame: &Frame, positions: &[V3]) -> Result<Measured, &'static str> {
        if extras.name == Some("n_jedimalek") {
            return Err("Malak's lower jaw is metal, not skin");
        }
        let Some(triangles) = extras.triangles else { return measure(surface) };
        let covered = cover(extras, frame, positions, triangles, extras.fit_triangles);
        if !covered.iter().any(|&c| c) {
            return measure(surface);
        }
        // the measure reads the mesh without its hair and cloth, and gives up the rows whose part of the face they cover
        let local: Vec<[V3; 3]> = triangles
            .iter()
            .zip(&covered)
            .filter(|(_, c)| !**c)
            .map(|(t, _)| t.map(|i| frame.local(positions[i as usize])))
            .filter(|t| t.iter().any(|&p| WINDOW.contains(p)))
            .collect();
        let count_in = |w: &Window| {
            triangles
                .iter()
                .zip(&covered)
                .filter(|(t, c)| {
                    **c && {
                        let m = t.iter().fold([0.0; 3], |a, &i| crate::add(a, crate::scale(frame.local(positions[i as usize]), 1.0 / 3.0)));
                        w.contains(m)
                    }
                })
                .count()
        };
        let hidden_rows: Vec<&str> = [(&SIDES, &SIDE_ROWS[..]), (&CHIN, &CHIN_ROWS[..])]
            .iter()
            .filter(|(w, _)| count_in(w) >= 4)
            .flat_map(|(_, rows)| rows.iter().copied())
            .collect();
        let mut m = measure(&Surface::from_local_in(local, &WINDOW))?;
        for key in hidden_rows {
            if let Some(i) = PROPORTIONS.iter().position(|p| p.key == key) {
                m.proportions[i] = f64::NAN;
            } else if let Some(o) = OUTLINES.iter().position(|o| o.key == key) {
                m.outlines[o].clear();
            }
        }
        Ok(m)
    }
    /// The lower face below the mouth, the jaw's sides and the skin under the jaw, 1 inside and fading to 0 over 6 mm toward the lips
    /// (the mouth zone), over 12 mm toward the ears, over 25 mm toward the neck's rim, and from 50 to 65 mm below the eyes toward the
    /// cheeks, and from 90 to 110 mm behind the eyes toward the nape. Every vertex the smoothing holds still (the rim and below, the ears, the inside of the mouth) has 0.
    fn zone(&self, mesh: &HeadMesh, _base: &Measured) -> Zone {
        let frame = Frame::from_eyes(mesh.eyes);
        let local: Vec<V3> = mesh.first.iter().map(|&i| frame.local(mesh.base[i as usize])).collect();
        let mouth = 1 + crate::mesh::ZONES.iter().position(|z| *z == "mouth").unwrap() as u8;
        let n = mesh.merged_count();
        let lips: Vec<bool> = (0..n).map(|g| mesh.zone[mesh.first[g] as usize] == mouth).collect();
        let held: Vec<bool> = (0..n).map(|g| mesh.keep[mesh.first[g] as usize] && !mesh.hidden[g]).collect();
        // the rim and below are held vertices low on the neck; the ears are held vertices far to the side
        let rim: Vec<bool> = (0..n).map(|g| held[g] && local[g][1] < -100.0 && local[g][0].abs() < 60.0).collect();
        let ears: Vec<bool> = (0..n).map(|g| held[g] && local[g][0].abs() >= 60.0).collect();
        let (dl, dr, de) = (distances(&local, &lips), distances(&local, &rim), distances(&local, &ears));
        let weight = (0..n)
            .map(|g| {
                if held[g] {
                    return 0.0;
                }
                let p = local[g];
                smoothstep(dl[g] / 6.0)
                    * smoothstep(dr[g] / 25.0)
                    * smoothstep(de[g] / 12.0)
                    * smoothstep((-50.0 - p[1]) / 15.0)
                    * smoothstep((p[2] + 110.0) / 20.0)
            })
            .collect();
        Zone { weight }
    }
    /// The parts of the jaw on the base head: the chin (within 22 mm of the middle, more than 85 mm below the eyes), the jaw's corners
    /// (more than 38 mm to the side and 20 mm behind the eyes), and the jawline (the rest: the jaw's sides and the skin under the jaw).
    fn part_of(&self, p: V3, _base: &Measured) -> &'static str {
        if p[0].abs() < 22.0 && p[1] < -85.0 {
            "chin"
        } else if p[0].abs() >= 38.0 && p[2] < -20.0 {
            "corners"
        } else {
            "jawline"
        }
    }
    fn bumps(&self) -> BumpRule {
        BumpRule { size_mm: 6.0, spacing_mm: 5.0, min_weight: 0.6, allowed }
    }
    fn miss_words(&self, i: usize, g: f64) -> String {
        let a = g.abs();
        let (more, less) = match PROPORTIONS[i].key {
            "jaw_width" => ("The jaw is too wide", "The jaw is too narrow"),
            "jaw_low" => ("The jaw is too wide near the chin", "The jaw is too narrow near the chin"),
            "jaw_slope" => ("The jawline rises too steeply toward the ear", "The jawline runs too level"),
            "chin_bottom" => ("The chin reaches too far down", "The chin does not reach far enough down"),
            "chin_width" => ("The chin is too broad", "The chin is too narrow"),
            "chin_projection" => ("The chin stands out too far", "The chin does not stand out enough"),
            "under_angle" => ("The line under the chin rises too much toward the neck", "The line under the chin is too level"),
            _ => ("Too much", "Too little"),
        };
        let unit = if PROPORTIONS[i].unit == "degrees" { "degrees" } else { "mm" };
        format!("{} by {a:.1} {unit}.", if g > 0.0 { more } else { less })
    }
    fn carries_hidden(&self) -> bool {
        false
    }
    fn span_quantile(&self) -> f64 {
        0.95
    }
    fn solve_by_spread(&self) -> bool {
        true
    }
    fn guards(&self) -> &'static [Guard] {
        &[Guard::Rim, Guard::LipLine, Guard::Lips, Guard::MouthBag, Guard::Ears]
    }
    fn check_bump(&self) -> V3 {
        [45.0, -42.0, 0.0]
    }
    fn shots(&self) -> &'static [Shot] {
        &SHOTS
    }
}

pub const SPECS: [Spec; 6] = [
    Spec {
        measure: "jaw_width",
        name: "Jaw width",
        hint: "How wide the jaw is below the mouth, seen from the front. The jaw near the chin widens a little with it.",
        group: "jaw",
        parts: &["corners", "jawline"],
        repair_parts: &[],
        free: &[],
        loose: &["jaw_low"],
    },
    Spec {
        measure: "jaw_low",
        name: "Lower jaw width",
        hint: "How wide the jaw stays near the chin, seen from the front. A wide lower jaw looks square, and a narrow one tapers to the chin.",
        group: "jaw",
        parts: &["jawline"],
        repair_parts: &["chin"],
        free: &["front"],
        loose: &["jaw_width"],
    },
    Spec {
        measure: "chin_bottom",
        name: "Chin length",
        hint: "How far down the chin reaches. The jawline beside the chin follows it.",
        group: "chin",
        parts: &["chin", "jawline"],
        repair_parts: &[],
        free: &[],
        loose: &["profile"],
    },
    Spec {
        measure: "chin_width",
        name: "Chin width",
        hint: "How broad the front of the chin is.",
        group: "chin",
        parts: &["chin"],
        repair_parts: &["jawline"],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "chin_projection",
        name: "Chin projection",
        hint: "How far the chin stands out, from receding to jutting.",
        group: "chin",
        parts: &["chin"],
        repair_parts: &["jawline"],
        free: &["profile"],
        loose: &[],
    },
    Spec {
        measure: "under_angle",
        name: "Under the chin",
        hint: "Seen from the side, whether the line under the chin runs level or rises toward the neck.",
        group: "chin",
        parts: &["chin", "jawline"],
        repair_parts: &[],
        free: &[],
        loose: &["profile"],
    },
];
