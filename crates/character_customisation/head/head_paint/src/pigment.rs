//! The base skin tone, built from two pigments and one unnatural-colour control (paint track, step C2).
//!
//! The colour of skin comes from a spectral model of two layers, as in Alotaibi and Smith (2017, "A Biophysical 3D Morphable Model of Face
//! Appearance", ICCV workshops). The epidermis absorbs light with melanin (Lambert-Beer law). The dermis scatters light and absorbs it
//! with blood (Kubelka-Munk theory). The light passes the epidermis twice. The spectrum is turned into linear sRGB under the D65
//! illuminant. As in Jimenez et al. (2010, "A Practical Appearance Model for Dynamic Facial Color"), the model is baked into a 2D table
//! indexed by the cube roots of the two pigment amounts, and a recipe reads its colour from the table.
//!
//! One change was made to the paper's melanin term. The paper multiplies the melanin extinction by the epidermis thickness and by the
//! melanin concentration. That gives an absorption of about 800 per centimetre at 550 nm for a 2% melanosome fraction, so skin would be
//! black. The extinction formulas come from Jacques, who gives them as absorption coefficients in 1 per centimetre. This code uses them as
//! absorption coefficients and multiplies only by the epidermis thickness. The concentrations in the paper's Table 1 are therefore unused.

use crate::colour::{lab_to_srgb, srgb_to_lab};

const SPECTRA: &str = include_str!("data/skin_spectra.csv");
const EPIDERMIS_CM: f64 = 0.021;
const DERMIS_CM: f64 = 0.2;
const EUMELANIN_SHARE: f64 = 0.61;
const OXYGENATED_SHARE: f64 = 0.75;
const HAEMOGLOBIN_G_PER_L: f64 = 150.0;
const HAEMOGLOBIN_G_PER_MOL: f64 = 64_500.0;
const SAMPLES: usize = 33;

/// The most melanosome volume fraction the table holds. Alotaibi and Smith give 0.01 to 0.43 for real skin.
pub const MELANIN_MAX: f64 = 0.5;
/// The most blood volume fraction the table holds. Alotaibi and Smith give 0.02 to 0.07 for real skin.
pub const HAEMOGLOBIN_MAX: f64 = 0.10;
/// The table has this many entries along each axis.
pub const GRID: usize = 64;

struct Spectra {
    oxy: [f64; SAMPLES],
    deoxy: [f64; SAMPLES],
    illuminant: [f64; SAMPLES],
}

fn spectra() -> Spectra {
    let mut s = Spectra {
        oxy: [0.0; SAMPLES],
        deoxy: [0.0; SAMPLES],
        illuminant: [0.0; SAMPLES],
    };
    let rows = SPECTRA
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("wavelength"));
    for (i, line) in rows.enumerate() {
        let cells: Vec<f64> = line
            .split(',')
            .map(|c| c.parse().expect("number in skin_spectra.csv"))
            .collect();
        s.oxy[i] = cells[1];
        s.deoxy[i] = cells[2];
        s.illuminant[i] = cells[3];
    }
    s
}

fn wavelength(i: usize) -> f64 {
    400.0 + 10.0 * i as f64
}

fn lobe(l: f64, mu: f64, low: f64, high: f64) -> f64 {
    let t = (l - mu) / if l < mu { low } else { high };
    (-0.5 * t * t).exp()
}

/// The CIE 1931 colour matching functions, from the multi-lobe Gaussian fit of Wyman, Sloan and Shirley (2013, JCGT 2(2)).
fn matching(l: f64) -> [f64; 3] {
    [
        1.056 * lobe(l, 599.8, 37.9, 31.0) + 0.362 * lobe(l, 442.0, 16.0, 26.7)
            - 0.065 * lobe(l, 501.1, 20.4, 26.2),
        0.821 * lobe(l, 568.8, 46.9, 40.5) + 0.286 * lobe(l, 530.9, 16.3, 31.1),
        1.217 * lobe(l, 437.0, 11.8, 36.0) + 0.681 * lobe(l, 459.0, 26.0, 13.8),
    ]
}

