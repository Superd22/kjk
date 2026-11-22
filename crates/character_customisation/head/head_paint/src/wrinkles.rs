//! The wrinkle and fold layer: creases painted along curves on the head (paint track, fine skin detail).
//!
//! Each wrinkle set is a group of lines placed by points of the texture layout, such as the crow's feet that fan out from the outer corner
//! of each eye. The points were placed on the creases that KOTOR paints on Kreia, Canderous, Terena Adare and the old men's heads. A line
//! is a curve through its points, lifted onto the head, so it follows the sliders. Each pixel reads its distance to every line in
//! millimetres, measured in 3D at the pixel's own point, because our mesh has a vertex only every 2 to 3 mm.
//!
//! A line's cross-section is the profile of Bando, Kuratate and Nishita (2002), S(x) = -(1 - x) exp(-x), with x the distance from the
//! furrow divided by the line's width. It is -1 in the furrow, crosses zero at one width and rises to a soft ridge of 0.17 at two widths.
//! The layer turns it into a change of lightness: the furrow is darker by the line's depth and the ridges are a little lighter. Beyond
//! three widths the profile fades to zero by five widths, so a line costs nothing further away.
//!
//! A line's strength fades in over its first 12% and out over the last third of its kept length, and changes by up to 35% along it. A
//! line also wobbles sideways along the skin by a slow noise, so it does not look drawn with a ruler. The wobble and the changes come from
//! the recipe's seed.
//!
//! The age switches the sets on in the order that Flament and colleagues (2013) found in 298 women: crow's feet first, then under-eye
//! lines, nasolabial folds, glabellar lines, forehead lines and mouth lines. Each set's weight rises from 0 at its onset age to 1 at its
//! full age with a smoothstep. The onset ages are the design values of `docs/research/2026-11-19-procedural-skin-detail.md`.

use crate::layout_points::{LayoutLocator, LayoutPixel, length, mirrored, sub, uv_of};

/// One wrinkle set as it is placed on every head.
pub struct SetDefinition {
    pub id: &'static str,
    /// The name the player sees.
    pub label: &'static str,
    /// The age at which the set starts to show, and the age at which it is at full depth.
    pub onset_age: f64,
    pub full_age: f64,
    /// How much darker the furrow is at full weight, strength 1 and depth 1, in L*.
    pub depth: f64,
    /// The distance from the furrow at which the profile crosses zero, in millimetres, at width 1.
    pub width_mm: f64,
    /// The lines in order of importance: a count of 2 draws the first two. Each line is a list of layout pixels and starts at its anchor,
    /// such as the eye corner for the crow's feet. The length control keeps the part of a line nearest its anchor.
    pub lines: &'static [&'static [LayoutPixel]],
    /// Whether each line is drawn on both sides of the face as well as where its points lie.
    pub both_sides: bool,
    /// Whether the line's anchor is the face's middle line. Such a line is written from the middle out to one side, and is drawn as one
    /// line across both sides. It is at full strength in the middle and fades towards both ends, and the length control keeps its middle.
    pub open_anchor: bool,
}

