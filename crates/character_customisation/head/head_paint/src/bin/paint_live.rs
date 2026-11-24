//! Data for the live paint page and for viewer 2's live paint (paint track, part C).
//!
//! `paint_live data <prep dir> <shading fit.json> <skin detail fit.json> <out dir>` writes what the page reads:
//! - `layout/uv.f64`, `layout/tris.u32`, `layout/vmap.u32` and `layout/mesh_tris.u32`, the texture layout as little-endian arrays;
//! - `heads/<name>.f64`, each head's fitted positions in metres;
//! - `recipes.json`, each head's fitted recipe as one value per control id, and the list of heads.
//!
//! The recipes are read from the fit files, so the page starts from the paint that viewer 2 shows. The
//! five heads that have forehead lines set by hand in `LINE_HAND` start with those lines. Every other head starts with the lines' amount at zero.
//! The eye paint comes from the eye fit, the hand-set makeup and the eyeball fit (`EYE_FITS`, `EYE_MAKEUP` and `EYE_BALL`). Only the GPU painter paints it.
//!
//! The prep dir is written by `kotor_bake_prep.py` and holds `common/` and `heads/<name>/`.

#[path = "shared/bundle.rs"]
mod bundle;

use bundle::Bundle;
use head_paint::controls::{DEFAULT_PORES, values_json};
use head_paint::lines::ForeheadLines;
use head_paint::paint::Recipe;
use head_paint::pigment::BaseTone;
use head_paint::shading::Shading;
use head_paint::sheen::Sheen;
use head_paint::wrinkles::{SET_COUNT, SETS, WrinkleSet, Wrinkles};
use serde_json::{Value, json};
use std::path::Path;

/// The forehead lines set by hand for five heads, as `line_hand` reads them.
const LINE_HAND: &str = "/Users/david/.claude/tools/likeness-fit-data/paint/lines/hand.json";

fn number(v: &Value, key: &str) -> f64 {
    v[key].as_f64().unwrap_or_else(|| panic!("the fit has no {key}"))
}

