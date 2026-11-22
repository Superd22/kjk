//! Colour maths: sRGB to and from CIE Lab (D65 white), with the 0 to 1 sRGB values that a picture file holds.

const WHITE: [f64; 3] = [0.95047, 1.0, 1.08883];
const TO_XYZ: [[f64; 3]; 3] = [[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722], [0.0193, 0.1192, 0.9505]];
const FROM_XYZ: [[f64; 3]; 3] = [[3.2406, -1.5372, -0.4986], [-0.9689, 1.8758, 0.0415], [0.0557, -0.2040, 1.0570]];
const EPSILON: f64 = 216.0 / 24389.0;
const KAPPA: f64 = 24389.0 / 27.0;

fn multiply(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    m.map(|row| row[0] * v[0] + row[1] * v[1] + row[2] * v[2])
}

/// Converts an sRGB colour (each channel from 0 to 1) to Lab.
pub fn srgb_to_lab(rgb: [f64; 3]) -> [f64; 3] {
    let linear = rgb.map(|c| if c > 0.04045 { ((c + 0.055) / 1.055).powf(2.4) } else { c / 12.92 });
    let xyz = multiply(&TO_XYZ, linear);
    let f: Vec<f64> = (0..3)
        .map(|i| {
            let t = xyz[i] / WHITE[i];
            if t > EPSILON { t.cbrt() } else { (KAPPA * t + 16.0) / 116.0 }
        })
        .collect();
    [116.0 * f[1] - 16.0, 500.0 * (f[0] - f[1]), 200.0 * (f[1] - f[2])]
}

/// Converts a Lab colour to sRGB, with each channel clamped to 0..1. A Lab colour outside the sRGB gamut is clamped, not mapped.
pub fn lab_to_srgb(lab: [f64; 3]) -> [f64; 3] {
    let fy = (lab[0] + 16.0) / 116.0;
    let f = [fy + lab[1] / 500.0, fy, fy - lab[2] / 200.0];
    let xyz: Vec<f64> = (0..3)
        .map(|i| {
            let cube = f[i] * f[i] * f[i];
            (if cube > EPSILON { cube } else { (116.0 * f[i] - 16.0) / KAPPA }) * WHITE[i]
        })
        .collect();
    multiply(&FROM_XYZ, [xyz[0], xyz[1], xyz[2]]).map(|c| {
        let encoded = if c > 0.0031308 { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 } else { 12.92 * c };
        encoded.clamp(0.0, 1.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_is_l_100_and_black_is_l_0() {
        let white = srgb_to_lab([1.0, 1.0, 1.0]);
        assert!((white[0] - 100.0).abs() < 0.05 && white[1].abs() < 0.1 && white[2].abs() < 0.1, "{white:?}");
        let black = srgb_to_lab([0.0, 0.0, 0.0]);
        assert!(black[0].abs() < 1e-9);
    }

    #[test]
    fn srgb_red_is_the_known_lab_colour() {
        let red = srgb_to_lab([1.0, 0.0, 0.0]);
        assert!((red[0] - 53.24).abs() < 0.1 && (red[1] - 80.09).abs() < 0.3 && (red[2] - 67.2).abs() < 0.3, "{red:?}");
    }

    #[test]
    fn colours_survive_a_round_trip() {
        for r in [0.0, 0.02, 0.2, 0.5, 0.93, 1.0] {
            for g in [0.0, 0.01, 0.4, 0.77, 1.0] {
                for b in [0.0, 0.03, 0.3, 0.6, 1.0] {
                    let back = lab_to_srgb(srgb_to_lab([r, g, b]));
                    for (a, c) in back.iter().zip([r, g, b]) {
                        assert!((a - c).abs() < 2e-3, "{r} {g} {b} came back as {back:?}");
                    }
                }
            }
        }
    }
}
