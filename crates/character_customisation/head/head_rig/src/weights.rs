//! Skin weights computed from the head's own shape. The weights of the mouth's free vertices are a shared map plus a linear correction
//! from the head's shape, learned on KOTOR's 57 test heads. The shared map was fitted to KOTOR's own motion with smoothness over the
//! mesh, so the skin around the lips does not crease. Each lip only takes its own bones near the opening.

use crate::V3;
use base64::Engine;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFile {
    pub n: usize,
    pub bones: Vec<String>,
    pub lip_line: usize,
    pub shape_vertices: Vec<usize>,
    /// Pairs of a mesh vertex and the free vertex whose weights it takes.
    pub outputs: Vec<(usize, usize)>,
    pub components: usize,
    pub mean: String,
    pub basis: String,
    pub scale: String,
    pub base: String,
    pub change: String,
    pub allowed: String,
}

pub struct Model {
    pub n: usize,
    pub bones: Vec<String>,
    lip_line: usize,
    shape_vertices: Vec<usize>,
    pub outputs: Vec<(usize, usize)>,
    components: usize,
    mean: Vec<f32>,
    basis: Vec<f32>,
    scale: Vec<f32>,
    base: Vec<f32>,
    change: Vec<f32>,
    allowed: Vec<u8>,
}

fn decode(s: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .expect("base64")
}

fn f32s(s: &str) -> Vec<f32> {
    decode(s)
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect()
}

impl Model {
    pub fn from_file(f: ModelFile) -> Model {
        let m = Model {
            n: f.n,
            bones: f.bones,
            lip_line: f.lip_line,
            shape_vertices: f.shape_vertices,
            outputs: f.outputs,
            components: f.components,
            mean: f32s(&f.mean),
            basis: f32s(&f.basis),
            scale: f32s(&f.scale),
            base: f32s(&f.base),
            change: f32s(&f.change),
            allowed: decode(&f.allowed),
        };
        let (free, nb, k) = (m.shape_vertices.len(), m.bones.len(), m.components);
        assert_eq!(m.mean.len(), 3 * free);
        assert_eq!(m.basis.len(), k * 3 * free);
        assert_eq!(m.scale.len(), k);
        assert_eq!(m.base.len(), free * nb);
        assert_eq!(m.change.len(), k * free * nb);
        assert_eq!(m.allowed.len(), free * nb);
        m
    }

    /// The head's shape coordinates: its free vertices' places relative to the lip line, projected on the learned components.
    pub fn shape_coordinates(&self, positions: &[V3]) -> Vec<f64> {
        let lip = positions[self.lip_line];
        let x: Vec<f64> = self
            .shape_vertices
            .iter()
            .flat_map(|&v| (0..3).map(move |k| positions[v][k] - lip[k]))
            .collect();
        let len = x.len();
        (0..self.components)
            .map(|c| {
                let row = &self.basis[c * len..(c + 1) * len];
                let dot: f64 = x
                    .iter()
                    .zip(&self.mean)
                    .zip(row)
                    .map(|((xi, mi), bi)| (xi - *mi as f64) * *bi as f64)
                    .sum();
                dot / self.scale[c] as f64
            })
            .collect()
    }

    /// Each free vertex's weights on `bones`, which sum to 1.
    pub fn weights(&self, positions: &[V3]) -> Vec<Vec<f64>> {
        assert_eq!(
            positions.len(),
            self.n,
            "the model belongs to a mesh of {} vertices",
            self.n
        );
        let z = self.shape_coordinates(positions);
        let nb = self.bones.len();
        let per = self.base.len();
        (0..self.shape_vertices.len())
            .map(|v| {
                let mut w: Vec<f64> = (0..nb)
                    .map(|j| {
                        let i = v * nb + j;
                        let x = self.base[i] as f64
                            + z.iter()
                                .enumerate()
                                .map(|(c, zc)| zc * self.change[c * per + i] as f64)
                                .sum::<f64>();
                        if self.allowed[i] == 0 {
                            0.0
                        } else {
                            x.max(0.0)
                        }
                    })
                    .collect();
                let s: f64 = w.iter().sum();
                if s > 0.0 {
                    w.iter_mut().for_each(|x| *x /= s);
                }
                w
            })
            .collect()
    }
}

/// The four largest weights of a vertex, scaled to sum to 1, as bone numbers and weights. KOTOR's skins hold four bones a vertex.
pub fn top4(w: &[f64]) -> ([usize; 4], [f64; 4]) {
    let mut order: Vec<usize> = (0..w.len()).collect();
    order.sort_by(|a, b| w[*b].total_cmp(&w[*a]));
    let mut idx = [0; 4];
    let mut out = [0.0; 4];
    for k in 0..4.min(w.len()) {
        idx[k] = order[k];
        out[k] = if w[order[k]] > 0.004 {
            w[order[k]]
        } else {
            0.0
        };
    }
    let s: f64 = out.iter().sum();
    if s > 0.0 {
        out.iter_mut().for_each(|x| *x /= s);
    }
    (idx, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b64f(v: &[f32]) -> String {
        base64::engine::general_purpose::STANDARD
            .encode(v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>())
    }

    fn model() -> Model {
        // two free vertices, two bones, one component that moves weight from bone 0 to bone 1 as vertex 0 rises
        Model::from_file(ModelFile {
            n: 3,
            bones: vec!["a".into(), "b".into()],
            lip_line: 2,
            shape_vertices: vec![0, 1],
            outputs: vec![(0, 0), (1, 1)],
            components: 1,
            mean: b64f(&[0.0; 6]),
            basis: b64f(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0]),
            scale: b64f(&[1.0]),
            base: b64f(&[0.6, 0.4, 0.5, 0.5]),
            change: b64f(&[-0.5, 0.5, 0.0, 0.0]),
            allowed: base64::engine::general_purpose::STANDARD.encode([1u8, 1, 1, 0]),
        })
    }

    #[test]
    fn the_shape_moves_the_weights() {
        let m = model();
        let flat = m.weights(&[[0.0; 3], [0.0; 3], [0.0; 3]]);
        let risen = m.weights(&[[0.0, 0.2, 0.0], [0.0; 3], [0.0; 3]]);
        assert!((flat[0][0] - 0.6).abs() < 1e-6);
        assert!((risen[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn a_bone_that_is_not_allowed_gets_nothing_and_the_rest_sums_to_one() {
        let w = model().weights(&[[0.0; 3]; 3]);
        assert_eq!(w[1], vec![1.0, 0.0]);
    }

    #[test]
    fn negative_weights_are_cut_before_scaling() {
        let w = model().weights(&[[0.0, 2.0, 0.0], [0.0; 3], [0.0; 3]]);
        assert_eq!(w[0][0], 0.0);
        assert!((w[0][1] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn top4_keeps_the_largest_and_sums_to_one() {
        let (i, w) = top4(&[0.1, 0.5, 0.05, 0.2, 0.15]);
        assert_eq!(i, [1, 3, 4, 0]);
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }
}
