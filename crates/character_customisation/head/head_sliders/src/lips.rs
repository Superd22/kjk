//! The lips' shape, measured as outlines and proportions on any triangle mesh in the frame of the eyes (`measure::Frame`), and the
//! lip sliders (roadmap B3 and B5 for the mouth). Viewer 2 does not use it: only the `sliders lips` tools read it.

use crate::V3;
use crate::measure::Surface;

/// The part of the face the measure reads, in mm in the frame of the eyes.
pub const WINDOW: crate::measure::Window = crate::measure::Window { x: 45.0, y: [-130.0, -40.0], z: -40.0 };


use crate::measure::Cut;

/// The proportions: key, words for a person, unit.
pub const PROPORTIONS: [(&str, &str); 9] = [
    ("width", "Mouth width, from one corner of the lips to the other, seen from the front (mm)"),
    ("upper_height", "Height of the upper lip in the middle, from the line between the lips to its border (mm)"),
    ("lower_height", "Height of the lower lip in the middle, from the line between the lips to its border (mm)"),
    ("upper_out", "How far the upper lip stands out in front of the nose's base (mm)"),
    ("lower_out", "How far the lower lip stands out in front of the fold above the chin (mm)"),
    ("corner_height", "How high the mouth's corners lie against the middle of the line between the lips (mm)"),
    ("corner_depth", "How far the mouth's corners lie behind the middle of the lips (mm)"),
    ("bow", "The Cupid's bow: how far the peaks of the upper lip's border rise above its middle (mm)"),
    ("lip_line", "How deep the line between the lips sinks behind the two lips in the middle (mm)"),
];

/// The readings of the outlines: the profile down the middle, the profile 10 mm to each side, and each half of the lips seen from
/// the front.
pub const READINGS: [&str; 3] = ["middle profile", "profile 10 mm", "front"];
/// The side profiles' distance from the middle, in mm.
pub const SIDE_X: f64 = 10.0;
/// How deep the line between the lips may show in a profile outline, behind the less prominent lip, in mm. KOTOR's lips part in a
/// slit on most heads, and a cut through the slit sees the inside of the mouth.
pub const LINE_CAP: f64 = 3.0;

/// One column of the lips: a cut upright at `x` across the face.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Column {
    pub x: f64,
    /// Heights and depths: the line between the lips (stomion), the upper and lower lips' crests (their most forward places), and
    /// the upper and lower borders.
    pub line: [f64; 2],
    pub upper: [f64; 2],
    pub lower: [f64; 2],
    pub border_up: f64,
    pub border_down: f64,
    /// How deep the line sinks behind the less prominent crest.
    pub valley: f64,
}

/// One head's lips.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Lips {
    /// The nose's base (subnasale) and the fold above the chin, down the middle, as (depth, height).
    pub nose: [f64; 2],
    pub fold: [f64; 2],
    /// The outlines' anchor: the middle's upper lip depth and its line's height.
    pub anchor: [f64; 2],
    /// How far each corner lies from the middle, +x side first.
    pub halves: [f64; 2],
    /// The columns every 0.5 mm from the middle, on the +x side and on the -x side, out to the corners.
    pub columns: [Vec<Column>; 2],
    /// Each reading's outline on the +x and -x sides (the middle profile only in the first place), lined up on the nose's base.
    pub outlines: Vec<[Option<Vec<[f64; 2]>>; 2]>,
    pub proportions: Vec<f64>,
}

fn cut(surface: &Surface, x: f64) -> Cut {
    raw_cut(surface, x).blurred(0.5)
}

/// The cut without blurring, so a thin slit between the lips still sinks.
fn raw_cut(surface: &Surface, x: f64) -> Cut {
    let from = -125.0;
    let mut z = surface.front(0, x, 1, from, 360);
    // where the lips part and the cut sees no surface between them, the slit is filled as a deep V whose bottom is its middle
    let valid: Vec<usize> = (0..z.len()).filter(|&k| !z[k].is_nan()).collect();
    for w in valid.windows(2) {
        let (a, b) = (w[0], w[1]);
        if b > a + 1 {
            let c = 0.5 * (a + b) as f64;
            let low = z[a].min(z[b]) - 6.0;
            for k in a + 1..b {
                z[k] = low + 0.01 * (k as f64 - c).abs();
            }
        }
    }
    Cut { from, z }
}

fn argmax(c: &Cut, lo: f64, hi: f64) -> Option<(f64, f64)> {
    (c.index(lo)..=c.index(hi)).filter(|&k| !c.z[k].is_nan()).map(|k| (c.u(k), c.z[k])).max_by(|a, b| a.1.total_cmp(&b.1))
}

fn argmin(c: &Cut, lo: f64, hi: f64) -> Option<(f64, f64)> {
    (c.index(lo)..=c.index(hi)).filter(|&k| !c.z[k].is_nan()).map(|k| (c.u(k), c.z[k])).min_by(|a, b| a.1.total_cmp(&b.1))
}

/// A line that sinks deeper than this behind the lips may be an open mouth, in mm.
const OPEN_SINK: f64 = 2.0;

/// The line between the lips in a cut, searched from `lo` to `hi`: a place that sinks behind the lips within 7 mm above and 9 mm
/// below it. The fold above the chin also sinks between two crests, so the highest place that sinks at least half as deep as the
/// deepest is taken.
fn line_in(c: &Cut, raw: &Cut, lo: f64, hi: f64) -> Option<(f64, f64, f64)> {
    let mut all: Vec<(f64, f64, f64)> = Vec::new();
    for k in c.index(lo)..=c.index(hi) {
        let (y, z) = (c.u(k), c.z[k].min(raw.z[k]));
        if z.is_nan() {
            continue;
        }
        let (Some(u), Some(d)) = (argmax(c, y + 0.25, y + 7.0), argmax(c, y - 9.0, y - 0.25)) else { continue };
        all.push((y, z, u.1.min(d.1) - z));
    }
    let deepest = all.iter().map(|a| a.2).fold(f64::NEG_INFINITY, f64::max);
    // the deepest place of the highest stretch that sinks at least half as deep
    let top = all.iter().rev().position(|a| a.2 >= 0.5 * deepest)?;
    let mut k = all.len() - 1 - top;
    while k > 0 && all[k - 1].2 >= all[k].2 {
        k -= 1;
    }
    // an open mouth sinks deep, and about as deep all across its gap: the line is the middle of the places within 1 mm of the
    // deepest next to it
    let (mut a, mut b) = (k, k);
    if all[k].2 < OPEN_SINK {
        return Some(all[k]);
    }
    while a + 1 < all.len() && all[a + 1].2 >= all[k].2 - 1.0 {
        a += 1;
    }
    while b > 0 && all[b - 1].2 >= all[k].2 - 1.0 {
        b -= 1;
    }
    Some(all[(a + b) / 2])
}

/// The longest stretch of the line between the lips that may sink less than 0.3 mm before the corner, in mm.
const CORNER_GAP: f64 = 3.0;
/// How far from a lip's crest the hollow beside it is searched, in mm.
const HOLLOW: f64 = 10.0;
/// A lip whose surface falls back less than this from its crest to the hollow has no border, in mm.
const MIN_FALL: f64 = 0.4;

/// The column at `x`, with the line searched within 3 mm of `near`, or None where the lips have ended.
fn column(surface: &Surface, x: f64, near: f64, nose: [f64; 2], fold: [f64; 2]) -> Option<Column> {
    let raw = raw_cut(surface, x);
    let c = raw.blurred(0.5);
    let (ly, lz, v) = line_in(&c, &raw, near - 3.0, near + 3.0)?;
    if v < 0.3 {
        return None;
    }
    let up = argmax(&c, ly + 0.25, ly + 7.0)?;
    let down = argmax(&c, ly - 9.0, ly - 0.25)?;
    // a lip's border is where its surface has fallen back half way from the lip's crest to the hollow beside it (the deepest place
    // within `HOLLOW` mm above the upper crest, or below the lower one), on the cut blurred by 1 mm. It reads on smooth lips that show
    // no crease at the border, KOTOR's and ours alike.
    let soft = raw.blurred(1.0);
    let half_fall = |crest: f64, dir: f64| -> f64 {
        let (Some(top), Some(hollow)) = (soft.z.get(soft.index(crest)).copied(), if dir > 0.0 { argmin(&soft, crest, crest + HOLLOW) } else { argmin(&soft, crest - HOLLOW, crest) }) else { return f64::NAN };
        let fall = top - hollow.1;
        if !(fall >= MIN_FALL) {
            return f64::NAN;
        }
        let (from, to) = (soft.index(crest), soft.index(hollow.0));
        let ks: Vec<usize> = if to >= from { (from..=to).collect() } else { (to..=from).rev().collect() };
        ks.into_iter().find(|&k| soft.z[k] <= top - 0.5 * fall).map_or(f64::NAN, |k| soft.u(k))
    };
    let bu = half_fall(up.0, 1.0);
    let bd = half_fall(down.0, -1.0);
    let _ = (nose, fold);
    Some(Column { x, line: [lz, ly], upper: [up.1, up.0], lower: [down.1, down.0], border_up: bu, border_down: bd, valley: v })
}

/// How far the profiles reach above and below the line between the lips, in mm: past the lips, short of the nose and the chin.
pub const ABOVE: f64 = 11.0;
pub const BELOW: f64 = 13.0;

/// A profile from `ABOVE` the line between the lips down to `BELOW` it at `x`, lined up on the anchor (the middle's line height and
/// its upper lip's depth), with the line between the
/// lips capped at `LINE_CAP` behind the less prominent lip.
fn profile(surface: &Surface, x: f64, col: &Column, anchor: [f64; 2]) -> Option<Vec<[f64; 2]>> {
    let c = cut(surface, x);
    let floor = col.upper[0].min(col.lower[0]) - LINE_CAP;
    let pts: Vec<[f64; 2]> = (c.index(anchor[1] - BELOW)..=c.index(anchor[1] + ABOVE)).rev().filter(|&k| !c.z[k].is_nan()).map(|k| [c.z[k].max(floor) - anchor[0], c.u(k) - anchor[1]]).collect();
    (pts.len() > 40).then_some(pts)
}

/// Measures the lips of a surface, or says what could not be found.
pub fn measure(surface: &Surface) -> Result<Lips, &'static str> {
    measure_with(surface, None)
}

/// How far from the guide the middle's line between the lips is searched, in mm.
const GUIDE_REACH: f64 = 4.0;

/// The upper and lower lip borders of one of our heads, each from one corner to the other, in the frame of the eyes (mm). They are
/// the lip paint's border (`head_paint::lips`), which follows the head's vertices, so they read on our smooth lips where the shading
/// shows no border.
#[cfg(not(target_arch = "wasm32"))]
pub fn our_borders(local: &[V3]) -> [Vec<V3>; 2] {
    let [u, l, _] = our_curves(local);
    [u, l]
}

