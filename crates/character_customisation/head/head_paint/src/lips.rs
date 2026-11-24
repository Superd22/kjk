//! The lip paint: the lips' colour, their border with the skin, the dark line where they meet, and the light and shade KOTOR paints into
//! lips, computed from the lip border of the head it is painted on (paint track, part C3).
//!
//! **Where the lips are.** Four curves of 43 anchors each give the upper border, the lower border, and the upper and lower edges of the
//! opening, from one mouth corner to the other. Each anchor is three mesh vertices and their weights, so the curves follow any head and
//! any slider that reshapes the lips. They were read on the median of KOTOR's 57 bakes in our layout, where KOTOR paints the border
//! (`head_paint_gpu` tool `lip_border`). The x > 0 half is the mirror image of the x < 0 half in 3D.
//!
//! **The mouth's frame.** The curves are laid out in a frame of the mouth: `u` runs across the mouth, measured around a vertical
//! cylinder of radius 50 mm whose front passes through the middle of the opening, so that the lips' curve round the face is unrolled.
//! `v` runs up the face, and `w` is the height off that cylinder, forward being positive. The paint reads each pixel's point in this
//! frame. The border, the opening and every slider's change of shape are curves of `v` against `u`, sampled every 0.25 mm.
//!
//! **Inside the mouth.** The lips curl back into the mouth, and the inner surfaces show when the mouth opens. A map of the head's
//! frontmost surface over the mouth (`w` at each `u` and `v`, every 0.5 mm) tells how far behind the front a point lies. Points more than
//! 1 to 3 mm behind it, between the borders, get the lip colour whatever their place, and darken slowly deeper in, so the paint has no
//! edge where the lip turns into the mouth.
//!
//! **Two halves.** This file holds the processor's half: the sliders, the curves after the sliders' changes and the front map. The paint
//! per pixel is the shader `lips.wgsl` of `head_paint_gpu`.
//!
//! # The sliders
//!
//! `SLIDERS` holds the list for the live page, with each one's range, default and help sentence.

use crate::folds::Anchor;
use crate::folds::Anchors;
use crate::layout_points::{cross, dot, normalize, sub};
use crate::pigment::{BaseTone, SpectralModel, linear_to_lab};

/// The radius of the cylinder the mouth is unrolled on, in millimetres.
pub const RADIUS_MM: f64 = 50.0;
/// The step of the curves along `u`, in millimetres.
pub const STEP_MM: f64 = 0.25;
/// How far the curves go on past each corner, in millimetres.
pub const PAST_CORNER_MM: f64 = 8.0;
/// The front map's cell, in millimetres, and its extent around the middle of the opening.
pub const FRONT_CELL_MM: f64 = 0.5;
pub const FRONT_U_MM: f64 = 45.0;
pub const FRONT_V_MM: [f64; 2] = [-30.0, 25.0];

pub struct Slider {
    pub id: &'static str,
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step: f64,
    pub unit: &'static str,
    pub help: &'static str,
}

/// The lip sliders, in the order of the fields of `Lips`.
pub const SLIDERS: [Slider; 21] = [
    Slider { id: "blood", label: "Lip redness", min: 0.0, max: 1.0, default: 0.25, step: 0.002, unit: "", help: "How much more blood the lips have than the skin. The natural lip colour is the skin's own pigments with this much more blood, so it follows the skin tone. More blood makes the lips both redder and darker, so a light pink lip needs a negative lip darkness too. At 0 the lips are the skin's colour." },
    Slider { id: "darkness", label: "Lip darkness", min: -30.0, max: 45.0, default: -6.0, step: 0.1, unit: "L*", help: "How much darker the natural lip colour is than the redness alone makes it, in L*. A negative value makes the lips paler." },
    Slider { id: "tint_degrees", label: "Lip tint", min: -40.0, max: 40.0, default: -12.0, step: 0.5, unit: "degrees", help: "Turns the natural lip colour's hue. A negative value makes the lips pinker, and a positive value browner or more orange." },
    Slider { id: "stick", label: "Lipstick", min: 0.0, max: 1.0, default: 0.0, step: 0.01, unit: "", help: "How much lipstick covers the natural lip colour. At 0 there is none, and at 1 the lips are fully the lipstick's colour." },
    Slider { id: "stick_lightness", label: "Lipstick lightness", min: 5.0, max: 90.0, default: 35.0, step: 0.5, unit: "L*", help: "How light the lipstick is, from near black at 5 to very pale at 90." },
    Slider { id: "stick_chroma", label: "Lipstick strength", min: 0.0, max: 80.0, default: 40.0, step: 0.5, unit: "", help: "How strong the lipstick's colour is. At 0 it is grey, black or white." },
    Slider { id: "stick_hue", label: "Lipstick hue", min: 0.0, max: 360.0, default: 20.0, step: 1.0, unit: "degrees", help: "The lipstick's hue around the colour wheel. 0 to 40 degrees is red, 60 is orange-brown, 100 is yellow, 160 is green, 270 is blue and 330 is purple-pink." },
    Slider { id: "softness_mm", label: "Border softness", min: 0.1, max: 4.0, default: 1.2, step: 0.05, unit: "mm", help: "How wide the blend between lip and skin is, in millimetres. A small value gives a crisp, painted border, and a large value a soft natural one." },
    Slider { id: "upper_mm", label: "Upper lip size", min: -4.0, max: 4.0, default: 0.0, step: 0.05, unit: "mm", help: "Moves the upper lip's painted border up, by this many millimetres at the middle and less toward the corners. A negative value moves it down, inside the lip's shape." },
    Slider { id: "lower_mm", label: "Lower lip size", min: -4.0, max: 6.0, default: 2.5, step: 0.05, unit: "mm", help: "Moves the lower lip's painted border down, by this many millimetres at the middle and less toward the corners. A negative value moves it up." },
    Slider { id: "width_mm", label: "Lip width", min: -6.0, max: 6.0, default: 0.0, step: 0.05, unit: "mm", help: "Moves each painted mouth corner outward by this many millimetres, or inward for a negative value." },
    Slider { id: "bow_mm", label: "Cupid's bow", min: -1.5, max: 2.0, default: 0.0, step: 0.05, unit: "mm", help: "Raises the two peaks of the upper lip's border and lowers the dip between them, by this many millimetres. A negative value flattens the bow." },
    Slider { id: "lopsided_mm", label: "Lopsided lips", min: -2.0, max: 2.0, default: 0.0, step: 0.05, unit: "mm", help: "Makes the lips fuller on one side and thinner on the other, by this many millimetres at the side's widest point. At 0 both sides are the same." },
    Slider { id: "upper_shade", label: "Darker upper lip", min: -20.0, max: 25.0, default: 4.0, step: 0.1, unit: "L*", help: "How much darker the upper lip is made, in L*, as if lit from above. A negative value darkens the lower lip by that much instead." },
    Slider { id: "highlight", label: "Lower lip light", min: 0.0, max: 20.0, default: 9.0, step: 0.1, unit: "L*", help: "How much the light models the lips, in L*. The lower lip is this much lighter at its fullest part than at its edges, and the upper lip gets a softer light along its roll. Both lips darken toward the corners." },
    Slider { id: "gloss", label: "Gloss", min: 0.0, max: 1.0, default: 0.25, step: 0.01, unit: "", help: "How wet the lips look. At 0 the lips are matte. More gloss adds whitish light to the fullest part of the lower lip and a little to the upper lip, broken into thin vertical streaks and small bright specks, and deepens the lip lines." },
    Slider { id: "shadow", label: "Shadow under lip", min: 0.0, max: 20.0, default: 3.0, step: 0.1, unit: "L*", help: "How much darker the skin just under the lower lip is, in L*, as the lip casts a soft shadow about 3 mm tall." },
    Slider { id: "mouth_line", label: "Mouth line", min: 0.0, max: 50.0, default: 18.0, step: 0.1, unit: "L*", help: "How dark the line where the lips meet is, in L*. The mouth corners darken with it and the shadow spreads a little past them." },
    Slider { id: "liner", label: "Lip liner", min: 0.0, max: 30.0, default: 3.0, step: 0.1, unit: "L*", help: "How much darker a thin band just inside the border is, in L*, like a lip liner. At 0 there is none." },
    Slider { id: "white_roll", label: "Light rim", min: 0.0, max: 15.0, default: 2.0, step: 0.1, unit: "L*", help: "How much lighter the thin ridge of skin just outside the border is, in L*. Real lips have this pale rim, most of all along the upper lip." },
    Slider { id: "lines", label: "Lip lines", min: 0.0, max: 15.0, default: 3.0, step: 0.1, unit: "L*", help: "How dark the fine vertical lines across the lips are, in L*. They are about 1 mm apart and are most visible on the lower lip." },
];

