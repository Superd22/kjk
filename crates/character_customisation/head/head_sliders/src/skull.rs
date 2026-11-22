//! The skull refitted under KOTOR's hair, on the heads with modelled hair (roadmap B2 for the skull, and A2).
//!
//! Our head's skull was fitted to KOTOR's outer surface, hair included. Here the skull alone is moved toward each head's bald reference
//! (`bald fill`), along its normals, as one smooth field. The face zones, the hidden insides, the neck's rim and the neck under the skull
//! are held still. This is a prototype: it writes its own folders, and nothing the viewer uses by default reads it.

use crate::V3;
use crate::bald::tools::{Reference, kotor};
use crate::bald::{Shape, merged_normals};
use crate::mesh::HeadMesh;
use crate::mesh::input::Input;
use std::path::{Path, PathBuf};

/// How hard the filled skull pulls against KOTOR's real skin, which pulls with 1: the fill is uncertain by 3 to 6 mm. `SKULL_FILLED`
/// overrides it.
pub fn filled_pull() -> f64 {
    std::env::var("SKULL_FILLED").ok().and_then(|v| v.parse().ok()).unwrap_or(0.6)
}

/// How much the skull resists bending, against the pull of one vertex. `SKULL_STIFFNESS` overrides it.
pub fn stiffness() -> f64 {
    std::env::var("SKULL_STIFFNESS").ok().and_then(|v| v.parse().ok()).unwrap_or(60.0)
}
/// Below this height on the base head (mm under the eyes) nothing pulls: the nape and the neck are not the skull.
const PULL_Y: f64 = -35.0;
/// Below this height on the base head nothing moves.
const HOLD_Y: f64 = -75.0;
/// Above this height on the base head (mm above the eyes) the ear area is skull, above the ear flap.
const EAR_TOP_Y: f64 = 32.0;
/// Over this distance (mm) from the held parts the pull fades in. `SKULL_FADE` overrides it.
fn fade_mm() -> f64 {
    std::env::var("SKULL_FADE").ok().and_then(|v| v.parse().ok()).unwrap_or(45.0)
}
/// In front of this depth on the base head (mm behind the eyes), below the brows, nothing moves: the lids' linings lie there.
const FRONT_Z: f64 = -25.0;

/// Which merged vertices are held still and which are pulled toward the reference, on one head.
pub struct Parts {
    pub held: Vec<bool>,
    pub pulled: Vec<bool>,
    pub ear: Vec<bool>,
    /// Per merged vertex, 0 next to a held vertex rising to 1 at `fade_mm()` from the nearest, so that the skull bends into the held
    /// parts without a step.
    pub fade: Vec<f64>,
}

impl Parts {
    /// A face zone's vertex is held where it lies inside the face (between the brows, the eyes' outer corners and the chin) or where
    /// the reference is KOTOR's real skin. The rest of a zone, such as the forehead zone's top and the temples under the hair, may move.
    pub fn new(input: &Input, mesh: &HeadMesh, real: &[bool]) -> Self {
        let frame = crate::measure::Frame::head();
        let base: Vec<V3> = mesh.first.iter().map(|&i| frame.local(mesh.base[i as usize])).collect();
        let region = crate::bald::tools::region(input, mesh);
        let ear = crate::bald::tools::ear_area(input, mesh);
        let n = mesh.merged_count();
        let held: Vec<bool> = (0..n)
            .map(|g| {
                let i = mesh.first[g] as usize;
                let p = base[g];
                let face = p[0].abs() < 48.0 && p[1] < 32.0 && p[2] > -15.0;
                (mesh.zone[i] != 0 && (face || real[g]))
                    || mesh.hidden[g]
                    || (mesh.keep[i] && !ear[g])
                    || !region[g]
                    || p[1] < HOLD_Y
                    || (p[2] > FRONT_Z && p[1] < 40.0)
            })
            .collect();
        // the ear area reaches 50 mm above the eyes; above the ear flap it is skull like the rest
        let ear: Vec<bool> = (0..n).map(|g| ear[g] && base[g][1] < EAR_TOP_Y).collect();
        // `SKULL_ZONE_FREE` leaves a face zone's freed part (under the hair) unpulled, so it only bends between the held face and the
        // skull: a little smoother at the temples, but the forehead slope misses more
        let zone_free = std::env::var("SKULL_ZONE_FREE").is_ok();
        let pulled: Vec<bool> = (0..n).map(|g| !held[g] && !ear[g] && base[g][1] > PULL_Y && !(zone_free && mesh.zone[mesh.first[g] as usize] != 0)).collect();
        let fence: Vec<V3> = (0..n).filter(|&g| held[g] && region[g] && !mesh.hidden[g]).map(|g| base[g]).collect();
        let fade = fade_mm();
        let fade = (0..n)
            .map(|g| {
                if !pulled[g] {
                    return 0.0;
                }
                let d = fence.iter().map(|q| crate::length(crate::sub(base[g], *q))).fold(f64::INFINITY, f64::min);
                let t = (d / fade).min(1.0);
                t * t * (3.0 - 2.0 * t)
            })
            .collect();
        Self { held, pulled, ear, fade }
    }
}

