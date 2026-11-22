//! The eye audit's measures (2026-11-20). Usage: `eye_audit <command> <data> [heads...]`, with `SLIDERS_SET` naming the eye set.
//!
//! - `presets`: the eye presets solved (`bench::presets`), then placed whole (`eyes::tools::place_whole`), then per head the faults
//!   with and without the press, the measures of the built preset, KOTOR's and the stand-in's, written to `eyes/audit.json`.
//! - `starts`: per head, how the start shape's eyes differ from the base head's, written to `eyes/starts_audit.json`.
//! - `sweep <heads>`: each slider from its low end to its high end on each head's preset, with its own measure read on the built head
//!   with and without the press, written to `eyes/sweep.json`.
//! - `iris`: how much of the iris KOTOR's skin and KOTOR's lid piece cover, and how much our skin and our open lid piece cover on the
//!   presets, written to `eyes/iris.json`.
//! - `pictures <experiment data> <heads>`: KOTOR, today's preset and the experiment's preset side by side, in clay and in a light like
//!   viewer 2's "In game" light, close in three-quarter view and from the front (`EA_OUT` names the folder of the sheets).
//!
//! `EA_PASTE=1` replaces each start shape's eye area with the base head's (`paste`), for every command.

use head_sliders::V3;
use head_sliders::bench;
use head_sliders::check::Checker;
use head_sliders::checks::Marks;
use head_sliders::eye_curves;
use head_sliders::eyes::EYES;
use head_sliders::eyes::tools::{self, Scene};
use head_sliders::feature::{Feature, Shot, proportion_index};
use head_sliders::heads::{Head, World, parallel, threads};
use head_sliders::measure::{EYES_APART_MM, Frame, Surface};
use head_sliders::mesh::input::Input;
use head_sliders::set::{SMOOTHING, SliderSet};
use head_sliders::{add, length, scale, sub};
use image::{Rgb, RgbImage};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Per vertex, how much of the base head's eye area replaces the start's: 1 within an oval 17 mm across and 11 mm up and down from
/// each eye's centre (the eye zone's core), fading to 0 at 1.7 times that oval, above 14 mm up, below 13 mm down, and more than 25 mm
/// behind the eye's centre. The second value is 1 on the core only.
fn paste_weights(world: &World) -> Vec<(f64, bool)> {
    let frame = Frame::from_eyes(world.mesh.eyes);
    world
        .mesh
        .base
        .iter()
        .map(|&b| {
            let p = frame.local(b);
            let (u, up) = (p[0].abs() - 0.5 * EYES_APART_MM, p[1]);
            let e = ((u / 17.0).powi(2) + (up / 11.0).powi(2)).sqrt();
            let w = smoothstep((1.7 - e) / 0.7) * smoothstep((14.0 - up) / 3.0) * smoothstep((up + 13.0) / 3.0) * smoothstep((p[2] + 25.0) / 5.0);
            (w, e < 1.0 && p[2] > -20.0)
        })
        .collect()
}

/// The start with its eye area replaced by the base head's, each eye carried by one translation: the mean movement of the start from
/// the base over that eye's core. Inside the core the start is the base head moved rigidly; the weight fades it into the start.
fn paste(world: &World, weights: &[(f64, bool)], start: &[V3]) -> Vec<V3> {
    let frame = Frame::from_eyes(world.mesh.eyes);
    let base = &world.mesh.base;
    let side = |v: usize| if frame.local(base[v])[0] > 0.0 { 0 } else { 1 };
    let mut t = [[0.0; 3]; 2];
    let mut n = [0.0; 2];
    for v in 0..base.len() {
        if weights[v].1 {
            let s = side(v);
            t[s] = add(t[s], sub(start[v], base[v]));
            n[s] += 1.0;
        }
    }
    let t = [scale(t[0], 1.0 / n[0]), scale(t[1], 1.0 / n[1])];
    (0..base.len())
        .map(|v| {
            let w = weights[v].0;
            let target = add(base[v], t[side(v)]);
            add(start[v], scale(sub(target, start[v]), w))
        })
        .collect()
}

fn smoothed(world: &World, p: &[V3]) -> Vec<V3> {
    world.smoother.smooth(p, SMOOTHING)
}

fn measure_built(h: &Head, built: &[V3], world: &World) -> Option<head_sliders::feature::Measured> {
    measure_lowered(h, built, world, [0.0; 2])
}

/// The measure of a built head as viewer 2 shows it, with the lid pieces lowered by `lower` mm.
fn measure_lowered(h: &Head, built: &[V3], world: &World, lower: [f64; 2]) -> Option<head_sliders::feature::Measured> {
    EYES.measure(&head_sliders::eyes::shown_surface(built, &world.mesh.triangles, &h.frame, lower)).ok()
}

fn faults(world: &World, h: &Head, set: &SliderSet, values: &[f64], press: bool) -> usize {
    let before = smoothed(world, &h.start);
    let window = EYES.window();
    let near: Vec<u32> = (0..before.len() as u32).filter(|&v| window.contains(h.frame.local(before[v as usize]))).collect();
    let checker = Checker::new(&world.mesh, &before, &near);
    let built = if press { set.build(&world.smoother, &h.start, values) } else { smoothed(world, &set.unsmoothed(&h.start, values)) };
    checker.faults(&built).total()
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
}

fn measured_json(m: &Option<head_sliders::feature::Measured>) -> Value {
    m.as_ref().map(|m| json!({"proportions": m.proportions, "outlines": m.outlines})).unwrap_or(Value::Null)
}

/// Turns a place for a shot, as the clay pictures do.
fn turn(shot: &Shot, p: V3) -> V3 {
    let (sy, cy) = shot.yaw.to_radians().sin_cos();
    let (x, z) = (p[0] * cy + p[2] * sy, -p[0] * sy + p[2] * cy);
    let (sp, cp) = shot.pitch.to_radians().sin_cos();
    let y = p[1] - shot.pivot;
    [x, cp * y + sp * z + shot.pivot, -sp * y + cp * z]
}

/// A scene drawn with a light like viewer 2's "In game" light: a sky light from above (white, ground grey 0x444444, strength 0.75)
/// and one key light from the upper left (strength 0.75, from (-0.5, 0.6, 1.0)), both fixed to the head. Skin is a light skin tone.
fn lit(scene: &Scene, shot: &Shot) -> RgbImage {
    let px = std::env::var("SLIDERS_PX").ok().and_then(|v| v.parse().ok()).unwrap_or(2.5);
    let window = shot.window;
    let (w, h) = (((window[1] - window[0]) * px) as u32, ((window[3] - window[2]) * px) as u32);
    let p: Vec<V3> = scene.p.iter().map(|&q| turn(shot, q)).collect();
    let mut normals = vec![[0.0; 3]; p.len()];
    for t in &scene.t {
        let n = head_sliders::cross(sub(scene.p[t[1] as usize], scene.p[t[0] as usize]), sub(scene.p[t[2] as usize], scene.p[t[0] as usize]));
        for &i in t {
            normals[i as usize] = add(normals[i as usize], n);
        }
    }
    let key = head_sliders::normalize([-0.5, 0.6, 1.0]);
    let mut img = RgbImage::from_pixel(w, h, Rgb([40, 44, 52]));
    let mut depth = vec![f64::NEG_INFINITY; (w * h) as usize];
    let to_px = |q: V3| [(q[0] - window[0]) * px, (window[3] - q[1]) * px, q[2]];
    for (k, t) in scene.t.iter().enumerate() {
        let v = t.map(|i| to_px(p[i as usize]));
        let area = (v[1][0] - v[0][0]) * (v[2][1] - v[0][1]) - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
        if area.abs() < 1e-9 {
            continue;
        }
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
                let mut n = [0.0; 3];
                for (b, &i) in [b0, b1, b2].iter().zip(t) {
                    n = add(n, scale(normals[i as usize], *b));
                }
                let n = head_sliders::normalize(n);
                let sky = 0.75 * (0.267 + (1.0 - 0.267) * (0.5 * n[1] + 0.5));
                let light = sky + 0.75 * head_sliders::dot(n, key).max(0.0);
                let base = scene.tint[k].unwrap_or([214.0, 170.0, 146.0]);
                img.put_pixel(x as u32, y as u32, Rgb(base.map(|c| (c * light).clamp(0.0, 255.0) as u8)));
            }
        }
    }
    img
}

fn is(t: &Option<[f64; 3]>, c: [f64; 3]) -> bool {
    t.is_some_and(|x| x == c)
}
const WHITE: [f64; 3] = [238.0, 234.0, 226.0];
const IRIS: [f64; 3] = [96.0, 70.0, 48.0];
const PUPIL: [f64; 3] = [18.0, 16.0, 16.0];
const PIECE: [f64; 3] = [196.0, 150.0, 128.0];
const KOTOR_LID: [f64; 3] = [120.0, 100.0, 150.0];

