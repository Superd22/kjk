//! How dark the inside of the mouth is painted. KOTOR's mouths read dark when they open, because the inside behind the lips is painted
//! dark red to black. Ours is computed from the head's shape: a vertex of the mouth's inside darkens with its depth behind the front of
//! the lips, so the lips' own curl keeps its colour and the throat is nearly black.

use crate::V3;

/// The colour a vertex's paint is multiplied by at full darkness: a deep, slightly red shadow.
pub const DEEP: V3 = [0.16, 0.06, 0.06];
/// Depths behind the lips' front, in metres, where darkening starts and where it is full.
pub const START: f64 = 0.002;
pub const FULL: f64 = 0.009;

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The colour factor of each listed inside vertex, given the head's positions (head axes, +Z forward) and the lips' front depth.
pub fn inside_colours(positions: &[V3], inside: &[usize], lip_front_z: f64) -> Vec<V3> {
    inside
        .iter()
        .map(|&v| {
            let d = smoothstep(START, FULL, lip_front_z - positions[v][2]);
            [0, 1, 2].map(|k| 1.0 + (DEEP[k] - 1.0) * d)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lips_front_keeps_its_colour_and_the_back_goes_dark() {
        let pos = [[0.0, 0.0, 0.150], [0.0, 0.0, 0.130]];
        let c = inside_colours(&pos, &[0, 1], 0.151);
        assert_eq!(c[0], [1.0, 1.0, 1.0]);
        assert!((0..3).all(|k| (c[1][k] - DEEP[k]).abs() < 1e-12));
    }
}
