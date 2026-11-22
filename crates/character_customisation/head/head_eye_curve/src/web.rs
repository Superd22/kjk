//! The browser functions: `new CurveMaker(triangles, group, first, hidden)` once, then `build(positions, handles)` per head, then the
//! lid pieces of that build per eye.

use crate::V3;
use crate::maker::{Maker, Mesh};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CurveMaker {
    maker: Maker,
}

#[wasm_bindgen]
pub fn curve_handles() -> String {
    serde_json::json!(crate::curve::HANDLES).to_string()
}

#[wasm_bindgen]
impl CurveMaker {
    /// `triangles` three indices per triangle, `group` each vertex's merged vertex, `first` one vertex per merged vertex, `hidden` one
    /// byte per merged vertex (not 0 inside the nostrils and the mouth).
    #[wasm_bindgen(constructor)]
    pub fn new(triangles: &[u32], group: &[u32], first: &[u32], hidden: &[u8]) -> CurveMaker {
        let mesh = Mesh::new(triangles.as_chunks::<3>().0.to_vec(), group.to_vec(), first.to_vec(), hidden.iter().map(|&h| h != 0).collect());
        CurveMaker { maker: Maker::new(mesh) }
    }

    /// The head `positions` (three numbers per vertex, metres) with both openings built on the curve of the ten `handles`.
    pub fn build(&mut self, positions: &[f32], handles: &[f64]) -> Vec<f32> {
        let p: Vec<V3> = positions.as_chunks::<3>().0.iter().map(|c| c.map(|x| x as f64)).collect();
        self.maker.build(&p, handles).iter().flat_map(|q| q.map(|x| x as f32)).collect()
    }

    /// Eye `e`'s open lid piece of the last build: its points, metres, relative to the eyeball's centre.
    pub fn lid_positions(&self, e: usize) -> Vec<f32> {
        self.maker.lids[e].points.iter().flat_map(|q| q.map(|x| x as f32)).collect()
    }

    pub fn lid_triangles(&self, e: usize) -> Vec<u32> {
        self.maker.lids[e].triangles.iter().flatten().copied().collect()
    }

    /// Eye `e`'s upper (`upper` true) or lower arch of the last build, from the inner tip to the outer one, on the eyeball (metres).
    pub fn arch(&self, e: usize, upper: bool) -> Vec<f32> {
        let l = &self.maker.lids[e];
        (if upper { &l.upper } else { &l.lower }).iter().flat_map(|q| q.map(|x| x as f32)).collect()
    }

    /// The turn that closes eye `e`'s piece, as JSON: `centre` (metres), `axis` (unit, right-hand rule) and `angle` (degrees).
    pub fn lid_info(&self, e: usize) -> String {
        let l = &self.maker.lids[e];
        serde_json::json!({ "centre": l.centre, "axis": l.axis, "angle": l.angle }).to_string()
    }
}
