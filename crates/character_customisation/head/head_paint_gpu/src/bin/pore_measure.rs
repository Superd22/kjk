//! Measures the fine band of the skin (the pores and grain) on KOTOR's own heads and on our paint, at the same places and the same
//! scale in millimetres. Analysis only (pores, second round).
//!
//! `pore_measure <out.json> <name>... [--set id=value]... [--label <name>]`
//!
//! For each head and each of the cheek, the upper cheek, the forehead and the nose, it reads a front view 16 by 16 mm at 20 samples per
//! millimetre of KOTOR's head with its own texture, and of our head with our paint. On each it computes the band: the lightness L*
//! minus its blur at 0.5 mm. It writes per view: the band's spread and skew; the light spots and dark spots per cm² beyond one spread;
//! the mean size of the light and dark blobs; the band's autocorrelation radius and the distance to its first ring (the grain's spacing);
//! the autocorrelation's long-to-short axis ratio; and how the band's spread follows the local lightness (the light baked in), as the
//! correlation over 2 mm blocks and the spread in the brightest third of the blocks against the darkest third. For KOTOR it also gives the
//! texture's texels per millimetre at the view's centre.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use bundle::Bundle;
use fold_common::{Common, KotorFront, heads_dir};
use head_paint::colour::srgb_to_lab;
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint::folds::Landmarks;
use head_paint::live::Layout;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::{Value, json};
use std::path::Path;

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
const N: u32 = 2048;
const VIEW_MM: f64 = 16.0;
const PER_MM: f64 = 20.0;

fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap().chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().unwrap())).collect()
}
fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path).unwrap().chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect()
}
fn layout() -> Layout {
    let l = Path::new(DATA).join("layout");
    let triples = |v: Vec<u32>| v.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect::<Vec<_>>();
    Layout {
        uv: f64s(&l.join("uv.f64")).chunks_exact(2).map(|p| [p[0], p[1]]).collect(),
        triangles: triples(u32s(&l.join("tris.u32"))),
        layout_to_mesh: u32s(&l.join("vmap.u32")).into_iter().map(|i| i as usize).collect(),
        mesh_triangles: triples(u32s(&l.join("mesh_tris.u32"))),
    }
}
fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}
fn mid(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|k| 0.5 * (a[k] + b[k]))
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn regions(l: &Landmarks) -> Vec<(&'static str, [f64; 3])> {
    let f = l.frame;
    let cheek = add(add(l.lower_lid[0], f.up, -17.0), f.out(0), 2.0);
    let upper = add(add(l.lower_lid[0], f.up, -9.0), f.out(0), 10.0);
    let eyes = mid(mid(l.eye_inner[0], l.eye_outer[0]), mid(l.eye_inner[1], l.eye_outer[1]));
    let forehead = add(eyes, f.up, 26.0);
    let nose = add(l.nose_tip, f.up, 8.0);
    vec![("cheek", cheek), ("cheekbone", upper), ("forehead", forehead), ("nose", nose)]
}

/// A square grid of L*, `n` by `n`, with a mask of the samples that hit the head.
struct Field {
    n: usize,
    l: Vec<f64>,
    ok: Vec<bool>,
}

fn sample(front: &KotorFront, centre: [f64; 2]) -> Field {
    let n = (VIEW_MM * PER_MM) as usize;
    let mut l = vec![0.0; n * n];
    let mut ok = vec![false; n * n];
    for y in 0..n {
        for x in 0..n {
            let q = [centre[0] - 0.5 * VIEW_MM + (x as f64 + 0.5) / PER_MM, centre[1] + 0.5 * VIEW_MM - (y as f64 + 0.5) / PER_MM];
            if let Some(c) = front.front_rgb(q) {
                l[y * n + x] = srgb_to_lab(c.map(|v| f64::from(v) / 255.0))[0];
                ok[y * n + x] = true;
            }
        }
    }
    Field { n, l, ok }
}

