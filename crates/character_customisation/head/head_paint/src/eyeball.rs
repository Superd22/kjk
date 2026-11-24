//! The eyeball's paint: the iris, the pupil and the eye white, painted by a procedure instead of copied from KOTOR (paint track, part C,
//! the eye step). Only the GPU painter paints it (`head_paint_gpu/shaders/eyeball.wgsl`).
//!
//! The eyeball's texture holds both eyes side by side, each a square seen from the front: a point of the eyeball whose unit direction from
//! the eyeball's centre is (x, y, z), with x toward the head's x > 0 side, y up and z forward, sits at (x, y) of its square. The eye on
//! the x > 0 side takes the left half. The viewer's eyeball mesh computes the same texture coordinates.
//!
//! Every value is a slider of `SLIDERS`, in that order. Colours are given as lightness (L*), hue in degrees and chroma. Both eyes take the
//! same values, except that the second eye's iris hue is turned by `odd_hue`.

use crate::lips::Slider;

/// The eyeball sliders, in the order of `Eyeball::values`. Each id gets the prefix `eye_` in the control list.
pub const SLIDERS: [Slider; 24] = [
    Slider { id: "iris_hue", label: "Hue", min: 0.0, max: 360.0, default: 60.0, step: 1.0, unit: "degrees", help: "The iris colour's hue around the colour wheel. Brown is near 60 degrees, hazel and amber near 80, green near 130, grey-blue near 230 and violet near 300." },
    Slider { id: "iris_lightness", label: "Lightness", min: 3.0, max: 95.0, default: 30.0, step: 0.5, unit: "L*", help: "How light the iris is. A dark brown iris is about 20, a light blue one about 55. Above 70 the iris looks milky." },
    Slider { id: "iris_chroma", label: "Colour strength", min: 0.0, max: 60.0, default: 18.0, step: 0.5, unit: "", help: "How strong the iris colour is. At 0 the iris is grey." },
    Slider { id: "iris_mm", label: "Iris size", min: 8.0, max: 15.0, default: 11.8, step: 0.05, unit: "mm", help: "The width of the iris as seen from the front. A real iris is about 11.8 mm wide. KOTOR's irises range from about 9 to 14 mm on our eyeball." },
    Slider { id: "fibres", label: "Fibre contrast", min: 0.0, max: 2.0, default: 0.45, step: 0.01, unit: "", help: "How strongly the iris's fibres, blotches and specks stand out. The fibres are fine light and dark streaks that run from the pupil to the iris's edge. At 0 the iris is one smooth colour. Above 1 the iris looks coarse and grainy, as many of KOTOR's low-resolution eyes do." },
    Slider { id: "pattern", label: "Pattern", min: 0.0, max: 99.0, default: 0.0, step: 1.0, unit: "", help: "Picks one of 100 patterns of fibres, crypts and veins. Each number gives another pattern with the same look. The second eye's pattern differs a little, as in real eyes." },
    Slider { id: "ring", label: "Outer ring darkness", min: 0.0, max: 1.0, default: 0.7, step: 0.01, unit: "", help: "How dark the ring at the iris's outer edge is (the limbal ring). At 1 the ring is almost black. Young and light eyes often have a strong ring." },
    Slider { id: "ring_width", label: "Outer ring width", min: 0.02, max: 0.5, default: 0.25, step: 0.005, unit: "", help: "How wide the outer ring is, as a share of the iris's radius. The ring fades in over this width toward the edge." },
    Slider { id: "inner", label: "Amount", min: 0.0, max: 1.0, default: 0.35, step: 0.01, unit: "", help: "How much of the second colour the iris has round the pupil. A hazel eye is brown or amber round the pupil and green further out. At 0 the iris is one colour." },
    Slider { id: "inner_width", label: "Width", min: 0.05, max: 0.95, default: 0.35, step: 0.01, unit: "", help: "How far the second colour reaches from the pupil, as a share of the distance from the pupil's edge to the iris's edge. Its edge is a wavy, lighter ring (the collarette)." },
    Slider { id: "inner_hue", label: "Hue", min: 0.0, max: 360.0, default: 70.0, step: 1.0, unit: "degrees", help: "The second colour's hue. Amber and gold are near 75 degrees." },
    Slider { id: "inner_lightness", label: "Lightness", min: 3.0, max: 95.0, default: 40.0, step: 0.5, unit: "L*", help: "How light the second colour is." },
    Slider { id: "inner_chroma", label: "Colour strength", min: 0.0, max: 60.0, default: 24.0, step: 0.5, unit: "", help: "How strong the second colour is. At 0 it is grey." },
    Slider { id: "pupil", label: "Pupil size", min: 0.15, max: 0.7, default: 0.32, step: 0.005, unit: "", help: "The pupil's width as a share of the iris's width. Real pupils measure 0.26 of the iris in bright light and 0.57 at rest in the dark." },
    Slider { id: "clouding", label: "Clouding", min: 0.0, max: 1.0, default: 0.0, step: 0.01, unit: "", help: "How milky the iris and the pupil are, as in a blind eye or a cataract. At 1 the iris and the pupil are a pale haze of the eye white's colour, with the iris's outline faintly showing. Kreia's eyes are near 0.9." },
    Slider { id: "white_lightness", label: "Lightness", min: 40.0, max: 98.0, default: 72.0, step: 0.5, unit: "L*", help: "How light the eye white is in the middle. KOTOR's eye whites lie between about 60 and 90." },
    Slider { id: "white_hue", label: "Tint hue", min: 0.0, max: 360.0, default: 50.0, step: 1.0, unit: "degrees", help: "The hue of the eye white's tint. Near 30 degrees the white is pinkish, near 85 yellowish and near 240 bluish." },
    Slider { id: "white_chroma", label: "Tint strength", min: 0.0, max: 30.0, default: 6.0, step: 0.5, unit: "", help: "How strong the eye white's tint is. At 0 the white is a pure grey or white." },
    Slider { id: "white_red", label: "Redness", min: 0.0, max: 1.0, default: 0.25, step: 0.01, unit: "", help: "How red the eye white is toward the corners of the eye, with thin veins. At 1 the eye looks bloodshot." },
    Slider { id: "white_shade", label: "Shading", min: 0.0, max: 1.0, default: 0.6, step: 0.01, unit: "", help: "How much darker the eye white is toward its edges and under the upper lid, as the lids shade it. At 0 the white is evenly lit." },
    Slider { id: "catchlight", label: "Catchlight", min: 0.0, max: 1.0, default: 0.4, step: 0.01, unit: "", help: "How bright the painted reflection of a light is, a soft white spot just above the pupil. KOTOR paints one into most of its eyes." },
    Slider { id: "gaze_up_mm", label: "Look up or down", min: -4.0, max: 4.0, default: 0.0, step: 0.05, unit: "mm", help: "Moves both irises up (positive) or down (negative), in millimetres, from the middle of the eye's opening. The iris is painted into the eyeball's texture, as KOTOR does, so this sets where the character looks." },
    Slider { id: "gaze_in_mm", label: "Look inward or outward", min: -3.0, max: 3.0, default: 0.0, step: 0.05, unit: "mm", help: "Moves both irises toward the nose (positive) or toward the temples (negative), in millimetres, from the middle of the eye's opening." },
    Slider { id: "odd_hue", label: "Other eye's hue", min: -180.0, max: 180.0, default: 0.0, step: 1.0, unit: "degrees", help: "Turns the iris hue of the eye on the head's right side (on the x < 0 side) by this much, for eyes of two colours. At 0 both eyes are alike." },
];

