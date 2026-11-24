//! The eyes as a feature of the slider kit: the opening of each eye, seen from the front, and the skin of the lids around it.
//!
//! Everything is measured in the frame of the eyes (`measure::Frame`). Each eye is read on its own, in its own coordinates: across (u),
//! toward the temple, and up (v), from the eyeball's centre. The two eyes' values are then averaged, so the right eye counts as the
//! mirror of the left. The sliders are made for one eye and mirrored by the kit's mirrored bumps.
//!
//! The opening is where the eyeball shows. A depth picture of the face seen from the front is made every 0.1 mm around each eye. A
//! pixel shows the eyeball when the eyeball's sphere lies in front of every other surface there. On KOTOR's own heads, the eyeball is a
//! sphere fitted to KOTOR's eyeball mesh, and KOTOR's upper lid piece is drawn with the skin, because its lower edge is the top of the
//! eye a player sees. On our heads, the eyeball is our fixed sphere of 12.3 mm at the eye's centre. The opening is the patch of eyeball
//! pixels joined to the middle of the eye.
//!
//! From the opening's columns (its top and bottom at each place across) come the corners (the last columns at least 0.5 mm tall), the
//! width, the height, the tilt, the curves of the two lids above and below the line between the corners, and the corners' angles. The
//! lids' depth is read on the skin alone (without KOTOR's lid piece), along the column through the middle of the opening.

use crate::V3;
use crate::feature::{BumpRule, Feature, Guard, KotorExtras, Measured, OutlineDef, Shot, Spec};
use crate::measure::{EYES_APART_MM, Frame, Proportion, Surface, Window};
use crate::mesh::HeadMesh;
use crate::zone::Zone;
use serde_json::json;

pub struct Eyes;

pub static EYES: Eyes = Eyes;

pub const PROPORTIONS: [Proportion; 47] = [
    Proportion { key: "width", label: "Length of the eye along the eyeball, from corner to corner", low: "short eye", high: "long eye", unit: "mm" },
    Proportion { key: "height", label: "Height of the eye's opening", low: "narrow opening", high: "tall opening", unit: "mm" },
    Proportion { key: "tilt", label: "Tilt of the eye, outer corner above the inner one", low: "outer corner low", high: "outer corner high", unit: "degrees" },
    Proportion { key: "upper_rise", label: "How high the upper lid arches above the corners", low: "flat upper lid", high: "arched upper lid", unit: "mm" },
    Proportion { key: "lower_drop", label: "How low the lower lid curves below the corners", low: "flat lower lid", high: "curved lower lid", unit: "mm" },
    Proportion { key: "upper_peak", label: "Where the upper lid is highest, from the inner corner", low: "peak near the nose", high: "peak near the temple", unit: "share" },
    Proportion { key: "lower_low", label: "Where the lower lid is lowest, from the inner corner", low: "low point near the nose", high: "low point near the temple", unit: "share" },
    Proportion { key: "inner_angle", label: "How open the inner corner is", low: "sharp inner corner", high: "round inner corner", unit: "degrees" },
    Proportion { key: "outer_angle", label: "How open the outer corner is", low: "sharp outer corner", high: "round outer corner", unit: "degrees" },
    Proportion { key: "upper_depth", label: "How far the upper lid stands in front of the skin under the brow", low: "deep-set lid", high: "full lid", unit: "mm" },
    Proportion { key: "lower_depth", label: "How far the lower lid stands in front of the top of the cheek", low: "sunken lower lid", high: "full lower lid", unit: "mm" },
    Proportion { key: "ball", label: "Radius of the eyeball", low: "small eyeball", high: "large eyeball", unit: "mm" },
    Proportion { key: "upper_inner", label: "How high the upper lid stands above the corners, a quarter of the way from the inner corner", low: "low inner upper lid", high: "high inner upper lid", unit: "mm" },
    Proportion { key: "upper_outer", label: "How high the upper lid stands above the corners, a quarter of the way from the outer corner", low: "low outer upper lid", high: "high outer upper lid", unit: "mm" },
    Proportion { key: "lower_inner", label: "How low the lower lid sits below the corners, a quarter of the way from the inner corner", low: "high inner lower lid", high: "low inner lower lid", unit: "mm" },
    Proportion { key: "lower_outer", label: "How low the lower lid sits below the corners, a quarter of the way from the outer corner", low: "high outer lower lid", high: "low outer lower lid", unit: "mm" },
    Proportion { key: "inner_height", label: "How high the inner corner sits above the middle of the opening near it", low: "low inner corner", high: "high inner corner", unit: "mm" },
    Proportion { key: "outer_lift", label: "How high the outer corner sits above the middle of the opening near it", low: "drooping outer corner", high: "lifted outer corner", unit: "mm" },
    Proportion { key: "spacing", label: "Distance between the middles of the two eyes' openings", low: "close-set eyes", high: "wide-set eyes", unit: "mm" },
    Proportion { key: "level", label: "How high the middle of the eye's opening sits on the head", low: "low eyes", high: "high eyes", unit: "mm" },
    Proportion { key: "depth", label: "How far forward the rim of the eye's opening sits on the head", low: "deep-set eyes", high: "prominent eyes", unit: "mm" },
    Proportion { key: "pitch", label: "Angle of the skin around the eye from upright, the skin under the eye in front of the skin above it", low: "eye pointing down", high: "eye pointing up", unit: "degrees" },
    Proportion { key: "inner_reach", label: "How far the inner corner lies round the eyeball from its front, toward the nose", low: "inner corner near the middle", high: "inner corner far toward the nose", unit: "mm" },
    Proportion { key: "outer_reach", label: "How far the outer corner lies round the eyeball from its front, toward the temple", low: "outer corner near the middle", high: "outer corner far toward the temple", unit: "mm" },
    Proportion { key: "inner_round", label: "How much wider the inner corner opens in its first millimetre than further in", low: "sharp inner corner tip", high: "round inner corner tip", unit: "degrees" },
    Proportion { key: "outer_round", label: "How much wider the outer corner opens in its first millimetre than further in", low: "sharp outer corner tip", high: "round outer corner tip", unit: "degrees" },
    Proportion { key: "cover", label: "How far the upper lid piece hangs below the skin's own lid edge, over the middle of the eye", low: "lid piece up under the skin", high: "lid piece low over the eye", unit: "mm" },
    Proportion { key: "nose_skin", label: "How far the skin 2 to 6 mm on the nose side of the inner corner stands in front of the eyeball's front", low: "inner corner at the surface", high: "inner corner buried", unit: "mm" },
    Proportion { key: "inner_tip", label: "How far the inner corner sits in front of the eyeball's front", low: "inner corner far back", high: "inner corner forward", unit: "mm" },
    Proportion { key: "outer_back", label: "How far the outer corner sits in front of the eyeball's front", low: "outer corner far back", high: "outer corner forward", unit: "mm" },
    Proportion { key: "ball_proud", label: "How far the eyeball's front stands in front of the opening's edge", low: "flat eye", high: "eyeball standing out", unit: "mm" },
    Proportion { key: "profile_ball", label: "How much of the eyeball shows in profile", low: "eyeball hidden in profile", high: "eyeball showing in profile", unit: "mm2" },
    Proportion { key: "outer_bulge", label: "How far the skin 1 to 5 mm beyond the outer corner stands in front of the corner", low: "flat outer corner", high: "skin bulging round the outer corner", unit: "mm" },
    Proportion { key: "outer_tip", label: "How far the skin at the outer corner's tip sits in front of the eyeball's front", low: "outer corner's end far back", high: "outer corner's end forward", unit: "mm" },
    Proportion { key: "profile_h2", label: "How tall the eyeball seen in profile is 2 mm behind its front", low: "short in profile", high: "tall in profile", unit: "mm" },
    Proportion { key: "profile_h5", label: "How tall the eyeball seen in profile is 5 mm behind its front", low: "short in profile", high: "tall in profile", unit: "mm" },
    Proportion { key: "profile_h8", label: "How tall the eyeball seen in profile is 8 mm behind its front", low: "short in profile", high: "tall in profile", unit: "mm" },
    Proportion { key: "profile_h11", label: "How tall the eyeball seen in profile is 11 mm behind its front", low: "short in profile", high: "tall in profile", unit: "mm" },
    Proportion { key: "profile_m2", label: "How high the middle of the eyeball seen in profile sits 2 mm behind its front, against the eyeball's centre", low: "low in profile", high: "high in profile", unit: "mm" },
    Proportion { key: "profile_m5", label: "How high the middle of the eyeball seen in profile sits 5 mm behind its front, against the eyeball's centre", low: "low in profile", high: "high in profile", unit: "mm" },
    Proportion { key: "profile_m8", label: "How high the middle of the eyeball seen in profile sits 8 mm behind its front, against the eyeball's centre", low: "low in profile", high: "high in profile", unit: "mm" },
    Proportion { key: "profile_m11", label: "How high the middle of the eyeball seen in profile sits 11 mm behind its front, against the eyeball's centre", low: "low in profile", high: "high in profile", unit: "mm" },
    Proportion { key: "profile_len", label: "How far behind its front the eyeball still shows in profile", low: "short in profile", high: "long in profile", unit: "mm" },
    Proportion { key: "temple_len", label: "How long the eye's opening looks seen from 50 degrees toward the temple", low: "opening hidden toward the temple", high: "opening reaching far back", unit: "mm" },
    Proportion { key: "temple_tip", label: "How tall the opening seen from 50 degrees toward the temple is 1 mm from its outer end, against 6 mm from it", low: "pointed outer end", high: "blunt outer end", unit: "share" },
    Proportion { key: "front_tip", label: "How tall the opening seen from the front is 1 mm from its outer end, against 6 mm from it", low: "pointed outer corner", high: "blunt outer corner", unit: "share" },
    Proportion { key: "under_dip", label: "How deep the deepest hollow in the skin under the eye is, from the lower lid down to the cheekbone", low: "smooth, full skin under the eye", high: "hollow under the eye", unit: "mm" },
];

pub const OUTLINES: [OutlineDef; 2] = [
    OutlineDef { key: "upper", label: "Upper lid's edge", points: 13, widths: false },
    OutlineDef { key: "lower", label: "Lower lid's edge", points: 13, widths: false },
];

const GROUPS: [&str; 5] = ["whole", "size", "lids", "corners", "caruncle"];

const SHOTS: [Shot; 4] = [
    Shot { name: "both eyes", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-60.0, 60.0, -24.0, 26.0] },
    Shot { name: "one eye", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [8.0, 56.0, -16.0, 16.0] },
    Shot { name: "three-quarter", yaw: -35.0, pitch: 0.0, pivot: 0.0, window: [0.0, 52.0, -20.0, 20.0] },
    Shot { name: "side", yaw: -90.0, pitch: 0.0, pivot: 0.0, window: [-30.0, 26.0, -20.0, 20.0] },
];

/// One eye seen from above, looking down at 40 degrees and turned 20 degrees toward the temple.
pub const ABOVE: Shot = Shot { name: "above", yaw: -20.0, pitch: -40.0, pivot: 0.0, window: [4.0, 60.0, -24.0, 8.0] };

/// The window the measure reads.
const WINDOW: Window = Window { x: 58.0, y: [-26.0, 26.0], z: -30.0 };
/// The depth picture's step, in mm.
const PIXEL: f64 = 0.1;
/// The picture around each eye's centre: across and up, in mm.
const HALF_U: f64 = 21.0;
const HALF_V: f64 = 18.0;
/// A column of the opening counts toward the corners when it is at least this tall, in mm.
const CORNER_COLUMN: f64 = 0.5;
/// Our eyeball's radius in metres, and the distance between the base head's eyeballs, in metres.
pub const OUR_BALL_M: f64 = 0.0123;
const BASE_APART_M: f64 = 0.059_003;

/// An eyeball for the measure, in the frame of the eyes (mm).
#[derive(Clone, Debug)]
pub struct Ball {
    pub centre: V3,
    pub radius: f64,
    /// The eyeball's size for the measure, in mm on the head: our eyeball's radius, or for KOTOR the radius about our eye's centre that
    /// best matches the front of KOTOR's eyeball.
    pub size: f64,
    /// KOTOR's own eyeball triangles, when the eyeball is KOTOR's: the eye shows where they are in front, not the fitted sphere.
    pub tris: Vec<[V3; 3]>,
}

/// Our eyeballs in the frame of the eyes: at the frame's eye centres, with our radius of 12.3 mm at the frame's scale (`scale`, mm per
/// metre), or at the base head's when the scale is not known.
pub fn our_balls(scale: f64) -> [Ball; 2] {
    let r = OUR_BALL_M * if scale.is_finite() { scale } else { EYES_APART_MM / BASE_APART_M };
    [-1.0, 1.0].map(|s: f64| Ball { centre: [-s * 0.5 * EYES_APART_MM, 0.0, 0.0], radius: r, size: OUR_BALL_M * 1000.0, tris: Vec::new() })
}

/// The median distance of the 12 surface vertices nearest an eye's centre, in mm of the frame. They lie on the socket sheet and the
/// lids' lining, which cover the eyeball, so this follows the eyeball's radius.
fn socket_median(surface: &Surface, centre: V3) -> f64 {
    let mut d: Vec<f64> = surface.triangles.iter().flatten().map(|p| crate::length(crate::sub(*p, centre))).filter(|&r| r < 30.0).collect();
    d.sort_by(f64::total_cmp);
    d.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    if d.len() < 12 { f64::NAN } else { d[5..7].iter().sum::<f64>() / 2.0 }
}

/// The socket medians of the base head's smoothed surface (the +x eye first), in mm of the frame, where our eyeball has its own radius of 12.3 mm.
pub const BASE_SOCKET_MM: [f64; 2] = [13.0514, 13.1918];

/// Our eyeballs on a surface: at the frame's eye centres, with the radius read from the socket around them.
pub fn our_balls_on(surface: &Surface) -> [Ball; 2] {
    let scale = if surface.scale.is_finite() { surface.scale } else { EYES_APART_MM / BASE_APART_M };
    let mut balls = our_balls(scale);
    if let Some(eyes) = surface.eyes {
        for (b, c) in balls.iter_mut().zip(eyes) {
            b.centre = c;
        }
    }
    for (b, base) in balls.iter_mut().zip(BASE_SOCKET_MM) {
        let m = socket_median(surface, b.centre);
        if m.is_finite() {
            b.radius = OUR_BALL_M * scale + (m - base);
            b.size = b.radius * 1000.0 / scale;
        }
    }
    balls
}

/// The front-most depth of a surface around one eye, every `PIXEL` mm, NaN where there is none.
struct Picture {
    x0: f64,
    y0: f64,
    w: usize,
    h: usize,
    px: f64,
    z: Vec<f64>,
}

impl Picture {
    fn new(triangles: &[[V3; 3]], centre: V3) -> Self {
        Self::with_pixel(triangles, centre, PIXEL)
    }

    fn with_pixel(triangles: &[[V3; 3]], centre: V3, px: f64) -> Self {
        let (x0, y0) = (centre[0] - HALF_U, centre[1] - HALF_V);
        let w = (2.0 * HALF_U / px) as usize;
        let h = (2.0 * HALF_V / px) as usize;
        let mut z = vec![f64::NAN; w * h];
        for t in triangles {
            let q = t.map(|p| [(p[0] - x0) / px, (p[1] - y0) / px, p[2]]);
            let area = (q[1][0] - q[0][0]) * (q[2][1] - q[0][1]) - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1]);
            if area.abs() < 1e-12 {
                continue;
            }
            let lo = |c: usize| q.iter().map(|p| p[c]).fold(f64::MAX, f64::min);
            let hi = |c: usize| q.iter().map(|p| p[c]).fold(f64::MIN, f64::max);
            if hi(0) < 0.0 || hi(1) < 0.0 || lo(0) > w as f64 || lo(1) > h as f64 {
                continue;
            }
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
        Self { x0, y0, w, h, px, z }
    }

    fn place(&self, i: usize, j: usize) -> (f64, f64) {
        (self.x0 + (i as f64 + 0.5) * self.px, self.y0 + (j as f64 + 0.5) * self.px)
    }

    /// The depth at a place, from the nearest pixel.
    fn depth(&self, x: f64, y: f64) -> f64 {
        let (i, j) = (((x - self.x0) / self.px).floor(), ((y - self.y0) / self.px).floor());
        if i < 0.0 || j < 0.0 || i as usize >= self.w || j as usize >= self.h {
            return f64::NAN;
        }
        self.z[j as usize * self.w + i as usize]
    }
}

/// One eye's opening read from its picture: per column across (u toward the temple, every pixel), the top and bottom of the opening
/// (v, up from the eye's centre), or None where the opening has no pixel.
struct Opening {
    columns: Vec<Option<[f64; 2]>>,
    /// The u of column 0.
    u0: f64,
    /// The area of eyeball that shows but is not joined to the opening, in mm²: the eyeball poking through the skin.
    stray: f64,
}

fn opening(p: &Picture, ball: &Ball, temple: f64) -> Result<Opening, &'static str> {
    let (w, h) = (p.w, p.h);
    let mut shows = vec![false; w * h];
    let mesh_ball = (!ball.tris.is_empty()).then(|| Picture::new(&ball.tris, ball.centre));
    for j in 0..h {
        for i in 0..w {
            let (x, y) = p.place(i, j);
            let d2 = (x - ball.centre[0]).powi(2) + (y - ball.centre[1]).powi(2);
            if d2 >= ball.radius * ball.radius {
                continue;
            }
            let front = match &mesh_ball {
                Some(b) => b.z[j * w + i],
                None => ball.centre[2] + (ball.radius * ball.radius - d2).sqrt(),
            };
            if front.is_nan() {
                continue;
            }
            let z = p.z[j * w + i];
            shows[j * w + i] = z.is_nan() || z < front - 0.02;
        }
    }
    // the patch joined to the eyeball pixel nearest the eye's middle
    let (ci, cj) = (((ball.centre[0] - p.x0) / PIXEL) as i64, ((ball.centre[1] - p.y0) / PIXEL) as i64);
    let mut seed = None;
    'search: for r in 0..60i64 {
        for dj in -r..=r {
            for di in -r..=r {
                if di.abs() != r && dj.abs() != r {
                    continue;
                }
                let (i, j) = (ci + di, cj + dj);
                if i >= 0 && j >= 0 && (i as usize) < w && (j as usize) < h && shows[j as usize * w + i as usize] {
                    seed = Some(j as usize * w + i as usize);
                    break 'search;
                }
            }
        }
    }
    let seed = seed.ok_or("no eyeball shows")?;
    let mut inside = vec![false; w * h];
    inside[seed] = true;
    let mut stack = vec![seed];
    while let Some(at) = stack.pop() {
        let (i, j) = (at % w, at / w);
        let mut visit = |n: usize| {
            if shows[n] && !inside[n] {
                inside[n] = true;
                stack.push(n);
            }
        };
        if i > 0 {
            visit(at - 1);
        }
        if i + 1 < w {
            visit(at + 1);
        }
        if j > 0 {
            visit(at - w);
        }
        if j + 1 < h {
            visit(at + w);
        }
    }
    // columns in u from the nose to the temple
    let mut columns = vec![None; w];
    for i in 0..w {
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for j in 0..h {
            if inside[j * w + i] {
                let (_, y) = p.place(i, j);
                lo = lo.min(y - 0.5 * PIXEL);
                hi = hi.max(y + 0.5 * PIXEL);
            }
        }
        if lo < hi {
            let k = if temple > 0.0 { i } else { w - 1 - i };
            columns[k] = Some([lo - ball.centre[1], hi - ball.centre[1]]);
        }
    }
    let stray = (0..w * h).filter(|&a| shows[a] && !inside[a]).count() as f64 * PIXEL * PIXEL;
    Ok(Opening { columns, u0: -HALF_U + 0.5 * PIXEL, stray })
}

/// One eye's measure, with its drawable points in the frame of the eyes.
struct EyeMeasure {
    proportions: [f64; 47],
    stray: f64,
    upper: Vec<[f64; 2]>,
    lower: Vec<[f64; 2]>,
    dots: Vec<V3>,
}

