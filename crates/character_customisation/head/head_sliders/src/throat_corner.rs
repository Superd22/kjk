//! How tight the bend is where the underside of the jaw turns into the front of the neck, and the outline that the throat
//! construction (`chin_neck::ThroatShape`) lays the skin on (roadmap B3 for the jaw and chin).
//!
//! The bend is read in sections parallel to the middle of the face, in the frame of the eyes. In each section the line under the
//! chin (8 to 20 mm behind the chin's front) and the line of the neck's front (20 to 32 mm below the chin's bottom) are fitted as in
//! `under_jaw`, and they meet at the throat's corner. The bend's gap is how far the skin lies from that corner, along the line that
//! halves the angle between them. A circle touching both lines at that gap has the bend's radius. A sharp corner has a gap of 0,
//! whatever the angle; a long, soft bend has a large one.

use crate::V3;
use crate::measure::Surface;

/// The sections across the throat, in mm from the middle.
pub const SECTIONS: [f64; 5] = [-16.0, -8.0, 0.0, 8.0, 16.0];

/// The bend in one section.
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Bend {
    /// The angle between the two lines, in degrees.
    pub angle: f64,
    /// How far the skin lies from the lines' meeting point, along the line that halves their angle, in mm.
    pub gap: f64,
    /// The radius of the circle touching both lines at that gap, in mm.
    pub radius: f64,
    /// The meeting point and the skin's place on the halving line, as (depth, height).
    pub corner: [f64; 2],
    pub skin: [f64; 2],
    /// Unit directions from the corner toward the chin and down the neck, as (depth, height).
    pub to_chin: [f64; 2],
    pub down_neck: [f64; 2],
}

const STEP: f64 = 0.25;

fn fit_line(p: &[[f64; 2]]) -> ([f64; 2], [f64; 2]) {
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
}

/// The section of a surface at `x` from the middle, as segments of (depth, height).
pub fn section(surface: &Surface, x: f64) -> Vec<[[f64; 2]; 2]> {
    surface.cut(0, x, 1).into_iter().map(|[a, b]| [[a[1], a[0]], [b[1], b[0]]]).collect()
}

/// The bend in the section at `x`, from the chin's front `pog` (depth) and bottom `bottom` (height) read down the middle.
pub fn bend_at(surface: &Surface, pog: f64, bottom: f64, x: f64) -> Option<Bend> {
    bend_with(surface, pog, bottom, x, 8.0, 20.0, 34.0)
}