/// The lip paint's upper border, lower border and opening (the middle of its upper and lower edges) on one of our heads (local mm).
#[cfg(not(target_arch = "wasm32"))]
pub fn our_curves(local: &[V3]) -> [Vec<V3>; 3] {
    let point = |a: &head_paint::folds::Anchor| -> V3 {
        let mut p = [0.0; 3];
        for (&v, w) in a.vertices.iter().zip(a.weights) {
            p = crate::add(p, crate::scale(local[v as usize], w));
        }
        p
    };
    let curve = |a: &[head_paint::folds::Anchor]| a.iter().map(point).collect::<Vec<V3>>();
    let (ou, ol) = (curve(&head_paint::lips::OPENING_UPPER), curve(&head_paint::lips::OPENING_LOWER));
    let opening = ou.iter().zip(&ol).map(|(a, b)| crate::scale(crate::add(*a, *b), 0.5)).collect();
    [curve(&head_paint::lips::UPPER), curve(&head_paint::lips::LOWER), opening]
}

/// Measures the lips of one of our heads (local mm), with the line between the lips searched near the lip paint's opening.
#[cfg(not(target_arch = "wasm32"))]
pub fn measure_ours(local: &[V3], triangles: &[[u32; 3]]) -> Result<Lips, &'static str> {
    measure_with(&tools::local(local, triangles), Some(opening_height(local)))
}

/// The height of the lip paint's opening in the middle of one of our heads (local mm). Our fit of a KOTOR head gives the guide for
/// KOTOR's own mesh too: the fold under the nose sinks as deep as a thin slit, and without a guide the search took it for the line
/// on 21 of our heads and 6 of KOTOR's.
#[cfg(not(target_arch = "wasm32"))]
pub fn opening_height(local: &[V3]) -> f64 {
    border_at(&our_curves(local)[2], 0.0)
}

/// The height of a border curve at `x`, or NaN past its ends.
fn border_at(curve: &[V3], x: f64) -> f64 {
    for w in curve.windows(2) {
        let (a, b) = (w[0], w[1]);
        if (a[0] - x) * (b[0] - x) <= 0.0 && a[0] != b[0] {
            let t = (x - a[0]) / (b[0] - a[0]);
            return a[1] + t * (b[1] - a[1]);
        }
    }
    f64::NAN
}

/// Measures the lips of a surface, with the middle's line between the lips searched within `GUIDE_REACH` of `guide` when one is given.
pub fn measure_with(surface: &Surface, guide: Option<f64>) -> Result<Lips, &'static str> {
    let raw = raw_cut(surface, 0.0);
    let mid = raw.blurred(0.5);
    let (lo, hi) = guide.filter(|g| g.is_finite()).map_or((-92.0, -58.0), |g| (g - GUIDE_REACH, g + GUIDE_REACH));
    let (ly, _, _) = line_in(&mid, &raw, lo, hi).ok_or("no line between the lips")?;
    let up = argmax(&mid, ly + 0.25, ly + 7.0).ok_or("no upper lip")?;
    let down = argmax(&mid, ly - 9.0, ly - 0.25).ok_or("no lower lip")?;
    let nose = argmin(&mid, up.0, up.0 + 15.0).ok_or("no nose base")?;
    // the fold above the chin: the deepest place between the lower lip and the chin's front, the front-most place 5 to 25 mm under
    // the lower lip
    let chin = argmax(&mid, down.0 - 25.0, down.0 - 5.0).ok_or("no chin")?;
    let fold = argmin(&mid, chin.0, down.0).ok_or("no fold above the chin")?;
    let (nose, fold) = ([nose.1, nose.0], [fold.1, fold.0]);
    let mut columns: [Vec<Column>; 2] = [Vec::new(), Vec::new()];
    for (s, side) in [1.0, -1.0].iter().enumerate() {
        let mut near = ly;
        let mut misses = 0;
        for k in 0..100 {
            let x = side * k as f64 * 0.5;
            let Some(c) = column(surface, x, near, nose, fold) else {
                // a shallow stretch of the line up to `CORNER_GAP` mm long does not end the mouth
                misses += 1;
                if k == 0 || misses as f64 * 0.5 > CORNER_GAP {
                    break;
                }
                continue;
            };
            misses = 0;
            near = c.line[1];
            columns[s].push(c);
        }
    }
    // the borders are smoothed along the mouth over 2 mm each way: our lips' vertices lie about 2 mm apart
    for cols in columns.iter_mut() {
        let up: Vec<f64> = cols.iter().map(|c| c.border_up).collect();
        let down: Vec<f64> = cols.iter().map(|c| c.border_down).collect();
        let smooth = |v: &[f64], i: usize| -> f64 {
            if v[i].is_nan() {
                return f64::NAN;
            }
            let w: Vec<f64> = v[i.saturating_sub(4)..(i + 5).min(v.len())].iter().copied().filter(|x| x.is_finite()).collect();
            w.iter().sum::<f64>() / w.len() as f64
        };
        for i in 0..cols.len() {
            cols[i].border_up = smooth(&up, i);
            cols[i].border_down = smooth(&down, i);
        }
    }
    let m = *columns[0].first().ok_or("the middle column cannot be read")?;
    let mut outlines: Vec<[Option<Vec<[f64; 2]>>; 2]> = Vec::new();
    let anchor = [m.upper[0], m.line[1]];
    outlines.push([profile(surface, 0.0, &m, anchor), None]);
    outlines.push(std::array::from_fn(|s| {
        let c = columns[s].iter().min_by(|a, b| (a.x.abs() - SIDE_X).abs().total_cmp(&(b.x.abs() - SIDE_X).abs())).filter(|c| (c.x.abs() - SIDE_X).abs() < 1.01)?;
        profile(surface, c.x, c, anchor)
    }));
    outlines.push(std::array::from_fn(|s| {
        let cols = &columns[s];
        (cols.len() > 10).then(|| {
            let mut pts: Vec<[f64; 2]> = cols.iter().filter(|c| c.border_up.is_finite()).map(|c| [c.x.abs(), c.border_up - anchor[1]]).collect();
            pts.extend(cols.iter().rev().filter(|c| c.border_down.is_finite()).map(|c| [c.x.abs(), c.border_down - anchor[1]]));
            pts
        })
    }));
    let corners: Vec<&Column> = columns.iter().filter_map(|c| c.last()).collect();
    let mean = |f: &dyn Fn(&Column) -> f64| corners.iter().map(|c| f(c)).sum::<f64>() / corners.len() as f64;
    let width = columns.iter().map(|c| c.last().map_or(0.0, |c| c.x.abs())).sum::<f64>();
    let bow = columns
        .iter()
        .flat_map(|cs| cs.iter().filter(|c| c.x.abs() >= 1.5 && c.x.abs() <= 9.0))
        .map(|c| c.border_up)
        .filter(|b| b.is_finite())
        .fold(f64::NEG_INFINITY, f64::max)
        - m.border_up;
    let proportions = vec![
        width,
        m.border_up - m.line[1],
        m.line[1] - m.border_down,
        m.upper[0] - nose[0],
        m.lower[0] - fold[0],
        mean(&|c| c.line[1]) - m.line[1],
        0.5 * (m.upper[0] + m.lower[0]) - mean(&|c| 0.5 * (c.upper[0] + c.lower[0])),
        bow,
        m.valley.min(LINE_CAP),
    ];
    let halves = columns.clone().map(|c| c.last().map_or(f64::NAN, |c| c.x.abs()));
    Ok(Lips { nose, fold, anchor, halves, columns, outlines, proportions })
}

/// The pass mark of each reading's outline gap, in mm: the gap between the 5% most alike pairs of KOTOR's heads (checks of
/// 2026-11-22, round 2). Below it, ours is as close to KOTOR as two nearly identical KOTOR heads are.
pub const OUTLINE_MARKS: [f64; 3] = [0.35, 0.55, 0.90];
/// The pass mark of each proportion, in its unit: a quarter of KOTOR's spread (10th to 90th percentile). The line's depth is not
/// scored (KOTOR's lips part on most heads, and the depth is capped).
pub const PROPORTION_MARKS: [f64; 9] = [3.1, 1.0, 1.2, 0.76, 1.06, 1.1, 1.7, 0.52, f64::INFINITY];

/// How much each proportion counts in the cost. Width and the corners' depth have no outline that reads them well (the front
/// outline's ends are the corners, but a narrow mouth's outline still lies on a wide one's), so they count fully.
pub const PROPORTION_WEIGHTS: [f64; 9] = [1.0, 0.25, 0.25, 0.25, 0.25, 0.25, 1.0, 0.25, 0.0];

/// How far `ours` is from `kotor`, as one number: each reading's gap and each proportion's miss over its mark, squared and summed.
pub fn cost(kotor: &Lips, ours: &Lips) -> f64 {
    let mut c = 0.0;
    for r in 0..READINGS.len() {
        c += 2.0 * (reading_gap(kotor, ours, r).unwrap_or(if kotor.outlines[r].iter().any(|o| o.is_some()) { 5.0 } else { 0.0 }) / OUTLINE_MARKS[r]).powi(2);
    }
    for (i, m) in PROPORTION_MARKS.iter().enumerate() {
        let d = kotor.proportions[i] - ours.proportions[i];
        if d.is_finite() {
            c += PROPORTION_WEIGHTS[i] * (d / m).powi(2);
        }
    }
    c
}

/// The gap between two heads in one reading: the mean over the sides both can read, or None.
pub fn reading_gap(a: &Lips, b: &Lips, r: usize) -> Option<f64> {
    let g: Vec<f64> = (0..2).filter_map(|s| Some(crate::chin_neck::gap(a.outlines[r][s].as_ref()?, b.outlines[r][s].as_ref()?))).collect();
    (!g.is_empty()).then(|| g.iter().sum::<f64>() / g.len() as f64)
}

/// The lip sliders: name, the proportion it sets, its hint, and its group.
pub const SLIDERS: [(&str, &str, &str); 8] = [
    ("Mouth width", "width", "How wide the mouth is, from one corner to the other. The lips stretch to the corners."),
    ("Mouth corner depth", "corner_depth", "How far back the corners of the mouth sink into the cheeks."),
    ("Mouth corner height", "corner_height", "How high the corners of the mouth sit. High corners turn the mouth up, low corners turn it down."),
    ("Upper lip fullness", "upper_out", "How far the upper lip stands out."),
    ("Lower lip fullness", "lower_out", "How far the lower lip stands out."),
    ("Upper lip height", "upper_height", "How tall the upper lip is, from the line between the lips to its border."),
    ("Lower lip height", "lower_height", "How tall the lower lip is, from the line between the lips to its border."),
    ("Cupid's bow", "bow", "How high the two peaks of the upper lip rise above the dip between them."),
];

/// The ranges the solve may use, before the KOTOR fits set them.
pub const RANGES: [(f64, f64); 8] = [(-3.0, 6.0), (-6.0, 6.0), (-4.0, 4.0), (-4.0, 5.0), (-4.0, 5.0), (-6.0, 6.0), (-6.0, 6.0), (-2.0, 3.0)];
/// The ranges a player gets: the values the 57 solved heads use (round 2, loop 2) with 0.5 more on each side, within the ends that
/// were drawn clean. Mouth width's ends are the solve's own, since narrowing past -3 and widening past 6 turn triangles at the corners.
pub const PLAYER_RANGES: [(f64, f64); 8] = [(-3.0, 6.0), (-1.13, 5.28), (-4.0, 3.94), (-1.17, 1.63), (-0.92, 1.91), (-5.0, 4.62), (-2.0, 6.0), (-1.84, 2.6)];
/// The sliders whose movement is added after the build's smoothing (kind "after"): the smoothing spreads a corner's move along the
/// lips and erases a bump as narrow as the bow.
pub const AFTER: [bool; 8] = [true, false, false, false, false, false, false, true];