/// The lip sliders. Their meaning, ranges and defaults are in `SLIDERS`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lips {
    pub blood: f64,
    pub darkness: f64,
    pub tint_degrees: f64,
    pub stick: f64,
    pub stick_lightness: f64,
    pub stick_chroma: f64,
    pub stick_hue: f64,
    pub softness_mm: f64,
    pub upper_mm: f64,
    pub lower_mm: f64,
    pub width_mm: f64,
    pub bow_mm: f64,
    pub lopsided_mm: f64,
    pub upper_shade: f64,
    pub highlight: f64,
    pub gloss: f64,
    pub shadow: f64,
    pub mouth_line: f64,
    pub liner: f64,
    pub white_roll: f64,
    pub lines: f64,
}

impl Lips {
    /// Every slider at its default.
    pub const DEFAULT: Lips = Lips {
        blood: 0.25,
        darkness: -6.0,
        tint_degrees: -12.0,
        stick: 0.0,
        stick_lightness: 35.0,
        stick_chroma: 40.0,
        stick_hue: 20.0,
        softness_mm: 1.2,
        upper_mm: 0.0,
        lower_mm: 2.5,
        width_mm: 0.0,
        bow_mm: 0.0,
        lopsided_mm: 0.0,
        upper_shade: 4.0,
        highlight: 9.0,
        gloss: 0.25,
        shadow: 3.0,
        mouth_line: 18.0,
        liner: 3.0,
        white_roll: 2.0,
        lines: 3.0,
    };

    /// Lips that paint nothing: no extra colour and no light or shade. The shape sliders keep their defaults.
    pub const NONE: Lips = Lips { blood: 0.0, darkness: 0.0, tint_degrees: 0.0, stick: 0.0, upper_shade: 0.0, highlight: 0.0, gloss: 0.0, shadow: 0.0, mouth_line: 0.0, liner: 0.0, white_roll: 0.0, lines: 0.0, ..Lips::DEFAULT };

    /// Whether the lips paint nothing.
    pub fn is_none(&self) -> bool {
        self.blood == 0.0 && self.darkness == 0.0 && self.tint_degrees == 0.0 && self.stick == 0.0 && self.upper_shade == 0.0 && self.highlight == 0.0 && self.shadow == 0.0 && self.mouth_line == 0.0 && self.liner == 0.0 && self.white_roll == 0.0 && self.lines == 0.0
    }

    pub fn values(&self) -> [f64; 21] {
        [self.blood, self.darkness, self.tint_degrees, self.stick, self.stick_lightness, self.stick_chroma, self.stick_hue, self.softness_mm, self.upper_mm, self.lower_mm, self.width_mm, self.bow_mm, self.lopsided_mm, self.upper_shade, self.highlight, self.gloss, self.shadow, self.mouth_line, self.liner, self.white_roll, self.lines]
    }

    pub fn field(&mut self, k: usize) -> &mut f64 {
        match k {
            0 => &mut self.blood,
            1 => &mut self.darkness,
            2 => &mut self.tint_degrees,
            3 => &mut self.stick,
            4 => &mut self.stick_lightness,
            5 => &mut self.stick_chroma,
            6 => &mut self.stick_hue,
            7 => &mut self.softness_mm,
            8 => &mut self.upper_mm,
            9 => &mut self.lower_mm,
            10 => &mut self.width_mm,
            11 => &mut self.bow_mm,
            12 => &mut self.lopsided_mm,
            13 => &mut self.upper_shade,
            14 => &mut self.highlight,
            15 => &mut self.gloss,
            16 => &mut self.shadow,
            17 => &mut self.mouth_line,
            18 => &mut self.liner,
            19 => &mut self.white_roll,
            _ => &mut self.lines,
        }
    }

