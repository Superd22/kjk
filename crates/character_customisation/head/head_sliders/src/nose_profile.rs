//! The nose seen from the side, read finely enough to see a local bend in the bridge (2026-11-21, the brief `nose-profile`).
//!
//! The profile is the front-most skin in the plane between the eyes, read every 0.25 mm of height in the head's frame.

use crate::V3;
use crate::measure::{Frame, STEP, Surface, Window};

/// The heights the profile is read over, in mm in the head's frame: from well above the brow to well under the tip.
pub const READ_FROM: f64 = -85.0;
pub const READ_TO: f64 = 30.0;

const WINDOW: Window = Window { x: 12.0, y: [READ_FROM - 2.0, READ_TO + 2.0], z: -25.0 };

/// The midline profile as (height, depth) pairs in mm, every 0.25 mm from `READ_FROM` to `READ_TO`, depth NaN where there is no skin.
pub fn profile(positions: &[V3], triangles: &[[u32; 3]], frame: &Frame) -> Vec<[f64; 2]> {
    let surface = Surface::within(positions, triangles, frame, &WINDOW);
    let count = ((READ_TO - READ_FROM) / STEP).round() as usize + 1;
    let z = surface.front(0, 0.0, 1, READ_FROM, count);
    z.iter().enumerate().map(|(k, &z)| [READ_FROM + k as f64 * STEP, z]).collect()
}

/// The profile's light blur, as the nose's measure blurs its midline (mm).
const BLUR: f64 = 0.75;
/// The length of the straight lines fitted on each side of a point to read the turn there (mm).
const CORNER_REACH: f64 = 8.0;
/// The spacing of the points on KOTOR's profile where the gaps are read (mm of arc).
pub const GAP_STEP: f64 = 1.0;
/// The gaps start at this height (mm above the eye line), or at the brow's front if it is lower. Above it the brow's own shape
/// decides the profile, which the nose's sliders do not move.
pub const SCORE_TOP: f64 = 10.0;
/// The gaps on the bridge stop this far above the tip, along the profile (mm).
const BRIDGE_END: f64 = 6.0;

/// The fine side profile of a nose.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Fine {
    /// The whole midline read, blurred, as (height, depth), top first, every 0.25 mm of height where there is skin.
    pub line: Vec<[f64; 2]>,
    /// The profile from the brow's front down to the base of the nose, every 0.25 mm along it, as (height, depth).
    pub curve: Vec<[f64; 2]>,
    /// The length along `curve` from the brow's front to the tip (mm).
    pub arc_tip: f64,
    pub brow: [f64; 2],
    pub tip: [f64; 2],
    pub base: [f64; 2],
    /// Where the profile turns forward most sharply, between the brow and 8 mm above the tip: the turn (degrees, between the lines
    /// fitted to the 8 mm above and the 8 mm below) and its height.
    pub corner: [f64; 2],
}

/// The fine profile read from a surface in the head's frame.
pub fn read(surface: &Surface) -> Result<Fine, &'static str> {
    let count = ((READ_TO - READ_FROM) / STEP).round() as usize + 1;
    let raw = crate::measure::Cut { from: READ_FROM, z: surface.front(0, 0.0, 1, READ_FROM, count) };
    let mid = raw.blurred(BLUR);
    let y = |k: usize| READ_FROM + k as f64 * STEP;
    let best = |lo: f64, hi: f64| -> Option<usize> {
        (0..count).filter(|&k| y(k) > lo && y(k) < hi && !mid.z[k].is_nan()).max_by(|&a, &b| mid.z[a].total_cmp(&mid.z[b]))
    };
    let tip = best(-85.0, -20.0).ok_or("no tip")?;
    let brow = best(4.0, 25.0).ok_or("no brow")?;
    // the base: going down from the tip, the deepest point before the profile comes forward again by 1 mm, within 25 mm
    let (mut k, mut base) = (tip, tip);
    while k > 0 && !mid.z[k - 1].is_nan() && y(k) > y(tip) - 25.0 {
        k -= 1;
        if mid.z[k] < mid.z[base] {
            base = k;
        } else if mid.z[k] > mid.z[base] + 1.0 {
            break;
        }
    }
    if (base..=brow).any(|k| mid.z[k].is_nan()) {
        return Err("a gap in the profile");
    }
    let line: Vec<[f64; 2]> = (0..count).rev().filter(|&k| !mid.z[k].is_nan()).map(|k| [y(k), mid.z[k]]).collect();
    let polyline: Vec<[f64; 2]> = (base..=brow).rev().map(|k| [y(k), mid.z[k]]).collect();
    let (curve, arcs) = resample(&polyline, STEP);
    let arc_tip = arcs[brow - tip];
    let corner = corner(&curve, arc_tip);
    Ok(Fine { line, curve, arc_tip, brow: [y(brow), mid.z[brow]], tip: [y(tip), mid.z[tip]], base: [y(base), mid.z[base]], corner })
}

/// A polyline resampled every `step` mm along it, and the arc length at each of the polyline's own points.
fn resample(p: &[[f64; 2]], step: f64) -> (Vec<[f64; 2]>, Vec<f64>) {
    let mut arcs = vec![0.0];
    for w in p.windows(2) {
        arcs.push(arcs.last().unwrap() + ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt());
    }
    let total = *arcs.last().unwrap();
    let mut out = Vec::new();
    let mut j = 0;
    let mut s = 0.0;
    while s <= total {
        while j + 2 < arcs.len() && arcs[j + 1] < s {
            j += 1;
        }
        let t = if arcs[j + 1] > arcs[j] { ((s - arcs[j]) / (arcs[j + 1] - arcs[j])).clamp(0.0, 1.0) } else { 0.0 };
        out.push([p[j][0] + t * (p[j + 1][0] - p[j][0]), p[j][1] + t * (p[j + 1][1] - p[j][1])]);
        s += step;
    }
    (out, arcs)
}

