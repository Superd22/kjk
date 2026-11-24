//! The slider kit's tool. Usage: `sliders <feature> <command> [data folder]`, the data folder by default
//! `~/.claude/tools/likeness-fit-data/sliders`, which must hold `input/` from `prep_nose.py` and `prep_extra.py`.
//!
//! Features: `nose`, `jaw`. Commands:
//! - `all`: `marks`, then `make`, then `bench`, then `sheet`: the whole chain from the inputs to the page's data.
//! - `marks`: check 1 of the measure and the provisional pass marks, and checks 3 and 4 (copies, shared meshes, kinds, most alike and
//!   most different pairs), written to the feature's marks and checks files. It prints each measure's wobble against KOTOR's spread.
//! - `needs`: per measure, the base head, KOTOR's spread, and how far each KOTOR head lies from its own stand-in.
//! - `make`: the named sliders (`SLIDERS_ONLY=key,key` remakes some into `<feature>/sliders_try.json`).
//! - `presets`: the quick mode, for use during a round. The 57 heads are solved once with all the sliders and ranges from every
//!   head, and the presets, the page's data and a short report are written where `bench` writes them.
//! - `bench`: the full grading, for the end of a round. The 57 heads are solved held out and with ranges from every head, then come
//!   the scorecard, the caps, the safety checks, the leak table and the page's data. `SLIDERS_SHEETS=<folder>` also draws the
//!   sampled heads there.
//! - `sheet`: each slider at its low end, 0 and its high end on the base head, in every shot of the feature
//!   (`SLIDERS_SHEET=<file.png>`, by default `<feature>_sheet.png` in the data folder).
//! - `kotor-sheet`: KOTOR's own meshes of the 57 heads in the feature's shots, with what is not skin in blue
//!   (`SLIDERS_SHEET=<file.png>`).
//!
//! `sliders all-features starts` writes the stand-ins with every feature's own shape taken out, for the page's live sliders.