/// The bend in the section at `x`, with the line under the chin fitted from `under_from` to `under_to` mm behind the chin's front, and
/// the line of the neck's front over the 12 mm above `neck_to` mm below the chin's bottom, which the neck must reach without a gap.
pub fn bend_with(surface: &Surface, pog: f64, bottom: f64, x: f64, under_from: f64, under_to: f64, neck_to: f64) -> Option<Bend> {
    let segs = section(surface, x);
    // the line under the chin: the lowest skin at each depth from 8 to 20 mm behind the chin's front
    let under: Vec<[f64; 2]> = (0..=((under_to - under_from) / STEP) as usize)
        .filter_map(|k| {
            let z = pog - under_from - k as f64 * STEP;
            segs.iter()
                .filter_map(|[a, b]| {
                    let (lo, hi) = if a[0] < b[0] { (a, b) } else { (b, a) };
                    (lo[0] <= z && z <= hi[0] && hi[0] > lo[0]).then(|| lo[1] + (z - lo[0]) / (hi[0] - lo[0]) * (hi[1] - lo[1]))
                })
                .filter(|&y| y > bottom - 30.0 && y < bottom + 30.0)
                .fold(None, |m: Option<f64>, y| Some(m.map_or(y, |m| m.min(y))))
                .map(|y| [z, y])
        })
        .collect();
    // the line of the neck's front: the front-most skin at each height from 20 to 32 mm below the chin's bottom, which must reach
    // 34 mm below it without a gap
    let front = |y: f64| -> Option<f64> {
        segs.iter()
            .filter_map(|[a, b]| {
                let (lo, hi) = if a[1] < b[1] { (a, b) } else { (b, a) };
                (lo[1] <= y && y <= hi[1] && hi[1] > lo[1]).then(|| lo[0] + (y - lo[1]) / (hi[1] - lo[1]) * (hi[0] - lo[0]))
            })
            .filter(|&z| z > pog - 90.0)
            .fold(None, |m: Option<f64>, z| Some(m.map_or(z, |m| m.max(z))))
    };
    if (0..=((neck_to - 20.0) / STEP) as usize).any(|k| front(bottom - 20.0 - k as f64 * STEP).is_none()) {
        return None;
    }
    let neck: Vec<[f64; 2]> = (0..=48).map(|k| bottom - neck_to + 2.0 + k as f64 * STEP).map(|y| [front(y).unwrap(), y]).collect();
    if under.len() < 20 {
        return None;
    }
    let (um, mut ud) = fit_line(&under);
    if ud[0] > 0.0 {
        ud = [-ud[0], -ud[1]];
    }
    let (nm, mut nd) = fit_line(&neck);
    if nd[1] > 0.0 {
        nd = [-nd[0], -nd[1]];
    }
    let to_chin = [-ud[0], -ud[1]];
    let angle = (to_chin[0] * nd[0] + to_chin[1] * nd[1]).clamp(-1.0, 1.0).acos();
    if angle.to_degrees() > 165.0 {
        return None;
    }
    let det = ud[0] * (-nd[1]) - ud[1] * (-nd[0]);
    let (bx, by) = (nm[0] - um[0], nm[1] - um[1]);
    let s = (bx * (-nd[1]) - by * (-nd[0])) / det;
    let corner = [um[0] + s * ud[0], um[1] + s * ud[1]];
    // along the line halving the angle, into the body, the first skin from the corner
    let h = [to_chin[0] + nd[0], to_chin[1] + nd[1]];
    let hl = (h[0] * h[0] + h[1] * h[1]).sqrt();
    let d = [h[0] / hl, h[1] / hl];
    let mut hit: Option<f64> = None;
    for [p, q] in &segs {
        let e = [q[0] - p[0], q[1] - p[1]];
        let den = d[0] * (-e[1]) - d[1] * (-e[0]);
        if den.abs() < 1e-12 {
            continue;
        }
        let (bx, by) = (p[0] - corner[0], p[1] - corner[1]);
        let t = (bx * (-e[1]) - by * (-e[0])) / den;
        let u = (d[0] * by - d[1] * bx) / den;
        if (0.0..=1.0).contains(&u) && t > -15.0 && t < 40.0 && hit.is_none_or(|f| t < f) {
            hit = Some(t);
        }
    }
    let gap = hit?;
    let radius = gap / (1.0 / (angle / 2.0).sin() - 1.0);
    Some(Bend { angle: angle.to_degrees(), gap, radius, corner, skin: [corner[0] + gap * d[0], corner[1] + gap * d[1]], to_chin, down_neck: nd })
}

/// How the profile turns from under the chin into the neck in one section.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Turn {
    /// The whole turn from 12 mm behind the chin's front to 34 mm below the chin's bottom, in degrees.
    pub total: f64,
    /// The length of skin over which the middle 80% of that turn happens, in mm.
    pub length: f64,
    /// The share of the turn made over the sharpest 10 mm of skin, in percent.
    pub sharpest: f64,
    /// The turn over the sharpest 10 mm and the sharpest 6 mm of skin, in degrees.
    pub turn10: f64,
    pub turn6: f64,
    /// Where the sharpest 10 mm start, as (depth, height).
    pub at: [f64; 2],
    /// The profile, every 0.5 mm, as (depth, height).
    pub path: Vec<[f64; 2]>,
}