/// The wrinkle sets. Their order is the order of the recipe's `sets`.
pub const SETS: [SetDefinition; 9] = [
    SetDefinition {
        id: "crows_feet",
        label: "Crow's feet",
        onset_age: 25.0,
        full_age: 60.0,
        depth: 8.0,
        width_mm: 0.5,
        lines: &[
            &[(405.0, 850.0), (320.0, 845.0), (235.0, 850.0)],
            &[(410.0, 820.0), (330.0, 790.0), (250.0, 765.0)],
            &[(410.0, 878.0), (330.0, 900.0), (255.0, 930.0)],
            &[(420.0, 905.0), (355.0, 950.0), (300.0, 995.0)],
            &[(408.0, 795.0), (345.0, 745.0), (290.0, 705.0)],
        ],
        both_sides: true,
        open_anchor: false,
    },
    SetDefinition {
        id: "under_eye",
        label: "Under-eye lines",
        onset_age: 30.0,
        full_age: 70.0,
        depth: 6.0,
        width_mm: 0.6,
        lines: &[
            &[(445.0, 895.0), (510.0, 925.0), (600.0, 935.0), (690.0, 905.0)],
            &[(480.0, 995.0), (590.0, 1010.0), (700.0, 975.0)],
        ],
        both_sides: true,
        open_anchor: false,
    },
    SetDefinition {
        id: "nasolabial",
        label: "Nasolabial folds",
        onset_age: 25.0,
        full_age: 70.0,
        depth: 10.0,
        width_mm: 1.6,
        lines: &[&[(745.0, 1125.0), (708.0, 1205.0), (668.0, 1300.0), (642.0, 1395.0), (640.0, 1475.0)]],
        both_sides: true,
        open_anchor: false,
    },
    SetDefinition {
        id: "marionette",
        label: "Marionette lines",
        onset_age: 40.0,
        full_age: 80.0,
        depth: 7.0,
        width_mm: 1.1,
        lines: &[&[(682.0, 1478.0), (668.0, 1560.0), (650.0, 1650.0)]],
        both_sides: true,
        open_anchor: false,
    },
    SetDefinition {
        id: "glabella",
        label: "Frown lines",
        onset_age: 40.0,
        full_age: 70.0,
        depth: 7.0,
        width_mm: 0.8,
        lines: &[&[(872.0, 668.0), (866.0, 620.0), (870.0, 570.0)], &[(842.0, 650.0), (840.0, 615.0), (846.0, 585.0)]],
        both_sides: true,
        open_anchor: false,
    },
    SetDefinition {
        id: "forehead",
        label: "Forehead lines",
        onset_age: 30.0,
        full_age: 75.0,
        depth: 6.0,
        width_mm: 0.9,
        lines: &[
            &[(926.0, 515.0), (740.0, 521.0), (560.0, 543.0)],
            &[(926.0, 465.0), (750.0, 470.0), (582.0, 490.0)],
            &[(926.0, 415.0), (770.0, 419.0), (612.0, 436.0)],
            &[(926.0, 368.0), (790.0, 371.0), (652.0, 385.0)],
        ],
        both_sides: true,
        open_anchor: true,
    },
    SetDefinition {
        id: "upper_lip",
        label: "Upper lip lines",
        onset_age: 40.0,
        full_age: 75.0,
        depth: 4.0,
        width_mm: 0.45,
        lines: &[
            &[(820.0, 1399.0), (812.0, 1350.0), (800.0, 1300.0)],
            &[(885.0, 1396.0), (882.0, 1350.0), (878.0, 1310.0)],
            &[(762.0, 1406.0), (748.0, 1362.0), (730.0, 1320.0)],
            &[(852.0, 1397.0), (846.0, 1350.0), (838.0, 1305.0)],
            &[(738.0, 1409.0), (722.0, 1375.0), (705.0, 1340.0)],
            &[(790.0, 1402.0), (780.0, 1355.0), (766.0, 1312.0)],
        ],
        both_sides: true,
        open_anchor: false,
    },
    SetDefinition {
        id: "chin",
        label: "Chin crease",
        onset_age: 35.0,
        full_age: 75.0,
        depth: 6.0,
        width_mm: 1.3,
        lines: &[&[(926.0, 1572.0), (830.0, 1585.0)]],
        both_sides: true,
        open_anchor: true,
    },
    SetDefinition {
        id: "cheeks",
        label: "Cheek folds",
        onset_age: 45.0,
        full_age: 85.0,
        depth: 5.0,
        width_mm: 1.0,
        lines: &[
            &[(380.0, 1000.0), (345.0, 1180.0), (320.0, 1360.0)],
            &[(470.0, 1060.0), (450.0, 1220.0), (430.0, 1380.0)],
            &[(300.0, 980.0), (270.0, 1150.0), (255.0, 1300.0)],
        ],
        both_sides: true,
        open_anchor: false,
    },
];

pub const SET_COUNT: usize = SETS.len();

/// The controls of one wrinkle set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WrinkleSet {
    /// How deep the set is, against its published depth at this age. From 0 to 3, and 1 is the set's default.
    pub strength: f64,
    /// How wide the furrows are, against the set's default width. From 0.5 to 2.
    pub width: f64,
    /// The share of each line that is drawn, from its anchor. From 0.3 to 1.
    pub length: f64,
    /// How many lines are drawn, from 0 to the set's number of lines. A fraction draws the last line at that share of its depth.
    pub count: f64,
    /// How far the set's lines are moved sideways along the skin, in millimetres, from -4 to 4. A positive shift moves a line to the
    /// left of its direction, seen from outside the head, and the mirrored line on the other side moves the mirrored way.
    pub shift_mm: f64,
}

impl WrinkleSet {
    pub const DEFAULT: WrinkleSet = WrinkleSet { strength: 1.0, width: 1.0, length: 1.0, count: 99.0, shift_mm: 0.0 };
}

