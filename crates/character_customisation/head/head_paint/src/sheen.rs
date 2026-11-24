//! The sheen layer: a whitish highlight along the cheekbones, along the ridge of the nose and on the chin (paint track, step C2).
//!
//! Three dark KOTOR heads (pfhb04, pmba01 and mod_hp_pfhb02) have such a highlight, and it follows no light direction. Each region's mask
//! is a Gaussian of the distance to landmarks on the head's surface, or to the line between two landmarks, so it follows the sliders. Each region has its own amount, because
//! each of the three heads lights a different region: pfhb04 the cheeks and the nose, pmba01 the nose, and mod_hp_pfhb02 the chin.

use crate::shading::shift_lightness;

/// The share of the sheen's lightening that is white. The three heads with a sheen have 50%, 61% and 82%.
pub const SHEEN_WHITENESS: f64 = 0.6;

/// Where the sheen regions sit on a head: vertex numbers of the head's mesh, and the width of each region.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SheenLandmarks {
    /// The left and the right cheekbone under the eyes, each as a line from its inner end to its outer end. Each cheek region lies along
    /// its line.
    pub cheekbones: [[usize; 2]; 2],
    /// The root of the nose between the eyes, and the tip of the nose. The nose region lies along the line between them.
    pub nose_ridge: [usize; 2],
    /// The front of the chin.
    pub chin: usize,
    /// The standard deviation of each region's Gaussian, in metres, for the cheeks, the nose and the chin.
    pub widths: [f64; 3],
}

impl SheenLandmarks {
    /// The landmarks on our head's mesh, in its 4,750-vertex numbering. On the base head each cheekbone line runs from 21 mm to 42 mm out
    /// from the middle, about 28 mm below the nose's root. The nose's root and tip lie on the middle line, and the chin point lies on the
    /// middle line 59 mm below the tip of the nose.
    pub const OUR_HEAD: SheenLandmarks = SheenLandmarks { cheekbones: [[1853, 1999], [4184, 4322]], nose_ridge: [141, 281], chin: 1884, widths: [0.012, 0.006, 0.010] };
}

/// The amount of sheen in each region, as the lightness it adds in L* at the region's centre. Zero paints nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sheen {
    pub cheeks: f64,
    pub nose: f64,
    pub chin: f64,
}

impl Sheen {
    pub const NONE: Sheen = Sheen { cheeks: 0.0, nose: 0.0, chin: 0.0 };

    /// The lightness the sheen adds at a point with these region weights, in L*.
    pub fn lightness_shift(&self, weights: [f64; 3]) -> f64 {
        self.cheeks * weights[0] + self.nose * weights[1] + self.chin * weights[2]
    }

    /// The colour of a point after the sheen, from its linear sRGB colour before.
    pub fn apply(&self, linear: [f64; 3], weights: [f64; 3]) -> [f64; 3] {
        shift_lightness(linear, self.lightness_shift(weights), SHEEN_WHITENESS)
    }
}

fn distance_to_segment(p: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let length = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if length > 0.0 { ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / length).clamp(0.0, 1.0) } else { 0.0 };
    let d = [ap[0] - t * ab[0], ap[1] - t * ab[1], ap[2] - t * ab[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Weights below this are set to zero, so that the sheen ends instead of reaching across the whole head.
const CUTOFF: f64 = 0.01;

/// The weight of each sheen region (cheeks, nose, chin) at every vertex of the head, from 0 to 1. It is 1 at a region's landmark and falls
/// off as a Gaussian of the straight distance to it.
pub fn sheen_weights(positions: &[[f64; 3]], landmarks: &SheenLandmarks) -> Vec<[f64; 3]> {
    let cheeks = landmarks.cheekbones.map(|line| line.map(|i| positions[i]));
    let ridge = landmarks.nose_ridge.map(|i| positions[i]);
    let chin = positions[landmarks.chin];
    let gauss = |d: f64, width: f64| {
        let w = (-d * d / (2.0 * width * width)).exp();
        if w < CUTOFF { 0.0 } else { w }
    };
    positions
        .iter()
        .map(|&p| {
            let cheek = distance_to_segment(p, cheeks[0][0], cheeks[0][1]).min(distance_to_segment(p, cheeks[1][0], cheeks[1][1]));
            [gauss(cheek, landmarks.widths[0]), gauss(distance_to_segment(p, ridge[0], ridge[1]), landmarks.widths[1]), gauss(distance(p, chin), landmarks.widths[2])]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Points along the x axis every 5 mm, with landmarks at the first five.
    fn line() -> (Vec<[f64; 3]>, SheenLandmarks) {
        let points = (0..40).map(|i| [i as f64 * 0.005, 0.0, 0.0]).collect();
        (points, SheenLandmarks { cheekbones: [[0, 0], [1, 1]], nose_ridge: [2, 3], chin: 4, widths: [0.01, 0.005, 0.01] })
    }

    #[test]
    fn a_region_weighs_one_at_its_landmark() {
        let (points, marks) = line();
        let w = sheen_weights(&points, &marks);
        assert_eq!(w[0][0], 1.0);
        assert_eq!(w[1][0], 1.0);
        assert_eq!(w[2][1], 1.0);
        assert_eq!(w[3][1], 1.0);
        assert_eq!(w[4][2], 1.0);
    }

    #[test]
    fn the_sheen_is_zero_outside_its_mask() {
        let (points, marks) = line();
        let w = sheen_weights(&points, &marks);
        let sheen = Sheen { cheeks: 10.0, nose: 10.0, chin: 10.0 };
        let skin = [0.05, 0.03, 0.02];
        for (i, weights) in w.iter().enumerate().skip(20) {
            assert_eq!(*weights, [0.0; 3], "point {i}");
            assert_eq!(sheen.apply(skin, *weights), skin);
        }
    }

    #[test]
    fn no_sheen_changes_nothing_inside_the_mask() {
        let (points, marks) = line();
        let skin = [0.05, 0.03, 0.02];
        for weights in sheen_weights(&points, &marks) {
            assert_eq!(Sheen::NONE.apply(skin, weights), skin);
        }
    }

    #[test]
    fn the_sheen_lightens_and_whitens() {
        let skin = [0.05, 0.03, 0.02];
        let lit = Sheen { cheeks: 8.0, ..Sheen::NONE }.apply(skin, [1.0, 0.0, 0.0]);
        let (before, after) = (crate::colour::srgb_to_lab(skin.map(crate::pigment::encode)), crate::colour::srgb_to_lab(lit.map(crate::pigment::encode)));
        assert!((after[0] - before[0] - 8.0).abs() < 0.05);
        assert!(after[1].hypot(after[2]) < before[1].hypot(before[2]) * (after[0] / before[0]));
    }

    #[test]
    fn the_nose_region_follows_the_whole_ridge() {
        let points = vec![[0.0, 0.0, 0.0], [0.0, 0.03, 0.0], [0.0, 0.015, 0.0], [0.02, 0.015, 0.0]];
        let marks = SheenLandmarks { cheekbones: [[3, 3], [3, 3]], nose_ridge: [0, 1], chin: 3, widths: [0.01, 0.005, 0.01] };
        let w = sheen_weights(&points, &marks);
        assert_eq!(w[2][1], 1.0);
        assert_eq!(w[3][1], 0.0);
    }
}