/// The profile from under the chin into the neck in the section at `x`, resampled every 0.5 mm of skin, from 12 mm behind the chin's
/// front to 34 mm below its bottom. It follows the lowest skin going back from the chin until the skin falls more steeply than 45
/// degrees, then the front-most skin going down the neck.
pub fn profile(surface: &Surface, pog: f64, bottom: f64, x: f64) -> Option<Vec<[f64; 2]>> {
    let segs = section(surface, x);
    let under = |z: f64| -> Option<f64> {
        segs.iter()
            .filter_map(|[a, b]| {
                let (lo, hi) = if a[0] < b[0] { (a, b) } else { (b, a) };
                (lo[0] <= z && z <= hi[0] && hi[0] > lo[0]).then(|| lo[1] + (z - lo[0]) / (hi[0] - lo[0]) * (hi[1] - lo[1]))
            })
            .filter(|&y| y > bottom - 45.0 && y < bottom + 40.0)
            .fold(None, |m: Option<f64>, y| Some(m.map_or(y, |m| m.min(y))))
    };
    let front = |y: f64| -> Option<f64> {
        segs.iter()
            .filter_map(|[a, b]| {
                let (lo, hi) = if a[1] < b[1] { (a, b) } else { (b, a) };
                (lo[1] <= y && y <= hi[1] && hi[1] > lo[1]).then(|| lo[0] + (y - lo[1]) / (hi[1] - lo[1]) * (hi[0] - lo[0]))
            })
            .filter(|&z| z > pog - 120.0 && z < pog - 5.0)
            .fold(None, |m: Option<f64>, z| Some(m.map_or(z, |m| m.max(z))))
    };
    let mut pts = Vec::new();
    let mut z = pog - 12.0;
    let mut y = under(z)?;
    pts.push([z, y]);
    loop {
        match under(z - STEP) {
            Some(y2) if y - y2 <= STEP && z - STEP > pog - 120.0 => {
                z -= STEP;
                y = y2;
                pts.push([z, y]);
            }
            _ => break,
        }
    }
    let mut yy = y - STEP;
    while yy >= bottom - 34.0 {
        // the neck's front starts where the underside stopped: skin in front of that place is the overhang just walked
        let z2 = front(yy)?.min(z);
        pts.push([z2, yy]);
        yy -= STEP;
    }
    if pts.len() < 40 {
        return None;
    }
    let raw = &pts[..];
    let mut out = vec![raw[0]];
    let mut carry = 0.0;
    for w in raw.windows(2) {
        let (a, b) = (w[0], w[1]);
        let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        if l == 0.0 {
            continue;
        }
        let mut s = 0.5 - carry;
        while s <= l {
            out.push([a[0] + s / l * (b[0] - a[0]), a[1] + s / l * (b[1] - a[1])]);
            s += 0.5;
        }
        carry = l - (s - 0.5);
    }
    Some(out)
}

/// How the profile turns in the section at `x`.
pub fn turn_at(surface: &Surface, pog: f64, bottom: f64, x: f64) -> Option<Turn> {
    let path = profile(surface, pog, bottom, x)?;
    // directions over 2 mm chords, so a step where the walk turns from the underside to the neck does not count as a turn and back
    let dir: Vec<f64> = path.windows(5).map(|w| (w[4][1] - w[0][1]).atan2(w[4][0] - w[0][0])).collect();
    // the direction at each end, over its 5 mm, and the turn between, counted round the bend (down and forward is turning right)
    let mean = |a: &[f64]| a.iter().sum::<f64>() / a.len() as f64;
    let wrap = |a: f64| (a + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI;
    let mut cum = vec![0.0];
    for w in dir.windows(2) {
        cum.push(cum.last().unwrap() + wrap(w[1] - w[0]));
    }
    let n = cum.len();
    if n < 30 {
        return None;
    }
    let (a0, a1) = (mean(&cum[..10]), mean(&cum[n - 10..]));
    let total = a1 - a0;
    if total.abs() < 10f64.to_radians() {
        return None;
    }
    let share = |c: f64| (c - a0) / total;
    let at = |f: f64| cum.iter().position(|&c| share(c) >= f).unwrap_or(n - 1);
    let length = 0.5 * (at(0.9) as f64 - at(0.1) as f64);
    let (mut best, mut best_k) = (0.0, 0);
    for k in 0..n.saturating_sub(20) {
        let s = (cum[k + 20] - cum[k]) / total;
        if s > best {
            best = s;
            best_k = k;
        }
    }
    let most = |w: usize| (0..n.saturating_sub(w)).map(|k| (cum[k + w] - cum[k]) * total.signum()).fold(0.0, f64::max).to_degrees();
    Some(Turn { total: total.to_degrees().abs(), length, sharpest: 100.0 * best, turn10: most(20), turn6: most(12), at: path[best_k], path })
}

/// The bend in every section, from the chin read by `under_jaw`.
pub fn bends(surface: &Surface) -> Result<(f64, f64, Vec<Option<Bend>>), &'static str> {
    let m = crate::under_jaw::measure(surface)?;
    let pog = m.marks["pogonion"][2].as_f64().unwrap();
    let bottom = m.marks["chin_bottom"].as_f64().unwrap();
    Ok((pog, bottom, SECTIONS.iter().map(|&x| bend_at(surface, pog, bottom, x)).collect()))
}

