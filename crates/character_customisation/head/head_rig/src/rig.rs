//! The whole face rig of one head, computed from its own vertex positions. The `mouth_rig` tool writes it for each fitted head, and the
//! browser computes it again each time a slider moves the head, so both take the same steps in the same order:
//! - the regressor places the face bones and the 20 teeth points on the head, and the key scale follows from the bones' places;
//! - the weights rule gives the skin weights of the face, from the head before its lip line is opened;
//! - the lip line is opened, which moves the upper lip's vertices up;
//! - the teeth strips are built through the points and moved to the lips' edges, and the inside of the mouth is shaded, both on the
//!   head with the lip line open.

use crate::V3;
use crate::bones::{Bone, Quat, key_scale, relocate};
use crate::lips::{LipLine, open};
use crate::regressor::{PlaceError, Regressor};
use crate::shade::inside_colours;
use crate::teeth::{Row, point_names, strip, texture, to_lip_edge};
use crate::weights::{Model, top4};
use serde::Deserialize;

/// How many columns of triangles each teeth strip has.
pub const TEETH_COLUMNS: usize = 13;
/// The bone each teeth strip is moved by, upper strip first.
pub const TEETH_BONES: [(Row, &str); 2] = [(Row::Upper, "head_g"), (Row::Lower, "f_jaw_g")];
/// The size of the teeth texture the viewer shows.
pub const TEETH_TEXTURE: (usize, usize) = (512, 128);

/// The teeth texture as RGB bytes, `TEETH_TEXTURE` pixels.
pub fn teeth_texture() -> Vec<u8> {
    texture(TEETH_TEXTURE.0, TEETH_TEXTURE.1)
}

/// `lips.json`: the vertices of the lips and the numbers that place the teeth against them.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lips {
    pub upper_edge: Vec<usize>,
    pub lower_edge: Vec<usize>,
    pub upper_offset: f64,
    pub lower_offset: f64,
    pub inside: Vec<usize>,
    /// The lip line is only opened when these four are all present.
    pub open_upper: Option<Vec<usize>>,
    pub open_lower: Option<Vec<usize>>,
    pub corners: Option<Vec<usize>>,
    pub gap: Option<f64>,
    pub falloff: Option<f64>,
}

/// What a rig needs besides a head's positions, loaded once.
pub struct RigKit {
    pub names: Vec<String>,
    pub parents: Vec<Option<usize>>,
    pub regressor: Regressor,
    pub weights: Option<Model>,
    pub lips: Option<Lips>,
    /// For each bone of the weights rule, its number in `names`.
    bone_of: Vec<usize>,
}

/// A head's bones before the regressor places them: the local place and rest turn of each bone, in the order of `RigKit::names`.
pub type BaseBones<'a> = &'a [(V3, Quat)];

/// The teeth strip of one row, as unshared triangle corners.
pub struct TeethPart {
    pub bone: &'static str,
    pub positions: Vec<f32>,
    pub uv: Vec<f32>,
}

#[derive(Default)]
pub struct Skin {
    pub vertices: Vec<u16>,
    /// Four bone numbers (in the order of `RigKit::names`) per vertex.
    pub index: Vec<u8>,
    /// Four weights per vertex.
    pub weight: Vec<f32>,
}

#[derive(Default)]
pub struct LipLift {
    pub vertices: Vec<u16>,
    /// How far each vertex moves up, in metres.
    pub up: Vec<f32>,
}

#[derive(Default)]
pub struct Shade {
    pub vertices: Vec<u16>,
    /// Three colour factors per vertex.
    pub colour: Vec<f32>,
}

pub struct Rig {
    pub bones: Vec<Bone>,
    pub key_scale: f64,
    pub teeth: Vec<TeethPart>,
    pub skin: Option<Skin>,
    pub lift: Option<LipLift>,
    pub shade: Option<Shade>,
    pub tongue: Option<crate::tongue::Tongue>,
}

impl RigKit {
    /// Panics when the weights rule names a bone that `names` lacks.
    pub fn new(
        names: Vec<String>,
        parents: Vec<Option<usize>>,
        regressor: Regressor,
        weights: Option<Model>,
        lips: Option<Lips>,
    ) -> RigKit {
        let bone_of = weights.as_ref().map_or_else(Vec::new, |m| {
            m.bones
                .iter()
                .map(|b| {
                    names
                        .iter()
                        .position(|n| n == b)
                        .expect("a bone of common.json")
                })
                .collect()
        });
        RigKit {
            names,
            parents,
            regressor,
            weights,
            lips,
            bone_of,
        }
    }

