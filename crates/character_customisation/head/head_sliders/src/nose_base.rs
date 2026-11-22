//! The nose seen from below and from above, with the nostril holes, and the height of the nostril wings in profile (2026-11-22, the
//! brief `nose-nostrils`).
//!
//! Everything is read on height maps in the head's frame (mm): the face is sampled every 0.25 mm across (x) and out of the face (z),
//! and each sample keeps the lowest skin (seen from below) or the highest skin (seen from above) between two heights.

use crate::V3;
use crate::measure::{Frame, Surface, Window};

/// The spacing of the height maps (mm).
pub const CELL: f64 = 0.25;
/// The height maps cover |x| up to this (mm).
pub const HALF_WIDTH: f64 = 24.0;
/// The height maps cover depths from `tip - DEPTH_SPAN` to `tip + 1` (mm).
pub const DEPTH_SPAN: f64 = 34.0;

pub const WINDOW: Window = Window { x: 30.0, y: [-90.0, 10.0], z: -10.0 };

/// A height map over (x, z): `y[row * cols + col]` at x = `x0 + col * CELL`, z = `z0 + row * CELL`, NaN where no skin lies between
/// the two heights.
#[derive(Clone)]
pub struct Map {
    pub x0: f64,
    pub z0: f64,
    pub cols: usize,
    pub rows: usize,
    pub y: Vec<f64>,
}

impl Map {
    pub fn at(&self, r: usize, c: usize) -> f64 {
        self.y[r * self.cols + c]
    }
    pub fn x(&self, c: usize) -> f64 {
        self.x0 + c as f64 * CELL
    }
    pub fn z(&self, r: usize) -> f64 {
        self.z0 + r as f64 * CELL
    }
}

/// The height map of the skin between heights `lo` and `hi`: the lowest skin when `below`, else the highest.
pub fn height_map(surface: &Surface, lo: f64, hi: f64, z0: f64, z1: f64, below: bool) -> Map {
    let cols = (2.0 * HALF_WIDTH / CELL).round() as usize + 1;
    let rows = ((z1 - z0) / CELL).round() as usize + 1;
    let x0 = -HALF_WIDTH;
    let mut y = vec![f64::NAN; rows * cols];
    for t in &surface.triangles {
        if t.iter().all(|p| p[1] < lo) || t.iter().all(|p| p[1] > hi) {
            continue;
        }
        let area = (t[1][0] - t[0][0]) * (t[2][2] - t[0][2]) - (t[2][0] - t[0][0]) * (t[1][2] - t[0][2]);
        if area.abs() < 1e-12 {
            continue;
        }
        let xs = t.map(|p| p[0]);
        let zs = t.map(|p| p[2]);
        let c0 = ((xs.iter().copied().fold(f64::INFINITY, f64::min) - x0) / CELL).ceil().max(0.0) as usize;
        let c1 = (((xs.iter().copied().fold(f64::NEG_INFINITY, f64::max) - x0) / CELL).floor() as isize).min(cols as isize - 1);
        let r0 = ((zs.iter().copied().fold(f64::INFINITY, f64::min) - z0) / CELL).ceil().max(0.0) as usize;
        let r1 = (((zs.iter().copied().fold(f64::NEG_INFINITY, f64::max) - z0) / CELL).floor() as isize).min(rows as isize - 1);
        if c1 < 0 || r1 < 0 {
            continue;
        }
        for r in r0..=r1 as usize {
            let pz = z0 + r as f64 * CELL;
            for c in c0..=c1 as usize {
                let px = x0 + c as f64 * CELL;
                let b0 = ((t[1][0] - px) * (t[2][2] - pz) - (t[2][0] - px) * (t[1][2] - pz)) / area;
                let b1 = ((t[2][0] - px) * (t[0][2] - pz) - (t[0][0] - px) * (t[2][2] - pz)) / area;
                let b2 = 1.0 - b0 - b1;
                if b0 < -1e-9 || b1 < -1e-9 || b2 < -1e-9 {
                    continue;
                }
                let h = b0 * t[0][1] + b1 * t[1][1] + b2 * t[2][1];
                if h < lo || h > hi {
                    continue;
                }
                let v = &mut y[r * cols + c];
                if v.is_nan() || (below && h < *v) || (!below && h > *v) {
                    *v = h;
                }
            }
        }
    }
    Map { x0, z0, cols, rows, y }
}

/// How deep each sample lies in a pit of the map, seen from the side the map was read from: the map closed by a disc of `radius` mm
/// (the deepest a ball of that size can reach) minus the map itself. A pit narrower than the disc reads deep, a broad dish reads 0.
pub fn pits(m: &Map, radius: f64, below: bool) -> Vec<f64> {
    let s = if below { -1.0 } else { 1.0 };
    // seen from the reading side, the skin's height toward the viewer
    let h: Vec<f64> = m.y.iter().map(|&y| s * y).collect();
    let k = (radius / CELL).round() as isize;
    let disc: Vec<(isize, isize)> = (-k..=k).flat_map(|a| (-k..=k).map(move |b| (a, b))).filter(|&(a, b)| a * a + b * b <= k * k).collect();
    let (rows, cols) = (m.rows as isize, m.cols as isize);
    let pass = |src: &[f64], max: bool| -> Vec<f64> {
        let mut out = vec![f64::NAN; src.len()];
        for r in 0..rows {
            for c in 0..cols {
                if src[(r * cols + c) as usize].is_nan() {
                    continue;
                }
                let mut best = f64::NAN;
                for &(a, b) in &disc {
                    let (rr, cc) = (r + a, c + b);
                    if rr < 0 || cc < 0 || rr >= rows || cc >= cols {
                        continue;
                    }
                    let v = src[(rr * cols + cc) as usize];
                    if v.is_nan() {
                        continue;
                    }
                    if best.is_nan() || (max && v > best) || (!max && v < best) {
                        best = v;
                    }
                }
                out[(r * cols + c) as usize] = best;
            }
        }
        out
    };
    let closed = pass(&pass(&h, true), false);
    closed.iter().zip(&h).map(|(c, v)| if v.is_nan() { f64::NAN } else { (c - v).max(0.0) }).collect()
}

/// A projection of the skin along one axis of the head's frame: over a grid of the two other axes (`u` across the columns, `v` down
/// the rows, every `CELL` mm), the lowest or highest value of the third axis among the skin whose third coordinate lies in `h`.
#[derive(Clone, Debug)]
pub struct Grid {
    pub u0: f64,
    pub v0: f64,
    pub cols: usize,
    pub rows: usize,
    pub h: Vec<f64>,
}

impl Grid {
    pub fn at(&self, r: usize, c: usize) -> f64 {
        self.h[r * self.cols + c]
    }
    pub fn u(&self, c: usize) -> f64 {
        self.u0 + c as f64 * CELL
    }
    pub fn v(&self, r: usize) -> f64 {
        self.v0 + r as f64 * CELL
    }
}

/// The skin projected along axis `axes[2]` onto a grid of `axes[0]` over `u` and `axes[1]` over `v`, keeping skin whose `axes[2]`
/// coordinate lies in `h`, the lowest when `low` and else the highest.
pub fn project(surface: &Surface, axes: [usize; 3], u: [f64; 2], v: [f64; 2], h: [f64; 2], low: bool) -> Grid {
    let cols = ((u[1] - u[0]) / CELL).round() as usize + 1;
    let rows = ((v[1] - v[0]) / CELL).round() as usize + 1;
    let mut out = vec![f64::NAN; rows * cols];
    let [a, b, c] = axes;
    for t in &surface.triangles {
        if t.iter().all(|p| p[c] < h[0]) || t.iter().all(|p| p[c] > h[1]) {
            continue;
        }
        let area = (t[1][a] - t[0][a]) * (t[2][b] - t[0][b]) - (t[2][a] - t[0][a]) * (t[1][b] - t[0][b]);
        if area.abs() < 1e-12 {
            continue;
        }
        let lo = |k: usize| t.iter().map(|p| p[k]).fold(f64::INFINITY, f64::min);
        let hi = |k: usize| t.iter().map(|p| p[k]).fold(f64::NEG_INFINITY, f64::max);
        let c0 = ((lo(a) - u[0]) / CELL).ceil().max(0.0) as usize;
        let c1 = (((hi(a) - u[0]) / CELL).floor() as isize).min(cols as isize - 1);
        let r0 = ((lo(b) - v[0]) / CELL).ceil().max(0.0) as usize;
        let r1 = (((hi(b) - v[0]) / CELL).floor() as isize).min(rows as isize - 1);
        if c1 < 0 || r1 < 0 {
            continue;
        }
        for r in r0..=r1 as usize {
            let pv = v[0] + r as f64 * CELL;
            for cc in c0..=c1 as usize {
                let pu = u[0] + cc as f64 * CELL;
                let b0 = ((t[1][a] - pu) * (t[2][b] - pv) - (t[2][a] - pu) * (t[1][b] - pv)) / area;
                let b1 = ((t[2][a] - pu) * (t[0][b] - pv) - (t[0][a] - pu) * (t[2][b] - pv)) / area;
                let b2 = 1.0 - b0 - b1;
                if b0 < -1e-9 || b1 < -1e-9 || b2 < -1e-9 {
                    continue;
                }
                let x = b0 * t[0][c] + b1 * t[1][c] + b2 * t[2][c];
                if x < h[0] || x > h[1] {
                    continue;
                }
                let o = &mut out[r * cols + cc];
                if o.is_nan() || (low && x < *o) || (!low && x > *o) {
                    *o = x;
                }
            }
        }
    }
    Grid { u0: u[0], v0: v[0], cols, rows, h: out }
}

/// How deep each cell lies in a pit of the grid, seen from the side it was read from (`low` reads from the low side): the grid closed
/// by a disc of `radius` mm minus the grid. Cells outside `inside` (or NaN) take no part.
pub fn pit_depths(g: &Grid, inside: &[bool], radius: f64, low: bool) -> Vec<f64> {
    let s = if low { -1.0 } else { 1.0 };
    let h: Vec<f64> = g.h.iter().zip(inside).map(|(&y, &i)| if i { s * y } else { f64::NAN }).collect();
    let k = (radius / CELL).round() as isize;
    let disc: Vec<(isize, isize)> = (-k..=k).flat_map(|a| (-k..=k).map(move |b| (a, b))).filter(|&(a, b)| a * a + b * b <= k * k).collect();
    let (rows, cols) = (g.rows as isize, g.cols as isize);
    let pass = |src: &[f64], max: bool| -> Vec<f64> {
        let mut out = vec![f64::NAN; src.len()];
        for r in 0..rows {
            for c in 0..cols {
                if src[(r * cols + c) as usize].is_nan() {
                    continue;
                }
                let mut best = f64::NAN;
                for &(a, b) in &disc {
                    let (rr, cc) = (r + a, c + b);
                    if rr < 0 || cc < 0 || rr >= rows || cc >= cols {
                        continue;
                    }
                    let v = src[(rr * cols + cc) as usize];
                    if !v.is_nan() && (best.is_nan() || (max && v > best) || (!max && v < best)) {
                        best = v;
                    }
                }
                out[(r * cols + c) as usize] = best;
            }
        }
        out
    };
    let closed = pass(&pass(&h, true), false);
    closed.iter().zip(&h).map(|(c, v)| if v.is_nan() || c.is_nan() { 0.0 } else { (c - v).max(0.0) }).collect()
}

