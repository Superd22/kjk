//! The eye paint's measure and fit (paint track, part C, the eye makeup, liner and shadow step). Analysis only.
//!
//! `eye_measure kotor <name>... | all` measures KOTOR's eyes on its own head with its own texture and writes `measure_kotor.json`.
//! `eye_measure fit <name>... | all` measures ours with the eye paint and moves the sliders until our measures meet KOTOR's, then writes
//! `fits.json` (the slider values of each head) and `measure_ours.json`.
//! `eye_measure time <name>...` times the eye layer's repaint at 2,048 pixels on the GPU.
//! `eye_measure ours <values json> <name>...` measures ours with the head's values and the given per-head overrides.
//!
//! Every measure is read on a front view of both eyes at 10 pixels per millimetre with a plain eyeball drawn in, the same way on KOTOR's
//! head and ours. For each column over the middle of each eye's visible eyeball, it walks up from the eyeball's top and down from its
//! bottom and reads the skin's L* and a*. The reference is the cheek 10 to 14 mm below the eyeball. Each measure is the median over the
//! columns, then the mean of the two eyes.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;
#[path = "eyes_shared/common.rs"]
mod common;

use common::*;
use fold_common::KotorFront;
use head_paint::colour::srgb_to_lab;
use head_paint::folds::Landmarks;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

const OUT: &str = "/Users/david/.claude/tools/likeness-fit-data/paint/eyes";
const PER_MM: f64 = 10.0;
const W_MM: f64 = 84.0;
const H_MM: f64 = 44.0;

/// What is read of one head's eyes.
#[derive(Clone, Copy, Debug, Default)]
struct Measure {
    /// How much darker than the cheek the darkest skin is within 2.5 mm above the eyeball, in L*.
    upper_dark: f64,
    /// How far above the eyeball that dark band reaches, in mm: the skin darker than the cheek by half of `upper_dark`.
    upper_width: f64,
    lower_dark: f64,
    lower_width: f64,
    /// How much darker than the cheek the skin is from 0.5 to 3.5 mm past the dark band above the eyeball, in L*, as a median, leaving
    /// out pixels more than 35 L* darker than the cheek, which are brows or lashes.
    socket: f64,
    /// How much darker than the cheek the skin 2.5 to 7 mm below the eyeball is, in L*.
    under: f64,
    /// How much redder (a*) than the cheek the skin 0 to 1.5 mm below the eyeball is.
    rim_red: f64,
    /// How much redder (a*) than the cheek the same skin as `socket` is, as a median.
    lid_red: f64,
    /// The colour of the skin 1 to 5 mm above the eyeball, in Lab.
    lid_lab: [f64; 3],
    /// The cheek's colour, in Lab.
    cheek_lab: [f64; 3],
    /// How many columns were read.
    columns: usize,
}

