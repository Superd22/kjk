//! Small vector helpers for points and directions in 3D.

pub type V3 = [f64; 3];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// `a + b * s`.
pub fn mad(a: V3, b: V3, s: f64) -> V3 {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s]
}

pub fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn length(a: V3) -> f64 {
    dot(a, a).sqrt()
}

pub fn distance(a: V3, b: V3) -> f64 {
    length(sub(a, b))
}

/// The unit vector along `a`, or zero when `a` is too short to have a direction.
pub fn normalize(a: V3) -> V3 {
    let l = length(a);
    if l > 1e-12 { scale(a, 1.0 / l) } else { [0.0; 3] }
}

pub fn lerp(a: V3, b: V3, t: f64) -> V3 {
    mad(a, sub(b, a), t)
}

/// `a` with its part along the unit vector `n` removed.
pub fn reject(a: V3, n: V3) -> V3 {
    mad(a, n, -dot(a, n))
}

/// A unit vector at a right angle to the unit vector `n`.
pub fn any_perpendicular(n: V3) -> V3 {
    let helper = if n[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
    normalize(cross(n, helper))
}

pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A small, fast random number generator (SplitMix64), so that a hairstyle is the same on every run.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number from 0 (included) to 1 (excluded).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    /// A number from a normal distribution with mean 0 and standard deviation 1.
    pub fn normal(&mut self) -> f64 {
        let u = self.unit().max(1e-12);
        let v = self.unit();
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
    }

    /// A direction spread evenly over the sphere.
    pub fn direction(&mut self) -> V3 {
        let y = self.range(-1.0, 1.0);
        let a = self.range(0.0, std::f64::consts::TAU);
        let r = (1.0 - y * y).max(0.0).sqrt();
        [r * a.sin(), y, r * a.cos()]
    }
}

/// A hash of three integers to a number from 0 to 1, for noise that is the same wherever it is sampled from.
pub fn hash3(x: i64, y: i64, z: i64, seed: u64) -> f64 {
    let mut h = seed ^ (x as u64).wrapping_mul(0x8DA6_B343) ^ (y as u64).wrapping_mul(0xD816_3841) ^ (z as u64).wrapping_mul(0xCB1A_B31F);
    h = (h ^ (h >> 33)).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h = (h ^ (h >> 33)).wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^= h >> 33;
    (h >> 11) as f64 / (1u64 << 53) as f64
}
