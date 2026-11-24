//! The hair cap: the shell as a coarse mesh, with its texture laid out by an azimuthal projection centred on the hair region.
//!
//! A direction is mapped to a point of the unit disc: its distance from the centre is its angle from the pole divided by the largest
//! angle, and its bearing is its bearing around the pole. Angles along any line through the pole keep their length (an azimuthal
//! equidistant projection), and the region's edge, about 85 degrees from the pole, is squeezed by about a third along the edge.

use crate::cards::Mesh;
use crate::frame::angle_between;
use crate::shell::Shell;
use crate::v3::{V3, add, any_perpendicular, cross, dot, normalize, reject, scale};

#[derive(Clone, Copy, Debug)]
pub struct Projection {
    pub pole: V3,
    e1: V3,
    e2: V3,
    /// The angle from the pole, in radians, that maps to the disc's edge.
    pub max_angle: f64,
}

impl Projection {
    pub fn new(pole: V3, max_angle: f64) -> Self {
        let pole = normalize(pole);
        let mut e1 = normalize(reject([1.0, 0.0, 0.0], pole));
        if crate::v3::length(e1) == 0.0 {
            e1 = any_perpendicular(pole);
        }
        Projection { pole, e1, e2: cross(pole, e1), max_angle }
    }

    /// The projection that fits every direction of `region`, plus `margin_deg`, centred on their mean.
    pub fn fitting(region: &[V3], margin_deg: f64) -> Self {
        let mean = normalize(region.iter().fold([0.0; 3], |a, d| add(a, *d)));
        let widest = region.iter().map(|d| angle_between(*d, mean)).fold(0.0, f64::max);
        Projection::new(mean, (widest + margin_deg).min(175.0).to_radians())
    }

    pub fn to_plane(&self, d: V3) -> [f64; 2] {
        let theta = dot(d, self.pole).clamp(-1.0, 1.0).acos();
        let psi = dot(d, self.e2).atan2(dot(d, self.e1));
        let r = theta / self.max_angle;
        [r * psi.cos(), r * psi.sin()]
    }

    pub fn from_plane(&self, q: [f64; 2]) -> V3 {
        let r = (q[0] * q[0] + q[1] * q[1]).sqrt();
        let theta = r * self.max_angle;
        let psi = q[1].atan2(q[0]);
        normalize(add(scale(self.pole, theta.cos()), scale(add(scale(self.e1, psi.cos()), scale(self.e2, psi.sin())), theta.sin())))
    }
}

/// Where the cap's texture lies on the atlas, as a square of texture coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CapRect {
    pub u: [f64; 2],
    pub v: [f64; 2],
}

impl CapRect {
    pub fn uv(&self, q: [f64; 2]) -> [f64; 2] {
        [self.u[0] + (q[0] + 1.0) / 2.0 * (self.u[1] - self.u[0]), self.v[0] + (q[1] + 1.0) / 2.0 * (self.v[1] - self.v[0])]
    }
}

/// One ring of the cap: its radius on the disc at each bearing, and its number of sectors.
type Ring<'a> = (Box<dyn Fn(f64) -> f64 + 'a>, usize);

/// The largest radius of the disc, along the bearing `psi`, at which `level` mm past the hairline is still reached, by a scan in steps of
/// 1% of the radius and a bisection.
fn edge_radius(shell: &Shell, projection: &Projection, psi: f64, level: f64) -> f64 {
    let at = |r: f64| shell.hairline_mm(projection.from_plane([r * psi.cos(), r * psi.sin()])) - level;
    let mut last_inside = 0.0;
    let mut r = 0.0;
    while r <= 1.0 {
        if at(r) >= 0.0 {
            last_inside = r;
        }
        r += 0.01;
    }
    let (mut lo, mut hi) = (last_inside, (last_inside + 0.01).min(1.0));
    for _ in 0..30 {
        let mid = 0.5 * (lo + hi);
        if at(mid) >= 0.0 { lo = mid } else { hi = mid }
    }
    lo
}

