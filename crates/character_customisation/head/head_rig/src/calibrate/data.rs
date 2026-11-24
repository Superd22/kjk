//! Viewer 2's data, as the calibration reads it. `common.json` holds the bones, KOTOR's clips and the base skin weights. Each head's file
//! holds our fitted positions and KOTOR's own mesh, and `kskin/<name>.json` holds KOTOR's skin, bones and the turn into our frame.

use crate::V3;
use crate::bones::Bone;
use base64::Engine;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub fn read_json(path: &Path) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn bytes(v: &Value) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(v.as_str().expect("base64 string"))
        .expect("base64")
}

pub fn f32s(v: &Value) -> Vec<f64> {
    bytes(v)
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c) as f64)
        .collect()
}

fn u16s(v: &Value) -> Vec<usize> {
    bytes(v)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c) as usize)
        .collect()
}

fn points(values: &[f64]) -> Vec<V3> {
    values.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
}

fn triple(v: &Value) -> V3 {
    [0, 1, 2].map(|k| v[k].as_f64().expect("number"))
}

#[derive(Clone, Debug, Default)]
pub struct Track {
    pub p: Vec<(f64, V3)>,
    pub r: Vec<(f64, [f64; 4])>,
}

#[derive(Clone, Debug)]
pub struct Clip {
    pub length: f64,
    pub tracks: BTreeMap<String, Track>,
}

pub struct Common {
    pub n: usize,
    pub bone_names: Vec<String>,
    pub parents: Vec<Option<usize>>,
    pub clips: BTreeMap<String, Clip>,
    pub lip_line: usize,
    pub heads: Vec<String>,
    /// The base skin of every vertex as dense weights over the bones, from `build_rig.py`.
    pub base_skin: Vec<Vec<f64>>,
    pub edges: Vec<[usize; 2]>,
}

fn keys<const N: usize>(v: Option<&Value>) -> Vec<(f64, [f64; N])> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|k| {
                    (
                        k[0].as_f64().unwrap(),
                        std::array::from_fn(|i| k[i + 1].as_f64().unwrap()),
                    )
                })
                .collect()
        })
        .unwrap_or_else(Vec::new)
}

impl Common {
    pub fn read(dir: &Path) -> Common {
        let c = read_json(&dir.join("common.json"));
        let n = c["n"].as_u64().unwrap() as usize;
        let bones = c["bones"].as_array().unwrap();
        let bone_names: Vec<String> = bones
            .iter()
            .map(|b| b["name"].as_str().unwrap().to_string())
            .collect();
        let parents = bones
            .iter()
            .map(|b| b["parent"].as_i64().and_then(|p| usize::try_from(p).ok()))
            .collect();
        let clips = c["clips"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, cl)| {
                let tracks = cl["tracks"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(b, t)| {
                        (
                            b.clone(),
                            Track {
                                p: keys::<3>(t.get("p")),
                                r: keys::<4>(t.get("r")),
                            },
                        )
                    })
                    .collect();
                (
                    name.clone(),
                    Clip {
                        length: cl["length"].as_f64().unwrap(),
                        tracks,
                    },
                )
            })
            .collect();
        let si = bytes(&c["skinIndex"]);
        let sw = f32s(&c["skinWeight"]);
        let base_skin = (0..n)
            .map(|v| {
                let mut w = vec![0.0; bone_names.len()];
                for k in 0..4 {
                    w[si[4 * v + k] as usize] += sw[4 * v + k];
                }
                w
            })
            .collect();
        let e = u16s(&c["edges"]);
        Common {
            n,
            clips,
            lip_line: c["lipLine"].as_u64().unwrap() as usize,
            heads: c["heads"]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| h["name"].as_str().unwrap().to_string())
                .collect(),
            base_skin,
            edges: e.chunks(2).map(|p| [p[0], p[1]]).collect(),
            bone_names,
            parents,
        }
    }

    pub fn bone(&self, name: &str) -> usize {
        self.bone_names
            .iter()
            .position(|b| b == name)
            .unwrap_or_else(|| panic!("no bone {name}"))
    }

    fn bones_from(&self, list: &Value) -> Vec<Bone> {
        list.as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, b)| Bone {
                name: self.bone_names[i].clone(),
                parent: self.parents[i],
                local: triple(&b["local"]),
                rot: [0, 1, 2, 3].map(|k| b["rot"][k].as_f64().unwrap()),
            })
            .collect()
    }
}

/// One head: our fitted head and KOTOR's own head, skinned, in our frame through `to_fit`.
pub struct Head {
    pub name: String,
    pub pos: Vec<V3>,
    /// Our bones as the fit wrote them, before the regressor places them.
    pub bones: Vec<Bone>,
    /// KOTOR's mesh as unshared triangle corners.
    pub kpos: Vec<V3>,
    pub kidx: Vec<[u8; 4]>,
    pub kw: Vec<[f64; 4]>,
    pub kbones: Vec<Bone>,
    pub kkey_scale: V3,
    /// KOTOR's model space into our frame: three rows of an affine map.
    pub to_fit: [[f64; 4]; 3],
}

impl Head {
    pub fn read(dir: &Path, common: &Common, name: &str) -> Head {
        let d = read_json(&dir.join("heads").join(format!("{name}.json")));
        let k = read_json(&dir.join("kskin").join(format!("{name}.json")));
        let kpos = d["kotor"]["materials"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|m| points(&f32s(&m["pos"])))
            .collect();
        let mut kidx = Vec::new();
        let mut kw = Vec::new();
        for m in k["materials"].as_array().unwrap() {
            kidx.extend(bytes(&m["idx"]).chunks(4).map(|c| [c[0], c[1], c[2], c[3]]));
            kw.extend(f32s(&m["w"]).chunks(4).map(|c| [c[0], c[1], c[2], c[3]]));
        }
        let t = &k["toFit"];
        Head {
            name: name.to_string(),
            pos: points(&f32s(&d["ours"]["pos"])),
            bones: common.bones_from(&d["ours"]["bones"]),
            kpos,
            kidx,
            kw,
            kbones: common.bones_from(&k["bones"]),
            kkey_scale: triple(&k["keyScale"]),
            to_fit: std::array::from_fn(|r| std::array::from_fn(|c| t[r][c].as_f64().unwrap())),
        }
    }
}

/// The base head in viewer 2's vertex order, written by `calibration_template.py` in the likeness tools.
pub struct Template {
    pub pos: Vec<V3>,
    pub landmarks: BTreeMap<String, V3>,
    pub mouth_y: f64,
    /// 1 on the upper lip's side of the mouth opening, 0 on the lower lip's.
    pub upper: Vec<f64>,
    /// How strongly the side rule applies, 1 near the opening.
    pub side: Vec<f64>,
    /// The vertices inside the mouth, behind the lips.
    pub mouth_inside: Vec<usize>,
}

impl Template {
    pub fn read(path: &Path) -> Template {
        let t = read_json(path);
        let list = |k: &str| -> Vec<f64> {
            t[k].as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
                .collect()
        };
        Template {
            pos: t["pos"].as_array().unwrap().iter().map(triple).collect(),
            landmarks: t["landmarks"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), triple(v)))
                .collect(),
            mouth_y: t["mouthY"].as_f64().unwrap(),
            upper: list("upper"),
            side: list("side"),
            mouth_inside: list("mouthInside").iter().map(|&v| v as usize).collect(),
        }
    }
}