const FROM_XYZ: [[f64; 3]; 3] = [
    [3.2406, -1.5372, -0.4986],
    [-0.9689, 1.8758, 0.0415],
    [0.0557, -0.2040, 1.0570],
];

/// The spectral skin model for one pair of pigment amounts, kept with the constants it needs.
pub struct SpectralModel {
    spectra: Spectra,
    white: [f64; 3],
}

impl Default for SpectralModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralModel {
    pub fn new() -> Self {
        let spectra = spectra();
        let mut model = SpectralModel {
            spectra,
            white: [1.0; 3],
        };
        model.white = model.integrate(&[1.0; SAMPLES]);
        model
    }

    fn integrate(&self, reflectance: &[f64; SAMPLES]) -> [f64; 3] {
        let mut xyz = [0.0; 3];
        for (i, r) in reflectance.iter().enumerate() {
            let m = matching(wavelength(i));
            for (sum, weight) in xyz.iter_mut().zip(m) {
                *sum += self.spectra.illuminant[i] * weight * r;
            }
        }
        xyz
    }

    /// The fraction of light that skin sends back at each of the 33 wavelengths from 400 to 720 nm.
    pub fn reflectance(&self, melanin: f64, haemoglobin: f64) -> [f64; SAMPLES] {
        let mut out = [0.0; SAMPLES];
        for (i, r) in out.iter_mut().enumerate() {
            let l = wavelength(i);
            let baseline = 0.244 + 85.3 * (-(l - 154.0) / 66.2).exp();
            let eumelanin = 6.6e11 * l.powf(-3.33);
            let pheomelanin = 2.9e10 * l.powf(-4.75);
            let mu_melanin = EUMELANIN_SHARE * eumelanin + (1.0 - EUMELANIN_SHARE) * pheomelanin;
            let epidermis = melanin * mu_melanin + (1.0 - melanin) * baseline;
            let transmitted = (-epidermis * EPIDERMIS_CM).exp();
            let to_mu = 2.303 * HAEMOGLOBIN_G_PER_L / HAEMOGLOBIN_G_PER_MOL;
            let blood = (OXYGENATED_SHARE * self.spectra.oxy[i]
                + (1.0 - OXYGENATED_SHARE) * self.spectra.deoxy[i])
                * to_mu;
            let k = haemoglobin * blood + (1.0 - haemoglobin) * baseline;
            let s = 2e5 * l.powf(-1.5) + 2e12 * l.powf(-4.0);
            let big_k = (k * (k + 2.0 * s)).sqrt();
            let beta = (k / (k + 2.0 * s)).sqrt();
            let (up, down) = ((big_k * DERMIS_CM).exp(), (-big_k * DERMIS_CM).exp());
            let dermis = (1.0 - beta * beta) * (up - down)
                / ((1.0 + beta).powi(2) * up - (1.0 - beta).powi(2) * down);
            *r = transmitted * transmitted * dermis;
        }
        out
    }

    /// The linear sRGB colour of skin with the given melanosome and blood volume fractions. A perfect reflector gives (1, 1, 1).
    pub fn colour(&self, melanin: f64, haemoglobin: f64) -> [f64; 3] {
        let xyz = self.integrate(&self.reflectance(melanin, haemoglobin));
        let scaled = [
            xyz[0] / self.white[0] * 0.95047,
            xyz[1] / self.white[1],
            xyz[2] / self.white[2] * 1.08883,
        ];
        FROM_XYZ.map(|row| row[0] * scaled[0] + row[1] * scaled[1] + row[2] * scaled[2])
    }
}

/// The skin colour for every pair of pigment amounts, in linear sRGB. Entry (i, j) holds melanin `(i / 63 * MELANIN_MAX^(1/3))^3` and
/// blood `(j / 63 * HAEMOGLOBIN_MAX^(1/3))^3`, so the grid is even in the cube roots of the amounts, as in Jimenez et al.
pub struct PigmentTable {
    entries: Vec<[f64; 3]>,
}

fn axis_amount(index: f64, max: f64) -> f64 {
    (index / (GRID - 1) as f64 * max.cbrt()).powi(3)
}

fn axis_index(amount: f64, max: f64) -> f64 {
    (amount.clamp(0.0, max).cbrt() / max.cbrt()) * (GRID - 1) as f64
}