/// The settings of the wrinkle layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wrinkles {
    /// The age in years, from 15 to 90. It sets each set's weight.
    pub age: f64,
    /// How deep every set is, from 0 to 2. Zero paints nothing.
    pub depth: f64,
    /// How wide every furrow is, from 0.5 to 2.
    pub width: f64,
    /// How far a line wobbles sideways, in millimetres, from 0 to 1.5.
    pub wobble_mm: f64,
    pub seed: u64,
    /// How much darker the fine lines are in their furrows, in L*, from 0 to 10. The fine lines are a net of thin creases over the
    /// skin, as on crepey old skin, besides the named sets. They are painted wherever the pores are (`PoreLandmarks::skin`).
    pub fine: f64,
    pub sets: [WrinkleSet; SET_COUNT],
}

/// The fine lines' depth at full age, in L*, and the ages over which they come in, for `Wrinkles::at_age`.
pub const FINE_DEPTH: f64 = 4.0;
pub const FINE_ONSET_AGE: f64 = 40.0;
pub const FINE_FULL_AGE: f64 = 85.0;
impl Wrinkles {
    pub const NONE: Wrinkles = Wrinkles { age: 15.0, depth: 0.0, width: 1.0, wobble_mm: 0.0, seed: 0, fine: 0.0, sets: [WrinkleSet::DEFAULT; SET_COUNT] };

    /// Every set at its default for an age.
    pub fn at_age(age: f64) -> Wrinkles {
        Wrinkles { age, depth: 1.0, width: 1.0, wobble_mm: 0.5, seed: 1, fine: FINE_DEPTH * smoothstep(FINE_ONSET_AGE, FINE_FULL_AGE, age), sets: [WrinkleSet::DEFAULT; SET_COUNT] }
    }

    /// The weight of a set at the recipe's age, from 0 to 1.
    pub fn age_weight(&self, set: usize) -> f64 {
        let d = &SETS[set];
        smoothstep(d.onset_age, d.full_age, self.age)
    }

    /// How much darker a set's furrows are, in L*, before the changes along each line.
    pub fn set_depth(&self, set: usize) -> f64 {
        self.depth * self.sets[set].strength * SETS[set].depth * self.age_weight(set)
    }
}

pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How much the ridge is raised so that the furrow and the ridge still cancel out after the fade between 3 and 5 widths. Without it the
/// faded profile would darken the skin by 21% of the furrow's area on average.
const RIDGE_BALANCE: f64 = 1.264;

/// Bando's cross-section at `x` widths from the furrow, faded to zero between 3 and 5 widths.
pub fn profile(x: f64) -> f64 {
    if x >= 5.0 {
        return 0.0;
    }
    let s = -(1.0 - x) * (-x).exp();
    if x <= 1.0 {
        s
    } else if x <= 3.0 {
        RIDGE_BALANCE * s
    } else {
        RIDGE_BALANCE * s * smoothstep(5.0, 3.0, x)
    }
}

/// Where the profile reaches, in widths.
pub const REACH: f64 = 5.0;

/// The spacing of the points along a line, in millimetres.
const STEP_MM: f64 = 0.4;

/// One line of a set on a head, before the recipe's wobble: points along it in millimetres, with the skin's normal and the distance
/// along the line at each.
#[derive(Clone, Debug)]
pub struct HeadLine {
    pub set: usize,
    /// The line's place in its set's order of importance.
    pub rank: usize,
    /// A number that differs between the two sides and between lines, for the line's own random choices.
    pub key: u64,
    pub points: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub along_mm: Vec<f64>,
}

/// Every wrinkle line on one head.
#[derive(Clone, Debug, Default)]
pub struct HeadLines {
    pub lines: Vec<HeadLine>,
}

fn catmull_rom(p: &[[f64; 2]], segment: usize, t: f64) -> [f64; 2] {
    let at = |i: isize| p[i.clamp(0, p.len() as isize - 1) as usize];
    let s = segment as isize;
    let (p0, p1, p2, p3) = (at(s - 1), at(s), at(s + 1), at(s + 2));
    let (t2, t3) = (t * t, t * t * t);
    [0, 1].map(|c| 0.5 * (2.0 * p1[c] + (-p0[c] + p2[c]) * t + (2.0 * p0[c] - 5.0 * p1[c] + 4.0 * p2[c] - p3[c]) * t2 + (-p0[c] + 3.0 * p1[c] - 3.0 * p2[c] + p3[c]) * t3))
}

