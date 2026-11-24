//! The brow fit (paint track, part C, the brows). Analysis only.
//!
//! `brow_fit <out dir> [name...] [--rounds 3]`
//!
//! **The measure.** For each brow, columns every millimetre along the brow's base line (`BrowHead::base_line`), from 14 mm before the
//! inner landmark to 16 mm past the outer one. In each column the tool walks up and down the skin of our fitted head from the base line,
//! every 0.25 mm from 8 mm below it to 16 mm above it, and reads the colour that KOTOR's own head, with its hair meshes, shows from the
//! front at each point. The skin's colour is the median over 10 to 16 mm above the base line. Walking down from the top, the brow starts
//! at the first point darker than the skin by a threshold (a third of the column's contrast, at least 6 L*), and ends where the
//! colour comes back within 70% of the threshold of the skin, or 7 mm below the base line. Scanning down from the forehead finds the brow
//! before the eye's makeup or the socket's shadow, which lie under it. A column holds brow when the band is at least 0.75 mm thick and its
//! core is at least 8 L* darker than the skin. A column where a KOTOR hair mesh lies more than 1 mm in front of the skin is covered and
//! not used, so a fringe over one brow leaves the other brow to set the numbers.
//!
//! **The fit.** The brow's ends are the ends of the longest run of brow columns, allowing gaps of two columns. The middle of the band
//! against the place along the brow is fitted by least squares with the line's five numbers (height, tilt, arch, arch peak and peak
//! angle) for each place of the peak angle from -0.6 to 0.6, keeping the best. The thickness is fitted with its three numbers. The colour
//! is the median of the bands' cores.
//!
//! **The loop.** Our brow is then painted with those numbers and measured the same way, and every number is moved by the difference
//! between KOTOR's measure and ours. This runs `--rounds` times. It writes `<out dir>/fit.json` (the brow controls per head, with the
//! measures) and `<out dir>/measure/<name>.png`, KOTOR's brows with the measured lower edge in green, the upper edge in red and the
//! ends in magenta.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::{Common, KotorFront};
use head_paint::brows::{BrowHead, Brows, SLIDERS};
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint::folds::{Anchors, Landmarks};
use head_paint::lines::BROW_ANCHORS;
use head_paint::live::Layout;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
const N: u32 = 2048;
const STEP: f64 = 0.25;
const BELOW: usize = 32;
const ABOVE: usize = 80;

fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().expect("8"))).collect()
}
fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().expect("4"))).collect()
}
fn layout() -> Layout {
    let l = Path::new(DATA).join("layout");
    let triples = |v: Vec<u32>| v.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect::<Vec<_>>();
    Layout {
        uv: f64s(&l.join("uv.f64")).chunks_exact(2).map(|p| [p[0], p[1]]).collect(),
        triangles: triples(u32s(&l.join("tris.u32"))),
        layout_to_mesh: u32s(&l.join("vmap.u32")).into_iter().map(|i| i as usize).collect(),
        mesh_triangles: triples(u32s(&l.join("mesh_tris.u32"))),
    }
}
fn positions(name: &str) -> Vec<[f64; 3]> {
    f64s(&Path::new(DATA).join("heads").join(format!("{name}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect()
}
fn lab(c: [u8; 3]) -> [f64; 3] {
    head_paint::colour::srgb_to_lab(c.map(|v| f64::from(v) / 255.0))
}
fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// The skin points of one column: index `BELOW + j` is `STEP * j` mm up the skin from the base line, negative j going down.
struct Column {
    side: usize,
    s: f64,
    points: Vec<[f64; 3]>,
    covered: bool,
}

/// What one column reads on a head's front view.
#[derive(Clone, Debug)]
struct Reading {
    present: bool,
    lower: f64,
    upper: f64,
    core: [f64; 3],
    skin: [f64; 3],
}

fn columns(head: &BrowHead, k: &KotorFront, kh: &KotorFront) -> (Vec<Column>, [f64; 2]) {
    let mut cols = Vec::new();
    let mut fulls = [0.0; 2];
    for side in 0..2 {
        let (base, dist, full) = head.base_line(side);
        fulls[side] = full;
        let mut s = -14.0;
        while s <= full + 16.0 {
            let p0 = BrowHead::base_at(&base, &dist, s);
            let p0 = head.on_skin(p0).map_or(p0, |x| x.0);
            let mut up = vec![p0];
            for _ in 0..ABOVE {
                up.push(head.walk_up(*up.last().unwrap(), STEP));
            }
            let mut down = vec![p0];
            for _ in 0..BELOW {
                down.push(head.walk_up(*down.last().unwrap(), -STEP));
            }
            let mut points: Vec<[f64; 3]> = down.into_iter().skip(1).rev().collect();
            points.extend(up);
            let probe = points[BELOW + 8];
            let q = k.project(probe).0;
            let covered = match (k.front_point(q), kh.front_point(q)) {
                (Some(a), Some(b)) => {
                    let f = head.forward;
                    (b[0] - a[0]) * f[0] + (b[1] - a[1]) * f[1] + (b[2] - a[2]) * f[2] > 1.0
                }
                _ => false,
            };
            cols.push(Column { side, s, points, covered });
            s += 1.0;
        }
    }
    (cols, fulls)
}

/// Reads a column. `limit` is the brow's typical depth and thickness from a first pass, or `None` in the first pass. `sign` is 1 for a
/// brow darker than the skin and -1 for a lighter one, such as a white brow on pale skin. The skin's lightness is the column's 80th
/// percentile of lightness (20th for a light brow), which a fringe or a hair mesh over part of the forehead does not move.
fn read(view: &KotorFront, col: &Column, limit: Option<(f64, f64)>, sign: f64) -> Reading {
    let raw: Vec<Option<[f64; 3]>> = col.points.iter().map(|p| view.front_rgb(view.project(*p).0).map(lab)).collect();
    // Smoothed over 5 samples (1.25 mm) along the column, so that single hairs and specks do not break the band.
    let labs: Vec<Option<[f64; 3]>> = (0..raw.len())
        .map(|i| {
            let near: Vec<[f64; 3]> = (i.saturating_sub(2)..(i + 3).min(raw.len())).filter_map(|j| raw[j]).collect();
            raw[i].map(|_| [0, 1, 2].map(|c| near.iter().map(|x| x[c]).sum::<f64>() / near.len() as f64))
        })
        .collect();
    let o = |i: usize| (i as f64 - BELOW as f64) * STEP;
    let none = Reading { present: false, lower: 0.0, upper: 0.0, core: [0.0; 3], skin: [0.0; 3] };
    let mut ls: Vec<f64> = raw[BELOW - 28..].iter().flatten().map(|x| x[0]).collect();
    if ls.len() < 30 {
        return none;
    }
    ls.sort_by(f64::total_cmp);
    let skin_l = if sign > 0.0 { ls[ls.len() * 4 / 5] } else { ls[ls.len() / 5] };
    let near_skin: Vec<[f64; 3]> = raw.iter().flatten().filter(|x| (x[0] - skin_l).abs() < 3.0).copied().collect();
    let skin = [skin_l, median(&mut near_skin.iter().map(|x| x[1]).collect::<Vec<_>>()), median(&mut near_skin.iter().map(|x| x[2]).collect::<Vec<_>>())];
    let depth = |x: &[f64; 3]| sign * (skin_l - x[0]);
    let dmax = labs[BELOW - 24..BELOW + 56].iter().flatten().map(depth).fold(0.0, f64::max);
    let thr = (0.33 * dmax).max(6.0);
    let mut top = None;
    for i in (BELOW - 28..BELOW + 56).rev() {
        if labs[i].as_ref().is_some_and(|x| depth(x) > thr) {
            top = Some(i);
            break;
        }
    }
    let Some(top) = top else { return Reading { skin, ..none } };
    // The band at half its depth: the deepest point within 8 mm under the first deep point, and the stretch around it deeper than half.
    let deepest = limit.map_or(BELOW - 28, |(_, th)| top.saturating_sub((1.8 * th / STEP) as usize).max(BELOW - 28));
    let reach = top.saturating_sub(32).max(deepest);
    let (imin, dpk) = (reach..=top).filter_map(|i| labs[i].as_ref().map(|x| (i, depth(x)))).max_by(|a, b| a.1.total_cmp(&b.1)).unwrap_or((top, 0.0));
    let half = 0.5 * dpk;
    let mut top = imin;
    while top + 1 < labs.len() && labs[top + 1].as_ref().is_some_and(|x| depth(x) > half) {
        top += 1;
    }
    let mut bottom = imin;
    while bottom > deepest && labs[bottom - 1].as_ref().is_some_and(|x| depth(x) > half) {
        bottom -= 1;
    }
    let (lower, upper) = (o(bottom) - 0.5 * STEP, o(top) + 0.5 * STEP);
    let n = top - bottom + 1;
    let inner: Vec<[f64; 3]> = labs[bottom + n / 5..=top - n / 5].iter().flatten().copied().collect();
    if inner.is_empty() {
        return Reading { skin, ..none };
    }
    let core = [0, 1, 2].map(|c| median(&mut inner.iter().map(|x| x[c]).collect::<Vec<_>>()));
    let d = depth(&core);
    let present = upper - lower >= 0.75 && d >= limit.map_or(8.0, |(dm, _)| (0.5 * dm).max(8.0));
    Reading { present, lower, upper, core, skin }
}

/// Reads every column in two passes: the second keeps only columns at least 45% as dark as the middle of the brow, and lets the band
/// reach at most 1.8 times the brow's typical thickness below its upper edge.
fn read_all(view: &KotorFront, cols: &[Column], fulls: &[f64; 2], sign: Option<f64>) -> (Vec<Reading>, f64) {
    let middle = |c: &Column| !c.covered && c.s > 0.2 * fulls[c.side] && c.s < 0.8 * fulls[c.side];
    // Every brow is read as darker than the skin. A light reading was tried and picked the wrong heads, so white brows are set by hand.
    let sg = sign.unwrap_or(1.0);
    let first: Vec<Reading> = cols.iter().map(|c| read(view, c, None, sg)).collect();
    let mid: Vec<&Reading> = cols.iter().zip(&first).filter(|(c, r)| r.present && middle(c)).map(|(_, r)| r).collect();
    if mid.len() < 4 {
        return (first, sg);
    }
    let d = median(&mut mid.iter().map(|r| (r.skin[0] - r.core[0]).abs()).collect::<Vec<_>>());
    let th = median(&mut mid.iter().map(|r| r.upper - r.lower).collect::<Vec<_>>());
    (cols.iter().map(|c| read(view, c, Some((d, th)), sg)).collect(), sg)
}

/// The numbers a set of readings gives, in the units of the brow sliders, with the colour as Lab.
#[derive(Clone, Copy, Debug, Default)]
struct Fit {
    inner: f64,
    outer: f64,
    line: [f64; 5],
    corner_at: f64,
    thick: [f64; 3],
    core: [f64; 3],
    columns: usize,
    residual: f64,
}

fn tent(t: f64, c: f64) -> f64 {
    (if t < c { (t + 1.0) / (c + 1.0) } else { (1.0 - t) / (1.0 - c) }).clamp(0.0, 1.0)
}

fn solve(a: &mut [Vec<f64>], b: &mut [f64]) -> Vec<f64> {
    let n = b.len();
    for i in 0..n {
        let p = (i..n).max_by(|&x, &y| a[x][i].abs().total_cmp(&a[y][i].abs())).unwrap();
        a.swap(i, p);
        b.swap(i, p);
        for r in i + 1..n {
            let f = a[r][i] / a[i][i];
            for c in i..n {
                a[r][c] -= f * a[i][c];
            }
            b[r] -= f * b[i];
        }
    }
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        x[i] = (b[i] - (i + 1..n).map(|c| a[i][c] * x[c]).sum::<f64>()) / a[i][i];
    }
    x
}

/// Weighted least squares with a small ridge on every number but the first.
fn lsq(rows: &[(Vec<f64>, f64, f64)], ridge: f64) -> (Vec<f64>, f64) {
    let n = rows[0].0.len();
    let mut a = vec![vec![0.0; n]; n];
    let mut b = vec![0.0; n];
    for (x, y, w) in rows {
        for i in 0..n {
            b[i] += w * x[i] * y;
            for j in 0..n {
                a[i][j] += w * x[i] * x[j];
            }
        }
    }
    for (i, row) in a.iter_mut().enumerate().skip(1) {
        row[i] += ridge;
    }
    let x = solve(&mut a, &mut b);
    let res = rows.iter().map(|(xs, y, w)| w * (xs.iter().zip(&x).map(|(p, q)| p * q).sum::<f64>() - y).powi(2)).sum::<f64>() / rows.iter().map(|r| r.2).sum::<f64>();
    (x, res.sqrt())
}

fn fit(cols: &[Column], reads: &[Reading], fulls: [f64; 2]) -> Option<Fit> {
    // The ends, per side, from the longest run of brow columns.
    let mut ends = Vec::new();
    for side in 0..2 {
        let idx: Vec<usize> = (0..cols.len()).filter(|&i| cols[i].side == side).collect();
        if idx.iter().filter(|&&i| cols[i].covered).count() > idx.len() / 3 {
            continue;
        }
        let mut best: Option<(usize, usize)> = None;
        let mut start: Option<usize> = None;
        let mut gap = 0;
        let mut last = 0;
        for (j, &i) in idx.iter().enumerate() {
            if reads[i].present && !cols[i].covered {
                if start.is_none() {
                    start = Some(j);
                }
                last = j;
                gap = 0;
            } else if start.is_some() {
                gap += 1;
                if gap > 2 {
                    let run = (start.unwrap(), last);
                    if best.is_none_or(|b| run.1 - run.0 > b.1 - b.0) {
                        best = Some(run);
                    }
                    start = None;
                }
            }
        }
        if let Some(s0) = start {
            let run = (s0, last);
            if best.is_none_or(|b| run.1 - run.0 > b.1 - b.0) {
                best = Some(run);
            }
        }
        if let Some((a, b)) = best {
            if b - a >= 8 {
                ends.push((side, cols[idx[a]].s - 0.5, cols[idx[b]].s + 0.5));
            }
        }
    }
    if ends.is_empty() {
        return None;
    }
    let inner = ends.iter().map(|e| -e.1).sum::<f64>() / ends.len() as f64;
    let outer = ends.iter().map(|e| e.2 - fulls[e.0]).sum::<f64>() / ends.len() as f64;
    let sides: Vec<usize> = ends.iter().map(|e| e.0).collect();
    let mut rows_line = Vec::new();
    let mut rows_thick = Vec::new();
    let mut cores = Vec::new();
    for (c, r) in cols.iter().zip(reads) {
        if !sides.contains(&c.side) || c.covered || !r.present {
            continue;
        }
        let (start, end) = (-inner, fulls[c.side] + outer);
        let t = 2.0 * (c.s - start) / (end - start) - 1.0;
        if !(-1.0..=1.0).contains(&t) {
            continue;
        }
        let w = (r.skin[0] - r.core[0]).abs().max(1.0);
        rows_line.push((t, 0.5 * (r.lower + r.upper), w));
        rows_thick.push((vec![1.0, 0.5 * t, -t * t], r.upper - r.lower, w));
        cores.push(r.core);
    }
    if rows_line.len() < 8 {
        return None;
    }
    let mut best: Option<(Vec<f64>, f64, f64)> = None;
    for k in -3..=6 {
        let c = 0.1 * f64::from(k);
        let rows: Vec<(Vec<f64>, f64, f64)> = rows_line.iter().map(|&(t, y, w)| (vec![1.0, 0.5 * t, 1.0 - t * t, 1.3 * t * (1.0 - t * t), tent(t, c)], y, w)).collect();
        let mean_w = rows.iter().map(|r| r.2).sum::<f64>() / rows.len() as f64;
        let (x0, _) = lsq(&rows, 3.0 * mean_w);
        // Robust: a column more than 1 mm off the first fit counts less, as 1 / (1 + (miss / 1 mm)^2), so that a tail that drops into
        // the socket's shadow does not bend the line.
        let rows: Vec<(Vec<f64>, f64, f64)> = rows
            .into_iter()
            .map(|(xs, y, w)| {
                let miss = xs.iter().zip(&x0).map(|(p, q)| p * q).sum::<f64>() - y;
                (xs, y, w / (1.0 + miss * miss))
            })
            .collect();
        let (x, res) = lsq(&rows, 3.0 * mean_w);
        // The angle must earn its place: its fit is kept only when it lowers the residual by more than 0.05 mm.
        let res = res + if x[4].abs() > 0.5 { 0.05 } else { 0.0 };
        if best.as_ref().is_none_or(|b| res < b.1 - 1e-9) {
            best = Some((x, res, c));
        }
    }
    let (line, residual, corner_at) = best.unwrap();
    let (thick, _) = lsq(&rows_thick, 0.05 * rows_thick.iter().map(|r| r.2).sum::<f64>() / rows_thick.len() as f64);
    let core = [0, 1, 2].map(|k| median(&mut cores.iter().map(|x| x[k]).collect::<Vec<_>>()));
    let inner = inner.clamp(-6.0, 8.0);
    let outer = outer.clamp(-6.0, 10.0);
    Some(Fit { inner, outer, line: [line[0], line[1], line[2].clamp(-4.0, 6.0), line[3], line[4].clamp(-2.5, 5.0)], corner_at, thick: [thick[0], thick[1], thick[2]], core, columns: rows_line.len(), residual })
}

fn clamp_slider(id: &str, v: f64) -> f64 {
    let s = SLIDERS.iter().find(|s| s.id == id).expect("slider");
    v.clamp(s.min, s.max)
}

/// Applies a fit's differences to the brow numbers. `start` true takes the fit as it is.
fn apply(b: &mut Brows, k: &Fit, o: Option<&Fit>) {
    let d = |a: f64, b: f64| a - b;
    match o {
        None => {
            b.inner_mm = k.inner;
            b.outer_mm = k.outer;
            b.height_mm = k.line[0];
            b.tilt_mm = k.line[1];
            b.arch_mm = k.line[2];
            b.peak_mm = k.line[3];
            b.corner_mm = k.line[4];
            b.corner_at = k.corner_at;
            b.thick_mm = k.thick[0];
            b.taper_mm = k.thick[1];
            b.belly_mm = k.thick[2];
            b.lightness = k.core[0];
            b.chroma = (k.core[1].powi(2) + k.core[2].powi(2)).sqrt().min(25.0);
            b.fill = 0.85;
            // Grooming from the brow's thickness: a thin brow has short, tidy hairs and a clean edge, a thick one longer, looser hairs.
            let t = k.thick[0];
            b.spill_mm = (0.12 * t).clamp(0.15, 1.2);
            b.length_mm = (2.0 + 0.25 * t).clamp(2.4, 4.8);
            b.messy = (0.15 + 0.04 * t).clamp(0.2, 0.55);
            b.trim = if t < 5.0 { 0.8 } else { 0.3 };
            b.hue = k.core[2].atan2(k.core[1]).to_degrees().rem_euclid(360.0);
        }
        Some(o) => {
            b.inner_mm += d(k.inner, o.inner);
            b.outer_mm += d(k.outer, o.outer);
            b.height_mm += 0.8 * d(k.line[0], o.line[0]);
            b.tilt_mm += 0.8 * d(k.line[1], o.line[1]);
            b.arch_mm += 0.8 * d(k.line[2], o.line[2]);
            b.peak_mm += 0.8 * d(k.line[3], o.line[3]);
            b.corner_mm += 0.8 * d(k.line[4], o.line[4]);
            b.thick_mm = (b.thick_mm + 0.8 * d(k.thick[0], o.thick[0])).max(0.6 * k.thick[0]);
            b.taper_mm += 0.8 * d(k.thick[1], o.thick[1]);
            b.belly_mm += 0.8 * d(k.thick[2], o.thick[2]);
            b.lightness += 0.9 * d(k.core[0], o.core[0]);
            // At the colour's floor, a brow that is still too light gets more fill and more hairs.
            if b.lightness < 6.0 && o.core[0] > k.core[0] + 3.0 {
                b.fill = (b.fill + 0.12).min(1.0);
                b.density = (b.density + 0.05).min(1.0);
            }
            let h = b.hue.to_radians();
            let (a0, b0) = (b.chroma * h.cos() + 0.5 * d(k.core[1], o.core[1]), b.chroma * h.sin() + 0.5 * d(k.core[2], o.core[2]));
            b.chroma = (a0 * a0 + b0 * b0).sqrt().min(25.0);
            b.hue = b0.atan2(a0).to_degrees().rem_euclid(360.0);
        }
    }
    // Brow hair is brown, black, grey or blond: the measured core takes some of the skin's tint, so the hue is kept between 30 and 90
    // degrees and the colour strength under 18.
    b.hue = if b.hue > 270.0 { 30.0 } else { b.hue.clamp(30.0, 90.0) };
    b.chroma = b.chroma.min(18.0);
    // The loop may not take the shape past what the measure trusts.
    b.inner_mm = b.inner_mm.clamp(-6.0, 10.0);
    b.outer_mm = b.outer_mm.clamp(-8.0, 12.0);
    b.corner_mm = b.corner_mm.clamp(-3.0, 6.0);
    b.arch_mm = b.arch_mm.clamp(-4.5, 7.0);
    for (k, s) in SLIDERS.iter().enumerate() {
        let v = clamp_slider(s.id, b.values()[k]);
        *b.field(k) = v;
    }
}

fn brow_json(b: &Brows) -> serde_json::Map<String, Value> {
    SLIDERS.iter().zip(b.values()).map(|(s, v)| (format!("brow_{}", s.id), json!((v * 1000.0).round() / 1000.0))).collect()
}

fn fit_json(f: &Fit) -> Value {
    let r = |v: f64| (v * 100.0).round() / 100.0;
    json!({ "inner": r(f.inner), "outer": r(f.outer), "line": f.line.map(r), "corner_at": r(f.corner_at), "thick": f.thick.map(r), "core_lab": f.core.map(r), "columns": f.columns, "residual_mm": r(f.residual) })
}

fn mark(img: &mut image::RgbImage, xy: (f64, f64), c: [u8; 3]) {
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            let (x, y) = (xy.0 as i32 + dx, xy.1 as i32 + dy);
            if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                img.put_pixel(x as u32, y as u32, image::Rgb(c));
            }
        }
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let rounds: usize = args.iter().position(|a| a == "--rounds").map_or(3, |k| {
        let v = args.remove(k + 1).parse().expect("rounds");
        args.remove(k);
        v
    });
    let out = PathBuf::from(&args[0]);
    let names: Vec<String> = if args.len() > 1 { args[1..].to_vec() } else { fold_common::head_names() };
    std::fs::create_dir_all(out.join("measure")).expect("out");
    let lay = layout();
    let common = Common::load();
    let mut painter = GpuPainter::new(Gpu::blocking().expect("GPU"), layout());
    let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
    let list = controls();
    let switches = vec![true; head_paint::controls::GROUPS.len()];
    let mut report = serde_json::Map::new();
    for name in &names {
        let pos = positions(name);
        let marks = Landmarks::new(&pos);
        let f = marks.frame;
        let head = BrowHead::new(&pos, &lay.triangles, &lay.layout_to_mesh).expect("brow head");
        let k = KotorFront::open(name, f.across, f.up, f.forward);
        let kh = KotorFront::open_with_hair(name, f.across, f.up, f.forward);
        let (cols, fulls) = columns(&head, &k, &kh);
        let (kreads, sign) = read_all(&kh, &cols, &fulls, None);

        // The measure drawn on KOTOR's brows, framed as the wall.
        let mid = Anchors::point(&BROW_ANCHORS.brow_middle, &pos);
        let pk = |s: usize| Anchors::point(&BROW_ANCHORS.brow_peak[s], &pos);
        let centre3 = [0, 1, 2].map(|i| 0.5 * mid[i] + 0.25 * (pk(0)[i] + pk(1)[i]));
        let cq = kh.project(centre3).0;
        let c = [cq[0], cq[1] - 2.0];
        let mut img = kh.tile_at(c, 130.0, 50.0, 6.0);
        let to_px = |p: [f64; 3]| {
            let q = kh.project(p).0;
            ((q[0] - (c[0] - 65.0)) * 6.0, ((c[1] + 25.0) - q[1]) * 6.0)
        };
        for (col, r) in cols.iter().zip(&kreads) {
            if col.covered {
                mark(&mut img, to_px(col.points[BELOW + 8]), [255, 255, 0]);
                continue;
            }
            if r.present {
                let at = |o: f64| col.points[(BELOW as f64 + o / STEP).round().clamp(0.0, (col.points.len() - 1) as f64) as usize];
                mark(&mut img, to_px(at(r.lower)), [0, 220, 0]);
                mark(&mut img, to_px(at(r.upper)), [255, 40, 40]);
            }
        }
        let kfit = fit(&cols, &kreads, fulls);
        if let Some(kf) = &kfit {
            for side in 0..2 {
                let (base, dist, full) = head.base_line(side);
                for s in [-kf.inner, full + kf.outer] {
                    let p = BrowHead::base_at(&base, &dist, s);
                    for dy in -6..=6 {
                        mark(&mut img, to_px(head.walk_up(p, f64::from(dy))), [255, 0, 255]);
                    }
                }
            }
        }
        img.save(out.join("measure").join(format!("{name}.png"))).expect("save");

        let Some(kf) = kfit else {
            println!("{name}: no brow could be read; it keeps the defaults");
            report.insert(name.clone(), json!({ "readable": false }));
            continue;
        };
        // The loop: paint ours, measure it, move the numbers by the difference.
        let base_values: Vec<f64> = list.iter().map(|c| recipes["heads"][name.as_str()][&c.id].as_f64().unwrap_or(c.default)).collect();
        let mut b = Brows::DEFAULT;
        apply(&mut b, &kf, None);
        if kf.core[0] > 45.0 && (kf.core[1].powi(2) + kf.core[2].powi(2)).sqrt() < 8.0 {
            b.grey = 0.3;
        }
        painter.set_head(pos.clone());
        pollster::block_on(painter.prepare(N));
        let mut history = Vec::new();
        for round in 0..=rounds + 1 {
            let mut recipe = recipe_from(&base_values);
            recipe.brows = b;
            painter.paint(N, &recipe, &layers_from(&switches));
            let rgba = pollster::block_on(painter.read_rgba(N));
            let tex = image::RgbImage::from_fn(N, N, |x, y| {
                let at = 4 * (y * N + x) as usize;
                image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
            });
            let ours = KotorFront::ours(&common, &pos, tex, f.across, f.up, f.forward);
            let (oreads, _) = read_all(&ours, &cols, &fulls, Some(sign));
            let Some(of) = fit(&cols, &oreads, fulls) else {
                history.push(json!({ "round": round, "ours": null }));
                break;
            };
            history.push(json!({ "round": round, "ours": fit_json(&of) }));
            if round == rounds + 1 {
                break;
            }
            apply(&mut b, &kf, Some(&of));
        }
        // The measured core still holds some skin between the hairs, so dark brows come out a little light and tinted: they are made 6 L*
        // darker and a quarter less colourful.
        if b.lightness < 45.0 {
            b.lightness = (b.lightness - 6.0).max(3.0);
        }
        b.chroma *= 0.75;
        println!("{name}: {} columns, residual {:.2} mm, lightness {:.0}, thickness {:.1} mm", kf.columns, kf.residual, b.lightness, b.thick_mm);
        report.insert(name.clone(), json!({ "readable": true, "kotor": fit_json(&kf), "rounds": history, "values": brow_json(&b), "covered_columns": cols.iter().filter(|c| c.covered).count(), "light_brow": sign < 0.0 }));
    }
    std::fs::write(out.join("fit.json"), serde_json::to_string_pretty(&Value::Object(report)).unwrap()).expect("write");
}