/// A front z-buffer every 0.05 mm over a square of 30 mm round `c` (x, y), of the triangles `keep` picks.
fn zbuf(scene: &Scene, c: V3, keep: &dyn Fn(usize) -> bool) -> (Vec<f64>, Vec<i32>) {
    let (px, half) = (0.05, 15.0);
    let n = (2.0 * half / px) as usize;
    let mut z = vec![f64::NEG_INFINITY; n * n];
    let mut who = vec![-1i32; n * n];
    for (k, t) in scene.t.iter().enumerate() {
        if !keep(k) {
            continue;
        }
        let v = t.map(|i| {
            let q = scene.p[i as usize];
            [(q[0] - c[0] + half) / px, (q[1] - c[1] + half) / px, q[2]]
        });
        let area = (v[1][0] - v[0][0]) * (v[2][1] - v[0][1]) - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
        if area.abs() < 1e-12 {
            continue;
        }
        let lo = |a: usize| v.iter().map(|q| q[a]).fold(f64::MAX, f64::min);
        let hi = |a: usize| v.iter().map(|q| q[a]).fold(f64::MIN, f64::max);
        if hi(0) < 0.0 || hi(1) < 0.0 || lo(0) > n as f64 || lo(1) > n as f64 {
            continue;
        }
        for j in (lo(1).floor().max(0.0) as usize)..=(hi(1).ceil().min(n as f64 - 1.0) as usize) {
            for i in (lo(0).floor().max(0.0) as usize)..=(hi(0).ceil().min(n as f64 - 1.0) as usize) {
                let (fx, fy) = (i as f64 + 0.5, j as f64 + 0.5);
                let b0 = ((v[1][0] - fx) * (v[2][1] - fy) - (v[2][0] - fx) * (v[1][1] - fy)) / area;
                let b1 = ((v[2][0] - fx) * (v[0][1] - fy) - (v[0][0] - fx) * (v[2][1] - fy)) / area;
                let b2 = 1.0 - b0 - b1;
                if b0 < -1e-9 || b1 < -1e-9 || b2 < -1e-9 {
                    continue;
                }
                let d = b0 * v[0][2] + b1 * v[1][2] + b2 * v[2][2];
                if d > z[j * n + i] {
                    z[j * n + i] = d;
                    who[j * n + i] = k as i32;
                }
            }
        }
    }
    (z, who)
}

/// For one eye of a scene: the share of the iris (iris and pupil, seen from the front) that the skin covers, the share the skin and the
/// lid piece cover together, and the height of the top of the visible eye above the eyeball's centre on the column through it, in iris
/// radii, for the skin alone and with the piece. `piece` is the lid piece's tint.
fn iris_cover(scene: &Scene, centre: V3, radius: f64, piece: [f64; 3]) -> Value {
    let ball = |k: usize| is(&scene.tint[k], WHITE) || is(&scene.tint[k], IRIS) || is(&scene.tint[k], PUPIL);
    let near = |k: usize| scene.t[k].iter().any(|&i| length(sub(scene.p[i as usize], centre)) < 1.6 * radius);
    let (bz, bwho) = zbuf(scene, centre, &|k| ball(k) && near(k));
    let (sz, _) = zbuf(scene, centre, &|k| scene.tint[k].is_none());
    let (pz, _) = zbuf(scene, centre, &|k| is(&scene.tint[k], piece) && near(k));
    let n = (bz.len() as f64).sqrt() as usize;
    let (mut iris, mut by_skin, mut by_both) = (0usize, 0usize, 0usize);
    for a in 0..bz.len() {
        if bwho[a] < 0 {
            continue;
        }
        let t = &scene.tint[bwho[a] as usize];
        if !(is(t, IRIS) || is(t, PUPIL)) {
            continue;
        }
        iris += 1;
        let skin = sz[a] > bz[a];
        let both = skin || pz[a] > bz[a];
        by_skin += skin as usize;
        by_both += both as usize;
    }
    let iris_r = radius * 0.42f64.sin();
    let col = n / 2;
    let top = |occ: &dyn Fn(usize) -> bool| -> f64 {
        for j in n / 2..n {
            let a = j * n + col;
            if bwho[a] < 0 || occ(a) {
                return (j as f64 + 0.5 - n as f64 / 2.0) * 0.05 / iris_r;
            }
        }
        f64::NAN
    };
    let bottom = |occ: &dyn Fn(usize) -> bool| -> f64 {
        for j in (0..n / 2).rev() {
            let a = j * n + col;
            if bwho[a] < 0 || occ(a) {
                return (j as f64 + 0.5 - n as f64 / 2.0) * 0.05;
            }
        }
        f64::NAN
    };
    let skin_top = top(&|a| sz[a] > bz[a]);
    let both_top = top(&|a| sz[a] > bz[a] || pz[a] > bz[a]);
    json!({"iris_px": iris, "skin": by_skin as f64 / iris.max(1) as f64, "with_piece": by_both as f64 / iris.max(1) as f64,
           "top_skin": skin_top, "top_with_piece": both_top, "iris_r_mm": iris_r,
           "bottom_mm": bottom(&|a| sz[a] > bz[a] || pz[a] > bz[a])})
}


/// Each eye's two lids as edge places (indices into `edges()[e].edge`), from the inner corner's tip to the outer corner's tip, upper first.
fn lids_of(e: &eye_curves::EyeEdges) -> [Vec<usize>; 2] {
    let tip = |inner: bool| e.edge.iter().position(|&v| v == if inner { e.inner } else { e.outer }).unwrap();
    let (a, b) = (tip(true), tip(false));
    [true, false].map(|up| {
        let mut ks: Vec<usize> = (0..e.edge.len()).filter(|&k| k != a && k != b && e.upper[k] == up).collect();
        ks.sort_by(|&x, &y| e.share[x].total_cmp(&e.share[y]));
        ks.insert(0, a);
        ks.push(b);
        ks
    })
}

fn solve3(m: [[f64; 3]; 3], b: [f64; 3]) -> [f64; 3] {
    let det = |m: [[f64; 3]; 3]| m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let d = det(m);
    if d.abs() < 1e-12 { return [0.0; 3]; }
    let mut out = [0.0; 3];
    for k in 0..3 {
        let mut mk = m;
        for r in 0..3 { mk[r][k] = b[r]; }
        out[k] = det(mk) / d;
    }
    out
}

/// How wavy a built head's lid edges are (frame of the head, mm): per lid, each inner point's distance from the same edge smoothed along
/// its length (a quadratic through it and two neighbours on each side), seen from the front and in depth, for points 3 mm or more from both tips, and the largest change of the front-view turn
/// between neighbouring points (degrees). Returns the largest of each over the four lids.
fn waviness(built: &[V3], frame: &Frame) -> [f64; 3] {
    let mut out = [0.0f64; 3];
    for e in eye_curves::edges() {
        for lid in lids_of(e) {
            let p: Vec<V3> = lid.iter().map(|&k| frame.local(built[e.edge[k] as usize])).collect();
            let n = p.len();
            let mut s = vec![0.0; n];
            for i in 1..n {
                s[i] = s[i - 1] + length(sub(p[i], p[i - 1]));
            }
            let total = s[n - 1];
            if std::env::var("EA_WHERE").is_ok_and(|v| v == "2") {
                eprintln!("  {} {}", if e.upper[lid[1]] { "upper" } else { "lower" }, p.iter().take(5).map(|q| format!("({:.1},{:.1},{:.1})", q[0], q[1], q[2])).collect::<Vec<_>>().join(" "));
            }
            // the lid bends one way seen from the front (and in depth); a turn against that bend, at a point 3 mm or more from both
            // tips, is wobble. turn[i] is the front-view turn at point i + 1 (degrees); bend[i] the depth's second divided difference.
            let turn: Vec<f64> = (1..n - 1)
                .map(|i| {
                    let a = (p[i][1] - p[i - 1][1]).atan2(p[i][0] - p[i - 1][0]);
                    let b = (p[i + 1][1] - p[i][1]).atan2(p[i + 1][0] - p[i][0]);
                    let mut d = (b - a).to_degrees();
                    while d > 180.0 { d -= 360.0; }
                    while d < -180.0 { d += 360.0; }
                    d
                })
                .collect();
            let inner = |i: usize| s[i + 1] >= 3.0 && total - s[i + 1] >= 3.0;
            let main: f64 = (0..turn.len()).filter(|&i| inner(i)).map(|i| turn[i]).sum::<f64>().signum();
            for i in (0..turn.len()).filter(|&i| inner(i)) {
                let against = (-main * turn[i]).max(0.0);
                if against > 10.0 && std::env::var("EA_WHERE").is_ok() {
                    eprintln!("  turn against {against:.1} deg at {:.1} mm of {total:.1} (point {} of {n}, {})", s[i + 1], i + 1, if e.upper[lid[1]] { "upper" } else { "lower" });
                }
                out[0] = out[0].max(against);
                if against > 0.5 { out[2] += 1.0; }
            }
            let bend: Vec<f64> = (1..n - 1)
                .map(|i| {
                    let (h0, h1) = (s[i] - s[i - 1], s[i + 1] - s[i]);
                    ((p[i + 1][2] - p[i][2]) / h1.max(1e-6) - (p[i][2] - p[i - 1][2]) / h0.max(1e-6)) / (0.5 * (h0 + h1)).max(1e-6)
                })
                .collect();
            let mb: f64 = (0..bend.len()).filter(|&i| inner(i)).map(|i| bend[i]).sum::<f64>().signum();
            for i in (0..bend.len()).filter(|&i| inner(i)) {
                out[1] = out[1].max((-mb * bend[i]).max(0.0));
            }
        }
    }
    out
}