/// Our head at `local` (every vertex, head frame, mm) with its skull moved toward the reference, as every vertex's new place.
/// `outer` is KOTOR's whole head with its hair: a skull vertex left outside it is pushed 1 mm inside, the push spread smoothly.
pub fn refit(input: &Input, mesh: &HeadMesh, local: &[V3], r: &Reference, outer: &crate::bald::Soup) -> Vec<V3> {
    let parts = Parts::new(input, mesh, &r.real);
    let n = mesh.merged_count();
    let start = mesh.merge(local);
    let mut x = start.clone();
    let w: Vec<f64> = (0..n).map(|g| parts.fade[g] * if !parts.pulled[g] { 0.0 } else if r.real[g] { 1.0 } else { filled_pull() }).collect();
    for _ in 0..4 {
        let normals = merged_normals(mesh, &x);
        let target: Vec<f64> = (0..n).map(|g| if w[g] > 0.0 { crate::dot(normals[g], crate::sub(r.merged[g], x[g])) } else { 0.0 }).collect();
        let s = solve(mesh, &parts.held, &w, &target);
        for g in 0..n {
            x[g] = crate::add(x[g], crate::scale(normals[g], s[g]));
        }
    }
    for _ in 0..3 {
        let normals = merged_normals(mesh, &x);
        let mut push: Vec<V3> = vec![[0.0; 3]; n];
        let mut fixed = parts.held.clone();
        for g in 0..n {
            if !parts.pulled[g] || r.real[g] || outer.first(x[g], normals[g], 120.0).is_some() {
                continue;
            }
            if let Some(t) = outer.first(x[g], crate::scale(normals[g], -1.0), 30.0) {
                push[g] = crate::scale(normals[g], -(t + 1.0));
                fixed[g] = true;
            }
        }
        crate::bald::harmonic(mesh, &mut push, &fixed, 60);
        for g in 0..n {
            x[g] = crate::add(x[g], push[g]);
        }
    }
    let mut moved: Vec<V3> = (0..n).map(|g| crate::sub(x[g], start[g])).collect();
    // the ear is a thin flap whose normals turn every way: it takes the move of the skin around it, spread smoothly over it
    let fixed: Vec<bool> = parts.ear.iter().map(|e| !e).collect();
    crate::bald::harmonic(mesh, &mut moved, &fixed, 300);
    local.iter().zip(&mesh.group).map(|(&p, &g)| crate::add(p, moved[g as usize])).collect()
}

