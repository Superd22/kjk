//! Builds hairstyles from style files and writes them into the head viewer's data (roadmap part D, first milestone).
//!
//! For each style it reads the head named in the style from `<viewer data>/common.json` and `<viewer data>/heads/<name>.json`, builds the
//! hair, and writes:
//! - `ourhair/<name>.json`: the cap and the cards as parts in the format of `hair/<name>.json`, with normals and `"alpha": true`, the
//!   painted layer's file, and the numbers (timings, triangles, KOTOR's triangles);
//! - `tex/ourhair/<name>_hair.png`: the hair's atlas, with alpha;
//! - `tex/ourhair/<name>_paint.png`: the painted hair layer in the head's texture layout, with alpha, to put over any of our paints;
//! - `tex/ourhair/<name>_skin.jpg`: that layer already put over the default layered paint with every layer on;
//! - `ourhair/debug/<name>.json`: some strands and the placed curves, as lines, for checking pictures;
//! - `ourhair.json`: the list of heads that have our hair, updated.
//!
//! Options, before the paths:
//! - `--cells N` builds every style on a grid of N cells instead of the style's own;
//! - `--bench` also moves the first point after the start of the first stream curve by 3 degrees of azimuth, and times a re-solve that
//!   starts from the previous field against one that starts from zero. The numbers go in the head's `stats.bench`.
//!
//! Usage: make_hair [--cells N] [--bench] <viewer data dir> <style.json> [<style.json> ...]

use base64::Engine;
use head_hair::cards::Mesh;
use head_hair::curves::PlacedCurves;
use head_hair::field_grid;
use head_hair::paint::{PixelPoint, composite, paint_layer};
use head_hair::style::Style;
use head_hair::v3::{V3, dot, normalize, sub};
use head_paint::closest::Bvh;
use head_paint::mesh::vertex_normals;
use head_paint::pixel_map::{PixelMap, nearest_true};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use serde_json::{Value, json};
use std::path::Path;
use std::time::Instant;

const PAINT_RES: usize = 2048;
const SEAM_GROWTH: f64 = 8.0;

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn unb64(v: &Value) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD.decode(v.as_str().expect("base64 string")).expect("base64")
}

fn f32s(v: &Value) -> Vec<f32> {
    unb64(v).as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect()
}

fn u16s(v: &Value) -> Vec<u16> {
    unb64(v).as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect()
}

