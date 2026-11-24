//! The eyebrows, drawn from sliders on the head's own surface (paint track, part C, the brows).
//!
//! **Where the brow lies.** Each brow starts from three landmarks of `lines::BROW_ANCHORS`: its inner end, its highest point and its
//! outer end. On KOTOR's heads they run along the lower edge of the painted brow. A quadratic through them, laid on the skin, is the
//! brow's base line. The brow's own line is the base line moved up the skin by an offset that changes along the brow, from four
//! numbers: height, tilt, arch and where the arch peaks. Its thickness changes along the brow from three numbers: overall, outer against
//! inner, and middle against ends. Two more numbers move the inner and the outer end. This is the model of `scan/brows.py`, measured in
//! millimetres on the surface instead of on a front photo. The x < 0 brow is computed, and the x > 0 brow uses the same numbers on its
//! own landmarks, so the two are mirrored.
//!
//! **What the shader reads.** For each brow, 33 points of its middle line on the skin, with the distance along the brow, the direction
//! up the skin across the brow, and the half thickness. The shader finds a pixel's place along the brow and its signed distance from
//! the middle line, both in millimetres, and draws the hairs there (`head_paint_gpu/shaders/brows.wgsl`).
//!
//! **Two halves.** This file holds the processor's half: the sliders and the geometry. Everything per pixel is in the shader.

use crate::closest::closest_on_triangle;
use crate::folds::Anchors;
use crate::layout_points::{cross, dot, normalize, sub};
use crate::lines::BROW_ANCHORS;

/// The number of points of each brow's middle line.
pub const POINTS: usize = 33;

pub struct Slider {
    pub id: &'static str,
    pub label: &'static str,
    /// A smaller heading inside the group.
    pub part: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step: f64,
    pub unit: &'static str,
    pub help: &'static str,
}