impl Default for PigmentTable {
    fn default() -> Self {
        Self::build()
    }
}

impl PigmentTable {
    pub fn build() -> Self {
        let model = SpectralModel::new();
        let mut entries = Vec::with_capacity(GRID * GRID);
        for i in 0..GRID {
            for j in 0..GRID {
                entries.push(model.colour(
                    axis_amount(i as f64, MELANIN_MAX),
                    axis_amount(j as f64, HAEMOGLOBIN_MAX),
                ));
            }
        }
        PigmentTable { entries }
    }

    /// The linear sRGB colour for the given amounts, filtered bilinearly between the four nearest entries. Amounts outside the table are clamped.
    pub fn lookup(&self, melanin: f64, haemoglobin: f64) -> [f64; 3] {
        let (u, v) = (
            axis_index(melanin, MELANIN_MAX),
            axis_index(haemoglobin, HAEMOGLOBIN_MAX),
        );
        let (i, j) = (
            (u.floor() as usize).min(GRID - 2),
            (v.floor() as usize).min(GRID - 2),
        );
        let (fu, fv) = (u - i as f64, v - j as f64);
        let at = |a: usize, b: usize| self.entries[a * GRID + b];
        let mut out = [0.0; 3];
        for (k, o) in out.iter_mut().enumerate() {
            *o = at(i, j)[k] * (1.0 - fu) * (1.0 - fv)
                + at(i + 1, j)[k] * fu * (1.0 - fv)
                + at(i, j + 1)[k] * (1.0 - fu) * fv
                + at(i + 1, j + 1)[k] * fu * fv;
        }
        out
    }
}

/// The settings of one face's base tone.
///
/// `melanin` and `haemoglobin` are the volume fractions of the model. The other three controls act on the pigment colour in CIE Lab.
/// `chroma_scale` multiplies the chroma and keeps the lightness and hue. At 1 the chroma is the model's, and a smaller value greys the skin.
/// `undertone_degrees` turns the hue around the grey axis by a small angle. A negative angle turns skin toward red and a positive one toward
/// yellow. `unnatural_degrees` turns the hue the same way, for the whole wheel. Skin sits near 60 degrees of hue, so +80 gives a grey-green,
/// +210 a grey-blue and -60 a purple. The two angles add up, and they are separate controls so that a player can offer a small range for one
/// and a wide range for the other. Zero for both, with a chroma scale of 1, leaves the pigment colour alone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaseTone {
    pub melanin: f64,
    pub haemoglobin: f64,
    pub chroma_scale: f64,
    pub undertone_degrees: f64,
    pub unnatural_degrees: f64,
}

/// Rotates a Lab colour's hue by an angle in degrees, keeping its lightness and chroma.
pub fn rotate_hue(lab: [f64; 3], degrees: f64) -> [f64; 3] {
    let (s, c) = degrees.to_radians().sin_cos();
    [lab[0], lab[1] * c - lab[2] * s, lab[1] * s + lab[2] * c]
}

/// Multiplies a Lab colour's chroma by a factor, keeping its lightness and hue.
pub fn scale_chroma(lab: [f64; 3], factor: f64) -> [f64; 3] {
    [lab[0], lab[1] * factor, lab[2] * factor]
}

/// Encodes a linear sRGB channel with the sRGB curve. A negative value keeps its sign.
pub fn encode(c: f64) -> f64 {
    let m = c.abs();
    let e = if m > 0.0031308 {
        1.055 * m.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * m
    };
    e.copysign(c)
}

/// Decodes an encoded sRGB channel to linear.
pub fn decode(c: f64) -> f64 {
    if c > 0.04045 {
        ((c + 0.055) / 1.055).powf(2.4)
    } else {
        c / 12.92
    }
}

/// The Lab colour of a linear sRGB colour. A colour outside the sRGB gamut keeps its out-of-range value.
pub fn linear_to_lab(linear: [f64; 3]) -> [f64; 3] {
    srgb_to_lab(linear.map(encode))
}