/// The median length of the mesh's edges in the throat, from the chin's bottom to 35 mm below it, within 25 mm of the middle, in mm.
pub fn edge_length(v: &[V3], f: &[[u32; 3]], pog: f64, bottom: f64) -> f64 {
    let mut l: Vec<f64> = Vec::new();
    for t in f {
        let c = t.map(|i| v[i as usize]);
        let m = [(c[0][0] + c[1][0] + c[2][0]) / 3.0, (c[0][1] + c[1][1] + c[2][1]) / 3.0, (c[0][2] + c[1][2] + c[2][2]) / 3.0];
        if m[0].abs() < 25.0 && m[1] < bottom + 5.0 && m[1] > bottom - 35.0 && m[2] > pog - 75.0 {
            for (i, j) in [(0, 1), (1, 2), (2, 0)] {
                l.push(crate::length(crate::sub(c[i], c[j])));
            }
        }
    }
    l.sort_by(f64::total_cmp);
    l.get(l.len() / 2).copied().unwrap_or(f64::NAN)
}

/// The places of the outline's corner.
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Corner {
    /// Where the corner lies behind the chin's front, in mm.
    #[serde(default)]
    pub back: f64,
    /// How high the corner lies above the chin's bottom, in mm.
    #[serde(default)]
    pub height: f64,
    /// The radius of the bend at the corner, in mm.
    #[serde(default)]
    pub radius: f64,
    /// How much higher the corner lies under the jaw's corners than under the chin, in mm.
    #[serde(default)]
    pub side_rise: f64,
}

/// Where the outline leaves the skin under the chin, behind the chin's front, and where it meets the neck's front, below the chin's
/// bottom, in mm.
const OUTLINE_FROM: f64 = 12.0;
const OUTLINE_TO: f64 = 34.0;
/// How far from the middle the jaw's corners lie, and how far behind the chin's front the corner lies there, in mm: KOTOR's mean
/// jaw corner. A corner set further back under the jaw's corners made a hooked crease there.
const JAW_SIDE: f64 = 45.0;
const SIDE_BACK: f64 = 84.0;

/// The moves of a head with its hidden vertices moved as their 6 nearest skin vertices move, the way the build places them.
/// `before` and `after` are in the frame of the eyes.
pub fn follow_hidden(mesh: &crate::mesh::HeadMesh, before: &[V3], after: &mut [V3]) {
    let skin: Vec<usize> = (0..mesh.hidden.len()).filter(|&g| !mesh.hidden[g]).collect();
    let moved = |g: usize| {
        let i = mesh.first[g] as usize;
        crate::sub(after[i], before[i])
    };
    let mut by_hidden = vec![[0.0; 3]; mesh.hidden.len()];
    for (k, g) in (0..mesh.hidden.len()).filter(|&g| mesh.hidden[g]).enumerate() {
        let mut d = [0.0; 3];
        for j in 0..6 {
            d = crate::add(d, crate::scale(moved(skin[mesh.near[k][j] as usize]), mesh.near_w[k][j]));
        }
        by_hidden[g] = d;
    }
    for i in 0..after.len() {
        let g = mesh.group[i] as usize;
        if mesh.hidden[g] && !mesh.keep[i] {
            after[i] = crate::add(before[i], by_hidden[g]);
        }
    }
}