/// The index of each slider in `SLIDERS` and in `Eyeball::values`.
pub mod k {
    pub const IRIS_HUE: usize = 0;
    pub const IRIS_LIGHTNESS: usize = 1;
    pub const IRIS_CHROMA: usize = 2;
    pub const IRIS_MM: usize = 3;
    pub const FIBRES: usize = 4;
    pub const PATTERN: usize = 5;
    pub const RING: usize = 6;
    pub const RING_WIDTH: usize = 7;
    pub const INNER: usize = 8;
    pub const INNER_WIDTH: usize = 9;
    pub const INNER_HUE: usize = 10;
    pub const INNER_LIGHTNESS: usize = 11;
    pub const INNER_CHROMA: usize = 12;
    pub const PUPIL: usize = 13;
    pub const CLOUDING: usize = 14;
    pub const WHITE_LIGHTNESS: usize = 15;
    pub const WHITE_HUE: usize = 16;
    pub const WHITE_CHROMA: usize = 17;
    pub const WHITE_RED: usize = 18;
    pub const WHITE_SHADE: usize = 19;
    pub const CATCHLIGHT: usize = 20;
    pub const GAZE_UP_MM: usize = 21;
    pub const GAZE_IN_MM: usize = 22;
    pub const ODD_HUE: usize = 23;
}

/// The eyeball's settings: one value per slider of `SLIDERS`, in that order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eyeball {
    pub values: [f64; SLIDERS.len()],
}