/// The direction of the straight line fitted to some points (degrees; 0 is straight down, more is turned forward).
fn direction(p: &[[f64; 2]]) -> f64 {
    let n = p.len() as f64;
    let c = [p.iter().map(|q| q[0]).sum::<f64>() / n, p.iter().map(|q| q[1]).sum::<f64>() / n];
    let (mut a, mut b, mut d) = (0.0, 0.0, 0.0);
    for q in p {
        let (u, v) = (q[0] - c[0], q[1] - c[1]);
        a += u * u;
        b += u * v;
        d += v * v;
    }
    let theta = 0.5 * (2.0 * b).atan2(a - d);
    let (mut dy, mut dz) = (theta.cos(), theta.sin());
    if dy > 0.0 {
        dy = -dy;
        dz = -dz;
    }
    dz.atan2(-dy).to_degrees()
}

fn corner(curve: &[[f64; 2]], arc_tip: f64) -> [f64; 2] {
    let n = (CORNER_REACH / STEP) as usize;
    let mut best = [f64::NEG_INFINITY, f64::NAN];
    for i in n..curve.len().saturating_sub(n) {
        let s = i as f64 * STEP;
        if s < 2.0 || s > arc_tip - CORNER_REACH {
            continue;
        }
        let turn = direction(&curve[i..=i + n]) - direction(&curve[i - n..=i]);
        if turn > best[0] {
            best = [turn, curve[i][0]];
        }
    }
    best
}

/// The gaps from KOTOR's profile to ours: at every `GAP_STEP` mm along KOTOR's curve, the distance to the nearest point of our whole
/// midline, positive where ours stands out in front of KOTOR's (along KOTOR's outward normal). The second list says which gaps lie on
/// the bridge, from the brow's front (or 10 mm above the eye line) to 6 mm above the tip.
pub fn gaps(kotor: &Fine, ours: &Fine) -> (Vec<f64>, Vec<bool>) {
    let every = (GAP_STEP / STEP).round() as usize;
    let mut g = Vec::new();
    let mut bridge = Vec::new();
    let c = &kotor.curve;
    for i in (0..c.len()).step_by(every) {
        let p = c[i];
        if p[0] > SCORE_TOP {
            continue;
        }
        let (a, b) = (c[i.saturating_sub(1)], c[(i + 1).min(c.len() - 1)]);
        let t = [b[0] - a[0], b[1] - a[1]];
        let l = (t[0] * t[0] + t[1] * t[1]).sqrt().max(1e-9);
        let normal = [t[1] / l, -t[0] / l];
        let mut best = (f64::INFINITY, [0.0, 0.0]);
        for w in ours.line.windows(2) {
            let d = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
            let dd = d[0] * d[0] + d[1] * d[1];
            let s = if dd > 0.0 { (((p[0] - w[0][0]) * d[0] + (p[1] - w[0][1]) * d[1]) / dd).clamp(0.0, 1.0) } else { 0.0 };
            let q = [w[0][0] + s * d[0], w[0][1] + s * d[1]];
            let e = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2);
            if e < best.0 {
                best = (e, q);
            }
        }
        let side = (best.1[0] - p[0]) * normal[0] + (best.1[1] - p[1]) * normal[1];
        g.push(best.0.sqrt() * if side < 0.0 { -1.0 } else { 1.0 });
        bridge.push((i as f64 * STEP) < kotor.arc_tip - BRIDGE_END);
    }
    (g, bridge)
}

/// The length along KOTOR's profile over which the swing of the gaps is read (mm).
pub const SWING_REACH: f64 = 8.0;

/// The scores of the gaps: the root mean square over the bridge and over the whole profile, the largest gap on the bridge, and the
/// swing, the largest change of the gap within 8 mm along the bridge. A profile that is only moved scores no swing. A bend that KOTOR
/// does not have, or a bend KOTOR has that ours lacks, scores a swing.
pub fn scores(g: &[f64], bridge: &[bool]) -> [f64; 4] {
    let rms = |f: &dyn Fn(usize) -> bool| {
        let v: Vec<f64> = (0..g.len()).filter(|&i| f(i)).map(|i| g[i] * g[i]).collect();
        (v.iter().sum::<f64>() / v.len().max(1) as f64).sqrt()
    };
    let worst = (0..g.len()).filter(|&i| bridge[i]).map(|i| g[i].abs()).fold(0.0, f64::max);
    let reach = (SWING_REACH / GAP_STEP).round() as usize;
    let on: Vec<f64> = (0..g.len()).filter(|&i| bridge[i]).map(|i| g[i]).collect();
    let mut swing: f64 = 0.0;
    for i in 0..on.len() {
        let w = &on[i..(i + reach + 1).min(on.len())];
        let hi = w.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let lo = w.iter().copied().fold(f64::INFINITY, f64::min);
        swing = swing.max(hi - lo);
    }
    [rms(&|i| bridge[i]), rms(&|_| true), worst, swing]
}

#[cfg(not(target_arch = "wasm32"))]
pub mod tools {
    use super::*;
    use crate::feature::by_key;
    use crate::heads::World;
    use crate::mesh::input::Input;
    use crate::set::{SMOOTHING, SliderSet};
    use std::path::Path;

    fn read_f32(path: &str) -> Vec<f32> {
        let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
    }

    /// `sliders noseprofile <command> <data> ...`; returns false for an unknown command.
    pub fn run(command: &str, args: &[String], data: &Path, input: &Input) -> bool {
        match command {
            "dump" => dump(args, data, input),
            "ablate" => ablate(args, data, input),
            "checks" => checks(args, data, input),
            "solve" => solve_all(args, data, input),
            "make" => make(args, data, input),
            "ends" => ends(args, data, input),
            "pictures" => pictures(args, data, input),
            _ => return false,
        }
        true
    }

