//! The audit of the nose, the jaw, the brow and the cheeks (2026-11-20), the eye audit's measures for any feature. Audit only.
//! Usage: `feature_audit <feature> <command> <data> [args]`.
//!
//! - `presets`: the presets solved (`bench::presets`), then per head the faults of the built preset in the feature's window, written to
//!   `<f>/audit_faults.json`.
//! - `faults`: the faults alone, from the data folder's page.
//! - `starts`: per head, how far the start shape is from the base head over the feature's core (zone weight 1), with its place taken
//!   out three ways (a translation, a rigid move, a rigid move with a size), and how far the eye paste moves the stand-in's measures.
//!   Written to `<f>/starts_audit.json`.
//! - `sweep <heads>`: each slider in 13 steps from its low end to its high end on each head's preset, every measure read each time,
//!   written to `<f>/sweep.json`.
//! - `pictures <heads>`: close-ups in three-quarter view, in clay and in a light like viewer 2's, of KOTOR and of the versions named by
//!   `FA_VERSIONS` ("label|data|starts;;..." with starts "world", "nopaste" or "paste"). `FA_SHOT` is "close" or "seam", `FA_OUT` the
//!   picture.
//!
//! `FA_PASTE=1` replaces each start's feature area with the base head's, moved rigidly (`paste`), before any command.

use head_sliders::bench;
use head_sliders::check::Checker;
use head_sliders::checks::Marks;
use head_sliders::eyes::tools::{self, Scene};
use head_sliders::feature::{Feature, Shot, by_key, proportion_index};
use head_sliders::heads::{Head, World, parallel, threads};
use head_sliders::measure::Surface;
use head_sliders::mesh::input::Input;
use head_sliders::set::{SMOOTHING, SliderSet};
use head_sliders::solve::{Evaluator, Smoothed};
use head_sliders::{V3, add, length, scale, sub};
use image::{Rgb, RgbImage};
use nalgebra::{Matrix3, Vector3};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The zone's weight on every vertex.
fn weights(world: &World) -> Vec<f64> {
    (0..world.mesh.base.len()).map(|v| world.zone.weight[world.mesh.group[v] as usize]).collect()
}

/// The best move of `from` onto `to` over the vertices `idx`: a rotation (when `rotate`), a size (when `size`) and a translation.
fn fit(from: &[V3], to: &[V3], idx: &[usize], rotate: bool, size: bool) -> impl Fn(V3) -> V3 {
    let n = idx.len() as f64;
    let mean = |p: &[V3]| -> Vector3<f64> { idx.iter().fold(Vector3::zeros(), |a, &i| a + Vector3::from(p[i])) / n };
    let (ca, cb) = (mean(from), mean(to));
    let mut r = Matrix3::identity();
    let mut s = 1.0;
    if rotate {
        let mut h = Matrix3::zeros();
        for &i in idx {
            h += (Vector3::from(from[i]) - ca) * (Vector3::from(to[i]) - cb).transpose();
        }
        let svd = h.svd(true, true);
        let (u, vt) = (svd.u.unwrap(), svd.v_t.unwrap());
        let d = (vt.transpose() * u.transpose()).determinant().signum();
        let fix = Matrix3::from_diagonal(&Vector3::new(1.0, 1.0, d));
        r = vt.transpose() * fix * u.transpose();
        if size {
            let num: f64 = idx.iter().map(|&i| (r * (Vector3::from(from[i]) - ca)).dot(&(Vector3::from(to[i]) - cb))).sum();
            let den: f64 = idx.iter().map(|&i| (Vector3::from(from[i]) - ca).norm_squared()).sum();
            s = num / den;
        }
    }
    move |p: V3| {
        let q = s * (r * (Vector3::from(p) - ca)) + cb;
        [q[0], q[1], q[2]]
    }
}

