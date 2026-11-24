//! Cards: strands clustered by k-means, and one strip of quads laid along each cluster's mean strand.
//!
//! Every strand is resampled to `SAMPLES` points spread evenly along its own length. The distance between two strands is the sum of the
//! squared distances between matching points, so it compares both the roots and the directions (DiffHairCard's distance). A card's half
//! width at a point is the largest distance of the cluster's strands from the mean strand, sideways, clamped to the lock's width range.

use crate::field::resample_polyline;
use crate::v3::{Rng, V3, add, cross, distance, dot, length, normalize, scale, sub};

pub const SAMPLES: usize = 8;

/// A triangle mesh with a texture point and a normal per vertex.
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub positions: Vec<V3>,
    pub normals: Vec<V3>,
    pub uv: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
}

impl Mesh {
    pub fn append(&mut self, other: &Mesh) {
        let base = self.positions.len() as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.uv.extend_from_slice(&other.uv);
        self.triangles.extend(other.triangles.iter().map(|t| t.map(|i| i + base)));
    }
}

/// A polyline resampled to `n` points spread evenly along its length.
pub fn even_points(points: &[V3], n: usize) -> Vec<V3> {
    let total: f64 = points.windows(2).map(|w| distance(w[0], w[1])).sum();
    if points.len() < 2 || total <= 0.0 {
        return vec![points.first().copied().unwrap_or([0.0; 3]); n];
    }
    let mut out = resample_polyline(points, total / (n - 1) as f64 * 0.999_999);
    out.truncate(n);
    while out.len() < n {
        out.push(points[points.len() - 1]);
    }
    let last = out.len() - 1;
    out[last] = points[points.len() - 1];
    out
}

fn strand_distance(a: &[V3], b: &[V3]) -> f64 {
    a.iter().zip(b).map(|(p, q)| dot(sub(*p, *q), sub(*p, *q))).sum()
}

/// Clusters equal-length samplings of strands into `k` groups with k-means (k-means++ start, 30 rounds). Returns each strand's group.
pub fn kmeans(strands: &[Vec<V3>], k: usize, rng: &mut Rng) -> Vec<usize> {
    let n = strands.len();
    if n == 0 || k == 0 {
        return vec![0; n];
    }
    let k = k.min(n);
    let mut centres: Vec<Vec<V3>> = vec![strands[(rng.unit() * n as f64) as usize % n].clone()];
    while centres.len() < k {
        let d: Vec<f64> = strands.iter().map(|s| centres.iter().map(|c| strand_distance(s, c)).fold(f64::INFINITY, f64::min)).collect();
        let total: f64 = d.iter().sum();
        let mut pick = rng.unit() * total;
        let mut chosen = n - 1;
        for (i, w) in d.iter().enumerate() {
            pick -= w;
            if pick <= 0.0 {
                chosen = i;
                break;
            }
        }
        centres.push(strands[chosen].clone());
    }
    let mut groups = vec![0; n];
    for _ in 0..30 {
        let mut changed = false;
        for (i, s) in strands.iter().enumerate() {
            let best = (0..k).min_by(|&a, &b| strand_distance(s, &centres[a]).total_cmp(&strand_distance(s, &centres[b]))).unwrap_or(0);
            if groups[i] != best {
                groups[i] = best;
                changed = true;
            }
        }
        for (c, centre) in centres.iter_mut().enumerate() {
            let members: Vec<&Vec<V3>> = strands.iter().zip(&groups).filter(|(_, g)| **g == c).map(|(s, _)| s).collect();
            if members.is_empty() {
                continue;
            }
            *centre = mean_strand(&members);
        }
        if !changed {
            break;
        }
    }
    groups
}

/// The mean of equal-length samplings of strands.
pub fn mean_strand(members: &[&Vec<V3>]) -> Vec<V3> {
    let m = members[0].len();
    (0..m).map(|i| scale(members.iter().fold([0.0; 3], |acc, s| add(acc, s[i])), 1.0 / members.len() as f64)).collect()
}

/// Where on the atlas a card's texture lies: `u` across the strip, and `v` from the root (`v_root`) to the tip (`v_tip`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StripRect {
    pub u: [f64; 2],
    pub v_root: f64,
    pub v_tip: f64,
}

/// A strip of `segments` quads along `centre` (any number of points), with the given half width at the root and at the tip. The strip
/// lies across `out(p)`, the direction away from the head, so it faces outward. Its normals point outward too. With `lean_deg`, the
/// strip turns about its centre line so that its upper edge (the one higher in y) lifts off the head by that angle.
pub fn card_strip(centre: &[V3], segments: usize, half_root: f64, half_tip: f64, out: impl Fn(V3) -> V3, rect: StripRect, lean_deg: f64) -> Mesh {
    let pts = even_points(centre, segments + 1);
    let n = pts.len();
    let mut mesh = Mesh::default();
    let mut prev_b: Option<V3> = None;
    for (i, &p) in pts.iter().enumerate() {
        let t = normalize(sub(pts[(i + 1).min(n - 1)], pts[i.saturating_sub(1)]));
        let o = out(p);
        let mut b = normalize(cross(t, o));
        if length(b) == 0.0 {
            b = prev_b.unwrap_or([1.0, 0.0, 0.0]);
        }
        if let Some(pb) = prev_b
            && dot(pb, b) < 0.0
        {
            b = scale(b, -1.0);
        }
        prev_b = Some(b);
        if lean_deg != 0.0 {
            let mut up = normalize(cross(b, t));
            if dot(up, o) < 0.0 {
                up = scale(up, -1.0);
            }
            let sign = if b[1] >= 0.0 { 1.0 } else { -1.0 };
            let (c, sn) = (lean_deg.to_radians().cos(), lean_deg.to_radians().sin());
            b = normalize(add(scale(b, c), scale(up, sign * sn)));
        }
        let s = i as f64 / (n - 1) as f64;
        let half = half_root + (half_tip - half_root) * s;
        let mut normal = normalize(cross(b, t));
        if dot(normal, o) < 0.0 {
            normal = scale(normal, -1.0);
        }
        let v = rect.v_root + (rect.v_tip - rect.v_root) * s;
        mesh.positions.push(add(p, scale(b, -half)));
        mesh.positions.push(add(p, scale(b, half)));
        mesh.normals.push(normal);
        mesh.normals.push(normal);
        mesh.uv.push([rect.u[0], v]);
        mesh.uv.push([rect.u[1], v]);
    }
    for i in 0..(n - 1) as u32 {
        let (l0, r0, l1, r1) = (2 * i, 2 * i + 1, 2 * i + 2, 2 * i + 3);
        mesh.triangles.push([l0, r0, l1]);
        mesh.triangles.push([r0, r1, l1]);
    }
    mesh
}

