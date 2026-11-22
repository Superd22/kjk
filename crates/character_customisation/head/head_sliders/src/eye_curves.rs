//! The eye's lids and corners as curves on the eyeball (stage B of the eye rebuild), and the last step of every built head: the lid's
//! edge pressed onto the eyeball.
//!
//! Each eye is read in its own frame: origin at the eyeball's centre (the mean of its 8 centre vertices), x toward the temple, y up and
//! z forward, in mm. A place is then read on the sphere about that centre: its azimuth (the angle round the vertical axis, toward the
//! temple positive), its elevation (the angle up from the horizontal plane) and its radius. The edge of the opening (`lid_edges.json`)
//! gives two curves of elevation against azimuth, one for each lid, between the two corners.
//!
//! Every lid and corner slider is a turn about a line through the eyeball's centre, by an angle that changes smoothly from place to
//! place. A turn keeps each place's distance from the centre, so the lid's edge slides over the eyeball and never enters it or leaves
//! it. The upper and lower lids turn about the horizontal line (they go up or down); the corners and the width turn about the vertical
//! line (they go along the eye, round the eyeball and back into the head).

use crate::V3;
use std::sync::OnceLock;

/// `lid_edges.json`: the edge of each eye's opening on our head mesh, the same file the lid piece reads.
const LID_EDGES: &str = include_str!("lid_edges.json");
/// The eyeball's radius when its slider is at 0, and the centre ring's distance from its mean at that radius, in mm.
const BALL_MM: f64 = 12.3;
const RING_MM: f64 = 6.907;

/// One eye's description from `lid_edges.json`.
#[derive(Clone, Debug)]
pub struct EyeEdges {
    pub edge: Vec<u32>,
    pub upper: Vec<bool>,
    pub share: Vec<f64>,
    pub inner: u32,
    pub outer: u32,
    pub ring: Vec<u32>,
    /// +1 when the temple lies toward +x.
    pub sign: f64,
}

/// Both eyes, the +x eye first.
pub fn edges() -> &'static [EyeEdges; 2] {
    static E: OnceLock<[EyeEdges; 2]> = OnceLock::new();
    E.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(LID_EDGES).expect("lid_edges.json");
        let u = |x: &serde_json::Value| x.as_u64().unwrap() as u32;
        ["L", "R"].map(|k| {
            let e = &v["eyes"][k];
            EyeEdges {
                edge: e["edge"].as_array().unwrap().iter().map(u).collect(),
                upper: e["lid"].as_array().unwrap().iter().map(|x| x.as_str() == Some("upper")).collect(),
                share: e["share_from_inner_corner"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect(),
                inner: u(&e["inner_corner"]),
                outer: u(&e["outer_corner"]),
                ring: e["eyeball_centre_vertices"].as_array().unwrap().iter().map(u).collect(),
                sign: if e["temple_is_plus_x"].as_bool().unwrap() { 1.0 } else { -1.0 },
            }
        })
    })
}

fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A smooth hat: 1 at `centre`, 0 at `half` or further from it.
fn hat(x: f64, centre: f64, half: f64) -> f64 {
    smoothstep(1.0 - (x - centre).abs() / half)
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

/// An eye's centre (metres) and radius (mm) on a head, from its centre ring.
pub fn ball(p: &[V3], ring: &[u32]) -> (V3, f64) {
    let c = mean(p, ring);
    let ms = ring.iter().map(|&v| crate::length(crate::sub(p[v as usize], c)).powi(2)).sum::<f64>() / ring.len() as f64;
    (c, BALL_MM * ms.sqrt() * 1000.0 / RING_MM)
}

/// One eye read on the base head: its frame and its two lid curves.
#[derive(Clone, Debug)]
pub struct EyeGeo {
    pub c: V3,
    pub sign: f64,
    pub r: f64,
    /// Azimuths of the inner and outer corners, radians.
    pub phi_in: f64,
    pub phi_out: f64,
    /// (azimuth, elevation) of each lid's edge, from the inner corner to the outer one, corners included.
    pub upper: Vec<[f64; 2]>,
    pub lower: Vec<[f64; 2]>,
}

/// A place in an eye's frame: (x toward the temple, y up, z forward) in mm.
fn local(g: &EyeGeo, p: V3) -> V3 {
    let d = crate::sub(p, g.c);
    [g.sign * d[0] * 1000.0, d[1] * 1000.0, d[2] * 1000.0]
}

/// Azimuth, elevation and radius (mm) of a place in an eye's frame.
fn sph(q: V3) -> (f64, f64, f64) {
    let h = q[0].hypot(q[2]);
    (q[0].atan2(q[2]), q[1].atan2(h), h.hypot(q[1]))
}

fn interp(curve: &[[f64; 2]], phi: f64) -> f64 {
    if phi <= curve[0][0] {
        return curve[0][1];
    }
    for w in curve.windows(2) {
        if phi <= w[1][0] {
            let t = if w[1][0] > w[0][0] { (phi - w[0][0]) / (w[1][0] - w[0][0]) } else { 0.0 };
            return w[0][1] + t * (w[1][1] - w[0][1]);
        }
    }
    curve[curve.len() - 1][1]
}

impl EyeGeo {
    pub fn new(base: &[V3], e: &EyeEdges) -> Self {
        let (c, r) = ball(base, &e.ring);
        let mut g = EyeGeo { c, sign: e.sign, r, phi_in: 0.0, phi_out: 0.0, upper: Vec::new(), lower: Vec::new() };
        let at = |v: u32| {
            let (phi, th, _) = sph(local(&g, base[v as usize]));
            [phi, th]
        };
        let (pi, po) = (at(e.inner), at(e.outer));
        let mut up = vec![pi, po];
        let mut lo = vec![pi, po];
        for (k, &v) in e.edge.iter().enumerate() {
            if v == e.inner || v == e.outer {
                continue;
            }
            if e.upper[k] { up.push(at(v)) } else { lo.push(at(v)) }
        }
        up.sort_by(|a, b| a[0].total_cmp(&b[0]));
        lo.sort_by(|a, b| a[0].total_cmp(&b[0]));
        g.phi_in = pi[0];
        g.phi_out = po[0];
        g.upper = up;
        g.lower = lo;
        g
    }
    /// The eye's length along the eyeball, in mm.
    pub fn length(&self) -> f64 {
        self.r * (self.phi_out - self.phi_in)
    }
    fn s(&self, phi: f64) -> f64 {
        (phi - self.phi_in) / (self.phi_out - self.phi_in)
    }
}

/// Both eyes on the base head.
pub fn geos(base: &[V3]) -> [EyeGeo; 2] {
    let e = edges();
    [EyeGeo::new(base, &e[0]), EyeGeo::new(base, &e[1])]
}

/// The settings of the fades, in mm, chosen from pictures.
#[derive(Clone, Copy, Debug)]
pub struct Fades {
    /// The skin above the upper lid's edge follows over this distance, along the eyeball.
    pub upper: f64,
    /// The skin below the lower lid's edge follows over this distance.
    pub lower: f64,
    /// Around a corner: the reach along the eye into the lids, and beyond the corner into the skin.
    pub corner_in: f64,
    pub corner_out: f64,
}

impl Default for Fades {
    fn default() -> Self {
        Self { upper: 7.0, lower: 6.0, corner_in: 8.0, corner_out: 6.0 }
    }
}

/// What a curve slider does.
#[derive(Clone, Copy, Debug)]
pub enum Curve {
    /// A lid's control point (upper or lower; 0 inner, 1 middle, 2 outer) raised by mm.
    Lid { upper: bool, point: usize },
    /// Both lids apart by mm in the middle, half each.
    Opening,
    /// The corners along the eyeball, away from the eye's middle: a stretch of the azimuth by this share.
    Width,
    /// A corner (inner or outer) along the eye, away from the eye's middle, by mm.
    CornerAlong { inner: bool },
    /// A corner up by mm.
    CornerHeight { inner: bool },
    /// The angle the two lids make at a corner, wider by degrees.
    CornerAngle { inner: bool },
    /// The lids just next to a corner apart by mm: a rounder end.
    CornerRound { inner: bool },
    /// A corner forward by mm, along the eye's forward axis: the tip most, the lids and the skin around it less.
    CornerDepth { inner: bool },
    /// The lower lid's edge toward the straight line between the corners, by this share of its own dip below that line (1 is straight).
    LowerFlat,
    /// The eyeball forward by mm, along the eye's forward axis, with the socket that wraps it, the lids and the skin up to 4 mm outside
    /// the opening; the skin fades out from there and away from the eyeball (`prominence_fades`).
    Prominence,
}

/// How much a place this many mm outside the eyeball belongs to the lining behind the lids: 1 up to 1 mm out, 0 from 3.5 mm out.
fn deep_weight(out: f64) -> f64 {
    1.0 - smoothstep((out - 1.0) / 2.5)
}

/// Turns q (eye frame, mm) about the horizontal axis (x) by `a` radians, up for positive.
fn turn_x(q: V3, a: f64) -> V3 {
    let (s, c) = a.sin_cos();
    [q[0], q[1] * c + q[2] * s, -q[1] * s + q[2] * c]
}

/// The fades of a corner's depth, in mm: along the lids, beyond the tip, and up and down from 3 mm.
fn corner_depth_fades() -> (f64, f64, f64) {
    (9.0, 7.0, 8.0)
}

/// The fades of Outer corner depth, in mm, as `corner_depth_fades`.
fn outer_depth_fades() -> (f64, f64, f64) {
    (9.0, 7.0, 8.0)
}

/// The fades of Eye prominence, in mm: the skin moves wholly up to the first value outside the opening and fades out over the second;
/// it moves wholly up to the third value in front of the eyeball and fades out over the fourth.
fn prominence_fades() -> [f64; 4] {
    [1.0, 12.0, 2.0, 10.0]
}

/// The share of Eye prominence the lids and the skin keep at the outer corner, and where along the eye (a share from the inner
/// corner) they start to keep back.
fn prominence_outer() -> [f64; 2] {
    [1.0, 0.5]
}

/// Where Eye width stops turning the skin under the eye and only slides it: from the first value in mm outside the eyeball, wholly
/// from the second.
fn width_slide() -> [f64; 2] {
    [2.0, 5.0]
}

/// Turns q about the vertical axis (y) by `b` radians, toward the temple for positive.
fn turn_y(q: V3, b: f64) -> V3 {
    let (s, c) = b.sin_cos();
    [q[0] * c + q[2] * s, q[1], -q[0] * s + q[2] * c]
}

/// The movement of every vertex (metres) for a curve slider at `amount`, on the base head. The eyeball's centre vertices never move.
pub fn field(base: &[V3], curve: Curve, amount: f64, fades: &Fades) -> Vec<V3> {
    let gs = geos(base);
    let e = edges();
    let mut out = vec![[0.0; 3]; base.len()];
    let ring: Vec<u32> = e.iter().flat_map(|x| x.ring.iter().copied()).collect();
    for (v, &p) in base.iter().enumerate() {
        if ring.contains(&(v as u32)) {
            if let Curve::Prominence = curve {
                out[v] = crate::scale(depth_axis(), amount / 1000.0);
            }
            continue;
        }
        // the eye on the side of the face this place lies on
        let k = if (p[0] - 0.5 * (gs[0].c[0] + gs[1].c[0])) * gs[0].sign > 0.0 { 0 } else { 1 };
        let g = &gs[k];
        let q = local(g, p);
        let (phi, th, rad) = sph(q);
        if let Curve::Prominence = curve {
            // the socket and the lids' lining, which wrap the eyeball, move with it wholly, wherever they are round it
            let off = rad - g.r;
            let mut w = 1.0 - smoothstep((off - 1.0) / 1.5);
            if q[2] > -6.0 && phi.abs() <= 2.2 {
                let [open_full, open_fade, off_full, off_fade] = prominence_fades();
                let s = g.s(phi);
                let along = if s < 0.0 { -s * g.length() } else if s > 1.0 { (s - 1.0) * g.length() } else { 0.0 };
                let (th_up, th_lo) = (interp(&g.upper, phi), interp(&g.lower, phi));
                let up_down = if th >= 0.5 * (th_up + th_lo) { g.r * (th - th_up) } else { g.r * (th_lo - th) };
                let outside = along.hypot(up_down.max(0.0));
                let skin = (1.0 - smoothstep((outside - open_full) / open_fade)) * (1.0 - smoothstep((off - off_full) / off_fade));
                // toward the outer corner the lids and the skin keep back, so the eyeball shows past them in profile
                let [keep, keep_from] = prominence_outer();
                let temple = (s - keep_from) * g.length();
                let held = 1.0 - (1.0 - keep) * smoothstep(temple / ((1.0 - keep_from) * g.length()));
                w = (w * held.max(1.0)).max(skin * held);
            }
            if w > 0.0 {
                out[v] = crate::scale(depth_axis(), w * amount / 1000.0);
            }
            continue;
        }
        // only the front of the eye's half of the face, within 24 mm of the eyeball's surface
        if rad - g.r > 24.0 || q[2] < -6.0 || phi.abs() > 2.2 {
            continue;
        }
        let w_r = 1.0 - smoothstep((rad - g.r - 12.0) / 8.0);
        if w_r <= 0.0 {
            continue;
        }
        let s = g.s(phi);
        let len = g.length();
        let th_up = interp(&g.upper, phi);
        let th_lo = interp(&g.lower, phi);
        let th_mid = 0.5 * (th_up + th_lo);
        let above = th >= th_mid;
        // mm along the eyeball above the upper edge, and below the lower edge
        let a_up = g.r * (th - th_up);
        let a_lo = g.r * (th_lo - th);
        // the upper lid's sliders move what lies above the opening's middle line and the lower lid's what lies below; near the eyeball
        // the change is sharp (0.6 mm), where the opening parts the lids, but more than 3 mm out from the eyeball, in front of the inner
        // corner on the side of the nose, there is no opening and it widens by 1 mm per mm, so the two lids' movements do not shear the
        // skin into a fold
        let side_w = 0.6 + (rad - g.r - 3.0).max(0.0);
        let side_up = smoothstep((th - th_mid) * g.r / side_w + 0.5);
        let side_lo = 1.0 - side_up;
        let follow_up = 1.0 - smoothstep((a_up - 1.0) / fades.upper);
        let follow_lo = 1.0 - smoothstep((a_lo - 1.0) / fades.lower);
        let lid_w = |upper: bool| if upper { side_up * follow_up } else { side_lo * follow_lo };
        let q2 = match curve {
            Curve::Prominence => q,
            Curve::Lid { upper, point } => {
                let b = match point {
                    0 => hat(s, 0.25, 0.35),
                    1 => hat(s, 0.5, 0.42),
                    _ => hat(s, 0.75, 0.35),
                };
                let w = b * lid_w(upper) * w_r;
                turn_x(q, w * amount / g.r)
            }
            Curve::Opening => {
                let b = if (0.0..=1.0).contains(&s) { (std::f64::consts::PI * s).sin() } else { 0.0 };
                let w = 0.5 * b * w_r;
                turn_x(q, w * amount / g.r * (lid_w(true) - lid_w(false)))
            }
            Curve::LowerFlat => {
                // the straight line between the corners, as an elevation along the eye
                let (th_a, th_b) = (g.upper[0][1], g.upper[g.upper.len() - 1][1]);
                let t = ((phi - g.phi_in) / (g.phi_out - g.phi_in)).clamp(0.0, 1.0);
                let th_line = th_a + t * (th_b - th_a);
                // most in the middle, nothing at the corners, so the corners themselves stay where they are
                let b = if (0.0..=1.0).contains(&s) { (std::f64::consts::PI * s).sin() } else { 0.0 };
                turn_x(q, amount * (th_line - th_lo) * b * lid_w(false) * w_r)
            }
            Curve::Width => {
                let pm = 0.5 * (g.phi_in + g.phi_out);
                let h = 0.5 * (g.phi_out - g.phi_in);
                let d = phi - pm;
                let reach = fades.corner_out * 1.5 / g.r;
                // beyond a corner the skin's turn fades out, but the lining close to the eyeball keeps the corner's turn much further,
                // so the corner never slides past the lining behind it
                let deep = deep_weight(rad - g.r);
                let fade = |r: f64| 1.0 - smoothstep((d.abs() - h) / r);
                let gd = if d.abs() <= h { d } else { d.signum() * h * ((1.0 - deep) * fade(reach) + deep * fade(1.6 * reach)) };
                let band = (th - th_mid).abs() * g.r;
                let w = (1.0 - smoothstep((band - 7.0) / 8.0)) * w_r;
                let turned = turn_y(q, amount * gd * w);
                // the skin under the outer half of the eye, away from the eyeball, keeps its depth and only slides toward the temple, so
                // it stretches along the cheek instead of swinging back round the eyeball with the corner and cutting a trough
                let [near, far] = width_slide();
                let slide = if far > near { smoothstep((rad - g.r - near) / (far - near)) * side_lo * smoothstep((s - 0.4) / 0.3) } else { 0.0 };
                [turned[0], turned[1], turned[2] + slide * (q[2] - turned[2])]
            }
            Curve::CornerDepth { inner } => {
                // mm along the eye from this corner (negative beyond it, toward the nose for the inner corner), and mm up or down
                // from the corner's elevation; the tip moves the whole amount, fading along the lids, into the skin beyond the tip, and
                // from 3 mm above or below (`corner_depth_fades`)
                let sigma = if inner { s * len } else { (1.0 - s) * len };
                let th_c = if inner { g.upper[0][1] } else { g.upper[g.upper.len() - 1][1] };
                let vert = (th - th_c).abs() * g.r;
                let (fa, fb, fv) = if inner { corner_depth_fades() } else { outer_depth_fades() };
                let w_along = if sigma >= 0.0 { 1.0 - smoothstep((sigma - 0.5) / fa) } else { 1.0 - smoothstep((-sigma - 0.5) / fb) };
                let w_vert = 1.0 - smoothstep((vert - 3.0) / fv);
                let w = w_along * w_vert * w_r;
                // along the fixed frame's forward axis, the one the press works along, so that seen from the front nothing moves
                let ax = depth_axis();
                let m = [g.sign * ax[0] * w * amount, ax[1] * w * amount, ax[2] * w * amount];
                [q[0] + m[0], q[1] + m[1], q[2] + m[2]]
            }
            Curve::CornerAlong { inner } | Curve::CornerHeight { inner } | Curve::CornerAngle { inner } | Curve::CornerRound { inner } => {
                // mm along the eye from this corner (negative beyond it), and mm up or down from the corner's elevation
                let sigma = if inner { s * len } else { (1.0 - s) * len };
                let th_c = if inner { g.upper[0][1] } else { g.upper[g.upper.len() - 1][1] };
                let vert = (th - th_c).abs() * g.r;
                let deep = deep_weight(rad - g.r);
                // a corner moved far along the eyeball spreads its turn over a longer stretch of lid and skin (1 mm per mm of
                // the movement), so the skin is not crowded into a ridge
                let spread = if matches!(curve, Curve::CornerAlong { .. }) { amount.abs() } else { 0.0 };
                let (c_in, c_out) = (fades.corner_in + spread, fades.corner_out + spread);
                let w_along = if sigma >= 0.0 {
                    1.0 - smoothstep((sigma - 3.0) / c_in)
                } else {
                    (1.0 - deep) * (1.0 - smoothstep((-sigma - 2.0) / c_out)) + deep * (1.0 - smoothstep((-sigma - 2.0) / (3.0 * c_out)))
                };
                let w_vert = 1.0 - smoothstep((vert - 3.0) / 6.0);
                let w = w_along * w_vert * w_r;
                let toward = if inner { -1.0 } else { 1.0 };
                match curve {
                    Curve::CornerAlong { .. } => turn_y(q, toward * w * amount / g.r),
                    Curve::CornerHeight { .. } => turn_x(q, w * amount / g.r),
                    Curve::CornerAngle { .. } => {
                        // each lid turns about the corner by half the angle, over the first 5 mm from the tip, held to 5 mm and faded
                        // out by 5 + the corner's reach
                        let near = sigma.clamp(0.0, 5.0) * (1.0 - smoothstep((sigma - 5.0) / fades.corner_in));
                        let mm = near * (0.5 * amount.to_radians()).tan();
                        let lid = if above { side_up * follow_up } else { side_lo * follow_lo };
                        let dir = if above { 1.0 } else { -1.0 };
                        turn_x(q, dir * lid * mm * w_r / g.r)
                    }
                    _ => {
                        let near = hat(sigma, 2.0, 2.2) * if sigma > 0.0 { 1.0 } else { 0.0 };
                        let lid = if above { side_up * follow_up } else { side_lo * follow_lo };
                        let dir = if above { 1.0 } else { -1.0 };
                        turn_x(q, dir * lid * near * 0.5 * amount * w_r / g.r)
                    }
                }
            }
        };
        out[v] = [g.sign * (q2[0] - q[0]) / 1000.0, (q2[1] - q[1]) / 1000.0, (q2[2] - q[2]) / 1000.0];
    }
    // merged copies move together: each takes its group's mean movement
    merge_copies(base, &mut out);
    out
}

/// Vertices at the same place on the base head (seams) take the same movement: the one of the copy that moves most.
fn merge_copies(base: &[V3], m: &mut [V3]) {
    let mut idx: Vec<usize> = (0..base.len()).collect();
    idx.sort_by(|&a, &b| base[a].partial_cmp(&base[b]).unwrap());
    let mut i = 0;
    while i < idx.len() {
        let mut j = i + 1;
        while j < idx.len() && crate::length(crate::sub(base[idx[j]], base[idx[i]])) < 1e-9 {
            j += 1;
        }
        if j - i > 1 {
            let best = idx[i..j].iter().copied().max_by(|&a, &b| crate::length(m[a]).total_cmp(&crate::length(m[b]))).unwrap();
            let mv = m[best];
            for &k in &idx[i..j] {
                m[k] = mv;
            }
        }
        i = j;
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// The press: the lid's edge onto the eyeball, after every slider and the smoothing.

/// How far outside the eyeball the pressed edge lies at least, in mm.
pub const PRESS_GAP_MM: f64 = 0.15;

/// One eye's press, fixed for the mesh: each pressed vertex follows a smooth blend of the edge's corrections around it.
#[derive(Clone, Debug)]
pub struct PressEye {
    pub ring: Vec<u32>,
    pub edge: Vec<u32>,
    /// Each edge vertex's distance outside the eyeball on the base head (mm), at least `PRESS_GAP_MM`: its fixed thickness.
    pub gap: Vec<f64>,
    /// Each edge vertex's weight from the corners, 1 along the lids, lower at a corner's tip.
    pub corner: Vec<f64>,
    /// Each edge vertex's share of the inner corner's freedom (1 at the inner corner's tip, 0 from `INNER_FREE` of the edge on):
    /// beyond the eyeball's silhouette seen from the front, the press never pulls it toward the eyeball's centre.
    pub inner: Vec<f64>,
    /// For each lid (upper, lower), the edge vertices from the inner corner's tip to the first one the press seats on the eyeball
    /// (`INNER_SPAN` of the lid), in order, and each one's place along that stretch (0 at the tip, 1 at the last).
    pub span: Vec<(Vec<usize>, Vec<f64>)>,
    /// For each pressed vertex: the edge vertices it follows and their weights, which already hold the fade with distance.
    pub follow: Vec<(u32, Vec<(usize, f64)>)>,
    /// For each edge vertex: the edge vertices whose corrections it averages (Gaussian along the edge), so that neighbours never
    /// take very different corrections.
    pub smooth: Vec<Vec<(usize, f64)>>,
    /// Each lid (upper, then lower) as edge places from the inner corner's tip to the outer corner's tip, and each place's share of
    /// the lid's length on the base head: the smoother lays the edge points out again at these shares.
    pub lids: Vec<(Vec<usize>, Vec<f64>)>,
    /// Whether each edge vertex lies on the inner half of the eye (share from the inner corner under a half).
    pub half_inner: Vec<bool>,
    /// Each edge vertex's share of the outer corner's freedom (1 at the outer corner's tip, 0 from `outer_free` of the edge on): where
    /// it lies outside the eyeball, the press does not pull it in, so the lid's end may lie behind the eyeball's silhouette.
    pub outer: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct Press {
    pub eyes: Vec<PressEye>,
    pub vertex_count: usize,
}

/// The press's sizes, in mm: how far round an edge vertex its correction spreads along the edge (`blend`), the distance from the edge
/// over which every vertex takes the whole correction (`hold`), and the distance over which it then fades out (`fade`). Distances are
/// straight-line distances on the base head, so both sides of a fold, which lie close together, move together.
#[derive(Clone, Copy, Debug)]
pub struct PressSizes {
    pub blend: f64,
    pub hold: f64,
    pub fade: f64,
    /// How many times the press is applied; each pass presses what the last one left.
    pub passes: usize,
    /// The share of the correction a corner's tip takes (1 presses the tips like the rest of the edge).
    pub tip: f64,
    /// The spread of the averaging of the corrections along the edge, in mm.
    pub smooth: f64,
    /// How far from each corner's tip the press fades in, as a share of the eye's edge.
    pub corner_fade: f64,
    /// The most an edge vertex stays outside the eyeball after the press, in mm.
    pub gap_cap: f64,
}

impl Default for PressSizes {
    fn default() -> Self {
        Self { blend: 2.0, hold: 4.0, fade: 18.0, passes: 3, tip: 1.0, smooth: 1.0, corner_fade: 0.12, gap_cap: 0.3 }
    }
}

impl Press {
    pub fn new(base: &[V3], _triangles: &[[u32; 3]]) -> Self {
        let n = base.len();
        let sz = PressSizes::default();
        let eyes = edges()
            .iter()
            .map(|e| {
                let (c, r) = ball(base, &e.ring);
                let gap: Vec<f64> = e.edge.iter().map(|&v| (crate::length(crate::sub(base[v as usize], c)) * 1000.0 - r).max(PRESS_GAP_MM)).collect();
                // the base head's own edge leaves the eyeball by up to 3 mm near the inner corner; no edge vertex keeps more than the
                // edge's median thickness or `gap_cap`, so the lids sit on the eyeball into the corners
                let mut sorted = gap.clone();
                sorted.sort_by(f64::total_cmp);
                let median = sorted[sorted.len() / 2];
                let keep = false;
                let cap = median.min(sz.gap_cap);
                let gap: Vec<f64> = if keep { gap } else { gap.iter().map(|&g| g.min(cap)).collect() };
                let corner: Vec<f64> = e.share.iter().map(|&s| {
                    let tipness = 1.0 - smoothstep(s.min(1.0 - s) / sz.corner_fade);
                    1.0 - (1.0 - sz.tip) * tipness
                }).collect();
                let edge_pos: Vec<V3> = e.edge.iter().map(|&v| base[v as usize]).collect();
                let m = edge_pos.len();
                let reach = sz.hold + sz.fade;
                let mut follow = Vec::new();
                for v in 0..n {
                    if e.ring.contains(&(v as u32)) {
                        continue;
                    }
                    let p = base[v];
                    if crate::length(crate::sub(p, c)) * 1000.0 > r + reach + 4.0 {
                        continue;
                    }
                    // distance to the edge as a closed line, in mm
                    let mut d_line = f64::MAX;
                    for k in 0..m {
                        let (a, b) = (edge_pos[k], edge_pos[(k + 1) % m]);
                        let ab = crate::sub(b, a);
                        let t = (crate::dot(crate::sub(p, a), ab) / crate::dot(ab, ab).max(1e-18)).clamp(0.0, 1.0);
                        d_line = d_line.min(crate::length(crate::sub(p, crate::add(a, crate::scale(ab, t)))) * 1000.0);
                    }
                    if d_line >= reach {
                        continue;
                    }
                    let fall = 1.0 - smoothstep((d_line - sz.hold) / sz.fade);
                    let ws: Vec<f64> = edge_pos.iter().map(|&q| {
                        let d = crate::length(crate::sub(p, q)) * 1000.0 - d_line;
                        (-(d / sz.blend).powi(2)).exp()
                    }).collect();
                    let tot: f64 = ws.iter().sum();
                    let list: Vec<(usize, f64)> = ws.iter().enumerate().filter(|(_, w)| **w / tot > 1e-3).map(|(k, w)| (k, w / tot * fall)).collect();
                    follow.push((v as u32, list));
                }
                // the averaging along the edge: arc length round the closed edge, Gaussian weights
                let mut arc = vec![0.0; m + 1];
                for k in 0..m {
                    arc[k + 1] = arc[k] + crate::length(crate::sub(edge_pos[(k + 1) % m], edge_pos[k])) * 1000.0;
                }
                let total = arc[m];
                let smooth: Vec<Vec<(usize, f64)>> = (0..m)
                    .map(|a| {
                        let ws: Vec<(usize, f64)> = (0..m)
                            .map(|b| {
                                let d = (arc[a] - arc[b]).abs();
                                let d = d.min(total - d);
                                (b, (-(d / sz.smooth.max(1e-6)).powi(2)).exp())
                            })
                            .filter(|x| x.1 > 1e-3)
                            .collect();
                        let t: f64 = ws.iter().map(|x| x.1).sum();
                        ws.into_iter().map(|(b, w)| (b, w / t)).collect()
                    })
                    .collect();
                let inner: Vec<f64> = e.share.iter().map(|&s| 1.0 - smoothstep(s / inner_free())).collect();
                let tip = (0..e.edge.len()).min_by(|&a, &b| e.share[a].total_cmp(&e.share[b])).unwrap();
                let span: Vec<(Vec<usize>, Vec<f64>)> = [true, false]
                    .iter()
                    .map(|&up| {
                        let mut ks: Vec<usize> = (0..e.edge.len()).filter(|&k| k != tip && e.upper[k] == up && e.share[k] <= inner_span()).collect();
                        ks.sort_by(|&a, &b| e.share[a].total_cmp(&e.share[b]));
                        ks.insert(0, tip);
                        let last = e.share[*ks.last().unwrap()].max(1e-9);
                        let ts = ks.iter().map(|&k| e.share[k] / last).collect();
                        (ks, ts)
                    })
                    .collect();
                // the inner corner's tip follows its own correction alone, not a blend of its neighbours', so that it keeps its place
                // seen from the front
                let mut follow = follow;
                let tip_at = base[e.edge[tip] as usize];
                for (v, list) in follow.iter_mut() {
                    let d = crate::length(crate::sub(base[*v as usize], tip_at)) * 1000.0;
                    if d < tip_reach() {
                        // the skin and the lining right at the tip go with it, so no triangle there turns over
                        let fall: f64 = list.iter().map(|x| x.1).sum();
                        let own = 1.0 - smoothstep(d / tip_reach());
                        for x in list.iter_mut() {
                            x.1 *= 1.0 - own;
                        }
                        list.push((tip, own * fall));
                    }
                }
                let outer_tip = (0..e.edge.len()).max_by(|&a, &b| e.share[a].total_cmp(&e.share[b])).unwrap();
                // the skin and the lining near the outer corner's tip go with the tip alone, so that a tip set back behind the eyeball
                // takes the skin beyond it back too, instead of the lid next to it, which the eyeball holds forward
                let outer_at = base[e.edge[outer_tip] as usize];
                if outer_reach() > 0.0 {
                    for (v, list) in follow.iter_mut() {
                        let d = crate::length(crate::sub(base[*v as usize], outer_at)) * 1000.0;
                        if d < outer_reach() {
                            let fall: f64 = list.iter().map(|x| x.1).sum();
                            let own = 1.0 - smoothstep(d / outer_reach());
                            for x in list.iter_mut() {
                                x.1 *= 1.0 - own;
                            }
                            list.push((outer_tip, own * fall));
                        }
                    }
                }
                let lids: Vec<(Vec<usize>, Vec<f64>)> = [true, false]
                    .iter()
                    .map(|&up| {
                        let mut ks: Vec<usize> = (0..e.edge.len()).filter(|&k| k != tip && k != outer_tip && e.upper[k] == up).collect();
                        ks.sort_by(|&a, &b| e.share[a].total_cmp(&e.share[b]));
                        ks.insert(0, tip);
                        ks.push(outer_tip);
                        let mut arc = vec![0.0];
                        for w in ks.windows(2) {
                            arc.push(arc.last().unwrap() + crate::length(crate::sub(edge_pos[w[1]], edge_pos[w[0]])));
                        }
                        let total = arc.last().copied().unwrap_or(1.0).max(1e-12);
                        (ks, arc.iter().map(|a| a / total).collect())
                    })
                    .collect();
                // with the smoother, each edge vertex (and each copy of it on a seam) takes its own correction alone, so that the
                // smoothed edge is reached exactly; the rows behind it blend the corrections of the edge around them
                for (v, list) in follow.iter_mut() {
                    let p = base[*v as usize];
                    if let Some(k) = edge_pos.iter().position(|&q| crate::length(crate::sub(p, q)) < 1e-9) {
                        let fall: f64 = list.iter().map(|x| x.1).sum::<f64>().max(1.0);
                        *list = vec![(k, fall)];
                    }
                }
                let half_inner = e.share.iter().map(|&s| s < 0.5).collect();
                let outer = e.share.iter().map(|&s| if outer_free() > 0.0 { 1.0 - smoothstep((1.0 - s) / outer_free()) } else { 0.0 }).collect();
                PressEye { ring: e.ring.clone(), edge: e.edge.clone(), gap, corner, inner, span, follow, smooth, lids, half_inner, outer }
            })
            .collect();
        Self { eyes, vertex_count: n }
    }

    /// Presses each eye's edge onto its eyeball (the eyeball read from the head's own centre ring). Each edge vertex's correction is the
    /// distance that would bring it to its fixed thickness outside the eyeball. Every vertex near the edge moves by a blend of the
    /// corrections of the edge around it, each along its edge vertex's direction from the eyeball's centre, by the whole amount up to
    /// `hold` mm from the edge and fading over `fade` mm. The press is applied `passes` times.
    pub fn apply(&self, p: &mut [V3], _start: Option<&[V3]>) {
        if p.len() != self.vertex_count {
            return;
        }
        let passes = PressSizes::default().passes;
        for _ in 0..passes {
            let mut all = Vec::new();
            for e in &self.eyes {
                let (c, r) = ball(p, &e.ring);
                let raw = e.depth_corrections(&|v| p[v as usize], c, r);
                for (v, list) in &e.follow {
                    let d = list.iter().fold([0.0; 3], |a, &(k, w)| crate::add(a, crate::scale(raw[k], w / 1000.0)));
                    all.push((*v, d));
                }
            }
            for (v, d) in all {
                p[v as usize] = crate::add(p[v as usize], d);
            }
        }
    }
}

/// The share of the depth press an edge point takes from how far from the eyeball's centre it lies seen from the front, as a share of
/// the pressed sphere's radius: all of it up to 0.8, none from 0.97, where the sphere's surface turns away from the view and no eyeball
/// lies behind the point. There the corner's own curve holds the depth.
fn depth_axis() -> V3 {
    crate::measure::Frame::head().axes[2]
}

/// How far along the edge from the inner corner's tip (a share of the edge) the press keeps from pulling the edge in past the
/// silhouette.
fn inner_free() -> f64 {
    0.06
}

/// How much of each lid from the inner corner's tip (a share of the lid) the press spreads evenly between the tip and the eyeball.
fn inner_span() -> f64 {
    0.14
}

/// How far from the inner corner's tip (mm, on the base head) the vertices go with the tip's own correction, fading out.
fn tip_reach() -> f64 {
    2.0
}

/// How far along the edge from the outer corner's tip (a share of the edge) the press keeps from pulling the edge in onto the eyeball.
fn outer_free() -> f64 {
    0.08
}

/// How far from the outer corner's tip (mm, on the base head) the vertices go with the tip's own correction, fading out.
fn outer_reach() -> f64 {
    4.0
}

fn silhouette_fade(share: f64) -> f64 {
    1.0 - smoothstep((share - 0.8) / 0.17)
}

/// The steepest the free stretch at the inner corner may climb from the tip to the eyeball, as depth over distance seen from the
/// front.
fn free_slope() -> f64 {
    0.5
}

/// Over how many mm from each corner's tip the outline's smoothing fades in.
fn tip_fade() -> f64 {
    3.0
}

/// How many times the quadratic smoothing runs.
fn quad_passes() -> usize {
    2
}

/// One pass of the quadratic smoothing: each inner point moves to the least-squares quadratic, in length along the lid, through the
/// points from two before it to two after it (fewer next to a tip). The tips stay.
fn quad_pass(p: &[V3]) -> Vec<V3> {
    let n = p.len();
    let mut s = vec![0.0; n];
    for i in 1..n {
        s[i] = s[i - 1] + crate::length(crate::sub(p[i], p[i - 1]));
    }
    (0..n)
        .map(|i| {
            if i == 0 || i == n - 1 {
                return p[i];
            }
            let (a, b) = (i.saturating_sub(2), (i + 2).min(n - 1));
            let mut m = [[0.0f64; 3]; 3];
            let mut rhs = [[0.0f64; 3]; 3];
            for j in a..=b {
                let x = s[j] - s[i];
                let r = [1.0, x, x * x];
                for u in 0..3 {
                    for v in 0..3 {
                        m[u][v] += r[u] * r[v];
                    }
                    for c in 0..3 {
                        rhs[c][u] += r[u] * p[j][c];
                    }
                }
            }
            [0, 1, 2].map(|c| solve3(m, rhs[c])[0])
        })
        .collect()
}

/// A lid's points smoothed as one curve, then laid out again along it at `shares` of its length. Each inner point moves to the
/// quadratic (in length along the lid) fitted through it and its two neighbours on each side (`quad_pass`): a curve that bends evenly is
/// kept as it is, and only the wobble from point to point goes.
fn smooth_lid(p: &[V3]) -> Vec<V3> {
    let own: Vec<f64> = {
        let mut s = vec![0.0; p.len()];
        for i in 1..p.len() {
            s[i] = s[i - 1] + crate::length(crate::sub(p[i], p[i - 1]));
        }
        let t = s.last().copied().unwrap_or(1.0).max(1e-12);
        s.iter().map(|x| x / t).collect()
    };
    let mut twice = p.to_vec();
    for _ in 0..quad_passes() {
        twice = quad_pass(&twice);
    }
    resample(&twice, &own)
}

/// Points laid out along a polyline at `shares` of its length, its ends kept.
fn resample(twice: &[V3], shares: &[f64]) -> Vec<V3> {
    let n = twice.len();
    let mut s = vec![0.0; n];
    for i in 1..n {
        s[i] = s[i - 1] + crate::length(crate::sub(twice[i], twice[i - 1]));
    }
    let total = s[n - 1].max(1e-12);
    (0..n)
        .map(|i| {
            if i == 0 || i == n - 1 {
                return twice[i];
            }
            let want = shares[i] * total;
            let j = s.partition_point(|&x| x < want).clamp(1, n - 1);
            let t = ((want - s[j - 1]) / (s[j] - s[j - 1]).max(1e-12)).clamp(0.0, 1.0);
            crate::add(twice[j - 1], crate::scale(crate::sub(twice[j], twice[j - 1]), t))
        })
        .collect()
}

impl PressEye {
    /// The press with the lid smoother (round 6), as corrections (mm vectors) for each edge vertex:
    /// 1. Each lid's outline seen from the front (across the forward axis of `measure::Frame::head`) is smoothed as one curve
    ///    (`smooth_lid`), its corners' tips kept, the smoothing fading in over `tip_fade` mm from each tip.
    /// 2. Each edge point with the eyeball behind it seen from the front is pressed onto the eyeball. On the inner half of the eye only in
    ///    depth, so the smoothed outline stays; on the outer half as before.
    /// 3. At the inner corner, from the tip to the first point that the eyeball lies behind and that the press moves in depth by no more
    ///    than `free_slope` times its distance from the tip, the lid is free: its points keep the sliders' depth plus the press's change
    ///    at that first point, fading to none at the tip. The tip is only pushed out of the eyeball, never pulled in, so Inner corner
    ///    depth keeps its effect.
    fn smoothed_corrections(&self, at: &dyn Fn(u32) -> V3, c: V3, r: f64) -> Vec<V3> {
        let ax = depth_axis();
        let q: Vec<V3> = self.edge.iter().map(|&v| crate::scale(crate::sub(at(v), c), 1000.0)).collect();
        let split = |x: V3| -> (V3, f64) {
            let along = crate::dot(x, ax);
            (crate::sub(x, crate::scale(ax, along)), along)
        };
        // 1. the outline seen from the front, smoothed lid by lid (depth is set by the press below)
        let mut q2 = q.clone();
        for (ks, _) in self.lids.iter() {
            let flat: Vec<V3> = ks.iter().map(|&k| split(q[k]).0).collect();
            let smooth = smooth_lid(&flat);
            // the smoothing fades in over `tip_fade` mm from each corner's tip, where the corners' own sliders hold the shape
            let mut arc = vec![0.0; flat.len()];
            for i in 1..flat.len() {
                arc[i] = arc[i - 1] + crate::length(crate::sub(flat[i], flat[i - 1]));
            }
            let total = arc.last().copied().unwrap_or(0.0);
            for (j, &k) in ks.iter().enumerate() {
                let f = smoothstep(arc[j].min(total - arc[j]) / tip_fade());
                let moved = crate::add(flat[j], crate::scale(crate::sub(smooth[j], flat[j]), f));
                q2[k] = crate::add(moved, crate::scale(ax, split(q[k]).1));
            }
        }
        // 2. the press of every point, on the smoothed outline
        let tip = self.lids.first().map(|l| l.0[0]);
        let mut target: Vec<V3> = (0..self.edge.len())
            .map(|k| {
                let x = q2[k];
                let big = r + self.gap[k];
                let (flat, along) = split(x);
                let rho = crate::length(flat);
                let share = rho / big;
                if Some(k) == tip {
                    // the tip: only out of the eyeball, forward, where the eyeball lies behind it
                    if share < 1.0 {
                        let front = (big * big - rho * rho).max(0.0).sqrt();
                        return crate::add(flat, crate::scale(ax, along.max(front)));
                    }
                    return x;
                }
                // on the inner half of the eye the press moves the edge in depth only, so the outline seen from the front stays the smoothed one
                // (a point where that would make a cliff joins the free stretch below); the outer half hands over to the press toward the
                // eyeball's centre near the silhouette, as before
                let w = if self.half_inner[k] { if share < 1.0 { 1.0 } else { 0.0 } } else { silhouette_fade(share) };
                let depth = if share < 1.0 { (big * big - rho * rho).max(0.0).sqrt() - along } else { 0.0 };
                // toward the outer corner's tip the edge is only pushed out of the eyeball, never pulled in onto it
                let depth = depth - depth.min(0.0) * self.outer[k];
                let dist = crate::length(x).max(1e-9);
                let pull = (big - dist).min(0.0) * self.inner[k].max(self.outer[k]);
                let radial = crate::scale(x, (big - dist - pull) / dist);
                crate::add(x, crate::add(crate::scale(ax, w * depth), crate::scale(radial, 1.0 - w)))
            })
            .collect();
        // 3. the free stretch at the inner corner, lid by lid
        if let Some(tip) = tip {
            for (ks, _) in &self.lids {
                let seated = |k: usize| {
                    let (flat, _) = split(q2[k]);
                    crate::length(flat) < r + self.gap[k]
                };
                // the lid leaves the eyeball at the first point from the tip that the eyeball lies behind and that the press moves in
                // depth by no more than `free_slope` times its distance from the tip seen from the front: nearer the tip, pressing
                // would make a cliff between the tip, where the sliders hold it, and the eyeball
                let tip_flat = split(q2[tip]).0;
                let gentle = |k: usize| {
                    let moved = (split(target[k]).1 - split(q2[k]).1).abs();
                    moved <= free_slope() * crate::length(crate::sub(split(q2[k]).0, tip_flat))
                };
                let Some(j) = ks.iter().position(|&k| k != tip && seated(k) && gentle(k)) else { continue };
                if j < 2 || j + 1 >= ks.len() {
                    continue;
                }
                let leave = ks[j];
                let (fl, zl) = split(target[leave]);
                // front-view distance from the leaving point toward the tip
                let mut dist = vec![0.0; j + 1];
                let mut prev = fl;
                for i in (0..j).rev() {
                    let f = split(q2[ks[i]]).0;
                    dist[i] = dist[i + 1] + crate::length(crate::sub(f, prev));
                    prev = f;
                }
                let total = dist[0].max(1e-6);
                for i in 1..j {
                    let k = ks[i];
                    let (flat, _) = split(q2[k]);
                    let s = dist[i];
                    // the sliders' own depth, carrying the press's change at the leaving point, fading to none at the tip
                    let own = split(q2[k]).1;
                    let mut z = own + (zl - split(q2[leave]).1) * (1.0 - smoothstep(s / total));
                    let big = r + self.gap[k];
                    let rho = crate::length(flat);
                    if rho < big {
                        z = z.max((big * big - rho * rho).sqrt());
                    }
                    target[k] = crate::add(flat, crate::scale(ax, z));
                }
            }
        }
        target.iter().zip(&q).map(|(&t, &x)| crate::sub(t, x)).collect()
    }
}

impl PressEye {
    /// The press's corrections (mm vectors) for each edge vertex, smoothed along the edge and faded toward the corners. Where the
    /// eyeball lies behind an edge vertex seen from the front, the vertex is moved in depth (along the fixed frame's forward axis,
    /// `measure::Frame::head`) onto the sphere of the eyeball's radius plus its fixed thickness, keeping its place seen from the front.
    /// Toward the eyeball's silhouette, where the sphere turns away from the view, the depth press hands over smoothly to a press toward
    /// the eyeball's centre, so the lids sit on the eyeball as far into the corners as it reaches.
    fn depth_corrections(&self, at: &dyn Fn(u32) -> V3, c: V3, r: f64) -> Vec<V3> {
        self.smoothed_corrections(at, c, r)
    }
}

impl Press {
    /// The vertices the press reads: each eye's centre ring and edge.
    pub fn read_vertices(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self.eyes.iter().flat_map(|e| e.ring.iter().chain(&e.edge).copied()).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// The press of `apply` on part of a head: `p` holds the vertices whose slot is not `u32::MAX` (`slot[vertex]` is the place in
    /// `p`), and must hold every vertex of `read_vertices`. The vertices held move exactly as `apply` (without its trial switches) moves
    /// them on the whole head.
    pub fn apply_some(&self, p: &mut [V3], slot: &[u32]) {
        let passes = PressSizes::default().passes;
        let at = |p: &[V3], v: u32| p[slot[v as usize] as usize];
        let ball_of = |p: &[V3], ring: &[u32]| {
            let pts: Vec<V3> = ring.iter().map(|&v| at(p, v)).collect();
            let ids: Vec<u32> = (0..pts.len() as u32).collect();
            ball(&pts, &ids)
        };
        for _ in 0..passes {
            let mut all = Vec::new();
            for e in &self.eyes {
                let (c, r) = ball_of(p, &e.ring);
                let raw = e.depth_corrections(&|v| at(p, v), c, r);
                for (v, list) in &e.follow {
                    let s = slot[*v as usize];
                    if s == u32::MAX {
                        continue;
                    }
                    let d = list.iter().fold([0.0; 3], |a, &(k, w)| crate::add(a, crate::scale(raw[k], w / 1000.0)));
                    all.push((s, d));
                }
            }
            for (s, d) in all {
                p[s as usize] = crate::add(p[s as usize], d);
            }
        }
    }
}

/// The prepared press, when the head mesh is ours and the press is not switched off.
pub fn prepared() -> Option<&'static Press> {
    PRESS.get()
}

fn solve3(m: [[f64; 3]; 3], b: [f64; 3]) -> [f64; 3] {
    let det = |m: [[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(m);
    if d.abs() < 1e-12 {
        return [0.0; 3];
    }
    let mut out = [0.0; 3];
    for k in 0..3 {
        let mut mk = m;
        for r in 0..3 {
            mk[r][k] = b[r];
        }
        out[k] = det(mk) / d;
    }
    out
}

static PRESS: OnceLock<Press> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static RIG: OnceLock<head_eyes::piece::LidRig> = OnceLock::new();

/// Prepares the press and the lid pieces' rig for the head mesh. Called once wherever the base head and its triangles are known.
pub fn init(base: &[V3], triangles: &[[u32; 3]]) {
    if base.len() == 4750 {
        let _ = PRESS.get_or_init(|| Press::new(base, triangles));
        #[cfg(not(target_arch = "wasm32"))]
        let _ = RIG.get_or_init(|| head_eyes::piece::LidRig::new(triangles, base, eye_edges()));
    }
}

/// Both eyes' edges as the lid piece and the caruncle read them, the +x eye first.
#[cfg(not(target_arch = "wasm32"))]
pub fn eye_edges() -> &'static [head_eyes::piece::EyeEdge] {
    static E: OnceLock<Vec<head_eyes::piece::EyeEdge>> = OnceLock::new();
    E.get_or_init(|| {
        edges()
            .iter()
            .map(|e| head_eyes::piece::EyeEdge {
                edge: e.edge.clone(),
                upper: e.upper.clone(),
                inner_corner: e.inner,
                outer_corner: e.outer,
                centre: e.ring.clone(),
                temple_plus_x: e.sign > 0.0,
            })
            .collect()
    })
}

/// The lid pieces' rig, once `init` has run (native builds only; the browser builds the pieces in `head_eyes`'s own module).
#[cfg(not(target_arch = "wasm32"))]
pub fn lid_rig() -> Option<&'static head_eyes::piece::LidRig> {
    RIG.get()
}

/// Whether the lid pieces' rig is prepared (never in the browser).
pub fn has_lid_rig() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return RIG.get().is_some();
    #[cfg(target_arch = "wasm32")]
    false
}

/// Presses a built head's lid edges onto its eyeballs, when the press is prepared and not switched off.
pub fn press(p: &mut [V3], start: &[V3]) {
    if let Some(pr) = PRESS.get() {
        pr.apply(p, Some(start));
    }
}