/// How far round each corner the corner fields move the skin as one piece, in mm.
const FLAT: f64 = 6.0;
/// The mouth's inside follows the lips fully this far behind the line between the lips, then fades out over the second number (mm).
const LIPS_INSIDE: (f64, f64) = (10.0, 10.0);

/// The lip sliders' movements, made once on the base head. Each is a smooth movement of the vertices round the mouth, in mm for one
/// unit of the slider, so the same value gives the same movement on any head.
pub mod fields {
    use super::*;
    use crate::jaw_edge::smoothstep;
    use crate::mesh::HeadMesh;

    /// The base head's lips, read by the measure, as functions of the distance from the middle.
    struct Base {
        corner: f64,
        xs: Vec<f64>,
        line: Vec<[f64; 2]>,
        upper: Vec<f64>,
        lower: Vec<f64>,
        border_up: Vec<f64>,
        border_down: Vec<f64>,
        nose: f64,
        fold: f64,
    }

    impl Base {
        fn new(l: &Lips) -> Self {
            let cols = &l.columns[0];
            let m = cols[0];
            let fill = |v: f64, d: f64| if v.is_finite() { v } else { d };
            Self {
                corner: 0.5 * (l.halves[0] + l.halves[1]),
                xs: cols.iter().map(|c| c.x).collect(),
                line: cols.iter().map(|c| [c.line[0], c.line[1]]).collect(),
                upper: cols.iter().map(|c| c.upper[1]).collect(),
                lower: cols.iter().map(|c| c.lower[1]).collect(),
                border_up: cols.iter().map(|c| fill(c.border_up, m.border_up)).collect(),
                border_down: cols.iter().map(|c| fill(c.border_down, m.border_down)).collect(),
                nose: l.nose[1],
                fold: l.fold[1],
            }
        }

        /// A column value at distance `x` from the middle, held at the corner past it.
        fn at(&self, v: &[f64], x: f64) -> f64 {
            let x = x.abs();
            let k = self.xs.iter().position(|&a| a >= x).unwrap_or(self.xs.len() - 1);
            if k == 0 {
                return v[0];
            }
            let (a, b) = (self.xs[k - 1], self.xs[k]);
            let t = if b > a { ((x - a) / (b - a)).clamp(0.0, 1.0) } else { 0.0 };
            v[k - 1] + t * (v[k] - v[k - 1])
        }

        fn line_y(&self, x: f64) -> f64 {
            self.at(&self.line.iter().map(|l| l[1]).collect::<Vec<_>>(), x)
        }

        fn line_z(&self, x: f64) -> f64 {
            self.at(&self.line.iter().map(|l| l[0]).collect::<Vec<_>>(), x)
        }
    }

    /// Each merged vertex's share of the upper lip: 1 on the upper lip, 0 on the lower lip, blended round the corners. It is found
    /// by the distance along the skin to the skin clearly above and clearly below the line, so the upper lip's curl into the mouth,
    /// which hangs below the line, stays with the upper lip.
    fn upper_share(mesh: &HeadMesh, q: &[V3], b: &Base) -> Vec<f64> {
        let n = q.len();
        let near_mouth = |p: V3| p[0].abs() < b.corner + 15.0 && p[2] > b.line_z(p[0]) - 15.0;
        let dist = |seed: &dyn Fn(V3) -> bool| -> Vec<f64> {
            let mut d = vec![f64::INFINITY; n];
            let mut heap = std::collections::BinaryHeap::new();
            for g in 0..n {
                if !mesh.hidden[g] && near_mouth(q[g]) && seed(q[g]) {
                    d[g] = 0.0;
                    heap.push(std::cmp::Reverse((0u64, g)));
                }
            }
            while let Some(std::cmp::Reverse((dk, g))) = heap.pop() {
                let dg = dk as f64 / 1e6;
                if dg > d[g] {
                    continue;
                }
                for &j in mesh.neighbours_of(g) {
                    let j = j as usize;
                    if mesh.hidden[j] {
                        continue;
                    }
                    let nd = dg + crate::length(crate::sub(q[j], q[g]));
                    if nd < d[j] {
                        d[j] = nd;
                        heap.push(std::cmp::Reverse(((nd * 1e6) as u64, j)));
                    }
                }
            }
            d
        };
        let up = dist(&|p| p[1] > b.line_y(p[0]) + 2.0);
        let down = dist(&|p| p[1] < b.line_y(p[0]) - 2.0);
        (0..n)
            .map(|g| match (up[g].is_finite(), down[g].is_finite()) {
                (true, true) if up[g] + down[g] > 0.0 => down[g] / (up[g] + down[g]),
                (true, true) => 0.5,
                (true, false) => 1.0,
                (false, true) => 0.0,
                (false, false) => if q[g][1] > b.line_y(q[g][0]) { 1.0 } else { 0.0 },
            })
            .collect()
    }

    /// The movement of slider `k` (of `SLIDERS`) at a merged skin vertex `p` (local mm) with upper share `w`, in local mm.
    fn movement(k: usize, b: &Base, p: V3, w: f64) -> V3 {
        // nothing moves outside the mouth's window: from 5 mm under the fold above the chin to 3 mm over the nose's base, and
        // within 15 mm behind the line
        let v = p[1];
        let window = (1.0 - smoothstep((b.fold - 5.0 - v) / 4.0)) * (1.0 - smoothstep((v - b.nose - 3.0) / 4.0)) * (1.0 - smoothstep((b.line_z(p[0]) - 15.0 - p[2]) / 5.0));
        if window <= 0.0 {
            return [0.0; 3];
        }
        crate::scale(movement_in(k, b, p, w), window)
    }

    fn movement_in(k: usize, b: &Base, p: V3, w: f64) -> V3 {
        let (x, y) = (p[0], p[1]);
        let ax = x.abs();
        let a = ax / b.corner;
        let sx = if x >= 0.0 { 1.0 } else { -1.0 };
        let v = y - b.line_y(x);
        // round the corner the corner fields are flat for `FLAT` mm each way, so a head whose corner lies elsewhere than the base
        // head's still moves its corner as one piece; past that every movement fades out over 20 mm
        let past = 1.0 - smoothstep((ax - b.corner - FLAT) / 20.0);
        // across: the lips' middle part, fading to the corners
        let core = 1.0 - smoothstep((a - 0.55) / 0.45);
        let wide = 1.0 - smoothstep((a - 0.75) / 0.35);
        let inner = b.corner - FLAT;
        // the corners: rising from a third of the way out to `FLAT` mm short of the corner
        let corner = smoothstep((ax - 0.3 * b.corner) / (inner - 0.3 * b.corner)) * past;
        let (cu, cl) = (b.at(&b.upper, x) - b.line_y(x), b.line_y(x) - b.at(&b.lower, x));
        let (bu, bl) = ((b.at(&b.border_up, x) - b.line_y(x)).max(cu + 0.5), (b.line_y(x) - b.at(&b.border_down, x)).max(cl + 0.5));
        let (nu, fl) = ((b.nose - b.line_y(0.0)).max(bu + 2.0), (b.line_y(0.0) - b.fold).max(bl + 2.0));
        match k {
            // width: the corners move out along the mouth, the middle stays; the move follows the face's turn backward
            0 => {
                // the width's move spreads further up and down and past the corner than the other fields, so a wide move shears
                // the cheek gently
                let e = 1.0 - smoothstep((v.abs() - 6.0) / 22.0);
                // the lips stretch evenly from the middle to the corners (a move after the smoothing keeps that shape), and the
                // cheek past the corners gives way over 32 mm
                let m = (ax / b.corner).min(1.0) * (1.0 - smoothstep((ax - b.corner - FLAT) / 32.0));
                // the face's turn backward, read over 4 mm and held from 3 mm short of the corner on, so the corner's tip moves as
                // one piece and its thin triangles do not turn over
                let a0 = ax.min(b.corner - 3.0).max(2.0);
                let slope = ((b.line_z(a0 + 2.0) - b.line_z(a0 - 2.0)) / 4.0).clamp(-2.0, 2.0);
                let dx = sx * m * e;
                [dx, 0.0, slope * m * e]
            }
            1 => [0.0, 0.0, -corner * (1.0 - smoothstep((v.abs() - 3.0) / 10.0))],
            2 => [0.0, corner * (1.0 - smoothstep((v.abs() - 3.0) / 10.0)), 0.0],
            // fullness: the lip comes forward from the line to its crest, fading out by 3 mm past its border
            3 => {
                let h = if v < 0.0 { 1.0 - smoothstep(-v / 3.0) } else if v <= cu { 1.0 } else { 1.0 - smoothstep((v - cu) / (bu - cu + 3.0)) };
                [0.0, 0.0, w * core * past * h]
            }
            4 => {
                let h = if v > 0.0 { 1.0 - smoothstep(v / 3.0) } else if -v <= cl { 1.0 } else { 1.0 - smoothstep((-v - cl) / (bl - cl + 3.0)) };
                [0.0, 0.0, (1.0 - w) * core * past * h]
            }
            // height: the border moves 1 mm, the line stays, and the skin up to the nose's base (down to the fold) gives way
            5 => {
                let h = if v <= 0.0 { 0.0 } else if v <= bu { v / bu } else { 1.0 - smoothstep((v - bu) / (nu - bu)) };
                [0.0, w * wide * past * h, 0.0]
            }
            6 => {
                let h = if v >= 0.0 { 0.0 } else if -v <= bl { -v / bl } else { 1.0 - smoothstep((-v - bl) / (fl - bl)) };
                [0.0, -(1.0 - w) * wide * past * h, 0.0]
            }
            // the bow: along the upper border the lip comes forward at the two peaks and goes back at the dip between them, so the
            // border (where the lip has fallen back half way) rises at the peaks and sinks in the middle
            7 => {
                let bow = (-((ax - 6.0) / 3.0).powi(2)).exp() - 1.5 * (-(x / 3.0).powi(2)).exp();
                let g = (-((v - bu) / 3.0).powi(2)).exp();
                [0.0, 0.0, w * bow * g]
            }
            _ => [0.0; 3],
        }
    }

