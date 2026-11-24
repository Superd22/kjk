//! The outline of the mesh hair on the skin: where the cap and the cards actually cover the head.
//!
//! A direction is covered when the cap's texture is opaque there (at the alpha test of 0.5), or when a card lies within `CONTACT` of the
//! skin there. Both are drawn into a mask on the cap's projection disc, at the cap texture's resolution. A distance transform then gives
//! every direction outside the mask its distance to it, in millimetres along the head. The painted hair follows this outline, so its
//! fade starts exactly where the mesh hair meets the skin, frayed strands and card roots included.

use crate::atlas::{CAP_PIXELS, Image};
use crate::cap::Projection;
use crate::cards::Mesh;
use crate::shell::Shell;
use crate::v3::V3;
use head_paint::pixel_map::nearest_true;

/// A card closer than this to the skin counts as touching it, in metres.
pub const CONTACT: f64 = 0.005;

pub struct Outline {
    projection: Projection,
    size: usize,
    inside: Vec<bool>,
    distance_mm: Vec<f32>,
}

impl Outline {
    /// The outline from the baked atlas (whose top left `CAP_PIXELS` square is the cap's disc) and the cards.
    pub fn new(shell: &Shell, projection: &Projection, atlas: &Image, cards: &Mesh) -> Self {
        let size = CAP_PIXELS;
        let mut inside = vec![false; size * size];
        for y in 0..size {
            for x in 0..size {
                inside[y * size + x] = atlas.px[y * atlas.width + x][3] >= 0.5;
            }
        }
        let to_px = |p: V3| {
            let q = projection.to_plane(shell.dir(p));
            ([(q[0] + 1.0) / 2.0 * size as f64, (1.0 - q[1]) / 2.0 * size as f64], q[0] * q[0] + q[1] * q[1] <= 1.0)
        };
        for t in &cards.triangles {
            let corners = t.map(|i| cards.positions[i as usize]);
            if corners.iter().any(|&p| shell.above_head(p) > CONTACT) {
                continue;
            }
            let px = corners.map(to_px);
            if px.iter().any(|(_, on)| !on) {
                continue;
            }
            fill_triangle(&mut inside, size, px.map(|(p, _)| p));
        }
        let nearest = nearest_true(&inside, size, size);
        let mm_per_px = 2.0 / size as f64 * projection.max_angle * shell.frame.radius * 1000.0;
        let distance_mm = (0..size * size)
            .map(|at| {
                let s = nearest[at];
                if s == u32::MAX {
                    return f32::INFINITY;
                }
                let s = s as usize;
                let (dy, dx) = ((at / size) as f64 - (s / size) as f64, (at % size) as f64 - (s % size) as f64);
                ((dx * dx + dy * dy).sqrt() * mm_per_px) as f32
            })
            .collect();
        Outline { projection: *projection, size, inside, distance_mm }
    }

    /// The signed distance from a direction to the outline, in millimetres: 1 where the mesh hair covers the head, and minus the
    /// distance to the nearest covered direction elsewhere. Directions outside the projection's disc are infinitely far.
    pub fn signed_mm(&self, d: V3) -> f64 {
        let q = self.projection.to_plane(d);
        if q[0] * q[0] + q[1] * q[1] > 1.0 {
            return f64::NEG_INFINITY;
        }
        let x = (((q[0] + 1.0) / 2.0 * self.size as f64) as usize).min(self.size - 1);
        let y = (((1.0 - q[1]) / 2.0 * self.size as f64) as usize).min(self.size - 1);
        let at = y * self.size + x;
        if self.inside[at] { 1.0 } else { -f64::from(self.distance_mm[at]) }
    }
}

/// Sets every pixel whose centre lies inside a triangle given in pixel coordinates.
fn fill_triangle(mask: &mut [bool], size: usize, p: [[f64; 2]; 3]) {
    let lo = [p[0][0].min(p[1][0]).min(p[2][0]).floor().max(0.0) as usize, p[0][1].min(p[1][1]).min(p[2][1]).floor().max(0.0) as usize];
    let hi = [(p[0][0].max(p[1][0]).max(p[2][0]).ceil() as usize).min(size), (p[0][1].max(p[1][1]).max(p[2][1]).ceil() as usize).min(size)];
    let edge = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    let area = edge(p[0], p[1], p[2]);
    if area.abs() < 1e-12 {
        return;
    }
    for y in lo[1]..hi[1] {
        for x in lo[0]..hi[0] {
            let c = [x as f64 + 0.5, y as f64 + 0.5];
            let (w0, w1, w2) = (edge(p[1], p[2], c) / area, edge(p[2], p[0], c) / area, edge(p[0], p[1], c) / area);
            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                mask[y * size + x] = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filled_triangle_covers_its_inside_only() {
        let mut m = vec![false; 100];
        fill_triangle(&mut m, 10, [[0.0, 0.0], [10.0, 0.0], [0.0, 10.0]]);
        assert!(m[10 + 1] && !m[9 * 10 + 9]);
    }

    #[test]
    fn the_outline_is_inside_where_the_cap_is_opaque_and_measures_the_gap_elsewhere() {
        let shell = crate::shell::tests::sphere_shell();
        let projection = Projection::new([0.0, 1.0, 0.0], 100f64.to_radians());
        let mut atlas = Image::new(crate::atlas::SIZE, crate::atlas::SIZE);
        // The cap is opaque in a disc of half the projection's radius, that is 50 degrees from the top.
        for y in 0..CAP_PIXELS {
            for x in 0..CAP_PIXELS {
                let (u, v) = ((x as f64 + 0.5) / CAP_PIXELS as f64 * 2.0 - 1.0, 1.0 - (y as f64 + 0.5) / CAP_PIXELS as f64 * 2.0);
                if u * u + v * v <= 0.25 {
                    atlas.px[y * crate::atlas::SIZE + x][3] = 1.0;
                }
            }
        }
        let o = Outline::new(&shell, &projection, &atlas, &Mesh::default());
        assert_eq!(o.signed_mm(crate::frame::direction(0.0, 60.0)), 1.0);
        // 10 degrees past the edge on a sphere of 80 mm is about 14 mm.
        let gap = -o.signed_mm(crate::frame::direction(0.0, 30.0));
        assert!((gap - 14.0).abs() < 1.0, "{gap}");
    }
}
