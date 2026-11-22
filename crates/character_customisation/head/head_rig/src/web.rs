//! The rig of a head computed in the browser, with the code that `mouth_rig` writes the rigs with.

use crate::V3;
use crate::regressor::Regressor;
use crate::rig::{Lips, Rig, RigKit, teeth_texture};
use crate::weights::{Model, ModelFile};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RigMaker {
    kit: RigKit,
}

/// One head's rig. Each getter returns a new typed array.
#[wasm_bindgen]
pub struct LiveRig {
    rig: Rig,
}

fn bad(what: &str, e: serde_json::Error) -> JsError {
    JsError::new(&format!("{what}: {e}"))
}

#[wasm_bindgen]
impl RigMaker {
    /// `bones` is the list `[{"name": .., "parent": ..}, ..]` of `common.json`, and the other three are the files `weights_model.json`,
    /// `regressor.json` and `lips.json` of the animation calibration.
    #[wasm_bindgen(constructor)]
    pub fn new(
        bones: &str,
        weights: &str,
        regressor: &str,
        lips: &str,
    ) -> Result<RigMaker, JsError> {
        #[derive(serde::Deserialize)]
        struct B {
            name: String,
            parent: i64,
        }
        let list: Vec<B> = serde_json::from_str(bones).map_err(|e| bad("bones", e))?;
        let names = list.iter().map(|b| b.name.clone()).collect();
        let parents = list
            .iter()
            .map(|b| usize::try_from(b.parent).ok())
            .collect();
        let model: ModelFile = serde_json::from_str(weights).map_err(|e| bad("weights", e))?;
        let regressor: Regressor =
            serde_json::from_str(regressor).map_err(|e| bad("regressor", e))?;
        let lips: Lips = serde_json::from_str(lips).map_err(|e| bad("lips", e))?;
        Ok(RigMaker {
            kit: RigKit::new(
                names,
                parents,
                regressor,
                Some(Model::from_file(model)),
                Some(lips),
            ),
        })
    }

    /// `positions` holds three numbers per vertex of the viewer's mesh, and `bones` seven per bone: the local place, then the rest turn
    /// as x, y, z, w.
    pub fn rig(&self, positions: &[f32], bones: &[f32]) -> Result<LiveRig, JsError> {
        let pos: Vec<V3> = positions
            .as_chunks::<3>()
            .0
            .iter()
            .map(|c| c.map(f64::from))
            .collect();
        let base: Vec<(V3, [f64; 4])> = bones
            .as_chunks::<7>()
            .0
            .iter()
            .map(|b| {
                (
                    [b[0], b[1], b[2]].map(f64::from),
                    [b[3], b[4], b[5], b[6]].map(f64::from),
                )
            })
            .collect();
        if base.len() != self.kit.names.len() {
            return Err(JsError::new("the head has another number of bones"));
        }
        let rig = self
            .kit
            .rig(&pos, &base)
            .map_err(|e| JsError::new(&format!("{e:?}")))?;
        Ok(LiveRig { rig })
    }

    /// The teeth texture as RGB bytes, 512 by 128 pixels.
    pub fn teeth_texture(&self) -> Vec<u8> {
        teeth_texture()
    }
}

#[wasm_bindgen]
impl LiveRig {
    /// Seven numbers per bone: the local place, then the rest turn as x, y, z, w.
    pub fn bones(&self) -> Vec<f32> {
        self.rig
            .bones
            .iter()
            .flat_map(|b| b.local.iter().chain(&b.rot).map(|&x| x as f32))
            .collect()
    }
    pub fn key_scale(&self) -> f64 {
        self.rig.key_scale
    }
    /// The positions of a teeth strip's corners (0 for the upper strip, 1 for the lower).
    pub fn teeth_positions(&self, strip: usize) -> Vec<f32> {
        self.rig.teeth[strip].positions.clone()
    }
    pub fn teeth_uv(&self, strip: usize) -> Vec<f32> {
        self.rig.teeth[strip].uv.clone()
    }
    /// The tongue's corners, each one's weight on the tongue bone (the rest is on the jaw), and each one's colour. Empty without lips data.
    pub fn tongue_positions(&self) -> Vec<f32> {
        self.rig
            .tongue
            .as_ref()
            .map_or_else(Vec::new, |t| t.positions.clone())
    }
    pub fn tongue_weight(&self) -> Vec<f32> {
        self.rig
            .tongue
            .as_ref()
            .map_or_else(Vec::new, |t| t.weight.clone())
    }
    pub fn tongue_colour(&self) -> Vec<f32> {
        self.rig
            .tongue
            .as_ref()
            .map_or_else(Vec::new, |t| t.colour.clone())
    }
    pub fn teeth_bone(&self, strip: usize) -> String {
        self.rig.teeth[strip].bone.to_string()
    }
    pub fn skin_vertices(&self) -> Vec<u16> {
        self.rig
            .skin
            .as_ref()
            .map(|s| s.vertices.clone())
            .unwrap_or_default()
    }
    pub fn skin_index(&self) -> Vec<u8> {
        self.rig
            .skin
            .as_ref()
            .map(|s| s.index.clone())
            .unwrap_or_default()
    }
    pub fn skin_weight(&self) -> Vec<f32> {
        self.rig
            .skin
            .as_ref()
            .map(|s| s.weight.clone())
            .unwrap_or_default()
    }
    pub fn lift_vertices(&self) -> Vec<u16> {
        self.rig
            .lift
            .as_ref()
            .map(|s| s.vertices.clone())
            .unwrap_or_default()
    }
    /// How far each of those vertices moves up, in metres.
    pub fn lift_up(&self) -> Vec<f32> {
        self.rig
            .lift
            .as_ref()
            .map(|s| s.up.clone())
            .unwrap_or_default()
    }
    pub fn shade_vertices(&self) -> Vec<u16> {
        self.rig
            .shade
            .as_ref()
            .map(|s| s.vertices.clone())
            .unwrap_or_default()
    }
    pub fn shade_colour(&self) -> Vec<f32> {
        self.rig
            .shade
            .as_ref()
            .map(|s| s.colour.clone())
            .unwrap_or_default()
    }
}
