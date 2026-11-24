//! The profile from the chin to the neck, measured as outlines on any triangle mesh in the frame of the eyes (`measure::Frame`)
//! (roadmap B3 and B5 for the jaw and chin). Viewer 2 does not use it: only the `sliders chinneck` tools read it.
//!
//! One outline runs from the chin's front, down round the chin, back along the underside of the jaw, round the throat's corner and
//! down the front of the neck to 34 mm below the chin's bottom. It is read in sections parallel to the middle of the face, at 0, 15
//! and 30 mm from the middle on each side, and once on the whole head seen from the side (the silhouette, which carries the jaw's
//! corners). Outlines are compared after their place and size are taken out: each is moved so that the chin's front and bottom
//! lie at 0, and scaled so that the chin's bottom lies `FACE` mm under the eyes. The gap between two outlines is the mean distance
//! from each one's points to the other line, both ways, in mm.

use crate::V3;
use crate::measure::{Cut, STEP, Surface};

/// The sections, in mm from the middle. Each is read on both sides; the silhouette is read once.
pub const SECTIONS: [f64; 3] = [0.0, 12.0, 24.0];
/// The readings' names: the three sections, then the silhouette.
pub const READINGS: [&str; 4] = ["middle", "12 mm", "24 mm", "side view"];
/// How far below the chin's bottom the outline ends, in the scaled mm.
pub const NECK_TO: f64 = 34.0;
/// The face height every outline is scaled to: the chin's bottom this many mm under the eyes.
pub const FACE: f64 = 120.0;
/// How far from the middle the silhouette reads the head, in mm: past the jaw's corners, short of the ears.
pub const SIDE_REACH: f64 = 55.0;
/// The points of an outline lie this far apart along it, in mm.
const SPACING: f64 = 0.5;

/// The chin's front (its depth and height) and the chin's bottom (its height), read down the middle as `under_jaw` reads them.
pub fn chin(surface: &Surface) -> Option<([f64; 2], f64)> {
    let from = -200.0;
    let mid = Cut { from, z: surface.front(0, 0.0, 1, from, 800) };
    let (pk, pog) = (mid.index(-125.0)..=mid.index(-85.0)).filter(|&k| !mid.z[k].is_nan()).map(|k| (k, mid.z[k])).max_by(|a, b| a.1.total_cmp(&b.1))?;
    let ufrom = pog - 60.0;
    let under = crate::jaw::underside(surface, ufrom, 241);
    let bottom = (0..under.len()).filter(|&k| ufrom + k as f64 * STEP > pog - 25.0 && !under[k].is_nan()).map(|k| under[k]).fold(f64::INFINITY, f64::min);
    if !bottom.is_finite() {
        return None;
    }
    // a lower lip that stands out further than the chin is found first: the chin's front is the front-most place from 2 to 25 mm
    // above the chin's bottom
    let (pk, pog) = (mid.index(bottom + 2.0)..=mid.index(bottom + 25.0)).filter(|&k| !mid.z[k].is_nan()).map(|k| (k, mid.z[k])).max_by(|a, b| a.1.total_cmp(&b.1)).unwrap_or((pk, pog));
    Some(([pog, mid.u(pk)], bottom))
}

/// A section's segments at `x` from the middle, as (depth, height).
fn section(surface: &Surface, x: f64) -> Vec<[[f64; 2]; 2]> {
    surface.cut(0, x, 1).into_iter().map(|[a, b]| [[a[1], a[0]], [b[1], b[0]]]).collect()
}

/// The silhouette's segments: every edge of every triangle whose middle lies within `SIDE_REACH` of the middle of the face, seen
/// from the side, as (depth, height). The outline of the whole head seen from the side is made of these edges.
fn silhouette(surface: &Surface) -> Vec<[[f64; 2]; 2]> {
    let mut out = Vec::new();
    for t in &surface.triangles {
        if ((t[0][0] + t[1][0] + t[2][0]) / 3.0).abs() > SIDE_REACH {
            continue;
        }
        for (i, j) in [(0, 1), (1, 2), (2, 0)] {
            out.push([[t[i][2], t[i][1]], [t[j][2], t[j][1]]]);
        }
    }
    out
}

/// The walk round the outer edge of a set of segments: front-most skin going down from the chin's front, lowest skin going back once
/// the front recedes faster than 45 degrees, and front-most skin going down the neck once the underside drops faster than 45 degrees,
/// to `end` (a height). None when the neck does not reach `end` without a gap.
fn walk(segs: &[[[f64; 2]; 2]], start: [f64; 2], end: f64) -> Option<Vec<[f64; 2]>> {
    let pog = start[0];
    let front = |y: f64| -> Option<f64> {
        segs.iter()
            .filter_map(|[a, b]| {
                let (lo, hi) = if a[1] < b[1] { (a, b) } else { (b, a) };
                (lo[1] <= y && y <= hi[1] && hi[1] > lo[1]).then(|| lo[0] + (y - lo[1]) / (hi[1] - lo[1]) * (hi[0] - lo[0]))
            })
            .filter(|&z| z > pog - 130.0 && z < pog + 15.0)
            .fold(None, |m: Option<f64>, z| Some(m.map_or(z, |m| m.max(z))))
    };
    let under = |z: f64, near: f64| -> Option<f64> {
        segs.iter()
            .filter_map(|[a, b]| {
                let (lo, hi) = if a[0] < b[0] { (a, b) } else { (b, a) };
                (lo[0] <= z && z <= hi[0] && hi[0] > lo[0]).then(|| lo[1] + (z - lo[0]) / (hi[0] - lo[0]) * (hi[1] - lo[1]))
            })
            .filter(|&y| y > near - 40.0 && y < near + 25.0)
            .fold(None, |m: Option<f64>, y| Some(m.map_or(y, |m| m.min(y))))
    };
    let mut pts = Vec::new();
    let mut y = start[1];
    let mut z = front(y)?;
    pts.push([z, y]);
    while let Some(z2) = front(y - STEP) {
        if z - z2 > STEP || y - STEP < end {
            break;
        }
        y -= STEP;
        z = z2;
        pts.push([z, y]);
    }
    // the underside starts below the last place of the front: the lowest skin at the next depth back
    while let Some(y2) = under(z - STEP, y) {
        if y - y2 > STEP || z - STEP < pog - 125.0 {
            break;
        }
        z -= STEP;
        y = y2;
        pts.push([z, y]);
    }
    let mut yy = y - STEP;
    while yy >= end {
        // the neck's front starts where the underside stopped: skin in front of that place is the overhang just walked
        let z2 = front(yy)?.min(z);
        pts.push([z2, yy]);
        yy -= STEP;
    }
    (pts.len() >= 40).then(|| even(&pts))
}

/// A path resampled every `SPACING` mm along its length.
fn even(raw: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut out = vec![raw[0]];
    let mut carry = 0.0;
    for w in raw.windows(2) {
        let (a, b) = (w[0], w[1]);
        let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        if l == 0.0 {
            continue;
        }
        let mut s = SPACING - carry;
        while s <= l {
            out.push([a[0] + s / l * (b[0] - a[0]), a[1] + s / l * (b[1] - a[1])]);
            s += SPACING;
        }
        carry = l - (s - SPACING);
    }
    out
}

/// One head's outlines.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    /// The chin's front (depth, height) and the chin's bottom (height), in the frame of the eyes.
    pub pog: [f64; 2],
    pub bottom: f64,
    /// How much the outlines were scaled: `FACE` over the chin's bottom's depth under the eyes.
    pub scale: f64,
    /// For each section, the +x side and the -x side, then the silhouette alone (in the first place), lined up and scaled: the chin's
    /// front at depth 0 and the chin's bottom at height 0.
    pub outlines: Vec<[Option<Vec<[f64; 2]>>; 2]>,
}