/// The half width a cluster needs at its root and its tip: the largest sideways distance of its strands from the mean, clamped.
pub fn cluster_half_widths(members: &[&Vec<V3>], mean: &[V3], out: impl Fn(V3) -> V3, width_mm: [f64; 2]) -> (f64, f64) {
    let m = mean.len();
    let mut widest = vec![0.0f64; m];
    for i in 0..m {
        let t = normalize(sub(mean[(i + 1).min(m - 1)], mean[i.saturating_sub(1)]));
        let b = normalize(cross(t, out(mean[i])));
        for s in members {
            widest[i] = widest[i].max(dot(sub(s[i], mean[i]), b).abs());
        }
    }
    let (lo, hi) = (width_mm[0] / 2000.0, width_mm[1] / 2000.0);
    let root = widest[..m / 2].iter().cloned().fold(0.0, f64::max).clamp(lo, hi);
    let tip = (widest[m / 2..].iter().cloned().fold(0.0, f64::max) * 0.6).clamp(lo, hi);
    (root, tip.min(root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v3::cross;

    fn triangle_normal(m: &Mesh, t: [u32; 3]) -> V3 {
        let [a, b, c] = t.map(|i| m.positions[i as usize]);
        cross(sub(b, a), sub(c, a))
    }

    #[test]
    fn the_card_builder_makes_closed_non_degenerate_strips() {
        // A curved strand over a sphere of radius 0.1, from the front over the top.
        let centre: Vec<V3> = (0..=20).map(|i| {
            let a = i as f64 * 0.06;
            [0.0, 0.1 * a.sin(), 0.1 * a.cos()]
        }).collect();
        let rect = StripRect { u: [0.75, 0.8125], v_root: 1.0, v_tip: 0.0 };
        for segments in [1, 3, 6] {
            let m = card_strip(&centre, segments, 0.006, 0.003, normalize, rect, 0.0);
            assert_eq!(m.positions.len(), 2 * (segments + 1));
            assert_eq!(m.triangles.len(), 2 * segments);
            for &t in &m.triangles {
                assert!(length(triangle_normal(&m, t)) > 1e-9, "a triangle has no area");
                assert!(t[0] != t[1] && t[1] != t[2] && t[0] != t[2]);
            }
            // Each quad shares its far edge with the next quad: no gaps along the strip.
            for q in 0..segments.saturating_sub(1) {
                let a = m.triangles[2 * q + 1];
                let b = m.triangles[2 * q + 2];
                let shared = a.iter().filter(|i| b.contains(i)).count();
                assert_eq!(shared, 2, "quads {q} and {} are not joined", q + 1);
            }
            // Every triangle faces the same side, so the strip never folds over.
            let first = normalize(triangle_normal(&m, m.triangles[0]));
            for &t in &m.triangles {
                assert!(dot(normalize(triangle_normal(&m, t)), first) > 0.0);
            }
            // The strip faces away from the sphere's centre, and a leaning strip still does.
            let leaning = card_strip(&centre, segments, 0.006, 0.003, normalize, rect, 30.0);
            for t in &leaning.triangles {
                assert!(length(triangle_normal(&leaning, *t)) > 1e-9);
            }
            for (p, n) in m.positions.iter().zip(&m.normals) {
                assert!(dot(*n, normalize(*p)) > 0.5);
            }
        }
    }

    #[test]
    fn kmeans_separates_two_bundles() {
        let mut rng = Rng::new(3);
        let mut strands = Vec::new();
        for i in 0..20 {
            let x = if i < 10 { 0.0 } else { 1.0 };
            strands.push((0..SAMPLES).map(|k| [x + 0.01 * (i % 3) as f64, k as f64 * 0.1, 0.0]).collect::<Vec<V3>>());
        }
        let g = kmeans(&strands, 2, &mut rng);
        assert!(g[..10].iter().all(|&x| x == g[0]) && g[10..].iter().all(|&x| x == g[10]) && g[0] != g[10]);
    }

    #[test]
    fn even_points_spreads_points_along_the_length() {
        let p = even_points(&[[0.0; 3], [1.0, 0.0, 0.0], [1.0, 2.0, 0.0]], 4);
        assert_eq!(p.len(), 4);
        assert_eq!(p[3], [1.0, 2.0, 0.0]);
        assert!((distance(p[0], p[1]) - 1.0).abs() < 1e-5);
    }
}
