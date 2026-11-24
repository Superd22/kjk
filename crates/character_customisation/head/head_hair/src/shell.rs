//! The hair shell: the scalp pushed outward by a height that sets the hair's volume.
//!
//! The height comes from the style's volume: a base height inside the hairline, local bumps, and a taper to nothing at the hairline.
//! Outside the hairline the shell lies `MIN_LIFT` above the skin, so loose hair such as bangs never sinks into the face. Every distance
//! is measured along the ray from the head's centre.

use crate::frame::{HeadFrame, RadialDepth, angle_between, angles, direction};
use crate::style::{Bump, Style, bump_weight, hairline_el};
use crate::v3::{V3, length, mad, normalize, smoothstep, sub};

/// How far above the skin the shell lies where it has no volume, in metres.
pub const MIN_LIFT: f64 = 0.0008;

/// The head's triangles, for an exact test of whether a point is inside the skin where the ray from the centre grazes the surface, as
/// it does along the back of the neck.
pub struct Skin {
    bvh: head_paint::closest::Bvh,
    normals: Vec<V3>,
}

impl Skin {
    pub fn new(positions: &[V3], triangles: &[[u32; 3]]) -> Self {
        let normals = triangles
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| positions[i as usize]);
                normalize(crate::v3::cross(sub(b, a), sub(c, a)))
            })
            .collect();
        Skin { bvh: head_paint::closest::Bvh::new(positions.to_vec(), triangles.to_vec()), normals }
    }

    /// The point moved out along the normal of the nearest triangle, if needed, so that it lies at least `margin` outside the skin.
    pub fn push_out(&self, p: V3, margin: f64) -> V3 {
        let Some(c) = self.bvh.closest(p) else { return p };
        let n = self.normals[c.triangle as usize];
        let side = crate::v3::dot(sub(p, c.point), n);
        if side < margin && c.distance < 0.02 { mad(c.point, n, margin) } else { p }
    }
}

pub struct Shell {
    pub frame: HeadFrame,
    pub skin: Option<Skin>,
    pub depth: RadialDepth,
    hairline: Vec<[f64; 2]>,
    base_mm: f64,
    taper_mm: f64,
    bumps: Vec<Bump>,
}

/// A base value pulled toward each bump's value by the bump's weight, with the weights' sum capped at 1.
pub fn bumps_at(bumps: &[Bump], d: V3, base: f64) -> f64 {
    let mut value = base;
    let mut total = 0.0;
    for b in bumps {
        let w = bump_weight(angle_between(d, direction(b.az, b.el)), b.radius_deg);
        if w <= 0.0 {
            continue;
        }
        let w = w.min(1.0 - total);
        value += w * (b.mm - base);
        total += w;
        if total >= 1.0 {
            break;
        }
    }
    value
}

impl Shell {
    pub fn new(frame: HeadFrame, depth: RadialDepth, style: &Style) -> Self {
        Shell {
            frame,
            skin: None,
            depth,
            hairline: style.hairline.clone(),
            base_mm: style.volume.base_mm,
            taper_mm: style.volume.taper_mm,
            bumps: style.volume.bumps.clone(),
        }
    }

    pub fn dir(&self, p: V3) -> V3 {
        self.frame.dir(p)
    }

    /// The signed distance from a direction to the hairline, in millimetres on the head. It is positive where hair grows.
    pub fn hairline_mm(&self, d: V3) -> f64 {
        let (az, el) = angles(d);
        self.frame.deg_to_mm(el - hairline_el(&self.hairline, az))
    }

    /// The signed distance from a direction to another line of `[az, el]` knots, in millimetres, positive above the line.
    pub fn line_mm(&self, knots: &[[f64; 2]], d: V3) -> f64 {
        let (az, el) = angles(d);
        self.frame.deg_to_mm(el - hairline_el(knots, az))
    }

    /// The height of the hair above the skin along a direction, in metres.
    pub fn height(&self, d: V3) -> f64 {
        let inside = self.hairline_mm(d);
        if inside <= 0.0 {
            return MIN_LIFT;
        }
        let mut extra = self.base_mm;
        for b in &self.bumps {
            extra += b.mm * bump_weight(angle_between(d, direction(b.az, b.el)), b.radius_deg);
        }
        MIN_LIFT + extra.max(0.0) / 1000.0 * smoothstep(0.0, self.taper_mm.max(1e-6), inside)
    }

    /// The distance from the centre to the shell along a direction.
    pub fn radius(&self, d: V3) -> f64 {
        self.depth.at(d) + self.height(d)
    }

    /// The point `lift` metres above the shell along a direction.
    pub fn point(&self, d: V3, lift: f64) -> V3 {
        mad(self.frame.centre, d, self.radius(d) + lift)
    }

    /// How far a point lies outside the shell, along the ray from the centre. It is negative inside.
    pub fn above_shell(&self, p: V3) -> f64 {
        length(sub(p, self.frame.centre)) - self.radius(self.dir(p))
    }

    /// How far a point lies outside the skin, along the ray from the centre.
    pub fn above_head(&self, p: V3) -> f64 {
        length(sub(p, self.frame.centre)) - self.depth.at(self.dir(p))
    }

    /// The point moved out along its ray, if needed, so that it lies at least `margin` above the shell.
    pub fn push_out(&self, p: V3, margin: f64) -> V3 {
        let d = self.dir(p);
        let r = length(sub(p, self.frame.centre));
        let floor = self.radius(d) + margin;
        if r < floor { mad(self.frame.centre, d, floor) } else { p }
    }

    /// `push_out`, then the exact push out of the skin when the shell has the head's triangles.
    pub fn push_out_exact(&self, p: V3, margin: f64) -> V3 {
        let p = self.push_out(p, margin);
        match &self.skin {
            Some(skin) => skin.push_out(p, margin),
            None => p,
        }
    }

    /// The outward normal of the shell's level surfaces at a point: the gradient of `above_shell`, by central differences of 1 mm.
    pub fn normal(&self, p: V3) -> V3 {
        let h = 0.001;
        let g = [0, 1, 2].map(|k| {
            let mut a = p;
            let mut b = p;
            a[k] += h;
            b[k] -= h;
            self.above_shell(a) - self.above_shell(b)
        });
        let n = normalize(g);
        if length(n) > 0.0 { n } else { self.dir(p) }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A shell over a sphere head of radius 80 mm, with hair everywhere above the equator and 10 mm of volume.
    pub fn sphere_shell() -> Shell {
        let frame = HeadFrame { centre: [0.0, 0.0, 0.0], radius: 0.08 };
        let text = r##"{"name":"t","head":"h","colour":{"root":"#000000","tip":"#ffffff","painted":"#101010"},
            "hairline":[[0,0],[180,0]],"volume":{"base_mm":10,"taper_mm":5},"length":{"default_mm":100}}"##;
        let style = Style::from_json(text).expect("style").expanded();
        Shell::new(frame, RadialDepth::sphere([0.0; 3], 0.08, 2.0), &style)
    }

    #[test]
    fn the_shell_is_lifted_inside_the_hairline_and_flat_outside() {
        let s = sphere_shell();
        assert!((s.radius([0.0, 1.0, 0.0]) - (0.08 + MIN_LIFT + 0.010)).abs() < 1e-9);
        assert!((s.radius([0.0, -1.0, 0.0]) - (0.08 + MIN_LIFT)).abs() < 1e-9);
        let p = s.push_out([0.0, 0.05, 0.0], 0.001);
        assert!(s.above_shell(p) >= 0.001 - 1e-9);
    }
}
