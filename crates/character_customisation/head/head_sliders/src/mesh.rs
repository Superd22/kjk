//! The head mesh that every head shares, and the inputs that `prep_nose.py` writes for the tools.
//!
//! The mesh has copies of a vertex wherever the texture layout splits it. Copies are merged into one "merged vertex" when a movement
//! is built, so every copy moves alike. The insides of the nostrils and of the mouth are hidden vertices: they move with the skin
//! nearest to them, as the fit moved them.

use crate::V3;

#[derive(Clone)]
pub struct HeadMesh {
    pub base: Vec<V3>,
    pub triangles: Vec<[u32; 3]>,
    /// The smoothing holds these vertices still.
    pub keep: Vec<bool>,
    /// Each vertex's merged vertex, and one vertex of each merged vertex.
    pub group: Vec<u32>,
    pub first: Vec<u32>,
    /// Per merged vertex.
    pub hidden: Vec<bool>,
    /// For each hidden merged vertex in order, its 6 nearest skin merged vertices (as places in the list of skin merged vertices) and
    /// their weights.
    pub near: Vec<[u32; 6]>,
    pub near_w: Vec<[f64; 6]>,
    /// Each vertex's zone on the base head: 0 for none, then the zones of `ZONES` from 1.
    pub zone: Vec<u8>,
    pub eyes: [V3; 2],
    /// Merged neighbours, as offsets into `neighbours`.
    offsets: Vec<u32>,
    neighbours: Vec<u32>,
}

pub const ZONES: [&str; 8] = [
    "forehead",
    "brows",
    "around the eyes",
    "nose",
    "cheeks",
    "mouth",
    "chin",
    "jaw",
];

impl HeadMesh {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base: Vec<V3>,
        triangles: Vec<[u32; 3]>,
        keep: Vec<bool>,
        group: Vec<u32>,
        first: Vec<u32>,
        hidden: Vec<bool>,
        near: Vec<[u32; 6]>,
        near_w: Vec<[f64; 6]>,
        zone: Vec<u8>,
        eyes: [V3; 2],
    ) -> Self {
        let n = first.len();
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for t in &triangles {
            let g = t.map(|v| group[v as usize]);
            for (a, b) in [(g[0], g[1]), (g[1], g[2]), (g[2], g[0])] {
                if a != b {
                    edges.push((a, b));
                    edges.push((b, a));
                }
            }
        }
        edges.sort_unstable();
        edges.dedup();
        let mut offsets = vec![0u32; n + 1];
        for &(a, _) in &edges {
            offsets[a as usize + 1] += 1;
        }
        for i in 0..n {
            offsets[i + 1] += offsets[i];
        }
        let neighbours = edges.iter().map(|&(_, b)| b).collect();
        Self {
            base,
            triangles,
            keep,
            group,
            first,
            hidden,
            near,
            near_w,
            zone,
            eyes,
            offsets,
            neighbours,
        }
    }

    pub fn merged_count(&self) -> usize {
        self.first.len()
    }

    pub fn neighbours_of(&self, g: usize) -> &[u32] {
        &self.neighbours[self.offsets[g] as usize..self.offsets[g + 1] as usize]
    }

    /// A movement given per merged vertex, spread to every vertex, with the hidden vertices moved as their nearest skin moves.
    pub fn expand(&self, merged: &[V3]) -> Vec<V3> {
        let mut m = merged.to_vec();
        let skin: Vec<usize> = (0..m.len()).filter(|&g| !self.hidden[g]).collect();
        let hidden: Vec<usize> = (0..m.len()).filter(|&g| self.hidden[g]).collect();
        for (k, &g) in hidden.iter().enumerate() {
            let mut s = [0.0; 3];
            for j in 0..6 {
                let src = m[skin[self.near[k][j] as usize]];
                for c in 0..3 {
                    s[c] += self.near_w[k][j] * src[c];
                }
            }
            m[g] = s;
        }
        self.group.iter().map(|&g| m[g as usize]).collect()
    }

    /// Positions per merged vertex.
    pub fn merge(&self, positions: &[V3]) -> Vec<V3> {
        self.first.iter().map(|&i| positions[i as usize]).collect()
    }
}

/// The arrays that `prep_nose.py` wrote, by name.
#[cfg(not(target_arch = "wasm32"))]
pub mod input {
    use super::HeadMesh;
    use crate::V3;
    use serde_json::Value;
    use std::path::{Path, PathBuf};

    pub struct Input {
        pub dir: PathBuf,
        pub manifest: Value,
    }

    impl Input {
        pub fn open(dir: &Path) -> Self {
            let text = std::fs::read_to_string(dir.join("manifest.json"))
                .expect("the inputs' manifest.json");
            let mut manifest: Value = serde_json::from_str(&text).expect("a JSON manifest");
            // the arrays that `prep_extra.py` added for the features after the nose
            if let Ok(extra) = std::fs::read_to_string(dir.join("manifest_extra.json")) {
                let extra: Value = serde_json::from_str(&extra).expect("a JSON manifest");
                if let (Some(arrays), Some(more)) = (manifest["arrays"].as_object_mut(), extra["arrays"].as_object()) {
                    arrays.extend(more.clone());
                }
            }
            Self { dir: dir.to_path_buf(), manifest }
        }

        pub fn has(&self, name: &str) -> bool {
            self.manifest["arrays"].get(name).is_some()
        }

        fn bytes(&self, name: &str, dtype: &str) -> Vec<u8> {
            let entry = &self.manifest["arrays"][name];
            assert_eq!(
                entry["dtype"].as_str(),
                Some(dtype),
                "{name} is not {dtype}"
            );
            std::fs::read(self.dir.join(format!("{name}.bin")))
                .unwrap_or_else(|e| panic!("{name}: {e}"))
        }

        pub fn f64s(&self, name: &str) -> Vec<f64> {
            self.bytes(name, "float64")
                .as_chunks::<8>()
                .0
                .iter()
                .map(|c| f64::from_le_bytes(*c))
                .collect()
        }

        pub fn f32s(&self, name: &str) -> Vec<f64> {
            self.bytes(name, "float32")
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| f32::from_le_bytes(*c) as f64)
                .collect()
        }

        pub fn u32s(&self, name: &str) -> Vec<u32> {
            self.bytes(name, "uint32")
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_le_bytes(*c))
                .collect()
        }

        pub fn u8s(&self, name: &str) -> Vec<u8> {
            self.bytes(name, "uint8")
        }

        pub fn points(&self, name: &str) -> Vec<V3> {
            let dtype = self.manifest["arrays"][name]["dtype"]
                .as_str()
                .unwrap_or("");
            let v = if dtype == "float32" {
                self.f32s(name)
            } else {
                self.f64s(name)
            };
            v.as_chunks::<3>().0.to_vec()
        }

        pub fn triangles(&self, name: &str) -> Vec<[u32; 3]> {
            self.u32s(name).as_chunks::<3>().0.to_vec()
        }

        pub fn head_mesh(&self) -> HeadMesh {
            let e = self.points("base_eyes");
            HeadMesh::new(
                self.points("base"),
                self.triangles("tris"),
                self.u8s("keep").iter().map(|&k| k != 0).collect(),
                self.u32s("group"),
                self.u32s("first"),
                self.u8s("hidden").iter().map(|&k| k != 0).collect(),
                self.u32s("near").as_chunks::<6>().0.to_vec(),
                self.f64s("near_w").as_chunks::<6>().0.to_vec(),
                self.u8s("zone"),
                [e[0], e[1]],
            )
        }
    }
}
