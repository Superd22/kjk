//! The lid piece in the browser, with the same code as the native tool.

use crate::piece::{EyeEdge, EyePiece, LidRig, Settings, V3};
use wasm_bindgen::prelude::*;

/// Makes both eyes' lid pieces for any head of the default mesh.
#[wasm_bindgen]
pub struct LidMaker {
    triangles: Vec<[u32; 3]>,
    base: Vec<V3>,
    edges: Vec<EyeEdge>,
    rig: Option<LidRig>,
    last: Vec<EyePiece>,
    /// The slider Upper lid cover, in mm, used by every later `build`.
    lower: f64,
    /// Per eye, the opening's curve (upper and lower arches, metres) that later builds put the piece on, when set.
    curves: Vec<Option<(Vec<V3>, Vec<V3>)>>,
}

fn points(flat: &[f32]) -> Vec<V3> {
    flat.chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect()
}

#[wasm_bindgen]
impl LidMaker {
    /// `triangles` holds three indices per triangle and `base` the base head's positions (metres, three numbers per vertex).
    #[wasm_bindgen(constructor)]
    pub fn new(triangles: &[u32], base: &[f32]) -> LidMaker {
        LidMaker {
            triangles: triangles.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect(),
            base: points(base),
            edges: Vec::new(),
            rig: None,
            last: Vec::new(),
            lower: 0.0,
            curves: Vec::new(),
        }
    }

    /// Adds one eye from `lid_edges.json`: the edge's vertices in order, 1 for each upper one, the corners, the centre's
    /// vertices and whether the temple is toward +x.
    pub fn add_eye(&mut self, edge: &[u32], upper: &[u8], inner_corner: u32, outer_corner: u32, centre: &[u32], temple_plus_x: bool) {
        self.edges.push(EyeEdge {
            edge: edge.to_vec(),
            upper: upper.iter().map(|&u| u != 0).collect(),
            inner_corner,
            outer_corner,
            centre: centre.to_vec(),
            temple_plus_x,
        });
        self.rig = None;
    }

    /// Builds both pieces on a head (positions in metres). Angles are in degrees and lengths in mm.
    #[allow(clippy::too_many_arguments)]
    pub fn build(&mut self, head: &[f32], open: f64, lash_length: f64, lash_thickness: f64, lash_tilt: f64, match_clip: bool, pivot_back: f64) {
        if self.rig.is_none() {
            self.rig = Some(LidRig::new(&self.triangles, &self.base, &self.edges));
        }
        let s = Settings { open, lash_length, lash_thickness, lash_tilt, match_clip, pivot_back, lower: self.lower, ..Settings::default() };
        let (rig, head) = (self.rig.as_ref().unwrap(), points(head));
        self.last = rig
            .eyes
            .iter()
            .enumerate()
            .map(|(e, eye)| match self.curves.get(e).and_then(|c| c.as_ref()) {
                Some((u, l)) => crate::piece::build_eye_on_curve(eye, &rig.triangles, &head, &s, u, l),
                None => crate::piece::build_eye(eye, &rig.triangles, &head, &s),
            })
            .collect();
    }

    /// Puts eye `eye`'s piece on an opening drawn as a curve on the eyeball, for every later `build`: `upper` and `lower` are its arches
    /// from the inner tip to the outer one (metres, three numbers per point). Empty arrays take the curve away again.
    pub fn set_curve(&mut self, eye: usize, upper: &[f32], lower: &[f32]) {
        if self.curves.len() <= eye {
            self.curves.resize(eye + 1, None);
        }
        self.curves[eye] = (!upper.is_empty() && !lower.is_empty()).then(|| (points(upper), points(lower)));
    }

    /// Sets how far the open piece's edge is lowered over the eyeball in the middle of the lid, in mm (the slider Upper lid cover), for
    /// every later `build`. It is 0 until set.
    pub fn set_lower(&mut self, mm: f64) {
        self.lower = mm;
    }

    pub fn positions(&self, eye: usize) -> Vec<f32> {
        self.last[eye].positions.iter().flatten().copied().collect()
    }
    pub fn triangles(&self, eye: usize) -> Vec<u32> {
        self.last[eye].triangles.iter().flatten().copied().collect()
    }
    pub fn colour_from(&self, eye: usize) -> Vec<u32> {
        self.last[eye].colour_from.iter().flatten().copied().collect()
    }
    pub fn colour_weights(&self, eye: usize) -> Vec<f32> {
        self.last[eye].colour_weights.iter().flatten().copied().collect()
    }
    pub fn shade(&self, eye: usize) -> Vec<f32> {
        self.last[eye].shade.clone()
    }
    pub fn part(&self, eye: usize) -> Vec<u8> {
        self.last[eye].part.clone()
    }
    pub fn column(&self, eye: usize) -> Vec<u16> {
        self.last[eye].column.clone()
    }
    pub fn lash_positions(&self, eye: usize) -> Vec<f32> {
        self.last[eye].lash_positions.iter().flatten().copied().collect()
    }
    pub fn lash_triangles(&self, eye: usize) -> Vec<u32> {
        self.last[eye].lash_triangles.iter().flatten().copied().collect()
    }
    pub fn hidden_faces(&self, eye: usize) -> Vec<u32> {
        self.last[eye].hidden_faces.clone()
    }
    /// One eye's caruncle on a head (positions in metres), three numbers per vertex in metres. `width` adds to the length of its pink
    /// part along the lids' edges from the inner corner, in mm, as the slider Caruncle width gives it.
    pub fn caruncle_positions(&self, head: &[f32], eye: usize, width: f64) -> Vec<f32> {
        crate::caruncle::positions(&points(head), &self.edges[eye], width).iter().flat_map(|p| p.map(|x| x as f32)).collect()
    }
    /// The caruncle's triangles, the same for every head and both eyes.
    pub fn caruncle_triangles(&self) -> Vec<u32> {
        crate::caruncle::triangles().iter().flatten().copied().collect()
    }
    /// One eye's caruncle colour at each vertex, three numbers from 0 to 1 per vertex: a tint that multiplies the eyeball's colour
    /// under it. `width` is the same as for `caruncle_positions`.
    pub fn caruncle_colours(&self, head: &[f32], eye: usize, width: f64) -> Vec<f32> {
        crate::caruncle::colours(&points(head), &self.edges[eye], width)
    }
    /// One eye's caruncle normal at each vertex, three numbers per vertex. `width` is the same as for `caruncle_positions`.
    pub fn caruncle_normals(&self, head: &[f32], eye: usize, width: f64) -> Vec<f32> {
        crate::caruncle::normals(&points(head), &self.edges[eye], width)
    }
    /// The pivot (metres, head frame), the axis, the closing angle and other numbers, as JSON.
    pub fn info(&self, eye: usize) -> String {
        crate::piece::info_json(&self.last[eye])
    }
}
