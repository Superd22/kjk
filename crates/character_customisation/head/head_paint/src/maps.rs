//! Light from maps: the settings of the maps that the GPU painter writes beside the colour texture, so that the fine detail of the skin
//! answers to a light that moves (paint track, part C, `docs/research/2026-11-20-head-detail-maps.md`).
//!
//! While the group's switch is on, the painter writes a second texture. Its red and green bytes hold a tangent-space normal, its blue
//! byte the gloss mask and its alpha byte the mask of the repeated detail tile. Each layer then paints less of its own light into the
//! colour, by the "kept" sliders below. While the switch is off, the colour texture is exactly what it was before these maps existed.

use crate::lips::Slider;

/// The settings of the maps, in the order of `SLIDERS`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightMaps {
    pub skin_gloss: f64,
    pub t_zone_gloss: f64,
    pub sheen_gloss: f64,
    pub lip_gloss: f64,
    pub wet_rims: f64,
    pub dry: f64,
    pub sheen_kept: f64,
    pub lip_shine_kept: f64,
    pub relief_mm: f64,
    pub wrinkle_relief: f64,
    pub lip_relief: f64,
    pub eye_relief: f64,
    pub light_kept: f64,
    pub dark_kept: f64,
    pub pore_rim_kept: f64,
    pub grain_kept: f64,
    pub grain_relief_mm: f64,
}

/// The maps' sliders, in the order of the fields of `LightMaps`. Each `part` names the step of the work it belongs to.
pub const SLIDERS: [(Slider, &str); 17] = [
    (Slider { id: "skin_gloss", label: "Skin gloss", min: 0.0, max: 1.0, default: 0.2, step: 0.01, unit: "", help: "How strongly the whole skin mirrors a light, before the regions below add to it. At 0 the skin is matte, as KOTOR's own heads are." }, "Gloss"),
    (Slider { id: "t_zone_gloss", label: "Oily T-zone", min: 0.0, max: 1.0, default: 0.5, step: 0.01, unit: "", help: "How much more the forehead, the nose and the chin shine than the cheeks. These parts of the face have the most oil glands." }, "Gloss"),
    (Slider { id: "sheen_gloss", label: "Gloss on the sheen", min: 0.0, max: 1.0, default: 0.15, step: 0.01, unit: "", help: "How much more the regions of the sheen layer shine: the cheekbones, the ridge of the nose and the front of the chin." }, "Gloss"),
    (Slider { id: "lip_gloss", label: "Lip gloss", min: 0.0, max: 1.0, default: 0.5, step: 0.01, unit: "", help: "How strongly the lips mirror a light. The lip layer's own gloss slider adds to it, so a wet lip shines more than a dry one." }, "Gloss"),
    (Slider { id: "wet_rims", label: "Wet lid rims", min: 0.0, max: 1.0, default: 0.8, step: 0.01, unit: "", help: "How strongly the wet rim of the lower lid and the pink corner of the eye mirror a light." }, "Gloss"),
    (Slider { id: "dry", label: "Dry skin", min: 0.0, max: 1.0, default: 0.4, step: 0.01, unit: "", help: "How much a strong skin grain takes away from the skin's gloss. Rough, dry skin shines less than smooth skin." }, "Gloss"),
    (Slider { id: "sheen_kept", label: "Painted sheen kept", min: 0.0, max: 1.0, default: 0.3, step: 0.01, unit: "", help: "The share of the sheen layer's painted light that stays in the colour while the light comes from the maps. At 1 the colour keeps all of it." }, "Gloss"),
    (Slider { id: "lip_shine_kept", label: "Painted lip shine kept", min: 0.0, max: 1.0, default: 0.25, step: 0.01, unit: "", help: "The share of the lips' painted gloss that stays in the colour while the light comes from the maps. At 1 the colour keeps all of it." }, "Gloss"),
    (Slider { id: "relief_mm", label: "Relief depth", min: 0.0, max: 1.0, default: 0.35, step: 0.005, unit: "mm", help: "How deep a crease is in the normal map when the layers paint it 10 L* darker than the skin. The normal map tilts the skin on both sides of the crease by this depth." }, "Relief"),
    (Slider { id: "wrinkle_relief", label: "Wrinkles and forehead lines", min: 0.0, max: 2.0, default: 1.0, step: 0.01, unit: "", help: "How much relief the wrinkle sets and the forehead lines give the normal map, against the relief depth above. The fine lines get 30% of it, because at full relief they stand out as scratches." }, "Relief"),
    (Slider { id: "lip_relief", label: "Lips", min: 0.0, max: 2.0, default: 0.45, step: 0.01, unit: "", help: "How much relief the lips give the normal map: the mouth line, the fine vertical lines, the border and the roll of each lip." }, "Relief"),
    (Slider { id: "eye_relief", label: "Crease and bags of the eye", min: 0.0, max: 3.0, default: 2.0, step: 0.01, unit: "", help: "How much relief the crease above the upper lid and the bags under the eye give the normal map." }, "Relief"),
    (Slider { id: "light_kept", label: "Painted light band kept", min: 0.0, max: 1.0, default: 0.3, step: 0.01, unit: "", help: "The share of the painted lighter band beside each crease that stays in the colour while the normal map gives the relief. At 1 the colour keeps all of it." }, "Relief"),
    (Slider { id: "dark_kept", label: "Painted dark line kept", min: 0.0, max: 1.0, default: 0.8, step: 0.01, unit: "", help: "The share of the painted dark line of each crease that stays in the colour while the normal map gives the relief. At 1 the colour keeps all of it." }, "Relief"),
    (Slider { id: "pore_rim_kept", label: "Painted pore rim kept", min: 0.0, max: 1.0, default: 0.15, step: 0.01, unit: "", help: "The share of each pore's painted light rim that stays in the colour while the detail tile gives the pores their relief. The pore's dark middle always stays." }, "Pores"),
    (Slider { id: "grain_kept", label: "Painted light grains kept", min: 0.0, max: 1.0, default: 0.35, step: 0.01, unit: "", help: "The share of the pore layer's painted light grains and dark points that stays in the colour while the normal map gives the grains their relief. At 1 the colour keeps all of it." }, "Pores"),
    (Slider { id: "grain_relief", label: "Light grain height", min: 0.0, max: 0.15, default: 0.04, step: 0.001, unit: "mm", help: "How high each light grain of the pore layer stands in the normal map, when the light grains are at 10 L*. A higher grain breaks the highlight into more and stronger sparks." }, "Pores"),
];