fn med(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn mean_of(v: &[f64]) -> f64 {
    let ok: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if ok.is_empty() { f64::NAN } else { ok.iter().sum::<f64>() / ok.len() as f64 }
}

/// Reads one eye: the columns `x0..x1` of the view.
fn measure_eye(lab: &[[f64; 3]], mask: &[f32], w: usize, h: usize, x0: usize, x1: usize) -> Option<Measure> {
    let cols: Vec<usize> = (x0..x1).filter(|&x| (0..h).any(|y| mask[y * w + x] > 0.5)).collect();
    if cols.len() < 20 {
        return None;
    }
    let (a, b) = (cols[0], *cols.last().unwrap());
    let span = b - a;
    let mut f: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut lid = [Vec::new(), Vec::new(), Vec::new()];
    let mut cheek = [Vec::new(), Vec::new(), Vec::new()];
    let px = |mm: f64| (mm * PER_MM).round() as i64;
    for x in (a + span / 4)..=(a + span * 3 / 4) {
        let ys: Vec<usize> = (0..h).filter(|&y| mask[y * w + x] > 0.5).collect();
        if ys.is_empty() {
            continue;
        }
        let (top, bottom) = (ys[0] as i64, *ys.last().unwrap() as i64);
        let at = |y: i64| -> Option<[f64; 3]> { (y >= 0 && (y as usize) < h && mask[y as usize * w + x] < 0.05).then(|| lab[y as usize * w + x]) };
        let range = |from: f64, to: f64, up: bool| -> Vec<[f64; 3]> {
            (px(from)..=px(to)).filter_map(|d| at(if up { top - 1 - d } else { bottom + 1 + d })).collect()
        };
        let ref_px = range(10.0, 14.0, false);
        if ref_px.len() < 10 {
            continue;
        }
        let mut lr: Vec<f64> = ref_px.iter().map(|c| c[0]).collect();
        let mut ar: Vec<f64> = ref_px.iter().map(|c| c[1]).collect();
        let mut br: Vec<f64> = ref_px.iter().map(|c| c[2]).collect();
        let (l_ref, a_ref) = (med(&mut lr), med(&mut ar));
        cheek[0].push(l_ref);
        cheek[1].push(a_ref);
        cheek[2].push(med(&mut br));
        let band = |up: bool, from: f64, to: f64| range(from, to, up);
        let darkest = |v: &[[f64; 3]]| v.iter().map(|c| c[0]).fold(f64::INFINITY, f64::min);
        for (up, key_d, key_w, reach) in [(true, "upper_dark", "upper_width", 2.5), (false, "lower_dark", "lower_width", 2.0)] {
            let v = band(up, 0.0, reach);
            if v.is_empty() {
                continue;
            }
            let dark = l_ref - darkest(&v);
            f.entry(key_d).or_default().push(dark);
            let wide = range(0.0, 4.0, up);
            let n = wide.iter().take_while(|c| c[0] < l_ref - 0.5 * dark).count();
            f.entry(key_w).or_default().push(n as f64 / PER_MM);
        }
        let mean_l = |v: &[[f64; 3]]| v.iter().map(|c| c[0]).sum::<f64>() / v.len().max(1) as f64;
        let mean_a = |v: &[[f64; 3]]| v.iter().map(|c| c[1]).sum::<f64>() / v.len().max(1) as f64;
        // The socket is read on the skin just past the column's own dark band, and pixels more than 35 L* darker than the cheek, which are
        // brows or lashes, are left out.
        let reach_up = f.get("upper_width").and_then(|v| v.last().copied()).unwrap_or(1.0);
        let s: Vec<[f64; 3]> = band(true, reach_up + 0.5, reach_up + 3.5).into_iter().filter(|c| c[0] > l_ref - 35.0).collect();
        if s.len() >= 5 {
            let mut l: Vec<f64> = s.iter().map(|c| c[0]).collect();
            f.entry("socket").or_default().push(l_ref - med(&mut l));
            let mut a: Vec<f64> = s.iter().map(|c| c[1]).collect();
            f.entry("lid_red").or_default().push(med(&mut a) - a_ref);
        }
        let u = band(false, 2.5, 7.0);
        if !u.is_empty() {
            f.entry("under").or_default().push(l_ref - mean_l(&u));
        }
        let r = band(false, 0.0, 1.5);
        if !r.is_empty() {
            f.entry("rim_red").or_default().push(mean_a(&r) - a_ref);
        }
        for c in band(true, 1.0, 5.0) {
            for k in 0..3 {
                lid[k].push(c[k]);
            }
        }
    }
    let mut get = |k: &str| f.get_mut(k).map(|v| med(v)).unwrap_or(f64::NAN);
    let m = Measure {
        upper_dark: get("upper_dark"),
        upper_width: get("upper_width"),
        lower_dark: get("lower_dark"),
        lower_width: get("lower_width"),
        socket: get("socket"),
        under: get("under"),
        rim_red: get("rim_red"),
        lid_red: get("lid_red"),
        lid_lab: [0, 1, 2].map(|k| med(&mut lid[k])),
        cheek_lab: [0, 1, 2].map(|k| med(&mut cheek[k])),
        columns: cheek[0].len(),
    };
    (m.columns >= 10).then_some(m)
}

/// Reads both eyes of a view and averages them.
fn measure(img: &image::RgbImage, mask: &[f32]) -> Option<Measure> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let lab: Vec<[f64; 3]> = img.pixels().map(|p| srgb_to_lab(p.0.map(|v| f64::from(v) / 255.0))).collect();
    let eyes: Vec<Measure> = [(0, w / 2), (w / 2, w)].iter().filter_map(|&(a, b)| measure_eye(&lab, mask, w, h, a, b)).collect();
    if eyes.is_empty() {
        return None;
    }
    let avg = |f: fn(&Measure) -> f64| mean_of(&eyes.iter().map(f).collect::<Vec<_>>());
    Some(Measure {
        upper_dark: avg(|m| m.upper_dark),
        upper_width: avg(|m| m.upper_width),
        lower_dark: avg(|m| m.lower_dark),
        lower_width: avg(|m| m.lower_width),
        socket: avg(|m| m.socket),
        under: avg(|m| m.under),
        rim_red: avg(|m| m.rim_red),
        lid_red: avg(|m| m.lid_red),
        lid_lab: [0, 1, 2].map(|k| mean_of(&eyes.iter().map(|m| m.lid_lab[k]).collect::<Vec<_>>())),
        cheek_lab: [0, 1, 2].map(|k| mean_of(&eyes.iter().map(|m| m.cheek_lab[k]).collect::<Vec<_>>())),
        columns: eyes.iter().map(|m| m.columns).sum(),
    })
}

