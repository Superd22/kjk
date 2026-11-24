//! Fits the pigment base tone to the 57 KOTOR base tones (paint track, step C2). This is an analysis tool and never ships.
//!
//! For each head the tool finds the recipe whose colour is closest to the head's base tone, measured in CIEDE2000. It fits twice. The first
//! fit uses melanin and haemoglobin only. The second adds the chroma scale and an undertone limited to a few degrees. The third adds the unnatural colour, to show which heads would want it. Each fit starts from the best entry of a
//! 160 by 160 scan of the pigment range and is refined with Nelder-Mead.
//!
//! Usage: fit_base_tone <targets json> <out fit.json> [undertone limit in degrees]
//! The report holds "heads" and "base tones (Lab)". The tool writes the recipe, colours and differences of both fits, and a sample of the
//! pigment model's reachable colours for the gamut picture.

#[path = "shared/tone_fit.rs"]
mod tone_fit;

use head_paint::delta_e::{delta_e_76, delta_e_2000};
use head_paint::pigment::{HAEMOGLOBIN_MAX, MELANIN_MAX, PigmentTable, linear_to_lab};
use serde_json::{Value, json};
use tone_fit::{Fit, Mode, amounts, fit};

fn fit_json(fit: &Fit, target: [f64; 3]) -> Value {
    let in_gamut = {
        let natural = linear_to_lab(table_free_rgb(&fit.lab));
        delta_e_76(natural, fit.lab) < 0.5
    };
    json!({
        "melanin": fit.tone.melanin,
        "haemoglobin": fit.tone.haemoglobin,
        "chroma_scale": fit.tone.chroma_scale,
        "undertone_degrees": fit.tone.undertone_degrees,
        "unnatural_degrees": fit.tone.unnatural_degrees,
        "lab": fit.lab,
        "delta_e_2000": delta_e_2000(fit.lab, target),
        "delta_e_76": delta_e_76(fit.lab, target),
        "inside_srgb": in_gamut,
    })
}

fn table_free_rgb(lab: &[f64; 3]) -> [f64; 3] {
    let clamped = head_paint::colour::lab_to_srgb(*lab);
    clamped.map(|c| {
        if c > 0.04045 {
            ((c + 0.055) / 1.055).powf(2.4)
        } else {
            c / 12.92
        }
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 && args.len() != 4 {
        eprintln!("usage: fit_base_tone <targets json> <out fit.json> [undertone limit in degrees, default 30]");
        std::process::exit(2);
    }
    let report: Value =
        serde_json::from_str(&std::fs::read_to_string(&args[1]).expect("read report"))
            .expect("parse report");
    let names: Vec<String> = report["heads"]
        .as_array()
        .expect("heads")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let targets: Vec<[f64; 3]> = report["base tones (Lab)"]
        .as_array()
        .expect("base tones")
        .iter()
        .map(|v| {
            let a = v.as_array().unwrap();
            [
                a[0].as_f64().unwrap(),
                a[1].as_f64().unwrap(),
                a[2].as_f64().unwrap(),
            ]
        })
        .collect();
    let cap: f64 = args.get(3).map_or(30.0, |a| a.parse().expect("limit"));
    let table = PigmentTable::build();
    let mut heads = Vec::new();
    for (i, (name, target)) in names.iter().zip(&targets).enumerate() {
        let plain = fit(&table, *target, Mode::Pigments);
        let tinted = fit(&table, *target, Mode::Undertone(cap));
        let strange = fit(&table, *target, Mode::Unnatural(cap));
        heads.push(json!({
            "number": i + 1,
            "name": name,
            "target_lab": target,
            "pigments_only": fit_json(&plain, *target),
            "base_tone_v2": fit_json(&tinted, *target),
            "with_unnatural": fit_json(&strange, *target),
        }));
    }
    let mut gamut = Vec::new();
    let steps = 60;
    for i in 0..steps {
        for j in 0..steps {
            let (m, h) = amounts([i as f64 / (steps - 1) as f64, j as f64 / (steps - 1) as f64]);
            gamut.push(json!([m, h, linear_to_lab(table.lookup(m, h))]));
        }
    }
    let out = json!({ "heads": heads, "gamut": gamut, "melanin_max": MELANIN_MAX, "haemoglobin_max": HAEMOGLOBIN_MAX, "undertone_limit": cap });
    std::fs::write(&args[2], serde_json::to_string(&out).unwrap()).expect("write fit.json");
    let de: Vec<(f64, f64)> = heads
        .iter()
        .map(|h| {
            (
                h["pigments_only"]["delta_e_2000"].as_f64().unwrap(),
                h["base_tone_v2"]["delta_e_2000"].as_f64().unwrap(),
            )
        })
        .collect();
    println!(
        "mean delta E 2000: pigments only {:.2}, base tone v2 {:.2}",
        de.iter().map(|d| d.0).sum::<f64>() / 57.0,
        de.iter().map(|d| d.1).sum::<f64>() / 57.0
    );
}