fn blur(v: &[f64], n: usize, sigma_px: f64) -> Vec<f64> {
    let r = (3.0 * sigma_px).ceil() as i64;
    let k: Vec<f64> = (-r..=r).map(|i| (-(i * i) as f64 / (2.0 * sigma_px * sigma_px)).exp()).collect();
    let pass = |src: &[f64], horizontal: bool| {
        let mut out = vec![0.0; n * n];
        for y in 0..n {
            for x in 0..n {
                let (mut s, mut w) = (0.0, 0.0);
                for (j, kk) in k.iter().enumerate() {
                    let d = j as i64 - r;
                    let (xx, yy) = if horizontal { (x as i64 + d, y as i64) } else { (x as i64, y as i64 + d) };
                    if xx < 0 || yy < 0 || xx >= n as i64 || yy >= n as i64 {
                        continue;
                    }
                    s += kk * src[yy as usize * n + xx as usize];
                    w += kk;
                }
                out[y * n + x] = s / w;
            }
        }
        out
    };
    pass(&pass(v, true), false)
}

/// Connected blobs of `mask` (4-neighbours): their count and mean area in samples.
fn blobs(mask: &[bool], n: usize) -> (usize, f64) {
    let mut seen = vec![false; n * n];
    let (mut count, mut total) = (0usize, 0usize);
    for s in 0..n * n {
        if !mask[s] || seen[s] {
            continue;
        }
        let mut stack = vec![s];
        seen[s] = true;
        let mut area = 0;
        while let Some(p) = stack.pop() {
            area += 1;
            let (x, y) = (p % n, p / n);
            let mut push = |q: usize| {
                if mask[q] && !seen[q] {
                    seen[q] = true;
                    stack.push(q);
                }
            };
            if x > 0 { push(p - 1); }
            if x + 1 < n { push(p + 1); }
            if y > 0 { push(p - n); }
            if y + 1 < n { push(p + n); }
        }
        count += 1;
        total += area;
    }
    (count, if count > 0 { total as f64 / count as f64 } else { 0.0 })
}