/// The cap: rings of quads that follow the hairline. The outer ring lies `fade_mm` past the hairline, where the texture has faded out,
/// and the next ring lies on the hairline itself, so the cap's edge follows the hairline instead of a grid. Further rings are spaced
/// about `cell_mm` apart toward the projection's pole. A ring has as many sectors as fit the hairline's length at `cell_mm`, and an inner
/// ring halves its sectors whenever its quads would be narrower than three quarters of `cell_mm`. Its points lie on the shell, lifted by
/// the sag of a quad.
pub fn cap_mesh(shell: &Shell, projection: &Projection, cell_mm: f64, fade_mm: f64, rect: CapRect) -> Mesh {
    let samples = 360;
    let psi = |j: usize, n: usize| std::f64::consts::TAU * j as f64 / n as f64;
    let hair: Vec<f64> = (0..samples).map(|j| edge_radius(shell, projection, psi(j, samples), 0.0)).collect();
    let fade: Vec<f64> = (0..samples).map(|j| edge_radius(shell, projection, psi(j, samples), -fade_mm).max(hair[j])).collect();
    let point = |r: f64, a: f64| shell.point(projection.from_plane([r * a.cos(), r * a.sin()]), 0.0);
    let length_mm = |radii: &[f64]| -> f64 {
        (0..samples).map(|j| crate::v3::distance(point(radii[j], psi(j, samples)), point(radii[(j + 1) % samples], psi(j + 1, samples))) * 1000.0).sum()
    };
    let mut sectors = ((length_mm(&hair) / cell_mm).round() as usize).max(8);
    sectors = sectors.div_ceil(4) * 4;
    let radius_at = |radii: &[f64], a: f64| {
        let x = a / std::f64::consts::TAU * samples as f64;
        let j = x.floor() as usize % samples;
        let f = x - x.floor();
        radii[j] * (1.0 - f) + radii[(j + 1) % samples] * f
    };
    let reach_mm = (0..samples).map(|j| crate::v3::distance(point(hair[j], psi(j, samples)), point(0.0, 0.0)) * 1000.0).fold(0.0, f64::max);
    let inner = ((reach_mm / cell_mm).round() as usize).max(1);
    // Each ring: a fraction of the way from the hairline to the pole (or the fade line, for ring 0), and its sector count.
    let mut rings: Vec<Ring> = Vec::new();
    rings.push((Box::new(|a| radius_at(&fade, a)), sectors));
    let mut n = sectors;
    for k in 0..inner {
        let t = 1.0 - k as f64 / inner as f64;
        let ring_mm: f64 = (0..n).map(|j| crate::v3::distance(point(radius_at(&hair, psi(j, n)) * t, psi(j, n)), point(radius_at(&hair, psi(j + 1, n)) * t, psi(j + 1, n))) * 1000.0).sum();
        while k > 0 && n >= 8 && n.is_multiple_of(2) && ring_mm / (n as f64) < cell_mm * 0.75 {
            n /= 2;
        }
        let hair_ref = &hair;
        rings.push((Box::new(move |a| radius_at(hair_ref, a) * t), n));
    }
    let mut mesh = Mesh::default();
    // A flat quad of side c on a curve of radius r dips c² / 8r below it at its middle, so the cap is lifted by that much and the skin
    // never shows through a large quad.
    let sag = (cell_mm / 1000.0).powi(2) / (8.0 * shell.frame.radius);
    let push = |q: [f64; 2], mesh: &mut Mesh| -> u32 {
        let d = projection.from_plane(q);
        let p = shell.point(d, sag);
        mesh.positions.push(p);
        mesh.normals.push(shell.normal(p));
        mesh.uv.push(rect.uv(q));
        (mesh.positions.len() - 1) as u32
    };
    let mut ring_ids: Vec<Vec<u32>> = Vec::new();
    for (radius, n) in &rings {
        let ids = (0..*n).map(|j| { let a = psi(j, *n); let r = radius(a); push([r * a.cos(), r * a.sin()], &mut mesh) }).collect();
        ring_ids.push(ids);
    }
    let pole = push([0.0, 0.0], &mut mesh);
    for w in ring_ids.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let (na, nb) = (a.len(), b.len());
        if na == nb {
            for j in 0..na {
                let j1 = (j + 1) % na;
                mesh.triangles.push([a[j], a[j1], b[j1]]);
                mesh.triangles.push([a[j], b[j1], b[j]]);
            }
        } else {
            for i in 0..nb {
                let (a0, a1, a2) = (a[2 * i], a[2 * i + 1], a[(2 * i + 2) % na]);
                let (b0, b1) = (b[i], b[(i + 1) % nb]);
                mesh.triangles.push([a0, a1, b0]);
                mesh.triangles.push([a1, b1, b0]);
                mesh.triangles.push([a1, a2, b1]);
            }
        }
    }
    if let Some(last) = ring_ids.last() {
        for j in 0..last.len() {
            mesh.triangles.push([last[j], last[(j + 1) % last.len()], pole]);
        }
    }
    // Where the disc's edge clips both the fade line and the hairline, two rings meet and their triangles have no area.
    let positions = &mesh.positions;
    mesh.triangles.retain(|t| {
        let [a, b, c] = t.map(|i| positions[i as usize]);
        crate::v3::length(cross(crate::v3::sub(b, a), crate::v3::sub(c, a))) > 1e-12
    });
    // The winding depends on the projection's handedness. Turn the triangles so that they face away from the head.
    if let Some(&t) = mesh.triangles.first() {
        let [a, b, c] = t.map(|i| mesh.positions[i as usize]);
        let n = cross(crate::v3::sub(b, a), crate::v3::sub(c, a));
        if dot(n, mesh.normals[t[0] as usize]) < 0.0 {
            for t in &mut mesh.triangles {
                t.swap(1, 2);
            }
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::direction;

    #[test]
    fn the_projection_round_trips() {
        let p = Projection::new(direction(180.0, 40.0), 90f64.to_radians());
        for (az, el) in [(180.0, 40.0), (0.0, 80.0), (120.0, -10.0), (-150.0, 0.0)] {
            let d = direction(az, el);
            let back = p.from_plane(p.to_plane(d));
            assert!(angle_between(d, back) < 1e-6, "{az} {el}");
        }
        let edge = p.to_plane(direction(180.0, -50.0));
        assert!(((edge[0] * edge[0] + edge[1] * edge[1]).sqrt() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn the_cap_lies_on_the_shell_and_faces_out() {
        let shell = crate::shell::tests::sphere_shell();
        let region: Vec<V3> = (0..200).map(|i| direction(i as f64 * 7.0, 5.0 + (i % 17) as f64 * 5.0)).collect();
        let p = Projection::fitting(&region, 5.0);
        let m = cap_mesh(&shell, &p, 15.0, 4.0, CapRect { u: [0.0, 0.75], v: [0.25, 1.0] });
        assert!(m.triangles.len() > 50);
        for q in &m.positions {
            assert!((0.0..0.002).contains(&shell.above_shell(*q)));
        }
        for t in &m.triangles {
            let [a, b, c] = t.map(|i| m.positions[i as usize]);
            let n = cross(crate::v3::sub(b, a), crate::v3::sub(c, a));
            assert!(dot(n, normalize(a)) > 0.0, "a triangle faces into the head");
        }
    }
}
