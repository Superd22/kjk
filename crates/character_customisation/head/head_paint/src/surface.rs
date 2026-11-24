//! Maps of a head's shape, computed at each vertex from the head's own surface: how far a point stands out of the surface around it, and how
//! much of the sky above the head the point sees past the head itself. The shading layer of the paint reads them, and so does the study of
//! KOTOR's painted shading (`shading_maps`). Positions are in metres, with y pointing up.

use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

type Vec3 = [f64; 3];

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(a: Vec3) -> Vec3 {
    let l = dot(a, a).sqrt();
    if l > 0.0 { a.map(|x| x / l) } else { [0.0; 3] }
}

/// A uniform grid over points, for finding the points within a radius.
pub struct PointGrid {
    cell: f64,
    cells: HashMap<[i32; 3], Vec<u32>>,
}

impl PointGrid {
    pub fn new(points: &[Vec3], cell: f64) -> Self {
        let mut cells: HashMap<[i32; 3], Vec<u32>> = HashMap::new();
        for (i, p) in points.iter().enumerate() {
            cells.entry(p.map(|x| (x / cell).floor() as i32)).or_default().push(i as u32);
        }
        PointGrid { cell, cells }
    }

    /// Calls `f` with every point in the cells that a cube of half-size `radius` around `p` touches. Some of those points lie farther than
    /// `radius`, so the caller checks the distance when it matters.
    pub fn near(&self, p: Vec3, radius: f64, mut f: impl FnMut(u32)) {
        let lo = p.map(|x| ((x - radius) / self.cell).floor() as i32);
        let hi = p.map(|x| ((x + radius) / self.cell).floor() as i32);
        for a in lo[0]..=hi[0] {
            for b in lo[1]..=hi[1] {
                for c in lo[2]..=hi[2] {
                    if let Some(list) = self.cells.get(&[a, b, c]) {
                        list.iter().for_each(|&i| f(i));
                    }
                }
            }
        }
    }
}

/// A head's surface, ready for the maps: its points, their normals, the area each point stands for, and a grid to find neighbours.
pub struct Surface<'a> {
    pub positions: &'a [Vec3],
    pub normals: Vec<Vec3>,
    /// A third of the area of each triangle that uses the point. A point that no triangle uses has no area and is left out of every map.
    pub areas: Vec<f64>,
    grid: PointGrid,
}

impl<'a> Surface<'a> {
    pub fn new(positions: &'a [Vec3], triangles: &[[u32; 3]]) -> Self {
        Surface {
            positions,
            normals: crate::mesh::vertex_normals(positions, triangles),
            areas: crate::mesh::vertex_areas(positions, triangles),
            grid: PointGrid::new(positions, 0.01),
        }
    }

    pub fn is_used(&self, point: usize) -> bool {
        self.areas[point] > 0.0
    }

    /// How far the point stands out of the surface around it, along its normal, in metres. The surface around it is the mean of the points
    /// within three `scale`s, weighted by their area and by a Gaussian of standard deviation `scale`. It is positive on a bump and negative in
    /// a crease.
    pub fn bump(&self, point: usize, scale: f64) -> f64 {
        let p = self.positions[point];
        let (mut sum, mut weight) = ([0.0; 3], 0.0);
        self.grid.near(p, 3.0 * scale, |j| {
            let q = self.positions[j as usize];
            let d = sub(q, p);
            let w = self.areas[j as usize] * (-dot(d, d) / (2.0 * scale * scale)).exp();
            for c in 0..3 {
                sum[c] += w * q[c];
            }
            weight += w;
        });
        let mean = sum.map(|x| x / weight.max(1e-30));
        dot(self.normals[point], sub(p, mean))
    }

    /// The horizon around the point, out to `reach` metres, in `HORIZON_SECTORS` directions around its normal.
    pub fn horizon(&self, point: usize, reach: f64) -> Horizon {
        let p = self.positions[point];
        let n = self.normals[point];
        let helper = if n[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
        let t1 = unit(cross(n, helper));
        let t2 = cross(n, t1);
        let mut sine = [0.0f64; HORIZON_SECTORS];
        self.grid.near(p, reach, |j| {
            if j as usize == point || !self.is_used(j as usize) {
                return;
            }
            let d = sub(self.positions[j as usize], p);
            let length = dot(d, d).sqrt();
            if length > reach || length < 1e-6 {
                return;
            }
            let s = dot(d, n) / length;
            if s <= 0.0 {
                return;
            }
            let angle = dot(d, t2).atan2(dot(d, t1));
            let sector = (((angle + PI) / TAU * HORIZON_SECTORS as f64) as usize).min(HORIZON_SECTORS - 1);
            sine[sector] = sine[sector].max(s);
        });
        Horizon { sine, normal: n, tangents: [t1, t2] }
    }
}

/// The number of directions around the normal in which a horizon is measured.
pub const HORIZON_SECTORS: usize = 16;
const SKY_STEPS: usize = 24;

/// How high the surrounding surface rises above a point's tangent plane, in each of `HORIZON_SECTORS` directions around its normal.
pub struct Horizon {
    /// The sine of the highest elevation of the surface in each direction, seen from the point. Zero is a flat horizon.
    pub sine: [f64; HORIZON_SECTORS],
    normal: Vec3,
    tangents: [Vec3; 2],
}

impl Horizon {
    /// The share of a uniform sky over the point's whole hemisphere that the horizon lets through, weighted by the cosine to the normal. One
    /// is fully open and zero is fully shut.
    pub fn openness(&self) -> f64 {
        self.sine.iter().map(|s| 1.0 - s * s).sum::<f64>() / HORIZON_SECTORS as f64
    }

