//! The brow ridge, the forehead and the temples as a feature of the slider kit, and the cuts both upper face features share.
//!
//! Everything is measured in the frame of the eyes (`measure::Frame`), on any triangle mesh. The eyes' centres are the origin, so a
//! depth is how far in front of the eyeballs' centres a place lies.
//!
//! The brow ridge is read on upright cuts above each eye: its crest is the front-most place of the cut between 2 and 34 mm above the
//! eyes. The forehead is read on the cut down the middle of the face and on level cuts across it. The forehead's zone stops 80 to 95 mm
//! above the eyes, below the top of the head, because the skull's size and shape belong to the global part of the head (roadmap A).

use crate::V3;
use crate::feature::{BumpRule, Feature, Guard, KotorExtras, Measured, OutlineDef, Shot, Spec};
use crate::measure::{Cut, Frame, Proportion, Surface, Window, resample};
use crate::mesh::HeadMesh;
use crate::zone::Zone;
use serde_json::json;

pub struct Brow;

pub static BROW: Brow = Brow;

pub const PROPORTIONS: [Proportion; 10] = [
    Proportion { key: "brow_projection", label: "How far the brow ridge stands out over the eyes", low: "flat brow", high: "heavy brow", unit: "mm" },
    Proportion { key: "brow_height", label: "How high the brow ridge's lower edge sits above the eyes", low: "low brow", high: "high brow", unit: "mm" },
    Proportion { key: "brow_slant", label: "Slant of the brow ridge, seen from the front", low: "outer end lower", high: "outer end higher", unit: "degrees" },
    Proportion { key: "brow_middle", label: "How far the brow between the eyes stands out beyond the brow ridge over the eyes", low: "flat between the brows", high: "jutting between the brows", unit: "mm" },
    Proportion { key: "forehead_slope", label: "Slope of the forehead, seen from the side", low: "upright forehead", high: "sloping forehead", unit: "degrees" },
    Proportion { key: "forehead_curve", label: "How far the forehead falls back 30 mm to each side of the middle", low: "flat forehead", high: "rounded forehead", unit: "mm" },
    Proportion { key: "temple_width", label: "Width of the face at the temples", low: "narrow temples", high: "wide temples", unit: "mm" },
    Proportion { key: "brow_sweep", label: "How far the brow ridge's outer end lies behind its inner end", low: "brow flat across", high: "brow swept back", unit: "mm" },
    Proportion { key: "brow_bulge", label: "How far the brow ridge bulges out from the forehead above it", low: "smooth brow", high: "bulging brow", unit: "mm" },
    Proportion { key: "brow_edge", label: "How steeply the brow ridge's lower edge falls into the eye socket", low: "soft brow edge", high: "sharp brow edge", unit: "degrees" },
];

pub const OUTLINES: [OutlineDef; 2] = [
    OutlineDef { key: "brow_line", label: "Lower edge of the brow ridge, from the front", points: 8, widths: false },
    OutlineDef { key: "forehead_profile", label: "Forehead from the side", points: 16, widths: false },
];

const GROUPS: [&str; 3] = ["brow", "forehead", "temples"];

const SHOTS: [Shot; 4] = [
    Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-85.0, 85.0, -40.0, 100.0] },
    Shot { name: "three-quarter", yaw: 40.0, pitch: 0.0, pivot: 0.0, window: [-95.0, 75.0, -40.0, 100.0] },
    Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-110.0, 60.0, -40.0, 100.0] },
    Shot { name: "above", yaw: 0.0, pitch: -55.0, pivot: 20.0, window: [-85.0, 85.0, -30.0, 110.0] },
];

/// The window the measure reads.
const WINDOW: Window = Window { x: 95.0, y: [-15.0, 100.0], z: -80.0 };

/// The places across where the brow line is read, on each side, in mm from the middle.
const BROW_XS: [f64; 8] = [14.0, 18.0, 22.0, 26.0, 30.0, 34.0, 38.0, 42.0];

pub fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The front-most depth of the upright cut at `x`, every 0.25 mm of height from `y0` to `y1`, lightly blurred.
pub fn upright(surface: &Surface, x: f64, y0: f64, y1: f64) -> Cut {
    let n = ((y1 - y0) / crate::measure::STEP) as usize + 1;
    Cut { from: y0, z: surface.front(0, x, 1, y0, n) }.blurred(0.75)
}

/// The front-most depth of the level cut at height `y`, every 0.25 mm across from -100 to 100 mm, lightly blurred.
pub fn level(surface: &Surface, y: f64) -> Cut {
    Cut { from: -100.0, z: surface.front(1, y, 0, -100.0, 801) }.blurred(0.75)
}

/// The crest of a cut between `lo` and `hi`: the middle of the places within 0.3 mm of its front-most place, and that depth.
pub fn crest(c: &Cut, lo: f64, hi: f64) -> Option<(f64, f64)> {
    let ks: Vec<usize> = (c.index(lo)..=c.index(hi)).filter(|&k| !c.z[k].is_nan()).collect();
    let top = ks.iter().map(|&k| c.z[k]).fold(f64::NEG_INFINITY, f64::max);
    if !top.is_finite() {
        return None;
    }
    let near: Vec<usize> = ks.into_iter().filter(|&k| c.z[k] > top - 0.3).collect();
    Some((near.iter().map(|&k| c.u(k)).sum::<f64>() / near.len() as f64, top))
}