/// Where a faulty triangle lies, read on the base head: "inner corner" within 7 mm of an inner corner's tip, "lids" within 4 mm of a
/// lid's edge, "nose side" more than 14 mm from an eye's centre toward the middle, else "other".
fn zone_of(world: &World, t: [u32; 3]) -> &'static str {
    let b = &world.mesh.base;
    let c = scale(add(add(b[t[0] as usize], b[t[1] as usize]), b[t[2] as usize]), 1.0 / 3.0);
    for e in eye_curves::edges() {
        if length(sub(c, b[e.inner as usize])) * 1000.0 < 7.0 {
            return "inner corner";
        }
    }
    for e in eye_curves::edges() {
        if e.edge.iter().any(|&v| length(sub(c, b[v as usize])) * 1000.0 < 4.0) {
            return "lids";
        }
    }
    let frame = Frame::from_eyes(world.mesh.eyes);
    let q = frame.local(c);
    if q[0].abs() - 0.5 * EYES_APART_MM < -14.0 { "nose side" } else { "other" }
}

/// Values with `EA_SET="<slider>=<value>;..."` on top.
fn with_ea_set(set: &SliderSet, mut values: Vec<f64>) -> Vec<f64> {
    for a in std::env::var("EA_SET").unwrap_or_default().split(';').filter(|a| !a.is_empty()) {
        let (n, v) = a.split_once('=').unwrap();
        let k = set.sliders.iter().position(|s| s.name == n).unwrap_or_else(|| panic!("no slider {n}"));
        values[k] = v.parse().unwrap();
    }
    values
}

