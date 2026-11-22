//! What the skin colour tools share (`skin_measure`, `skin_fit`): the live page's layout and heads, the zones of the face placed from a
//! head's landmarks, and the colour helpers. The tools include it with `#[path]`, after `bundle.rs` and `fold_common.rs`.

#![allow(dead_code)]

use head_paint::colour::srgb_to_lab;
use head_paint::folds::Landmarks;
use head_paint::live::Layout;
use head_paint::pores::PoreLandmarks;
use serde_json::{Value, json};
use std::path::Path;

pub const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";

pub fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().expect("8"))).collect()
}

pub fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().expect("4"))).collect()
}

pub fn layout() -> Layout {
    let l = Path::new(DATA).join("layout");
    let triples = |v: Vec<u32>| v.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect::<Vec<_>>();
    Layout {
        uv: f64s(&l.join("uv.f64")).chunks_exact(2).map(|p| [p[0], p[1]]).collect(),
        triangles: triples(u32s(&l.join("tris.u32"))),
        layout_to_mesh: u32s(&l.join("vmap.u32")).into_iter().map(|i| i as usize).collect(),
        mesh_triangles: triples(u32s(&l.join("mesh_tris.u32"))),
    }
}

pub fn positions(name: &str) -> Vec<[f64; 3]> {
    f64s(&Path::new(DATA).join("heads").join(format!("{name}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect()
}

pub fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}
pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn mid(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|k| 0.5 * (a[k] + b[k]))
}
pub fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(sub(a, b), sub(a, b)).sqrt()
}
pub fn seg(p: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    let t = if l2 > 0.0 { (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
    dist(p, add(a, ab, t))
}

pub const ZONES: [&str; 10] = ["cheeks", "nose", "ears", "forehead", "round the eyes", "round the mouth", "chin", "jaw", "temples", "middle"];
pub const ZONE_TINT: [[u8; 3]; 10] = [[255, 60, 60], [255, 160, 0], [200, 0, 255], [255, 255, 0], [0, 120, 255], [0, 220, 120], [255, 0, 160], [0, 230, 230], [140, 255, 0], [255, 255, 255]];

/// Where the zones lie on one head, from its landmarks in millimetres.
pub struct Zones {
    pub l: Landmarks,
    pub m: PoreLandmarks,
    pub eyes: [f64; 3],
    pub ears: [[f64; 3]; 2],
    pub cheeks: [[f64; 3]; 2],
    pub jaw: [[f64; 3]; 2],
    pub temples: [[f64; 3]; 2],
    pub chin: [f64; 3],
}

impl Zones {
    pub fn new(l: Landmarks, m: PoreLandmarks, positions_mm: &[[f64; 3]]) -> Self {
        let eyes = mid(mid(l.eye_inner[0], l.eye_outer[0]), mid(l.eye_inner[1], l.eye_outer[1]));
        let f = l.frame;
        let mut ears = [eyes; 2];
        for side in 0..2 {
            let out = f.out(side);
            let mut best = f64::MIN;
            for &p in positions_mm {
                let r = sub(p, eyes);
                let (y, z) = (dot(r, f.up), dot(r, f.forward));
                if (-40.0..=0.0).contains(&y) && (-110.0..=-55.0).contains(&z) {
                    let x = dot(r, out);
                    if x > best {
                        best = x;
                        ears[side] = p;
                    }
                }
            }
            ears[side] = add(ears[side], out, -6.0);
        }
        // A centre placed by offsets from the landmarks can lie in front of the skin, so it is moved to the nearest vertex of the head.
        let snap = |c: [f64; 3]| positions_mm.iter().copied().min_by(|a, b| dist(*a, c).total_cmp(&dist(*b, c))).unwrap_or(c);
        let cheeks = [0, 1].map(|s| snap(add(add(l.lower_lid[s], f.up, -17.0), f.out(s), 4.0)));
        let jaw = [0, 1].map(|s| snap(add(add(l.mouth_corner[s], f.out(s), 28.0), f.up, -22.0)));
        let temples = [0, 1].map(|s| snap(add(l.eye_outer[s], f.out(s), 16.0)));
        let chin = snap(add(l.chin_middle, f.up, -8.0));
        Zones { l, m, eyes, ears, cheeks, jaw, temples, chin }
    }

    /// The point in the face's frame: across, up and forward from the middle of the eyes.
    pub fn local(&self, p: [f64; 3]) -> [f64; 3] {
        let r = sub(p, self.eyes);
        let f = self.l.frame;
        [dot(r, f.across), dot(r, f.up), dot(r, f.forward)]
    }

    /// Whether a point is left out of every measure: the lips, the eye openings and the band of the brows.
    pub fn excluded(&self, p: [f64; 3]) -> bool {
        let l = &self.l;
        let mouth = mid(l.mouth_corner[0], l.mouth_corner[1]);
        let half = 0.5 * dist(l.mouth_corner[0], l.mouth_corner[1]);
        let r = sub(p, mouth);
        let (mx, my) = (dot(r, l.frame.across), dot(r, l.frame.up));
        let lips = mx.abs() < half + 1.0 && (-9.0..=7.0).contains(&my) && dot(r, l.frame.forward) > -15.0;
        let opening = (0..2).any(|s| seg(p, l.eye_inner[s], l.eye_outer[s]) < 5.0);
        let q = self.local(p);
        let brows = (0..2).any(|s| {
            let e = mid(l.eye_inner[s], l.eye_outer[s]);
            let r = sub(p, e);
            let (x, y) = (dot(r, l.frame.across), dot(r, l.frame.up));
            x.abs() < 26.0 && (5.0..=26.0).contains(&y)
        });
        lips || opening || brows || q[2] < -120.0
    }

    /// Whether a point counts toward the face's mean: the front of the face from the chin to the forehead.
    pub fn face(&self, p: [f64; 3]) -> bool {
        let q = self.local(p);
        q[0].abs() < 62.0 && (-95.0..=55.0).contains(&q[1]) && q[2] > -45.0
    }

    /// The zone a point belongs to, or none. The zones do not overlap: the first that holds the point wins.
    pub fn zone(&self, p: [f64; 3]) -> Option<usize> {
        let l = &self.l;
        let f = l.frame;
        let q = self.local(p);
        let mouth = mid(l.mouth_corner[0], l.mouth_corner[1]);
        if (0..2).any(|s| dist(p, self.ears[s]) < 16.0) {
            return Some(2);
        }
        if seg(p, self.m.nose[0], self.m.nose[1]) < 7.0 || dist(p, l.nose_tip) < 9.0 {
            return Some(1);
        }
        if (0..2).any(|s| dist(p, self.cheeks[s]) < 11.0) {
            return Some(0);
        }
        let around_mouth = seg(p, l.mouth_corner[0], l.mouth_corner[1]);
        let my = dot(sub(p, mouth), f.up);
        if (7.0..=14.0).contains(&around_mouth) && (-14.0..=12.0).contains(&my) {
            return Some(5);
        }
        if (0..2).any(|s| {
            let d = seg(p, l.eye_inner[s], l.eye_outer[s]);
            let e = mid(l.eye_inner[s], l.eye_outer[s]);
            (5.0..=12.0).contains(&d) && dot(sub(p, e), f.up) < 4.0
        }) {
            return Some(4);
        }
        if dist(p, self.chin) < 10.0 {
            return Some(6);
        }
        if (0..2).any(|s| dist(p, self.jaw[s]) < 11.0) {
            return Some(7);
        }
        if (0..2).any(|s| dist(p, self.temples[s]) < 9.0) {
            return Some(8);
        }
        if q[0].abs() < 16.0 && (28.0..=55.0).contains(&q[1]) && q[2] > -30.0 {
            return Some(3);
        }
        None
    }

    /// Where a beard or stubble grows on a man's face: the upper lip, the chin, the jaw and the lower cheeks, from 2 mm below the nose tip
    /// down to 55 mm below the mouth and up to 62 mm to each side. The lips themselves are left out by `excluded`.
    pub fn beard(&self, p: [f64; 3]) -> bool {
        let l = &self.l;
        let f = l.frame;
        let mouth = mid(l.mouth_corner[0], l.mouth_corner[1]);
        let r = sub(p, mouth);
        let (x, y, z) = (dot(r, f.across), dot(r, f.up), dot(r, f.forward));
        let below_nose = dot(sub(p, l.nose_tip), f.up) < -2.0;
        below_nose && x.abs() < 62.0 && y > -55.0 && y < 18.0 && z > -70.0
    }

    /// The middle of the face (the nose, the middle of the forehead, the upper lip and the chin) against its sides (beyond 40 mm across).
    pub fn middle_or_side(&self, p: [f64; 3]) -> Option<bool> {
        let q = self.local(p);
        if !self.face(p) {
            return None;
        }
        if q[0].abs() < 14.0 {
            Some(true)
        } else if q[0].abs() > 40.0 {
            Some(false)
        } else {
            None
        }
    }
}

pub fn lab_of(px: [u8; 3]) -> [f64; 3] {
    srgb_to_lab(px.map(|v| f64::from(v) / 255.0))
}

pub fn chroma(lab: [f64; 3]) -> f64 {
    lab[1].hypot(lab[2])
}
pub fn hue(lab: [f64; 3]) -> f64 {
    lab[2].atan2(lab[1]).to_degrees()
}

pub fn mean(v: &[[f64; 3]]) -> [f64; 3] {
    let n = v.len().max(1) as f64;
    [0, 1, 2].map(|k| v.iter().map(|x| x[k]).sum::<f64>() / n)
}

pub fn r2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// The ramp of a set of Lab colours: sorted by lightness, in 10 bins of equal count, each bin's mean L*, chroma and hue.
pub fn ramp(mut v: Vec<[f64; 3]>) -> Value {
    v.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let n = v.len();
    let bins: Vec<Value> = (0..10)
        .map(|b| {
            let part = &v[b * n / 10..((b + 1) * n / 10).max(b * n / 10 + 1).min(n)];
            let m = mean(part);
            json!({ "L": r2(m[0]), "C": r2(chroma(m)), "h": r2(hue(m)), "a": r2(m[1]), "b": r2(m[2]) })
        })
        .collect();
    // The slope of chroma and of hue against lightness, by least squares over every pixel.
    let ml = v.iter().map(|x| x[0]).sum::<f64>() / n as f64;
    let mc = v.iter().map(|x| chroma(*x)).sum::<f64>() / n as f64;
    let mh = v.iter().map(|x| hue(*x)).sum::<f64>() / n as f64;
    let var = v.iter().map(|x| (x[0] - ml).powi(2)).sum::<f64>();
    let cl = v.iter().map(|x| (x[0] - ml) * (chroma(*x) - mc)).sum::<f64>() / var;
    let hl = v.iter().map(|x| (x[0] - ml) * (hue(*x) - mh)).sum::<f64>() / var;
    let sd = (var / n as f64).sqrt();
    json!({ "bins": bins, "chroma_per_L": (cl * 1000.0).round() / 1000.0, "hue_per_L": (hl * 1000.0).round() / 1000.0, "L_spread": r2(sd) })
}

pub fn delta(a: [f64; 3], b: [f64; 3]) -> Value {
    json!({ "dL": r2(a[0] - b[0]), "da": r2(a[1] - b[1]), "db": r2(a[2] - b[2]), "dC": r2(chroma(a) - chroma(b)), "dh": r2(hue(a) - hue(b)) })
}

/// The face's colour as a function of lightness: a* and b* averaged in 20 bins of equal count, read between the bins' mean lightness.
pub struct RampFn {
    pub l: Vec<f64>,
    pub a: Vec<f64>,
    pub b: Vec<f64>,
}

impl RampFn {
    pub fn new(v: &[[f64; 3]]) -> Self {
        let mut v = v.to_vec();
        v.sort_by(|a, b| a[0].total_cmp(&b[0]));
        let n = v.len();
        let bins: Vec<[f64; 3]> = (0..20).map(|b| mean(&v[b * n / 20..((b + 1) * n / 20).max(b * n / 20 + 1).min(n)])).collect();
        RampFn { l: bins.iter().map(|x| x[0]).collect(), a: bins.iter().map(|x| x[1]).collect(), b: bins.iter().map(|x| x[2]).collect() }
    }

    pub fn at(&self, l: f64) -> [f64; 2] {
        let k = self.l.iter().position(|&x| x > l).unwrap_or(self.l.len()).clamp(1, self.l.len() - 1);
        let t = ((l - self.l[k - 1]) / (self.l[k] - self.l[k - 1]).max(1e-6)).clamp(0.0, 1.0);
        [self.a[k - 1] + t * (self.a[k] - self.a[k - 1]), self.b[k - 1] + t * (self.b[k] - self.b[k - 1])]
    }

    /// The mean of a* and b* minus what the face's ramp gives at each pixel's own lightness: the zone's colour beyond its light.
    pub fn beyond(&self, v: &[[f64; 3]]) -> Value {
        let n = v.len().max(1) as f64;
        let (mut da, mut db) = (0.0, 0.0);
        for x in v {
            let r = self.at(x[0]);
            da += (x[1] - r[0]) / n;
            db += (x[2] - r[1]) / n;
        }
        json!({ "da": r2(da), "db": r2(db) })
    }
}