fn read_json(path: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The forehead lines of one head: its hand-set values if it has any, and no lines otherwise.
fn forehead_of(name: &str) -> ForeheadLines {
    let Ok(text) = std::fs::read_to_string(LINE_HAND) else { return ForeheadLines::NONE };
    let hand: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{LINE_HAND}: {e}"));
    let Some(v) = hand["heads"].get(name) else { return ForeheadLines::NONE };
    let (d, g) = (ForeheadLines::NONE, |k: &str, x: f64| v[k].as_f64().unwrap_or(x));
    ForeheadLines {
        amount: g("amount", d.amount),
        spacing: g("spacing", d.spacing),
        continuity: g("continuity", d.continuity),
        waviness: g("waviness", d.waviness),
        variation: g("variation", f64::from(d.variation)).round().max(0.0) as u32,
        depth: g("depth", d.depth),
        sharpness: g("sharpness", d.sharpness),
        ridge: g("ridge", d.ridge),
        dip_mm: g("dip_mm", d.dip_mm),
        reach: g("reach", d.reach),
    }
}

/// Where the eye paint's fitted sliders of each head are, written by `head_paint_gpu`'s `eye_measure fit`, and the makeup set by hand on a
/// few heads, which the measure does not read.
const EYE_FITS: &str = "/Users/david/.claude/tools/likeness-fit-data/paint/eyes/fits.json";
const EYE_MAKEUP: &str = "/Users/david/.claude/tools/likeness-fit-data/paint/eyes/makeup.json";
/// The eyeball's fitted sliders of each head (iris, pupil and eye white), written by `head_paint_gpu`'s `eyeball_fit` and corrected by
/// hand where its measure failed.
const EYE_BALL: &str = "/Users/david/.claude/tools/likeness-fit-data/paint/eyeball/fits.json";

/// The eye paint of one head: the defaults, then the head's fitted sliders, then its makeup. A head missing from both files keeps the
/// defaults.
fn eyes_of(name: &str) -> head_paint::eyes::EyePaint {
    let mut r = Recipe { eyes: head_paint::eyes::EyePaint::DEFAULT, ..head_paint::controls::default_recipe() };
    let list = head_paint::controls::controls();
    for path in [EYE_FITS, EYE_MAKEUP, EYE_BALL] {
        let Ok(text) = std::fs::read_to_string(path) else { continue };
        let v: Value = serde_json::from_str(&text).expect("eye json");
        if let Some(o) = v[name].as_object() {
            for (id, x) in o {
                let c = list.iter().find(|c| &c.id == id).unwrap_or_else(|| panic!("{path}: no control {id}"));
                head_paint::controls::set(&mut r, c.field, x.as_f64().expect("number"));
            }
        }
    }
    r.eyes
}

/// The lip fit of every head, written by the lip agent's `lip_fit` and `make_fit.py`. A head that is missing gets the default lips.
const LIP_FIT: &str = "/Users/david/.claude/tools/likeness-fit-data/paint/lips/fit.json";

fn lips_of(name: &str) -> head_paint::lips::Lips {
    let mut lips = head_paint::lips::Lips::DEFAULT;
    let Ok(text) = std::fs::read_to_string(LIP_FIT) else { return lips };
    let fit: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{LIP_FIT}: {e}"));
    for (k, s) in head_paint::lips::SLIDERS.iter().enumerate() {
        if let Some(v) = fit["heads"][name][s.id].as_f64() {
            *lips.field(k) = v;
        }
    }
    lips
}

/// The recipe of one head, read from the fit files.
fn recipe_of(name: &str, shading: &Value, detail: &Value) -> Recipe {
    let find = |fit: &Value| fit["heads"].as_array().expect("heads").iter().find(|h| h["name"] == name).cloned().unwrap_or_else(|| panic!("{name} is not in a fit"));
    let (fit, d) = (find(shading), find(detail));
    let tone = &fit["base_tone_v3"];
    let w = &d["wrinkles"];
    let mut wrinkles = Wrinkles {
        age: number(w, "age"),
        depth: number(w, "depth"),
        width: number(w, "width"),
        wobble_mm: number(w, "wobble_mm"),
        seed: w["seed"].as_u64().unwrap_or(1),
        fine: w["fine"].as_f64().unwrap_or(0.0),
        sets: [WrinkleSet::DEFAULT; SET_COUNT],
    };
    for (s, entry) in w["sets"].as_array().expect("sets").iter().enumerate().take(SET_COUNT) {
        assert_eq!(entry["id"].as_str(), Some(SETS[s].id), "the sets of the skin detail fit are in another order");
        wrinkles.sets[s].strength = number(entry, "strength");
        wrinkles.sets[s].shift_mm = number(entry, "shift_mm");
    }
    Recipe {
        base_tone: BaseTone {
            chroma_scale: number(tone, "chroma_scale"),
            undertone_degrees: number(tone, "undertone_degrees"),
            unnatural_degrees: number(tone, "unnatural_degrees"),
            ..BaseTone::from_pigments(number(tone, "melanin"), number(tone, "haemoglobin"))
        },
        shading: Shading { strength: number(&fit["shading"], "strength"), highlight_whiteness: number(&fit["shading"], "highlight_whiteness") },
        sheen: Sheen { cheeks: number(&fit["sheen"], "cheeks"), nose: number(&fit["sheen"], "nose"), chin: number(&fit["sheen"], "chin") },
        wrinkles,
        forehead: forehead_of(name),
        lips: lips_of(name),
        brows: head_paint::brows::Brows::DEFAULT,
        // The pores take the GPU pore layer's defaults on every head. The old fit took its pore settings from the noise of the 512-pixel
        // textures (docs/research/2026-11-20-pores-and-grain.md, section 4), and the new layer reads them differently.
        pores: DEFAULT_PORES,
        eyes: eyes_of(name),
        maps: head_paint::maps::LightMaps::DEFAULT,
        skin: head_paint::skin::SkinColour::NONE,
    }
}

fn write_le<T: Copy>(path: &Path, values: &[T], bytes: impl Fn(T) -> Vec<u8>) {
    let out: Vec<u8> = values.iter().flat_map(|&v| bytes(v)).collect();
    std::fs::write(path, out).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "Usage: paint_live data <prep dir> <shading fit.json> <skin detail fit.json> <out dir>";
    if args.len() != 5 || args[0] != "data" {
        eprintln!("{usage}");
        std::process::exit(2);
    }
    let prep = Path::new(&args[1]);
    let (shading, detail) = (read_json(&args[2]), read_json(&args[3]));
    let out = Path::new(&args[4]);
    let common = Bundle::open(&prep.join("common"));
    let names: Vec<String> = shading["heads"].as_array().expect("heads").iter().map(|h| h["name"].as_str().expect("name").to_string()).collect();

    std::fs::create_dir_all(out.join("layout")).expect("out dir");
    std::fs::create_dir_all(out.join("heads")).expect("out dir");
    write_le(&out.join("layout/uv.f64"), &common.f64s("uv"), |v: f64| v.to_le_bytes().to_vec());
    write_le(&out.join("layout/tris.u32"), &common.u32s("tris"), |v: u32| v.to_le_bytes().to_vec());
    write_le(&out.join("layout/vmap.u32"), &common.u32s("vmap"), |v: u32| v.to_le_bytes().to_vec());
    write_le(&out.join("layout/mesh_tris.u32"), &common.u32s("old_tris"), |v: u32| v.to_le_bytes().to_vec());
    let mut recipes = serde_json::Map::new();
    for name in &names {
        let fitted = Bundle::open(&prep.join("heads").join(name)).f64s("fitted");
        write_le(&out.join("heads").join(format!("{name}.f64")), &fitted, |v: f64| v.to_le_bytes().to_vec());
        recipes.insert(name.clone(), values_json(&recipe_of(name, &shading, &detail)));
    }
    let list = json!({
        "note": "Each head's paint recipe as fitted on 2026-11-19, one value per control id of head_paint's controls. Written by head_paint's paint_live tool.",
        "sources": { "prep": args[1], "shading fit": args[2], "skin detail fit": args[3] },
        "heads": recipes,
    });
    std::fs::write(out.join("recipes.json"), serde_json::to_string(&list).expect("json")).expect("recipes.json");
    println!("wrote the layout, {} heads and their recipes to {}", names.len(), out.display());
}
