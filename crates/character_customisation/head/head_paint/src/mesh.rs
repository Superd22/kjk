//! Mesh code: the neighbours of each point, and a smoothing that only counts the points that hold a valid value.

/// For each point of a triangle mesh, the points that share an edge with it, each listed once.
pub struct Neighbours {
    lists: Vec<Vec<u32>>,
}

impl Neighbours {
    pub fn new(triangles: &[[u32; 3]], point_count: usize) -> Self {
        let mut lists = vec![Vec::new(); point_count];
        for t in triangles {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                lists[a as usize].push(b);
                lists[b as usize].push(a);
            }
        }
        for list in &mut lists {
            list.sort_unstable();
            list.dedup();
        }
        Neighbours { lists }
    }

    pub fn len(&self) -> usize {
        self.lists.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lists.is_empty()
    }

    pub fn of(&self, point: usize) -> &[u32] {
        &self.lists[point]
    }

    /// Replaces each value by the mean of its neighbours' values. A point with no neighbour gets zero.
    fn average<const N: usize>(&self, values: &[[f64; N]]) -> Vec<[f64; N]> {
        self.lists
            .iter()
            .map(|list| {
                let mut sum = [0.0; N];
                for &j in list {
                    for (s, v) in sum.iter_mut().zip(values[j as usize]) {
                        *s += v;
                    }
                }
                let count = list.len().max(1) as f64;
                sum.map(|s| s / count)
            })
            .collect()
    }
}

/// Smooths per-point values over the surface. Each round replaces a point's value by half of its neighbours' mean and half of its own
/// starting value, counting only the points where `valid` is true. The result at a point is the weighted mean of the valid values that
/// reached it, so a point that is not valid still gets a value from the valid points around it.
pub fn smooth_masked<const N: usize>(values: &[[f64; N]], valid: &[bool], neighbours: &Neighbours, rounds: usize) -> Vec<[f64; N]> {
    let start: Vec<[f64; N]> = values.iter().zip(valid).map(|(v, &ok)| if ok { *v } else { [0.0; N] }).collect();
    let start_weight: Vec<f64> = valid.iter().map(|&ok| f64::from(ok)).collect();
    let mut v = start.clone();
    let mut w = start_weight.clone();
    for _ in 0..rounds {
        let spread_v = neighbours.average(&v);
        let spread_w = neighbours.average(&w.iter().map(|&x| [x]).collect::<Vec<_>>());
        for i in 0..v.len() {
            for c in 0..N {
                v[i][c] = 0.5 * spread_v[i][c] + 0.5 * start[i][c];
            }
            w[i] = 0.5 * spread_w[i][0] + 0.5 * start_weight[i];
        }
    }
    v.iter().zip(&w).map(|(v, &w)| v.map(|c| c / w.max(1e-9))).collect()
}

/// The unit normal of each point of a triangle mesh: the sum of the normals of its triangles, each weighted by the triangle's area. A point
/// that no triangle uses, or whose triangles cancel out, gets a zero normal.
pub fn vertex_normals(positions: &[[f64; 3]], triangles: &[[u32; 3]]) -> Vec<[f64; 3]> {
    let mut sum = vec![[0.0; 3]; positions.len()];
    for t in triangles {
        let [a, b, c] = t.map(|i| positions[i as usize]);
        let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        for &i in t {
            for k in 0..3 {
                sum[i as usize][k] += n[k];
            }
        }
    }
    sum.into_iter()
        .map(|n| {
            let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if length > 0.0 { n.map(|x| x / length) } else { [0.0; 3] }
        })
        .collect()
}

/// The area each point of a triangle mesh stands for: a third of the area of every triangle that uses it.
pub fn vertex_areas(positions: &[[f64; 3]], triangles: &[[u32; 3]]) -> Vec<f64> {
    let mut area = vec![0.0; positions.len()];
    for t in triangles {
        let [a, b, c] = t.map(|i| positions[i as usize]);
        let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        let third = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() / 6.0;
        for &i in t {
            area[i as usize] += third;
        }
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_point_areas_add_up_to_the_mesh_area() {
        let areas = vertex_areas(&[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [2.0, 3.0, 0.0]], &[[0, 1, 2], [1, 3, 2]]);
        assert!((areas.iter().sum::<f64>() - 6.0).abs() < 1e-12);
        assert!((areas[0] - 1.0).abs() < 1e-12 && (areas[1] - 2.0).abs() < 1e-12);
    }

    fn strip() -> Neighbours {
        // Four points in a row, made of two triangles: 0 1 2 and 1 2 3.
        Neighbours::new(&[[0, 1, 2], [1, 2, 3]], 4)
    }

    #[test]
    fn neighbours_are_listed_once_each() {
        let n = strip();
        assert_eq!(n.of(0), &[1, 2]);
        assert_eq!(n.of(1), &[0, 2, 3]);
        assert_eq!(n.of(3), &[1, 2]);
    }

    #[test]
    fn the_normals_of_a_flat_mesh_all_point_the_same_way() {
        let n = vertex_normals(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]], &[[0, 1, 2], [1, 3, 2]]);
        for v in n {
            assert!((v[2] - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn smoothing_a_constant_field_changes_nothing() {
        let out = smooth_masked(&[[5.0]; 4], &[true; 4], &strip(), 20);
        for v in out {
            assert!((v[0] - 5.0).abs() < 1e-9);
        }
    }

    #[test]
    fn smoothing_ignores_the_values_of_points_that_are_not_valid() {
        let out = smooth_masked(&[[1.0], [1.0], [1.0], [1000.0]], &[true, true, true, false], &strip(), 20);
        for v in out {
            assert!((v[0] - 1.0).abs() < 1e-9, "{v:?}");
        }
    }

    #[test]
    fn smoothing_pulls_a_spike_toward_its_neighbours() {
        let out = smooth_masked(&[[0.0], [0.0], [0.0], [8.0]], &[true; 4], &strip(), 20);
        assert!(out[3][0] < 8.0 && out[3][0] > out[0][0]);
        assert!(out.iter().all(|v| v[0] >= 0.0 && v[0] <= 8.0));
    }
}