impl Eyeball {
    /// Every slider at its default: a plain mid-brown eye.
    pub const DEFAULT: Eyeball = {
        let mut values = [0.0; SLIDERS.len()];
        let mut i = 0;
        while i < SLIDERS.len() {
            values[i] = SLIDERS[i].default;
            i += 1;
        }
        Eyeball { values }
    };

    pub fn field(&mut self, i: usize) -> &mut f64 {
        &mut self.values[i]
    }

    pub fn get(&self, i: usize) -> f64 {
        self.values[i]
    }

    /// The whole-number pattern, from 0 to 99.
    pub fn pattern(&self) -> u32 {
        self.values[k::PATTERN].round().clamp(0.0, 99.0) as u32
    }

    /// The iris's radius as a share of the radius of an eyeball of `radius_mm` (`eyes::eyeball_radii`), which is also its radius in the
    /// texture's square. The iris keeps its width in millimetres on a larger or smaller eyeball, and so does the pupil, which is a share
    /// of the iris.
    pub fn iris_share(&self, radius_mm: f64) -> f64 {
        (0.5 * self.values[k::IRIS_MM] / radius_mm).min(0.95)
    }
}

/// The heading of slider `i` on the page.
pub fn part(i: usize) -> &'static str {
    match i {
        0..=7 => "Iris",
        8..=12 => "Second colour round the pupil",
        13 | 14 => "Pupil",
        15..=19 => "Eye white",
        21 | 22 => "Gaze",
        _ => "Light and other eye",
    }
}

/// Where each iris sits on its eyeball's square, in millimetres across (toward the head's x > 0 side) and up from the eyeball's front:
/// the middle of the eye's opening, moved by the gaze sliders, at most 5 mm from the front. The first eye is the one on the x < 0 side.
/// The middle of the opening is halfway between its two corners across, and halfway between the upper lid's highest point and the lower
/// lid's lowest point up. A head that is not on the default mesh gets the eyeball's front.
pub fn iris_centres(positions: &[[f64; 3]], ball: &Eyeball) -> [[f64; 2]; 2] {
    iris_centres_on(positions, ball, crate::eyes::LidEdge::Loop)
}

/// `iris_centres` with the opening read from the lids' edges `edge`.
pub fn iris_centres_on(positions: &[[f64; 3]], ball: &Eyeball, edge: crate::eyes::LidEdge) -> [[f64; 2]; 2] {
    let Some(eyes) = crate::eyes::eye_shapes_on(positions, edge) else { return [[0.0; 2]; 2] };
    [0, 1].map(|side| {
        let e = &eyes[side];
        let rel = |p: [f64; 3]| [p[0] - e.centre[0], p[1] - e.centre[1]];
        let across = 0.5 * (rel(e.inner())[0] + rel(e.outer())[0]);
        let top = e.upper.iter().map(|&p| rel(p)[1]).fold(f64::NEG_INFINITY, f64::max);
        let bottom = e.lower.iter().map(|&p| rel(p)[1]).fold(f64::INFINITY, f64::min);
        // Toward the nose is toward x = 0, which is +x for the eye on the x < 0 side.
        let inward = if side == 0 { 1.0 } else { -1.0 };
        let x = across + inward * ball.get(k::GAZE_IN_MM);
        let y = 0.5 * (top + bottom) + ball.get(k::GAZE_UP_MM);
        [x.clamp(-5.0, 5.0), y.clamp(-5.0, 5.0)]
    })
}

/// The texture of the two eyeballs: its width is twice its height, and each eye's square is `EYE_PIXELS` wide.
pub const EYE_PIXELS: u32 = 512;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_lie_in_their_ranges_and_ids_are_unique() {
        let mut ids: Vec<&str> = SLIDERS.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), SLIDERS.len());
        for (i, s) in SLIDERS.iter().enumerate() {
            assert!(s.min <= s.default && s.default <= s.max && s.step > 0.0, "{}", s.id);
            assert_eq!(Eyeball::DEFAULT.get(i), s.default);
        }
        assert!((Eyeball::DEFAULT.iris_share(crate::eyes::EYEBALL_RADIUS_MM) - 0.4797).abs() < 1e-3);
    }

    #[test]
    fn the_iris_keeps_its_width_on_a_larger_eyeball() {
        let b = Eyeball::DEFAULT;
        let width = |r: f64| 2.0 * b.iris_share(r) * r;
        assert!((width(crate::eyes::EYEBALL_RADIUS_MM) - 11.8).abs() < 1e-9 && (width(13.5) - 11.8).abs() < 1e-9);
    }
}
