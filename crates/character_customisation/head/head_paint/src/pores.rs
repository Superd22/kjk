//! The settings and the landmarks of the pore and grain layer (paint track, fine skin detail).
//!
//! The GPU painter paints the pores, the grain and the mottling. This file holds the layer's settings (`Pores`), the landmarks of the
//! pore regions on a head (`PoreLandmarks`), the regions themselves (`pore_regions`) and the pore density and size they give at a point
//! (`region_factors`). The landmarks also tell how much skin texture and T-zone a point has, and give the fine lines' origin and flow.

use crate::layout_points::{LayoutLocator, LayoutPixel, dot, length, mirrored, sub, uv_of};

/// Where the pore regions sit, as pixels of the 2,048-pixel layout. The left side's points are mirrored for the right side.
pub mod landmarks {
    use crate::layout_points::LayoutPixel;
    pub const NOSE_ROOT: LayoutPixel = (926.0, 720.0);
    pub const NOSE_TIP: LayoutPixel = (926.0, 1118.0);
    pub const FOREHEAD: LayoutPixel = (926.0, 450.0);
    pub const CHIN: LayoutPixel = (930.0, 1690.0);
    pub const MOUTH_CORNER: LayoutPixel = (702.0, 1448.0);
    pub const EYE_OUTER: LayoutPixel = (446.0, 854.0);
    pub const EYE_INNER: LayoutPixel = (746.0, 790.0);
}

/// The landmarks of the pore regions on one head, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoreLandmarks {
    pub nose: [[f64; 3]; 2],
    pub forehead: [f64; 3],
    pub chin: [f64; 3],
    pub mouth: [[f64; 3]; 2],
    pub eyes: [[[f64; 3]; 2]; 2],
}

/// How far the T-zone reaches from the nose's ridge, from the middle of the forehead and from the chin, in millimetres (a Gaussian's
/// standard deviation).
const T_ZONE_REACH: [f64; 3] = [9.0, 26.0, 12.0];
/// The lips are free of pores within this distance of the line between the mouth's corners, and pores fade in over the next 5 mm.
const LIPS_MM: f64 = 6.0;
/// The eye opening is free of pores within this distance of the line between the eye's corners, and pores fade in over the next 5 mm.
const EYES_MM: f64 = 2.5;
const FADE_MM: f64 = 5.0;