/// The brow sliders, in the order of the fields of `Brows`.
pub const SLIDERS: [Slider; 29] = [
    Slider { id: "amount", label: "Amount", part: "Brow", min: 0.0, max: 1.0, default: 1.0, step: 0.01, unit: "", help: "How strongly the brows are painted. At 0 there are no brows. A low value gives faint, nearly absent brows." },
    Slider { id: "height_mm", label: "Height", part: "Line", min: -6.0, max: 8.0, default: 0.5, step: 0.05, unit: "mm", help: "How far above the brow ridge the middle of the brow runs, in millimetres along the skin." },
    Slider { id: "tilt_mm", label: "Tilt", part: "Line", min: -10.0, max: 6.0, default: -5.0, step: 0.05, unit: "mm", help: "How much higher the outer end is than the inner end, in millimetres. A negative value makes the outer end lower." },
    Slider { id: "arch_mm", label: "Arch", part: "Line", min: -6.0, max: 8.0, default: 1.5, step: 0.05, unit: "mm", help: "How much higher the middle of the brow is than a straight line between its ends, in millimetres. A negative value makes a brow that sags in the middle." },
    Slider { id: "peak_mm", label: "Arch peak", part: "Line", min: -4.0, max: 4.0, default: 0.8, step: 0.05, unit: "mm", help: "Moves the arch's highest point. A positive value moves it toward the temple and a negative value toward the nose." },
    Slider { id: "corner_mm", label: "Peak angle", part: "Line", min: -6.0, max: 10.0, default: 0.0, step: 0.05, unit: "mm", help: "Lifts the brow to a sharp angle at one point, by this many millimetres, with straight sides falling to both ends. KOTOR's men often have this angle. A negative value makes a dip there instead." },
    Slider { id: "corner_at", label: "Peak angle place", part: "Line", min: -0.8, max: 0.8, default: 0.35, step: 0.01, unit: "", help: "Where along the brow the peak angle sits, from -0.8 near the inner end to 0.8 near the outer end. At 0 it is in the middle." },
    Slider { id: "thick_mm", label: "Thickness", part: "Thickness", min: 0.5, max: 16.0, default: 6.0, step: 0.05, unit: "mm", help: "How thick the brow is in the middle, in millimetres across the brow." },
    Slider { id: "taper_mm", label: "Outer thickness", part: "Thickness", min: -8.0, max: 4.0, default: -1.5, step: 0.05, unit: "mm", help: "How much thicker the outer half of the brow is than the inner half, in millimetres. A negative value gives a brow that thins toward the temple." },
    Slider { id: "belly_mm", label: "Middle thickness", part: "Thickness", min: -4.0, max: 6.0, default: 0.5, step: 0.05, unit: "mm", help: "How much thicker the middle of the brow is than its ends, in millimetres." },
    Slider { id: "inner_mm", label: "Inner end", part: "Ends", min: -12.0, max: 12.0, default: 0.0, step: 0.1, unit: "mm", help: "Moves the brow's inner end toward the nose by this many millimetres, or away from it for a negative value." },
    Slider { id: "outer_mm", label: "Outer end", part: "Ends", min: -14.0, max: 12.0, default: 5.0, step: 0.1, unit: "mm", help: "Moves the brow's outer end toward the temple by this many millimetres, or back toward the nose for a negative value." },
    Slider { id: "tail", label: "Pointed tail", part: "Ends", min: 0.0, max: 1.0, default: 0.6, step: 0.01, unit: "", help: "How the outer end finishes. At 0 it is blunt. At 1 the brow narrows to a point over its last third." },
    Slider { id: "density", label: "Hair density", part: "Hairs", min: 0.0, max: 1.0, default: 0.9, step: 0.01, unit: "", help: "How many hairs grow in the brow. At 1 there are about 5 hairs per square millimetre in the middle of the brow. Raising it adds hairs and never moves the ones already there." },
    Slider { id: "length_mm", label: "Hair length", part: "Hairs", min: 1.5, max: 9.0, default: 3.6, step: 0.05, unit: "mm", help: "The typical length of a hair, in millimetres. Each hair is between 60% and 140% of it." },
    Slider { id: "hair_mm", label: "Hair thickness", part: "Hairs", min: 0.03, max: 0.3, default: 0.12, step: 0.005, unit: "mm", help: "How thick a hair is at its root, in millimetres. Real brow hairs are about 0.1 mm thick. Each hair thins toward its tip." },
    Slider { id: "angle_degrees", label: "Hair angle", part: "Hairs", min: 0.0, max: 45.0, default: 24.0, step: 0.5, unit: "degrees", help: "How steeply the hairs of the body of the brow cross it. The lower hairs grow up toward the temple and the upper hairs down toward it, at this angle to the brow's line." },
    Slider { id: "inner_up", label: "Upright inner hairs", part: "Hairs", min: 0.0, max: 0.5, default: 0.2, step: 0.01, unit: "", help: "How much of the brow, from its inner end, has hairs that grow straight up. It is a share of the brow's length. Groomed brows have more." },
    Slider { id: "messy", label: "Messiness", part: "Hairs", min: 0.0, max: 1.0, default: 0.4, step: 0.01, unit: "", help: "How much each hair strays from its neighbours' direction and how much it bends. At 0 the hairs lie neatly side by side." },
    Slider { id: "spill_mm", label: "Stray hairs", part: "Hairs", min: 0.0, max: 4.0, default: 0.6, step: 0.05, unit: "mm", help: "How far past the brow's edge hairs still grow, in millimetres. At 0 the brow has a clean edge. A high value gives a bushy, unkempt brow." },
    Slider { id: "trim", label: "Lower edge trim", part: "Grooming", min: 0.0, max: 1.0, default: 0.3, step: 0.01, unit: "", help: "How cleanly the lower edge is plucked. At 1 no hair crosses the lower edge." },
    Slider { id: "inner_full", label: "Inner end fullness", part: "Grooming", min: 0.0, max: 1.0, default: 0.6, step: 0.01, unit: "", help: "How full and dark the inner end is. At 1 it is as full as the middle. At 0 it is sparse and pale, and fills in over about a quarter of the brow." },
    Slider { id: "fill", label: "Fill", part: "Grooming", min: 0.0, max: 1.0, default: 0.75, step: 0.01, unit: "", help: "How much the skin between the hairs takes the brow's colour, as with a brow pencil or as seen from far away. At 0 only the hairs are painted." },
    Slider { id: "shadow", label: "Shadow", part: "Grooming", min: 0.0, max: 20.0, default: 4.0, step: 0.1, unit: "L*", help: "How much darker the skin under and around the brow is, in L*, as the hairs cast a soft shadow." },
    Slider { id: "lightness", label: "Lightness", part: "Colour", min: 3.0, max: 95.0, default: 18.0, step: 0.5, unit: "L*", help: "How light the hairs are, from black at 3 to white at 95. Blond and grey brows are light." },
    Slider { id: "hue", label: "Hue", part: "Colour", min: 0.0, max: 360.0, default: 60.0, step: 1.0, unit: "degrees", help: "The hairs' hue around the colour wheel. Brown and red hair is between 30 and 70 degrees." },
    Slider { id: "chroma", label: "Colour strength", part: "Colour", min: 0.0, max: 60.0, default: 12.0, step: 0.5, unit: "", help: "How strong the hairs' colour is. At 0 they are black, grey or white." },
    Slider { id: "tone_var", label: "Tone variation", part: "Colour", min: 0.0, max: 1.0, default: 0.5, step: 0.01, unit: "", help: "How much the hairs differ in lightness from each other. At 0 every hair has the same colour." },
    Slider { id: "grey", label: "Grey hairs", part: "Colour", min: 0.0, max: 1.0, default: 0.0, step: 0.01, unit: "", help: "The share of the hairs that are grey or white. At 1 every hair is white." },
];