    /// The light from a sky above the head that reaches the point past the horizon, weighted by the cosine to the normal. `uniform` is a sky
    /// of the same brightness in every upward direction, and `cosine` a sky that is brightest straight up. Each is a share of what an open
    /// point facing straight up receives, so a point facing up with a flat horizon gets 1 and a point facing down gets 0.
    pub fn sky(&self) -> Sky {
        let [t1, t2] = self.tangents;
        let n = self.normal;
        let (mut uniform, mut cosine) = (0.0, 0.0);
        for (k, &h) in self.sine.iter().enumerate() {
            let phi = (k as f64 + 0.5) / HORIZON_SECTORS as f64 * TAU - PI;
            let t = [0, 1, 2].map(|c| phi.cos() * t1[c] + phi.sin() * t2[c]);
            let from = h.clamp(0.0, 1.0).asin();
            let step = (FRAC_PI_2 - from) / SKY_STEPS as f64;
            for e in 0..SKY_STEPS {
                let el = from + (e as f64 + 0.5) * step;
                let w = [0, 1, 2].map(|c| el.cos() * t[c] + el.sin() * n[c]);
                let weight = el.sin() * el.cos() * step * TAU / HORIZON_SECTORS as f64;
                if w[1] > 0.0 {
                    uniform += weight;
                    cosine += weight * w[1];
                }
            }
        }
        Sky { uniform: uniform / PI, cosine: cosine / (2.0 * PI / 3.0) }
    }
}

/// The two skies of `Horizon::sky`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sky {
    pub uniform: f64,
    pub cosine: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square grid of points `size` metres wide in the plane y = height(x, z), split into triangles with their normals pointing up.
    fn sheet(count: usize, size: f64, height: impl Fn(f64, f64) -> f64) -> (Vec<Vec3>, Vec<[u32; 3]>) {
        let mut points = Vec::new();
        for i in 0..count {
            for j in 0..count {
                let (x, z) = ((i as f64 / (count - 1) as f64 - 0.5) * size, (j as f64 / (count - 1) as f64 - 0.5) * size);
                points.push([x, height(x, z), z]);
            }
        }
        let mut triangles = Vec::new();
        for i in 0..count - 1 {
            for j in 0..count - 1 {
                let a = (i * count + j) as u32;
                let (b, c, d) = (a + 1, a + count as u32, a + count as u32 + 1);
                triangles.push([a, b, c]);
                triangles.push([b, d, c]);
            }
        }
        (points, triangles)
    }

    fn centre(count: usize) -> usize {
        (count / 2) * count + count / 2
    }

    #[test]
    fn an_open_flat_floor_sees_the_whole_sky() {
        let (points, triangles) = sheet(31, 0.1, |_, _| 0.0);
        let surface = Surface::new(&points, &triangles);
        assert!(surface.normals[centre(31)][1] > 0.999, "the sheet must face up");
        let h = surface.horizon(centre(31), 0.025);
        assert!((h.openness() - 1.0).abs() < 1e-9);
        let sky = h.sky();
        assert!((sky.uniform - 1.0).abs() < 0.01 && (sky.cosine - 1.0).abs() < 0.01, "{sky:?}");
    }

    #[test]
    fn the_bottom_of_a_groove_sees_less_sky_than_open_ground() {
        let (points, triangles) = sheet(41, 0.1, |x, _| 0.3 * x.abs().min(0.02));
        let surface = Surface::new(&points, &triangles);
        let groove = surface.horizon(centre(41), 0.025).sky().uniform;
        assert!(groove < 0.97, "{groove}");
    }

    #[test]
    fn a_bump_stands_out_and_a_dent_sinks_in() {
        let bell = |sign: f64| move |x: f64, z: f64| sign * 0.01 * (-(x * x + z * z) / (2.0 * 0.015f64.powi(2))).exp();
        for (sign, positive) in [(1.0, true), (-1.0, false)] {
            let (points, triangles) = sheet(61, 0.2, bell(sign));
            let surface = Surface::new(&points, &triangles);
            let b = surface.bump(centre(61), 0.02);
            assert_eq!(b > 0.001, positive, "{b}");
            assert_eq!(b < -0.001, !positive, "{b}");
        }
    }

    #[test]
    fn a_flat_sheet_has_no_bump() {
        let (points, triangles) = sheet(31, 0.2, |_, _| 0.0);
        let surface = Surface::new(&points, &triangles);
        assert!(surface.bump(centre(31), 0.02).abs() < 1e-12);
    }

    #[test]
    fn a_point_that_no_triangle_uses_is_not_used() {
        let (mut points, triangles) = sheet(5, 0.1, |_, _| 0.0);
        points.push([0.0, 0.0, 0.0]);
        let surface = Surface::new(&points, &triangles);
        assert!(!surface.is_used(points.len() - 1));
        assert!(surface.is_used(0));
    }
}
