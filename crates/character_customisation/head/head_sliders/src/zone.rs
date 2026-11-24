//! A feature's zone with a soft edge, and the part of a head's shape that belongs to the feature alone.
//!
//! A head's movement from the base head inside the zone is split in two. The broad part is what the face around the zone would give:
//! the movement at the zone's edge, carried smoothly inward (a harmonic fill, which is how a stretched membrane would lie). The rest
//! is the feature's own shape. Nose sliders move only that rest, so a nose made by sliders sits on whatever face is around it.

use crate::measure::Frame;
use crate::mesh::HeadMesh;
use crate::{V3, length, sub};

/// The nose's zone on the base head: a weight per vertex, 1 on the nose and falling smoothly to 0 over `MARGIN_MM` around it.
pub struct Zone {
    /// Per merged vertex.
    pub weight: Vec<f64>,
}

const MARGIN_MM: f64 = 7.0;

impl Zone {
    /// The nose: from 2 mm below the nose's base up to 6 mm above the eyes' centres, 20 mm to each side of the middle at the wings,
    /// narrowing to 11 mm between the eyes. The eyes and their lids (within 14 mm of an eye's centre) and the inside of the head
    /// are left out. `tip_y` and `base_y` are the base head's tip and nose base in the frame of its eyes.
    pub fn nose(mesh: &HeadMesh, tip_y: f64, base_y: f64) -> Self {
        let frame = Frame::from_eyes(mesh.eyes);
        let eye_local = mesh.eyes.map(|e| frame.local(e));
        let top = 6.0;
        let bottom = base_y - 2.0;
        let half = |y: f64| {
            if y <= tip_y {
                20.0
            } else {
                let s = ((y - tip_y) / (top - tip_y)).clamp(0.0, 1.0);
                20.0 + s * (11.0 - 20.0)
            }
        };
        let weight = mesh
            .first
            .iter()
            .enumerate()
            .map(|(g, &i)| {
                let p = frame.local(mesh.base[i as usize]);
                if eye_local.iter().any(|e| length(sub(p, *e)) < 14.0)
                    || p[2] < 0.0
                    || (mesh.keep[i as usize] && !mesh.hidden[g])
                {
                    return 0.0;
                }
                let dx = (p[0].abs() - half(p[1].clamp(bottom, top))).max(0.0);
                let dy = (bottom - p[1]).max(0.0).max(p[1] - top);
                let d = (dx * dx + dy * dy).sqrt();
                let t = (1.0 - d / MARGIN_MM).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            })
            .collect();
        Self { weight }
    }

    pub fn contains(&self, g: usize) -> bool {
        self.weight[g] > 0.0
    }

    /// The feature's own shape in a movement given per merged vertex: the movement minus its harmonic fill from the zone's edge,
    /// times the zone's weight. It is zero outside the zone.
    pub fn own_shape(&self, mesh: &HeadMesh, movement: &[V3]) -> Vec<V3> {
        let fill = self.fill(mesh, movement);
        movement
            .iter()
            .zip(&fill)
            .zip(&self.weight)
            .map(|((m, f), &w)| std::array::from_fn(|c| w * (m[c] - f[c])))
            .collect()
    }

    /// The movement inside the zone replaced by the harmonic fill of the movement at its edge.
    pub fn fill(&self, mesh: &HeadMesh, movement: &[V3]) -> Vec<V3> {
        let n = movement.len();
        let inside: Vec<bool> = (0..n).map(|g| self.contains(g)).collect();
        let mut out = movement.to_vec();
        for c in 0..3 {
            // (D - A) x = 0 inside, with the values outside fixed: solved by conjugate gradients on the inside vertices
            let apply = |x: &[f64]| -> Vec<f64> {
                (0..n)
                    .map(|g| {
                        if !inside[g] {
                            return 0.0;
                        }
                        let nb = mesh.neighbours_of(g);
                        let mut s = nb.len() as f64 * x[g];
                        for &j in nb {
                            if inside[j as usize] {
                                s -= x[j as usize];
                            }
                        }
                        s
                    })
                    .collect()
            };
            let b: Vec<f64> = (0..n)
                .map(|g| {
                    if !inside[g] {
                        return 0.0;
                    }
                    mesh.neighbours_of(g)
                        .iter()
                        .filter(|&&j| !inside[j as usize])
                        .map(|&j| movement[j as usize][c])
                        .sum()
                })
                .collect();
            let x0: Vec<f64> = (0..n)
                .map(|g| if inside[g] { movement[g][c] } else { 0.0 })
                .collect();
            let x = conjugate_gradient(&apply, &b, x0, 1e-10, 3000);
            for g in 0..n {
                if inside[g] {
                    out[g][c] = x[g];
                }
            }
        }
        out
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn conjugate_gradient(
    apply: &dyn Fn(&[f64]) -> Vec<f64>,
    b: &[f64],
    mut x: Vec<f64>,
    tolerance: f64,
    rounds: usize,
) -> Vec<f64> {
    let ax = apply(&x);
    let mut r: Vec<f64> = b.iter().zip(&ax).map(|(b, a)| b - a).collect();
    let mut d = r.clone();
    let mut rr = dot(&r, &r);
    let stop = tolerance * dot(b, b).sqrt().max(1e-30);
    for _ in 0..rounds {
        if rr.sqrt() <= stop {
            break;
        }
        let ad = apply(&d);
        let dad = dot(&d, &ad);
        if dad <= 0.0 {
            break;
        }
        let step = rr / dad;
        for i in 0..x.len() {
            x[i] += step * d[i];
            r[i] -= step * ad[i];
        }
        let rr_new = dot(&r, &r);
        let beta = rr_new / rr;
        rr = rr_new;
        for i in 0..d.len() {
            d[i] = r[i] + beta * d[i];
        }
    }
    x
}