fn f32_b64(values: impl Iterator<Item = f64>) -> String {
    let bytes: Vec<u8> = values.flat_map(|x| (x as f32).to_le_bytes()).collect();
    b64(&bytes)
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A mesh as a part of the viewer's hair format: every triangle's three corners, unshared.
fn part(mesh: &Mesh, material: &str, texture: &str) -> Value {
    let corners: Vec<u32> = mesh.triangles.iter().flatten().copied().collect();
    json!({
        "material": material,
        "source": "ours",
        "triangles": mesh.triangles.len(),
        "pos": f32_b64(corners.iter().flat_map(|&i| mesh.positions[i as usize])),
        "uv": f32_b64(corners.iter().flat_map(|&i| mesh.uv[i as usize])),
        "normal": f32_b64(corners.iter().flat_map(|&i| mesh.normals[i as usize])),
        "texture": texture,
        "alpha": true,
    })
}

fn write_png(path: &Path, rgba: &[u8], w: usize, h: usize) {
    let file = std::fs::File::create(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    PngEncoder::new(std::io::BufWriter::new(file)).write_image(rgba, w as u32, h as u32, ExtendedColorType::Rgba8).expect("png");
}

fn write_jpeg(path: &Path, rgb: &[u8], size: usize) {
    let file = std::fs::File::create(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    JpegEncoder::new_with_quality(std::io::BufWriter::new(file), 90).write_image(rgb, size as u32, size as u32, ExtendedColorType::Rgb8).expect("jpg");
}

/// Copies every pixel of the layer that no triangle covers from the nearest covered pixel within `SEAM_GROWTH`, so that the texture's
/// filtering across a patch edge shows no line.
fn grow_layer(layer: &mut head_hair::atlas::Image, covered: &[bool]) {
    let res = layer.width;
    let nearest = nearest_true(covered, res, res);
    for at in 0..res * res {
        let s = nearest[at];
        if covered[at] || s == u32::MAX {
            continue;
        }
        let s = s as usize;
        let (dy, dx) = ((at / res) as f64 - (s / res) as f64, (at % res) as f64 - (s % res) as f64);
        if (dx * dx + dy * dy).sqrt() <= SEAM_GROWTH {
            layer.px[at] = layer.px[s];
        }
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut cells: Option<usize> = None;
    let mut bench = false;
    while let Some(first) = args.first().cloned() {
        match first.as_str() {
            "--cells" => {
                cells = args.get(1).and_then(|n| n.parse().ok());
                args.drain(..2.min(args.len()));
            }
            "--bench" => {
                bench = true;
                args.remove(0);
            }
            _ => break,
        }
    }
    if args.len() < 2 {
        eprintln!("Usage: make_hair [--cells N] [--bench] <viewer data dir> <style.json> [<style.json> ...]");
        std::process::exit(2);
    }
    let data = Path::new(&args[0]);
    let common = read_json(&data.join("common.json"));
    let n = common["n"].as_u64().expect("n") as usize;
    let tris: Vec<[u32; 3]> = u16s(&common["tris"]).as_chunks::<3>().0.iter().map(|t| t.map(u32::from)).collect();
    let edges: Vec<[usize; 2]> = u16s(&common["edges"]).as_chunks::<2>().0.iter().map(|e| e.map(usize::from)).collect();
    let uv: Vec<[f64; 2]> = f32s(&common["uv"]).as_chunks::<2>().0.iter().map(|p| p.map(f64::from)).collect();
    let t = Instant::now();
    let map = PixelMap::new(&uv, &tris, PAINT_RES);
    let mut covered = vec![false; PAINT_RES * PAINT_RES];
    for p in &map.pixels {
        covered[p.y as usize * PAINT_RES + p.x as usize] = true;
    }
    println!("pixel map of the head's layout: {:.2} s, {} pixels", t.elapsed().as_secs_f64(), map.pixels.len());
    std::fs::create_dir_all(data.join("ourhair/debug")).expect("dir");
    std::fs::create_dir_all(data.join("tex/ourhair")).expect("dir");
    let listing_path = data.join("ourhair.json");
    let mut listing = if listing_path.exists() { read_json(&listing_path) } else { json!({ "heads": {} }) };

    for style_path in &args[1..] {
        let mut style = Style::from_json(&std::fs::read_to_string(style_path).unwrap_or_else(|e| panic!("{style_path}: {e}"))).unwrap_or_else(|e| panic!("{style_path}: {e}"));
        if let Some(n) = cells {
            style.grid.cells = n;
        }
        let name = style.head.clone();
        let head = read_json(&data.join("heads").join(format!("{name}.json")));
        let base: Vec<V3> = f32s(&head["ours"]["pos"]).as_chunks::<3>().0.iter().map(|p| p.map(f64::from)).collect();
        assert_eq!(base.len(), n, "{name}: the head has another vertex count than common.json");
        let mut positions = base.clone();
        positions.extend(edges.iter().map(|e| [0, 1, 2].map(|k| (base[e[0]][k] + base[e[1]][k]) / 2.0)));
        let eyes: Vec<f32> = f32s(&head["ours"]["eyes"]["pos"]);
        let eye_y = eyes.chunks(3).map(|p| f64::from(p[1])).sum::<f64>() / (eyes.len() / 3) as f64;

        let started = Instant::now();
        let hair = head_hair::build(&style, &positions, &tris, eye_y);
        let build_seconds = started.elapsed().as_secs_f64();

        let t = Instant::now();
        let normals = vertex_normals(&positions, &tris);
        let at_pixels = map.interpolate(&tris, &positions);
        let n_at_pixels = map.interpolate(&tris, &normals);
        let pixels: Vec<PixelPoint> = map.pixels.iter().zip(at_pixels.iter().zip(&n_at_pixels)).map(|(p, (pos, nrm))| PixelPoint { x: p.x, y: p.y, position: *pos, normal: normalize(*nrm) }).collect();
        let expanded = style.expanded();
        let bench_stats = if bench {
            let mut moved = expanded.clone();
            if let Some(s) = moved.streams.first_mut()
                && s.points.len() > 1
            {
                s.points[1][0] += 3.0;
            }
            let grid = field_grid(&moved, &hair.shell.frame);
            let placed = PlacedCurves::new(&moved, &hair.shell, grid.cell);
            let t = Instant::now();
            let mut constraints = placed.constraints(&grid, &hair.shell);
            constraints.extend(head_hair::curves::tangent_rule(&hair.field, &hair.shell));
            let setup_ms = t.elapsed().as_secs_f64() * 1000.0;
            let (_, warm) = head_hair::field::solve(&grid, &constraints, Some(hair.field.values.clone()), head_hair::TOLERANCE, 4000);
            let (_, cold) = head_hair::field::solve(&grid, &constraints, None, head_hair::TOLERANCE, 4000);
            json!({
                "moved": "the second point of the first stream curve, by 3 degrees of azimuth",
                "constraints_ms": setup_ms,
                "warm_start": { "iterations": warm.iterations, "ms": warm.seconds * 1000.0 },
                "cold_start": { "iterations": cold.iterations, "ms": cold.seconds * 1000.0 },
            })
        } else {
            Value::Null
        };
        let mut layer = paint_layer(&hair.shell, &hair.field, &expanded.paint, Some(&hair.outline), &expanded.colour.painted, &pixels, PAINT_RES, style.seed);
        grow_layer(&mut layer, &covered);
        let paint_seconds = t.elapsed().as_secs_f64();
        write_png(&data.join(format!("tex/ourhair/{name}_paint.png")), &layer.rgba8(), PAINT_RES, PAINT_RES);
        let skin_from = format!("ours/{name}_layered_111.jpg");
        let skin_path = data.join("tex").join(&skin_from);
        let painted_skin = if let Ok(img) = image::open(&skin_path) {
            let mut rgb = img.to_rgb8().into_raw();
            composite(&mut rgb, &layer);
            write_jpeg(&data.join(format!("tex/ourhair/{name}_skin.jpg")), &rgb, PAINT_RES);
            Some(format!("ourhair/{name}_skin.jpg"))
        } else {
            None
        };
        let a = &hair.atlas;
        write_png(&data.join(format!("tex/ourhair/{name}_hair.png")), &a.rgba8_cut(0.5), a.width, a.height);

        // How many strand points lie inside the head, by the exact closest point and the side of its triangle.
        let bvh = Bvh::new(positions.clone(), tris.clone());
        let (mut inside, mut total, mut inside_lock) = (0usize, 0usize, 0usize);
        let mut inside_where: Vec<[i64; 3]> = Vec::new();
        for s in &hair.strands {
            for &p in s.points.iter().step_by(3) {
                total += 1;
                if let Some(c) = bvh.closest(p) {
                    let [a, b, cc] = tris[c.triangle as usize].map(|i| positions[i as usize]);
                    let nrm = head_hair::v3::cross(sub(b, a), sub(cc, a));
                    if dot(sub(p, c.point), nrm) < 0.0 && c.distance > 1e-5 {
                        inside += 1;
                        inside_lock += usize::from(s.lock.is_some());
                        if inside_where.len() < 12 {
                            let (az, el) = head_hair::frame::angles(hair.shell.dir(p));
                            inside_where.push([az.round() as i64, el.round() as i64, (c.distance * 1e4).round() as i64]);
                        }
                    }
                }
            }
        }

        let kotor = read_json(&data.join("hair").join(format!("{name}.json")));
        let kotor_parts = kotor["parts"].as_array().cloned().unwrap_or(Vec::new());
        let kotor_object: u64 = kotor_parts.iter().filter(|p| p["source"] == "object").map(|p| p["triangles"].as_u64().unwrap_or(0)).sum();
        let kotor_painted: u64 = kotor_parts.iter().filter(|p| p["source"] == "painted").map(|p| p["triangles"].as_u64().unwrap_or(0)).sum();
        let tm = &hair.timings;
        let solves: Vec<Value> = tm.solves.iter().map(|s| json!({ "unknowns": s.unknowns, "iterations": s.iterations, "residual": s.residual, "ms": s.seconds * 1000.0 })).collect();
        let locks: Vec<usize> = (0..hair.locks.len()).map(|k| hair.strands.iter().filter(|s| s.lock == Some(k)).count()).collect();
        let grid = field_grid(&expanded, &hair.shell.frame);
        let stats = json!({
            "frame": { "centre": hair.shell.frame.centre, "radius_mm": hair.shell.frame.radius * 1000.0 },
            "grid": { "dims": grid.dims, "cell_mm": grid.cell * 1000.0 },
            "ms": {
                "depth_table": tm.depth * 1000.0, "constraints": tm.constraints * 1000.0, "field_total": tm.field_total * 1000.0,
                "strands": tm.strands * 1000.0, "cap_and_cards": tm.cap_and_cards * 1000.0, "atlas": tm.atlas * 1000.0,
                "build_total": build_seconds * 1000.0, "painted_layer": paint_seconds * 1000.0,
            },
            "solves": solves,
            "bench": bench_stats,
            "strands": hair.strands.len(),
            "lock_members": locks,
            "strand_points_checked": total,
            "strand_points_inside_head": inside,
            "of_which_on_locks": inside_lock,
            "first_inside_points_az_el_depth_tenths_of_mm": inside_where,
            "triangles": { "cap": hair.cap.triangles.len(), "cards": hair.cards.triangles.len(), "card_count": hair.card_count, "crossed_cards": hair.crossed_count,
                           "total": hair.cap.triangles.len() + hair.cards.triangles.len() },
            "kotor_triangles": { "hair_objects": kotor_object, "face_mesh_triangles_painted_as_hair": kotor_painted },
        });
        let texture = format!("ourhair/{name}_hair.png");
        let mut parts = Vec::new();
        if !hair.cap.triangles.is_empty() {
            parts.push(part(&hair.cap, "cap", &texture));
        }
        if !hair.cards.triangles.is_empty() {
            parts.push(part(&hair.cards, "cards", &texture));
        }
        let out = json!({
            "name": name,
            "style": style.name,
            "note": style.note,
            "parts": parts,
            "paintLayer": format!("ourhair/{name}_paint.png"),
            "paintedSkin": painted_skin,
            "paintedSkinFrom": skin_from,
            "stats": stats,
        });
        std::fs::write(data.join(format!("ourhair/{name}.json")), serde_json::to_string(&out).expect("json")).expect("write");

        // Lines for checking pictures: every fourth strand, and the placed curves.
        let placed = PlacedCurves::new(&expanded, &hair.shell, grid.cell);
        let mut lines: Vec<Value> = Vec::new();
        for s in hair.strands.iter().step_by(4) {
            lines.push(json!({ "kind": if s.lock.is_some() { "lock" } else { "strand" }, "pos": f32_b64(s.points.iter().step_by(2).flatten().copied()) }));
        }
        for (pts, _, lock) in &placed.streams {
            lines.push(json!({ "kind": if lock.is_some() { "lock_curve" } else { "stream" }, "pos": f32_b64(pts.iter().flatten().copied()) }));
        }
        for pts in &placed.partings {
            lines.push(json!({ "kind": "parting", "pos": f32_b64(pts.iter().flatten().copied()) }));
        }
        for (spine, ..) in &placed.ponytails {
            lines.push(json!({ "kind": "ponytail", "pos": f32_b64(spine.iter().flatten().copied()) }));
        }
        std::fs::write(data.join(format!("ourhair/debug/{name}.json")), serde_json::to_string(&json!({ "lines": lines })).expect("json")).expect("write");

        listing["heads"][&name] = json!({
            "file": format!("ourhair/{name}.json"),
            "label": style.name,
            "triangles": hair.cap.triangles.len() + hair.cards.triangles.len(),
            "kotorTriangles": kotor_object,
        });
        println!("{name}: {}", serde_json::to_string_pretty(&out["stats"]).expect("json"));
    }
    std::fs::write(&listing_path, serde_json::to_string_pretty(&listing).expect("json")).expect("write ourhair.json");
}