/// The start with its feature area replaced by the base head's moved rigidly onto it over the core (zone weight above 0.999), faded
/// into the start by the zone's weight.
fn paste(world: &World, w: &[f64], start: &[V3]) -> Vec<V3> {
    let core: Vec<usize> = (0..w.len()).filter(|&v| w[v] > 0.999).collect();
    let moved = fit(&world.mesh.base, start, &core, true, false);
    (0..start.len()).map(|v| add(start[v], scale(sub(moved(world.mesh.base[v]), start[v]), w[v]))).collect()
}

fn smoothed(world: &World, p: &[V3]) -> Vec<V3> {
    world.smoother.smooth(p, SMOOTHING)
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
}

fn read_set(f: &dyn Feature, data: &Path) -> SliderSet {
    let path = std::env::var("SLIDERS_SET").map(PathBuf::from).unwrap_or_else(|_| data.join(f.files().sliders));
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
}

/// Faults of a built head against its smoothed start, over the triangles near the feature's window.
fn faults(world: &World, h: &Head, start: &[V3], built: &[V3]) -> (usize, usize) {
    let before = smoothed(world, start);
    let window = world.feature.window();
    let near: Vec<u32> = (0..before.len() as u32).filter(|&v| window.contains(h.frame.local(before[v as usize]))).collect();
    let f = Checker::new(&world.mesh, &before, &near).faults(built);
    (f.flipped, f.folded)
}

fn fault_rows(world: &World, set: &SliderSet, page: &Value) -> Vec<Value> {
    parallel(world.heads.len(), threads(), |i| {
        let h = &world.heads[i];
        let values = tools::solved_values(page, set, &h.name);
        let built = set.build(&world.smoother, &h.start, &values);
        let (flipped, folded) = faults(world, h, &h.start, &built);
        json!({"name": h.name, "values": values, "flipped": flipped, "folded": folded})
    })
}

/// Turns a place for a shot, as the clay pictures do.
fn turn(shot: &Shot, p: V3) -> V3 {
    let (sy, cy) = shot.yaw.to_radians().sin_cos();
    let (x, z) = (p[0] * cy + p[2] * sy, -p[0] * sy + p[2] * cy);
    let (sp, cp) = shot.pitch.to_radians().sin_cos();
    let y = p[1] - shot.pivot;
    [x, cp * y + sp * z + shot.pivot, -sp * y + cp * z]
}

/// A scene in a light like viewer 2's "In game" light: a sky light from above and one key light from the upper left.
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

