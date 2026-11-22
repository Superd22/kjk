//! The shading layer: the light and shade that KOTOR paints into its face textures, computed from our head's own shape (paint track, step C2).
//!
//! KOTOR's painted shading has no key light and no side light. It is the light of a uniform sky above the head that the head's own surface
//! blocks within 25 mm, plus a broad lightening of the points that stand out of the surface at a 20 mm scale. The study behind this layer is
//! `shading_maps` and its notes (`docs/research/2026-11-19-head-roadmap.md`, "What KOTOR's painted shading is"). One strength per head scales
//! the whole pattern. The layer shifts the skin's lightness. Where it darkens, the skin keeps its chroma. Where it lightens, part of the
//! change is white, by the recipe's highlight whiteness.

use crate::colour::{lab_to_srgb, srgb_to_lab};
use crate::pigment::{decode, encode};
use crate::surface::Surface;

/// How far the head's surface can block the sky, in metres.
pub const SKY_REACH: f64 = 0.025;
/// The scale of the bump term, in metres.
pub const BUMP_SCALE: f64 = 0.020;
/// The lightness change, in L*, per unit of sky light at strength 1. It is the shared pattern of the 57 KOTOR heads.
pub const SKY_GAIN: f64 = 34.0;
/// The lightness change, in L*, per millimetre that a point stands out of the surface at the bump scale, at strength 1. Creases get no
/// term of their own, because the sky light already darkens them.
pub const BUMP_GAIN: f64 = 0.73;
/// The sky light at which the layer changes nothing. It is the median sky light of the face skin of our base head, so the base tone is the
/// skin's colour at a typical point of the face.
pub const SKY_REFERENCE: f64 = 0.476;
/// The highlight whiteness of a typical KOTOR head: the median share of the lightening that is white.
pub const TYPICAL_HIGHLIGHT_WHITENESS: f64 = 0.54;

/// The settings of the shading layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shading {
    /// How strongly the shared pattern is painted. One is the shared pattern, and zero paints nothing.
    pub strength: f64,
    /// The share of the lightening that goes toward white instead of keeping the skin's colour, from 0 to 1.
    pub highlight_whiteness: f64,
}

impl Shading {
    pub const NONE: Shading = Shading { strength: 0.0, highlight_whiteness: TYPICAL_HIGHLIGHT_WHITENESS };

    /// The lightness change in L* at strength 1, for a point with this sky light and this bump in millimetres.
    pub fn pattern(sky: f64, bump_mm: f64) -> f64 {
        SKY_GAIN * (sky - SKY_REFERENCE) + BUMP_GAIN * bump_mm.max(0.0)
    }

    /// The lightness change in L* that the layer paints at a point.
    pub fn lightness_shift(&self, sky: f64, bump_mm: f64) -> f64 {
        self.strength * Self::pattern(sky, bump_mm)
    }
}

/// The two maps that the shading layer reads, at each vertex of a head.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeMaps {
    /// The light from a uniform sky above the head that reaches the vertex past the head's surface within `SKY_REACH`, from 0 to 1.
    pub sky: Vec<f64>,
    /// How far the vertex stands out of the surface around it at `BUMP_SCALE`, in millimetres. It is negative in a crease.
    pub bump_mm: Vec<f64>,
}

impl ShapeMaps {
    /// Computes the maps from the head's current shape. A vertex that no triangle uses gets the values at which the layer changes nothing.
    pub fn compute(positions: &[[f64; 3]], triangles: &[[u32; 3]]) -> Self {
        let surface = Surface::new(positions, triangles);
        let mut maps = ShapeMaps { sky: vec![SKY_REFERENCE; positions.len()], bump_mm: vec![0.0; positions.len()] };
        for i in 0..positions.len() {
            if surface.is_used(i) {
                maps.sky[i] = surface.horizon(i, SKY_REACH).sky().uniform;
                maps.bump_mm[i] = 1000.0 * surface.bump(i, BUMP_SCALE);
            }
        }
        maps
    }
}

fn luminance(linear: [f64; 3]) -> f64 {
    0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]
}

fn lightness_to_luminance(l: f64) -> f64 {
    let f = (l + 16.0) / 116.0;
    if f * f * f > 216.0 / 24389.0 { f * f * f } else { l * 27.0 / 24389.0 }
}

