//! The line generator: the forehead lines, grown from the head (paint track, part C, fine detail).
//!
//! **Where the lines run.** The forehead gets its own frame from six landmarks (`BROW_ANCHORS`): the inner end, the highest point and the
//! outer end of each brow, the point between the brows, the middle of the forehead and the middle of the hairline. `a` is the distance
//! across the face from the middle line, and `v` the distance up the forehead from the point between the brows, measured along the
//! forehead's own slope. Each brow is a smooth curve through its landmarks. A line sits at a fixed share of the height between the brow
//! curve and the hairline, so the lowest lines bend with the brows and dip between them, and the highest lines run nearly straight under
//! the hairline. Each landmark is three mesh vertices and their weights, so the lines follow any head, and a small change of the head
//! gives a small change of the lines.
//!
//! **What the style sliders do.** The spacing sets how far apart the lines are, from 12 mm to 3.5 mm at the middle of the forehead. The
//! continuity cuts each line into pieces, from one piece across the whole forehead to pieces about 18 mm long, with gaps between them and
//! some pieces missing. The waviness moves each line up and down by a slow noise along it. The variation picks another arrangement of the
//! same style. All the lines fade out before the brows, the hairline and the temples.
//!
//! **What the amount does.** Every line has a place in a fixed order, which only the style sliders and the variation decide. The lines
//! nearest the lower middle of the forehead come first. Raising the amount reveals the lines in that order, and deepens and lengthens the
//! lines already shown. It never moves a line or changes the order. At an amount of 0.15 a face shows one or two faint, short lines.
//!
//! **The shape KOTOR gives them.** Seen from the front on Kreia, old_w_m, old_a_f and Jolee, KOTOR's forehead lines dip down at the
//! middle line, between the brows, in a wide U. The dip slider bends the lines down there, most for the lowest lines.
//!
//! **The cross-section.** KOTOR paints each forehead line as a soft dark valley, often with a lighter band just below it, as if light
//! from above fell on the bulge of skin under the crease. So a line here is a dark valley, and the ridge slider adds a light band whose
//! middle lies two half widths below the valley's middle. Both falloffs are `folds::profile`: exp(-(d / w)^1.6), scaled to be one half
//! at the half width, and zero beyond 3.5 half widths.
//!
//! **Two stages.** `LineField::new` is the geometry stage. It runs once per head and per slider change, grows the lines, and ends with
//! a plain list of segments in the forehead's (a, v) plane, every 0.5 mm along each line, each end with its strength in L*. The pieces,
//! the waviness, the dip, the fades and every yes-or-no choice are settled there. The per-pixel stage is the GPU painter's shader. It
//! projects a point onto the forehead's plane, finds the nearest segment of each line in a 2 mm grid, and applies the cross-section.
//!
//! **Determinism.** The same values on the same head always give the same pixels. Every yes-or-no choice, such as whether a piece of a
//! line is dropped, comes from a whole-number hash (`noise::hash3`). The code has no threads, clocks or file reading.
//!
//! # The sliders
//!
//! `SLIDERS` holds the same list for the live page.
//!
//! | Slider | Range | Default | What it does |
//! |---|---|---|---|
//! | `amount` | 0 to 1 | 0 | How many forehead lines show, and how deep and long they are. At 0 there are no lines. At 1 every line shows at full depth and length. |
//! | `spacing` | 0 to 1 | 0.5 | How close together the lines are. At 0 a few lines lie 12 mm apart. At 1 many lines lie 3.5 mm apart. |
//! | `continuity` | 0 to 1 | 0.3 | How broken the lines are. At 0 each line runs unbroken across the forehead. At 1 each line breaks into pieces about 18 mm long, with gaps between them, and 3 in 10 pieces are missing. |
//! | `waviness` | 0 to 1 | 0.3 | How wavy the lines are. At 0 they are smooth curves. At 1 they wander up and down by about 2 mm. |
//! | `variation` | 0 to 99 | 0 | Picks another arrangement of the same style: where each line starts, ends and breaks. It is a whole number. |
//! | `depth` | 0 to 20 L* | 6 | How much darker the middle of a line is at full amount, in L*. |
//! | `sharpness` | 0 to 1 | 0.5 | How narrow the lines are. At 0 a line falls to half its darkness 2 mm from its middle. At 0.5 this is 0.9 mm, and at 1 it is 0.4 mm. |
//! | `ridge` | 0 to 1 | 0.3 | How much lighter the band just below each line is, as a share of the line's depth. At 0 there is no light band. |
//! | `dip_mm` | 0 to 10 mm | 3 | How far the lowest line dips down at the middle line, between the brows. Higher lines dip less. |
//! | `reach` | 0.2 to 1 | 0.7 | How far up the forehead the lines go, as a share of the height between the lowest line and the hairline. |