/// The line fitted to points (u, v): its slope dv/du.
pub fn slope(p: &[[f64; 2]]) -> f64 {
    let n = p.len() as f64;
    let (mu, mv) = (p.iter().map(|q| q[0]).sum::<f64>() / n, p.iter().map(|q| q[1]).sum::<f64>() / n);
    let (suu, suv) = p.iter().fold((0.0, 0.0), |(a, b), q| (a + (q[0] - mu).powi(2), b + (q[0] - mu) * (q[1] - mv)));
    suv / suu.max(1e-9)
}

/// Where a level cut falls more than `depth` mm behind its depth at `x` (positive), walking outward from `x`: the distance from the
/// middle, or None if the cut ends first.
pub fn side_at_depth(c: &Cut, x: f64, depth: f64) -> Option<f64> {
    let start = c.at(x);
    if start.is_nan() {
        return None;
    }
    let level = start - depth;
    let mut k = c.index(x);
    let dir: isize = if x >= 0.0 { 1 } else { -1 };
    loop {
        let next = k as isize + dir;
        if next < 0 || next >= c.z.len() as isize || c.z[next as usize].is_nan() {
            return None;
        }
        let next = next as usize;
        if c.z[next] < level {
            let (a, b) = (c.z[k], c.z[next]);
            let s = (a - level) / (a - b);
            return Some((c.u(k) + s * dir as f64 * crate::measure::STEP).abs());
        }
        k = next;
    }
}

/// The distance from each merged vertex of the base head to the nearest vertex of `fence`, in mm on the reference face.
pub fn distances(local: &[V3], fence: &[bool]) -> Vec<f64> {
    let f: Vec<V3> = local.iter().zip(fence).filter(|(_, x)| **x).map(|(p, _)| *p).collect();
    local.iter().map(|p| f.iter().map(|q| crate::length(crate::sub(*p, *q))).fold(f64::INFINITY, f64::min)).collect()
}

/// Whether a place (frame of the eyes, mm) lies within 17 mm of an eye's centre: the eyeball, the lids and their linings, which only
/// the eye sliders move.
pub fn near_eye(p: V3) -> bool {
    [-1.0, 1.0].iter().any(|&s| crate::length(crate::sub(p, [s * 0.5 * crate::measure::EYES_APART_MM, 0.0, 0.0])) < 17.0)
}

/// A KOTOR vertex is hair or cloth when the hairless wrap did not take it for skin and it stands more than this far outside our fit
/// of the head, in mm.
const COVER_MM: f64 = 6.0;

/// Which of KOTOR's triangles are hair or cloth over the face: every corner is not skin and stands out from our fit of the head. The
/// fit only locates the skin; nothing is measured on it. (The same rule as the jaw's.)
pub fn cover(extras: &KotorExtras, frame: &Frame, positions: &[V3], triangles: &[[u32; 3]]) -> Vec<bool> {
    let (Some(skin), Some(fit)) = (extras.skin, extras.fit) else { return vec![false; triangles.len()] };
    let fit_local: Vec<V3> = fit.iter().map(|&p| frame.local(p)).collect();
    let mut normals = vec![[0.0; 3]; fit.len()];
    if let Some(ft) = extras.fit_triangles {
        for t in ft {
            let n = crate::cross(
                crate::sub(fit_local[t[1] as usize], fit_local[t[0] as usize]),
                crate::sub(fit_local[t[2] as usize], fit_local[t[0] as usize]),
            );
            for &i in t {
                normals[i as usize] = crate::add(normals[i as usize], n);
            }
        }
    }
    let outside: Vec<bool> = positions
        .iter()
        .zip(skin)
        .map(|(&p, &is_skin)| {
            if is_skin {
                return false;
            }
            let q = frame.local(p);
            let (k, _) = fit_local
                .iter()
                .enumerate()
                .map(|(k, f)| (k, crate::length(crate::sub(q, *f))))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();
            crate::dot(crate::sub(q, fit_local[k]), crate::normalize(normals[k])) > COVER_MM
        })
        .collect();
    triangles.iter().map(|t| t.iter().all(|&i| outside.get(i as usize).copied().unwrap_or(false))).collect()
}

/// A KOTOR head's measure with the rows whose part of the face hair or cloth covers made unreadable: `parts` lists each part's box
/// and its rows, and a part is hidden when 4 or more covering triangles have their middle in its box. The measure reads the mesh
/// without the covering triangles.
pub fn measure_uncovered(
    f: &dyn Feature,
    surface: &Surface,
    extras: &KotorExtras,
    frame: &Frame,
    positions: &[V3],
    parts: &[(fn(V3) -> bool, &[&str])],
) -> Result<Measured, &'static str> {
    let window = f.window();
    let Some(triangles) = extras.triangles else { return f.measure(surface) };
    let covered = cover(extras, frame, positions, triangles);
    let local: Vec<[V3; 3]> = triangles
        .iter()
        .zip(&covered)
        .filter(|(_, c)| !**c)
        .map(|(t, _)| t.map(|i| frame.local(positions[i as usize])))
        .filter(|t| t.iter().any(|&p| window.contains(p)))
        .collect();
    let middles: Vec<V3> = triangles
        .iter()
        .zip(&covered)
        .filter(|(_, c)| **c)
        .map(|(t, _)| t.iter().fold([0.0; 3], |a, &i| crate::add(a, crate::scale(frame.local(positions[i as usize]), 1.0 / 3.0))))
        .collect();
    let hidden: Vec<&str> = parts
        .iter()
        .filter(|(inside, _)| middles.iter().filter(|m| inside(**m)).count() >= 4)
        .flat_map(|(_, rows)| rows.iter().copied())
        .collect();
    let mut m = f.measure(&Surface::from_local_in(local, &window))?;
    for key in hidden {
        if let Some(i) = f.proportions().iter().position(|p| p.key == key) {
            m.proportions[i] = f64::NAN;
        } else if let Some(o) = f.outlines().iter().position(|o| o.key == key) {
            m.outlines[o].clear();
        }
    }
    Ok(m)
}

