//! KOTOR's clips played on a set of bones, the way viewer 2 plays them. A clip's position keys are offsets added to a bone's local place
//! after scaling by the key scale, and its turn keys replace the bone's rest turn. The lids stay at rest.

use super::data::{Clip, Common, Track};
use crate::V3;
use crate::bones::{Bone, Quat, quat_matrix};

pub type M4 = [[f64; 4]; 4];

/// Each bone's offset and turn in one frame, by bone number.
pub type Pose = Vec<(Option<V3>, Option<Quat>)>;

fn lerp_keys<const N: usize>(keys: &[(f64, [f64; N])], t: f64, turn: bool) -> Option<[f64; N]> {
    let first = keys.first()?;
    let last = keys.last()?;
    if t <= first.0 {
        return Some(first.1);
    }
    if t >= last.0 {
        return Some(last.1);
    }
    let i = keys.windows(2).position(|w| w[1].0 > t)?;
    let (a, b) = (keys[i], keys[i + 1]);
    let f = (t - a.0) / (b.0 - a.0).max(1e-6);
    let mut vb = b.1;
    if turn && (0..N).map(|k| a.1[k] * vb[k]).sum::<f64>() < 0.0 {
        vb = vb.map(|x| -x);
    }
    let mut v: [f64; N] = std::array::from_fn(|k| a.1[k] + (vb[k] - a.1[k]) * f);
    if turn {
        let l = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        v = v.map(|x| x / l);
    }
    Some(v)
}

fn sample(track: &Track, t: f64) -> (Option<V3>, Option<Quat>) {
    (lerp_keys(&track.p, t, false), lerp_keys(&track.r, t, true))
}

pub fn clip_pose(common: &Common, clip: &Clip, t: f64) -> Pose {
    common
        .bone_names
        .iter()
        .map(|b| {
            clip.tracks
                .get(b)
                .map(|tr| sample(tr, t))
                .unwrap_or((None, None))
        })
        .collect()
}

/// The poses the fit learns from: the 15 lip sync shapes of the talk clip, and `per_clip` frames spread over each expression clip.
pub fn poses(common: &Common, per_clip: usize) -> Vec<Pose> {
    let mut out: Vec<Pose> = (1..16)
        .map(|s| clip_pose(common, &common.clips["talk"], s as f64 / 30.0))
        .collect();
    for name in ["tlklaugh", "tlkplead", "tlknorm", "tlksad", "tlkforce"] {
        let clip = &common.clips[name];
        for i in 0..per_clip {
            let f = 0.1 + 0.8 * i as f64 / (per_clip - 1).max(1) as f64;
            out.push(clip_pose(common, clip, f * clip.length));
        }
    }
    out
}

fn mul(a: &M4, b: &M4) -> M4 {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..4).map(|k| a[i][k] * b[k][j]).sum()))
}

/// The inverse of an affine map whose last row is 0, 0, 0, 1.
pub fn inverse(m: &M4) -> M4 {
    let a = |i: usize, j: usize| m[i][j];
    let det = a(0, 0) * (a(1, 1) * a(2, 2) - a(1, 2) * a(2, 1))
        - a(0, 1) * (a(1, 0) * a(2, 2) - a(1, 2) * a(2, 0))
        + a(0, 2) * (a(1, 0) * a(2, 1) - a(1, 1) * a(2, 0));
    let c = |i: usize, j: usize| {
        let (r0, r1) = ((i + 1) % 3, (i + 2) % 3);
        let (c0, c1) = ((j + 1) % 3, (j + 2) % 3);
        a(r0, c0) * a(r1, c1) - a(r0, c1) * a(r1, c0)
    };
    let turn: [[f64; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| c(j, i) / det));
    std::array::from_fn(|i| {
        if i == 3 {
            return [0.0, 0.0, 0.0, 1.0];
        }
        let r = turn[i];
        [
            r[0],
            r[1],
            r[2],
            -(0..3).map(|k| r[k] * m[k][3]).sum::<f64>(),
        ]
    })
}

pub fn apply(m: &M4, p: V3) -> V3 {
    std::array::from_fn(|i| m[i][0] * p[0] + m[i][1] * p[1] + m[i][2] * p[2] + m[i][3])
}

pub fn affine(rows: &[[f64; 4]; 3]) -> M4 {
    [rows[0], rows[1], rows[2], [0.0, 0.0, 0.0, 1.0]]
}

/// Each bone's world matrix in KOTOR's axes, with the pose's offsets and turns.
pub fn worlds(bones: &[Bone], key_scale: V3, pose: &Pose) -> Vec<M4> {
    let mut out: Vec<M4> = Vec::with_capacity(bones.len());
    for (i, b) in bones.iter().enumerate() {
        let (off, turn) = pose.get(i).copied().unwrap_or((None, None));
        let lid = b.name == "eyellid" || b.name == "eyerlid";
        let rot = quat_matrix(match turn {
            Some(q) if !lid => q,
            _ => b.rot,
        });
        let mut p = b.local;
        if let Some(o) = off {
            p = std::array::from_fn(|k| p[k] + key_scale[k] * o[k]);
        }
        let m: M4 = [
            [rot[0][0], rot[0][1], rot[0][2], p[0]],
            [rot[1][0], rot[1][1], rot[1][2], p[1]],
            [rot[2][0], rot[2][1], rot[2][2], p[2]],
            [0.0, 0.0, 0.0, 1.0],
        ];
        out.push(match b.parent {
            Some(q) => mul(&out[q], &m),
            None => m,
        });
    }
    out
}

/// The matrices that move a vertex from rest to the pose, in the head's axes, one per bone.
pub fn skin_matrices(bones: &[Bone], key_scale: V3, pose: &Pose) -> Vec<M4> {
    let swap: M4 = [
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let w = worlds(bones, key_scale, pose);
    let w0 = worlds(bones, key_scale, &vec![(None, None); bones.len()]);
    w.iter()
        .zip(&w0)
        .map(|(a, b)| mul(&swap, &mul(&mul(a, &inverse(b)), &swap)))
        .collect()
}

/// Positions moved by four bones each, with their weights scaled to sum to 1. A bone number past the matrices means no bone.
pub fn skin(pos: &[V3], idx: &[[usize; 4]], w: &[[f64; 4]], mats: &[M4]) -> Vec<V3> {
    pos.iter()
        .zip(idx.iter().zip(w))
        .map(|(&p, (ix, wt))| {
            let mut out = [0.0; 3];
            let mut total = 0.0;
            for k in 0..4 {
                let moved = mats.get(ix[k]).map(|m| apply(m, p)).unwrap_or(p);
                for d in 0..3 {
                    out[d] += wt[k] * moved[d];
                }
                total += wt[k];
            }
            out.map(|x| x / total)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_inverse_undoes_a_scaled_turn() {
        let m: M4 = [
            [0.0, -2.0, 0.0, 1.0],
            [2.0, 0.0, 0.0, 2.0],
            [0.0, 0.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let p = [0.3, -0.2, 0.7];
        let back = apply(&inverse(&m), apply(&m, p));
        assert!((0..3).all(|k| (back[k] - p[k]).abs() < 1e-12));
    }

    #[test]
    fn turn_keys_take_the_short_way() {
        let keys = [(0.0, [0.0, 0.0, 0.0, 1.0]), (1.0, [0.0, 0.0, 0.0, -1.0])];
        let q = lerp_keys(&keys, 0.5, true).unwrap();
        assert!((q[3] - 1.0).abs() < 1e-12);
    }
}