    /// `sliders noseprofile dump <data> <out.json> [name=heads.bin ...]`: every head's midline profile on KOTOR's mesh, the fit, the nose
    /// stand-in, the stand-in with the nose preset (values from `NOSE_VALUES`, a dump of viewer 2's values), and each extra file of
    /// 57 built heads (float32, 4,750 vertices each, the fit's frame).
    fn dump(args: &[String], data: &Path, input: &Input) {
        let world = World::load(input, by_key("nose").unwrap());
        let set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/sliders_r2.json")).unwrap()).unwrap();
        let values: Option<serde_json::Value> =
            std::env::var("NOSE_VALUES").ok().map(|p| serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap());
        let extras: Vec<(String, Vec<f32>)> = args[5..]
            .iter()
            .map(|a| {
                let (n, p) = a.split_once('=').expect("name=file");
                (n.to_string(), read_f32(p))
            })
            .collect();
        let tris = &world.mesh.triangles;
        let vc = world.mesh.base.len();
        let mut out = serde_json::Map::new();
        for (i, h) in world.heads.iter().enumerate() {
            let frame = Frame::head();
            let mut m = serde_json::Map::new();
            let put = |m: &mut serde_json::Map<String, serde_json::Value>, k: &str, p: Vec<[f64; 2]>| {
                m.insert(k.into(), serde_json::json!(p.iter().map(|q| if q[1].is_nan() { serde_json::Value::Null } else { serde_json::json!(q[1]) }).collect::<Vec<_>>()));
            };
            put(&mut m, "kotor", profile(&h.kotor_v, &h.kotor_f, &frame));
            let (sv, sf, _) = crate::checks::butterfly(&h.kotor_v, &h.kotor_f);
            let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
            put(&mut m, "kotor_sub", profile(&sv, &sf, &frame));
            if input.has(&format!("heads/{}/fit_smooth", h.name)) {
                put(&mut m, "fit", profile(&input.points(&format!("heads/{}/fit_smooth", h.name)), tris, &frame));
            }
            put(&mut m, "standin", profile(&world.smoother.smooth(&h.start, SMOOTHING), tris, &frame));
            if let Some(v) = &values {
                let nose: Vec<f64> = v["values"][&h.name].as_array().unwrap()[..set.sliders.len()].iter().map(|x| x.as_f64().unwrap()).collect();
                put(&mut m, "preset", profile(&set.build(&world.smoother, &h.start, &nose), tris, &frame));
            }
            for (n, flat) in &extras {
                let p: Vec<V3> = flat[i * vc * 3..(i + 1) * vc * 3].chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect();
                put(&mut m, n, profile(&p, tris, &frame));
            }
            out.insert(h.name.clone(), serde_json::Value::Object(m));
        }
        let doc = serde_json::json!({ "from": READ_FROM, "step": STEP, "heads": out });
        std::fs::write(&args[4], doc.to_string()).unwrap();
        eprintln!("wrote {}", args[4]);
    }

    /// `sliders noseprofile ablate <data> <head> ...` with `NOSE_VALUES`: each nose slider's own share of the preset's profile (the
    /// preset minus the preset with that slider at 0), at a few heights.
    fn ablate(args: &[String], data: &Path, input: &Input) {
        let world = World::load(input, by_key("nose").unwrap());
        let set: SliderSet = serde_json::from_str(&std::fs::read_to_string(std::env::var("SLIDERS_SET").map(std::path::PathBuf::from).unwrap_or_else(|_| data.join("web/sliders_r2.json"))).unwrap()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(std::env::var("NOSE_VALUES").unwrap()).unwrap()).unwrap();
        let ys = [8.0, 5.0, 3.0, 1.0, -1.0, -3.0, -5.0, -7.0, -9.0, -12.0, -16.0, -20.0];
        let frame = Frame::head();
        for name in &args[4..] {
            let h = world.heads.iter().find(|h| &h.name == name).unwrap();
            let vals: Vec<f64> = v["values"][name].as_array().unwrap()[..set.sliders.len()].iter().map(|x| x.as_f64().unwrap()).collect();
            let at = |vals: &[f64]| {
                let p = profile(&set.build(&world.smoother, &h.start, vals), &world.mesh.triangles, &frame);
                ys.map(|y| p[((y - READ_FROM) / STEP).round() as usize][1])
            };
            let full = at(&vals);
            println!("{name}\n{:>16} {}", "y", ys.map(|y| format!("{y:6.0}")).join(""));
            println!("{:>16} {}", "preset", full.map(|z| format!("{z:6.2}")).join(""));
            for k in 0..vals.len() {
                let mut w = vals.clone();
                w[k] = 0.0;
                let z = at(&w);
                println!("{:>16} {}", set.sliders[k].name, (0..ys.len()).map(|i| format!("{:6.2}", full[i] - z[i])).collect::<String>());
            }
        }
    }

