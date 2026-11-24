//! The jaw-to-neck and skull build (roadmap B3 and B5 for the jaw, the chin and the skull, and A2): the tools that check the throat
//! construction close up, build the skull sliders and solve the 57 heads with the throat, jaw and skull sliders. Viewer 2 does not use
//! it: only the `sliders jawskull` tools read it.

use crate::V3;
use crate::feature::Shot;
use crate::mesh::input::Input;
use std::path::{Path, PathBuf};

/// A place turned for a shot, as `render` turns it.
fn turn(shot: &Shot, p: V3) -> V3 {
    let (sy, cy) = shot.yaw.to_radians().sin_cos();
    let (x, z) = (p[0] * cy + p[2] * sy, -p[0] * sy + p[2] * cy);
    let (sp, cp) = shot.pitch.to_radians().sin_cos();
    let y = p[1] - shot.pivot;
    [x, cp * y + sp * z + shot.pivot, -sp * y + cp * z]
}

fn px_per_mm() -> f64 {
    std::env::var("SLIDERS_PX").ok().and_then(|v| v.parse().ok()).unwrap_or(2.5)
}

/// The depth of the nearest surface at each pixel of a shot, as `render::clay` draws it (larger is nearer).
fn depth(p: &[V3], triangles: &[[u32; 3]], shot: &Shot, w: u32, h: u32) -> Vec<f64> {
    let px = px_per_mm();
    let to_px = |q: V3| [(q[0] - shot.window[0]) * px, (shot.window[3] - q[1]) * px, q[2]];
    let mut depth = vec![f64::NEG_INFINITY; (w * h) as usize];
    for t in triangles {
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
                if z > depth[at] {
                    depth[at] = z;
                }
            }
        }
    }
    depth
}

/// Draws the mesh's edges that can be seen as thin dark lines over a clay picture made with `shot`.
pub fn wire(img: &mut image::RgbImage, shot: &Shot, positions: &[V3], triangles: &[[u32; 3]], colour: [u8; 3]) {
    let px = px_per_mm();
    let p: Vec<V3> = positions.iter().map(|&q| turn(shot, q)).collect();
    let (w, h) = (img.width(), img.height());
    let near = depth(&p, triangles, shot, w, h);
    let to_px = |q: V3| [(q[0] - shot.window[0]) * px, (shot.window[3] - q[1]) * px, q[2]];
    for t in triangles {
        let v = t.map(|i| p[i as usize]);
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            let (pa, pb) = (to_px(v[a]), to_px(v[b]));
            let steps = ((pb[0] - pa[0]).abs().max((pb[1] - pa[1]).abs()) as usize).max(1);
            for s in 0..=steps {
                let f = s as f64 / steps as f64;
                let (x, y, z) = (pa[0] + f * (pb[0] - pa[0]), pa[1] + f * (pb[1] - pa[1]), pa[2] + f * (pb[2] - pa[2]));
                if x >= 0.0 && y >= 0.0 && (x as u32) < w && (y as u32) < h && z >= near[(y as u32 * w + x as u32) as usize] - 0.4 {
                    let px = img.get_pixel_mut(x as u32, y as u32);
                    for c in 0..3 {
                        px.0[c] = ((px.0[c] as u32 + colour[c] as u32) / 2) as u8;
                    }
                }
            }
        }
    }
}

/// Draws points as coloured squares of half-size `r` pixels.
pub fn spots(img: &mut image::RgbImage, shot: &Shot, points: &[V3], colour: [u8; 3], r: i64) {
    let px = px_per_mm();
    for &d in points {
        let q = turn(shot, d);
        let (x, y) = ((q[0] - shot.window[0]) * px, (shot.window[3] - q[1]) * px);
        for dy in -r..=r {
            for dx in -r..=r {
                let (i, j) = (x as i64 + dx, y as i64 + dy);
                if i >= 0 && j >= 0 && (i as u32) < img.width() && (j as u32) < img.height() {
                    img.put_pixel(i as u32, j as u32, image::Rgb(colour));
                }
            }
        }
    }
}

/// The close-up shots of the throat: from below, three-quarter from below, and the side.
pub const CLOSE: [Shot; 4] = [
    Shot { name: "below", yaw: 0.0, pitch: 70.0, pivot: -110.0, window: [-75.0, 75.0, -200.0, -95.0] },
    Shot { name: "three-quarter below", yaw: 50.0, pitch: 25.0, pivot: -100.0, window: [-95.0, 45.0, -190.0, -85.0] },
    Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-115.0, 35.0, -185.0, -80.0] },
    Shot { name: "jaw corner", yaw: 60.0, pitch: 15.0, pivot: -100.0, window: [-90.0, -10.0, -165.0, -95.0] },
];

/// The forehead-to-skull profile: the middle cut of the head read along rays from one fixed place inside the skull, `PROFILE_CENTRE`
/// (depth, height, head frame, mm), every `PROFILE_STEP` degrees from straight ahead (the brow) over the top to straight back (the
/// back of the head). Each ray gives the farthest skin it meets, in mm from the centre. The radii are not lined up or scaled: the
/// place of the forehead and of the skull both count, so a step where the forehead meets the skull reads as a miss.
pub const PROFILE_CENTRE: [f64; 2] = [-80.0, 20.0];
pub const PROFILE_STEP: f64 = 3.0;
pub const PROFILE_RAYS: usize = 61;

pub fn skull_profile(shape: &crate::bald::Shape) -> Vec<f64> {
    let segs = crate::bald::cut(shape, 0, 0.0);
    (0..PROFILE_RAYS)
        .map(|k| {
            let a = (k as f64 * PROFILE_STEP).to_radians();
            let d = [a.cos(), a.sin()];
            let o = PROFILE_CENTRE;
            segs.iter()
                .filter_map(|([p, q], _)| {
                    let e = [q[0] - p[0], q[1] - p[1]];
                    let den = d[0] * (-e[1]) - d[1] * (-e[0]);
                    if den.abs() < 1e-12 {
                        return None;
                    }
                    let (bx, by) = (p[0] - o[0], p[1] - o[1]);
                    let t = (bx * (-e[1]) - by * (-e[0])) / den;
                    let u = (d[0] * by - d[1] * bx) / den;
                    ((0.0..=1.0).contains(&u) && t > 0.0).then_some(t)
                })
                .fold(f64::NAN, f64::max)
        })
        .collect()
}

/// The forehead's slope read as the brow feature reads it (`brow::measure`): the line fitted to the front of the middle cut from 30
/// to 62 mm above the eyes, in degrees from upright (positive leans back).
pub fn brow_style_slope(shape: &crate::bald::Shape) -> f64 {
    let pts: Vec<[f64; 2]> = crate::bald::side_outline(shape).into_iter().filter(|(y, e)| (30.0..=62.0).contains(y) && e.is_some()).map(|(y, e)| [y, e.unwrap().0.0]).collect();
    if pts.len() < 20 {
        return f64::NAN;
    }
    (-crate::brow::slope(&pts)).atan().to_degrees()
}

/// The ledge seen from the front: how far the jaw stands out over the neck on each side, in mm. The jaw's reach is its widest place
/// from the chin's bottom to 30 mm above it; the neck's is its widest place 25 mm below the chin's bottom. Both are read only in front
/// of the ears (within 95 mm behind the chin's front). `pog` and `bottom` are the chin's front (depth) and bottom (height).
pub fn ledge(shape: &crate::bald::Shape, pog: f64, bottom: f64) -> [f64; 2] {
    let reach = |y: f64| -> [f64; 2] {
        let mut out = [f64::NAN; 2];
        for ([p, q], _) in crate::bald::cut(shape, 1, y) {
            for a in [p, q] {
                if a[1] > pog - 95.0 {
                    let side = if a[0] > 0.0 { 0 } else { 1 };
                    out[side] = out[side].max(a[0].abs());
                }
            }
        }
        out
    };
    let neck = reach(bottom - 25.0);
    let mut jaw = [f64::NAN; 2];
    for k in 0..=30 {
        let r = reach(bottom + k as f64);
        jaw = [jaw[0].max(r[0]), jaw[1].max(r[1])];
    }
    [jaw[0] - neck[0], jaw[1] - neck[1]]
}

/// The gap between two profiles: the mean and the largest difference of their radii over the rays both read, in mm.
pub fn profile_gap(a: &[f64], b: &[f64]) -> (f64, f64) {
    let d: Vec<f64> = a.iter().zip(b).map(|(x, y)| (x - y).abs()).filter(|d| d.is_finite()).collect();
    if d.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    (d.iter().sum::<f64>() / d.len() as f64, d.iter().copied().fold(0.0, f64::max))
}

/// The skull sliders: name, hint, low end, high end. Each is a fixed smooth movement of the base head's vertices, in mm per unit, so
/// that the same value moves every head alike. The face does not move: every movement fades to 0 in front of the temples, below the
/// brows and down the nape, before the neck's rim.
pub const SKULL_SLIDERS: [(&str, &str); 4] = [
    ("Skull width", "How wide the skull is above the ears. One unit widens it by 1 mm."),
    ("Skull length", "How far back the back of the head lies. One unit moves it back by 1 mm."),
    ("Skull height", "How high the top of the head lies. One unit raises it by 1 mm."),
    ("Crown place", "Where the top of the head lies from front to back. One unit moves it forward by 1 mm."),
];