use crate::folds::{Anchor, Anchors};
use crate::layout_points::{cross, dot, normalize, sub};
use crate::noise::{hash3, line_noise, unit};
use crate::wrinkles::smoothstep;
use std::collections::HashMap;

/// The forehead's landmarks. A pair holds the point on the x < 0 side first, then its mirror on the x > 0 side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrowAnchors {
    pub brow_inner: [Anchor; 2],
    pub brow_peak: [Anchor; 2],
    pub brow_outer: [Anchor; 2],
    pub brow_middle: Anchor,
    pub forehead_middle: Anchor,
    pub hairline_middle: Anchor,
}

/// Made by `src/bin/line_anchors.rs` from layout pixels on the base head.
pub const BROW_ANCHORS: BrowAnchors = BrowAnchors {
    brow_inner: [Anchor { vertices: [219, 191, 194], weights: [0.26327, 0.16052, 0.57621] }, Anchor { vertices: [2623, 2596, 2595], weights: [0.21571, 0.70477, 0.07952] }],
    brow_peak: [Anchor { vertices: [226, 200, 195], weights: [0.58802, 0.02514, 0.38685] }, Anchor { vertices: [2630, 2599, 2602], weights: [0.42988, 0.41007, 0.16005] }],
    brow_outer: [Anchor { vertices: [231, 260, 217], weights: [0.76587, 0.12404, 0.11009] }, Anchor { vertices: [2634, 2621, 2660], weights: [0.60666, 0.10951, 0.28383] }],
    brow_middle: Anchor { vertices: [235, 230, 236], weights: [0.04902, 0.72219, 0.22879] },
    forehead_middle: Anchor { vertices: [245, 246, 1790], weights: [0.04434, 0.50828, 0.44738] },
    hairline_middle: Anchor { vertices: [1819, 1818, 1933], weights: [0.02653, 0.30363, 0.66984] },
};

impl BrowAnchors {
    pub fn all(&self) -> Vec<Anchor> {
        let mut v = Vec::new();
        for pair in [self.brow_inner, self.brow_peak, self.brow_outer] {
            v.extend(pair);
        }
        v.extend([self.brow_middle, self.forehead_middle, self.hairline_middle]);
        v
    }
}

/// One slider as the live page shows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slider {
    pub id: &'static str,
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub help: &'static str,
}

