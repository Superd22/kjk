//! The colour of the skin under light (paint track, part C, skin colour round of 2026-11-20): how the painted light and shade change the
//! skin's colour, and colour zones that change the two pigments by place. Only the GPU painter paints them (`head_paint_gpu`,
//! `shaders/skin.wgsl`). With every slider at zero, the paint is exactly what it was before these sliders existed.
//!
//! **Light along the skin's own colours.** Light that enters skin comes out tinted by the blood and the melanin it passed. So on real skin,
//! and on KOTOR's painted heads, the lit skin is more saturated than the skin in shade, the edge of a shadow turns red-brown, and only strong
//! highlights go toward white. The measure of 2026-11-20 on six KOTOR heads found the chroma rising with lightness on every head, by 0.02
//! to 0.45 chroma per L*, while ours fell by up to 0.14. These sliders act on the shading, the sheen, and the light and dark of the
//! wrinkles, the forehead lines and the pores. They never change a texel's lightness, only its colour at that lightness.
//!
//! **Colour zones.** Each zone adds blood or multiplies the melanin over a soft area placed from the head's landmarks, so the zones follow
//! the shape sliders. The colour of the changed amounts is read from the same pigment model as the base tone, with the base tone's colour
//! strength and hue controls applied.

use crate::lips::Slider;

/// The skin's light and zones, in the order of `SLIDERS`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkinColour {
    pub linear_shade: f64,
    pub red_shade: f64,
    pub warm_light: f64,
    pub strong_white: f64,
    pub cheeks_blood: f64,
    pub nose_blood: f64,
    pub ears_blood: f64,
    pub lips_blood: f64,
    pub chin_blood: f64,
    pub forehead_yellow: f64,
    pub eyes_dark: f64,
    pub mouth_dark: f64,
    pub middle_light: f64,
    pub zone_size: f64,
    pub front_light: f64,
}

/// The blood volume fraction a zone slider adds at 1 in the middle of its zone.
pub const ZONE_BLOOD: f64 = 0.03;
/// The melanin factor a darkening zone slider applies at 1 in the middle of its zone, as a power of two: 2^0.3, about 1.23 times.
pub const ZONE_MELANIN_LOG2: f64 = 0.3;
/// The table of colours the shader reads the zones from: melanin factors 2^u for u from -1 to 1 in `TABLE_U` steps, and blood added
/// from `TABLE_BLOOD[0]` to `TABLE_BLOOD[1]` in `TABLE_B` steps.
pub const TABLE_U: usize = 9;
pub const TABLE_B: usize = 13;
pub const TABLE_BLOOD: [f64; 2] = [-0.04, 0.08];