fn smooth01(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The skull sliders' movements at one unit, per vertex, in the head frame (mm). `top` is the base head's top above the eyes.
pub fn skull_moves(local: &[V3], top: f64) -> [Vec<V3>; 4] {
    let rise = |y: f64| ((y - 40.0) / (top - 40.0)).clamp(0.0, 1.2).powi(2);
    let width = local
        .iter()
        .map(|p| {
            // the temples narrow with the skull, so that no shoulder forms where the held forehead meets them: the width fades in
            // from the eyes' height to 60 mm above them, and to nothing in front of the forehead's sides
            let side = smooth01((p[0].abs() - 25.0) / 35.0);
            let above = smooth01(p[1] / 60.0);
            let front = smooth01((-p[2] - 5.0) / 30.0);
            [0.5 * p[0].signum() * side * above * front, 0.0, 0.0]
        })
        .collect();
    let length = local
        .iter()
        .map(|p| {
            let back = smooth01((-p[2] - 90.0) / 60.0);
            let up = smooth01((p[1] + 60.0) / 60.0);
            [0.0, 0.0, -back * up]
        })
        .collect();
    let height = local.iter().map(|p| [0.0, rise(p[1]), 0.0]).collect();
    let crown = local.iter().map(|p| [0.0, 0.0, rise(p[1])]).collect();
    [width, length, height, crown]
}

/// The skull sliders' movements at one unit, in metres, for every vertex of the kit, after the smoothing every built head gets: each
/// is the smoothed base head with the movement less the smoothed base head. The smoothing is linear, so a head with skull values `v`
/// is its built head plus the sum of `v` times these.
#[cfg(not(target_arch = "wasm32"))]
pub fn skull_world_fields(h: &crate::jaw_edge::tools::Heads, input: &Input) -> Vec<Vec<V3>> {
    let frame = crate::measure::Frame::head();
    let base: Vec<V3> = h.mesh.base.iter().map(|&p| frame.local(p)).collect();
    let top = base_top(&h.mesh, &base);
    let plain = h.smoother.smooth(&h.mesh.base, crate::set::SMOOTHING);
    // `SKULL_EAR_RIGID` spreads the skin's movement smoothly over the ear area instead; it turned more triangles below the ears
    let ear = crate::bald::tools::ear_area(input, &h.mesh);
    let fixed: Vec<bool> = ear.iter().map(|e| !e).collect();
    skull_moves(&base, top)
        .iter()
        .map(|m| {
            let mut merged = h.mesh.merge(m);
            if std::env::var("SKULL_EAR_RIGID").is_ok() {
                crate::bald::harmonic(&h.mesh, &mut merged, &fixed, 300);
            }
            let m: Vec<V3> = h.mesh.group.iter().map(|&g| merged[g as usize]).collect();
            let moved: Vec<V3> = base.iter().zip(&m).map(|(&q, &d)| frame.world(crate::add(q, d))).collect();
            let s = h.smoother.smooth(&moved, crate::set::SMOOTHING);
            s.iter().zip(&plain).map(|(a, b)| crate::sub(*a, *b)).collect()
        })
        .collect()
}

/// The base head's top above the eyes (mm): the highest skin within 10 mm of the middle.
pub fn base_top(mesh: &crate::mesh::HeadMesh, local: &[V3]) -> f64 {
    (0..mesh.merged_count()).filter(|&g| !mesh.hidden[g]).map(|g| local[mesh.first[g] as usize]).filter(|p| p[0].abs() < 10.0).map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max)
}

pub mod tools {
    use super::*;
    use crate::chin_neck::tools::{Built, build_throat};
    use crate::chin_neck::ThroatShape;
    use crate::heads::{parallel, threads};