/// The close-up of each feature in three-quarter view, and the brow and cheek beside the eye.
fn close_shot(key: &str, which: &str) -> Shot {
    if which == "seam" {
        return Shot { name: "seam", yaw: -35.0, pitch: 0.0, pivot: 0.0, window: [-5.0, 70.0, -50.0, 45.0] };
    }
    let window = match key {
        "nose" => [-35.0, 35.0, -72.0, 12.0],
        "jaw" => [-80.0, 80.0, -150.0, -30.0],
        "brow" => [-75.0, 75.0, -25.0, 80.0],
        _ => [-80.0, 80.0, -90.0, 15.0],
    };
    Shot { name: "close", yaw: 35.0, pitch: 0.0, pivot: 0.0, window }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let feature = by_key(&args[1]).expect("feature");
    let command = args[2].as_str();
    let data = PathBuf::from(&args[3]);
    let input = Input::open(&data.join("input"));
    let t0 = std::time::Instant::now();
    let mut world = World::load(&input, feature);
    eprintln!("{} heads in {:.1?}", world.heads.len(), t0.elapsed());
    let w = weights(&world);
    let today: Vec<Vec<V3>> = world.heads.iter().map(|h| h.start.clone()).collect();
    let pasted: Vec<Vec<V3>> = parallel(world.heads.len(), threads(), |i| paste(&world, &w, &world.heads[i].start));
    if std::env::var("FA_PASTE").is_ok_and(|v| v == "1") {
        let window = feature.window();
        let news: Vec<_> = parallel(world.heads.len(), threads(), |i| {
            feature.measure(&Surface::within(&smoothed(&world, &pasted[i]), &world.mesh.triangles, &world.heads[i].frame, &window)).ok()
        });
        for ((h, p), m) in world.heads.iter_mut().zip(&pasted).zip(news) {
            h.start = p.clone();
            h.start_nose = m;
        }
        eprintln!("pasted the base head's {} area into every start", feature.key());
    }
    let world = &world;
    let key = feature.key();
    let audit = |name: &str| data.join(format!("{key}/{name}"));
    std::fs::create_dir_all(data.join(key)).unwrap();
    match command {
        "presets" | "faults" => {
            if command == "presets" {
                bench::presets(world, &Marks::read(&data.join(feature.files().marks)), &data);
            }
            let set = read_set(feature, &data);
            let page = read_json(&data.join(feature.files().page));
            let rows = fault_rows(world, &set, &page);
            std::fs::write(audit("audit_faults.json"), serde_json::to_string(&rows).unwrap()).unwrap();
            let bad = rows.iter().filter(|r| r["flipped"].as_u64().unwrap() + r["folded"].as_u64().unwrap() > 0).count();
            eprintln!("faulty heads {bad} of {} in {:.1?}", rows.len(), t0.elapsed());
        }
        "judge" => {
            // another folder's presets built on this world's starts: `judge <data> <presets data>`
            let other = PathBuf::from(&args[4]);
            let set = read_set(feature, &other);
            let page = read_json(&other.join(feature.files().page));
            let marks = Marks::read(&data.join(feature.files().marks)).marks;
            let rows: Vec<(bool, bool, usize)> = parallel(world.heads.len(), threads(), |i| {
                let h = &world.heads[i];
                let values = tools::solved_values(&page, &set, &h.name);
                let built = set.build(&world.smoother, &h.start, &values);
                let m = feature.measure(&Surface::within(&built, &world.mesh.triangles, &h.frame, &feature.window())).ok();
                let made = m.is_some_and(|m| head_sliders::verdict::judge(feature, &m, &h.kotor, &marks)["made"].as_bool() == Some(true));
                let (a, b) = faults(world, h, &h.start, &built);
                (h.kotor_error.is_none(), made, a + b)
            });
            let scored = rows.iter().filter(|r| r.0).count();
            let made = rows.iter().filter(|r| r.0 && r.1).count();
            let bad = rows.iter().filter(|r| r.2 > 0).count();
            eprintln!("{key}: presets of {} on these starts: {made} of {scored} made, {bad} heads with a fault", other.display());
        }
        "starts" => {
            let base = &world.mesh.base;
            let core: Vec<usize> = (0..w.len()).filter(|&v| w[v] > 0.999).collect();
            let half: Vec<usize> = (0..w.len()).filter(|&v| w[v] > 0.5).collect();
            let nopaste: Option<World> = matches!(key, "brow" | "cheeks" | "nose").then(|| {
                unsafe { std::env::set_var("EYES_PASTE", "0") };
                let w0 = World::load(&input, feature);
                unsafe { std::env::remove_var("EYES_PASTE") };
                w0
            });
            let one = |start: &[V3]| -> Value {
                let res = |m: &dyn Fn(V3) -> V3, idx: &[usize]| -> (f64, f64) {
                    let d: Vec<f64> = idx.iter().map(|&v| length(sub(start[v], m(base[v]))) * 1000.0).collect();
                    ((d.iter().map(|x| x * x).sum::<f64>() / d.len() as f64).sqrt(), d.iter().fold(0.0f64, |a, &b| a.max(b)))
                };
                let (tr, tm) = res(&fit(base, start, &core, false, false), &core);
                let (rr, rm) = res(&fit(base, start, &core, true, false), &core);
                let (sr, sm) = res(&fit(base, start, &core, true, true), &core);
                let (hr, hm) = res(&fit(base, start, &half, true, false), &half);
                json!({"translation": [tr, tm], "rigid": [rr, rm], "sized": [sr, sm], "rigid_half": [hr, hm]})
            };
            let rows: Vec<Value> = parallel(world.heads.len(), threads(), |i| {
                let h = &world.heads[i];
                let mut row = json!({"name": h.name, "today": one(&today[i]), "pasted": one(&pasted[i])});
                if let Some(w0) = &nopaste {
                    let h0 = &w0.heads[i];
                    let shift: Vec<f64> = match (&h.start_nose, &h0.start_nose) {
                        (Some(a), Some(b)) => a.proportions.iter().zip(&b.proportions).map(|(x, y)| x - y).collect(),
                        _ => vec![],
                    };
                    row["eye_paste_shift"] = json!(shift);
                    row["no_eye_paste"] = one(&h0.start);
                }
                row
            });
            let out = json!({"core_vertices": core.len(), "half_vertices": half.len(), "heads": rows,
                "proportions": feature.proportions().iter().map(|p| p.key).collect::<Vec<_>>()});
            std::fs::write(audit("starts_audit.json"), serde_json::to_string(&out).unwrap()).unwrap();
            eprintln!("wrote starts_audit.json ({} core vertices)", core.len());
        }
        "sweep" => {
            let set = read_set(feature, &data);
            let page = read_json(&data.join(feature.files().page));
            let sm = Smoothed::new(&set, &world.smoother);
            let steps = 13;
            let found: Vec<usize> = args[4..].iter().filter_map(|nm| world.heads.iter().position(|h| &h.name == nm)).collect();
            let n = set.sliders.len();
            let jobs: Vec<(usize, usize)> = found.iter().flat_map(|&i| (0..n).map(move |j| (i, j))).collect();
            let res: Vec<Value> = parallel(jobs.len(), threads(), |q| {
                let (i, j) = jobs[q];
                let h = &world.heads[i];
                let sl = &set.sliders[j];
                let ev = Evaluator::with(feature, &set, &sm, &world.smoother, &h.start, &world.mesh.triangles, h.frame);
                let preset = tools::solved_values(&page, &set, &h.name);
                let k = proportion_index(feature, &sl.measure);
                let pts: Vec<Value> = (0..steps)
                    .map(|s| {
                        let mut values = preset.clone();
                        let v = sl.low + (sl.high - sl.low) * s as f64 / (steps - 1) as f64;
                        values[j] = v;
                        let m = ev.measure(&values).ok();
                        json!({"value": v, "own": m.as_ref().map(|m| m.proportions[k]), "all": m.as_ref().map(|m| m.proportions.clone())})
                    })
                    .collect();
                json!({"head": h.name, "slider": sl.name, "measure": sl.measure, "preset": preset[j], "low": sl.low, "high": sl.high, "steps": pts})
            });
            std::fs::write(audit("sweep.json"), serde_json::to_string(&res).unwrap()).unwrap();
            eprintln!("wrote sweep.json ({} sweeps) in {:.1?}", res.len(), t0.elapsed());
        }
        "ridge" => {
            // `ridge <slider> <heads>`: the brow ridge's cuts (`brow::ridge_report`) at the slider's low end, preset and high end
            let set = read_set(feature, &data);
            let page = read_json(&data.join(feature.files().page));
            let sm = Smoothed::new(&set, &world.smoother);
            let j = set.sliders.iter().position(|s| s.name == args[4]).expect("slider");
            let sl = &set.sliders[j];
            let res: Vec<Value> = args[5..]
                .iter()
                .filter_map(|nm| world.heads.iter().find(|h| &h.name == nm))
                .map(|h| {
                    let ev = Evaluator::with(feature, &set, &sm, &world.smoother, &h.start, &world.mesh.triangles, h.frame);
                    let preset = tools::solved_values(&page, &set, &h.name);
                    let at = |v: f64| {
                        let mut values = preset.clone();
                        values[j] = v;
                        json!({"value": v, "ridge": head_sliders::brow::ridge_report(&ev.surface(&values))})
                    };
                    json!({"head": h.name, "steps": [at(sl.low), at(preset[j]), at(sl.high)]})
                })
                .collect();
            std::fs::write(audit("ridge.json"), serde_json::to_string(&res).unwrap()).unwrap();
            eprintln!("wrote ridge.json ({} heads)", res.len());
        }
        "pictures" => {
            let names = &args[4..];
            let shot = close_shot(key, &std::env::var("FA_SHOT").unwrap_or_else(|_| "close".into()));
            let out = PathBuf::from(std::env::var("FA_OUT").unwrap_or_else(|_| "close.png".into()));
            let rig = tools::lid_rig(&data, &world.mesh);
            let versions: Vec<(String, PathBuf, String)> = std::env::var("FA_VERSIONS")
                .expect("FA_VERSIONS")
                .split(";;")
                .map(|v| {
                    let p: Vec<&str> = v.split('|').collect();
                    (p[0].to_string(), PathBuf::from(p[1]), p[2].to_string())
                })
                .collect();
            let nopaste: Option<World> = versions.iter().any(|v| v.2 == "nopaste").then(|| {
                unsafe { std::env::set_var("EYES_PASTE", "0") };
                let w0 = World::load(&input, feature);
                unsafe { std::env::remove_var("EYES_PASTE") };
                w0
            });
            let loaded: Vec<(String, SliderSet, Value, String)> = versions
                .iter()
                .map(|(l, d, s)| (l.clone(), read_set(feature, d), read_json(&d.join(feature.files().page)), s.clone()))
                .collect();
            let found: Vec<usize> = names.iter().filter_map(|nm| world.heads.iter().position(|h| &h.name == nm)).collect();
            let rows: Vec<Vec<RgbImage>> = parallel(found.len(), threads(), |j| {
                let i = found[j];
                let h = &world.heads[i];
                let mut scenes = vec![(format!("{} KOTOR", h.name.replace('_', " ").to_uppercase()), tools::kotor_scene(&h.name, &h.kotor_v, &h.kotor_f, &h.frame))];
                for (label, set, page, starts) in &loaded {
                    let start: &[V3] = match starts.as_str() {
                        "nopaste" => &nopaste.as_ref().unwrap().heads[i].start,
                        "paste" => &pasted[i],
                        _ => &today[i],
                    };
                    // a label "name@Slider=value,Slider=value" sets those sliders over the preset
                    let mut values = tools::solved_values(page, set, &h.name);
                    let (name, over) = label.split_once('@').unwrap_or((label.as_str(), ""));
                    for (sn, v) in over.split(',').filter_map(|kv| kv.split_once('=')) {
                        let j = set.sliders.iter().position(|s| s.name == sn).expect("slider");
                        values[j] = v.parse().expect("value");
                    }
                    let built = set.build(&world.smoother, start, &values);
                    let scene = tools::our_scene(&rig, &built, &h.frame, 0.0, &head_eyes::piece::Settings::default()).0;
                    scenes.push((name.to_uppercase(), scene));
                }
                let mut row: Vec<RgbImage> = Vec::new();
                for (label, s) in &scenes {
                    let mut img = s.draw(&shot);
                    head_sliders::render::label(&mut img, label);
                    row.push(img);
                }
                for (label, s) in &scenes {
                    let mut img = lit(s, &shot);
                    head_sliders::render::label(&mut img, &format!("{} LIGHT", label.split(' ').next_back().unwrap_or("")));
                    row.push(img);
                }
                row
            });
            let columns = rows.first().map_or(1, |r| r.len());
            tools::grid(&rows.into_iter().flatten().collect::<Vec<_>>(), columns).save(&out).unwrap();
            eprintln!("wrote {}", out.display());
        }
        other => panic!("unknown command {other}"),
    }
}