/// The forehead lines' sliders, in the order of the table above.
pub const SLIDERS: [Slider; 10] = [
    Slider { id: "amount", label: "Forehead lines", min: 0.0, max: 1.0, default: 0.0, help: "How many forehead lines show, and how deep and long they are. Raising it adds lines in a fixed order and never moves the ones already there." },
    Slider { id: "spacing", label: "Line spacing", min: 0.0, max: 1.0, default: 0.5, help: "At the left, a few lines lie 12 mm apart. At the right, many lines lie 3.5 mm apart." },
    Slider { id: "continuity", label: "Broken lines", min: 0.0, max: 1.0, default: 0.3, help: "At the left, each line runs unbroken across the forehead. At the right, each line breaks into short pieces and some pieces are missing." },
    Slider { id: "waviness", label: "Waviness", min: 0.0, max: 1.0, default: 0.3, help: "At the left, the lines are smooth curves. At the right, they wander up and down by about 2 mm." },
    Slider { id: "variation", label: "Variation", min: 0.0, max: 99.0, default: 0.0, help: "Picks another arrangement of the same style, with lines that start, end and break in other places." },
    Slider { id: "depth", label: "Line depth", min: 0.0, max: 20.0, default: 6.0, help: "How much darker the middle of a line is at full amount, in L*." },
    Slider { id: "sharpness", label: "Line sharpness", min: 0.0, max: 1.0, default: 0.5, help: "How narrow the lines are. At the left a line is a soft band 4 mm wide. At the right it is a sharp crease under 1 mm wide." },
    Slider { id: "ridge", label: "Light under lines", min: 0.0, max: 1.0, default: 0.3, help: "How much lighter the skin just below each line is, as a share of the line's darkness." },
    Slider { id: "dip_mm", label: "Dip between brows", min: 0.0, max: 10.0, default: 3.0, help: "How far the lowest line dips down between the brows, in millimetres. Higher lines dip less." },
    Slider { id: "reach", label: "Lines reach up", min: 0.2, max: 1.0, default: 0.7, help: "How far up the forehead the lines go. At the left they stay just above the brows. At the right they reach the hairline." },
];

/// The forehead lines' sliders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForeheadLines {
    pub amount: f64,
    pub spacing: f64,
    pub continuity: f64,
    pub waviness: f64,
    pub variation: u32,
    pub depth: f64,
    pub sharpness: f64,
    pub ridge: f64,
    pub dip_mm: f64,
    pub reach: f64,
}

impl ForeheadLines {
    /// No lines, with every style slider at its default.
    pub const NONE: ForeheadLines = ForeheadLines { amount: 0.0, spacing: 0.5, continuity: 0.3, waviness: 0.3, variation: 0, depth: 6.0, sharpness: 0.5, ridge: 0.3, dip_mm: 3.0, reach: 0.7 };

    /// Whether the lines paint nothing.
    pub fn is_none(&self) -> bool {
        self.amount <= 0.0 || self.depth <= 0.0
    }

    /// The distance from a line's middle at which it is half as dark, in millimetres.
    pub fn half_width_mm(&self) -> f64 {
        2.0 * 0.2f64.powf(self.sharpness.clamp(0.0, 1.0))
    }
}

/// A brow as a smooth curve of the height `v` against the distance `x` = |a| from the middle line: through the point between the brows,
/// the brow's inner end, its highest point and its outer end. It is flat at the middle line and at the highest point, and goes on in a
/// straight line past the outer end.
#[derive(Clone, Copy, Debug, PartialEq)]
struct BrowCurve {
    x: [f64; 4],
    v: [f64; 4],
    slope: [f64; 4],
}

impl BrowCurve {
    fn new(middle: f64, inner: [f64; 2], peak: [f64; 2], outer: [f64; 2]) -> Self {
        let x1 = inner[0].max(1.0);
        let x2 = peak[0].max(x1 + 1.0);
        let x3 = outer[0].max(x2 + 1.0);
        let x = [0.0, x1, x2, x3];
        let v = [middle, inner[1], peak[1], outer[1]];
        let (s01, s12) = ((v[1] - v[0]) / x1, (v[2] - v[1]) / (x2 - x1));
        let inner_slope = if s01 * s12 <= 0.0 { 0.0 } else { 2.0 / (1.0 / s01 + 1.0 / s12) };
        BrowCurve { x, v, slope: [0.0, inner_slope, 0.0, (v[3] - v[2]) / (x3 - x2)] }
    }