/// Where a ray first meets a path: the distance along the ray, and the distance along the path.
pub fn ray_hit(o: [f64; 2], d: [f64; 2], path: &[[f64; 2]]) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64)> = None;
    let mut along = 0.0;
    for w in path.windows(2) {
        let (p, q) = (w[0], w[1]);
        let e = [q[0] - p[0], q[1] - p[1]];
        let l = (e[0] * e[0] + e[1] * e[1]).sqrt();
        let den = d[0] * (-e[1]) - d[1] * (-e[0]);
        if den.abs() > 1e-12 {
            let (bx, by) = (p[0] - o[0], p[1] - o[1]);
            let t = (bx * (-e[1]) - by * (-e[0])) / den;
            let u = (d[0] * by - d[1] * bx) / den;
            if (0.0..=1.0).contains(&u) && t > 0.0 && best.is_none_or(|b| t < b.0) {
                best = Some((t, along + u * l));
            }
        }
        along += l;
    }
    best
}

/// The place a distance along a path.
pub fn at_length(path: &[[f64; 2]], s: f64) -> [f64; 2] {
    let mut along = 0.0;
    for w in path.windows(2) {
        let l = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
        if along + l >= s && l > 0.0 {
            let u = (s - along) / l;
            return [w[0][0] + u * (w[1][0] - w[0][0]), w[0][1] + u * (w[1][1] - w[0][1])];
        }
        along += l;
    }
    *path.last().unwrap()
}

pub fn path_length(path: &[[f64; 2]]) -> f64 {
    path.windows(2).map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt()).sum()
}

/// The outline in one section, every 0.5 mm, from the skin under the chin to the neck's front, or none where the section does
/// not reach both.
pub fn outline(surface: &Surface, pog: f64, bottom: f64, x: f64, c: &Corner) -> Option<Vec<[f64; 2]>> {
    let segs = section(surface, x);
    // the jaw seen from below is a U: the outline starts further back toward the sides
    let side = (x / JAW_SIDE).powi(2);
    let z0 = pog - OUTLINE_FROM - x * x / 60.0;
    let y0 = segs
        .iter()
        .filter_map(|[a, b]| {
            let (lo, hi) = if a[0] < b[0] { (a, b) } else { (b, a) };
            (lo[0] <= z0 && z0 <= hi[0] && hi[0] > lo[0]).then(|| lo[1] + (z0 - lo[0]) / (hi[0] - lo[0]) * (hi[1] - lo[1]))
        })
        .filter(|&y| y > bottom - 20.0 && y < bottom + 30.0)
        .fold(None, |m: Option<f64>, y| Some(m.map_or(y, |m| m.min(y))))?;
    // the neck's front 34 mm below the chin
    let front_at = |y1: f64| segs
        .iter()
        .filter_map(|[a, b]| {
            let (lo, hi) = if a[1] < b[1] { (a, b) } else { (b, a) };
            (lo[1] <= y1 && y1 <= hi[1] && hi[1] > lo[1]).then(|| lo[0] + (y1 - lo[1]) / (hi[1] - lo[1]) * (hi[0] - lo[0]))
        })
        .filter(|&z| z > pog - 120.0 && z < pog - 5.0)
        .fold(None, |m: Option<f64>, z: f64| Some(m.map_or(z, |m: f64| m.max(z))));
    let y1 = bottom - OUTLINE_TO;
    let z1 = front_at(y1)?;
    let back = c.back + (SIDE_BACK.max(c.back) - c.back) * side;
    let k = [pog - back, bottom + c.height + c.side_rise * side];
    let (a, b) = ([z0, y0], [z1, y1]);
    if k[0] >= z0 - 2.0 || k[1] <= y1 + 2.0 {
        return None;
    }
    // the corner rounded: the arc touches both lines at the radius's distance along them, shortened to fit
    let len = |p: [f64; 2], q: [f64; 2]| ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt();
    let ua = [(a[0] - k[0]) / len(k, a), (a[1] - k[1]) / len(k, a)];
    let ub = [(b[0] - k[0]) / len(k, b), (b[1] - k[1]) / len(k, b)];
    let theta = (ua[0] * ub[0] + ua[1] * ub[1]).clamp(-1.0, 1.0).acos();
    let mut tangent = c.radius.max(0.0) / (theta / 2.0).tan();
    tangent = tangent.min(0.8 * len(k, a)).min(0.8 * len(k, b));
    let (ta, tb) = ([k[0] + tangent * ua[0], k[1] + tangent * ua[1]], [k[0] + tangent * ub[0], k[1] + tangent * ub[1]]);
    let mut pts = vec![a, ta];
    // a quadratic curve from one touching place to the other, with the corner as its middle control, stands in for the arc
    for i in 1..40 {
        let t = i as f64 / 40.0;
        let m = [(1.0 - t) * (1.0 - t), 2.0 * t * (1.0 - t), t * t];
        pts.push([m[0] * ta[0] + m[1] * k[0] + m[2] * tb[0], m[0] * ta[1] + m[1] * k[1] + m[2] * tb[1]]);
    }
    pts.push(tb);
    pts.push(b);
    Some(pts)
}