fn measure_eye(opening_pic: &Picture, skin_pic: &Picture, ball: &Ball, temple: f64, with_lids: bool, profile: Profile, side: TempleView) -> Result<EyeMeasure, &'static str> {
    let profile_ball = profile.area;
    let size = ball.size;
    let o = opening(opening_pic, ball, temple)?;
    let u = |k: usize| o.u0 + k as f64 * PIXEL;
    let tall: Vec<usize> = (0..o.columns.len()).filter(|&k| o.columns[k].is_some_and(|c| c[1] - c[0] >= CORNER_COLUMN)).collect();
    let (&k_in, &k_out) = (tall.first().ok_or("no opening")?, tall.last().ok_or("no opening")?);
    if k_out - k_in < (8.0 / PIXEL) as usize {
        return Err("the opening is shorter than 8 mm");
    }
    let mid = |k: usize| o.columns[k].map(|c| 0.5 * (c[0] + c[1])).unwrap_or(f64::NAN);
    let (u_in, u_out) = (u(k_in), u(k_out));
    if std::env::var("EYES_DEBUG_OPENING").is_ok() {
        let col = |k: usize| o.columns[k].map(|c| format!("{:.1}..{:.1}", c[0], c[1])).unwrap_or("-".into());
        let first = (0..o.columns.len()).find(|&k| o.columns[k].is_some()).unwrap_or(0);
        eprintln!("opening r {:.2} first u {:.2} k_in u {:.2}: {}", ball.radius, u(first), u_in, (first..first + 40).step_by(4).map(|k| format!("{:.1}:{}", u(k), col(k))).collect::<Vec<_>>().join(" "));
    }
    let (v_in, v_out) = (mid(k_in), mid(k_out));
    let width = u_out - u_in;
    let chord = |uu: f64| v_in + (v_out - v_in) * (uu - u_in) / width;
    let (mut height, mut rise, mut drop, mut k_rise, mut k_drop) = (0.0f64, f64::MIN, f64::MIN, k_in, k_in);
    for k in k_in..=k_out {
        let Some(c) = o.columns[k] else { continue };
        height = height.max(c[1] - c[0]);
        if c[1] - chord(u(k)) > rise {
            rise = c[1] - chord(u(k));
            k_rise = k;
        }
        if chord(u(k)) - c[0] > drop {
            drop = chord(u(k)) - c[0];
            k_drop = k;
        }
    }
    // the edges at a place across, interpolated between the columns that have one
    let edge = |uu: f64, top: bool| -> f64 {
        let k = ((uu - o.u0) / PIXEL).round().clamp(k_in as f64, k_out as f64) as usize;
        let (mut a, mut b) = (k, k);
        while o.columns[a].is_none() && a > k_in {
            a -= 1;
        }
        while o.columns[b].is_none() && b < k_out {
            b += 1;
        }
        let pick = |kk: usize| o.columns[kk].map(|c| if top { c[1] } else { c[0] }).unwrap_or(f64::NAN);
        if a == b { pick(a) } else { 0.5 * (pick(a) + pick(b)) }
    };
    // the corners' angles: the angle the two lids make, seen from the corner, over 2 to 3 mm into the eye
    let corner_angle_at = |uc: f64, vc: f64, dir: f64, ds: [f64; 3]| -> f64 {
        let mut s = 0.0;
        for d in ds {
            let uu = uc + dir * d;
            s += ((edge(uu, true) - vc) / d).atan() + ((vc - edge(uu, false)) / d).atan();
        }
        (s / 3.0).to_degrees()
    };
    let corner_angle = |uc: f64, vc: f64, dir: f64| corner_angle_at(uc, vc, dir, [2.0, 2.5, 3.0]);
    let inner_angle = corner_angle(u_in, v_in, 1.0);
    let outer_angle = corner_angle(u_out, v_out, -1.0);
    let inner_round = corner_angle_at(u_in, v_in, 1.0, [0.6, 0.9, 1.2]) - inner_angle;
    let outer_round = corner_angle_at(u_out, v_out, -1.0, [0.6, 0.9, 1.2]) - outer_angle;
    // the lids' depth along the column through the middle of the opening, on the skin alone
    let um = 0.5 * (u_in + u_out);
    let xm = ball.centre[0] + temple * um;
    let (top, bottom) = (edge(um, true) + ball.centre[1], edge(um, false) + ball.centre[1]);
    // the upper lid's front as it shows: the lid piece where it stands in front of the skin
    let upper_depth = opening_pic.depth(xm, top + 3.0) - opening_pic.depth(xm, top + 7.0);
    let lower_depth = skin_pic.depth(xm, bottom - 2.5) - skin_pic.depth(xm, bottom - 6.0);
    let tilt = ((v_out - v_in) / width).atan().to_degrees();
    // how far round the eyeball each corner lies from its front, in mm along the eyeball at the corner's height
    let round_ball = |uu: f64, vv: f64| {
        let rh = (ball.radius * ball.radius - vv * vv).max(1e-6).sqrt();
        ball.radius * (uu / rh).clamp(-1.0, 1.0).asin()
    };
    let (reach_in, reach_out) = (round_ball(-u_in, v_in), round_ball(u_out, v_out));
    // how far the lid piece hangs below the skin's own upper edge, over the middle 40% of the eye
    let cover = if with_lids {
        match opening(skin_pic, ball, temple) {
            Ok(skin) => {
                let (mut sum, mut count) = (0.0, 0.0);
                for k in k_in..=k_out {
                    let share = (k - k_in) as f64 / (k_out - k_in) as f64;
                    if !(0.3..=0.7).contains(&share) {
                        continue;
                    }
                    if let (Some(a), Some(b)) = (skin.columns[k], o.columns[k]) {
                        sum += a[1] - b[1];
                        count += 1.0;
                    }
                }
                if count > 0.0 { sum / count } else { f64::NAN }
            }
            Err(_) => f64::NAN,
        }
    } else {
        0.0
    };
    let (q1, q3) = (u_in + 0.25 * width, u_in + 0.75 * width);
    let middle = |uu: f64| 0.5 * (edge(uu, true) + edge(uu, false));
    let extra = [
        size,
        edge(q1, true) - chord(q1),
        edge(q3, true) - chord(q3),
        chord(q1) - edge(q1, false),
        chord(q3) - edge(q3, false),
        v_in - middle(q1),
        v_out - middle(q3),
    ];
    let n = OUTLINES[0].points;
    let along = |i: usize| u_in + width * i as f64 / (n - 1) as f64;
    let line = |top: bool| -> Vec<[f64; 2]> {
        (0..n)
            .map(|i| {
                let uu = along(i);
                let v = if i == 0 { v_in } else if i == n - 1 { v_out } else { edge(uu, top) };
                [uu, v]
            })
            .collect()
    };
    let (upper, lower) = (line(true), line(false));
    // the whole eye: the middle of the opening's area, and the rim just outside the opening's edges (KOTOR's lid piece included)
    let (mut area, mut su, mut sv) = (0.0, 0.0, 0.0);
    for k in k_in..=k_out {
        if let Some(c) = o.columns[k] {
            let a = c[1] - c[0];
            area += a;
            su += a * u(k);
            sv += a * 0.5 * (c[0] + c[1]);
        }
    }
    let (cu, cv) = (su / area, sv / area);
    let spacing = 2.0 * temple * (ball.centre[0] + temple * cu);
    let level = ball.centre[1] + cv;
    let rim = |p: [f64; 2], out: f64| opening_pic.depth(ball.centre[0] + temple * p[0], ball.centre[1] + p[1] + out);
    let finite_mean = |v: Vec<f64>| -> f64 {
        let f: Vec<f64> = v.into_iter().filter(|x| x.is_finite()).collect();
        if f.is_empty() { f64::NAN } else { f.iter().sum::<f64>() / f.len() as f64 }
    };
    let inner_points = 1..n - 1;
    let depth = finite_mean(inner_points.clone().flat_map(|i| [rim(upper[i], 0.4), rim(lower[i], -0.4)]).collect());
    // the socket's angle: the skin 4 to 6 mm above the opening against the skin 3 to 5 mm below it, over the middle 8 mm of the eye
    let (mut dz, mut dy) = (Vec::new(), Vec::new());
    for k in 0..9 {
        let uu = um - 4.0 + k as f64;
        let x = ball.centre[0] + temple * uu;
        for off in [0.0, 1.0, 2.0] {
            let (ya, yb) = (edge(uu, true) + ball.centre[1] + 4.0 + off, edge(uu, false) + ball.centre[1] - 3.0 - off);
            dz.push(skin_pic.depth(x, yb) - skin_pic.depth(x, ya));
            dy.push(ya - yb);
        }
    }
    let pitch = finite_mean(dz).atan2(finite_mean(dy)).to_degrees();
    let to_local = |p: [f64; 2]| -> V3 {
        let (x, y) = (ball.centre[0] + temple * p[0], ball.centre[1] + p[1]);
        let z = skin_pic.depth(x, y);
        let d2 = (x - ball.centre[0]).powi(2) + (y - ball.centre[1]).powi(2);
        let front = ball.centre[2] + (ball.radius * ball.radius - d2).max(0.0).sqrt();
        [x, y, if z.is_nan() { front } else { z.max(front) }]
    };
    let dots = upper.iter().chain(&lower).map(|&p| to_local(p)).collect();
    // the eyeball's depth at a place of the opening, from KOTOR's eyeball mesh or from our sphere
    let mesh_ball = (!ball.tris.is_empty()).then(|| Picture::new(&ball.tris, ball.centre));
    let ball_z = |uu: f64, vv: f64| -> f64 {
        let (x, y) = (ball.centre[0] + temple * uu, ball.centre[1] + vv);
        match &mesh_ball {
            Some(b) => b.depth(x, y),
            None => {
                let d2 = (x - ball.centre[0]).powi(2) + (y - ball.centre[1]).powi(2);
                if d2 >= ball.radius * ball.radius { f64::NAN } else { ball.centre[2] + (ball.radius * ball.radius - d2).sqrt() }
            }
        }
    };
    // the eyeball's front over the whole opening, its mean depth along the opening's edge, and its mean depth down each corner's column
    let (mut ball_front, mut rim) = (f64::NEG_INFINITY, Vec::new());
    for k in k_in..=k_out {
        let Some(c) = o.columns[k] else { continue };
        let mut vv = c[0] + 0.5 * PIXEL;
        while vv < c[1] {
            ball_front = ball_front.max(ball_z(u(k), vv));
            vv += PIXEL;
        }
        rim.push(ball_z(u(k), c[0] + 0.5 * PIXEL));
        rim.push(ball_z(u(k), c[1] - 0.5 * PIXEL));
    }
    let column_depth = |k: usize| -> f64 {
        let Some(c) = o.columns[k] else { return f64::NAN };
        let mut d = Vec::new();
        let mut vv = c[0] + 0.5 * PIXEL;
        while vv < c[1] {
            d.push(ball_z(u(k), vv));
            vv += PIXEL;
        }
        finite_mean(d)
    };
    let ball_proud = ball_front - finite_mean(rim);
    // the inner corner of the skin's own opening (our caruncle and KOTOR's lid piece left out), and the highest skin 2 to 6 mm on its
    // nose side, within 2 mm above or below it
    let skin_open = if with_lids { opening(skin_pic, ball, temple).ok() } else { None };
    let skin_o = skin_open.as_ref().unwrap_or(&o);
    let k_skin = (0..skin_o.columns.len()).find(|&k| skin_o.columns[k].is_some_and(|c| c[1] - c[0] >= CORNER_COLUMN)).unwrap_or(k_in).min(k_in);
    let (u_sk, v_sk) = (u(k_skin), skin_o.columns[k_skin].map(|c| 0.5 * (c[0] + c[1])).unwrap_or(v_in));
    let mut skin_in = f64::NEG_INFINITY;
    for a in 0..=40 {
        for b in 0..=40 {
            let (uu, vv) = (u_sk - 2.0 - 0.1 * a as f64, v_sk - 2.0 + 0.1 * b as f64);
            let z = skin_pic.depth(ball.centre[0] + temple * uu, ball.centre[1] + vv);
            if z.is_finite() {
                skin_in = skin_in.max(z);
            }
        }
    }
    let nose_skin = skin_in - ball_front;
    // the inner corner's front: the eyeball, or our caruncle where it lies in front, down the skin opening's first column
    let inner_tip = {
        let Some(c) = skin_o.columns[k_skin] else { return Err("no inner corner") };
        let mut d = Vec::new();
        let mut vv = c[0] + 0.5 * PIXEL;
        while vv < c[1] {
            let front = opening_pic.depth(ball.centre[0] + temple * u(k_skin), ball.centre[1] + vv);
            let b = ball_z(u(k_skin), vv);
            d.push(if front.is_finite() && !(b > front) { front } else { b });
            vv += PIXEL;
        }
        finite_mean(d)
    } - ball_front;
    let outer_back = column_depth(k_out) - ball_front;
    // the highest skin 1 to 5 mm beyond the outer corner, within 3 mm above or below it, against the corner's column
    let mut skin_out = f64::NEG_INFINITY;
    for a in 0..=40 {
        for b in 0..=60 {
            let (uu, vv) = (u_out + 1.0 + 0.1 * a as f64, v_out - 3.0 + 0.1 * b as f64);
            let z = skin_pic.depth(ball.centre[0] + temple * uu, ball.centre[1] + vv);
            if z.is_finite() {
                skin_out = skin_out.max(z);
            }
        }
    }
    let outer_bulge = skin_out - column_depth(k_out);
    // the skin at the outer corner's tip: its mean depth over the first millimetre beyond the corner, within 0.5 mm above or below it
    let mut tip_out = Vec::new();
    for a in 0..=10 {
        for b in 0..=10 {
            tip_out.push(skin_pic.depth(ball.centre[0] + temple * (u_out + 0.1 * a as f64), ball.centre[1] + v_out - 0.5 + 0.1 * b as f64));
        }
    }
    let outer_tip = finite_mean(tip_out) - ball_front;
    let col_h = |k: Option<usize>| k.and_then(|k| o.columns[k]).map(|c| c[1] - c[0]).unwrap_or(0.0);
    let front_tip = col_h(k_out.checked_sub((1.0 / PIXEL) as usize)) / col_h(k_out.checked_sub((6.0 / PIXEL) as usize)).max(PIXEL);
    let under_dip = under_dip(skin_pic, opening_pic, ball, temple);
    Ok(EyeMeasure {
        stray: o.stray,
        proportions: [
            reach_in + reach_out,
            height,
            tilt,
            rise,
            drop,
            (u(k_rise) - u_in) / width,
            (u(k_drop) - u_in) / width,
            inner_angle,
            outer_angle,
            upper_depth,
            lower_depth,
            extra[0],
            extra[1],
            extra[2],
            extra[3],
            extra[4],
            extra[5],
            extra[6],
            spacing,
            level,
            depth,
            pitch,
            reach_in,
            reach_out,
            inner_round,
            outer_round,
            cover,
            nose_skin,
            inner_tip,
            outer_back,
            ball_proud,
            profile_ball,
            outer_bulge,
            outer_tip,
            profile.height[0],
            profile.height[1],
            profile.height[2],
            profile.height[3],
            profile.middle[0],
            profile.middle[1],
            profile.middle[2],
            profile.middle[3],
            profile.length,
            side.length,
            side.tip,
            front_tip,
            under_dip,
        ],
        upper,
        lower,
        dots,
    })
}

/// Measures both eyes of a surface, with the eyeballs given and with extra triangles (KOTOR's lid pieces) that hide the eyeball but are
/// not skin.
pub fn measure_with(surface: &Surface, balls: &[Ball; 2], lids: &[[V3; 3]]) -> Result<Measured, &'static str> {
    let mut eyes = Vec::with_capacity(2);
    for (s, ball) in balls.iter().enumerate() {
        let temple = if s == 0 { 1.0 } else { -1.0 };
        let skin_pic = Picture::new(&surface.triangles, ball.centre);
        let profile = profile_ball(surface.triangles.iter().chain(lids), ball, temple);
        let side = temple_view(surface.triangles.iter().chain(lids), ball, temple);
        let m = if lids.is_empty() {
            measure_eye(&skin_pic, &skin_pic, ball, temple, false, profile, side)?
        } else {
            let all: Vec<[V3; 3]> = surface.triangles.iter().chain(lids).copied().collect();
            measure_eye(&Picture::new(&all, ball.centre), &skin_pic, ball, temple, true, profile, side)?
        };
        eyes.push(m);
    }
    let proportions = (0..PROPORTIONS.len()).map(|i| 0.5 * (eyes[0].proportions[i] + eyes[1].proportions[i])).collect();
    let mean = |a: &[[f64; 2]], b: &[[f64; 2]]| -> Vec<[f64; 2]> { a.iter().zip(b).map(|(p, q)| [0.5 * (p[0] + q[0]), 0.5 * (p[1] + q[1])]).collect() };
    let outlines = vec![mean(&eyes[0].upper, &eyes[1].upper), mean(&eyes[0].lower, &eyes[1].lower)];
    let dots: Vec<V3> = eyes.iter().flat_map(|e| e.dots.iter().copied()).collect();
    let per_eye: Vec<Vec<f64>> = eyes.iter().map(|e| e.proportions.to_vec()).collect();
    let stray: Vec<f64> = eyes.iter().map(|e| e.stray).collect();
    Ok(Measured { proportions, outlines, marks: json!({"dots": dots, "per_eye": per_eye, "stray": stray}), cuts: Vec::new() })
}

/// The pixel of the profile picture, in mm.
const PROFILE_PIXEL: f64 = 0.2;

/// How far behind the eyeball's front the profile's columns are read, in mm.
const PROFILE_STATIONS: [f64; 4] = [2.0, 5.0, 8.0, 11.0];

/// One eyeball seen in profile from its own temple's side: the area that shows (mm²), and the outline of what shows, read in columns
/// `PROFILE_STATIONS` behind the eyeball's front: its height there (0 where none shows) and the height of its middle against the
/// eyeball's centre (NaN where none shows), and how far behind the front it still shows.
#[derive(Clone, Copy)]
struct Profile {
    area: f64,
    height: [f64; 4],
    middle: [f64; 4],
    length: f64,
}

/// The eyeball of one eye that shows in profile, seen from its own temple's side: its pixels in front of every triangle of `cover`
/// (the skin, and the lid pieces and caruncles).
fn profile_ball<'a>(cover: impl Iterator<Item = &'a [V3; 3]>, ball: &Ball, temple: f64) -> Profile {
    // across is the depth (forward), up is up, and the picture's depth is toward the temple
    let side = |p: V3| [p[2], p[1], temple * p[0]];
    let near = temple * ball.centre[0] - ball.radius - 1.0;
    let tris: Vec<[V3; 3]> = cover.filter(|t| t.iter().any(|p| temple * p[0] > near)).map(|t| t.map(side)).collect();
    let c = side(ball.centre);
    let pic = Picture::with_pixel(&tris, c, PROFILE_PIXEL);
    let mesh_ball = (!ball.tris.is_empty()).then(|| Picture::with_pixel(&ball.tris.iter().map(|t| t.map(side)).collect::<Vec<_>>(), c, PROFILE_PIXEL));
    let mut shown = Vec::new();
    for j in 0..pic.h {
        for i in 0..pic.w {
            let (x, y) = pic.place(i, j);
            let front = match &mesh_ball {
                Some(b) => b.z[j * b.w + i],
                None => {
                    let d2 = (x - c[0]).powi(2) + (y - c[1]).powi(2);
                    if d2 >= ball.radius * ball.radius { f64::NAN } else { c[2] + (ball.radius * ball.radius - d2).sqrt() }
                }
            };
            // the front half of the eyeball only: the measure's window cuts the head's sides off, so the back half may show through
            if front.is_nan() || x < c[0] {
                continue;
            }
            let z = pic.z[j * pic.w + i];
            if z.is_nan() || z < front - 0.02 {
                shown.push((x, y));
            }
        }
    }
    let ball_front = c[0] + ball.radius;
    let mut height = [0.0; 4];
    let mut middle = [f64::NAN; 4];
    for (k, s) in PROFILE_STATIONS.iter().enumerate() {
        let col: Vec<f64> = shown.iter().filter(|p| (p.0 - (ball_front - s)).abs() <= 0.5 * PROFILE_PIXEL + 1e-9).map(|p| p.1).collect();
        if col.is_empty() {
            continue;
        }
        let (lo, hi) = (col.iter().copied().fold(f64::MAX, f64::min), col.iter().copied().fold(f64::MIN, f64::max));
        height[k] = hi - lo + PROFILE_PIXEL;
        middle[k] = 0.5 * (hi + lo) - c[1];
    }
    let length = shown.iter().map(|p| ball_front - p.0).fold(0.0, f64::max);
    Profile { area: shown.len() as f64 * PROFILE_PIXEL * PROFILE_PIXEL, height, middle, length }
}

/// The eye's opening seen from 50 degrees toward its own temple: how long the eyeball that shows is (columns at least 0.3 mm tall), and
/// how tall it is 1 mm in from its outer end against 6 mm in.
#[derive(Clone, Copy)]
struct TempleView {
    length: f64,
    tip: f64,
}

fn temple_view<'a>(cover: impl Iterator<Item = &'a [V3; 3]>, ball: &Ball, temple: f64) -> TempleView {
    let (sn, cs) = 50f64.to_radians().sin_cos();
    let c = ball.centre;
    // across runs toward the temple and the back, the picture's depth toward the viewer
    let side = |p: V3| {
        let (dx, dz) = (temple * (p[0] - c[0]), p[2] - c[2]);
        [dx * cs - dz * sn, p[1], dx * sn + dz * cs]
    };
    let tris: Vec<[V3; 3]> = cover.filter(|t| t.iter().any(|p| temple * (p[0] - c[0]) > -40.0)).map(|t| t.map(side)).collect();
    let o = [0.0, c[1], 0.0];
    let pic = Picture::with_pixel(&tris, o, PIXEL);
    let mesh_ball = (!ball.tris.is_empty()).then(|| Picture::with_pixel(&ball.tris.iter().map(|t| t.map(side)).collect::<Vec<_>>(), o, PIXEL));
    let mut col = vec![(f64::MAX, f64::MIN); pic.w];
    for j in 0..pic.h {
        for i in 0..pic.w {
            let (x, y) = pic.place(i, j);
            let front = match &mesh_ball {
                Some(b) => b.z[j * b.w + i],
                None => {
                    let d2 = x * x + (y - c[1]).powi(2);
                    if d2 >= ball.radius * ball.radius { f64::NAN } else { (ball.radius * ball.radius - d2).sqrt() }
                }
            };
            if front.is_nan() {
                continue;
            }
            let z = pic.z[j * pic.w + i];
            if z.is_nan() || z < front - 0.02 {
                col[i] = (col[i].0.min(y), col[i].1.max(y));
            }
        }
    }
    let h: Vec<f64> = col.iter().map(|&(lo, hi)| if hi >= lo { hi - lo + PIXEL } else { 0.0 }).collect();
    let tall: Vec<usize> = (0..h.len()).filter(|&i| h[i] >= 0.3).collect();
    let (Some(&a), Some(&b)) = (tall.first(), tall.last()) else { return TempleView { length: 0.0, tip: f64::NAN } };
    let at = |mm: f64| b.checked_sub((mm / PIXEL) as usize).map(|i| h[i]).unwrap_or(0.0);
    TempleView { length: (b - a) as f64 * PIXEL, tip: at(1.0) / at(6.0).max(1.0) }
}

/// The deepest hollow in the skin under one eye seen from the front, in mm: along lines across 6, 9, 12 and 15 mm below the eyeball's
/// centre (from 10 mm toward the nose to 18 mm toward the temple) and down from 6 to 18 mm below it (under the centre and 9 mm toward
/// the temple), the largest gap between the skin's depth and the straight bridge over it (the line's upper convex hull). Places where
/// the eyeball or a lid piece (in `all`) lies in front of the skin are left out.
fn under_dip(skin: &Picture, all: &Picture, ball: &Ball, temple: f64) -> f64 {
    let mesh_ball = (!ball.tris.is_empty()).then(|| Picture::new(&ball.tris, ball.centre));
    const LINES: [(f64, f64, f64, f64); 6] =
        [(-10.0, -6.0, 18.0, -6.0), (-10.0, -9.0, 18.0, -9.0), (-10.0, -12.0, 18.0, -12.0), (-10.0, -15.0, 18.0, -15.0), (0.0, -6.0, 0.0, -18.0), (9.0, -6.0, 9.0, -18.0)];
    let mut worst = 0.0f64;
    for (u0, v0, u1, v1) in LINES {
        let len = (u1 - u0).hypot(v1 - v0);
        let n = (len / 0.25) as usize;
        let pts: Vec<(f64, f64)> = (0..=n)
            .filter_map(|k| {
                let t = k as f64 / n as f64;
                let (x, y) = (ball.centre[0] + temple * (u0 + t * (u1 - u0)), ball.centre[1] + v0 + t * (v1 - v0));
                let d = skin.depth(x, y);
                // not where the eyeball or a lid piece lies in front of the skin: the line would look through the opening
                let eye = match &mesh_ball {
                    Some(b) => b.depth(x, y),
                    None => {
                        let d2 = (x - ball.centre[0]).powi(2) + (y - ball.centre[1]).powi(2);
                        if d2 < ball.radius * ball.radius { ball.centre[2] + (ball.radius * ball.radius - d2).sqrt() } else { f64::NAN }
                    }
                };
                (d.is_finite() && !(eye > d) && !(all.depth(x, y) > d + 0.05)).then_some((t * len, d))
            })
            .collect();
        let mut hull: Vec<(f64, f64)> = Vec::new();
        for &p in &pts {
            while hull.len() >= 2 {
                let (a, b) = (hull[hull.len() - 2], hull[hull.len() - 1]);
                if (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0) >= 0.0 {
                    hull.pop();
                } else {
                    break;
                }
            }
            hull.push(p);
        }
        if hull.len() < 2 {
            continue;
        }
        for &(x, d) in &pts {
            let i = hull.iter().position(|q| q.0 >= x).unwrap_or(hull.len() - 1).max(1);
            let (a, b) = (hull[i - 1], hull[i]);
            let top = if b.0 > a.0 { a.1 + (x - a.0) / (b.0 - a.0) * (b.1 - a.1) } else { a.1 };
            worst = worst.max(top - d);
        }
    }
    worst
}

pub fn measure(surface: &Surface) -> Result<Measured, &'static str> {
    measure_with(surface, &our_balls_on(surface), &surface.lids)
}

/// The slider that lowers the open upper lid piece over the eyeball, in mm.
pub const COVER_SLIDER: &str = "Upper lid cover";

/// Our generated upper lid pieces' triangles on a built head (positions in metres), in `frame`, with the open piece lowered by `lower`
/// mm (the slider Upper lid cover), as viewer 2 builds them (open 0, pivot at the eyeball's centre). Empty when the lid rig is not
/// prepared (`eye_curves::init`) or `EYES_MEASURE_PIECE=0` (native only).
#[cfg(target_arch = "wasm32")]
pub fn piece_triangles(_positions: &[V3], _frame: &Frame, _shown: [f64; 2]) -> Vec<[V3; 3]> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn piece_triangles(positions: &[V3], frame: &Frame, shown: [f64; 2]) -> Vec<[V3; 3]> {
    let lower = shown[0];
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var("EYES_MEASURE_PIECE").is_ok_and(|v| v == "0") {
        return Vec::new();
    }
    let Some(rig) = crate::eye_curves::lid_rig() else { return Vec::new() };
    if positions.len() != crate::measure::OUR_VERTEX_COUNT {
        return Vec::new();
    }
    let settings = head_eyes::piece::Settings { lower, ..head_eyes::piece::Settings::default() };
    let pieces = head_eyes::piece::build(rig, positions, &settings);
    let mut out = Vec::new();
    for p in &pieces {
        let at = |i: u32| {
            let q = p.positions[i as usize];
            frame.local([q[0] as f64, q[1] as f64, q[2] as f64])
        };
        out.extend(p.triangles.iter().map(|t| t.map(at)));
    }
    // the caruncles fill the inner corners, so the eyeball shows only up to them. A caruncle lies on the eyeball, so the measure sees it
    // 0.3 mm further forward, where it hides the eyeball behind it
    let car_t = head_eyes::caruncle::triangles();
    for e in crate::eye_curves::eye_edges() {
        let q: Vec<V3> = head_eyes::caruncle::positions(positions, e, shown[1])
            .iter()
            .map(|&p| {
                let l = frame.local(p);
                [l[0], l[1], l[2] + 0.3]
            })
            .collect();
        if std::env::var("EYES_DEBUG_CARUNCLE").is_ok() {
            let lo = q.iter().fold([f64::MAX; 3], |a, p| [a[0].min(p[0]), a[1].min(p[1]), a[2].min(p[2])]);
            let hi = q.iter().fold([f64::MIN; 3], |a, p| [a[0].max(p[0]), a[1].max(p[1]), a[2].max(p[2])]);
            eprintln!("caruncle {} points {} triangles, from {lo:.1?} to {hi:.1?}", q.len(), car_t.len());
        }
        out.extend(car_t.iter().map(|t| t.map(|i| q[i as usize])));
    }
    out
}