impl BaseTone {
    /// A tone from two pigments, with the three other controls at their neutral values.
    pub fn from_pigments(melanin: f64, haemoglobin: f64) -> Self {
        BaseTone { melanin, haemoglobin, chroma_scale: 1.0, undertone_degrees: 0.0, unnatural_degrees: 0.0 }
    }

    fn is_plain(&self) -> bool {
        self.chroma_scale == 1.0 && self.undertone_degrees == 0.0 && self.unnatural_degrees == 0.0
    }

    fn adjust(&self, lab: [f64; 3]) -> [f64; 3] {
        rotate_hue(scale_chroma(lab, self.chroma_scale), self.undertone_degrees + self.unnatural_degrees)
    }

    /// The tone's colour in Lab, before it is clamped to the sRGB gamut.
    pub fn lab(&self, table: &PigmentTable) -> [f64; 3] {
        let lab = linear_to_lab(table.lookup(self.melanin, self.haemoglobin));
        if self.is_plain() { lab } else { self.adjust(lab) }
    }

    /// The tone's colour in linear sRGB, clamped to the gamut. With the neutral controls this is exactly the table's colour.
    pub fn linear_rgb(&self, table: &PigmentTable) -> [f64; 3] {
        self.adjusted(table.lookup(self.melanin, self.haemoglobin))
    }