    /// The lip sliders as a set, made on `base` (the smoothed base head, metres). Each slider moves 1 mm of its measure per unit,
    /// roughly; the solve and the ranges take care of the rest. Hidden vertices (the mouth's inside) follow their nearest skin, fading
    /// out from 4 to 12 mm behind the line between the lips.
    pub fn make(mesh: &HeadMesh, base: &[V3]) -> crate::set::SliderSet {
        let frame = crate::measure::Frame::head();
        let local: Vec<V3> = base.iter().map(|&p| frame.local(p)).collect();
        let lips = measure(&tools::local(&local, &mesh.triangles)).expect("the base head's lips");
        let b = Base::new(&lips);
        let q = mesh.merge(&local);
        let share = upper_share(mesh, &q, &b);
        let mut sliders = Vec::new();
        for (k, (name, key, hint)) in SLIDERS.iter().enumerate() {
            let mut m: Vec<V3> = (0..q.len()).map(|g| if mesh.hidden[g] { [0.0; 3] } else { movement(k, &b, q[g], share[g]) }).collect();
            // three passes of smoothing over the skin, so no movement has a kink (one for the bow, which is narrow)
            for _ in 0..if k == 7 { 1 } else { 3 } {
                let prev = m.clone();
                for g in 0..q.len() {
                    if mesh.hidden[g] {
                        continue;
                    }
                    let nb: Vec<usize> = mesh.neighbours_of(g).iter().map(|&j| j as usize).filter(|&j| !mesh.hidden[j]).collect();
                    if nb.is_empty() {
                        continue;
                    }
                    let mut s = [0.0; 3];
                    for &j in &nb {
                        s = crate::add(s, prev[j]);
                    }
                    m[g] = crate::add(crate::scale(prev[g], 0.5), crate::scale(s, 0.5 / nb.len() as f64));
                }
            }
            let all = mesh.expand(&m);
            let mut vertices = Vec::new();
            let mut moves = Vec::new();
            for (i, d) in all.iter().enumerate() {
                // the vertices the smoothing keeps still move too: the lining of the mouth's corners is kept, and a corner whose
                // lining stays behind tears open
                let d = *d;
                if crate::length(d) < 1e-4 {
                    continue;
                }
                let w = crate::sub(frame.world(crate::add(local[i], d)), frame.world(local[i]));
                vertices.push(i as u32);
                moves.extend(w.map(|c| c as f32));
            }
            sliders.push(crate::set::Slider {
                name: name.to_string(),
                kind: if AFTER[k] { "after" } else { "proportion" }.into(),
                zone: "mouth".into(),
                unit: "1 mm".into(),
                low: if std::env::var("LIPS_PLAYER").is_ok() { PLAYER_RANGES[k].0 } else { RANGES[k].0 },
                high: if std::env::var("LIPS_PLAYER").is_ok() { PLAYER_RANGES[k].1 } else { RANGES[k].1 },
                vertices,
                moves,
                hint: hint.to_string(),
                measure: key.to_string(),
                keys: Vec::new(),
                group: "lips".into(),
            });
        }
        crate::set::SliderSet { vertex_count: base.len(), sliders, fields: Vec::new() }
    }
}

pub mod tools {
    use super::*;
    use crate::heads::{parallel, threads};
    use crate::mesh::input::Input;
    use crate::under_jaw::tools::{kotor_bare, names};
    use serde_json::json;
    use std::path::Path;

    /// Viewer 2's live head as it builds it today, with the eye opening on the curve: the five features' sets merged, built on
    /// `starts_all.bin` with each head's solved values (the `page_curve.json` files where the curve has them).
    pub struct Live {
        pub mesh: crate::mesh::HeadMesh,
        pub frame: crate::measure::Frame,
        pub smoother: head_shape::smooth::Smoother,
        pub set: crate::set::SliderSet,
        pub names: Vec<String>,
        starts: Vec<f32>,
        values: Vec<Vec<f64>>,
        pub lips_first: usize,
    }

    const FILES: [(&str, &str); 6] = [
        ("sliders_r3_nostril.json", "page_r3_nostril.json"),
        ("jaw/sliders.json", "jaw/page_throat.json"),
        ("brow/sliders.json", "brow/page_fold.json"),
        ("cheeks/sliders.json", "cheeks/page_curve.json"),
        ("eyes/sliders_curve.json", "eyes/page_curve.json"),
        ("skull/sliders.json", "skull/page.json"),
    ];

    impl Live {
        pub fn load(input: &Input, data: &Path) -> Self {
            let web = data.join("web");
            let mesh = input.head_mesh();
            let read = |f: &str| std::fs::read_to_string(web.join(f)).unwrap_or_else(|e| panic!("{f}: {e}"));
            let heads: serde_json::Value = serde_json::from_str(&read("heads.json")).unwrap();
            let names: Vec<String> = heads.as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect();
            let mut sets = Vec::new();
            let mut values = vec![Vec::new(); names.len()];
            for (sf, pf) in FILES {
                let text = read(sf);
                let set: crate::set::SliderSet = serde_json::from_str(&text).unwrap();
                let raw: serde_json::Value = serde_json::from_str(&text).unwrap();
                // a curve slider rests at its middle, every other slider at 0 (as `sliders.js` sets `zero`)
                let zero: Vec<f64> = raw["sliders"].as_array().unwrap().iter().map(|s| if s["kind"] == "curve" { s["mid"].as_f64().unwrap_or(0.0) } else { 0.0 }).collect();
                let page: serde_json::Value = serde_json::from_str(&read(pf)).unwrap();
                let count = set.sliders.len();
                let last = page["counts"].as_array().and_then(|c| c.last()).map(|c| c.to_string()).unwrap_or_default();
                let order: Option<Vec<Option<usize>>> = page["sliders"]
                    .as_array()
                    .map(|a| a.iter().map(|x| set.sliders.iter().position(|sl| x["name"].as_str() == Some(sl.name.as_str()))).collect());
                for (i, name) in names.iter().enumerate() {
                    let mut out = zero.clone();
                    if let Some(h) = page["heads"].as_array().unwrap().iter().find(|h| h["name"].as_str() == Some(name.as_str())) {
                        let s = if h["solved"]["full"].is_object() { &h["solved"]["full"] } else { &h["solved"]["held_out"] };
                        if let Some(v) = s["counts"][&last]["values"].as_array() {
                            for (k, x) in v.iter().enumerate() {
                                let j = order.as_ref().map_or(Some(k), |o| o.get(k).copied().flatten());
                                if let Some(j) = j.filter(|&j| j < count) {
                                    out[j] = x.as_f64().unwrap_or(0.0);
                                }
                            }
                        }
                    }
                    values[i].extend(out);
                }
                sets.push(set);
            }
            let set = crate::set::SliderSet::merge(&sets);
            let f32s = |f: &str| -> Vec<f32> { std::fs::read(web.join(f)).unwrap().chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect() };
            let base: Vec<V3> = f32s("base.bin").chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect();
            crate::eye_curves::init(&base, &mesh.triangles);
            crate::set::init_mesh(&mesh.triangles, &mesh.keep);
            crate::set::init_curve(mesh.group.clone(), mesh.first.clone(), mesh.hidden.clone());
            let smoother = crate::set::smoother(&base, &mesh.triangles, &mesh.keep);
            let lips_first = set.sliders.len();
            Self { frame: crate::measure::Frame::head(), starts: f32s("starts_all.bin"), mesh, smoother, set, names, values, lips_first }
        }

        pub fn index(&self, name: &str) -> usize {
            self.names.iter().position(|n| n == name).unwrap_or_else(|| panic!("no head {name}"))
        }

        /// A head's live slider values.
        pub fn values(&self, name: &str) -> Vec<f64> {
            self.values[self.index(name)].clone()
        }

        /// A head built from its start shape and `values`, in metres.
        pub fn build(&self, name: &str, values: &[f64]) -> Vec<V3> {
            let i = self.index(name);
            let n = self.mesh.base.len();
            let start: Vec<V3> = self.starts[i * n * 3..(i + 1) * n * 3].chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect();
            self.set.build(&self.smoother, &start, values)
        }

        /// The live head in metres.
        pub fn live(&self, name: &str) -> Vec<V3> {
            self.build(name, &self.values[self.index(name)])
        }

        /// Adds the lip sliders after the live ones: a head's values are then its live values followed by its lip values.
        pub fn with_lips(&mut self, lips: &crate::set::SliderSet) {
            self.lips_first = self.set.sliders.len();
            self.set = crate::set::SliderSet::merge(&[self.set.clone(), lips.clone()]);
        }

        /// A head built with its live values and the lip values `lips`, in the frame of the eyes (mm).
        pub fn with(&self, name: &str, lips: &[f64]) -> Vec<V3> {
            let mut v = self.values[self.index(name)].clone();
            v.truncate(self.lips_first);
            v.extend_from_slice(lips);
            self.build(name, &v).iter().map(|&x| self.frame.local(x)).collect()
        }

        /// The base head's start shape (unsmoothed) built with its live values (none) and `lips`.
        pub fn base_with(&self, lips: &[f64]) -> Vec<V3> {
            let mut v = vec![0.0; self.lips_first];
            v.extend_from_slice(lips);
            self.set.build(&self.smoother, &self.mesh.base, &v).iter().map(|&x| self.frame.local(x)).collect()
        }

        /// The lip sliders' ranges.
        pub fn lip_ranges(&self) -> Vec<(f64, f64)> {
            self.set.sliders[self.lips_first..].iter().map(|s| (s.low, s.high)).collect()
        }
    }