/// The brow sliders. Their meaning, ranges and defaults are in `SLIDERS`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Brows {
    pub amount: f64,
    pub height_mm: f64,
    pub tilt_mm: f64,
    pub arch_mm: f64,
    pub peak_mm: f64,
    pub corner_mm: f64,
    pub corner_at: f64,
    pub thick_mm: f64,
    pub taper_mm: f64,
    pub belly_mm: f64,
    pub inner_mm: f64,
    pub outer_mm: f64,
    pub tail: f64,
    pub density: f64,
    pub length_mm: f64,
    pub hair_mm: f64,
    pub angle_degrees: f64,
    pub inner_up: f64,
    pub messy: f64,
    pub spill_mm: f64,
    pub trim: f64,
    pub inner_full: f64,
    pub fill: f64,
    pub shadow: f64,
    pub lightness: f64,
    pub hue: f64,
    pub chroma: f64,
    pub tone_var: f64,
    pub grey: f64,
}

impl Brows {
    /// Every slider at its default.
    pub const DEFAULT: Brows = Brows {
        amount: SLIDERS[0].default,
        height_mm: SLIDERS[1].default,
        tilt_mm: SLIDERS[2].default,
        arch_mm: SLIDERS[3].default,
        peak_mm: SLIDERS[4].default,
        corner_mm: SLIDERS[5].default,
        corner_at: SLIDERS[6].default,
        thick_mm: SLIDERS[7].default,
        taper_mm: SLIDERS[8].default,
        belly_mm: SLIDERS[9].default,
        inner_mm: SLIDERS[10].default,
        outer_mm: SLIDERS[11].default,
        tail: SLIDERS[12].default,
        density: SLIDERS[13].default,
        length_mm: SLIDERS[14].default,
        hair_mm: SLIDERS[15].default,
        angle_degrees: SLIDERS[16].default,
        inner_up: SLIDERS[17].default,
        messy: SLIDERS[18].default,
        spill_mm: SLIDERS[19].default,
        trim: SLIDERS[20].default,
        inner_full: SLIDERS[21].default,
        fill: SLIDERS[22].default,
        shadow: SLIDERS[23].default,
        lightness: SLIDERS[24].default,
        hue: SLIDERS[25].default,
        chroma: SLIDERS[26].default,
        tone_var: SLIDERS[27].default,
        grey: SLIDERS[28].default,
    };