    /// A pigment colour in linear sRGB with this tone's colour strength and hue controls applied, as `linear_rgb` applies them to the
    /// table's colour.
    pub fn adjusted(&self, linear: [f64; 3]) -> [f64; 3] {
        if self.is_plain() {
            return linear;
        }
        lab_to_srgb(self.adjust(linear_to_lab(linear))).map(decode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lab_of(model: &SpectralModel, melanin: f64, haemoglobin: f64) -> [f64; 3] {
        linear_to_lab(model.colour(melanin, haemoglobin))
    }

    #[test]
    fn a_perfect_reflector_is_white() {
        let model = SpectralModel::new();
        let xyz = model.integrate(&[1.0; SAMPLES]);
        assert_eq!(xyz, model.white);
        let white = [1.0; 3].map(|_| 1.0);
        let lab = linear_to_lab(white);
        assert!(lab[0] > 99.9 && lab[1].abs() < 0.1 && lab[2].abs() < 0.1);
    }

    #[test]
    fn the_colour_matching_functions_peak_where_the_standard_does() {
        assert!((matching(555.0)[1] - 1.0).abs() < 0.02);
        assert!((matching(600.0)[0] - 1.06).abs() < 0.03);
        assert!((matching(445.0)[2] - 1.78).abs() < 0.1);
    }

    #[test]
    fn more_melanin_makes_the_colour_darker() {
        let model = SpectralModel::new();
        let mut last = f64::MAX;
        for melanin in [0.01, 0.03, 0.06, 0.1, 0.2, 0.4] {
            let l = lab_of(&model, melanin, 0.04)[0];
            assert!(l < last, "melanin {melanin} gave L {l}, after {last}");
            last = l;
        }
    }

    #[test]
    fn more_haemoglobin_makes_the_colour_redder() {
        let model = SpectralModel::new();
        let mut last = f64::MIN;
        for haemoglobin in [0.0, 0.01, 0.02, 0.04, 0.07, 0.1] {
            let a = lab_of(&model, 0.03, haemoglobin)[1];
            assert!(a > last, "blood {haemoglobin} gave a* {a}, after {last}");
            last = a;
        }
    }

    #[test]
    fn typical_skin_has_a_skin_like_colour() {
        let lab = lab_of(&SpectralModel::new(), 0.03, 0.04);
        assert!(lab[0] > 40.0 && lab[0] < 90.0, "{lab:?}");
        assert!(lab[1] > 3.0 && lab[2] > 8.0, "{lab:?}");
        let hue = lab[2].atan2(lab[1]).to_degrees();
        assert!(hue > 40.0 && hue < 85.0, "{lab:?}");
    }

    #[test]
    fn the_table_corners_match_the_spectral_model() {
        let model = SpectralModel::new();
        let table = PigmentTable::build();
        for (m, h) in [
            (0.0, 0.0),
            (MELANIN_MAX, 0.0),
            (0.0, HAEMOGLOBIN_MAX),
            (MELANIN_MAX, HAEMOGLOBIN_MAX),
        ] {
            let a = table.lookup(m, h);
            let b = model.colour(m, h);
            for k in 0..3 {
                assert!(
                    (a[k] - b[k]).abs() < 1e-12,
                    "({m}, {h}) channel {k}: {} against {}",
                    a[k],
                    b[k]
                );
            }
        }
    }

    #[test]
    fn the_table_is_close_to_the_spectral_model_between_entries() {
        let model = SpectralModel::new();
        let table = PigmentTable::build();
        for (m, h) in [(0.02, 0.03), (0.05, 0.05), (0.15, 0.02), (0.3, 0.08)] {
            let d =
                crate::delta_e::delta_e_76(linear_to_lab(table.lookup(m, h)), lab_of(&model, m, h));
            assert!(d < 0.5, "({m}, {h}) differs by {d}");
        }
    }

    fn tone(chroma_scale: f64, undertone_degrees: f64, unnatural_degrees: f64) -> BaseTone {
        BaseTone { melanin: 0.03, haemoglobin: 0.04, chroma_scale, undertone_degrees, unnatural_degrees }
    }

    fn chroma(lab: [f64; 3]) -> f64 {
        lab[1].hypot(lab[2])
    }

    fn hue(lab: [f64; 3]) -> f64 {
        lab[2].atan2(lab[1]).to_degrees()
    }

    #[test]
    fn neutral_controls_leave_the_colour_alone() {
        let table = PigmentTable::build();
        let t = tone(1.0, 0.0, 0.0);
        assert_eq!(t.linear_rgb(&table), table.lookup(0.03, 0.04));
        assert_eq!(t.lab(&table), linear_to_lab(table.lookup(0.03, 0.04)));
        assert_eq!(t, BaseTone::from_pigments(0.03, 0.04));
    }

    #[test]
    fn a_chroma_scale_below_one_lowers_chroma_and_keeps_lightness_and_hue() {
        let table = PigmentTable::build();
        let plain = tone(1.0, 0.0, 0.0).lab(&table);
        let grey = tone(0.4, 0.0, 0.0).lab(&table);
        assert!((plain[0] - grey[0]).abs() < 1e-9);
        assert!((chroma(grey) - 0.4 * chroma(plain)).abs() < 1e-9);
        assert!((hue(grey) - hue(plain)).abs() < 1e-9);
    }

    #[test]
    fn a_chroma_scale_above_one_raises_chroma() {
        let table = PigmentTable::build();
        assert!(chroma(tone(1.3, 0.0, 0.0).lab(&table)) > chroma(tone(1.0, 0.0, 0.0).lab(&table)));
    }

    #[test]
    fn the_undertone_turns_the_hue_and_keeps_lightness_and_chroma() {
        let table = PigmentTable::build();
        let plain = tone(1.0, 0.0, 0.0).lab(&table);
        let redder = tone(1.0, -15.0, 0.0).lab(&table);
        assert!((plain[0] - redder[0]).abs() < 1e-9);
        assert!((chroma(plain) - chroma(redder)).abs() < 1e-9);
        assert!((hue(redder) - hue(plain) + 15.0).abs() < 1e-6);
    }

    #[test]
    fn the_unnatural_colour_turns_the_hue_toward_green() {
        let table = PigmentTable::build();
        let plain = tone(1.0, 0.0, 0.0).lab(&table);
        let green = tone(1.0, 0.0, 80.0).lab(&table);
        assert!((plain[0] - green[0]).abs() < 1e-9);
        assert!((chroma(plain) - chroma(green)).abs() < 1e-9);
        assert!((hue(green) - hue(plain) - 80.0).abs() < 1e-6);
        assert!(green[1] < 0.0, "a hue near 140 degrees is greenish: {green:?}");
    }

    #[test]
    fn the_undertone_and_the_unnatural_colour_add_up() {
        let table = PigmentTable::build();
        let split = tone(1.0, 10.0, 20.0).lab(&table);
        let one = tone(1.0, 30.0, 0.0).lab(&table);
        for k in 0..3 {
            assert!((split[k] - one[k]).abs() < 1e-9);
        }
    }
}
