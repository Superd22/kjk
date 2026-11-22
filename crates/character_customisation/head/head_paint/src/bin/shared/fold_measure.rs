//! The measure of one fold on one side of a face. The fold tools include it with `#[path]`, after `fold_common.rs`.
//!
//! The measure looks across a search path that `folds::PATHS` places from the head's landmarks. Every `STATION_MM` along that
//! path, it reads the skin's lightness (L*) on a line across the path, from `REACH_MM` inside to `REACH_MM` outside, as the head's surface
//! carries it. The lines are first averaged along the path over 2 mm, so that short dark strokes such as stubble or pores fade while a
//! crease that runs along the path stays.
//!
//! - **Darkness.** At each point of a line, each side's skin is the lighter of two bands, 1.2 to 3 mm and 2.5 to 6 mm away, so that
//!   a broad dark band with one sharp edge counts. The contrast is the darker side's skin minus the mean lightness within 0.3 mm. It is positive only where the point is darker than the skin on both sides, so the edge of
//!   a beard or of the lips, which is dark on one side only, does not count.
//! - **The lips.** Samples on the lips are left out: inside an ellipse round the mouth's middle that reaches 1.5 mm past each corner
//!   and 9 mm up and down, and within 4 mm of either corner, where KOTOR often paints a small shadow of its own.
//! - **The lids.** For the under-eye crease, samples above a line 3 mm under the lower lid's edge are left out.
//! - **The fold's path.** A path through the lines that keeps the most contrast is found by dynamic programming. It may move at most
//!   0.1 mm across per 0.25 mm along, and it pays 0.3 L* for each 0.1 mm it moves, so it follows a long crease and not a stroke.
//! - **Presence and length.** The contrast along the found path is averaged over 2 mm. The fold is the longest stretch where it is above
//!   `PRESENT_LSTAR`, with gaps of up to 2 mm bridged. It is present if that stretch is at least `MIN_LENGTH_MM` long.
//! - **Depth, cross-section and sharpness.** The depth is the median contrast over the stretch, and the edge the median contrast against
//!   the lighter side. The cross-section is the mean of the
//!   lines over the stretch, each centred on the found path, minus the mean lightness 5 to 7 mm from the centre. The sharpness is read
//!   from it as the distance on each side at which the darkening falls to half.

#![allow(dead_code)]

use crate::fold_common::{mean, median};
use head_paint::folds::{FoldHead, FoldKind, Mark, PATHS, SurfacePath};
use serde_json::{Value, json};

pub const STATION_MM: f64 = 0.25;
pub const STEP_MM: f64 = 0.1;
pub const REACH_MM: f64 = 13.0;
pub const SEARCH_MM: f64 = 7.0;
pub const PRESENT_LSTAR: f64 = 1.5;
pub const MIN_LENGTH_MM: f64 = 5.0;
const ALONG_AVERAGE_MM: f64 = 1.0;
const FLANKS: [[f64; 2]; 2] = [[1.2, 3.0], [2.5, 6.0]];
const CORE: f64 = 0.3;
const MOVE_COST: f64 = 0.3;
const PROFILE_MM: f64 = 7.0;
const BASELINE: [f64; 2] = [5.0, 7.0];

/// A sample point of the measure: where it lies on the head and in the layout.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub uv: [f64; 2],
    pub point: [f64; 3],
    /// The layout triangle the point lies on, and its weights there.
    pub triangle: usize,
    pub barycentric: [f64; 3],
}

/// Whether a point of the head lies on the lips or at a mouth corner.
pub fn on_lips(head: &FoldHead, p: [f64; 3]) -> bool {
    use head_paint::layout_points::{dot, length, sub};
    let l = &head.landmarks;
    let [a, b] = l.mouth_corner;
    let middle = [0, 1, 2].map(|k| 0.5 * (a[k] + b[k]));
    let half = 0.5 * length(sub(b, a));
    let r = sub(p, middle);
    let (x, y) = (dot(r, l.frame.across) / (half + 1.5), dot(r, l.frame.up) / 9.0);
    x * x + y * y < 1.0 || length(sub(p, a)) < 4.0 || length(sub(p, b)) < 4.0
}