impl HeadLines {
    /// Places every line on a head. `positions` holds the head's positions in metres, one per vertex of the layout. A line's curve is drawn
    /// through its points in the layout and each point of it is then found on the head, so the line lies on the skin.
    pub fn new(locator: &LayoutLocator, positions: &[[f64; 3]]) -> Self {
        let mut lines = Vec::new();
        for (set, def) in SETS.iter().enumerate() {
            for (rank, pixels) in def.lines.iter().enumerate() {
                if def.open_anchor {
                    // One line from the far end on the other side, through the anchor on the middle line, to the far end on this side.
                    let pts: Vec<[f64; 2]> = pixels.iter().rev().map(|&p| uv_of(mirrored(p))).chain(pixels.iter().skip(1).map(|&p| uv_of(p))).collect();
                    let key = (set as u64) << 16 | (rank as u64) << 1;
                    if let Some(line) = Self::lift(locator, positions, &pts, set, rank, key) {
                        lines.push(line);
                    }
                    continue;
                }
                let sides: &[bool] = if def.both_sides { &[false, true] } else { &[false] };
                for &mirror in sides {
                    let pts: Vec<[f64; 2]> = pixels.iter().map(|&p| uv_of(if mirror { mirrored(p) } else { p })).collect();
                    let key = (set as u64) << 16 | (rank as u64) << 1 | u64::from(mirror);
                    if let Some(line) = Self::lift(locator, positions, &pts, set, rank, key) {
                        lines.push(line);
                    }
                }
            }
        }
        HeadLines { lines }
    }

    fn lift(locator: &LayoutLocator, positions: &[[f64; 3]], pts: &[[f64; 2]], set: usize, rank: usize, key: u64) -> Option<HeadLine> {
        // Sample the curve finely in the layout, find each sample on the head, then keep points about STEP_MM apart.
        let mut dense = Vec::new();
        for s in 0..pts.len() - 1 {
            for i in 0..40 {
                dense.push(catmull_rom(pts, s, f64::from(i) / 40.0));
            }
        }
        dense.push(pts[pts.len() - 1]);
        let mut line = HeadLine { set, rank, key, points: Vec::new(), normals: Vec::new(), along_mm: Vec::new() };
        let mut along = 0.0;
        let mut last: Option<[f64; 3]> = None;
        for (i, uv) in dense.iter().enumerate() {
            let Some(spot) = locator.spot(*uv, positions) else { continue };
            if let Some(prev) = last {
                let step = length(sub(spot.position_mm, prev));
                if step < STEP_MM && i + 1 != dense.len() {
                    continue;
                }
                along += step;
            }
            last = Some(spot.position_mm);
            line.points.push(spot.position_mm);
            line.normals.push(spot.normal);
            line.along_mm.push(along);
        }
        // The layout maps to the head one flat triangle at a time, so a smooth curve in the layout has small kinks on the head where it
        // crosses a triangle's edge. Three passes of a [1/4, 1/2, 1/4] average smooth them out over about a millimetre. The ends stay.
        for _ in 0..3 {
            let p = line.points.clone();
            for i in 1..p.len().saturating_sub(1) {
                line.points[i] = [0, 1, 2].map(|c| 0.25 * p[i - 1][c] + 0.5 * p[i][c] + 0.25 * p[i + 1][c]);
            }
        }
        (line.points.len() >= 2).then_some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_profile_has_a_sharp_furrow_soft_ridges_and_ends() {
        assert_eq!(profile(0.0), -1.0);
        assert!(profile(1.0).abs() < 1e-12);
        assert!((profile(2.0) - RIDGE_BALANCE * (-2.0f64).exp()).abs() < 1e-12);
        assert_eq!(profile(5.0), 0.0);
        let area: f64 = (0..5000).map(|i| profile(i as f64 * 0.001 + 0.0005) * 0.001).sum();
        assert!(area.abs() < 0.01, "the furrow and the ridge cancel: {area}");
    }

    #[test]
    fn the_age_switches_the_sets_on_in_order() {
        let young = Wrinkles::at_age(28.0);
        assert!(young.set_depth(0) > 0.0 && young.set_depth(4) == 0.0 && young.set_depth(6) == 0.0);
        let old = Wrinkles::at_age(80.0);
        assert!((0..SET_COUNT).all(|s| old.set_depth(s) > 0.0));
        assert!(old.set_depth(0) > young.set_depth(0));
    }
}
