//! The caruncle: the soft pink wedge in the inner corner of the eye, a piece of its own next to the head mesh.
//!
//! It is made from the built head alone, so it follows every slider and the blink without help. At the inner corner the lids leave the
//! eyeball, and a hollow opens between the eyeball's side and the corner. The caruncle's piece fills that whole hollow. Its two long
//! edges are the upper and the lower lids' edges (the edge of the opening, `lid_edges.json`), from the corner tip along the eye, so it
//! meets both lids flush. Its far end lies on the eyeball, just outside it, so it meets the eyeball flush. The piece reaches up to where
//! both lids' edges come within `TOUCH_MM` of the eyeball, which is where the hollow ends.
//!
//! Only the part next to the corner tip is the pink caruncle. It is `PINK_MM` long along the lids' edges, and the width setting adds to
//! that. That part bulges out a little. Past it the piece comes down onto the eyeball and takes the eyeball's own colour (`colours`),
//! so the hollow stays closed while the pink looks as narrow as a real caruncle.

use crate::piece::{EyeEdge, V3, eyeball_radius};

/// The lids' edges count as touching the eyeball within this distance of it, in mm.
pub const TOUCH_MM: f64 = 0.6;
/// The pink part's length along the lids' edges at width 0, in mm.
pub const PINK_MM: f64 = 4.5;
/// The least pink length, in mm.
const PINK_MIN: f64 = 2.0;
/// The length over which the pink fades into the eyeball's colour, in mm.
const FADE_MM: f64 = 1.2;
/// The least and the most reach of the whole piece along the lids' edges, in mm.
const REACH_MIN: f64 = 1.0;
const REACH_MAX: f64 = 12.0;
/// How far outside the eyeball its far end lies, in mm.
const GAP_MM: f64 = 0.1;
/// How far the middle of the pink part bulges out of the straight surface between the lids, in mm.
const BULGE_MM: f64 = 0.35;
/// Its mesh: columns along the eye and rows across, from the upper lid to the lower one.
const COLUMNS: usize = 24;
const ROWS: usize = 9;
/// The columns crowd toward the corner tip, where the pink part is, by this power.
const CROWD: f64 = 1.6;

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn len(a: V3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn lerp(a: V3, b: V3, t: f64) -> V3 {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}
fn mean(p: &[V3], ids: &[u32]) -> V3 {
    let mut s = [0.0; 3];
    for &i in ids {
        for k in 0..3 {
            s[k] += p[i as usize][k] / ids.len() as f64;
        }
    }
    s
}
fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The caruncle's triangles, the same for every head and both eyes.
pub fn triangles() -> Vec<[u32; 3]> {
    let at = |i: usize, j: usize| (i * ROWS + j) as u32;
    let mut t = Vec::new();
    for i in 0..COLUMNS - 1 {
        for j in 0..ROWS - 1 {
            t.push([at(i, j), at(i + 1, j + 1), at(i + 1, j)]);
            t.push([at(i, j), at(i, j + 1), at(i + 1, j + 1)]);
        }
    }
    t
}

/// One lid's edge from the inner corner along the eye: the points (metres) and their distance from the corner along the edge (mm).
fn chain(head: &[V3], e: &EyeEdge, upper: bool) -> (Vec<V3>, Vec<f64>) {
    let m = e.edge.len();
    let ic = e.edge.iter().position(|&v| v == e.inner_corner).unwrap_or(0);
    let walk = |step: usize| -> Vec<usize> {
        let mut out = vec![ic];
        let mut i = ic;
        for _ in 0..m / 2 {
            i = (i + step) % m;
            out.push(i);
            if e.edge[i] == e.outer_corner {
                break;
            }
        }
        out
    };
    let a = walk(1);
    let b = walk(m - 1);
    let ups = |w: &[usize]| w[1..].iter().filter(|&&k| e.upper[k]).count() as f64 / (w.len() - 1).max(1) as f64;
    let pick = if (ups(&a) >= ups(&b)) == upper { a } else { b };
    let pts: Vec<V3> = pick.iter().map(|&k| head[e.edge[k] as usize]).collect();
    let mut d = vec![0.0];
    for w in pts.windows(2) {
        d.push(d[d.len() - 1] + len(sub(w[1], w[0])) * 1000.0);
    }
    (pts, d)
}

fn at_distance(pts: &[V3], d: &[f64], x: f64) -> V3 {
    for k in 1..pts.len() {
        if x <= d[k] || k == pts.len() - 1 {
            let t = if d[k] > d[k - 1] { ((x - d[k - 1]) / (d[k] - d[k - 1])).clamp(0.0, 1.0) } else { 0.0 };
            return lerp(pts[k - 1], pts[k], t);
        }
    }
    pts[0]
}

/// The reach along the lids' edges (mm) where both edges first come within `TOUCH_MM` of the eyeball.
pub fn auto_reach(head: &[V3], e: &EyeEdge) -> f64 {
    let c = mean(head, &e.centre);
    let r = eyeball_radius(head, &e.centre);
    let (up, du) = chain(head, e, true);
    let (lo, dl) = chain(head, e, false);
    let off = |p: V3| len(sub(p, c)) * 1000.0 - r;
    let mut x = REACH_MIN;
    while x < 9.0 {
        if off(at_distance(&up, &du, x)) < TOUCH_MM && off(at_distance(&lo, &dl, x)) < TOUCH_MM {
            return x;
        }
        x += 0.25;
    }
    9.0
}

/// The pink part's length and the whole piece's reach along the lids' edges, in mm. The piece always fills the hollow, and it always
/// reaches past the pink part's fade.
fn lengths(head: &[V3], e: &EyeEdge, width: f64) -> (f64, f64) {
    let pink = (PINK_MM + width).max(PINK_MIN);
    let reach = auto_reach(head, e).max(pink + FADE_MM + 0.5).clamp(REACH_MIN, REACH_MAX);
    (pink, reach)
}

/// How far along the lids' edges column `i` lies, in mm.
fn column_mm(i: usize, reach: f64) -> f64 {
    reach * (i as f64 / (COLUMNS - 1) as f64).powf(CROWD)
}

/// The caruncle of one eye on a head (positions in metres), in metres. `width` adds to the pink part's length along the lids' edges,
/// in mm.
pub fn positions(head: &[V3], e: &EyeEdge, width: f64) -> Vec<V3> {
    let c = mean(head, &e.centre);
    let r = eyeball_radius(head, &e.centre);
    let (up, du) = chain(head, e, true);
    let (lo, dl) = chain(head, e, false);
    let (pink, reach) = lengths(head, e, width);
    let mut out = Vec::with_capacity(COLUMNS * ROWS);
    for i in 0..COLUMNS {
        let x = column_mm(i, reach);
        let a = at_distance(&up, &du, x);
        let b = at_distance(&lo, &dl, x);
        // past the middle of the pink part, the middle of each cross-section comes down onto the eyeball, and lies on it by the end
        // of the fade
        let onto = smoothstep((x - 0.3 * pink) / (0.7 * pink + FADE_MM));
        let hump = (std::f64::consts::PI * (x / (pink + 0.5 * FADE_MM)).min(1.0)).sin();
        for j in 0..ROWS {
            let t = j as f64 / (ROWS - 1) as f64;
            let across = (std::f64::consts::PI * t).sin();
            let p = lerp(a, b, t);
            let d = sub(p, c);
            let dist = len(d).max(1e-9);
            let straight = dist * 1000.0;
            let ball = r + GAP_MM;
            let mut rad = straight + (ball - straight) * onto * across;
            rad += BULGE_MM * across * hump;
            // never inside the eyeball
            let rad = rad.max(ball);
            out.push([c[0] + d[0] / dist * rad / 1000.0, c[1] + d[1] / dist * rad / 1000.0, c[2] + d[2] / dist * rad / 1000.0]);
        }
    }
    out
}

/// The colour of each vertex of one eye's caruncle, three numbers per vertex from 0 to 1 (red, green, blue). It is a tint that
/// multiplies the eyeball's own colour under the vertex: pink on the caruncle, and 1 past it, where the piece shows the eyeball. The
/// tint is a little darker next to the lids and at the corner tip, where the skin shades the caruncle.
pub fn colours(head: &[V3], e: &EyeEdge, width: f64) -> Vec<f32> {
    let (pink, reach) = lengths(head, e, width);
    let edge = [0.95, 0.56, 0.55];
    let centre = [0.9, 0.48, 0.48];
    let mut out = Vec::with_capacity(COLUMNS * ROWS * 3);
    for i in 0..COLUMNS {
        let x = column_mm(i, reach);
        let white = smoothstep((x - pink) / FADE_MM);
        let tip = 1.0 - 0.22 * (1.0 - smoothstep(x / 0.9));
        for j in 0..ROWS {
            let t = j as f64 / (ROWS - 1) as f64;
            let across = (std::f64::consts::PI * t).sin();
            let lid = 1.0 - 0.18 * (1.0 - smoothstep(across / 0.55));
            let shade = 1.0 - (1.0 - tip * lid) * (1.0 - 0.6 * white);
            let mid = smoothstep(across);
            for k in 0..3 {
                let pinkish = edge[k] + (centre[k] - edge[k]) * mid;
                out.push(((pinkish + (1.0 - pinkish) * white) * shade) as f32);
            }
        }
    }
    out
}

/// The normal of each vertex of one eye's caruncle, three numbers per vertex. On the pink part it is the surface's own normal. Where
/// the surface comes down onto the eyeball and past it, it turns into the eyeball's normal, so that the far part is lit as the eyeball
/// is and no bright crease shows where the surface drops.
pub fn normals(head: &[V3], e: &EyeEdge, width: f64) -> Vec<f32> {
    let c = mean(head, &e.centre);
    let p = positions(head, e, width);
    let (pink, reach) = lengths(head, e, width);
    let mut n = vec![[0.0; 3]; p.len()];
    for t in triangles() {
        let [a, b, d] = t.map(|i| p[i as usize]);
        let (u, v) = (sub(b, a), sub(d, a));
        let f = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        for i in t {
            for k in 0..3 {
                n[i as usize][k] += f[k];
            }
        }
    }
    let unit = |a: V3| {
        let l = len(a).max(1e-12);
        [a[0] / l, a[1] / l, a[2] / l]
    };
    // The triangles face inward on one eye, because the eyes are mirrored. The normals keep the triangles' side, since a renderer that
    // draws both sides turns the normal round on the back side.
    let outward: f64 = (0..p.len()).map(|k| (0..3).map(|q| n[k][q] * (p[k][q] - c[q])).sum::<f64>()).sum();
    let side = if outward < 0.0 { -1.0 } else { 1.0 };
    let mut out = Vec::with_capacity(p.len() * 3);
    for i in 0..COLUMNS {
        let x = column_mm(i, reach);
        let ball = smoothstep((x - 0.35 * pink) / (0.35 * pink));
        for j in 0..ROWS {
            let k = i * ROWS + j;
            let radial = unit(sub(p[k], c));
            let own = unit(n[k]).map(|v| v * side);
            let m = unit(lerp(own, radial, ball)).map(|v| v * side);
            out.extend(m.map(|x| x as f32));
        }
    }
    out
}