/// The pit depth that counts as a nostril hole seen from below, and the disc that reads pits (mm).
pub const HOLE_DEPTH: f64 = 0.6;
pub const HOLE_RADIUS: f64 = 3.0;
/// A cell is under the nose, seen from below, when its lowest skin lies this far above the cut under the nose (mm).
const OVER_CUT: f64 = 0.5;
/// The cut under the nose lies this far below its base (mm), so the lip is left out.
const UNDER_BASE: f64 = 2.0;
/// The cut over the wings, seen from above, lies this far above the tip (mm).
const OVER_TIP: f64 = 14.0;
/// The wings, seen from below, are the nose's area beyond this share of its half-width.
pub const WING_FROM: f64 = 0.55;
/// Two neighbouring cells of the nose's area seen from below differ by less than this in height (mm).
const STEP_UP: f64 = 0.75;

/// The cells of `on` connected to the cell under (0, `z`) through neighbours whose heights differ by less than `STEP_UP`.
fn grow(g: &Grid, on: &[bool], z: f64) -> Vec<bool> {
    let mut out = vec![false; on.len()];
    let r0 = (((z - g.v0) / CELL).round() as usize).min(g.rows - 1);
    let c0 = ((-g.u0 / CELL).round() as usize).min(g.cols - 1);
    // the lowest cell within 2 mm of the midline, from 2 to 8 mm behind `z`
    let start = (r0.saturating_sub(32)..=r0.saturating_sub(8))
        .flat_map(|r| (c0.saturating_sub(8)..=(c0 + 8).min(g.cols - 1)).map(move |c| r * g.cols + c))
        .filter(|&k| on[k])
        .min_by(|&a, &b| g.h[a].total_cmp(&g.h[b]));
    let Some(start) = start else { return out };
    let mut stack = vec![start];
    out[start] = true;
    while let Some(k) = stack.pop() {
        let (r, c) = (k / g.cols, k % g.cols);
        let mut near = Vec::with_capacity(4);
        if r > 0 { near.push(k - g.cols); }
        if r + 1 < g.rows { near.push(k + g.cols); }
        if c > 0 { near.push(k - 1); }
        if c + 1 < g.cols { near.push(k + 1); }
        for n in near {
            if on[n] && !out[n] && (g.h[n] - g.h[k]).abs() < STEP_UP {
                out[n] = true;
                stack.push(n);
            }
        }
    }
    out
}

/// The outlines are read every this many mm of depth.
pub const OUTLINE_STEP: f64 = 0.5;

/// The nose seen from below and from above, and its wings in profile, in the head's frame (mm).
#[derive(Clone, Debug, serde::Serialize)]
pub struct Base {
    /// The tip (y, z) and the base (y, z), from the midline profile.
    pub tip: [f64; 2],
    pub base: [f64; 2],
    /// The wings' half-width from the nose's measure.
    pub half_width: f64,
    /// Seen from the side: at each depth z, every `OUTLINE_STEP` mm from `z_from`, the lowest skin of the nose, as (z, y).
    pub lower: Vec<[f64; 2]>,
    /// Seen from the side: at each depth, the lowest skin of the wings alone (beyond `WING_FROM` of the half-width), as (z, y).
    pub wing: Vec<[f64; 2]>,
    /// Seen from below and from above: at each depth, the nose's half-width (mean of both sides), as (z, half-width).
    pub below: Vec<[f64; 2]>,
    pub above: Vec<[f64; 2]>,
    /// The nostril holes seen from below: their area (mm², both sides), their centre (|x|, z) and their front (largest z).
    pub hole_area: f64,
    pub hole_centre: [f64; 2],
    pub hole_front: f64,
    /// The holes' mean length (along z) and width (across), per side, and the share of the hole's bounding box it fills (a wedge
    /// fills about half, a round spot about four fifths).
    pub hole_len: f64,
    pub hole_wid: f64,
    pub hole_fill: f64,
    /// The wing in profile: the top of its crease and its lower edge (y), and the height between them.
    pub crease_top: f64,
    pub wing_low: f64,
    pub wing_height: f64,
    /// For drawing: the grids from below and from the side, their cells under the nose and their pits.
    #[serde(skip)]
    pub grid_below: Option<(Grid, Vec<bool>, Vec<f64>)>,
    #[serde(skip)]
    pub grid_side: Option<(Grid, Vec<bool>, Vec<f64>)>,
}

fn percentile(mut v: Vec<f64>, q: f64) -> f64 {
    v.retain(|x| !x.is_nan());
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// The half-width of the cells marked in each row of a grid over (x, z), as (z, half-width), the mean of both sides.
fn widths(g: &Grid, on: &[bool]) -> Vec<(f64, f64)> {
    (0..g.rows)
        .map(|r| {
            let xs: Vec<f64> = (0..g.cols).filter(|&c| on[r * g.cols + c]).map(|c| g.u(c)).collect();
            if xs.is_empty() {
                return (g.v(r), f64::NAN);
            }
            let (lo, hi) = (xs.iter().copied().fold(f64::INFINITY, f64::min), xs.iter().copied().fold(f64::NEG_INFINITY, f64::max));
            (g.v(r), if lo < 0.0 && hi > 0.0 { 0.5 * (hi - lo) } else { f64::NAN })
        })
        .collect()
}

/// Reads the nose from below, from above and its wings in profile, from the surface near the nose.
pub fn read(v: &[V3], f: &[[u32; 3]]) -> Result<Base, &'static str> {
    let s = surface(v, f);
    let fine = crate::nose_profile::read(&s)?;
    let (tip, base) = (fine.tip, fine.base);
    let hw = wing_half_width(v, f);
    if hw.is_nan() {
        return Err("no wings");
    }
    let snap = |z: f64| (z / OUTLINE_STEP).floor() * OUTLINE_STEP;
    let zr = [snap(tip[1] - DEPTH_SPAN), snap(tip[1] + 1.0)];
    let xr = [-HALF_WIDTH, HALF_WIDTH];
    // from below: x across, z down the rows, the lowest y between the cut under the nose and the eye line
    let cut = base[0] - UNDER_BASE;
    let gb = project(&s, [0, 2, 1], xr, zr, [cut, 0.0], true);
    let any_under: Vec<bool> = gb.h.iter().map(|&y| !y.is_nan() && y > cut + OVER_CUT).collect();
    // the nose's own area from below: grown from under the tip across neighbours whose lowest skin differs by less than STEP_UP,
    // so it stops where the wing's side rises to the cheek
    let under = grow(&gb, &any_under, tip[1]);
    // the holes are read on all the skin under the cut, then kept within each row's span of the nose's area
    let pits_b = pit_depths(&gb, &any_under, HOLE_RADIUS, true);
    let spans: Vec<f64> = widths(&gb, &under).iter().map(|w| w.1).collect();
    // from above: the highest y under the cut over the wings
    let top = tip[0] + OVER_TIP;
    let ga = project(&s, [0, 2, 1], xr, zr, [-90.0, top], false);
    let over: Vec<bool> = ga.h.iter().map(|&y| !y.is_nan() && y < top - OVER_CUT).collect();
    // the holes: pits deeper than HOLE_DEPTH, not on the midline's columella (|x| > 1 mm)
    let hole: Vec<bool> = (0..gb.h.len())
        .map(|k| any_under[k] && pits_b[k] > HOLE_DEPTH && gb.u(k % gb.cols).abs() > 1.0 && gb.u(k % gb.cols).abs() < spans[k / gb.cols])
        .collect();
    let cells: Vec<usize> = (0..hole.len()).filter(|&k| hole[k]).collect();
    let hole_area = cells.len() as f64 * CELL * CELL;
    let hole_centre = if cells.is_empty() {
        [f64::NAN, f64::NAN]
    } else {
        let n = cells.len() as f64;
        [cells.iter().map(|&k| gb.u(k % gb.cols).abs()).sum::<f64>() / n, cells.iter().map(|&k| gb.v(k / gb.cols)).sum::<f64>() / n]
    };
    let hole_front = cells.iter().map(|&k| gb.v(k / gb.cols)).fold(f64::NAN, f64::max);
    let mut shape = Vec::new();
    for sgn in [1.0, -1.0] {
        let side: Vec<usize> = cells.iter().copied().filter(|&k| gb.u(k % gb.cols) * sgn > 0.0).collect();
        if side.len() < 4 {
            continue;
        }
        let zs: Vec<f64> = side.iter().map(|&k| gb.v(k / gb.cols)).collect();
        let xs: Vec<f64> = side.iter().map(|&k| gb.u(k % gb.cols)).collect();
        let span = |v: &[f64]| v.iter().copied().fold(f64::NEG_INFINITY, f64::max) - v.iter().copied().fold(f64::INFINITY, f64::min) + CELL;
        let (l, w) = (span(&zs), span(&xs));
        shape.push([l, w, side.len() as f64 * CELL * CELL / (l * w)]);
    }
    let mean = |i: usize| if shape.is_empty() { f64::NAN } else { shape.iter().map(|q| q[i]).sum::<f64>() / shape.len() as f64 };
    let (hole_len, hole_wid, hole_fill) = (mean(0), mean(1), mean(2));
    // seen from the side: the lowest skin under the nose at each depth, over both sides within the wings
    let z_from = snap(base[1] + 1.0) + OUTLINE_STEP;
    let mut lower = Vec::new();
    let mut wing = Vec::new();
    let mut z = z_from;
    while z <= tip[1] {
        let r = ((z - gb.v0) / CELL).round() as usize;
        let ys: Vec<f64> = (0..gb.cols).filter(|&c| under[r * gb.cols + c] && gb.u(c).abs() < hw + 2.0).map(|c| gb.at(r, c)).collect();
        lower.push([z, ys.iter().copied().fold(f64::NAN, f64::min)]);
        let ws: Vec<f64> = (0..gb.cols).filter(|&c| under[r * gb.cols + c] && !hole[r * gb.cols + c] && gb.u(c).abs() > WING_FROM * hw).map(|c| gb.at(r, c)).collect();
        wing.push([z, ws.iter().copied().fold(f64::NAN, f64::min)]);
        z += OUTLINE_STEP;
    }
    let every = (OUTLINE_STEP / CELL).round() as usize;
    let pick = |w: Vec<(f64, f64)>| -> Vec<[f64; 2]> { w.into_iter().step_by(every).map(|(z, x)| [z, x]).collect() };
    let below = pick(widths(&gb, &under));
    let above = pick(widths(&ga, &over));
    // the wing's lower edge: the lowest tenth of the skin under the wings (|x| beyond 55% of the half-width) seen from below
    let wing_ys: Vec<f64> = (0..gb.h.len()).filter(|&k| under[k] && gb.u(k % gb.cols).abs() > WING_FROM * hw && !hole[k]).map(|k| gb.h[k]).collect();
    let wing_low = percentile(wing_ys, 0.1);
    // the crease in profile: seen from each side, the skin's place across (|x|) over (z, y), the cheek beyond the wings left out;
    // the crease is the pit round the wing, and its top is its highest point above the wing's lower edge and behind the tip
    let yr = [base[0] - 6.0, tip[0] + 20.0];
    let zr2 = [base[1] - 6.0, tip[1] + 1.0];
    let mut tops: Vec<f64> = Vec::new();
    let mut grid_side = None;
    for side in [1.0, -1.0] {
        let g = project(&s, [2, 1, 0], zr2, yr, if side > 0.0 { [2.0, hw + 4.0] } else { [-(hw + 4.0), -2.0] }, side < 0.0);
        let inside: Vec<bool> = g.h.iter().map(|x| !x.is_nan()).collect();
        let pits = pit_depths(&g, &inside, 2.5, side < 0.0);
        let crease: Vec<usize> = (0..g.h.len())
            .filter(|&k| pits[k] > 0.4 && g.v(k / g.cols) > wing_low && g.v(k / g.cols) < tip[0] + 15.0 && g.u(k % g.cols) < tip[1] - 3.0 && g.u(k % g.cols) > base[1] - 3.0)
            .collect();
        tops.push(crease_top_of(&g, &pits, &crease));
        if side < 0.0 {
            grid_side = Some((g, inside, pits));
        }
    }
    if std::env::var("NB_CREASE").is_ok_and(|m| m == "turn") {
        tops = turn_tops(&s, hw, wing_low, tip[0]);
    }
    let crease_top = if tops.iter().all(|t| t.is_nan()) { f64::NAN } else { tops.iter().copied().filter(|t| !t.is_nan()).sum::<f64>() / tops.iter().filter(|t| !t.is_nan()).count() as f64 };
    Ok(Base {
        tip,
        base,
        half_width: hw,
        lower,
        wing,
        below,
        above,
        hole_area,
        hole_centre,
        hole_front,
        hole_len,
        hole_wid,
        hole_fill,
        crease_top,
        wing_low,
        wing_height: crease_top - wing_low,
        grid_below: Some((gb, under, pits_b.iter().zip(&hole).map(|(p, h)| if *h { *p } else { 0.0 }).collect())),
        grid_side,
    })
}