/// The brow ridge above one eye at `x` from the middle (both sides averaged): its crest (height, depth) and the height of its lower
/// edge, where the cut going down from the crest falls 3 mm behind it. The edge rolls steeply into the eye socket, so its height is
/// steady where the crest's height is not. The cut reaches 8 mm below the eyes, because on a deep-set eye under a low brow (Kreia's)
/// the skin first falls 3 mm behind the crest below the eyes' centres. Where the outer brow and the socket's rim make one flat face
/// that never falls 3 mm behind the crest, the edge is the cut's lowest place, which is where a fall that only just reaches 3 mm puts
/// it too.
fn ridge_at(surface: &Surface, x: f64, dots: &mut Vec<V3>) -> Option<(f64, f64)> {
    let (mut depth, mut edge) = (0.0, 0.0);
    for side in [-1.0, 1.0] {
        let c = upright(surface, side * x, -8.0, 40.0);
        let (cy, cz) = crest(&c, 2.0, 34.0)?;
        let y = fall_below(&c, cy, cz, 3.0);
        dots.extend([[side * x, cy, cz], [side * x, y, cz - 3.0]]);
        depth += 0.5 * cz;
        edge += 0.5 * y;
    }
    Some((depth, edge))
}

/// The height where an upright cut, going down from its crest (`cy`, `cz`), first lies `fall` mm behind the crest, or the cut's
/// lowest place when it never does.
fn fall_below(c: &Cut, cy: f64, cz: f64, fall: f64) -> f64 {
    let mut k = c.index(cy);
    loop {
        if k == 0 || c.z[k - 1].is_nan() {
            return c.u(k);
        }
        if c.z[k - 1] < cz - fall {
            let (a, b) = (c.z[k], c.z[k - 1]);
            return c.u(k) - (a - (cz - fall)) / (a - b) * crate::measure::STEP;
        }
        k -= 1;
    }
}

/// The falls behind the crest between which the brow's edge is read, in mm.
const EDGE_FALLS: [f64; 6] = [3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

/// How steeply the brow ridge's lower edge falls into the eye socket, in degrees from upright, both sides averaged, from 22 to 34 mm
/// from the middle every millimetre. On each upright cut the heights where the cut lies 3, 4, ... 8 mm behind its crest get a fitted
/// line, and the edge's steepness is the angle of that line: a sharp edge falls those 5 mm within a short height. Single heights
/// jumped by up to 2 mm where a facet of KOTOR's coarse meshes lies across one of them; the fitted line steadies them. The crest is
/// looked for below 24 mm above the eyes, so a fringe of hair modelled into the face mesh above the brow is not taken for it.
fn edge_steepness(surface: &Surface, dots: &mut Vec<V3>) -> f64 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for x in (22..=34).map(f64::from) {
        for side in [-1.0, 1.0] {
            let c = upright(surface, side * x, -8.0, 40.0);
            let Some((cy, cz)) = crest(&c, 2.0, 24.0) else { return f64::NAN };
            let pts: Vec<[f64; 2]> = EDGE_FALLS.iter().map(|&f| [f, fall_below(&c, cy, cz, f)]).collect();
            let height = -slope(&pts) * (EDGE_FALLS[5] - EDGE_FALLS[0]);
            if x % 4.0 == 2.0 {
                dots.extend([pts[0], pts[5]].map(|p| [side * x, p[1], cz - p[0]]));
            }
            sum += ((EDGE_FALLS[5] - EDGE_FALLS[0]) / height.max(0.1)).atan().to_degrees();
            n += 1.0;
        }
    }
    sum / n
}

/// For the audit: on each side, every 2 mm from 14 to 50 mm from the middle, the crest of the upright cut (height, depth) and the
/// height where the cut first falls 3 mm behind it going down, or null where that fails.
pub fn ridge_report(surface: &Surface) -> serde_json::Value {
    let mut rows = Vec::new();
    for side in [-1.0, 1.0] {
        for x in (14..=50).step_by(2).map(f64::from) {
            let c = upright(surface, side * x, -8.0, 40.0);
            let Some((cy, cz)) = crest(&c, 2.0, 34.0) else {
                rows.push(json!({"x": side * x, "crest": null}));
                continue;
            };
            let mut k = c.index(cy);
            let edge = loop {
                if k == 0 || c.z[k - 1].is_nan() {
                    break None;
                }
                if c.z[k - 1] < cz - 3.0 {
                    break Some(c.u(k));
                }
                k -= 1;
            };
            let low = (c.index(-8.0)..=c.index(cy)).filter(|&k| !c.z[k].is_nan()).map(|k| c.z[k]).fold(f64::INFINITY, f64::min);
            let cut: Vec<f64> = (-8..=40).map(|y| c.at(f64::from(y))).collect();
            rows.push(json!({"x": side * x, "crest": [cy, cz], "edge": edge, "lowest_below": low, "cut": cut}));
        }
    }
    json!(rows)
}