fn to_json(m: &Option<Measure>) -> Value {
    let r = |x: f64| if x.is_finite() { json!((x * 100.0).round() / 100.0) } else { Value::Null };
    match m {
        None => Value::Null,
        Some(m) => json!({
            "upper_dark": r(m.upper_dark), "upper_width": r(m.upper_width), "lower_dark": r(m.lower_dark), "lower_width": r(m.lower_width),
            "socket": r(m.socket), "under": r(m.under), "rim_red": r(m.rim_red), "lid_red": r(m.lid_red),
            "lid_lab": m.lid_lab.map(r), "cheek_lab": m.cheek_lab.map(r), "columns": m.columns,
        }),
    }
}

fn from_json(v: &Value) -> Option<Measure> {
    if v.is_null() {
        return None;
    }
    let g = |k: &str| v[k].as_f64().unwrap_or(f64::NAN);
    let t = |k: &str| [0, 1, 2].map(|i| v[k][i].as_f64().unwrap_or(f64::NAN));
    Some(Measure {
        upper_dark: g("upper_dark"),
        upper_width: g("upper_width"),
        lower_dark: g("lower_dark"),
        lower_width: g("lower_width"),
        socket: g("socket"),
        under: g("under"),
        rim_red: g("rim_red"),
        lid_red: g("lid_red"),
        lid_lab: t("lid_lab"),
        cheek_lab: t("cheek_lab"),
        columns: v["columns"].as_u64().unwrap_or(0) as usize,
    })
}

struct Head {
    pos: Vec<[f64; 3]>,
    marks: Landmarks,
    centre: [f64; 2],
}

fn head(name: &str) -> Head {
    let pos = positions(name);
    let marks = Landmarks::new(&pos);
    let e = head_paint::eyes::eye_shapes(&pos).expect("default mesh");
    let f = marks.frame;
    let c = add(mid(e[0].centre, e[1].centre), f.up, -5.0);
    let centre = [dot(c, f.across), dot(c, f.up)];
    Head { pos, marks, centre }
}

fn measure_kotor(name: &str) -> Option<Measure> {
    let h = head(name);
    let f = h.marks.frame;
    let (balls, tris) = kotor_balls(name);
    if balls.is_empty() {
        return None;
    }
    let k = KotorFront::open(name, f.across, f.up, f.forward);
    let (img, mask) = view_masked(&k, &balls, &tris, f.forward, f.across, f.up, h.centre, W_MM, H_MM, PER_MM);
    measure(&img, &mask)
}

fn measure_ours(p: &mut Painter, name: &str, h: &Head, values: &[f64]) -> Option<Measure> {
    let f = h.marks.frame;
    let tex = p.texture(values, &[true; 6]);
    let ours = KotorFront::ours(&p.common, &h.pos, tex, f.across, f.up, f.forward);
    let (img, mask) = view_masked(&ours, &our_balls(&h.pos), &Vec::new(), f.forward, f.across, f.up, h.centre, W_MM, H_MM, PER_MM);
    let _ = name;
    measure(&img, &mask)
}

fn names_of(args: &[String]) -> Vec<String> {
    if args.len() == 1 && args[0] == "all" { head_names() } else { args.to_vec() }
}

fn write(name: &str, v: &Value) {
    std::fs::create_dir_all(OUT).expect("out");
    std::fs::write(Path::new(OUT).join(name), serde_json::to_string_pretty(v).expect("json")).expect("write");
}

fn read(name: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(Path::new(OUT).join(name)).unwrap_or_else(|_| "{}".into())).expect("json")
}

/// The sliders the fit moves, each with how much one unit of its measure's gap moves it, and its range.
const FITTED: [(&str, f64, f64, f64); 8] = [
    ("eye_upper_lash", 1.0 / 40.0, 0.0, 1.0),
    ("eye_upper_lash_mm", 0.8, 0.3, 2.5),
    ("eye_lower_lash", 1.0 / 30.0, 0.0, 1.0),
    ("eye_lower_lash_mm", 0.8, 0.2, 2.0),
    ("eye_socket", 0.9, 0.0, 30.0),
    ("eye_under_eye", 0.9, 0.0, 30.0),
    ("eye_rims", 1.0 / 12.0, 0.0, 1.0),
    ("eye_lid_red", 1.0 / 10.0, 0.0, 1.0),
];

