//! The hair grid's points and values: the shared primitive of the painted hair (paint track, part C, decisions 60 and 61). A layer
//! fills a grid's field from its sliders and hair groups, and the shader `hair_grid.wgsl` of `head_paint_gpu` draws single hairs from
//! it. The facial hair is its first user (`facial_hair.rs`), on the face grid of `face_grid.rs`. The brows can move onto it later, and
//! part D's coarse grid around the head is the same idea, so a 3D beard mesh could grow from the same points.
//!
//! **What a point holds.** Its place on the skin and the skin's normal, its texture's size (millimetres per texel at 2,048 pixels), and
//! the hair field: the density in hairs per square millimetre, the length in millimetres, the direction of growth along the skin, a
//! signed distance to an edge that no hair root may cross, such as the lips' border, in millimetres, positive where hair may grow, and
//! the look its hairs take (their colour, thickness and curl), as a number in the layer's list of looks.
//!
//! **Sliders, groups and drawing.** A layer's sliders write a preset into the field as its base. Hair groups (`hair_groups.rs`) are
//! written over it, and values written straight onto the grid (`GridEdit`) last.

use crate::layout_points::{dot, normalize, sub};

/// One point of the grid on the skin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridPoint {
    /// Whether skin was found in this direction.
    pub valid: bool,
    pub position: [f64; 3],
    pub normal: [f64; 3],
    /// The texture's size here: millimetres of skin per texel of a 2,048-pixel texture.
    pub mm_per_texel: f64,
}

/// The hair at each point of a grid, in the grid's order.
#[derive(Clone, Debug, PartialEq)]
pub struct HairField {
    /// Hairs per square millimetre.
    pub density: Vec<f64>,
    /// The typical length of a hair, in millimetres.
    pub length_mm: Vec<f64>,
    /// The direction of growth, a unit vector along the skin.
    pub direction: Vec<[f64; 3]>,
    /// The signed distance to the edge no root may cross, in millimetres, positive where hair may grow.
    pub edge_mm: Vec<f64>,
    /// How much darker the skin under the hair is, in L*, for a shadow that does not wait for single hairs.
    pub shadow: Vec<f64>,
    /// The number of the look the hairs take, in the layer's list of looks, one per hair group.
    pub look: Vec<u32>,
}

/// Values written straight onto some points of a face grid (`face_grid::FaceGrid`), as the grid editor will write them. Each point
/// names its column and row, and sets the density, the length and, if given, the direction of growth in the face's directions (across,
/// up, forward). With `only` set, every point the edit does not name loses its hair. Otherwise it keeps the preset's values.
#[derive(Clone, Debug, PartialEq)]
pub struct GridEdit {
    /// The size of the grid the edit was made on. An edit made on a grid of another size is not laid.
    pub cols: usize,
    pub rows: usize,
    pub only: bool,
    pub points: Vec<EditPoint>,
}

impl GridEdit {
    /// An edit from JSON: `{"cols": 102, "rows": 63, "only": true, "points": [[i, j, density, length], [i, j, density, length, x, y, z],
    /// ...]}`, where x, y and z are the direction in the grid's own directions. `None` when the text is not such an object.
    pub fn from_json(text: &str) -> Option<Self> {
        let v: serde_json::Value = serde_json::from_str(text).ok()?;
        let points = v["points"]
            .as_array()?
            .iter()
            .map(|p| {
                let n: Vec<f64> = p.as_array()?.iter().map(|x| x.as_f64()).collect::<Option<_>>()?;
                (n.len() >= 4).then(|| EditPoint { i: n[0] as usize, j: n[1] as usize, density: n[2], length_mm: n[3], direction: (n.len() >= 7).then(|| [n[4], n[5], n[6]]) })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(GridEdit { cols: v["cols"].as_u64()? as usize, rows: v["rows"].as_u64()? as usize, only: v["only"].as_bool().unwrap_or(false), points })
    }

    /// The edit as JSON, in the form `from_json` reads.
    pub fn to_json(&self) -> String {
        let points: Vec<serde_json::Value> = self
            .points
            .iter()
            .map(|p| {
                let mut v = vec![p.i as f64, p.j as f64, (p.density * 1000.0).round() / 1000.0, (p.length_mm * 100.0).round() / 100.0];
                if let Some(d) = p.direction {
                    v.extend(d.map(|x| (x * 1000.0).round() / 1000.0));
                }
                serde_json::json!(v)
            })
            .collect();
        serde_json::json!({ "cols": self.cols, "rows": self.rows, "only": self.only, "points": points }).to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditPoint {
    pub i: usize,
    pub j: usize,
    pub density: f64,
    pub length_mm: f64,
    pub direction: Option<[f64; 3]>,
}

impl HairField {
    /// Lays an edit over the field, with the look `look`. The skin's shadow under a named point follows its density: `shadow_per_density`
    /// times it, at most 1, times `shadow_l` in L*.
    pub fn apply(&mut self, grid: &crate::face_grid::FaceGrid, edit: &GridEdit, shadow_per_density: f64, shadow_l: f64, look: u32) {
        if edit.cols != grid.cols || edit.rows != grid.rows {
            return;
        }
        if edit.only {
            self.density.iter_mut().for_each(|d| *d = 0.0);
            self.shadow.iter_mut().for_each(|s| *s = 0.0);
        }
        for e in &edit.points {
            if e.i >= grid.cols || e.j >= grid.rows {
                continue;
            }
            let k = e.j * grid.cols + e.i;
            self.density[k] = e.density;
            self.length_mm[k] = e.length_mm;
            self.shadow[k] = (shadow_per_density * e.density).min(1.0) * shadow_l;
            self.look[k] = look;
            if let Some(d) = e.direction {
                let n = grid.points[k].normal;
                let w = [0, 1, 2].map(|a| d[0] * grid.across[a] + d[1] * grid.up[a] + d[2] * grid.forward[a]);
                let t = sub(w, n.map(|x| x * dot(w, n)));
                if dot(t, t) > 1e-12 {
                    self.direction[k] = normalize(t);
                }
            }
        }
    }

    pub fn empty(len: usize) -> Self {
        HairField { density: vec![0.0; len], length_mm: vec![0.0; len], direction: vec![[0.0; 3]; len], edge_mm: vec![0.0; len], shadow: vec![0.0; len], look: vec![0; len] }
    }
}
