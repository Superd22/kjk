//! The exact closest point on a triangle mesh, found with a bounding volume hierarchy.
//!
//! The bake of a texture asks for the closest point of a KOTOR head for every pixel of our head, and the fit's score asks for the distance from
//! every vertex to the target. Both need the true closest point on the triangles, not the closest of some sampled points.

use std::thread;

type Vec3 = [f64; 3];

const LEAF_SIZE: usize = 4;

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn at(origin: Vec3, edge: Vec3, along: f64) -> Vec3 {
    [origin[0] + edge[0] * along, origin[1] + edge[1] * along, origin[2] + edge[2] * along]
}

/// The closest point of a mesh to a query point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Closest {
    /// The closest point, in the mesh's space.
    pub point: [f64; 3],
    /// The distance from the query point to `point`.
    pub distance: f64,
    /// The index of the triangle that holds `point`. When the point lies on an edge or a corner that several triangles share, it is one of them.
    pub triangle: u32,
    /// The weights of the triangle's three corners that give `point`. They are never negative and add up to 1.
    pub barycentric: [f64; 3],
}

/// The closest point of one triangle to a point, as the weights of the triangle's corners.
pub fn closest_on_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> [f64; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return [0.0, 1.0, 0.0];
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [1.0 - v, v, 0.0];
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return [0.0, 0.0, 1.0];
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [1.0 - w, 0.0, w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }
    let sum = va + vb + vc;
    if sum == 0.0 {
        // A triangle with no area: the closest point of its longest edge's corner is close enough.
        return [1.0, 0.0, 0.0];
    }
    let v = vb / sum;
    let w = vc / sum;
    [1.0 - v - w, v, w]
}

#[derive(Clone, Copy)]
struct Node {
    min: Vec3,
    max: Vec3,
    /// For an inner node, the index of its right child (the left child follows it). For a leaf, the first entry of `order`.
    first: u32,
    /// Zero for an inner node, otherwise the number of triangles in the leaf.
    count: u32,
}

/// A bounding volume hierarchy over the triangles of a mesh.
pub struct Bvh {
    vertices: Vec<Vec3>,
    triangles: Vec<[u32; 3]>,
    order: Vec<u32>,
    nodes: Vec<Node>,
}