/// Viewer 2's "shown" head: the five features' sets merged, each head's start from web/starts_all.bin and its solved values of every
/// feature, as viewer 2's sliders.js reads them. Returns the merged set and, per head of web/heads.json, its name, start and values.
fn shown_heads(data: &Path, vc: usize) -> (SliderSet, Vec<(String, Vec<V3>, Vec<f64>)>) {
    let web = data.join("web");
    let parts = [("sliders_r2.json", "page_r2.json"), ("jaw/sliders.json", "jaw/page.json"), ("brow/sliders.json", "brow/page.json"), ("cheeks/sliders.json", "cheeks/page.json"), ("eyes/sliders.json", "eyes/page.json")];
    let sets: Vec<SliderSet> = parts.iter().map(|p| serde_json::from_str(&std::fs::read_to_string(web.join(p.0)).unwrap()).unwrap()).collect();
    let pages: Vec<Value> = parts.iter().map(|p| read_json(&web.join(p.1))).collect();
    let merged = SliderSet::merge(&sets);
    let heads = read_json(&web.join("heads.json"));
    let raw = std::fs::read(web.join("starts_all.bin")).unwrap();
    let f: Vec<f32> = raw.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
    let rows = heads
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let name = h["name"].as_str().unwrap().to_string();
            let start: Vec<V3> = (0..vc).map(|v| { let o = (i * vc + v) * 3; [f[o] as f64, f[o + 1] as f64, f[o + 2] as f64] }).collect();
            let mut values = vec![0.0; merged.sliders.len()];
            let mut first = 0;
            for (s, p) in sets.iter().zip(&pages) {
                let names: Vec<&str> = s.sliders.iter().map(|x| x.name.as_str()).collect();
                let last = p["counts"].as_array().and_then(|c| c.last()).map(|c| c.to_string()).unwrap_or_default();
                let hd = p["heads"].as_array().unwrap().iter().find(|x| x["name"] == name.as_str());
                let solved = hd.and_then(|x| { let s = &x["solved"]; if s["full"].is_object() { Some(&s["full"]) } else if s["held_out"].is_object() { Some(&s["held_out"]) } else { None } });
                if let Some(v) = solved.and_then(|s| s["counts"][&last]["values"].as_array()) {
                    for (j, x) in v.iter().enumerate() {
                        let k = p["sliders"].as_array().map_or(j as i64, |ps| names.iter().position(|n| ps.get(j).and_then(|q| q["name"].as_str()) == Some(*n)).map_or(-1, |q| q as i64));
                        if k >= 0 && (k as usize) < s.sliders.len() {
                            values[first + k as usize] = x.as_f64().unwrap_or(0.0);
                        }
                    }
                }
                first += s.sliders.len();
            }
            (name, start, values)
        })
        .collect();
    (merged, rows)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("starts");
    let data = PathBuf::from(args.get(2).expect("data folder"));
    let input = Input::open(&data.join("input"));
    let t0 = std::time::Instant::now();
    let mut world = World::load(&input, &EYES);
    eprintln!("{} heads in {:.1?}", world.heads.len(), t0.elapsed());
    let weights = paste_weights(&world);
    // the starts without the pasted eye area, as before round 4
    let originals: Vec<Vec<V3>> = if command == "r4-pictures" {
        unsafe { std::env::set_var("EYES_PASTE", "0") };
        let w0 = World::load(&input, &EYES);
        unsafe { std::env::remove_var("EYES_PASTE") };
        w0.heads.iter().map(|h| h.start.clone()).collect()
    } else {
        world.heads.iter().map(|h| h.start.clone()).collect()
    };
    let pasted: Vec<Vec<V3>> = parallel(world.heads.len(), threads(), |i| paste(&world, &weights, &world.heads[i].start));
    if std::env::var("EA_PASTE").is_ok_and(|v| v == "1") {
        let news: Vec<Option<head_sliders::feature::Measured>> = parallel(world.heads.len(), threads(), |i| {
            let h = &world.heads[i];
            measure_built(h, &smoothed(&world, &pasted[i]), &world)
        });
        for ((h, p), m) in world.heads.iter_mut().zip(&pasted).zip(news) {
            h.start = p.clone();
            h.start_nose = m;
        }
        eprintln!("pasted the base head's eye area into every start");
    }
    let set = tools::read_set(&data);
    let world = &world;
    match command {
        "presets" => {
            let m = Marks::read(&data.join("eyes/marks.json"));
            bench::presets(world, &m, &data);
            let before = read_json(&data.join("web/eyes/page.json"));
            if !std::env::var("EA_NO_PLACE").is_ok_and(|v| v == "1") {
                tools::place_whole(world, &set, &data);
            }
            let page = read_json(&data.join("web/eyes/page.json"));
            let rows: Vec<Value> = parallel(world.heads.len(), threads(), |i| {
                let h = &world.heads[i];
                let solved = tools::solved_values(&before, &set, &h.name);
                let placed = tools::solved_values(&page, &set, &h.name);
                let built = set.build(&world.smoother, &h.start, &placed);
                let unpressed = smoothed(world, &set.unsmoothed(&h.start, &placed));
                json!({"name": h.name, "scored": h.kotor_error.is_none(), "solved": solved, "values": placed,
                    "faults": faults(world, h, &set, &placed, true), "faults_no_press": faults(world, h, &set, &placed, false),
                    "faults_solved": faults(world, h, &set, &solved, true),
                    "kotor": {"proportions": h.kotor.proportions, "outlines": h.kotor.outlines},
                    "standin": measured_json(&h.start_nose), "built": measured_json(&measure_lowered(h, &built, world, head_sliders::eyes::cover_of(&set, &placed))),
                    "built_no_press": measured_json(&measure_built(h, &unpressed, world))})
            });
            let sliders: Vec<Value> =
                set.sliders.iter().map(|s| json!({"name": s.name, "measure": s.measure, "low": s.low, "high": s.high, "keys": s.keys.len()})).collect();
            let out = json!({"sliders": sliders, "proportions": EYES.proportions().iter().map(|p| p.key).collect::<Vec<_>>(), "heads": rows});
            std::fs::write(data.join("eyes/audit.json"), serde_json::to_string(&out).unwrap()).unwrap();
            eprintln!("wrote eyes/audit.json in {:.1?}", t0.elapsed());
        }
        "starts" => {
            let base_s = smoothed(world, &world.mesh.base);
            let bframe = Frame::from_eyes(world.mesh.eyes);
            let one = |start: &[V3], h: &Head| -> Value {
                let s = smoothed(world, start);
                let eyes: Vec<Value> = eye_curves::edges()
                    .iter()
                    .map(|e| {
                        let (c, r) = eye_curves::ball(&s, &e.ring);
                        let (cb, rb) = eye_curves::ball(&base_s, &e.ring);
                        let (lc, lb) = (h.frame.local(c), bframe.local(cb));
                        let place = [e.sign * (lc[0] - lb[0]), lc[1] - lb[1], lc[2] - lb[2]];
                        let gap: Vec<f64> = e.edge.iter().map(|&v| length(sub(s[v as usize], c)) * 1000.0 - r).collect();
                        let gap_b: Vec<f64> = e.edge.iter().map(|&v| length(sub(base_s[v as usize], cb)) * 1000.0 - rb).collect();
                        let off: Vec<f64> = gap.iter().zip(&gap_b).map(|(a, b)| a - b).collect();
                        let jump = |g: &[f64]| (0..g.len()).map(|k| (g[(k + 1) % g.len()] - g[k]).abs()).fold(0.0, f64::max);
                        // the shape apart from its place: the start's movement from the base over the eye's core, less its mean
                        let core: Vec<usize> = (0..s.len())
                            .filter(|&v| weights[v].1 && e.sign * bframe.local(world.mesh.base[v])[0] > 0.0)
                            .collect();
                        let d: Vec<V3> = core.iter().map(|&v| sub(s[v], base_s[v])).collect();
                        let mean = scale(d.iter().fold([0.0; 3], |a, &b| add(a, b)), 1.0 / d.len() as f64);
                        let res: Vec<f64> = d.iter().map(|&x| length(sub(x, mean)) * 1000.0).collect();
                        let rms = (res.iter().map(|x| x * x).sum::<f64>() / res.len() as f64).sqrt();
                        let edge_res: Vec<f64> = e.edge.iter().map(|&v| length(sub(sub(s[v as usize], base_s[v as usize]), mean)) * 1000.0).collect();
                        json!({"place": place, "radius": r, "radius_base": rb,
                            "gap_mean": off.iter().sum::<f64>() / off.len() as f64,
                            "gap_abs_mean": off.iter().map(|x| x.abs()).sum::<f64>() / off.len() as f64,
                            "gap_max": off.iter().fold(0.0f64, |a, b| a.max(b.abs())), "gap_jump": jump(&gap), "gap_jump_base": jump(&gap_b),
                            "shape_rms": rms, "shape_max": res.iter().fold(0.0f64, |a, &b| a.max(b)),
                            "edge_shape_rms": (edge_res.iter().map(|x| x * x).sum::<f64>() / edge_res.len() as f64).sqrt()})
                    })
                    .collect();
                json!({"eyes": eyes, "measured": measured_json(&measure_built(h, &s, world))})
            };
            let rows: Vec<Value> = parallel(world.heads.len(), threads(), |i| {
                let h = &world.heads[i];
                json!({"name": h.name, "today": one(&originals[i], h), "pasted": one(&pasted[i], h)})
            });
            let base_m = measured_json(&EYES.measure(&Surface::within(&base_s, &world.mesh.triangles, &bframe, &EYES.window())).ok());
            let out = json!({"base": base_m, "heads": rows, "proportions": EYES.proportions().iter().map(|p| p.key).collect::<Vec<_>>()});
            std::fs::write(data.join("eyes/starts_audit.json"), serde_json::to_string(&out).unwrap()).unwrap();
            eprintln!("wrote eyes/starts_audit.json");
        }
        "sweep" => {
            let page = read_json(&data.join("web/eyes/page.json"));
            let names = &args[3..];
            let steps = 13;
            let n = set.sliders.len();
            let found: Vec<usize> = names.iter().filter_map(|nm| world.heads.iter().position(|h| &h.name == nm)).collect();
            let measured: Vec<usize> = (0..n).filter(|&j| !set.sliders[j].measure.is_empty()).collect();
            let measured = &measured;
            let jobs: Vec<(usize, usize, usize)> =
                found.iter().flat_map(|&i| measured.iter().flat_map(move |&j| (0..steps).map(move |s| (i, j, s)))).collect();
            let res: Vec<Value> = parallel(jobs.len(), threads(), |q| {
                let (i, j, s) = jobs[q];
                let h = &world.heads[i];
                let sl = &set.sliders[j];
                let mut values = tools::solved_values(&page, &set, &h.name);
                let v = sl.low + (sl.high - sl.low) * s as f64 / (steps - 1) as f64;
                values[j] = v;
                let k = proportion_index(&EYES, &sl.measure);
                let lower = head_sliders::eyes::cover_of(&set, &values);
                let pressed = measure_lowered(h, &set.build(&world.smoother, &h.start, &values), world, lower).map(|m| m.proportions[k]);
                let free = measure_built(h, &smoothed(world, &set.unsmoothed(&h.start, &values)), world).map(|m| m.proportions[k]);
                json!({"head": h.name, "slider": sl.name, "value": v, "pressed": pressed, "free": free})
            });
            std::fs::write(data.join("eyes/sweep.json"), serde_json::to_string(&res).unwrap()).unwrap();
            eprintln!("wrote eyes/sweep.json ({} builds)", jobs.len());
        }
        "iris" => {
            let rig = tools::lid_rig(&data, &world.mesh);
            let page = read_json(&data.join("web/eyes/page.json"));
            let rows: Vec<Value> = parallel(world.heads.len(), threads(), |i| {
                let h = &world.heads[i];
                let ks = tools::kotor_scene(&h.name, &h.kotor_v, &h.kotor_f, &h.frame);
                let kotor_eyes: &[head_sliders::eyes::kotor::KotorEye] = head_sliders::eyes::kotor::all().get(h.name.as_str()).map(|v| &v[..]).unwrap_or(&[]);
                let kotor: Vec<Value> = kotor_eyes
                    .iter()
                    .map(|e| {
                        let mut v = iris_cover(&ks, h.frame.local(e.centre), e.radius * h.frame.scale, KOTOR_LID);
                        v["lid_triangles"] = json!(e.lid.len());
                        v["x"] = json!(h.frame.local(e.centre)[0]);
                        v
                    })
                    .collect();
                let values = tools::solved_values(&page, &set, &h.name);
                tools::caruncle_from(&set, &values);
                let built = set.build(&world.smoother, &h.start, &values);
                let lower = head_sliders::eyes::cover_of(&set, &values)[0];
                let (os, _) = tools::our_scene(&rig, &built, &h.frame, 0.0, &head_eyes::piece::Settings { lower, ..Default::default() });
                let ours: Vec<Value> = rig
                    .eyes
                    .iter()
                    .map(|e| {
                        let c = e.centre.iter().fold([0.0; 3], |a, &i| add(a, scale(built[i as usize], 1.0 / e.centre.len() as f64)));
                        let r = head_eyes::piece::eyeball_radius(&built, &e.centre) / 1000.0 * h.frame.scale;
                        let mut v = iris_cover(&os, h.frame.local(c), r, PIECE);
                        v["x"] = json!(h.frame.local(c)[0]);
                        v
                    })
                    .collect();
                json!({"name": h.name, "kotor": kotor, "ours": ours})
            });
            std::fs::write(data.join("eyes/iris.json"), serde_json::to_string(&rows).unwrap()).unwrap();
            eprintln!("wrote eyes/iris.json");
        }
        "pictures" => {
            let exp = PathBuf::from(args.get(3).expect("experiment data folder"));
            let names = &args[4..];
            let out = PathBuf::from(std::env::var("EA_OUT").unwrap_or_else(|_| ".".into()));
            std::fs::create_dir_all(&out).unwrap();
            let rig = tools::lid_rig(&data, &world.mesh);
            let today = read_json(&data.join("web/eyes/page.json"));
            let later = read_json(&exp.join("web/eyes/page.json"));
            let close = Shot { name: "close", yaw: -40.0, pitch: 0.0, pivot: 0.0, window: [6.0, 36.0, -10.0, 10.0] };
            let front = Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [12.0, 52.0, -12.0, 12.0] };
            let found: Vec<usize> = names.iter().filter_map(|nm| world.heads.iter().position(|h| &h.name == nm)).collect();
            for (shot, file) in [(&close, "close"), (&front, "front")] {
                let rows: Vec<Vec<RgbImage>> = parallel(found.len(), threads(), |j| {
                    let i = found[j];
                    let h = &world.heads[i];
                    let ks = tools::kotor_scene(&h.name, &h.kotor_v, &h.kotor_f, &h.frame);
                    let scene_of = |page: &Value, start: &[V3]| {
                        let values = tools::solved_values(page, &set, &h.name);
                        tools::caruncle_from(&set, &values);
                        let built = set.build(&world.smoother, start, &values);
                        tools::our_scene(&rig, &built, &h.frame, 0.0, &head_eyes::piece::Settings::default()).0
                    };
                    let a = scene_of(&today, &originals[i]);
                    let b = scene_of(&later, &pasted[i]);
                    let mut row = Vec::new();
                    for (k, (s, tag)) in [(&ks, "KOTOR"), (&a, "TODAY"), (&b, "EXPERIMENT")].iter().enumerate() {
                        let mut img = s.draw(shot);
                        head_sliders::render::label(&mut img, &if k == 0 { format!("{} {tag}", h.name.replace('_', " ").to_uppercase()) } else { tag.to_string() });
                        row.push(img);
                    }
                    for (s, tag) in [(&ks, "KOTOR LIGHT"), (&a, "TODAY LIGHT"), (&b, "EXPERIMENT LIGHT")] {
                        let mut img = lit(s, shot);
                        head_sliders::render::label(&mut img, tag);
                        row.push(img);
                    }
                    row
                });
                let path = out.join(format!("{file}.png"));
                tools::grid(&rows.into_iter().flatten().collect::<Vec<_>>(), 6).save(&path).unwrap();
                eprintln!("wrote {}", path.display());
            }
        }
        "r4-pictures" => {
            // `eye_audit r4-pictures <data> <before folder> <heads...>`: KOTOR, the live preset before round 4 (its own set, page and
            // start without the pasted eye area) and the round 4 preset, in clay and in viewer 2's light. EA_OUT names the folder;
            // EA_MODE=close (three-quarter close-ups, open and closed) or EA_MODE=views (front, side, three-quarter, above).
            let before_dir = PathBuf::from(args.get(3).expect("before folder"));
            let names = &args[4..];
            let out = PathBuf::from(std::env::var("EA_OUT").unwrap_or_else(|_| ".".into()));
            std::fs::create_dir_all(&out).unwrap();
            let mode = std::env::var("EA_MODE").unwrap_or_else(|_| "close".into());
            let rig = tools::lid_rig(&data, &world.mesh);
            let old_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(before_dir.join("web_eyes/sliders.json")).unwrap()).unwrap();
            let old_page = read_json(&before_dir.join("web_eyes/page.json"));
            let new_page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            let close = Shot { name: "close", yaw: -40.0, pitch: 0.0, pivot: 0.0, window: [8.0, 38.0, -11.0, 9.0] };
            let views = [
                Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-52.0, 52.0, -22.0, 18.0] },
                Shot { name: "side", yaw: -90.0, pitch: 0.0, pivot: 0.0, window: [-30.0, 26.0, -20.0, 20.0] },
                Shot { name: "3q", yaw: -35.0, pitch: 0.0, pivot: 0.0, window: [0.0, 52.0, -20.0, 20.0] },
                head_sliders::eyes::ABOVE,
            ];
            let found: Vec<usize> = names.iter().filter_map(|nm| world.heads.iter().position(|h| &h.name == nm)).collect();
            let rows: Vec<Vec<RgbImage>> = parallel(found.len(), threads(), |j| {
                let i = found[j];
                let h = &world.heads[i];
                let ks = tools::kotor_scene(&h.name, &h.kotor_v, &h.kotor_f, &h.frame);
                let scene_of = |set: &SliderSet, page: &Value, start: &[V3], shut: f64| {
                    let values = tools::solved_values(page, set, &h.name);
                    tools::caruncle_from(set, &values);
                    let built = set.build(&world.smoother, start, &values);
                    let lower = head_sliders::eyes::cover_of(set, &values)[0];
                    tools::our_scene(&rig, &built, &h.frame, shut, &head_eyes::piece::Settings { lower, ..Default::default() }).0
                };
                let tag = h.name.replace('_', " ").to_uppercase();
                // EA_BLIND=1 names the two presets X and Y, swapped on every other head, and prints which is which
                let blind = std::env::var("EA_BLIND").is_ok_and(|v| v == "1");
                let swap = blind && h.name.bytes().map(|b| b as usize).sum::<usize>() % 2 == 1;
                let (n_before, n_after) = if !blind { ("BEFORE", "AFTER") } else if swap { ("Y", "X") } else { ("X", "Y") };
                if blind {
                    eprintln!("key {}: X is {}", h.name, if swap { "after" } else { "before" });
                }
                let mut row = Vec::new();
                if mode == "views" {
                    let a = scene_of(&old_set, &old_page, &originals[i], 0.0);
                    let b = scene_of(&full_set, &new_page, &h.start, 0.0);
                    let order = if swap { [(&ks, "KOTOR"), (&b, n_after), (&a, n_before)] } else { [(&ks, "KOTOR"), (&a, n_before), (&b, n_after)] };
                    for (s, name) in order {
                        for (k, shot) in views.iter().enumerate() {
                            let mut img = s.draw(shot);
                            if k == 0 {
                                head_sliders::render::label(&mut img, &format!("{tag} {name}"));
                            }
                            row.push(img);
                        }
                        for shot in &views[..1] {
                            let mut img = lit(s, shot);
                            head_sliders::render::label(&mut img, &format!("{name} LIGHT"));
                            row.push(img);
                        }
                    }
                } else {
                    let a = scene_of(&old_set, &old_page, &originals[i], 0.0);
                    let b = scene_of(&full_set, &new_page, &h.start, 0.0);
                    let bc = scene_of(&full_set, &new_page, &h.start, 1.0);
                    let order = if swap { [(&ks, "KOTOR"), (&b, n_after), (&a, n_before)] } else { [(&ks, "KOTOR"), (&a, n_before), (&b, n_after)] };
                    for (k, (s, name)) in order.iter().enumerate() {
                        let mut img = s.draw(&close);
                        head_sliders::render::label(&mut img, &if k == 0 { format!("{tag} {name}") } else { name.to_string() });
                        row.push(img);
                    }
                    for (s, name) in order {
                        let mut img = lit(s, &close);
                        head_sliders::render::label(&mut img, &format!("{name} LIGHT"));
                        row.push(img);
                    }
                    if !blind {
                        let mut img = bc.draw(&close);
                        head_sliders::render::label(&mut img, "AFTER CLOSED");
                        row.push(img);
                    }
                    let above = Shot { name: "above close", yaw: -20.0, pitch: -40.0, pivot: 0.0, window: [12.0, 44.0, -18.0, 3.0] };
                    for (s, name) in order {
                        let mut img = s.draw(&above);
                        head_sliders::render::label(&mut img, &format!("{name} ABOVE"));
                        row.push(img);
                    }
                }
                row
            });
            let columns = if mode == "views" { 5 } else { rows.first().map_or(1, |r| r.len()) };
            let path = out.join(format!("{mode}.png"));
            tools::grid(&rows.into_iter().flatten().collect::<Vec<_>>(), columns).save(&path).unwrap();
            eprintln!("wrote {}", path.display());
        }
        "inner" => {
            // `eye_audit inner <data> <heads...>` ("base" is the base head at 0): the +x eye's inner corner on the set's presets, from the
            // front and from three-quarter on the nose side and the temple side, in clay and in viewer 2's light, open (EA_CLOSED=1 shut).
            // EA_SET="<slider>=<value>;..." puts values on top. EA_OUT names the folder, EA_NAME the file.
            let names = &args[3..];
            let out = PathBuf::from(std::env::var("EA_OUT").unwrap_or_else(|_| ".".into()));
            std::fs::create_dir_all(&out).unwrap();
            let rig = tools::lid_rig(&data, &world.mesh);
            let page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            let shut = if std::env::var("EA_CLOSED").is_ok_and(|v| v == "1") { 1.0 } else { 0.0 };
            let shots = if std::env::var("EA_WIDE").is_ok_and(|v| v == "1") {
                // the whole eye: front, three-quarter from the temple side (looking across the eye at the inner corner), from below
                // on the temple side (the lower lid's line), from above
                [
                    Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [12.0, 52.0, -14.0, 12.0] },
                    Shot { name: "three-quarter", yaw: -40.0, pitch: 0.0, pivot: 0.0, window: [-2.0, 38.0, -14.0, 12.0] },
                    Shot { name: "three-quarter low", yaw: -35.0, pitch: 15.0, pivot: 0.0, window: [0.0, 40.0, -16.0, 10.0] },
                    Shot { name: "above", yaw: -20.0, pitch: -50.0, pivot: 0.0, window: [6.0, 46.0, -26.0, 0.0] },
                ]
            } else {
                [
                    Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [10.0, 34.0, -10.0, 6.0] },
                    Shot { name: "nose side", yaw: 35.0, pitch: 0.0, pivot: 0.0, window: [12.0, 36.0, -10.0, 6.0] },
                    Shot { name: "temple side", yaw: -40.0, pitch: 0.0, pivot: 0.0, window: [4.0, 28.0, -10.0, 6.0] },
                    Shot { name: "above", yaw: 15.0, pitch: -45.0, pivot: 0.0, window: [10.0, 34.0, -16.0, 0.0] },
                ]
            };
            // EA_VARIANTS="<label>|<slider>=<value>,...;;..." draws the first head once per variant instead (`closed` in a label shuts it)
            let variants: Vec<(String, String)> = std::env::var("EA_VARIANTS")
                .unwrap_or_default()
                .split(";;")
                .filter(|a| !a.is_empty())
                .map(|a| {
                    let (l, v) = a.split_once('|').unwrap_or((a, ""));
                    (l.to_string(), v.replace(',', ";"))
                })
                .collect();
            let count = if variants.is_empty() { names.len() } else { variants.len() };
            let rows: Vec<Vec<RgbImage>> = parallel(count, threads(), |j| {
                let nm = if variants.is_empty() { &names[j] } else { &names[0] };
                let extra = variants.get(j).map(|v| v.1.clone()).unwrap_or_default();
                let shut = if variants.get(j).is_some_and(|v| v.0.contains("closed")) { 1.0 } else { shut };
                let (start, frame, mut values) = if nm == "base" {
                    (world.mesh.base.clone(), head_sliders::measure::head_frame(world.mesh.eyes), vec![0.0; full_set.sliders.len()])
                } else {
                    let h = world.heads.iter().find(|h| &h.name == nm).expect("head");
                    (h.start.clone(), h.frame, tools::solved_values(&page, &full_set, &h.name))
                };
                for a in std::env::var("EA_SET").unwrap_or_default().split(';').chain(extra.split(';')).filter(|a| !a.is_empty()) {
                    let (n, v) = a.split_once('=').unwrap();
                    let k = full_set.sliders.iter().position(|s| s.name == n).unwrap_or_else(|| panic!("no slider {n}"));
                    values[k] = v.parse().unwrap();
                }
                tools::caruncle_from(&full_set, &values);
                let built = full_set.build(&world.smoother, &start, &values);
                let lower = head_sliders::eyes::cover_of(&full_set, &values)[0];
                let (scene, _) = tools::our_scene(&rig, &built, &frame, shut, &head_eyes::piece::Settings { lower, ..Default::default() });
                let mut row = Vec::new();
                for (k, shot) in shots.iter().enumerate() {
                    let mut a = scene.draw(shot);
                    if k == 0 {
                        let l = variants.get(j).map(|v| format!("{} {}", nm, v.0)).unwrap_or_else(|| nm.to_string());
                        head_sliders::render::label(&mut a, &l.replace('_', " ").to_uppercase());
                    }
                    row.push(a);
                    row.push(lit(&scene, shot));
                }
                row
            });
            let path = out.join(std::env::var("EA_NAME").unwrap_or_else(|_| "inner.png".into()));
            tools::grid(&rows.into_iter().flatten().collect::<Vec<_>>(), 8).save(&path).unwrap();
            eprintln!("wrote {}", path.display());
        }
        "edgegap" => {
            // the +x eye's edge vertices near the inner corner on a head's preset ("base" at 0): share from the inner corner, upper or
            // lower, distance outside the eyeball (mm) unpressed and pressed, and distance from the eye's axis seen from the front over r
            let page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            for nm in &args[3..] {
                let (start, values) = if nm == "base" {
                    (world.mesh.base.clone(), vec![0.0; full_set.sliders.len()])
                } else {
                    let h = world.heads.iter().find(|h| &h.name == nm).unwrap();
                    (h.start.clone(), tools::solved_values(&page, &full_set, &h.name))
                };
                let free = smoothed(world, &full_set.unsmoothed(&start, &values));
                let built = full_set.build(&world.smoother, &start, &values);
                let e = &eye_curves::edges()[0];
                let (c, r) = eye_curves::ball(&built, &e.ring);
                let ax = head_sliders::measure::Frame::head().axes[2];
                let mut rows: Vec<(f64, String)> = Vec::new();
                for (k, &v) in e.edge.iter().enumerate() {
                    if e.share[k] > 0.15 { continue; }
                    let g = |p: &[V3]| length(sub(p[v as usize], c)) * 1000.0 - r;
                    let q = scale(sub(built[v as usize], c), 1000.0);
                    let along = head_sliders::dot(q, ax);
                    let rho = length(sub(q, scale(ax, along)));
                    rows.push((e.share[k] * if e.upper[k] { 1.0 } else { -1.0 }, format!("{} {:.3} free {:.2} pressed {:.2} front {:.2}", if e.upper[k] { "up" } else { "lo" }, e.share[k], g(&free), g(&built), rho / r)));
                }
                rows.sort_by(|a, b| a.0.total_cmp(&b.0));
                eprintln!("{nm}: r {r:.2}");
                for (_, l) in rows { eprintln!("  {l}"); }
            }
        }
        "foldmap" => {
            // `eye_audit foldmap <data>`: every faulty triangle of every head's preset (web/eyes/page.json), its centre on the start in
            // the frame of the eyes (mm), written to eyes/foldmap.json
            let page = read_json(&data.join("web/eyes/page.json"));
            let rows: Vec<Value> = parallel(world.heads.len(), threads(), |i| {
                let h = &world.heads[i];
                let values = tools::solved_values(&page, &set, &h.name);
                let before = smoothed(world, &h.start);
                let window = EYES.window();
                let near: Vec<u32> = (0..before.len() as u32).filter(|&v| window.contains(h.frame.local(before[v as usize]))).collect();
                let checker = Checker::new(&world.mesh, &before, &near);
                let built = set.build(&world.smoother, &h.start, &values);
                let marks = checker.marks(&built);
                let at: Vec<[f64; 3]> = marks
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| **m)
                    .map(|(k, _)| {
                        let t = world.mesh.triangles[k];
                        let c = scale(add(add(before[t[0] as usize], before[t[1] as usize]), before[t[2] as usize]), 1.0 / 3.0);
                        h.frame.local(c)
                    })
                    .collect();
                json!({"name": h.name, "at": at})
            });
            std::fs::write(data.join("eyes/foldmap.json"), serde_json::to_string(&rows).unwrap()).unwrap();
            eprintln!("wrote eyes/foldmap.json");
        }
        "tipdepth" => {
            // `eye_audit tipdepth <data> <heads>` ("base" is the base head at 0, "all" every head): how far each inner corner's tip lies
            // behind the face around it (the mean front-most depth 8 mm toward the nose, 8 mm above and 8 mm below the tip, seen from
            // the front), in mm, on the preset of web/eyes/page.json, both eyes averaged; and the tip's distance outside its eyeball.
            // EA_SET="<slider>=<value>;..." puts values on top.
            let page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            let mut names: Vec<String> = args[3..].to_vec();
            if names.iter().any(|n| n == "all") {
                names = world.heads.iter().map(|h| h.name.clone()).collect();
            }
            let rows: Vec<(String, f64, f64, f64)> = parallel(names.len(), threads(), |j| {
                let nm = &names[j];
                let (start, frame, mut values) = if nm == "base" {
                    (world.mesh.base.clone(), head_sliders::measure::head_frame(world.mesh.eyes), vec![0.0; full_set.sliders.len()])
                } else {
                    let h = world.heads.iter().find(|h| &h.name == nm).expect("head");
                    (h.start.clone(), h.frame, tools::solved_values(&page, &full_set, &h.name))
                };
                for a in std::env::var("EA_SET").unwrap_or_default().split(';').filter(|a| !a.is_empty()) {
                    let (n, v) = a.split_once('=').unwrap();
                    let k = full_set.sliders.iter().position(|s| s.name == n).unwrap_or_else(|| panic!("no slider {n}"));
                    values[k] = v.parse().unwrap();
                }
                let built = full_set.build(&world.smoother, &start, &values);
                let surface = head_sliders::eyes::shown_surface(&built, &world.mesh.triangles, &frame, head_sliders::eyes::cover_of(&full_set, &values));
                let (mut behind, mut gap, mut behind_lids) = (0.0, 0.0, 0.0);
                for e in eye_curves::edges() {
                    let t = frame.local(built[e.inner as usize]);
                    let toward = if t[0] > 0.0 { -8.0 } else { 8.0 };
                    let side = head_sliders::brow::upright(&surface, t[0] + toward, t[1] - 10.0, t[1] + 10.0).at(t[1]);
                    let mid = head_sliders::brow::upright(&surface, t[0], t[1] - 10.0, t[1] + 10.0);
                    let r = (side + mid.at(t[1] + 8.0) + mid.at(t[1] - 8.0)) / 3.0;
                    behind += 0.5 * (r - t[2]);
                    // the lid edges 3 mm from the tip, seen from the front
                    let mut lids = 0.0;
                    for up in [true, false] {
                        let best = e.edge.iter().zip(&e.upper).filter(|(_, u)| **u == up).map(|(&v, _)| frame.local(built[v as usize])).min_by(|a, b| {
                            let da = ((a[0] - t[0]).hypot(a[1] - t[1]) - 3.0).abs();
                            let db = ((b[0] - t[0]).hypot(b[1] - t[1]) - 3.0).abs();
                            da.total_cmp(&db)
                        });
                        lids += 0.25 * best.map_or(f64::NAN, |q| q[2] - t[2]);
                    }
                    behind_lids += lids;
                    let (c, rad) = eye_curves::ball(&built, &e.ring);
                    gap += 0.5 * (length(sub(built[e.inner as usize], c)) * 1000.0 - rad);
                }
                (nm.clone(), behind, gap, behind_lids)
            });
            for (nm, b, g, l) in &rows {
                eprintln!("{nm:24} tip {b:6.2} mm behind the face 8 mm around it, {l:5.2} mm behind the lid edges 3 mm away, {g:5.2} mm off the eyeball");
            }
            let mut b: Vec<f64> = rows.iter().map(|r| r.1).filter(|x| x.is_finite()).collect();
            b.sort_by(f64::total_cmp);
            if !b.is_empty() {
                eprintln!("median {:.2}, largest {:.2}", b[b.len() / 2], b[b.len() - 1]);
            }
        }
        "faultsat" => {
            // `eye_audit faultsat <data> <heads|all>`: the faults (fold check) of each head's preset of web/eyes/page.json with the full set
            // of web/eyes/sliders.json, with EA_SET="<slider>=<value>;..." on top ("base" is the base head at 0)
            let page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            let mut names: Vec<String> = args[3..].to_vec();
            if names.iter().any(|n| n == "all") {
                names = world.heads.iter().map(|h| h.name.clone()).collect();
            }
            let rows: Vec<(String, usize)> = parallel(names.len(), threads(), |j| {
                let nm = &names[j];
                let h = if nm == "base" { &world.heads[0] } else { world.heads.iter().find(|h| &h.name == nm).expect("head") };
                let mut values = if nm == "base" { vec![0.0; full_set.sliders.len()] } else { tools::solved_values(&page, &full_set, &h.name) };
                for a in std::env::var("EA_SET").unwrap_or_default().split(';').filter(|a| !a.is_empty()) {
                    let (n, v) = a.split_once('=').unwrap();
                    let k = full_set.sliders.iter().position(|s| s.name == n).unwrap_or_else(|| panic!("no slider {n}"));
                    values[k] = v.parse().unwrap();
                }
                if nm == "base" {
                    let mut hb = Head { name: "base".into(), title: String::new(), is_mod: false, frame: head_sliders::measure::head_frame(world.mesh.eyes), eyes: world.mesh.eyes, kotor_v: Vec::new(), kotor_f: Vec::new(), kotor_skin: None, kotor: h.kotor.clone(), kotor_error: None, start: world.mesh.base.clone(), start_nose: None };
                    hb.start = world.mesh.base.clone();
                    return (nm.clone(), faults(world, &hb, &full_set, &values, true));
                }
                if std::env::var("EA_WHERE").is_ok_and(|v| v == "1") {
                    let before = smoothed(world, &h.start);
                    let window = EYES.window();
                    let near: Vec<u32> = (0..before.len() as u32).filter(|&v| window.contains(h.frame.local(before[v as usize]))).collect();
                    let checker = Checker::new(&world.mesh, &before, &near);
                    let built = full_set.build(&world.smoother, &h.start, &values);
                    for (k, m) in checker.marks(&built).iter().enumerate() {
                        if *m {
                            let t = world.mesh.triangles[k];
                            let c = scale(add(add(before[t[0] as usize], before[t[1] as usize]), before[t[2] as usize]), 1.0 / 3.0);
                            let l = h.frame.local(c);
                            // how far the built triangle's corners lie outside the nearer eyeball, in mm (least and most)
                            let out: Vec<f64> = t.iter().map(|&v| {
                                head_sliders::eye_curves::edges().iter().map(|e| {
                                    let (bc, br) = head_sliders::eye_curves::ball(&built, &e.ring);
                                    head_sliders::length(head_sliders::sub(built[v as usize], bc)) * 1000.0 - br
                                }).fold(f64::INFINITY, f64::min)
                            }).collect();
                            println!("FOLD {} {:.1} {:.1} {:.1} out {:.2} {:.2}", h.name, l[0], l[1], l[2], out.iter().cloned().fold(f64::INFINITY, f64::min), out.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
                        }
                    }
                }
                (nm.clone(), faults(world, h, &full_set, &values, true))
            });
            let bad: Vec<String> = rows.iter().filter(|r| r.1 > 0).map(|r| format!("{} {}", r.0, r.1)).collect();
            eprintln!("faulty {} of {}, triangles {}: {}", bad.len(), rows.len(), rows.iter().map(|r| r.1).sum::<usize>(), bad.join(", "));
        }
        "depthcheck" => {
            // `eye_audit depthcheck <data> <heads>` ("base" is the base head at 0): Inner corner depth at its low end, 0 and its high end
            // on each head's preset (EA_SET on top): the faults, the inner tips' place seen from the front (mm, both eyes), the largest
            // change of the opening's front-view outline against 0 (mm), and the tips' depth
            let page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            let j = full_set.sliders.iter().position(|s| s.name == "Inner corner depth").expect("Inner corner depth");
            let ends = [full_set.sliders[j].low, 0.0, full_set.sliders[j].high];
            for nm in &args[3..] {
                let (h, mut values) = if nm == "base" {
                    (&world.heads[0], vec![0.0; full_set.sliders.len()])
                } else {
                    let h = world.heads.iter().find(|h| &h.name == nm).expect("head");
                    (h, tools::solved_values(&page, &full_set, &h.name))
                };
                for a in std::env::var("EA_SET").unwrap_or_default().split(';').filter(|a| !a.is_empty()) {
                    let (n, v) = a.split_once('=').unwrap();
                    let k = full_set.sliders.iter().position(|s| s.name == n).unwrap_or_else(|| panic!("no slider {n}"));
                    values[k] = v.parse().unwrap();
                }
                let (start, frame) = if nm == "base" { (world.mesh.base.clone(), head_sliders::measure::head_frame(world.mesh.eyes)) } else { (h.start.clone(), h.frame) };
                let mut hb = Head { name: nm.clone(), title: String::new(), is_mod: false, frame, eyes: h.eyes, kotor_v: Vec::new(), kotor_f: Vec::new(), kotor_skin: None, kotor: h.kotor.clone(), kotor_error: None, start: start.clone(), start_nose: None };
                hb.start = start.clone();
                let rows: Vec<(usize, Vec<V3>, Option<head_sliders::feature::Measured>)> = parallel(3, 3, |i| {
                    let mut v = values.clone();
                    v[j] = ends[i];
                    let built = full_set.build(&world.smoother, &start, &v);
                    let tips: Vec<V3> = eye_curves::edges().iter().map(|e| frame.local(built[e.inner as usize])).collect();
                    let m = head_sliders::eyes::measure(&head_sliders::eyes::shown_surface(&built, &world.mesh.triangles, &frame, head_sliders::eyes::cover_of(&full_set, &v))).ok();
                    (faults(world, &hb, &full_set, &v, true), tips, m)
                });
                for (i, (f, tips, m)) in rows.iter().enumerate() {
                    let d = match (m, &rows[1].2) {
                        (Some(a), Some(b)) => a.outlines.iter().zip(&b.outlines).flat_map(|(x, y)| x.iter().zip(y).map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))).fold(0.0, f64::max),
                        _ => f64::NAN,
                    };
                    let tipmove = tips.iter().zip(&rows[1].1).map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1])).fold(0.0, f64::max);
                    eprintln!("{nm:20} depth {:5.1}: faults {f:3}, tips seen from the front moved {tipmove:.2} mm, outline moved {d:.2} mm at most, tip depth {:.2} mm", ends[i], 0.5 * (tips[0][2] + tips[1][2]));
                }
            }
        }
        "startfaults" => {
            // `eye_audit startfaults <data>`: per head, the triangles of the smoothed start, in the eye window and up to 40 mm below,
            // turned over or folded against the smoothed base head, and where (frame of the eyes, mm)
            let base_s = smoothed(world, &world.mesh.base);
            let bframe = Frame::from_eyes(world.mesh.eyes);
            let near: Vec<u32> = (0..base_s.len() as u32)
                .filter(|&v| {
                    let p = bframe.local(base_s[v as usize]);
                    p[0].abs() < 60.0 && p[1] > -40.0 && p[1] < 30.0 && p[2] > -30.0
                })
                .collect();
            let checker = Checker::new(&world.mesh, &base_s, &near);
            let mut total = 0;
            let mut heads = 0;
            for h in &world.heads {
                let f = checker.faults(&smoothed(world, &h.start));
                let n = f.flipped + f.folded;
                if n > 0 {
                    heads += 1;
                    total += n;
                    let at: Vec<String> = f.at.iter().take(4).map(|p| { let q = bframe.local(*p); format!("({:.0},{:.0},{:.0})", q[0], q[1], q[2]) }).collect();
                    eprintln!("{:24} {:3} {}", h.name, n, at.join(" "));
                }
            }
            eprintln!("starts with a fault: {heads}, faulty triangles {total}");
        }
        "browcut" => {
            // `eye_audit browcut <data> <heads>`: the stand-in's upright cut 26 mm from the middle (both sides), depth every 2 mm from
            // 0 to 30 mm above the eyes, as the brow measure reads it, and the brow measure's error if it fails
            let window = head_sliders::brow::BROW.window();
            let bw = World::load(&input, &head_sliders::brow::BROW);
            for nm in &args[3..] {
                let h = bw.heads.iter().find(|h| &h.name == nm).unwrap();
                let s = smoothed(world, &h.start);
                let surface = Surface::within(&s, &world.mesh.triangles, &h.frame, &window);
                for x in [-42.0, -38.0, -34.0, -30.0, -26.0, -22.0, -18.0, -14.0, 14.0, 18.0, 22.0, 26.0, 30.0, 34.0, 38.0, 42.0] {
                    let c = head_sliders::brow::upright(&surface, x, -2.0, 40.0);
                    let row: Vec<String> = (0..=21).map(|k| format!("{:.1}", c.at(-2.0 + 2.0 * k as f64))).collect();
                    eprintln!("{nm} x {x}: {}", row.join(" "));
                }
                eprintln!("{nm} brow measure: {:?}", head_sliders::brow::measure(&surface).map(|m| m.proportions[1]));
            }
        }
        "piecetime" => {
            let rig = tools::lid_rig(&data, &world.mesh);
            let h = &world.heads[0];
            let built = set.build(&world.smoother, &h.start, &vec![0.0; set.sliders.len()]);
            let t = std::time::Instant::now();
            for _ in 0..20 {
                let _ = head_eyes::piece::build(&rig, &built, &head_eyes::piece::Settings::default());
            }
            eprintln!("one build of both pieces: {:.2?}", t.elapsed() / 20);
        }
        "wavy" => {
            // `eye_audit wavy <data> <heads|all>` ("base" is the base head at 0): the lid edges' waviness (`waviness`) on each head's
            // preset of web/eyes/page.json with the set of web/eyes/sliders.json (EA_SET on top)
            let page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            let mut names: Vec<String> = args[3..].to_vec();
            if names.iter().any(|n| n == "all") {
                names = world.heads.iter().map(|h| h.name.clone()).collect();
            }
            if args[3..].iter().any(|n| n == "base") && !names.iter().any(|n| n == "base") {
                names.insert(0, "base".into());
            }
            let rows: Vec<(String, [f64; 3])> = parallel(names.len(), threads(), |j| {
                let nm = &names[j];
                let (start, frame, values) = if nm == "base" {
                    (world.mesh.base.clone(), head_sliders::measure::head_frame(world.mesh.eyes), vec![0.0; full_set.sliders.len()])
                } else {
                    let h = world.heads.iter().find(|h| &h.name == nm).expect("head");
                    (h.start.clone(), h.frame, tools::solved_values(&page, &full_set, &h.name))
                };
                let values = with_ea_set(&full_set, values);
                (nm.clone(), waviness(&full_set.build(&world.smoother, &start, &values), &frame))
            });
            for (nm, w) in &rows {
                eprintln!("{nm:24} largest turn against the bend {:.1} deg, depth bend against {:.3} /mm, points turning back {}", w[0], w[1], w[2]);
            }
            let n = rows.len() as f64;
            let mean = |k: usize| rows.iter().map(|r| r.1[k]).sum::<f64>() / n;
            let max = |k: usize| rows.iter().map(|r| r.1[k]).fold(0.0, f64::max);
            eprintln!("WAVY mean of heads: turn against the bend {:.2} deg, depth bend against {:.3} /mm, points turning back {:.2}; largest: {:.1} deg, {:.3} /mm, {:.0} points", mean(0), mean(1), mean(2), max(0), max(1), max(2));
        }
        "zones" | "shown" => {
            // `eye_audit zones <data> <heads|all>`: the faulty triangles of each head's eye preset (eye set, eye starts, as the kit), by
            // place (`zone_of`). `shown` does the same on viewer 2's shown head (`shown_heads`) and gives, for the heads with a fault on the
            // side of the nose, the sliders whose zeroing removes those triangles. EA_SET on top (eye sliders by name).
            let shown = command == "shown";
            let blame_zone = std::env::var("EA_BLAME_ZONE").unwrap_or_else(|_| "nose side".into());
            let page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            let (merged, shown_rows) = if shown { shown_heads(&data, world.mesh.base.len()) } else { (full_set.clone(), Vec::new()) };
            let eyes_page = read_json(&data.join("web/page.json"));
            let mut names: Vec<String> = args[3..].to_vec();
            if names.iter().any(|n| n == "all") {
                names = world.heads.iter().map(|h| h.name.clone()).collect();
            }
            let rows: Vec<(String, Vec<(&'static str, usize)>, String)> = parallel(names.len(), threads(), |j| {
                let nm = &names[j];
                let (set, start, frame, values) = if shown {
                    let r = shown_rows.iter().find(|r| &r.0 == nm).expect("head in heads.json");
                    let eh = eyes_page["heads"].as_array().unwrap().iter().find(|x| x["name"] == nm.as_str()).unwrap();
                    let ev = |k: usize| -> V3 { [0, 1, 2].map(|c| eh["eyes"][k][c].as_f64().unwrap()) };
                    (&merged, r.1.clone(), head_sliders::measure::head_frame([ev(0), ev(1)]), with_ea_set(&merged, r.2.clone()))
                } else {
                    let h = world.heads.iter().find(|h| &h.name == nm).expect("head");
                    (&full_set, h.start.clone(), h.frame, with_ea_set(&full_set, tools::solved_values(&page, &full_set, &h.name)))
                };
                let before = smoothed(world, &start);
                let window = EYES.window();
                let near: Vec<u32> = (0..before.len() as u32).filter(|&v| window.contains(frame.local(before[v as usize]))).collect();
                let hidden: Vec<bool> = (0..before.len()).map(|i| world.mesh.hidden[world.mesh.group[i] as usize]).collect();
                let checker = Checker::from_parts(&world.mesh.triangles, &hidden, &before, &near);
                let built = set.build(&world.smoother, &start, &values);
                let marks = checker.marks(&built);
                let mut z: Vec<(&'static str, usize)> = vec![("kit count", checker.faults(&built).total())];
                let mut nose = Vec::new();
                for (k, m) in marks.iter().enumerate() {
                    if *m {
                        let zn = zone_of(world, world.mesh.triangles[k]);
                        if zn == blame_zone.as_str() { nose.push(k); }
                        match z.iter_mut().find(|x| x.0 == zn) { Some(x) => x.1 += 1, None => z.push((zn, 1)) }
                    }
                }
                let mut blame = String::new();
                if (shown || std::env::var("EA_BLAME_ZONE").is_ok()) && !nose.is_empty() {
                    let mut b: Vec<(usize, String)> = Vec::new();
                    for k in 0..values.len() {
                        if values[k] == 0.0 { continue; }
                        let mut v = values.clone();
                        v[k] = 0.0;
                        let after = checker.marks(&set.build(&world.smoother, &start, &v));
                        let gone = nose.iter().filter(|&&t| !after[t]).count();
                        if gone > 0 { b.push((gone, format!("{} ({:.2})", set.sliders[k].name, values[k]))); }
                    }
                    b.sort_by(|a, c| c.0.cmp(&a.0));
                    let at: Vec<String> = nose.iter().take(3).map(|&t| { let tr = world.mesh.triangles[t]; let c = scale(add(add(before[tr[0] as usize], before[tr[1] as usize]), before[tr[2] as usize]), 1.0 / 3.0); let q = frame.local(c); format!("({:.0},{:.0},{:.0})", q[0], q[1], q[2]) }).collect();
                    blame = format!("{blame_zone} triangles {:?} at {} blamed on: {}", nose, at.join(" "), b.iter().take(6).map(|x| format!("{} saves {}", x.1, x.0)).collect::<Vec<_>>().join(", "));
                }
                (nm.clone(), z, blame)
            });
            let mut tot: Vec<(&'static str, usize, usize)> = Vec::new();
            let (mut heads, mut all) = (0, 0);
            for (nm, z, blame) in &rows {
                let n: usize = z.iter().filter(|x| x.0 != "kit count").map(|x| x.1).sum();
                if n > 0 { heads += 1; all += n; }
                for (zn, c) in z {
                    match tot.iter_mut().find(|x| x.0 == *zn) { Some(x) => { x.1 += c; x.2 += 1; } None => tot.push((zn, *c, 1)) }
                }
                if n > 0 { eprintln!("{nm:24} {n:4} {}", z.iter().map(|x| format!("{} {}", x.0, x.1)).collect::<Vec<_>>().join(", ")); }
                if !blame.is_empty() { eprintln!("    {blame}"); }
            }
            eprintln!("ZONES heads with a fault {heads} of {}, triangles {all}; {}", rows.len(), tot.iter().map(|x| format!("{}: {} triangles on {} heads", x.0, x.1, x.2)).collect::<Vec<_>>().join("; "));
        }
        "moveat" => {
            // `eye_audit moveat <data> <vertices...>`: how far each eye slider at its low end moves each vertex on the base head (mm),
            // built (smoothed and pressed) and unsmoothed
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            let vs: Vec<usize> = args[3..].iter().map(|a| a.parse().unwrap()).collect();
            let base = world.mesh.base.clone();
            let zero = vec![0.0; full_set.sliders.len()];
            let b0 = full_set.build(&world.smoother, &base, &zero);
            let u0 = full_set.unsmoothed(&base, &zero);
            for (j, sl) in full_set.sliders.iter().enumerate() {
                let mut v = zero.clone();
                v[j] = sl.low;
                let b = full_set.build(&world.smoother, &base, &v);
                let u = full_set.unsmoothed(&base, &v);
                let row: Vec<String> = vs.iter().map(|&i| format!("{:.2}/{:.2}", length(sub(b[i], b0[i])) * 1000.0, length(sub(u[i], u0[i])) * 1000.0)).collect();
                eprintln!("{:28} low {:6.2}: {}", sl.name, sl.low, row.join(" "));
            }
        }
        "props" => {
            // `eye_audit props <data> <heads>` ("base" at 0): the eye measure's proportions of each head's preset (EA_SET on top)
            let page = read_json(&data.join("web/eyes/page.json"));
            let full_set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join("web/eyes/sliders.json")).unwrap()).unwrap();
            for nm in &args[3..] {
                let (start, frame, values) = if nm == "base" {
                    (world.mesh.base.clone(), head_sliders::measure::head_frame(world.mesh.eyes), vec![0.0; full_set.sliders.len()])
                } else {
                    let h = world.heads.iter().find(|h| &h.name == nm).expect("head");
                    (h.start.clone(), h.frame, tools::solved_values(&page, &full_set, &h.name))
                };
                let values = with_ea_set(&full_set, values);
                let built = full_set.build(&world.smoother, &start, &values);
                let m = head_sliders::eyes::measure(&head_sliders::eyes::shown_surface(&built, &world.mesh.triangles, &frame, head_sliders::eyes::cover_of(&full_set, &values)));
                match m {
                    Ok(m) => eprintln!("{nm}: {}", EYES.proportions().iter().zip(&m.proportions).map(|(p, v)| format!("{} {:.2}", p.key, v)).collect::<Vec<_>>().join(", ")),
                    Err(e) => eprintln!("{nm}: {e}"),
                }
            }
        }
        "evalcheck" => {
            // the solver's measure (Evaluator, pressed in part) against the measure of the whole built and pressed head
            let smoothed_set = head_sliders::solve::Smoothed::new(&set, &world.smoother);
            for (i, h) in world.heads.iter().enumerate().take(4) {
                let ev = head_sliders::solve::Evaluator::with(&EYES, &set, &smoothed_set, &world.smoother, &h.start, &world.mesh.triangles, h.frame);
                let values: Vec<f64> = set.sliders.iter().enumerate().map(|(j, s)| s.low + (s.high - s.low) * (((i * 7 + j * 3) % 10) as f64 / 10.0)).collect();
                let a = ev.measure(&values).unwrap();
                let b = measure_lowered(h, &set.build(&world.smoother, &h.start, &values), world, head_sliders::eyes::cover_of(&set, &values)).unwrap();
                let d = a.proportions.iter().zip(&b.proportions).map(|(x, y)| (x - y).abs()).filter(|x| x.is_finite()).fold(0.0, f64::max);
                eprintln!("{}: largest difference {d:.2e}; front view {}", h.name, ["width", "height", "tilt", "upper_rise", "lower_drop", "inner_reach", "outer_reach", "inner_angle", "outer_angle", "spacing", "level"].iter().map(|k| format!("{k} {:.2}", a.proportions[proportion_index(&EYES, k)])).collect::<Vec<_>>().join(" "));
            }
        }
        other => panic!("unknown command {other}"),
    }
}
