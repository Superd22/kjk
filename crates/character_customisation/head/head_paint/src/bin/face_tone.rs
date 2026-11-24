//! Measures the colour of KOTOR's whole painted face and of our baked flat tone (paint track, step C2). This is an analysis tool and never ships.
//!
//! For each head the tool reads KOTOR's paint baked into our layout (`<kotor dir>/<name>_ours.jpg`) at every pixel that lies on skin, outside
//! the lids and outside the hidden faces of the lip slit, and drops pixels that the bake left grey. It writes the mean and the median Lab colour
//! of those pixels, and the Lab colour of our baked tone (`<ours dir>/<name>_<variant>.jpg`) when that file exists.
//!
//! Usage: face_tone <common bundle dir> <kotor texture dir> <our texture dir> <our variant id> <skin_tone report.json> <out json>

use head_paint::colour::srgb_to_lab;
use head_paint::pixel_map::PixelMap;
use serde_json::{Value, json};
use std::path::Path;

const GREY: i32 = 123;

fn read_array(manifest: &Value, bytes: &[u8], name: &str, size: usize) -> Vec<u8> {
    let entry = &manifest["arrays"][name];
    let count: usize = entry["shape"].as_array().expect("shape").iter().map(|d| d.as_u64().unwrap_or(0) as usize).product();
    let offset = entry["offset"].as_u64().unwrap_or(0) as usize;
    bytes[offset..offset + count * size].to_vec()
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn lab_of(rgb: &[u8]) -> [f64; 3] {
    srgb_to_lab([rgb[0], rgb[1], rgb[2]].map(|c| f64::from(c) / 255.0))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        eprintln!("Usage: face_tone <common bundle dir> <kotor texture dir> <our texture dir> <our variant id> <skin_tone report.json> <out json>");
        std::process::exit(2);
    }
    let dir = Path::new(&args[0]);
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("bundle.json")).expect("bundle.json")).expect("parse");
    let bytes = std::fs::read(dir.join("bundle.bin")).expect("bundle.bin");
    let uv: Vec<[f64; 2]> = read_array(&manifest, &bytes, "uv", 8).chunks(16).map(|c| [f64::from_le_bytes(c[..8].try_into().unwrap()), f64::from_le_bytes(c[8..].try_into().unwrap())]).collect();
    let u32s = |name: &str| -> Vec<u32> { read_array(&manifest, &bytes, name, 4).chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect() };
    let triangles: Vec<[u32; 3]> = u32s("tris").chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
    let vmap: Vec<usize> = u32s("vmap").into_iter().map(|i| i as usize).collect();
    let skin = read_array(&manifest, &bytes, "skin", 1);
    let lid = read_array(&manifest, &bytes, "lid", 1);
    let dark: Vec<f64> = read_array(&manifest, &bytes, "dark", 8).chunks(8).map(|c| f64::from_le_bytes(c.try_into().unwrap())).collect();

    let map = PixelMap::new(&uv, &triangles, 2048);
    let per_pixel = |values: Vec<f64>| -> Vec<f64> {
        let new: Vec<[f64; 1]> = vmap.iter().map(|&o| [values[o]]).collect();
        map.interpolate(&triangles, &new).into_iter().map(|v| v[0]).collect()
    };
    let on_skin = per_pixel(skin.iter().map(|&v| f64::from(v)).collect());
    let on_lid = per_pixel(lid.iter().map(|&v| f64::from(v)).collect());
    let hidden = per_pixel(dark);

    let report: Value = serde_json::from_str(&std::fs::read_to_string(&args[4]).expect("report")).expect("parse report");
    let names: Vec<String> = report["heads"].as_array().expect("heads").iter().map(|v| v.as_str().unwrap().to_string()).collect();
    let mut out = Vec::new();
    for (k, name) in names.iter().enumerate() {
        let img = image::open(Path::new(&args[1]).join(format!("{name}_ours.jpg"))).unwrap_or_else(|e| panic!("{name}: {e}")).to_rgb8();
        let (mut l, mut a, mut b, mut c) = (vec![], vec![], vec![], vec![]);
        for (i, p) in map.pixels.iter().enumerate() {
            if on_skin[i] < 0.999 || on_lid[i] > 0.0 || hidden[i] > 0.0 {
                continue;
            }
            let px = img.get_pixel(u32::from(p.x), u32::from(p.y)).0;
            if px.iter().all(|&v| (i32::from(v) - GREY).abs() <= 2) {
                continue;
            }
            let lab = lab_of(&px);
            l.push(lab[0]);
            a.push(lab[1]);
            b.push(lab[2]);
            c.push(lab[1].hypot(lab[2]));
        }
        let n = l.len();
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let mean_lab = [mean(&l), mean(&a), mean(&b)];
        let mean_chroma = mean(&c);
        let median_lab = [median(&mut l), median(&mut a), median(&mut b)];
        let median_chroma = median(&mut c);
        let ours_file = Path::new(&args[2]).join(format!("{name}_{}.jpg", args[3]));
        let ours = image::open(&ours_file).ok().map(|i| lab_of(&i.to_rgb8().get_pixel(1024, 1024).0));
        out.push(json!({
            "name": name, "pixels": n,
            "target_lab": report["base tones (Lab)"][k],
            "mean_lab": mean_lab, "median_lab": median_lab, "mean_chroma": mean_chroma, "median_chroma": median_chroma,
            "ours_lab": ours,
        }));
    }
    std::fs::write(&args[5], serde_json::to_string_pretty(&json!({ "heads": out })).unwrap()).expect("write");
}