/// The amounts `s` (one per merged vertex, 0 where held) that minimise the pull `w (s - t)²` plus the bending
/// `stiffness() |Δs|²`, Δ being each vertex against the mean of its neighbours, by conjugate gradients.
fn solve(mesh: &HeadMesh, held: &[bool], w: &[f64], t: &[f64]) -> Vec<f64> {
    let n = w.len();
    let lap = |s: &[f64]| -> Vec<f64> {
        (0..n)
            .map(|g| {
                let nb = mesh.neighbours_of(g);
                if nb.is_empty() { 0.0 } else { s[g] - nb.iter().map(|&j| s[j as usize]).sum::<f64>() / nb.len() as f64 }
            })
            .collect()
    };
    let lap_t = |y: &[f64]| -> Vec<f64> {
        let mut out = y.to_vec();
        for g in 0..n {
            let nb = mesh.neighbours_of(g);
            for &j in nb {
                out[j as usize] -= y[g] / nb.len() as f64;
            }
        }
        out
    };
    let k = stiffness();
    let apply = |s: &[f64]| -> Vec<f64> {
        let b = lap_t(&lap(s));
        (0..n).map(|g| if held[g] { 0.0 } else { w[g] * s[g] + k * b[g] }).collect()
    };
    let rhs: Vec<f64> = (0..n).map(|g| if held[g] { 0.0 } else { w[g] * t[g] }).collect();
    let mut s = vec![0.0; n];
    let mut r = rhs.clone();
    let mut p = r.clone();
    let dot = |a: &[f64], b: &[f64]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    let mut rr = dot(&r, &r);
    for _ in 0..3000 {
        if rr < 1e-12 {
            break;
        }
        let ap = apply(&p);
        let a = rr / dot(&p, &ap);
        for g in 0..n {
            s[g] += a * p[g];
            r[g] -= a * ap[g];
        }
        let next = dot(&r, &r);
        for g in 0..n {
            p[g] = r[g] + next / rr * p[g];
        }
        rr = next;
    }
    s
}

/// The skull's width above the ears: the widest level cut 45 to 90 mm above the eyes, in mm. `bald::ROWS`'s width starts at 30 mm,
/// where it can read the top of our ear area (up to 43 mm above the eyes).
pub fn skull_width(shape: &Shape) -> f64 {
    (45..=90)
        .filter_map(|y| {
            let segs = crate::bald::cut(shape, 1, y as f64);
            let xs = segs.iter().flat_map(|(s, _)| s.iter().map(|p| p[0]));
            let (lo, hi) = xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| (a.min(x), b.max(x)));
            (hi > lo).then_some(hi - lo)
        })
        .fold(f64::NAN, f64::max)
}

/// The skull rows of the whole-head measure that the guard reads, by their key in `bald::ROWS`.
pub const SKULL_ROWS: [&str; 10] =
    ["top", "back", "back_height", "length", "width", "length_ratio", "width_ratio", "top_ratio", "forehead_slope", "crown_front"];

fn b64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let v = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            out.push(if k <= c.len() { A[(v >> (18 - 6 * k) & 63) as usize] as char } else { '=' });
        }
    }
    out
}

/// The nearest distance from each point to a set of points, by brute force (mm).
fn nearest(points: &[V3], to: &[V3]) -> Vec<f64> {
    points.iter().map(|p| to.iter().map(|q| crate::length(crate::sub(*p, *q))).fold(f64::INFINITY, f64::min)).collect()
}

/// The open edges' vertices of a triangle soup whose corners are given three by three (KOTOR's hair parts), merged by place.
fn open_edge_points(v: &[V3]) -> Vec<V3> {
    use std::collections::HashMap;
    let key = |p: V3| p.map(|c| (c * 100.0).round() as i64);
    let edge = |a: V3, b: V3| {
        let (ka, kb) = (key(a), key(b));
        if ka < kb { (ka, kb) } else { (kb, ka) }
    };
    let mut count: HashMap<([i64; 3], [i64; 3]), usize> = HashMap::new();
    for t in v.chunks_exact(3) {
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            *count.entry(edge(t[a], t[b])).or_default() += 1;
        }
    }
    let mut out: Vec<V3> = Vec::new();
    for t in v.chunks_exact(3) {
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            if count[&edge(t[a], t[b])] == 1 {
                out.push(t[a]);
            }
        }
    }
    out
}