impl Bvh {
    /// Builds the hierarchy. Every index in `triangles` must be below `vertices.len()`.
    pub fn new(vertices: Vec<[f64; 3]>, triangles: Vec<[u32; 3]>) -> Self {
        assert!(
            triangles.iter().flatten().all(|&i| (i as usize) < vertices.len()),
            "a triangle uses a vertex that does not exist"
        );
        let centres: Vec<Vec3> = triangles
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| vertices[i as usize]);
                [0, 1, 2].map(|k| (a[k] + b[k] + c[k]) / 3.0)
            })
            .collect();
        let mut order: Vec<u32> = (0..triangles.len() as u32).collect();
        let mut bvh = Bvh { vertices, triangles, order: Vec::new(), nodes: Vec::new() };
        if !order.is_empty() {
            let end = order.len();
            bvh.build_range(&mut order, 0, end, &centres);
        }
        bvh.order = order;
        bvh
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Adds the node for `order[start..end]` and its children, and returns the node's index.
    fn build_range(&mut self, order: &mut [u32], start: usize, end: usize, centres: &[Vec3]) -> usize {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut cmin = min;
        let mut cmax = max;
        for &t in &order[start..end] {
            for &i in &self.triangles[t as usize] {
                for k in 0..3 {
                    min[k] = min[k].min(self.vertices[i as usize][k]);
                    max[k] = max[k].max(self.vertices[i as usize][k]);
                }
            }
            for k in 0..3 {
                cmin[k] = cmin[k].min(centres[t as usize][k]);
                cmax[k] = cmax[k].max(centres[t as usize][k]);
            }
        }
        let index = self.nodes.len();
        self.nodes.push(Node { min, max, first: start as u32, count: (end - start) as u32 });
        if end - start <= LEAF_SIZE {
            return index;
        }
        let axis = (0..3).max_by(|&a, &b| (cmax[a] - cmin[a]).total_cmp(&(cmax[b] - cmin[b]))).unwrap_or(0);
        let mid = start + (end - start) / 2;
        order[start..end].select_nth_unstable_by(mid - start, |&a, &b| centres[a as usize][axis].total_cmp(&centres[b as usize][axis]));
        self.build_range(order, start, mid, centres);
        let right = self.build_range(order, mid, end, centres);
        self.nodes[index].first = right as u32;
        self.nodes[index].count = 0;
        index
    }

    /// The closest point of the mesh to `p`, or `None` when the mesh has no triangle.
    pub fn closest(&self, p: [f64; 3]) -> Option<Closest> {
        self.closest_with(p, |_| true)
    }

    /// The closest point among the triangles that `accept` takes, or `None` when it takes none. The bake uses it to skip triangles that face
    /// the other way, such as the other lip across a closed mouth.
    pub fn closest_with(&self, p: [f64; 3], mut accept: impl FnMut(u32) -> bool) -> Option<Closest> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best: Option<(f64, u32, [f64; 3])> = None;
        let mut best_sq = f64::INFINITY;
        let mut stack = [0u32; 64];
        let mut top = 1;
        while top > 0 {
            top -= 1;
            let node = &self.nodes[stack[top] as usize];
            if box_distance_sq(p, node.min, node.max) >= best_sq {
                continue;
            }
            if node.count > 0 {
                for &t in &self.order[node.first as usize..(node.first + node.count) as usize] {
                    if !accept(t) {
                        continue;
                    }
                    let [a, b, c] = self.triangles[t as usize].map(|i| self.vertices[i as usize]);
                    let w = closest_on_triangle(p, a, b, c);
                    let q = point_of(w, a, b, c);
                    let d = sub(p, q);
                    let sq = dot(d, d);
                    if sq < best_sq {
                        best_sq = sq;
                        best = Some((sq, t, w));
                    }
                }
            } else {
                let left = stack[top] + 1;
                let right = node.first;
                let near_left = box_distance_sq(p, self.nodes[left as usize].min, self.nodes[left as usize].max)
                    <= box_distance_sq(p, self.nodes[right as usize].min, self.nodes[right as usize].max);
                let (far, near) = if near_left { (right, left) } else { (left, right) };
                stack[top] = far;
                stack[top + 1] = near;
                top += 2;
            }
        }
        best.map(|(sq, triangle, barycentric)| {
            let [a, b, c] = self.triangles[triangle as usize].map(|i| self.vertices[i as usize]);
            Closest { point: point_of(barycentric, a, b, c), distance: sq.sqrt(), triangle, barycentric }
        })
    }

    /// The closest point for each query point, spread over the machine's cores. A mesh with no triangle gives `None` for every point.
    pub fn closest_all(&self, points: &[[f64; 3]]) -> Vec<Option<Closest>> {
        self.closest_all_with(points, &|_, _| true)
    }

    /// Like `closest_all`, but for the point with index `i` only the triangles `t` with `accept(i, t)` count.
    pub fn closest_all_with(&self, points: &[[f64; 3]], accept: &(dyn Fn(usize, u32) -> bool + Sync)) -> Vec<Option<Closest>> {
        let workers = thread::available_parallelism().map_or(1, |n| n.get()).min(points.len() / 256 + 1);
        let mut out = vec![None; points.len()];
        if workers <= 1 {
            for (i, (o, &p)) in out.iter_mut().zip(points).enumerate() {
                *o = self.closest_with(p, |t| accept(i, t));
            }
            return out;
        }
        let chunk = points.len().div_ceil(workers);
        thread::scope(|s| {
            for (k, (o, p)) in out.chunks_mut(chunk).zip(points.chunks(chunk)).enumerate() {
                s.spawn(move || {
                    for (i, (o, &p)) in o.iter_mut().zip(p).enumerate() {
                        let index = k * chunk + i;
                        *o = self.closest_with(p, |t| accept(index, t));
                    }
                });
            }
        });
        out
    }
}

fn point_of(w: [f64; 3], a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    at(at(a, sub(b, a), w[1]), sub(c, a), w[2])
}