pub const LID_GAP_MM: f64 = 3.0;

/// Whether a point of the head lies above the line `LID_GAP_MM` under the lower lid's edge (the line through the eye's inner corner, the
/// lower lid and the outer corner), between the eye's corners and up to 3 mm beyond them. The under-eye crease is not looked for there,
/// because the lid's lashes and liner would count as creases.
pub fn above_lid(head: &FoldHead, side: usize, p: [f64; 3]) -> bool {
    use head_paint::layout_points::dot;
    let l = &head.landmarks;
    let f = &l.frame;
    let plane = |q: [f64; 3]| [dot(q, f.across), dot(q, f.up)];
    let mut pts = [plane(l.eye_inner[side]), plane(l.lower_lid[side]), plane(l.eye_outer[side])];
    pts.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let q = plane(p);
    if q[0] < pts[0][0] - 3.0 || q[0] > pts[2][0] + 3.0 {
        return false;
    }
    let x = q[0].clamp(pts[0][0], pts[2][0]);
    let i = if x <= pts[1][0] { 0 } else { 1 };
    let t = (x - pts[i][0]) / (pts[i + 1][0] - pts[i][0]).max(1e-9);
    let lid = pts[i][1] + t * (pts[i + 1][1] - pts[i][1]);
    q[1] > lid - LID_GAP_MM
}

/// The lines across the search path of one fold on one side of one head.
pub struct Frame {
    pub kind: FoldKind,
    pub side: usize,
    pub path: SurfacePath,
    pub stations: usize,
    pub offsets: Vec<f64>,
    pub samples: Vec<Option<Sample>>,
}

impl Frame {
    pub fn new(head: &FoldHead, uv: &[[f64; 2]], tris: &[[u32; 3]], kind: FoldKind, side: usize) -> Self {
        let path = head.lay(kind, &PATHS[kind as usize], side);
        let stations = (path.total_mm() / STATION_MM).floor() as usize + 1;
        let n = (REACH_MM / STEP_MM).round() as i64;
        let offsets: Vec<f64> = (-n..=n).map(|i| i as f64 * STEP_MM).collect();
        let mut samples = Vec::with_capacity(stations * offsets.len());
        for s in 0..stations {
            let (p, _, across) = path.at(s as f64 * STATION_MM);
            for &u in &offsets {
                let q = [p[0] + u * across[0], p[1] + u * across[1], p[2] + u * across[2]];
                samples.push(head.index.closest(q).filter(|h| h.distance < 3.0 && !on_lips(head, h.point) && !(kind == FoldKind::UnderEye && above_lid(head, side, h.point))).map(|h| Sample { uv: head.index.uv_of(h, uv, tris), point: h.point, triangle: h.triangle, barycentric: h.barycentric }));
            }
        }
        Frame { kind, side, path, stations, offsets, samples }
    }

    pub fn width(&self) -> usize {
        self.offsets.len()
    }

    pub fn sample(&self, station: usize, offset: usize) -> Option<Sample> {
        self.samples[station * self.width() + offset]
    }
}