fn measure(field: &Field) -> Value {
    let n = field.n;
    // Only the inner square, away from the blur's edges, counts. A view whose samples miss the head anywhere there is skipped.
    let m = 20usize;
    let inner = |x: usize, y: usize| x >= m && y >= m && x < n - m && y < n - m;
    if (0..n * n).any(|s| inner(s % n, s / n) && !field.ok[s]) {
        return json!(null);
    }
    let low = blur(&field.l, n, 0.5 * PER_MM);
    let band: Vec<f64> = field.l.iter().zip(&low).map(|(a, b)| a - b).collect();
    let soft = blur(&band, n, 0.06 * PER_MM);
    let ids: Vec<usize> = (0..n * n).filter(|&s| inner(s % n, s / n)).collect();
    let cnt = ids.len() as f64;
    let mean = ids.iter().map(|&s| band[s]).sum::<f64>() / cnt;
    let var = ids.iter().map(|&s| (band[s] - mean).powi(2)).sum::<f64>() / cnt;
    let sd = var.sqrt();
    let skew = ids.iter().map(|&s| (band[s] - mean).powi(3)).sum::<f64>() / cnt / sd.powi(3);
    // The spots: local extremes of the softened band beyond one spread.
    let ssd = (ids.iter().map(|&s| soft[s] * soft[s]).sum::<f64>() / cnt).sqrt();
    let (mut light, mut dark) = (0, 0);
    for &s in &ids {
        let (x, y) = (s % n, s / n);
        let v = soft[s];
        let mut is_max = v > ssd;
        let mut is_min = v < -ssd;
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let q = (y as i64 + dy) as usize * n + (x as i64 + dx) as usize;
                is_max &= v > soft[q];
                is_min &= v < soft[q];
            }
        }
        light += is_max as usize;
        dark += is_min as usize;
    }
    let area_cm2 = cnt / (PER_MM * PER_MM) / 100.0;
    let mask_light: Vec<bool> = (0..n * n).map(|s| inner(s % n, s / n) && soft[s] > ssd).collect();
    let mask_dark: Vec<bool> = (0..n * n).map(|s| inner(s % n, s / n) && soft[s] < -ssd).collect();
    let (bl, al) = blobs(&mask_light, n);
    let (bd, ad) = blobs(&mask_dark, n);
    let diameter = |area: f64| 2.0 * (area / std::f64::consts::PI).sqrt() / PER_MM;
    // The autocorrelation of the band, out to 1.5 mm.
    let r = (1.5 * PER_MM) as i64;
    let w = (2 * r + 1) as usize;
    let mut acf = vec![0.0; w * w];
    let core: Vec<usize> = ids.iter().copied().filter(|&s| {
        let (x, y) = ((s % n) as i64, (s / n) as i64);
        x >= m as i64 + r && y >= m as i64 + r && x < (n - m) as i64 - r && y < (n - m) as i64 - r
    }).collect();
    for dy in -r..=r {
        for dx in -r..=r {
            let mut s = 0.0;
            for &p in &core {
                let q = (p as i64 + dy * n as i64 + dx) as usize;
                s += (band[p] - mean) * (band[q] - mean);
            }
            acf[((dy + r) as usize) * w + (dx + r) as usize] = s / core.len() as f64 / var;
        }
    }
    // Radial profile, in steps of one sample.
    let mut radial = vec![(0.0, 0usize); r as usize + 1];
    for dy in -r..=r {
        for dx in -r..=r {
            let d = ((dx * dx + dy * dy) as f64).sqrt().round() as usize;
            if d <= r as usize {
                radial[d].0 += acf[((dy + r) as usize) * w + (dx + r) as usize];
                radial[d].1 += 1;
            }
        }
    }
    let prof: Vec<f64> = radial.iter().map(|(s, c)| s / *c as f64).collect();
    let half = prof.iter().position(|&v| v < 0.5).map(|i| i as f64 / PER_MM).unwrap_or(f64::NAN);
    let zero = prof.iter().position(|&v| v < 0.0);
    let ring = zero.and_then(|z| (z + 1..prof.len() - 1).find(|&i| prof[i] > prof[i - 1] && prof[i] >= prof[i + 1])).map(|i| i as f64 / PER_MM);
    // The central lobe's shape: the second moments of the samples above 0.3.
    let (mut sxx, mut syy, mut sxy, mut sw) = (0.0, 0.0, 0.0, 0.0);
    for dy in -r..=r {
        for dx in -r..=r {
            let v = acf[((dy + r) as usize) * w + (dx + r) as usize];
            if v > 0.3 && ((dx * dx + dy * dy) as f64) < (r * r) as f64 {
                sxx += v * (dx * dx) as f64;
                syy += v * (dy * dy) as f64;
                sxy += v * (dx * dy) as f64;
                sw += v;
            }
        }
    }
    let (a, b, c) = (sxx / sw, syy / sw, sxy / sw);
    let t = 0.5 * (a + b);
    let d = (0.25 * (a - b) * (a - b) + c * c).sqrt();
    let ratio = ((t + d) / (t - d).max(1e-9)).sqrt();
    let angle = 0.5 * (2.0 * c).atan2(a - b).to_degrees();
    // The band's spread against the local lightness, over blocks of 2 mm.
    let bsz = (2.0 * PER_MM) as usize;
    let mut blocks: Vec<(f64, f64, f64)> = Vec::new();
    let mut by = m;
    while by + bsz <= n - m {
        let mut bx = m;
        while bx + bsz <= n - m {
            let (mut sl, mut sb, mut sb2, mut sb3) = (0.0, 0.0, 0.0, 0.0);
            for y in by..by + bsz {
                for x in bx..bx + bsz {
                    let s = y * n + x;
                    sl += low[s];
                    sb += band[s];
                    sb2 += band[s] * band[s];
                    sb3 += band[s].powi(3);
                }
            }
            let k = (bsz * bsz) as f64;
            let mu = sb / k;
            let v2 = sb2 / k - mu * mu;
            let sk = (sb3 / k - 3.0 * mu * sb2 / k + 2.0 * mu.powi(3)) / v2.max(1e-12).powf(1.5);
            blocks.push((sl / k, v2.sqrt(), sk));
            bx += bsz;
        }
        by += bsz;
    }
    let nb = blocks.len() as f64;
    let ml = blocks.iter().map(|b| b.0).sum::<f64>() / nb;
    let ms = blocks.iter().map(|b| b.1).sum::<f64>() / nb;
    let cov = blocks.iter().map(|b| (b.0 - ml) * (b.1 - ms)).sum::<f64>();
    let vl = blocks.iter().map(|b| (b.0 - ml).powi(2)).sum::<f64>();
    let vs = blocks.iter().map(|b| (b.1 - ms).powi(2)).sum::<f64>();
    let corr = cov / (vl * vs).sqrt();
    let slope = cov / vl;
    let mut sorted = blocks.clone();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
    let third = sorted.len() / 3;
    let avg = |s: &[(f64, f64, f64)], f: fn(&(f64, f64, f64)) -> f64| s.iter().map(f).sum::<f64>() / s.len() as f64;
    let dark_third = &sorted[..third];
    let bright_third = &sorted[sorted.len() - third..];
    let r3 = |v: f64| (v * 1000.0).round() / 1000.0;
    json!({
        "lightness": r3(ml),
        "lightness_range": r3(sorted[sorted.len() - 1].0 - sorted[0].0),
        "band_sd": r3(sd),
        "skew": r3(skew),
        "light_spots_per_cm2": r3(light as f64 / area_cm2),
        "dark_spots_per_cm2": r3(dark as f64 / area_cm2),
        "light_blobs_per_cm2": r3(bl as f64 / area_cm2),
        "dark_blobs_per_cm2": r3(bd as f64 / area_cm2),
        "light_blob_mm": r3(diameter(al)),
        "dark_blob_mm": r3(diameter(ad)),
        "acf_half_mm": r3(half),
        "acf_ring_mm": ring.map(r3),
        "acf_axis_ratio": r3(ratio),
        "acf_angle_deg": r3(angle),
        "sd_vs_lightness_corr": r3(corr),
        "sd_per_l": r3(slope),
        "sd_bright_third": r3(avg(bright_third, |b| b.1)),
        "sd_dark_third": r3(avg(dark_third, |b| b.1)),
        "skew_bright_third": r3(avg(bright_third, |b| b.2)),
        "skew_dark_third": r3(avg(dark_third, |b| b.2)),
    })
}