/// Measures the brow ridge, the forehead and the temples of a surface. A part that cannot be read (for example a forehead under
/// hair) gives NaN rows; the measure fails only when the brow ridge itself cannot be found.
pub fn measure(surface: &Surface) -> Result<Measured, &'static str> {
    let mut dots: Vec<V3> = Vec::new();
    // the brow ridge above the eye, from 14 to 42 mm from the middle: its depth and its lower edge
    let mut line: Vec<[f64; 2]> = Vec::with_capacity(BROW_XS.len());
    let mut depth: Vec<f64> = Vec::with_capacity(BROW_XS.len());
    for &x in &BROW_XS {
        let (d, y) = ridge_at(surface, x, &mut dots).ok_or("no brow ridge")?;
        line.push([x, y]);
        depth.push(d);
    }
    let over: Vec<usize> = (0..BROW_XS.len()).filter(|&i| (22.0..=42.0).contains(&BROW_XS[i])).collect();
    let brow_projection = over.iter().map(|&i| depth[i]).sum::<f64>() / over.len() as f64;
    // how far the ridge bulges out from the forehead: its depth minus the forehead's depth 15 mm above its lower edge, at the same
    // places across
    let mut brow_bulge = 0.0;
    for &i in &over {
        let x = BROW_XS[i];
        let mut f = 0.0;
        for side in [-1.0, 1.0] {
            let c = upright(surface, side * x, line[i][1] + 10.0, line[i][1] + 20.0);
            let z = c.at(line[i][1] + 15.0);
            dots.push([side * x, line[i][1] + 15.0, z]);
            f += 0.5 * z;
        }
        brow_bulge += (depth[i] - f) / over.len() as f64;
    }
    let brow_height = over.iter().map(|&i| line[i][1]).sum::<f64>() / over.len() as f64;
    let brow_edge = edge_steepness(surface, &mut dots);
    let brow_slant = slope(&line).atan().to_degrees();
    // how far the ridge's outer end lies behind its inner end, 14 to 18 mm from the middle. The outer end is read 46 and 50 mm from
    // the middle; on a narrow head whose ridge ends nearer the middle (in the fixed frame since stage A of the eyes), it is read at the
    // ridge's own last 4 mm, found in steps of 1 mm down to 42 mm.
    let outer: Option<f64> = (42..=50)
        .rev()
        .map(f64::from)
        .find_map(|end| [end - 4.0, end].iter().map(|&x| ridge_at(surface, x, &mut Vec::new()).map(|r| r.0)).sum::<Option<f64>>());
    let brow_sweep = outer.map_or(f64::NAN, |o| 0.5 * (depth[0] + depth[1]) - 0.5 * o);
    // the cut down the middle: the brow between the eyes, and the forehead from 30 to 62 mm above the eyes
    let mid = upright(surface, 0.0, 0.0, 95.0);
    // how far the brow between the eyes stands out beyond the ridge over the eyes
    let brow_middle = crest(&mid, 8.0, 24.0).map_or(f64::NAN, |(gy, gz)| {
        dots.push([0.0, gy, gz]);
        gz - brow_projection
    });
    let fore: Vec<[f64; 2]> = (mid.index(30.0)..=mid.index(62.0)).filter(|&k| !mid.z[k].is_nan()).map(|k| [mid.u(k), mid.z[k]]).collect();
    // degrees the forehead leans back from upright: the depth falls as the height rises
    let forehead_slope = if fore.len() >= 100 { (-slope(&fore)).atan().to_degrees() } else { f64::NAN };
    let prof: Vec<[f64; 2]> = (mid.index(20.0)..=mid.index(70.0)).filter(|&k| !mid.z[k].is_nan()).map(|k| [mid.u(k), mid.z[k]]).collect();
    let forehead_profile = if prof.len() >= 160 { resample(&prof, OUTLINES[1].points) } else { Vec::new() };
    dots.extend(forehead_profile.iter().map(|p| [0.0, p[0], p[1]]));
    // how rounded the forehead is across, 35 to 55 mm above the eyes: a parabola z = a + b x^2 fitted to the level cut from 32 mm left
    // to 32 mm right, read as how far it falls back 30 mm to each side of the middle
    let mut forehead_curve = 0.0;
    for y in [35.0, 40.0, 45.0, 50.0, 55.0] {
        let c = level(surface, y);
        let pts: Vec<[f64; 2]> = (0..=64).map(|k| -32.0 + k as f64).map(|x| [x * x, c.at(x)]).filter(|p| !p[1].is_nan()).collect();
        if pts.len() < 50 {
            forehead_curve = f64::NAN;
            continue;
        }
        let b = slope(&pts);
        forehead_curve += -b * 900.0 / 5.0;
        let a = c.at(0.0);
        dots.extend([[-30.0, y, a + b * 900.0], [30.0, y, a + b * 900.0]]);
    }
    // the width at the temples: 15 to 25 mm above the eyes, where the face falls 25 mm behind the brow ridge's depth 42 mm from the
    // middle
    let mut temple_width = 0.0;
    for y in [15.0, 20.0, 25.0] {
        let c = level(surface, y);
        for side in [-1.0, 1.0] {
            match side_at_depth(&c, side * 42.0, 25.0) {
                Some(h) => {
                    dots.push([side * h, y, c.at(side * 42.0) - 25.0]);
                    temple_width += h / 3.0;
                }
                None => temple_width = f64::NAN,
            }
        }
    }
    Ok(Measured {
        proportions: vec![brow_projection, brow_height, brow_slant, brow_middle, forehead_slope, forehead_curve, temple_width, brow_sweep, brow_bulge, brow_edge],
        outlines: vec![line, forehead_profile],
        marks: json!({"dots": dots}),
        cuts: vec![(0..mid.z.len()).step_by(2).filter(|&k| !mid.z[k].is_nan()).map(|k| [mid.u(k), mid.z[k]]).collect()],
    })
}