/// What the measure finds of one fold on one side.
#[derive(Clone, Debug, Default)]
pub struct Measure {
    pub present: bool,
    /// The median contrast over the fold, in L*.
    pub depth: f64,
    /// The median contrast against the lighter side, in L*. A fold with one sharp edge and one soft side has an edge well above its depth.
    pub edge: f64,
    pub length_mm: f64,
    /// The length of the stretch's core, where the contrast along the fold is at least half its depth, in millimetres along the search
    /// path. Faint tails count in the length but not in the core.
    pub core_mm: f64,
    /// The stations of the fold's stretch, first and last.
    pub run: Option<(usize, usize)>,
    /// The found path's offset from the search path at every station, in millimetres (positive is outward, or down for the chin).
    pub track: Vec<f64>,
    /// The contrast along the found path at every station, averaged over 2 mm.
    pub contrast: Vec<f64>,
    /// The mean cross-section over the fold, from -7 to 7 mm in 0.1 mm steps, in L* against the skin 5 to 7 mm away.
    pub profile: Vec<f64>,
    /// The distance from the centre at which the darkening falls to half, inward (toward the face's middle, or up) and outward.
    pub half_width_in: f64,
    pub half_width_out: f64,
    /// The fold's ends on the head, and their offsets from the landmarks of the search path's first and last control points, in the face's
    /// outward and up directions, in millimetres.
    pub start: [f64; 3],
    pub end: [f64; 3],
    pub start_from_mark: [f64; 2],
    pub end_from_mark: [f64; 2],
}

fn smooth_along(lines: &[Vec<f64>], half: usize) -> Vec<Vec<f64>> {
    let n = lines.len();
    let w = lines.first().map_or(0, Vec::len);
    let mut out = vec![vec![f64::NAN; w]; n];
    for s in 0..n {
        for u in 0..w {
            let (mut sum, mut count, mut total) = (0.0, 0, 0);
            for t in s.saturating_sub(half)..(s + half + 1).min(n) {
                total += 1;
                let v = lines[t][u];
                if v.is_finite() {
                    sum += v;
                    count += 1;
                }
            }
            if 2 * count >= total {
                out[s][u] = sum / count as f64;
            }
        }
    }
    out
}

/// The mean of a line's finite values whose offsets lie in a band, with the share of the band that is finite.
fn band_mean(line: &[f64], offsets: &[f64], centre: f64, lo: f64, hi: f64) -> (f64, f64) {
    let (mut sum, mut count, mut total) = (0.0, 0, 0);
    for (v, &o) in line.iter().zip(offsets) {
        let d = (o - centre).abs();
        if d >= lo && d <= hi {
            total += 1;
            if v.is_finite() {
                sum += v;
                count += 1;
            }
        }
    }
    if count == 0 { (f64::NAN, 0.0) } else { (sum / count as f64, count as f64 / total.max(1) as f64) }
}

impl Frame {
    /// Measures the fold. `lightness` gives L* at a sample, or `None` where the picture holds no skin.
    pub fn measure(&self, head: &FoldHead, lightness: &dyn Fn(&Sample) -> Option<f64>) -> Measure {
        let w = self.width();
        let raw: Vec<Vec<f64>> = (0..self.stations).map(|s| (0..w).map(|u| self.sample(s, u).and_then(|x| lightness(&x)).unwrap_or(f64::NAN)).collect()).collect();
        self.measure_lines(head, &raw)
    }

