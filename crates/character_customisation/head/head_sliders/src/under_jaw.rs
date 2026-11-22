//! The underside of the jaw and the jaw's corner, measured on any triangle mesh in the frame of the eyes (`measure::Frame`).
//!
//! This is a measure only. It has no sliders yet: it serves to see what the jaw and chin sliders cannot reach under the jaw.

use crate::V3;
use crate::feature::{KotorExtras, Measured, OutlineDef, Shot};
use crate::measure::{Cut, Frame, Proportion, STEP, Surface, Window, resample};
use serde_json::json;

/// The pictures of the area: from the side, from in front, three-quarter from below, and from below.
pub const SHOTS: [Shot; 4] = [
    Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-125.0, 55.0, -190.0, -40.0] },
    Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-80.0, 80.0, -185.0, -45.0] },
    Shot { name: "three-quarter below", yaw: 50.0, pitch: 25.0, pivot: -100.0, window: [-90.0, 70.0, -185.0, -45.0] },
    Shot { name: "below", yaw: 0.0, pitch: 70.0, pivot: -110.0, window: [-80.0, 80.0, -185.0, -45.0] },
];

/// KOTOR's own triangles without the hair and cloth over the face (`jaw::cover`), in the frame of the eyes.
pub fn bare_kotor(extras: &KotorExtras, frame: &Frame, positions: &[V3]) -> Vec<[u32; 3]> {
    let triangles = extras.triangles.expect("KOTOR's triangles");
    let covered = crate::jaw::cover(extras, frame, positions, triangles, extras.fit_triangles);
    triangles.iter().zip(&covered).filter(|(_, c)| !**c).map(|(t, _)| *t).collect()
}

/// The window the measure reads.
pub const WINDOW: Window = Window { x: 90.0, y: [-185.0, -40.0], z: -120.0 };

pub const PROPORTIONS: [Proportion; 12] = [
    Proportion { key: "throat_angle", label: "Angle between the line under the chin and the front of the neck, seen from the side", low: "sharp throat corner", high: "open throat corner", unit: "degrees" },
    Proportion { key: "throat_back", label: "How far the throat's corner lies behind the front of the chin", low: "short underside of the chin", high: "long underside of the chin", unit: "mm" },
    Proportion { key: "throat_drop", label: "How far the throat's corner lies below the bottom of the chin", low: "level under the chin", high: "dropping under the chin", unit: "mm" },
    Proportion { key: "neck_slope", label: "Slope of the front of the neck from upright, seen from the side", low: "neck leaning back", high: "neck leaning forward", unit: "degrees" },
    Proportion { key: "corner_back", label: "How far the jaw's corner lies behind the front of the chin", low: "short jaw", high: "long jaw", unit: "mm" },
    Proportion { key: "corner_height", label: "How high the jaw's corner lies above the bottom of the chin", low: "low jaw corner", high: "high jaw corner", unit: "mm" },
    Proportion { key: "edge_mid", label: "How high the middle third of the line under the jaw lies above the bottom of the chin", low: "low line under the jaw", high: "high line under the jaw", unit: "mm" },
    Proportion { key: "ledge", label: "How far the jaw's lower edge stands out over what lies under it, seen from the front, on the back half of the line", low: "jaw fading into the neck", high: "jaw standing out over the neck", unit: "mm" },
    Proportion { key: "edge_width", label: "Width between the jaw's lower edges on the back third of the line", low: "narrow jaw from below", high: "wide jaw from below", unit: "mm" },
    Proportion { key: "neck_width", label: "Width of the neck 24 mm below the chin, over its front 35 mm", low: "thin neck", high: "thick neck", unit: "mm" },
    Proportion { key: "edge_back_height", label: "How high the jaw's lower edge lies above the bottom of the chin, 56 to 72 mm behind the chin's front", low: "line under the jaw dropping toward the back", high: "line under the jaw rising toward the back", unit: "mm" },
    Proportion { key: "jaw_over_neck", label: "Width between the jaw's lower edges less the width of the neck", low: "jaw as wide as the neck", high: "jaw wider than the neck", unit: "mm" },
];

pub const OUTLINES: [OutlineDef; 3] = [
    OutlineDef { key: "edge", label: "Lower edge of the jaw from the side", points: EDGE_POINTS, widths: false },
    OutlineDef { key: "below", label: "Lower edge of the jaw from below", points: EDGE_POINTS, widths: false },
    OutlineDef { key: "throat", label: "Line from the chin into the throat, from the side", points: THROAT_RAYS, widths: false },
];

/// The cross-sections that read the jaw's edge: every 4 mm from 8 mm behind the chin's front, this many.
const SLICES: usize = 24;
/// The rays of a cross-section, every 1.5 degrees from straight out to straight down.
const EDGE_POINTS: usize = 12;
const THROAT_RAYS: usize = 19;

