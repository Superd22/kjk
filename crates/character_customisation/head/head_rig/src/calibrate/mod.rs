//! The animation calibration (roadmap decision 62). KOTOR's clips move a head through its face bones, and the skin follows the bones by
//! its weights. The weights of an edited head are computed from its own shape by a rule (`crate::weights::Model`). This module learns
//! that rule from our fitted heads, with KOTOR's own motion as the target. It is run on demand after a fit, not on every fit.
//! - `data`: viewer 2's data (our fitted heads, KOTOR's skinned heads and clips) and the template of the base head;
//! - `pose`: KOTOR's clips sampled and played on a set of bones, and the skin moved by them;
//! - `kotor`: each of our vertices matched to a point of KOTOR's face, and KOTOR's motion there;
//! - `solve`: a smooth fit of the weights to that motion, with each vertex's weights not negative and summing to 1;
//! - `model`: the shared fit, the fit of each head and the correction from the head's shape, as a weights model;
//! - `regions`: which vertices and bones each part of the face fits, and the hints it starts from;
//! - `regress`: where the face bones and the teeth points lie on a head, learned from KOTOR's own on the fitted heads.

pub mod data;
pub mod kotor;
pub mod model;
pub mod pose;
pub mod regions;
pub mod regress;
pub mod solve;

/// The cubic step from 0 at `a` to 1 at `b`, as `build_rig.smooth` in the likeness tools.
pub fn smooth(a: f64, b: f64, t: f64) -> f64 {
    let x = ((t - a) / (b - a)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
