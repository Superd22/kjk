//! Solid noise for the paint layers: values defined at every point of 3D space, so a layer painted from them has no seams and no
//! stretching in the texture layout (paint track, fine skin detail).
//!
//! Every function takes a point in millimetres and a seed. The same point and seed always give the same value.

/// Mixes three cell coordinates and a seed into 64 random bits.
pub fn hash3(i: i64, j: i64, k: i64, seed: u64) -> u64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15;
    for v in [i, j, k] {
        h ^= (v as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
    }
    h
}

/// A number from 0 to 1 taken from `bits` random bits starting at bit `shift` of a hash.
pub fn unit(h: u64, shift: u32) -> f64 {
    ((h >> shift) & 0xFFFF) as f64 / 65536.0
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn gradient(h: u64, d: [f64; 3]) -> f64 {
    match h % 12 {
        0 => d[0] + d[1],
        1 => -d[0] + d[1],
        2 => d[0] - d[1],
        3 => -d[0] - d[1],
        4 => d[0] + d[2],
        5 => -d[0] + d[2],
        6 => d[0] - d[2],
        7 => -d[0] - d[2],
        8 => d[1] + d[2],
        9 => -d[1] + d[2],
        10 => d[1] - d[2],
        _ => -d[1] - d[2],
    }
}

/// Perlin's gradient noise at `p`, with a feature size of one unit. The value lies roughly between -1 and 1 and is 0 on the grid points.
pub fn gradient_noise(p: [f64; 3], seed: u64) -> f64 {
    let cell = p.map(f64::floor);
    let f = [p[0] - cell[0], p[1] - cell[1], p[2] - cell[2]];
    let c = cell.map(|x| x as i64);
    let u = f.map(fade);
    let mut corners = [0.0; 8];
    for (n, corner) in corners.iter_mut().enumerate() {
        let o = [(n & 1) as i64, ((n >> 1) & 1) as i64, ((n >> 2) & 1) as i64];
        let h = hash3(c[0] + o[0], c[1] + o[1], c[2] + o[2], seed);
        *corner = gradient(h, [f[0] - o[0] as f64, f[1] - o[1] as f64, f[2] - o[2] as f64]);
    }
    let lerp = |a: f64, b: f64, t: f64| a + t * (b - a);
    let x00 = lerp(corners[0], corners[1], u[0]);
    let x10 = lerp(corners[2], corners[3], u[0]);
    let x01 = lerp(corners[4], corners[5], u[0]);
    let x11 = lerp(corners[6], corners[7], u[0]);
    lerp(lerp(x00, x10, u[1]), lerp(x01, x11, u[1]), u[2])
}

/// Gradient noise summed over `octaves` scales. The first octave has features of `wavelength` millimetres, each next octave half the
/// size and `gain` times the amplitude. The sum is divided by the total amplitude, so it stays roughly between -1 and 1.
pub fn fractal_noise(p: [f64; 3], wavelength: f64, octaves: u32, gain: f64, seed: u64) -> f64 {
    let (mut sum, mut total, mut amplitude, mut scale) = (0.0, 0.0, 1.0, 1.0 / wavelength);
    for o in 0..octaves {
        sum += amplitude * gradient_noise(p.map(|x| x * scale), seed.wrapping_add(u64::from(o) * 7919));
        total += amplitude;
        amplitude *= gain;
        scale *= 2.0;
    }
    sum / total
}

/// One-dimensional gradient noise along a line, for wobbles and strength changes along a wrinkle.
pub fn line_noise(t: f64, seed: u64) -> f64 {
    gradient_noise([t, 0.37, 0.61], seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_point_and_seed_give_the_same_value() {
        let p = [12.3, -4.5, 88.1];
        assert_eq!(gradient_noise(p, 3), gradient_noise(p, 3));
        assert_ne!(gradient_noise(p, 3), gradient_noise(p, 4));
    }

    #[test]
    fn gradient_noise_is_continuous_and_bounded() {
        let mut worst: f64 = 0.0;
        for i in 0..2000 {
            let p = [i as f64 * 0.0137, (i as f64 * 0.31).sin() * 5.0, 2.0];
            let a = gradient_noise(p, 1);
            let b = gradient_noise([p[0] + 1e-4, p[1], p[2]], 1);
            worst = worst.max((a - b).abs());
            assert!(a.abs() <= 1.5);
        }
        assert!(worst < 1e-3, "jump {worst}");
    }

    #[test]
    fn fractal_noise_has_some_spread() {
        let values: Vec<f64> = (0..4000).map(|i| fractal_noise([i as f64 * 0.21, (i / 63) as f64 * 0.37, 0.5], 2.0, 3, 0.5, 9)).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let sd = (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt();
        assert!(mean.abs() < 0.05 && sd > 0.1 && sd < 0.6, "mean {mean} sd {sd}");
    }
}