/// The crease's top from its pit cells on the side grid: by default (`NB_CREASE` unset or "max") the highest cell; "comp" the highest
/// cell of the largest connected group of cells; "comp90" that group's 90th percentile of height; "wmean" the depth-weighted mean
/// height of the group's upper half.
fn crease_top_of(g: &Grid, pits: &[f64], cells: &[usize]) -> f64 {
    let mode = std::env::var("NB_CREASE").unwrap_or_default();
    if cells.is_empty() {
        return f64::NAN;
    }
    if mode.is_empty() || mode == "max" {
        return cells.iter().map(|&k| g.v(k / g.cols)).fold(f64::NAN, f64::max);
    }
    let set: std::collections::HashSet<usize> = cells.iter().copied().collect();
    let mut seen = std::collections::HashSet::new();
    let mut best: Vec<usize> = Vec::new();
    for &c in cells {
        if seen.contains(&c) {
            continue;
        }
        let mut comp = vec![c];
        seen.insert(c);
        let mut i = 0;
        while i < comp.len() {
            let k = comp[i];
            let (r, cc) = ((k / g.cols) as isize, (k % g.cols) as isize);
            for dr in -1..=1isize {
                for dc in -1..=1isize {
                    let (rr, c2) = (r + dr, cc + dc);
                    if rr < 0 || c2 < 0 || rr >= g.rows as isize || c2 >= g.cols as isize {
                        continue;
                    }
                    let n = rr as usize * g.cols + c2 as usize;
                    if set.contains(&n) && seen.insert(n) {
                        comp.push(n);
                    }
                }
            }
            i += 1;
        }
        if comp.len() > best.len() {
            best = comp;
        }
    }
    let ys: Vec<f64> = best.iter().map(|&k| g.v(k / g.cols)).collect();
    match mode.as_str() {
        "comp" => ys.iter().copied().fold(f64::NAN, f64::max),
        "comp90" => percentile(ys, 0.9),
        _ => {
            let mid = percentile(ys.clone(), 0.5);
            let (mut a, mut w) = (0.0, 0.0);
            for &k in &best {
                let y = g.v(k / g.cols);
                if y >= mid {
                    a += pits[k] * y;
                    w += pits[k];
                }
            }
            a / w
        }
    }
}

/// The crease's top read as turns: on vertical cuts through each wing at 65, 75 and 85% of the half-width, the front-most skin
/// z(y) blurred by 1 mm, and the height above the wing's lower edge + 3 mm where it bends inward most (the largest second
/// difference over 2 mm). One value per cut.
fn turn_tops(s: &Surface, hw: f64, wing_low: f64, tip_y: f64) -> Vec<f64> {
    let mut out = Vec::new();
    let (from, to) = (wing_low + 3.0, tip_y + 15.0);
    if !(to > from + 4.0) {
        return out;
    }
    for sgn in [1.0, -1.0] {
        for r in [0.65, 0.75, 0.85] {
            let raw = crate::measure::Cut { from: from - 3.0, z: cut_at(s, sgn * r * hw, from - 3.0, to + 3.0).iter().map(|q| q[1]).collect() };
            let c = raw.blurred(1.0);
            let k = (2.0 / crate::measure::STEP) as usize;
            let mut best = (f64::NEG_INFINITY, f64::NAN);
            for i in k..c.z.len().saturating_sub(k) {
                let y = c.from + i as f64 * crate::measure::STEP;
                if y < from || y > to {
                    continue;
                }
                let t = c.z[i - k] + c.z[i + k] - 2.0 * c.z[i];
                if !t.is_nan() && t > best.0 {
                    best = (t, y);
                }
            }
            out.push(best.1);
        }
    }
    out
}

/// The gaps between two outlines given as (z, value) on the same depths: ours minus KOTOR's at each depth both have.
pub fn outline_gaps(kotor: &[[f64; 2]], ours: &[[f64; 2]]) -> Vec<f64> {
    kotor
        .iter()
        .filter(|k| !k[1].is_nan())
        .filter_map(|k| {
            let o = ours.iter().find(|o| (o[0] - k[0]).abs() < 0.01 * OUTLINE_STEP)?;
            (!o[1].is_nan()).then_some(o[1] - k[1])
        })
        .collect()
}

pub fn rms(g: &[f64]) -> f64 {
    if g.is_empty() { f64::NAN } else { (g.iter().map(|x| x * x).sum::<f64>() / g.len() as f64).sqrt() }
}

/// The rows a solve or a check reads, ours against KOTOR's: the side lower outline's root mean square gap (KOTOR's depths, ours read
/// on KOTOR's depths), the below and above outlines' gaps, the wing's lower edge, crease top and height.
pub fn compare(kotor: &Base, ours: &Base) -> [f64; 7] {
    [
        rms(&outline_gaps(&kotor.lower, &ours.lower)),
        rms(&outline_gaps(&kotor.wing, &ours.wing)),
        rms(&outline_gaps(&kotor.below, &ours.below)),
        rms(&outline_gaps(&kotor.above, &ours.above)),
        ours.wing_low - kotor.wing_low,
        ours.crease_top - kotor.crease_top,
        ours.wing_height - kotor.wing_height,
    ]
}

/// The front-most skin of the cut at `x` across, as (height, depth) every 0.25 mm of height from `from` to `to`, NaN where none.
pub fn cut_at(surface: &Surface, x: f64, from: f64, to: f64) -> Vec<[f64; 2]> {
    let count = ((to - from) / crate::measure::STEP).round() as usize + 1;
    let z = surface.front(0, x, 1, from, count);
    z.iter().enumerate().map(|(k, &z)| [from + k as f64 * crate::measure::STEP, z]).collect()
}

/// The half-width of the nose at its wings, read by the nose's measure (mm), NaN when it cannot be read.
pub fn wing_half_width(v: &[V3], f: &[[u32; 3]]) -> f64 {
    crate::measure::measure(&Surface::new(v, f, &Frame::head())).map(|n| 0.5 * (n.marks.wings[0][0] - n.marks.wings[1][0]).abs()).unwrap_or(f64::NAN)
}

/// The surface of a head near the nose, in the head's frame.
pub fn surface(v: &[V3], f: &[[u32; 3]]) -> Surface {
    Surface::within(v, f, &Frame::head(), &WINDOW)
}

#[cfg(not(target_arch = "wasm32"))]
pub mod tools {
    use super::*;
    use crate::feature::Shot;
    use crate::mesh::input::Input;
    use crate::nose_profile;
    use crate::render::{label, sheet};
    use image::{Rgb, RgbImage};
    use std::path::Path;

    fn read_f32(path: &str) -> Vec<f32> {
        let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
    }

    fn built(flat: &[f32], i: usize, vc: usize) -> Vec<V3> {
        flat[i * vc * 3..(i + 1) * vc * 3].chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect()
    }

    /// `sliders nosebase <command> <data> ...`; returns false for an unknown command.
    pub fn run(command: &str, args: &[String], data: &Path, input: &Input) -> bool {
        match command {
            "look" => look(args, data, input),
            "close" => close(args, data, input),
            "checks" => checks(args, data, input),
            "solve" => solve_all(args, data, input),
            "make" => make(args, data, input),
            "ends" => ends(args, data, input),
            "pairs" => pairs(args, data, input),
            _ => return false,
        }
        true
    }

    /// A place turned for a shot, as `render::clay` turns it.
    fn turn(shot: &Shot, p: V3) -> V3 {
        let (sy, cy) = shot.yaw.to_radians().sin_cos();
        let (x, z) = (p[0] * cy + p[2] * sy, -p[0] * sy + p[2] * cy);
        let (sp, cp) = shot.pitch.to_radians().sin_cos();
        let y = p[1] - shot.pivot;
        [x, cp * y + sp * z + shot.pivot, -sp * y + cp * z]
    }

