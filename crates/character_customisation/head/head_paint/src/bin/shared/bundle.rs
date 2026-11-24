//! Reads the bundles that `kotor_bake_prep.py` writes: a `bundle.json` manifest that lists named arrays, and a `bundle.bin` that holds them.
//! The analysis tools share it with `#[path]`, because it never ships.

use serde_json::Value;
use std::path::Path;

pub struct Bundle {
    pub manifest: Value,
    bytes: Vec<u8>,
}

#[allow(dead_code)]
impl Bundle {
    pub fn open(dir: &Path) -> Self {
        let text = std::fs::read_to_string(dir.join("bundle.json")).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let bytes = std::fs::read(dir.join("bundle.bin")).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        Bundle { manifest: serde_json::from_str(&text).expect("bundle.json"), bytes }
    }

    fn raw(&self, name: &str, size: usize) -> &[u8] {
        let entry = &self.manifest["arrays"][name];
        let count: usize = entry["shape"].as_array().unwrap_or_else(|| panic!("no array {name}")).iter().map(|d| d.as_u64().unwrap_or(0) as usize).product();
        let offset = entry["offset"].as_u64().unwrap_or(0) as usize;
        &self.bytes[offset..offset + count * size]
    }

    pub fn f64s(&self, name: &str) -> Vec<f64> {
        self.raw(name, 8).as_chunks::<8>().0.iter().map(|c| f64::from_le_bytes(*c)).collect()
    }

    pub fn u32s(&self, name: &str) -> Vec<u32> {
        self.raw(name, 4).as_chunks::<4>().0.iter().map(|c| u32::from_le_bytes(*c)).collect()
    }

    pub fn u8s(&self, name: &str) -> Vec<u8> {
        self.raw(name, 1).to_vec()
    }

    pub fn points(&self, name: &str) -> Vec<[f64; 3]> {
        self.f64s(name).as_chunks::<3>().0.to_vec()
    }

    pub fn triangles(&self, name: &str) -> Vec<[u32; 3]> {
        self.u32s(name).as_chunks::<3>().0.to_vec()
    }

    pub fn uv(&self) -> Vec<[f64; 2]> {
        self.f64s("uv").as_chunks::<2>().0.to_vec()
    }

    /// The mesh vertex of each layout vertex.
    pub fn layout_to_mesh(&self) -> Vec<usize> {
        self.u32s("vmap").into_iter().map(|i| i as usize).collect()
    }
}