/// The cross-section of a surface by the plane at depth `z`, as segments (across, height).
fn slice(triangles: &[[V3; 3]], z: f64) -> Vec<[[f64; 2]; 2]> {
    let mut out = Vec::new();
    for t in triangles {
        let d = t.map(|p| if p[2] == z { 1e-9 } else { p[2] - z });
        let mut ends = Vec::with_capacity(2);
        for (i, j) in [(0, 1), (1, 2), (2, 0)] {
            if (d[i] > 0.0) != (d[j] > 0.0) {
                let s = d[i] / (d[i] - d[j]);
                ends.push([t[i][0] + s * (t[j][0] - t[i][0]), t[i][1] + s * (t[j][1] - t[i][1])]);
            }
        }
        if ends.len() == 2 {
            out.push([ends[0], ends[1]]);
        }
    }
    out
}

/// The outermost place of a cross-section at each height, on one side (`side` 1 for +x, made positive), every `DY` mm from `top`
/// down, `count` heights, NaN where the section has nothing on that side.
fn outermost(segs: &[[[f64; 2]; 2]], side: f64, top: f64, count: usize) -> Vec<f64> {
    let mut x = vec![f64::NAN; count];
    for [a, b] in segs {
        let (a, b) = ([side * a[0], a[1]], [side * b[0], b[1]]);
        if a[0] < 0.0 && b[0] < 0.0 {
            continue;
        }
        let (lo, hi) = if a[1] < b[1] { (a, b) } else { (b, a) };
        let k0 = ((top - hi[1]) / DY).ceil().max(0.0) as usize;
        let k1 = (((top - lo[1]) / DY).floor() as isize).min(count as isize - 1);
        if k1 < 0 {
            continue;
        }
        for (k, xk) in x.iter_mut().enumerate().take(k1 as usize + 1).skip(k0) {
            let y = top - k as f64 * DY;
            let f = if hi[1] > lo[1] { (y - lo[1]) / (hi[1] - lo[1]) } else { 0.0 };
            let v = lo[0] + f * (hi[0] - lo[0]);
            if v > 0.0 && (xk.is_nan() || v > *xk) {
                *xk = v;
            }
        }
    }
    x
}

/// The heights of a cross-section's outline, every this many mm.
const DY: f64 = 0.5;

/// The jaw's edge in one cross-section on one side, from the section's outermost place at each height (the section seen from the
/// front): the first place, going down, where that outline steps inward faster than it goes down, over 2 mm. Returns the edge (across, height) and how far the outline
/// steps in from 3 mm above the edge to 6 mm below it (the jaw's ledge over what lies under it), or to its end when the section ends
/// first (the jaw seen against nothing, near the chin).
fn slice_edge(segs: &[[[f64; 2]; 2]], side: f64, top: f64, bottom: f64) -> Option<([f64; 2], f64)> {
    let count = ((top - bottom) / DY) as usize + 1;
    let x = outermost(segs, side, top, count);
    // the section's last height with surface, going down from the top without a gap
    let first = x.iter().position(|v| !v.is_nan())?;
    let mut last = first;
    while last + 1 < count && !x[last + 1].is_nan() {
        last += 1;
    }
    if last - first < 12 {
        return None;
    }
    // the first height, going down, where the outline steps in faster than it goes down (the skin faces more down than out), over 2 mm
    let h = 2usize;
    let k = (first + h..=last.saturating_sub(h)).find(|&k| x[k - h] - x[k + h] > 2.0 * h as f64 * DY)?;
    // an edge at the top of the window is the lower lip or the cheek above it, not the jaw
    if k < first + 4 {
        return None;
    }
    let ledge = x[(k.saturating_sub(6)).max(first)] - x[(k + 12).min(last)];
    Some(([x[k], top - k as f64 * DY], ledge))
}

fn get_found(found: &[Option<(([f64; 2], f64), ([f64; 2], f64))>], k: usize) -> ([f64; 2], f64) {
    let (a, b) = found[k].expect("a section next to a missing one");
    ([0.5 * (a.0[0] + b.0[0]), 0.5 * (a.0[1] + b.0[1])], 0.5 * (a.1 + b.1))
}

/// How far above the chin's bottom the line under the jaw may rise before it counts as the jaw's back edge, in mm.
const RISE: f64 = 45.0;
/// How far above the chin's bottom the cross-sections are read from, in mm: below the lower lip.
const TOP: f64 = 24.0;
/// How far below the chin's bottom the front of the neck must reach for the throat's rows to be read, in mm.
const NECK_BELOW: f64 = 34.0;
/// How far above the last section's edge the next section is read from, in mm.
const FOLLOW: f64 = 12.0;
/// How far apart in height the edges of the two sides of a section may lie before the section cannot be read, in mm.
const SIDES_APART: f64 = 8.0;