fn distance_to_segment(p: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let ab = sub(b, a);
    let ap = sub(p, a);
    let l2 = dot(ab, ab);
    let t = if l2 > 0.0 { (dot(ap, ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
    length([ap[0] - t * ab[0], ap[1] - t * ab[1], ap[2] - t * ab[2]])
}

fn gauss(d: f64, sigma: f64) -> f64 {
    (-d * d / (2.0 * sigma * sigma)).exp()
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    crate::wrinkles::smoothstep(a, b, x)
}

impl PoreLandmarks {
    /// Finds the landmarks on a head. `positions` holds the head's positions in metres, one per vertex of the layout.
    pub fn new(locator: &LayoutLocator, positions: &[[f64; 3]]) -> Option<Self> {
        let at = |p: LayoutPixel| locator.spot(uv_of(p), positions).map(|s| s.position_mm);
        use landmarks::*;
        Some(PoreLandmarks {
            nose: [at(NOSE_ROOT)?, at(NOSE_TIP)?],
            forehead: at(FOREHEAD)?,
            chin: at(CHIN)?,
            mouth: [at(MOUTH_CORNER)?, at(mirrored(MOUTH_CORNER))?],
            eyes: [[at(EYE_OUTER)?, at(EYE_INNER)?], [at(mirrored(EYE_OUTER))?, at(mirrored(EYE_INNER))?]],
        })
    }

    /// How much a point belongs to the T-zone (the nose, the middle of the forehead and the chin), from 0 to 1.
    pub fn t_zone(&self, p: [f64; 3]) -> f64 {
        let nose = gauss(distance_to_segment(p, self.nose[0], self.nose[1]), T_ZONE_REACH[0]);
        let forehead = gauss(length(sub(p, self.forehead)), T_ZONE_REACH[1]);
        let chin = gauss(length(sub(p, self.chin)), T_ZONE_REACH[2]);
        nose.max(forehead).max(chin)
    }

    /// A point between the eyes, from which the fine lines' noise is stretched.
    pub fn origin(&self) -> [f64; 3] {
        [0, 1, 2].map(|a| 0.25 * (self.eyes[0][0][a] + self.eyes[0][1][a] + self.eyes[1][0][a] + self.eyes[1][1][a]))
    }

    /// The direction the fine lines run along at a point, as a unit vector. They run across the forehead above the eyes, out from the
    /// middle of each eye within 22 to 35 mm of it, and down the face elsewhere. The directions come from the landmarks, so they follow
    /// the head.
    pub fn flow(&self, p: [f64; 3]) -> [f64; 3] {
        let centre = |e: &[[f64; 3]; 2]| [0, 1, 2].map(|a| 0.5 * (e[0][a] + e[1][a]));
        let (left, right) = (centre(&self.eyes[0]), centre(&self.eyes[1]));
        let across = crate::layout_points::normalize(sub(left, right));
        let middle = [0, 1, 2].map(|a| 0.5 * (left[a] + right[a]));
        let down = crate::layout_points::normalize(sub(self.chin, middle));
        // How high above the eyes the point is, along the face's up direction.
        let height = -dot(sub(p, middle), down);
        let forehead = smoothstep(10.0, 20.0, height);
        let (near, eye) = if length(sub(p, left)) < length(sub(p, right)) { (length(sub(p, left)), left) } else { (length(sub(p, right)), right) };
        let around_eye = (1.0 - smoothstep(22.0, 35.0, near)) * (1.0 - forehead);
        let radial = crate::layout_points::normalize(sub(p, eye));
        let rest = (1.0 - forehead) * (1.0 - around_eye);
        crate::layout_points::normalize([0, 1, 2].map(|a| forehead * across[a] + around_eye * radial[a] + rest * down[a]))
    }

    /// How much skin texture a point has, from 0 on the lips and the eye openings to 1 elsewhere.
    pub fn skin(&self, p: [f64; 3]) -> f64 {
        let lips = distance_to_segment(p, self.mouth[0], self.mouth[1]);
        let eye = distance_to_segment(p, self.eyes[0][0], self.eyes[0][1]).min(distance_to_segment(p, self.eyes[1][0], self.eyes[1][1]));
        smoothstep(LIPS_MM, LIPS_MM + FADE_MM, lips) * smoothstep(EYES_MM, EYES_MM + FADE_MM, eye)
    }
}

/// The settings of the pore and grain layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pores {
    /// Visible pores per cm² of skin in the T-zone, from 0 to 150. Real cheeks have 10 to 80 (Flament et al. 2015).
    pub density: f64,
    /// How much fewer pores the cheeks have than the T-zone, from 0 (as many) to 1 (none).
    pub t_zone: f64,
    /// The mean radius of a pore in millimetres, from 0.1 to 0.6. Real visible pores are 0.10 to 0.26 mm.
    pub size_mm: f64,
    /// How much darker a pore's centre is, in L*, from 0 to 30.
    pub depth: f64,
    /// The amplitude of the fine grain, in L*, from 0 to 10.
    pub grain: f64,
    /// The feature size of the fine grain, in millimetres, from 0.25 to 2.
    pub grain_mm: f64,
    /// The amplitude of the broad mottling, in L*, from 0 to 10.
    pub mottling: f64,
    pub seed: u64,
    /// How much fewer and smaller the pores are away from the nose, from 0 (the same everywhere) through 1 (the measured gradient of
    /// `PoreRegion`) to 2 (twice as steep). Only the GPU painter reads it.
    pub gradient: f64,
    /// How much the grain's furrows run along the flow, from 0 (no direction) to 1. Only the GPU painter reads it.
    pub grain_direction: f64,
    /// How much the pores are stretched along the flow, from 0 (round) to 1 (the long axis 2.5 times the short one). Only the GPU painter
    /// reads it.
    pub stretch: f64,
    /// How bright the small light grains are: the L* a grain's top adds to skin of L* 50 where the painted light is plain, from 0 to 25.
    /// The grains add light, as a lit bump does, so they show more on dark skin than on pale skin. Only the GPU painter reads it.
    pub glint: f64,
    /// How far apart the light grains are, in millimetres, from 0.25 to 1.2. Only the GPU painter reads it.
    pub glint_mm: f64,
    /// How much darker the points between the grains are, in L* on skin of L* 60, from 0 to 20. Only the GPU painter reads it.
    pub glint_points: f64,
    /// How much the grains differ from each other in brightness, from 0 (all the same) to 1. Only the GPU painter reads it.
    pub glint_variation: f64,
    /// How much the grains follow the painted light of the shading and the sheen, from 0 (the same everywhere) to 1. Only the GPU painter
    /// reads it.
    pub glint_follow: f64,
    /// How much each pore is drawn as a dent lit from above instead of a dark dot, from 0 (a dark dot, as before) to 1. Only the GPU
    /// painter reads it.
    pub crater: f64,
    /// How many times larger the pores are drawn than their real size, from 1 to 3, with the pores per cm² divided by the square of this
    /// number, so that the share of skin they cover stays the same. Only the GPU painter reads it.
    pub crater_scale: f64,
    /// How much each crater gleams, as oily skin does: a small near-white glint on its lit rim, strongest on the oily T-zone, nose and
    /// inner cheeks, from 0 to 1. It acts only on craters. Only the GPU painter reads it.
    pub gleam: f64,
}

impl Pores {
    pub const NONE: Pores = Pores { density: 0.0, t_zone: 0.5, size_mm: 0.2, depth: 0.0, grain: 0.0, grain_mm: 0.5, mottling: 0.0, seed: 0, gradient: 1.0, grain_direction: 0.5, stretch: 0.5, glint: 0.0, glint_mm: 0.45, glint_points: 0.0, glint_variation: 0.5, glint_follow: 0.5, crater: 0.0, crater_scale: 1.0, gleam: 0.0 };
}

/// One region of the pore map: a segment of the face (a point when both ends are the same), how far it reaches, and the pore density and
/// pore size there, against the nose's. The GPU painter blends the regions at each pore's place, each weighted by a Gaussian of the
/// distance to its segment, with the rest of the skin (`PORE_BACKGROUND`) under them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoreRegion {
    pub a: [f64; 3],
    pub b: [f64; 3],
    /// The Gaussian's standard deviation, in millimetres.
    pub reach_mm: f64,
    pub density: f64,
    pub size: f64,
}

/// The pore density and size of the skin that no region covers, which is the middle of the cheek, the jaw and the scalp, and the weight
/// of that background against a region's Gaussian.
pub const PORE_BACKGROUND: [f64; 3] = [0.5, 0.7, 0.08];

/// The regions of the pore map on one head, from its landmarks and the wrinkle layer's age in years. The values are the research's
/// design values (`docs/research/2026-11-20-pores-and-grain.md`, recommendation 2): the nose 1, the upper inner cheek beside the nose 0.8
/// with pores as large as the nose's, the middle of the forehead 0.7, the lower cheek beside the mouth 0.35, the chin from 0.3 at 15 years
/// to 0.8 at 70, the temples 0.15 and the skin around the eyes 0.1.
pub fn pore_regions(m: &PoreLandmarks, face: &crate::folds::Landmarks, age: f64) -> Vec<PoreRegion> {
    let add = |p: [f64; 3], d: [f64; 3], s: f64| [p[0] + s * d[0], p[1] + s * d[1], p[2] + s * d[2]];
    let mid = |a: [f64; 3], b: [f64; 3]| [0, 1, 2].map(|k| 0.5 * (a[k] + b[k]));
    let point = |p: [f64; 3], reach_mm: f64, density: f64, size: f64| PoreRegion { a: p, b: p, reach_mm, density, size };
    let f = face.frame;
    let old = ((age - 15.0) / 55.0).clamp(0.0, 1.0);
    let mut list = vec![
        PoreRegion { a: m.nose[0], b: m.nose[1], reach_mm: 7.0, density: 1.0, size: 1.0 },
        point(m.forehead, 20.0, 0.7, 0.7),
        point(face.chin_middle, 11.0, 0.3 + 0.5 * old, 0.5 + 0.3 * old),
    ];
    for side in 0..2 {
        let out = f.out(side);
        list.push(point(face.nose_wing[side], 6.0, 1.0, 1.0));
        list.push(point(add(mid(face.nose_wing[side], face.lower_lid[side]), out, 6.0), 9.0, 0.8, 1.0));
        list.push(point(add(add(face.eye_outer[side], out, 15.0), f.up, 12.0), 13.0, 0.15, 0.4));
        list.push(PoreRegion { a: face.eye_inner[side], b: face.eye_outer[side], reach_mm: 7.0, density: 0.1, size: 0.4 });
        list.push(point(add(face.mouth_corner[side], out, 9.0), 9.0, 0.35, 0.5));
    }
    list
}

/// The pore density and size factors at a point, from the regions, as the GPU painter blends them.
pub fn region_factors(regions: &[PoreRegion], p: [f64; 3]) -> [f64; 2] {
    let [d0, s0, w0] = PORE_BACKGROUND;
    let (mut w, mut d, mut s) = (w0, w0 * d0, w0 * s0);
    for r in regions {
        let g = gauss(distance_to_segment(p, r.a, r.b), r.reach_mm);
        w += g;
        d += g * r.density;
        s += g * r.size;
    }
    [d / w, s / w]
}