/// Brow edge sharpness's step (`set::DrawnStep`): it sharpens the lower edge of the brow on the built head, wherever this head has
/// it, by pushing the edge forward by `amount` mm (a negative amount pulls it back and rounds the edge off). On each side, the edge
/// is read every 2 mm from 10 to 50 mm from the middle, as the height where the upright cut lies 3 mm behind its crest, and the push
/// is a Gaussian of 3 mm across it. Across the 57 stand-ins this sharpened the median head by 12 degrees at 4 mm with no fold. Also
/// pulling the skin under the edge back turned sharper per millimetre, but it creased the upper lid's fold where it met the fade
/// below, on a quarter of the heads at 3 mm; narrower Gaussians creased the skin too, because the head's vertex rows run across the
/// brow's edge 3 to 4.5 mm apart. A movement fixed to the base head's edge missed the edge of heads whose brow sits higher or lower,
/// and softened some of them.
///
/// Along the brow the step fades in from 10 to 16 mm from the middle and out from 38 to 46 mm. It acts fully down to 8 mm above each
/// eye's centre and fades to 0 at 6 mm, where the upper lid's skin and fold begin, which the eye sliders own; the eyes' centres are
/// read on the built head, after the whole-eye sliders. Where the edge itself comes down to 10 mm above the eye's centre the step
/// fades too, to 0 where the edge is 7 mm above it, as on the outer brow of most heads: there the push reached the upper lid's skin
/// and, with the eye sliders at their ends, folded it on Bastila, Kreia and Zherron. Only the face's front moves (5 mm or more in
/// front of the eye's centre), never a vertex the smoothing keeps.
pub fn edge_step(p: &mut [V3], ids: &[u32], triangles: &[[u32; 3]], amount: f64) {
    let frame = Frame::head();
    let local: Vec<V3> = p.iter().map(|&q| frame.local(q)).collect();
    let rings = crate::measure::OUR_RINGS.map(|r| {
        let at: Vec<usize> = (0..ids.len()).filter(|&i| r.contains(&(ids[i] as usize))).collect();
        (at.len() == r.len()).then(|| at.iter().fold([0.0; 3], |a, &i| crate::add(a, crate::scale(local[i], 1.0 / at.len() as f64))))
    });
    let half = 0.5 * crate::measure::EYES_APART_MM;
    let eyes = [rings[0].unwrap_or([half, 0.0, 0.0]), rings[1].unwrap_or([-half, 0.0, 0.0])];
    let surface = Surface::from_local_in(triangles.iter().map(|t| t.map(|i| local[i as usize])).collect(), &WINDOW);
    let xs: Vec<f64> = (5..=25).map(|k| f64::from(2 * k)).collect();
    let edges: Vec<Vec<f64>> = [1.0, -1.0]
        .iter()
        .map(|&side| {
            let found: Vec<Option<f64>> = xs
                .iter()
                .map(|&x| {
                    let c = upright(&surface, side * x, -8.0, 40.0);
                    crest(&c, 2.0, 24.0).map(|(cy, cz)| fall_below(&c, cy, cz, 3.0))
                })
                .collect();
            (0..xs.len())
                .map(|i| {
                    let near = (0..xs.len()).filter(|&k| found[k].is_some()).min_by_key(|&k| k.abs_diff(i));
                    near.and_then(|k| found[k]).unwrap_or(f64::NAN)
                })
                .collect()
        })
        .collect();
    let edge_at = |side: usize, ax: f64| -> f64 {
        let t = ((ax - xs[0]) / 2.0).clamp(0.0, (xs.len() - 1) as f64);
        let i = (t as usize).min(xs.len() - 2);
        let f = t - i as f64;
        edges[side][i] * (1.0 - f) + edges[side][i + 1] * f
    };
    let profile = |d: f64| (-0.5 * (d / 3.0).powi(2)).exp();
    let mut push = vec![0.0; p.len()];
    let mut free = vec![false; p.len()];
    for (i, dz) in push.iter_mut().enumerate() {
        let l = local[i];
        let side = usize::from(l[0] < 0.0);
        let eye = eyes[side];
        let ax = l[0].abs();
        if crate::set::kept(ids[i]) || l[2] < eye[2] + 5.0 || !(10.0..46.0).contains(&ax) {
            continue;
        }
        let e = edge_at(side, ax);
        let w = smoothstep((ax - 10.0) / 6.0)
            * smoothstep((46.0 - ax) / 8.0)
            * smoothstep((l[1] - eye[1] - 6.0) / 2.0)
            * smoothstep((e - eye[1] - 7.0) / 3.0);
        let d = l[1] - e;
        if w == 0.0 || d.is_nan() {
            continue;
        }
        *dz = amount * w * profile(d);
        free[i] = true;
    }
    // the push is sampled on vertices 2 to 7 mm apart, unevenly, which shaded the brow in blotches; two rounds of averaging with the
    // neighbours (half each) even it out, among the vertices the step reaches, so nothing below 6 mm above the eyes moves
    let mut neighbours: Vec<Vec<u32>> = vec![Vec::new(); p.len()];
    for t in triangles {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            neighbours[a as usize].push(b);
            neighbours[b as usize].push(a);
        }
    }
    for _ in 0..SMOOTH_ROUNDS {
        push = (0..p.len())
            .map(|i| {
                let nb = &neighbours[i];
                if !free[i] || nb.is_empty() {
                    return push[i];
                }
                0.5 * push[i] + 0.5 * nb.iter().map(|&j| push[j as usize]).sum::<f64>() / nb.len() as f64
            })
            .collect();
    }
    for (q, dz) in p.iter_mut().zip(&push) {
        if *dz != 0.0 {
            *q = crate::add(*q, crate::scale(frame.axes[2], dz / frame.scale));
        }
    }
}

