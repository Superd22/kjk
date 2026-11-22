//! Checks of the lid piece over the whole blink, on one head.
//!
//! Each check turns the piece through nine poses from open to closed and
//! casts rays against the skin as it is drawn (without the faces the piece
//! hides) and against the piece itself.
//! - Into the eyeball: the least distance of the piece from the eyeball's
//!   surface. Below 0 the piece cuts into the eyeball.
//! - Gap at the edge: the largest gap between the piece's edge and the
//!   eyeball, where that edge can be seen from the front.
//! - Poking through skin: points of the piece that lie outside the skin's
//!   front, away from the lids' own rolls, which the piece may cover.
//! - Eyeball showing when closed: the eyeball's area seen past the closed
//!   piece and the skin, from the front and from three-quarter.

use crate::piece::{EyePiece, EyeRig, V3, eyeball_radius, turn};

/// The poses checked, from open (0) to closed.
pub const POSES: usize = 9;

/// The results for one eye, lengths in mm and areas in mm².
#[derive(Clone, Copy, Debug, Default)]
pub struct EyeCheck {
    pub into_eyeball: f64,
    pub edge_gap: f64,
    pub poke_points: u32,
    pub poke_depth: f64,
    /// The same for the lash strip, which is expected to rest on the lower lid when closed.
    pub lash_poke_points: u32,
    pub lash_poke_depth: f64,
    pub shown_closed_front: f64,
    pub shown_closed_three: f64,
    pub shown_open_front: f64,
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn norm(a: V3) -> f64 {
    dot(a, a).sqrt()
}

/// The distances along a ray at which it meets a triangle (Möller and Trumbore).
fn hit(o: V3, d: V3, t: &[V3; 3]) -> Option<f64> {
    let e1 = sub(t[1], t[0]);
    let e2 = sub(t[2], t[0]);
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-14 {
        return None;
    }
    let inv = 1.0 / det;
    let s = sub(o, t[0]);
    let u = dot(s, p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let dist = dot(e2, q) * inv;
    (dist > 0.0).then_some(dist)
}

/// Checks one eye's piece on a head (positions in metres).
pub fn check_eye(rig: &EyeRig, triangles: &[[u32; 3]], head: &[V3], piece: &EyePiece) -> EyeCheck {
    let n = rig.centre.len() as f64;
    let mut c = [0.0; 3];
    for &v in &rig.centre {
        for k in 0..3 {
            c[k] += head[v as usize][k] / n;
        }
    }
    let radius = eyeball_radius(head, &rig.centre);
    let r_eye = radius / 1000.0;
    let hidden: std::collections::HashSet<u32> = piece.hidden_faces.iter().copied().collect();
    let coverable: std::collections::HashSet<u32> = rig.coverable_faces.iter().copied().collect();
    let skin: Vec<(u32, [V3; 3])> = rig
        .near_faces
        .iter()
        .filter(|f| !hidden.contains(f))
        .map(|&f| (f, triangles[f as usize].map(|v| head[v as usize])))
        .collect();
    let at = |angle: f64| -> (Vec<V3>, Vec<V3>) {
        let p = piece.positions.iter().map(|q| turn(q.map(|v| v as f64), piece.pivot, angle)).collect();
        let l = piece.lash_positions.iter().map(|q| turn(q.map(|v| v as f64), piece.pivot, angle)).collect();
        (p, l)
    };
    let mut out = EyeCheck { into_eyeball: f64::INFINITY, ..Default::default() };
    let toward_front = [0.0, 0.0, 1.0];
    for k in 0..POSES {
        let angle = piece.closed_angle * k as f64 / (POSES - 1) as f64;
        let (p, lash) = at(angle);
        for (i, q) in p.iter().enumerate() {
            let d = (norm(sub(*q, c)) - r_eye) * 1000.0;
            out.into_eyeball = out.into_eyeball.min(d);
            // the edge's gap, where the edge is seen from the front past the skin
            if piece.part[i] == 3 && (sub(*q, c)[0].abs() * 1000.0) < radius - 1.0 {
                let blocked = skin.iter().any(|(_, t)| hit(*q, toward_front, t).is_some());
                if !blocked {
                    out.edge_gap = out.edge_gap.max(d);
                }
            }
        }
        for l in &lash {
            out.into_eyeball = out.into_eyeball.min((norm(sub(*l, c)) - r_eye) * 1000.0);
        }
        // poking through skin: a point of the outer face (or the lash) that a ray from the eye's centre reaches only after passing
        // the skin's front, where that skin is not a lid's own roll
        for (i, q) in p.iter().enumerate().filter(|(i, _)| piece.part[*i] == 0).map(|(i, q)| (Some(i), *q)).chain(lash.iter().map(|q| (None, *q))) {
            let v = sub(q, c);
            let len = norm(v);
            let d = v.map(|x| x / len);
            let mut beyond = false;
            let mut nearer: Option<(u32, f64)> = None;
            for (f, t) in &skin {
                if let Some(s) = hit(c, d, t) {
                    if s > len + 1e-5 {
                        beyond = true;
                        break;
                    }
                    // only skin that faces outward, away from the eyeball, can be poked through; the lining faces the eyeball
                    let nrm = cross(sub(t[1], t[0]), sub(t[2], t[0]));
                    if dot(nrm, d) > 0.0 && nearer.is_none_or(|(_, b)| s > b) {
                        nearer = Some((*f, s));
                    }
                }
            }
            if !beyond && let Some((f, s)) = nearer {
                if !coverable.contains(&f) && i.is_none() {
                    out.lash_poke_points += 1;
                    out.lash_poke_depth = out.lash_poke_depth.max((len - s) * 1000.0);
                } else if !coverable.contains(&f) {
                    if std::env::var("LID_DEBUG").is_ok() && k % 4 == 0 {
                        let l = [rig.sign * (q[0] - c[0]) * 1000.0, (q[1] - c[1]) * 1000.0, (q[2] - c[2]) * 1000.0];
                        eprintln!("poke pose {k} part {:?} x {:6.2} y {:6.2} z {:6.2} depth {:.2} face {f}", i.map(|i| piece.part[i]), l[0], l[1], l[2], (len - s) * 1000.0);
                    }
                    out.poke_points += 1;
                    out.poke_depth = out.poke_depth.max((len - s) * 1000.0);
                }
            }
        }
    }
    // the eyeball's area seen past the skin and the piece, at rest and closed
    let (closed, closed_lash) = at(piece.closed_angle);
    let tris_at = |p: &[V3], l: &[V3]| -> Vec<[V3; 3]> {
        let mut v: Vec<[V3; 3]> = piece.triangles.iter().map(|t| t.map(|i| p[i as usize])).collect();
        v.extend(piece.lash_triangles.iter().map(|t| t.map(|i| l[i as usize])));
        v
    };
    let (open, open_lash) = at(0.0);
    let closed_tris = tris_at(&closed, &closed_lash);
    let open_tris = tris_at(&open, &open_lash);
    let toward = |deg: f64, sign: f64| -> V3 {
        let a = deg.to_radians();
        [sign * a.sin(), 0.0, a.cos()]
    };
    let side = if rig.sign > 0.0 { 1.0 } else { -1.0 };
    let shown = |dir: V3, piece_tris: &[[V3; 3]]| -> f64 {
        // points on the eyeball facing the camera, each standing for an equal share of the eyeball's projected disc
        let steps = 70;
        let cell = (2.0 * radius / steps as f64).powi(2);
        let up = [0.0, 1.0, 0.0];
        let right = cross(up, dir);
        let mut area = 0.0;
        for a in 0..steps {
            for b in 0..steps {
                let u = (a as f64 + 0.5) / steps as f64 * 2.0 - 1.0;
                let w = (b as f64 + 0.5) / steps as f64 * 2.0 - 1.0;
                let h = 1.0 - u * u - w * w;
                if h <= 0.0 {
                    continue;
                }
                let local = [u * right[0] + w * up[0] + h.sqrt() * dir[0], u * right[1] + w * up[1] + h.sqrt() * dir[1], u * right[2] + w * up[2] + h.sqrt() * dir[2]];
                let q = [c[0] + r_eye * local[0], c[1] + r_eye * local[1], c[2] + r_eye * local[2]];
                let o = [q[0] + 1e-5 * dir[0], q[1] + 1e-5 * dir[1], q[2] + 1e-5 * dir[2]];
                if skin.iter().any(|(_, t)| hit(o, dir, t).is_some()) {
                    continue;
                }
                if piece_tris.iter().any(|t| hit(o, dir, t).is_some()) {
                    continue;
                }
                if std::env::var("LID_SHOWN").is_ok() {
                    eprintln!("shown x {:6.2} y {:6.2} z {:6.2}", rig.sign * (q[0] - c[0]) * 1000.0, (q[1] - c[1]) * 1000.0, (q[2] - c[2]) * 1000.0);
                }
                area += cell;
            }
        }
        area
    };
    out.shown_closed_front = shown(toward_front, &closed_tris);
    out.shown_closed_three = shown(toward(40.0, side), &closed_tris);
    out.shown_open_front = shown(toward_front, &open_tris);
    out
}

impl EyeCheck {
    /// Whether the piece passes: it never enters the eyeball, its visible
    /// edge stays within 0.5 mm of the eyeball, no more than 0.5 mm² of
    /// eyeball shows when closed, and it lies no more than 1.5 mm in front
    /// of the skin where it overlaps the lower lid when closed.
    pub fn passes(&self) -> bool {
        self.into_eyeball >= 0.0
            && self.edge_gap <= 0.5
            && self.shown_closed_front <= 0.5
            && self.shown_closed_three <= 0.5
            && self.poke_depth <= 1.5
    }
}

/// Builds and checks both eyes' pieces on a head: the call to make after a
/// slider changes the head (positions in metres).
pub fn build_and_check(rig: &crate::piece::LidRig, head: &[V3], s: &crate::piece::Settings) -> (Vec<EyePiece>, Vec<EyeCheck>) {
    let pieces = crate::piece::build(rig, head, s);
    let checks = pieces.iter().zip(&rig.eyes).map(|(p, e)| check_eye(e, &rig.triangles, head, p)).collect();
    (pieces, checks)
}