    /// The close-up shots of the mouth: from the front and from the side.
    pub const FRONT: crate::feature::Shot = crate::feature::Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-40.0, 40.0, -120.0, -55.0] };
    pub const SIDE: crate::feature::Shot = crate::feature::Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-25.0, 55.0, -120.0, -55.0] };
    pub const THREE: crate::feature::Shot = crate::feature::Shot { name: "three-quarter", yaw: 40.0, pitch: 0.0, pivot: 0.0, window: [-40.0, 40.0, -120.0, -55.0] };


    /// Draws a line given in a shot's two coordinates (across and up, in mm) into its picture.
    pub fn line(img: &mut image::RgbImage, shot: &crate::feature::Shot, pts: &[[f64; 2]], colour: [u8; 3], thick: i64) {
        let px = std::env::var("SLIDERS_PX").ok().and_then(|v| v.parse().ok()).unwrap_or(2.5);
        let w = shot.window;
        let put = |img: &mut image::RgbImage, p: [f64; 2]| {
            let (x, y) = ((p[0] - w[0]) * px, (w[3] - p[1]) * px);
            for dy in -thick..=thick {
                for dx in -thick..=thick {
                    let (i, j) = (x as i64 + dx, y as i64 + dy);
                    if i >= 0 && j >= 0 && (i as u32) < img.width() && (j as u32) < img.height() {
                        img.put_pixel(i as u32, j as u32, image::Rgb(colour));
                    }
                }
            }
        };
        if pts.len() == 1 {
            put(img, pts[0]);
        }
        for s in pts.windows(2) {
            for t in 0..6 {
                let f = t as f64 / 6.0;
                put(img, [s[0][0] + f * (s[1][0] - s[0][0]), s[0][1] + f * (s[1][1] - s[0][1])]);
            }
        }
    }

    /// Draws `of`'s outlines lined up on `on`'s nose base: the front shot gets the front outline on both sides (mirrored back) and
    /// the line between the lips; the side shot gets the middle profile, and the 10 mm profiles when `sides`.
    pub fn draw_lips(front: &mut image::RgbImage, side: &mut image::RgbImage, on: &Lips, of: &Lips, colour: [u8; 3], thick: i64, sides: bool) {
        for s in 0..2 {
            let sign = if s == 0 { 1.0 } else { -1.0 };
            if let Some(o) = &of.outlines[2][s] {
                let pts: Vec<[f64; 2]> = o.iter().map(|p| [sign * p[0], p[1] + on.anchor[1]]).collect();
                line(front, &FRONT, &pts, colour, thick);
            }
            let l: Vec<[f64; 2]> = of.columns[s].iter().map(|c| [c.x, c.line[1] - of.anchor[1] + on.anchor[1]]).collect();
            line(front, &FRONT, &l, colour, 0);
        }
        let mut readings = vec![(0, 0)];
        if sides {
            readings.extend([(1, 0), (1, 1)]);
        }
        for (r, s) in readings {
            if let Some(o) = &of.outlines[r][s] {
                let pts: Vec<[f64; 2]> = o.iter().map(|p| [p[0] + on.anchor[0], p[1] + on.anchor[1]]).collect();
                line(side, &SIDE, &pts, colour, if r == 0 { thick } else { 0 });
            }
        }
    }

    /// KOTOR's lips on its own bare mesh.
    pub fn kotor(input: &Input, name: &str) -> Result<Lips, &'static str> {
        let (v, f, _) = kotor_bare(input, name);
        measure_with(&local(&v, &f), guide(input, name))
    }

    /// The guide for KOTOR's line between the lips: the lip paint's opening on our smoothed fit of that head.
    pub fn guide(input: &Input, name: &str) -> Option<f64> {
        let key = format!("heads/{name}/fit_smooth");
        if !input.has(&key) {
            return None;
        }
        let frame = crate::measure::Frame::head();
        let fit: Vec<V3> = input.points(&key).iter().map(|&p| frame.local(p)).collect();
        (fit.len() == input.head_mesh().base.len()).then(|| opening_height(&fit))
    }

    fn subdivided(input: &Input, name: &str) -> Result<Lips, &'static str> {
        let (v, f, _) = kotor_bare(input, name);
        let (sv, sf, _) = crate::checks::butterfly(&v, &f);
        let (sv, sf, _) = crate::checks::butterfly(&sv, &sf);
        measure_with(&local(&sv, &sf), guide(input, name))
    }

    pub fn pct(v: &mut [f64], q: f64) -> f64 {
        v.sort_by(f64::total_cmp);
        if v.is_empty() { f64::NAN } else { v[((v.len() - 1) as f64 * q).round() as usize] }
    }

    /// Our heads from a folder of kit-order bins (metres), in the frame of the eyes.
    pub fn ours(dir: &Path, name: &str, triangles: &[[u32; 3]]) -> Option<Lips> {
        let p = crate::chin_neck::tools::read_bin(dir, name)?;
        measure_ours(&p, triangles).ok()
    }

    /// How far past the corner the slit's end is left out of the fold count, in mm.
    const SLIT_PAST: f64 = 3.0;

    /// The triangles inside the slit between the lips on a head (local mm) whose lips are `l`: within 1.5 mm of the line's height
    /// and behind the lips' front, or facing up or down there. No one sees them, and the thin slivers there turn over under any move
    /// of one lip against the other, so the fold count leaves them out.
    pub fn slit(p: &[V3], triangles: &[[u32; 3]], l: &Lips) -> Vec<bool> {
        // the slit runs on a little past the corner, where the line sinks less than the measure's 0.3 mm
        let line_at = |x: f64| -> Option<&Column> {
            let cols = &l.columns[if x >= 0.0 { 0 } else { 1 }];
            cols.iter().min_by(|a, b| (a.x - x).abs().total_cmp(&(b.x - x).abs())).filter(|c| (c.x - x).abs() < 4.0 || (x.abs() > c.x.abs() && x.abs() - c.x.abs() < SLIT_PAST))
        };
        triangles
            .iter()
            .map(|t| {
                let c = t.map(|i| p[i as usize]);
                let m = [(c[0][0] + c[1][0] + c[2][0]) / 3.0, (c[0][1] + c[1][1] + c[2][1]) / 3.0, (c[0][2] + c[1][2] + c[2][2]) / 3.0];
                let Some(col) = line_at(m[0]) else { return false };
                if (m[1] - col.line[1]).abs() > 1.5 {
                    return false;
                }
                let n = crate::normalize(crate::cross(crate::sub(c[1], c[0]), crate::sub(c[2], c[0])));
                n[2].abs() < 0.6 || m[2] < col.upper[0].min(col.lower[0]) - 0.8
            })
            .collect()
    }

    /// Turned triangles of `built` against the checker's head, outside the slit.
    pub fn folds(checker: &crate::check::Checker, built_world: &[V3], slit: &[bool]) -> Vec<bool> {
        checker.marks(built_world).iter().zip(slit).map(|(&r, &s)| r && !s).collect()
    }

    /// The live heads with the lip sliders of `file` (made by `make`) added.
    pub fn live_lips(input: &Input, data: &Path, file: &str) -> Live {
        let mut live = Live::load(input, data);
        let set: crate::set::SliderSet = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
        live.with_lips(&set);
        live
    }

    /// Runs a command of the lips, or says it is not one: `sliders lips <command> <data> ...`.
    pub fn run(command: &str, args: &[String], data: &Path, input: &Input) -> bool {
        match command {
            // KOTOR's bare meshes and the live heads in clay, front, three-quarter and side: `look <data> <out.png> [head...]`
            "look" => {
                let out = args[4].clone();
                let heads: Vec<String> = if args.len() > 5 { args[5..].to_vec() } else { names(input) };
                let live = Live::load(input, data);
                let frame = crate::measure::Frame::head();
                let shots = [FRONT, THREE, SIDE];
                let tiles: Vec<image::RgbImage> = parallel(heads.len() * 2, threads(), |q| {
                    let name = &heads[q / 2];
                    let mut row = Vec::new();
                    if q % 2 == 0 {
                        let (v, f, _) = kotor_bare(input, name);
                        for s in &shots {
                            row.push(crate::render::clay(&v, &f, s, &[], &[]));
                        }
                        crate::render::label(&mut row[0], &format!("{} kotor", name.replace('_', " ")));
                    } else {
                        let p: Vec<V3> = live.live(name).iter().map(|&x| frame.local(x)).collect();
                        for s in &shots {
                            row.push(crate::render::clay(&p, &live.mesh.triangles, s, &[], &[]));
                        }
                        crate::render::label(&mut row[0], "live");
                    }
                    crate::render::sheet(&row, 3)
                });
                crate::render::sheet(&tiles, 4).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }

            // KOTOR's bare mesh in clay, front and side, with KOTOR's outlines in red and ours from folders of kit-order bins (or
            // "live") over it: `overlay <data> <out.png> <columns> <label=dir|live>... -- <head>...|all`
            "overlay" => {
                let out = args[4].clone();
                let columns: usize = args[5].parse().unwrap();
                let split = args.iter().position(|a| a == "--").unwrap();
                let palette = [[255, 220, 60], [80, 230, 110], [80, 200, 255], [240, 120, 240]];
                let variants: Vec<(String, String)> = args[6..split].iter().map(|v| { let (a, b) = v.split_once('=').unwrap(); (a.to_string(), b.to_string()) }).collect();
                let heads: Vec<String> = if args[split + 1] == "all" { names(input) } else { args[split + 1..].to_vec() };
                let live = variants.iter().any(|v| v.1 == "live").then(|| Live::load(input, data));
                let mesh = input.head_mesh();
                let tiles: Vec<image::RgbImage> = parallel(heads.len(), threads(), |q| {
                    let name = &heads[q];
                    let (v, f, _) = kotor_bare(input, name);
                    let mut fr = crate::render::clay(&v, &f, &FRONT, &[], &[]);
                    let mut sd = crate::render::clay(&v, &f, &SIDE, &[], &[]);
                    let mut text = name.replace('_', " ");
                    match kotor(input, name) {
                        Err(e) => text.push_str(&format!(" {e}")),
                        Ok(k) => {
                            draw_lips(&mut fr, &mut sd, &k, &k, [235, 60, 50], 1, true);
                            for (i, (label, dir)) in variants.iter().enumerate() {
                                let o = if dir == "live" {
                                    let l = live.as_ref().unwrap();
                                    let p: Vec<V3> = l.live(name).iter().map(|&x| l.frame.local(x)).collect();
                                    measure_ours(&p, &mesh.triangles).ok()
                                } else {
                                    ours(Path::new(dir), name, &mesh.triangles)
                                };
                                let Some(o) = o else { continue };
                                draw_lips(&mut fr, &mut sd, &k, &o, palette[i % 4], 0, false);
                                let g: Vec<String> = (0..3).map(|r| reading_gap(&k, &o, r).map_or("-".into(), |g| format!("{g:.1}"))).collect();
                                text.push_str(&format!(" {}:{}", label.chars().next().unwrap(), g.join("/")));
                            }
                        }
                    }
                    crate::render::label(&mut fr, &text);
                    crate::render::sheet(&[fr, sd], 2)
                });
                crate::render::sheet(&tiles, columns).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // the four checks (1, 3 and 4's numbers): KOTOR on its bare mesh and subdivided twice, and our heads from folders of
            // kit-order bins or "live": `checks <data> <out.json> [label=dir|live]...`
            "checks" => {
                let out = args[4].clone();
                let variants: Vec<(String, String)> = args[5..].iter().map(|v| { let (a, b) = v.split_once('=').unwrap(); (a.to_string(), b.to_string()) }).collect();
                let live = variants.iter().any(|v| v.1 == "live").then(|| Live::load(input, data));
                let mesh = input.head_mesh();
                let all = names(input);
                let rows: Vec<(String, Option<Lips>, Option<Lips>, Vec<Option<Lips>>)> = parallel(all.len(), threads(), |q| {
                    let name = &all[q];
                    let k = kotor(input, name);
                    if let Err(e) = &k {
                        eprintln!("{name}: {e}");
                    }
                    let s = subdivided(input, name).ok();
                    let o = variants.iter().map(|(_, dir)| if dir == "live" {
                        let l = live.as_ref().unwrap();
                        let p: Vec<V3> = l.live(name).iter().map(|&x| l.frame.local(x)).collect();
                        measure_ours(&p, &mesh.triangles).ok()
                    } else { ours(Path::new(dir), name, &mesh.triangles) }).collect();
                    (name.clone(), k.ok(), s, o)
                });
                let mut report = serde_json::Map::new();
                for r in 0..READINGS.len() {
                    let mut wob: Vec<f64> = rows.iter().filter_map(|(_, k, s, _)| reading_gap(k.as_ref()?, s.as_ref()?, r)).collect();
                    let mut sides: Vec<f64> = rows.iter().filter_map(|(_, k, _, _)| { let k = k.as_ref()?; Some(crate::chin_neck::gap(k.outlines[r][0].as_ref()?, k.outlines[r][1].as_ref()?)) }).collect();
                    let ks: Vec<(&String, &Lips)> = rows.iter().filter_map(|(n, k, _, _)| Some((n, k.as_ref()?))).collect();
                    let mut pairs = Vec::new();
                    for i in 0..ks.len() {
                        for j in i + 1..ks.len() {
                            if let Some(g) = reading_gap(ks[i].1, ks[j].1, r) { pairs.push(g); }
                        }
                    }
                    let (mut own_sub, mut read_sub, mut own_side, mut read_side) = (0, 0, 0, 0);
                    for (n, _, s, _) in &rows {
                        let Some(s) = s else { continue };
                        if !ks.iter().any(|(m, _)| *m == n) { continue; }
                        let Some(a) = &s.outlines[r][0] else { continue };
                        read_sub += 1;
                        let best = ks.iter().filter_map(|(m, k)| Some((*m, crate::chin_neck::gap(a, k.outlines[r][0].as_ref()?)))).min_by(|a, b| a.1.total_cmp(&b.1));
                        if best.is_some_and(|b| b.0 == n) { own_sub += 1; }
                    }
                    if r > 0 {
                        for (n, k) in &ks {
                            let (Some(right), Some(_)) = (&k.outlines[r][0], &k.outlines[r][1]) else { continue };
                            read_side += 1;
                            let best = ks.iter().filter_map(|(m, o)| Some((*m, crate::chin_neck::gap(right, o.outlines[r][1].as_ref()?)))).min_by(|a, b| a.1.total_cmp(&b.1));
                            if best.is_some_and(|b| b.0 == *n) { own_side += 1; }
                        }
                    }
                    let ours_r: Vec<serde_json::Value> = variants.iter().enumerate().map(|(vi, (label, _))| {
                        let mut g: Vec<f64> = rows.iter().filter_map(|(_, k, _, o)| reading_gap(k.as_ref()?, o[vi].as_ref()?, r)).collect();
                        json!({"label": label, "heads": g.len(), "median": pct(&mut g, 0.5), "p95": pct(&mut g, 0.95)})
                    }).collect();
                    let row = json!({
                        "reading": READINGS[r], "kotor_heads": ks.len(),
                        "wobble_subdivided": {"heads": wob.len(), "median": pct(&mut wob, 0.5), "p95": pct(&mut wob, 0.95)},
                        "wobble_sides": {"heads": sides.len(), "median": pct(&mut sides, 0.5), "p95": pct(&mut sides, 0.95)},
                        "between_heads": {"pairs": pairs.len(), "p5": pct(&mut pairs, 0.05), "median": pct(&mut pairs, 0.5)},
                        "check3_subdivided_finds_itself": [own_sub, read_sub],
                        "check3_right_side_finds_own_left": [own_side, read_side],
                        "ours": ours_r,
                    });
                    println!("{row}");
                    report.insert(READINGS[r].to_string(), row);
                }
                // the proportions: wobble (subdivided, and left against right where a proportion has sides) against KOTOR's spread
                let mut props = Vec::new();
                for (i, (key, words)) in PROPORTIONS.iter().enumerate() {
                    let mut wob: Vec<f64> = rows.iter().filter_map(|(_, k, s, _)| { let d = k.as_ref()?.proportions[i] - s.as_ref()?.proportions[i]; d.is_finite().then_some(d.abs()) }).collect();
                    let mut vals: Vec<f64> = rows.iter().filter_map(|(_, k, _, _)| k.as_ref().map(|k| k.proportions[i])).filter(|v| v.is_finite()).collect();
                    let (p10, p90) = (pct(&mut vals, 0.1), pct(&mut vals, 0.9));
                    let ours_p: Vec<serde_json::Value> = variants.iter().enumerate().map(|(vi, (label, _))| {
                        let mut g: Vec<f64> = rows.iter().filter_map(|(_, k, _, o)| { let d = k.as_ref()?.proportions[i] - o[vi].as_ref()?.proportions[i]; d.is_finite().then_some(d.abs()) }).collect();
                        let mut b: Vec<f64> = rows.iter().filter_map(|(_, k, _, o)| { let d = o[vi].as_ref()?.proportions[i] - k.as_ref()?.proportions[i]; d.is_finite().then_some(d) }).collect();
                        json!({"label": label, "median_gap": pct(&mut g, 0.5), "median_bias": pct(&mut b, 0.5)})
                    }).collect();
                    let row = json!({"key": key, "words": words, "heads": vals.len(), "kotor_p10": p10, "kotor_median": pct(&mut vals, 0.5), "kotor_p90": p90, "spread": p90 - p10, "wobble_median": pct(&mut wob, 0.5), "wobble_p95": pct(&mut wob, 0.95), "ours": ours_p});
                    println!("{row}");
                    props.push(row);
                }
                let mut halves: Vec<f64> = rows.iter().filter_map(|(_, k, _, _)| { let k = k.as_ref()?; let d = 2.0 * (k.halves[0] - k.halves[1]); d.is_finite().then_some(d.abs()) }).collect();
                let row = json!({"width_left_against_right": {"median": pct(&mut halves, 0.5), "p95": pct(&mut halves, 0.95)}});
                println!("{row}");
                props.push(row);
                report.insert("proportions".into(), json!(props));
                let per_head: Vec<serde_json::Value> = rows.iter().map(|(n, k, s, o)| json!({
                    "name": n,
                    "kotor": k.as_ref().map(|k| k.proportions.clone()),
                    "subdivided": s.as_ref().map(|s| s.proportions.clone()),
                    "wobble": (0..3).map(|r| k.as_ref().zip(s.as_ref()).and_then(|(k, s)| reading_gap(k, s, r))).collect::<Vec<_>>(),
                    "ours": variants.iter().enumerate().map(|(vi, (label, _))| json!({"label": label, "gaps": (0..3).map(|r| k.as_ref().zip(o[vi].as_ref()).and_then(|(k, o)| reading_gap(k, o, r))).collect::<Vec<_>>(), "proportions": o[vi].as_ref().map(|o| o.proportions.clone())})).collect::<Vec<_>>(),
                })).collect();
                let ks: Vec<(&String, &Lips)> = rows.iter().filter_map(|(n, k, _, _)| Some((n, k.as_ref()?))).collect();
                let mut pairs: Vec<(f64, String, String)> = Vec::new();
                for i in 0..ks.len() {
                    for j in i + 1..ks.len() {
                        let g: Vec<f64> = (0..3).filter_map(|r| reading_gap(ks[i].1, ks[j].1, r)).collect();
                        if g.len() == 3 { pairs.push((g.iter().sum::<f64>() / 3.0, ks[i].0.clone(), ks[j].0.clone())); }
                    }
                }
                pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
                report.insert("per_head".into(), json!(per_head));
                report.insert("most_alike".into(), json!(pairs.iter().take(6).collect::<Vec<_>>()));
                report.insert("most_different".into(), json!(pairs.iter().rev().take(6).collect::<Vec<_>>()));
                std::fs::write(&out, serde_json::to_string_pretty(&serde_json::Value::Object(report)).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }

            // the lip sliders made on the base head: `make <data> <out sliders.json>`
            "make" => {
                let live = Live::load(input, data);
                let base = live.smoother.smooth(&live.mesh.base, crate::set::SMOOTHING);
                let set = fields::make(&live.mesh, &base);
                for sl in &set.sliders {
                    eprintln!("{}: {} vertices", sl.name, sl.vertices.len());
                }
                std::fs::write(&args[4], serde_json::to_string(&set).unwrap()).unwrap();
                eprintln!("wrote {}", args[4]);
                true
            }
            // each lip slider's effect on every measure at -2 and +2 on the heads named, as the change per unit:
            // `gain <data> <sliders.json> <head>...`
            "gain" => {
                let live = live_lips(input, data, &args[4]);
                let n = SLIDERS.len();
                for name in &args[5..] {
                    let k0 = measure_ours(&live.with(name, &vec![0.0; n]), &live.mesh.triangles).unwrap();
                    println!("{name}: {}", PROPORTIONS.iter().map(|p| p.0).collect::<Vec<_>>().join(" "));
                    for i in 0..n {
                        let at = |v: f64| { let mut x = vec![0.0; n]; x[i] = v; measure_ours(&live.with(name, &x), &live.mesh.triangles).ok() };
                        let (lo, hi) = (at(-2.0), at(2.0));
                        let row: Vec<String> = (0..PROPORTIONS.len()).map(|j| match (&lo, &hi) { (Some(a), Some(b)) => format!("{:6.2}", (b.proportions[j] - a.proportions[j]) / 4.0), _ => "  -   ".into() }).collect();
                        println!("  {:22} {}", SLIDERS[i].0, row.join(" "));
                    }
                    let _ = k0;
                }
                true
            }
            // every lip slider at both ends (and 0) on the heads named, in clay, front, three-quarter and side, with turned triangles in
            // red: `ends <data> <sliders.json> <out.png> <amount> <head>...`
            "ends" => {
                let live = live_lips(input, data, &args[4]);
                let out = args[5].clone();
                // "range" draws each slider at its own low and high ends
                let amount: f64 = args[6].parse().unwrap_or(f64::NAN);
                let heads = args[7..].to_vec();
                let n = SLIDERS.len();
                let all: Vec<u32> = (0..live.mesh.base.len() as u32).collect();
                let shots = [FRONT, THREE, SIDE];
                let jobs: Vec<(String, usize, f64)> = heads.iter().flat_map(|h| (0..n).flat_map(move |i| [-1.0, 1.0].map(|s| (h.clone(), i, s)))).collect();
                let tiles: Vec<image::RgbImage> = parallel(jobs.len(), threads(), |q| {
                    let (name, i, sign) = &jobs[q];
                    let zero: Vec<V3> = live.with(name, &vec![0.0; n]).iter().map(|&p| live.frame.world(p)).collect();
                    let mut x = vec![0.0; n];
                    let r = &live.set.sliders[live.lips_first + *i];
                    x[*i] = if amount.is_nan() { if *sign < 0.0 { r.low } else { r.high } } else { sign * amount };
                    let shown = x[*i];
                    let p = live.with(name, &x);
                    let world: Vec<V3> = p.iter().map(|&q| live.frame.world(q)).collect();
                    let zl: Vec<V3> = zero.iter().map(|&q| live.frame.local(q)).collect();
                    let sl = slit(&zl, &live.mesh.triangles, &measure_ours(&zl, &live.mesh.triangles).unwrap());
                    let red = folds(&crate::check::Checker::from_parts(&live.mesh.triangles, &vec![false; world.len()], &zero, &all), &world, &sl);
                    let count = red.iter().filter(|r| **r).count();
                    if count > 0 {
                        let hid = red.iter().enumerate().filter(|(_, r)| **r).filter(|(t, _)| live.mesh.triangles[*t].iter().any(|&i| live.mesh.hidden[live.mesh.group[i as usize] as usize])).count();
                        eprintln!("  of which {hid} touch a hidden vertex");
                        let at: Vec<String> = red.iter().enumerate().filter(|(_, r)| **r).take(4).map(|(t, _)| { let c = live.mesh.triangles[t].map(|i| p[i as usize]); format!("({:.1},{:.1},{:.1})", (c[0][0] + c[1][0] + c[2][0]) / 3.0, (c[0][1] + c[1][1] + c[2][1]) / 3.0, (c[0][2] + c[1][2] + c[2][2]) / 3.0) }).collect();
                        eprintln!("{name} {} {:+}: {count} folds at {}", SLIDERS[*i].0, shown, at.join(" "));
                    }
                    let mut row: Vec<image::RgbImage> = shots.iter().map(|s| crate::render::clay(&p, &live.mesh.triangles, s, &red, &[])).collect();
                    crate::render::label(&mut row[0], &format!("{} {} {:+} folds {count}", name.replace('_', " "), SLIDERS[*i].0, shown));
                    crate::render::sheet(&row, 3)
                });
                crate::render::sheet(&tiles, 2).save(&out).unwrap();
                eprintln!("wrote {out}");
                true
            }

            // each head solved toward KOTOR's lips with the lip sliders on top of its live values, with every turned triangle (against
            // the live head) and a moved nose base costed: `solve <data> <sliders.json> <out.json> <head>...|all`
            "solve" => {
                let live = live_lips(input, data, &args[4]);
                let out = args[5].clone();
                let heads: Vec<String> = if args[6] == "all" { names(input) } else { args[6..].to_vec() };
                let n = SLIDERS.len();
                let ranges = live.lip_ranges();
                let all: Vec<u32> = (0..live.mesh.base.len() as u32).collect();
                let rows: Vec<serde_json::Value> = parallel(heads.len(), threads(), |q| {
                    let name = &heads[q];
                    let Ok(k) = kotor(input, name) else { return json!({"name": name, "error": "KOTOR's lips cannot be read"}) };
                    let zero = live.with(name, &vec![0.0; n]);
                    let zw: Vec<V3> = zero.iter().map(|&p| live.frame.world(p)).collect();
                    let checker = crate::check::Checker::from_parts(&live.mesh.triangles, &vec![false; zw.len()], &zw, &all);
                    let Ok(m0) = measure_ours(&zero, &live.mesh.triangles) else { return json!({"name": name, "error": "our lips cannot be read"}) };
                    // LIPS_STRICT=1 counts every turned triangle, those inside the slit too
                    let sl = if std::env::var("LIPS_STRICT").is_ok() { vec![false; live.mesh.triangles.len()] } else { slit(&zero, &live.mesh.triangles, &m0) };
                    let live_gaps: Vec<Option<f64>> = (0..3).map(|r| reading_gap(&k, &m0, r)).collect();
                    let parts = |x: &[f64]| -> (f64, usize, Option<Lips>) {
                        let p = live.with(name, x);
                        let folds = folds(&checker, &p.iter().map(|&q| live.frame.world(q)).collect::<Vec<_>>(), &sl).iter().filter(|r| **r).count();
                        match measure_ours(&p, &live.mesh.triangles) {
                            Ok(m) => {
                                let nose = ((m.nose[0] - m0.nose[0]).powi(2) + (m.nose[1] - m0.nose[1]).powi(2)).sqrt();
                                // an outline worse than today's costs steeply, since the guard refuses it
                                let worse: f64 = (0..3).map(|r| match (reading_gap(&k, &m, r), live_gaps[r]) { (Some(a), Some(b)) => (50.0 * ((a - b).max(0.0) / OUTLINE_MARKS[r]).powi(2)), _ => 0.0 }).sum();
                                (cost(&k, &m) + worse + (nose / 0.5).powi(2) + 3.0 * folds as f64, folds, Some(m))
                            }
                            Err(_) => (1e9, folds, None),
                        }
                    };
                    let mut x = vec![0.0; n];
                    let mut best = parts(&x).0;
                    // a coarse look at each slider over its range, then finer steps
                    for _ in 0..2 {
                        for i in 0..n {
                            for t in 0..=8 {
                                let mut y = x.clone();
                                y[i] = ranges[i].0 + (ranges[i].1 - ranges[i].0) * t as f64 / 8.0;
                                let c = parts(&y).0;
                                if c < best { best = c; x = y; }
                            }
                        }
                    }
                    for pass in 0..4 {
                        for i in 0..n {
                            let step = (ranges[i].1 - ranges[i].0) / 16.0 / 2f64.powi(pass);
                            for dir in [-1.0, 1.0] {
                                loop {
                                    let mut y = x.clone();
                                    y[i] = (x[i] + dir * step).clamp(ranges[i].0, ranges[i].1);
                                    if y[i] == x[i] { break; }
                                    let c = parts(&y).0;
                                    if c < best { best = c; x = y; } else { break; }
                                }
                            }
                        }
                    }
                    let gaps = |o: &Lips| (0..3).map(|r| reading_gap(&k, o, r)).collect::<Vec<_>>();
                    // the guard: of the tries below, the cheapest with no turned triangle outside the slit, no outline worse than live
                    // and no lip height past KOTOR's from the side today's head is on. The tries are shares of the move, each also
                    // with the two lip heights left out, and the whole move with one slider left out.
                    let g0 = gaps(&m0);
                    let solved = x.clone();
                    let mut share = 0.0;
                    let mut tries: Vec<(f64, Vec<usize>)> = [1.0, 0.875, 0.75, 0.625, 0.5, 0.375, 0.25, 0.125].iter().flat_map(|&sh| [(sh, vec![]), (sh, vec![5, 6])]).collect();
                    tries.extend((0..n).map(|i| (1.0, vec![i])));
                    let mut best_try = f64::INFINITY;
                    for (sh, out) in tries {
                        let mut y: Vec<f64> = solved.iter().map(|v| v * sh).collect();
                        for &i in &out {
                            y[i] = 0.0;
                        }
                        let (c, f, m) = parts(&y);
                        let Some(m) = m else { continue };
                        let g = gaps(&m);
                        let mut worse = g.iter().zip(&g0).any(|(a, b)| match (a, b) { (Some(a), Some(b)) => *a > *b, (None, Some(_)) => true, _ => false });
                        for i in [1, 2] {
                            let (kv, lv, sv) = (k.proportions[i], m0.proportions[i], m.proportions[i]);
                            if (lv <= kv && sv > kv) || (lv >= kv && sv < kv) {
                                worse = true;
                            }
                        }
                        if f == 0 && !worse && c < best_try {
                            best_try = c;
                            share = sh;
                            x = y;
                        }
                    }
                    if share == 0.0 {
                        x = vec![0.0; n];
                    }
                    let (c1, f1, m1) = parts(&x);
                    let m1 = m1.unwrap();
                    eprintln!("{name}: cost {:.1} -> {c1:.1}, folds {f1}", cost(&k, &m0));
                    json!({"name": name, "values": x, "solved_values": solved, "share": share, "cost_live": cost(&k, &m0), "cost": c1, "folds": f1, "gaps_live": gaps(&m0), "gaps": gaps(&m1), "kotor": k.proportions, "live": m0.proportions, "solved": m1.proportions})
                });
                std::fs::write(&out, serde_json::to_string_pretty(&json!({"sliders": SLIDERS.iter().map(|s| s.0).collect::<Vec<_>>(), "heads": rows})).unwrap()).unwrap();
                eprintln!("wrote {out}");
                true
            }
            // the power of existing sliders: each head solved toward KOTOR's lips with only the sliders of the zones named, nothing
            // held: `power <data> <zones,comma> <head>...`
            "power" => {
                let zones: Vec<&str> = args[4].split(',').collect();
                let live = Live::load(input, data);
                let idx: Vec<usize> = (0..live.set.sliders.len()).filter(|&i| zones.contains(&live.set.sliders[i].zone.as_str())).collect();
                let heads = args[5..].to_vec();
                let rows: Vec<String> = parallel(heads.len(), threads(), |q| {
                    let name = &heads[q];
                    let k = kotor(input, name).unwrap();
                    let v0 = live.values(name);
                    let measure_at = |v: &[f64]| { let p: Vec<V3> = live.build(name, v).iter().map(|&x| live.frame.local(x)).collect(); measure_ours(&p, &live.mesh.triangles).ok() };
                    let f = |v: &[f64]| measure_at(v).map_or(1e9, |o| cost(&k, &o));
                    let mut v = v0.clone();
                    let mut best = f(&v);
                    for pass in 0..3 {
                        for &i in &idx {
                            let s = &live.set.sliders[i];
                            let step = (s.high - s.low) / 6.0 / 2f64.powi(pass);
                            for dir in [-1.0, 1.0] {
                                loop {
                                    let mut y = v.clone();
                                    y[i] = (v[i] + dir * step).clamp(s.low, s.high);
                                    if y[i] == v[i] { break; }
                                    let c = f(&y);
                                    if c < best { best = c; v = y; } else { break; }
                                }
                            }
                        }
                    }
                    let (a, b) = (measure_at(&v0).unwrap(), measure_at(&v).unwrap());
                    let g = |o: &Lips| (0..3).map(|r| reading_gap(&k, o, r).map_or("-".into(), |g| format!("{g:.2}"))).collect::<Vec<_>>().join(" ");
                    let moved: Vec<String> = idx.iter().filter(|&&i| (v[i] - v0[i]).abs() > 1e-9).map(|&i| format!("{} {:.2}->{:.2}", live.set.sliders[i].name, v0[i], v[i])).collect();
                    format!("{name}: cost {:.1} -> {:.1}; gaps live {} -> {}; width {:.1} -> {:.1} (KOTOR {:.1}); corner depth {:.1} -> {:.1} (KOTOR {:.1})\n  {}", f(&v0), best, g(&a), g(&b), a.proportions[0], b.proportions[0], k.proportions[0], a.proportions[6], b.proportions[6], k.proportions[6], moved.join(", "))
                });
                for r in rows { println!("{r}"); }
                true
            }
            // one head's lips at given lip values: gaps, proportions, cost and folds: `try <data> <sliders.json> <head> <v,v,...>`
            "try" => {
                let live = live_lips(input, data, &args[4]);
                let name = &args[5];
                let x: Vec<f64> = args[6].split(',').map(|v| v.parse().unwrap()).collect();
                let k = kotor(input, name).unwrap();
                let all: Vec<u32> = (0..live.mesh.base.len() as u32).collect();
                let zero = live.with(name, &vec![0.0; x.len()]);
                let m0 = measure_ours(&zero, &live.mesh.triangles).unwrap();
                let zw: Vec<V3> = zero.iter().map(|&p| live.frame.world(p)).collect();
                let checker = crate::check::Checker::from_parts(&live.mesh.triangles, &vec![false; zw.len()], &zw, &all);
                let sl = slit(&zero, &live.mesh.triangles, &m0);
                let p = live.with(name, &x);
                let m = measure_ours(&p, &live.mesh.triangles).unwrap();
                let f = folds(&checker, &p.iter().map(|&q| live.frame.world(q)).collect::<Vec<_>>(), &sl).iter().filter(|r| **r).count();
                println!("gaps {:?} cost {:.2} folds {f}", (0..3).map(|r| reading_gap(&k, &m, r)).collect::<Vec<_>>(), cost(&k, &m));
                if std::env::var("LIPS_COLS").is_ok() {
                    for c in m.columns[0].iter().step_by(2).take(12) {
                        println!("  x {:4.1} line {:6.2} crest {:6.2} border {:6.2} {:6.2}", c.x, c.line[1], c.upper[1], c.border_up, c.border_down);
                    }
                }
                println!("kotor {:?}\nours  {:?}", k.proportions.iter().map(|v| (v * 100.0).round() / 100.0).collect::<Vec<_>>(), m.proportions.iter().map(|v| (v * 100.0).round() / 100.0).collect::<Vec<_>>());
                true
            }
            // one sheet per head: KOTOR, today's head and ours in clay, front and side, each with KOTOR's outlines in red and its own
            // over them (today yellow, ours green), lined up on that head's line between the lips: `sheet <data> <out prefix>
            // <today dir> <ours dir> <head>...`
            "sheet" => {
                let prefix = args[4].clone();
                let (today, new) = (Path::new(&args[5]), Path::new(&args[6]));
                let mesh = input.head_mesh();
                let frame = crate::measure::Frame::head();
                for name in &args[7..] {
                    let (v, f, _) = kotor_bare(input, name);
                    let k = kotor(input, name).unwrap();
                    let mut tiles = Vec::new();
                    let mut col = |p: &[V3], tris: &[[u32; 3]], own: Option<&Lips>, colour: [u8; 3], label: &str| {
                        let mut fr = crate::render::clay(p, tris, &FRONT, &[], &[]);
                        let mut sd = crate::render::clay(p, tris, &SIDE, &[], &[]);
                        let on = own.unwrap_or(&k);
                        draw_lips(&mut fr, &mut sd, on, &k, [235, 60, 50], 1, true);
                        if let Some(o) = own {
                            draw_lips(&mut fr, &mut sd, o, o, colour, 0, true);
                        }
                        crate::render::label(&mut fr, label);
                        tiles.push(fr);
                        tiles.push(sd);
                    };
                    col(&v, &f, None, [0, 0, 0], &format!("{} KOTOR", name.replace('_', " ")));
                    for (dir, colour, word) in [(today, [255, 220, 60], "today"), (new, [80, 230, 110], "ours")] {
                        let p: Vec<V3> = crate::chin_neck::tools::read_bin(dir, name).unwrap();
                        let o = measure_ours(&p, &mesh.triangles).unwrap();
                        let g: Vec<String> = (0..3).map(|r| reading_gap(&k, &o, r).map_or("-".into(), |g| format!("{g:.2}"))).collect();
                        col(&p, &mesh.triangles, Some(&o), colour, &format!("{word} {}", g.join(" ")));
                    }
                    let _ = frame;
                    // rows: front then side; columns: KOTOR, today, ours
                    let order: Vec<image::RgbImage> = [0, 2, 4, 1, 3, 5].iter().map(|&i| tiles[i].clone()).collect();
                    let out = format!("{prefix}{name}.png");
                    crate::render::sheet(&order, 3).save(&out).unwrap();
                    eprintln!("wrote {out}");
                }
                true
            }
            // the heads of a solve written as kit-order bins (metres): `bins <data> <sliders.json> <solve.json> <out dir>`; with
            // the solve "-" every head is written live (lip values 0)
            "bins" => {
                let live = live_lips(input, data, &args[4]);
                let out = Path::new(&args[6]);
                std::fs::create_dir_all(out).unwrap();
                let n = live.set.sliders.len() - live.lips_first;
                let table: std::collections::HashMap<String, Vec<f64>> = if args[5] == "-" {
                    Default::default()
                } else {
                    let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[5]).unwrap()).unwrap();
                    d["heads"].as_array().unwrap().iter().filter_map(|h| Some((h["name"].as_str()?.to_string(), serde_json::from_value(h["values"].clone()).ok()?))).collect()
                };
                let heads = live.names.clone();
                let _: Vec<()> = parallel(heads.len(), threads(), |q| {
                    let name = &heads[q];
                    let x = table.get(name).cloned().unwrap_or(vec![0.0; n]);
                    let p: Vec<V3> = live.with(name, &x).iter().map(|&q| live.frame.world(q)).collect();
                    crate::chin_neck::tools::write_bin(out, name, &p);
                });
                eprintln!("wrote {}", out.display());
                true
            }
            // every head at its lip values in a solve's output, against the same head with the lips at 0: turned triangles outside the
            // slit, turned triangles anywhere, where they are, and the outlines against KOTOR: `verify <data> <sliders.json> <solve.json>`
            "verify" => {
                let live = live_lips(input, data, &args[4]);
                let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[5]).unwrap()).unwrap();
                let table: std::collections::HashMap<String, Vec<f64>> = d["heads"].as_array().unwrap().iter().filter_map(|h| Some((h["name"].as_str()?.to_string(), serde_json::from_value(h["values"].clone()).ok()?))).collect();
                let n = SLIDERS.len();
                let all: Vec<u32> = (0..live.mesh.base.len() as u32).collect();
                let heads = live.names.clone();
                let rows: Vec<String> = parallel(heads.len(), threads(), |q| {
                    let name = &heads[q];
                    let x = table.get(name).cloned().unwrap_or(vec![0.0; n]);
                    let zero = live.with(name, &vec![0.0; n]);
                    let zw: Vec<V3> = zero.iter().map(|&p| live.frame.world(p)).collect();
                    let checker = crate::check::Checker::from_parts(&live.mesh.triangles, &vec![false; zw.len()], &zw, &all);
                    let Ok(m0) = measure_ours(&zero, &live.mesh.triangles) else { return format!("{name}: unread") };
                    let sl = slit(&zero, &live.mesh.triangles, &m0);
                    let p = live.with(name, &x);
                    let pw: Vec<V3> = p.iter().map(|&q| live.frame.world(q)).collect();
                    let marks = checker.marks(&pw);
                    let any: Vec<usize> = marks.iter().enumerate().filter(|(_, r)| **r).map(|(t, _)| t).collect();
                    let outside = any.iter().filter(|&&t| !sl[t]).count();
                    let at: Vec<String> = any.iter().take(4).map(|&t| { let c = live.mesh.triangles[t].map(|i| p[i as usize]); format!("({:.1},{:.1},{:.1}){}", (c[0][0] + c[1][0] + c[2][0]) / 3.0, (c[0][1] + c[1][1] + c[2][1]) / 3.0, (c[0][2] + c[1][2] + c[2][2]) / 3.0, if sl[t] { "s" } else { "" }) }).collect();
                    let k = kotor(input, name).ok();
                    let worse = match (&k, measure_ours(&p, &live.mesh.triangles)) { (Some(k), Ok(m)) => (0..3).any(|r| match (reading_gap(k, &m, r), reading_gap(k, &m0, r)) { (Some(a), Some(b)) => a > b + 1e-9, (None, Some(_)) => true, _ => false }), _ => false };
                    format!("{name:22} turned outside slit {outside} anywhere {} faults {} worse {worse} {}", any.len(), checker.faults(&pw).total(), at.join(" "))
                });
                for r in rows { println!("{r}"); }
                true
            }
            // the lip borders of every live head read two ways, from the shading and from the lip paint's border, and how many of
            // KOTOR's columns read a border: `borders <data>`
            "borders" => {
                let live = Live::load(input, data);
                let all = names(input);
                let rows: Vec<String> = parallel(all.len(), threads(), |q| {
                    let name = &all[q];
                    let p: Vec<V3> = live.live(name).iter().map(|&x| live.frame.local(x)).collect();
                    let Ok(a) = measure_ours(&p, &live.mesh.triangles) else { return format!("{name}: unread") };
                    let share = |l: &Lips, up: bool| { let c: Vec<&Column> = l.columns.iter().flatten().collect(); c.iter().filter(|c| if up { c.border_up } else { c.border_down }.is_finite()).count() as f64 / c.len() as f64 };
                    let k = kotor(input, name).ok();
                    format!("{name:22} line ours {:6.1} paint {:6.1} kotor {:6.1} guide {:6.1} | borders read kotor {:4.2} {:4.2} ours {:4.2} {:4.2} | heights ours {:5.2} {:5.2} kotor {} | width ours {:4.1} kotor {}",
                        a.columns[0][0].line[1], opening_height(&p), k.as_ref().map_or(f64::NAN, |k| k.columns[0][0].line[1]), guide(input, name).unwrap_or(f64::NAN),
                        k.as_ref().map_or(f64::NAN, |k| share(k, true)), k.as_ref().map_or(f64::NAN, |k| share(k, false)), share(&a, true), share(&a, false),
                        a.proportions[1], a.proportions[2], k.as_ref().map_or("-".into(), |k| format!("{:5.2} {:5.2}", k.proportions[1], k.proportions[2])), a.proportions[0], k.as_ref().map_or("-".into(), |k| format!("{:4.1}", k.proportions[0])))
                });
                for r in rows { println!("{r}"); }
                true
            }
            // one live head's measure with the paint's borders, and the border curves: `ourcols <data> <head>`
            "ourcols" => {
                let live = Live::load(input, data);
                let p: Vec<V3> = live.live(&args[4]).iter().map(|&x| live.frame.local(x)).collect();
                let b = our_borders(&p);
                for (i, c) in b.iter().enumerate() {
                    println!("curve {i}: {}", c.iter().step_by(3).map(|q| format!("({:.1},{:.1},{:.1})", q[0], q[1], q[2])).collect::<Vec<_>>().join(" "));
                }
                let k = measure_ours(&p, &live.mesh.triangles).unwrap();
                for c in k.columns[0].iter().step_by(4) {
                    println!("{:5.1} line {:6.1} {:5.1} up {:6.1} {:5.1} down {:6.1} {:5.1} borders {:6.1} {:6.1} valley {:4.1}", c.x, c.line[1], c.line[0], c.upper[1], c.upper[0], c.lower[1], c.lower[0], c.border_up, c.border_down, c.valley);
                }
                println!("{:?}", k.proportions);
                true
            }
            // one head's measure: `cols <data> <head>`
            "cols" => {
                let k = kotor(input, &args[4]).unwrap();
                println!("nose {:?} fold {:?}", k.nose, k.fold);
                for c in &k.columns[0] {
                    println!("{:5.1} line {:6.1} {:5.1} up {:6.1} {:5.1} down {:6.1} {:5.1} borders {:6.1} {:6.1} valley {:4.1}", c.x, c.line[1], c.line[0], c.upper[1], c.upper[0], c.lower[1], c.lower[0], c.border_up, c.border_down, c.valley);
                }
                println!("{:?}", k.proportions);
                true
            }
            // the middle cut's front depth every mm, KOTOR's and live: `probe <data> <head> [x]`
            "probe" => {
                let name = &args[4];
                let x: f64 = args.get(5).map(|a| a.parse().unwrap()).unwrap_or(0.0);
                let live = Live::load(input, data);
                let (v, f, _) = kotor_bare(input, name);
                let k = local(&v, &f);
                let p: Vec<V3> = live.live(name).iter().map(|&q| live.frame.local(q)).collect();
                let o = local(&p, &live.mesh.triangles);
                let (ck, co) = (k.front(0, x, 1, -130.0, 360), o.front(0, x, 1, -130.0, 360));
                for i in (0..360).step_by(4) {
                    println!("{:7.1} {:7.2} {:7.2}", -130.0 + i as f64 * 0.25, ck[i], co[i]);
                }
                true
            }
            _ => false,
        }
    }

    /// A surface of a mesh in the frame of the eyes, near the mouth.
    pub fn local(v: &[V3], f: &[[u32; 3]]) -> Surface {
        let t = f.iter().map(|t| t.map(|i| v[i as usize])).filter(|t| t.iter().any(|&p| WINDOW.contains(p))).collect();
        Surface::from_local_in(t, &WINDOW)
    }
}