    pub fn run(command: &str, args: &[String], data: &Path, input: &Input) -> bool {
        match command {
            // viewer 2's files for the fold of the throat and the skull, from the final solve: `fold-data <data> <final.json>`. It
            // writes web/throat/ (the construction's free vertices and the hidden vertices' skin neighbours), web/skull/sliders.json
            // (the four throat sliders and the four skull sliders), web/skull/page.json (every head's values) and
            // web/jaw/page_throat.json (jaw/page.json with the jaw values solved with the throat)
            "fold-data" => {
                use crate::chin_neck::{THROAT_KEYS, THROAT_SLIDERS};
                let web = data.join("web");
                let fin: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[4]).unwrap()).unwrap();
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let shape = ThroatShape::new(&h.mesh);
                std::fs::create_dir_all(web.join("throat")).unwrap();
                std::fs::create_dir_all(web.join("skull")).unwrap();
                let free: Vec<u8> = shape.free_flags().iter().map(|&f| f as u8).collect();
                std::fs::write(web.join("throat/free.bin"), free).unwrap();
                let near: Vec<u8> = h.mesh.near.iter().flatten().flat_map(|x| x.to_le_bytes()).collect();
                let near_w: Vec<u8> = h.mesh.near_w.iter().flatten().flat_map(|x| x.to_le_bytes()).collect();
                std::fs::write(web.join("throat/near.bin"), near).unwrap();
                std::fs::write(web.join("throat/near_w.bin"), near_w).unwrap();
                let mut sliders = Vec::new();
                for (k, (name, low, high, start, hint)) in THROAT_SLIDERS.iter().enumerate() {
                    let mut s = crate::set::Slider::from_movement(name, "throat", "1 mm", &[]);
                    (s.zone, s.low, s.high, s.hint, s.measure, s.group) = ("jaw".into(), *low, *high, hint.to_string(), THROAT_KEYS[k].into(), "throat".into());
                    let mut v = serde_json::to_value(&s).unwrap();
                    v["mid"] = serde_json::json!(start);
                    sliders.push(v);
                }
                let fields = skull_world_fields(&h, input);
                for (k, (name, hint)) in SKULL_SLIDERS.iter().enumerate() {
                    let mut s = crate::set::Slider::from_movement(name, "after", "1 mm", &fields[k]);
                    let r = &fin["skull_ranges"][k];
                    (s.zone, s.low, s.high, s.hint, s.group) = ("skull".into(), r["low"].as_f64().unwrap(), r["high"].as_f64().unwrap(), hint.to_string(), "skull".into());
                    sliders.push(serde_json::to_value(&s).unwrap());
                }
                let set = serde_json::json!({ "vertex_count": h.mesh.base.len(), "sliders": sliders, "fields": [] });
                std::fs::write(web.join("skull/sliders.json"), serde_json::to_string(&set).unwrap()).unwrap();
                let names: Vec<&str> = THROAT_SLIDERS.iter().map(|s| s.0).chain(SKULL_SLIDERS.iter().map(|s| s.0)).collect();
                let rows: Vec<serde_json::Value> = fin["heads"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| {
                        let values: Vec<f64> = r["throat"].as_array().unwrap().iter().chain(r["skull"].as_array().unwrap()).map(|x| x.as_f64().unwrap()).collect();
                        serde_json::json!({ "name": r["name"], "solved": { "full": { "counts": { "8": { "values": values } } } }, "skull_share": r["share"] })
                    })
                    .collect();
                let page = serde_json::json!({ "feature": "skull", "counts": [8], "sliders": names.iter().map(|n| serde_json::json!({ "name": n })).collect::<Vec<_>>(), "heads": rows });
                std::fs::write(web.join("skull/page.json"), serde_json::to_string(&page).unwrap()).unwrap();
                let mut jaw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(web.join("jaw/page.json")).unwrap()).unwrap();
                let last = jaw["counts"].as_array().unwrap().last().unwrap().to_string();
                let order: Vec<String> = jaw["sliders"].as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap().to_string()).collect();
                let set_names: Vec<String> = h.set.sliders[h.jaw_first..h.jaw_first + h.jaw_count].iter().map(|s| s.name.clone()).collect();
                let mut changed = 0;
                for head in jaw["heads"].as_array_mut().unwrap() {
                    let Some(r) = fin["heads"].as_array().unwrap().iter().find(|r| r["name"] == head["name"]) else { continue };
                    let Some(v) = r["jaw"].as_array() else { continue };
                    // the solve's values are in the set's order; the page lists its own order
                    let by_set: Vec<f64> = v.iter().map(|x| x.as_f64().unwrap()).collect();
                    let values: Vec<f64> = order.iter().map(|n| by_set[set_names.iter().position(|s| s == n).unwrap()]).collect();
                    let s = if head["solved"]["full"].is_object() { "full" } else { "held_out" };
                    head["solved"][s]["counts"][&last]["values"] = serde_json::json!(values);
                    changed += 1;
                }
                std::fs::write(web.join("jaw/page_throat.json"), serde_json::to_string(&jaw).unwrap()).unwrap();
                eprintln!("wrote web/throat, web/skull and web/jaw/page_throat.json ({changed} heads' jaw values)");
                true
            }
            // each feature's proportions on two folders of kit-order bins, with every row that moved more than its mark: `feature-diff
            // <data> <out.json> <before dir> <after dir>`. Only the curve's eye rows are read for the eyes.
            "feature-diff" => {
                let out = PathBuf::from(&args[4]);
                let (a, b) = (PathBuf::from(&args[5]), PathBuf::from(&args[6]));
                let mesh = input.head_mesh();
                let tris = mesh.triangles.clone();
                let frame = crate::measure::Frame::head();
                let features: Vec<(&str, &'static dyn crate::feature::Feature, Vec<f64>)> =
                    [("nose", "marks.json"), ("jaw", "jaw/marks.json"), ("brow", "brow/marks.json"), ("cheeks", "cheeks/marks.json"), ("eyes", "eyes/marks_r3.json")]
                        .iter()
                        .map(|(k, f)| (*k, crate::feature::by_key(k).unwrap(), crate::checks::Marks::read(&data.join(f)).marks.proportions))
                        .collect();
                let world = |dir: &Path, name: &str| -> Option<Vec<V3>> {
                    let raw = std::fs::read(dir.join(format!("{name}.bin"))).ok()?;
                    Some(raw.chunks_exact(12).map(|c| std::array::from_fn(|k| f32::from_le_bytes(c[4 * k..4 * k + 4].try_into().unwrap()) as f64)).collect())
                };
                let names: Vec<String> = std::fs::read_dir(&a).unwrap().filter_map(|e| e.ok()?.file_name().to_str()?.strip_suffix(".bin").map(String::from)).collect();
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let (Some(pa), Some(pb)) = (world(&a, name), world(&b, name)) else { return serde_json::json!({ "name": name }) };
                    let mut moved = Vec::new();
                    let mut worst = serde_json::Map::new();
                    for (key, f, marks) in &features {
                        let m = |p: &[V3]| f.measure(&crate::measure::Surface::within(p, &tris, &frame, &f.window())).ok().map(|m| m.proportions);
                        let (Some(x), Some(y)) = (m(&pa), m(&pb)) else { moved.push(format!("{key}: not measured")); continue };
                        let mut w: f64 = 0.0;
                        let mut w_key = "";
                        for (i, (u, v)) in x.iter().zip(&y).enumerate() {
                            let d = v - u;
                            if d.is_finite() && marks.get(i).is_some_and(|mk| mk.is_finite() && *mk > 0.0) {
                                if d.abs() / marks[i] > w {
                                    w_key = f.proportions()[i].key;
                                }
                                w = w.max(d.abs() / marks[i]);
                                if d.abs() > marks[i] {
                                    moved.push(format!("{key} {} {:+.2} (mark {:.2})", f.proportions()[i].key, d, marks[i]));
                                }
                            }
                        }
                        worst.insert(key.to_string(), serde_json::json!([w, w_key]));
                    }
                    // the triangles turned in the second head against the first, with the places of a few (head frame, mm)
                    let all: Vec<u32> = (0..pa.len() as u32).collect();
                    let checker = crate::check::Checker::from_parts(&tris, &vec![false; pa.len()], &pa, &all);
                    let marks = checker.marks(&pb);
                    let at: Vec<String> = tris.iter().zip(&marks).filter(|(_, m)| **m).take(4).map(|(t, _)| {
                        let c = frame.local(pb[t[0] as usize]);
                        format!("({:.0} {:.0} {:.0})", c[0], c[1], c[2])
                    }).collect();
                    let turned = checker.faults(&pb).total();
                    if turned > 0 {
                        moved.push(format!("{turned} turned at {}", at.join(" ")));
                    }
                    serde_json::json!({ "name": name, "moved": moved, "worst_share_of_mark": worst, "turned": turned })
                });
                std::fs::write(&out, serde_json::to_string_pretty(&rows).unwrap()).unwrap();
                for r in &rows {
                    if let Some(m) = r["moved"].as_array().filter(|m| !m.is_empty()) {
                        eprintln!("{}: {}", r["name"].as_str().unwrap(), m.iter().map(|x| x.as_str().unwrap()).collect::<Vec<_>>().join(", "));
                    }
                }
                eprintln!("wrote {}", out.display());
                true
            }
            // the throat sliders' ends and pair corners on every head as viewer 2 builds it, the other values at the head's own: the
            // triangles turned against the head at its own values, with a few places (head frame, mm): `throat-corners <data> <out.json>
            // [head]...`. `TC_RANGES=lo,hi;lo,hi;lo,hi;lo,hi` tries other ranges.
            "throat-corners" => {
                use crate::chin_neck::{THROAT_SLIDERS, Throat};
                use crate::jaw_edge::tools::{FOLD_FILES, Heads};
                let out = PathBuf::from(&args[4]);
                let mut h = Heads::load_viewer(input, data, &FOLD_FILES);
                let names: Vec<String> = if args.len() > 5 { args[5..].to_vec() } else { h.names.clone() };
                let first = h.set.sliders.iter().position(|s| s.kind == "throat").unwrap();
                // the head without the throat: the throat sliders renamed, so the build skips them, and the throat applied after
                for s in h.set.sliders.iter_mut().filter(|s| s.kind == "throat") {
                    s.kind = "throat_off".into();
                }
                let shape = ThroatShape::new(&h.mesh);
                let mut ranges: Vec<(f64, f64)> = THROAT_SLIDERS.iter().map(|s| (s.1, s.2)).collect();
                if let Ok(r) = std::env::var("TC_RANGES") {
                    ranges = r.split(';').map(|p| { let (a, b) = p.split_once(',').unwrap(); (a.parse().unwrap(), b.parse().unwrap()) }).collect();
                }
                let frame = crate::measure::Frame::head();
                let tris = h.mesh.triangles.clone();
                let all: Vec<u32> = (0..h.mesh.base.len() as u32).collect();
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let v = h.values(name);
                    let own: Vec<f64> = v[first..first + 4].to_vec();
                    let pre = h.build(name, &v);
                    let at = |t: &[f64]| shape.apply(&pre, &Throat::from_slice(t));
                    let mine = at(&own);
                    let checker = crate::check::Checker::from_parts(&tris, &vec![false; mine.len()], &mine, &all);
                    let count = |t: &[f64]| -> (usize, Vec<String>) {
                        let p = at(t);
                        let marks = checker.marks(&p);
                        let places = tris.iter().zip(&marks).filter(|(_, m)| **m).take(3).map(|(tr, _)| {
                            let c = frame.local(p[tr[0] as usize]);
                            format!("({:.0} {:.0} {:.0})", c[0], c[1], c[2])
                        }).collect();
                        (checker.faults(&p).total(), places)
                    };
                    let mut cases = Vec::new();
                    for a in 0..4 {
                        for (ea, xa) in [("low", ranges[a].0), ("high", ranges[a].1)] {
                            let mut t = own.clone();
                            t[a] = xa;
                            let (n, at) = count(&t);
                            if n > 0 {
                                cases.push(serde_json::json!({ "case": format!("{} {ea}", THROAT_SLIDERS[a].0), "turned": n, "at": at }));
                            }
                            for b in a + 1..4 {
                                for (eb, xb) in [("low", ranges[b].0), ("high", ranges[b].1)] {
                                    let mut t = t.clone();
                                    t[b] = xb;
                                    let (n, at) = count(&t);
                                    if n > 0 {
                                        cases.push(serde_json::json!({ "case": format!("{} {ea} and {} {eb}", THROAT_SLIDERS[a].0, THROAT_SLIDERS[b].0), "turned": n, "at": at }));
                                    }
                                }
                            }
                        }
                    }
                    // how far the corner's middle rises above the outline's start, at most over the sections, at the head's own values
                    let rise = shape.section_probe(&pre, &Throat::from_slice(&own)).iter().filter(|s| s.1).map(|s| s.3[1] - s.2[1]).fold(f64::NEG_INFINITY, f64::max);
                    // how far the corner's middle lies behind the neck's front 34 mm below the chin, at most over the sections within 20 mm of the middle
                    let behind = shape.section_probe(&pre, &Throat::from_slice(&own)).iter().filter(|s| s.1 && s.0.abs() <= 20.0).map(|s| s.4[0] - s.3[0]).fold(f64::NEG_INFINITY, f64::max);
                    serde_json::json!({ "name": name, "own": own, "own_turned": count(&own).0, "own_corner_above_start": rise, "own_corner_behind_neck": behind, "cases": cases })
                });
                let mut by_case: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
                for r in &rows {
                    for c in r["cases"].as_array().unwrap() {
                        let e = by_case.entry(c["case"].as_str().unwrap().to_string()).or_default();
                        e.0 += 1;
                        e.1 = e.1.max(c["turned"].as_u64().unwrap() as usize);
                    }
                }
                let heads_hit = rows.iter().filter(|r| !r["cases"].as_array().unwrap().is_empty()).count();
                for (k, (n, m)) in &by_case {
                    eprintln!("{k}: {n} heads, up to {m}");
                }
                eprintln!("{heads_hit} of {} heads turn a triangle at an end or a pair corner", rows.len());
                std::fs::write(&out, serde_json::to_string_pretty(&serde_json::json!({ "ranges": ranges, "by_case": by_case, "heads": rows })).unwrap()).unwrap();
                true
            }
            // one head's throat sections and turned triangles at throat values: `throat-probe <data> <head> <depth,height,rise,radius>`
            "throat-probe" => {
                use crate::chin_neck::Throat;
                use crate::jaw_edge::tools::{FOLD_FILES, Heads};
                let mut h = Heads::load_viewer(input, data, &FOLD_FILES);
                let name = &args[4];
                let t: Vec<f64> = args[5].split(',').map(|x| x.parse().unwrap()).collect();
                let first = h.set.sliders.iter().position(|s| s.kind == "throat").unwrap();
                for s in h.set.sliders.iter_mut().filter(|s| s.kind == "throat") {
                    s.kind = "throat_off".into();
                }
                let shape = ThroatShape::new(&h.mesh);
                let v = h.values(name);
                let pre = h.build(name, &v);
                let mine = shape.apply(&pre, &Throat::from_slice(&v[first..first + 4]));
                let p = shape.apply(&pre, &Throat::from_slice(&t));
                for (x, ok, start, corner, _) in shape.section_probe(&pre, &Throat::from_slice(&t)) {
                    if (x as i64) % 2 == 0 {
                        println!("x {x:5.0} {} start ({:6.1} {:6.1}) corner ({:6.1} {:6.1})", if ok { "read" } else { "NONE" }, start[0], start[1], corner[0], corner[1]);
                    }
                }
                let frame = crate::measure::Frame::head();
                let all: Vec<u32> = (0..h.mesh.base.len() as u32).collect();
                let checker = crate::check::Checker::from_parts(&h.mesh.triangles, &vec![false; mine.len()], &mine, &all);
                let marks = checker.marks(&p);
                println!("turned {}", checker.faults(&p).total());
                for (tr, _) in h.mesh.triangles.iter().zip(&marks).filter(|(_, m)| **m) {
                    let a: Vec<String> = tr.iter().map(|&i| {
                        let (b, c) = (frame.local(mine[i as usize]), frame.local(p[i as usize]));
                        let g = h.mesh.group[i as usize] as usize;
                        format!("v{i}{} ({:.0} {:.0} {:.0})->({:.0} {:.0} {:.0})", if h.mesh.hidden[g] { "h" } else { "" }, b[0], b[1], b[2], c[0], c[1], c[2])
                    }).collect();
                    println!("  {}", a.join("  "));
                }
                true
            }
            // the brow's Forehead slope solved on every head as viewer 2 builds it, against the front of the forehead-to-skull profile
            // (rays 0 to `FS_RAYS`, straight ahead to 60 degrees up by default) of KOTOR where its skull shows and of the bald
            // reference elsewhere. Only Forehead slope moves. A head takes the value, or the largest share of its move (1, 0.75, 0.5,
            // 0.25), whose head keeps every other proportion of the nose, jaw, brow, cheeks and eyes within its mark, turns no triangle,
            // and has a whole profile no worse: `forehead-solve <data> <out.json> <ref dir> [head]...`
            "forehead-solve" => {
                use crate::bald::tools::{hair_faces, kotor, kotor_shape, load_reference, our_shape, reference_shape};
                use crate::jaw_edge::tools::{FOLD_FILES, Heads};
                let out = PathBuf::from(&args[4]);
                let refs = PathBuf::from(&args[5]);
                let hair = PathBuf::from(std::env::var("BALD_HAIR").expect("BALD_HAIR"));
                let extra = std::env::var("V2_EXTRA").ok();
                let mut files: Vec<(&str, &str)> = FOLD_FILES.to_vec();
                if let Some((a, b)) = extra.as_deref().and_then(|e| e.split_once(':')) {
                    files.insert(files.len() - 1, (a, b));
                }
                let h = Heads::load_viewer(input, data, &files);
                let rays: usize = std::env::var("FS_RAYS").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
                let names: Vec<String> = if args.len() > 6 { args[6..].to_vec() } else { h.names.clone() };
                let k_fs = h.set.sliders.iter().position(|s| s.measure == "forehead_slope").unwrap();
                let (lo, hi) = (h.set.sliders[k_fs].low, h.set.sliders[k_fs].high);
                let mesh = &h.mesh;
                let region = crate::bald::tools::region(input, mesh);
                let frame = crate::measure::Frame::head();
                let tris = mesh.triangles.clone();
                let all: Vec<u32> = (0..mesh.base.len() as u32).collect();
                let features: Vec<(&str, &'static dyn crate::feature::Feature, Vec<f64>)> =
                    [("nose", "marks.json"), ("jaw", "jaw/marks.json"), ("brow", "brow/marks.json"), ("cheeks", "cheeks/marks.json"), ("eyes", "eyes/marks_r3.json")]
                        .iter()
                        .map(|(k, f)| (*k, crate::feature::by_key(k).unwrap(), crate::checks::Marks::read(&data.join(f)).marks.proportions))
                        .collect();
                let props = |p: &[V3]| -> Vec<Option<Vec<f64>>> {
                    features.iter().map(|(_, f, _)| f.measure(&crate::measure::Surface::within(p, &tris, &frame, &f.window())).ok().map(|m| m.proportions)).collect()
                };
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    let shows = crate::bald::SKULL_SHOWS.contains(&name.as_str());
                    let has_ref = refs.join(format!("{name}_ref.bin")).exists();
                    let target_shape = if shows || !has_ref { kotor_shape(&k, &hf) } else { reference_shape(&k, &hf, &load_reference(&refs, name, mesh, &region), mesh) };
                    let target = skull_profile(&target_shape);
                    let v0 = h.values(name);
                    let at = |x: f64| { let mut v = v0.clone(); v[k_fs] = x; h.build(name, &v) };
                    let shape_of = |p: &[V3]| { let l: Vec<V3> = p.iter().map(|&a| frame.local(a)).collect(); our_shape(mesh, &l) };
                    let gaps = |p: &[V3]| -> (f64, f64, f64) {
                        let sh = shape_of(p);
                        let pr = skull_profile(&sh);
                        let front: Vec<f64> = (0..=rays).filter_map(|r| { let d = pr[r] - target[r]; d.is_finite().then_some(d.abs()) }).collect();
                        let whole = profile_gap(&pr, &target).0;
                        (front.iter().sum::<f64>() / front.len().max(1) as f64, whole, brow_style_slope(&sh))
                    };
                    let now = at(v0[k_fs]);
                    let (f0, w0, s0) = gaps(&now);
                    // `FS_COST=slope` solves the brow feature's reading of the slope toward the target's instead of the front profile
                    let target_slope = brow_style_slope(&target_shape);
                    let by_slope = std::env::var("FS_COST").is_ok_and(|c| c == "slope") && target_slope.is_finite();
                    let gaps = |p: &[V3]| -> (f64, f64, f64) { let (f, w, sl) = gaps(p); (if by_slope { (sl - target_slope).abs() } else { f }, w, sl) };
                    let f0 = if by_slope { (s0 - target_slope).abs() } else { f0 };
                    // a coarse search over the range, then a finer one round the best
                    let mut best = (f0, v0[k_fs]);
                    for i in 0..=44 {
                        let x = lo + (hi - lo) * i as f64 / 44.0;
                        let c = gaps(&at(x)).0;
                        if c < best.0 { best = (c, x); }
                    }
                    let step = (hi - lo) / 44.0;
                    for i in -5..=5 {
                        let x = (best.1 + step * i as f64 / 5.0).clamp(lo, hi);
                        let c = gaps(&at(x)).0;
                        if c < best.0 { best = (c, x); }
                    }
                    let base_props = props(&now);
                    let checker = crate::check::Checker::from_parts(&tris, &vec![false; now.len()], &now, &all);
                    let guard = |p: &[V3]| -> Vec<String> {
                        let mut why = Vec::new();
                        for (((key, f, marks), old), new) in features.iter().zip(&base_props).zip(props(p)) {
                            let (Some(old), Some(new)) = (old, new) else { continue };
                            for (i, (a, c)) in new.iter().zip(old).enumerate() {
                                if f.proportions()[i].key == "forehead_slope" && *key == "brow" { continue; }
                                if (a - c).is_finite() && marks.get(i).is_some_and(|m| (a - c).abs() > *m) {
                                    why.push(format!("{key} {} {:+.2}", f.proportions()[i].key, a - c));
                                }
                            }
                        }
                        let n = checker.faults(p).total();
                        if n > 0 { why.push(format!("{n} turned")); }
                        why
                    };
                    let mut chosen = (v0[k_fs], 0.0, Vec::new());
                    let mut why_full = Vec::new();
                    for (i, share) in [1.0, 0.75, 0.5, 0.25].iter().enumerate() {
                        let x = v0[k_fs] + share * (best.1 - v0[k_fs]);
                        let p = at(x);
                        let mut why = guard(&p);
                        let (_, w, _) = gaps(&p);
                        if w > w0 + 1e-9 { why.push(format!("whole profile {w:.2} against {w0:.2}")); }
                        if i == 0 { why_full = why.clone(); }
                        if why.is_empty() { chosen = (x, *share, why); break; }
                    }
                    let p = at(chosen.0);
                    let (f1, w1, s1) = gaps(&p);
                    serde_json::json!({ "name": name, "target": if shows || !has_ref { "kotor" } else { "reference" }, "before": v0[k_fs], "best": best.1, "after": chosen.0, "share": chosen.1,
                        "why_full_failed": why_full, "front_gap": [f0, f1], "whole_gap": [w0, w1], "slope": [s0, s1], "slope_target": brow_style_slope(&target_shape) })
                });
                for r in &rows {
                    eprintln!("{:22} {:9} slope value {:+6.2} -> {:+6.2} (best {:+6.2}, share {}) front gap {:.2} -> {:.2}, whole {:.2} -> {:.2}, slope {:.1} -> {:.1} (target {:.1}) {}",
                        r["name"].as_str().unwrap(), r["target"].as_str().unwrap(), r["before"].as_f64().unwrap(), r["after"].as_f64().unwrap(), r["best"].as_f64().unwrap(), r["share"],
                        r["front_gap"][0].as_f64().unwrap(), r["front_gap"][1].as_f64().unwrap(), r["whole_gap"][0].as_f64().unwrap(), r["whole_gap"][1].as_f64().unwrap(),
                        r["slope"][0].as_f64().unwrap(), r["slope"][1].as_f64().unwrap(), r["slope_target"].as_f64().unwrap_or(f64::NAN),
                        r["why_full_failed"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect::<Vec<_>>().join(", "));
                }
                std::fs::write(&out, serde_json::to_string_pretty(&rows).unwrap()).unwrap();
                true
            }
            // the heads as viewer 2 builds them, in kit order (metres): `v2-bins <data> <out dir> <before|fold> [head]...`, and the
            // time of each build
            "v2-bins" => {
                use crate::jaw_edge::tools::{FOLD_FILES, Heads, VIEWER_FILES};
                let out = PathBuf::from(&args[4]);
                std::fs::create_dir_all(&out).unwrap();
                // `V2_EXTRA=<sliders file>:<page file>` adds a feature after the list, as another conversation adds one to `sliders.js`
                let extra = std::env::var("V2_EXTRA").ok();
                let mut files: Vec<(&str, &str)> = if args[5] == "fold" { FOLD_FILES.to_vec() } else { VIEWER_FILES.to_vec() };
                if let Some((a, b)) = extra.as_deref().and_then(|e| e.split_once(':')) {
                    files.insert(files.len() - usize::from(args[5] == "fold"), (a, b));
                }
                // `V2_BROW=<page file>` replaces the brow's presets
                let brow = std::env::var("V2_BROW").ok();
                if let Some(b) = brow.as_deref() {
                    for f in files.iter_mut().filter(|f| f.0 == "brow/sliders.json") {
                        f.1 = b;
                    }
                }
                let h = Heads::load_viewer(input, data, &files);
                let names: Vec<String> = if args.len() > 6 { args[6..].to_vec() } else { h.names.clone() };
                let ms: Vec<f64> = parallel(names.len(), threads(), |q| {
                    let t = std::time::Instant::now();
                    let p = h.build(&names[q], &h.values(&names[q]));
                    let ms = t.elapsed().as_secs_f64() * 1e3;
                    crate::chin_neck::tools::write_bin(&out, &names[q], &p);
                    ms
                });
                let mut s = ms.clone();
                s.sort_by(f64::total_cmp);
                eprintln!("wrote {} heads to {}, build {:.0} ms median", names.len(), out.display(), s[s.len() / 2]);
                true
            }
            // heads built from a values file (as `chinneck throat-bins` reads it), in clay with the mesh's edges, the hidden vertices in
            // blue and the corner line's places in red, close up from below, three-quarter below and the side: `wire <data> <out.png>
            // <values.json> -- <head>...`
            "wire" => {
                let out = args[4].clone();
                let table: std::collections::HashMap<String, Built> = serde_json::from_str(&std::fs::read_to_string(&args[5]).unwrap()).unwrap();
                let split = args.iter().position(|a| a == "--").unwrap();
                let heads: Vec<String> = args[split + 1..].to_vec();
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let shape = ThroatShape::new(&h.mesh);
                let frame = crate::measure::Frame::head();
                let tiles: Vec<Vec<image::RgbImage>> = parallel(heads.len(), threads(), |q| {
                    let name = &heads[q];
                    let b = table.get(name).or_else(|| table.get("*")).expect("values");
                    let p = build_throat(&h, &shape, name, b);
                    let local: Vec<V3> = p.iter().map(|&a| frame.local(a)).collect();
                    let hidden: Vec<V3> = (0..h.mesh.merged_count()).filter(|&g| h.mesh.hidden[g]).map(|g| local[h.mesh.first[g] as usize]).collect();
                    let all: Vec<u32> = (0..h.mesh.base.len() as u32).collect();
                    let live = h.live_with(name, None);
                    let red = crate::check::Checker::from_parts(&h.mesh.triangles, &vec![false; live.len()], &live, &all).marks(&p);
                    eprintln!("{name}: {} turned", red.iter().filter(|r| **r).count());
                    if std::env::var("JS_TURNED").is_ok() {
                        let lv: Vec<V3> = live.iter().map(|&a| frame.local(a)).collect();
                        let lp: Vec<V3> = p.iter().map(|&a| frame.local(a)).collect();
                        for (t, _) in h.mesh.triangles.iter().zip(&red).filter(|(_, r)| **r).take(4) {
                            for &i in t {
                                let g = h.mesh.group[i as usize] as usize;
                                eprintln!("  v{i} g{g} hidden {} keep {} live {:?} new {:?}", h.mesh.hidden[g], h.mesh.keep[i as usize], lv[i as usize].map(|c| c.round()), lp[i as usize].map(|c| c.round()));
                            }
                        }
                    }
                    let corner = b.throat.as_ref().map(|t| shape.corner_line(&p, &crate::chin_neck::Throat::from_slice(t))).unwrap_or_default();
                    CLOSE
                        .iter()
                        .enumerate()
                        .map(|(s, shot)| {
                            let mut img = crate::render::clay(&local, &h.mesh.triangles, shot, &red, &[]);
                            if std::env::var("JS_NOWIRE").is_err() {
                                wire(&mut img, shot, &local, &h.mesh.triangles, [30, 30, 30]);
                            }
                            spots(&mut img, shot, &hidden, [60, 120, 255], 1);
                            spots(&mut img, shot, &corner, [235, 40, 40], 1);
                            if s == 0 {
                                crate::render::label(&mut img, name);
                            }
                            img
                        })
                        .collect()
                });
                let flat: Vec<image::RgbImage> = tiles.into_iter().flatten().collect();
                crate::render::sheet(&flat, CLOSE.len()).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // KOTOR's bare head from the side in clay (the reference on haired heads), with the profile of KOTOR or the reference in
            // red, live in yellow and each folder of kit-order bins in its colour: `skull-overlay <data> <out.png> <ref dir>
            // <columns> [label=<bins dir>]... -- <head>...`
            "skull-overlay" => {
                use crate::bald::tools::{hair_faces, kotor, kotor_shape, load_reference, our_shape, reference_mesh};
                let out = args[4].clone();
                let refs = PathBuf::from(&args[5]);
                let columns: usize = args[6].parse().unwrap();
                let split = args.iter().position(|a| a == "--").unwrap();
                let variants: Vec<(String, PathBuf)> = args[7..split].iter().map(|v| { let (a, b) = v.split_once('=').unwrap(); (a.to_string(), PathBuf::from(b)) }).collect();
                let heads: Vec<String> = args[split + 1..].to_vec();
                let hair = PathBuf::from(std::env::var("BALD_HAIR").expect("BALD_HAIR"));
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let mesh = &h.mesh;
                let region = crate::bald::tools::region(input, mesh);
                let frame = crate::measure::Frame::head();
                let shot = crate::bald::SHOTS[0];
                let line = |prof: &[f64]| -> Vec<V3> {
                    let mut pts = Vec::new();
                    for k in 0..prof.len().saturating_sub(1) {
                        for s in 0..10 {
                            let a = (k as f64 + s as f64 / 10.0) * PROFILE_STEP;
                            let r = prof[k] + (prof[k + 1] - prof[k]) * s as f64 / 10.0;
                            let a = a.to_radians();
                            pts.push([0.0, PROFILE_CENTRE[1] + r * a.sin(), PROFILE_CENTRE[0] + r * a.cos()]);
                        }
                    }
                    pts.into_iter().filter(|p| p[1].is_finite()).collect()
                };
                let palette = [[80u8, 230, 110], [80, 200, 255], [240, 120, 240]];
                let tiles: Vec<image::RgbImage> = parallel(heads.len(), threads(), |q| {
                    let name = &heads[q];
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    let has_ref = refs.join(format!("{name}_ref.bin")).exists();
                    let (target, mut img) = if has_ref && !crate::bald::SKULL_SHOWS.contains(&name.as_str()) {
                        let r = load_reference(&refs, name, mesh, &region);
                        let (v, f, tint) = reference_mesh(&k, &hf, &r, mesh, false);
                        let sh = crate::bald::Shape { tris: f.iter().map(|t| t.map(|i| v[i as usize])).collect(), filled: tint.iter().map(|t| t.is_some()).collect() };
                        (skull_profile(&sh), crate::render::clay(&v, &f, &shot, &[], &tint))
                    } else {
                        let bare: Vec<[u32; 3]> = k.f.iter().zip(&hf).filter(|(_, h)| !**h).map(|(t, _)| *t).collect();
                        (skull_profile(&kotor_shape(&k, &hf)), crate::render::clay(&k.v, &bare, &shot, &[], &[]))
                    };
                    let live: Vec<V3> = h.live_with(name, None).iter().map(|&p| frame.local(p)).collect();
                    spots(&mut img, &shot, &line(&target), [235, 50, 40], 1);
                    spots(&mut img, &shot, &line(&skull_profile(&our_shape(mesh, &live))), [255, 220, 60], 0);
                    for (i, (_, dir)) in variants.iter().enumerate() {
                        if let Some(p) = crate::chin_neck::tools::read_bin(dir, name) {
                            spots(&mut img, &shot, &line(&skull_profile(&our_shape(mesh, &p))), palette[i % 3], 0);
                        }
                    }
                    crate::render::label(&mut img, name);
                    img
                });
                crate::render::sheet(&tiles, columns).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // the skull sliders solved on every head toward KOTOR's skull (the 36 heads whose skull shows) or the bald reference (the
            // 21 haired heads): the forehead-to-skull profile and the skull's width, by a coordinate search from 0. Writes the values
            // and the heads as kit-order bins (live plus the smoothed skull movements): `skull-solve <data> <out dir> <ref dir>`
            "skull-solve" => {
                use crate::bald::tools::{hair_faces, kotor, kotor_shape, load_reference, our_shape, reference_shape};
                let out = PathBuf::from(&args[4]);
                let refs = PathBuf::from(&args[5]);
                std::fs::create_dir_all(out.join("bins")).unwrap();
                let hair = PathBuf::from(std::env::var("BALD_HAIR").expect("BALD_HAIR"));
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let mesh = &h.mesh;
                let region = crate::bald::tools::region(input, mesh);
                let frame = crate::measure::Frame::head();
                let fields = skull_world_fields(&h, input);
                let names = crate::bald::tools::names(input);
                let lim = 45.0;
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    let shows = crate::bald::SKULL_SHOWS.contains(&name.as_str());
                    let has_ref = refs.join(format!("{name}_ref.bin")).exists();
                    let target = if shows { Some(kotor_shape(&k, &hf)) } else if has_ref { Some(reference_shape(&k, &hf, &load_reference(&refs, name, mesh, &region), mesh)) } else { None };
                    let live = h.live_with(name, None);
                    let build = |v: &[f64]| -> Vec<V3> {
                        let mut p = live.clone();
                        for (f, &x) in fields.iter().zip(v) {
                            for (a, d) in p.iter_mut().zip(f) {
                                *a = crate::add(*a, crate::scale(*d, x));
                            }
                        }
                        p
                    };
                    let read = |p: &[V3]| { let l: Vec<V3> = p.iter().map(|&a| frame.local(a)).collect(); let sh = our_shape(mesh, &l); (skull_profile(&sh), crate::skull::skull_width(&sh)) };
                    let mut v = vec![0.0; 4];
                    let mut row = serde_json::json!({"name": name, "skull_shows": shows});
                    if let Some(t) = &target {
                        let (tp, tw) = (skull_profile(t), crate::skull::skull_width(t));
                        let cost = |v: &[f64]| { let (p, w) = read(&build(v)); let (g, m) = profile_gap(&tp, &p); (g / 1.0).powi(2) + (m / 4.0).powi(2) + if tw.is_finite() && w.is_finite() { ((w - tw) / 2.0).powi(2) } else { 0.0 } };
                        let mut best = cost(&v);
                        let c0 = best;
                        for step in [16.0, 8.0, 4.0, 2.0, 1.0, 0.5, 0.25] {
                            let mut moved = true;
                            while moved {
                                moved = false;
                                for s in 0..4 {
                                    for dir in [-1.0, 1.0] {
                                        let mut y = v.clone();
                                        y[s] = (v[s] + dir * step).clamp(-lim, lim);
                                        let c = cost(&y);
                                        if c < best - 1e-9 {
                                            best = c;
                                            v = y;
                                            moved = true;
                                        }
                                    }
                                }
                            }
                        }
                        let (lp, lw) = read(&live);
                        let (np, nw) = read(&build(&v));
                        row["target_width"] = serde_json::json!(tw);
                        row["live"] = serde_json::json!({"gap": profile_gap(&tp, &lp).0, "max": profile_gap(&tp, &lp).1, "width": lw, "cost": c0});
                        row["new"] = serde_json::json!({"gap": profile_gap(&tp, &np).0, "max": profile_gap(&tp, &np).1, "width": nw, "cost": best});
                    }
                    row["values"] = serde_json::json!(v);
                    crate::chin_neck::tools::write_bin(&out.join("bins"), name, &build(&v));
                    row
                });
                let sliders: Vec<&str> = SKULL_SLIDERS.iter().map(|s| s.0).collect();
                std::fs::write(out.join("solve.json"), serde_json::to_string_pretty(&serde_json::json!({"sliders": sliders, "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {}", out.display());
                true
            }
            // the ledge seen from the front on every head: KOTOR's bare mesh, the same subdivided twice, live and folders of kit-order
            // bins: `ledge-check <data> <out.json> [label=<bins dir>]...`
            "ledge-check" => {
                use crate::under_jaw::tools::{kotor_bare, kotor_measure, local_surface};
                let out = args[4].clone();
                let variants: Vec<(String, PathBuf)> = args[5..].iter().map(|v| { let (a, b) = v.split_once('=').unwrap(); (a.to_string(), PathBuf::from(b)) }).collect();
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let frame = crate::measure::Frame::head();
                let names = crate::bald::tools::names(input);
                let read = |v: &[V3], f: &[[u32; 3]]| -> Option<[f64; 2]> {
                    let (pog, bottom) = crate::chin_neck::chin(&local_surface(v, f))?;
                    let sh = crate::bald::Shape { tris: f.iter().map(|t| t.map(|i| v[i as usize])).collect(), filled: vec![false; f.len()] };
                    Some(ledge(&sh, pog[0], bottom))
                };
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let (v, f, _) = kotor_bare(input, name);
                    let hidden = kotor_measure(input, name).1;
                    let (sv, sf, _) = crate::checks::butterfly(&v, &f);
                    let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
                    let live: Vec<V3> = h.live_with(name, None).iter().map(|&p| frame.local(p)).collect();
                    let mut row = serde_json::json!({"name": name, "hidden": hidden, "kotor": read(&v, &f), "kotor_sub": read(&sv, &sf), "live": read(&live, &h.mesh.triangles)});
                    for (label, dir) in &variants {
                        if let Some(p) = crate::chin_neck::tools::read_bin(dir, name) {
                            row[label] = serde_json::json!(read(&p, &h.mesh.triangles));
                        }
                    }
                    row
                });
                std::fs::write(&out, serde_json::to_string(&serde_json::json!({"heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // the throat and jaw sliders solved on every head toward KOTOR's chin-to-neck outline, holding the jaw measure where the live
            // preset has it and the chin in place, then guarded: `throat-solve <data> <out dir> [head...]`. Writes `values.json` (as
            // `chinneck throat-bins` reads it) and `solve.json` (the gaps, the guard and why a head kept its jaw preset).
            "throat-solve" => {
                use crate::chin_neck::{THROAT_SLIDERS, Throat, chin, measure, whole_gap};
                use crate::under_jaw::tools::local_surface;
                let out = PathBuf::from(&args[4]);
                std::fs::create_dir_all(&out).unwrap();
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let shape = crate::chin_neck::ThroatShape::new(&h.mesh);
                let names: Vec<String> = if args.len() > 5 { args[5..].to_vec() } else { crate::bald::tools::names(input) };
                let all: Vec<u32> = (0..h.mesh.base.len() as u32).collect();
                let tris = h.mesh.triangles.clone();
                let guard_features: Vec<(&str, &'static dyn crate::feature::Feature, Vec<f64>)> = [("nose", "marks.json"), ("brow", "brow/marks.json"), ("cheeks", "cheeks/marks.json"), ("jaw", "jaw/marks.json")]
                    .iter()
                    .map(|(k, f)| (*k, crate::feature::by_key(k).unwrap(), crate::checks::Marks::read(&data.join(f)).marks.proportions))
                    .collect();
                let feature_of = |f: &dyn crate::feature::Feature, p: &[V3]| f.measure(&crate::measure::Surface::within(p, &tris, &h.frame, &f.window())).ok().map(|m| m.proportions);
                let n_jaw = h.jaw_count;
                let ranges: Vec<(f64, f64)> = h.jaw_ranges().into_iter().chain(THROAT_SLIDERS.iter().map(|s| (s.1, s.2))).collect();
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let preset = h.preset(h.index(name));
                    let live = h.live_with(name, None);
                    let local = |p: &[V3]| -> Vec<V3> { p.iter().map(|&a| h.frame.local(a)).collect() };
                    let checker = crate::check::Checker::from_parts(&tris, &vec![false; live.len()], &live, &all);
                    let build = |x: &[f64]| -> Vec<V3> { shape.apply(&h.live_with(name, Some(&x[..n_jaw])), &Throat::from_slice(&x[n_jaw..])) };
                    let (target, _) = crate::chin_neck::tools::kotor(input, name);
                    let target = target.ok().filter(|t| t.outlines[0][0].is_some() && t.outlines[3][0].is_some());
                    let live_feats: Vec<Option<Vec<f64>>> = guard_features.iter().map(|(_, f, _)| feature_of(*f, &live)).collect();
                    let live_gap = target.as_ref().and_then(|t| measure(&local_surface(&local(&live), &tris)).ok().and_then(|m| whole_gap(t, &m)));
                    let Some(target) = target else {
                        return serde_json::json!({"name": name, "readable": false, "live_gap": live_gap, "jaw": preset});
                    };
                    let window = crate::feature::Feature::window(&crate::jaw::JAW);
                    let jaw_of = |p: &[V3]| crate::jaw::measure(&crate::measure::Surface::within(p, &tris, &h.frame, &window)).ok();
                    let held = jaw_of(&live).expect("the preset's jaw");
                    let live_chin = chin(&local_surface(&local(&live), &tris)).unwrap();
                    let parts = |x: &[f64], hold_jaw: bool| -> (f64, f64, usize) {
                        let p = build(x);
                        let Ok(m) = measure(&local_surface(&local(&p), &tris)) else { return (1e9, 1e9, 0) };
                        let g = whole_gap(&target, &m).unwrap_or(1e3);
                        let mut hold = 0.0;
                        if hold_jaw {
                            match jaw_of(&p) {
                                Some(j) => for (a, b) in j.proportions.iter().zip(&held.proportions) { if (a - b).is_finite() { hold += ((a - b) / 1.5).powi(2); } },
                                None => hold += 1e6,
                            }
                        }
                        hold += ((m.pog[0] - live_chin.0[0]).powi(2) + (m.bottom - live_chin.1).powi(2)).sqrt().powi(2);
                        ((g / 0.5).powi(2) + hold, g, checker.faults(&p).total())
                    };
                    let search = |x0: Vec<f64>, free_jaw: bool| -> Vec<f64> {
                        let cost = |x: &[f64]| { let (c, _, f) = parts(x, true); c + 3.0 * f as f64 };
                        let mut x = x0;
                        let mut best = cost(&x);
                        let first = if free_jaw { 0 } else { n_jaw };
                        for _ in 0..2 {
                            for s in n_jaw..x.len() {
                                let (lo, hi) = ranges[s];
                                for k in 0..=8 {
                                    let mut y = x.clone();
                                    y[s] = lo + (hi - lo) * k as f64 / 8.0;
                                    let c = cost(&y);
                                    if c < best { best = c; x = y; }
                                }
                            }
                        }
                        for pass in 0..5 {
                            for s in first..x.len() {
                                let (lo, hi) = ranges[s];
                                let step = (hi - lo) / 8.0 / 2f64.powi(pass);
                                for dir in [-1.0, 1.0] {
                                    loop {
                                        let mut y = x.clone();
                                        y[s] = (x[s] + dir * step).clamp(lo, hi);
                                        if y[s] == x[s] { break; }
                                        let c = cost(&y);
                                        if c < best { best = c; x = y; } else { break; }
                                    }
                                }
                            }
                        }
                        x
                    };
                    // the guard: the other features within their marks of live, the jaw within its marks of live, and no turned triangle at
                    // the solved values, at each throat slider's ends and at each pair's four corners
                    let guard = |x: &[f64]| -> serde_json::Value {
                        let p = build(x);
                        let mut moved: Vec<String> = Vec::new();
                        for ((key, f, marks), old) in guard_features.iter().zip(&live_feats) {
                            let (Some(old), Some(new)) = (old, feature_of(*f, &p)) else { continue };
                            for (k, (a, b)) in new.iter().zip(old).enumerate() {
                                if (a - b).is_finite() && (a - b).abs() > marks[k] {
                                    moved.push(format!("{key} {} {:.1}", f.proportions()[k].key, a - b));
                                }
                            }
                        }
                        let mut folds = vec![("solved".to_string(), checker.faults(&p).total())];
                        for s in n_jaw..x.len() {
                            for (e, v) in [("low", ranges[s].0), ("high", ranges[s].1)] {
                                let mut y = x.to_vec();
                                y[s] = v;
                                folds.push((format!("{} {e}", THROAT_SLIDERS[s - n_jaw].0), checker.faults(&build(&y)).total()));
                            }
                            for t in s + 1..x.len() {
                                for (a, b) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                                    let mut y = x.to_vec();
                                    y[s] = if a == 0 { ranges[s].0 } else { ranges[s].1 };
                                    y[t] = if b == 0 { ranges[t].0 } else { ranges[t].1 };
                                    folds.push((format!("{} {} and {} {}", THROAT_SLIDERS[s - n_jaw].0, ["low", "high"][a], THROAT_SLIDERS[t - n_jaw].0, ["low", "high"][b]), checker.faults(&build(&y)).total()));
                                }
                            }
                        }
                        let folded: Vec<String> = folds.iter().filter(|f| f.1 > 0).map(|f| format!("{}: {}", f.0, f.1)).collect();
                        // a head passes on its own values; the ends and pair corners are a property of the ranges, reported apart
                        serde_json::json!({"pass": moved.is_empty() && folds[0].1 == 0, "moved": moved, "folds": folded})
                    };
                    let mut x0 = preset.clone();
                    x0.extend(Throat::start().to_vec());
                    let full = search(x0, true);
                    let g_full = guard(&full);
                    let (chosen, kept) = if g_full["pass"].as_bool().unwrap() {
                        (full.clone(), false)
                    } else {
                        let mut x1 = preset.clone();
                        x1.extend_from_slice(&full[n_jaw..]);
                        (search(x1, false), true)
                    };
                    let g = if kept { guard(&chosen) } else { g_full.clone() };
                    let (_, gap, _) = parts(&chosen, true);
                    eprintln!("{name}: live {:.2} new {gap:.2}{} {}", live_gap.unwrap_or(f64::NAN), if kept { " (jaw preset kept)" } else { "" }, if g["pass"].as_bool().unwrap() { "passes" } else { "FAILS" });
                    serde_json::json!({"name": name, "readable": true, "live_gap": live_gap, "gap": gap, "jaw": chosen[..n_jaw], "throat": chosen[n_jaw..], "kept_jaw_preset": kept, "full": {"values": full, "gap": parts(&full, true).1, "guard": g_full}, "guard": g})
                });
                // the heads whose outline cannot be read take the median of the solved throat values
                let solved: Vec<&serde_json::Value> = rows.iter().filter(|r| r["readable"].as_bool().unwrap()).collect();
                let median: Vec<f64> = (0..4).map(|k| { let mut v: Vec<f64> = solved.iter().map(|r| r["throat"][k].as_f64().unwrap()).collect(); v.sort_by(f64::total_cmp); v[v.len() / 2] }).collect();
                let mut values = serde_json::Map::new();
                for r in &rows {
                    let throat = if r["readable"].as_bool().unwrap() { r["throat"].clone() } else { serde_json::json!(median) };
                    values.insert(r["name"].as_str().unwrap().to_string(), serde_json::json!({"jaw": r["jaw"], "throat": throat}));
                }
                std::fs::write(out.join("values.json"), serde_json::to_string_pretty(&values).unwrap()).unwrap();
                std::fs::write(out.join("solve.json"), serde_json::to_string_pretty(&serde_json::json!({"median_throat": median, "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {}", out.display());
                true
            }
            // every head built with its solved jaw, throat and skull values, guarded, written as kit-order bins:
            // `final <data> <out dir> <throat values.json> <skull solve.json> <ref dir>`. The skull values of a head that fails its guard
            // are halved, then quartered, then dropped; the share kept is reported. The skull sliders' ranges are the fits' values,
            // and each end and pair corner is checked for turned triangles on every head.
            "final" => {
                use crate::bald::tools::{hair_faces, kotor, our_shape};
                use crate::chin_neck::Throat;
                let out = PathBuf::from(&args[4]);
                std::fs::create_dir_all(out.join("bins")).unwrap();
                let table: std::collections::HashMap<String, crate::chin_neck::tools::Built> = serde_json::from_str(&std::fs::read_to_string(&args[5]).unwrap()).unwrap();
                let skull: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[6]).unwrap()).unwrap();
                let refs = PathBuf::from(&args[7]);
                let hair = PathBuf::from(std::env::var("BALD_HAIR").expect("BALD_HAIR"));
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let mesh = &h.mesh;
                let region = crate::bald::tools::region(input, mesh);
                let shape = crate::chin_neck::ThroatShape::new(mesh);
                let fields = skull_world_fields(&h, input);
                let frame = crate::measure::Frame::head();
                let names = crate::bald::tools::names(input);
                let all: Vec<u32> = (0..mesh.base.len() as u32).collect();
                let tris = mesh.triangles.clone();
                let sv = |name: &str| -> Vec<f64> {
                    skull["heads"].as_array().unwrap().iter().find(|r| r["name"].as_str() == Some(name)).map(|r| r["values"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect()).unwrap_or(vec![0.0; 4])
                };
                // the skull sliders' ranges: the lowest and highest values the fits use, rounded out to the next unit
                let mut ranges = [(0.0f64, 0.0f64); 4];
                for n in &names {
                    for (k, v) in sv(n).iter().enumerate() {
                        ranges[k] = (ranges[k].0.min(v.floor()), ranges[k].1.max(v.ceil()));
                    }
                }
                let guard_features: Vec<(&str, &'static dyn crate::feature::Feature, Vec<f64>)> = [("nose", "marks.json"), ("brow", "brow/marks.json"), ("cheeks", "cheeks/marks.json"), ("jaw", "jaw/marks.json")]
                    .iter()
                    .map(|(k, f)| (*k, crate::feature::by_key(k).unwrap(), crate::checks::Marks::read(&data.join(f)).marks.proportions))
                    .collect();
                let feature_of = |f: &dyn crate::feature::Feature, p: &[V3]| f.measure(&crate::measure::Surface::within(p, &tris, &h.frame, &f.window())).ok().map(|m| m.proportions);
                let throat_solve: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(PathBuf::from(&args[5]).with_file_name("solve.json")).unwrap()).unwrap();
                let readable: Vec<String> = throat_solve["heads"].as_array().unwrap().iter().filter(|r| r["readable"].as_bool() == Some(true)).map(|r| r["name"].as_str().unwrap().to_string()).collect();
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let b = &table[name];
                    let jawed = h.live_with(name, b.jaw.as_deref());
                    let live0 = h.live_with(name, None);
                    let check0 = crate::check::Checker::from_parts(&tris, &vec![false; live0.len()], &live0, &all);
                    // a head whose outline cannot be read takes the median throat values; when they turn a triangle on it, the corner is
                    // rounded 4 mm at a time until none turns
                    let mut throat = b.throat.clone().unwrap();
                    if !readable.contains(name) && check0.faults(&shape.apply(&jawed, &Throat::from_slice(&throat))).total() > 0 {
                        let median = throat.clone();
                        let mut best: Option<(f64, Vec<f64>)> = None;
                        for dd in [-12.0, -6.0, 0.0, 6.0] {
                            for dh in [-6.0, -3.0, 0.0, 3.0] {
                                for r in [median[3], median[3] + 5.0, 25.0] {
                                    let t = vec![(median[0] + dd).clamp(45.0, 84.0), (median[1] + dh).clamp(-12.0, 12.0), median[2], r.min(25.0)];
                                    let d = dd.abs() / 6.0 + dh.abs() / 3.0 + (r - median[3]) / 5.0;
                                    if best.as_ref().is_some_and(|b| b.0 <= d) {
                                        continue;
                                    }
                                    if check0.faults(&shape.apply(&jawed, &Throat::from_slice(&t))).total() == 0 {
                                        best = Some((d, t));
                                    }
                                }
                            }
                        }
                        if let Some((_, t)) = best {
                            throat = t;
                        }
                    }
                    let throat = throat;
                    let with_skull = |v: &[f64]| -> Vec<V3> {
                        let mut p = jawed.clone();
                        for (f, &x) in fields.iter().zip(v) {
                            for (a, d) in p.iter_mut().zip(f) {
                                *a = crate::add(*a, crate::scale(*d, x));
                            }
                        }
                        shape.apply(&p, &Throat::from_slice(&throat))
                    };
                    let no_skull = with_skull(&[0.0; 4]);
                    let live = h.live_with(name, None);
                    let checker = crate::check::Checker::from_parts(&tris, &vec![false; live.len()], &live, &all);
                    let base_feats: Vec<Option<Vec<f64>>> = guard_features.iter().map(|(_, f, _)| feature_of(*f, &no_skull)).collect();
                    let k = kotor(input, &hair, name);
                    let soup = crate::bald::Soup::new(k.f.iter().map(|t| t.map(|j| k.v[j as usize])).collect());
                    let haired = refs.join(format!("{name}_ref.bin")).exists() && !crate::bald::SKULL_SHOWS.contains(&name.as_str());
                    let _ = hair_faces;
                    // the skull standing outside KOTOR's head with its hair: skin above the brows whose outward ray meets nothing of KOTOR
                    // while its inward ray meets KOTOR within 30 mm
                    let outside = |p: &[V3]| -> usize {
                        let l: Vec<V3> = p.iter().map(|&a| frame.local(a)).collect();
                        let merged = mesh.merge(&l);
                        let normals = crate::bald::merged_normals(mesh, &merged);
                        (0..merged.len())
                            .filter(|&g| region[g] && merged[g][1] > 40.0)
                            .filter(|&g| soup.first(merged[g], normals[g], 120.0).is_none() && soup.first(merged[g], crate::scale(normals[g], -1.0), 30.0).is_some_and(|t| t > 2.0))
                            .count()
                    };
                    let outside_live = if haired { outside(&live) } else { 0 };
                    let guard = |v: &[f64]| -> (bool, Vec<String>) {
                        let p = with_skull(v);
                        let mut why = Vec::new();
                        for ((key, f, marks), old) in guard_features.iter().zip(&base_feats) {
                            let (Some(old), Some(new)) = (old, feature_of(*f, &p)) else { continue };
                            for (i, (a, c)) in new.iter().zip(old).enumerate() {
                                // Skull width narrows the temples on purpose: the brow feature's temple width is not guarded here
                                if f.proportions()[i].key == "temple_width" {
                                    continue;
                                }
                                if (a - c).is_finite() && (a - c).abs() > marks[i] {
                                    why.push(format!("{key} {} {:.1}", f.proportions()[i].key, a - c));
                                }
                            }
                        }
                        let folds = checker.faults(&p).total();
                        if folds > 0 {
                            let marks = checker.marks(&p);
                            let at: Vec<String> = tris.iter().zip(&marks).filter(|(_, m)| **m).take(3).map(|(t, _)| {
                                let c = frame.local(p[t[0] as usize]);
                                format!("({:.0} {:.0} {:.0})", c[0], c[1], c[2])
                            }).collect();
                            why.push(format!("{folds} turned triangles at {}", at.join(" ")));
                        }
                        if haired {
                            let o = outside(&p);
                            if o > outside_live {
                                why.push(format!("{o} places outside KOTOR's hair, live {outside_live}"));
                            }
                        }
                        (why.is_empty(), why)
                    };
                    let full = sv(name);
                    let mut share = 0.0;
                    let mut why_full = Vec::new();
                    for (i, f) in [1.0, 0.5, 0.25].iter().enumerate() {
                        let v: Vec<f64> = full.iter().map(|x| x * f).collect();
                        let (ok, why) = guard(&v);
                        if i == 0 {
                            why_full = why;
                        }
                        if ok {
                            share = *f;
                            break;
                        }
                    }
                    let v: Vec<f64> = full.iter().map(|x| x * share).collect();
                    let p = with_skull(&v);
                    crate::chin_neck::tools::write_bin(&out.join("bins"), name, &p);
                    // the skull sliders' ends and pair corners, the other skull values held
                    let mut ends = Vec::new();
                    for s in 0..4 {
                        for (e, x) in [("low", ranges[s].0), ("high", ranges[s].1)] {
                            let mut y = v.clone();
                            y[s] = x;
                            let n = checker.faults(&with_skull(&y)).total();
                            if n > 0 { ends.push(format!("{} {e}: {n}", SKULL_SLIDERS[s].0)); }
                        }
                        for t in s + 1..4 {
                            for (a, c) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                                let mut y = v.clone();
                                y[s] = if a == 0 { ranges[s].0 } else { ranges[s].1 };
                                y[t] = if c == 0 { ranges[t].0 } else { ranges[t].1 };
                                let n = checker.faults(&with_skull(&y)).total();
                                if n > 0 { ends.push(format!("{} {} and {} {}: {n}", SKULL_SLIDERS[s].0, ["low", "high"][a], SKULL_SLIDERS[t].0, ["low", "high"][c])); }
                            }
                        }
                    }
                    let _ = our_shape;
                    eprintln!("{name}: skull share {share}{}", if why_full.is_empty() { String::new() } else { format!(" ({})", why_full.join(", ")) });
                    serde_json::json!({"name": name, "jaw": b.jaw, "throat": throat, "skull_full": full, "skull": v, "share": share, "why_full_failed": why_full, "haired": haired, "outside_live": outside_live, "outside_new": if haired { outside(&p) } else { 0 }, "skull_ends": ends})
                });
                let r: Vec<serde_json::Value> = ranges.iter().zip(SKULL_SLIDERS).map(|(r, s)| serde_json::json!({"name": s.0, "low": r.0, "high": r.1})).collect();
                std::fs::write(out.join("final.json"), serde_json::to_string_pretty(&serde_json::json!({"skull_ranges": r, "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {}", out.display());
                true
            }
            // the forehead-to-skull profile on every head: KOTOR's (the 36 heads whose skull shows), KOTOR subdivided twice, the bald
            // reference (the 21 haired heads), live and the heads of a folder of kit-order bins, with the whole-head rows:
            // `skull-check <data> <out.json> <ref dir> [label=<bins dir>]...`
            "skull-check" => {
                use crate::bald::tools::{hair_faces, kotor, kotor_shape, load_reference, our_shape, reference_shape};
                let out = args[4].clone();
                let refs = PathBuf::from(&args[5]);
                let variants: Vec<(String, PathBuf)> = args[6..].iter().map(|v| { let (a, b) = v.split_once('=').unwrap(); (a.to_string(), PathBuf::from(b)) }).collect();
                let hair = PathBuf::from(std::env::var("BALD_HAIR").expect("BALD_HAIR"));
                let h = crate::jaw_edge::tools::Heads::load(input, data);
                let mesh = &h.mesh;
                let region = crate::bald::tools::region(input, mesh);
                let frame = crate::measure::Frame::head();
                let names = crate::bald::tools::names(input);
                let rows: Vec<serde_json::Value> = parallel(names.len(), threads(), |q| {
                    let name = &names[q];
                    let k = kotor(input, &hair, name);
                    let hf = hair_faces(input, name, &k);
                    let ks = kotor_shape(&k, &hf);
                    let bare: Vec<[u32; 3]> = k.f.iter().zip(&hf).filter(|(_, h)| !**h).map(|(t, _)| *t).collect();
                    let (sv, sf, _) = crate::checks::butterfly(&k.v, &bare);
                    let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
                    let ss = crate::bald::Shape { tris: sf.iter().map(|t| t.map(|i| sv[i as usize])).collect(), filled: vec![false; sf.len()] };
                    let has_ref = refs.join(format!("{name}_ref.bin")).exists();
                    let rs = has_ref.then(|| reference_shape(&k, &hf, &load_reference(&refs, name, mesh, &region), mesh));
                    let live: Vec<V3> = h.live_with(name, None).iter().map(|&p| frame.local(p)).collect();
                    let mut row = serde_json::json!({
                        "name": name,
                        "skull_shows": crate::bald::SKULL_SHOWS.contains(&name.as_str()),
                        "kotor": skull_profile(&ks),
                        "kotor_sub": skull_profile(&ss),
                        "reference": rs.as_ref().map(skull_profile),
                        "live": skull_profile(&our_shape(mesh, &live)),
                        "rows_kotor": crate::bald::measure_head(&ks).iter().map(|r| r.0).collect::<Vec<_>>(),
                        "rows_reference": rs.as_ref().map(|r| crate::bald::measure_head(r).iter().map(|r| r.0).collect::<Vec<_>>()),
                        "rows_live": crate::bald::measure_head(&our_shape(mesh, &live)).iter().map(|r| r.0).collect::<Vec<_>>(),
                        "brow_slope_kotor": brow_style_slope(&ks),
                        "brow_slope_reference": rs.as_ref().map(brow_style_slope),
                        "brow_slope_live": brow_style_slope(&our_shape(mesh, &live)),
                    });
                    for (label, dir) in &variants {
                        if let Some(p) = crate::chin_neck::tools::read_bin(dir, name) {
                            let sh = our_shape(mesh, &p);
                            row[label] = serde_json::json!(skull_profile(&sh));
                            row[format!("rows_{label}")] = serde_json::json!(crate::bald::measure_head(&sh).iter().map(|r| r.0).collect::<Vec<_>>());
                            row[format!("brow_slope_{label}")] = serde_json::json!(brow_style_slope(&sh));
                        }
                    }
                    row
                });
                let rows_keys: Vec<&str> = crate::bald::ROWS.iter().map(|r| r.0).collect();
                std::fs::write(&out, serde_json::to_string(&serde_json::json!({"rows": rows_keys, "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }
            _ => false,
        }
    }
}