/// The sliders, with the id of the group each one sits in and its part inside the group.
pub const SLIDERS: [(Slider, &str, &str); 15] = [
    (Slider { id: "skin_linear_shade", label: "Shade in linear light", min: 0.0, max: 1.0, default: 0.0, step: 0.01, unit: "", help: "How the painted shade darkens the skin. At 0 the shade keeps the skin's colour strength, as before. At 1 it dims the light that reaches the skin, as a real shadow does, so the skin in shade is darker and less colourful and the lit skin stands out warmer." }, "shading", "Light in the skin"),
    (Slider { id: "skin_red_shade", label: "Red in the shade", min: 0.0, max: 1.0, default: 0.0, step: 0.01, unit: "", help: "How much the shade takes the colour of light that travelled under the skin. The shade turns toward the skin's own reddest hue, red-brown on dark skin and pink-red on pale skin, without changing how dark it is." }, "shading", "Light in the skin"),
    (Slider { id: "skin_warm_light", label: "Warm light", min: 0.0, max: 1.0, default: 0.0, step: 0.01, unit: "", help: "How much more colourful the lit skin gets, in its own hue, without changing how light it is. At 1 a lightening that doubles the light gives the skin 1.5 times its colour strength." }, "shading", "Light in the skin"),
    (Slider { id: "skin_strong_white", label: "White only in strong light", min: 0.0, max: 1.0, default: 0.0, step: 0.01, unit: "", help: "At 0 every lightening goes toward white by the share \"Highlight whiteness\" gives. At 1 a lightening of 4 L* or less keeps the skin's colour, and only a lightening of 24 L* or more takes the whole whiteness." }, "shading", "Light in the skin"),
    (Slider { id: "skin_cheeks_blood", label: "Blood in the cheeks", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Adds blood to the skin over the cheeks. At 1 the middle of each cheek has 0.03 more blood volume, which is a blush on pale skin and a warmer brown on dark skin." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_nose_blood", label: "Blood on the nose", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Adds blood to the skin along the nose, most at its tip and wings. At 1 the nose has 0.03 more blood volume." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_ears_blood", label: "Blood in the ears", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Adds blood to the ears, which are thin and show the blood under them. At 1 the ears have 0.03 more blood volume." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_lips_blood", label: "Blood round the lips", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Adds blood to the skin in a ring round the lips, not on the lips themselves. At 1 the ring has 0.03 more blood volume." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_chin_blood", label: "Blood on the chin", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Adds blood to the front of the chin. At 1 the chin has 0.03 more blood volume." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_forehead_yellow", label: "Yellower forehead", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Takes blood away from the forehead and adds a little melanin, so the forehead is more yellow-brown than the cheeks. At 1 the forehead has 0.03 less blood and 1.11 times the melanin." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_eyes_dark", label: "Darker round the eyes", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Adds melanin to the skin round the eyes, most under them and at their corners. At 1 that skin has 1.23 times the melanin." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_mouth_dark", label: "Darker round the mouth", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Adds melanin to the skin in a ring round the lips, as on many dark-skinned faces. At 1 that ring has 1.23 times the melanin." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_middle_light", label: "Lighter middle of the face", min: -1.0, max: 2.0, default: 0.0, step: 0.01, unit: "", help: "Takes melanin away from the middle of the face, from the forehead down the nose to the chin, and adds a little blood, so the middle is lighter and warmer than the sides. At 1 the middle has 0.81 times the melanin and 0.01 more blood." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_zone_size", label: "Zone size", min: 0.5, max: 2.0, default: 1.0, step: 0.01, unit: "", help: "Multiplies the size of every colour zone. A larger size spreads each zone's colour over more of the face with a softer border." }, "base_tone", "Colour zones"),
    (Slider { id: "skin_front_light", label: "Light from the front", min: 0.0, max: 1.5, default: 0.0, step: 0.01, unit: "", help: "A painted light from in front of the face, as KOTOR paints into its heads: the middle of the face lighter and its sides darker, as the skin turns away. At 1 the middle is 12 L* lighter, the skin turned halfway away is unchanged, and the far sides and the back of the head are up to 24 L* darker. It moves the colour along the skin's own colours as the shading does." }, "shading", "Light in the skin"),
];

impl SkinColour {
    /// Every slider at the value that changes nothing.
    pub const NONE: SkinColour = SkinColour {
        linear_shade: 0.0,
        red_shade: 0.0,
        warm_light: 0.0,
        strong_white: 0.0,
        cheeks_blood: 0.0,
        nose_blood: 0.0,
        ears_blood: 0.0,
        lips_blood: 0.0,
        chin_blood: 0.0,
        forehead_yellow: 0.0,
        eyes_dark: 0.0,
        mouth_dark: 0.0,
        middle_light: 0.0,
        zone_size: 1.0,
        front_light: 0.0,
    };

    pub fn values(&self) -> [f64; 15] {
        [
            self.linear_shade,
            self.red_shade,
            self.warm_light,
            self.strong_white,
            self.cheeks_blood,
            self.nose_blood,
            self.ears_blood,
            self.lips_blood,
            self.chin_blood,
            self.forehead_yellow,
            self.eyes_dark,
            self.mouth_dark,
            self.middle_light,
            self.zone_size,
            self.front_light,
        ]
    }

    pub fn field(&mut self, k: usize) -> &mut f64 {
        match k {
            0 => &mut self.linear_shade,
            1 => &mut self.red_shade,
            2 => &mut self.warm_light,
            3 => &mut self.strong_white,
            4 => &mut self.cheeks_blood,
            5 => &mut self.nose_blood,
            6 => &mut self.ears_blood,
            7 => &mut self.lips_blood,
            8 => &mut self.chin_blood,
            9 => &mut self.forehead_yellow,
            10 => &mut self.eyes_dark,
            11 => &mut self.mouth_dark,
            12 => &mut self.middle_light,
            13 => &mut self.zone_size,
            _ => &mut self.front_light,
        }
    }

    /// Whether the light sliders change nothing.
    pub fn light_is_none(&self) -> bool {
        self.linear_shade == 0.0 && self.red_shade == 0.0 && self.warm_light == 0.0 && self.strong_white == 0.0
    }

    /// Whether the zone sliders change nothing.
    pub fn zones_are_none(&self) -> bool {
        self.values()[4..13].iter().all(|&v| v == 0.0)
    }

    /// The linear sRGB colours of the zones' table for a base tone, entry (i, j) at melanin factor 2^(-1 + 2 i / (TABLE_U - 1)) and
    /// blood added `TABLE_BLOOD[0] + j` steps. Entry (TABLE_U / 2, 4) is the base tone itself.
    /// Blood beyond the pigment table's `HAEMOGLOBIN_MAX` is read from the spectral model itself, because several fitted base tones sit at
    /// that maximum and a zone must still be able to add blood there.
    pub fn table(tone: &crate::pigment::BaseTone, pigments: &crate::pigment::PigmentTable, model: &crate::pigment::SpectralModel) -> Vec<[f64; 3]> {
        let mut out = Vec::with_capacity(TABLE_U * TABLE_B);
        for j in 0..TABLE_B {
            for i in 0..TABLE_U {
                let u = -1.0 + 2.0 * i as f64 / (TABLE_U - 1) as f64;
                let d = TABLE_BLOOD[0] + (TABLE_BLOOD[1] - TABLE_BLOOD[0]) * j as f64 / (TABLE_B - 1) as f64;
                let t = crate::pigment::BaseTone { melanin: tone.melanin * 2f64.powf(u), haemoglobin: (tone.haemoglobin + d).max(0.0), ..*tone };
                out.push(if u == 0.0 && d.abs() < 1e-12 {
                    tone.linear_rgb(pigments)
                } else if t.haemoglobin > crate::pigment::HAEMOGLOBIN_MAX {
                    t.adjusted(model.colour(t.melanin.min(crate::pigment::MELANIN_MAX), t.haemoglobin))
                } else {
                    t.linear_rgb(pigments)
                });
            }
        }
        out
    }
}

impl Default for SkinColour {
    fn default() -> Self {
        SkinColour::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slider_reaches_its_own_field() {
        let mut s = SkinColour::NONE;
        for k in 0..SLIDERS.len() {
            *s.field(k) = 100.0 + k as f64;
        }
        for (k, v) in s.values().iter().enumerate() {
            assert_eq!(*v, 100.0 + k as f64);
        }
        for (k, (sl, group, _)) in SLIDERS.iter().enumerate() {
            assert_eq!(SkinColour::NONE.values()[k], sl.default, "{}", sl.id);
            assert!(sl.min <= sl.default && sl.default <= sl.max, "{}", sl.id);
            assert!(["shading", "base_tone"].contains(group));
        }
        assert!(SkinColour::NONE.light_is_none() && SkinColour::NONE.zones_are_none());
    }

    #[test]
    fn the_table_holds_the_base_tone_at_its_centre() {
        let pigments = crate::pigment::PigmentTable::build();
        let tone = crate::pigment::BaseTone { melanin: 0.2, haemoglobin: 0.01, chroma_scale: 0.5, undertone_degrees: -5.0, unnatural_degrees: 0.0 };
        let t = SkinColour::table(&tone, &pigments, &crate::pigment::SpectralModel::new());
        assert_eq!(t[4 * TABLE_U + TABLE_U / 2], tone.linear_rgb(&pigments));
        // More blood is redder, more melanin is darker.
        let lab = |c: [f64; 3]| crate::pigment::linear_to_lab(c);
        assert!(lab(t[8 * TABLE_U + TABLE_U / 2])[1] > lab(t[4 * TABLE_U + TABLE_U / 2])[1]);
        assert!(lab(t[4 * TABLE_U + TABLE_U - 1])[0] < lab(t[4 * TABLE_U + TABLE_U / 2])[0]);
    }
}
