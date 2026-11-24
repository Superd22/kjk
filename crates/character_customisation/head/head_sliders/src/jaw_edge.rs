//! The zone under the jaw that the throat construction (`chin_neck::ThroatShape`) may move, and the tools' heads built as viewer 2
//! builds them (roadmap B3 and B5 for the jaw and chin).
//!
//! The zone is 0 on every vertex the smoothing holds (the neck's rim and below, the ears, the inside of the mouth) and on the lips,
//! and fades in within RIM_FADE of the rim and LIP_FADE of the lips, so the rim never moves (decision 44). The jaw edge prototype
//! that first used it (four values: Jaw corner height, Jaw edge depth, Throat angle and Neck width) was deleted on 2026-11-22, when
//! the throat construction became viewer 2's default.

use crate::V3;
use crate::mesh::HeadMesh;
#[cfg(not(target_arch = "wasm32"))]
use crate::measure::Frame;

/// How far from the neck's rim the zone fades in, in mm.
const RIM_FADE: f64 = 14.0;
/// How far from the lips the zone fades in, in mm.
const LIP_FADE: f64 = 8.0;

pub fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The throat's zone on one head mesh.
pub struct ThroatZone {
    /// Per vertex: 0 on held vertices and the lips, fading in from the rim and the lips.
    pub weight: Vec<f64>,
}

impl ThroatZone {
    pub fn new(mesh: &HeadMesh) -> Self {
        let frame = crate::measure::Frame::head();
        let n = mesh.merged_count();
        let local: Vec<V3> = mesh.first.iter().map(|&i| frame.local(mesh.base[i as usize])).collect();
        let mouth = 1 + crate::mesh::ZONES.iter().position(|z| *z == "mouth").unwrap() as u8;
        let held: Vec<bool> = (0..n).map(|g| mesh.keep[mesh.first[g] as usize] && !mesh.hidden[g]).collect();
        let lips: Vec<bool> = (0..n).map(|g| mesh.zone[mesh.first[g] as usize] == mouth || mesh.hidden[g]).collect();
        let near = |fence: &[bool]| -> Vec<f64> {
            let f: Vec<V3> = local.iter().zip(fence).filter(|(_, x)| **x).map(|(p, _)| *p).collect();
            local.iter().map(|p| f.iter().map(|q| crate::length(crate::sub(*p, *q))).fold(f64::INFINITY, f64::min)).collect()
        };
        let (dh, dl) = (near(&held), near(&lips));
        let merged: Vec<f64> = (0..n)
            .map(|g| if held[g] || lips[g] { 0.0 } else { smoothstep(dh[g] / RIM_FADE) * smoothstep(dl[g] / LIP_FADE) })
            .collect();
        let weight = mesh.group.iter().map(|&g| merged[g as usize]).collect();
        Self { weight }
    }