impl Profile {
    /// A reading: the section's two sides, or the silhouette.
    pub fn sides(&self, r: usize) -> impl Iterator<Item = &Vec<[f64; 2]>> {
        self.outlines[r].iter().flatten()
    }

    /// A lined-up outline put back in this head's frame of the eyes.
    pub fn unalign(&self, p: [f64; 2]) -> [f64; 2] {
        [self.pog[0] + p[0] / self.scale, self.bottom + p[1] / self.scale]
    }
}

/// Measures the outlines of a surface, or says what could not be found.
pub fn measure(surface: &Surface) -> Result<Profile, &'static str> {
    let (pog, bottom) = chin(surface).ok_or("no chin")?;
    let scale = FACE / -bottom;
    let end = bottom - NECK_TO / scale;
    let align = |path: Vec<[f64; 2]>| -> Vec<[f64; 2]> { path.into_iter().map(|p| [(p[0] - pog[0]) * scale, (p[1] - bottom) * scale]).collect() };
    let mut outlines: Vec<[Option<Vec<[f64; 2]>>; 2]> = SECTIONS
        .iter()
        .map(|&x| [1.0, -1.0].map(|side| walk(&section(surface, side * x), pog, end).map(align)))
        .collect();
    outlines.push([walk(&silhouette(surface), pog, end).map(align), None]);
    if outlines[0][0].is_none() {
        return Err("the middle cannot be read");
    }
    Ok(Profile { pog, bottom, scale, outlines })
}

/// The mean distance from the points of `a` to the line `b`, in mm.
fn one_way(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    let d = |p: [f64; 2]| {
        b.windows(2)
            .map(|w| {
                let e = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
                let l2 = e[0] * e[0] + e[1] * e[1];
                let t = if l2 > 0.0 { (((p[0] - w[0][0]) * e[0] + (p[1] - w[0][1]) * e[1]) / l2).clamp(0.0, 1.0) } else { 0.0 };
                ((p[0] - w[0][0] - t * e[0]).powi(2) + (p[1] - w[0][1] - t * e[1]).powi(2)).sqrt()
            })
            .fold(f64::INFINITY, f64::min)
    };
    a.iter().map(|&p| d(p)).sum::<f64>() / a.len() as f64
}

/// The gap between two outlines: the mean distance from each one's points to the other line, both ways, in mm.
pub fn gap(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    0.5 * (one_way(a, b) + one_way(b, a))
}

/// The gap between two heads in one reading: the mean over the sides both can read, or None.
pub fn reading_gap(a: &Profile, b: &Profile, r: usize) -> Option<f64> {
    let g: Vec<f64> = (0..2).filter_map(|s| Some(gap(a.outlines[r][s].as_ref()?, b.outlines[r][s].as_ref()?))).collect();
    (!g.is_empty()).then(|| g.iter().sum::<f64>() / g.len() as f64)
}

/// How much each reading counts in the whole gap: the middle, 12 mm, 24 mm and the side view. The 24 mm section fails check 1 (its
/// two sides differ by up to 4.5 mm on KOTOR, as much as two heads differ) and reads on only 19 of KOTOR's heads, so it does not count.
pub const WEIGHTS: [f64; 4] = [1.0, 1.0, 0.0, 1.0];

/// The whole gap between two heads: the readings' gaps weighted by `WEIGHTS` over the readings both have.
pub fn whole_gap(a: &Profile, b: &Profile) -> Option<f64> {
    let (mut s, mut w) = (0.0, 0.0);
    for r in 0..READINGS.len() {
        if let Some(g) = reading_gap(a, b, r) {
            s += WEIGHTS[r] * g;
            w += WEIGHTS[r];
        }
    }
    (w > 0.0).then(|| s / w)
}

#[cfg(not(target_arch = "wasm32"))]
pub mod tools {
    use super::*;
    use crate::heads::{parallel, threads};
    use crate::mesh::input::Input;
    use crate::under_jaw::tools::{kotor_bare, kotor_measure, local_surface, names};
    use serde_json::json;
    use std::path::{Path, PathBuf};

    /// A head in kit vertex order from a folder of `<head>.bin` files (little-endian float32, metres), in the frame of the eyes.
    pub fn read_bin(dir: &Path, name: &str) -> Option<Vec<V3>> {
        let frame = crate::measure::Frame::head();
        let raw = std::fs::read(dir.join(format!("{name}.bin"))).ok()?;
        Some(raw.chunks_exact(12).map(|c| frame.local(std::array::from_fn(|k| f32::from_le_bytes(c[4 * k..4 * k + 4].try_into().unwrap()) as f64))).collect())
    }