use head_sliders::bench;
use head_sliders::checks::Marks;
use head_sliders::feature::by_key;
use head_sliders::heads::World;
use head_sliders::mesh::input::Input;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let key = args.get(1).map(String::as_str).unwrap_or("nose");
    let command = args.get(2).map(String::as_str).unwrap_or("needs");
    let data = args
        .get(3)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap()).join(".claude/tools/likeness-fit-data/sliders"));
    let input = Input::open(&data.join("input"));
    if key == "underjaw" {
        if !head_sliders::under_jaw::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "nosebase" {
        if !head_sliders::nose_base::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "noseprofile" {
        if !head_sliders::nose_profile::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "bald" {
        if !head_sliders::bald::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "skull" {
        if !head_sliders::skull::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "jawskull" {
        if !head_sliders::jaw_skull::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "lips" {
        if !head_sliders::lips::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "chinneck" {
        if !head_sliders::chin_neck::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "throatcorner" {
        if !head_sliders::throat_corner::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "jawedge" {
        if !head_sliders::jaw_edge::tools::run(command, &args, &data, &input) {
            panic!("unknown command {command}");
        }
        return;
    }
    if key == "all-features" {
        head_sliders::bench::all_starts(&input, &data);
        return;
    }
    let feature = by_key(key).unwrap_or_else(|| panic!("unknown feature {key}"));
    if key == "eyes" && head_sliders::eyes::tools::run(command, &args, &data, &input, &|| World::load(&input, feature)) {
        return;
    }
    if command == "leaks" {
        head_sliders::brow::tools::leaks(&input, &data, feature);
        return;
    }
    if command == "probe-sheet" {
        // KOTOR's mesh and the same subdivided twice, with the measure's dots, in the feature's shots, for the heads named after the
        // data folder: `sliders <feature> probe-sheet <data> <head> <head> ...` (SLIDERS_SHEET names the picture)
        use head_sliders::render::{clay, dots, label, sheet};
        let shots = feature.shots();
        let mut tiles = Vec::new();
        for name in &args[4..] {
            let e = input.points(&format!("heads/{name}/eyes"));
            let frame = head_sliders::measure::head_frame([e[0], e[1]]);
            let v = input.points(&format!("heads/{name}/kotor_v"));
            let f = input.triangles(&format!("heads/{name}/kotor_f"));
            let (sv, sf, _) = head_sliders::checks::butterfly(&v, &f);
            let (sv, sf, _) = head_sliders::checks::butterfly(&sv, &sf);
            let fit = input.points(&format!("heads/{name}/fit_smooth"));
            let mesh = input.head_mesh();
            for (vv, ff, tag) in [(&v, &f, "kotor"), (&sv, &sf, "subdivided"), (&fit, &mesh.triangles, "ours")] {
                let m = feature.measure(&head_sliders::measure::Surface::within(vv, ff, &frame, &feature.window()));
                let marked: Vec<head_sliders::V3> = m.as_ref().ok().and_then(|m| serde_json::from_value(m.marks["dots"].clone()).ok()).unwrap_or_default();
                let local: Vec<head_sliders::V3> = vv.iter().map(|&p| frame.local(p)).collect();
                for (k, s) in shots.iter().enumerate() {
                    let mut img = clay(&local, ff, s, &[], &[]);
                    dots(&mut img, s, &marked);
                    if k == 0 {
                        label(&mut img, &format!("{} {tag}", name.replace('_', " ")));
                    }
                    tiles.push(img);
                }
            }
        }
        let out = std::env::var("SLIDERS_SHEET").unwrap_or_else(|_| "probe.png".into());
        sheet(&tiles, 3 * shots.len()).save(&out).unwrap();
        return;
    }
    if command == "probe" {
        // one head's KOTOR mesh (or "base" for the base head), measured alone: `sliders <feature> probe <data> <head>`
        let name = args.get(4).map(String::as_str).unwrap_or("base");
        let (v, f, frame) = if name == "base" {
            let mesh = input.head_mesh();
            let sm = head_sliders::set::smoother(&mesh.base, &mesh.triangles, &mesh.keep);
            (sm.smooth(&mesh.base, head_sliders::set::SMOOTHING), mesh.triangles.clone(), head_sliders::measure::Frame::from_eyes(mesh.eyes))
        } else {
            let e = input.points(&format!("heads/{name}/eyes"));
            (input.points(&format!("heads/{name}/kotor_v")), input.triangles(&format!("heads/{name}/kotor_f")), head_sliders::measure::head_frame([e[0], e[1]]))
        };
        let m = feature.measure(&head_sliders::measure::Surface::within(&v, &f, &frame, &feature.window()));
        match m {
            Ok(m) => {
                for (p, x) in feature.proportions().iter().zip(&m.proportions) {
                    println!("{:18} {x:8.2}", p.key);
                }
                for (o, l) in feature.outlines().iter().zip(&m.outlines) {
                    println!("{}: {}", o.key, l.iter().map(|p| format!("({:.1},{:.1})", p[0], p[1])).collect::<Vec<_>>().join(" "));
                }
            }
            Err(e) => println!("error: {e}"),
        }
        return;
    }
    let t0 = std::time::Instant::now();
    let world = World::load(&input, feature);
    eprintln!("{} heads in {:.1?}", world.heads.len(), t0.elapsed());
    let marks_path = data.join(feature.files().marks);
    let sheet_path = std::env::var("SLIDERS_SHEET").unwrap_or_else(|_| data.join(format!("{key}_sheet.png")).display().to_string());
    match command {
        "marks" => {
            bench::marks(&world, &input, &data);
        }
        "needs" => bench::needs(&world),
        "make" => {
            bench::make(&world, &Marks::read(&marks_path), &data);
        }
        "bench" => bench::bench(&world, &Marks::read(&marks_path), &data),
        "presets" => bench::presets(&world, &Marks::read(&marks_path), &data),
        "sheet" => {
            let set = serde_json::from_str(&std::fs::read_to_string(data.join(feature.files().sliders)).unwrap()).unwrap();
            bench::slider_sheet(&world, &set).save(&sheet_path).unwrap();
            eprintln!("wrote {sheet_path}");
        }
        "made-sheet" => {
            // `sliders <feature> made-sheet <data> <head> ...`, with SLIDERS_MODE=held_out for the ranges set without the head
            let mode = std::env::var("SLIDERS_MODE").unwrap_or_else(|_| "full".into());
            bench::made_sheet(&world, &data, &args[4..], &mode).save(&sheet_path).unwrap();
            eprintln!("wrote {sheet_path}");
        }
        "zone-sheet" => {
            bench::zone_sheet(&world).save(&sheet_path).unwrap();
            eprintln!("wrote {sheet_path}");
        }
        "kotor-sheet" => {
            bench::kotor_sheet(&world).save(&sheet_path).unwrap();
            eprintln!("wrote {sheet_path}");
        }
        "all" => {
            let m = bench::marks(&world, &input, &data);
            let set = bench::make(&world, &m, &data);
            bench::bench(&world, &m, &data);
            bench::slider_sheet(&world, &set).save(&sheet_path).unwrap();
            eprintln!("wrote {sheet_path}");
        }
        other => panic!("unknown command {other}"),
    }
}