    /// The chin's front (depth) and bottom (height) of a head, in the frame of the eyes.
    pub fn chin(&self, local: &[V3]) -> (f64, f64) {
        let pog = local
            .iter()
            .zip(&self.weight)
            .filter(|(q, w)| **w > 0.0 && q[0].abs() < 6.0 && q[1] < -85.0 && q[1] > -130.0)
            .map(|(q, _)| q[2])
            .fold(f64::NEG_INFINITY, f64::max);
        let bottom = local
            .iter()
            .zip(&self.weight)
            .filter(|(q, w)| **w > 0.0 && q[0].abs() < 10.0 && q[2] > pog - 25.0 && q[1] < -80.0)
            .map(|(q, _)| q[1])
            .fold(f64::INFINITY, f64::min);
        (pog, bottom)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod tools {
    use super::*;
    use crate::mesh::input::Input;
    use std::path::Path;

    /// Runs a command of the throat's zone, or says it is not one: `sliders jawedge <command> <data> ...`.
    pub fn run(command: &str, _args: &[String], _data: &Path, input: &Input) -> bool {
        match command {
            // the base head's chin, rim and neck, in the frame of the eyes: `probe <data>`
            "probe" => {
                let mesh = input.head_mesh();
                let sm = crate::set::smoother(&mesh.base, &mesh.triangles, &mesh.keep);
                let base = sm.smooth(&mesh.base, crate::set::SMOOTHING);
                let proto = ThroatZone::new(&mesh);
                let frame = Frame::head();
                let local: Vec<V3> = base.iter().map(|&p| frame.local(p)).collect();
                let (pog, bottom) = proto.chin(&local);
                println!("chin front {pog:.1}, bottom {bottom:.1}");
                let rim: Vec<&V3> = local.iter().zip(&mesh.keep).filter(|(q, k)| **k && q[1] < -100.0 && q[0].abs() < 60.0).map(|(q, _)| q).collect();
                let hi = rim.iter().map(|q| q[1]).fold(f64::NEG_INFINITY, f64::max);
                let front = rim.iter().map(|q| q[2]).fold(f64::NEG_INFINITY, f64::max);
                println!("held low vertices {}, highest {hi:.1} ({:.1} below the chin), front-most {front:.1}", rim.len(), bottom - hi);
                let mut front_rim: Vec<&V3> = local.iter().zip(&mesh.keep).filter(|(q, k)| **k && q[1] < bottom + 5.0 && q[2] > pog - 90.0).map(|(q, _)| q).collect();
                front_rim.sort_by(|a, b| a[0].total_cmp(&b[0]));
                front_rim.dedup_by(|a, b| crate::length(crate::sub(**a, **b)) < 0.5);
                for q in &front_rim {
                    println!("held: x {:6.1} below the chin {:6.1} behind the chin {:6.1}", q[0], bottom - q[1], pog - q[2]);
                }
                println!("vertices {}, moving {}", local.len(), proto.weight.iter().filter(|w| **w > 0.0).count());
                for y in [10.0, 20.0, 30.0, 40.0, 50.0] {
                    let row: Vec<&V3> = local.iter().zip(&proto.weight).filter(|(q, w)| **w > 0.0 && (q[1] - (bottom - y)).abs() < 1.5).map(|(q, _)| q).collect();
                    let zmax = row.iter().map(|q| q[2]).fold(f64::NEG_INFINITY, f64::max);
                    let wmax = local.iter().zip(&proto.weight).filter(|(q, _)| (q[1] - (bottom - y)).abs() < 1.5).map(|(_, w)| *w).fold(0.0, f64::max);
                    println!("{y} mm below the chin: {} moving vertices, front {zmax:.1}, largest weight {wmax:.2}", row.len());
                }
                true
            }
            _ => false,
        }
    }

    /// What the tools share: the head mesh, viewer 2's live slider head (the five features' sets merged, built on `starts_all.bin`
    /// with each head's solved values, as `sliders.js` builds it with the eye curve off), and the prototype.
    pub struct Heads {
        pub mesh: HeadMesh,
        pub frame: Frame,
        pub zone: ThroatZone,
        pub smoother: head_shape::smooth::Smoother,
        pub set: crate::set::SliderSet,
        /// Where the jaw's sliders start in the merged set, and how many there are.
        pub jaw_first: usize,
        pub jaw_count: usize,
        pub names: Vec<String>,
        starts: Vec<f32>,
        values: Vec<Vec<f64>>,
    }

    const FEATURE_FILES: [(&str, &str); 5] = [
        ("sliders_r2.json", "page_r2.json"),
        ("jaw/sliders.json", "jaw/page.json"),
        ("brow/sliders.json", "brow/page.json"),
        ("cheeks/sliders.json", "cheeks/page.json"),
        ("eyes/sliders.json", "eyes/page.json"),
    ];

    /// Viewer 2's feature files before the fold of the throat and the skull (2026-11-22), in the order `sliders.js` loads them.
    pub const VIEWER_FILES: [(&str, &str); 5] = [
        ("sliders_r3_nostril.json", "page_r3_nostril.json"),
        ("jaw/sliders.json", "jaw/page.json"),
        ("brow/sliders.json", "brow/page_curve.json"),
        ("cheeks/sliders.json", "cheeks/page_curve.json"),
        ("eyes/sliders_curve.json", "eyes/page_curve.json"),
    ];

    /// Viewer 2's feature files with the throat and the skull folded in, and the haired heads' Forehead slope solved against the
    /// bald reference (`brow/page_fold.json`).
    pub const FOLD_FILES: [(&str, &str); 6] = [
        ("sliders_r3_nostril.json", "page_r3_nostril.json"),
        ("jaw/sliders.json", "jaw/page_throat.json"),
        ("brow/sliders.json", "brow/page_fold.json"),
        ("cheeks/sliders.json", "cheeks/page_curve.json"),
        ("eyes/sliders_curve.json", "eyes/page_curve.json"),
        ("skull/sliders.json", "skull/page.json"),
    ];

    impl Heads {
        pub fn load(input: &Input, data: &Path) -> Self {
            Self::load_files(input, data, &FEATURE_FILES, false)
        }

        /// The heads as viewer 2 builds them from `files`: with the eye curve, the throat construction (when the files have its
        /// sliders), and a curve or throat slider at its middle when a head has no value for it.
        pub fn load_viewer(input: &Input, data: &Path, files: &[(&str, &str)]) -> Self {
            Self::load_files(input, data, files, true)
        }

        fn load_files(input: &Input, data: &Path, files: &[(&str, &str)], viewer: bool) -> Self {
            let web = data.join("web");
            let mesh = input.head_mesh();
            let read = |f: &str| std::fs::read_to_string(web.join(f)).unwrap_or_else(|e| panic!("{f}: {e}"));
            let heads: serde_json::Value = serde_json::from_str(&read("heads.json")).unwrap();
            let names: Vec<String> = heads.as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect();
            let mut sets = Vec::new();
            let mut values = vec![Vec::new(); names.len()];
            let (mut jaw_first, mut jaw_count) = (0, 0);
            for &(sf, pf) in files {
                let text = read(sf);
                let set: crate::set::SliderSet = serde_json::from_str(&text).unwrap();
                let raw: serde_json::Value = serde_json::from_str(&text).unwrap();
                // a curve or throat slider rests at its middle, every other slider at 0 (as `sliders.js` sets `zero`)
                let zero: Vec<f64> = raw["sliders"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|s| if viewer && (s["kind"] == "curve" || s["kind"] == "throat") { s["mid"].as_f64().unwrap_or(0.0) } else { 0.0 })
                    .collect();
                let page: serde_json::Value = serde_json::from_str(&read(pf)).unwrap();
                let count = set.sliders.len();
                if sf.starts_with("jaw/") {
                    jaw_first = values[0].len();
                    jaw_count = count;
                }
                let last = page["counts"].as_array().and_then(|c| c.last()).map(|c| c.to_string()).unwrap_or_default();
                let order: Option<Vec<Option<usize>>> = page["sliders"]
                    .as_array()
                    .map(|a| a.iter().map(|x| set.sliders.iter().position(|sl| x["name"].as_str() == Some(sl.name.as_str()))).collect());
                for (i, name) in names.iter().enumerate() {
                    let mut out = zero.clone();
                    if let Some(h) = page["heads"].as_array().unwrap().iter().find(|h| h["name"].as_str() == Some(name.as_str())) {
                        let s = if h["solved"]["full"].is_object() { &h["solved"]["full"] } else { &h["solved"]["held_out"] };
                        if let Some(v) = s["counts"][&last]["values"].as_array() {
                            for (k, x) in v.iter().enumerate() {
                                let j = order.as_ref().map_or(Some(k), |o| o.get(k).copied().flatten());
                                if let Some(j) = j.filter(|&j| j < count) {
                                    out[j] = x.as_f64().unwrap_or(0.0);
                                }
                            }
                        }
                    }
                    values[i].extend(out);
                }
                sets.push(set);
            }
            let set = crate::set::SliderSet::merge(&sets);
            let f32s = |f: &str| -> Vec<f32> { std::fs::read(web.join(f)).unwrap().chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect() };
            let base: Vec<V3> = f32s("base.bin").chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect();
            assert_eq!(base.len(), mesh.base.len(), "the page's base head has the kit's vertices");
            crate::eye_curves::init(&base, &mesh.triangles);
            crate::set::init_mesh(&mesh.triangles, &mesh.keep);
            if viewer {
                crate::set::init_curve(mesh.group.clone(), mesh.first.clone(), mesh.hidden.clone());
                crate::set::init_throat(crate::chin_neck::ThroatShape::new(&mesh));
            }
            let smoother = crate::set::smoother(&base, &mesh.triangles, &mesh.keep);
            Self { zone: ThroatZone::new(&mesh), frame: Frame::head(), starts: f32s("starts_all.bin"), mesh, smoother, set, jaw_first, jaw_count, names, values }
        }

        /// A head's values for every slider of the merged set.
        pub fn values(&self, name: &str) -> Vec<f64> {
            self.values[self.index(name)].clone()
        }

        /// A head built from its start shape and `values` (every slider of the merged set), in metres.
        pub fn build(&self, name: &str, values: &[f64]) -> Vec<V3> {
            let i = self.index(name);
            let n = self.mesh.base.len();
            let start: Vec<V3> = self.starts[i * n * 3..(i + 1) * n * 3].chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect();
            self.set.build(&self.smoother, &start, values)
        }

        /// The jaw's solved values of a head.
        pub fn preset(&self, i: usize) -> Vec<f64> {
            self.values[i][self.jaw_first..self.jaw_first + self.jaw_count].to_vec()
        }

        pub fn index(&self, name: &str) -> usize {
            self.names.iter().position(|n| n == name).unwrap_or_else(|| panic!("no head {name}"))
        }

        /// KOTOR's bare mesh in the frame of the eyes (none for the base head), and the live head in metres. `jaw` replaces the jaw
        /// sliders' values.
        pub fn pair_with(&self, input: &Input, name: &str, jaw: Option<&[f64]>) -> (Option<(Vec<V3>, Vec<[u32; 3]>)>, Vec<V3>) {
            if name == "base" {
                let base: Vec<V3> = self.smoother.smooth(&self.mesh.base, crate::set::SMOOTHING);
                return (None, base);
            }
            let i = self.index(name);
            let n = self.mesh.base.len();
            let start: Vec<V3> = self.starts[i * n * 3..(i + 1) * n * 3].chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect();
            let mut v = self.values[i].clone();
            if let Some(j) = jaw {
                v[self.jaw_first..self.jaw_first + self.jaw_count].copy_from_slice(j);
            }
            let (kv, kf, _) = crate::under_jaw::tools::kotor_bare(input, name);
            (Some((kv, kf)), self.set.build(&self.smoother, &start, &v))
        }

        /// The live head in metres, without KOTOR's mesh. `jaw` replaces the jaw sliders' values.
        pub fn live_with(&self, name: &str, jaw: Option<&[f64]>) -> Vec<V3> {
            if name == "base" {
                return self.smoother.smooth(&self.mesh.base, crate::set::SMOOTHING);
            }
            let i = self.index(name);
            let n = self.mesh.base.len();
            let start: Vec<V3> = self.starts[i * n * 3..(i + 1) * n * 3].chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect();
            let mut v = self.values[i].clone();
            if let Some(j) = jaw {
                v[self.jaw_first..self.jaw_first + self.jaw_count].copy_from_slice(j);
            }
            self.set.build(&self.smoother, &start, &v)
        }

        pub fn pair(&self, input: &Input, name: &str) -> (Option<(Vec<V3>, Vec<[u32; 3]>)>, Vec<V3>) {
            self.pair_with(input, name, None)
        }

        /// The jaw sliders' ranges.
        pub fn jaw_ranges(&self) -> Vec<(f64, f64)> {
            self.set.sliders[self.jaw_first..self.jaw_first + self.jaw_count].iter().map(|s| (s.low, s.high)).collect()
        }
    }
}