    /// Brows that paint nothing.
    pub const NONE: Brows = Brows { amount: 0.0, ..Brows::DEFAULT };

    pub fn is_none(&self) -> bool {
        self.amount <= 0.0
    }

    pub fn values(&self) -> [f64; 29] {
        [self.amount, self.height_mm, self.tilt_mm, self.arch_mm, self.peak_mm, self.corner_mm, self.corner_at, self.thick_mm, self.taper_mm, self.belly_mm, self.inner_mm, self.outer_mm, self.tail, self.density, self.length_mm, self.hair_mm, self.angle_degrees, self.inner_up, self.messy, self.spill_mm, self.trim, self.inner_full, self.fill, self.shadow, self.lightness, self.hue, self.chroma, self.tone_var, self.grey]
    }

    pub fn field(&mut self, k: usize) -> &mut f64 {
        match k {
            0 => &mut self.amount,
            1 => &mut self.height_mm,
            2 => &mut self.tilt_mm,
            3 => &mut self.arch_mm,
            4 => &mut self.peak_mm,
            5 => &mut self.corner_mm,
            6 => &mut self.corner_at,
            7 => &mut self.thick_mm,
            8 => &mut self.taper_mm,
            9 => &mut self.belly_mm,
            10 => &mut self.inner_mm,
            11 => &mut self.outer_mm,
            12 => &mut self.tail,
            13 => &mut self.density,
            14 => &mut self.length_mm,
            15 => &mut self.hair_mm,
            16 => &mut self.angle_degrees,
            17 => &mut self.inner_up,
            18 => &mut self.messy,
            19 => &mut self.spill_mm,
            20 => &mut self.trim,
            21 => &mut self.inner_full,
            22 => &mut self.fill,
            23 => &mut self.shadow,
            24 => &mut self.lightness,
            25 => &mut self.hue,
            26 => &mut self.chroma,
            27 => &mut self.tone_var,
            _ => &mut self.grey,
        }
    }

    /// The offset of the brow's middle line from the base line, in millimetres, at `t` from -1 at the inner end to 1 at the outer end.
    pub fn offset(&self, t: f64) -> f64 {
        let c = self.corner_at.clamp(-0.9, 0.9);
        let tent = if t < c { (t + 1.0) / (c + 1.0) } else { (1.0 - t) / (1.0 - c) };
        self.height_mm + self.tilt_mm * 0.5 * t + self.arch_mm * (1.0 - t * t) + self.peak_mm * 1.3 * t * (1.0 - t * t) + self.corner_mm * tent.clamp(0.0, 1.0)
    }

    /// The brow's full thickness at `t`, before the ends are shaped, in millimetres.
    pub fn thickness(&self, t: f64) -> f64 {
        (self.thick_mm + self.taper_mm * 0.5 * t + self.belly_mm * (1.0 - t * t) - self.belly_mm).max(0.0)
    }
}

/// What the brows need of a head: the three landmarks of each brow, the face's directions, and the skin near the brows to lay lines on.
#[derive(Clone, Debug, PartialEq)]
pub struct BrowHead {
    /// The inner end, the highest point and the outer end of each brow, in millimetres; the x < 0 side first.
    pub anchors: [[[f64; 3]; 3]; 2],
    pub across: [f64; 3],
    pub up: [f64; 3],
    pub forward: [f64; 3],
    /// The point between the brows, in millimetres.
    pub middle: [f64; 3],
    /// Triangles of skin near the brows that face forward, in millimetres.
    triangles: Vec<[[f64; 3]; 3]>,
    /// Every triangle of skin near the brows, in millimetres, with its corners in the order that makes its normal face out of the head.
    skin: Vec<[[f64; 3]; 3]>,
}