/// Shifts the lightness of a linear sRGB colour by `delta_l` in L*. A darker colour keeps its a* and b*, and so its chroma. A lighter
/// colour is a mix of two colours with the new lightness: the same colour made brighter, and the colour with white light added. `whiteness`
/// is the share of the second. The result is clamped to the sRGB gamut.
pub fn shift_lightness(linear: [f64; 3], delta_l: f64, whiteness: f64) -> [f64; 3] {
    if delta_l == 0.0 {
        return linear;
    }
    if delta_l < 0.0 {
        let lab = srgb_to_lab(linear.map(encode));
        return lab_to_srgb([(lab[0] + delta_l).max(0.0), lab[1], lab[2]]).map(decode);
    }
    let y = luminance(linear);
    let l = srgb_to_lab(linear.map(encode))[0];
    let target = lightness_to_luminance((l + delta_l).min(100.0));
    let w = whiteness.clamp(0.0, 1.0);
    let scale = if y > 1e-9 { target / y } else { 1.0 };
    let added = target - if y > 1e-9 { y } else { 0.0 };
    linear.map(|c| ((1.0 - w) * c * scale + w * (c + added)).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKIN: [f64; 3] = [0.35, 0.2, 0.13];

    fn lab(linear: [f64; 3]) -> [f64; 3] {
        srgb_to_lab(linear.map(encode))
    }

    fn chroma(linear: [f64; 3]) -> f64 {
        let c = lab(linear);
        c[1].hypot(c[2])
    }

    #[test]
    fn zero_strength_changes_nothing() {
        let none = Shading { strength: 0.0, highlight_whiteness: 0.7 };
        for (sky, bump) in [(0.0, 0.0), (0.9, 3.0), (0.2, -4.0)] {
            let d = none.lightness_shift(sky, bump);
            assert_eq!(d, 0.0);
            assert_eq!(shift_lightness(SKIN, d, 0.7), SKIN);
        }
    }

    #[test]
    fn a_crease_is_darker_and_a_bump_is_lighter() {
        let s = Shading { strength: 1.0, highlight_whiteness: 0.5 };
        let crease = s.lightness_shift(0.15, -2.0);
        let bump = s.lightness_shift(0.7, 3.0);
        assert!(crease < -5.0 && bump > 5.0, "{crease} {bump}");
        let base = lab(SKIN)[0];
        assert!(lab(shift_lightness(SKIN, crease, 0.5))[0] < base - 5.0);
        assert!(lab(shift_lightness(SKIN, bump, 0.5))[0] > base + 5.0);
    }

    #[test]
    fn the_bump_term_lightens_the_points_that_stand_out_but_does_not_darken_creases() {
        assert!(Shading::pattern(SKY_REFERENCE, 2.0) > 0.0);
        assert_eq!(Shading::pattern(SKY_REFERENCE, -2.0), 0.0);
    }

    #[test]
    fn the_lightness_moves_by_the_asked_amount() {
        for d in [-12.0, -3.0, 4.0, 15.0] {
            for w in [0.0, 0.5, 1.0] {
                let moved = lab(shift_lightness(SKIN, d, w))[0] - lab(SKIN)[0];
                assert!((moved - d).abs() < 0.05, "{d} with whiteness {w} moved {moved}");
            }
        }
    }

    #[test]
    fn darkening_keeps_the_chroma() {
        let before = lab(SKIN);
        let after = lab(shift_lightness(SKIN, -10.0, 0.5));
        assert!((after[1] - before[1]).abs() < 0.2 && (after[2] - before[2]).abs() < 0.2, "{before:?} {after:?}");
    }

    #[test]
    fn highlight_whiteness_lowers_the_chroma() {
        let coloured = chroma(shift_lightness(SKIN, 12.0, 0.0));
        let half = chroma(shift_lightness(SKIN, 12.0, 0.5));
        let white = chroma(shift_lightness(SKIN, 12.0, 1.0));
        assert!(coloured > half && half > white, "{coloured} {half} {white}");
    }

    #[test]
    fn the_maps_of_an_open_upward_sheet_change_nothing_beyond_the_sky_term() {
        let mut points = Vec::new();
        for i in 0..21 {
            for j in 0..21 {
                points.push([(i as f64 - 10.0) * 0.005, 0.0, (j as f64 - 10.0) * 0.005]);
            }
        }
        let mut triangles = Vec::new();
        for i in 0..20u32 {
            for j in 0..20u32 {
                let a = i * 21 + j;
                triangles.push([a, a + 1, a + 21]);
                triangles.push([a + 1, a + 22, a + 21]);
            }
        }
        points.push([1.0, 1.0, 1.0]);
        let maps = ShapeMaps::compute(&points, &triangles);
        let centre = 10 * 21 + 10;
        assert!((maps.sky[centre] - 1.0).abs() < 0.01 && maps.bump_mm[centre].abs() < 1e-9);
        let unused = points.len() - 1;
        assert_eq!((maps.sky[unused], maps.bump_mm[unused]), (SKY_REFERENCE, 0.0));
    }
}
