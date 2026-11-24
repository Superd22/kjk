//! The colour of the skin on KOTOR's bake and on our paint, measured in the same pixels of our layout (paint track, part C, skin colour
//! round of 2026-11-20). Analysis only.
//!
//! `skin_measure <out dir> <name>... [--set <control id>=<value>]... [--label <label>]`
//!
//! For each head it paints the head's recipe from the live page's data on the GPU at 2,048 pixels and reads KOTOR's bake in the same
//! pixels. Pixels that are hair on KOTOR's bake (brows, stubble, beards, fringes), the lips, the eye openings and the band of the brows are
//! left out of both. It writes `<out dir>/<label>/<name>.json` with:
//! - the ramp: the face's skin pixels sorted by KOTOR's lightness into 10 bins of equal count, with each bin's mean L*, chroma and hue,
//!   for KOTOR and for ours (ours sorted by our own lightness);
//! - the zones: each zone's mean Lab minus the face's mean Lab, for KOTOR and for ours;
//! - the middle of the face against its sides.
//!
//! It also writes front views at face distance (110 by 120 mm at 4 pixels per millimetre): `<name>_kotor.jpg` (KOTOR's own head with its own
//! texture), `<name>_ours.jpg` and `<name>_zones.jpg` (ours with the zones tinted), and `<name>_ours.png`, the texture.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::{Common, KotorFront, Lightness, RES, bake_path};
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[path = "skin_shared/zones.rs"]
mod zones;

use zones::*;
const N: u32 = 2048;