    /// The change in Lab from the skin's colour to the natural lip colour: the same pigments with `blood` more haemoglobin, turned by the
    /// tone's colour controls and then by `tint_degrees`, its a* and b* change made stronger past 0.4 more blood, then `darkness` darker.
    pub fn natural_shift(&self, tone: &BaseTone, model: &SpectralModel) -> [f64; 3] {
        let adjust = |lab: [f64; 3]| crate::pigment::rotate_hue(crate::pigment::scale_chroma(lab, tone.chroma_scale), tone.undertone_degrees + tone.unnatural_degrees);
        let skin = adjust(linear_to_lab(model.colour(tone.melanin, tone.haemoglobin)));
        let lip = crate::pigment::rotate_hue(adjust(linear_to_lab(model.colour(tone.melanin, tone.haemoglobin + self.blood.max(0.0)))), self.tint_degrees);
        // The pigment model's redness levels off above about 0.4 more blood, while KOTOR paints lips up to 25 a* redder than the skin. Past
        // 0.4 the lip's change of a* and b* from the skin grows by 1.2 times the extra blood, up to 1.72 times at 1.
        let boost = 1.0 + 1.2 * (self.blood - 0.4).max(0.0);
        [lip[0] - skin[0] - self.darkness, boost * (lip[1] - skin[1]), boost * (lip[2] - skin[2])]
    }

    /// The lipstick's colour in Lab.
    pub fn stick_lab(&self) -> [f64; 3] {
        let (s, c) = self.stick_hue.to_radians().sin_cos();
        [self.stick_lightness, self.stick_chroma * c, self.stick_chroma * s]
    }
}