    pub fn measure_lines(&self, head: &FoldHead, raw: &[Vec<f64>]) -> Measure {
        let w = self.width();
        let half = (ALONG_AVERAGE_MM / STATION_MM).round() as usize;
        let lines = smooth_along(raw, half);
        let search: Vec<usize> = (0..w).filter(|&u| self.offsets[u].abs() <= SEARCH_MM + 1e-9).collect();
        // The contrast at every station and searched offset.
        // At each searched offset: the contrast against the darker side, and against the lighter side.
        let both: Vec<Vec<(f64, f64)>> = lines
            .iter()
            .map(|line| {
                search
                    .iter()
                    .map(|&u| {
                        let o = self.offsets[u];
                        let (core, cs) = band_mean(line, &self.offsets, o, 0.0, CORE);
                        // Each side's skin is the lighter of its near and far bands, so a broad dark band with one sharp edge counts.
                        let side = |part: &[f64], offs: &[f64]| -> f64 {
                            FLANKS.iter().map(|f| band_mean(part, offs, o, f[0], f[1])).filter(|(_, share)| *share >= 0.5).map(|(v, _)| v).fold(f64::NAN, f64::max)
                        };
                        let (lo, hi) = (side(&line[..u], &self.offsets[..u]), side(&line[u + 1..], &self.offsets[u + 1..]));
                        if cs < 0.5 || !lo.is_finite() || !hi.is_finite() { (f64::NAN, f64::NAN) } else { (lo.min(hi) - core, lo.max(hi) - core) }
                    })
                    .collect()
            })
            .collect();
        let contrast: Vec<Vec<f64>> = both.iter().map(|r| r.iter().map(|x| x.0).collect()).collect();
        // The path with the most contrast.
        let m = search.len();
        let score = |c: f64| if c.is_finite() { c } else { -0.5 };
        let mut total = vec![vec![f64::NEG_INFINITY; m]; self.stations];
        let mut from = vec![vec![0usize; m]; self.stations];
        for j in 0..m {
            total[0][j] = score(contrast[0][j]);
        }
        for s in 1..self.stations {
            for j in 0..m {
                let mut best = (f64::NEG_INFINITY, j);
                for k in j.saturating_sub(1)..(j + 2).min(m) {
                    let v = total[s - 1][k] - if k == j { 0.0 } else { MOVE_COST };
                    if v > best.0 {
                        best = (v, k);
                    }
                }
                total[s][j] = best.0 + score(contrast[s][j]);
                from[s][j] = best.1;
            }
        }
        let mut j = (0..m).max_by(|&a, &b| total[self.stations - 1][a].total_cmp(&total[self.stations - 1][b])).unwrap_or(0);
        let mut track_idx = vec![0usize; self.stations];
        for s in (0..self.stations).rev() {
            track_idx[s] = j;
            j = from[s][j];
        }
        let along: Vec<f64> = track_idx.iter().enumerate().map(|(s, &j)| contrast[s][j]).collect();
        let smooth: Vec<f64> = (0..self.stations)
            .map(|s| {
                let v: Vec<f64> = (s.saturating_sub(4)..(s + 5).min(self.stations)).map(|t| along[t]).filter(|v| v.is_finite()).collect();
                if v.is_empty() { f64::NAN } else { mean(&v) }
            })
            .collect();
        // The longest stretch above the threshold, with gaps of up to 2 mm bridged.
        let gap = (2.0 / STATION_MM) as usize;
        let mut best: Option<(usize, usize)> = None;
        let mut current: Option<(usize, usize)> = None;
        let mut since = 0usize;
        for (s, v) in smooth.iter().enumerate() {
            if v.is_finite() && *v > PRESENT_LSTAR {
                current = Some(match current {
                    Some((a, _)) if since <= gap => (a, s),
                    _ => (s, s),
                });
                since = 0;
                if let Some((a, b)) = current
                    && best.is_none_or(|(x, y)| b - a > y - x)
                {
                    best = Some((a, b));
                }
            } else {
                since += 1;
            }
        }
        let track: Vec<f64> = track_idx.iter().map(|&j| self.offsets[search[j]]).collect();
        let point_at = |s: usize| -> [f64; 3] {
            let (p, _, a) = self.path.at(s as f64 * STATION_MM);
            let u = track[s];
            let q = [p[0] + u * a[0], p[1] + u * a[1], p[2] + u * a[2]];
            head.index.closest(q).map_or(q, |h| h.point)
        };
        let mut result = Measure { track: track.clone(), contrast: smooth.clone(), ..Measure::default() };
        let Some((a, b)) = best else { return result };
        let mut length = 0.0;
        let mut prev = point_at(a);
        for s in a + 1..=b {
            let p = point_at(s);
            length += head_paint::layout_points::length(head_paint::layout_points::sub(p, prev));
            prev = p;
        }
        result.run = Some((a, b));
        result.length_mm = length;
        result.present = length >= MIN_LENGTH_MM;
        result.depth = median(&(a..=b).map(|s| along[s]).filter(|v| v.is_finite()).collect::<Vec<_>>());
        result.core_mm = (a..=b).filter(|&s| smooth[s].is_finite() && smooth[s] >= 0.5 * result.depth).count() as f64 * STATION_MM;
        result.edge = median(&(a..=b).map(|s| both[s][track_idx[s]].1).filter(|v| v.is_finite()).collect::<Vec<_>>());
        // The cross-section.
        let steps = (PROFILE_MM / STEP_MM).round() as i64;
        let mut sums = vec![(0.0, 0usize); (2 * steps + 1) as usize];
        for s in a..=b {
            let centre = search[track_idx[s]] as i64;
            let line = &lines[s];
            let profile: Vec<f64> = (-steps..=steps).map(|k| line.get((centre + k) as usize).copied().unwrap_or(f64::NAN)).collect();
            let base: Vec<f64> = profile.iter().enumerate().filter(|(i, v)| v.is_finite() && { let d = ((*i as i64 - steps).abs() as f64) * STEP_MM; d >= BASELINE[0] && d <= BASELINE[1] }).map(|(_, v)| *v).collect();
            if base.len() < 10 {
                continue;
            }
            let b0 = mean(&base);
            for (k, v) in profile.iter().enumerate() {
                if v.is_finite() {
                    sums[k].0 += v - b0;
                    sums[k].1 += 1;
                }
            }
        }
        result.profile = sums.iter().map(|(s, n)| if *n > 0 { s / *n as f64 } else { f64::NAN }).collect();
        let centre = steps as usize;
        let bottom = result.profile[centre.saturating_sub(5)..=centre + 5].iter().cloned().filter(|v| v.is_finite()).fold(f64::INFINITY, f64::min);
        let half_at = |dir: i64| -> f64 {
            let mut k = centre as i64;
            while k > 0 && (k as usize) < result.profile.len() - 1 {
                let v = result.profile[k as usize];
                if v.is_finite() && v > 0.5 * bottom {
                    return (k - centre as i64).abs() as f64 * STEP_MM;
                }
                k += dir;
            }
            PROFILE_MM
        };
        result.half_width_in = half_at(-1);
        result.half_width_out = half_at(1);
        result.start = point_at(a);
        result.end = point_at(b);
        let shape = &PATHS[self.kind as usize];
        let rel = |p: [f64; 3], mark: Mark| -> [f64; 2] {
            let l = &head.landmarks;
            let base = match mark {
                Mark::NoseWing => l.nose_wing[self.side],
                Mark::MouthCorner => l.mouth_corner[self.side],
                Mark::ChinMiddle => l.chin_middle,
                Mark::EyeInner => l.eye_inner[self.side],
                Mark::LowerLid => l.lower_lid[self.side],
                Mark::EyeOuter => l.eye_outer[self.side],
                Mark::Middle => l.chin_middle,
            };
            let d = head_paint::layout_points::sub(p, base);
            [head_paint::layout_points::dot(d, l.frame.out(self.side)), head_paint::layout_points::dot(d, l.frame.up)]
        };
        result.start_from_mark = rel(result.start, shape.controls[0].from);
        result.end_from_mark = rel(result.end, shape.controls[2].from);
        result
    }
}