fn box_distance_sq(p: Vec3, min: Vec3, max: Vec3) -> f64 {
    (0..3)
        .map(|k| {
            let d = (min[k] - p[k]).max(p[k] - max[k]).max(0.0);
            d * d
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> Bvh {
        Bvh::new(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], vec![[0, 1, 2]])
    }

    fn near(a: [f64; 3], b: [f64; 3]) {
        assert!((0..3).all(|k| (a[k] - b[k]).abs() < 1e-12), "{a:?} is not {b:?}");
    }

    #[test]
    fn a_point_above_a_face() {
        let c = unit_triangle().closest([0.25, 0.25, 2.0]).unwrap();
        near(c.point, [0.25, 0.25, 0.0]);
        assert!((c.distance - 2.0).abs() < 1e-12);
        assert_eq!(c.triangle, 0);
        near(c.barycentric, [0.5, 0.25, 0.25]);
    }

    #[test]
    fn a_point_beside_an_edge() {
        let c = unit_triangle().closest([0.5, -1.0, 0.0]).unwrap();
        near(c.point, [0.5, 0.0, 0.0]);
        assert!((c.distance - 1.0).abs() < 1e-12);
        near(c.barycentric, [0.5, 0.5, 0.0]);
    }

    #[test]
    fn a_point_beside_the_long_edge() {
        let c = unit_triangle().closest([1.0, 1.0, 0.0]).unwrap();
        near(c.point, [0.5, 0.5, 0.0]);
        near(c.barycentric, [0.0, 0.5, 0.5]);
    }

    #[test]
    fn a_point_near_a_corner() {
        let c = unit_triangle().closest([-1.0, -1.0, 1.0]).unwrap();
        near(c.point, [0.0, 0.0, 0.0]);
        assert!((c.distance - 3f64.sqrt()).abs() < 1e-12);
        near(c.barycentric, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn a_point_on_the_surface_has_distance_zero() {
        let c = unit_triangle().closest([0.1, 0.2, 0.0]).unwrap();
        assert!(c.distance < 1e-15);
    }

    #[test]
    fn a_triangle_that_is_not_accepted_is_skipped() {
        let bvh = Bvh::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        assert_eq!(bvh.closest([0.2, 0.2, 0.1]).unwrap().triangle, 0);
        let c = bvh.closest_with([0.2, 0.2, 0.1], |t| t == 1).unwrap();
        assert_eq!(c.triangle, 1);
        assert!((c.distance - 0.9).abs() < 1e-12);
        assert!(bvh.closest_with([0.2, 0.2, 0.1], |_| false).is_none());
    }

    #[test]
    fn a_mesh_with_no_triangle_has_no_closest_point() {
        assert!(Bvh::new(vec![], vec![]).closest([0.0; 3]).is_none());
    }

    #[test]
    fn a_triangle_with_no_area_does_not_give_nan() {
        let bvh = Bvh::new(vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]], vec![[0, 1, 2]]);
        let c = bvh.closest([1.0, 1.0, 0.0]).unwrap();
        assert!(c.distance.is_finite());
    }

    struct Random(u64);

    impl Random {
        fn next(&mut self) -> f64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    #[test]
    fn it_agrees_with_brute_force_on_a_random_mesh() {
        let mut rng = Random(7);
        let vertices: Vec<Vec3> = (0..600).map(|_| [rng.next(), rng.next(), rng.next()]).collect();
        let triangles: Vec<[u32; 3]> = (0..1500)
            .map(|_| {
                let a = (rng.next() * 600.0) as u32;
                let near = |r: f64| ((a as f64 + r * 20.0) as u32).min(599);
                [a, near(rng.next()), near(rng.next())]
            })
            .collect();
        let bvh = Bvh::new(vertices.clone(), triangles.clone());
        let points: Vec<Vec3> = (0..3000).map(|_| [rng.next() * 1.4 - 0.2, rng.next() * 1.4 - 0.2, rng.next() * 1.4 - 0.2]).collect();
        let found = bvh.closest_all(&points);
        for (p, f) in points.iter().zip(found) {
            let f = f.unwrap();
            let brute = triangles
                .iter()
                .map(|t| {
                    let [a, b, c] = t.map(|i| vertices[i as usize]);
                    let q = point_of(closest_on_triangle(*p, a, b, c), a, b, c);
                    let d = sub(*p, q);
                    dot(d, d).sqrt()
                })
                .fold(f64::INFINITY, f64::min);
            assert!((f.distance - brute).abs() < 1e-12, "{} against {brute}", f.distance);
            let [a, b, c] = triangles[f.triangle as usize].map(|i| vertices[i as usize]);
            near(f.point, point_of(f.barycentric, a, b, c));
            assert!((f.barycentric.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(f.barycentric.iter().all(|&w| w >= -1e-12));
        }
    }
}
