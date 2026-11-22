//! KOTOR's face bones on a head. A bone's place is stored relative to its parent, in the parent's rest turn, in KOTOR's axes (+Y
//! forward, +Z up, the character's left at -X). The head's own axes are +Z forward and +Y up. Clips add their position keys to these
//! local places after scaling them by the key scale.

use crate::{V3, length, scale, sub};
use std::collections::BTreeMap;

/// Turns the head's axes into KOTOR's and back, since the swap is its own inverse.
pub fn swap_axes(v: V3) -> V3 {
    [-v[0], v[2], v[1]]
}

pub type Quat = [f64; 4];

/// The rotation matrix of a unit quaternion stored as x, y, z, w.
pub fn quat_matrix(q: Quat) -> [[f64; 3]; 3] {
    let [x, y, z, w] = q;
    [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ]
}

fn mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut m = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

fn apply(m: &[[f64; 3]; 3], v: V3) -> V3 {
    [0, 1, 2].map(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}

fn apply_transposed(m: &[[f64; 3]; 3], v: V3) -> V3 {
    [0, 1, 2].map(|i| m[0][i] * v[0] + m[1][i] * v[1] + m[2][i] * v[2])
}

#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub local: V3,
    pub rot: Quat,
}

/// Each bone's world rest turn and world place in the head's axes. Parents come before their children.
pub fn rest_world(bones: &[Bone]) -> Vec<([[f64; 3]; 3], V3)> {
    let mut out: Vec<([[f64; 3]; 3], V3)> = Vec::with_capacity(bones.len());
    for b in bones {
        let r = quat_matrix(b.rot);
        let (rot, at) = match b.parent {
            Some(p) => {
                let (pr, pa) = out[p];
                (mul(&pr, &r), crate::add(pa, apply(&pr, b.local)))
            }
            None => (r, b.local),
        };
        out.push((rot, at));
    }
    out
}

/// The bones moved to new world places, given in the head's axes. A bone with no new place keeps its world place, so the bones the
/// regressor does not place (the eyes and lids) stay where they are while their parents move. Rest turns are kept.
pub fn relocate(bones: &[Bone], places: &BTreeMap<String, V3>) -> Vec<Bone> {
    let world = rest_world(bones);
    let at: Vec<V3> = bones
        .iter()
        .zip(&world)
        .map(|(b, (_, a))| places.get(&b.name).map(|&p| swap_axes(p)).unwrap_or(*a))
        .collect();
    bones
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let local = match b.parent {
                Some(p) => apply_transposed(&world[p].0, sub(at[i], at[p])),
                None => at[i],
            };
            Bone { local, ..b.clone() }
        })
        .collect()
}

/// How large the head's bone layout is against KOTOR's average layout: the ratio of their spreads about their own centres. KOTOR's clip
/// offsets are in metres of KOTOR model space, so this is the factor they are scaled by on this head.
pub fn key_scale(places: &BTreeMap<String, V3>, reference: &BTreeMap<String, V3>) -> Option<f64> {
    let names: Vec<&String> = reference
        .keys()
        .filter(|n| places.contains_key(*n))
        .collect();
    if names.len() < 3 {
        return None;
    }
    let centre = |pts: &dyn Fn(&String) -> V3| {
        let mut c = [0.0; 3];
        for n in &names {
            c = crate::add(c, pts(n));
        }
        scale(c, 1.0 / names.len() as f64)
    };
    let a = |n: &String| places[n];
    let b = |n: &String| reference[n];
    let (ca, cb) = (centre(&a), centre(&b));
    let spread = |pts: &dyn Fn(&String) -> V3, c: V3| {
        names
            .iter()
            .map(|n| length(sub(pts(n), c)).powi(2))
            .sum::<f64>()
    };
    let sb = spread(&b, cb);
    (sb > 0.0).then(|| (spread(&a, ca) / sb).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn_z(deg: f64) -> Quat {
        let h = deg.to_radians() / 2.0;
        [0.0, 0.0, h.sin(), h.cos()]
    }

    fn chain() -> Vec<Bone> {
        vec![
            Bone {
                name: "root".into(),
                parent: None,
                local: [0.0, 0.0, 0.1],
                rot: turn_z(90.0),
            },
            Bone {
                name: "jaw".into(),
                parent: Some(0),
                local: [0.02, 0.0, 0.0],
                rot: [0.0, 0.0, 0.0, 1.0],
            },
            Bone {
                name: "eye".into(),
                parent: Some(0),
                local: [0.0, 0.03, 0.01],
                rot: [0.0, 0.0, 0.0, 1.0],
            },
        ]
    }

    fn close(a: V3, b: V3) -> bool {
        length(sub(a, b)) < 1e-12
    }

    #[test]
    fn a_relocated_bone_lands_on_its_new_place() {
        let bones = chain();
        let target = [0.01, 0.05, 0.08];
        let moved = relocate(&bones, &BTreeMap::from([("jaw".to_string(), target)]));
        let world = rest_world(&moved);
        assert!(close(swap_axes(world[1].1), target));
    }

    #[test]
    fn a_bone_left_out_keeps_its_world_place_when_its_parent_moves() {
        let bones = chain();
        let before = rest_world(&bones)[2].1;
        let moved = relocate(
            &bones,
            &BTreeMap::from([("root".to_string(), [0.3, 0.2, 0.1])]),
        );
        assert!(close(rest_world(&moved)[2].1, before));
    }

    #[test]
    fn the_key_scale_is_the_size_ratio() {
        let reference = BTreeMap::from([
            ("a".to_string(), [0.0, 0.0, 0.0]),
            ("b".to_string(), [1.0, 0.0, 0.0]),
            ("c".to_string(), [0.0, 1.0, 0.0]),
        ]);
        let places: BTreeMap<String, V3> = reference
            .iter()
            .map(|(k, v)| (k.clone(), crate::add(scale(*v, 0.9), [5.0, 1.0, 2.0])))
            .collect();
        assert!((key_scale(&places, &reference).unwrap() - 0.9).abs() < 1e-12);
    }
}