/// KOTOR's texels per millimetre near a point: from the triangle of KOTOR's head whose centre is nearest, its texture area against its area
/// in 3D.
fn texels_per_mm(name: &str, p: [f64; 3]) -> f64 {
    let b = Bundle::open(&heads_dir().join(name));
    let v: Vec<[f64; 3]> = b.points("kv").into_iter().map(|q| q.map(|x| 1000.0 * x)).collect();
    let faces = b.triangles("kf");
    let kuv = b.f64s("kuv");
    let mat = b.u32s("kmat");
    let sizes: Vec<(u32, u32)> = b.manifest["textures"].as_array().unwrap().iter().map(|t| image::image_dimensions(t.as_str().unwrap()).unwrap()).collect();
    let mut best = (f64::INFINITY, 0.0);
    for (f, t) in faces.iter().enumerate() {
        let c = [0, 1, 2].map(|k| (v[t[0] as usize][k] + v[t[1] as usize][k] + v[t[2] as usize][k]) / 3.0);
        let d = [c[0] - p[0], c[1] - p[1], c[2] - p[2]];
        let dist = dot(d, d);
        if dist < best.0 {
            let (tw, th) = sizes[(mat[f] as usize).min(sizes.len() - 1)];
            let e1 = [0, 1, 2].map(|k| v[t[1] as usize][k] - v[t[0] as usize][k]);
            let e2 = [0, 1, 2].map(|k| v[t[2] as usize][k] - v[t[0] as usize][k]);
            let cr = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let a3 = 0.5 * dot(cr, cr).sqrt();
            let u = |c: usize, k: usize| kuv[f * 6 + c * 2 + k];
            let a2 = 0.5 * ((u(1, 0) - u(0, 0)) * (u(2, 1) - u(0, 1)) - (u(2, 0) - u(0, 0)) * (u(1, 1) - u(0, 1))).abs() * f64::from(tw) * f64::from(th);
            if a3 > 1e-6 {
                best = (dist, (a2 / a3).sqrt());
            }
        }
    }
    best.1
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut overrides: Vec<(String, f64)> = Vec::new();
    while let Some(k) = args.iter().position(|a| a == "--set") {
        let (id, v) = args[k + 1].split_once('=').unwrap();
        overrides.push((id.to_string(), v.parse().unwrap()));
        args.drain(k..k + 2);
    }
    let mut layers = "11111".to_string();
    if let Some(k) = args.iter().position(|a| a == "--layers") {
        layers = args[k + 1].clone();
        args.drain(k..k + 2);
    }
    let no_kotor = args.iter().any(|a| a == "--no-kotor");
    args.retain(|a| a != "--no-kotor");
    let out = args[0].clone();
    let layout = layout();
    let common = Common::load();
    let mut painter = GpuPainter::new(Gpu::blocking().expect("GPU"), layout);
    let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).unwrap()).unwrap();
    let list = controls();
    let mut result = serde_json::Map::new();
    for name in &args[1..] {
        let pos: Vec<[f64; 3]> = f64s(&Path::new(DATA).join("heads").join(format!("{name}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect();
        let marks = Landmarks::new(&pos);
        let f = marks.frame;
        painter.set_head(pos.clone());
        pollster::block_on(painter.prepare(N));
        let head = &recipes["heads"][name.as_str()];
        let mut values: Vec<f64> = list.iter().map(|c| head[&c.id].as_f64().unwrap_or(c.default)).collect();
        for (id, x) in &overrides {
            let k = list.iter().position(|c| &c.id == id).unwrap_or_else(|| panic!("no control {id}"));
            values[k] = *x;
        }
        painter.paint(N, &recipe_from(&values), &layers_from(&layers.chars().map(|c| c == '1').collect::<Vec<_>>()));
        let rgba = pollster::block_on(painter.read_rgba(N));
        let tex = image::RgbImage::from_fn(N, N, |x, y| {
            let at = 4 * (y * N + x) as usize;
            image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
        });
        let ours = KotorFront::ours(&common, &pos, tex, f.across, f.up, f.forward);
        let kotor = (!no_kotor).then(|| KotorFront::open(name, f.across, f.up, f.forward));
        let mut per = serde_json::Map::new();
        for (region, centre) in regions(&marks) {
            let q = ours.project(centre).0;
            let mut entry = serde_json::Map::new();
            entry.insert("ours".into(), measure(&sample(&ours, q)));
            if let Some(k) = &kotor {
                let mut m = measure(&sample(k, q));
                if let Some(o) = m.as_object_mut() {
                    o.insert("texels_per_mm".into(), json!((texels_per_mm(name, centre) * 100.0).round() / 100.0));
                }
                entry.insert("kotor".into(), m);
            }
            per.insert(region.into(), Value::Object(entry));
        }
        eprintln!("{name}: measured");
        result.insert(name.clone(), Value::Object(per));
    }
    std::fs::write(&out, serde_json::to_string_pretty(&Value::Object(result)).unwrap()).unwrap();
}
