//! Step 1 of the fold work: measures each fold on both sides of the 57 KOTOR bakes (paint track, fine detail). Analysis only.
//!
//! For every head it writes the measure of each fold and side to `<data dir>/measure_kotor.json`, and two front views of KOTOR's own
//! head with its own texture around the fold to `<pictures dir>/<fold>/tiles/`: `<name>.jpg`, plain, and `<name>_track.jpg`, with the
//! found fold drawn in green where it is present and in yellow where the search found too little contrast. The under-eye crease is
//! measured on KOTOR's own head seen from the front, the other folds on KOTOR's bake in our layout, with its hair masked
//! (`Lightness::hair`).
//!
//! Usage: fold_measure [<fold id> ...]   (the default is every fold)

#[path = "shared/bundle.rs"]
mod bundle;
#[path = "shared/fold_common.rs"]
mod fold_common;
#[path = "shared/fold_measure.rs"]
mod fold_measure;

use fold_common::*;
use fold_measure::{Frame, KotorSkin, STATION_MM};
use head_paint::folds::{FoldHead, FoldKind, KINDS};
use serde_json::{Value, json};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let kinds: Vec<FoldKind> = if args.is_empty() { KINDS.to_vec() } else { KINDS.iter().copied().filter(|k| args.iter().any(|a| a == k.id())).collect() };
    let started = std::time::Instant::now();
    let c = Common::load();
    let names = head_names();
    std::fs::create_dir_all(DATA).unwrap();
    for k in &kinds {
        std::fs::create_dir_all(Path::new(PAGES).join(k.id()).join("tiles")).unwrap();
    }
    let results: Vec<Value> = parallel(&names, 8, |name| {
        let positions = c.fitted(name);
        let head = FoldHead::new(&c.tris, &c.vmap, &positions).expect("our head holds the landmarks");
        let skin = KotorSkin::open(name, &head);
        if std::env::var("FOLD_HAIR").is_ok() {
            let mut img = image::open(bake_path(name)).unwrap().to_rgb8();
            for (i, h) in skin.hair.iter().enumerate() {
                if *h {
                    img.put_pixel((i % RES) as u32, (i / RES) as u32, image::Rgb([255, 0, 255]));
                }
            }
            image::imageops::crop_imm(&img, 300, 600, 1250, 1200).to_image().save(Path::new(PAGES).join("hair_mask").join(format!("{name}.jpg"))).unwrap();
        }
        let front = &skin.front;
        let mut out = serde_json::Map::new();
        out.insert("name".into(), json!(name));
        for &kind in &kinds {
            let mut sides = Vec::new();
            let mut front_dots: Vec<([f64; 2], [u8; 3])> = Vec::new();
            for side in 0..2 {
                let frame = Frame::new(&head, &c.uv, &c.tris, kind, side);
                let m = frame.measure(&head, &|s| skin.lightness(kind, s));
                for s in 0..frame.stations {
                    let (p, _, a) = frame.path.at(s as f64 * STATION_MM);
                    let u = m.track[s];
                    let inside = m.present && m.run.is_some_and(|(x, y)| s >= x && s <= y);
                    if inside || s % 4 == 0 {
                        if let Some(hit) = head.index.closest([p[0] + u * a[0], p[1] + u * a[1], p[2] + u * a[2]]) {
                            front_dots.push((front.project(hit.point).0, if inside { [0, 255, 0] } else { [255, 220, 0] }));
                        }
                    }
                }
                if std::env::var("FOLD_STRIPS").is_ok() {
                    // The straightened strip: one row per station, from the search path's start at the top, and one column per offset,
                    // inward on the left. Each row is shown against its own median, 3 grey levels per L*.
                    let w = frame.width() as u32;
                    let mut strip = image::RgbImage::new(w, frame.stations as u32);
                    for s in 0..frame.stations {
                        let row: Vec<f64> = (0..frame.width()).map(|u| frame.sample(s, u).and_then(|x| skin.lightness(kind, &x)).unwrap_or(f64::NAN)).collect();
                        let mid = median(&row.iter().copied().filter(|v| v.is_finite()).collect::<Vec<_>>());
                        for (u, v) in row.iter().enumerate() {
                            let g = if v.is_finite() { (128.0 + 3.0 * (v - mid)).clamp(0.0, 255.0) as u8 } else { 0 };
                            let inside = m.present && m.run.is_some_and(|(a, b)| s >= a && s <= b);
                            let on_track = (frame.offsets[u] - m.track[s]).abs() < 0.05;
                            strip.put_pixel(u as u32, s as u32, image::Rgb(if on_track && s % 2 == 0 { if inside { [0, 255, 0] } else { [255, 200, 0] } } else { [g, g, g] }));
                        }
                    }
                    let dir = Path::new(PAGES).join(kind.id()).join("tiles");
                    strip.save(dir.join(format!("{name}_strip{side}.png"))).unwrap();
                }
                sides.push(m.to_json());
            }
            out.insert(kind.id().into(), json!(sides));
            let dir = Path::new(PAGES).join(kind.id()).join("tiles");
            let (centre, w_mm, h_mm) = fold_measure::tile_window(kind, &head);
            let centre = front.project(centre).0;
            let plain = front.tile(centre, w_mm, h_mm);
            let mut tracked = plain.clone();
            fold_measure::draw_on_tile(&mut tracked, centre, w_mm, h_mm, &front_dots);
            plain.save(dir.join(format!("{name}.jpg"))).unwrap();
            tracked.save(dir.join(format!("{name}_track.jpg"))).unwrap();
        }
        println!("{name}");
        Value::Object(out)
    });
    let path = Path::new(DATA).join("measure_kotor.json");
    let mut doc: Value = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({ "heads": {} }));
    for r in results {
        let name = r["name"].as_str().unwrap().to_string();
        let entry = doc["heads"].as_object_mut().unwrap().entry(name).or_insert(json!({}));
        for (k, v) in r.as_object().unwrap() {
            if k != "name" {
                entry[k] = v.clone();
            }
        }
    }
    doc["method"] = json!({
        "station_mm": STATION_MM,
        "present_lstar": fold_measure::PRESENT_LSTAR,
        "min_length_mm": fold_measure::MIN_LENGTH_MM,
        "sides": "side 0 is the x < 0 side of our head (the left of the layout), side 1 the x > 0 side",
        "offsets": "track_mm and the profile are positive outward (away from the face's middle) for the nose-to-mouth and mouth corner folds, and downward for the chin and under-eye creases",
    });
    std::fs::write(&path, serde_json::to_string(&doc).unwrap()).unwrap();
    println!("{} heads in {:.1} s", names.len(), started.elapsed().as_secs_f64());
}