    /// A head built with the throat construction: its jaw values (the live preset when absent) and the throat's four values.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct Built {
        #[serde(default)]
        pub jaw: Option<Vec<f64>>,
        pub throat: Option<Vec<f64>>,
    }

    /// The head of `b` in metres.
    pub fn build_throat(h: &crate::jaw_edge::tools::Heads, shape: &ThroatShape, name: &str, b: &Built) -> Vec<V3> {
        let live = h.live_with(name, b.jaw.as_deref());
        let shaped = match &b.throat {
            Some(t) => shape.apply(&live, &Throat::from_slice(t)),
            None => live,
        };
        shaped
    }

    /// A head in metres written as `<dir>/<head>.bin`, in kit vertex order.
    pub fn write_bin(dir: &Path, name: &str, p: &[V3]) {
        let kit: Vec<u8> = p.iter().flat_map(|q| q.map(|c| c as f32)).flat_map(|c| c.to_le_bytes()).collect();
        std::fs::write(dir.join(format!("{name}.bin")), kit).unwrap();
    }

    /// KOTOR's outlines of one head on its own bare mesh, with every reading hidden when hair, a hood or a beard lies on the back
    /// half of the jaw's edge (as `under_jaw` hides its edge rows). Also says whether they were hidden.
    pub fn kotor(input: &Input, name: &str) -> (Result<Profile, &'static str>, bool) {
        let (v, f, _) = kotor_bare(input, name);
        let hidden = kotor_measure(input, name).1;
        let mut p = measure(&local_surface(&v, &f));
        if hidden && let Ok(p) = &mut p {
            for r in 0..READINGS.len() {
                p.outlines[r] = [None, None];
            }
        }
        (p, hidden)
    }

    fn subdivided(input: &Input, name: &str) -> Result<Profile, &'static str> {
        let (v, f, _) = kotor_bare(input, name);
        let (sv, sf, _) = crate::checks::butterfly(&v, &f);
        let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
        measure(&local_surface(&sv, &sf))
    }

    fn pct(v: &mut [f64], q: f64) -> f64 {
        v.sort_by(f64::total_cmp);
        if v.is_empty() { f64::NAN } else { v[((v.len() - 1) as f64 * q).round() as usize] }
    }

    /// Draws a lined-up outline into a picture of the side shot, put back in `head`'s frame.
    pub fn draw(img: &mut image::RgbImage, shot: &crate::feature::Shot, head: &Profile, line: &[[f64; 2]], colour: [u8; 3], thick: i64) {
        let px = std::env::var("SLIDERS_PX").ok().and_then(|v| v.parse().ok()).unwrap_or(2.5);
        for w in line.windows(2) {
            for s in 0..8 {
                let t = s as f64 / 8.0;
                let p = head.unalign([w[0][0] + t * (w[1][0] - w[0][0]), w[0][1] + t * (w[1][1] - w[0][1])]);
                // the side shot looks along -x: depth runs to the right, height up
                let (x, y) = ((p[0] - shot.window[0]) * px, (shot.window[3] - p[1]) * px);
                for dy in -thick..=thick {
                    for dx in -thick..=thick {
                        let (i, j) = (x as i64 + dx, y as i64 + dy);
                        if i >= 0 && j >= 0 && (i as u32) < img.width() && (j as u32) < img.height() {
                            img.put_pixel(i as u32, j as u32, image::Rgb(colour));
                        }
                    }
                }
            }
        }
    }

    /// The side shot for the outlines.
    pub const SIDE: crate::feature::Shot = crate::feature::Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-135.0, 45.0, -185.0, -60.0] };

    /// Runs a command of the chin-to-neck profile, or says it is not one: `sliders chinneck <command> <data> ...`.
    pub fn run(command: &str, args: &[String], data: &Path, input: &Input) -> bool {
        match command {
            // the measure's four checks: every KOTOR head on its bare mesh, on the same subdivided twice, and our heads from folders
            // of kit-order bins: `checks <data> <out.json> [label=<bins dir>]...`
            "checks" => {
                let out = args[4].clone();
                let variants: Vec<(String, PathBuf)> = args[5..].iter().map(|v| { let (a, b) = v.split_once('=').unwrap(); (a.to_string(), PathBuf::from(b)) }).collect();
                let mesh = input.head_mesh();
                let all = names(input);
                let rows: Vec<(String, Option<Profile>, bool, Option<Profile>, Vec<Option<Profile>>)> = parallel(all.len(), threads(), |q| {
                    let name = &all[q];
                    let (k, hidden) = kotor(input, name);
                    let s = subdivided(input, name).ok();
                    let ours = variants.iter().map(|(_, dir)| read_bin(dir, name).and_then(|p| measure(&local_surface(&p, &mesh.triangles)).ok())).collect();
                    (name.clone(), k.ok(), hidden, s, ours)
                });
                let _ = data;
                let mut report = serde_json::Map::new();
                let mut summary = Vec::new();
                for r in 0..READINGS.len() {
                    // check 1: KOTOR against its subdivided copy, and a section's two sides
                    let mut wob: Vec<f64> = rows.iter().filter_map(|(_, k, _, s, _)| {
                        let (k, s) = (k.as_ref()?, s.as_ref()?);
                        if k.outlines[r][0].is_none() { return None; }
                        reading_gap(k, s, r)
                    }).collect();
                    let mut sides: Vec<f64> = rows.iter().filter_map(|(_, k, _, _, _)| {
                        let k = k.as_ref()?;
                        Some(gap(k.outlines[r][0].as_ref()?, k.outlines[r][1].as_ref()?))
                    }).filter(|_| r > 0 && r < 3).collect();
                    // the spread: the gap between two different KOTOR heads
                    let ks: Vec<(&String, &Profile)> = rows.iter().filter_map(|(n, k, _, _, _)| Some((n, k.as_ref()?))).filter(|(_, k)| k.outlines[r][0].is_some()).collect();
                    let mut pairs = Vec::new();
                    for i in 0..ks.len() {
                        for j in i + 1..ks.len() {
                            if let Some(g) = reading_gap(ks[i].1, ks[j].1, r) {
                                pairs.push(g);
                            }
                        }
                    }
                    // check 3: each head's subdivided copy (and, in the sections, its +x side) against every head's KOTOR mesh (its -x side)
                    let mut own_sub = 0;
                    let mut own_side = 0;
                    let mut read_side = 0;
                    for (n, _, _, s, _) in &rows {
                        let Some(s) = s else { continue };
                        if s.outlines[r][0].is_none() || !ks.iter().any(|(m, _)| *m == n) { continue; }
                        let best = ks.iter().filter_map(|(m, k)| Some((*m, gap(s.outlines[r][0].as_ref()?, k.outlines[r][0].as_ref()?)))).min_by(|a, b| a.1.total_cmp(&b.1));
                        if best.is_some_and(|b| b.0 == n) { own_sub += 1; }
                    }
                    if r > 0 && r < 3 {
                        for (n, k) in &ks {
                            let Some(right) = &k.outlines[r][0] else { continue };
                            if k.outlines[r][1].is_none() { continue; }
                            read_side += 1;
                            let best = ks.iter().filter_map(|(m, o)| Some((*m, gap(right, o.outlines[r][1].as_ref()?)))).min_by(|a, b| a.1.total_cmp(&b.1));
                            if best.is_some_and(|b| b.0 == *n) { own_side += 1; }
                        }
                    }
                    // our heads against KOTOR
                    let ours: Vec<serde_json::Value> = variants.iter().enumerate().map(|(vi, (label, _))| {
                        let mut g: Vec<f64> = rows.iter().filter_map(|(_, k, _, _, o)| reading_gap(k.as_ref()?, o[vi].as_ref()?, r).filter(|_| k.as_ref().unwrap().outlines[r][0].is_some())).collect();
                        let n = g.len();
                        json!({"label": label, "heads": n, "median": pct(&mut g, 0.5), "p95": pct(&mut g, 0.95)})
                    }).collect();
                    let (nk, nw, ns) = (ks.len(), wob.len(), sides.len());
                    let row = json!({
                        "reading": READINGS[r],
                        "kotor_heads": nk,
                        "wobble_subdivided": {"heads": nw, "median": pct(&mut wob, 0.5), "p95": pct(&mut wob, 0.95)},
                        "wobble_sides": {"heads": ns, "median": pct(&mut sides, 0.5), "p95": pct(&mut sides, 0.95)},
                        "between_heads": {"pairs": pairs.len(), "p5": pct(&mut pairs, 0.05), "median": pct(&mut pairs, 0.5)},
                        "check3_subdivided_finds_itself": own_sub,
                        "check3_right_side_finds_own_left": [own_side, read_side],
                        "ours": ours,
                    });
                    summary.push(format!("{}: {}", READINGS[r], row));
                    report.insert(READINGS[r].to_string(), row);
                }
                // the whole gap: every head's gap to KOTOR, and the extremes for check 4
                let per_head: Vec<serde_json::Value> = rows.iter().map(|(n, k, hidden, s, o)| {
                    let readable: Vec<bool> = (0..READINGS.len()).map(|r| k.as_ref().is_some_and(|k| k.outlines[r][0].is_some())).collect();
                    json!({
                        "name": n,
                        "hidden": hidden,
                        "readable": readable,
                        "wobble": k.as_ref().zip(s.as_ref()).and_then(|(k, s)| whole_gap(k, s)),
                        "per_reading": variants.iter().enumerate().map(|(vi, (label, _))| json!({"label": label, "gaps": (0..READINGS.len()).map(|r| k.as_ref().zip(o[vi].as_ref()).and_then(|(k, o)| reading_gap(k, o, r))).collect::<Vec<_>>(), "whole": k.as_ref().zip(o[vi].as_ref()).and_then(|(k, o)| whole_gap(k, o))})).collect::<Vec<_>>(),
                    })
                }).collect();
                let ks: Vec<(&String, &Profile)> = rows.iter().filter_map(|(n, k, _, _, _)| Some((n, k.as_ref()?))).collect();
                let mut pairs: Vec<(f64, String, String)> = Vec::new();
                for i in 0..ks.len() {
                    for j in i + 1..ks.len() {
                        if let Some(g) = whole_gap(ks[i].1, ks[j].1) {
                            pairs.push((g, ks[i].0.clone(), ks[j].0.clone()));
                        }
                    }
                }
                pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
                report.insert("per_head".into(), json!(per_head));
                report.insert("most_alike".into(), json!(pairs.iter().take(6).collect::<Vec<_>>()));
                report.insert("most_different".into(), json!(pairs.iter().rev().take(6).collect::<Vec<_>>()));
                std::fs::write(&out, serde_json::to_string_pretty(&serde_json::Value::Object(report)).unwrap()).unwrap();
                for s in summary {
                    println!("{s}");
                }
                eprintln!("wrote {out}");
                true
            }
            // KOTOR's bare mesh in clay from the side, with KOTOR's outlines in red and ours from folders of kit-order bins over it,
            // lined up on KOTOR's chin as the measure compares them: `overlay <data> <out.png> <columns> <reading>
            // <label=bins dir[=r,g,b]>... -- <head>...`. The reading is 0 to 3 (`READINGS`), or "all".
            "overlay" => {
                let out = args[4].clone();
                let columns: usize = args[5].parse().unwrap();
                let reading = args[6].clone();
                let split = args.iter().position(|a| a == "--").unwrap();
                let palette = [[255, 220, 60], [80, 230, 110], [80, 200, 255], [240, 120, 240]];
                let variants: Vec<(String, PathBuf, [u8; 3])> = args[7..split]
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let parts: Vec<&str> = v.split('=').collect();
                        let col = parts.get(2).map(|c| { let c: Vec<u8> = c.split(',').map(|x| x.parse().unwrap()).collect(); [c[0], c[1], c[2]] }).unwrap_or(palette[i % 4]);
                        (parts[0].to_string(), PathBuf::from(parts[1]), col)
                    })
                    .collect();
                let heads: Vec<String> = if args[split + 1] == "all" { names(input) } else { args[split + 1..].to_vec() };
                let mesh = input.head_mesh();
                let readings: Vec<usize> = if reading == "all" { (0..READINGS.len()).collect() } else { vec![reading.parse().unwrap()] };
                let tiles: Vec<image::RgbImage> = parallel(heads.len() * readings.len(), threads(), |q| {
                    let name = &heads[q / readings.len()];
                    let r = readings[q % readings.len()];
                    let (v, f, tint) = kotor_bare(input, name);
                    let mut img = crate::render::clay(&v, &f, &SIDE, &[], &tint);
                    let (k, hidden) = kotor(input, name);
                    let Ok(k) = k else {
                        crate::render::label(&mut img, &format!("{} not read", name.replace('_', " ")));
                        return img;
                    };
                    for line in k.sides(r) {
                        draw(&mut img, &SIDE, &k, line, [235, 60, 50], 1);
                    }
                    let mut text = format!("{} {}", name.replace('_', " "), READINGS[r]);
                    if hidden && r > 0 {
                        text.push_str(" hidden");
                    }
                    for (label, dir, col) in &variants {
                        let Some(p) = read_bin(dir, name) else { continue };
                        let Ok(o) = measure(&local_surface(&p, &mesh.triangles)) else { continue };
                        for line in o.sides(r) {
                            draw(&mut img, &SIDE, &k, line, *col, 0);
                        }
                        if let Some(g) = reading_gap(&k, &o, r) {
                            text.push_str(&format!(" {} {g:.1}", label.chars().next().unwrap()));
                        }
                    }
                    crate::render::label(&mut img, &text);
                    img
                });
                crate::render::sheet(&tiles, columns).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // pairs of KOTOR heads: the first head's bare mesh in clay with its outline in red and the second head's in cyan, lined
            // up on the first one's chin, in the middle and the side view: `pairs <data> <out.png> <a>:<b>...`
            "pairs" => {
                let out = args[4].clone();
                let pairs: Vec<(String, String)> = args[5..].iter().map(|p| { let (a, b) = p.split_once(':').unwrap(); (a.to_string(), b.to_string()) }).collect();
                let tiles: Vec<image::RgbImage> = parallel(pairs.len() * 2, threads(), |q| {
                    let (a, b) = &pairs[q / 2];
                    let r = [0, 3][q % 2];
                    let (v, f, tint) = kotor_bare(input, a);
                    let mut img = crate::render::clay(&v, &f, &SIDE, &[], &tint);
                    let (ka, kb) = (kotor(input, a).0.unwrap(), kotor(input, b).0.unwrap());
                    for line in ka.sides(r) {
                        draw(&mut img, &SIDE, &ka, line, [235, 60, 50], 1);
                    }
                    for line in kb.sides(r) {
                        draw(&mut img, &SIDE, &ka, line, [80, 220, 255], 0);
                    }
                    crate::render::label(&mut img, &format!("{} {} {} {:.1}", a.replace('_', " "), b.replace('_', " "), READINGS[r], reading_gap(&ka, &kb, r).unwrap_or(f64::NAN)));
                    img
                });
                crate::render::sheet(&tiles, 4).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // heads built with the throat construction, written as kit-order bins: `throat-bins <data> <out dir> <values.json> --
            // <head>...|all`. The values file maps a head (or "*") to {"jaw": [5 jaw values] (the live preset when
            // absent), "throat": [depth, corner height, jaw corner rise, roundness]}.
            "throat-bins" => {
                let out = PathBuf::from(&args[4]);
                let table: std::collections::HashMap<String, Built> = serde_json::from_str(&std::fs::read_to_string(&args[5]).unwrap()).unwrap();
                let split = args.iter().position(|a| a == "--").unwrap();
                let heads: Vec<String> = if args[split + 1] == "all" { names(input) } else { args[split + 1..].to_vec() };
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let shape = ThroatShape::new(&h.mesh);
                std::fs::create_dir_all(&out).unwrap();
                let _: Vec<()> = parallel(heads.len(), threads(), |q| {
                    let name = &heads[q];
                    let Some(b) = table.get(name).or_else(|| table.get("*")) else { return };
                    let p = build_throat(&h, &shape, name, b);
                    write_bin(&out, name, &p);
                });
                eprintln!("wrote {}", out.display());
                true
            }
            // KOTOR, then one row per folder of kit-order bins, in clay from the side, the front, three-quarter from below and from
            // below, with each row's turned triangles against the first folder printed and painted red: `clay <data> <out.png>
            // <label=bins dir>... -- <head>...`
            "clay" => {
                let out = args[4].clone();
                let split = args.iter().position(|a| a == "--").unwrap();
                let variants: Vec<(String, PathBuf)> = args[5..split].iter().map(|v| { let (a, b) = v.split_once('=').unwrap(); (a.to_string(), PathBuf::from(b)) }).collect();
                let heads: Vec<String> = args[split + 1..].to_vec();
                let mesh = input.head_mesh();
                let frame = crate::measure::Frame::head();
                let shots = crate::under_jaw::SHOTS;
                // every triangle: the hidden vertices under the jaw carry visible skin
                let shown: Vec<[u32; 3]> = mesh.triangles.clone();
                let all: Vec<u32> = (0..mesh.base.len() as u32).collect();
                let mut tiles = Vec::new();
                for name in &heads {
                    let (v, f, _) = kotor_bare(input, name);
                    let mut row: Vec<image::RgbImage> = parallel(shots.len(), threads(), |s| crate::render::clay(&v, &f, &shots[s], &[], &[]));
                    crate::render::label(&mut row[0], &format!("{} kotor", name.replace('_', " ")));
                    tiles.extend(row);
                    let first: Option<Vec<V3>> = variants.first().and_then(|(_, d)| read_bin(d, name)).map(|p| p.iter().map(|&q| frame.world(q)).collect());
                    for (label, dir) in &variants {
                        let Some(p) = read_bin(dir, name) else { continue };
                        let world: Vec<V3> = p.iter().map(|&q| frame.world(q)).collect();
                        let red = match &first {
                            Some(f0) => crate::check::Checker::from_parts(&shown, &vec![false; f0.len()], f0, &all).marks(&world),
                            None => vec![false; shown.len()],
                        };
                        let count = red.iter().filter(|r| **r).count();
                        eprintln!("{name} {label}: {count} turned triangles");
                        let mut row: Vec<image::RgbImage> = parallel(shots.len(), threads(), |s| crate::render::clay(&p, &shown, &shots[s], &red, &[]));
                        crate::render::label(&mut row[0], label);
                        tiles.extend(row);
                    }
                }
                crate::render::sheet(&tiles, shots.len()).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // solves heads toward KOTOR's outlines: first with the 5 jaw sliders alone ("old"), then with the jaw sliders and the 4
            // throat sliders ("new"), from the live values and the throat's start values. It holds the jaw measure where the live
            // preset has it, holds the chin's front and bottom, and costs every turned triangle. No value is set by hand:
            // `solve <data> <out.json> <head>...`
            "solve" => {
                let out = args[4].clone();
                let heads_named: Vec<String> = args[5..].to_vec();
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let shape = ThroatShape::new(&h.mesh);
                let all: Vec<u32> = (0..h.mesh.base.len() as u32).collect();
                let rows: Vec<serde_json::Value> = parallel(heads_named.len(), threads(), |q| {
                    let name = &heads_named[q];
                    let preset = h.preset(h.index(name));
                    let target = kotor(input, name).0.expect("KOTOR's outlines");
                    let live = h.live_with(name, None);
                    let window = crate::feature::Feature::window(&crate::jaw::JAW);
                    let jaw_of = |p: &[V3]| crate::jaw::measure(&crate::measure::Surface::within(p, &h.mesh.triangles, &h.frame, &window)).ok();
                    let held = jaw_of(&live).expect("the preset's jaw");
                    let local = |p: &[V3]| -> Vec<V3> { p.iter().map(|&a| h.frame.local(a)).collect() };
                    let live_chin = chin(&local_surface(&local(&live), &h.mesh.triangles)).unwrap();
                    let checker = crate::check::Checker::from_parts(&h.mesh.triangles, &vec![false; live.len()], &live, &all);
                    let n = preset.len();
                    let build = |x: &[f64]| -> Vec<V3> {
                        let p = h.live_with(name, Some(&x[..n]));
                        if x.len() > n { shape.apply(&p, &Throat::from_slice(&x[n..])) } else { p }
                    };
                    let parts = |x: &[f64]| -> (f64, f64, f64, usize) {
                        let p = build(x);
                        let surface = local_surface(&local(&p), &h.mesh.triangles);
                        let Ok(m) = measure(&surface) else { return (1e9, 1e9, 1e9, 0) };
                        let g = whole_gap(&target, &m).unwrap_or(1e3);
                        let mut hold = 0.0;
                        match jaw_of(&p) {
                            Some(j) => {
                                for (a, b) in j.proportions.iter().zip(&held.proportions) {
                                    if (a - b).is_finite() {
                                        hold += ((a - b) / 1.5).powi(2);
                                    }
                                }
                            }
                            None => hold += 1e6,
                        }
                        let ch = ((m.pog[0] - live_chin.0[0]).powi(2) + (m.bottom - live_chin.1).powi(2)).sqrt();
                        hold += (ch / 1.0).powi(2);
                        let folds = checker.faults(&p).total();
                        ((g / 0.5).powi(2), hold, g, folds)
                    };
                    let cost = |x: &[f64]| { let (a, b, _, f) = parts(x); a + b + 3.0 * f as f64 };
                    let ranges: Vec<(f64, f64)> = h.jaw_ranges().into_iter().chain(THROAT_SLIDERS.iter().map(|s| (s.1, s.2))).collect();
                    let search = |x0: Vec<f64>| -> Vec<f64> {
                        let mut x = x0;
                        let mut best = cost(&x);
                        // a coarse look at each throat slider over its whole range, then fine steps on every slider
                        for _ in 0..2 {
                            for s in n..x.len() {
                                let (lo, hi) = ranges[s];
                                for k in 0..=6 {
                                    let mut y = x.clone();
                                    y[s] = lo + (hi - lo) * k as f64 / 6.0;
                                    let c = cost(&y);
                                    if c < best {
                                        best = c;
                                        x = y;
                                    }
                                }
                            }
                        }
                        for pass in 0..5 {
                            for s in 0..x.len() {
                                let (lo, hi) = ranges[s];
                                let step = (hi - lo) / 8.0 / 2f64.powi(pass);
                                for dir in [-1.0, 1.0] {
                                    loop {
                                        let mut y = x.clone();
                                        y[s] = (x[s] + dir * step).clamp(lo, hi);
                                        if y[s] == x[s] {
                                            break;
                                        }
                                        let c = cost(&y);
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
                        x
                    };
                    let old = search(preset.clone());
                    let mut x0 = preset.clone();
                    x0.extend(Throat::start().to_vec());
                    let new = search(x0);
                    let report = |x: &[f64]| { let (a, b, g, f) = parts(x); json!({"values": x, "gap": g, "fit": a, "hold": b, "folds": f}) };
                    let r = json!({"name": name, "live": report(&preset), "old": report(&old), "new": report(&new)});
                    eprintln!("{name}: gap live {:.2}, jaw sliders {:.2}, with the throat {:.2}", r["live"]["gap"], r["old"]["gap"], r["new"]["gap"]);
                    r
                });
                std::fs::write(&out, serde_json::to_string_pretty(&json!({"sliders": THROAT_SLIDERS.iter().map(|s| s.0).collect::<Vec<_>>(), "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // one head's outlines and gaps: `probe <data> <head> [label=<bins dir>]...`
            "probe" => {
                let name = &args[4];
                let mesh = input.head_mesh();
                let (k, hidden) = kotor(input, name);
                let k = k.unwrap();
                println!("{name}: chin front {:?} bottom {:.1} scale {:.3} hidden {hidden}", k.pog, k.bottom, k.scale);
                for r in 0..READINGS.len() {
                    println!("  {}: {:?} points", READINGS[r], k.outlines[r].iter().map(|o| o.as_ref().map(|o| o.len())).collect::<Vec<_>>());
                }
                for v in &args[5..] {
                    let (label, dir) = v.split_once('=').unwrap();
                    let p = read_bin(Path::new(dir), name).unwrap();
                    let o = measure(&local_surface(&p, &mesh.triangles)).unwrap();
                    println!("  {label}: {:?} whole {:?}", (0..READINGS.len()).map(|r| reading_gap(&k, &o, r).map(|g| (g * 100.0).round() / 100.0)).collect::<Vec<_>>(), whole_gap(&k, &o));
                }
                let _ = data;
                true
            }
            _ => false,
        }
    }
}

/// The throat's shape, as four sliders. Each sets one place or size of the outline that the construction lays the skin on.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Throat {
    /// Throat depth: how far behind the chin's front the throat's corner lies, in mm. A deeper throat has a longer, level underside.
    pub depth: f64,
    /// Throat corner height: how high the throat's corner lies above the chin's bottom, in mm. Below 0 the underside slopes down
    /// toward the neck; above 0 it rises.
    pub height: f64,
    /// Jaw corner rise: how much higher the outline lies under the jaw's corners than under the chin, in mm.
    pub rise: f64,
    /// Throat roundness: the radius of the bend at the throat's corner, in mm. Small is a sharp corner, large a soft curve.
    pub radius: f64,
}

/// The keys of the throat's sliders in a slider set (`kind` "throat", the key in `measure`), in the order of `Throat::from_slice`.
pub const THROAT_KEYS: [&str; 4] = ["throat_depth", "throat_height", "jaw_corner_rise", "throat_roundness"];

/// The sliders: name, low end, high end, start value (the base head's), and the hint.
pub const THROAT_SLIDERS: [(&str, f64, f64, f64, &str); 4] = [
    ("Throat depth", 45.0, 84.0, 60.0, "How far back under the chin the throat's corner lies. A deeper throat has a longer underside."),
    ("Throat corner height", -12.0, 12.0, 0.0, "How high the throat's corner lies against the bottom of the chin. Low values slope the underside down into the neck."),
    ("Jaw corner rise", 0.0, 28.0, 10.0, "How much higher the line under the jaw runs toward the jaw's corners than under the chin."),
    ("Throat roundness", 6.0, 25.0, 12.0, "The radius of the bend where the underside turns into the neck. Low values make a sharp corner."),
];

impl Throat {
    pub fn from_slice(v: &[f64]) -> Self {
        Self { depth: v[0], height: v[1], rise: v[2], radius: v[3] }
    }

    pub fn to_vec(&self) -> Vec<f64> {
        vec![self.depth, self.height, self.rise, self.radius]
    }

    pub fn start() -> Self {
        Self::from_slice(&THROAT_SLIDERS.map(|s| s.3))
    }

    fn corner(&self) -> crate::throat_corner::Corner {
        // under the jaw's corners the outline rises by the corner's height plus the rise; past `SIDE_TOP` it climbs into the jaw and
        // folds the skin there, so the rise gives way
        let rise = self.rise.min(SIDE_TOP - self.height).max(0.0);
        crate::throat_corner::Corner { back: self.depth, height: self.height, radius: self.radius, side_rise: rise }
    }
}

/// The highest the outline may lie under the jaw's corners, above the chin's bottom, in mm: just past the 57 heads' fits (30.6).
const SIDE_TOP: f64 = 32.0;

/// How much the skin's smoothness counts against reaching the outline, per edge of the mesh. Larger values spread the move more.
const STIFF: f64 = 1.0;
/// How many sweeps the smooth field takes.
const SWEEPS: usize = 400;
/// The place the rays start from, behind the chin's front and above its bottom, in mm, as in `throat_corner`.
const RAY_BACK: f64 = 115.0;
const RAY_UP: f64 = 25.0;
/// Across the jaw: the outline is laid fully within this many mm of the middle, and fades over the next.
const ACROSS_FULL: f64 = 28.0;
const ACROSS_FADE: f64 = 22.0;
/// The share of the outline's span of rays over which its pull eases in at the chin's end.
const EASE: f64 = 0.15;

/// The place of the corner's middle in an outline of `throat_corner::outline`: the chin's end, the first touching place,
/// then the 39 places of the rounded corner, of which this is the 20th.
const CORNER_POINT: usize = 21;

/// The outlines of the sections across the jaw, for one head and one set of throat values.
struct Sections {
    pog: f64,
    bottom: f64,
    reach: f64,
    lines: Vec<Vec<[f64; 2]>>,
    /// How far along each outline the corner's middle lies, in mm.
    corner: Vec<f64>,
}

/// The throat construction: in each section across the jaw, the outline of `throat_corner::outline` (from the skin
/// under the chin, level back to the throat's corner, rounded, and down to the neck's front 34 mm below the chin). Each skin vertex
/// near the outline is pulled onto it along a ray from one place behind and above the throat. The chin, the face above the outline's
/// start, the lips and the neck's rim are held. The move is then spread as one smooth field over the skin between, solved on the
/// mesh's edges, so the skin past the outline's ends fades to the rim with no step.
pub struct ThroatShape {
    frame: crate::measure::Frame,
    mesh: crate::mesh::HeadMesh,
    /// Per merged vertex: free to move (not held, not hidden, near the throat on the base head).
    free: Vec<bool>,
}

impl ThroatShape {
    pub fn new(mesh: &crate::mesh::HeadMesh) -> Self {
        let frame = crate::measure::Frame::head();
        let weight = crate::jaw_edge::ThroatZone::new(mesh).weight;
        let free = (0..mesh.merged_count())
            .map(|g| {
                let i = mesh.first[g] as usize;
                let q = frame.local(mesh.base[i]);
                // the hidden vertices under the jaw carry visible skin: they are laid on the outline like skin, since placing them from
                // their skin neighbours puts them inside a sharp corner
                let under_jaw = !mesh.hidden[g] || q[1] < -100.0;
                weight[i] > 0.0 && under_jaw && !mesh.keep[i] && q[0].abs() < 75.0 && q[1] < -60.0 && q[2] > -110.0
            })
            .collect();
        Self { frame, mesh: mesh.clone(), free }
    }

    /// The construction from a head mesh and its free merged vertices (`free_flags` of a construction made with `new`), for the
    /// browser, which has no zones or skin weights of the mesh.
    pub fn from_parts(mesh: crate::mesh::HeadMesh, free: Vec<bool>) -> Self {
        Self { frame: crate::measure::Frame::head(), mesh, free }
    }

    /// Per merged vertex: whether the construction may move it.
    pub fn free_flags(&self) -> &[bool] {
        &self.free
    }

    pub fn vertex_count(&self) -> usize {
        self.mesh.base.len()
    }

    /// The outline of every section across the jaw, 1 mm apart from `-reach` to `reach`, each with how far along it the corner's
    /// middle lies (mm). Sections that cannot be read take their inner neighbour's outline. None when the middle cannot be read.
    fn sections(&self, surface: &Surface, t: &Throat) -> Option<Sections> {
        let (pog, bottom) = chin(surface)?;
        let (pog, bottom) = (pog[0], bottom);
        let c = t.corner();
        let reach = ACROSS_FULL + ACROSS_FADE;
        let xs: Vec<f64> = (0..=((2.0 * reach) as usize)).map(|k| k as f64 - reach).collect();
        let mut outlines: Vec<Option<Vec<[f64; 2]>>> = xs.iter().map(|&x| crate::throat_corner::outline(surface, pog, bottom, x, &c)).collect();
        let mid = xs.len() / 2;
        outlines[mid].as_ref()?;
        for k in (0..mid).rev() {
            if outlines[k].is_none() {
                outlines[k] = outlines[k + 1].clone();
            }
        }
        for k in mid + 1..xs.len() {
            if outlines[k].is_none() {
                outlines[k] = outlines[k - 1].clone();
            }
        }
        let lines: Vec<Vec<[f64; 2]>> = outlines.into_iter().map(Option::unwrap).collect();
        let corner = lines.iter().map(|l| crate::throat_corner::path_length(&l[..=CORNER_POINT])).collect();
        Some(Sections { pog, bottom, reach, lines, corner })
    }

    /// Per section across the jaw (1 mm apart): its place across, whether its own outline could be read, the outline's start, the
    /// corner's middle and the outline's end on the neck's front (depth and height, head frame, mm). For the tools' probes.
    pub fn section_probe(&self, positions: &[V3], t: &Throat) -> Vec<(f64, bool, [f64; 2], [f64; 2], [f64; 2])> {
        let local: Vec<V3> = positions.iter().map(|&p| self.frame.local(p)).collect();
        let surface = tools_surface(&local, &self.mesh.triangles);
        let Some((pog, bottom)) = chin(&surface).map(|(p, b)| (p[0], b)) else { return Vec::new() };
        let c = t.corner();
        let reach = ACROSS_FULL + ACROSS_FADE;
        (0..=((2.0 * reach) as usize))
            .map(|k| {
                let x = k as f64 - reach;
                match crate::throat_corner::outline(&surface, pog, bottom, x, &c) {
                    Some(l) => (x, true, l[0], l[CORNER_POINT], *l.last().unwrap()),
                    None => (x, false, [f64::NAN; 2], [f64::NAN; 2], [f64::NAN; 2]),
                }
            })
            .collect()
    }

    /// The corner line's places (head frame, mm): the middle of each section's rounded corner, 1 mm apart across the jaw, where the
    /// outline is laid in full or in part.
    pub fn corner_line(&self, positions: &[V3], t: &Throat) -> Vec<V3> {
        let local: Vec<V3> = positions.iter().map(|&p| self.frame.local(p)).collect();
        let surface = tools_surface(&local, &self.mesh.triangles);
        let Some(s) = self.sections(&surface, t) else { return Vec::new() };
        s.lines.iter().enumerate().map(|(k, l)| { let p = l[CORNER_POINT]; [k as f64 - s.reach, p[1], p[0]] }).collect()
    }

    /// The head with the throat built by `t`; `positions` is a built head in metres.
    pub fn apply(&self, positions: &[V3], t: &Throat) -> Vec<V3> {
        let local: Vec<V3> = positions.iter().map(|&p| self.frame.local(p)).collect();
        let surface = tools_surface(&local, &self.mesh.triangles);
        let Some(sec) = self.sections(&surface, t) else { return positions.to_vec() };
        let (pog, bottom, reach) = (sec.pog, sec.bottom, sec.reach);
        let xs: Vec<f64> = (0..sec.lines.len()).map(|k| k as f64 - reach).collect();
        let outlines: Vec<Option<Vec<[f64; 2]>>> = sec.lines.iter().cloned().map(Some).collect();
        let o = [pog - RAY_BACK, bottom + RAY_UP];
        let angle = |p: [f64; 2]| (p[0] - o[0]).atan2(-(p[1] - o[1]));
        let merged = self.mesh.merge(&local);
        let n = merged.len();
        // per merged vertex: what the outline asks (a move in depth and height) and how strongly
        let mut want = vec![[0.0f64; 2]; n];
        let mut weight = vec![0.0f64; n];
        // per merged vertex: the share of its weight that pulls it onto the outline
        let mut share = vec![0.0f64; n];
        for g in 0..n {
            if !self.free[g] {
                continue;
            }
            let q = merged[g];
            let across = 1.0 - crate::jaw_edge::smoothstep((q[0].abs() - ACROSS_FULL) / ACROSS_FADE);
            let f = (q[0] + reach).clamp(0.0, (xs.len() - 1) as f64);
            let i = (f.floor() as usize).min(xs.len() - 2);
            let s = f - i as f64;
            let p = [q[2], q[1]];
            let d = [p[0] - o[0], p[1] - o[1]];
            let r = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let dir = [d[0] / r, d[1] / r];
            let a = angle(p);
            // what one section's outline asks: (move, pull, hold)
            let ask = |line: &Vec<[f64; 2]>| -> ([f64; 2], f64, f64) {
                let (a_chin, a_neck) = (angle(line[0]), angle(*line.last().unwrap()));
                let span = (a_chin - a_neck).abs();
                // toward the chin from the outline's start: the chin and the face, held
                let from_chin = (a_chin - a) * (a_chin - a_neck).signum();
                if from_chin <= 0.0 {
                    return ([0.0, 0.0], 0.0, 1.0);
                }
                let Some((hit, along)) = crate::throat_corner::ray_hit(o, dir, line) else {
                    // past the neck's end of the outline: free, so the skin fades to the rim
                    return ([0.0, 0.0], 0.0, 0.0);
                };
                // only the skin near the outline is pulled; skin far inside it (the back of the neck) is held
                let near = 1.0 - crate::jaw_edge::smoothstep(((hit - r).abs() - 30.0) / 15.0);
                let ease = crate::jaw_edge::smoothstep(from_chin / (EASE * span));
                let target = crate::throat_corner::at_length(line, along);
                ([target[0] - p[0], target[1] - p[1]], near * ease, (1.0 - near) + (1.0 - ease) * near)
            };
            let (m0, w0, h0) = ask(outlines[i].as_ref().unwrap());
            let (m1, w1, h1) = ask(outlines[i + 1].as_ref().unwrap());
            let pull = across * ((1.0 - s) * w0 + s * w1);
            let hold = (1.0 - across) + across * ((1.0 - s) * h0 + s * h1);
            let mv = [(1.0 - s) * m0[0] + s * m1[0], (1.0 - s) * m0[1] + s * m1[1]];
            weight[g] = pull + hold.min(1.0);
            if weight[g] > 0.0 {
                share[g] = pull / weight[g];
            }
            if weight[g] > 0.0 {
                want[g] = [mv[0] * pull / weight[g], mv[1] * pull / weight[g]];
            }
        }
        // the smooth field: each free vertex's move is the mean of what the outline asks and its neighbours' moves, weighted
        let mut u = vec![[0.0f64; 2]; n];
        for _ in 0..SWEEPS {
            for g in 0..n {
                if !self.free[g] {
                    continue;
                }
                let mut sum = [weight[g] * want[g][0], weight[g] * want[g][1]];
                let mut w = weight[g];
                for &j in self.mesh.neighbours_of(g) {
                    let j = j as usize;
                    if self.mesh.hidden[j] && !self.free[j] {
                        continue;
                    }
                    sum[0] += STIFF * u[j][0];
                    sum[1] += STIFF * u[j][1];
                    w += STIFF;
                }
                if w > 0.0 {
                    u[g] = [sum[0] / w, sum[1] / w];
                }
            }
        }
        let mut after: Vec<V3> = merged.iter().zip(&u).map(|(q, m)| [q[0], q[1] + m[1], q[2] + m[0]]).collect();
        self.slide(&sec, o, &share, &mut after);
        let moves: Vec<V3> = after.iter().zip(&merged).map(|(a, b)| crate::sub(*a, *b)).collect();
        let spread = self.mesh.expand(&moves);
        let mut out: Vec<V3> = local.iter().enumerate().map(|(i, &q)| if self.mesh.keep[i] { q } else { crate::add(q, spread[i]) }).collect();
        // the hidden vertices under the jaw carry visible skin: they move as their 6 nearest skin vertices move, as the build places them
        crate::throat_corner::follow_hidden(&self.mesh, &local, &mut out);
        for (i, q) in out.iter_mut().enumerate() {
            let g = self.mesh.group[i] as usize;
            if self.free[g] && self.mesh.hidden[g] && !self.mesh.keep[i] {
                *q = crate::add(local[i], spread[i]);
            }
        }
        out.into_iter().map(|q| self.frame.world(q)).collect()
    }

    /// Slides the skin along each section's outline so that one row of existing vertices lies on the corner's middle. The row is the
    /// path over the mesh's edges, across the throat, whose vertices lie nearest the corner along the outline. Each section is then
    /// remapped along its outline: the row moves onto the corner, and the move fades smoothly within `SLIDE_BAND` mm on each side, so
    /// the vertices keep their order and no new vertex is made. `after` holds the merged vertices (head frame, mm) on the outlines.
    fn slide(&self, sec: &Sections, o: [f64; 2], share: &[f64], after: &mut [V3]) {
        let n = after.len();
        let last = sec.lines.len() - 1;
        let lengths: Vec<f64> = sec.lines.iter().map(|l| crate::throat_corner::path_length(l)).collect();
        // per merged vertex: its two sections and the blend between them, how far along the outline it lies, the corner's place
        // along the outline, and how much of the outline is laid there
        let mut at: Vec<Option<(usize, f64, f64, f64, f64)>> = vec![None; n];
        // the snap of each vertex onto the outline, in depth and height, applied only once the row is found: without a row on the
        // corner, a sharp corner laid across the mesh's rows shows as a sawtooth
        let mut snaps = vec![[0.0f64; 2]; n];
        for g in 0..n {
            if !self.free[g] {
                continue;
            }
            let q = after[g];
            let across = 1.0 - crate::jaw_edge::smoothstep((q[0].abs() - ACROSS_FULL) / ACROSS_FADE);
            if across <= 0.0 {
                continue;
            }
            let f = (q[0] + sec.reach).clamp(0.0, last as f64);
            let i = (f.floor() as usize).min(last - 1);
            let s = f - i as f64;
            let d = [q[2] - o[0], q[1] - o[1]];
            let r = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let dir = [d[0] / r, d[1] / r];
            let (Some((h0, a0)), Some((h1, a1))) = (crate::throat_corner::ray_hit(o, dir, &sec.lines[i]), crate::throat_corner::ray_hit(o, dir, &sec.lines[i + 1])) else { continue };
            if std::env::var("THROAT_DEBUG2").is_ok() && q[0].abs() < 30.0 {
                eprintln!("off {:.1} along {:.1} corner {:.1} x {:.1}", (1.0 - s) * h0 + s * h1 - r, (1.0 - s) * a0 + s * a1, (1.0 - s) * sec.corner[i] + s * sec.corner[i + 1], q[0]);
            }
            // only skin lying on the outline slides: skin far off it (the back of the neck) does not
            if ((1.0 - s) * h0 + s * h1 - r).abs() > 10.0 {
                continue;
            }
            // the smooth field leaves the skin short of a sharp corner by up to 10 mm, by a different amount on each vertex: the skin
            // is laid onto the outline along its ray, as far as the outline pulls it
            let off = (1.0 - s) * h0 + s * h1 - r;
            let snap = SNAP_VALUE() * share[g];
            snaps[g] = [snap * off * dir[0], snap * off * dir[1]];
            at[g] = Some((i, s, (1.0 - s) * a0 + s * a1, (1.0 - s) * sec.corner[i] + s * sec.corner[i + 1], across));
        }
        let snap_all = |after: &mut [V3]| {
            for g in 0..n {
                after[g][2] += snaps[g][0];
                after[g][1] += snaps[g][1];
            }
        };
        if std::env::var("THROAT_NO_SLIDE").is_ok() {
            snap_all(after);
            return;
        }
        // the row: the cheapest path over the edges from one side of the throat to the other, each vertex costing more the further it
        // lies from the corner along the outline, and more the more it changes that distance from the last vertex. Every step goes at least `SLIDE_STEP` mm further across the throat, so the row changes
        // from one row of the mesh to the next along a diagonal edge and two of its vertices never land on the same place
        let dev = |g: usize| at[g].map(|a| a.2 - a.3);
        let candidate = |g: usize| dev(g).is_some_and(|d| d.abs() < SLIDE_PICK) && after[g][0].abs() <= SLIDE_X;
        let mut cost = vec![f64::INFINITY; n];
        let mut prev = vec![usize::MAX; n];
        let mut heap = std::collections::BinaryHeap::new();
        let key = |c: f64| std::cmp::Reverse((c * 1e6) as i64);
        // the row starts among the left-most candidates and ends among the right-most, which lie at up to `SLIDE_X` mm out
        let (lo, hi) = (0..n).filter(|&g| candidate(g)).fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), g| (a.min(after[g][0]), b.max(after[g][0])));
        if hi - lo < 30.0 {
            return;
        }
        for g in (0..n).filter(|&g| candidate(g) && after[g][0] < lo + 4.0) {
            cost[g] = 1.0 + (dev(g).unwrap() / 3.0).powi(2);
            heap.push((key(cost[g]), g));
        }
        let mut end = None;
        while let Some((std::cmp::Reverse(c), g)) = heap.pop() {
            if (c as f64) / 1e6 > cost[g] + 1e-6 {
                continue;
            }
            if after[g][0] > hi - 4.0 {
                end = Some(g);
                break;
            }
            for &j in self.mesh.neighbours_of(g) {
                let j = j as usize;
                if !candidate(j) || after[j][0] < after[g][0] + SLIDE_STEP {
                    continue;
                }
                let cj = cost[g] + 1.0 + (dev(j).unwrap() / 3.0).powi(2) + ((dev(j).unwrap() - dev(g).unwrap()) / 3.0).powi(2);
                if cj < cost[j] {
                    cost[j] = cj;
                    prev[j] = g;
                    heap.push((key(cj), j));
                }
            }
        }
        if std::env::var("THROAT_DEBUG").is_ok() {
            let cands = (0..n).filter(|&g| candidate(g)).count();
            let starts = (0..n).filter(|&g| candidate(g) && after[g][0] < lo + 4.0).count();
            eprintln!("slide: {} on outlines, {cands} candidates, {starts} starts, row found {}", at.iter().filter(|a| a.is_some()).count(), end.is_some());
            let mut c: Vec<(f64, f64, usize)> = (0..n).filter_map(|g| Some((after[g][0], dev(g)?, g))).filter(|c| c.1.abs() < 14.0).collect();
            c.sort_by(|a, b| a.0.total_cmp(&b.0));
            for (x, d, g) in c {
                let nb: Vec<String> = self.mesh.neighbours_of(g).iter().filter(|&&j| dev(j as usize).is_some_and(|d| d.abs() < 14.0)).map(|&j| format!("{:.0}/{:.0}", after[j as usize][0], dev(j as usize).unwrap())).collect();
                eprintln!("  cand {x:6.1} {d:6.1}  nb {}", nb.join(" "));
            }
        }
        let Some(mut g) = end else { return };
        snap_all(after);
        let mut row = vec![g];
        while prev[g] != usize::MAX {
            g = prev[g];
            row.push(g);
        }
        row.reverse();
        let row: Vec<(f64, f64)> = row.iter().map(|&g| (after[g][0], dev(g).unwrap())).collect();
        if std::env::var("THROAT_DEBUG").is_ok() {
            eprintln!("row: {:?}", row.iter().map(|r| format!("{:.1}:{:.1}", r.0, r.1)).collect::<Vec<_>>());
        }
        // the row's distance from the corner along the outline at any place across: from the row's edge spanning it, or its end
        let row_dev = |x: f64, d: f64| -> f64 {
            let mut best: Option<(f64, f64)> = None;
            for w in row.windows(2) {
                let ((xa, da), (xb, db)) = (w[0], w[1]);
                let (lo, hi) = if xa < xb { (xa, xb) } else { (xb, xa) };
                if x < lo || x > hi {
                    continue;
                }
                let t = if hi > lo { (x - xa) / (xb - xa) } else { 0.5 };
                let v = da + t * (db - da);
                if best.is_none_or(|b| (d - v).abs() < b.1) {
                    best = Some((v, (d - v).abs()));
                }
            }
            best.map_or_else(|| if x < row[0].0 { row[0].1 } else { row[row.len() - 1].1 }, |b| b.0)
        };
        for g in 0..n {
            let Some((i, s, along, corner, across)) = at[g] else { continue };
            let d = along - corner;
            let rd = row_dev(after[g][0], d);
            let t = (d - rd) / SLIDE_BAND;
            if t.abs() >= 1.0 {
                continue;
            }
            let w = (1.0 - t * t).powi(2);
            let to = (along - across * rd * w).clamp(0.0, lengths[i].min(lengths[i + 1]));
            let place = |l: f64| {
                let (p0, p1) = (crate::throat_corner::at_length(&sec.lines[i], l), crate::throat_corner::at_length(&sec.lines[i + 1], l));
                [(1.0 - s) * p0[0] + s * p1[0], (1.0 - s) * p0[1] + s * p1[1]]
            };
            let (from, now) = (place(along), place(to));
            after[g][2] += now[0] - from[0];
            after[g][1] += now[1] - from[1];
        }
    }
}

/// How much of the gap to the outline the snap closes, times the outline's share of each vertex's weight. `THROAT_SNAP` overrides it.
static SNAP_CELL: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
#[allow(non_snake_case)]
fn SNAP_VALUE() -> f64 {
    *SNAP_CELL.get_or_init(|| std::env::var("THROAT_SNAP").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0))
}

/// The slide: vertices within this many mm of the corner along the outline may be picked for the row on the corner.
const SLIDE_PICK: f64 = 15.0;
/// The row runs across the throat to this many mm from the middle on each side.
const SLIDE_X: f64 = 40.0;
/// The slide fades out over this many mm along the outline on each side of the row.
const SLIDE_BAND: f64 = 26.0;
/// Each step of the row goes at least this many mm across the throat.
const SLIDE_STEP: f64 = 2.0;

fn tools_surface(v: &[V3], f: &[[u32; 3]]) -> Surface {
    crate::under_jaw::local_surface(v, f)
}
