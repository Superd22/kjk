//! The tongue. 51 of KOTOR's 57 test heads have one: a sheet of about 30 triangles, about 45 mm wide, lying at the height of the lip line
//! from about 14 to 50 mm behind the lips. Its bone, `f_tonguetip_g`, sits at the sheet's back end and is a child of the jaw, so the talk
//! clip lifts the front of the tongue by turning the bone up to 49 degrees. Our tongue is a domed sheet from the tongue bone to 4 mm
//! behind the lower front teeth, as wide as the lower teeth. Its back follows the jaw and its front the tongue bone, as on Bastila's
//! KOTOR head.

use crate::V3;
use crate::shade::DEEP;

/// The gap between the tip of the tongue and the back of the lower front teeth, in metres.
const BEHIND_TEETH: f64 = 0.004;
/// How far the tongue's middle stands above its sides, and how far its top lies below the lower teeth's top, in metres.
const DOME: f64 = 0.003;
const BELOW_TEETH: f64 = 0.002;
/// The tongue's own colour before the mouth's darkness: a muted pink.
const PINK: V3 = [0.62, 0.32, 0.32];
const ACROSS: usize = 9;
const ALONG: usize = 8;

pub struct Tongue {
    /// Unshared triangle corners, three numbers each.
    pub positions: Vec<f32>,
    /// Each corner's weight on the tongue bone. The rest of its weight is on the jaw.
    pub weight: Vec<f32>,
    /// Each corner's colour, darker deeper behind the lips.
    pub colour: Vec<f32>,
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The tongue of a head in the head's axes (+Y up, +Z forward), from the tongue bone's place, the tops of the lower teeth from left to
/// right, and the depth of the lips' front.
pub fn tongue(bone: V3, lower_teeth_tops: &[V3], lip_front_z: f64) -> Tongue {
    let n = lower_teeth_tops.len();
    let middle = lower_teeth_tops[n / 2];
    let half_width = (lower_teeth_tops[n - 1][0] - lower_teeth_tops[0][0]).abs() / 2.0;
    let centre_x = (lower_teeth_tops[n - 1][0] + lower_teeth_tops[0][0]) / 2.0;
    let front = middle[2] - BEHIND_TEETH;
    let back = bone[2].min(front - 0.02);
    let top = middle[1] - BELOW_TEETH;
    let point = |i: usize, j: usize| -> V3 {
        let u = 2.0 * i as f64 / (ACROSS - 1) as f64 - 1.0;
        let v = j as f64 / (ALONG - 1) as f64;
        let tip = ((v - 0.6) / 0.4).max(0.0);
        let x = centre_x + u * half_width * (1.0 - 0.6 * tip * tip);
        let y = top + DOME * (1.0 - u * u) - 0.004 * u.powi(4)
            + (bone[1] - top).min(0.0) * (1.0 - v) * 0.5;
        [x, y, back + (front - back) * v]
    };
    let mut out = Tongue {
        positions: Vec::new(),
        weight: Vec::new(),
        colour: Vec::new(),
    };
    for j in 0..ALONG - 1 {
        for i in 0..ACROSS - 1 {
            let (a, b, c, d) = (
                point(i, j),
                point(i + 1, j),
                point(i + 1, j + 1),
                point(i, j + 1),
            );
            for p in [a, c, b, a, d, c] {
                out.positions.extend(p.map(|x| x as f32));
                out.weight
                    .push(smoothstep(back, back + 0.6 * (front - back), p[2]) as f32);
                let dark = smoothstep(0.002, 0.03, lip_front_z - p[2]);
                out.colour.extend(
                    (0..3).map(|k| (PINK[k] * (1.0 + (DEEP[k] / PINK[k] - 1.0) * dark)) as f32),
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn teeth() -> Vec<V3> {
        (0..5)
            .map(|i| {
                [
                    -0.02 + 0.01 * i as f64,
                    0.66,
                    0.14 - 0.01 * (i as f64 - 2.0).abs(),
                ]
            })
            .collect()
    }

    #[test]
    fn the_tongue_lies_behind_the_teeth_and_in_front_of_its_bone() {
        let t = tongue([0.0, 0.66, 0.10], &teeth(), 0.15);
        let z: Vec<f32> = t.positions.chunks(3).map(|p| p[2]).collect();
        let max = z.iter().copied().fold(f32::MIN, f32::max);
        let min = z.iter().copied().fold(f32::MAX, f32::min);
        assert!((max - 0.136).abs() < 1e-6, "{max}");
        assert!((min - 0.10).abs() < 1e-6, "{min}");
    }

    #[test]
    fn the_back_follows_the_jaw_and_the_front_the_tongue_bone() {
        let t = tongue([0.0, 0.66, 0.10], &teeth(), 0.15);
        let at = |z: f32| {
            t.positions
                .chunks(3)
                .zip(&t.weight)
                .find(|(p, _)| (p[2] - z).abs() < 1e-6)
                .map(|(_, w)| *w)
                .unwrap()
        };
        assert_eq!(at(0.10), 0.0);
        assert_eq!(at(0.136), 1.0);
    }
}