/// The rounds of averaging of Brow edge sharpness's push.
const SMOOTH_ROUNDS: usize = 2;

/// The parts of the face that hair or cloth may cover on a KOTOR head, and the rows each carries.
fn brow_box(p: V3) -> bool {
    p[0].abs() < 55.0 && p[1] > 4.0 && p[1] < 32.0 && p[2] > -40.0
}
fn forehead_box(p: V3) -> bool {
    p[0].abs() < 45.0 && p[1] > 28.0 && p[1] < 75.0 && p[2] > -40.0
}
fn temple_box(p: V3) -> bool {
    p[0].abs() > 45.0 && p[0].abs() < 85.0 && p[1] > 8.0 && p[1] < 35.0 && p[2] > -60.0
}
const BROW_ROWS: [&str; 8] = ["brow_projection", "brow_height", "brow_slant", "brow_middle", "brow_sweep", "brow_bulge", "brow_edge", "brow_line"];
const FOREHEAD_ROWS: [&str; 3] = ["forehead_slope", "forehead_curve", "forehead_profile"];
const TEMPLE_ROWS: [&str; 1] = ["temple_width"];

fn allowed(_p: V3) -> bool {
    true
}

impl Feature for Brow {
    fn key(&self) -> &'static str {
        "brow"
    }
    fn title(&self) -> &'static str {
        "Brow and forehead"
    }
    fn proportions(&self) -> &'static [Proportion] {
        &PROPORTIONS
    }
    fn outlines(&self) -> &'static [OutlineDef] {
        &OUTLINES
    }
    fn specs(&self) -> &'static [Spec] {
        &SPECS
    }
    fn groups(&self) -> &'static [&'static str] {
        &GROUPS
    }
    fn window(&self) -> Window {
        WINDOW
    }
    fn region(&self) -> Window {
        Window { x: 70.0, y: [5.0, 70.0], z: -40.0 }
    }
    fn measure(&self, surface: &Surface) -> Result<Measured, &'static str> {
        measure(surface)
    }
    fn measure_kotor(&self, surface: &Surface, extras: &KotorExtras, frame: &Frame, positions: &[V3]) -> Result<Measured, &'static str> {
        measure_uncovered(self, surface, extras, frame, positions, &[(brow_box, &BROW_ROWS), (forehead_box, &FOREHEAD_ROWS), (temple_box, &TEMPLE_ROWS)])
    }
    /// The brow ridge, the forehead and the temples: 1 inside, fading to 0 over 6 mm toward the skin round the eyes (the zone "around
    /// the eyes", which the eye sliders own), over 6 mm toward the nose, from 80 to 95 mm above the eyes toward the top of the head,
    /// from 30 to 55 mm behind the eyes toward the sides of the head, and over 10 mm toward the ears. Every vertex the smoothing holds
    /// still (the lids, the eyeballs, the ears) has 0.
    fn zone(&self, mesh: &HeadMesh, _base: &Measured) -> Zone {
        let frame = Frame::from_eyes(mesh.eyes);
        let local: Vec<V3> = mesh.first.iter().map(|&i| frame.local(mesh.base[i as usize])).collect();
        let n = mesh.merged_count();
        let zone_of = |name: &str| 1 + crate::mesh::ZONES.iter().position(|z| *z == name).unwrap() as u8;
        let (eyes, nose) = (zone_of("around the eyes"), zone_of("nose"));
        let is = |g: usize, z: u8| mesh.zone[mesh.first[g] as usize] == z;
        let held: Vec<bool> = (0..n).map(|g| mesh.keep[mesh.first[g] as usize] && !mesh.hidden[g]).collect();
        let socket: Vec<bool> = (0..n).map(|g| is(g, eyes) || (held[g] && local[g][0].abs() < 60.0 && local[g][1].abs() < 20.0)).collect();
        let noses: Vec<bool> = (0..n).map(|g| is(g, nose)).collect();
        let ears: Vec<bool> = (0..n).map(|g| held[g] && local[g][0].abs() >= 60.0).collect();
        let (ds, dn, de) = (distances(&local, &socket), distances(&local, &noses), distances(&local, &ears));
        let weight = (0..n)
            .map(|g| {
                let p = local[g];
                if held[g] || socket[g] || p[1] < 0.0 || near_eye(p) {
                    return 0.0;
                }
                smoothstep(ds[g] / 10.0)
                    * smoothstep(dn[g] / 6.0)
                    * smoothstep(de[g] / 10.0)
                    * smoothstep((95.0 - p[1]) / 15.0)
                    * smoothstep((p[2] + 55.0) / 25.0)
            })
            .collect();
        Zone { weight }
    }
    /// The parts on the base head: the brow ridge (below 30 mm above the eyes, more than 9 mm from the middle and less than 52 mm),
    /// the middle of the brow (within 9 mm of the middle, below 30 mm), the temples (52 mm or more from the middle, or below 30 mm and
    /// more than 30 mm behind the eyes), and the forehead (the rest).
    fn part_of(&self, p: V3, _base: &Measured) -> &'static str {
        let ax = p[0].abs();
        if ax >= 52.0 || (p[1] < 30.0 && p[2] < -30.0) {
            "temples"
        } else if p[1] < 30.0 && ax < 9.0 {
            "middle"
        } else if p[1] < 30.0 {
            "ridge"
        } else {
            "forehead"
        }
    }
    fn bumps(&self) -> BumpRule {
        BumpRule { size_mm: 9.0, spacing_mm: 7.0, min_weight: 0.6, allowed }
    }
    fn drawn_step(&self, measure: &str) -> Option<crate::set::DrawnStep> {
        (measure == "brow_edge").then_some(edge_step as crate::set::DrawnStep)
    }
    fn miss_words(&self, i: usize, g: f64) -> String {
        let a = g.abs();
        let (more, less) = match PROPORTIONS[i].key {
            "brow_projection" => ("The brow ridge stands out too far", "The brow ridge does not stand out enough"),
            "brow_height" => ("The brow ridge sits too high", "The brow ridge sits too low"),
            "brow_slant" => ("The brow's outer end is too high", "The brow's outer end is too low"),
            "brow_middle" => ("The brow stands out too far between the eyes", "The brow is too flat between the eyes"),
            "forehead_slope" => ("The forehead slopes back too much", "The forehead is too upright"),
            "forehead_curve" => ("The forehead is too rounded", "The forehead is too flat"),
            "temple_width" => ("The temples are too wide", "The temples are too narrow"),
            "brow_bulge" => ("The brow ridge bulges out too far from the forehead", "The brow ridge does not bulge out enough from the forehead"),
            "brow_sweep" => ("The brow's outer end is swept back too far", "The brow is too flat across"),
            "brow_edge" => ("The brow's lower edge is too sharp", "The brow's lower edge is too soft"),
            _ => ("Too much", "Too little"),
        };
        let unit = if PROPORTIONS[i].unit == "degrees" { "degrees" } else { "mm" };
        format!("{} by {a:.1} {unit}.", if g > 0.0 { more } else { less })
    }
    fn carries_hidden(&self) -> bool {
        false
    }
    fn span_quantile(&self) -> f64 {
        0.95
    }
    /// Brow bulge and Brow sweep reach every head: their far ends were stuck on the heaviest and most swept brows (Jolee, Malek,
    /// Zherron).
    fn span_quantile_of(&self, measure: &str) -> f64 {
        if matches!(measure, "brow_bulge" | "brow_sweep") { 1.0 } else { self.span_quantile() }
    }
    /// KOTOR's brows barely differ in bulge (a spread of 1.3 mm), so Jolee's 5.4 mm lies beyond four spreads.
    fn span_cap_of(&self, measure: &str) -> f64 {
        if measure == "brow_bulge" { 5.0 } else { 4.0 }
    }
    fn solve_by_spread(&self) -> bool {
        true
    }
    fn guards(&self) -> &'static [Guard] {
        &[Guard::Ears, Guard::Lids]
    }
    fn check_bump(&self) -> V3 {
        [0.0, -12.0, 25.0]
    }
    fn shots(&self) -> &'static [Shot] {
        &SHOTS
    }
}

