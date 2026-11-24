//! Faults of a built head against the head it started from, among the triangles near the nose: triangles turned over (their normal
//! turned by more than 90 degrees), folds (neighbouring triangles meeting at more than 90 degrees where they met at under 45), and
//! nostril triangles turned over (a nostril turned inside out).

use crate::mesh::HeadMesh;
use crate::{V3, add, cross, dot, normalize, scale, sub};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Faults {
    pub flipped: usize,
    pub folded: usize,
    pub nostrils_flipped: usize,
    /// Where the first 40 faults are, on the head before (metres).
    pub at: Vec<V3>,
}

impl Faults {
    pub fn total(&self) -> usize {
        self.flipped + self.folded + self.nostrils_flipped
    }
}

pub struct Checker {
    triangles: Vec<[u32; 3]>,
    before: Vec<V3>,
    normals: Vec<V3>,
    nostril: Vec<bool>,
    /// The triangles near the nose, and their pairs across each shared edge.
    near: Vec<usize>,
    pairs: Vec<[usize; 2]>,
}

pub fn face_normals(p: &[V3], tris: &[[u32; 3]]) -> Vec<V3> {
    tris.iter()
        .map(|t| normalize(cross(sub(p[t[1] as usize], p[t[0] as usize]), sub(p[t[2] as usize], p[t[0] as usize]))))
        .collect()
}

impl Checker {
    /// `before` is the built head the faults are counted against, and `near` the vertices near the nose.
    pub fn new(mesh: &HeadMesh, before: &[V3], near: &[u32]) -> Self {
        let hidden: Vec<bool> = (0..before.len()).map(|i| mesh.hidden[mesh.group[i] as usize]).collect();
        Self::from_parts(&mesh.triangles, &hidden, before, near)
    }

    /// The same as `new`, from the triangles and one flag per vertex for the hidden insides (nostrils, mouth).
    pub fn from_parts(triangles: &[[u32; 3]], hidden: &[bool], before: &[V3], near: &[u32]) -> Self {
        let triangles = triangles.to_vec();
        let mut is_near = vec![false; before.len()];
        for &v in near {
            is_near[v as usize] = true;
        }
        // a nostril's inside: hidden vertices above the mouth
        let nostril: Vec<bool> = triangles.iter().map(|t| t.iter().all(|&i| hidden[i as usize] && before[i as usize][1] > 0.6)).collect();
        let near_tris: Vec<usize> = (0..triangles.len()).filter(|&k| triangles[k].iter().all(|&i| is_near[i as usize])).collect();
        let mut edges: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
        for &k in &near_tris {
            if nostril[k] {
                continue;
            }
            let t = triangles[k];
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                edges.entry((a.min(b), a.max(b))).or_default().push(k);
            }
        }
        let pairs = edges.values().filter(|f| f.len() == 2).map(|f| [f[0], f[1]]).collect();
        let normals = face_normals(before, &triangles);
        Self { triangles, before: before.to_vec(), normals, nostril, near: near_tris, pairs }
    }

    /// Which triangles are faults: turned over, or on either side of a fold.
    pub fn marks(&self, built: &[V3]) -> Vec<bool> {
        let n = face_normals(built, &self.triangles);
        let mut out = vec![false; self.triangles.len()];
        for &k in &self.near {
            if dot(n[k], self.normals[k]) < 0.0 {
                out[k] = true;
            }
        }
        for &[a, b] in &self.pairs {
            if dot(n[a], n[b]) < 0.0 && dot(self.normals[a], self.normals[b]) > 0.707 {
                out[a] = true;
                out[b] = true;
            }
        }
        out
    }

    pub fn faults(&self, built: &[V3]) -> Faults {
        let n = face_normals(built, &self.triangles);
        let mut f = Faults::default();
        let centre = |k: usize| {
            let t = self.triangles[k];
            scale(add(add(self.before[t[0] as usize], self.before[t[1] as usize]), self.before[t[2] as usize]), 1.0 / 3.0)
        };
        for &k in &self.near {
            if dot(n[k], self.normals[k]) < 0.0 {
                if self.nostril[k] {
                    f.nostrils_flipped += 1;
                } else {
                    f.flipped += 1;
                    f.at.push(centre(k));
                }
            }
        }
        for &[a, b] in &self.pairs {
            if dot(n[a], n[b]) < 0.0 && dot(self.normals[a], self.normals[b]) > 0.707 {
                f.folded += 1;
                f.at.push(centre(a));
            }
        }
        f.at.truncate(40);
        f
    }
}