impl Measure {
    /// How strongly the fold reads, for sorting: the mean of its depth and its edge, times its length up to 20 mm over 20 mm. It is 0
    /// when the fold is not present.
    pub fn score(&self) -> f64 {
        if self.present { 0.5 * (self.depth + self.edge) * (self.length_mm / 20.0).min(1.0) } else { 0.0 }
    }

    pub fn to_json(&self) -> Value {
        let r = |x: f64| (x * 1000.0).round() / 1000.0;
        json!({
            "present": self.present,
            "depth_lstar": r(self.depth),
            "edge_lstar": r(self.edge),
            "score": r(self.score()),
            "length_mm": r(self.length_mm),
            "core_mm": r(self.core_mm),
            "half_width_in_mm": r(self.half_width_in),
            "half_width_out_mm": r(self.half_width_out),
            "start_from_mark_mm": self.start_from_mark.map(r),
            "end_from_mark_mm": self.end_from_mark.map(r),
            "run": self.run.map(|(a, b)| [a, b]),
            "track_mm": self.track.iter().map(|&x| r(x)).collect::<Vec<_>>(),
            "contrast": self.contrast.iter().map(|&x| if x.is_finite() { r(x) } else { 0.0 }).collect::<Vec<_>>(),
            "profile": self.profile.iter().map(|&x| if x.is_finite() { r(x) } else { 0.0 }).collect::<Vec<_>>(),
        })
    }
}

