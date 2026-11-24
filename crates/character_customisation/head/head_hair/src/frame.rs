//! The head's frame for hair: a centre inside the skull, angles around it, and the distance from the centre to the head's surface in
//! every direction.
//!
//! The axes are those of the head data: x points to the character's left, y up and z forward, out of the face. The centre and radius
//! come from a sphere fitted to the head's points above the eyes, so a hairstyle written in angles fits heads of any size.

use crate::v3::{V3, cross, dot, normalize, sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeadFrame {
    pub centre: V3,
    /// The radius of the fitted sphere, in the head data's units (metres).
    pub radius: f64,
}

/// The unit direction at an azimuth and an elevation, in degrees.
pub fn direction(az: f64, el: f64) -> V3 {
    let (a, e) = (az.to_radians(), el.to_radians());
    [e.cos() * a.sin(), e.sin(), e.cos() * a.cos()]
}

/// The azimuth and elevation of a unit direction, in degrees.
pub fn angles(d: V3) -> (f64, f64) {
    (d[0].atan2(d[2]).to_degrees(), d[1].clamp(-1.0, 1.0).asin().to_degrees())
}

/// The angle between two unit directions, in degrees.
pub fn angle_between(a: V3, b: V3) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos().to_degrees()
}

impl HeadFrame {
    /// Fits a sphere, by least squares, to the points whose height is above `above_y`.
    pub fn fit(points: &[V3], above_y: f64) -> Self {
        let mut ata = nalgebra::Matrix4::<f64>::zeros();
        let mut atb = nalgebra::Vector4::<f64>::zeros();
        for p in points.iter().filter(|p| p[1] > above_y) {
            let row = nalgebra::Vector4::new(2.0 * p[0], 2.0 * p[1], 2.0 * p[2], 1.0);
            ata += row * row.transpose();
            atb += row * dot(*p, *p);
        }
        let x = ata.lu().solve(&atb).unwrap_or_else(nalgebra::Vector4::zeros);
        let centre = [x[0], x[1], x[2]];
        HeadFrame { centre, radius: (x[3] + dot(centre, centre)).max(0.0).sqrt() }
    }

    /// The direction from the centre to a point.
    pub fn dir(&self, p: V3) -> V3 {
        normalize(sub(p, self.centre))
    }

    /// Converts millimetres on the head to degrees of angle at the sphere's radius.
    pub fn mm_to_deg(&self, mm: f64) -> f64 {
        (mm / 1000.0 / self.radius).to_degrees()
    }

    pub fn deg_to_mm(&self, deg: f64) -> f64 {
        deg.to_radians() * self.radius * 1000.0
    }
}

/// The distance from the frame's centre to the outermost surface of the head along every direction, kept in a table of directions
/// and read by bilinear interpolation.
pub struct RadialDepth {
    centre: V3,
    step: f64,
    n_az: usize,
    n_el: usize,
    r: Vec<f64>,
}

/// Where a ray from `origin` along `d` crosses a triangle, as the distance along the ray.
fn ray_triangle(origin: V3, d: V3, a: V3, b: V3, c: V3) -> Option<f64> {
    let (e1, e2) = (sub(b, a), sub(c, a));
    let h = cross(d, e2);
    let det = dot(e1, h);
    if det.abs() < 1e-14 {
        return None;
    }
    let inv = 1.0 / det;
    let s = sub(origin, a);
    let u = dot(s, h) * inv;
    if !(-1e-9..=1.0 + 1e-9).contains(&u) {
        return None;
    }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < -1e-9 || u + v > 1.0 + 1e-9 {
        return None;
    }
    let t = dot(e2, q) * inv;
    (t > 0.0).then_some(t)
}

