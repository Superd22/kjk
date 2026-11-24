//! KOTOR's own motion at each of our vertices. Each vertex is matched to the closest point of KOTOR's face, as a triangle and its corner
//! weights, and that point is moved with KOTOR's own skin. Near the mouth opening a vertex on the upper lip's side only matches triangles
//! of KOTOR's upper lip, and the lower side only the lower lip, so that the lips are not mixed where they nearly touch.

use super::data::{Common, Head};
use super::pose::{Pose, affine, inverse, skin, skin_matrices};
use crate::V3;
use head_paint::closest::Bvh;
use std::collections::HashMap;

const LOWER: [&str; 4] = ["f_jaw_g", "f_llm_g", "f_rlm_g", "f_tonguetip_g"];

/// The skin corners of KOTOR's head that belong to its largest connected piece, the face, without the eyes and teeth.
pub fn face_mask(kpos: &[V3]) -> Vec<bool> {
    let mut ids: HashMap<[i64; 3], usize> = HashMap::new();
    let welded: Vec<usize> = kpos
        .iter()
        .map(|p| {
            let key = p.map(|x| (x * 1e5).round() as i64);
            let next = ids.len();
            *ids.entry(key).or_insert(next)
        })
        .collect();
    let mut parent: Vec<usize> = (0..ids.len()).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    for t in welded.chunks(3) {
        for k in 1..3 {
            let (a, b) = (find(&mut parent, t[0]), find(&mut parent, t[k]));
            parent[a] = b;
        }
    }
    let roots: Vec<usize> = (0..ids.len()).map(|v| find(&mut parent, v)).collect();
    let mut size: HashMap<usize, usize> = HashMap::new();
    for &r in &roots {
        *size.entry(r).or_default() += 1;
    }
    let biggest = size
        .iter()
        .max_by_key(|(r, s)| (**s, std::cmp::Reverse(**r)))
        .map(|(r, _)| *r);
    welded.iter().map(|&w| Some(roots[w]) == biggest).collect()
}

/// KOTOR's weights of each skin corner over the bones of `common.json`, summing to 1.
pub fn dense_weights(head: &Head, bones: usize) -> Vec<Vec<f64>> {
    head.kidx
        .iter()
        .zip(&head.kw)
        .map(|(ix, w)| {
            let mut out = vec![0.0; bones];
            for k in 0..4 {
                if (ix[k] as usize) < bones {
                    out[ix[k] as usize] += w[k];
                }
            }
            let s: f64 = out.iter().sum();
            out.iter_mut().for_each(|x| *x /= s.max(1e-9));
            out
        })
        .collect()
}

/// Where each of our vertices lands on KOTOR's face: the distance, the triangle of KOTOR's corners and its corner weights.
pub struct Matches {
    pub distance: Vec<f64>,
    pub triangle: Vec<usize>,
    pub barycentric: Vec<[f64; 3]>,
}

/// `upper` and `side` are the template's lip sides of our vertices. A vertex with `side` over 0.5 only matches the triangles of its own lip.
pub fn match_face(head: &Head, common: &Common, upper: &[f64], side: &[f64]) -> Matches {
    let face = face_mask(&head.kpos);
    let dense = dense_weights(head, common.bone_names.len());
    let lower: Vec<usize> = LOWER.iter().map(|b| common.bone(b)).collect();
    let ntri = head.kpos.len() / 3;
    let tris: Vec<usize> = (0..ntri)
        .filter(|&t| (0..3).all(|k| face[3 * t + k]))
        .collect();
    let lower_tri: Vec<bool> = tris
        .iter()
        .map(|&t| {
            (0..3)
                .map(|k| lower.iter().map(|&b| dense[3 * t + k][b]).sum::<f64>())
                .sum::<f64>()
                / 3.0
                >= 0.5
        })
        .collect();
    let bvh = Bvh::new(
        head.kpos.clone(),
        tris.iter()
            .map(|&t| [3 * t as u32, 3 * t as u32 + 1, 3 * t as u32 + 2])
            .collect(),
    );
    let wants: Vec<Option<bool>> = (0..head.pos.len())
        .map(|v| (side[v] > 0.5).then(|| super::smooth(0.3, 0.7, upper[v]) < 0.5))
        .collect();
    let found = bvh.closest_all_with(&head.pos, &|v, t| {
        wants[v].is_none_or(|lo| lower_tri[t as usize] == lo)
    });
    let mut m = Matches {
        distance: Vec::with_capacity(found.len()),
        triangle: Vec::with_capacity(found.len()),
        barycentric: Vec::with_capacity(found.len()),
    };
    for f in found {
        let f = f.expect("a face triangle");
        m.distance.push(f.distance);
        m.triangle.push(tris[f.triangle as usize]);
        m.barycentric.push(f.barycentric);
    }
    m
}

/// KOTOR's skin corners, which are stored in our frame, moved to a pose. The bones move them in KOTOR's model space.
pub fn kotor_posed(head: &Head, pose: &Pose) -> Vec<V3> {
    let t = affine(&head.to_fit);
    let ti = inverse(&t);
    let mats: Vec<_> = skin_matrices(&head.kbones, head.kkey_scale, pose)
        .iter()
        .map(|m| mul3(&t, m, &ti))
        .collect();
    let idx: Vec<[usize; 4]> = head.kidx.iter().map(|i| i.map(|x| x as usize)).collect();
    skin(&head.kpos, &idx, &head.kw, &mats)
}

fn mul3(a: &super::pose::M4, b: &super::pose::M4, c: &super::pose::M4) -> super::pose::M4 {
    let ab: super::pose::M4 =
        std::array::from_fn(|i| std::array::from_fn(|j| (0..4).map(|k| a[i][k] * b[k][j]).sum()));
    std::array::from_fn(|i| std::array::from_fn(|j| (0..4).map(|k| ab[i][k] * c[k][j]).sum()))
}

/// The matched points of KOTOR's posed corners.
pub fn matched_points(posed: &[V3], m: &Matches) -> Vec<V3> {
    m.triangle
        .iter()
        .zip(&m.barycentric)
        .map(|(&t, b)| std::array::from_fn(|d| (0..3).map(|k| b[k] * posed[3 * t + k][d]).sum()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_face_is_the_largest_connected_piece() {
        let tri = |x: f64| vec![[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]];
        let mut kpos = tri(0.0);
        kpos.extend(tri(1.0));
        kpos.extend(tri(10.0));
        assert_eq!(face_mask(&kpos), [vec![true; 6], vec![false; 3]].concat());
    }
}