    fn read_mesh(v: &[V3], f: &[[u32; 3]]) -> Result<Fine, &'static str> {
        read(&Surface::within(v, f, &Frame::head(), &WINDOW))
    }

    fn built(flat: &[f32], i: usize, vc: usize) -> Vec<V3> {
        flat[i * vc * 3..(i + 1) * vc * 3].chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect()
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

    fn sd(v: &[f64]) -> f64 {
        let v: Vec<f64> = v.iter().copied().filter(|x| !x.is_nan()).collect();
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    }

    /// `sliders noseprofile checks <data> <out.json> [name=heads.bin ...]`: the four checks of the fine profile, and every head's KOTOR
    /// profile, KOTOR's subdivided copy and each given set of built heads with their gaps to KOTOR, for the pictures.
    fn checks(args: &[String], _data: &Path, input: &Input) {
        let mesh = input.head_mesh();
        let vc = mesh.base.len();
        let names: Vec<String> = input.manifest["heads"].as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect();
        let extras: Vec<(String, Vec<f32>)> = args[5..]
            .iter()
            .map(|a| {
                let (n, p) = a.split_once('=').expect("name=file");
                (n.to_string(), read_f32(p))
            })
            .collect();
        struct Row {
            kotor: Result<Fine, &'static str>,
            sub: Result<Fine, &'static str>,
            rounded: Result<Fine, &'static str>,
            built: Vec<Result<Fine, &'static str>>,
        }
        let rows: Vec<Row> = crate::heads::parallel(names.len(), crate::heads::threads(), |i| {
            let n = &names[i];
            let v = input.points(&format!("heads/{n}/kotor_v"));
            let f = input.triangles(&format!("heads/{n}/kotor_f"));
            let (sv, sf, _) = crate::checks::butterfly(&v, &f);
            let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
            let (lv, lf, _) = crate::checks::loop_subdivide(&v, &f);
            Row {
                kotor: read_mesh(&v, &f),
                sub: read_mesh(&sv, &sf),
                rounded: read_mesh(&lv, &lf),
                built: extras.iter().map(|(_, flat)| read_mesh(&built(flat, i, vc), &mesh.triangles)).collect(),
            }
        });
        let ok: Vec<usize> = (0..names.len()).filter(|&i| rows[i].kotor.is_ok() && rows[i].sub.is_ok()).collect();
        for i in 0..names.len() {
            if let Err(e) = &rows[i].kotor {
                eprintln!("{}: KOTOR's profile cannot be read: {e}", names[i]);
            }
        }
        let k = |i: usize| rows[i].kotor.as_ref().unwrap();
        // check 1: KOTOR against its subdivided copy, and against its rounded copy
        let mut wob = vec![Vec::new(); 7];
        for &i in &ok {
            let s = rows[i].sub.as_ref().unwrap();
            let (g, b) = gaps(k(i), s);
            let sc = scores(&g, &b);
            wob[0].push(sc[0]);
            wob[1].push(sc[1]);
            wob[2].push(sc[2]);
            wob[6].push(sc[3]);
            wob[3].push((k(i).corner[0] - s.corner[0]).abs());
            wob[4].push((k(i).corner[1] - s.corner[1]).abs());
            if let Ok(r) = &rows[i].rounded {
                wob[5].push(scores(&gaps(k(i), r).0, &gaps(k(i), r).1)[0]);
            }
        }
        // KOTOR's spread: every pair of heads
        let mut pair = vec![Vec::new(); 4];
        let mut nearest_ok = 0;
        let mut nearest = Vec::new();
        for &i in &ok {
            let mut best = (f64::INFINITY, 0);
            for &j in &ok {
                let sc = scores(&gaps(k(i), k(j)).0, &gaps(k(i), k(j)).1);
                if j != i {
                    for c in 0..4 {
                        pair[c].push(sc[c]);
                    }
                }
                // check 3: the rounded copy's nearest KOTOR head
                if let Ok(r) = &rows[i].rounded {
                    let (g, b) = gaps(k(j), r);
                    let x = scores(&g, &b)[0];
                    if x < best.0 {
                        best = (x, j);
                    }
                }
            }
            if best.1 == i {
                nearest_ok += 1;
            }
            nearest.push(serde_json::json!({"head": names[i], "nearest": names[best.1], "gap": best.0}));
        }
        let angles: Vec<f64> = ok.iter().map(|&i| k(i).corner[0]).collect();
        let heights: Vec<f64> = ok.iter().map(|&i| k(i).corner[1]).collect();
        eprintln!("check 1, KOTOR against its subdivided copy (median, 95th percentile), against KOTOR's spread:");
        eprintln!("  bridge gap rms   {:5.2} {:5.2} mm, between two heads median {:5.2} mm", median(wob[0].clone()), p95(wob[0].clone()), median(pair[0].clone()));
        eprintln!("  whole gap rms    {:5.2} {:5.2} mm, between two heads median {:5.2} mm", median(wob[1].clone()), p95(wob[1].clone()), median(pair[1].clone()));
        eprintln!("  bridge worst gap {:5.2} {:5.2} mm, between two heads median {:5.2} mm", median(wob[2].clone()), p95(wob[2].clone()), median(pair[2].clone()));
        eprintln!("  bridge swing     {:5.2} {:5.2} mm, between two heads median {:5.2} mm", median(wob[6].clone()), p95(wob[6].clone()), median(pair[3].clone()));
        eprintln!("  corner turn      {:5.1} {:5.1} deg, spread (sd) {:5.1}, median {:5.1}", median(wob[3].clone()), p95(wob[3].clone()), sd(&angles), median(angles.clone()));
        eprintln!("  corner height    {:5.2} {:5.2} mm, spread (sd) {:5.2}, median {:5.2}", median(wob[4].clone()), p95(wob[4].clone()), sd(&heights), median(heights.clone()));
        eprintln!("  rounded copy (Loop) bridge gap rms median {:5.2}, p95 {:5.2}", median(wob[5].clone()), p95(wob[5].clone()));
        eprintln!("check 3: the rounded copy's nearest KOTOR head is itself for {nearest_ok} of {}", ok.len());
        let mut heads = serde_json::Map::new();
        for i in 0..names.len() {
            let mut h = serde_json::Map::new();
            let fine = |f: &Result<Fine, &'static str>| match f {
                Ok(f) => serde_json::json!({"line": f.line, "curve": f.curve, "arc_tip": f.arc_tip, "brow": f.brow, "tip": f.tip, "base": f.base, "corner": f.corner}),
                Err(e) => serde_json::json!({"error": e}),
            };
            h.insert("kotor".into(), fine(&rows[i].kotor));
            h.insert("kotor_sub".into(), fine(&rows[i].sub));
            for (e, (n, _)) in extras.iter().enumerate() {
                let mut v = fine(&rows[i].built[e]);
                if let (Ok(kk), Ok(b)) = (&rows[i].kotor, &rows[i].built[e]) {
                    let (g, br) = gaps(kk, b);
                    v["gaps"] = serde_json::json!(g);
                    v["bridge"] = serde_json::json!(br);
                    v["scores"] = serde_json::json!(scores(&g, &br));
                }
                h.insert(n.clone(), v);
            }
            heads.insert(names[i].clone(), serde_json::Value::Object(h));
        }
        let doc = serde_json::json!({
            "check1": {"bridge_rms": [median(wob[0].clone()), p95(wob[0].clone())], "whole_rms": [median(wob[1].clone()), p95(wob[1].clone())],
                       "bridge_worst": [median(wob[2].clone()), p95(wob[2].clone())], "bridge_swing": [median(wob[6].clone()), p95(wob[6].clone())], "corner_turn": [median(wob[3].clone()), p95(wob[3].clone())],
                       "corner_height": [median(wob[4].clone()), p95(wob[4].clone())], "rounded_bridge_rms": [median(wob[5].clone()), p95(wob[5].clone())]},
            "spread": {"bridge_rms_pairs": median(pair[0].clone()), "whole_rms_pairs": median(pair[1].clone()), "bridge_worst_pairs": median(pair[2].clone()), "bridge_swing_pairs": median(pair[3].clone()),
                       "corner_turn_sd": sd(&angles), "corner_height_sd": sd(&heights)},
            "check3": {"self": nearest_ok, "of": ok.len(), "rows": nearest},
            "heads": heads,
        });
        std::fs::write(&args[4], doc.to_string()).unwrap();
        eprintln!("wrote {}", args[4]);
    }

    /// The pass mark of the fine profile's gaps in the solve (mm): a gap of this size on every point costs as much as one proportion
    /// missed by its mark. `NP_MARK` overrides it.
    const MARK: f64 = 0.4;

    fn mark() -> f64 {
        std::env::var("NP_MARK").ok().and_then(|v| v.parse().ok()).unwrap_or(MARK)
    }

    struct Fit {
        values: Vec<f64>,
        cost: f64,
    }

    /// The nose rows of the kit's solve (`solve::gaps`) and the fine profile's gaps, each over its mark.
    /// With `NP_VIEWER`, a head is measured as viewer 2 builds it: viewer 2's head with every nose slider at 0, plus the nose sliders'
    /// smoothed movement. The curve of the eyes and the other features stay as viewer 2 has them.
    struct Shown {
        base: Vec<V3>,
        zero: Vec<V3>,
        triangles: Vec<[u32; 3]>,
    }

    thread_local! {
        static SHOWN: std::cell::RefCell<Option<std::sync::Arc<Shown>>> = const { std::cell::RefCell::new(None) };
    }

    fn surface_of(ev: &crate::solve::Evaluator, v: &[f64]) -> Surface {
        let shown = SHOWN.with(|s| s.borrow().clone());
        match shown {
            None => ev.surface(v),
            Some(sh) => {
                let mut full = sh.base.clone();
                for ((&i, p), z) in ev.vertices.iter().zip(ev.positions(v)).zip(&sh.zero) {
                    for c in 0..3 {
                        full[i as usize][c] += p[c] - z[c];
                    }
                }
                Surface::within(&full, &sh.triangles, &ev.frame, &ev.feature.window())
            }
        }
    }

    fn rows(ev: &crate::solve::Evaluator, target: &crate::feature::Measured, fine: &Fine, marks: &crate::feature::Scale, v: &[f64], spread: &[f64]) -> Option<Vec<f64>> {
        let surface = surface_of(ev, v);
        let m = ev.feature.measure(&surface).ok()?;
        let ours = read(&surface).ok()?;
        let mut r = crate::solve::gaps(ev.feature, &m, target, marks);
        let (g, _) = gaps(fine, &ours);
        let k = 1.0 / (mark() * (g.len() as f64).sqrt());
        r.extend(g.iter().map(|x| x * k));
        r.extend(v.iter().zip(spread).map(|(x, s)| PRIOR.sqrt() * x / s));
        Some(r)
    }

    const PRIOR: f64 = 0.01;

    /// Levenberg and Marquardt's method, as `solve::solve_within`, on `rows`.
    fn descend(ev: &crate::solve::Evaluator, target: &crate::feature::Measured, fine: &Fine, marks: &crate::feature::Scale, spread: &[f64], start: &[f64], bounds: &[[f64; 2]]) -> Fit {
        let k = start.len();
        let mut v: Vec<f64> = start.iter().zip(bounds).map(|(x, b)| x.clamp(b[0], b[1])).collect();
        let cost = |r: &[f64]| r.iter().map(|x| x * x).sum::<f64>();
        let Some(mut r) = rows(ev, target, fine, marks, &v, spread) else { return Fit { values: v, cost: f64::INFINITY } };
        let mut c = cost(&r);
        let mut mu = 1e-2;
        for _ in 0..40 {
            let m = r.len();
            let mut jac = vec![vec![0.0; k]; m];
            for j in 0..k {
                let h = 0.05 * spread[j];
                let h = if v[j] + h > bounds[j][1] { -h } else { h };
                let mut w = v.clone();
                w[j] += h;
                if let Some(rj) = rows(ev, target, fine, marks, &w, spread) {
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
                    a[(d, d)] += mu * (jtj[(d, d)] + 1e-9);
                }
                let Some(step) = a.cholesky().map(|ch| ch.solve(&(-&jtr))) else {
                    mu *= 10.0;
                    continue;
                };
                let w: Vec<f64> = v.iter().zip(step.iter()).zip(bounds).map(|((x, s), b)| (x + s).clamp(b[0], b[1])).collect();
                if let Some(rw) = rows(ev, target, fine, marks, &w, spread) {
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

    /// `sliders noseprofile solve <data> <out.json>` with `NOSE_VALUES` (viewer 2's values): every head solved with the 14 nose sliders
    /// against the nose's rows and the fine profile, from its live values, then guarded. A head keeps the largest share of its move
    /// (1, 0.75, 0.5, 0.25) that passes: no more folded triangles near the nose than live, no more than two of the nose's 17 rows worse
    /// than live by more than half their pass mark, and on the fine profile a bridge gap at least 0.1 mm smaller and a swing at most 0.25 mm larger (its wobble is 0.75 mm). `NP_HEADS=a,b` solves
    /// only those heads. `SLIDERS_SET` names another slider set.
    fn solve_all(args: &[String], data: &Path, input: &Input) {
        let world = World::load(input, by_key("nose").unwrap());
        let f = world.feature;
        let set_path = std::env::var("SLIDERS_SET").map(std::path::PathBuf::from).unwrap_or_else(|_| data.join("web/sliders_r2.json"));
        let set: SliderSet = serde_json::from_str(&std::fs::read_to_string(&set_path).unwrap()).unwrap();
        let live_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/sliders_r2.json")).unwrap()).unwrap();
        let m = crate::checks::Marks::read(&data.join(f.files().marks));
        let spread: Vec<f64> = set
            .sliders
            .iter()
            .map(|s| if s.measure.is_empty() { ROOT_PLACE_SPREAD } else { m.spread_p[crate::feature::proportion_index(f, &s.measure)] })
            .collect();
        let bounds: Vec<[f64; 2]> = set.sliders.iter().map(|s| [s.low, s.high]).collect();
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(std::env::var("NOSE_VALUES").unwrap()).unwrap()).unwrap();
        let only: Option<Vec<String>> = std::env::var("NP_HEADS").ok().map(|s| s.split(',').map(String::from).collect());
        let viewer: Option<Vec<f32>> = std::env::var("NP_VIEWER").ok().map(|p| read_f32(&p));
        let smoothed = crate::solve::Smoothed::new(&set, &world.smoother);
        let live_smoothed = crate::solve::Smoothed::new(&live_set, &world.smoother);
        let window = f.window();
        let np = f.proportions().len();
        let t0 = std::time::Instant::now();
        let out = crate::heads::parallel(world.heads.len(), crate::heads::threads(), |i| {
            let h = &world.heads[i];
            if only.as_ref().is_some_and(|o| !o.contains(&h.name)) {
                return None;
            }
            let kotor_fine = read_mesh(&h.kotor_v, &h.kotor_f).ok()?;
            let live_vals: Vec<f64> = v["values"][&h.name].as_array().unwrap()[..live_set.sliders.len()].iter().map(|x| x.as_f64().unwrap()).collect();
            // the live values in this set's order, by name; a new slider starts at 0
            let start: Vec<f64> = set.sliders.iter().map(|s| live_set.sliders.iter().position(|l| l.name == s.name).map_or(0.0, |j| live_vals[j])).collect();
            let ev = crate::solve::Evaluator::with(f, &set, &smoothed, &world.smoother, &h.start, &world.mesh.triangles, h.frame);
            if let Some(flat) = &viewer {
                let zero = ev.positions(&vec![0.0; set.sliders.len()]);
                let sh = Shown { base: built(flat, i, world.mesh.base.len()), zero, triangles: world.mesh.triangles.clone() };
                SHOWN.with(|s| *s.borrow_mut() = Some(std::sync::Arc::new(sh)));
            }
            let live_ev = crate::solve::Evaluator::with(f, &live_set, &live_smoothed, &world.smoother, &h.start, &world.mesh.triangles, h.frame);
            // several starts: live, live with the root slid up, the same with no root dip, and the stand-in alone
            let added: Vec<usize> = set.sliders.iter().enumerate().filter(|(_, s)| s.name == "Root place" || s.name == "Upper bridge").map(|(j, _)| j).collect();
            let dip = set.sliders.iter().position(|s| s.name == "Root dip");
            let mut starts = vec![start.clone()];
            for &r in &added {
                for up in [2.5, 5.0] {
                    let mut a = start.clone();
                    a[r] = up;
                    starts.push(a.clone());
                    if let Some(d) = dip {
                        a[d] = 0.0;
                        starts.push(a);
                    }
                }
            }
            starts.push(vec![0.0; start.len()]);
            let fit = starts
                .iter()
                .map(|s| descend(&ev, &h.kotor, &kotor_fine, &m.marks, &spread, s, &bounds))
                .min_by(|a, b| a.cost.total_cmp(&b.cost))
                .unwrap();
            // the guard
            let before = world.smoother.smooth(&h.start, SMOOTHING);
            let near: Vec<u32> = (0..before.len() as u32).filter(|&q| window.contains(h.frame.local(before[q as usize]))).collect();
            let checker = crate::check::Checker::new(&world.mesh, &before, &near);
            let grade = |e: &crate::solve::Evaluator, s: &SliderSet, vals: &[f64]| {
                let surface = surface_of(e, vals);
                let meas = f.measure(&surface).ok();
                let fine = read(&surface).ok();
                let folds = checker.faults(&s.build(&world.smoother, &h.start, vals)).total();
                let sc = fine.as_ref().map(|x| {
                    let (g, b) = gaps(&kotor_fine, x);
                    scores(&g, &b)
                });
                (meas, sc, folds, fine)
            };
            let row_gaps = |meas: &crate::feature::Measured| -> Vec<f64> {
                let mut r: Vec<f64> = (0..np).map(|k| ((meas.proportions[k] - h.kotor.proportions[k]) / m.marks.proportions[k]).abs()).collect();
                for (o, def) in f.outlines().iter().enumerate() {
                    if h.kotor.outlines[o].is_empty() || meas.outlines[o].is_empty() {
                        r.push(0.0);
                    } else {
                        r.push(crate::measure::outline_difference(o, &h.kotor.outlines[o], &meas.outlines[o], m.marks.references[o]) / m.marks.outlines[o]);
                    }
                    let _ = def;
                }
                r.iter().map(|x| if x.is_nan() { 0.0 } else { *x }).collect()
            };
            let (lm, lsc, lfolds, lfine) = grade(&live_ev, &live_set, &live_vals);
            let lrows = lm.as_ref().map(row_gaps);
            let mut chosen = None;
            let mut tried = Vec::new();
            for share in [1.0, 0.75, 0.5, 0.25] {
                let vals: Vec<f64> = start.iter().zip(&fit.values).map(|(a, b)| a + share * (b - a)).collect();
                let (nm, nsc, nfolds, _) = grade(&ev, &set, &vals);
                let worse: Vec<usize> = match (&nm, &lrows) {
                    (Some(nm), Some(lr)) => row_gaps(nm).iter().zip(lr).enumerate().filter(|(_, (a, b))| **a > **b + 0.5).map(|(k, _)| k).collect(),
                    _ => (0..np + 3).collect(),
                };
                let better = match (nsc, lsc) {
                    (Some(n), Some(l)) => n[0] <= l[0] - 0.1 && n[3] <= l[3] + 0.25,
                    _ => false,
                };
                let pass = nfolds <= lfolds && worse.len() < 3 && better;
                tried.push(serde_json::json!({"share": share, "folds": nfolds, "worse_rows": worse, "scores": nsc, "pass": pass}));
                if pass {
                    chosen = Some((share, vals));
                    break;
                }
            }
            let (share, values) = chosen.clone().unwrap_or((0.0, start.clone()));
            let (_, fsc, ffolds, ffine) = grade(&ev, &set, &values);
            let full_line = grade(&ev, &set, &fit.values).3.map(|x| x.line);
            eprintln!("{:22} share {share:4.2}  swing {:5.2} -> {:5.2}  bridge {:5.2} -> {:5.2}  folds {lfolds} -> {ffolds}",
                h.name, lsc.map_or(f64::NAN, |x| x[3]), fsc.map_or(f64::NAN, |x| x[3]), lsc.map_or(f64::NAN, |x| x[0]), fsc.map_or(f64::NAN, |x| x[0]));
            Some((h.name.clone(), serde_json::json!({
                "share": share, "values": values, "solved": fit.values, "live": start, "cost": fit.cost,
                "live_scores": lsc, "scores": fsc, "live_folds": lfolds, "folds": ffolds, "tried": tried,
                "live_corner": lfine.as_ref().map(|x| x.corner), "corner": ffine.as_ref().map(|x| x.corner), "kotor_corner": kotor_fine.corner,
                "kotor_line": kotor_fine.line, "live_line": lfine.as_ref().map(|x| x.line.clone()), "line": ffine.as_ref().map(|x| x.line.clone()), "full_line": full_line,
            })))
        });
        let heads: serde_json::Map<String, serde_json::Value> = out.into_iter().flatten().collect();
        let doc = serde_json::json!({"sliders": set.sliders.iter().map(|s| s.name.clone()).collect::<Vec<_>>(), "mark": mark(), "heads": heads});
        std::fs::write(&args[4], serde_json::to_string_pretty(&doc).unwrap()).unwrap();
        eprintln!("solved in {:.1?}; wrote {}", t0.elapsed(), args[4]);
    }

    /// KOTOR's spread of the root corner's height (mm), the spread a slider without a nose measure is solved with.
    const ROOT_PLACE_SPREAD: f64 = 3.7;

    /// The movement of the slider Root place at 1 mm: the skin round the top of the nose slides up the face, most at the eye line
    /// and fading out over about 9 mm up and down, 10 mm to each side and toward the eyes' depth. Read on the base head, so it is the
    /// same movement on every head.
    pub fn root_place_movement(base: &[V3]) -> Vec<V3> {
        let frame = Frame::head();
        let (cy, sy, sx) = (env_or("NP_RP_CENTRE", -1.0), env_or("NP_RP_HEIGHT", 9.0), env_or("NP_RP_WIDTH", 10.0));
        let up = frame.axes[1];
        base.iter()
            .map(|&p| {
                let q = frame.local(p);
                let smooth = |t: f64| { let t = t.clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
                let w = (-((q[1] - cy) / sy).powi(2)).exp() * (-(q[0] / sx).powi(2)).exp() * smooth((q[2] - 12.0) / 10.0);
                up.map(|c| c * w / frame.scale)
            })
            .collect()
    }

    fn env_or(k: &str, d: f64) -> f64 {
        std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
    }

    /// The movement of the slider Upper bridge at 1 mm: the skin of the bridge's top third, from the eye line down to 6 mm below it,
    /// moves straight out of the face by 1 mm, fading to nothing 8 mm above the eye line and 18 mm below it with smooth (cosine)
    /// edges, and over about 8 mm to each side. Read on the base head.
    pub fn upper_bridge_movement(base: &[V3]) -> Vec<V3> {
        let frame = Frame::head();
        let (top, flat_lo, bottom, sx) = (env_or("NP_UB_TOP", 8.0), env_or("NP_UB_FLAT", -6.0), env_or("NP_UB_BOTTOM", -18.0), env_or("NP_UB_WIDTH", 8.0));
        let out = frame.axes[2];
        let ramp = |t: f64| 0.5 - 0.5 * (std::f64::consts::PI * t.clamp(0.0, 1.0)).cos();
        base.iter()
            .map(|&p| {
                let q = frame.local(p);
                let wy = if q[1] > 0.0 { ramp((top - q[1]) / top) } else if q[1] > flat_lo { 1.0 } else { ramp((q[1] - bottom) / (flat_lo - bottom)) };
                let w = wy * (-(q[0] / sx).powi(2)).exp() * ramp((q[2] - 12.0) / 10.0);
                out.map(|c| c * w / frame.scale)
            })
            .collect()
    }

    /// `sliders noseprofile make <data> <out.json>`: the live nose set with the slider Root place added at the end, or with
    /// `NP_NEW=upper` the slider Upper bridge.
    fn make(args: &[String], data: &Path, input: &Input) {
        let mesh = input.head_mesh();
        let from = std::env::var("NP_BASE_SET").map(std::path::PathBuf::from).unwrap_or_else(|_| data.join("web/sliders_r2.json"));
        let mut set: SliderSet = serde_json::from_str(&std::fs::read_to_string(from).unwrap()).unwrap();
        let upper = std::env::var("NP_NEW").is_ok_and(|v| v == "upper");
        let mut s = if upper {
            let mut s = crate::set::Slider::from_movement("Upper bridge", "proportion", "mm", &upper_bridge_movement(&mesh.base));
            s.group = "bridge".into();
            s.hint = "Seen from the side, how far the top of the bridge stands out, just under the eyes. A higher value fills the hollow under the top of the nose, so the bridge runs straight from between the eyes.".into();
            s
        } else {
            crate::set::Slider::from_movement("Root place", "proportion", "mm", &root_place_movement(&mesh.base))
        };
        s.low = env_or("NP_RP_LOW", -6.0);
        s.high = env_or("NP_RP_HIGH", 6.0);
        if !upper {
        s.group = "root".into();
        s.hint = "Seen from the side, how high the dip at the top of the nose sits between the eyes. A higher value moves the dip up toward the brow, so the bridge starts higher.".into();
        }
        set.sliders.push(s);
        std::fs::write(&args[4], serde_json::to_string(&set).unwrap()).unwrap();
        eprintln!("wrote {} with {} sliders", args[4], set.sliders.len());
    }

    /// `sliders noseprofile ends <data> <out.png> <head> ...` with `SLIDERS_SET` and `NP_SOLVE` (a solve's output): each added slider
    /// (Root place, Upper bridge) at its low end, at the head's solved value and at its high end, from the side and three-quarter, on
    /// the base head ("base") and the heads named. It also counts the folded triangles near the nose at both ends on all 57 heads.
    fn ends(args: &[String], _data: &Path, input: &Input) {
        use crate::feature::Shot;
        use crate::render::{clay, label, sheet};
        let world = World::load(input, by_key("nose").unwrap());
        let set: SliderSet = serde_json::from_str(&std::fs::read_to_string(std::env::var("SLIDERS_SET").unwrap()).unwrap()).unwrap();
        let solved: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(std::env::var("NP_SOLVE").unwrap()).unwrap()).unwrap();
        let added: Vec<usize> = set.sliders.iter().enumerate().filter(|(_, s)| s.name == "Root place" || s.name == "Upper bridge").map(|(j, _)| j).collect();
        let shots = [
            Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-12.0, 48.0, -50.0, 22.0] },
            Shot { name: "three-quarter", yaw: 40.0, pitch: 0.0, pivot: 0.0, window: [-30.0, 30.0, -50.0, 22.0] },
        ];
        let values_of = |name: &str| -> Vec<f64> {
            solved["heads"][name]["values"].as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap()).collect()).unwrap_or_else(|| vec![0.0; set.sliders.len()])
        };
        let frame = Frame::head();
        let mut tiles = Vec::new();
        for name in &args[5..] {
            let (start, vals) = if name == "base" {
                (world.mesh.base.clone(), vec![0.0; set.sliders.len()])
            } else {
                let h = world.heads.iter().find(|h| &h.name == name).unwrap();
                (h.start.clone(), values_of(name))
            };
            for &j in &added {
                let s = &set.sliders[j];
                for (tag, x) in [("low", s.low), ("solved", vals[j]), ("high", s.high)] {
                    let mut v = vals.clone();
                    v[j] = x;
                    let p = set.build(&world.smoother, &start, &v);
                    let local: Vec<V3> = p.iter().map(|&q| frame.local(q)).collect();
                    for sh in &shots {
                        let mut img = clay(&local, &world.mesh.triangles, sh, &[], &[]);
                        if sh.name == "side" {
                            label(&mut img, &format!("{} {} {tag} {x:.1}", name.replace('_', " "), s.name));
                        }
                        tiles.push(img);
                    }
                }
            }
        }
        sheet(&tiles, 6).save(&args[4]).unwrap();
        eprintln!("wrote {}", args[4]);
        let window = world.feature.window();
        for &j in &added {
            let s = &set.sliders[j];
            let counts = crate::heads::parallel(world.heads.len(), crate::heads::threads(), |i| {
                let h = &world.heads[i];
                let before = world.smoother.smooth(&h.start, SMOOTHING);
                let near: Vec<u32> = (0..before.len() as u32).filter(|&q| window.contains(h.frame.local(before[q as usize]))).collect();
                let c = crate::check::Checker::new(&world.mesh, &before, &near);
                let vals = values_of(&h.name);
                [s.low, vals[j], s.high].map(|x| {
                    let mut v = vals.clone();
                    v[j] = x;
                    c.faults(&set.build(&world.smoother, &h.start, &v)).total()
                })
            });
            let heads_with = |k: usize| counts.iter().filter(|c| c[k] > 0).count();
            let tris = |k: usize| counts.iter().map(|c| c[k]).sum::<usize>();
            eprintln!("{}: heads with a fold at the low end {} ({} triangles), at the solved value {} ({}), at the high end {} ({})",
                s.name, heads_with(0), tris(0), heads_with(1), tris(1), heads_with(2), tris(2));
        }
    }

    /// `sliders noseprofile pictures <data> <out folder> <today.bin> <new.bin> <head> ...`: per head, a picture of three enlarged
    /// clay profiles side by side (KOTOR's mesh, today's head in viewer 2 and the new head), each with the three midline outlines drawn
    /// over it: KOTOR's in black (thick, under the others), today's in red and the new one in green. `SLIDERS_PX` sets the pixels per mm (8 by default here).
    fn pictures(args: &[String], _data: &Path, input: &Input) {
        use crate::feature::Shot;
        use crate::render::{clay, label, sheet};
        use image::Rgb;
        let mesh = input.head_mesh();
        let vc = mesh.base.len();
        if std::env::var("SLIDERS_PX").is_err() {
            // SAFETY: set before any thread starts, and read only by this tool's drawing
            unsafe { std::env::set_var("SLIDERS_PX", "8") };
        }
        let px: f64 = std::env::var("SLIDERS_PX").unwrap().parse().unwrap();
        let today = read_f32(&args[5]);
        let new = read_f32(&args[6]);
        let names: Vec<String> = input.manifest["heads"].as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect();
        let frame = Frame::head();
        let out = Path::new(&args[4]);
        std::fs::create_dir_all(out).unwrap();
        for name in &args[7..] {
            let i = names.iter().position(|n| n == name).unwrap();
            let kv = input.points(&format!("heads/{name}/kotor_v"));
            let kf = input.triangles(&format!("heads/{name}/kotor_f"));
            let tv = built(&today, i, vc);
            let nv = built(&new, i, vc);
            let lines: Vec<Vec<[f64; 2]>> = [(&kv, &kf), (&tv, &mesh.triangles), (&nv, &mesh.triangles)]
                .iter()
                .map(|(v, f)| read_mesh(v, f).map(|x| x.line).unwrap_or_default())
                .collect();
            let tip = read_mesh(&kv, &kf).map(|x| x.tip[1]).unwrap_or(45.0);
            let shot = Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [tip - 40.0, tip + 6.0, -48.0, 22.0] };
            let colours = [Rgb([10, 10, 10]), Rgb([225, 40, 40]), Rgb([30, 170, 60])];
            let mut tiles = Vec::new();
            for (k, (v, f, tag)) in [(&kv, &kf, "KOTOR"), (&tv, &mesh.triangles, "today"), (&nv, &mesh.triangles, "new")].into_iter().enumerate() {
                let local: Vec<V3> = v.iter().map(|&p| frame.local(p)).collect();
                let mut img = clay(&local, f, &shot, &[], &[]);
                for (n, (l, c)) in lines.iter().zip(colours).enumerate() {
                    let r: &[i64] = if n == 0 { &[-2, -1, 0, 1] } else { &[-1, 0] };
                    for w in l.windows(2) {
                        let steps = 12;
                        for t in 0..=steps {
                            let a = t as f64 / steps as f64;
                            let (y, z) = (w[0][0] + a * (w[1][0] - w[0][0]), w[0][1] + a * (w[1][1] - w[0][1]));
                            let (x, yy) = ((z - shot.window[0]) * px, (shot.window[3] - y) * px);
                            for &d in r {
                                for &e in r {
                                    let (xi, yi) = (x as i64 + d, yy as i64 + e);
                                    if xi >= 0 && yi >= 0 && (xi as u32) < img.width() && (yi as u32) < img.height() {
                                        img.put_pixel(xi as u32, yi as u32, c);
                                    }
                                }
                            }
                        }
                    }
                }
                label(&mut img, &format!("{} {tag}", name.replace('_', " ")));
                let _ = k;
                tiles.push(img);
            }
            let path = out.join(format!("profile_{name}.png"));
            sheet(&tiles, 3).save(&path).unwrap();
            eprintln!("wrote {}", path.display());
        }
    }
}