/// KOTOR's skin as the fold measure reads it on one head: the bake in our layout with its hair masked, and KOTOR's own head seen from
/// the front, which the under-eye crease reads instead of the bake.
pub struct KotorSkin {
    pub bake: crate::fold_common::Lightness,
    pub hair: Vec<bool>,
    pub front: crate::fold_common::KotorFront,
}

impl KotorSkin {
    pub fn open(name: &str, head: &FoldHead) -> Self {
        use crate::fold_common::*;
        let bake = Lightness::open(&bake_path(name));
        let hair = bake.hair();
        let f = head.landmarks.frame;
        KotorSkin { bake, hair, front: KotorFront::open(name, f.across, f.up, f.forward) }
    }

    pub fn lightness(&self, kind: FoldKind, s: &Sample) -> Option<f64> {
        use crate::fold_common::RES;
        if kind == FoldKind::UnderEye {
            return self.front.lightness(s.point);
        }
        let (x, y) = ((s.uv[0] * RES as f64) as usize, ((1.0 - s.uv[1]) * RES as f64) as usize);
        if x >= RES || y >= RES || self.hair[y * RES + x] { None } else { self.bake.at_uv(s.uv).map(f64::from) }
    }
}

/// The window of a fold's pictures: a point of the head at its centre, and its width and height in millimetres, seen from the front.
pub fn tile_window(kind: FoldKind, head: &FoldHead) -> ([f64; 3], f64, f64) {
    let l = &head.landmarks;
    let avg = |pts: &[[f64; 3]]| -> [f64; 3] { [0, 1, 2].map(|k| pts.iter().map(|p| p[k]).sum::<f64>() / pts.len() as f64) };
    let up = |p: [f64; 3], mm: f64| -> [f64; 3] { [0, 1, 2].map(|k| p[k] + mm * l.frame.up[k]) };
    match kind {
        FoldKind::Nasolabial => (avg(&[l.nose_wing[0], l.nose_wing[1], l.mouth_corner[0], l.mouth_corner[1]]), 74.0, 44.0),
        FoldKind::Marionette => (up(avg(&[l.mouth_corner[0], l.mouth_corner[1]]), -9.0), 70.0, 34.0),
        FoldKind::Chin => (up(l.chin_middle, 1.0), 50.0, 26.0),
        FoldKind::UnderEye => (avg(&[l.lower_lid[0], l.lower_lid[1], l.eye_inner[0], l.eye_inner[1]]), 90.0, 26.0),
    }
}

/// Draws dots, given as points of the face's plane, on a tile made by `KotorFront::tile` with the same centre and size.
pub fn draw_on_tile(img: &mut image::RgbImage, centre: [f64; 2], w_mm: f64, h_mm: f64, dots: &[([f64; 2], [u8; 3])]) {
    let origin = [centre[0] - 0.5 * w_mm, centre[1] + 0.5 * h_mm];
    for (q, colour) in dots {
        let (x, y) = (((q[0] - origin[0]) / 0.1) as i64, ((origin[1] - q[1]) / 0.1) as i64);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if x + dx >= 0 && y + dy >= 0 && x + dx < i64::from(img.width()) && y + dy < i64::from(img.height()) {
                    img.put_pixel((x + dx) as u32, (y + dy) as u32, image::Rgb(*colour));
                }
            }
        }
    }
}