/// One brow's middle line on the skin, ready for the shader.
#[derive(Clone, Debug, PartialEq)]
pub struct BrowLine {
    /// The points in millimetres, with the distance along the brow from its inner end in millimetres.
    pub points: Vec<([f64; 3], f64)>,
    /// At each point, the direction up the skin across the brow, and the half thickness in millimetres.
    pub across: Vec<([f64; 3], f64)>,
    /// The brow's length in millimetres.
    pub length: f64,
}

fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}

fn length(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl BrowHead {
    /// The brows of a head, from its mesh positions in metres. It is `None` for a mesh that does not hold the landmarks' vertices.
    pub fn new(positions: &[[f64; 3]], layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize]) -> Option<Self> {
        let b = &BROW_ANCHORS;
        if b.all().iter().any(|a| a.vertices.iter().any(|&v| v as usize >= positions.len())) {
            return None;
        }
        let pt = |a: &crate::folds::Anchor| Anchors::point(a, positions);
        let anchors = [0, 1].map(|s| [pt(&b.brow_inner[s]), pt(&b.brow_peak[s]), pt(&b.brow_outer[s])]);
        let middle = pt(&b.brow_middle);
        let marks = crate::folds::Landmarks::new(positions);
        let (across, up, forward) = (marks.frame.across, marks.frame.up, marks.frame.forward);
        let reach = 0.5 * length(sub(anchors[1][2], anchors[0][2])) + 25.0;
        let mut triangles = Vec::new();
        let mut skin = Vec::new();
        let mut outward = 0i64;
        for t in layout_triangles {
            let v = t.map(|i| positions[layout_to_mesh[i as usize]].map(|x| 1000.0 * x));
            let m = [0, 1, 2].map(|k| (v[0][k] + v[1][k] + v[2][k]) / 3.0);
            if length(sub(m, middle)) > reach {
                continue;
            }
            let n = cross(sub(v[1], v[0]), sub(v[2], v[0]));
            let l = length(n);
            if l < 1e-9 {
                continue;
            }
            skin.push(v);
            let facing = dot(n, forward) / l;
            if facing > 0.5 {
                outward += 1;
            } else if facing < -0.5 {
                outward -= 1;
            }
            if facing.abs() < 0.05 {
                continue;
            }
            triangles.push(v);
        }
        let order = |a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]| a.iter().flatten().map(|x| x.to_bits()).cmp(b.iter().flatten().map(|x| x.to_bits()));
        triangles.sort_by(order);
        triangles.dedup();
        if outward < 0 {
            skin.iter_mut().for_each(|t| t.swap(1, 2));
        }
        skin.sort_by(order);
        skin.dedup();
        Some(BrowHead { anchors, across, up, forward, middle, triangles, skin })
    }

    /// The skin seen along `forward` through a point, nearest to the point in depth, with the skin's normal facing forward. `None` when no
    /// skin lies within 20 mm of it in depth.
    pub fn on_skin(&self, p: [f64; 3]) -> Option<([f64; 3], [f64; 3])> {
        let f = self.forward;
        let mut best: Option<(f64, [f64; 3], [f64; 3])> = None;
        for t in &self.triangles {
            let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
            let h = cross(f, e2);
            let a = dot(e1, h);
            if a.abs() < 1e-12 {
                continue;
            }
            let s = sub(p, t[0]);
            let u = dot(s, h) / a;
            if !(-1e-9..=1.0 + 1e-9).contains(&u) {
                continue;
            }
            let q = cross(s, e1);
            let v = dot(f, q) / a;
            if v < -1e-9 || u + v > 1.0 + 1e-9 {
                continue;
            }
            let depth = dot(e2, q) / a;
            if depth.abs() > 20.0 || best.as_ref().is_some_and(|b| b.0 <= depth.abs()) {
                continue;
            }
            let mut n = normalize(cross(e1, e2));
            if dot(n, f) < 0.0 {
                n = n.map(|x| -x);
            }
            best = Some((depth.abs(), add(p, f, depth), n));
        }
        best.map(|b| (b.1, b.2))
    }

    /// The closest point of the skin near the brows to a point, with the skin's outward normal there.
    fn nearest_skin(&self, p: [f64; 3]) -> Option<([f64; 3], [f64; 3])> {
        let mut best: Option<(f64, [f64; 3], [f64; 3])> = None;
        for t in &self.skin {
            let w = closest_on_triangle(p, t[0], t[1], t[2]);
            let q = [0, 1, 2].map(|k| w[0] * t[0][k] + w[1] * t[1][k] + w[2] * t[2][k]);
            let d = length(sub(p, q));
            if best.as_ref().is_none_or(|b| d < b.0) {
                best = Some((d, q, normalize(cross(sub(t[1], t[0]), sub(t[2], t[0])))));
            }
        }
        best.map(|b| (b.1, b.2))
    }

    /// The point `mm` millimetres up the skin from a point on the skin, walking in steps of 0.5 mm along the face's up direction. Each
    /// step goes back to the closest point of the skin rather than along the face's forward direction, so that a walk down the
    /// underside of the brow ridge, which the forward direction only grazes, stays on the ridge instead of jumping to the upper lid.
    pub fn walk_up(&self, start: [f64; 3], mm: f64) -> [f64; 3] {
        let steps = (mm.abs() / 0.5).ceil() as usize;
        let Some((mut p, mut n)) = self.nearest_skin(start) else { return start };
        let mut left = mm.abs();
        for _ in 0..steps {
            let dir = normalize(add(self.up, n, -dot(self.up, n))).map(|x| x * mm.signum());
            let step = left.min(0.5);
            let Some(next) = self.nearest_skin(add(p, dir, step)) else { break };
            (p, n) = next;
            left -= step;
        }
        p
    }

    /// The base line of one brow: the quadratic through its three landmarks, laid on the skin, from `t0` to `t1` where 0 is the inner
    /// landmark and 1 the outer one, every 0.01.
    fn base(&self, side: usize, t0: f64, t1: f64) -> Vec<[f64; 3]> {
        let p = self.anchors[side];
        let a = length(sub(p[1], p[0]));
        let s = (a / (a + length(sub(p[2], p[1])))).clamp(0.2, 0.8);
        let n = ((t1 - t0) / 0.01).round() as usize;
        (0..=n)
            .map(|i| {
                let t = t0 + (t1 - t0) * i as f64 / n as f64;
                // Past the landmarks the line goes on straight along the quadratic's direction at its end, so that a longer brow does
                // not curve down toward the nose or the cheek.
                let tc = t.clamp(0.0, 1.0);
                let at = |t: f64| {
                    let l0 = (t - s) * (t - 1.0) / s;
                    let l1 = t * (t - 1.0) / (s * (s - 1.0));
                    let l2 = t * (t - s) / (1.0 - s);
                    [0, 1, 2].map(|k| l0 * p[0][k] + l1 * p[1][k] + l2 * p[2][k])
                };
                let e = at(tc);
                let slope = [0, 1, 2].map(|k| (at(tc + 1e-3)[k] - at(tc - 1e-3)[k]) / 2e-3);
                let q = [0, 1, 2].map(|k| e[k] + (t - tc) * slope[k]);
                self.on_skin(q).map_or(q, |s| s.0)
            })
            .collect()
    }

    /// The base line of one brow sampled every 0.01 of the landmarks' span from -0.5 to 1.5, with the distance along it in millimetres
    /// from the inner landmark, and the distance from the inner to the outer landmark. The brow's ends and the measure use these.
    pub fn base_line(&self, side: usize) -> (Vec<[f64; 3]>, Vec<f64>, f64) {
        let base = self.base(side, -0.5, 1.5);
        let mut dist = vec![0.0; base.len()];
        for i in 1..base.len() {
            dist[i] = dist[i - 1] + length(sub(base[i], base[i - 1]));
        }
        let zero = dist[50];
        let full = dist[150] - zero;
        (base, dist.iter().map(|d| d - zero).collect(), full)
    }

    /// The point of a base line at a distance `s` along it from the inner landmark, as `base_line` gives it.
    pub fn base_at(base: &[[f64; 3]], dist: &[f64], s: f64) -> [f64; 3] {
        let d = s.clamp(dist[0], dist[dist.len() - 1]);
        let i = dist.partition_point(|&x| x < d).clamp(1, dist.len() - 1);
        let f = ((d - dist[i - 1]) / (dist[i] - dist[i - 1]).max(1e-9)).clamp(0.0, 1.0);
        [0, 1, 2].map(|k| base[i - 1][k] + f * (base[i][k] - base[i - 1][k]))
    }

    /// One brow's middle line for some slider values.
    pub fn line(&self, brows: &Brows, side: usize) -> BrowLine {
        // The base line from half its length before the inner landmark to half its length past the outer one, with the distance along it
        // measured from the inner landmark.
        let base = self.base(side, -0.5, 1.5);
        let mut dist = vec![0.0; base.len()];
        for i in 1..base.len() {
            dist[i] = dist[i - 1] + length(sub(base[i], base[i - 1]));
        }
        let zero = dist[50];
        let full = dist[150] - zero;
        let at = |s: f64| {
            let d = (s + zero).clamp(0.0, dist[dist.len() - 1]);
            let i = dist.partition_point(|&x| x < d).clamp(1, dist.len() - 1);
            let f = ((d - dist[i - 1]) / (dist[i] - dist[i - 1]).max(1e-9)).clamp(0.0, 1.0);
            [0, 1, 2].map(|k| base[i - 1][k] + f * (base[i][k] - base[i - 1][k]))
        };
        let start = -brows.inner_mm.min(0.45 * full + 6.0);
        let end = full + brows.outer_mm.max(-0.6 * full);
        let middle: Vec<[f64; 3]> = (0..POINTS)
            .map(|i| {
                let t = -1.0 + 2.0 * i as f64 / (POINTS - 1) as f64;
                let s = start + (end - start) * (t + 1.0) / 2.0;
                self.walk_up(at(s), brows.offset(t))
            })
            .collect();
        let mut points = Vec::with_capacity(POINTS);
        let mut s = 0.0;
        for i in 0..POINTS {
            if i > 0 {
                s += length(sub(middle[i], middle[i - 1]));
            }
            points.push((middle[i], s));
        }
        let total = s;
        let across = (0..POINTS)
            .map(|i| {
                let t = -1.0 + 2.0 * i as f64 / (POINTS - 1) as f64;
                let tangent = normalize(sub(middle[(i + 1).min(POINTS - 1)], middle[i.saturating_sub(1)]));
                let n = self.on_skin(middle[i]).map_or(self.forward, |x| x.1);
                let mut up = normalize(cross(n, tangent));
                if dot(up, self.up) < 0.0 {
                    up = up.map(|x| -x);
                }
                let along = points[i].1;
                // The inner end is rounded over its first 2.5 mm; the outer end narrows over the tail's share of the last third.
                let inner = (smoothstep(-0.4, 2.5, along)).sqrt();
                let tail_len = (0.33 * total * brows.tail).max(1.5);
                let outer = smoothstep(total + 0.4, total - tail_len, along).powf(0.5 + 0.7 * brows.tail);
                (up, 0.5 * brows.thickness(t) * inner * outer)
            })
            .collect();
        BrowLine { points, across, length: total }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sliders_match_fields() {
        let mut b = Brows::DEFAULT;
        for (k, s) in SLIDERS.iter().enumerate() {
            assert_eq!(b.values()[k], s.default, "{}", s.id);
            *b.field(k) = s.max;
            assert_eq!(b.values()[k], s.max, "{}", s.id);
            assert!(s.min <= s.default && s.default <= s.max, "{}", s.id);
        }
    }
}