/// Measures the underside of the jaw and the jaw's corner on a surface, or says what could not be found.
pub fn measure(surface: &Surface) -> Result<Measured, &'static str> {
    // the profile down the middle: the chin's front and bottom
    let from = -200.0;
    let mid = Cut { from, z: surface.front(0, 0.0, 1, from, 800) };
    let (pk, pogonion) = (mid.index(-125.0)..=mid.index(-85.0))
        .filter(|&k| !mid.z[k].is_nan())
        .map(|k| (k, mid.z[k]))
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .ok_or("no front of the chin")?;
    let ufrom = pogonion - 60.0;
    let under = crate::jaw::underside(surface, ufrom, 241);
    let chin_bottom = (0..under.len())
        .filter(|&k| ufrom + k as f64 * STEP > pogonion - 25.0 && !under[k].is_nan())
        .map(|k| under[k])
        .fold(f64::INFINITY, f64::min);
    if !chin_bottom.is_finite() {
        return Err("no bottom of the chin");
    }
    // the line under the chin: the underside from 8 to 20 mm behind the chin's front
    let under_pts: Vec<[f64; 2]> = (0..under.len())
        .map(|k| (ufrom + k as f64 * STEP, under[k]))
        .filter(|(z, y)| !y.is_nan() && *z <= pogonion - 8.0 && *z >= pogonion - 20.0)
        .map(|(z, y)| [z, y])
        .collect();
    if under_pts.len() < 20 {
        return Err("no underside of the chin");
    }
    // the front of the neck: the front-most surface down the middle from 20 to 32 mm below the chin's bottom; KOTOR's shortest
    // necks do not reach so far, and then the throat's rows cannot be read
    let neck_pts: Vec<[f64; 2]> = (mid.index(chin_bottom - 32.0)..=mid.index(chin_bottom - 20.0))
        .filter(|&k| !mid.z[k].is_nan())
        .map(|k| [mid.z[k], mid.u(k)])
        .collect();
    let line = |p: &[[f64; 2]]| -> ([f64; 2], [f64; 2]) {
        // a point and a unit direction, by principal axis
        let n = p.len() as f64;
        let m = [p.iter().map(|q| q[0]).sum::<f64>() / n, p.iter().map(|q| q[1]).sum::<f64>() / n];
        let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
        for q in p {
            let (a, b) = (q[0] - m[0], q[1] - m[1]);
            sxx += a * a;
            sxy += a * b;
            syy += b * b;
        }
        let t = 0.5 * (2.0 * sxy).atan2(sxx - syy);
        (m, [t.cos(), t.sin()])
    };
    let (um, mut ud) = line(&under_pts);
    if ud[0] > 0.0 {
        ud = [-ud[0], -ud[1]];
    }
    // the throat's angle: between going back along the line under the chin and going down the line of the neck's front. Where the
    // two lines meet at more than 20 degrees from straight, their meeting place is the throat's corner.
    // the neck must reach NECK_BELOW below the chin's bottom without a gap, or the line of its front bends with the stub's end
    let neck_long = (mid.index(chin_bottom - NECK_BELOW)..=mid.index(chin_bottom - 20.0)).all(|k| !mid.z[k].is_nan());
    let (throat_angle, neck_slope, corner) = if neck_pts.len() >= 40 && neck_long {
        let (nm, mut nd) = line(&neck_pts);
        if nd[1] > 0.0 {
            nd = [-nd[0], -nd[1]];
        }
        let angle = ((-ud[0]) * nd[0] + (-ud[1]) * nd[1]).clamp(-1.0, 1.0).acos().to_degrees();
        let corner = if angle < 160.0 {
            let det = ud[0] * (-nd[1]) - ud[1] * (-nd[0]);
            let (bx, by) = (nm[0] - um[0], nm[1] - um[1]);
            let s = (bx * (-nd[1]) - by * (-nd[0])) / det;
            [um[0] + s * ud[0], um[1] + s * ud[1]]
        } else {
            [f64::NAN; 2]
        };
        (angle, nd[0].atan2(-nd[1]).to_degrees(), corner)
    } else {
        (f64::NAN, f64::NAN, [f64::NAN; 2])
    };
    let (throat_back, throat_drop) = (pogonion - corner[0], chin_bottom - corner[1]);
    // the line from the chin into the throat: the farthest surface down the middle along rays from a place 35 mm behind the chin's
    // front and 15 mm above its bottom, every 5 degrees from 60 degrees forward of straight down to 30 degrees back of it
    let centre = [pogonion - 35.0, chin_bottom + 15.0];
    let segs: Vec<[[f64; 2]; 2]> = surface.cut(0, 0.0, 1).into_iter().map(|[a, b]| [[a[1], a[0]], [b[1], b[0]]]).collect();
    let mut throat = Vec::with_capacity(THROAT_RAYS);
    for k in 0..THROAT_RAYS {
        let a = (60.0 - 5.0 * k as f64).to_radians();
        let d = [a.sin(), -a.cos()];
        let mut far: Option<f64> = None;
        for [p, q] in &segs {
            // the ray centre + t d against the segment p + s (q - p)
            let e = [q[0] - p[0], q[1] - p[1]];
            let den = d[0] * (-e[1]) - d[1] * (-e[0]);
            if den.abs() < 1e-12 {
                continue;
            }
            let (bx, by) = (p[0] - centre[0], p[1] - centre[1]);
            let t = (bx * (-e[1]) - by * (-e[0])) / den;
            let s = (d[0] * by - d[1] * bx) / den;
            if (0.0..=1.0).contains(&s) && t > 0.0 && t < 70.0 && far.is_none_or(|f| t > f) {
                far = Some(t);
            }
        }
        match far {
            Some(t) => throat.push([centre[0] + t * d[0], centre[1] + t * d[1]]),
            None => {
                throat.clear();
                break;
            }
        }
    }
    // the jaw's edge in cross-sections every 4 mm from 8 mm behind the chin's front, both sides averaged, read down to 45 mm below the
    // chin's bottom. The line under the jaw ends at the jaw's corner: at the last section, going back, before two sections in a row
    // cannot be read, or the edge rises more than RISE above the chin's bottom. A single section that cannot be read takes its place between its neighbours.
    // Each section is read from TOP above the chin's bottom, or from FOLLOW above the edge of the last section read, whichever is
    // higher, so the window rises with the edge toward the corner without reaching the lower lip near the chin.
    let mut found: Vec<Option<(([f64; 2], f64), ([f64; 2], f64))>> = Vec::with_capacity(SLICES);
    let mut last_edge: Option<f64> = None;
    for k in 0..SLICES {
        let z = pogonion - 8.0 - 4.0 * k as f64;
        let segs = slice(&surface.triangles, z);
        let (top, bottom) = ((chin_bottom + TOP).max(last_edge.unwrap_or(chin_bottom) + FOLLOW).min(chin_bottom + RISE + FOLLOW), chin_bottom - 45.0);
        let f = slice_edge(&segs, 1.0, top, bottom)
            .zip(slice_edge(&segs, -1.0, top, bottom))
            .filter(|(a, b)| 0.5 * (a.0[1] + b.0[1]) < chin_bottom + RISE)
            .filter(|(a, b)| (a.0[1] - b.0[1]).abs() < SIDES_APART);
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var("UJ_DEBUG").is_ok() {
            let r = |side| slice_edge(&segs, side, top, bottom).map(|(p, l)| format!("({:.1},{:.1} l{:.1})", p[0], p[1] - chin_bottom, l));
            eprintln!("slice {k} z {z:.1} top {:.1}: right {:?} left {:?}", top - chin_bottom, r(1.0), r(-1.0));
        }
        if let Some((a, b)) = f {
            last_edge = Some(0.5 * (a.0[1] + b.0[1]));
        }
        found.push(f);
    }
    // the line starts at the first section that can be read within 20 mm of the chin's front (the chin's tip may not be)
    let start = (0..4).find(|&k| found[k].is_some()).ok_or("no edge under the chin")?;
    let mut end = start;
    while end < SLICES && (found[end].is_some() || (end + 1 < SLICES && found[end + 1].is_some())) {
        end += 1;
    }
    if end - start < 9 {
        return Err("the line under the jaw is shorter than 40 mm");
    }
    let get = |k: usize| -> ([f64; 2], f64) {
        match found[k] {
            Some(_) => get_found(&found, k),
            None => {
                let ((pi, li), (pj, lj)) = (get_found(&found, k - 1), get_found(&found, k + 1));
                ([0.5 * (pi[0] + pj[0]), 0.5 * (pi[1] + pj[1])], 0.5 * (li + lj))
            }
        }
    };
    let mut line = Vec::with_capacity(end);
    let mut edge_dots = Vec::new();
    for k in start..end {
        let z = pogonion - 8.0 - 4.0 * k as f64;
        if let Some((a, b)) = found[k] {
            edge_dots.push([a.0[0], a.0[1], z]);
            edge_dots.push([-b.0[0], b.0[1], z]);
        }
        let (p, l) = get(k);
        line.push((z, p, l));
    }
    let end = line.len();
    let (zc, pc, _) = line[end - 1];
    edge_dots.push([pc[0], pc[1], zc]);
    edge_dots.push([-pc[0], pc[1], zc]);
    let corner_back = pogonion - zc;
    let corner_height = pc[1] - chin_bottom;
    // the outlines run from the chin to the corner, resampled to fixed shares of that length
    let edge = resample(&line.iter().map(|(z, p, _)| [*z, p[1]]).collect::<Vec<_>>(), EDGE_POINTS);
    let below = resample(&line.iter().map(|(z, p, _)| [p[0], *z]).collect::<Vec<_>>(), EDGE_POINTS);
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    // how far the edge stands out over what lies under it, on the back half of the line, where the neck lies under it
    let ledge = mean(&line[end / 2..].iter().map(|l| l.2).collect::<Vec<_>>());
    let edge_mid = mean(&line[end / 3..2 * end / 3].iter().map(|l| l.1[1] - chin_bottom).collect::<Vec<_>>());
    // the edge's height at a fixed place behind the chin, from the sections that can be read there (at least two of the five)
    let back_at: Vec<f64> = (0..SLICES)
        .filter(|&k| (12..=16).contains(&k) && k < line.len() + start && k >= start)
        .filter_map(|k| found[k].map(|_| get_found(&found, k).0[1] - chin_bottom))
        .collect();
    let edge_back_height = if back_at.len() >= 2 { mean(&back_at) } else { f64::NAN };
    let edge_width = mean(&line[2 * end / 3..].iter().map(|l| 2.0 * l.1[0]).collect::<Vec<_>>());
    // the neck's width 24 mm below the chin, and the jaw's width below the mouth as `jaw` reads it
    // the neck's width 24 mm below the chin over its front 35 mm, so the nape (which KOTOR's short necks lack) is left out
    let neck_at = |y: f64| -> Option<f64> {
        let cut = surface.cut(1, y, 0);
        let front = cut.iter().flat_map(|s| [s[0], s[1]]).filter(|p| p[0].abs() < 10.0).map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
        if !front.is_finite() {
            return None;
        }
        let (mut lo, mut hi) = (0.0f64, 0.0f64);
        for p in cut.iter().flat_map(|s| [s[0], s[1]]).filter(|p| p[1] > front - 35.0) {
            lo = lo.min(p[0]);
            hi = hi.max(p[0]);
        }
        (lo < -10.0 && hi > 10.0).then_some(hi - lo)
    };
    let neck_width = [chin_bottom - 22.0, chin_bottom - 24.0, chin_bottom - 26.0]
        .iter()
        .map(|&y| neck_at(y))
        .sum::<Option<f64>>()
        .map_or(f64::NAN, |w| w / 3.0);
    // the edge's places on both sides on the back half of the line, for `tools::kotor_measure` to see whether hair or cloth lies there
    let edge_back: Vec<V3> = line[end / 2..].iter().flat_map(|(z, p, _)| [[p[0], p[1], *z], [-p[0], p[1], *z]]).collect();
    let mut dots: Vec<V3> = edge_dots;
    dots.extend(throat.iter().map(|p| [0.0, p[1], p[0]]));
    if corner[0].is_finite() {
        dots.push([0.0, corner[1], corner[0]]);
    }
    Ok(Measured {
        proportions: vec![throat_angle, throat_back, throat_drop, neck_slope, corner_back, corner_height, edge_mid, ledge, edge_width, neck_width, edge_back_height, edge_width - neck_width],
        outlines: vec![edge, below, throat],
        marks: json!({"pogonion": [0.0, mid.u(pk), pogonion], "chin_bottom": chin_bottom, "dots": dots, "edge_back": edge_back}),
        cuts: Vec::new(),
    })
}