fn parse(args: &mut Vec<String>, flag: &str) -> Vec<String> {
    let mut out = Vec::new();
    while let Some(k) = args.iter().position(|a| a == flag) {
        out.push(args.remove(k + 1));
        args.remove(k);
    }
    out
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let overrides: Vec<(String, f64)> = parse(&mut args, "--set")
        .iter()
        .map(|p| {
            let (id, v) = p.split_once('=').expect("--set id=value");
            (id.to_string(), v.parse().expect("number"))
        })
        .collect();
    let label = parse(&mut args, "--label").pop().unwrap_or_else(|| "now".into());
    let hand: Option<Value> = parse(&mut args, "--hand").pop().map(|p| serde_json::from_str(&std::fs::read_to_string(p).expect("hand")).expect("json"));
    let no_pictures = args.iter().any(|a| a == "--no-pictures");
    args.retain(|a| a != "--no-pictures");
    if args.len() < 2 {
        eprintln!("Usage: skin_measure <out dir> <name>... [--set id=value]... [--label label] [--hand values.json] [--no-pictures]");
        std::process::exit(2);
    }
    let out = PathBuf::from(&args[0]).join(&label);
    std::fs::create_dir_all(&out).expect("out");
    let lay = layout();
    let common = Common::load();
    assert_eq!(common.uv, lay.uv, "the live page's layout differs from the bundle's");
    let mut painter = GpuPainter::new(Gpu::blocking().expect("GPU"), lay);
    let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
    let list = controls();

    for name in &args[1..] {
        let pos = positions(name);
        painter.set_head(pos.clone());
        pollster::block_on(painter.prepare(N));
        let head = &recipes["heads"][name.as_str()];
        let mut values: Vec<f64> = list.iter().map(|c| head[&c.id].as_f64().unwrap_or(c.default)).collect();
        if let Some(h) = &hand {
            for (id, v) in h["*"].as_object().into_iter().flatten().chain(h[name.as_str()].as_object().into_iter().flatten()) {
                if let Some(k) = list.iter().position(|c| &c.id == id) {
                    values[k] = v.as_f64().expect("number");
                }
            }
        }
        for (id, x) in &overrides {
            let k = list.iter().position(|c| &c.id == id).unwrap_or_else(|| panic!("no control {id}"));
            values[k] = *x;
        }
        let mut switches = vec![true; 9];
        switches[8] = false;
        painter.paint(N, &recipe_from(&values), &layers_from(&switches));
        let rgba = pollster::block_on(painter.read_rgba(N));
        let ours_img = image::RgbImage::from_fn(N, N, |x, y| {
            let at = 4 * (y * N + x) as usize;
            image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
        });
        let detail = painter.head().and_then(|h| h.detail.as_ref()).expect("detail");
        let lm = detail.folds.as_ref().expect("folds").landmarks;
        let pm = detail.landmarks;
        let mm: Vec<[f64; 3]> = pos.iter().map(|p| p.map(|x| 1000.0 * x)).collect();
        let zones = Zones::new(lm, pm, &mm);

        let bake_img = image::open(bake_path(name)).expect("bake").to_rgb8();
        let bake = Lightness::from_rgb(&bake_img);
        let hair = bake.hair();
        let layout_mm: Vec<[f64; 3]> = common.vmap.iter().map(|&m| mm[m]).collect();

        let mut face_k = Vec::new();
        let mut face_o = Vec::new();
        let mut zone_k: Vec<Vec<[f64; 3]>> = vec![Vec::new(); ZONES.len()];
        let mut zone_o: Vec<Vec<[f64; 3]>> = vec![Vec::new(); ZONES.len()];
        let mut sides: [[Vec<[f64; 3]>; 2]; 2] = Default::default();
        for px in &common.map.pixels {
            let i = px.y as usize * RES + px.x as usize;
            if !bake.valid[i] || hair[i] {
                continue;
            }
            let t = common.tris[px.triangle as usize];
            let p = [0, 1, 2].map(|k| (0..3).map(|c| px.barycentric[c] * layout_mm[t[c] as usize][k]).sum::<f64>());
            if zones.excluded(p) {
                continue;
            }
            let k = lab_of(bake_img.get_pixel(u32::from(px.x), u32::from(px.y)).0);
            let o = lab_of(ours_img.get_pixel(u32::from(px.x), u32::from(px.y)).0);
            let z = zones.zone(p);
            // Zones count toward the face's mean only on the front of the face, and the ears never.
            if zones.face(p) && z != Some(2) {
                face_k.push(k);
                face_o.push(o);
            }
            if let Some(z) = z {
                zone_k[z].push(k);
                zone_o[z].push(o);
            }
            if z != Some(2)
                && let Some(m) = zones.middle_or_side(p)
            {
                let s = usize::from(!m);
                sides[0][s].push(k);
                sides[1][s].push(o);
            }
        }
        if std::env::var("SKIN_DEBUG").is_ok() {
            let mut raw = [0usize; 10];
            let mut why = [0usize; 3];
            for px in &common.map.pixels {
                let i = px.y as usize * RES + px.x as usize;
                let t = common.tris[px.triangle as usize];
                let p = [0, 1, 2].map(|k| (0..3).map(|c| px.barycentric[c] * layout_mm[t[c] as usize][k]).sum::<f64>());
                if let Some(z) = zones.zone(p) {
                    raw[z] += 1;
                    if z == 7 || z == 8 {
                        if !bake.valid[i] { why[0] += 1 } else if hair[i] { why[1] += 1 } else if zones.excluded(p) { why[2] += 1 }
                    }
                }
            }
            eprintln!("{name}: raw zone pixels {raw:?}, jaw and temples lost to invalid/hair/excluded {why:?}");
        }
        let (mk, mo) = (mean(&face_k), mean(&face_o));
        let (ramp_k, ramp_o) = (RampFn::new(&face_k), RampFn::new(&face_o));
        let mut zone_json = serde_json::Map::new();
        for (z, zname) in ZONES.iter().enumerate().take(9) {
            let (zk, zo) = (mean(&zone_k[z]), mean(&zone_o[z]));
            zone_json.insert(zname.to_string(), json!({ "pixels": zone_k[z].len(), "kotor": delta(zk, mk), "ours": delta(zo, mo), "kotor_beyond_light": ramp_k.beyond(&zone_k[z]), "ours_beyond_light": ramp_o.beyond(&zone_o[z]) }));
        }
        let mid_side = |s: &[Vec<[f64; 3]>; 2]| delta(mean(&s[0]), mean(&s[1]));
        let report = json!({
            "name": name,
            "pixels": face_k.len(),
            "face_mean": { "kotor": [r2(mk[0]), r2(mk[1]), r2(mk[2]), r2(chroma(mk)), r2(hue(mk))], "ours": [r2(mo[0]), r2(mo[1]), r2(mo[2]), r2(chroma(mo)), r2(hue(mo))] },
            "ramp": { "kotor": ramp(face_k.clone()), "ours": ramp(face_o.clone()) },
            "zones": zone_json,
            "middle_minus_sides": { "kotor": mid_side(&sides[0]), "ours": mid_side(&sides[1]) },
        });
        std::fs::write(out.join(format!("{name}.json")), serde_json::to_string_pretty(&report).unwrap()).expect("write");

        if !no_pictures {
            let f = lm.frame;
            let front = KotorFront::ours(&common, &pos, ours_img.clone(), f.across, f.up, f.forward);
            let centre = add(mid(lm.nose_tip, mid(lm.mouth_corner[0], lm.mouth_corner[1])), f.up, 8.0);
            let q = front.project(centre).0;
            front.tile_at(q, 110.0, 120.0, 4.0).save(out.join(format!("{name}_ours.jpg"))).expect("save");
            let kotor_path = PathBuf::from(&args[0]).join(format!("{name}_kotor.jpg"));
            if !kotor_path.exists() {
                KotorFront::open(name, f.across, f.up, f.forward).tile_at(q, 110.0, 120.0, 4.0).save(&kotor_path).expect("save");
            }
            if label == "now" {
                // The zones, tinted over our front view, to check where they lie.
                let mut img = front.tile_at(q, 110.0, 120.0, 4.0);
                let origin = [q[0] - 55.0, q[1] + 60.0];
                for (x, y, px) in img.enumerate_pixels_mut() {
                    let qq = [origin[0] + (f64::from(x) + 0.5) / 4.0, origin[1] - (f64::from(y) + 0.5) / 4.0];
                    if let Some(p) = front.front_point(qq) {
                        let tint = if zones.excluded(p) { Some([0, 0, 0]) } else { zones.zone(p).map(|z| ZONE_TINT[z]) };
                        if let Some(t) = tint {
                            for k in 0..3 {
                                px.0[k] = ((u16::from(px.0[k]) + u16::from(t[k])) / 2) as u8;
                            }
                        }
                    }
                }
                img.save(out.join(format!("{name}_zones.jpg"))).expect("save");
            }
        }
        println!("{name}: {} face pixels, KOTOR mean L {:.1} C {:.1} h {:.1}, ours L {:.1} C {:.1} h {:.1}", face_k.len(), mk[0], chroma(mk), hue(mk), mo[0], chroma(mo), hue(mo));
    }
}
