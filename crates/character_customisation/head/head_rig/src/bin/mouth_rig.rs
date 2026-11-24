//! Writes the face rig of every head into the head viewer's data (roadmap A1). Viewer 2's rig is written with both options below,
//! with the weights rule of the animation calibration (the `calibrate` tool), by `calibrate.sh` in the likeness tools. Without the
//! options the tool writes the older "New mouth" rig, which was removed from the viewer on 2026-11-20.
//!
//! For each head of `<viewer data>/common.json`, it reads our head's positions and bones from `heads/<name>.json`, places the face bones
//! and the teeth points with the regressor, and writes `<viewer data>/<out dir>/<name>.json`:
//! - `bones`: each bone's local place and rest turn, in the order of `common.json`'s bones;
//! - `keyScale`: the factor KOTOR's clip offsets are scaled by on this head, the same on every axis;
//! - `teeth`: an upper strip moved by `head_g` and a lower strip moved by `f_jaw_g`, as unshared triangle corners with texture
//!   coordinates, and the texture's file under `tex/`.
//!
//! It also writes the teeth texture to `<viewer data>/tex/<texture>`.
//!
//! Two options, before the paths, make mouth v2:
//! - `--weights <model.json>` computes the skin weights of the mouth from each head's own shape (`head_rig::weights`) and writes them
//!   in the head's file as `skin`: the vertices, their four bone numbers in `common.json`'s order and their weights;
//! - `--lips <lips.json>` moves each teeth strip so that its biting edge sits where KOTOR's sits against the lip edges, and writes `shade`:
//!   the inside vertices and the colour their paint is multiplied by (`head_rig::shade`). When the file lists the vertices near the
//!   opening, it also opens the lip line (`head_rig::lips`) and writes the moves as `posDelta`, added to whichever shape is shown. The
//!   bones and weights are placed on the head before the lip line opens, the teeth and the shade after.
//!
//! Usage: mouth_rig [--weights <model.json>] [--lips <lips.json>] <viewer data dir> <regressor.json> <out dir> <texture file>

use base64::Engine;
use head_rig::V3;
use head_rig::regressor::Regressor;
use head_rig::rig::{Lips, RigKit, TEETH_TEXTURE, teeth_texture};
use head_rig::weights::{Model, ModelFile};
use image::{ExtendedColorType, ImageEncoder};
use serde_json::{Value, json};
use std::path::Path;

fn read_json(path: &Path) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn f32s(v: &Value) -> Vec<f32> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(v.as_str().expect("base64 string"))
        .expect("base64");
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect()
}

fn f32_b64(values: impl Iterator<Item = f64>) -> String {
    let bytes: Vec<u8> = values.flat_map(|x| (x as f32).to_le_bytes()).collect();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn triple(v: &Value) -> V3 {
    [0, 1, 2].map(|k| v[k].as_f64().expect("number"))
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut take = |flag: &str| -> Option<String> {
        let i = args.iter().position(|a| a == flag)?;
        let v = args.get(i + 1).cloned();
        args.drain(i..(i + 2).min(args.len()));
        v
    };
    let model = take("--weights").map(|p| {
        Model::from_file(
            serde_json::from_value::<ModelFile>(read_json(Path::new(&p))).expect("weights model"),
        )
    });
    let lips = take("--lips")
        .map(|p| serde_json::from_value::<Lips>(read_json(Path::new(&p))).expect("lips"));
    let [data, reg, out_dir, tex] = &args[..] else {
        eprintln!(
            "usage: mouth_rig [--weights <model.json>] [--lips <lips.json>] <viewer data dir> <regressor.json> <out dir> <texture file>"
        );
        std::process::exit(2);
    };
    let data = Path::new(data);
    let reg: Regressor = serde_json::from_value(read_json(Path::new(reg))).expect("regressor");
    let common = read_json(&data.join("common.json"));
    let names: Vec<String> = common["bones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["name"].as_str().unwrap().to_string())
        .collect();
    let parents: Vec<Option<usize>> = common["bones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["parent"].as_i64().and_then(|p| usize::try_from(p).ok()))
        .collect();
    let kit = RigKit::new(names, parents, reg, model, lips);
    std::fs::create_dir_all(data.join(out_dir)).unwrap();
    let tex_path = data.join("tex").join(tex);
    std::fs::create_dir_all(tex_path.parent().unwrap()).unwrap();
    let (tw, th) = TEETH_TEXTURE;
    let file = std::fs::File::create(&tex_path).unwrap();
    image::codecs::png::PngEncoder::new(file)
        .write_image(
            &teeth_texture(),
            tw as u32,
            th as u32,
            ExtendedColorType::Rgb8,
        )
        .unwrap();
    for h in common["heads"].as_array().unwrap() {
        let name = h["name"].as_str().unwrap();
        let d = read_json(&data.join("heads").join(format!("{name}.json")));
        let o = &d["ours"];
        let p = f32s(&o["pos"]);
        let pos: Vec<V3> = p
            .chunks(3)
            .map(|c| [c[0] as f64, c[1] as f64, c[2] as f64])
            .collect();
        let base: Vec<(V3, [f64; 4])> = o["bones"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                (
                    triple(&b["local"]),
                    [0, 1, 2, 3].map(|k| b["rot"][k].as_f64().unwrap()),
                )
            })
            .collect();
        let rig = match kit.rig(&pos, &base) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{name}: skipped, {e:?}");
                continue;
            }
        };
        let s = rig.key_scale;
        let teeth: Vec<Value> = rig
            .teeth
            .iter()
            .map(|t| {
                json!({
                    "bone": t.bone,
                    "pos": f32_b64(t.positions.iter().map(|&x| x as f64)),
                    "uv": f32_b64(t.uv.iter().map(|&x| x as f64)),
                    "texture": tex,
                })
            })
            .collect();
        let mut out = json!({
            "bones": rig.bones.iter().map(|b| json!({"local": b.local, "rot": b.rot})).collect::<Vec<_>>(),
            "keyScale": [s, s, s],
            "teeth": teeth,
        });
        if let Some(t) = &rig.tongue {
            out["tongue"] = json!({
                "pos": f32_b64(t.positions.iter().map(|&x| x as f64)),
                "weight": f32_b64(t.weight.iter().map(|&x| x as f64)),
                "colour": f32_b64(t.colour.iter().map(|&x| x as f64)),
            });
        }
        if let Some(k) = &rig.skin {
            let vs: Vec<u8> = k.vertices.iter().flat_map(|v| v.to_le_bytes()).collect();
            let sw: Vec<u8> = k.weight.iter().flat_map(|x| x.to_le_bytes()).collect();
            out["skin"] =
                json!({"vertices": b64(&vs), "skinIndex": b64(&k.index), "skinWeight": b64(&sw)});
        }
        if let Some(l) = &rig.lift {
            let vs: Vec<u8> = l.vertices.iter().flat_map(|v| v.to_le_bytes()).collect();
            out["posDelta"] = json!({"vertices": b64(&vs), "delta": f32_b64(l.up.iter().flat_map(|&d| [0.0, d as f64, 0.0]))});
        }
        if let Some(sh) = &rig.shade {
            let vs: Vec<u8> = sh.vertices.iter().flat_map(|v| v.to_le_bytes()).collect();
            out["shade"] = json!({"vertices": b64(&vs), "colour": f32_b64(sh.colour.iter().map(|&x| x as f64))});
        }
        std::fs::write(
            data.join(out_dir).join(format!("{name}.json")),
            serde_json::to_string(&out).unwrap(),
        )
        .unwrap();
        println!("{name} key scale {s:.3}");
    }
}