pub mod tools {
    use super::*;
    use crate::bald::tools::{hair_faces, kotor_shape, load_reference, our_shape, reference_shape};
    use crate::bald::{ROWS, SKULL_SHOWS, Soup, measure_head};
    use crate::heads::{parallel, threads};

    pub fn run(command: &str, args: &[String], data: &Path, input: &Input) -> bool {
        let hair = PathBuf::from(std::env::var("BALD_HAIR").expect("BALD_HAIR: the folder of the hair parts"));
        match command {
            // KOTOR's heads in their texture's colour, then with the hair cut in blue, from the side, three-quarter from behind and
            // from behind: `colour <data> <out.png> <head>...`
            "colour" => {
                use crate::render::{clay, label, sheet};
                let out = args[4].clone();
                let mut tiles = Vec::new();
                for name in &args[5..] {
                    let mut k = kotor(input, &hair, name);
                    if std::env::var("SKULL_FACE_ONLY").is_ok() {
                        k.f.truncate(k.face_f);
                    }
                    let hf = hair_faces(input, name, &k);
                    let lab = input.f32s(&format!("extra/heads/{name}/kotor_colour"));
                    let colour: Vec<Option<[f64; 3]>> =
                        (0..k.f.len()).map(|i| if i < k.face_f { Some(lab_rgb([lab[3 * i], lab[3 * i + 1], lab[3 * i + 2]])) } else { Some([230.0, 140.0, 90.0]) }).collect();
                    let cut: Vec<Option<[f64; 3]>> = hf.iter().map(|&h| h.then_some([90.0, 140.0, 230.0])).collect();
                    for (row, tint) in [colour, cut].iter().enumerate() {
                        for (s, shot) in [crate::bald::SHOTS[0], crate::bald::SHOTS[2], crate::bald::SHOTS[3]].iter().enumerate() {
                            let mut img = clay(&k.v, &k.f, shot, &[], tint);
                            if s == 0 {
                                label(&mut img, &format!("{name} {}", if row == 0 { "colour" } else { "cut" }));
                            }
                            tiles.push(img);
                        }
                    }
                }
                sheet(&tiles, 6).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // The old head (yellow) and the new head (green) outlined over KOTOR's reference (real skin in clay, filled in orange), from
            // the side and the front, with KOTOR's hair off and on: `overlay <data> <ref dir> <run dir> <out prefix> <head>...`
            "overlay" => {
                use crate::bald::tools::{draw_outline, reference_mesh};
                use crate::render::{clay, label, sheet};
                let refs = PathBuf::from(&args[4]);
                let run = PathBuf::from(&args[5]);
                let prefix = &args[6];
                let mesh = input.head_mesh();
                let region = crate::bald::tools::region(input, &mesh);
                let frame = crate::measure::Frame::head();
                for name in &args[7..] {
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    let r = load_reference(&refs, name, &mesh, &region);
                    let read = |variant: &str| -> Vec<V3> {
                        let raw = std::fs::read(run.join("bins").join(variant).join(format!("{name}.bin"))).unwrap();
                        raw.chunks_exact(12).map(|c| frame.local(std::array::from_fn(|k| f32::from_le_bytes(c[4 * k..4 * k + 4].try_into().unwrap()) as f64))).collect()
                    };
                    let (old, new) = (our_shape(&mesh, &read("live")), our_shape(&mesh, &read("refit")));
                    let mut tiles = Vec::new();
                    for shot in [crate::bald::SHOTS[0], crate::bald::SHOTS[1]] {
                        for with_hair in [false, true] {
                            let (v, f, tint) = reference_mesh(&k, &hf, &r, &mesh, with_hair);
                            let mut img = clay(&v, &f, &shot, &[], &tint);
                            draw_outline(&mut img, &shot, &old, [255, 240, 60]);
                            draw_outline(&mut img, &shot, &new, [60, 230, 90]);
                            label(&mut img, &format!("{name} {} {}", shot.name, if with_hair { "hair on" } else { "hair off" }));
                            tiles.push(img);
                        }
                    }
                    let out = format!("{prefix}_{name}.png");
                    sheet(&tiles, 4).save(&out).unwrap();
                    eprintln!("wrote {out}");
                }
                true
            }
            // The old head, then the new head with its faults against the old one in red, from the side, the front, three-quarter
            // and behind: `show <data> <run dir> <out.png> <head>...`
            "show" => {
                use crate::render::{clay, label, sheet};
                let run = PathBuf::from(&args[4]);
                let mesh = input.head_mesh();
                let frame = crate::measure::Frame::head();
                let all: Vec<u32> = (0..mesh.base.len() as u32).collect();
                let read = |variant: &str, name: &str| -> Vec<V3> {
                    let raw = std::fs::read(run.join("bins").join(variant).join(format!("{name}.bin"))).unwrap();
                    raw.chunks_exact(12).map(|c| std::array::from_fn(|k| f32::from_le_bytes(c[4 * k..4 * k + 4].try_into().unwrap()) as f64)).collect()
                };
                let shown: Vec<[u32; 3]> = mesh.triangles.iter().filter(|t| t.iter().all(|&i| !mesh.hidden[mesh.group[i as usize] as usize])).copied().collect();
                let mut tiles = Vec::new();
                for name in &args[6..] {
                    let (old, new) = (read("live", name), read("refit", name));
                    let marks = crate::check::Checker::from_parts(&shown, &vec![false; old.len()], &old, &all).marks(&new);
                    for (row, p) in [(0, &old), (1, &new)] {
                        let local: Vec<V3> = p.iter().map(|&q| frame.local(q)).collect();
                        let red = if row == 1 { marks.clone() } else { vec![false; shown.len()] };
                        for (s, shot) in crate::bald::SHOTS.iter().enumerate() {
                            let mut img = clay(&local, &shown, shot, &red, &[]);
                            if s == 0 {
                                label(&mut img, &format!("{name} {}", if row == 0 { "old" } else { "new" }));
                            }
                            tiles.push(img);
                        }
                    }
                }
                sheet(&tiles, 4).save(&args[5]).unwrap();
                eprintln!("wrote {}", args[5]);
                true
            }
            // The held parts (blue) and the pulled parts (orange) on the base head, in the four shots: `parts <data> <out.png>`
            "widest" => {
                debug_widest(input, data, Path::new(&args[4]), Path::new(&args[5]), &args[6]);
                true
            }
            "parts" => {
                use crate::render::{clay, sheet};
                let mesh = input.head_mesh();
                let parts = Parts::new(input, &mesh, &vec![false; mesh.merged_count()]);
                let frame = crate::measure::Frame::head();
                let base: Vec<V3> = mesh.base.iter().map(|&p| frame.local(p)).collect();
                let tint: Vec<Option<[f64; 3]>> = mesh
                    .triangles
                    .iter()
                    .map(|t| {
                        let g = t.map(|i| mesh.group[i as usize] as usize);
                        if g.iter().all(|&j| parts.held[j]) {
                            Some([90.0, 140.0, 230.0])
                        } else if g.iter().all(|&j| parts.pulled[j]) {
                            Some([235.0, 150.0, 80.0])
                        } else {
                            None
                        }
                    })
                    .collect();
                let tiles: Vec<_> = crate::bald::SHOTS.iter().map(|s| clay(&base, &mesh.triangles, s, &[], &tint)).collect();
                sheet(&tiles, 4).save(&args[4]).unwrap();
                true
            }
            // The refit of every head with a reference in <ref dir>, its guard's readings, and viewer 2's files: `refit <data> <ref dir>
            // <out dir> <viewer_map.bin>`. The out dir gets `measure.json` and `viewer/{refit,live}/<head>.json` for all 57 heads; a
            // head without a reference gets its live head in both.
            "refit" => {
                let refs = PathBuf::from(&args[4]);
                let out = PathBuf::from(&args[5]);
                let vmap: Vec<u32> = std::fs::read(&args[6]).unwrap().chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
                for v in ["refit", "live"] {
                    std::fs::create_dir_all(out.join("viewer").join(v)).unwrap();
                }
                let heads = crate::jaw_edge::tools::Heads::load(input, data);
                let mesh = &heads.mesh;
                let region = crate::bald::tools::region(input, mesh);
                let frame = crate::measure::Frame::head();
                let names = crate::bald::tools::names(input);
                let all: Vec<u32> = (0..mesh.base.len() as u32).collect();
                let underjaw = |p: &[V3]| {
                    crate::under_jaw::measure(&crate::measure::Surface::within(p, &mesh.triangles, &frame, &crate::under_jaw::WINDOW))
                        .map(|m| m.proportions.to_vec())
                        .unwrap_or_default()
                };
                let opt = |x: &f64| if x.is_nan() { serde_json::Value::Null } else { serde_json::json!(x) };
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |i| {
                    let name = &names[i];
                    let (_, live) = heads.pair(input, name);
                    let live_local: Vec<V3> = live.iter().map(|&q| frame.local(q)).collect();
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    let has_ref = refs.join(format!("{name}_ref.bin")).exists();
                    let r = has_ref.then(|| load_reference(&refs, name, mesh, &region));
                    let soup = Soup::new(k.f.iter().map(|t| t.map(|j| k.v[j as usize])).collect());
                    let new_local = match &r {
                        Some(r) => refit(input, mesh, &live_local, r, &soup),
                        None => live_local.clone(),
                    };
                    let new: Vec<V3> = if has_ref { new_local.iter().map(|&q| frame.world(q)).collect() } else { live.clone() };
                    let write = |variant: &str, p: &[V3]| {
                        let bytes: Vec<u8> = vmap.iter().flat_map(|&v| p[v as usize]).flat_map(|c| (c as f32).to_le_bytes()).collect();
                        std::fs::write(out.join("viewer").join(variant).join(format!("{name}.json")), serde_json::json!({ "pos": b64(&bytes) }).to_string()).unwrap();
                        std::fs::create_dir_all(out.join("bins").join(variant)).unwrap();
                        let kit: Vec<u8> = p.iter().flat_map(|q| q.map(|c| c as f32)).flat_map(|c| c.to_le_bytes()).collect();
                        std::fs::write(out.join("bins").join(variant).join(format!("{name}.bin")), kit).unwrap();
                    };
                    write("refit", &new);
                    write("live", &live);
                    let mv = |m: Vec<(f64, bool)>| serde_json::json!(m.iter().map(|(v, _)| opt(v)).collect::<Vec<_>>());
                    let (ks, ls, ns) = (kotor_shape(&k, &hf), our_shape(mesh, &live_local), our_shape(mesh, &new_local));
                    let zones: Vec<f64> = (1..=crate::mesh::ZONES.len() as u8)
                        .map(|z| (0..live_local.len()).filter(|&v| mesh.zone[v] == z).map(|v| crate::length(crate::sub(live_local[v], new_local[v]))).fold(0.0, f64::max))
                        .collect();
                    let checker = crate::check::Checker::new(mesh, &live, &all);
                    let f = checker.faults(&new);
                    let ears = crate::bald::tools::ear_area(input, mesh);
                    let at_ear = checker.marks(&new).iter().zip(&mesh.triangles).filter(|(m, t)| **m && t.iter().any(|&v| ears[mesh.group[v as usize] as usize])).count();
                    let mut row = serde_json::json!({
                        "name": name,
                        "skull_shows": SKULL_SHOWS.contains(&name.as_str()),
                        "haired": has_ref,
                        "kotor": mv(measure_head(&ks)),
                        "live": mv(measure_head(&ls)),
                        "refit": mv(measure_head(&ns)),
                        "skull_width": {"kotor": opt(&skull_width(&ks)), "live": opt(&skull_width(&ls)), "refit": opt(&skull_width(&ns))},
                        "moved_max": live.iter().zip(&new).map(|(a, b)| 1000.0 * crate::length(crate::sub(*a, *b))).fold(0.0, f64::max),
                        "zone_moved_max": zones,
                        "underjaw_live": underjaw(&live).iter().map(opt).collect::<Vec<_>>(),
                        "underjaw_refit": underjaw(&new).iter().map(opt).collect::<Vec<_>>(),
                        "faults": {"flipped": f.flipped, "folded": f.folded, "marked_at_ear": at_ear, "marked": checker.marks(&new).iter().filter(|m| **m).count()},
                    });
                    if let Some(r) = &r {
                        // each face zone's root-mean-square gap to the reference along our normals, before and after, and its
                        // largest move where it is held (inside the face or on KOTOR's real skin)
                        let zone_gap = |p: &[V3]| -> Vec<f64> {
                            let merged = mesh.merge(p);
                            let normals = merged_normals(mesh, &merged);
                            (1..=crate::mesh::ZONES.len() as u8)
                                .map(|z| {
                                    let d: Vec<f64> = (0..merged.len())
                                        .filter(|&g| mesh.zone[mesh.first[g] as usize] == z && r.region[g] && !mesh.hidden[g])
                                        .map(|g| crate::dot(normals[g], crate::sub(r.merged[g], merged[g])))
                                        .collect();
                                    (d.iter().map(|x| x * x).sum::<f64>() / d.len().max(1) as f64).sqrt()
                                })
                                .collect()
                        };
                        let held = Parts::new(input, mesh, &r.real).held;
                        let held_moved = (0..live_local.len())
                            .filter(|&v| mesh.zone[v] != 0 && held[mesh.group[v] as usize])
                            .map(|v| crate::length(crate::sub(live_local[v], new_local[v])))
                            .fold(0.0, f64::max);
                        row["zone_gap"] = serde_json::json!({"live": zone_gap(&live_local), "refit": zone_gap(&new_local), "held_moved_max": held_moved});
                        let rs = reference_shape(&k, &hf, r, mesh);
                        row["reference"] = mv(measure_head(&rs));
                        row["skull_width"]["reference"] = opt(&skull_width(&rs));
                        let with_hair = Shape { tris: k.f.iter().map(|t| t.map(|j| k.v[j as usize])).collect(), filled: vec![false; k.f.len()] };
                        row["kotor_with_hair"] = mv(measure_head(&with_hair));
                        // the skull against KOTOR's head with its hair: a pulled vertex under the hair whose outward ray meets nothing
                        // of KOTOR, while its inward ray meets KOTOR within 30 mm, stands outside KOTOR's hair by that much
                        let parts = Parts::new(input, mesh, &r.real);
                        let outside = |p: &[V3]| {
                            let merged = mesh.merge(p);
                            let normals = merged_normals(mesh, &merged);
                            let mut out: Vec<f64> = Vec::new();
                            for g in 0..merged.len() {
                                if !parts.pulled[g] || r.real[g] {
                                    continue;
                                }
                                if soup.first(merged[g], normals[g], 120.0).is_none()
                                    && let Some(t) = soup.first(merged[g], crate::scale(normals[g], -1.0), 30.0)
                                {
                                    out.push(t);
                                }
                            }
                            out.sort_by(f64::total_cmp);
                            serde_json::json!({"count": out.len(), "worst": out.last()})
                        };
                        row["filled_pulled"] = serde_json::json!((0..mesh.merged_count()).filter(|&g| parts.pulled[g] && !r.real[g]).count());
                        row["outside_hair"] = outside(&new_local);
                        row["outside_hair_live"] = outside(&live_local);
                        // KOTOR's hair parts as viewer 2 shows them: how far their open edges lie from our head, before and after
                        let hp = crate::bald::hair_parts(&hair, name).0;
                        let edge = open_edge_points(&hp);
                        if !edge.is_empty() {
                            let gap = |p: &[V3]| {
                                let skin: Vec<V3> = mesh.merge(p).into_iter().enumerate().filter(|(g, _)| !mesh.hidden[*g]).map(|(_, q)| q).collect();
                                let mut d = nearest(&edge, &skin);
                                d.sort_by(f64::total_cmp);
                                let q = |f: f64| d[((d.len() - 1) as f64 * f) as usize];
                                serde_json::json!({"median": q(0.5), "p90": q(0.9), "max": q(1.0)})
                            };
                            row["hair_edge_gap"] = serde_json::json!({"live": gap(&live_local), "refit": gap(&new_local), "points": edge.len()});
                        }
                    }
                    row
                });
                let keys: Vec<&str> = ROWS.iter().map(|r| r.0).collect();
                let uk: Vec<&str> = crate::under_jaw::PROPORTIONS.iter().map(|p| p.key).collect();
                let all = serde_json::json!({"rows": keys, "underjaw": uk, "zones": crate::mesh::ZONES, "skull_rows": SKULL_ROWS, "heads": rows});
                std::fs::write(out.join("measure.json"), serde_json::to_string(&all).unwrap()).unwrap();
                eprintln!("wrote {}", out.display());
                true
            }
            _ => false,
        }
    }