pub const UPPER: [Anchor; 43] = [
    Anchor { vertices: [443, 1750, 1751], weights: [0.39515, 0.52482, 0.08004] },
    Anchor { vertices: [434, 432, 463], weights: [0.03615, 0.11066, 0.85319] },
    Anchor { vertices: [429, 432, 433], weights: [0.02303, 0.76377, 0.21320] },
    Anchor { vertices: [429, 433, 430], weights: [0.01331, 0.47021, 0.51648] },
    Anchor { vertices: [430, 482, 481], weights: [0.41829, 0.02193, 0.55978] },
    Anchor { vertices: [495, 494, 481], weights: [0.12068, 0.07013, 0.80919] },
    Anchor { vertices: [495, 481, 480], weights: [0.04695, 0.35903, 0.59402] },
    Anchor { vertices: [501, 480, 483], weights: [0.14071, 0.67449, 0.18480] },
    Anchor { vertices: [501, 480, 483], weights: [0.60036, 0.08569, 0.31395] },
    Anchor { vertices: [500, 501, 502], weights: [0.04537, 0.52865, 0.42598] },
    Anchor { vertices: [500, 501, 502], weights: [0.48476, 0.02162, 0.49363] },
    Anchor { vertices: [500, 502, 503], weights: [0.56701, 0.07287, 0.36012] },
    Anchor { vertices: [506, 500, 503], weights: [0.34525, 0.34237, 0.31238] },
    Anchor { vertices: [506, 500, 503], weights: [0.74088, 0.05454, 0.20459] },
    Anchor { vertices: [510, 506, 507], weights: [0.12158, 0.75237, 0.12605] },
    Anchor { vertices: [510, 506, 507], weights: [0.47955, 0.42015, 0.10031] },
    Anchor { vertices: [510, 506, 507], weights: [0.81638, 0.02181, 0.16181] },
    Anchor { vertices: [514, 510, 511], weights: [0.13081, 0.67170, 0.19749] },
    Anchor { vertices: [514, 510, 511], weights: [0.42916, 0.37526, 0.19558] },
    Anchor { vertices: [514, 510, 511], weights: [0.67847, 0.16015, 0.16138] },
    Anchor { vertices: [514, 511, 515], weights: [0.86876, 0.12945, 0.00179] },
    Anchor { vertices: [514, 511, 515], weights: [0.88746, 0.01106, 0.10149] },
    Anchor { vertices: [514, 515, 2896], weights: [0.87293, 0.01594, 0.11113] },
    Anchor { vertices: [514, 2896, 2895], weights: [0.69252, 0.15543, 0.15205] },
    Anchor { vertices: [514, 2896, 2895], weights: [0.43637, 0.18762, 0.37601] },
    Anchor { vertices: [514, 2896, 2895], weights: [0.13112, 0.18682, 0.68206] },
    Anchor { vertices: [2895, 2892, 2891], weights: [0.81481, 0.14990, 0.03530] },
    Anchor { vertices: [2895, 2892, 2891], weights: [0.47531, 0.09272, 0.43197] },
    Anchor { vertices: [2895, 2892, 2891], weights: [0.11284, 0.12123, 0.76593] },
    Anchor { vertices: [2891, 2886, 2885], weights: [0.69029, 0.19681, 0.11290] },
    Anchor { vertices: [2891, 2886, 2885], weights: [0.28460, 0.31351, 0.40189] },
    Anchor { vertices: [2885, 2886, 2887], weights: [0.56761, 0.27530, 0.15709] },
    Anchor { vertices: [2885, 2887, 2888], weights: [0.39613, 0.50898, 0.09489] },
    Anchor { vertices: [2888, 2887, 2866], weights: [0.54242, 0.38584, 0.07174] },
    Anchor { vertices: [2888, 2866, 2865], weights: [0.49240, 0.41157, 0.09603] },
    Anchor { vertices: [2888, 2866, 2865], weights: [0.02173, 0.30891, 0.66936] },
    Anchor { vertices: [2878, 2865, 2868], weights: [0.06332, 0.39876, 0.53792] },
    Anchor { vertices: [2814, 2879, 2868], weights: [0.06560, 0.12457, 0.80983] },
    Anchor { vertices: [2815, 2868, 2867], weights: [0.59861, 0.24020, 0.16119] },
    Anchor { vertices: [2816, 2815, 2817], weights: [0.25279, 0.24602, 0.50119] },
    Anchor { vertices: [2818, 2847, 2816], weights: [0.84004, 0.11949, 0.04047] },
    Anchor { vertices: [2849, 4086, 2848], weights: [0.24448, 0.58946, 0.16606] },
    Anchor { vertices: [2854, 2830, 2829], weights: [0.08624, 0.05180, 0.86197] },
];
pub const LOWER: [Anchor; 43] = [
    Anchor { vertices: [443, 1750, 1751], weights: [0.39515, 0.52482, 0.08004] },
    Anchor { vertices: [1717, 450, 447], weights: [0.46482, 0.19245, 0.34273] },
    Anchor { vertices: [454, 455, 456], weights: [0.34498, 0.23304, 0.42198] },
    Anchor { vertices: [392, 524, 457], weights: [0.17704, 0.17675, 0.64621] },
    Anchor { vertices: [392, 395, 524], weights: [0.35085, 0.26123, 0.38792] },
    Anchor { vertices: [528, 524, 395], weights: [0.02795, 0.27731, 0.69473] },
    Anchor { vertices: [528, 524, 395], weights: [0.33907, 0.21418, 0.44675] },
    Anchor { vertices: [528, 529, 524], weights: [0.60949, 0.00452, 0.38599] },
    Anchor { vertices: [528, 529, 524], weights: [0.69655, 0.20657, 0.09688] },
    Anchor { vertices: [528, 536, 529], weights: [0.54553, 0.13149, 0.32298] },
    Anchor { vertices: [528, 536, 529], weights: [0.43305, 0.34192, 0.22503] },
    Anchor { vertices: [528, 536, 529], weights: [0.44239, 0.55594, 0.00167] },
    Anchor { vertices: [528, 541, 536], weights: [0.25463, 0.14763, 0.59774] },
    Anchor { vertices: [528, 541, 536], weights: [0.08821, 0.01751, 0.89429] },
    Anchor { vertices: [731, 536, 541], weights: [0.08612, 0.88223, 0.03165] },
    Anchor { vertices: [731, 728, 536], weights: [0.15281, 0.10672, 0.74047] },
    Anchor { vertices: [731, 536, 541], weights: [0.41973, 0.57068, 0.00959] },
    Anchor { vertices: [731, 728, 536], weights: [0.47899, 0.08587, 0.43514] },
    Anchor { vertices: [731, 728, 536], weights: [0.44332, 0.25054, 0.30614] },
    Anchor { vertices: [731, 728, 536], weights: [0.40666, 0.39618, 0.19716] },
    Anchor { vertices: [731, 728, 536], weights: [0.40820, 0.47860, 0.11320] },
    Anchor { vertices: [731, 2916, 728], weights: [0.42142, 0.02761, 0.55097] },
    Anchor { vertices: [731, 2916, 728], weights: [0.41934, 0.16704, 0.41362] },
    Anchor { vertices: [731, 2916, 728], weights: [0.42532, 0.25073, 0.32395] },
    Anchor { vertices: [731, 2916, 728], weights: [0.47174, 0.35954, 0.16872] },
    Anchor { vertices: [731, 2921, 2916], weights: [0.51150, 0.00727, 0.48123] },
    Anchor { vertices: [731, 2921, 2916], weights: [0.36596, 0.10850, 0.52554] },
    Anchor { vertices: [731, 2921, 2916], weights: [0.20830, 0.01158, 0.78013] },
    Anchor { vertices: [731, 2921, 2916], weights: [0.03412, 0.15631, 0.80957] },
    Anchor { vertices: [2908, 2916, 2921], weights: [0.15207, 0.80343, 0.04450] },
    Anchor { vertices: [2908, 2916, 2921], weights: [0.32044, 0.51021, 0.16935] },
    Anchor { vertices: [2908, 2916, 2921], weights: [0.50956, 0.46692, 0.02352] },
    Anchor { vertices: [2908, 2909, 2916], weights: [0.52079, 0.20319, 0.27602] },
    Anchor { vertices: [2908, 2909, 2916], weights: [0.63317, 0.30123, 0.06560] },
    Anchor { vertices: [2908, 2904, 2909], weights: [0.77526, 0.18326, 0.04148] },
    Anchor { vertices: [2908, 2781, 2904], weights: [0.52709, 0.20809, 0.26482] },
    Anchor { vertices: [2908, 2781, 2904], weights: [0.25186, 0.68946, 0.05868] },
    Anchor { vertices: [2780, 2904, 2781], weights: [0.06809, 0.08360, 0.84831] },
    Anchor { vertices: [2780, 2904, 2781], weights: [0.45929, 0.16905, 0.37166] },
    Anchor { vertices: [2780, 2840, 2904], weights: [0.51966, 0.40430, 0.07604] },
    Anchor { vertices: [2839, 2840, 2841], weights: [0.08474, 0.16118, 0.75408] },
    Anchor { vertices: [4050, 2784, 4051], weights: [0.67645, 0.09208, 0.23147] },
    Anchor { vertices: [2854, 2830, 2829], weights: [0.08624, 0.05180, 0.86197] },
];
pub const OPENING_UPPER: [Anchor; 43] = [
    Anchor { vertices: [443, 1750, 1751], weights: [0.39515, 0.52482, 0.08004] },
    Anchor { vertices: [464, 1750, 1749], weights: [0.71273, 0.07951, 0.20777] },
    Anchor { vertices: [435, 434, 464], weights: [0.85719, 0.02162, 0.12119] },
    Anchor { vertices: [434, 435, 436], weights: [0.02438, 0.31966, 0.65596] },
    Anchor { vertices: [436, 2031, 486], weights: [0.69030, 0.28395, 0.02576] },
    Anchor { vertices: [436, 2031, 486], weights: [0.11468, 0.86421, 0.02111] },
    Anchor { vertices: [2031, 516, 487], weights: [0.48228, 0.49410, 0.02362] },
    Anchor { vertices: [516, 517, 498], weights: [0.85500, 0.13281, 0.01218] },
    Anchor { vertices: [516, 517, 498], weights: [0.25746, 0.71986, 0.02267] },
    Anchor { vertices: [499, 517, 518], weights: [0.02035, 0.69167, 0.28798] },
    Anchor { vertices: [499, 517, 518], weights: [0.00269, 0.14995, 0.84736] },
    Anchor { vertices: [505, 499, 518], weights: [0.29491, 0.01138, 0.69371] },
    Anchor { vertices: [505, 499, 518], weights: [0.69137, 0.01986, 0.28877] },
    Anchor { vertices: [508, 505, 509], weights: [0.01512, 0.93108, 0.05380] },
    Anchor { vertices: [508, 505, 509], weights: [0.01241, 0.58083, 0.40676] },
    Anchor { vertices: [508, 505, 509], weights: [0.00814, 0.25465, 0.73721] },
    Anchor { vertices: [512, 509, 513], weights: [0.00547, 0.95398, 0.04054] },
    Anchor { vertices: [512, 509, 513], weights: [0.01642, 0.68065, 0.30292] },
    Anchor { vertices: [512, 509, 513], weights: [0.02300, 0.43961, 0.53739] },
    Anchor { vertices: [512, 509, 513], weights: [0.02451, 0.23597, 0.73952] },
    Anchor { vertices: [512, 509, 513], weights: [0.01976, 0.07858, 0.90166] },
    Anchor { vertices: [512, 513, 2894], weights: [0.00214, 0.96988, 0.02798] },
    Anchor { vertices: [512, 513, 2894], weights: [0.01382, 0.85110, 0.13507] },
    Anchor { vertices: [512, 513, 2894], weights: [0.01099, 0.69436, 0.29465] },
    Anchor { vertices: [513, 2901, 2894], weights: [0.49808, 0.01252, 0.48940] },
    Anchor { vertices: [513, 2901, 2894], weights: [0.25292, 0.03423, 0.71285] },
    Anchor { vertices: [2894, 2901, 2900], weights: [0.94127, 0.03899, 0.01974] },
    Anchor { vertices: [2894, 2900, 2890], weights: [0.67186, 0.06232, 0.26583] },
    Anchor { vertices: [2894, 2900, 2890], weights: [0.34380, 0.07515, 0.58104] },
    Anchor { vertices: [2890, 2900, 2899], weights: [0.91073, 0.08067, 0.00860] },
    Anchor { vertices: [2890, 2899, 2882], weights: [0.58276, 0.36585, 0.05139] },
    Anchor { vertices: [2922, 2923, 2912], weights: [0.56973, 0.01025, 0.42003] },
    Anchor { vertices: [2922, 2912, 2913], weights: [0.32362, 0.27169, 0.40469] },
    Anchor { vertices: [2922, 2912, 2913], weights: [0.08820, 0.21078, 0.70102] },
    Anchor { vertices: [2905, 2913, 2912], weights: [0.15956, 0.53051, 0.30993] },
    Anchor { vertices: [4354, 2897, 2930], weights: [0.08636, 0.79710, 0.11655] },
    Anchor { vertices: [4354, 2897, 2930], weights: [0.73641, 0.08006, 0.18353] },
    Anchor { vertices: [2823, 4354, 4355], weights: [0.36762, 0.53302, 0.09936] },
    Anchor { vertices: [2821, 4354, 2823], weights: [0.42066, 0.03369, 0.54565] },
    Anchor { vertices: [2822, 2821, 2823], weights: [0.67544, 0.03252, 0.29204] },
    Anchor { vertices: [2824, 2851, 2822], weights: [0.23463, 0.68727, 0.07810] },
    Anchor { vertices: [2831, 4085, 4084], weights: [0.38502, 0.25242, 0.36256] },
    Anchor { vertices: [2854, 2830, 2829], weights: [0.08624, 0.05180, 0.86197] },
];
pub const OPENING_LOWER: [Anchor; 43] = [
    Anchor { vertices: [443, 1750, 1751], weights: [0.39515, 0.52482, 0.08004] },
    Anchor { vertices: [1717, 450, 447], weights: [0.04389, 0.69418, 0.26192] },
    Anchor { vertices: [473, 474, 1718], weights: [0.04976, 0.69191, 0.25833] },
    Anchor { vertices: [475, 474, 473], weights: [0.56334, 0.40353, 0.03313] },
    Anchor { vertices: [526, 475, 472], weights: [0.16025, 0.82469, 0.01506] },
    Anchor { vertices: [526, 475, 472], weights: [0.57771, 0.41039, 0.01191] },
    Anchor { vertices: [526, 475, 472], weights: [0.98245, 0.00685, 0.01069] },
    Anchor { vertices: [531, 532, 526], weights: [0.00386, 0.26365, 0.73249] },
    Anchor { vertices: [531, 532, 526], weights: [0.00233, 0.53579, 0.46188] },
    Anchor { vertices: [531, 532, 526], weights: [0.00086, 0.79766, 0.20148] },
    Anchor { vertices: [542, 533, 532], weights: [0.00269, 0.14061, 0.85670] },
    Anchor { vertices: [542, 533, 532], weights: [0.01546, 0.83730, 0.14725] },
    Anchor { vertices: [542, 533, 532], weights: [0.29872, 0.64774, 0.05354] },
    Anchor { vertices: [542, 533, 532], weights: [0.59845, 0.36849, 0.03307] },
    Anchor { vertices: [542, 533, 532], weights: [0.89118, 0.07098, 0.03784] },
    Anchor { vertices: [542, 706, 701], weights: [0.88211, 0.01751, 0.10038] },
    Anchor { vertices: [542, 706, 701], weights: [0.67775, 0.00848, 0.31377] },
    Anchor { vertices: [542, 706, 701], weights: [0.49330, 0.02226, 0.48444] },
    Anchor { vertices: [542, 706, 701], weights: [0.33047, 0.02179, 0.64774] },
    Anchor { vertices: [542, 706, 701], weights: [0.19268, 0.00482, 0.80250] },
    Anchor { vertices: [542, 706, 701], weights: [0.08647, 0.00296, 0.91057] },
    Anchor { vertices: [2922, 701, 706], weights: [0.03465, 0.96125, 0.00410] },
    Anchor { vertices: [2922, 701, 706], weights: [0.15626, 0.83221, 0.01154] },
    Anchor { vertices: [2922, 701, 706], weights: [0.26369, 0.73255, 0.00377] },
    Anchor { vertices: [2926, 701, 2922], weights: [0.02947, 0.59731, 0.37322] },
    Anchor { vertices: [2926, 701, 2922], weights: [0.07653, 0.43258, 0.49089] },
    Anchor { vertices: [2926, 701, 2922], weights: [0.14385, 0.24570, 0.61045] },
    Anchor { vertices: [2926, 701, 2922], weights: [0.19465, 0.03912, 0.76623] },
    Anchor { vertices: [2925, 2922, 2913], weights: [0.04489, 0.80240, 0.15271] },
    Anchor { vertices: [2925, 2922, 2913], weights: [0.12102, 0.49619, 0.38280] },
    Anchor { vertices: [2925, 2922, 2913], weights: [0.17759, 0.18198, 0.64043] },
    Anchor { vertices: [2905, 2913, 2912], weights: [0.13421, 0.64183, 0.22396] },
    Anchor { vertices: [2897, 2898, 2928], weights: [0.33287, 0.49446, 0.17267] },
    Anchor { vertices: [2905, 2912, 2906], weights: [0.12271, 0.68640, 0.19088] },
    Anchor { vertices: [2905, 2912, 2906], weights: [0.11122, 0.42438, 0.46441] },
    Anchor { vertices: [2905, 2912, 2906], weights: [0.09927, 0.15207, 0.74866] },
    Anchor { vertices: [2823, 4354, 4355], weights: [0.46788, 0.10492, 0.42719] },
    Anchor { vertices: [2824, 2823, 2825], weights: [0.08058, 0.34711, 0.57231] },
    Anchor { vertices: [2824, 2825, 2826], weights: [0.47770, 0.18583, 0.33646] },
    Anchor { vertices: [2844, 2859, 2860], weights: [0.22125, 0.15373, 0.62501] },
    Anchor { vertices: [2833, 4083, 4082], weights: [0.91327, 0.02946, 0.05727] },
    Anchor { vertices: [4050, 2784, 4051], weights: [0.68078, 0.00000, 0.31922] },
    Anchor { vertices: [2854, 2830, 2829], weights: [0.08624, 0.05180, 0.86197] },
];