/// How much a gap in each proportion counts in `reach`: a gap of this size costs 1, like 2 mm on an outline. The last row repeats
/// two others and is left out.
#[cfg(not(target_arch = "wasm32"))]
const REACH_SCALES: [f64; 12] = [5.0, 4.0, 4.0, 5.0, 8.0, 6.0, 2.0, 2.0, 4.0, 4.0, 3.0, f64::INFINITY];

/// A surface from places already in the frame of the eyes.
pub fn local_surface(v: &[V3], f: &[[u32; 3]]) -> Surface {
    let t = f.iter().map(|t| t.map(|i| v[i as usize])).filter(|t| t.iter().any(|&p| WINDOW.contains(p))).collect();
    Surface::from_local_in(t, &WINDOW)
}

#[cfg(not(target_arch = "wasm32"))]
pub mod tools {
    use super::*;
    pub use super::local_surface;
    use crate::heads::{parallel, threads};
    use crate::mesh::input::Input;
    use crate::render::{clay, dots, label, sheet};
    use image::RgbImage;
    use std::path::Path;

    /// The heads named in the manifest.
    pub fn names(input: &Input) -> Vec<String> {
        input.manifest["heads"].as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect()
    }

    /// One head's KOTOR mesh without its hair, in the frame of the eyes, with what is not skin tinted blue.
    pub fn kotor_bare(input: &Input, name: &str) -> (Vec<V3>, Vec<[u32; 3]>, Vec<Option<[f64; 3]>>) {
        let mesh = input.head_mesh();
        let frame = Frame::head();
        let v = input.points(&format!("heads/{name}/kotor_v"));
        let f = input.triangles(&format!("heads/{name}/kotor_f"));
        let skin: Option<Vec<bool>> = input
            .has(&format!("extra/heads/{name}/kotor_skin"))
            .then(|| input.u8s(&format!("extra/heads/{name}/kotor_skin")).iter().map(|&s| s != 0).collect());
        let fit: Option<Vec<V3>> = input.has(&format!("heads/{name}/fit_smooth")).then(|| input.points(&format!("heads/{name}/fit_smooth")));
        let extras = KotorExtras {
            name: Some(name),
            triangles: Some(&f),
            fit_triangles: Some(&mesh.triangles),
            skin: skin.as_deref(),
            colour: None,
            fit: fit.as_deref(),
        };
        let bare = bare_kotor(&extras, &frame, &v);
        let tint = bare
            .iter()
            .map(|t| match &skin {
                Some(s) if t.iter().any(|&i| !s[i as usize]) => Some([90.0, 140.0, 230.0]),
                _ => None,
            })
            .collect();
        (v.iter().map(|&p| frame.local(p)).collect(), bare, tint)
    }