    /// CIE Lab (D65) to sRGB, 0 to 255.
    fn lab_rgb(c: [f64; 3]) -> [f64; 3] {
        let fy = (c[0] + 16.0) / 116.0;
        let (fx, fz) = (fy + c[1] / 500.0, fy - c[2] / 200.0);
        let inv = |f: f64| if f > 6.0 / 29.0 { f * f * f } else { 3.0 * (6.0f64 / 29.0).powi(2) * (f - 4.0 / 29.0) };
        let (x, y, z) = (0.95047 * inv(fx), inv(fy), 1.08883 * inv(fz));
        let lin = [3.2406 * x - 1.5372 * y - 0.4986 * z, -0.9689 * x + 1.8758 * y + 0.0415 * z, 0.0557 * x - 0.2040 * y + 1.0570 * z];
        lin.map(|v| {
            let v = v.clamp(0.0, 1.0);
            255.0 * if v <= 0.0031308 { 12.92 * v } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
        })
    }
}


pub fn debug_widest(input: &Input, data: &Path, refs: &Path, run: &Path, name: &str) {
    let mesh = input.head_mesh();
    let frame = crate::measure::Frame::head();
    let region = crate::bald::tools::region(input, &mesh);
    let r = crate::bald::tools::load_reference(refs, name, &mesh, &region);
    let parts = Parts::new(input, &mesh, &r.real);
    let _ = data;
    let raw = std::fs::read(run.join("bins/refit").join(format!("{name}.bin"))).unwrap();
    let p: Vec<V3> = raw.chunks_exact(12).map(|c| frame.local(std::array::from_fn(|k| f32::from_le_bytes(c[4 * k..4 * k + 4].try_into().unwrap()) as f64))).collect();
    let m = mesh.merge(&p);
    let normals = merged_normals(&mesh, &m);
    let mut idx: Vec<usize> = (0..m.len()).filter(|&g| m[g][1] > 45.0 && m[g][1] < 90.0 && !mesh.hidden[g]).collect();
    idx.sort_by(|&a, &b| m[b][0].abs().total_cmp(&m[a][0].abs()));
    for &g in idx.iter().take(12) {
        let i = mesh.first[g] as usize;
        println!("x {:6.1} y {:5.1} z {:6.1} zone {} held {} pulled {} real {} fade {:.2} gap {:5.1} ref x {:6.1}", m[g][0], m[g][1], m[g][2], mesh.zone[i], parts.held[g], parts.pulled[g], r.real[g], parts.fade[g], crate::dot(normals[g], crate::sub(r.merged[g], m[g])), r.merged[g][0]);
    }
}