    fn at(&self, x: f64) -> f64 {
        let x = x.abs();
        if x >= self.x[3] {
            return self.v[3] + self.slope[3] * (x - self.x[3]);
        }
        let i = if x < self.x[1] { 0 } else if x < self.x[2] { 1 } else { 2 };
        let h = self.x[i + 1] - self.x[i];
        let t = (x - self.x[i]) / h;
        let (t2, t3) = (t * t, t * t * t);
        (2.0 * t3 - 3.0 * t2 + 1.0) * self.v[i] + (t3 - 2.0 * t2 + t) * h * self.slope[i] + (-2.0 * t3 + 3.0 * t2) * self.v[i + 1] + (t3 - t2) * h * self.slope[i + 1]
    }
}

/// What the forehead lines need of a head: the forehead's frame and its brow curves, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForeheadHead {
    pub origin: [f64; 3],
    /// From the x < 0 side to the x > 0 side.
    pub across: [f64; 3],
    /// Up the forehead's slope, from the point between the brows toward the middle of the hairline.
    pub up: [f64; 3],
    pub forward: [f64; 3],
    brows: [BrowCurve; 2],
    /// The distance of each brow's inner and outer end from the middle line.
    pub brow_inner_a: [f64; 2],
    pub brow_outer_a: [f64; 2],
    pub hairline_v: f64,
    /// The depth of the forehead's middle along `forward`.
    pub middle_depth: f64,
}

impl ForeheadHead {
    /// The forehead of a head, from its mesh positions in metres. It is `None` for a mesh that does not hold the landmarks' vertices.
    pub fn new(positions: &[[f64; 3]]) -> Option<Self> {
        let b = &BROW_ANCHORS;
        if b.all().iter().any(|a| a.vertices.iter().any(|&v| v as usize >= positions.len())) {
            return None;
        }
        let pt = |a: &Anchor| Anchors::point(a, positions);
        let pair = |p: &[Anchor; 2]| [pt(&p[0]), pt(&p[1])];
        let (inner, peak, outer) = (pair(&b.brow_inner), pair(&b.brow_peak), pair(&b.brow_outer));
        let origin = pt(&b.brow_middle);
        // The brows' peaks are left out, so that a raised brow bends the lines over it without tilting the whole frame.
        let left = [0, 1, 2].map(|k| inner[0][k] + outer[0][k]);
        let right = [0, 1, 2].map(|k| inner[1][k] + outer[1][k]);
        let across = normalize(sub(right, left));
        let rise = sub(pt(&b.hairline_middle), origin);
        let up = normalize(sub(rise, across.map(|x| x * dot(rise, across))));
        let forward = cross(across, up);
        let local = |p: [f64; 3]| {
            let r = sub(p, origin);
            [dot(r, across), dot(r, up)]
        };
        let brows = [0, 1].map(|s| {
            let (i, p, o) = (local(inner[s]), local(peak[s]), local(outer[s]));
            BrowCurve::new(0.0, [i[0].abs(), i[1]], [p[0].abs(), p[1]], [o[0].abs(), o[1]])
        });
        Some(ForeheadHead {
            origin,
            across,
            up,
            forward,
            brows,
            brow_inner_a: [local(inner[0])[0].abs(), local(inner[1])[0].abs()],
            brow_outer_a: [local(outer[0])[0].abs(), local(outer[1])[0].abs()],
            hairline_v: local(pt(&b.hairline_middle))[1],
            middle_depth: dot(sub(pt(&b.forehead_middle), origin), forward),
        })
    }

    /// The height of the brows' curve at a distance `a` across, blended between the two sides over the 20 mm around the middle line.
    pub fn brow_v(&self, a: f64) -> f64 {
        let w = smoothstep(-10.0, 10.0, a);
        (1.0 - w) * self.brows[0].at(a) + w * self.brows[1].at(a)
    }

    /// The outer end of the brow on the side of `a`, blended in the same way.
    fn outer_a(&self, a: f64) -> f64 {
        let w = smoothstep(-10.0, 10.0, a);
        (1.0 - w) * self.brow_outer_a[0] + w * self.brow_outer_a[1]
    }
}