    /// The rig of the head whose vertices are at `positions`. `base` holds the head's bones before they are placed. The bones the
    /// regressor does not place (the eyes and lids) keep their world places, and every bone keeps its rest turn.
    pub fn rig(&self, positions: &[V3], base: BaseBones) -> Result<Rig, PlaceError> {
        let places = self.regressor.place(positions)?;
        let bones: Vec<Bone> = base
            .iter()
            .enumerate()
            .map(|(i, &(local, rot))| Bone {
                name: self.names[i].clone(),
                parent: self.parents[i],
                local,
                rot,
            })
            .collect();
        let moved = relocate(&bones, &places);
        let s = key_scale(&places, &self.regressor.reference).expect("key scale");
        let skin = self.weights.as_ref().map(|m| {
            let w = m.weights(positions);
            let mut skin = Skin::default();
            for &(v, f) in &m.outputs {
                let (idx, wt) = top4(&w[f]);
                skin.vertices.push(v as u16);
                skin.index.extend(idx.map(|i| self.bone_of[i] as u8));
                skin.weight.extend(wt.map(|x| x as f32));
            }
            skin
        });
        let mut pos = positions.to_vec();
        let mut lift = None;
        let open_line = self.lips.as_ref().and_then(|l| {
            let corners = l.corners.as_ref()?;
            Some((
                l,
                l.open_upper.as_ref()?,
                l.open_lower.as_ref()?,
                [*corners.first()?, *corners.get(1)?],
                l.gap?,
                l.falloff?,
            ))
        });
        if let Some((_, up, lo, corners, gap, falloff)) = open_line {
            let line = LipLine {
                upper: up,
                lower: lo,
                corners,
            };
            let raised = open(&pos, &line, gap, falloff);
            let mut l = LipLift::default();
            for (&v, d) in up.iter().zip(raised).filter(|(_, d)| *d > 0.0) {
                pos[v][1] += d;
                l.vertices.push(v as u16);
                l.up.push(d as f32);
            }
            lift = Some(l);
        }
        let mut teeth = Vec::new();
        let mut lower_tops = Vec::new();
        for (row, bone) in TEETH_BONES {
            let pts: Vec<V3> = point_names(row).iter().map(|n| places[n]).collect();
            let mut tops: Vec<V3> = pts.iter().step_by(2).copied().collect();
            let mut bottoms: Vec<V3> = pts.iter().skip(1).step_by(2).copied().collect();
            if let Some(l) = &self.lips {
                let (edge, offset) = if row == Row::Upper {
                    (
                        l.upper_edge
                            .iter()
                            .map(|&v| pos[v][1])
                            .fold(f64::INFINITY, f64::min),
                        l.upper_offset,
                    )
                } else {
                    (
                        l.lower_edge
                            .iter()
                            .map(|&v| pos[v][1])
                            .fold(f64::NEG_INFINITY, f64::max),
                        l.lower_offset,
                    )
                };
                to_lip_edge(&mut tops, &mut bottoms, row, edge, offset);
            }
            if row == Row::Lower {
                lower_tops = tops.clone();
            }
            let mesh = strip(&tops, &bottoms, row, TEETH_COLUMNS);
            let corners: Vec<usize> = mesh
                .triangles
                .iter()
                .flatten()
                .map(|&i| i as usize)
                .collect();
            teeth.push(TeethPart {
                bone,
                positions: corners
                    .iter()
                    .flat_map(|&i| mesh.positions[i].map(|x| x as f32))
                    .collect(),
                uv: corners
                    .iter()
                    .flat_map(|&i| mesh.uv[i].map(|x| x as f32))
                    .collect(),
            });
        }
        let shade = self.lips.as_ref().map(|l| {
            let front = l
                .upper_edge
                .iter()
                .map(|&v| pos[v][2])
                .fold(f64::NEG_INFINITY, f64::max);
            let colours = inside_colours(&pos, &l.inside, front);
            Shade {
                vertices: l.inside.iter().map(|&v| v as u16).collect(),
                colour: colours.iter().flatten().map(|&x| x as f32).collect(),
            }
        });
        let lip_front = self.lips.as_ref().map_or(f64::NEG_INFINITY, |l| {
            l.upper_edge
                .iter()
                .map(|&v| pos[v][2])
                .fold(f64::NEG_INFINITY, f64::max)
        });
        let tongue = places
            .get("f_tonguetip_g")
            .filter(|_| lip_front.is_finite())
            .map(|&bone| crate::tongue::tongue(bone, &lower_tops, lip_front));
        Ok(Rig {
            bones: moved,
            key_scale: s,
            teeth,
            skin,
            lift,
            shade,
            tongue,
        })
    }
}