/// The values of Upper lid cover and Caruncle width in a set's values, each 0 when the set has no such slider: what the lid pieces
/// and the caruncles, which move no vertex of the head, read.
pub fn cover_of(set: &crate::set::SliderSet, values: &[f64]) -> [f64; 2] {
    [COVER_SLIDER, "Caruncle width"].map(|n| set.sliders.iter().position(|s| s.name == n).and_then(|j| values.get(j).copied()).unwrap_or(0.0))
}

/// The surface of a built head as viewer 2 shows it, for the eyes' measure: the skin, the eyeballs at their centre rings, and the
/// generated upper lid pieces lowered by `shown[0]` mm, and the caruncles at width `shown[1]`.
pub fn shown_surface(positions: &[V3], triangles: &[[u32; 3]], frame: &Frame, shown: [f64; 2]) -> Surface {
    let mut s = Surface::within(positions, triangles, frame, &WINDOW);
    s.lids = piece_triangles(positions, frame, shown);
    s
}

/// KOTOR's own eyeballs and lid pieces, from `sliders/eyes/kotor_eyes.json` (written by `prep_eyes.py`), in the fit's frame (metres).
#[cfg(not(target_arch = "wasm32"))]
pub mod kotor {
    use crate::V3;
    use std::collections::HashMap;
    use std::sync::OnceLock;

    pub struct KotorEye {
        pub centre: V3,
        pub radius: f64,
        pub lid: Vec<[V3; 3]>,
        pub ball: Vec<[V3; 3]>,
    }

    static EYES: OnceLock<HashMap<String, Vec<KotorEye>>> = OnceLock::new();

    pub fn path() -> std::path::PathBuf {
        std::env::var("EYES_KOTOR").map(Into::into).unwrap_or_else(|_| {
            std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".claude/tools/likeness-fit-data/sliders/eyes/kotor_eyes.json")
        })
    }

    pub fn all() -> &'static HashMap<String, Vec<KotorEye>> {
        EYES.get_or_init(|| {
            let Ok(text) = std::fs::read_to_string(path()) else { return HashMap::new() };
            let v: serde_json::Value = serde_json::from_str(&text).expect("kotor_eyes.json");
            let tris = |x: &serde_json::Value| -> Vec<[V3; 3]> { serde_json::from_value(x.clone()).unwrap_or_default() };
            v.as_object()
                .unwrap()
                .iter()
                .map(|(name, eyes)| {
                    let list = eyes
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|e| !e.is_null())
                        .map(|e| KotorEye {
                            centre: serde_json::from_value(e["centre"].clone()).unwrap(),
                            radius: e["radius"].as_f64().unwrap(),
                            lid: tris(&e["lid"]),
                            ball: tris(&e["ball"]),
                        })
                        .collect();
                    (name.clone(), list)
                })
                .collect()
        })
    }
}

/// The radius, in mm of the frame, of a ball about our eye's centre (the frame's) that best matches the front of a KOTOR eyeball
/// (centre and radius in the frame): the mean distance from our centre of the KOTOR eyeball's points within 35 degrees of straight
/// ahead. KOTOR's large eyeballs sit 5 to 10 mm further back than ours, so their front is much less large than their radius.
fn front_match(centre: V3, radius: f64, ours: V3) -> f64 {
    let (mut s, mut n) = (0.0, 0.0);
    for i in 0..8 {
        let a = (35.0f64 * (i as f64 + 0.5) / 8.0).to_radians();
        for j in 0..16 {
            let b = std::f64::consts::TAU * j as f64 / 16.0;
            let p = [centre[0] + radius * a.sin() * b.cos(), centre[1] + radius * a.sin() * b.sin(), centre[2] + radius * a.cos()];
            let w = a.sin();
            s += w * crate::length(crate::sub(p, ours));
            n += w;
        }
    }
    s / n
}

/// Our eye's centre on the side of `centre` (frame, mm): the fit's eyeball centre when known, else the base head's in the frame of the eyes.
fn ours_near(fit_eyes: Option<[V3; 2]>, centre: V3) -> V3 {
    match fit_eyes {
        Some(e) => e[if centre[0] > 0.0 { 0 } else { 1 }],
        None => [if centre[0] > 0.0 { 0.5 } else { -0.5 } * EYES_APART_MM, 0.0, 0.0],
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn fit_of(input: &crate::mesh::input::Input, name: &str) -> Vec<V3> {
    input.points(&format!("heads/{name}/fit_smooth"))
}

fn our_eyes_of(fit: &[V3], frame: &Frame) -> Option<[V3; 2]> {
    crate::measure::our_eyes(fit).map(|e| e.map(|p| frame.local(p)))
}

/// Our eyeball's radius on a head mesh (metres), read from the 8 eyeball-centre vertices nearest `centre`, which the eyeball size
/// slider scales with the radius (as `head_eyes::piece::eyeball_radius`).
pub fn ring_radius_m(positions: &[V3], centre: V3) -> f64 {
    let mut d: Vec<(f64, V3)> = positions.iter().map(|&p| (crate::length(crate::sub(p, centre)), p)).collect();
    d.sort_by(|a, b| a.0.total_cmp(&b.0));
    let ring: Vec<V3> = d[..8].iter().map(|x| x.1).collect();
    let c = ring.iter().fold([0.0; 3], |a, &p| crate::add(a, crate::scale(p, 1.0 / 8.0)));
    let ms = ring.iter().map(|&p| crate::length(crate::sub(p, c)).powi(2)).sum::<f64>() / 8.0;
    OUR_BALL_M * ms.sqrt() * 1000.0 / head_eyes_ring_mm()
}

const fn head_eyes_ring_mm() -> f64 {
    6.907
}

fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The reach of the eye paste, in mm from each eye's centre: the oval taken whole (`across` toward the temple, `nose` toward the
/// nose, `up` up and down), how far past it the paste fades out (a share of the oval), the heights where it has faded out above and
/// below, and the length of that fade. `EYES_PASTE_SHAPE` (native only, for trials) gives the seven numbers, joined by commas.
#[derive(Clone, Copy)]
struct PasteShape {
    across: f64,
    nose: f64,
    up: f64,
    fade: f64,
    top: f64,
    bottom: f64,
    edge: f64,
}

impl PasteShape {
    fn get() -> Self {
        let mut s = Self { across: 17.0, nose: 22.0, up: 7.0, fade: 0.5, top: 9.0, bottom: 13.0, edge: 3.0 };
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(v) = std::env::var("EYES_PASTE_SHAPE") {
            let n: Vec<f64> = v.split(',').filter_map(|x| x.parse().ok()).collect();
            if n.len() == 6 {
                s = Self { across: n[0], nose: n[0], up: n[1], fade: n[2], top: n[3], bottom: n[4], edge: n[5] };
            }
            if n.len() == 7 {
                s = Self { across: n[0], nose: n[1], up: n[2], fade: n[3], top: n[4], bottom: n[5], edge: n[6] };
            }
        }
        s
    }
}

/// Per vertex, how much of the base head's eye area replaces a stand-in's (`paste_start_mesh`): 1 within an oval 17 mm toward the
/// temple, 22 mm toward the nose, 7 mm up and 10 mm down from each eye's centre (the lids, the corners and the socket's rim), fading
/// to 0 at 1.5 times that oval, and over 3 mm to 9 mm above the eye's centre and 13 mm below it, so that the brow's lower edge and the
/// cheekbone keep the stand-in's shape. The oval reaches lower than it reaches up because the outer corner's slider turns the skin
/// close to the eyeball under the corner, which tore on the stand-ins' own shape there. Round 4's paste reached 14 mm up and 13 mm down and overwrote both. The paste also fades out
/// from 20 to 25 mm behind the eye's centre. The second value marks the oval itself.
fn paste_weights(base: &[V3], eyes: [V3; 2]) -> Vec<(f64, bool)> {
    let frame = Frame::from_eyes(eyes);
    let s = PasteShape::get();
    base.iter()
        .map(|&b| {
            let p = frame.local(b);
            let (u, up) = (p[0].abs() - 0.5 * EYES_APART_MM, p[1]);
            let a = if u < 0.0 { s.nose } else { s.across };
            let e = ((u / a).powi(2) + (up / if up < 0.0 { paste_down(s.up) } else { s.up }).powi(2)).sqrt();
            let w = smoothstep((1.0 + s.fade - e) / s.fade)
                * smoothstep((s.top - up) / s.edge)
                * smoothstep((up + s.bottom) / s.edge)
                * smoothstep((p[2] + 25.0) / 5.0);
            (w, e < 1.0 && p[2] > -20.0)
        })
        .collect()
}

/// Whether stand-ins get the base head's eye area (`EYES_PASTE=0`, native only, keeps the fit's own eye area, as before round 4).
pub fn paste_on() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var("EYES_PASTE").is_ok_and(|v| v == "0") {
        return false;
    }
    true
}

/// A stand-in with its eye area replaced by the base head's. Each eye is carried by one translation, the mean movement of the stand-in
/// from the base head over that eye's oval, so inside the oval the stand-in is the base head moved rigidly, and the eyeball's centre
/// vertices move with the lids. The weight fades the base head's eye area into the stand-in around it.
pub fn paste_start(base: &[V3], eyes: [V3; 2], start: &[V3]) -> Vec<V3> {
    let weights = paste_weights(base, eyes);
    let frame = Frame::from_eyes(eyes);
    let side = |v: usize| if frame.local(base[v])[0] > 0.0 { 0 } else { 1 };
    let mut t = [[0.0; 3]; 2];
    let mut n = [0.0; 2];
    for v in 0..base.len() {
        if weights[v].1 {
            let s = side(v);
            t[s] = crate::add(t[s], crate::sub(start[v], base[v]));
            n[s] += 1.0;
        }
    }
    let t = [crate::scale(t[0], 1.0 / n[0]), crate::scale(t[1], 1.0 / n[1])];
    (0..base.len())
        .map(|v| {
            let target = crate::add(base[v], t[side(v)]);
            crate::add(start[v], crate::scale(crate::sub(target, start[v]), weights[v].0))
        })
        .collect()
}

/// Whether the paste carries the eye area by one translation per eye, as in round 4 (`EYES_PASTE_MODE=rigid`, native only).
fn paste_rigid() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var("EYES_PASTE_MODE").is_ok_and(|v| v == "rigid") {
        return true;
    }
    false
}

/// The oval round each eye's centre (mm toward the temple, toward the nose, and up) whose movement is one affine map: the paste's
/// own oval. `EYES_PASTE_CORE` (native only, for trials) gives the three numbers; 0 toward the temple fills the oval too.
/// How far down from each eye's centre the paste's oval reaches, in mm, given how far up it reaches (`EYES_PASTE_DOWN`, native only,
/// for trials).
fn paste_down(up: f64) -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(v) = std::env::var("EYES_PASTE_DOWN").ok().and_then(|v| v.parse::<f64>().ok()) {
        return v;
    }
    up + 3.0
}

fn paste_core() -> (f64, f64, f64) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(v) = std::env::var("EYES_PASTE_CORE") {
        let n: Vec<f64> = v.split(',').filter_map(|x| x.parse().ok()).collect();
        if n.len() == 3 {
            return (n[0], n[1], n[2]);
        }
    }
    (17.0, 22.0, 7.0)
}

/// A stand-in with its eye area replaced by the base head's (round 5). The stand-in's movement from the base head is kept where the
/// paste's weight is 0. Inside the eye area, the movement is first filled in from the area's border as smoothly as it can be (each
/// merged vertex takes the mean movement of its neighbours). Over the oval (`paste_core`) it is then replaced by its best affine map,
/// so places close together move alike even across the lids' folds and no lid is torn, and the band around the oval is filled again.
/// The stand-in's own lids, corners and folds are gone, and the base head's eye area sits at the depth, the place and the tilt that the
/// stand-in's brow, cheek and nose around it give, without a step. The eyeball's centre vertices move with the oval. Hidden vertices
/// follow their nearest skin. The paste's weight blends the result into the stand-in's own movement. Round 4 moved the whole area by
/// one translation and reached 14 mm up and 13 mm down, which overwrote the brow's lower edge and the top of the cheek.
pub fn paste_start_mesh(mesh: &HeadMesh, start: &[V3]) -> Vec<V3> {
    if paste_rigid() {
        return paste_start(&mesh.base, mesh.eyes, start);
    }
    let base = &mesh.base;
    let weights = paste_weights(base, mesh.eyes);
    let frame = Frame::from_eyes(mesh.eyes);
    let (ca, cn, cu) = paste_core();
    let n = mesh.merged_count();
    let own: Vec<V3> = start.iter().zip(base).map(|(s, b)| crate::sub(*s, *b)).collect();
    let mut d = mesh.merge(&own);
    let local: Vec<V3> = (0..n).map(|g| frame.local(base[mesh.first[g] as usize])).collect();
    let side = |g: usize| if local[g][0] > 0.0 { 0 } else { 1 };
    let core: Vec<bool> = (0..n)
        .map(|g| {
            let p = local[g];
            let u = p[0].abs() - 0.5 * EYES_APART_MM;
            let a = if u < 0.0 { cn } else { ca };
            let b = if p[1] < 0.0 { paste_down(cu) } else { cu };
            ca > 0.0 && (u / a).powi(2) + (p[1] / b).powi(2) < 1.0 && p[2] > -20.0
        })
        .collect();
    // the fill: many sweeps of neighbour means over the skin that is not held. Hidden vertices (the linings behind the lids) take no
    // part: they follow the skin nearest them afterwards, so they never poke through it.
    let skin_nb: Vec<Vec<u32>> = (0..n).map(|g| mesh.neighbours_of(g).iter().copied().filter(|&h| !mesh.hidden[h as usize]).collect()).collect();
    let fill = |d: &mut Vec<V3>, held: &[bool]| {
        let order: Vec<usize> = (0..n).filter(|&g| !held[g] && !mesh.hidden[g] && !skin_nb[g].is_empty()).collect();
        for _ in 0..2000 {
            for &g in &order {
                let nb = &skin_nb[g];
                let m = nb.iter().fold([0.0; 3], |a, &h| crate::add(a, d[h as usize]));
                d[g] = crate::scale(m, 1.0 / nb.len() as f64);
            }
        }
    };
    let outside: Vec<bool> = (0..n).map(|g| weights[mesh.first[g] as usize].0 <= 0.0).collect();
    fill(&mut d, &outside);
    if ca > 0.0 {
        // inside the oval the movement is the best affine map of the filled movement (a translation, a turn, a stretch), which moves
        // places close together alike even across the lids' folds, so no lid is torn; then the band around it is filled again
        for side_k in 0..2 {
            let ids: Vec<usize> = (0..n).filter(|&g| core[g] && side(g) == side_k && !mesh.hidden[g]).collect();
            if ids.len() < 8 {
                continue;
            }
            let mut ata = nalgebra::Matrix4::<f64>::zeros();
            let mut atb = [nalgebra::Vector4::<f64>::zeros(); 3];
            for &g in &ids {
                let p = local[g];
                let row = nalgebra::Vector4::new(p[0] / 10.0, p[1] / 10.0, p[2] / 10.0, 1.0);
                ata += row * row.transpose();
                for c in 0..3 {
                    atb[c] += row * d[g][c];
                }
            }
            ata += nalgebra::Matrix4::identity() * 1e-9;
            let Some(inv) = ata.try_inverse() else { continue };
            let coef: Vec<nalgebra::Vector4<f64>> = (0..3).map(|c| inv * atb[c]).collect();
            for g in (0..n).filter(|&g| core[g] && side(g) == side_k) {
                let p = local[g];
                let row = nalgebra::Vector4::new(p[0] / 10.0, p[1] / 10.0, p[2] / 10.0, 1.0);
                d[g] = [coef[0].dot(&row), coef[1].dot(&row), coef[2].dot(&row)];
            }
        }
        let held: Vec<bool> = (0..n).map(|g| core[g] || outside[g]).collect();
        fill(&mut d, &held);
    }
    let own_m = mesh.merge(&own);
    let w_m: Vec<f64> = (0..n).map(|g| weights[mesh.first[g] as usize].0).collect();
    let blended: Vec<V3> = (0..n).map(|g| crate::add(crate::scale(own_m[g], 1.0 - w_m[g]), crate::scale(d[g], w_m[g]))).collect();
    let spread = mesh.expand(&blended);
    let mut out: Vec<V3> = (0..base.len())
        .map(|v| {
            let g = mesh.group[v] as usize;
            let m = if mesh.hidden[g] { crate::add(crate::scale(own[v], 1.0 - w_m[g]), crate::scale(spread[v], w_m[g])) } else { spread[v] };
            crate::add(base[v], m)
        })
        .collect();
    // the eyeball's centre vertices go with their eye's lid edge (or with the rigid middle, when there is one)
    for e in crate::eye_curves::edges() {
        let m = e.edge.iter().fold([0.0; 3], |a, &v| crate::add(a, crate::sub(out[v as usize], base[v as usize])));
        let m = crate::scale(m, 1.0 / e.edge.len() as f64);
        for &v in &e.ring {
            let g = mesh.group[v as usize] as usize;
            let t = if ca > 0.0 { d[g] } else { m };
            out[v as usize] = crate::add(base[v as usize], t);
        }
    }
    // the base head's inner corners sit deep, wrapped round the eyeball; the pasted eye area has them brought forward toward the
    // face's surface (`corner_forward`), as Inner corner depth would
    let forward = corner_forward();
    if forward != 0.0 {
        let f = crate::eye_curves::field(base, crate::eye_curves::Curve::CornerDepth { inner: true }, forward, &crate::eye_curves::Fades::default());
        for v in 0..base.len() {
            out[v] = crate::add(out[v], f[v]);
        }
    }
    out
}

/// How far forward the pasted eye area's inner corners are brought, in mm (`EYES_CORNER_FORWARD`, native only, for trials).
fn corner_forward() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(v) = std::env::var("EYES_CORNER_FORWARD").ok().and_then(|v| v.parse().ok()) {
        return v;
    }
    2.5
}

/// Where a place on the base head lies from the eye on its side: across toward the temple and up from the eye's centre, in mm.
fn eye_coords(p: V3) -> (f64, f64) {
    let c = 0.5 * EYES_APART_MM;
    (p[0].abs() - c, p[1])
}

/// Bumps go on the side of the face with the +x eye only; the kit mirrors each one.
fn allowed(p: V3) -> bool {
    p[0] > 1.0
}