/// The lowest line's height above the brow curve, at the middle line, in millimetres, before the spacing is added.
const LOWEST_MM: f64 = 7.0;
/// The top of the band of lines, below the hairline, in millimetres.
const TOP_MARGIN_MM: f64 = 5.0;

/// One forehead line, ready to be drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Line {
    /// The line's height as a share of the height between the brow curve and the hairline.
    share: f64,
    /// Its depth at this amount, in L*.
    depth: f64,
    /// The middle of the line across the face, and how far it reaches either side of it, in millimetres.
    centre_a: f64,
    half_length: f64,
    key: i64,
}

/// The forehead lines of one set of sliders on one head, ready to be read at any point.
#[derive(Clone, Debug, PartialEq)]
pub struct LineField {
    head: ForeheadHead,
    lines: Vec<Line>,
    half_mm: f64,
    piece_mm: f64,
    drop: f64,
    gap_share: f64,
    break_offset_mm: f64,
    wave_mm: f64,
    ridge: f64,
    dip_mm: f64,
    /// The half width of the dip across the face, in millimetres.
    dip_half_mm: f64,
    seed: u64,
    /// The geometry stage's result: the segments, and the 2 mm cells of the (a, v) plane that each segment reaches.
    segments: Vec<Segment>,
    cells: HashMap<[i32; 2], Vec<u32>>,
}

/// One drawn segment of a line in the forehead's (a, v) plane, in millimetres, with the strength at each end in L*: how much darker
/// the valley's middle is there, after the line's depth, its length, its pieces and the fades.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub a: [f64; 2],
    pub b: [f64; 2],
    pub strength: [f64; 2],
    pub line: u32,
}

/// The spacing of the segments' ends along a line, in millimetres, and the size of the grid's cells.
const SEGMENT_MM: f64 = 0.5;
const CELL_MM: f64 = 2.0;

fn cell_of(a: f64, v: f64) -> [i32; 2] {
    [(a / CELL_MM).floor() as i32, (v / CELL_MM).floor() as i32]
}