/// The mouth's frame on one head.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouthFrame {
    /// The middle of the opening, in millimetres.
    pub origin: [f64; 3],
    pub across: [f64; 3],
    pub up: [f64; 3],
    pub forward: [f64; 3],
}

impl MouthFrame {
    /// A point's (u, v, w) in millimetres.
    pub fn uvw(&self, p: [f64; 3]) -> [f64; 3] {
        let r = sub(p, self.origin);
        let x = dot(r, self.across);
        let z = dot(r, self.forward) + RADIUS_MM;
        [RADIUS_MM * x.atan2(z), dot(r, self.up), (x * x + z * z).sqrt() - RADIUS_MM]
    }
}

/// What the lip layer keeps of one head: its mouth frame, the four curves in (u, v) and the front map.
#[derive(Clone, Debug, PartialEq)]
pub struct LipHead {
    pub frame: MouthFrame,
    pub upper: Vec<[f64; 2]>,
    pub lower: Vec<[f64; 2]>,
    /// The middle of the opening: the mean of its upper and lower edges.
    pub opening: Vec<[f64; 2]>,
    /// The frontmost `w` in each cell, row by row from the lowest `v`, or NaN where no surface is.
    pub front: Vec<f32>,
    pub front_dims: [usize; 2],
}

fn curve(anchors: &[Anchor], positions: &[[f64; 3]]) -> Vec<[f64; 3]> {
    anchors.iter().map(|a| Anchors::point(a, positions)).collect()
}