impl Feature for Eyes {
    fn key(&self) -> &'static str {
        "eyes"
    }
    fn title(&self) -> &'static str {
        "Eyes"
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
        Window { x: 50.0, y: [-12.0, 12.0], z: -5.0 }
    }
    fn measure(&self, surface: &Surface) -> Result<Measured, &'static str> {
        measure(surface)
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn measure_kotor(&self, surface: &Surface, extras: &KotorExtras, frame: &Frame, _positions: &[V3]) -> Result<Measured, &'static str> {
        let Some(eyes) = extras.name.and_then(|n| kotor::all().get(n)) else { return Err("no KOTOR eyeball for this mesh") };
        if eyes.len() != 2 {
            return Err("KOTOR's eyeballs were not found");
        }
        let fit_eyes = extras.fit.and_then(crate::measure::our_eyes).map(|e| e.map(|p| frame.local(p)));
        let ours = |c: V3| ours_near(fit_eyes, c);
        let mut balls = [0, 1].map(|k| Ball {
            centre: frame.local(eyes[k].centre),
            radius: eyes[k].radius * frame.scale,
            size: front_match(frame.local(eyes[k].centre), eyes[k].radius * frame.scale, ours(frame.local(eyes[k].centre))) / frame.scale * 1000.0,
            tris: eyes[k].ball.iter().map(|t| t.map(|p| frame.local(p))).collect(),
        });
        // the +x eye first
        if balls[0].centre[0] < balls[1].centre[0] {
            balls.swap(0, 1);
        }
        let lids: Vec<[V3; 3]> = eyes.iter().flat_map(|e| e.lid.iter().map(|t| t.map(|p| frame.local(p)))).collect();
        measure_with(surface, &balls, &lids)
    }
    /// The skin round each eye: 1 within an oval 17 mm across and 11 mm up and down from the eye's centre, fading to 0 at 1.7 times
    /// that oval. Above the eye it stops 14 mm up, under the brow, and below the eye 13 mm down, at the top of the cheek. The lids'
    /// vertices that the smoothing holds still do move with the sliders (the smoothing then holds them where the sliders put them); only
    /// the eyeballs' centre vertices have 0.
    fn zone(&self, mesh: &HeadMesh, _base: &Measured) -> Zone {
        let frame = Frame::from_eyes(mesh.eyes);
        let n = mesh.merged_count();
        let weight = (0..n)
            .map(|g| {
                let v = mesh.first[g] as usize;
                let p = frame.local(mesh.base[v]);
                // the eyeballs' centre vertices, held by the smoothing and inside the eyeball
                if mesh.keep[v] && !mesh.hidden[g] && (p[0].abs() - 0.5 * EYES_APART_MM).hypot(p[1]).hypot(p[2]) < 9.0 {
                    return 0.0;
                }
                let (u, up) = eye_coords(p);
                let e = ((u / 17.0).powi(2) + (up / 11.0).powi(2)).sqrt();
                smoothstep((1.7 - e) / 0.7) * smoothstep((14.0 - up) / 3.0) * smoothstep((up + 13.0) / 3.0)
            })
            .collect();
        Zone { weight }
    }
    /// The parts round the eye on the base head: the inner corner (more than 9.5 mm toward the nose from the eye's centre), the outer
    /// corner (more than 9.5 mm toward the temple), and between them the upper lid (above the centre) and the lower lid (below it).
    fn part_of(&self, p: V3, _base: &Measured) -> &'static str {
        let (u, up) = eye_coords(p);
        if u < -9.5 {
            "inner corner"
        } else if u > 9.5 {
            "outer corner"
        } else if up >= 0.0 {
            if u < -3.5 { "upper lid inner" } else if u > 3.5 { "upper lid outer" } else { "upper lid" }
        } else if u < -3.5 {
            "lower lid inner"
        } else if u > 3.5 {
            "lower lid outer"
        } else {
            "lower lid"
        }
    }
    fn bumps(&self) -> BumpRule {
        BumpRule { size_mm: 4.0, spacing_mm: 3.5, min_weight: 0.5, allowed }
    }
    fn miss_words(&self, i: usize, g: f64) -> String {
        let a = g.abs();
        let (more, less) = match PROPORTIONS[i].key {
            "width" => ("The eye is too long", "The eye is too short"),
            "height" => ("The opening is too tall", "The opening is too narrow"),
            "tilt" => ("The outer corner sits too high", "The outer corner sits too low"),
            "upper_rise" => ("The upper lid arches too much", "The upper lid is too flat"),
            "lower_drop" => ("The lower lid curves down too much", "The lower lid is too flat"),
            "upper_peak" => ("The upper lid peaks too near the temple", "The upper lid peaks too near the nose"),
            "lower_low" => ("The lower lid dips too near the temple", "The lower lid dips too near the nose"),
            "inner_angle" => ("The inner corner is too round", "The inner corner is too sharp"),
            "outer_angle" => ("The outer corner is too round", "The outer corner is too sharp"),
            "upper_depth" => ("The upper lid is too full", "The upper lid sits too deep"),
            "lower_depth" => ("The lower lid is too full", "The lower lid sits too deep"),
            "ball" => ("The eyeball is too large", "The eyeball is too small"),
            "upper_inner" => ("The upper lid stands too high near the nose", "The upper lid sits too low near the nose"),
            "upper_outer" => ("The upper lid stands too high near the temple", "The upper lid sits too low near the temple"),
            "lower_inner" => ("The lower lid sits too low near the nose", "The lower lid sits too high near the nose"),
            "lower_outer" => ("The lower lid sits too low near the temple", "The lower lid sits too high near the temple"),
            "inner_height" => ("The inner corner sits too high", "The inner corner sits too low"),
            "outer_lift" => ("The outer corner is lifted too much", "The outer corner droops too much"),
            "inner_reach" => ("The inner corner reaches too far toward the nose", "The inner corner stops short of the nose"),
            "outer_reach" => ("The outer corner reaches too far toward the temple", "The outer corner stops short of the temple"),
            "cover" => ("The upper lid piece hangs too low over the eye", "The upper lid piece sits too high"),
            "nose_skin" => ("The inner corner is buried too deep behind the skin beside the nose", "The inner corner stands too far out from the skin beside the nose"),
            "inner_tip" => ("The inner corner sits too far forward", "The inner corner sits too far back"),
            "outer_back" => ("The outer corner sits too far forward", "The outer corner sits too far back"),
            "ball_proud" => ("The eyeball stands out too much", "The eyeball sits too flat in the opening"),
            "profile_ball" => ("Too much eyeball shows in profile", "Too little eyeball shows in profile"),
            "outer_bulge" => ("The skin bulges too much round the outer corner", "The skin round the outer corner sits too flat"),
            "outer_tip" => ("The outer corner's end sits too far forward", "The outer corner's end sits too far back"),
            "profile_h2" | "profile_h5" | "profile_h8" | "profile_h11" => ("The eyeball seen in profile is too tall there", "The eyeball seen in profile is too short there"),
            "profile_m2" | "profile_m5" | "profile_m8" | "profile_m11" => ("The eyeball seen in profile sits too high there", "The eyeball seen in profile sits too low there"),
            "profile_len" => ("The eyeball shows too far back in profile", "The eyeball stops too soon in profile"),
            "temple_len" => ("The opening reaches too far back, seen from the temple side", "The opening stops too soon, seen from the temple side"),
            "temple_tip" => ("The outer end is too blunt, seen from the temple side", "The outer end is too pointed, seen from the temple side"),
            "front_tip" => ("The outer corner is too blunt", "The outer corner is too pointed"),
            "under_dip" => ("The skin under the eye is too hollow", "The skin under the eye is too full"),
            _ => ("Too much", "Too little"),
        };
        let unit = match PROPORTIONS[i].unit {
            "degrees" => "degrees",
            "mm2" => "mm²",
            "share" => "of the eye's width",
            _ => "mm",
        };
        if unit == "of the eye's width" {
            return format!("{} by {:.0}% {unit}.", if g > 0.0 { more } else { less }, 100.0 * a);
        }
        format!("{} by {a:.1} {unit}.", if g > 0.0 { more } else { less })
    }
    fn carries_hidden(&self) -> bool {
        true
    }
    /// The lids' margins and their lining lie on the eyeball. A bump moves them along the eyeball's surface: the part of their movement
    /// toward or away from the eyeball's centre is taken out, wholly within 0.8 mm of the eyeball and fading to nothing 2.5 mm from it.
    fn shape_bump(&self, mesh: &HeadMesh, movement: &mut [V3]) {
        let balls = mesh.eyes.map(|c| ring_radius_m(&mesh.base, c));
        for (v, m) in movement.iter_mut().enumerate() {
            if *m == [0.0; 3] {
                continue;
            }
            for (c, ball) in mesh.eyes.iter().zip(balls) {
                let d = crate::sub(mesh.base[v], *c);
                let r = crate::length(d);
                let off = (r - ball) * 1000.0;
                if off > 2.5 || r < 1e-6 {
                    continue;
                }
                let w = smoothstep((2.5 - off) / 1.7);
                let n = crate::scale(d, 1.0 / r);
                *m = crate::sub(*m, crate::scale(n, w * crate::dot(*m, n)));
            }
        }
    }
    fn span_quantile(&self) -> f64 {
        0.95
    }
    fn solve_by_spread(&self) -> bool {
        true
    }
    /// The eyeball's size weighs less: KOTOR's large eyeballs sit far back, and matching their front with our eyeball's size alone
    /// inflated our eyeball to fake a forward eyeball, which Eye prominence now gives (`EYES_BALL_WEIGHT`, native only). The skin beside
    /// the nose, the bulge round the outer corner and the eyeball seen in profile weigh more, because without them the solve trades
    /// them away for the opening's outline (pmhc04's round 3).
    fn gap_weight(&self, measure: &str) -> f64 {
        // EYES_WEIGHTS="nose_skin=3;inner_tip=2" sets other proportions' weights, for trials
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(w) = std::env::var("EYES_WEIGHTS").ok().and_then(|v| {
            v.split(';').find_map(|p| p.split_once('=').filter(|(k, _)| *k == measure).and_then(|(_, x)| x.parse().ok()))
        }) {
            return w;
        }
        match measure {
            "nose_skin" => return 2.0,
            "outer_bulge" => return 2.5,
            "profile_ball" => return 1.5,
            "profile_h2" | "profile_h5" | "profile_h8" | "profile_h11" | "profile_m2" | "profile_m5" | "profile_m8" | "profile_m11" | "profile_len" => return 2.0,
            _ => {}
        }
        if measure != "ball" {
            return 1.0;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(w) = std::env::var("EYES_BALL_WEIGHT").ok().and_then(|v| v.parse().ok()) {
            return w;
        }
        0.3
    }
    fn guards(&self) -> &'static [Guard] {
        &[]
    }
    fn check_bump(&self) -> V3 {
        [0.0, -40.0, 0.0]
    }
    fn shots(&self) -> &'static [Shot] {
        &SHOTS
    }
}