impl LineField {
    pub fn new(sliders: &ForeheadLines, head: &ForeheadHead) -> Self {
        let s = ForeheadLines {
            amount: sliders.amount.clamp(0.0, 1.0),
            spacing: sliders.spacing.clamp(0.0, 1.0),
            continuity: sliders.continuity.clamp(0.0, 1.0),
            waviness: sliders.waviness.clamp(0.0, 1.0),
            variation: sliders.variation,
            depth: sliders.depth.clamp(0.0, 20.0),
            sharpness: sliders.sharpness,
            ridge: sliders.ridge.clamp(0.0, 1.0),
            dip_mm: sliders.dip_mm.clamp(0.0, 10.0),
            reach: sliders.reach.clamp(0.2, 1.0),
        };
        let seed = hash3(i64::from(s.variation), 0x11e5, 0, 0xF0E1);
        let c = s.continuity;
        let mut field = LineField {
            head: *head,
            lines: Vec::new(),
            half_mm: s.half_width_mm(),
            // From one piece of 1,000 mm, which never breaks on a forehead, to pieces of 18 mm.
            piece_mm: 18.0 * (1000.0f64 / 18.0).powf(1.0 - c),
            drop: 0.3 * c,
            gap_share: 0.35 * c,
            break_offset_mm: 1.2 * c,
            wave_mm: 5.0 * s.waviness,
            ridge: s.ridge,
            dip_mm: s.dip_mm,
            dip_half_mm: 2.2 * 0.5 * (head.brow_inner_a[0] + head.brow_inner_a[1]),
            seed,
            segments: Vec::new(),
            cells: HashMap::new(),
        };
        if s.is_none() {
            return field;
        }
        let height = head.hairline_v - head.brow_v(0.0);
        let spacing_mm = 12.0 + (3.5 - 12.0) * s.spacing;
        // The lines' heights above the brows at the middle line, each moved by up to a fifth of the spacing.
        let mut heights = Vec::new();
        let mut k = 0i64;
        loop {
            let jitter = (unit(hash3(k, 1, 0, seed), 0) - 0.5) * 0.4 * spacing_mm;
            let y = LOWEST_MM + 0.5 * spacing_mm + k as f64 * spacing_mm + jitter;
            if y > LOWEST_MM + s.reach * (height - TOP_MARGIN_MM - LOWEST_MM) || k > 40 {
                break;
            }
            heights.push((k, y));
            k += 1;
        }
        // The order in which the lines appear: the lines nearest 40% of the band's height come first, with a random share.
        let band = LOWEST_MM + s.reach * (height - TOP_MARGIN_MM - LOWEST_MM);
        let mut order: Vec<(f64, usize)> = heights
            .iter()
            .enumerate()
            .map(|(i, &(k, y))| (0.65 * (y - 0.4 * band).abs() / (0.5 * band) + 0.35 * unit(hash3(k, 2, 0, seed), 0), i))
            .collect();
        order.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let n = heights.len().max(1) as f64;
        let reach = 0.5 * (head.brow_outer_a[0] + head.brow_outer_a[1]);
        for (rank, &(_, i)) in order.iter().enumerate() {
            let (k, y) = heights[i];
            let h = hash3(k, 3, 0, seed);
            let start = 0.8 * rank as f64 / n;
            let grown = smoothstep(start, start + 0.2, s.amount);
            if grown == 0.0 {
                continue;
            }
            let depth = s.depth * grown * (0.35 + 0.65 * s.amount) * (0.7 + 0.3 * unit(h, 0));
            let full = reach * (0.8 + 0.4 * unit(h, 16));
            field.lines.push(Line {
                share: y / height,
                depth,
                centre_a: (unit(h, 32) - 0.5) * 0.3 * reach,
                half_length: full * (0.35 + 0.65 * grown),
                key: k,
            });
        }
        field.grow_segments();
        field
    }

    /// The geometry stage's last step: each line sampled every `SEGMENT_MM` across the forehead, with its strength at each sample.
    fn grow_segments(&mut self) {
        let head = self.head;
        let top = head.hairline_v;
        let limit = 1.2 * head.brow_outer_a[0].max(head.brow_outer_a[1]);
        let steps = (2.0 * limit / SEGMENT_MM).ceil() as i64;
        let reach = 5.5 * self.half_mm;
        for (id, line) in self.lines.iter().enumerate() {
            let mut previous: Option<([f64; 2], f64, i64)> = None;
            for i in 0..=steps {
                let a = -limit + i as f64 * SEGMENT_MM;
                let outer = head.outer_a(a);
                let b = head.brow_v(a);
                let (show, offset, piece) = self.piece(line.key, a);
                let v = b + line.share * (top - b) - self.dip(line.share, a) + self.wave(line.key, a) + offset;
                let length = 1.0 - smoothstep(line.half_length - 10.0, line.half_length, (a - line.centre_a).abs());
                let fades = smoothstep(b + 3.0, b + 7.0, v) * (1.0 - smoothstep(top - 7.0, top - 1.0, v)) * (1.0 - smoothstep(0.9 * outer, 1.2 * outer, a.abs()));
                let varied = 1.0 + 0.4 * line_noise(a / 10.0, self.seed ^ (line.key as u64).wrapping_mul(131) ^ 0x3C);
                let strength = line.depth * length * show * fades * varied;
                if let Some((p, w, j)) = previous
                    && j == piece
                    && (w > 0.0 || strength > 0.0)
                {
                    let n = self.segments.len() as u32;
                    self.segments.push(Segment { a: p, b: [a, v], strength: [w, strength], line: id as u32 });
                    let lo = cell_of(p[0].min(a) - reach, p[1].min(v) - reach);
                    let hi = cell_of(p[0].max(a) + reach, p[1].max(v) + reach);
                    for x in lo[0]..=hi[0] {
                        for y in lo[1]..=hi[1] {
                            self.cells.entry([x, y]).or_default().push(n);
                        }
                    }
                }
                previous = Some(([a, v], strength, piece));
            }
        }
    }