impl LightMaps {
    pub const DEFAULT: LightMaps = LightMaps {
        skin_gloss: SLIDERS[0].0.default,
        t_zone_gloss: SLIDERS[1].0.default,
        sheen_gloss: SLIDERS[2].0.default,
        lip_gloss: SLIDERS[3].0.default,
        wet_rims: SLIDERS[4].0.default,
        dry: SLIDERS[5].0.default,
        sheen_kept: SLIDERS[6].0.default,
        lip_shine_kept: SLIDERS[7].0.default,
        relief_mm: SLIDERS[8].0.default,
        wrinkle_relief: SLIDERS[9].0.default,
        lip_relief: SLIDERS[10].0.default,
        eye_relief: SLIDERS[11].0.default,
        light_kept: SLIDERS[12].0.default,
        dark_kept: SLIDERS[13].0.default,
        pore_rim_kept: SLIDERS[14].0.default,
        grain_kept: SLIDERS[15].0.default,
        grain_relief_mm: SLIDERS[16].0.default,
    };

    pub fn values(&self) -> [f64; 17] {
        [self.skin_gloss, self.t_zone_gloss, self.sheen_gloss, self.lip_gloss, self.wet_rims, self.dry, self.sheen_kept, self.lip_shine_kept, self.relief_mm, self.wrinkle_relief, self.lip_relief, self.eye_relief, self.light_kept, self.dark_kept, self.pore_rim_kept, self.grain_kept, self.grain_relief_mm]
    }

    pub fn field(&mut self, k: usize) -> &mut f64 {
        match k {
            0 => &mut self.skin_gloss,
            1 => &mut self.t_zone_gloss,
            2 => &mut self.sheen_gloss,
            3 => &mut self.lip_gloss,
            4 => &mut self.wet_rims,
            5 => &mut self.dry,
            6 => &mut self.sheen_kept,
            7 => &mut self.lip_shine_kept,
            8 => &mut self.relief_mm,
            9 => &mut self.wrinkle_relief,
            10 => &mut self.lip_relief,
            11 => &mut self.eye_relief,
            12 => &mut self.light_kept,
            13 => &mut self.dark_kept,
            14 => &mut self.pore_rim_kept,
            15 => &mut self.grain_kept,
            _ => &mut self.grain_relief_mm,
        }
    }
}

impl Default for LightMaps {
    fn default() -> Self {
        LightMaps::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slider_reaches_its_own_field() {
        let mut m = LightMaps::DEFAULT;
        for k in 0..SLIDERS.len() {
            *m.field(k) = 100.0 + k as f64;
        }
        for (k, v) in m.values().iter().enumerate() {
            assert_eq!(*v, 100.0 + k as f64);
        }
        for (k, (s, _)) in SLIDERS.iter().enumerate() {
            assert_eq!(LightMaps::DEFAULT.values()[k], s.default, "{}", s.id);
            assert!(s.min <= s.default && s.default <= s.max, "{}", s.id);
        }
    }
}
