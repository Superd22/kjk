//! The animation calibration (roadmap decision 62). It learns, from our fitted heads and KOTOR's own motion, the rules that compute
//! a head's skin weights from its shape. Run it after a fit when the rig should follow the new fit. It is not part of the fit.
//!
//! First it learns the regressor from all heads (`regressor.json`, see `head_rig::calibrate::regress`). Then, for each head of
//! `<viewer data>/common.json` that is not left out, the bones are placed by that regressor, each vertex is matched to KOTOR's face, and
//! KOTOR's motion over the lip sync shapes and 6 frames of each expression clip becomes the fit's data. It writes into `<out dir>`:
//! - `weights_model.json`: the rule, in the format `mouth_rig --weights` reads;
//! - `calibration.json`: the fingerprint of each head's fitted positions, so viewer 2 can tell when a head was fitted again after the
//!   calibration, and the test results.
//!
//! Usage: calibrate <viewer data dir> <template.json> <out dir> [--brows <smoothness>] [--neck <smoothness>] [--leave-out <name>,<name>]
//! The template is written by `calibration_template.py` in the likeness tools.

use head_rig::bones::{key_scale, relocate};
use head_rig::calibrate::data::{Common, Head, Template};
use head_rig::calibrate::kotor::match_face;
use head_rig::calibrate::model::{SETTINGS, Sample, calibrate, to_file};
use head_rig::calibrate::pose::poses;
use head_rig::calibrate::solve::normal_equations;
use head_rig::calibrate::{regions, regress};
use head_rig::regressor::Regressor;
use serde_json::json;
use std::path::Path;
use std::time::Instant;

/// FNV-1a over the positions as 32-bit floats, as viewer 2 holds them.
fn fingerprint(pos: &[[f64; 3]]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for x in pos.iter().flatten() {
        for b in (*x as f32).to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    format!("{h:016x}")
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let smoothness: Option<f64> = args.iter().position(|a| a == "--brows").map(|i| {
        let v = args[i + 1].parse().expect("--brows <smoothness>");
        args.drain(i..i + 2);
        v
    });
    let neck_smoothness: Option<f64> = args.iter().position(|a| a == "--neck").map(|i| {
        let v = args[i + 1].parse().expect("--neck <smoothness>");
        args.drain(i..i + 2);
        v
    });
    let leave_out: Vec<String> = match args.iter().position(|a| a == "--leave-out") {
        Some(i) => {
            let v = args.get(i + 1).cloned().unwrap_or_else(String::new);
            args.drain(i..(i + 2).min(args.len()));
            v.split(',').map(str::to_string).collect()
        }
        None => vec![],
    };
    let [data, template, out] = &args[..] else {
        eprintln!(
            "usage: calibrate <viewer data dir> <template.json> <out dir> [--brows <smoothness>] [--neck <smoothness>] [--leave-out <name>,<name>]"
        );
        std::process::exit(2);
    };
    let started = Instant::now();
    let data = Path::new(data);
    let common = Common::read(data);
    let template = Template::read(Path::new(template));
    let heads: Vec<Head> = common
        .heads
        .iter()
        .map(|n| Head::read(data, &common, n))
        .collect();
    let (reg_file, reg_report) = regress::learn(&common, &template, &heads);
    for line in &reg_report {
        println!("regressor: {line}");
    }
    let out = Path::new(out);
    std::fs::create_dir_all(out).unwrap();
    std::fs::write(
        out.join("regressor.json"),
        serde_json::to_string(&reg_file).unwrap(),
    )
    .unwrap();
    let reg: Regressor = serde_json::from_value(reg_file).expect("regressor");
    let brows = smoothness.map(|smoothness| regions::Brows { smoothness });
    let neck = neck_smoothness.map(|smoothness| regions::Neck { smoothness });
    let region = regions::face(&common, &template, brows, neck);
    println!(
        "{} free vertices, {} bones",
        region.problem.free.len(),
        region.problem.bones.len()
    );
    let frames = poses(&common, 6);
    let mut samples = Vec::new();
    let mut prints = serde_json::Map::new();
    for head in heads {
        let name = &head.name;
        prints.insert(name.clone(), json!(fingerprint(&head.pos)));
        if leave_out.contains(name) {
            continue;
        }
        let places = reg.place(&head.pos).expect("the regressor's mesh");
        let bones = relocate(&head.bones, &places);
        let s = key_scale(&places, &reg.reference).expect("key scale");
        let matches = match_face(&head, &common, &template.upper, &template.side);
        let equations = normal_equations(
            &head,
            &bones,
            [s; 3],
            &frames,
            &region.problem,
            &matches,
            0.0025,
        );
        let with_target = equations.count.iter().filter(|c| **c > 0.0).count();
        println!(
            "{name}: {with_target} of {} vertices have KOTOR targets",
            region.problem.free.len()
        );
        samples.push(Sample {
            name: name.clone(),
            pos: head.pos.clone(),
            equations,
        });
    }
    println!("data read in {:.1} s", started.elapsed().as_secs_f64());
    let (rule, report) = calibrate(&region, &samples, &SETTINGS);
    println!("face: {report}");
    std::fs::write(
        out.join("weights_model.json"),
        serde_json::to_string(&to_file(&region, &rule, common.n, &common.bone_names)).unwrap(),
    )
    .unwrap();
    std::fs::write(
        out.join("calibration.json"),
        serde_json::to_string_pretty(&json!({
            "parts": (["mouth"].into_iter().chain(brows.map(|_| "brows")).chain(neck.map(|_| "neck")).collect::<Vec<_>>()),
            "heads": prints,
            "learnedFrom": samples.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
            "results": {"face": report, "regressor": reg_report},
        }))
        .unwrap(),
    )
    .unwrap();
    println!("done in {:.1} s", started.elapsed().as_secs_f64());
}