#[cfg(not(target_arch = "wasm32"))]
pub mod tools {
    use super::*;
    use crate::heads::{parallel, threads};
    use crate::jaw_edge::tools::Heads;
    use crate::mesh::input::Input;
    use crate::under_jaw::tools::{kotor_bare, local_surface};
    use serde_json::json;
    use std::path::Path;

    fn read(surface: Surface) -> serde_json::Value {
        match bends(&surface) {
            Ok((pog, bottom, b)) => {
                let turns: Vec<Option<serde_json::Value>> = SECTIONS
                    .iter()
                    .map(|&x| turn_at(&surface, pog, bottom, x).map(|t| json!({"total": t.total, "length": t.length, "sharpest": t.sharpest, "turn10": t.turn10, "turn6": t.turn6, "at": t.at})))
                    .collect();
                json!({"pog": pog, "bottom": bottom, "sections": b, "turns": turns})
            }
            Err(e) => json!({"error": e}),
        }
    }

    /// Runs a command of the throat corner, or says it is not one: `sliders throatcorner <command> <data> ...`.
    pub fn run(command: &str, args: &[String], data: &Path, input: &Input) -> bool {
        match command {
            // every head's bend on KOTOR's bare mesh, the same subdivided twice, viewer 2's live slider head and our FLAME fit,
            // with the meshes' edge lengths in the throat: `dump <data> <out.json>`
            "dump" => {
                let out = args[4].clone();
                let heads = Heads::load(input, data);
                let local = |p: &[V3]| -> Vec<V3> { p.iter().map(|&x| heads.frame.local(x)).collect() };
                let tri = &heads.mesh.triangles;
                let names = crate::under_jaw::tools::names(input);
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let (v, f, _) = kotor_bare(input, name);
                    let (sv, sf, _) = crate::checks::butterfly(&v, &f);
                    let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
                    let kotor = read(local_surface(&v, &f));
                    let (pog, bottom) = (kotor["pog"].as_f64().unwrap_or(0.0), kotor["bottom"].as_f64().unwrap_or(-120.0));
                    let live = local(&heads.pair(input, name).1);
                    let fit = local(&input.points(&format!("heads/{name}/fit_smooth")));
                    let row = json!({
                        "name": name,
                        "kotor": kotor,
                        "subdivided": read(local_surface(&sv, &sf)),
                        "live": read(local_surface(&live, tri)),
                        "fit": read(local_surface(&fit, tri)),
                        "kotor_edge": edge_length(&v, &f, pog, bottom),
                        "ours_edge": edge_length(&live, tri, pog, bottom),
                    });
                    row
                });
                let base = local(&heads.pair(input, "base").1);
                std::fs::write(&out, serde_json::to_string(&json!({"sections": SECTIONS, "base": read(local_surface(&base, tri)), "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // the vertices under the jaw that the zone holds at 0, with why: `zeros <data>`
            "zeros" => {
                let heads = Heads::load(input, data);
                let m = &heads.mesh;
                let base: Vec<V3> = heads.pair(input, "base").1.iter().map(|&p| heads.frame.local(p)).collect();
                for (i, q) in base.iter().enumerate() {
                    let g = m.group[i] as usize;
                    if heads.zone.weight[i] == 0.0 && q[1] < -100.0 && q[1] > -150.0 && q[2] > -60.0 && q[0].abs() < 40.0 {
                        println!("{i} x {:6.1} y {:6.1} z {:6.1} keep {} hidden {} zone {}", q[0], q[1], q[2], m.keep[i], m.hidden[g], m.zone[i]);
                    }
                }
                true
            }
            _ => false,
        }
    }
}
