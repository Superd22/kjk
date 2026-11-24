//! The neck's open edge, where our head joins KOTOR's bodies.
//!
//! Our mesh carries a neck and shoulders, but only the part above one closed
//! loop of its edges is drawn and exported. That loop is the rim. KOTOR's
//! player heads all end on the same ring of vertices, and the bodies' torsos
//! share those vertices, so the rim is placed exactly on that ring. Every
//! vertex below the rim follows the rim, and the vertices above it follow it
//! less and less up to `reach` metres away.
//!
//! The rim's vertices are spread along the target ring by their share of the
//! rim's length, starting where the whole rim lies closest to the ring. Our
//! rim has more vertices than KOTOR's ring, so they sit on the ring's
//! straight segments, and the join to the body has no gap.

/// Where each rim vertex goes on the target ring. The start and direction along the ring are the ones that put the rim closest to it when
/// the rim's vertices keep their share of the loop's length. Then each corner of the ring takes one rim vertex.
pub fn rim_targets(rim: &[[f64; 3]], ring: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let share = shares(rim);
    let ring_len = loop_length(ring);
    let mut best = (f64::INFINITY, 0.0, 1.0);
    const STARTS: usize = 1440;
    for k in 0..STARTS {
        let start = ring_len * k as f64 / STARTS as f64;
        for dir in [1.0, -1.0] {
            let cost: f64 = rim.iter().zip(&share).map(|(p, s)| dist2(*p, at_length(ring, start + dir * s * ring_len))).sum();
            if cost < best.0 {
                best = (cost, start, dir);
            }
        }
    }
    let (_, start, dir) = best;
    // Each corner of the ring takes the rim vertex nearest to it along the loop, so the body's vertices all lie on the rim. The rim
    // vertices between two such vertices are spread along the ring's segment between the two corners by their share of the rim between them.
    let along: Vec<f64> = share.iter().map(|s| (dir * s * ring_len).rem_euclid(ring_len)).collect();
    let corners = corner_lengths(ring);
    let mut pinned: Vec<(usize, f64)> = Vec::new();
    for c in &corners {
        let rel = (c - start).rem_euclid(ring_len);
        let i = (0..rim.len())
            .min_by(|&a, &b| loop_gap(along[a], rel, ring_len).total_cmp(&loop_gap(along[b], rel, ring_len)))
            .unwrap_or(0);
        if !pinned.iter().any(|&(j, _)| j == i) {
            pinned.push((i, rel));
        }
    }
    pinned.sort_by_key(|&(i, _)| i);
    if pinned.len() < 2 {
        return along.iter().map(|a| at_length(ring, start + a)).collect();
    }
    let mut out = vec![[0.0; 3]; rim.len()];
    for k in 0..pinned.len() {
        let (i0, l0) = pinned[k];
        let (i1, l1) = pinned[(k + 1) % pinned.len()];
        let count = (i1 + rim.len() - i0) % rim.len();
        let count = if count == 0 { rim.len() } else { count };
        let s0 = share[i0];
        let span_rim = (share[i1] - s0).rem_euclid(1.0);
        let span_rim = if span_rim == 0.0 { 1.0 } else { span_rim };
        let span_ring = if dir > 0.0 { (l1 - l0).rem_euclid(ring_len) } else { -(l0 - l1).rem_euclid(ring_len) };
        for step in 0..count {
            let i = (i0 + step) % rim.len();
            let f = (share[i] - s0).rem_euclid(1.0) / span_rim;
            out[i] = at_length(ring, start + l0 + f * span_ring);
        }
    }
    out
}

fn loop_gap(a: f64, b: f64, len: f64) -> f64 {
    let d = (a - b).rem_euclid(len);
    d.min(len - d)
}

/// The distance along the closed polyline from its first point to each of its points.
fn corner_lengths(p: &[[f64; 3]]) -> Vec<f64> {
    let mut s = 0.0;
    let mut out = Vec::with_capacity(p.len());
    for i in 0..p.len() {
        out.push(s);
        s += dist2(p[i], p[(i + 1) % p.len()]).sqrt();
    }
    out
}

/// Moves the rim onto the target ring and carries the move to the rest of the head.
///
/// `base` is the base head (for distances, so a slider never changes which vertices follow), `positions` the head being built, `rim` the
/// rim's vertices in loop order, `below` marks every vertex under the rim, and `ring` is the target ring as a closed polyline. A vertex
/// at the place of a rim vertex moves exactly as it does, which covers the copies that the texture layout splits. A vertex below the rim
/// moves as the rim vertices near it do. A vertex above moves by that amount times a weight that falls smoothly from 1 at the rim to 0 at
/// `reach` metres from it.
pub fn place_rim(base: &[[f64; 3]], positions: &[[f64; 3]], rim: &[u32], below: &[bool], ring: &[[f64; 3]], reach: f64) -> Vec<[f64; 3]> {
    let now: Vec<[f64; 3]> = rim.iter().map(|&i| positions[i as usize]).collect();
    let goal = rim_targets(&now, ring);
    let moves: Vec<[f64; 3]> = goal.iter().zip(&now).map(|(g, p)| std::array::from_fn(|c| g[c] - p[c])).collect();
    let anchors: Vec<[f64; 3]> = rim.iter().map(|&i| base[i as usize]).collect();
    // The rim's vertices are 5 to 15 mm apart, so a vertex takes a blend of the nearest ones' moves over about that width.
    let width = 0.006;
    base.iter()
        .zip(positions)
        .zip(below)
        .map(|((b, p), &under)| {
            let d2: Vec<f64> = anchors.iter().map(|a| dist2(*a, *b)).collect();
            let nearest = d2.iter().copied().fold(f64::INFINITY, f64::min);
            let fade = if under {
                1.0
            } else {
                let t = (1.0 - nearest.sqrt() / reach).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            };
            if fade == 0.0 {
                return *p;
            }
            if nearest < 1e-14 {
                let k = d2.iter().position(|&d| d == nearest).unwrap_or(0);
                return std::array::from_fn(|c| p[c] + moves[k][c]);
            }
            let w: Vec<f64> = d2.iter().map(|d| (-(d - nearest) / (width * width)).exp()).collect();
            let total: f64 = w.iter().sum();
            std::array::from_fn(|c| p[c] + fade * w.iter().zip(&moves).map(|(w, m)| w * m[c]).sum::<f64>() / total)
        })
        .collect()
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|c| (a[c] - b[c]).powi(2)).sum()
}