    /// The geometry stage's segments.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// How many lines are drawn.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// The height `v` of a drawn line's middle at a distance `a` across, in the forehead's frame, before the pieces' offsets.
    pub fn centre_v(&self, line: usize, a: f64) -> f64 {
        let l = &self.lines[line];
        let b = self.head.brow_v(a);
        b + l.share * (self.head.hairline_v - b) - self.dip(l.share, a) + self.wave(l.key, a)
    }

    /// How far a line at a share of the forehead's height dips down at a distance `a` across.
    fn dip(&self, share: f64, a: f64) -> f64 {
        self.dip_mm * (1.0 - share) * (1.0 - smoothstep(0.0, self.dip_half_mm, a.abs()))
    }

    fn wave(&self, key: i64, a: f64) -> f64 {
        if self.wave_mm == 0.0 {
            return 0.0;
        }
        let s = self.seed ^ (key as u64).wrapping_mul(0x9E37_79B9);
        self.wave_mm * (0.75 * line_noise(a / 14.0, s) + 0.25 * line_noise(a / 5.0, s ^ 0x77))
    }

    /// How much of a line shows at `a`, from its pieces, the piece's offset up the forehead, and the piece's number.
    fn piece(&self, key: i64, a: f64) -> (f64, f64, i64) {
        let p = self.piece_mm;
        let phase = p * (0.25 + 0.5 * unit(hash3(key, 4, 0, self.seed), 0));
        let x = a - phase;
        let j = (x / p).floor();
        let t = x - j * p;
        let h = hash3(key, j as i64, 5, self.seed);
        if unit(h, 0) < self.drop {
            return (0.0, 0.0, j as i64);
        }
        let gap = self.gap_share * p;
        let (start, end) = (0.5 * gap * unit(h, 16), p - 0.5 * gap * unit(h, 32));
        let fade = (3.0 + self.half_mm).min(0.3 * (end - start));
        let show = if gap == 0.0 && self.drop == 0.0 { 1.0 } else { smoothstep(start, start + fade, t) * (1.0 - smoothstep(end - fade, end, t)) };
        let strength = 1.0 - 0.35 * self.drop / 0.3 * unit(h, 48);
        (show * strength, self.break_offset_mm * (unit(hash3(key, j as i64, 6, self.seed), 0) - 0.5) * 2.0, j as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mesh big enough for the anchors' vertex numbers, holding a flat forehead in the plane z = 100 mm, 200 mm wide and 120 mm tall,
    /// with every landmark's three vertices placed where a forehead has them. The grid's vertices come after the anchors' vertices.
    fn test_head(nudge: impl Fn(&mut Vec<[f64; 3]>)) -> Vec<[f64; 3]> {
        let top = BROW_ANCHORS.all().iter().flat_map(|a| a.vertices).max().unwrap() as usize + 1;
        let mut positions = vec![[0.0, 0.0, 0.1]; top];
        let b = &BROW_ANCHORS;
        let set = |positions: &mut Vec<[f64; 3]>, anchor: &Anchor, x: f64, y: f64| {
            for &v in &anchor.vertices {
                positions[v as usize] = [x / 1000.0, y / 1000.0, 0.1];
            }
        };
        for (side, sign) in [(0, -1.0), (1, 1.0)] {
            set(&mut positions, &b.brow_inner[side], sign * 15.0, 700.0);
            set(&mut positions, &b.brow_peak[side], sign * 32.0, 705.0);
            set(&mut positions, &b.brow_outer[side], sign * 52.0, 699.0);
        }
        set(&mut positions, &b.brow_middle, 0.0, 698.0);
        set(&mut positions, &b.forehead_middle, 0.0, 730.0);
        set(&mut positions, &b.hairline_middle, 0.0, 758.0);
        nudge(&mut positions);
        positions
    }

    fn sliders(amount: f64) -> ForeheadLines {
        ForeheadLines { amount, spacing: 0.6, continuity: 0.5, waviness: 0.5, variation: 3, depth: 8.0, sharpness: 0.6, ridge: 0.0, dip_mm: 3.0, reach: 0.8 }
    }

    #[test]
    fn zero_amount_paints_nothing() {
        let head = ForeheadHead::new(&test_head(|_| ())).expect("landmarks");
        let field = LineField::new(&ForeheadLines::NONE, &head);
        assert!(field.is_empty() && ForeheadLines::NONE.is_none());
    }

    #[test]
    fn the_amount_never_reshuffles_the_lines() {
        let head = ForeheadHead::new(&test_head(|_| ())).expect("landmarks");
        let mut shown = 0;
        for step in 0..=20 {
            let field = LineField::new(&sliders(f64::from(step) / 20.0), &head);
            assert!(field.line_count() >= shown, "lines went from {shown} to {} at step {step}", field.line_count());
            shown = field.line_count();
        }
        // With the light band on, every line keeps its place, and its depth and length only grow.
        let mut previous: Vec<Line> = Vec::new();
        for step in 0..=20 {
            let field = LineField::new(&ForeheadLines { ridge: 0.6, ..sliders(f64::from(step) / 20.0) }, &head);
            for old in &previous {
                let now = field.lines.iter().find(|l| l.key == old.key).expect("a line vanished");
                assert!(now.share == old.share && now.centre_a == old.centre_a && now.depth >= old.depth && now.half_length >= old.half_length);
            }
            previous = field.lines.clone();
        }
        // A low amount shows one or two lines.
        let low = LineField::new(&sliders(0.15), &head).line_count();
        assert!((1..=2).contains(&low), "{low} lines at 0.15");
    }

    #[test]
    fn a_small_change_of_the_head_gives_a_small_change_of_the_lines() {
        let head = ForeheadHead::new(&test_head(|_| ())).expect("landmarks");
        // Move the left brow's peak up by 0.1 mm and the hairline by 0.1 mm.
        let nudged = ForeheadHead::new(&test_head(|p| {
            for &v in BROW_ANCHORS.brow_peak[0].vertices.iter().chain(&BROW_ANCHORS.hairline_middle.vertices) {
                p[v as usize][1] += 0.0001;
            }
        }))
        .expect("landmarks");
        let (a, b) = (LineField::new(&sliders(1.0), &head), LineField::new(&sliders(1.0), &nudged));
        assert_eq!(a.line_count(), b.line_count());
        for line in 0..a.line_count() {
            for x in [-40.0, -25.0, -5.0, 10.0, 30.0] {
                let moved = (a.centre_v(line, x) - b.centre_v(line, x)).abs();
                assert!(moved < 0.15, "line {line} at {x} moved {moved} mm");
            }
        }
        // A 5 mm higher brow peak lifts the lowest lines over that brow and leaves the other side alone.
        let raised = ForeheadHead::new(&test_head(|p| {
            for &v in &BROW_ANCHORS.brow_peak[0].vertices {
                p[v as usize][1] += 0.005;
            }
        }))
        .expect("landmarks");
        let c = LineField::new(&sliders(1.0), &raised);
        let lowest = (0..a.line_count()).min_by(|&i, &j| a.lines[i].share.total_cmp(&a.lines[j].share)).unwrap();
        assert!(c.centre_v(lowest, -32.0) - a.centre_v(lowest, -32.0) > 2.0);
        assert!((c.centre_v(lowest, 32.0) - a.centre_v(lowest, 32.0)).abs() < 0.3);
    }
}