pub const SPECS: [Spec; 8] = [
    Spec {
        measure: "brow_projection",
        name: "Brow depth",
        hint: "How far the brow and the forehead just above it stand out over the eyes, seen from the side. A deep brow makes the eyes look deep set.",
        group: "brow",
        parts: &["ridge", "middle", "forehead"],
        repair_parts: &["temples"],
        free: &["forehead_profile"],
        loose: &["forehead_slope", "brow_sweep", "brow_height", "brow_line"],
    },
    Spec {
        measure: "brow_bulge",
        name: "Brow bulge",
        hint: "How far the brow ridge bulges out from the forehead above it, seen from the side. The brow between the eyes follows.",
        group: "brow",
        parts: &["ridge", "middle"],
        repair_parts: &["forehead"],
        free: &["brow_projection", "brow_middle"],
        loose: &["brow_height", "brow_line", "brow_sweep"],
    },
    Spec {
        measure: "brow_sweep",
        name: "Brow sweep",
        hint: "How far the outer ends of the brow ridge lie behind its inner ends, seen from above. A swept brow gives a narrow, angular face.",
        group: "brow",
        parts: &["ridge", "temples", "forehead"],
        repair_parts: &[],
        free: &["forehead_profile"],
        loose: &["brow_projection", "temple_width"],
    },
    Spec {
        measure: "brow_middle",
        name: "Brow between the eyes",
        hint: "How far the brow between the eyes stands out beyond the brows over the eyes, seen from the side. A jutting middle joins the two brows into one heavy ridge.",
        group: "brow",
        parts: &["middle", "ridge"],
        repair_parts: &["forehead"],
        free: &["forehead_profile"],
        loose: &["brow_bulge", "brow_sweep", "brow_height", "brow_line"],
    },
    Spec {
        measure: "brow_edge",
        name: "Brow edge sharpness",
        hint: "How sharply the lower edge of the brow turns down into the eye socket, seen from the side. A sharp edge throws a crisp shadow over the eyes. A soft edge rounds smoothly into the socket.",
        group: "brow",
        parts: &[],
        repair_parts: &[],
        free: &[],
        loose: &[],
    },
    Spec {
        measure: "forehead_slope",
        name: "Forehead slope",
        hint: "Whether the forehead rises upright or slopes back, seen from the side.",
        group: "forehead",
        parts: &["forehead", "temples"],
        repair_parts: &["ridge"],
        free: &["forehead_profile"],
        loose: &[],
    },
    Spec {
        measure: "forehead_curve",
        name: "Forehead roundness",
        hint: "Whether the forehead is flat or rounded across, seen from above.",
        group: "forehead",
        parts: &["forehead", "temples"],
        repair_parts: &[],
        free: &["forehead_profile"],
        loose: &["temple_width"],
    },
    Spec {
        measure: "temple_width",
        name: "Temple width",
        hint: "How wide the face is at the temples, beside the outer ends of the brows, seen from the front.",
        group: "temples",
        parts: &["temples", "forehead"],
        repair_parts: &[],
        free: &[],
        loose: &["brow_sweep", "forehead_curve"],
    },
];

