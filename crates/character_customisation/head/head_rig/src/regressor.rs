//! Points placed on a head as weighted sums of its vertices, the way FLAME and SMPL place their joints. The weights of each point sum to
//! 1, so the points move, turn and grow with the head.

use crate::V3;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize)]
pub struct Point {
    pub name: String,
    pub vertices: Vec<usize>,
    pub weights: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Regressor {
    /// The number of vertices of the mesh the vertex numbers belong to.
    pub n: usize,
    pub points: Vec<Point>,
    /// KOTOR's average bone layout in metres of KOTOR model space, turned into the head's axes, for the key scale.
    pub reference: BTreeMap<String, V3>,
}

#[derive(Debug, PartialEq)]
pub enum PlaceError {
    WrongMesh { expected: usize, got: usize },
}

impl Regressor {
    pub fn place(&self, positions: &[V3]) -> Result<BTreeMap<String, V3>, PlaceError> {
        if positions.len() != self.n {
            return Err(PlaceError::WrongMesh {
                expected: self.n,
                got: positions.len(),
            });
        }
        Ok(self
            .points
            .iter()
            .map(|p| {
                let mut at = [0.0; 3];
                for (&v, &w) in p.vertices.iter().zip(&p.weights) {
                    for k in 0..3 {
                        at[k] += w * positions[v][k];
                    }
                }
                (p.name.clone(), at)
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> Regressor {
        Regressor {
            n: 3,
            points: vec![Point {
                name: "mid".into(),
                vertices: vec![0, 1, 2],
                weights: vec![0.5, 0.25, 0.25],
            }],
            reference: BTreeMap::new(),
        }
    }

    #[test]
    fn a_point_follows_a_moved_and_scaled_head() {
        let r = reg();
        let pos = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]];
        let moved: Vec<V3> = pos
            .iter()
            .map(|p| [2.0 * p[0] + 1.0, 2.0 * p[1] - 3.0, 2.0 * p[2] + 0.5])
            .collect();
        let a = r.place(&pos).unwrap()["mid"];
        let b = r.place(&moved).unwrap()["mid"];
        assert_eq!(a, [1.0, 1.0, 0.0]);
        assert_eq!(b, [3.0, -1.0, 0.5]);
    }

    #[test]
    fn a_mesh_of_another_size_is_refused() {
        assert_eq!(
            reg().place(&[[0.0; 3]; 4]),
            Err(PlaceError::WrongMesh {
                expected: 3,
                got: 4
            })
        );
    }
}
