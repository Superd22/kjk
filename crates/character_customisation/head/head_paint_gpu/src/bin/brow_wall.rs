//! The brows before any paint (paint track, part C, the brows). Analysis only.
//!
//! `brow_wall density <out json> <name>...` measures how many millimetres of skin one pixel of the 2,048-pixel texture covers along each
//! brow, from its inner end to its outer end, on our fitted heads.
//!
//! `brow_wall wall <out dir> <name>...` writes, for each head, KOTOR's own head seen from the front around the brows (with and without its
//! hair meshes), and a few numbers read on it: how dark and how thick the brow is, and whether a hair mesh covers it. The numbers go to
//! `<out dir>/wall.json`.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::KotorFront;
use head_paint::folds::{Anchors, Landmarks};
use head_paint::lines::BROW_ANCHORS;
use std::path::{Path, PathBuf};

const LIVE: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";

fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().expect("8"))).collect()
}
fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().expect("4"))).collect()
}
fn positions(name: &str) -> Vec<[f64; 3]> {
    f64s(&Path::new(LIVE).join("heads").join(format!("{name}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect()
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [0, 1, 2].map(|k| a[k] + t * (b[k] - a[k]))
}

/// The brow's three anchors of one side, in millimetres: inner end, highest point, outer end.
fn brow(pos: &[[f64; 3]], side: usize) -> [[f64; 3]; 3] {
    let b = &BROW_ANCHORS;
    [b.brow_inner[side], b.brow_peak[side], b.brow_outer[side]].map(|a| Anchors::point(&a, pos))
}

/// A point along the brow, from 0 at the inner end to 1 at the outer end, on the quadratic through the three anchors with the peak at
/// the share of the chord where it lies.
fn along(p: &[[f64; 3]; 3], t: f64) -> [f64; 3] {
    let s = (len(sub(p[1], p[0])) / (len(sub(p[1], p[0])) + len(sub(p[2], p[1])))).clamp(0.2, 0.8);
    // Lagrange through (0, p0), (s, p1), (1, p2).
    let l0 = (t - s) * (t - 1.0) / (s * 1.0);
    let l1 = t * (t - 1.0) / (s * (s - 1.0));
    let l2 = t * (t - s) / (1.0 - s);
    [0, 1, 2].map(|k| l0 * p[0][k] + l1 * p[1][k] + l2 * p[2][k])
}

fn density(out: &Path, names: &[String]) {
    let l = Path::new(LIVE).join("layout");
    let uv: Vec<[f64; 2]> = f64s(&l.join("uv.f64")).chunks_exact(2).map(|p| [p[0], p[1]]).collect();
    let tris: Vec<[u32; 3]> = u32s(&l.join("tris.u32")).chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect();
    let vmap: Vec<usize> = u32s(&l.join("vmap.u32")).into_iter().map(|i| i as usize).collect();
    let n = 2048.0;
    let mut report = serde_json::Map::new();
    for name in names {
        let pos = positions(name);
        let p3: Vec<[f64; 3]> = vmap.iter().map(|&m| pos[m].map(|x| 1000.0 * x)).collect();
        let mut sides = Vec::new();
        for side in 0..2 {
            let b = brow(&pos, side);
            let mut rows = Vec::new();
            for i in 0..=10 {
                let t = i as f64 / 10.0;
                let c = along(&b, t);
                let tangent = {
                    let d = sub(along(&b, (t + 0.02).min(1.0)), along(&b, (t - 0.02).max(0.0)));
                    let l = len(d);
                    d.map(|x| x / l)
                };
                // Every layout triangle whose middle lies within 3 mm of the point: its pixels per millimetre along the brow and across it.
                let mut along_px = Vec::new();
                let mut across_px = Vec::new();
                let mut patches = std::collections::BTreeSet::new();
                for (ti, t3) in tris.iter().enumerate() {
                    let v = t3.map(|i| p3[i as usize]);
                    let m = [0, 1, 2].map(|k| (v[0][k] + v[1][k] + v[2][k]) / 3.0);
                    if len(sub(m, c)) > 3.0 {
                        continue;
                    }
                    let u = t3.map(|i| uv[i as usize]);
                    let (e1, e2) = (sub(v[1], v[0]), sub(v[2], v[0]));
                    let (d1, d2) = ([(u[1][0] - u[0][0]) * n, (u[1][1] - u[0][1]) * n], [(u[2][0] - u[0][0]) * n, (u[2][1] - u[0][1]) * n]);
                    // Write the direction as a e1 + b e2 in the triangle's plane, then map it to pixels as a d1 + b d2.
                    let g = [[dot(e1, e1), dot(e1, e2)], [dot(e1, e2), dot(e2, e2)]];
                    let det = g[0][0] * g[1][1] - g[0][1] * g[1][0];
                    if det.abs() < 1e-12 {
                        continue;
                    }
                    let px = |dir: [f64; 3]| {
                        let r = [dot(dir, e1), dot(dir, e2)];
                        let a = (g[1][1] * r[0] - g[0][1] * r[1]) / det;
                        let bb = (-g[1][0] * r[0] + g[0][0] * r[1]) / det;
                        let inplane = [0, 1, 2].map(|k| a * e1[k] + bb * e2[k]);
                        let mm = len(inplane).max(1e-9);
                        let pix = ((a * d1[0] + bb * d2[0]).powi(2) + (a * d1[1] + bb * d2[1]).powi(2)).sqrt();
                        pix / mm
                    };
                    let normal = {
                        let c = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                        let l = len(c);
                        c.map(|x| x / l)
                    };
                    let across = [tangent[1] * normal[2] - tangent[2] * normal[1], tangent[2] * normal[0] - tangent[0] * normal[2], tangent[0] * normal[1] - tangent[1] * normal[0]];
                    along_px.push(px(tangent));
                    across_px.push(px(across));
                    patches.insert((u[0][0] * 8.0) as i32 * 100 + (u[0][1] * 8.0) as i32);
                }
                let med = |v: &mut Vec<f64>| {
                    v.sort_by(f64::total_cmp);
                    if v.is_empty() { f64::NAN } else { v[v.len() / 2] }
                };
                let (a, x) = (med(&mut along_px), med(&mut across_px));
                rows.push(serde_json::json!({ "t": t, "px_per_mm_along": (a * 100.0).round() / 100.0, "px_per_mm_across": (x * 100.0).round() / 100.0, "mm_per_px_along": (100.0 / a).round() / 100.0, "mm_per_px_across": (100.0 / x).round() / 100.0, "triangles": along_px.len() }));
            }
            let length = (0..20).map(|i| len(sub(along(&b, (i + 1) as f64 / 20.0), along(&b, i as f64 / 20.0)))).sum::<f64>();
            sides.push(serde_json::json!({ "length_mm": (length * 10.0).round() / 10.0, "rows": rows }));
        }
        println!("{name}: done");
        report.insert(name.clone(), serde_json::Value::Array(sides));
    }
    std::fs::write(out, serde_json::to_string_pretty(&report).unwrap()).expect("write");
}

fn lstar(c: [u8; 3]) -> f64 {
    head_paint::colour::srgb_to_lab(c.map(|v| f64::from(v) / 255.0))[0]
}

fn wall(out: &Path, names: &[String]) {
    std::fs::create_dir_all(out).expect("out");
    let mut report = serde_json::Map::new();
    for name in names {
        let pos = positions(name);
        let marks = Landmarks::new(&pos);
        let f = marks.frame;
        let k = KotorFront::open(name, f.across, f.up, f.forward);
        let kh = KotorFront::open_with_hair(name, f.across, f.up, f.forward);
        let brows = [brow(&pos, 0), brow(&pos, 1)];
        let mid = Anchors::point(&BROW_ANCHORS.brow_middle, &pos);
        let centre = k.project(lerp(mid, lerp(brows[0][1], brows[1][1], 0.5), 0.5)).0;
        let c = [centre[0], centre[1] - 2.0];
        kh.tile_at(c, 130.0, 50.0, 6.0).save(out.join(format!("{name}_hair.png"))).expect("save");
        let mut plain = k.tile_at(c, 130.0, 50.0, 6.0);
        plain.save(out.join(format!("{name}.png"))).expect("save");
        // The anchors' curve, every 2% of the brow, and the three anchors as crosses.
        let to_px = |p: [f64; 3]| {
            let q = k.project(p).0;
            ((q[0] - (c[0] - 65.0)) * 6.0, ((c[1] + 25.0) - q[1]) * 6.0)
        };
        for side in 0..2 {
            for i in 0..=50 {
                let (x, y) = to_px(along(&brows[side], i as f64 / 50.0));
                if x >= 0.0 && y >= 0.0 && (x as u32) < plain.width() && (y as u32) < plain.height() {
                    plain.put_pixel(x as u32, y as u32, image::Rgb([0, 255, 255]));
                }
            }
            for a in brows[side] {
                let (x, y) = to_px(a);
                for d in -3i32..=3 {
                    for (xx, yy) in [(x as i32 + d, y as i32), (x as i32, y as i32 + d)] {
                        if xx >= 0 && yy >= 0 && (xx as u32) < plain.width() && (yy as u32) < plain.height() {
                            plain.put_pixel(xx as u32, yy as u32, image::Rgb([255, 0, 255]));
                        }
                    }
                }
            }
        }
        plain.save(out.join(format!("{name}_marked.png"))).expect("save");
        // Columns across each brow: KOTOR's lightness from 10 mm below the anchors' curve to 16 mm above it, every 0.25 mm.
        let mut sides = Vec::new();
        for side in 0..2 {
            let mut cols = Vec::new();
            let mut covered = 0;
            for i in 0..=12 {
                let t = -0.05 + 1.1 * i as f64 / 12.0;
                let p = along(&brows[side], t);
                let q = k.project(p).0;
                let ls: Vec<Option<f64>> = (0..=72).map(|j| kh.front_rgb([q[0], q[1] - 3.0 + 0.25 * j as f64]).map(lstar)).collect();
                if let (Some(a), Some(b)) = (k.front_point(q), kh.front_point(q)) {
                    if dot(sub(b, a), f.forward) > 1.0 {
                        covered += 1;
                    }
                }
                let skin: Vec<f64> = ls[60..].iter().flatten().copied().collect();
                if skin.len() < 6 {
                    continue;
                }
                let mut s = skin.clone();
                s.sort_by(f64::total_cmp);
                let skin_l = s[s.len() / 2];
                let (jmin, lmin) = ls.iter().enumerate().filter_map(|(j, l)| l.map(|l| (j, l))).min_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
                let dark = skin_l - lmin;
                let half = skin_l - 0.5 * dark;
                let mut lo = jmin;
                while lo > 0 && ls[lo - 1].is_some_and(|l| l < half) {
                    lo -= 1;
                }
                let mut hi = jmin;
                while hi < 72 && ls[hi + 1].is_some_and(|l| l < half) {
                    hi += 1;
                }
                cols.push(serde_json::json!({ "t": (t * 100.0).round() / 100.0, "skin_l": (skin_l * 10.0).round() / 10.0, "dark": (dark * 10.0).round() / 10.0, "mid_mm": -3.0 + 0.25 * (lo + hi) as f64 / 2.0, "thick_mm": 0.25 * (hi - lo + 1) as f64 }));
            }
            sides.push(serde_json::json!({ "cols": cols, "hair_covered_cols": covered }));
        }
        println!("{name}: done");
        report.insert(name.clone(), serde_json::Value::Array(sides));
    }
    std::fs::write(out.join("wall.json"), serde_json::to_string_pretty(&report).unwrap()).expect("write");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let names: Vec<String> = if args.len() > 2 { args[2..].to_vec() } else { fold_common::head_names() };
    match args.first().map(String::as_str) {
        Some("density") => density(&PathBuf::from(&args[1]), &names),
        Some("wall") => wall(&PathBuf::from(&args[1]), &names),
        _ => {
            eprintln!("Usage: brow_wall density|wall <out> [name...]");
            std::process::exit(2);
        }
    }
}
