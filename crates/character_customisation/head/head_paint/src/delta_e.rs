//! Colour differences between two Lab colours.

/// The CIE76 difference, which is the straight distance between the two colours in Lab.
pub fn delta_e_76(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// The CIEDE2000 difference, following Sharma, Wu and Dalal (2005).
pub fn delta_e_2000(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (l1, a1, b1) = (a[0], a[1], a[2]);
    let (l2, a2, b2) = (b[0], b[1], b[2]);
    let c_mean = (a1.hypot(b1) + a2.hypot(b2)) / 2.0;
    let c7 = c_mean.powi(7);
    let g = 0.5 * (1.0 - (c7 / (c7 + 25f64.powi(7))).sqrt());
    let (ap1, ap2) = ((1.0 + g) * a1, (1.0 + g) * a2);
    let (cp1, cp2) = (ap1.hypot(b1), ap2.hypot(b2));
    let hue = |b: f64, a: f64| {
        if a == 0.0 && b == 0.0 {
            0.0
        } else {
            b.atan2(a).to_degrees().rem_euclid(360.0)
        }
    };
    let (hp1, hp2) = (hue(b1, ap1), hue(b2, ap2));
    let dl = l2 - l1;
    let dc = cp2 - cp1;
    let dh_angle = if cp1 * cp2 == 0.0 {
        0.0
    } else {
        let d = hp2 - hp1;
        if d > 180.0 {
            d - 360.0
        } else if d < -180.0 {
            d + 360.0
        } else {
            d
        }
    };
    let dh = 2.0 * (cp1 * cp2).sqrt() * (dh_angle / 2.0).to_radians().sin();
    let l_mean = (l1 + l2) / 2.0;
    let cp_mean = (cp1 + cp2) / 2.0;
    let hp_mean = if cp1 * cp2 == 0.0 {
        hp1 + hp2
    } else if (hp1 - hp2).abs() <= 180.0 {
        (hp1 + hp2) / 2.0
    } else if hp1 + hp2 < 360.0 {
        (hp1 + hp2 + 360.0) / 2.0
    } else {
        (hp1 + hp2 - 360.0) / 2.0
    };
    let t = 1.0 - 0.17 * (hp_mean - 30.0).to_radians().cos()
        + 0.24 * (2.0 * hp_mean).to_radians().cos()
        + 0.32 * (3.0 * hp_mean + 6.0).to_radians().cos()
        - 0.20 * (4.0 * hp_mean - 63.0).to_radians().cos();
    let sl = 1.0 + 0.015 * (l_mean - 50.0).powi(2) / (20.0 + (l_mean - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * cp_mean;
    let sh = 1.0 + 0.015 * cp_mean * t;
    let cp7 = cp_mean.powi(7);
    let rc = 2.0 * (cp7 / (cp7 + 25f64.powi(7))).sqrt();
    let d_theta = 30.0 * (-((hp_mean - 275.0) / 25.0).powi(2)).exp();
    let rt = -(2.0 * d_theta).to_radians().sin() * rc;
    let (x, y, z) = (dl / sl, dc / sc, dh / sh);
    (x * x + y * y + z * z + rt * y * z).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ciede2000_matches_published_test_pairs() {
        let cases = [
            ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
            ([50.0, 3.1571, -77.2803], [50.0, 0.0, -82.7485], 2.8615),
            ([50.0, 2.5, 0.0], [73.0, 25.0, -18.0], 27.1492),
            (
                [60.2574, -34.0099, 36.2677],
                [60.4626, -34.1751, 39.4387],
                1.2644,
            ),
            ([50.0, 0.0, 0.0], [50.0, -1.0, 2.0], 2.3669),
        ];
        for (a, b, expected) in cases {
            let got = delta_e_2000(a, b);
            assert!(
                (got - expected).abs() < 1e-3,
                "{a:?} {b:?} gave {got}, expected {expected}"
            );
        }
    }

    #[test]
    fn identical_colours_differ_by_zero() {
        assert_eq!(delta_e_76([50.0, 10.0, 20.0], [50.0, 10.0, 20.0]), 0.0);
        assert_eq!(delta_e_2000([50.0, 10.0, 20.0], [50.0, 10.0, 20.0]), 0.0);
    }
}