fn loop_length(p: &[[f64; 3]]) -> f64 {
    (0..p.len()).map(|i| dist2(p[i], p[(i + 1) % p.len()]).sqrt()).sum()
}

/// Each point's distance along the closed loop from its first point, as a share of the loop's length.
fn shares(p: &[[f64; 3]]) -> Vec<f64> {
    let total = loop_length(p);
    let mut s = 0.0;
    let mut out = Vec::with_capacity(p.len());
    for i in 0..p.len() {
        out.push(s / total);
        s += dist2(p[i], p[(i + 1) % p.len()]).sqrt();
    }
    out
}

/// The point at `length` along the closed polyline, wrapping round.
fn at_length(p: &[[f64; 3]], length: f64) -> [f64; 3] {
    let total = loop_length(p);
    let mut l = length.rem_euclid(total);
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        let seg = dist2(a, b).sqrt();
        if l <= seg || i + 1 == p.len() {
            let t = if seg > 0.0 { (l / seg).min(1.0) } else { 0.0 };
            return std::array::from_fn(|c| a[c] + t * (b[c] - a[c]));
        }
        l -= seg;
    }
    p[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(n: usize, r: f64, y: f64, turn: f64) -> Vec<[f64; 3]> {
        (0..n)
            .map(|i| {
                let a = turn + std::f64::consts::TAU * i as f64 / n as f64;
                [r * a.cos(), y, r * a.sin()]
            })
            .collect()
    }

    fn on_polyline(p: [f64; 3], ring: &[[f64; 3]]) -> f64 {
        (0..ring.len())
            .map(|i| {
                let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
                let ab: [f64; 3] = std::array::from_fn(|c| b[c] - a[c]);
                let t = ((0..3).map(|c| (p[c] - a[c]) * ab[c]).sum::<f64>() / dist2(a, b)).clamp(0.0, 1.0);
                dist2(p, std::array::from_fn(|c| a[c] + t * ab[c])).sqrt()
            })
            .fold(f64::INFINITY, f64::min)
    }

    #[test]
    fn the_rim_lands_on_the_ring() {
        let rim = circle(40, 0.05, 0.6, 0.1);
        let ring = circle(8, 0.055, 0.59, 0.0);
        for p in rim_targets(&rim, &ring) {
            assert!(on_polyline(p, &ring) < 1e-9);
        }
    }

    #[test]
    fn the_targets_keep_the_rims_order_and_direction() {
        let rim = circle(40, 0.05, 0.6, 0.0);
        let ring = circle(10, 0.05, 0.6, 0.02);
        let t = rim_targets(&rim, &ring);
        for (p, q) in rim.iter().zip(&t) {
            assert!(dist2(*p, *q).sqrt() < 0.004, "{p:?} went to {q:?}");
        }
    }

    #[test]
    fn every_corner_of_the_ring_is_a_rim_vertex() {
        let rim = circle(42, 0.05, 0.6, 0.03);
        for n in [8, 10] {
            let ring = circle(n, 0.056, 0.58, 0.0);
            let t = rim_targets(&rim, &ring);
            for c in &ring {
                assert!(t.iter().any(|p| dist2(*p, *c) < 1e-18), "corner {c:?} is not on the rim");
            }
            for i in 0..t.len() {
                assert!(dist2(t[i], t[(i + 1) % t.len()]).sqrt() < 0.02, "rim vertices {i} and the next are far apart");
            }
        }
    }

    #[test]
    fn far_vertices_stay_and_copies_move_together() {
        let rim = circle(24, 0.05, 0.6, 0.0);
        let ring = circle(10, 0.06, 0.58, 0.0);
        let mut base = rim.clone();
        base.push(rim[3]);
        base.push([0.0, 0.7, 0.05]);
        base.push([0.05, 0.62, 0.0]);
        base.push([0.0, 0.55, 0.05]);
        let rim_ids: Vec<u32> = (0..24).collect();
        let mut below = vec![false; base.len()];
        below[27] = true;
        let out = place_rim(&base, &base, &rim_ids, &below, &ring, 0.03);
        for &i in &rim_ids {
            assert!(on_polyline(out[i as usize], &ring) < 1e-9);
        }
        assert!(dist2(out[24], out[3]) < 1e-20);
        assert!(dist2(out[25], base[25]) < 1e-20);
        let near = dist2(out[26], base[26]).sqrt();
        assert!(near > 0.0 && near < dist2(out[0], base[0]).sqrt());
        assert!(dist2(out[27], base[27]).sqrt() > 0.015);
    }
}