fn gap(id: &str, target: &Measure, ours: &Measure) -> f64 {
    let g = match id {
        "eye_upper_lash" => target.upper_dark - ours.upper_dark,
        "eye_upper_lash_mm" => target.upper_width - ours.upper_width,
        "eye_lower_lash" => target.lower_dark - ours.lower_dark,
        "eye_lower_lash_mm" => target.lower_width - ours.lower_width,
        "eye_socket" => target.socket - ours.socket,
        "eye_under_eye" => target.under - ours.under,
        "eye_rims" => target.rim_red - ours.rim_red,
        "eye_lid_red" => target.lid_red - ours.lid_red,
        _ => 0.0,
    };
    if g.is_finite() { g } else { 0.0 }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("kotor") => {
            let mut out = read("measure_kotor.json");
            for name in names_of(&args[1..]) {
                let m = measure_kotor(&name);
                println!("{name}: {}", to_json(&m));
                out[name.as_str()] = to_json(&m);
                write("measure_kotor.json", &out);
            }
        }
        Some("ours") => {
            let over: Value = serde_json::from_str(&std::fs::read_to_string(&args[1]).expect("values")).expect("json");
            let mut p = Painter::new();
            let list = head_paint::controls::controls();
            for name in names_of(&args[2..]) {
                let h = head(&name);
                p.set_head(&h.pos);
                let mut v = p.values(&name, &[]);
                if let Some(o) = over[name.as_str()].as_object() {
                    for (id, x) in o {
                        let k = list.iter().position(|c| &c.id == id).expect("control");
                        v[k] = x.as_f64().expect("number");
                    }
                }
                println!("{name}: {}", to_json(&measure_ours(&mut p, &name, &h, &v)));
            }
        }
        Some("fit") => {
            let kotor = read("measure_kotor.json");
            let mut fits = read("fits.json");
            let mut ours_out = read("measure_ours.json");
            let mut p = Painter::new();
            let list = head_paint::controls::controls();
            let at = |id: &str| list.iter().position(|c| c.id == id).unwrap_or_else(|| panic!("no control {id}"));
            for name in names_of(&args[1..]) {
                let Some(target) = from_json(&kotor[name.as_str()]) else {
                    println!("{name}: no KOTOR measure, left at the defaults");
                    continue;
                };
                let h = head(&name);
                p.set_head(&h.pos);
                let mut v = p.values(&name, &[]);
                // A first guess from the measure, then three rounds of corrections from our own measure.
                let first = [
                    ("eye_upper_lash", target.upper_dark / 45.0),
                    ("eye_upper_lash_mm", target.upper_width),
                    ("eye_lower_lash", target.lower_dark / 35.0),
                    ("eye_lower_lash_mm", target.lower_width),
                    ("eye_socket", target.socket),
                    ("eye_under_eye", target.under),
                    ("eye_rims", target.rim_red / 12.0),
                    ("eye_lid_red", target.lid_red / 10.0),
                ];
                for (id, x) in first {
                    let (_, _, lo, hi) = FITTED.iter().find(|f| f.0 == id).copied().expect("fitted");
                    v[at(id)] = if x.is_finite() { x.clamp(lo, hi) } else { list[at(id)].default };
                }
                let mut last = None;
                for _ in 0..3 {
                    let Some(ours) = measure_ours(&mut p, &name, &h, &v) else { break };
                    for (id, gain, lo, hi) in FITTED {
                        let k = at(id);
                        v[k] = (v[k] + gain * gap(id, &target, &ours)).clamp(lo, hi);
                    }
                    last = Some(ours);
                }
                let after = measure_ours(&mut p, &name, &h, &v).or(last);
                let mut entry = serde_json::Map::new();
                for (id, ..) in FITTED {
                    entry.insert(id.to_string(), json!((v[at(id)] * 1000.0).round() / 1000.0));
                }
                println!("{name}: {} -> {}", Value::Object(entry.clone()), to_json(&after));
                fits[name.as_str()] = Value::Object(entry);
                ours_out[name.as_str()] = to_json(&after);
                write("fits.json", &fits);
                write("measure_ours.json", &ours_out);
            }
        }
        Some("time") => {
            // The repaint at 2,048 pixels with the eye layer alone, every layer, and every layer but the eyes: two repaints to warm up,
            // then the median of 9, each from the request until the GPU has finished.
            let mut p = Painter::new();
            for name in names_of(&args[1..]) {
                let h = head(&name);
                p.set_head(&h.pos);
                let v = p.values(&name, &[]);
                let recipe = head_paint::controls::recipe_from(&v);
                for (label, sw) in [("eyes alone", [false, false, false, false, false, true]), ("every layer", [true; 6]), ("every layer but the eyes", [true, true, true, true, true, false])] {
                    let layers = head_paint::controls::layers_from(&sw);
                    let mut ms = Vec::new();
                    for k in 0..11 {
                        let t = std::time::Instant::now();
                        p.painter.paint(N, &recipe, &layers);
                        p.painter.gpu().wait();
                        if k >= 2 {
                            ms.push(t.elapsed().as_secs_f64() * 1000.0);
                        }
                    }
                    println!("{name}: {label}: {:.1} ms", med(&mut ms));
                }
            }
        }
        _ => {
            eprintln!("usage: eye_measure kotor <name>...|all, eye_measure fit <name>...|all, eye_measure ours <values json> <name>...");
            std::process::exit(2);
        }
    }
}