pub const SPECS: [Spec; 29] = [
    Spec {
        measure: "depth",
        name: "Eye depth",
        hint: "Moves the whole eye back into the socket or forward out of it. The eyeball, the lids and the lashes move together, and the skin around the eye follows smoothly. At a positive value the eye sits further forward.",
        group: "whole",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "profile_ball",
        name: "Eye prominence",
        hint: "Brings the eyeball forward out of the face, or sets it back. The lids and the skin up to about 4 mm around the opening move with the eyeball. The skin further out, beside the nose, under the brow and on the cheek, stays where it is. At a positive value more of the eyeball shows in profile. The value is in mm.",
        group: "whole",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "level",
        name: "Eye height",
        hint: "Moves the whole eye up or down on the face. The eyeball, the lids and the lashes move together, and the skin around the eye follows smoothly.",
        group: "whole",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "spacing",
        name: "Eye spacing",
        hint: "Moves both eyes toward the nose or away from it. At a positive value the eyes sit further apart. The value is the change of the distance between the two eyes.",
        group: "whole",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "pitch",
        name: "Socket angle",
        hint: "Turns the whole eye about a line through the eyeball's centre that runs from side to side. At a positive value the eye points a little upward. The lower lid then comes forward and the upper lid goes back.",
        group: "whole",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "tilt",
        name: "Eye tilt",
        hint: "Turns the whole eye about its centre. At a positive value the outer corner sits higher than the inner corner. The shape of the opening does not change.",
        group: "whole",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "ball",
        name: "Eyeball size",
        hint: "The radius of the eyeball. The lids and the skin around the eye move outward with a larger eyeball. A larger eyeball also has a larger iris.",
        group: "whole",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "width",
        name: "Eye width",
        hint: "How long the eye is, from the inner corner to the outer one, measured along the eyeball. Both corners move along the eyeball, so a longer eye wraps further round it and its corners go further back into the head. The value is the change of that length, in mm.",
        group: "size",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "height",
        name: "Eye opening height",
        hint: "How tall the eye's opening is. The upper lid moves up and the lower lid moves down by the same amount, most in the middle and not at all at the corners. The value is the change of the opening's height in the middle, in mm.",
        group: "size",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "upper_inner",
        name: "Upper lid, inner point",
        hint: "Raises or lowers the upper lid's edge a quarter of the way from the inner corner. The skin above the lid follows. The value is in mm.",
        group: "lids",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "upper_rise",
        name: "Upper lid, middle point",
        hint: "Raises or lowers the middle of the upper lid's edge. The skin above the lid follows. The value is in mm.",
        group: "lids",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "upper_outer",
        name: "Upper lid, outer point",
        hint: "Raises or lowers the upper lid's edge a quarter of the way from the outer corner. The skin above the lid follows. The value is in mm.",
        group: "lids",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "upper_depth",
        name: "Upper lid fullness",
        hint: "Whether the upper lid sits deep under the brow or stands full in front of it.",
        group: "lids",
        parts: &["upper lid"],
        repair_parts: &[],
        free: &[],
        loose: &["upper"],
    },
    Spec {
        measure: "lower_inner",
        name: "Lower lid drop, inner",
        hint: "Lowers or raises the lower lid's edge a quarter of the way from the inner corner. At a positive value the lid sits lower, and at a negative value it sits higher. The skin below the lid follows. The value is in mm.",
        group: "lids",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "lower_drop",
        name: "Lower lid drop, middle",
        hint: "Lowers or raises the middle of the lower lid's edge. At a positive value the lid sits lower and more of the eye shows below the iris. At a negative value it sits higher. The skin below the lid follows. The value is in mm.",
        group: "lids",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "lower_outer",
        name: "Lower lid drop, outer",
        hint: "Lowers or raises the lower lid's edge a quarter of the way from the outer corner. At a positive value the lid sits lower, and at a negative value it sits higher. The skin below the lid follows. The value is in mm.",
        group: "lids",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "",
        name: "Lower lid flatness",
        hint: "Straightens the lower lid's edge, or curves it more. At 1 the lower lid runs in a straight line from the inner corner to the outer corner. At 0 it keeps its own curve, and at a negative value it dips further below the line. The corners do not move, and the skin below the lid follows. The value is the share of the lid's dip below the line between the corners that is taken away.",
        group: "lids",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "inner_reach",
        name: "Inner corner position",
        hint: "Moves the inner corner along the eyeball. At a positive value the corner moves toward the nose, and the eye gets longer. The value is in mm.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "inner_tip",
        name: "Inner corner depth",
        hint: "Brings the inner corner forward, toward the surface of the face, or sets it back into the socket. The corner's tip moves the most, and the lids and the skin within about 5 mm of it follow. The caruncle fills the corner behind the tip. At a positive value the corner sits further forward. The value is in mm.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "inner_height",
        name: "Inner corner height",
        hint: "Raises or lowers the inner corner. The rest of the eye stays where it is. The value is in mm.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "inner_angle",
        name: "Inner corner angle",
        hint: "How wide the two lids open where they meet at the inner corner. At a positive value the lids part at a wider angle. The value is in degrees.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "inner_round",
        name: "Inner corner roundness",
        hint: "Whether the inner corner ends in a sharp point or a round end. At a positive value the lids part just next to the corner and the end is rounder. The value is in mm.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "outer_reach",
        name: "Outer corner position",
        hint: "Moves the outer corner along the eyeball. At a positive value the corner moves toward the temple, wraps further round the eyeball, and the eye gets longer. The value is in mm.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "outer_tip",
        name: "Outer corner depth",
        hint: "Sets the outer corner back toward the ear, or brings it forward. The corner's tip moves the most, and the lids and the skin within about 8 mm of it follow. The lids' ends may then lie behind the eyeball, so more of the eyeball shows from the side. At a negative value the corner sits further back. The value is in mm.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "outer_lift",
        name: "Outer corner height",
        hint: "Raises or lowers the outer corner, for a lifted or a drooping look, without turning the whole eye. The value is in mm.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "outer_angle",
        name: "Outer corner angle",
        hint: "How wide the two lids open where they meet at the outer corner. At a positive value the lids part at a wider angle. The value is in degrees.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "outer_round",
        name: "Outer corner roundness",
        hint: "Whether the outer corner ends in a sharp point or a round end. At a positive value the lids part just next to the corner and the end is rounder. The value is in mm.",
        group: "corners",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "cover",
        name: "Upper lid cover",
        hint: "How low the upper lid hangs over the eye when the eye is open. At a positive value the lid comes further down over the iris, most in the middle and not at all at the corners. The skin does not move. The eye still closes fully in a blink. The value is in mm in the middle of the lid.",
        group: "lids",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "",
        name: "Caruncle width",
        hint: "How far the pink of the caruncle, the small pink wedge in the inner corner of the eye, reaches from the corner. The caruncle always fills the corner between the lids and the eyeball, and past its pink part it takes the eyeball's own colour. At 0 the pink part is 4.5 mm long along the lids' edges, which shows as about 2 to 3 mm from the front. At -2 it is 2.5 mm long, and at +3 it is 7.5 mm long. The value is in mm along the lids' edges.",
        group: "caruncle",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
];

/// The eye's own tools (native builds only): pictures with the eyeballs and the generated upper lid pieces, and the lid piece's checks.
#[cfg(not(target_arch = "wasm32"))]
pub mod tools {
    use super::*;
    use crate::heads::{World, parallel, threads};
    use crate::render::{clay, dots, label};
    use crate::set::SMOOTHING;
    use head_eyes::piece::{EyeEdge, LidRig, Settings};
    use image::RgbImage;
    use std::path::Path;

    /// Pictures in a grid of `columns`, each in a cell as large as the largest, with a gap of 4 pixels.
    pub fn grid(tiles: &[RgbImage], columns: usize) -> RgbImage {
        let w = tiles.iter().map(|t| t.width()).max().unwrap_or(1);
        let h = tiles.iter().map(|t| t.height()).max().unwrap_or(1);
        let rows = tiles.len().div_ceil(columns);
        let mut out = RgbImage::from_pixel(columns as u32 * (w + 4), rows as u32 * (h + 4), image::Rgb([15, 15, 18]));
        for (i, t) in tiles.iter().enumerate() {
            let (cx, cy) = ((i % columns) as u32 * (w + 4), (i / columns) as u32 * (h + 4));
            image::imageops::overlay(&mut out, t, cx as i64, cy as i64);
        }
        out
    }

    /// The lid rig of our head mesh, from `lid_edges.json` in the data folder.
    pub fn lid_rig(data: &Path, mesh: &HeadMesh) -> LidRig {
        let edges: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("lid_edges.json")).expect("lid_edges.json")).unwrap();
        let mut eyes = Vec::new();
        for key in ["L", "R"] {
            let e = &edges["eyes"][key];
            let u = |v: &serde_json::Value| v.as_u64().unwrap() as u32;
            eyes.push(EyeEdge {
                edge: e["edge"].as_array().unwrap().iter().map(u).collect(),
                upper: e["lid"].as_array().unwrap().iter().map(|v| v.as_str() == Some("upper")).collect(),
                inner_corner: u(&e["inner_corner"]),
                outer_corner: u(&e["outer_corner"]),
                centre: e["eyeball_centre_vertices"].as_array().unwrap().iter().map(u).collect(),
                temple_plus_x: e["temple_is_plus_x"].as_bool().unwrap(),
            });
        }
        LidRig::new(&mesh.triangles, &mesh.base, &eyes)
    }

    /// Triangles to draw in the frame of the eyes, each with its own colour or the clay's.
    #[derive(Default)]
    pub struct Scene {
        pub p: Vec<V3>,
        pub t: Vec<[u32; 3]>,
        pub tint: Vec<Option<[f64; 3]>>,
    }

    const WHITE: [f64; 3] = [238.0, 234.0, 226.0];
    const IRIS: [f64; 3] = [96.0, 70.0, 48.0];
    const PUPIL: [f64; 3] = [18.0, 16.0, 16.0];
    const PIECE: [f64; 3] = [196.0, 150.0, 128.0];
    const LASH: [f64; 3] = [40.0, 30.0, 28.0];
    const KOTOR_LID: [f64; 3] = [120.0, 100.0, 150.0];

    impl Scene {
        pub fn add(&mut self, p: &[V3], t: &[[u32; 3]], tint: Option<[f64; 3]>) {
            let o = self.p.len() as u32;
            self.p.extend_from_slice(p);
            self.t.extend(t.iter().map(|x| x.map(|i| i + o)));
            self.tint.extend(std::iter::repeat_n(tint, t.len()));
        }
        /// An eyeball's triangles, coloured white, iris or pupil by the angle from straight ahead.
        pub fn add_ball(&mut self, tris: &[[V3; 3]], centre: V3) {
            for t in tris {
                let m = [0, 1, 2].map(|c| (t[0][c] + t[1][c] + t[2][c]) / 3.0 - centre[c]);
                let a = (m[2] / crate::length(m).max(1e-9)).clamp(-1.0, 1.0).acos();
                let tint = if a < 0.17 { PUPIL } else if a < 0.42 { IRIS } else { WHITE };
                let o = self.p.len() as u32;
                self.p.extend_from_slice(t);
                self.t.push([o, o + 1, o + 2]);
                self.tint.push(Some(tint));
            }
        }
        pub fn draw(&self, shot: &Shot) -> RgbImage {
            clay(&self.p, &self.t, shot, &[], &self.tint)
        }
    }

    /// A sphere's triangles (front half and a little more), in the frame given by `to_local`.
    pub fn sphere(centre: V3, r: f64) -> Vec<[V3; 3]> {
        let (rings, segs) = (28, 48);
        let at = |i: usize, j: usize| -> V3 {
            let th = std::f64::consts::PI * 0.6 * i as f64 / rings as f64;
            let ph = 2.0 * std::f64::consts::PI * j as f64 / segs as f64;
            [centre[0] + r * th.sin() * ph.cos(), centre[1] + r * th.sin() * ph.sin(), centre[2] + r * th.cos()]
        };
        let mut out = Vec::new();
        for i in 0..rings {
            for j in 0..segs {
                let (a, b, c, d) = (at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1));
                out.push([a, b, c]);
                out.push([a, c, d]);
            }
        }
        out
    }

    /// Our head (positions in metres) with its eyeballs and lid pieces, the pieces turned `closed` of the way shut (0 open, 1 closed).
    const CARUNCLE_PINK: [f64; 3] = [214.0, 128.0, 128.0];
    thread_local! {
        /// The caruncle's width (mm) for the next pictures on this thread.
        pub static CARUNCLE: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
    }
    /// Sets the caruncle of the next pictures on this thread from a set's values.
    pub fn caruncle_from(set: &crate::set::SliderSet, values: &[f64]) {
        let at = |name: &str| set.sliders.iter().position(|s| s.name == name).and_then(|k| values.get(k).copied()).unwrap_or(0.0);
        CARUNCLE.with(|c| c.set(at("Caruncle width")));
    }

    pub fn our_scene(rig: &LidRig, head: &[V3], frame: &Frame, closed: f64, settings: &Settings) -> (Scene, Vec<head_eyes::piece::EyePiece>) {
        let pieces = head_eyes::piece::build(rig, head, settings);
        let hidden: std::collections::HashSet<u32> = pieces.iter().flat_map(|p| p.hidden_faces.iter().copied()).collect();
        let local: Vec<V3> = head.iter().map(|&p| frame.local(p)).collect();
        let tris: Vec<[u32; 3]> = rig.triangles.iter().enumerate().filter(|(k, _)| !hidden.contains(&(*k as u32))).map(|(_, t)| *t).collect();
        let mut s = Scene::default();
        s.add(&local, &tris, None);
        for e in &rig.eyes {
            let c = e.centre.iter().fold([0.0; 3], |a, &i| crate::add(a, crate::scale(head[i as usize], 1.0 / e.centre.len() as f64)));
            let cl = frame.local(c);
            s.add_ball(&sphere(cl, head_eyes::piece::eyeball_radius(head, &e.centre) / 1000.0 * frame.scale), cl);
        }
        let width = CARUNCLE.with(|c| c.get());
        // EYES_NO_CARUNCLE=1 draws the eye as before stage B, without the caruncle
        let caruncles = if std::env::var("EYES_NO_CARUNCLE").is_ok_and(|v| v == "1") { &[][..] } else { &crate::eye_curves::edges()[..] };
        for e in caruncles {
            let edge = head_eyes::piece::EyeEdge {
                edge: e.edge.clone(),
                upper: e.upper.clone(),
                inner_corner: e.inner,
                outer_corner: e.outer,
                centre: e.ring.clone(),
                temple_plus_x: e.sign > 0.0,
            };
            let cp: Vec<V3> = head_eyes::caruncle::positions(head, &edge, width).iter().map(|&p| frame.local(p)).collect();
            let mut ct = head_eyes::caruncle::triangles();
            if e.sign < 0.0 {
                for t in &mut ct {
                    t.swap(1, 2);
                }
            }
            s.add(&cp, &ct, Some(CARUNCLE_PINK));
        }
        for p in &pieces {
            let angle = closed * p.closed_angle;
            let tl = |q: &[f32; 3]| frame.local(head_eyes::piece::turn_about([q[0] as f64, q[1] as f64, q[2] as f64], p.pivot, p.axis, angle));
            let pp: Vec<V3> = p.positions.iter().map(tl).collect();
            s.add(&pp, &p.triangles, Some(PIECE));
            let lp: Vec<V3> = p.lash_positions.iter().map(tl).collect();
            s.add(&lp, &p.lash_triangles, Some(LASH));
        }
        (s, pieces)
    }

    /// KOTOR's head with its eyeballs and its lid pieces (in violet, so they read as a separate part).
    pub fn kotor_scene(name: &str, v: &[V3], f: &[[u32; 3]], frame: &Frame) -> Scene {
        let mut s = Scene::default();
        let local: Vec<V3> = v.iter().map(|&p| frame.local(p)).collect();
        s.add(&local, f, None);
        if let Some(eyes) = kotor::all().get(name) {
            for e in eyes {
                let c = frame.local(e.centre);
                // KOTOR's own eyeball, its triangles split flat four times so the iris and pupil read as discs
                let mut ball: Vec<[V3; 3]> = e.ball.iter().map(|t| t.map(|p| frame.local(p))).collect();
                for _ in 0..4 {
                    ball = ball
                        .iter()
                        .flat_map(|t| {
                            let m = |a: V3, b: V3| [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1]), 0.5 * (a[2] + b[2])];
                            let (ab, bc, ca) = (m(t[0], t[1]), m(t[1], t[2]), m(t[2], t[0]));
                            [[t[0], ab, ca], [ab, t[1], bc], [ca, bc, t[2]], [ab, bc, ca]]
                        })
                        .collect();
                }
                s.add_ball(&ball, c);
                for t in &e.lid {
                    let o = s.p.len() as u32;
                    s.p.extend(t.iter().map(|&p| frame.local(p)));
                    s.t.push([o, o + 1, o + 2]);
                    s.tint.push(Some(KOTOR_LID));
                }
            }
        }
        s
    }

    fn dots_of(m: &Result<Measured, &'static str>) -> Vec<V3> {
        m.as_ref().ok().and_then(|m| serde_json::from_value(m.marks["dots"].clone()).ok()).unwrap_or_default()
    }

    /// KOTOR's eye next to our stand-in's default eye, for every head: KOTOR with its eyeball and lid piece, ours with our eyeball and
    /// the generated piece, both with the measured opening in red.
    pub fn kotor_eyes_sheet(world: &World, rig: &LidRig, names: &[String]) -> RgbImage {
        let shot = SHOTS[1];
        let heads: Vec<&crate::heads::Head> = world.heads.iter().filter(|h| names.is_empty() || names.contains(&h.name)).collect();
        let tiles: Vec<Vec<RgbImage>> = parallel(heads.len(), threads(), |i| {
            let h = heads[i];
            let mut k = kotor_scene(&h.name, &h.kotor_v, &h.kotor_f, &h.frame).draw(&shot);
            if h.kotor_error.is_none() {
                dots(&mut k, &shot, &dots_of(&Ok(h.kotor.clone())));
            }
            label(&mut k, &h.name.replace('_', " ").to_uppercase());
            let built = world.smoother.smooth(&h.start, SMOOTHING);
            let (scene, _) = our_scene(rig, &built, &h.frame, 0.0, &Settings::default());
            let mut o = scene.draw(&shot);
            dots(&mut o, &shot, &dots_of(&h.start_nose.clone().ok_or("")));
            label(&mut o, "OURS");
            vec![k, o]
        });
        grid(&tiles.into_iter().flatten().collect::<Vec<_>>(), 8)
    }

    /// Runs one of the eye's own commands. Returns false when `command` is not one of them.
    pub fn run(command: &str, args: &[String], data: &Path, input: &crate::mesh::input::Input, world_of: &dyn Fn() -> World) -> bool {
        let out = std::env::var("SLIDERS_SHEET").unwrap_or_else(|_| "eyes.png".into());
        match command {
            "kotor-eyes" => {
                let world = world_of();
                let rig = lid_rig(data, &world.mesh);
                kotor_eyes_sheet(&world, &rig, &args[4.min(args.len())..]).save(&out).unwrap();
                eprintln!("wrote {out}");
            }
            "default-eye" => default_eye(&world_of(), input),
            "ends" => {
                let world = world_of();
                let rig = lid_rig(data, &world.mesh);
                let set = read_set(data);
                ends_sheet(&world, &rig, &set, &args[4.min(args.len())..]).save(&out).unwrap();
                eprintln!("wrote {out}");
            }
            "default-sheet" => {
                // `sliders eyes default-sheet <data> <slider name>=<value> ... -- <head> ...`: the default eye next to the proposed one
                let world = world_of();
                let rig = lid_rig(data, &world.mesh);
                let set = read_set(data);
                let rest = &args[4.min(args.len())..];
                let split = rest.iter().position(|a| a == "--").unwrap_or(rest.len());
                let mut values = vec![0.0; set.sliders.len()];
                for a in &rest[..split] {
                    let (name, v) = a.split_once('=').expect("<slider name>=<value>");
                    let k = set.sliders.iter().position(|s| s.name == name).unwrap_or_else(|| panic!("no slider {name}"));
                    values[k] = v.parse().unwrap();
                }
                let names: Vec<String> = rest.get(split + 1..).unwrap_or(&[]).to_vec();
                default_sheet(&world, &rig, &set, &values, &names).save(&out).unwrap();
                eprintln!("wrote {out}");
            }
            "made-eyes" => {
                // `sliders eyes made-eyes <data> <head> ...`, SLIDERS_MODE=held_out for the ranges set without the head, and
                // SLIDERS_LABELS=0 for pictures without the heads' names
                let world = world_of();
                let rig = lid_rig(data, &world.mesh);
                let set = read_set(data);
                let mode = std::env::var("SLIDERS_MODE").unwrap_or_else(|_| "full".into());
                made_eyes_sheet(&world, &rig, &set, data, &args[4.min(args.len())..], &mode).save(&out).unwrap();
                eprintln!("wrote {out}");
            }
            "cap" => {
                // `sliders eyes cap <data> "<slider name>=<low>:<high>" ...`: trims web/eyes/sliders.json; the old set is kept as
                // web/eyes/sliders_uncapped.json
                let path = data.join("web/eyes/sliders.json");
                let mut set = read_set(data);
                std::fs::write(data.join("web/eyes/sliders_uncapped.json"), serde_json::to_string(&set).unwrap()).unwrap();
                for a in &args[4.min(args.len())..] {
                    let (name, range) = a.split_once('=').expect("<name>=<low>:<high>");
                    let (lo, hi) = range.split_once(':').expect("<low>:<high>");
                    let (lo, hi): (f64, f64) = (lo.parse().unwrap(), hi.parse().unwrap());
                    let nf = set.fields.len();
                    let s = set.sliders.iter_mut().find(|s| s.name == name).unwrap_or_else(|| panic!("no slider {name}"));
                    cap_slider(s, nf, lo, hi);
                    eprintln!("{name}: now {:.2} to {:.2}, {} keys", s.low, s.high, s.keys.len());
                }
                std::fs::write(&path, serde_json::to_string(&set).unwrap()).unwrap();
            }
            "fault-places" => {
                // where the solved heads' faults lie: per fault, across and up from the eye's centre and its distance from the eyeball
                let world = world_of();
                let set = read_set(data);
                let page: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/page.json")).unwrap()).unwrap();
                let n = set.sliders.len();
                let mut hist = std::collections::BTreeMap::<i64, usize>::new();
                let mut rows = Vec::new();
                for (i, h) in world.heads.iter().enumerate() {
                    let values: Vec<f64> = solved_values(&page, &set, &h.name);
                    let before = world.smoother.smooth(&h.start, SMOOTHING);
                    let near: Vec<u32> = (0..before.len() as u32).filter(|&v| WINDOW.contains(h.frame.local(before[v as usize]))).collect();
                    let checker = crate::check::Checker::new(&world.mesh, &before, &near);
                    let f = checker.faults(&set.build(&world.smoother, &h.start, &values));
                    for p in &f.at {
                        let q = h.frame.local(*p);
                        let c = if q[0] > 0.0 { 0.5 * EYES_APART_MM } else { -0.5 * EYES_APART_MM };
                        let r = ((q[0] - c).powi(2) + q[1].powi(2) + q[2].powi(2)).sqrt() / h.frame.scale * 1000.0;
                        *hist.entry(r.round() as i64).or_default() += 1;
                        rows.push(format!("{} u {:.1} v {:.1} r {:.1}", h.name, q[0].abs() - 0.5 * EYES_APART_MM, q[1], r));
                    }
                }
                println!("faults by distance from the eyeball's centre (mm): {hist:?}");
                for r in rows.iter().take(40) {
                    println!("{r}");
                }
            }
            "blame" => {
                // per slider: fault triangles over the 57 solved heads with that slider set to 0, against all values
                let world = world_of();
                let set = read_set(data);
                let page: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/page.json")).unwrap()).unwrap();
                let n = set.sliders.len();
                let per: Vec<Vec<usize>> = parallel(world.heads.len(), threads(), |i| {
                    let h = &world.heads[i];
                    let before = world.smoother.smooth(&h.start, SMOOTHING);
                    let near: Vec<u32> = (0..before.len() as u32).filter(|&v| WINDOW.contains(h.frame.local(before[v as usize]))).collect();
                    let checker = crate::check::Checker::new(&world.mesh, &before, &near);
                    let mut values = solved_values(&page, &set, &h.name);
                    // BLAME_KEEP="name;name": every other slider at 0
                    if let Ok(keep) = std::env::var("BLAME_KEEP") {
                        let keep: Vec<&str> = keep.split(';').collect();
                        for (k, sl) in set.sliders.iter().enumerate() {
                            if !keep.contains(&sl.name.as_str()) {
                                values[k] = 0.0;
                            }
                        }
                    }
                    let mut out = vec![checker.faults(&set.build(&world.smoother, &h.start, &values)).total()];
                    for k in 0..n {
                        let mut v = values.clone();
                        if v[k] == 0.0 {
                            out.push(out[0]);
                            continue;
                        }
                        v[k] = 0.0;
                        out.push(checker.faults(&set.build(&world.smoother, &h.start, &v)).total());
                    }
                    out
                });
                let all: usize = per.iter().map(|r| r[0]).sum();
                println!("all values: {} heads with a fault, {all} triangles", per.iter().filter(|r| r[0] > 0).count());
                for k in 0..n {
                    let t: usize = per.iter().map(|r| r[k + 1]).sum();
                    let h = per.iter().filter(|r| r[k + 1] > 0).count();
                    let ends = world.heads.iter().filter(|hh| { let v = solved_values(&page, &set, &hh.name)[k]; v != 0.0 && (v <= set.sliders[k].low + 1e-6 || v >= set.sliders[k].high - 1e-6) }).count();
                    println!("{:26} zeroed: {h:2} heads, {t:5} triangles; at an end on {ends} heads", set.sliders[k].name);
                }
            }
            "closeup" => {
                // `sliders eyes closeup <data> <head> ...`: close three-quarter views of one eye on the solved preset (CLOSE_ZERO=1 for
                // all-zero values): KOTOR, ours open, ours from a little below, ours closed
                let world = world_of();
                let rig = lid_rig(data, &world.mesh);
                let set = read_set(data);
                let page: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/page.json")).unwrap()).unwrap();
                let zero = std::env::var("CLOSE_ZERO").is_ok_and(|v| v == "1");
                let names = &args[4.min(args.len())..];
                let found: Vec<usize> = names.iter().filter_map(|nm| world.heads.iter().position(|h| &h.name == nm)).collect();
                let (wx, wy, ly) = (env_f("CLOSE_X0", 6.0), env_f("CLOSE_Y0", -10.0), env_f("CLOSE_LOW_Y0", -10.0));
                let close = Shot { name: "close", yaw: -40.0, pitch: 0.0, pivot: 0.0, window: [wx, wx + 30.0, wy, wy + 20.0] };
                let low = Shot { name: "close low", yaw: -40.0, pitch: 20.0, pivot: 0.0, window: [wx, wx + 30.0, ly, ly + 20.0] };
                let rows: Vec<Vec<RgbImage>> = parallel(found.len(), threads(), |j| {
                    let h = &world.heads[found[j]];
                    let mut values = if zero { vec![0.0; set.sliders.len()] } else { solved_values(&page, &set, &h.name) };
                    // CLOSE_SET="<slider name>=<value>;..." puts those values on top
                    for a in std::env::var("CLOSE_SET").unwrap_or_default().split(';').filter(|a| !a.is_empty()) {
                        let (nm, v) = a.split_once('=').expect("<slider name>=<value>");
                        let k = set.sliders.iter().position(|s| s.name == nm).unwrap_or_else(|| panic!("no slider {nm}"));
                        values[k] = v.parse().unwrap();
                    }
                    let mut k = kotor_scene(&h.name, &h.kotor_v, &h.kotor_f, &h.frame).draw(&close);
                    label(&mut k, &format!("{} KOTOR", h.name.replace('_', " ").to_uppercase()));
                    let built = set.build(&world.smoother, &h.start, &values);
                    caruncle_from(&set, &values);
                    let mut row = vec![k];
                    for (shot, shut) in [(&close, 0.0), (&low, 0.0), (&close, 1.0)] {
                        let (scene, _) = our_scene(&rig, &built, &h.frame, shut, &Settings::default());
                        row.push(scene.draw(shot));
                    }
                    row
                });
                grid(&rows.into_iter().flatten().collect::<Vec<_>>(), 4).save(&out).unwrap();
                eprintln!("wrote {out}");
            }
            "ball-base" => {
                let world = world_of();
                let frame = Frame::from_eyes(world.mesh.eyes);
                let built = world.smoother.smooth(&world.mesh.base, SMOOTHING);
                let s = Surface::within(&built, &world.mesh.triangles, &frame, &WINDOW);
                for b in our_balls(frame.scale) {
                    println!("socket median {:.4} mm of the frame", socket_median(&s, b.centre));
                }
                let rings = rings(data);
                for dr in [-2.0, 0.0, 2.0, 4.0, 6.0, 8.0] {
                    let mut p = world.mesh.base.clone();
                    for (q, d) in p.iter_mut().zip(ball_field(&world.mesh, &rings, dr)) {
                        *q = crate::add(*q, d);
                    }
                    let m = measure(&Surface::within(&world.smoother.smooth(&p, SMOOTHING), &world.mesh.triangles, &frame, &WINDOW));
                    println!("dR {dr}: measured ball {:?}", m.map(|m| m.proportions[11] - OUR_BALL_M * 1000.0));
                }
            }
            "make3" => make3(&world_of(), data),
            "make-whole" => make_whole(&world_of(), data),
            "make-b" => make_b(&world_of(), data),
            "make-r4" => make_r4(&world_of(), data),
            "make-r5" => make_r5(&world_of(), data),
            "make-pmhc-r1" => make_pmhc_r1(&world_of(), data),
            "make-pmhc-r2" => make_pmhc_r2(&world_of(), data),
            "marks-pmhc-r4" => marks_pmhc_r4(&world_of(), data),
            "guard-pmhc-r1" => guard_pmhc_r1(&world_of(), &read_set(data), data, Path::new(&args[4]), &args[5..]),
            "place-whole" => place_whole(&world_of(), &read_set(data), data),
            "pairs" => {
                // `sliders eyes pairs <data>` with PAIRS_A and PAIRS_B (names joined by semicolons), PAIRS_SHOT (index of the shots, default 1)
                // and PAIRS_CLOSED=1 for the lid piece shut
                let world = world_of();
                let rig = lid_rig(data, &world.mesh);
                let set = read_set(data);
                let list = |k: &str| -> Vec<String> { std::env::var(k).unwrap_or_default().split(';').filter(|x| !x.is_empty()).map(String::from).collect() };
                let shot = SHOTS[std::env::var("PAIRS_SHOT").ok().and_then(|v| v.parse().ok()).unwrap_or(1)];
                let closed = if std::env::var("PAIRS_CLOSED").is_ok_and(|v| v == "1") { 1.0 } else { 0.0 };
                pairs_sheet(&world, &rig, &set, &list("PAIRS_A"), &list("PAIRS_B"), shot, closed).save(&out).unwrap();
                eprintln!("wrote {out}");
            }
            "faults" => {
                // `sliders eyes faults <data> <slider name> <value> ...`: faults on the base head and how many of the 57 stand-ins fault
                let world = world_of();
                let set = read_set(data);
                let rest = &args[4.min(args.len())..];
                let k = set.sliders.iter().position(|s| s.name == rest[0]).expect("no such slider");
                for v in &rest[1..] {
                    let mut values = vec![0.0; set.sliders.len()];
                    values[k] = v.parse().unwrap();
                    let count = |start: &[V3], frame: &Frame| {
                        let before = world.smoother.smooth(start, SMOOTHING);
                        let near: Vec<u32> = (0..before.len() as u32).filter(|&i| WINDOW.contains(frame.local(before[i as usize]))).collect();
                        crate::check::Checker::new(&world.mesh, &before, &near).faults(&set.build(&world.smoother, start, &values)).total()
                    };
                    let base = count(&world.mesh.base, &Frame::from_eyes(world.mesh.eyes));
                    let heads: Vec<usize> = parallel(world.heads.len(), threads(), |i| count(&world.heads[i].start, &world.heads[i].frame));
                    println!("{} {v}: base {base}, stand-ins with a fault {} of {}", rest[0], heads.iter().filter(|&&c| c > 0).count(), heads.len());
                }
            }
            "outlines" => {
                // `sliders eyes outlines <data> <slider name> <value> ...`: the opening's two outlines (u, v per point, the eyes
                // averaged) on the base head at each value, as JSON on stdout
                let world = world_of();
                let set = read_set(data);
                let rest = &args[4.min(args.len())..];
                let k = set.sliders.iter().position(|s| s.name == rest[0]).expect("no such slider");
                let ev = crate::solve::Evaluator::new(&EYES, &set, &world.smoother, &world.mesh.base, &world.mesh.triangles, Frame::from_eyes(world.mesh.eyes));
                let mut out = Vec::new();
                for v in &rest[1..] {
                    let v: f64 = v.parse().unwrap();
                    let mut values = vec![0.0; set.sliders.len()];
                    values[k] = v;
                    let m = ev.measure(&values).unwrap();
                    let built = set.build(&world.smoother, &world.mesh.base, &values);
                    let frame = Frame::from_eyes(world.mesh.eyes);
                    let edges: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("lid_edges.json")).unwrap()).unwrap();
                    let edge: Vec<V3> = ["L", "R"]
                        .iter()
                        .flat_map(|k| edges["eyes"][*k]["edge"].as_array().unwrap().iter().map(|i| frame.local(built[i.as_u64().unwrap() as usize])).collect::<Vec<_>>())
                        .collect();
                    out.push(serde_json::json!({"value": v, "upper": m.outlines[0], "lower": m.outlines[1], "proportions": m.proportions, "edge": edge}));
                }
                println!("{}", serde_json::to_string(&out).unwrap());
            }
            "lid-safety" => {
                let world = world_of();
                let rig = lid_rig(data, &world.mesh);
                let set = read_set(data);
                let mut report = lid_safety(&world, &rig, &set);
                let fits: Vec<Vec<V3>> = std::iter::once(world.mesh.base.clone())
                    .chain(world.heads.iter().map(|h| input.points(&format!("heads/{}/fit_smooth", h.name))))
                    .collect();
                report["fits_as_they_are"] = lid_counts(&rig, &fits);
                eprintln!("the smoothed fits, without sliders: {}", report["fits_as_they_are"]);
                std::fs::write(data.join("eyes/lid_safety.json"), serde_json::to_string_pretty(&report).unwrap()).unwrap();
            }
            _ => return false,
        }
        true
    }

    /// A head's solved values from the page's results (the most sliders, ranges from all heads), matched to the set's sliders by name.
    pub fn solved_values(page: &serde_json::Value, set: &crate::set::SliderSet, name: &str) -> Vec<f64> {
        let mut out = vec![0.0; set.sliders.len()];
        let last = page["counts"].as_array().and_then(|c| c.last()).map(|c| c.to_string()).unwrap_or_default();
        let Some(h) = page["heads"].as_array().and_then(|a| a.iter().find(|h| h["name"] == name)) else { return out };
        let Some(v) = h["solved"]["full"]["counts"][&last]["values"].as_array() else { return out };
        for (k, x) in v.iter().enumerate() {
            let Some(sn) = page["sliders"][k]["name"].as_str() else { continue };
            if let Some(j) = set.sliders.iter().position(|s| s.name == sn) {
                out[j] = x.as_f64().unwrap_or(0.0);
            }
        }
        out
    }

    /// The eyeball-centre vertices of each eye, from `lid_edges.json`.
    pub fn rings(data: &Path) -> [Vec<u32>; 2] {
        let edges: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("lid_edges.json")).expect("lid_edges.json")).unwrap();
        ["L", "R"].map(|k| edges["eyes"][k]["eyeball_centre_vertices"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect())
    }

    fn mean_of(p: &[V3], ids: &[u32]) -> V3 {
        ids.iter().fold([0.0; 3], |a, &i| crate::add(a, crate::scale(p[i as usize], 1.0 / ids.len() as f64)))
    }

    /// The eyeball size slider's movement for a radius `dr` mm larger (metres, every vertex). The eyeball-centre ring scales with the
    /// radius about its own mean, so the eyeball's centre stays. Every vertex up to 1.5 mm outside the eyeball moves out from its
    /// centre by `dr`, so the lids' lining and margins stay on the larger eyeball. Further out the movement fades to nothing over
    /// max(14, 6 |dr|) mm (`EYES_BALL_FADE_MIN`, `EYES_BALL_FADE_K`), so the skin never folds and no ring of skin bulges.
    pub fn ball_field(mesh: &HeadMesh, rings: &[Vec<u32>; 2], dr: f64) -> Vec<V3> {
        let centres = [mean_of(&mesh.base, &rings[0]), mean_of(&mesh.base, &rings[1])];
        let r0 = OUR_BALL_M * 1000.0;
        let fade = (env_f("EYES_BALL_FADE_K", 6.0) * dr.abs()).max(env_f("EYES_BALL_FADE_MIN", 14.0));
        mesh.base
            .iter()
            .enumerate()
            .map(|(v, &p)| {
                let e = if crate::length(crate::sub(p, centres[0])) < crate::length(crate::sub(p, centres[1])) { 0 } else { 1 };
                let d = crate::sub(p, centres[e]);
                if rings[e].contains(&(v as u32)) {
                    return crate::scale(d, dr / r0);
                }
                let r = crate::length(d) * 1000.0;
                let f = 1.0 - smoothstep((r - r0 - 1.5) / fade);
                if f <= 0.0 || r < 1e-6 { [0.0; 3] } else { crate::scale(d, dr * f / r) }
            })
            .collect()
    }

    /// The eye tilt slider's movement for a turn of `deg` degrees (metres, every vertex): each eye turns about the line through its
    /// eyeball's centre that points forward, the outer corners both going up for a positive turn. Within 14 mm of that line the turn is
    /// whole, so the opening turns as one piece and slides over the eyeball; it fades to nothing 24 mm from the line.
    pub fn tilt_field(mesh: &HeadMesh, rings: &[Vec<u32>; 2], deg: f64) -> Vec<V3> {
        let frame = Frame::from_eyes(mesh.eyes);
        let centres = [mean_of(&mesh.base, &rings[0]), mean_of(&mesh.base, &rings[1])];
        let (x, y) = (frame.axes[0], frame.axes[1]);
        mesh.base
            .iter()
            .map(|&p| {
                let e = if crate::length(crate::sub(p, centres[0])) < crate::length(crate::sub(p, centres[1])) { 0 } else { 1 };
                let d = crate::sub(p, centres[e]);
                let (qx, qy) = (crate::dot(d, x), crate::dot(d, y));
                let rho = qx.hypot(qy) * 1000.0;
                let w = 1.0 - smoothstep((rho - 14.0) / 10.0);
                if w <= 0.0 {
                    return [0.0; 3];
                }
                let side = if crate::dot(crate::sub(centres[e], frame.origin), x) > 0.0 { 1.0 } else { -1.0 };
                let a = (side * deg * w).to_radians();
                let (nx, ny) = (qx * a.cos() - qy * a.sin(), qx * a.sin() + qy * a.cos());
                crate::add(crate::scale(x, nx - qx), crate::scale(y, ny - qy))
            })
            .collect()
    }

    /// The eye width slider's movement for a stretch `k` across the eye (metres, every vertex): each place moves away from the line
    /// through the eyeball's centre that points up, by `k` times its distance across, wholly within 10 mm of the eye's centre (seen from
    /// the front) and fading to nothing 26 mm from it. Near the eyeball the movement then slides on it, as every eye bump does.
    pub fn width_field(mesh: &HeadMesh, rings: &[Vec<u32>; 2], k: f64) -> Vec<V3> {
        let frame = Frame::from_eyes(mesh.eyes);
        let centres = [mean_of(&mesh.base, &rings[0]), mean_of(&mesh.base, &rings[1])];
        let (x, y) = (frame.axes[0], frame.axes[1]);
        let mut m: Vec<V3> = mesh
            .base
            .iter()
            .map(|&p| {
                let e = if crate::length(crate::sub(p, centres[0])) < crate::length(crate::sub(p, centres[1])) { 0 } else { 1 };
                let d = crate::sub(p, centres[e]);
                let (qx, qy) = (crate::dot(d, x), crate::dot(d, y));
                let w = 1.0 - smoothstep((qx.hypot(qy) * 1000.0 - 10.0) / 16.0);
                if w <= 0.0 { [0.0; 3] } else { crate::scale(x, k * qx * w) }
            })
            .collect();
        EYES.shape_bump(mesh, &mut m);
        m
    }

    /// How much of a whole-eye movement a place of the base head takes (0 to 1), and which eye it belongs to. Seen from the front,
    /// around the eye's centre, the eye is whole inside an oval 16 mm toward the nose, 19 mm toward the temple, 12 mm up and 11 mm
    /// down, which holds the eyeball, both lids with their linings and the corners. Outside it the movement fades to nothing at
    /// `fade` times that oval (smoothly), and to nothing within 3 mm of the face's middle line. Places more than 22 mm behind the
    /// eye's centre fade out by 32 mm behind it, so the back of the head never moves.
    pub fn whole_weight(frame: &Frame, centres: &[V3; 2], p: V3, fade: f64) -> (f64, usize) {
        let e = if crate::length(crate::sub(p, centres[0])) < crate::length(crate::sub(p, centres[1])) { 0 } else { 1 };
        let q = frame.local(p);
        let c = frame.local(centres[e]);
        let side = if c[0] > 0.0 { 1.0 } else { -1.0 };
        let (u, up, dz) = (side * (q[0] - c[0]), q[1] - c[1], q[2] - c[2]);
        let a = if u < 0.0 { 16.0 } else { 19.0 };
        let b = if up > 0.0 { 12.0 } else { 11.0 };
        let r = ((u / a).powi(2) + (up / b).powi(2)).sqrt();
        let w = (1.0 - smoothstep((r - 1.0) / (fade - 1.0))) * smoothstep((q[0].abs() - 3.0) / 6.0) * smoothstep((dz + 32.0) / 10.0);
        (w, e)
    }

    /// The whole-eye movement of every vertex (metres) for `amount`: "depth" (mm forward), "level" (mm up), "spacing" (mm each eye
    /// moves away from the nose) or "pitch" (degrees, the eye turning to point up about the line through its eyeball's centre that
    /// runs from side to side). The eyeball's centre ring, the lids and their linings take the whole movement, so they stay together.
    pub fn whole_field(mesh: &HeadMesh, rings: &[Vec<u32>; 2], kind: &str, amount: f64, fade: f64) -> Vec<V3> {
        let frame = Frame::from_eyes(mesh.eyes);
        let centres = [mean_of(&mesh.base, &rings[0]), mean_of(&mesh.base, &rings[1])];
        let [x, y, z] = frame.axes;
        let m = amount / 1000.0;
        mesh.base
            .iter()
            .map(|&p| {
                let (w, e) = whole_weight(&frame, &centres, p, fade);
                if w <= 0.0 {
                    return [0.0; 3];
                }
                let side = if crate::dot(crate::sub(centres[e], frame.origin), x) > 0.0 { 1.0 } else { -1.0 };
                match kind {
                    "depth" => crate::scale(z, w * m),
                    "level" => crate::scale(y, w * m),
                    "spacing" => crate::scale(x, side * w * m),
                    "pitch" => {
                        let d = crate::sub(p, centres[e]);
                        let (qy, qz) = (crate::dot(d, y), crate::dot(d, z));
                        let a = if std::env::var("EYES_PITCH_BEND").is_ok_and(|v| v == "1") { (amount * w).to_radians() } else { amount.to_radians() };
                        let k = if std::env::var("EYES_PITCH_BEND").is_ok_and(|v| v == "1") { 1.0 } else { w };
                        let (ny, nz) = (qy * a.cos() + qz * a.sin(), -qy * a.sin() + qz * a.cos());
                        crate::add(crate::scale(y, k * (ny - qy)), crate::scale(z, k * (nz - qz)))
                    }
                    _ => panic!("unknown whole-eye movement {kind}"),
                }
            })
            .collect()
    }

    /// A slider whose keys are whole movements, one field each, at the values given (0 needs no field).
    fn field_slider(spec: &Spec, unit: &str, keys: &[(f64, Vec<V3>)], n: usize) -> crate::set::SliderSet {
        let mut fields = Vec::new();
        let mut ks = Vec::new();
        for (v, m) in keys {
            if *v == 0.0 {
                ks.push(crate::set::Key { value: 0.0, fields: Vec::new(), weights: Vec::new() });
            } else {
                ks.push(crate::set::Key { value: *v, fields: vec![fields.len() as u32], weights: vec![1.0] });
                fields.push(crate::set::Field::from_movement(m));
            }
        }
        let mut s = crate::set::Slider::from_movement(spec.name, "named", unit, &[]);
        s.zone = "eyes".into();
        s.hint = spec.hint.into();
        s.measure = spec.measure.into();
        s.group = spec.group.into();
        s.low = ks.first().map(|k| k.value).unwrap_or(0.0);
        s.high = ks.last().map(|k| k.value).unwrap_or(0.0);
        s.keys = ks;
        crate::set::SliderSet { vertex_count: n, sliders: vec![s], fields }
    }

    fn env_f(name: &str, default: f64) -> f64 {
        std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    /// Round 3's eye sliders. Eye height, Upper lid arch, Lower lid curve and Upper lid fullness keep round 2's keys and fields
    /// (`eyes/round2/web/sliders.json`). Eyeball size and Eye tilt are built as whole movements. The other sliders are made by the
    /// kit's maker on the base head with the eyeball at its proposed default size (`EYES_BALL_DEFAULT`, mm larger than 12.3). Writes
    /// `web/eyes/sliders.json`, `eyes/sliders_solve.json` (the sliders the per-head solve uses) and `eyes/named.json`.
    pub fn make3(world: &World, data: &Path) {
        let n = world.mesh.base.len();
        let rings = rings(data);
        let by = |m: &str| SPECS.iter().find(|s| s.measure == m).unwrap();
        let old: crate::set::SliderSet =
            serde_json::from_str(&std::fs::read_to_string(data.join("eyes/round2/web/sliders.json")).expect("round 2's sliders")).unwrap();
        let kept = ["Eye height", "Upper lid arch", "Lower lid curve", "Upper lid fullness"];
        let a = crate::set::SliderSet {
            vertex_count: n,
            sliders: old.sliders.iter().filter(|s| kept.contains(&s.name.as_str())).cloned().collect(),
            fields: old.fields.clone(),
        };
        let (blo, bhi) = (env_f("EYES_BALL_LOW", -2.0), env_f("EYES_BALL_HIGH", 8.0));
        let mut ball_keys = Vec::new();
        let mut v = blo;
        while v <= bhi + 1e-9 {
            ball_keys.push((v, if v == 0.0 { Vec::new() } else { ball_field(&world.mesh, &rings, v) }));
            v = ((v + 0.5) * 2.0).round() / 2.0;
        }
        let b = field_slider(by("ball"), "1 mm of the radius of the eyeball", &ball_keys, n);
        let tmax = env_f("EYES_TILT_MAX", 12.0);
        let tilt_keys: Vec<(f64, Vec<V3>)> =
            (-(tmax as i64)..=tmax as i64).map(|d| (d as f64, if d == 0 { Vec::new() } else { tilt_field(&world.mesh, &rings, d as f64) })).collect();
        let c = field_slider(by("tilt"), "1 degrees of tilt of the eye, outer corner above the inner one", &tilt_keys, n);
        // Eye width: key values are the change of the distance between the lid edge's two corners, in mm of the frame
        let edges: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("lid_edges.json")).unwrap()).unwrap();
        let corner = |k: &str| edges["eyes"]["L"][k].as_u64().unwrap() as usize;
        let (ci, co) = (corner("inner_corner"), corner("outer_corner"));
        let base_frame = Frame::from_eyes(world.mesh.eyes);
        let apart = |p: &[V3]| crate::length(crate::sub(base_frame.local(p[ci]), base_frame.local(p[co])));
        let w0 = apart(&world.smoother.smooth(&world.mesh.base, SMOOTHING));
        let (wlo, whi) = (env_f("EYES_WIDTH_LOW", -0.3), env_f("EYES_WIDTH_HIGH", 0.4));
        let mut width_keys = Vec::new();
        let mut kk = wlo;
        while kk <= whi + 1e-9 {
            if kk.abs() < 1e-9 {
                width_keys.push((0.0, Vec::new()));
            } else {
                let f = width_field(&world.mesh, &rings, kk);
                let mut p = world.mesh.base.clone();
                for (q, d) in p.iter_mut().zip(&f) {
                    *q = crate::add(*q, *d);
                }
                width_keys.push((apart(&world.smoother.smooth(&p, SMOOTHING)) - w0, f));
            }
            kk = ((kk + 0.05) * 20.0).round() / 20.0;
        }
        let wd = field_slider(by("width"), "1 mm of width of the eye, from corner to corner", &width_keys, n);
        // the maker's base head: the eyeball at its default size
        let dr = env_f("EYES_BALL_DEFAULT", 1.5);
        let mut mesh2 = world.mesh.clone();
        for (q, d) in mesh2.base.iter_mut().zip(ball_field(&world.mesh, &rings, dr)) {
            *q = crate::add(*q, d);
        }
        let frame = Frame::from_eyes(mesh2.eyes);
        let base2 = measure(&Surface::within(&world.smoother.smooth(&mesh2.base, SMOOTHING), &mesh2.triangles, &frame, &WINDOW)).expect("the base eye");
        let marks = crate::checks::Marks::read(&data.join("eyes/marks.json"));
        let hold = crate::solve::Scale {
            proportions: marks.spread_p.iter().map(|s| 0.25 * s).collect(),
            outlines: marks.spread_o.iter().map(|s| 0.25 * s).collect(),
            references: marks.marks.references.clone(),
        };
        let bumps = crate::maker::bumps(&EYES, &mesh2, &world.zone, &base2);
        let bending = crate::maker::bending(&mesh2, &bumps);
        eprintln!("{} bumps; base eye with the eyeball {dr} mm larger: width {:.2}, ball {:.2}", bumps.movements.len(), base2.proportions[0], base2.proportions[11]);
        let maker = crate::maker::Maker::new(&EYES, &mesh2, &world.smoother, &bumps, &bending, &base2, &hold);
        // (measure, low span, high span, step), in the measure's units
        let plan: Vec<(&str, f64, f64, f64)> = vec![
            ("upper_inner", 1.6, 1.6, 0.25),
            ("upper_outer", 1.6, 1.6, 0.25),
            ("lower_inner", 1.6, 1.6, 0.25),
            ("lower_outer", 1.6, 1.6, 0.25),
            ("inner_angle", 25.0, 16.0, 3.0),
            ("inner_height", 1.6, 1.6, 0.25),
            ("outer_angle", 25.0, 24.0, 3.0),
            ("outer_lift", 1.6, 1.6, 0.25),
        ];
        let only = std::env::var("SLIDERS_ONLY").ok();
        let plan: Vec<_> = plan.into_iter().filter(|p| only.as_deref().is_none_or(|o| o.split(',').any(|x| x == p.0))).collect();
        let made = parallel(plan.len(), plan.len(), |i| {
            let (m, lo, hi, step) = plan[i];
            let t = std::time::Instant::now();
            let made = maker.make(by(m), [lo, hi], step);
            eprintln!("  {:22} keys {:2}, {:7.2} to {:6.2} (asked {:7.2} to {:6.2}), ends: {} / {} in {:.1?}", by(m).name, made.slider.keys.len(), made.slider.low, made.slider.high, -lo, hi, made.ends[0], made.ends[1], t.elapsed());
            made
        });
        let report: Vec<serde_json::Value> = made
            .iter()
            .zip(&plan)
            .map(|(m, p)| serde_json::json!({"name": by(p.0).name, "measure": p.0, "keys": m.keys, "asked": [-p.1, p.2], "ends": m.ends}))
            .collect();
        let d = crate::set::SliderSet { vertex_count: n, sliders: made.into_iter().map(|m| m.slider).collect(), fields: crate::maker::fields(&bumps) };
        if only.is_some() {
            std::fs::write(data.join("eyes/sliders_try.json"), serde_json::to_string(&d).unwrap()).unwrap();
            std::fs::write(data.join("eyes/named_only.json"), serde_json::to_string_pretty(&report).unwrap()).unwrap();
            return;
        }
        let mut set = crate::set::SliderSet::merge(&[a, b, c, wd, d]);
        set.sliders.sort_by_key(|s| SPECS.iter().position(|p| p.name == s.name).unwrap_or(99));
        std::fs::write(data.join("web/eyes/sliders.json"), serde_json::to_string(&set).unwrap()).unwrap();
        std::fs::write(data.join("eyes/named.json"), serde_json::to_string_pretty(&report).unwrap()).unwrap();
        let solved = ["width", "height", "tilt", "ball", "upper_rise", "lower_drop", "upper_depth", "outer_angle"];
        let mut solve = set.clone();
        solve.sliders.retain(|s| solved.contains(&s.measure.as_str()));
        std::fs::write(data.join("eyes/sliders_solve.json"), serde_json::to_string(&solve).unwrap()).unwrap();
        for s in &set.sliders {
            eprintln!("{:24} {:7.2} to {:6.2}, {} keys", s.name, s.low, s.high, s.keys.len());
        }
    }

    /// Stage A's eye sliders: round 3's set (`eyes/round3/web/sliders.json`) with "Eye height" renamed "Eye opening height", Eyeball
    /// size and Eye tilt moved to the group "whole", and the four whole-eye movements added as sliders of their own. Their keys are the
    /// change of their measure on the base head, in mm of the frame (the spacing counts both eyes) or, for the socket angle, the turn
    /// in degrees. Ranges come from `EYES_<KIND>_LOW` and `_HIGH`, the fade from `EYES_WHOLE_FADE`. Writes `web/eyes/sliders.json` and
    /// `eyes/sliders_solve.json` (round 3's solve set with Eye depth, Eye height and Eye spacing; the socket angle's measure wobbles
    /// too much on KOTOR's meshes to be solved).
    pub fn make_whole(world: &World, data: &Path) {
        let n = world.mesh.base.len();
        let rings = rings(data);
        let by = |m: &str| SPECS.iter().find(|s| s.measure == m).unwrap();
        let from = std::env::var("WHOLE_FROM").map(std::path::PathBuf::from).unwrap_or_else(|_| data.join("eyes/round3/web/sliders.json"));
        let mut old: crate::set::SliderSet = serde_json::from_str(&std::fs::read_to_string(&from).expect("round 3's sliders")).unwrap();
        for s in &mut old.sliders {
            if let Some(spec) = SPECS.iter().find(|p| p.measure == s.measure) {
                s.name = spec.name.into();
                s.hint = spec.hint.into();
                s.group = spec.group.into();
            }
        }
        let fade = env_f("EYES_WHOLE_FADE", 2.0);
        let scale = Frame::from_eyes(world.mesh.eyes).scale / 1000.0;
        let mut parts = vec![old.clone()];
        for (kind, lo, hi, step, per) in [
            ("depth", -5.0, 4.0, 0.0, scale),
            ("level", -4.0, 4.0, 0.0, scale),
            ("spacing", -3.0, 5.0, 0.0, 2.0 * scale),
            ("pitch", -12.0, 12.0, 1.0, 1.0),
        ] {
            let up = kind.to_uppercase();
            let (lo, hi) = (env_f(&format!("EYES_{up}_LOW"), lo), env_f(&format!("EYES_{up}_HIGH"), hi));
            let mut amounts = vec![lo, 0.0, hi];
            if step > 0.0 {
                amounts = Vec::new();
                let mut a = lo;
                while a <= hi + 1e-9 {
                    amounts.push((a / step).round() * step);
                    a += step;
                }
            }
            let keys: Vec<(f64, Vec<V3>)> =
                amounts.iter().map(|&a| (a * per, if a == 0.0 { Vec::new() } else { whole_field(&world.mesh, &rings, kind, a, fade) })).collect();
            let unit = if kind == "pitch" { "1 degrees of turn of the eye, pointing up" } else { "1 mm of the measure, in mm of the frame" };
            parts.push(field_slider(by(kind), unit, &keys, n));
            eprintln!("{:14} {:6.2} to {:6.2}", by(kind).name, lo * per, hi * per);
        }
        let mut set = crate::set::SliderSet::merge(&parts);
        set.sliders.sort_by_key(|s| SPECS.iter().position(|p| p.name == s.name).unwrap_or(99));
        std::fs::write(data.join("web/eyes/sliders.json"), serde_json::to_string(&set).unwrap()).unwrap();
        let solved = ["width", "height", "tilt", "ball", "upper_rise", "lower_drop", "upper_depth", "outer_angle", "depth", "level", "spacing"];
        let mut solve = set.clone();
        solve.sliders.retain(|s| solved.contains(&s.measure.as_str()));
        std::fs::write(data.join("eyes/sliders_solve.json"), serde_json::to_string(&solve).unwrap()).unwrap();
        for s in &set.sliders {
            eprintln!("{:24} {:7.2} to {:6.2}, {} keys, group {}", s.name, s.low, s.high, s.keys.len(), s.group);
        }
    }

    /// Stage B's eye sliders. The whole-eye sliders and Upper lid fullness keep stage A's keys and fields (`eyes/stage_a/web/sliders.json`,
    /// or `STAGE_A_FROM`). Every lid and corner slider is a curve slider of `eye_curves`: a turn about a line through the eyeball's
    /// centre that changes smoothly from place to place, keyed every step of its range. Eye width's keys are the change of the distance
    /// between the corners, in mm of the head. The two caruncle sliders move no vertex: the caruncle piece reads their values. Writes
    /// `web/eyes/sliders.json` and `eyes/sliders_solve.json` (the sliders whose measure KOTOR's heads can score; `EYES_SOLVE` lists
    /// their measures, joined by commas).
    pub fn make_b(world: &World, data: &Path) {
        use crate::eye_curves::{Curve, Fades, field};
        let n = world.mesh.base.len();
        let from = std::env::var("STAGE_A_FROM").map(std::path::PathBuf::from).unwrap_or_else(|_| data.join("eyes/stage_a/web/sliders.json"));
        let old: crate::set::SliderSet = serde_json::from_str(&std::fs::read_to_string(&from).unwrap_or_else(|e| panic!("{}: {e}", from.display()))).unwrap();
        let kept_names = ["Eye depth", "Eye height", "Eye spacing", "Socket angle", "Eye tilt", "Eyeball size", "Upper lid fullness"];
        let rebuild_ball = !std::env::var("EYES_BALL_REBUILD").is_ok_and(|v| v == "0");
        let a = crate::set::SliderSet {
            vertex_count: n,
            sliders: old.sliders.iter().filter(|s| kept_names.contains(&s.name.as_str()) && !(rebuild_ball && s.name == "Eyeball size")).cloned().collect(),
            fields: old.fields.clone(),
        };
        let ball_part = rebuild_ball.then(|| {
            let rings = rings(data);
            let keys: Vec<(f64, Vec<V3>)> = (-4..=8).map(|k| k as f64 * 0.5).map(|v| (v, if v == 0.0 { Vec::new() } else { ball_field(&world.mesh, &rings, v) })).collect();
            field_slider(SPECS.iter().find(|s| s.measure == "ball").unwrap(), "1 mm of the radius of the eyeball", &keys, n)
        });
        let by = |m: &str| SPECS.iter().find(|s| s.measure == m).unwrap();
        let fades = Fades::default();
        let steps = |lo: f64, hi: f64, step: f64| -> Vec<f64> {
            let mut v = Vec::new();
            let mut x = lo;
            while x <= hi + 1e-9 {
                v.push((x / step).round() * step);
                x += step;
            }
            if !v.contains(&0.0) {
                v.push(0.0);
                v.sort_by(f64::total_cmp);
            }
            v
        };
        let curve_slider = |m: &str, curve: Curve, lo: f64, hi: f64, step: f64, unit: &str| {
            let keys: Vec<(f64, Vec<V3>)> = steps(lo, hi, step).into_iter().map(|x| (x, if x == 0.0 { Vec::new() } else { field(&world.mesh.base, curve, x, &fades) })).collect();
            field_slider(by(m), unit, &keys, n)
        };
        let r = |k: &str, lo: f64, hi: f64| (env_f(&format!("EYES_{k}_LOW"), lo), env_f(&format!("EYES_{k}_HIGH"), hi));
        let mut parts = vec![a];
        parts.extend(ball_part);
        let lids = [("upper_inner", true, 0usize), ("upper_rise", true, 1), ("upper_outer", true, 2), ("lower_inner", false, 0), ("lower_drop", false, 1), ("lower_outer", false, 2)];
        for (m, upper, point) in lids {
            let (lo, hi) = r(&m.to_uppercase(), -2.0, 2.0);
            parts.push(curve_slider(m, Curve::Lid { upper, point }, lo, hi, 0.5, "1 mm of the lid's edge, up"));
        }
        let (lo, hi) = r("HEIGHT", -2.5, 3.0);
        parts.push(curve_slider("height", Curve::Opening, lo, hi, 0.5, "1 mm of height of the eye's opening, in the middle"));
        for inner in [true, false] {
            let (k, h, ang, rnd) = if inner { ("inner_reach", "inner_height", "inner_angle", "inner_round") } else { ("outer_reach", "outer_lift", "outer_angle", "outer_round") };
            let (lo, hi) = r(&k.to_uppercase(), -4.0, 4.0);
            parts.push(curve_slider(k, Curve::CornerAlong { inner }, lo, hi, 0.5, "1 mm of the corner along the eyeball, away from the eye's middle"));
            let (lo, hi) = r(&h.to_uppercase(), -2.5, 2.5);
            parts.push(curve_slider(h, Curve::CornerHeight { inner }, lo, hi, 0.5, "1 mm of the corner, up"));
            let (lo, hi) = r(&ang.to_uppercase(), -30.0, 35.0);
            parts.push(curve_slider(ang, Curve::CornerAngle { inner }, lo, hi, 5.0, "1 degree of the angle between the lids at the corner"));
            let (lo, hi) = r(&rnd.to_uppercase(), -1.5, 2.0);
            parts.push(curve_slider(rnd, Curve::CornerRound { inner }, lo, hi, 0.5, "1 mm of the lids apart next to the corner"));
        }
        // Eye width: a stretch of the azimuth about the eye's middle by a share k; its keys are the change of the eye's length along
        // the eyeball, k times the base eye's length (the corners already lie about 75 degrees round the eyeball, so the distance
        // between them barely grows)
        let len0 = crate::eye_curves::geos(&world.mesh.base)[0].length();
        let (klo, khi) = r("WIDTH_STRETCH", -0.2, 0.25);
        let width_keys: Vec<(f64, Vec<V3>)> =
            steps(klo, khi, 0.05).into_iter().map(|k| (k * len0, if k == 0.0 { Vec::new() } else { field(&world.mesh.base, Curve::Width, k, &fades) })).collect();
        eprintln!("the base eye is {len0:.1} mm long along the eyeball");
        parts.push(field_slider(by("width"), "1 mm of the length of the eye along the eyeball", &width_keys, n));
        for (name, lo, hi) in [("Caruncle width", -2.0, 3.0)] {
            let spec = SPECS.iter().find(|s| s.name == name).unwrap();
            let mut s = crate::set::Slider::from_movement(spec.name, "named", "1 mm", &[]);
            s.zone = "eyes".into();
            s.hint = spec.hint.into();
            s.group = spec.group.into();
            s.low = lo;
            s.high = hi;
            parts.push(crate::set::SliderSet { vertex_count: n, sliders: vec![s], fields: Vec::new() });
        }
        let mut set = crate::set::SliderSet::merge(&parts);
        set.sliders.sort_by_key(|s| SPECS.iter().position(|p| p.name == s.name).unwrap_or(99));
        std::fs::write(data.join("web/eyes/sliders.json"), serde_json::to_string(&set).unwrap()).unwrap();
        let solved: Vec<String> = std::env::var("EYES_SOLVE")
            .unwrap_or_else(|_| "width,height,tilt,ball,upper_rise,lower_drop,upper_depth,depth,level,spacing,upper_inner,upper_outer,lower_inner,lower_outer".into())
            .split(',')
            .map(String::from)
            .collect();
        let mut solve = set.clone();
        solve.sliders.retain(|s| solved.contains(&s.measure));
        std::fs::write(data.join("eyes/sliders_solve.json"), serde_json::to_string(&solve).unwrap()).unwrap();
        for s in &set.sliders {
            eprintln!("{:26} {:7.2} to {:6.2}, {:2} keys, group {}", s.name, s.low, s.high, s.keys.len(), s.group);
        }
    }

    /// pmhc04's round 1 (2026-11-21) on the eye set in `web/eyes/sliders.json` (or `SLIDERS_SET`): adds Eye prominence (keys from
    /// `EYES_PROM_LOW` to `EYES_PROM_HIGH` mm, every 0.5), gives Inner corner depth its measure inner_tip, and adds both to
    /// `eyes/sliders_solve.json`. The marks file `eyes/marks.json` gets the rows of the measures added in this round (KOTOR's spread,
    /// and a quarter of it as the mark) when it does not have them yet.
    pub fn make_pmhc_r1(world: &World, data: &Path) {
        use crate::eye_curves::{Curve, Fades, field};
        let mut set = read_set(data);
        let base = &world.mesh.base;
        let fades = Fades::default();
        let lo = env_f("EYES_PROM_LOW", 0.0);
        let hi = env_f("EYES_PROM_HIGH", 6.0);
        let values: Vec<f64> = ((lo / 0.5).round() as i64..=(hi / 0.5).round() as i64).map(|k| k as f64 * 0.5).collect();
        let name = "Eye prominence";
        let spec = SPECS.iter().find(|s| s.name == name).unwrap();
        set.sliders.retain(|s| s.name != name);
        let mut slider = crate::set::Slider::from_movement(spec.name, "named", "1 mm of the eyeball, forward", &[]);
        slider.zone = "eyes".into();
        slider.hint = spec.hint.into();
        slider.group = spec.group.into();
        slider.measure = spec.measure.into();
        for &v in &values {
            let fields = if v == 0.0 {
                Vec::new()
            } else {
                let f = set.fields.len() as u32;
                set.fields.push(crate::set::Field::from_movement(&field(base, Curve::Prominence, v, &fades)));
                vec![f]
            };
            let weights = vec![1.0; fields.len()];
            slider.keys.push(crate::set::Key { value: v, fields, weights });
        }
        slider.low = values[0];
        slider.high = values[values.len() - 1];
        set.sliders.push(slider);
        for s in set.sliders.iter_mut() {
            if s.name == "Inner corner depth" {
                s.measure = "inner_tip".into();
            }
        }
        set.sliders.sort_by_key(|s| SPECS.iter().position(|p| p.name == s.name).unwrap_or(99));
        set.vertex_count = base.len();
        std::fs::write(data.join("web/eyes/sliders.json"), serde_json::to_string(&set).unwrap()).unwrap();
        let solve_path = data.join("eyes/sliders_solve.json");
        if let Ok(text) = std::fs::read_to_string(&solve_path) {
            let old: crate::set::SliderSet = serde_json::from_str(&text).unwrap();
            let mut names: Vec<String> = old.sliders.iter().map(|s| s.name.clone()).collect();
            names.extend([name.to_string(), "Inner corner depth".to_string()]);
            let mut solve = set.clone();
            solve.sliders.retain(|s| names.contains(&s.name));
            std::fs::write(&solve_path, serde_json::to_string(&solve).unwrap()).unwrap();
        }
        let marks_path = data.join("eyes/marks.json");
        let mut m = crate::checks::Marks::read(&marks_path);
        let scored: Vec<&crate::heads::Head> = world.heads.iter().filter(|h| h.kotor_error.is_none()).collect();
        for i in m.spread_p.len()..PROPORTIONS.len() {
            let spread = crate::heads::sd(&scored.iter().map(|h| h.kotor.proportions[i]).collect::<Vec<_>>());
            m.spread_p.push(spread);
            m.marks.proportions.push(0.25 * spread);
            eprintln!("marks: {} spread {spread:.3}, mark {:.3}", PROPORTIONS[i].key, 0.25 * spread);
        }
        std::fs::write(&marks_path, serde_json::to_string_pretty(&m.to_json()).unwrap()).unwrap();
        for s in &set.sliders {
            eprintln!("{:26} {:7.2} to {:6.2}, {:2} keys, group {}, measure {}", s.name, s.low, s.high, s.keys.len(), s.group, s.measure);
        }
    }

    /// pmhc04's round 2 (2026-11-21) on the eye set in `web/eyes/sliders.json` (or `SLIDERS_SET`): adds Outer corner depth (keys from
    /// `EYES_OUTER_LOW` to `EYES_OUTER_HIGH` mm, every 0.5) with the measure outer_back, adds it to `eyes/sliders_solve.json`, and gives
    /// `eyes/marks.json` the rows of the measures added in this round (outer_bulge).
    /// pmhc04's round 4 (2026-11-21): gives `eyes/marks.json` the rows of the profile outline's measures (profile_h2 to profile_len),
    /// each with KOTOR's spread and a mark of a quarter of it.
    pub fn marks_pmhc_r4(world: &World, data: &Path) {
        let marks_path = data.join("eyes/marks.json");
        let mut m = crate::checks::Marks::read(&marks_path);
        let scored: Vec<&crate::heads::Head> = world.heads.iter().filter(|h| h.kotor_error.is_none()).collect();
        for i in m.spread_p.len()..PROPORTIONS.len() {
            let v: Vec<f64> = scored.iter().map(|h| h.kotor.proportions[i]).filter(|x| x.is_finite()).collect();
            let spread = crate::heads::sd(&v);
            m.spread_p.push(spread);
            m.marks.proportions.push(0.25 * spread);
            let mut wob: Vec<f64> = scored.iter().filter_map(|h| {
                let pe = &h.kotor.marks["per_eye"];
                let (a, b) = (pe[0][i].as_f64()?, pe[1][i].as_f64()?);
                (a.is_finite() && b.is_finite()).then(|| (a - b).abs())
            }).collect();
            wob.sort_by(f64::total_cmp);
            let w = wob.get(wob.len() / 2).copied().unwrap_or(f64::NAN);
            eprintln!("marks: {} spread {spread:.3}, mark {:.3}, {} heads, left against right median {w:.3}", PROPORTIONS[i].key, 0.25 * spread, v.len());
            // MARKS_LIST=1 also prints KOTOR's value per head, each eye apart
            if std::env::var("MARKS_LIST").is_ok() {
                for h in &scored {
                    let pe = &h.kotor.marks["per_eye"];
                    eprintln!("  {} {} {:.3} {:.3}", PROPORTIONS[i].key, h.name, pe[0][i].as_f64().unwrap_or(f64::NAN), pe[1][i].as_f64().unwrap_or(f64::NAN));
                }
            }
        }
        std::fs::write(&marks_path, serde_json::to_string_pretty(&m.to_json()).unwrap()).unwrap();
    }

    pub fn make_pmhc_r2(world: &World, data: &Path) {
        use crate::eye_curves::{Curve, Fades, field};
        let mut set = read_set(data);
        let base = &world.mesh.base;
        let fades = Fades::default();
        let lo = env_f("EYES_OUTER_LOW", -4.0);
        let hi = env_f("EYES_OUTER_HIGH", 2.0);
        let values: Vec<f64> = ((lo / 0.5).round() as i64..=(hi / 0.5).round() as i64).map(|k| k as f64 * 0.5).collect();
        let name = "Outer corner depth";
        let spec = SPECS.iter().find(|s| s.name == name).unwrap();
        set.sliders.retain(|s| s.name != name);
        let mut slider = crate::set::Slider::from_movement(spec.name, "named", "1 mm of the outer corner, forward", &[]);
        slider.zone = "eyes".into();
        slider.hint = spec.hint.into();
        slider.group = spec.group.into();
        slider.measure = spec.measure.into();
        for &v in &values {
            let fields = if v == 0.0 {
                Vec::new()
            } else {
                let f = set.fields.len() as u32;
                set.fields.push(crate::set::Field::from_movement(&field(base, Curve::CornerDepth { inner: false }, v, &fades)));
                vec![f]
            };
            let weights = vec![1.0; fields.len()];
            slider.keys.push(crate::set::Key { value: v, fields, weights });
        }
        slider.low = values[0];
        slider.high = values[values.len() - 1];
        set.sliders.push(slider);
        set.sliders.sort_by_key(|s| SPECS.iter().position(|p| p.name == s.name).unwrap_or(99));
        set.vertex_count = base.len();
        std::fs::write(data.join("web/eyes/sliders.json"), serde_json::to_string(&set).unwrap()).unwrap();
        let solve_path = data.join("eyes/sliders_solve.json");
        if let Ok(text) = std::fs::read_to_string(&solve_path) {
            let old: crate::set::SliderSet = serde_json::from_str(&text).unwrap();
            let mut names: Vec<String> = old.sliders.iter().map(|s| s.name.clone()).collect();
            names.push(name.to_string());
            let mut solve = set.clone();
            solve.sliders.retain(|s| names.contains(&s.name));
            std::fs::write(&solve_path, serde_json::to_string(&solve).unwrap()).unwrap();
        }
        let marks_path = data.join("eyes/marks.json");
        let mut m = crate::checks::Marks::read(&marks_path);
        let scored: Vec<&crate::heads::Head> = world.heads.iter().filter(|h| h.kotor_error.is_none()).collect();
        for i in m.spread_p.len()..PROPORTIONS.len() {
            let spread = crate::heads::sd(&scored.iter().map(|h| h.kotor.proportions[i]).collect::<Vec<_>>());
            m.spread_p.push(spread);
            m.marks.proportions.push(0.25 * spread);
            eprintln!("marks: {} spread {spread:.3}, mark {:.3}", PROPORTIONS[i].key, 0.25 * spread);
        }
        std::fs::write(&marks_path, serde_json::to_string_pretty(&m.to_json()).unwrap()).unwrap();
        for s in &set.sliders {
            eprintln!("{:26} {:7.2} to {:6.2}, {:2} keys, group {}, measure {}", s.name, s.low, s.high, s.keys.len(), s.group, s.measure);
        }
    }

    /// Round 5's changes to the eye set in `web/eyes/sliders.json` (or `SLIDERS_SET`). Adds Inner corner depth (keys from
    /// `EYES_DEPTH_LOW` to `EYES_DEPTH_HIGH` mm, every 0.5) and Lower lid flatness (from `EYES_FLAT_LOW` to `EYES_FLAT_HIGH`, every 0.1),
    /// replacing them when they are there. `EYES_REMAKE_LOWER_MIDDLE="<low>:<high>"` makes Lower lid drop, middle's keys again over that
    /// range of its own values (positive lowers the lid), every 0.5. Every key is made with today's fields. Neither new slider has a
    /// measure, so neither is solved. Writes `web/eyes/sliders.json`; `eyes/sliders_solve.json` gets the remade Lower lid drop, middle.
    pub fn make_r5(world: &World, data: &Path) {
        use crate::eye_curves::{Curve, Fades, field};
        let mut set = read_set(data);
        let n = world.mesh.base.len();
        let base = &world.mesh.base;
        let fades = Fades::default();
        let steps = |lo: f64, hi: f64, step: f64| -> Vec<f64> {
            let k0 = (lo / step).round() as i64;
            let k1 = (hi / step).round() as i64;
            (k0..=k1).map(|k| k as f64 * step).collect()
        };
        let mut add = |set: &mut crate::set::SliderSet, name: &str, curve: Curve, values: Vec<f64>, sign: f64, unit: &str| {
            let spec = SPECS.iter().find(|s| s.name == name).unwrap();
            set.sliders.retain(|s| s.name != name);
            let mut s = crate::set::Slider::from_movement(spec.name, "named", unit, &[]);
            s.zone = "eyes".into();
            s.hint = spec.hint.into();
            s.group = spec.group.into();
            s.measure = spec.measure.into();
            for &v in &values {
                let fields = if v == 0.0 {
                    Vec::new()
                } else {
                    let f = set.fields.len() as u32;
                    set.fields.push(crate::set::Field::from_movement(&field(base, curve, sign * v, &fades)));
                    vec![f]
                };
                let weights = vec![1.0; fields.len()];
                s.keys.push(crate::set::Key { value: v, fields, weights });
            }
            s.low = values[0];
            s.high = values[values.len() - 1];
            set.sliders.push(s);
        };
        let depth = steps(env_f("EYES_DEPTH_LOW", -3.0), env_f("EYES_DEPTH_HIGH", 5.0), 0.5);
        add(&mut set, "Inner corner depth", Curve::CornerDepth { inner: true }, depth, 1.0, "1 mm of the inner corner, forward");
        let flat = steps(env_f("EYES_FLAT_LOW", -0.5), env_f("EYES_FLAT_HIGH", 1.0), 0.1);
        add(&mut set, "Lower lid flatness", Curve::LowerFlat, flat, 1.0, "the share of the lower lid's dip taken away");
        if let Ok(r) = std::env::var("EYES_REMAKE_LOWER_MIDDLE") {
            let (lo, hi) = r.split_once(':').map(|(a, b)| (a.parse::<f64>().unwrap(), b.parse::<f64>().unwrap())).unwrap();
            let name = "Lower lid drop, middle";
            let old = set.sliders.iter().find(|s| s.name == name).unwrap().clone();
            add(&mut set, name, Curve::Lid { upper: false, point: 1 }, steps(lo, hi, 0.5), -1.0, "1 mm of the lid's edge, down");
            let s = set.sliders.iter_mut().find(|s| s.name == name).unwrap();
            s.measure = old.measure.clone();
            s.group = old.group.clone();
        }
        set.sliders.sort_by_key(|s| SPECS.iter().position(|p| p.name == s.name).unwrap_or(99));
        set.vertex_count = n;
        std::fs::write(data.join("web/eyes/sliders.json"), serde_json::to_string(&set).unwrap()).unwrap();
        let solve_path = data.join("eyes/sliders_solve.json");
        if let Ok(text) = std::fs::read_to_string(&solve_path) {
            let old: crate::set::SliderSet = serde_json::from_str(&text).unwrap();
            let names: Vec<String> = old.sliders.iter().map(|s| s.name.clone()).collect();
            let mut solve = set.clone();
            solve.sliders.retain(|s| names.contains(&s.name));
            std::fs::write(&solve_path, serde_json::to_string(&solve).unwrap()).unwrap();
        }
        for s in &set.sliders {
            eprintln!("{:26} {:7.2} to {:6.2}, {:2} keys, group {}", s.name, s.low, s.high, s.keys.len(), s.group);
        }
    }

    /// Round 4's changes to the eye set in `web/eyes/sliders.json` (or `SLIDERS_SET`): the lower lid sliders turned round so that a
    /// positive value lowers the lid, as their measures count; the slider Upper lid cover added (it moves no vertex; the lid piece reads
    /// it); and each keyed slider in `EYES_CALIBRATE` given key values equal to its own measure's change on the base head as the page
    /// shows it, so that one unit of the slider is one unit of its measure. Prints each slider's response on the base head. Writes
    /// `web/eyes/sliders.json` and `eyes/sliders_solve.json` (the sliders whose measures are in `EYES_SOLVE`).
    pub fn make_r4(world: &World, data: &Path) {
        let mut set = read_set(data);
        let n = world.mesh.base.len();
        for s in set.sliders.iter_mut() {
            let Some(spec) = SPECS.iter().find(|p| p.measure == s.measure && !s.measure.is_empty()) else { continue };
            if ["lower_inner", "lower_drop", "lower_outer"].contains(&s.measure.as_str()) && s.name != spec.name {
                for k in s.keys.iter_mut() {
                    k.value = -k.value;
                }
                s.keys.reverse();
                (s.low, s.high) = (-s.high, -s.low);
                s.unit = "1 mm of the lid's edge, down".into();
            }
            s.name = spec.name.into();
            s.hint = spec.hint.into();
        }
        if !set.sliders.iter().any(|s| s.name == COVER_SLIDER) {
            let spec = SPECS.iter().find(|s| s.name == COVER_SLIDER).unwrap();
            let mut s = crate::set::Slider::from_movement(spec.name, "named", "1 mm of the lid piece, down, in the middle", &[]);
            s.zone = "eyes".into();
            s.hint = spec.hint.into();
            s.group = spec.group.into();
            s.measure = spec.measure.into();
            s.low = env_f("EYES_COVER_LOW", -1.0);
            s.high = env_f("EYES_COVER_HIGH", 3.0);
            set.sliders.push(s);
        }
        set.sliders.sort_by_key(|s| SPECS.iter().position(|p| p.name == s.name).unwrap_or(99));
        // EYES_EXTEND="<measure>:<low>:<high>;..." makes a corner or lid curve slider reach further, with new keys every 0.5 beyond its
        // ends (in its values before any calibration)
        for a in std::env::var("EYES_EXTEND").unwrap_or_default().split(';').filter(|a| !a.is_empty()) {
            use crate::eye_curves::{Curve, Fades, field};
            let p: Vec<&str> = a.split(':').collect();
            let curve = match p[0] {
                "inner_reach" => Curve::CornerAlong { inner: true },
                "outer_reach" => Curve::CornerAlong { inner: false },
                other => panic!("no curve for {other}"),
            };
            let (lo, hi): (f64, f64) = (p[1].parse().unwrap(), p[2].parse().unwrap());
            let j = set.sliders.iter().position(|s| s.measure == p[0]).unwrap();
            // EYES_REMAKE=1 makes every key again with today's fields (for a changed field), not only the new ones
            if std::env::var("EYES_REMAKE").is_ok_and(|v| v == "1") {
                let s = &mut set.sliders[j];
                s.keys.retain(|k| k.value == 0.0);
                s.low = 0.0;
                s.high = 0.0;
            }
            let (slo, shi) = (set.sliders[j].low, set.sliders[j].high);
            let mut extra: Vec<f64> = Vec::new();
            let mut x = slo - 0.5;
            while x >= lo - 1e-9 {
                extra.push(x);
                x -= 0.5;
            }
            let mut x = shi + 0.5;
            while x <= hi + 1e-9 {
                extra.push(x);
                x += 0.5;
            }
            for x in extra {
                let f = set.fields.len() as u32;
                set.fields.push(crate::set::Field::from_movement(&field(&world.mesh.base, curve, x, &Fades::default())));
                let s = &mut set.sliders[j];
                s.keys.push(crate::set::Key { value: x, fields: vec![f], weights: vec![1.0] });
                s.keys.sort_by(|a, b| a.value.total_cmp(&b.value));
                s.low = s.low.min(x);
                s.high = s.high.max(x);
            }
        }
        // EYES_CAP="<slider name>:<low>:<high>;..." cuts a slider's range (in its values before any calibration)
        let field_count = set.fields.len();
        for a in std::env::var("EYES_CAP").unwrap_or_default().split(';').filter(|a| !a.is_empty()) {
            let p: Vec<&str> = a.split(':').collect();
            let s = set.sliders.iter_mut().find(|s| s.name == p[0]).unwrap_or_else(|| panic!("no slider {}", p[0]));
            cap_slider(s, field_count, p[1].parse().unwrap(), p[2].parse().unwrap());
        }
        let calibrate: Vec<String> = std::env::var("EYES_CALIBRATE").unwrap_or_default().split(',').filter(|x| !x.is_empty()).map(String::from).collect();
        let frame = crate::measure::head_frame(world.mesh.eyes);
        let base = &world.mesh.base;
        let measure_at = |set: &crate::set::SliderSet, v: &[f64]| {
            measure(&shown_surface(&set.build(&world.smoother, base, v), &world.mesh.triangles, &frame, cover_of(set, v))).ok()
        };
        let zero = vec![0.0; set.sliders.len()];
        let m0 = measure_at(&set, &zero).expect("the base head's eyes");
        eprintln!("base head: {}", PROPORTIONS.iter().zip(&m0.proportions).map(|(p, x)| format!("{} {x:.2}", p.key)).collect::<Vec<_>>().join(", "));
        let mut changed = Vec::new();
        for j in 0..set.sliders.len() {
            let s = &set.sliders[j];
            if s.measure.is_empty() {
                continue;
            }
            let k = crate::feature::proportion_index(&EYES, &s.measure);
            let points: Vec<f64> = if s.keys.is_empty() {
                (0..=8).map(|i| s.low + (s.high - s.low) * i as f64 / 8.0).collect()
            } else {
                s.keys.iter().map(|k| k.value).collect()
            };
            let resp: Vec<f64> = parallel(points.len(), threads(), |i| {
                let mut v = zero.clone();
                v[j] = points[i];
                measure_at(&set, &v).map(|m| m.proportions[k] - m0.proportions[k]).unwrap_or(f64::NAN)
            });
            let line: Vec<String> = points.iter().zip(&resp).map(|(a, b)| format!("{a:.2}:{b:.2}")).collect();
            eprintln!("{:26} {}", s.name, line.join(" "));
            if calibrate.contains(&s.measure) && !s.keys.is_empty() {
                changed.push((j, resp));
            }
        }
        for (j, resp) in changed {
            let s = &mut set.sliders[j];
            let z = s.keys.iter().position(|k| k.value == 0.0).expect("a key at 0");
            // pool the responses that step backward, so the key values only grow; then keep each step at least 2% of the typical step
            let mut r: Vec<f64> = resp.iter().map(|x| if x.is_finite() { *x } else { 0.0 }).collect();
            r[z] = 0.0;
            let mut blocks: Vec<(f64, usize)> = Vec::new();
            for &x in &r {
                blocks.push((x, 1));
                while blocks.len() > 1 && blocks[blocks.len() - 2].0 >= blocks[blocks.len() - 1].0 {
                    let (b, nb) = blocks.pop().unwrap();
                    let (a, na) = blocks.pop().unwrap();
                    blocks.push(((a * na as f64 + b * nb as f64) / (na + nb) as f64, na + nb));
                }
            }
            let mut out: Vec<f64> = blocks.iter().flat_map(|&(x, c)| std::iter::repeat_n(x, c)).collect();
            let typical = (out[out.len() - 1] - out[0]).abs() / (out.len() - 1) as f64;
            let shift = out[z];
            for x in out.iter_mut() {
                *x -= shift;
            }
            for i in z + 1..out.len() {
                out[i] = out[i].max(out[i - 1] + 0.02 * typical);
            }
            for i in (0..z).rev() {
                out[i] = out[i].min(out[i + 1] - 0.02 * typical);
            }
            for (key, x) in s.keys.iter_mut().zip(&out) {
                key.value = *x;
            }
            s.low = out[0];
            s.high = out[out.len() - 1];
            eprintln!("calibrated {}: {:.2} to {:.2}", s.name, s.low, s.high);
        }
        set.vertex_count = n;
        std::fs::write(data.join("web/eyes/sliders.json"), serde_json::to_string(&set).unwrap()).unwrap();
        let solved: Vec<String> = std::env::var("EYES_SOLVE")
            .unwrap_or_else(|_| "width,height,tilt,ball,upper_rise,lower_drop,upper_depth,depth,level,spacing,upper_inner,upper_outer,lower_inner,lower_outer,outer_reach,outer_lift,inner_reach,cover".into())
            .split(',')
            .map(String::from)
            .collect();
        let mut solve = set.clone();
        solve.sliders.retain(|s| solved.contains(&s.measure));
        std::fs::write(data.join("eyes/sliders_solve.json"), serde_json::to_string(&solve).unwrap()).unwrap();
        for s in &set.sliders {
            eprintln!("{:26} {:7.2} to {:6.2}, {:2} keys, group {}", s.name, s.low, s.high, s.keys.len(), s.group);
        }
    }

    /// Places each head's whole eye where KOTOR's is, after the solve: for every head of `web/eyes/page.json`, the solved values (full
    /// ranges, all sliders) are kept for every slider but Eye depth, Eye height, Eye spacing, Eye width and Eye tilt, whose values are
    /// moved by what their measure still misses, five times over, within their ranges. Their keys are in their measure's units, so one
    /// unit of value moves their measure by about one unit. Rewrites the values and the measure in the page, and prints the misses
    /// before and after.
    pub fn place_whole(world: &World, set: &crate::set::SliderSet, data: &Path) {
        let path = data.join("web/eyes/page.json");
        let mut page: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let n = set.sliders.len();
        let names: Vec<String> = page["sliders"].as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap().to_string()).collect();
        let order: Vec<usize> = names.iter().map(|nm| set.sliders.iter().position(|s| &s.name == nm).expect("a slider of the page")).collect();
        let whole: Vec<(usize, usize)> = ["depth", "level", "spacing", "width", "tilt"]
            .iter()
            .map(|m| (set.sliders.iter().position(|s| s.measure == *m).unwrap(), crate::feature::proportion_index(&EYES, m)))
            .collect();
        let results = parallel(world.heads.len(), threads(), |i| {
            let h = &world.heads[i];
            let entry = &page["heads"][i]["solved"]["full"]["counts"][names.len().to_string()];
            let Some(vals) = entry["values"].as_array() else { return None };
            let mut values = vec![0.0; n];
            for (k, v) in vals.iter().enumerate() {
                values[order[k]] = v.as_f64().unwrap_or(0.0);
            }
            if h.kotor_error.is_some() {
                return None;
            }
            let measure_at = |v: &[f64]| measure(&shown_surface(&set.build(&world.smoother, &h.start, v), &world.mesh.triangles, &h.frame, cover_of(set, v))).ok();
            let before = measure_at(&values)?;
            let mut m = before.clone();
            for _ in 0..5 {
                for &(j, k) in &whole {
                    let miss = h.kotor.proportions[k] - m.proportions[k];
                    if miss.is_finite() {
                        values[j] = (values[j] + miss).clamp(set.sliders[j].low, set.sliders[j].high);
                    }
                }
                m = measure_at(&values)?;
            }
            Some((values, before, m))
        });
        let mut rows = vec![Vec::new(); whole.len()];
        for (i, r) in results.into_iter().enumerate() {
            let Some((values, before, after)) = r else { continue };
            let h = &world.heads[i];
            for (w, &(_, k)) in whole.iter().enumerate() {
                rows[w].push(((h.kotor.proportions[k] - before.proportions[k]).abs(), (h.kotor.proportions[k] - after.proportions[k]).abs()));
            }
            let e = &mut page["heads"][i]["solved"]["full"]["counts"][names.len().to_string()];
            e["values"] = serde_json::json!(order.iter().map(|&j| values[j]).collect::<Vec<f64>>());
            e["measured"] = serde_json::to_value(&after).unwrap();
        }
        for (w, &(j, _)) in whole.iter().enumerate() {
            let mut b: Vec<f64> = rows[w].iter().map(|r| r.0).collect();
            let mut a: Vec<f64> = rows[w].iter().map(|r| r.1).collect();
            b.sort_by(f64::total_cmp);
            a.sort_by(f64::total_cmp);
            eprintln!("{:12} median miss {:.2} -> {:.2}, worst {:.2} -> {:.2} ({} heads)", set.sliders[j].name, b[b.len() / 2], a[a.len() / 2], b[b.len() - 1], a[a.len() - 1], a.len());
        }
        std::fs::write(&path, serde_json::to_string(&page).unwrap()).unwrap();
    }

    /// pmhc04's round 1 guard: the heads named keep their values from `old_page` (an earlier `web/eyes/page.json`), matched by slider
    /// name, with 0 for a slider that page does not have. Their measure in `web/eyes/page.json` is read again on those values.
    pub fn guard_pmhc_r1(world: &World, set: &crate::set::SliderSet, data: &Path, old_page: &Path, heads: &[String]) {
        let path = data.join("web/eyes/page.json");
        let mut page: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let old: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(old_page).unwrap()).unwrap();
        let names: Vec<String> = page["sliders"].as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap().to_string()).collect();
        let order: Vec<usize> = names.iter().map(|nm| set.sliders.iter().position(|s| &s.name == nm).expect("a slider of the page")).collect();
        for name in heads {
            let i = world.heads.iter().position(|h| &h.name == name).expect("head");
            let h = &world.heads[i];
            let values = solved_values(&old, set, name);
            let m = measure(&shown_surface(&set.build(&world.smoother, &h.start, &values), &world.mesh.triangles, &h.frame, cover_of(set, &values)));
            let e = &mut page["heads"][i]["solved"]["full"]["counts"][names.len().to_string()];
            e["values"] = serde_json::json!(order.iter().map(|&j| values[j]).collect::<Vec<f64>>());
            if let Ok(m) = m {
                e["measured"] = serde_json::to_value(&m).unwrap();
            }
            eprintln!("{name}: kept its earlier values");
        }
        std::fs::write(&path, serde_json::to_string(&page).unwrap()).unwrap();
    }

    pub fn read_set(data: &Path) -> crate::set::SliderSet {
        let path = std::env::var("SLIDERS_SET").map(std::path::PathBuf::from).unwrap_or_else(|_| data.join("web/eyes/sliders.json"));
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
    }

    /// Each slider at its low end, 0 and its high end on the base head (or on the heads named, as stand-ins): one eye from the front
    /// with the lid open and closed, and from three-quarter, with the eyeball and the generated lid piece.
    pub fn ends_sheet(world: &World, rig: &LidRig, set: &crate::set::SliderSet, names: &[String]) -> RgbImage {
        let base_frame = Frame::from_eyes(world.mesh.eyes);
        let mut starts: Vec<(String, Vec<V3>, Frame)> = Vec::new();
        if names.is_empty() {
            starts.push(("BASE".into(), world.mesh.base.clone(), base_frame));
        }
        for h in world.heads.iter().filter(|h| names.contains(&h.name)) {
            starts.push((h.name.replace('_', " ").to_uppercase(), h.start.clone(), h.frame));
        }
        let n = set.sliders.len();
        let jobs: Vec<(usize, usize, usize)> = (0..starts.len()).flat_map(|h| (0..n).flat_map(move |k| (0..3).map(move |e| (h, k, e)))).collect();
        // EYES_ENDS_SHOTS=whole draws both eyes from the front, the side and three-quarter, all open, for the whole-eye sliders
        let whole = std::env::var("EYES_ENDS_SHOTS").is_ok_and(|v| v == "whole");
        let above = std::env::var("EYES_ENDS_SHOTS").is_ok_and(|v| v == "above");
        let shots = if whole { [SHOTS[0], SHOTS[3], SHOTS[2]] } else if above { [SHOTS[1], ABOVE, SHOTS[2]] } else { [SHOTS[1], SHOTS[1], SHOTS[2]] };
        let tiles: Vec<Vec<RgbImage>> = parallel(jobs.len(), threads(), |j| {
            let (h, k, e) = jobs[j];
            let (name, start, frame) = &starts[h];
            let s = &set.sliders[k];
            let value = [s.low, 0.0, s.high][e];
            let mut values = vec![0.0; n];
            values[k] = value;
            let built = set.build(&world.smoother, start, &values);
            caruncle_from(set, &values);
            let mut out = Vec::new();
            for (t, shot) in shots.iter().enumerate() {
                let shown = cover_of(set, &values);
                let settings = Settings { lower: shown[0], ..Settings::default() };
                let (scene, _) = our_scene(rig, &built, frame, if t == 1 && !whole && !above { 1.0 } else { 0.0 }, &settings);
                let mut img = scene.draw(shot);
                if t == 0 {
                    let m = measure(&shown_surface(&built, &world.mesh.triangles, frame, shown));
                    dots(&mut img, shot, &dots_of(&m));
                    label(&mut img, &format!("{} {} {:+.1}", name, s.name.to_uppercase(), value));
                }
                out.push(img);
            }
            out
        });
        grid(&tiles.into_iter().flatten().collect::<Vec<_>>(), 9)
    }

    /// Every pair of a slider named in `a` with a slider named in `b` at the four corners of their ranges, on the base head: one eye
    /// from the front, open, and with `closed` the lid piece shut. Pairs are drawn once.
    pub fn pairs_sheet(world: &World, rig: &LidRig, set: &crate::set::SliderSet, a: &[String], b: &[String], shot: Shot, closed: f64) -> RgbImage {
        let n = set.sliders.len();
        let idx = |names: &[String]| -> Vec<usize> { names.iter().filter_map(|nm| set.sliders.iter().position(|s| &s.name == nm)).collect() };
        let (ia, ib) = (idx(a), idx(b));
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        for &x in &ia {
            for &y in &ib {
                if x != y && !pairs.contains(&(y, x)) && !pairs.contains(&(x, y)) {
                    pairs.push((x, y));
                }
            }
        }
        let jobs: Vec<(usize, usize, usize)> = pairs.iter().flat_map(|&(x, y)| (0..4).map(move |c| (x, y, c))).collect();
        let base_frame = Frame::from_eyes(world.mesh.eyes);
        let tiles = parallel(jobs.len(), threads(), |j| {
            let (x, y, c) = jobs[j];
            let mut values = vec![0.0; n];
            values[x] = if c & 1 == 0 { set.sliders[x].low } else { set.sliders[x].high };
            values[y] = if c & 2 == 0 { set.sliders[y].low } else { set.sliders[y].high };
            let built = set.build(&world.smoother, &world.mesh.base, &values);
            caruncle_from(set, &values);
            let (scene, _) = our_scene(rig, &built, &base_frame, closed, &Settings::default());
            let mut img = scene.draw(&shot);
            label(&mut img, &format!("{} {:+.1} {} {:+.1}", set.sliders[x].name.to_uppercase(), values[x], set.sliders[y].name.to_uppercase(), values[y]));
            img
        });
        grid(&tiles, 8)
    }

    /// For the base head and each head named (its stand-in): KOTOR's eye, the default eye and the eye with `values`, open, and the
    /// default and proposed eyes closed, all from the front.
    pub fn default_sheet(world: &World, rig: &LidRig, set: &crate::set::SliderSet, values: &[f64], names: &[String]) -> RgbImage {
        let shot = SHOTS[1];
        let zero = vec![0.0; values.len()];
        let mut jobs: Vec<Option<&crate::heads::Head>> = vec![None];
        jobs.extend(world.heads.iter().filter(|h| names.contains(&h.name)).map(Some));
        let tiles: Vec<Vec<RgbImage>> = parallel(jobs.len(), threads(), |i| {
            let (start, frame, title) = match jobs[i] {
                None => (world.mesh.base.clone(), Frame::from_eyes(world.mesh.eyes), "BASE".to_string()),
                Some(h) => (h.start.clone(), h.frame, h.name.replace('_', " ").to_uppercase()),
            };
            let mut row = Vec::new();
            match jobs[i] {
                Some(h) => {
                    let mut k = kotor_scene(&h.name, &h.kotor_v, &h.kotor_f, &h.frame).draw(&shot);
                    label(&mut k, &format!("{title} KOTOR"));
                    row.push(k);
                }
                None => row.push(RgbImage::from_pixel(1, 1, image::Rgb([40, 44, 52]))),
            }
            for (v, closed, tag) in [(&zero, 0.0, "DEFAULT"), (&values.to_vec(), 0.0, "PROPOSED"), (&zero, 1.0, "DEFAULT CLOSED"), (&values.to_vec(), 1.0, "PROPOSED CLOSED")] {
                let built = set.build(&world.smoother, &start, v);
                let (scene, _) = our_scene(rig, &built, &frame, closed, &Settings::default());
                let mut img = scene.draw(&shot);
                label(&mut img, tag);
                row.push(img);
            }
            let w = row[1].width();
            if row[0].width() != w {
                row[0] = RgbImage::from_pixel(w, row[1].height(), image::Rgb([40, 44, 52]));
            }
            row
        });
        grid(&tiles.into_iter().flatten().collect::<Vec<_>>(), 5)
    }

    /// KOTOR's eyes next to the eyes the sliders made for the same head (the solved values in the page's data): both eyes, one eye
    /// and three-quarter for each, and ours closed as well.
    pub fn made_eyes_sheet(world: &World, rig: &LidRig, set: &crate::set::SliderSet, data: &Path, names: &[String], mode: &str) -> RgbImage {
        let page: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/page.json")).unwrap()).unwrap();
        let n = set.sliders.len();
        let labels = std::env::var("SLIDERS_LABELS").map(|v| v != "0").unwrap_or(true);
        // EYES_MADE_SHOTS=side draws both eyes from the front, one eye from the side and three-quarter
        // EYES_MADE_SHOTS=all draws both eyes from the front, one eye from the side, three-quarter and from above
        let shots: Vec<Shot> = match std::env::var("EYES_MADE_SHOTS").as_deref() {
            Ok("side") => vec![SHOTS[0], SHOTS[3], SHOTS[2]],
            Ok("all") => vec![SHOTS[0], SHOTS[3], SHOTS[2], ABOVE],
            _ => vec![SHOTS[0], SHOTS[1], SHOTS[2]],
        };
        let found: Vec<usize> = names.iter().filter_map(|nm| world.heads.iter().position(|h| &h.name == nm)).collect();
        let rows: Vec<Vec<RgbImage>> = parallel(found.len(), threads(), |j| {
            let i = found[j];
            let h = &world.heads[i];
            // the page's values follow its own list of sliders (the solved ones, in the bench's order), at its largest count; they are
            // matched to this set's sliders by name, and the others stay at 0
            let mut values = vec![0.0; n];
            let page_names: Vec<&str> = page["sliders"].as_array().map(|a| a.iter().filter_map(|s| s["name"].as_str()).collect()).unwrap_or_default();
            if let Some(v) = page["heads"][i]["solved"][mode]["counts"][page_names.len().to_string()]["values"].as_array() {
                for (name, x) in page_names.iter().zip(v) {
                    if let Some(k) = set.sliders.iter().position(|s| s.name == *name) {
                        values[k] = x.as_f64().unwrap_or(0.0);
                    }
                }
            }
            let mut row = Vec::new();
            let k = kotor_scene(&h.name, &h.kotor_v, &h.kotor_f, &h.frame);
            for (t, shot) in shots.iter().enumerate() {
                let mut img = k.draw(shot);
                if t == 0 {
                    label(&mut img, &if labels { format!("{} KOTOR", h.name.replace('_', " ").to_uppercase()) } else { format!("{} A", j + 1) });
                }
                row.push(img);
            }
            row.push(RgbImage::from_pixel(1, 1, image::Rgb([15, 15, 18])));
            let built = set.build(&world.smoother, &h.start, &values);
            caruncle_from(set, &values);
            for (t, shot) in shots.iter().chain([&SHOTS[1]]).enumerate() {
                let (scene, _) = our_scene(rig, &built, &h.frame, if t == shots.len() { 1.0 } else { 0.0 }, &Settings::default());
                let mut img = scene.draw(shot);
                if t == 0 {
                    label(&mut img, &if labels { "OURS".to_string() } else { format!("{} B", j + 1) });
                }
                row.push(img);
            }
            row
        });
        let tiles: Vec<RgbImage> = rows.into_iter().flatten().collect();
        grid(&tiles, shots.len() + 1)
    }

    /// How many eyes pass the lid piece's checks (`EyeCheck::passes`) on a list of heads, with the worst numbers.
    fn lid_counts(rig: &LidRig, heads: &[Vec<V3>]) -> serde_json::Value {
        let checks: Vec<Vec<head_eyes::piece_check::EyeCheck>> =
            parallel(heads.len(), threads(), |i| head_eyes::piece_check::build_and_check(rig, &heads[i], &Settings::default()).1);
        let all: Vec<&head_eyes::piece_check::EyeCheck> = checks.iter().flatten().collect();
        let failing: Vec<usize> = (0..heads.len()).filter(|&i| checks[i].iter().any(|c| !c.passes())).collect();
        let worst = |f: &dyn Fn(&head_eyes::piece_check::EyeCheck) -> f64| all.iter().map(|c| f(c)).fold(f64::MIN, f64::max);
        json!({
            "eyes": all.len(),
            "pass": all.iter().filter(|c| c.passes()).count(),
            "fail_into_eyeball": all.iter().filter(|c| c.into_eyeball < 0.0).count(),
            "fail_edge_gap": all.iter().filter(|c| c.edge_gap > 0.5).count(),
            "fail_shown_closed": all.iter().filter(|c| c.shown_closed_front > 0.5 || c.shown_closed_three > 0.5).count(),
            "fail_poke": all.iter().filter(|c| c.poke_depth > 1.5).count(),
            "failing_heads": failing,
            "worst_into_eyeball": all.iter().map(|c| c.into_eyeball).fold(f64::MAX, f64::min),
            "worst_edge_gap": worst(&|c| c.edge_gap),
            "worst_poke": worst(&|c| c.poke_depth),
            "worst_shown_closed": worst(&|c| c.shown_closed_front.max(c.shown_closed_three)),
        })
    }

    /// The lid piece's checks on the base head and the 57 stand-ins: as they are, at each slider's two ends, at the four corners of
    /// every pair of sliders (base head), and on heads sampled within the ranges.
    pub fn lid_safety(world: &World, rig: &LidRig, set: &crate::set::SliderSet) -> serde_json::Value {
        let n = set.sliders.len();
        let mut starts: Vec<Vec<V3>> = vec![world.mesh.base.clone()];
        starts.extend(world.heads.iter().map(|h| h.start.clone()));
        let names: Vec<String> = std::iter::once("base".to_string()).chain(world.heads.iter().map(|h| h.name.clone())).collect();
        let build = |start: &[V3], values: &[f64]| set.build(&world.smoother, start, values);
        let zero = vec![0.0; n];
        let as_is: Vec<Vec<V3>> = starts.iter().map(|s| build(s, &zero)).collect();
        let mut out = json!({"heads": names, "as_is": lid_counts(rig, &as_is)});
        eprintln!("as they are: {}", out["as_is"]);
        let mut ends = Vec::new();
        for (k, s) in set.sliders.iter().enumerate() {
            for (end, v) in [("low", s.low), ("high", s.high)] {
                let mut values = zero.clone();
                values[k] = v;
                let heads: Vec<Vec<V3>> = starts.iter().map(|st| build(st, &values)).collect();
                let c = lid_counts(rig, &heads);
                eprintln!(
                    "{} {end} {v:.2}: {} of {} eyes pass (into eyeball {}, edge gap {}, shown closed {}, poke {})",
                    s.name, c["pass"], c["eyes"], c["fail_into_eyeball"], c["fail_edge_gap"], c["fail_shown_closed"], c["fail_poke"]
                );
                ends.push(json!({"slider": s.name, "end": end, "value": v, "checks": c}));
            }
        }
        out["ends"] = json!(ends);
        let mut pairs = Vec::new();
        let mut corners = Vec::new();
        for a in 0..n {
            for b in a + 1..n {
                for (ea, eb) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                    let mut values = zero.clone();
                    values[a] = if ea == 0 { set.sliders[a].low } else { set.sliders[a].high };
                    values[b] = if eb == 0 { set.sliders[b].low } else { set.sliders[b].high };
                    corners.push(build(&world.mesh.base, &values));
                    pairs.push(json!({"a": set.sliders[a].name, "b": set.sliders[b].name, "values": [values[a], values[b]]}));
                }
            }
        }
        let c = lid_counts(rig, &corners);
        eprintln!("pairs on the base head: {} of {} eyes pass, failing corners {}", c["pass"], c["eyes"], c["failing_heads"]);
        let failing: Vec<serde_json::Value> = c["failing_heads"].as_array().unwrap().iter().map(|i| pairs[i.as_u64().unwrap() as usize].clone()).collect();
        out["pairs"] = json!({"checks": c, "failing": failing});
        // 64 heads: each slider drawn evenly within its range, on stand-ins taken in turn
        let mut seed: u64 = 20261120;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let samples: Vec<Vec<V3>> = (0..64)
            .map(|i| {
                let values: Vec<f64> = set.sliders.iter().map(|s| s.low + (s.high - s.low) * next()).collect();
                build(&starts[1 + i % (starts.len() - 1)], &values)
            })
            .collect();
        let c = lid_counts(rig, &samples);
        eprintln!("64 sampled heads: {} of {} eyes pass", c["pass"], c["eyes"]);
        out["samples"] = c;
        out
    }

    /// Trims a keyed slider to `lo`..`hi`: keys beyond are dropped, and a key is added at each cap with the blend the slider already
    /// gives there, so the slider is unchanged inside the new range.
    pub fn cap_slider(s: &mut crate::set::Slider, field_count: usize, lo: f64, hi: f64) {
        let key_at = |s: &crate::set::Slider, v: f64| {
            let mut w = vec![0.0; field_count];
            s.add_key_weights(v, &mut w);
            let (fields, weights): (Vec<u32>, Vec<f32>) = (0..field_count).filter(|&f| w[f] != 0.0).map(|f| (f as u32, w[f] as f32)).unzip();
            crate::set::Key { value: v, fields, weights }
        };
        let (lo, hi) = (lo.max(s.low), hi.min(s.high));
        let (klo, khi) = (key_at(s, lo), key_at(s, hi));
        s.keys.retain(|k| k.value > lo + 1e-9 && k.value < hi - 1e-9);
        s.keys.insert(0, klo);
        s.keys.push(khi);
        s.low = lo;
        s.high = hi;
    }

    /// Where the default eye sits among KOTOR's 114 eyes (the left eyes and the mirrored right eyes pooled): the base head, and each
    /// stand-in in its own KOTOR head's frame. KOTOR is read twice: with its lid pieces (the eyeball a player sees) and on the skin alone.
    pub fn default_eye(world: &World, input: &crate::mesh::input::Input) {
        let np = PROPORTIONS.len();
        let mut kotor: Vec<Vec<f64>> = vec![Vec::new(); np];
        let mut skin: Vec<Vec<f64>> = vec![Vec::new(); np];
        let mut ours: Vec<Vec<f64>> = vec![Vec::new(); np];
        let mut need: Vec<Vec<f64>> = vec![Vec::new(); np];
        let mut fits: Vec<Vec<f64>> = vec![Vec::new(); np];
        for h in &world.heads {
            let Some(eyes) = kotor::all().get(&h.name) else { continue };
            if eyes.len() != 2 || h.kotor_error.is_some() {
                continue;
            }
            let per = |m: &Measured| -> Vec<Vec<f64>> { serde_json::from_value(m.marks["per_eye"].clone()).unwrap_or_default() };
            let mut balls = [0, 1].map(|k| Ball {
                centre: h.frame.local(eyes[k].centre),
                radius: eyes[k].radius * h.frame.scale,
                size: front_match(h.frame.local(eyes[k].centre), eyes[k].radius * h.frame.scale, ours_near(our_eyes_of(&fit_of(input, &h.name), &h.frame), h.frame.local(eyes[k].centre))) / h.frame.scale * 1000.0,
                tris: eyes[k].ball.iter().map(|t| t.map(|p| h.frame.local(p))).collect(),
            });
            if balls[0].centre[0] < balls[1].centre[0] {
                balls.swap(0, 1);
            }
            let surface = Surface::within(&h.kotor_v, &h.kotor_f, &h.frame, &WINDOW);
            let k_lid = per(&h.kotor);
            let k_skin = measure_with(&surface, &balls, &[]).map(|m| per(&m)).unwrap_or_default();
            let o = h.start_nose.as_ref().map(per).unwrap_or_default();
            let fit = input.points(&format!("heads/{}/fit_smooth", h.name));
            let f = measure(&Surface::within(&fit, &world.mesh.triangles, &h.frame, &WINDOW)).map(|m| per(&m)).unwrap_or_default();
            for i in 0..np {
                for e in 0..2 {
                    if let Some(v) = f.get(e).map(|r| r[i]) {
                        fits[i].push(v);
                    }
                }
            }
            for i in 0..np {
                for e in 0..2 {
                    if let Some(v) = k_lid.get(e).map(|r| r[i]) {
                        kotor[i].push(v);
                        if let Some(w) = o.get(e).map(|r| r[i]) {
                            need[i].push(v - w);
                        }
                    }
                    if let Some(v) = k_skin.get(e).map(|r| r[i]) {
                        skin[i].push(v);
                    }
                    if let Some(v) = o.get(e).map(|r| r[i]) {
                        ours[i].push(v);
                    }
                }
            }
        }
        let base: Vec<Vec<f64>> = serde_json::from_value(world.base_nose.marks["per_eye"].clone()).unwrap();
        let q = |v: &[f64], p: f64| -> f64 {
            let mut s: Vec<f64> = v.iter().copied().filter(|x| !x.is_nan()).collect();
            s.sort_by(f64::total_cmp);
            if s.is_empty() { f64::NAN } else { s[((s.len() - 1) as f64 * p).round() as usize] }
        };
        let rank = |v: &[f64], x: f64| -> f64 {
            let s: Vec<f64> = v.iter().copied().filter(|y| !y.is_nan()).collect();
            100.0 * s.iter().filter(|&&y| y < x).count() as f64 / s.len() as f64
        };
        println!("{:12} {:>7} {:>6} | KOTOR seen 5/25/50/75/95        | KOTOR skin 50 | stand-ins 50 | fits 50 | need 25/50/75", "measure", "base", "rank");
        for i in 0..np {
            let b = 0.5 * (base[0][i] + base[1][i]);
            println!(
                "{:12} {:7.2} {:5.0}% | {:6.2} {:6.2} {:6.2} {:6.2} {:6.2} | {:6.2} | {:6.2} | {:6.2} | {:6.2} {:6.2} {:6.2}  ({} eyes)",
                PROPORTIONS[i].key,
                b,
                rank(&kotor[i], b),
                q(&kotor[i], 0.05),
                q(&kotor[i], 0.25),
                q(&kotor[i], 0.5),
                q(&kotor[i], 0.75),
                q(&kotor[i], 0.95),
                q(&skin[i], 0.5),
                q(&ours[i], 0.5),
                q(&fits[i], 0.5),
                q(&need[i], 0.25),
                q(&need[i], 0.5),
                q(&need[i], 0.75),
                kotor[i].iter().filter(|x| !x.is_nan()).count()
            );
        }
    }
}