    /// The rows read on the jaw's edge, hidden on a KOTOR head whose hair or cloth lies on the back half of the edge.
    const EDGE_ROWS: [usize; 7] = [4, 5, 6, 7, 8, 10, 11];
    /// The share of the edge's places on the back half that may lie on what is not skin before the edge's rows are hidden.
    const COVERED_SHARE: f64 = 0.34;

    /// One head measured on KOTOR's own mesh without its hair, with the jaw's edge rows hidden (NaN, and empty outlines) when more
    /// than COVERED_SHARE of the edge's back half lies next to a vertex that is not skin, such as Kreia's hood or a modelled beard.
    /// Also says whether the edge was hidden.
    pub fn kotor_measure(input: &Input, name: &str) -> (Result<Measured, &'static str>, bool) {
        let (v, f, _) = kotor_bare(input, name);
        let mut m = measure(&local_surface(&v, &f));
        let key = format!("extra/heads/{name}/kotor_skin");
        let Ok(ref mut got) = m else { return (m, false) };
        if !input.has(&key) {
            return (m, false);
        }
        let skin: Vec<bool> = input.u8s(&key).iter().map(|&s| s != 0).collect();
        let mut used = vec![false; v.len()];
        for t in &f {
            for &i in t {
                used[i as usize] = true;
            }
        }
        let back: Vec<V3> = serde_json::from_value(got.marks["edge_back"].clone()).unwrap_or_default();
        if back.is_empty() {
            return (m, false);
        }
        let covered = back
            .iter()
            .filter(|p| {
                let k = (0..v.len()).filter(|&i| used[i]).min_by(|&a, &b| crate::length(crate::sub(v[a], **p)).total_cmp(&crate::length(crate::sub(v[b], **p)))).unwrap();
                !skin[k]
            })
            .count();
        let hide = covered as f64 > COVERED_SHARE * back.len() as f64;
        if hide {
            for r in EDGE_ROWS {
                got.proportions[r] = f64::NAN;
            }
            got.outlines[0].clear();
            got.outlines[1].clear();
        }
        (m, hide)
    }