/// Tools of the upper face features that the kit's commands do not cover (native builds only).
#[cfg(not(target_arch = "wasm32"))]
pub mod tools {
    use crate::feature::{Feature, by_key};
    use crate::measure::{Frame, Surface};
    use crate::mesh::input::Input;
    use crate::set::{SMOOTHING, SliderSet, smoother};
    use serde_json::{Value, json};
    use std::path::Path;

    /// The KOTOR heads' spread of each proportion of `o`, from its marks file (the nose's may be the data folder's own `marks.json`).
    fn spreads(data: &Path, o: &dyn Feature) -> Option<Vec<f64>> {
        let mut paths = vec![data.join(o.files().marks)];
        if o.key() == "nose" {
            paths.push(data.join("marks.json"));
        }
        paths.iter().find_map(|p| {
            let v: Value = serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()?;
            serde_json::from_value(v["spread_proportions"].clone()).ok()
        })
    }

    /// How each slider of `f`, at both ends on the base head, moves the measures of the other features: every change larger than a
    /// twentieth of KOTOR's spread of that measure, in the measure's unit and in spreads. Written to `<feature>/leaks.json` and
    /// printed.
    pub fn leaks(input: &Input, data: &Path, f: &dyn Feature) {
        let mesh = input.head_mesh();
        let sm = smoother(&mesh.base, &mesh.triangles, &mesh.keep);
        crate::set::init_mesh(&mesh.triangles, &mesh.keep);
        let frame = Frame::from_eyes(mesh.eyes);
        let set: SliderSet = serde_json::from_str(&std::fs::read_to_string(data.join(f.files().sliders)).unwrap()).unwrap();
        let base = sm.smooth(&mesh.base, SMOOTHING);
        let others: Vec<&dyn Feature> =
            ["nose", "jaw", "eyes", "brow", "cheeks"].iter().filter(|k| **k != f.key()).filter_map(|k| by_key(k)).collect();
        let measured = |o: &dyn Feature, p: &[crate::V3]| o.measure(&Surface::within(p, &mesh.triangles, &frame, &o.window())).ok();
        let before: Vec<_> = others.iter().map(|o| measured(*o, &base)).collect();
        let spread: Vec<_> = others.iter().map(|o| spreads(data, *o)).collect();
        let mut out = Vec::new();
        for (j, s) in set.sliders.iter().enumerate() {
            for (end, v) in [("low", s.low), ("high", s.high)] {
                let mut values = vec![0.0; set.sliders.len()];
                values[j] = v;
                let built = set.build(&sm, &mesh.base, &values);
                let mut moved = Vec::new();
                for (k, o) in others.iter().enumerate() {
                    let (Some(b), Some(a)) = (&before[k], measured(*o, &built)) else { continue };
                    for (i, p) in o.proportions().iter().enumerate() {
                        let d = a.proportions[i] - b.proportions[i];
                        let sp = spread[k].as_ref().and_then(|s| s.get(i).copied()).unwrap_or(f64::NAN);
                        if d.abs() > 0.05 * sp || (sp.is_nan() && d.abs() > 0.2) {
                            moved.push(json!({"feature": o.key(), "measure": p.key, "change": d, "spreads": d / sp}));
                        }
                    }
                }
                let worst = moved.iter().map(|m| m["spreads"].as_f64().unwrap_or(0.0).abs()).fold(0.0, f64::max);
                println!(
                    "{:24} {:4} {:7.2}: {}",
                    s.name,
                    end,
                    v,
                    if moved.is_empty() {
                        "nothing moves by a twentieth of a spread".to_string()
                    } else {
                        moved
                            .iter()
                            .map(|m| format!("{} {} {:+.2} ({:+.2} spreads)", m["feature"].as_str().unwrap(), m["measure"].as_str().unwrap(), m["change"].as_f64().unwrap(), m["spreads"].as_f64().unwrap_or(f64::NAN)))
                            .collect::<Vec<_>>()
                            .join("; ")
                    }
                );
                out.push(json!({"slider": s.name, "end": end, "value": v, "moved": moved, "worst_spreads": worst}));
            }
        }
        std::fs::write(data.join(format!("{}/leaks.json", f.key())), serde_json::to_string_pretty(&out).unwrap()).unwrap();
    }
}