impl LipHead {
    /// The lips of a head, from its mesh positions in metres, and the layout's triangles and the mesh vertex of each layout vertex for
    /// the front map. It is `None` for a mesh that does not hold the anchors' vertices.
    pub fn new(positions: &[[f64; 3]], layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize]) -> Option<Self> {
        let all = [&UPPER[..], &LOWER[..], &OPENING_UPPER[..], &OPENING_LOWER[..]];
        if all.iter().any(|c| c.iter().any(|a| a.vertices.iter().any(|&v| v as usize >= positions.len()))) {
            return None;
        }
        let up3 = curve(&UPPER, positions);
        let lo3 = curve(&LOWER, positions);
        let ou3 = curve(&OPENING_UPPER, positions);
        let ol3 = curve(&OPENING_LOWER, positions);
        let n = up3.len();
        let mid = n / 2;
        let origin = [0, 1, 2].map(|k| 0.5 * (ou3[mid][k] + ol3[mid][k]));
        let across = normalize(sub(up3[n - 1], up3[0]));
        let up_guess = sub(up3[mid], lo3[mid]);
        let forward = normalize(cross(across, normalize(up_guess)));
        let up = normalize(cross(forward, across));
        let frame = MouthFrame { origin, across, up, forward };
        let flat = |c: &[[f64; 3]]| -> Vec<[f64; 2]> { c.iter().map(|&p| { let q = frame.uvw(p); [q[0], q[1]] }).collect() };
        let opening: Vec<[f64; 2]> = flat(&ou3).iter().zip(flat(&ol3)).map(|(a, b)| [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])]).collect();
        let (front, front_dims) = front_map(&frame, positions, layout_triangles, layout_to_mesh);
        Some(LipHead { frame, upper: flat(&up3), lower: flat(&lo3), opening, front, front_dims })
    }
}