    /// Runs a command of the underside of the jaw, or says it is not one: `sliders underjaw <command> <data> ...`.
    pub fn run(command: &str, args: &[String], _data: &Path, input: &Input) -> bool {
        match command {
            // KOTOR's heads without hair in the shots, `per` heads to a sheet: `look <data> <out prefix> [per]`
            // one head measured, with its rows: `probe <data> <head>`
            "probe" => {
                let name = args.get(4).cloned().unwrap_or_else(|| "p_carthh".into());
                let (m, hidden) = kotor_measure(input, &name);
                if hidden {
                    println!("the jaw's edge lies on hair or cloth: its rows are hidden");
                }
                match m {
                    Ok(m) => {
                        for (p, x) in PROPORTIONS.iter().zip(&m.proportions) {
                            println!("{:16} {x:8.2}", p.key);
                        }
                        for (o, l) in OUTLINES.iter().zip(&m.outlines) {
                            println!("{}: {}", o.key, l.iter().map(|p| format!("({:.1},{:.1})", p[0], p[1])).collect::<Vec<_>>().join(" "));
                        }
                    }
                    Err(e) => println!("error: {e}"),
                }
                true
            }
            // every head measured on KOTOR's bare mesh, the same subdivided twice, our wrap of it, our FLAME fit of it, and the jaw
            // sliders' stand-in for it, written as JSON: `dump <data> <out.json>`
            "dump" => {
                let out = args.get(4).cloned().unwrap_or_else(|| "under_jaw.json".into());
                let world = crate::heads::World::load(input, &crate::jaw::JAW);
                let set: crate::set::SliderSet =
                    serde_json::from_str(&std::fs::read_to_string(_data.join("web/jaw/sliders.json")).unwrap()).unwrap();
                let page: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(_data.join("web/jaw/page.json")).unwrap()).unwrap();
                let n = set.sliders.len();
                let frame = Frame::head();
                let tri = &world.mesh.triangles;
                let rows: Vec<serde_json::Value> = parallel(world.heads.len(), threads(), |i| {
                    let h = &world.heads[i];
                    let (v, f, _) = kotor_bare(input, &h.name);
                    let (sv, sf, _) = crate::checks::butterfly(&v, &f);
                    let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
                    let ours = |p: &[V3]| measure(&Surface::within(p, tri, &frame, &WINDOW));
                    let values: Vec<f64> = page["heads"][i]["solved"]["full"]["counts"][n.to_string()]["values"]
                        .as_array()
                        .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect())
                        .unwrap_or_else(|| vec![0.0; n]);
                    let made = set.build(&world.smoother, &h.start, &values);
                    let base = world.smoother.smooth(&world.mesh.base, crate::set::SMOOTHING);
                    let as_json = |m: Result<Measured, &'static str>| match m {
                        Ok(m) => json!({"proportions": m.proportions.iter().map(|x| if x.is_nan() { serde_json::Value::Null } else { json!(x) }).collect::<Vec<_>>(), "outlines": m.outlines}),
                        Err(e) => json!({"error": e}),
                    };
                    json!({
                        "name": h.name,
                        "kotor": as_json(kotor_measure(input, &h.name).0),
                        "subdivided": as_json(measure(&local_surface(&sv, &sf))),
                        "wrap": as_json(ours(&input.points(&format!("heads/{}/wrap", h.name)))),
                        "fit": as_json(ours(&input.points(&format!("heads/{}/fit_smooth", h.name)))),
                        "made": as_json(ours(&made)),
                        "base": as_json(ours(&base)),
                        "values": values,
                    })
                });
                let keys: Vec<&str> = PROPORTIONS.iter().map(|p| p.key).collect();
                let okeys: Vec<&str> = OUTLINES.iter().map(|o| o.key).collect();
                std::fs::write(&out, serde_json::to_string(&json!({"proportions": keys, "outlines": okeys, "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // the cut across at a height below the eyes, for one head: `cut <data> <head|base> <y>`
            "cut" => {
                let name = args.get(4).cloned().unwrap_or_else(|| "base".into());
                let y: f64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(-130.0);
                let surface = if name == "base" {
                    let mesh = input.head_mesh();
                    let sm = crate::set::smoother(&mesh.base, &mesh.triangles, &mesh.keep);
                    Surface::within(&sm.smooth(&mesh.base, crate::set::SMOOTHING), &mesh.triangles, &Frame::head(), &WINDOW)
                } else {
                    let (v, f, _) = kotor_bare(input, &name);
                    local_surface(&v, &f)
                };
                let mut pts: Vec<[f64; 2]> = surface.cut(1, y, 0).iter().flat_map(|s| [s[0], s[1]]).collect();
                pts.sort_by(|a, b| a[1].total_cmp(&b[1]));
                for p in pts.iter().step_by(4) {
                    println!("{:7.1} {:7.1}", p[0], p[1]);
                }
                true
            }
            // how close the 5 jaw sliders alone can bring a head's underside of the jaw to KOTOR's: `reach <data> <out.json> <head>...`.
            // A coordinate search from the head's jaw preset, within the sliders' ranges, first on this measure alone, then also
            // holding the jaw measure where the preset has it.
            "reach" => {
                let out = args.get(4).cloned().unwrap_or_else(|| "reach.json".into());
                let world = crate::heads::World::load(input, &crate::jaw::JAW);
                let set: crate::set::SliderSet =
                    serde_json::from_str(&std::fs::read_to_string(_data.join("web/jaw/sliders.json")).unwrap()).unwrap();
                let page: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(_data.join("web/jaw/page.json")).unwrap()).unwrap();
                let n = set.sliders.len();
                let frame = Frame::head();
                let tri = &world.mesh.triangles;
                let names: Vec<String> = args[5..].to_vec();
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let i = world.heads.iter().position(|h| h.name == names[q]).expect("a head");
                    let h = &world.heads[i];
                    let (v, f, _) = kotor_bare(input, &h.name);
                    let _ = (&v, &f);
                    let target = kotor_measure(input, &h.name).0.expect("KOTOR's underside of the jaw");
                    let preset: Vec<f64> = page["heads"][i]["solved"]["full"]["counts"][n.to_string()]["values"]
                        .as_array()
                        .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect())
                        .unwrap_or_else(|| vec![0.0; n]);
                    let jaw_of = |p: &[V3]| crate::jaw::measure(&Surface::within(p, tri, &frame, &crate::feature::Feature::window(&crate::jaw::JAW))).ok();
                    let held = jaw_of(&set.build(&world.smoother, &h.start, &preset)).expect("the preset's jaw");
                    let cost = |values: &[f64], hold: bool| -> (f64, Option<Measured>) {
                        let p = set.build(&world.smoother, &h.start, values);
                        let Ok(m) = measure(&Surface::within(&p, tri, &frame, &WINDOW)) else { return (1e9, None) };
                        let mut c = 0.0;
                        for (k, scale) in REACH_SCALES.iter().enumerate() {
                            let g = m.proportions[k] - target.proportions[k];
                            if g.is_finite() {
                                c += (g / scale).powi(2);
                            }
                        }
                        for o in 0..2 {
                            let d = crate::feature::outline_difference(&OUTLINES[o], &m.outlines[o], &target.outlines[o], 20.0);
                            c += (d / 2.0).powi(2);
                        }
                        if hold {
                            match jaw_of(&p) {
                                Some(j) => {
                                    for (a, b) in j.proportions.iter().zip(&held.proportions) {
                                        c += ((a - b) / 1.5).powi(2);
                                    }
                                }
                                None => c += 1e6,
                            }
                        }
                        (c, Some(m))
                    };
                    let search = |hold: bool| -> (Vec<f64>, f64) {
                        let mut x = preset.clone();
                        let mut best = cost(&x, hold).0;
                        for pass in 0..6 {
                            for s in 0..n {
                                let (lo, hi) = (set.sliders[s].low, set.sliders[s].high);
                                let step = (hi - lo) / 4.0 / 2f64.powi(pass);
                                for dir in [-1.0, 1.0] {
                                    loop {
                                        let mut y = x.clone();
                                        y[s] = (x[s] + dir * step).clamp(lo, hi);
                                        if y[s] == x[s] {
                                            break;
                                        }
                                        let c = cost(&y, hold).0;
                                        if c < best {
                                            best = c;
                                            x = y;
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        (x, best)
                    };
                    let (free, cf) = search(false);
                    let (hold, ch) = search(true);
                    let m_of = |x: &[f64]| cost(x, false).1.map(|m| json!({"proportions": m.proportions, "outlines": m.outlines}));
                    let fit = measure(&Surface::within(&input.points(&format!("heads/{}/fit_smooth", h.name)), tri, &frame, &WINDOW)).ok();
                    json!({
                        "name": h.name,
                        "kotor": {"proportions": target.proportions, "outlines": target.outlines},
                        "preset": {"values": preset, "cost": cost(&preset, false).0, "m": m_of(&preset)},
                        "free": {"values": free, "cost": cf, "m": m_of(&free)},
                        "hold": {"values": hold, "cost": ch, "m": m_of(&hold)},
                        "fit": fit.map(|m| json!({"proportions": m.proportions, "outlines": m.outlines})),
                    })
                });
                std::fs::write(&out, serde_json::to_string(&json!({"sliders": set.sliders.iter().map(|s| &s.name).collect::<Vec<_>>(), "proportions": PROPORTIONS.iter().map(|p| p.key).collect::<Vec<_>>(), "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // KOTOR, the jaw preset, the held solve of `reach` and the FLAME fit, side by side, from the side and from below, with the
            // measure's points: `reach-sheet <data> <reach.json> <out.png>`
            "reach-sheet" => {
                let reach: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[4]).unwrap()).unwrap();
                let world = crate::heads::World::load(input, &crate::jaw::JAW);
                let set: crate::set::SliderSet =
                    serde_json::from_str(&std::fs::read_to_string(_data.join("web/jaw/sliders.json")).unwrap()).unwrap();
                let frame = Frame::head();
                let tri = &world.mesh.triangles;
                let shots = [SHOTS[0], SHOTS[3]];
                let heads = reach["heads"].as_array().unwrap();
                let rows: Vec<Vec<RgbImage>> = parallel(heads.len(), threads(), |q| {
                    let r = &heads[q];
                    let name = r["name"].as_str().unwrap();
                    let h = world.heads.iter().find(|h| h.name == name).unwrap();
                    let vals = |k: &str| -> Vec<f64> { r[k]["values"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect() };
                    let (kv, kf, _) = kotor_bare(input, name);
                    let local = |p: &[V3]| -> Vec<V3> { p.iter().map(|&x| frame.local(x)).collect() };
                    let meshes: Vec<(Vec<V3>, Vec<[u32; 3]>, String)> = vec![
                        (kv, kf, format!("{} kotor", name.replace('_', " "))),
                        (local(&set.build(&world.smoother, &h.start, &vals("preset"))), tri.clone(), "jaw preset".into()),
                        (local(&set.build(&world.smoother, &h.start, &vals("hold"))), tri.clone(), "jaw sliders solved".into()),
                        (local(&input.points(&format!("heads/{name}/fit_smooth"))), tri.clone(), "flame fit".into()),
                    ];
                    let mut row = Vec::new();
                    for s in &shots {
                        for (v, f, tag) in &meshes {
                            let marked: Vec<V3> = measure(&local_surface(v, f))
                                .ok()
                                .and_then(|m| serde_json::from_value(m.marks["dots"].clone()).ok())
                                .unwrap_or_default();
                            let mut img = clay(v, f, s, &[], &[]);
                            dots(&mut img, s, &marked);
                            label(&mut img, tag);
                            row.push(img);
                        }
                    }
                    row
                });
                let flat: Vec<RgbImage> = rows.into_iter().flatten().collect();
                sheet(&flat, 8).save(&args[5]).unwrap();
                eprintln!("wrote {}", args[5]);
                true
            }
            "look" => {
                let prefix = args.get(4).cloned().unwrap_or_else(|| "look".into());
                let per: usize = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(10);
                // `UJ_HEADS=a,b,c` draws only those heads, in that order
                let names: Vec<String> = match std::env::var("UJ_HEADS") {
                    Ok(list) => list.split(',').map(String::from).collect(),
                    Err(_) => names(input),
                };
                let tiles: Vec<Vec<RgbImage>> = parallel(names.len(), threads(), |i| {
                    let (v, f, tint) = kotor_bare(input, &names[i]);
                    let m = measure(&local_surface(&v, &f));
                    if let Err(e) = &m {
                        eprintln!("{}: {e}", names[i]);
                    }
                    let marked: Vec<V3> = m.as_ref().ok().and_then(|m| serde_json::from_value(m.marks["dots"].clone()).ok()).unwrap_or_default();
                    let mut row: Vec<RgbImage> = SHOTS.iter().map(|s| clay(&v, &f, s, &[], &if std::env::var("UJ_TINT").is_ok() { tint.clone() } else { Vec::new() })).collect();
                    for (img, s) in row.iter_mut().zip(&SHOTS) {
                        dots(img, s, &marked);
                    }
                    label(&mut row[0], &names[i].replace('_', " "));
                    row
                });
                for (k, chunk) in tiles.chunks(per).enumerate() {
                    let flat: Vec<RgbImage> = chunk.iter().flatten().cloned().collect();
                    let out = format!("{prefix}_{k}.png");
                    sheet(&flat, 2 * SHOTS.len()).save(&out).unwrap();
                    eprintln!("wrote {out}");
                }
                true
            }
            _ => false,
        }
    }
}