    /// Clay lit from both sides: faces seen from behind (the inside of a hole) are drawn blue-grey, so holes show.
    pub fn clay2(positions: &[V3], triangles: &[[u32; 3]], shot: &Shot, px: f64) -> RgbImage {
        use crate::{cross, dot, normalize, sub};
        let window = shot.window;
        let w = ((window[1] - window[0]) * px) as u32;
        let h = ((window[3] - window[2]) * px) as u32;
        let p: Vec<V3> = positions.iter().map(|&q| turn(shot, q)).collect();
        let light = normalize([-0.3, 0.5, 1.0]);
        let mut img = RgbImage::from_pixel(w, h, Rgb([40, 44, 52]));
        let mut depth = vec![f64::NEG_INFINITY; (w * h) as usize];
        let to_px = |q: V3| [(q[0] - window[0]) * px, (window[3] - q[1]) * px, q[2]];
        for t in triangles {
            let v = t.map(|i| to_px(p[i as usize]));
            let area = (v[1][0] - v[0][0]) * (v[2][1] - v[0][1]) - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
            if area.abs() < 1e-9 {
                continue;
            }
            let n = normalize(cross(sub(p[t[1] as usize], p[t[0] as usize]), sub(p[t[2] as usize], p[t[0] as usize])));
            let back = n[2] < 0.0;
            let nn = if back { n.map(|c| -c) } else { n };
            let lit = 0.3 + 0.7 * dot(nn, light).max(0.0);
            let base = if back { [120.0, 135.0, 175.0] } else { [222.0, 196.0, 172.0] };
            let col = Rgb(base.map(|c: f64| (c * lit).min(255.0) as u8));
            let x0 = v.iter().map(|q| q[0]).fold(f64::MAX, f64::min).floor().max(0.0) as i64;
            let x1 = v.iter().map(|q| q[0]).fold(f64::MIN, f64::max).ceil().min(w as f64 - 1.0) as i64;
            let y0 = v.iter().map(|q| q[1]).fold(f64::MAX, f64::min).floor().max(0.0) as i64;
            let y1 = v.iter().map(|q| q[1]).fold(f64::MIN, f64::max).ceil().min(h as f64 - 1.0) as i64;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
                    let b0 = ((v[1][0] - fx) * (v[2][1] - fy) - (v[2][0] - fx) * (v[1][1] - fy)) / area;
                    let b1 = ((v[2][0] - fx) * (v[0][1] - fy) - (v[0][0] - fx) * (v[2][1] - fy)) / area;
                    let b2 = 1.0 - b0 - b1;
                    if b0 < 0.0 || b1 < 0.0 || b2 < 0.0 {
                        continue;
                    }
                    let z = b0 * v[0][2] + b1 * v[1][2] + b2 * v[2][2];
                    let at = (y as u32 * w + x as u32) as usize;
                    if z <= depth[at] {
                        continue;
                    }
                    depth[at] = z;
                    img.put_pixel(x as u32, y as u32, col);
                }
            }
        }
        img
    }

    /// A height map drawn in false colour: the height relative to `mid`, grey, with pits deeper than 0.5 mm in red.
    fn map_picture(m: &Map, pit: &[f64], mid: f64, px: u32, flip: bool) -> RgbImage {
        let (w, h) = (m.cols as u32 * px, m.rows as u32 * px);
        let mut img = RgbImage::from_pixel(w, h, Rgb([40, 44, 52]));
        for r in 0..m.rows {
            for c in 0..m.cols {
                let y = m.at(r, c);
                if y.is_nan() {
                    continue;
                }
                let g = (128.0 + (y - mid) * 5.0).clamp(0.0, 255.0) as u8;
                let p = pit[r * m.cols + c];
                let col = if p > 0.5 { Rgb([(140.0 + p * 40.0).min(255.0) as u8, 30, 30]) } else { Rgb([g, g, g]) };
                let rr = if flip { r } else { m.rows - 1 - r };
                for a in 0..px {
                    for b in 0..px {
                        img.put_pixel(c as u32 * px + b, rr as u32 * px + a, col);
                    }
                }
            }
        }
        img
    }

    /// `sliders nosebase look <data> <out.png> <built.bin|kotor> <head> ...`: per head and per source (KOTOR's mesh, then each built
    /// file given before the heads as `name=file`), the nose in clay from below, from above, in profile and from three-quarter below,
    /// and the height maps from below and from above with their pits in red.
    fn look(args: &[String], _data: &Path, input: &Input) {
        let mesh = input.head_mesh();
        let vc = mesh.base.len();
        let names: Vec<String> = input.manifest["heads"].as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect();
        let mut extras = Vec::new();
        let mut heads = Vec::new();
        for a in &args[5..] {
            match a.split_once('=') {
                Some((n, p)) => extras.push((n.to_string(), read_f32(p))),
                None => heads.push(a.clone()),
            }
        }
        // SAFETY: set before any thread starts, and read only by the drawing
        unsafe { std::env::set_var("SLIDERS_PX", "7") };
        let frame = Frame::head();
        let mut tiles = Vec::new();
        for name in &heads {
            let i = names.iter().position(|n| n == name).unwrap();
            let kv = input.points(&format!("heads/{name}/kotor_v"));
            let kf = input.triangles(&format!("heads/{name}/kotor_f"));
            let mut sources: Vec<(String, Vec<V3>, Vec<[u32; 3]>)> = vec![("KOTOR".into(), kv, kf)];
            for (n, flat) in &extras {
                sources.push((n.clone(), built(flat, i, vc), mesh.triangles.clone()));
            }
            // KOTOR's tip and base place every view, for all sources alike
            let kfine = nose_profile::read(&surface(&sources[0].1, &sources[0].2)).expect("KOTOR's profile");
            let (tip, base) = (kfine.tip, kfine.base);
            for (tag, v, f) in &sources {
                let local: Vec<V3> = v.iter().map(|&p| frame.local(p)).collect();
                let keep = |lo: f64, hi: f64| -> Vec<[u32; 3]> {
                    f.iter().copied().filter(|t| t.iter().all(|&q| local[q as usize][1] > lo && local[q as usize][1] < hi && local[q as usize][0].abs() < 30.0)).collect()
                };
                let zw = [tip[1] - DEPTH_SPAN, tip[1] + 2.0];
                let below = Shot { name: "below", yaw: 0.0, pitch: 90.0, pivot: 0.0, window: [-22.0, 22.0, zw[0], zw[1]] };
                let above = Shot { name: "above", yaw: 0.0, pitch: -90.0, pivot: 0.0, window: [-22.0, 22.0, -zw[1], -zw[0]] };
                let side = Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [tip[1] - 26.0, tip[1] + 4.0, base[0] - 10.0, base[0] + 24.0] };
                let mid = base[0] + 0.5 * tip[1];
                let tq = Shot { name: "tq", yaw: 35.0, pitch: 30.0, pivot: base[0], window: [-22.0, 22.0, mid - 18.0, mid + 14.0] };
                let lo_cut = base[0] - 3.0;
                let hi_cut = tip[0] + 12.0;
                let mut img = clay2(&local, &keep(lo_cut, 0.0), &below, 7.0);
                label(&mut img, &format!("{} {tag} below", name.replace('_', " ")));
                tiles.push(img);
                let mut img = clay2(&local, &keep(-90.0, hi_cut), &above, 7.0);
                label(&mut img, &format!("{tag} above"));
                tiles.push(img);
                let mut img = clay2(&local, f, &side, 7.0);
                label(&mut img, &format!("{tag} side"));
                tiles.push(img);
                let mut img = clay2(&local, &keep(lo_cut, 0.0), &tq, 7.0);
                label(&mut img, &format!("{tag} three quarter below"));
                tiles.push(img);
                let s = surface(v, f);
                let mb = height_map(&s, lo_cut, 0.0, zw[0], zw[1], true);
                let pb = pits(&mb, 3.0, true);
                let mut img = map_picture(&mb, &pb, base[0] + 3.0, 2, false);
                label(&mut img, &format!("{tag} map below"));
                tiles.push(image::imageops::resize(&img, tiles[0].width(), tiles[0].height(), image::imageops::FilterType::Nearest));
                let ma = height_map(&s, -90.0, hi_cut, zw[0], zw[1], false);
                let pa = pits(&ma, 3.0, false);
                let mut img = map_picture(&ma, &pa, hi_cut - 6.0, 2, true);
                label(&mut img, &format!("{tag} map above"));
                tiles.push(image::imageops::resize(&img, tiles[0].width(), tiles[0].height(), image::imageops::FilterType::Nearest));
            }
        }
        sheet(&tiles, 6).save(&args[4]).unwrap();
        eprintln!("wrote {}", args[4]);
    }

    /// `sliders nosebase close <data> <out.png> name=built.bin ... <head> ...`: large close-ups, per head a row per source: the side at
    /// 0 and 15 degrees from below, the base from straight below with the lip cut away, and three-quarter from below.
    fn close(args: &[String], _data: &Path, input: &Input) {
        let mesh = input.head_mesh();
        let vc = mesh.base.len();
        let names: Vec<String> = input.manifest["heads"].as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect();
        let mut extras = Vec::new();
        let mut heads = Vec::new();
        for a in &args[5..] {
            match a.split_once('=') {
                Some((n, p)) => extras.push((n.to_string(), read_f32(p))),
                None => heads.push(a.clone()),
            }
        }
        let frame = Frame::head();
        let px = 10.0;
        let mut tiles = Vec::new();
        for name in &heads {
            let i = names.iter().position(|n| n == name).unwrap();
            let kv = input.points(&format!("heads/{name}/kotor_v"));
            let kf = input.triangles(&format!("heads/{name}/kotor_f"));
            let mut sources: Vec<(String, Vec<V3>, Vec<[u32; 3]>)> = vec![("KOTOR".into(), kv, kf)];
            for (n, flat) in &extras {
                sources.push((n.clone(), built(flat, i, vc), mesh.triangles.clone()));
            }
            let kfine = nose_profile::read(&surface(&sources[0].1, &sources[0].2)).expect("KOTOR's profile");
            let (tip, base) = (kfine.tip, kfine.base);
            // the cuts through the wing at 60, 75 and 90% of each source's own half-width, on the side the camera sees (-x)
            let cuts: Vec<Vec<Vec<[f64; 2]>>> = sources
                .iter()
                .map(|(_, v, f)| {
                    let hw = wing_half_width(v, f);
                    let s = surface(v, f);
                    [0.6, 0.75, 0.9].iter().map(|r| cut_at(&s, -r * hw, base[0] - 10.0, tip[0] + 25.0)).collect()
                })
                .collect();
            for (tag, v, f) in &sources {
                let local: Vec<V3> = v.iter().map(|&p| frame.local(p)).collect();
                let keep = |lo: f64| -> Vec<[u32; 3]> {
                    f.iter().copied().filter(|t| t.iter().all(|&q| local[q as usize][1] > lo && local[q as usize][1] < 5.0 && local[q as usize][0].abs() < 30.0)).collect()
                };
                // all windows 32 by 32 mm
                let side = Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [tip[1] - 28.0, tip[1] + 4.0, base[0] - 8.0, base[0] + 24.0] };
                let side15 = Shot { name: "side15", yaw: 90.0, pitch: 15.0, pivot: base[0], window: [tip[1] - 28.0, tip[1] + 4.0, base[0] - 8.0, base[0] + 24.0] };
                let zc = tip[1] - 10.0;
                let below = Shot { name: "below", yaw: 0.0, pitch: 90.0, pivot: 0.0, window: [-16.0, 16.0, zc - 20.0, zc + 12.0] };
                let s = 30f64.to_radians().sin();
                let yc = base[0] + 0.87 * 0.0 + s * (tip[1] - 6.0);
                let tq = Shot { name: "tq", yaw: 40.0, pitch: 30.0, pivot: base[0], window: [-16.0, 16.0, yc - 16.0, yc + 16.0] };
                for (k, (sh, cut)) in [(&side, None), (&side15, None), (&below, Some(base[0] - 1.5)), (&tq, Some(base[0] - 1.5))].into_iter().enumerate() {
                    let tris = match cut { Some(lo) => keep(lo), None => f.clone() };
                    let mut img = clay2(&local, &tris, sh, px);
                    if k == 0 {
                        let colours = [Rgb([10, 10, 10]), Rgb([225, 40, 40]), Rgb([30, 170, 60])];
                        for (n, c) in cuts.iter().enumerate() {
                            for (m, line) in c.iter().enumerate() {
                                if m != 1 {
                                    continue;
                                }
                                for q in line.iter().filter(|q| !q[1].is_nan()) {
                                    let (x, y) = ((q[1] - sh.window[0]) * px, (sh.window[3] - q[0]) * px);
                                    for d in -1..=1i64 {
                                        let (xi, yi) = (x as i64 + d, y as i64);
                                        if xi >= 0 && yi >= 0 && (xi as u32) < img.width() && (yi as u32) < img.height() {
                                            img.put_pixel(xi as u32, yi as u32, colours[n % 3]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let what = ["side", "side 15 below", "below", "three quarter below"][k];
                    label(&mut img, &format!("{} {tag} {what}", name.replace('_', " ")));
                    tiles.push(img);
                }
            }
        }
        sheet(&tiles, 4).save(&args[4]).unwrap();
        eprintln!("wrote {}", args[4]);
    }

    fn median(mut v: Vec<f64>) -> f64 {
        v.retain(|x| !x.is_nan());
        v.sort_by(f64::total_cmp);
        if v.is_empty() { f64::NAN } else { v[v.len() / 2] }
    }

    fn p95(mut v: Vec<f64>) -> f64 {
        v.retain(|x| !x.is_nan());
        v.sort_by(f64::total_cmp);
        if v.is_empty() { f64::NAN } else { v[((v.len() - 1) as f64 * 0.95).round() as usize] }
    }

    pub const ROWS: [&str; 7] = ["side lower outline", "wing lower outline", "outline from below", "outline from above", "wing lower edge", "crease top", "wing height"];

    fn json_of(b: &Result<Base, &'static str>) -> serde_json::Value {
        match b {
            Ok(b) => serde_json::to_value(b).unwrap(),
            Err(e) => serde_json::json!({"error": e}),
        }
    }

    /// `sliders nosebase checks <data> <out.json> <pictures folder> [name=built.bin ...] [head ...]`: the checks of the measure on
    /// KOTOR's 57 meshes (its subdivided copy for the wobble, its rounded copy for check 3), each built file's rows against KOTOR,
    /// and pictures of the heads named.
    fn checks(args: &[String], _data: &Path, input: &Input) {
        let mesh = input.head_mesh();
        let vc = mesh.base.len();
        let names: Vec<String> = input.manifest["heads"].as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect();
        let mut extras = Vec::new();
        let mut pics = Vec::new();
        for a in &args[6..] {
            match a.split_once('=') {
                Some((n, p)) => extras.push((n.to_string(), read_f32(p))),
                None => pics.push(a.clone()),
            }
        }
        struct Row {
            kotor: Result<Base, &'static str>,
            sub: Result<Base, &'static str>,
            rounded: Result<Base, &'static str>,
            built: Vec<Result<Base, &'static str>>,
        }
        let rows: Vec<Row> = crate::heads::parallel(names.len(), crate::heads::threads(), |i| {
            let n = &names[i];
            let v = input.points(&format!("heads/{n}/kotor_v"));
            let f = input.triangles(&format!("heads/{n}/kotor_f"));
            let (sv, sf, _) = crate::checks::butterfly(&v, &f);
            let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
            let (lv, lf, _) = crate::checks::loop_subdivide(&v, &f);
            Row {
                kotor: read(&v, &f),
                sub: read(&sv, &sf),
                rounded: read(&lv, &lf),
                built: extras.iter().map(|(_, flat)| read(&built(flat, i, vc), &mesh.triangles)).collect(),
            }
        });
        for (i, r) in rows.iter().enumerate() {
            if let Err(e) = &r.kotor {
                eprintln!("{}: KOTOR cannot be read: {e}", names[i]);
            }
        }
        let ok: Vec<usize> = (0..names.len()).filter(|&i| rows[i].kotor.is_ok() && rows[i].sub.is_ok()).collect();
        let k = |i: usize| rows[i].kotor.as_ref().unwrap();
        let mut wob = vec![Vec::new(); 7];
        for &i in &ok {
            let c = compare(k(i), rows[i].sub.as_ref().unwrap());
            for r in 0..7 {
                wob[r].push(c[r].abs());
            }
        }
        let mut pair = vec![Vec::new(); 7];
        for &i in &ok {
            for &j in &ok {
                if j != i {
                    let c = compare(k(i), k(j));
                    for r in 0..7 {
                        pair[r].push(c[r].abs());
                    }
                }
            }
        }
        // the wobble when KOTOR's side is read on a subdivided copy: its butterfly copy against its rounded (Loop) copy
        let mut wob_sub = vec![Vec::new(); 7];
        let mut pair_sub = vec![Vec::new(); 7];
        for &i in &ok {
            if let (Ok(a), Ok(b)) = (&rows[i].sub, &rows[i].rounded) {
                let c = compare(a, b);
                for r in 0..7 {
                    wob_sub[r].push(c[r].abs());
                }
                for &j in &ok {
                    if j != i {
                        let c = compare(a, rows[j].sub.as_ref().unwrap());
                        for r in 0..7 {
                            pair_sub[r].push(c[r].abs());
                        }
                    }
                }
            }
        }
        eprintln!("read on the subdivided copy: its butterfly copy against its rounded copy (median, 95th percentile), two heads' copies (median):");
        for r in 0..7 {
            eprintln!("  {:20} {:5.2} {:5.2} mm, two heads {:5.2} mm", ROWS[r], median(wob_sub[r].clone()), p95(wob_sub[r].clone()), median(pair_sub[r].clone()));
        }
        let w95: Vec<f64> = (0..7).map(|r| p95(wob[r].clone()).max(0.05)).collect();
        eprintln!("check 1, KOTOR against its subdivided copy (median, 95th percentile), and two different KOTOR heads (median):");
        for r in 0..7 {
            eprintln!("  {:20} {:5.2} {:5.2} mm, two heads {:5.2} mm", ROWS[r], median(wob[r].clone()), w95[r], median(pair[r].clone()));
        }
        // check 3: the rounded copy's nearest KOTOR head, by every row over its wobble
        let dist = |a: &Base, b: &Base| compare(a, b).iter().zip(&w95).map(|(x, w)| if x.is_nan() { 0.0 } else { (x / w).powi(2) }).sum::<f64>();
        let mut selfs = 0;
        let mut of = 0;
        for &i in &ok {
            let Ok(r) = &rows[i].rounded else { continue };
            of += 1;
            let best = ok.iter().copied().min_by(|&a, &b| dist(k(a), r).total_cmp(&dist(k(b), r))).unwrap();
            if best == i {
                selfs += 1;
            }
        }
        eprintln!("check 3: the rounded copy's nearest KOTOR head is itself for {selfs} of {of}");
        // the holes' shape: wobble against two heads
        let hole_of = |b: &Base| [b.hole_area, b.hole_centre[0], b.hole_centre[1], b.hole_len, b.hole_wid, b.hole_fill];
        let hole_names = ["hole area (mm2)", "hole centre across", "hole centre depth", "hole length", "hole width", "hole fill"];
        let mut hw_ = vec![Vec::new(); 6];
        let mut hp = vec![Vec::new(); 6];
        for &i in &ok {
            let (a, b) = (hole_of(k(i)), hole_of(rows[i].sub.as_ref().unwrap()));
            for q in 0..6 {
                hw_[q].push((a[q] - b[q]).abs());
            }
            for &j in &ok {
                if j != i {
                    let c = hole_of(k(j));
                    for q in 0..6 {
                        hp[q].push((a[q] - c[q]).abs());
                    }
                }
            }
        }
        eprintln!("the holes, KOTOR against its subdivided copy (median, 95th percentile), and two heads (median):");
        for q in 0..6 {
            eprintln!("  {:20} {:6.2} {:6.2}, two heads {:6.2}", hole_names[q], median(hw_[q].clone()), p95(hw_[q].clone()), median(hp[q].clone()));
        }
        for (e, (n, _)) in extras.iter().enumerate() {
            let gaps: Vec<Vec<f64>> = (0..6).map(|q| ok.iter().filter_map(|&i| rows[i].built[e].as_ref().ok().map(|b| hole_of(b)[q] - hole_of(k(i))[q])).collect()).collect();
            eprintln!("  {n}: median gap {}", (0..6).map(|q| format!("{} {:.2}", hole_names[q], median(gaps[q].clone()))).collect::<Vec<_>>().join(", "));
        }
        // check 4: the most alike and most different pairs, and the best and worst fits of the last built file, drawn
        if std::env::var("NB_CHECK4").is_ok() {
            let mut pairs = Vec::new();
            for (a, &i) in ok.iter().enumerate() {
                for &j in &ok[a + 1..] {
                    pairs.push((dist(k(i), k(j)), i, j));
                }
            }
            pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
            let dir = Path::new(&args[5]);
            std::fs::create_dir_all(dir).unwrap();
            let src = |i: usize| -> (String, Vec<V3>, Vec<[u32; 3]>) {
                (names[i].clone(), input.points(&format!("heads/{}/kotor_v", names[i])), input.triangles(&format!("heads/{}/kotor_f", names[i])))
            };
            let mut tiles = Vec::new();
            for (tag, list) in [("alike", &pairs[..2]), ("different", &pairs[pairs.len() - 2..])] {
                for &(d, i, j) in list {
                    let (a, b) = (src(i), src(j));
                    eprintln!("check 4 {tag}: {} and {} ({d:.1})", names[i], names[j]);
                    let srcs = vec![(a.0.clone(), a.1, a.2, k(i)), (b.0.clone(), b.1, b.2, k(j))];
                    tiles.push(picture(&format!("{tag} {}", names[i]), &srcs));
                }
            }
            if let Some(e) = extras.len().checked_sub(1) {
                let mut fits: Vec<(f64, usize)> = ok.iter().filter_map(|&i| rows[i].built[e].as_ref().ok().map(|b| (dist(k(i), b), i))).collect();
                fits.sort_by(|a, b| a.0.total_cmp(&b.0));
                for (tag, &(d, i)) in [("best fit", &fits[0]), ("worst fit", fits.last().unwrap())] {
                    eprintln!("check 4 {tag}: {} ({d:.1})", names[i]);
                    let a = src(i);
                    let ours = built(&extras[e].1, i, vc);
                    let srcs = vec![(a.0.clone(), a.1, a.2, k(i)), (extras[e].0.clone(), ours, mesh.triangles.clone(), rows[i].built[e].as_ref().unwrap())];
                    tiles.push(picture(&format!("{tag} {}", names[i]), &srcs));
                }
            }
            let w = tiles.iter().map(|t| t.width()).max().unwrap();
            let h: u32 = tiles.iter().map(|t| t.height() + 6).sum();
            let mut out = RgbImage::from_pixel(w, h, Rgb([15, 15, 18]));
            let mut y = 0;
            for t in &tiles {
                image::imageops::overlay(&mut out, t, 0, y as i64);
                y += t.height() + 6;
            }
            out.save(dir.join("check4.png")).unwrap();
        }
        let hole_heads = ok.iter().filter(|&&i| k(i).hole_area > 1.0).count();
        eprintln!("KOTOR heads with nostril holes over 1 mm²: {hole_heads} of {}", ok.len());
        let mut heads = serde_json::Map::new();
        for i in 0..names.len() {
            let mut h = serde_json::Map::new();
            h.insert("kotor".into(), json_of(&rows[i].kotor));
            h.insert("kotor_sub".into(), json_of(&rows[i].sub));
            for (e, (n, _)) in extras.iter().enumerate() {
                let mut v = json_of(&rows[i].built[e]);
                if let (Ok(a), Ok(b)) = (&rows[i].kotor, &rows[i].built[e]) {
                    v["rows"] = serde_json::json!(compare(a, b));
                }
                h.insert(n.clone(), v);
            }
            heads.insert(names[i].clone(), serde_json::Value::Object(h));
        }
        for (e, (n, _)) in extras.iter().enumerate() {
            eprintln!("{n}: median gaps over the heads KOTOR can be read on:");
            for r in 0..7 {
                let v: Vec<f64> = ok.iter().filter_map(|&i| rows[i].built[e].as_ref().ok().map(|b| compare(k(i), b)[r])).collect();
                eprintln!("  {:20} median {:6.2}  median |x| {:5.2}", ROWS[r], median(v.clone()), median(v.iter().map(|x| x.abs()).collect()));
            }
        }
        let doc = serde_json::json!({
            "rows": ROWS,
            "check1": {"median": (0..7).map(|r| median(wob[r].clone())).collect::<Vec<_>>(), "p95": w95},
            "pairs_median": (0..7).map(|r| median(pair[r].clone())).collect::<Vec<_>>(),
            "check3": {"self": selfs, "of": of},
            "heads": heads,
        });
        std::fs::write(&args[4], doc.to_string()).unwrap();
        eprintln!("wrote {}", args[4]);
        let dir = Path::new(&args[5]);
        std::fs::create_dir_all(dir).unwrap();
        for name in &pics {
            let i = names.iter().position(|n| n == name).unwrap();
            let Ok(kb) = &rows[i].kotor else { continue };
            let mut sources: Vec<(String, Vec<V3>, Vec<[u32; 3]>, &Base)> = vec![(
                "KOTOR".into(),
                input.points(&format!("heads/{name}/kotor_v")),
                input.triangles(&format!("heads/{name}/kotor_f")),
                kb,
            )];
            for (e, (n, flat)) in extras.iter().enumerate() {
                if let Ok(b) = &rows[i].built[e] {
                    sources.push((n.clone(), built(flat, i, vc), mesh.triangles.clone(), b));
                }
            }
            let img = picture(name, &sources);
            let path = dir.join(format!("base_{name}.png"));
            img.save(&path).unwrap();
            eprintln!("wrote {}", path.display());
        }
    }

    fn line(img: &mut RgbImage, pts: &[(f64, f64)], c: Rgb<u8>, thick: i64) {
        for w in pts.windows(2) {
            if w[0].0.is_nan() || w[0].1.is_nan() || w[1].0.is_nan() || w[1].1.is_nan() {
                continue;
            }
            for t in 0..=16 {
                let a = t as f64 / 16.0;
                let (x, y) = (w[0].0 + a * (w[1].0 - w[0].0), w[0].1 + a * (w[1].1 - w[0].1));
                for d in -thick..=thick {
                    for e in -thick..=thick {
                        let (xi, yi) = (x as i64 + d, y as i64 + e);
                        if xi >= 0 && yi >= 0 && (xi as u32) < img.width() && (yi as u32) < img.height() {
                            img.put_pixel(xi as u32, yi as u32, c);
                        }
                    }
                }
            }
        }
    }

    const COLOURS: [Rgb<u8>; 4] = [Rgb([10, 10, 10]), Rgb([225, 40, 40]), Rgb([30, 170, 60]), Rgb([40, 110, 230])];

    /// One head's picture: per source, the base from below (the skin under the nose in grey by height, the holes in red, every
    /// source's outline from below and from above), and the side close-up with every source's lower outline, crease top and lower edge.
    pub fn picture(name: &str, sources: &[(String, Vec<V3>, Vec<[u32; 3]>, &Base)]) -> RgbImage {
        let px = 10.0;
        let k = sources[0].3;
        let (tip, base) = (k.tip, k.base);
        let frame = Frame::head();
        let mut tiles = Vec::new();
        // from below: x across, z up the picture, 32 by 32 mm
        let win = [-16.0, 16.0, tip[1] - 26.0, tip[1] + 6.0];
        let to_b = |x: f64, z: f64| ((x - win[0]) * px, (win[3] - z) * px);
        for (tag, _, _, b) in sources {
            let mut img = RgbImage::from_pixel((32.0 * px) as u32, (32.0 * px) as u32, Rgb([40, 44, 52]));
            if let Some((g, under, pits)) = &b.grid_below {
                for r in 0..g.rows {
                    for c in 0..g.cols {
                        let q = r * g.cols + c;
                        if !under[q] && pits[q] <= HOLE_DEPTH {
                            continue;
                        }
                        let (x, y) = to_b(g.u(c), g.v(r));
                        let col = if pits[q] > HOLE_DEPTH {
                            Rgb([200, 40, 40])
                        } else {
                            let t = (150.0 + (g.at(r, c) - base[0]) * 9.0).clamp(30.0, 245.0) as u8;
                            Rgb([t, t, t])
                        };
                        for a in 0..(CELL * px) as i64 {
                            for e in 0..(CELL * px) as i64 {
                                let (xi, yi) = (x as i64 + a, y as i64 - e);
                                if xi >= 0 && yi >= 0 && (xi as u32) < img.width() && (yi as u32) < img.height() {
                                    img.put_pixel(xi as u32, yi as u32, col);
                                }
                            }
                        }
                    }
                }
            }
            for (n, (_, _, _, o)) in sources.iter().enumerate() {
                for (outline, thick) in [(&o.below, 1), (&o.above, 0)] {
                    for s in [1.0, -1.0] {
                        let pts: Vec<(f64, f64)> = outline.iter().map(|q| to_b(s * q[1], q[0])).collect();
                        line(&mut img, &pts, COLOURS[n % 4], thick);
                    }
                }
            }
            label(&mut img, &format!("{} {tag} below", name.replace('_', " ")));
            tiles.push(img);
        }
        // from above, the face cut 14 mm above KOTOR's tip so the brow does not hide the nose: x across, z down the picture
        let above = Shot { name: "above", yaw: 0.0, pitch: -90.0, pivot: 0.0, window: [-16.0, 16.0, -(tip[1] + 6.0), -(tip[1] - 26.0)] };
        let to_a = |x: f64, z: f64| ((x - above.window[0]) * px, (above.window[3] + z) * px);
        for (tag, v, f, _) in sources {
            let local: Vec<V3> = v.iter().map(|&p| frame.local(p)).collect();
            let keep: Vec<[u32; 3]> = f.iter().copied().filter(|t| t.iter().all(|&q| local[q as usize][1] < tip[0] + OVER_TIP)).collect();
            let mut img = clay2(&local, &keep, &above, px);
            for (n, (_, _, _, o)) in sources.iter().enumerate() {
                for sgn in [1.0, -1.0] {
                    let pts: Vec<(f64, f64)> = o.above.iter().map(|q| to_a(sgn * q[1], q[0])).collect();
                    line(&mut img, &pts, COLOURS[n % 4], if n == 0 { 1 } else { 0 });
                }
            }
            label(&mut img, &format!("{} {tag} above", name.replace('_', " ")));
            tiles.push(img);
        }
        // the side, as the camera sees the -x side: z across, y up
        let side = Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [tip[1] - 28.0, tip[1] + 4.0, base[0] - 8.0, base[0] + 24.0] };
        let to_s = |z: f64, y: f64| ((z - side.window[0]) * px, (side.window[3] - y) * px);
        for (tag, v, f, _) in sources {
            let local: Vec<V3> = v.iter().map(|&p| frame.local(p)).collect();
            let mut img = clay2(&local, f, &side, px);
            for (n, (_, _, _, o)) in sources.iter().enumerate() {
                let pts: Vec<(f64, f64)> = o.wing.iter().map(|q| to_s(q[0], q[1])).collect();
                line(&mut img, &pts, COLOURS[n % 4], if n == 0 { 1 } else { 0 });
                // the crease top and the lower edge, as short marks behind the wing
                let zm = o.base[1] + 2.0;
                for (y, w) in [(o.crease_top, 3.0), (o.wing_low, 3.0)] {
                    line(&mut img, &[to_s(zm - w, y), to_s(zm + w, y)], COLOURS[n % 4], 1);
                }
            }
            label(&mut img, &format!("{} {tag} side", name.replace('_', " ")));
            tiles.push(img);
        }
        sheet(&tiles, sources.len())
    }

    fn env_or(k: &str, d: f64) -> f64 {
        std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
    }

    /// The marks of the new rows in the solve (mm): a gap of this size on every point of an outline, or on a single number, costs one
    /// row at its mark. They are the 95th percentile of each row's wobble (check 1).
    const MARK_LOWER: f64 = 1.0;
    const MARK_WING_LINE: f64 = 1.55;
    const MARK_BELOW: f64 = 1.13;
    const MARK_ABOVE: f64 = 1.17;
    const MARK_WING_LOW: f64 = 1.07;
    const MARK_CREASE: f64 = 2.0;

    fn mark_crease() -> f64 {
        env_or("NB_MARK_CREASE", MARK_CREASE)
    }
    /// The fine side profile's mark in the solve, as the profile solve of 2026-11-21 used.
    const MARK_PROFILE: f64 = 0.15;
    const PRIOR: f64 = 0.01;

    /// Ours minus KOTOR's at each of KOTOR's points, `missing` where ours has nothing there, over `mark` and the root of the count.
    fn fixed_gaps(kotor: &[[f64; 2]], ours: &[[f64; 2]], mark: f64, missing: f64, out: &mut Vec<f64>) {
        let pts: Vec<&[f64; 2]> = kotor.iter().filter(|k| !k[1].is_nan()).collect();
        let w = 1.0 / (mark * (pts.len().max(1) as f64).sqrt());
        for k in pts {
            let o = ours.iter().find(|o| (o[0] - k[0]).abs() < 0.01 * OUTLINE_STEP).map(|o| o[1]).unwrap_or(f64::NAN);
            out.push(if o.is_nan() { missing } else { (o - k[1]) * w });
        }
    }

    /// The rows of the base, over their marks. `WEIGHT_WING` scales the wing's rows (its lower outline and lower edge).
    fn base_rows(k: &Base, o: &Base, out: &mut Vec<f64>) {
        let ww = env_or("NB_WEIGHT_WING", 1.0);
        let n0 = out.len();
        fixed_gaps(&k.lower, &o.lower, MARK_LOWER, 2.0, out);
        fixed_gaps(&k.below, &o.below, MARK_BELOW, 2.0, out);
        fixed_gaps(&k.above, &o.above, MARK_ABOVE, 2.0, out);
        let n1 = out.len();
        fixed_gaps(&k.wing, &o.wing, MARK_WING_LINE / ww, 2.0, out);
        let _ = (n0, n1);
        let num = |a: f64, b: f64, m: f64| if a.is_nan() || b.is_nan() { 0.0 } else { (a - b) / m };
        out.push(num(o.wing_low, k.wing_low, MARK_WING_LOW / ww));
        out.push(num(o.crease_top, k.crease_top, mark_crease()));
    }

    /// The head viewer 2 shows: viewer 2's head with the nose at 0, plus the nose sliders' smoothed movement.
    struct Shown {
        base: Vec<V3>,
        zero: Vec<V3>,
    }

    fn shown(ev: &crate::solve::Evaluator, sh: &Shown, v: &[f64]) -> Vec<V3> {
        let mut full = sh.base.clone();
        for ((&i, p), z) in ev.vertices.iter().zip(ev.positions(v)).zip(&sh.zero) {
            for c in 0..3 {
                full[i as usize][c] += p[c] - z[c];
            }
        }
        full
    }

    struct Target<'a> {
        kit: &'a crate::feature::Measured,
        fine: &'a nose_profile::Fine,
        base: &'a Base,
    }

    fn rows(ev: &crate::solve::Evaluator, sh: &Shown, tris: &[[u32; 3]], t: &Target, marks: &crate::feature::Scale, v: &[f64], spread: &[f64]) -> Option<Vec<f64>> {
        let full = shown(ev, sh, v);
        let s = surface(&full, tris);
        let kit = ev.feature.measure(&crate::measure::Surface::within(&full, tris, &ev.frame, &ev.feature.window())).ok()?;
        let fine = nose_profile::read(&s).ok()?;
        let b = read(&full, tris).ok()?;
        let mut r = crate::solve::gaps(ev.feature, &kit, t.kit, marks);
        let (g, _) = nose_profile::gaps(t.fine, &fine);
        let k = 1.0 / (MARK_PROFILE * (g.len() as f64).sqrt());
        r.extend(g.iter().map(|x| x * k));
        base_rows(t.base, &b, &mut r);
        r.extend(v.iter().zip(spread).map(|(x, s)| PRIOR.sqrt() * x / s));
        Some(r)
    }

    struct Fit {
        values: Vec<f64>,
        cost: f64,
    }

    #[allow(clippy::too_many_arguments)]
    fn descend(ev: &crate::solve::Evaluator, sh: &Shown, tris: &[[u32; 3]], t: &Target, marks: &crate::feature::Scale, spread: &[f64], start: &[f64], bounds: &[[f64; 2]], free: &[bool]) -> Fit {
        let k = start.len();
        let mut v: Vec<f64> = start.iter().zip(bounds).map(|(x, b)| x.clamp(b[0], b[1])).collect();
        let cost = |r: &[f64]| r.iter().map(|x| x * x).sum::<f64>();
        let Some(mut r) = rows(ev, sh, tris, t, marks, &v, spread) else { return Fit { values: v, cost: f64::INFINITY } };
        let mut c = cost(&r);
        let mut mu = 1e-2;
        for _ in 0..30 {
            let m = r.len();
            let mut jac = vec![vec![0.0; k]; m];
            for j in (0..k).filter(|&j| free[j]) {
                let h = 0.05 * spread[j];
                let h = if v[j] + h > bounds[j][1] { -h } else { h };
                let mut w = v.clone();
                w[j] += h;
                if let Some(rj) = rows(ev, sh, tris, t, marks, &w, spread) {
                    for i in 0..m.min(rj.len()) {
                        jac[i][j] = (rj[i] - r[i]) / h;
                    }
                }
            }
            let mut jtj = nalgebra::DMatrix::<f64>::zeros(k, k);
            let mut jtr = nalgebra::DVector::<f64>::zeros(k);
            for i in 0..m {
                for a in 0..k {
                    jtr[a] += jac[i][a] * r[i];
                    for b in 0..k {
                        jtj[(a, b)] += jac[i][a] * jac[i][b];
                    }
                }
            }
            let mut improved = false;
            for _ in 0..8 {
                let mut a = jtj.clone();
                for d in 0..k {
                    a[(d, d)] += mu * (jtj[(d, d)] + 1e-9) + if free[d] { 0.0 } else { 1e9 };
                }
                let Some(step) = a.cholesky().map(|ch| ch.solve(&(-&jtr))) else {
                    mu *= 10.0;
                    continue;
                };
                let w: Vec<f64> = v.iter().zip(step.iter()).zip(bounds).map(|((x, s), b)| (x + s).clamp(b[0], b[1])).collect();
                if let Some(rw) = rows(ev, sh, tris, t, marks, &w, spread) {
                    let cw = cost(&rw);
                    if cw < c {
                        improved = c - cw > 1e-6 * c.max(1e-9);
                        v = w;
                        r = rw;
                        c = cw;
                        mu = (mu * 0.3).max(1e-6);
                        break;
                    }
                }
                mu *= 10.0;
            }
            if !improved {
                break;
            }
        }
        Fit { values: v, cost: c }
    }

    /// `sliders nosebase solve <data> <out.json>` with `NOSE_VALUES` (viewer 2's values, as `dump.mjs` writes them), `NB_VIEWER`
    /// (viewer 2's heads with the nose at 0) and optionally `SLIDERS_SET` (by default the live 16-slider set) and `NB_HEADS=a,b`.
    /// Each head is solved from its live values against the kit's nose rows, the fine side profile and the base's rows, then guarded:
    /// it keeps the largest share of its move (1, 0.75, 0.5, 0.25) that has no more folded triangles near the nose than live, fewer
    /// than three of the kit's rows worse than live by more than half their mark, a side profile no worse (bridge gap +0.05 mm, swing
    /// +0.25 mm), and a base cost at least 10% lower than live's.
    fn solve_all(args: &[String], data: &Path, input: &Input) {
        use crate::heads::World;
        use crate::set::{SMOOTHING, SliderSet};
        let world = World::load(input, crate::feature::by_key("nose").unwrap());
        let f = world.feature;
        let live_path = data.join("web/sliders_r2_profile.json");
        let set_path = std::env::var("SLIDERS_SET").map(std::path::PathBuf::from).unwrap_or_else(|_| live_path.clone());
        let set: SliderSet = serde_json::from_str(&std::fs::read_to_string(&set_path).unwrap()).unwrap();
        let live_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(&live_path).unwrap()).unwrap();
        let m = crate::checks::Marks::read(&data.join(f.files().marks));
        let spread: Vec<f64> = set
            .sliders
            .iter()
            .map(|s| if s.measure.is_empty() { 0.25 * (s.high - s.low) } else { m.spread_p[crate::feature::proportion_index(f, &s.measure)] })
            .collect();
        let bounds: Vec<[f64; 2]> = set.sliders.iter().map(|s| [s.low, s.high]).collect();
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(std::env::var("NOSE_VALUES").unwrap()).unwrap()).unwrap();
        let only: Option<Vec<String>> = std::env::var("NB_HEADS").ok().map(|s| s.split(',').map(String::from).collect());
        let viewer = read_f32(&std::env::var("NB_VIEWER").unwrap());
        // NB_FREE=a,b lets only those sliders move (by name); by default all
        let free_names: Option<Vec<String>> = std::env::var("NB_FREE").ok().map(|s| s.split(',').map(String::from).collect());
        let free: Vec<bool> = set.sliders.iter().map(|s| free_names.as_ref().is_none_or(|n| n.contains(&s.name))).collect();
        let smoothed = crate::solve::Smoothed::new(&set, &world.smoother);
        let window = f.window();
        let np = f.proportions().len();
        let vc = world.mesh.base.len();
        let tris = world.mesh.triangles.clone();
        let t0 = std::time::Instant::now();
        let out = crate::heads::parallel(world.heads.len(), crate::heads::threads(), |i| {
            let h = &world.heads[i];
            if only.as_ref().is_some_and(|o| !o.contains(&h.name)) {
                return None;
            }
            let kv = &h.kotor_v;
            let kfine = nose_profile::read(&surface(kv, &h.kotor_f)).ok()?;
            let mut kbase = read(kv, &h.kotor_f).ok()?;
            // NB_TARGET_SUB=1: KOTOR's crease top read on its twice-subdivided copy, where it passes check 1
            if std::env::var("NB_TARGET_SUB").is_ok() {
                let (sv, sf, _) = crate::checks::butterfly(kv, &h.kotor_f);
                let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
                if let Ok(sb) = read(&sv, &sf) {
                    kbase.crease_top = sb.crease_top;
                    kbase.wing_height = kbase.crease_top - kbase.wing_low;
                }
            }
            let target = Target { kit: &h.kotor, fine: &kfine, base: &kbase };
            let live_vals: Vec<f64> = v["values"][&h.name].as_array().unwrap()[..live_set.sliders.len()].iter().map(|x| x.as_f64().unwrap()).collect();
            let start: Vec<f64> = set.sliders.iter().map(|s| live_set.sliders.iter().position(|l| l.name == s.name).map_or(0.0, |j| live_vals[j])).collect();
            let ev = crate::solve::Evaluator::with(f, &set, &smoothed, &world.smoother, &h.start, &tris, h.frame);
            let sh = Shown { base: built(&viewer, i, vc), zero: ev.positions(&vec![0.0; set.sliders.len()]) };
            let mut starts = vec![start.clone()];
            for (j, s) in set.sliders.iter().enumerate() {
                if s.group == "wings" && free[j] {
                    let mut a = start.clone();
                    a[j] = 0.5 * (a[j] + s.high);
                    starts.push(a);
                }
            }
            let fit = starts
                .iter()
                .map(|s| descend(&ev, &sh, &tris, &target, &m.marks, &spread, s, &bounds, &free))
                .min_by(|a, b| a.cost.total_cmp(&b.cost))
                .unwrap();
            // the guard
            let before = world.smoother.smooth(&h.start, SMOOTHING);
            let near: Vec<u32> = (0..before.len() as u32).filter(|&q| window.contains(h.frame.local(before[q as usize]))).collect();
            let checker = crate::check::Checker::new(&world.mesh, &before, &near);
            let grade = |vals: &[f64]| {
                let full = shown(&ev, &sh, vals);
                let kit = f.measure(&crate::measure::Surface::within(&full, &tris, &ev.frame, &window)).ok();
                let fine = nose_profile::read(&surface(&full, &tris)).ok().map(|x| {
                    let (g, b) = nose_profile::gaps(&kfine, &x);
                    nose_profile::scores(&g, &b)
                });
                let b = read(&full, &tris).ok();
                let mut br = Vec::new();
                let bcost = b.as_ref().map(|b| {
                    base_rows(&kbase, b, &mut br);
                    br.iter().map(|x| x * x).sum::<f64>()
                });
                let rows = b.as_ref().map(|b| compare(&kbase, b));
                let folds = checker.faults(&set.build(&world.smoother, &h.start, vals)).total();
                (kit, fine, bcost, rows, folds)
            };
            let row_gaps = |meas: &crate::feature::Measured| -> Vec<f64> {
                let r: Vec<f64> = (0..np).map(|k| ((meas.proportions[k] - h.kotor.proportions[k]) / m.marks.proportions[k]).abs()).collect();
                r.iter().map(|x| if x.is_nan() { 0.0 } else { *x }).collect()
            };
            let (lk, lfine, lcost, lrows, lfolds) = grade(&start);
            let mut chosen = None;
            let mut tried = Vec::new();
            for share in [1.0, 0.75, 0.5, 0.25] {
                let vals: Vec<f64> = start.iter().zip(&fit.values).map(|(a, b)| a + share * (b - a)).collect();
                let (nk, nfine, ncost, nrows, nfolds) = grade(&vals);
                let worse: Vec<usize> = match (&nk, &lk) {
                    (Some(a), Some(b)) => row_gaps(a).iter().zip(row_gaps(b)).enumerate().filter(|(_, (x, y))| **x > *y + 0.5).map(|(k, _)| k).collect(),
                    _ => (0..np).collect(),
                };
                let profile_ok = match (nfine, lfine) {
                    (Some(n), Some(l)) => n[0] <= l[0] + 0.05 && n[3] <= l[3] + 0.25,
                    _ => false,
                };
                let better = matches!((ncost, lcost), (Some(n), Some(l)) if n <= 0.9 * l);
                let pass = nfolds <= lfolds && worse.len() < 3 && profile_ok && better;
                tried.push(serde_json::json!({"share": share, "folds": nfolds, "worse_rows": worse, "profile": nfine, "base_cost": ncost, "rows": nrows, "pass": pass}));
                if pass {
                    chosen = Some((share, vals));
                    break;
                }
            }
            let (share, values) = chosen.unwrap_or((0.0, start.clone()));
            let (_, ffine, fcost, frows, ffolds) = grade(&values);
            eprintln!(
                "{:22} share {share:4.2}  base cost {:6.1} -> {:6.1}  wing low {:5.2} -> {:5.2}  wing height {:5.2} -> {:5.2}  folds {lfolds} -> {ffolds}",
                h.name,
                lcost.unwrap_or(f64::NAN),
                fcost.unwrap_or(f64::NAN),
                lrows.map_or(f64::NAN, |r| r[4]),
                frows.map_or(f64::NAN, |r| r[4]),
                lrows.map_or(f64::NAN, |r| r[6]),
                frows.map_or(f64::NAN, |r| r[6])
            );
            Some((h.name.clone(), serde_json::json!({
                "share": share, "values": values, "solved": fit.values, "live": start, "cost": fit.cost,
                "live_rows": lrows, "rows": frows, "live_base_cost": lcost, "base_cost": fcost, "live_profile": lfine, "profile": ffine,
                "live_folds": lfolds, "folds": ffolds, "tried": tried,
            })))
        });
        let heads: serde_json::Map<String, serde_json::Value> = out.into_iter().flatten().collect();
        let doc = serde_json::json!({"sliders": set.sliders.iter().map(|s| s.name.clone()).collect::<Vec<_>>(), "rows": ROWS, "heads": heads});
        std::fs::write(&args[4], serde_json::to_string_pretty(&doc).unwrap()).unwrap();
        eprintln!("solved in {:.1?}; wrote {}", t0.elapsed(), args[4]);
    }

    /// The movement of the slider Wing height at 1 mm, read on the base head: the wing's skin moves down, by nothing at its crease's
    /// top and by 1 mm at its lower edge and below, so the wing grows taller. Under the lower edge the push fades out over `NB_FADE`
    /// mm (4 by default). Across, it grows from 30% to 60% of the wings' half-width and fades out from 2 mm past the wing to 8 mm past
    /// it, so the tip, the columella and the cheek stay.
    pub fn wing_height_movement(base: &[V3], tris: &[[u32; 3]]) -> Vec<V3> {
        let frame = Frame::head();
        let b = read(base, tris).expect("the base head's nose");
        let (yt, yl, hw) = (b.crease_top, b.wing_low, b.half_width);
        let fade = env_or("NB_FADE", 4.0);
        let inner = env_or("NB_INNER", 0.3);
        let smooth = |t: f64| {
            let t = t.clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let down = frame.axes[1].map(|c| -c);
        eprintln!("base head: crease top {yt:.2}, wing lower edge {yl:.2}, half-width {hw:.2}");
        base.iter()
            .map(|&p| {
                let q = frame.local(p);
                let wy = if q[1] >= yt { 0.0 } else if q[1] >= yl { smooth((yt - q[1]) / (yt - yl)) } else { 1.0 - smooth((yl - q[1]) / fade) };
                let ax = q[0].abs();
                let wx = smooth((ax - inner * hw) / (0.3 * hw)) * (1.0 - smooth((ax - (hw + 2.0)) / 6.0));
                let wz = smooth((q[2] - (b.base[1] - 14.0)) / 6.0);
                down.map(|c| c * wy * wx * wz / frame.scale)
            })
            .collect()
    }

    /// `sliders nosebase make <data> <out.json>`: the live 16-slider nose set with Wing height added (`NB_LOW`, `NB_HIGH`).
    fn make(args: &[String], data: &Path, input: &Input) {
        let mesh = input.head_mesh();
        let mut set: crate::set::SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/sliders_r2_profile.json")).unwrap()).unwrap();
        let mut s = crate::set::Slider::from_movement("Wing height", "proportion", "mm", &wing_height_movement(&mesh.base, &mesh.triangles));
        s.group = "wings".into();
        s.low = env_or("NB_LOW", -3.0);
        s.high = env_or("NB_HIGH", 6.0);
        s.hint = "Seen from the side, how far the nostril wings reach down. A higher value brings the wings' lower edge down toward the lip, so the wings are taller from top to bottom. The tip does not move.".into();
        set.sliders.push(s);
        std::fs::write(&args[4], serde_json::to_string(&set).unwrap()).unwrap();
        eprintln!("wrote {} with {} sliders", args[4], set.sliders.len());
    }

    /// `sliders nosebase ends <data> <out.png> <head> ...` with `SLIDERS_SET`, `NB_SOLVE` (a solve's output, else every value at 0)
    /// and optionally `NB_VIEWER` (viewer 2's heads with the nose at 0): the slider named by `NB_SLIDER` (Wing height) at its low end,
    /// at the head's value and at its high end, in profile, from below and from three-quarter below, and the folded triangles near the
    /// nose at both ends on all 57 heads.
    fn ends(args: &[String], _data: &Path, input: &Input) {
        use crate::heads::World;
        use crate::set::{SMOOTHING, SliderSet};
        let world = World::load(input, crate::feature::by_key("nose").unwrap());
        let set: SliderSet = serde_json::from_str(&std::fs::read_to_string(std::env::var("SLIDERS_SET").unwrap()).unwrap()).unwrap();
        let solved: Option<serde_json::Value> = std::env::var("NB_SOLVE").ok().map(|p| serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap());
        let viewer: Option<Vec<f32>> = std::env::var("NB_VIEWER").ok().map(|p| read_f32(&p));
        let which = std::env::var("NB_SLIDER").unwrap_or_else(|_| "Wing height".into());
        let j = set.sliders.iter().position(|s| s.name == which).expect("the slider");
        let sl = &set.sliders[j];
        let values_of = |name: &str| -> Vec<f64> {
            let mut v: Vec<f64> = solved.as_ref().and_then(|s| s["heads"][name]["values"].as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap()).collect())).unwrap_or_default();
            v.resize(set.sliders.len(), 0.0);
            v
        };
        let smoothed = crate::solve::Smoothed::new(&set, &world.smoother);
        let vc = world.mesh.base.len();
        let frame = Frame::head();
        let px = 8.0;
        let mut tiles = Vec::new();
        for name in &args[5..] {
            let (i, h) = world.heads.iter().enumerate().find(|(_, h)| &h.name == name).unwrap();
            let vals = values_of(name);
            let ev = crate::solve::Evaluator::with(world.feature, &set, &smoothed, &world.smoother, &h.start, &world.mesh.triangles, h.frame);
            let sh = viewer.as_ref().map(|v| Shown { base: built(v, i, vc), zero: ev.positions(&vec![0.0; set.sliders.len()]) });
            let kb = read(&h.kotor_v, &h.kotor_f).ok();
            let (tip, base) = kb.as_ref().map_or(([-36.0, 48.0], [-54.0, 33.0]), |b| (b.tip, b.base));
            let side = Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [tip[1] - 28.0, tip[1] + 4.0, base[0] - 6.0, base[0] + 26.0] };
            let zc = tip[1] - 10.0;
            let below = Shot { name: "below", yaw: 0.0, pitch: 90.0, pivot: 0.0, window: [-16.0, 16.0, zc - 20.0, zc + 12.0] };
            let (sy, cy) = 25f64.to_radians().sin_cos();
            // the centre of the view, 6 mm above the base and 8 mm behind the tip, turned as the shot turns it
            let (sa, ca) = 40f64.to_radians().sin_cos();
            let (zc2, xc) = (tip[1] - 8.0, (tip[1] - 8.0) * sa);
            let yc = base[0] + cy * 6.0 + sy * zc2 * ca;
            let tq = Shot { name: "tq", yaw: 40.0, pitch: 25.0, pivot: base[0], window: [xc - 16.0, xc + 16.0, yc - 16.0, yc + 16.0] };
            for (tag, x) in [("low", sl.low), ("value", vals[j]), ("high", sl.high)] {
                let mut v = vals.clone();
                v[j] = x;
                let p = match &sh {
                    Some(sh) => shown(&ev, sh, &v),
                    None => set.build(&world.smoother, &h.start, &v),
                };
                let local: Vec<V3> = p.iter().map(|&q| frame.local(q)).collect();
                let keep: Vec<[u32; 3]> = world.mesh.triangles.iter().copied().filter(|t| t.iter().all(|&q| local[q as usize][1] > base[0] - 1.5 && local[q as usize][1] < 5.0)).collect();
                for (k, (shot, tris)) in [(&side, &world.mesh.triangles), (&below, &keep), (&tq, &world.mesh.triangles)].into_iter().enumerate() {
                    let mut img = clay2(&local, tris, shot, px);
                    if k == 0 {
                        label(&mut img, &format!("{} {tag} {x:.1}", name.replace('_', " ")));
                    }
                    tiles.push(img);
                }
            }
        }
        sheet(&tiles, 9).save(&args[4]).unwrap();
        eprintln!("wrote {}", args[4]);
        let window = world.feature.window();
        let counts = crate::heads::parallel(world.heads.len(), crate::heads::threads(), |i| {
            let h = &world.heads[i];
            let before = world.smoother.smooth(&h.start, SMOOTHING);
            let near: Vec<u32> = (0..before.len() as u32).filter(|&q| window.contains(h.frame.local(before[q as usize]))).collect();
            let c = crate::check::Checker::new(&world.mesh, &before, &near);
            let vals = values_of(&h.name);
            [sl.low, vals[j], sl.high].map(|x| {
                let mut v = vals.clone();
                v[j] = x;
                c.faults(&set.build(&world.smoother, &h.start, &v)).total()
            })
        });
        let heads_with = |k: usize| counts.iter().filter(|c| c[k] > 0).count();
        let tris = |k: usize| counts.iter().map(|c| c[k]).sum::<usize>();
        eprintln!("{}: heads with a fold at the low end {} ({} triangles), at the value {} ({}), at the high end {} ({})", sl.name, heads_with(0), tris(0), heads_with(1), tris(1), heads_with(2), tris(2));
    }

    /// `sliders nosebase pairs <data> <out.png> <head> ...` with `SLIDERS_SET` and `NB_SOLVE`: Wing height with each slider named in
    /// `NB_PARTNERS` (comma list) at the four corners of the pair, the folded triangles near the nose on all 57 heads at each corner,
    /// and the corners drawn in profile and from below on the heads named.
    fn pairs(args: &[String], _data: &Path, input: &Input) {
        use crate::heads::World;
        use crate::set::{SMOOTHING, SliderSet};
        let world = World::load(input, crate::feature::by_key("nose").unwrap());
        let set: SliderSet = serde_json::from_str(&std::fs::read_to_string(std::env::var("SLIDERS_SET").unwrap()).unwrap()).unwrap();
        let solved: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(std::env::var("NB_SOLVE").unwrap()).unwrap()).unwrap();
        let values_of = |name: &str| -> Vec<f64> {
            let mut v: Vec<f64> = solved["heads"][name]["values"].as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap()).collect()).unwrap_or_default();
            v.resize(set.sliders.len(), 0.0);
            v
        };
        let j = set.sliders.iter().position(|s| s.name == "Wing height").unwrap();
        let partners: Vec<usize> = std::env::var("NB_PARTNERS").unwrap().split(',').map(|n| set.sliders.iter().position(|s| s.name == n).unwrap_or_else(|| panic!("{n}"))).collect();
        let window = world.feature.window();
        let checkers: Vec<crate::check::Checker> = world
            .heads
            .iter()
            .map(|h| {
                let before = world.smoother.smooth(&h.start, SMOOTHING);
                let near: Vec<u32> = (0..before.len() as u32).filter(|&q| window.contains(h.frame.local(before[q as usize]))).collect();
                crate::check::Checker::new(&world.mesh, &before, &near)
            })
            .collect();
        let frame = Frame::head();
        let px = 7.0;
        let mut tiles = Vec::new();
        for &p in &partners {
            let (a, b) = (&set.sliders[j], &set.sliders[p]);
            let corners = [(a.low, b.low), (a.low, b.high), (a.high, b.low), (a.high, b.high)];
            let counts = crate::heads::parallel(world.heads.len(), crate::heads::threads(), |i| {
                let h = &world.heads[i];
                let vals = values_of(&h.name);
                // the folds Wing height adds to the partner alone at the same value (Wing height at the head's value)
                corners.map(|(x, y)| {
                    let mut v = vals.clone();
                    v[p] = y;
                    let alone = checkers[i].faults(&set.build(&world.smoother, &h.start, &v)).total();
                    v[j] = x;
                    let both = checkers[i].faults(&set.build(&world.smoother, &h.start, &v)).total();
                    [both.saturating_sub(alone), alone]
                })
            });
            for (c, (x, y)) in corners.iter().enumerate() {
                let heads = counts.iter().filter(|k| k[c][0] > 0).count();
                let tris: usize = counts.iter().map(|k| k[c][0]).sum();
                let alone_heads = counts.iter().filter(|k| k[c][1] > 0).count();
                eprintln!("Wing height {x:.1} with {} {y:.1}: Wing height adds folds on {heads} heads ({tris} triangles); the partner alone folds {alone_heads} heads", b.name);
            }
            for name in &args[5..] {
                let h = world.heads.iter().find(|h| &h.name == name).unwrap();
                let vals = values_of(name);
                let kb = read(&h.kotor_v, &h.kotor_f).ok();
                let (tip, base) = kb.as_ref().map_or(([-36.0, 48.0], [-54.0, 33.0]), |b| (b.tip, b.base));
                let side = Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [tip[1] - 28.0, tip[1] + 4.0, base[0] - 6.0, base[0] + 26.0] };
                let zc = tip[1] - 10.0;
                let below = Shot { name: "below", yaw: 0.0, pitch: 90.0, pivot: 0.0, window: [-16.0, 16.0, zc - 20.0, zc + 12.0] };
                for (x, y) in corners {
                    let mut v = vals.clone();
                    v[j] = x;
                    v[p] = y;
                    let pts = set.build(&world.smoother, &h.start, &v);
                    let local: Vec<V3> = pts.iter().map(|&q| frame.local(q)).collect();
                    let keep: Vec<[u32; 3]> = world.mesh.triangles.iter().copied().filter(|t| t.iter().all(|&q| local[q as usize][1] > base[0] - 1.5 && local[q as usize][1] < 5.0)).collect();
                    let mut img = clay2(&local, &world.mesh.triangles, &side, px);
                    label(&mut img, &format!("{} WH {x:.0} {} {y:.1}", name.replace('_', " "), b.name.to_uppercase()));
                    tiles.push(img);
                    tiles.push(clay2(&local, &keep, &below, px));
                }
            }
        }
        sheet(&tiles, 8).save(&args[4]).unwrap();
        eprintln!("wrote {}", args[4]);
    }
}