impl RadialDepth {
    /// Casts a ray from the centre every `step_deg` degrees of azimuth and elevation, and keeps the farthest crossing of the mesh.
    /// Directions where the ray leaves through a hole of the mesh, such as the bottom of the neck, take the value of the direction just
    /// above them.
    pub fn build(centre: V3, positions: &[V3], triangles: &[[u32; 3]], step_deg: f64) -> Self {
        let n_az = (360.0 / step_deg).round() as usize;
        let n_el = (180.0 / step_deg).round() as usize + 1;
        // Triangles binned by the cells of azimuth and elevation that they cover, so each ray tests only a few.
        let bin = 4.0;
        let (b_az, b_el) = ((360.0 / bin) as usize, (180.0 / bin) as usize + 1);
        let mut bins: Vec<Vec<u32>> = vec![Vec::new(); b_az * b_el];
        for (k, t) in triangles.iter().enumerate() {
            let corners = t.map(|i| sub(positions[i as usize], centre));
            // A large triangle bulges past its corners' angles, so it is sampled inside, about every 2 degrees.
            let size = (0..3).map(|i| angle_between(normalize(corners[i]), normalize(corners[(i + 1) % 3]))).fold(0.0, f64::max);
            let m = (size / 2.0).ceil().max(1.0) as usize;
            let mut ang = Vec::new();
            for i in 0..=m {
                for j in 0..=m - i {
                    let (a, b) = (i as f64 / m as f64, j as f64 / m as f64);
                    let p = [0, 1, 2].map(|c| corners[0][c] * (1.0 - a - b) + corners[1][c] * a + corners[2][c] * b);
                    ang.push(angles(normalize(p)));
                }
            }
            let mut el_lo = ang.iter().map(|a| a.1).fold(f64::INFINITY, f64::min);
            let mut el_hi = ang.iter().map(|a| a.1).fold(f64::NEG_INFINITY, f64::max);
            let pole_up = ray_triangle(centre, [0.0, 1.0, 0.0], positions[t[0] as usize], positions[t[1] as usize], positions[t[2] as usize]).is_some();
            let pole_down = ray_triangle(centre, [0.0, -1.0, 0.0], positions[t[0] as usize], positions[t[1] as usize], positions[t[2] as usize]).is_some();
            if pole_up {
                el_hi = 90.0;
            }
            if pole_down {
                el_lo = -90.0;
            }
            let mut az: Vec<f64> = ang.iter().map(|a| a.0).collect();
            let (mut lo, mut hi) = (az.iter().cloned().fold(f64::INFINITY, f64::min), az.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
            if hi - lo > 180.0 {
                for a in &mut az {
                    if *a < 0.0 {
                        *a += 360.0;
                    }
                }
                lo = az.iter().cloned().fold(f64::INFINITY, f64::min);
                hi = az.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            }
            let full = pole_up || pole_down || el_hi > 84.0 || el_lo < -84.0 || hi - lo > 180.0;
            let e0 = (((el_lo + 90.0) / bin).floor() as i64 - 1).max(0) as usize;
            let e1 = ((((el_hi + 90.0) / bin).floor() as i64 + 1).max(0) as usize).min(b_el - 1);
            let (a0, a1) = if full { (0, b_az as i64 - 1) } else { (((lo + 180.0) / bin).floor() as i64 - 1, ((hi + 180.0) / bin).floor() as i64 + 1) };
            for e in e0..=e1 {
                for a in a0..=a1 {
                    let a = a.rem_euclid(b_az as i64) as usize;
                    let cell = &mut bins[e * b_az + a];
                    if cell.last() != Some(&(k as u32)) {
                        cell.push(k as u32);
                    }
                }
            }
        }
        let mut r = vec![f64::NAN; n_az * n_el];
        for e in 0..n_el {
            let el = -90.0 + e as f64 * step_deg;
            for a in 0..n_az {
                let az = -180.0 + a as f64 * step_deg;
                let d = direction(az, el);
                let be = (((el + 90.0) / bin) as usize).min(b_el - 1);
                let ba = (((az + 180.0) / bin) as usize).min(b_az - 1);
                let mut best = f64::NAN;
                for &k in &bins[be * b_az + ba] {
                    let t = triangles[k as usize].map(|i| positions[i as usize]);
                    if let Some(hit) = ray_triangle(centre, d, t[0], t[1], t[2])
                        && (best.is_nan() || hit > best)
                    {
                        best = hit;
                    }
                }
                r[e * n_az + a] = best;
            }
        }
        for e in (0..n_el - 1).rev() {
            for a in 0..n_az {
                if r[e * n_az + a].is_nan() {
                    r[e * n_az + a] = r[(e + 1) * n_az + a];
                }
            }
        }
        let fallback = r.iter().cloned().filter(|x| x.is_finite()).fold(0.0, f64::max);
        for x in &mut r {
            if !x.is_finite() {
                *x = fallback;
            }
        }
        RadialDepth { centre, step: step_deg, n_az, n_el, r }
    }

    /// A table for a sphere of a radius around a centre, for tests.
    pub fn sphere(centre: V3, radius: f64, step_deg: f64) -> Self {
        let n_az = (360.0 / step_deg).round() as usize;
        let n_el = (180.0 / step_deg).round() as usize + 1;
        RadialDepth { centre, step: step_deg, n_az, n_el, r: vec![radius; n_az * n_el] }
    }

    pub fn centre(&self) -> V3 {
        self.centre
    }

    /// The distance from the centre to the head's surface along a unit direction.
    pub fn at(&self, d: V3) -> f64 {
        let (az, el) = angles(d);
        let x = (az + 180.0) / self.step;
        let y = ((el + 90.0) / self.step).clamp(0.0, (self.n_el - 1) as f64);
        let (x0, y0) = (x.floor(), y.floor().min((self.n_el - 2) as f64));
        let (fx, fy) = (x - x0, y - y0);
        let a0 = (x0 as i64).rem_euclid(self.n_az as i64) as usize;
        let a1 = (a0 + 1) % self.n_az;
        let e0 = y0 as usize;
        let v = |e: usize, a: usize| self.r[e * self.n_az + a];
        let bottom = v(e0, a0) * (1.0 - fx) + v(e0, a1) * fx;
        let top = v(e0 + 1, a0) * (1.0 - fx) + v(e0 + 1, a1) * fx;
        bottom * (1.0 - fy) + top * fy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_and_direction_round_trip() {
        for (az, el) in [(0.0, 0.0), (90.0, 10.0), (-120.0, -40.0), (179.0, 70.0)] {
            let (a, e) = angles(direction(az, el));
            assert!((a - az).abs() < 1e-9 && (e - el).abs() < 1e-9, "{az} {el} gave {a} {e}");
        }
        assert!((direction(90.0, 0.0)[0] - 1.0).abs() < 1e-12, "az 90 is +x, the character's left");
    }

    /// An octahedron-free test: a closed cube around the centre. Along an axis the ray meets a face at the half side.
    #[test]
    fn rays_find_the_surface_of_a_cube() {
        let h = 0.1;
        let p: Vec<V3> = (0..8).map(|i| [if i & 1 != 0 { h } else { -h }, if i & 2 != 0 { h } else { -h }, if i & 4 != 0 { h } else { -h }]).collect();
        let quads = [[0, 1, 3, 2], [4, 6, 7, 5], [0, 4, 5, 1], [2, 3, 7, 6], [0, 2, 6, 4], [1, 5, 7, 3]];
        let tris: Vec<[u32; 3]> = quads.iter().flat_map(|q| [[q[0], q[1], q[2]], [q[0], q[2], q[3]]]).collect();
        let depth = RadialDepth::build([0.0; 3], &p, &tris, 2.0);
        assert!((depth.at([1.0, 0.0, 0.0]) - h).abs() < 1e-6);
        assert!((depth.at([0.0, 1.0, 0.0]) - h).abs() < 1e-6);
        assert!((depth.at([0.0, 0.0, -1.0]) - h).abs() < 1e-6);
    }

    #[test]
    fn a_sphere_fit_finds_the_sphere() {
        let mut points = Vec::new();
        for i in 0..40 {
            for j in 0..20 {
                let d = direction(i as f64 * 9.0 - 180.0, j as f64 * 9.0 - 85.0);
                points.push([0.01 + 0.08 * d[0], 0.7 + 0.08 * d[1], 0.05 + 0.08 * d[2]]);
            }
        }
        let f = HeadFrame::fit(&points, 0.69);
        assert!((f.radius - 0.08).abs() < 1e-9 && (f.centre[1] - 0.7).abs() < 1e-9, "{f:?}");
    }
}