fn front_map(frame: &MouthFrame, positions: &[[f64; 3]], triangles: &[[u32; 3]], layout_to_mesh: &[usize]) -> (Vec<f32>, [usize; 2]) {
    let nu = (2.0 * FRONT_U_MM / FRONT_CELL_MM) as usize + 1;
    let nv = ((FRONT_V_MM[1] - FRONT_V_MM[0]) / FRONT_CELL_MM) as usize + 1;
    let mut map = vec![f32::NAN; nu * nv];
    let pts: Vec<[f64; 3]> = layout_to_mesh.iter().map(|&m| frame.uvw(positions[m].map(|x| 1000.0 * x))).collect();
    let to_cell = |q: [f64; 3]| [(q[0] + FRONT_U_MM) / FRONT_CELL_MM, (q[1] - FRONT_V_MM[0]) / FRONT_CELL_MM];
    // Only the face's own patch of the layout counts: the patch with the most triangles. The lips' inner surfaces and the inside of the
    // mouth lie on other patches, so the slit between the lips has no front, and it is filled below from the lips above and under it.
    let mut parent: Vec<usize> = (0..pts.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for t in triangles {
        let a = root(&mut parent, t[0] as usize);
        for &v in &t[1..] {
            let b = root(&mut parent, v as usize);
            parent[b] = a;
        }
    }
    let mut count = std::collections::HashMap::new();
    for t in triangles {
        *count.entry(root(&mut parent, t[0] as usize)).or_insert(0usize) += 1;
    }
    let face = count.iter().max_by_key(|(_, n)| **n).map(|(r, _)| *r).unwrap_or(0);
    for t in triangles.iter().filter(|t| root(&mut parent, t[0] as usize) == face) {
        let q = t.map(|i| pts[i as usize]);
        let c = q.map(to_cell);
        if c.iter().all(|p| p[0] < -2.0 || p[0] > nu as f64 + 1.0 || p[1] < -2.0 || p[1] > nv as f64 + 1.0) {
            continue;
        }
        // A triangle that spans more than 20 mm across is one that wraps round the back of the cylinder.
        if (q[0][0] - q[1][0]).abs().max((q[1][0] - q[2][0]).abs()).max((q[0][0] - q[2][0]).abs()) > 20.0 {
            continue;
        }
        let den = (c[1][1] - c[2][1]) * (c[0][0] - c[2][0]) + (c[2][0] - c[1][0]) * (c[0][1] - c[2][1]);
        if den.abs() < 1e-12 {
            continue;
        }
        let lo = [0, 1].map(|k| c[0][k].min(c[1][k]).min(c[2][k]).floor().max(0.0) as usize);
        let hi = [0, 1].map(|k| (c[0][k].max(c[1][k]).max(c[2][k]).ceil().max(0.0) as usize).min(if k == 0 { nu - 1 } else { nv - 1 }));
        for y in lo[1]..=hi[1] {
            for x in lo[0]..=hi[0] {
                let p = [x as f64, y as f64];
                let w0 = ((c[1][1] - c[2][1]) * (p[0] - c[2][0]) + (c[2][0] - c[1][0]) * (p[1] - c[2][1])) / den;
                let w1 = ((c[2][1] - c[0][1]) * (p[0] - c[2][0]) + (c[0][0] - c[2][0]) * (p[1] - c[2][1])) / den;
                let w2 = 1.0 - w0 - w1;
                if w0 < -1e-6 || w1 < -1e-6 || w2 < -1e-6 {
                    continue;
                }
                let w = (w0 * q[0][2] + w1 * q[1][2] + w2 * q[2][2]) as f32;
                let cell = &mut map[y * nu + x];
                if cell.is_nan() || w > *cell {
                    *cell = w;
                }
            }
        }
    }
    // The slit: each column's gaps of up to 10 mm are filled by a straight line between the cells above and below.
    let reach = (10.0 / FRONT_CELL_MM) as usize;
    for x in 0..nu {
        let mut y = 0;
        while y < nv {
            if !map[y * nu + x].is_nan() {
                y += 1;
                continue;
            }
            let start = y;
            while y < nv && map[y * nu + x].is_nan() {
                y += 1;
            }
            if start > 0 && y < nv && y - start <= reach {
                let (a, b) = (map[(start - 1) * nu + x], map[y * nu + x]);
                for k in start..y {
                    let t = (k - start + 1) as f32 / (y - start + 1) as f32;
                    map[k * nu + x] = a + t * (b - a);
                }
            }
        }
    }
    (map, [nu, nv])
}

/// The lips' curves on one head after the sliders' changes of shape, sampled every `STEP_MM` along `u` from `u0`.
#[derive(Clone, Debug, PartialEq)]
pub struct LipShape {
    pub u0: f64,
    pub upper: Vec<f64>,
    pub lower: Vec<f64>,
    pub opening: Vec<f64>,
    /// The (u, v) of the two corners, the x < 0 one first.
    pub corners: [[f64; 2]; 2],
}

/// `v` on a polyline at `u`, by a cubic Hermite curve with slopes that keep it monotone between points, and straight on past its ends.
fn hermite(pts: &[[f64; 2]], u: f64) -> f64 {
    let n = pts.len();
    let slope = |i: usize| -> f64 {
        if i == 0 {
            return (pts[1][1] - pts[0][1]) / (pts[1][0] - pts[0][0]);
        }
        if i == n - 1 {
            return (pts[n - 1][1] - pts[n - 2][1]) / (pts[n - 1][0] - pts[n - 2][0]);
        }
        let a = (pts[i][1] - pts[i - 1][1]) / (pts[i][0] - pts[i - 1][0]);
        let b = (pts[i + 1][1] - pts[i][1]) / (pts[i + 1][0] - pts[i][0]);
        if a * b <= 0.0 { 0.0 } else { 2.0 / (1.0 / a + 1.0 / b) }
    };
    if u <= pts[0][0] {
        return pts[0][1] + slope(0) * (u - pts[0][0]);
    }
    if u >= pts[n - 1][0] {
        return pts[n - 1][1] + slope(n - 1) * (u - pts[n - 1][0]);
    }
    let i = pts.windows(2).position(|w| u >= w[0][0] && u <= w[1][0]).unwrap_or(n - 2);
    let h = pts[i + 1][0] - pts[i][0];
    if h <= 1e-9 {
        return pts[i][1];
    }
    let t = (u - pts[i][0]) / h;
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * pts[i][1] + (t3 - 2.0 * t2 + t) * h * slope(i) + (-2.0 * t3 + 3.0 * t2) * pts[i + 1][1] + (t3 - t2) * h * slope(i + 1)
}

/// Makes the points' `u` strictly rising, in case a head folds the curve back on itself near a corner.
fn rising(mut pts: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    for i in 1..pts.len() {
        if pts[i][0] <= pts[i - 1][0] + 0.01 {
            pts[i][0] = pts[i - 1][0] + 0.01;
        }
    }
    pts
}

impl LipShape {
    pub fn new(lips: &Lips, head: &LipHead) -> Self {
        let n = head.upper.len();
        let corners = [head.upper[0], head.upper[n - 1]];
        // Each side is stretched about the middle so that its corner moves out by `width_mm`.
        let half = [-corners[0][0], corners[1][0]].map(|h| h.max(5.0));
        let stretch = |p: [f64; 2]| -> [f64; 2] {
            let side = usize::from(p[0] > 0.0);
            [p[0] * (half[side] + lips.width_mm) / half[side], p[1]]
        };
        // The share of the half width, 0 at the middle and 1 at the corner, of an original u.
        let share = |u: f64| (u / if u < 0.0 { half[0] } else { half[1] }).abs().min(1.0);
        let taper = |s: f64| (1.0 - s.powf(2.5)).max(0.0);
        let bow = |s: f64| 0.8 * (-((s - 0.22) / 0.2).powi(2)).exp() - (-(s / 0.14).powi(2)).exp();
        // The lopsidedness changes smoothly across the middle, over 8 mm on each side, so the middle has no step.
        let lop = |u: f64| { let t = ((u + 8.0) / 16.0).clamp(0.0, 1.0); lips.lopsided_mm * (2.0 * t * t * (3.0 - 2.0 * t) - 1.0) };
        let upper: Vec<[f64; 2]> = head.upper.iter().map(|&p| { let s = share(p[0]); stretch([p[0], p[1] + (lips.upper_mm + lop(p[0])) * taper(s) + lips.bow_mm * bow(s)]) }).collect();
        let lower: Vec<[f64; 2]> = head.lower.iter().map(|&p| { let s = share(p[0]); stretch([p[0], p[1] - (lips.lower_mm + lop(p[0])) * taper(s)]) }).collect();
        let opening: Vec<[f64; 2]> = head.opening.iter().map(|&p| stretch(p)).collect();
        let (upper, lower, opening) = (rising(upper), rising(lower), rising(opening));
        let new_corners = [upper[0], upper[n - 1]];
        let u0 = new_corners[0][0] - PAST_CORNER_MM;
        let count = ((new_corners[1][0] + PAST_CORNER_MM - u0) / STEP_MM).ceil() as usize + 1;
        // Sampled, then smoothed along u with a Gaussian of 1.5 mm, so that the anchors' small wobbles do not show as bumps.
        let sample = |pts: &[[f64; 2]]| -> Vec<f64> {
            let raw: Vec<f64> = (0..count).map(|i| hermite(pts, u0 + i as f64 * STEP_MM)).collect();
            let sigma = 1.5 / STEP_MM;
            let reach = (3.0 * sigma) as i64;
            let kernel: Vec<f64> = (-reach..=reach).map(|k| (-(k as f64 / sigma).powi(2) / 2.0).exp()).collect();
            (0..count as i64)
                .map(|i| {
                    let (mut s, mut w) = (0.0, 0.0);
                    for (j, kv) in (-reach..=reach).zip(&kernel) {
                        let at = i + j;
                        // Past the ends the curve runs straight on, so it is extended by reflection through its end point.
                        let v = if at < 0 { 2.0 * raw[0] - raw[(-at) as usize] } else if at >= count as i64 { 2.0 * raw[count - 1] - raw[(2 * (count as i64 - 1) - at) as usize] } else { raw[at as usize] };
                        s += kv * v;
                        w += kv;
                    }
                    s / w
                })
                .collect()
        };
        LipShape { u0, upper: sample(&upper), lower: sample(&lower), opening: sample(&opening), corners: new_corners }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sliders_have_their_defaults_in_order() {
        let mut d = Lips::DEFAULT;
        for (k, s) in SLIDERS.iter().enumerate() {
            assert_eq!(*d.field(k), s.default, "{}", s.id);
            assert!(s.min <= s.default && s.default <= s.max, "{}", s.id);
        }
        assert_eq!(Lips::DEFAULT.values().len(), SLIDERS.len());
    }

    #[test]
    fn hermite_passes_through_its_points_and_runs_straight_past_its_ends() {
        let pts = [[-2.0, 0.0], [0.0, 1.0], [2.0, 0.0]];
        assert!((hermite(&pts, 0.0) - 1.0).abs() < 1e-12);
        assert!((hermite(&pts, -2.0)).abs() < 1e-12);
        assert!((hermite(&pts, 3.0) - (-0.5)).abs() < 1e-12);
    }

    #[test]
    fn more_blood_makes_the_lips_redder_and_lipstick_has_its_hue() {
        let model = SpectralModel::new();
        let tone = BaseTone::from_pigments(0.05, 0.03);
        let s = Lips::DEFAULT.natural_shift(&tone, &model);
        assert!(s[1] > 2.0, "{s:?}");
        let none = Lips { blood: 0.0, darkness: 0.0, tint_degrees: 0.0, ..Lips::DEFAULT }.natural_shift(&tone, &model);
        assert!(none.iter().all(|v| v.abs() < 1e-9));
        let red = Lips { stick_hue: 0.0, ..Lips::DEFAULT }.stick_lab();
        assert!(red[1] > 39.0 && red[2].abs() < 1e-9);
    }
}
