//! Learning the regressor (`crate::regressor`): where each face bone and each teeth point lies, as a fixed weighted sum of 60 of our
//! head's vertices, from KOTOR's own bones and teeth on the fitted heads. The 60 vertices are spread over the mean head by farthest point
//! sampling, leaving out the inside of the mouth and the copies of one place. Each point's weights sum to 1, so the points move, turn and
//! grow with the head. Their smoothing is the one that places heads left out of the learning best.

use super::data::{Common, Head, Template};
use super::kotor::dense_weights;
use super::pose::{affine, apply, worlds};
use crate::V3;
use crate::bones::swap_axes;
use nalgebra::{DMatrix, DVector};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, HashSet};

const VERTICES: usize = 60;
const SMOOTHING: [f64; 4] = [1e-6, 1e-5, 1e-4, 1e-3];
const EYES: [&str; 4] = ["eyela", "eyera", "eyellid", "eyerlid"];

/// KOTOR's bones in our frame (the fit's), and in KOTOR's own model space turned into our axes.
fn kotor_bones(head: &Head) -> (Vec<V3>, Vec<V3>) {
    let w = worlds(
        &head.kbones,
        head.kkey_scale,
        &vec![(None, None); head.kbones.len()],
    );
    let t = affine(&head.to_fit);
    let model: Vec<V3> = w
        .iter()
        .map(|m| swap_axes([m[0][3], m[1][3], m[2][3]]))
        .collect();
    (model.iter().map(|&p| apply(&t, p)).collect(), model)
}

/// KOTOR's upper and lower teeth strips: the small separate pieces near the mouth moved by `head_g` or `f_jaw_g` alone. Each strip's 10
/// points come as 5 columns from one side to the other, top then bottom in each column. None when a head lacks a strip of 10 points.
pub fn teeth_points(head: &Head, common: &Common) -> Option<[Vec<V3>; 2]> {
    let mut ids: HashMap<[i64; 3], usize> = HashMap::new();
    let welded: Vec<usize> = head
        .kpos
        .iter()
        .map(|p| {
            let next = ids.len();
            *ids.entry(p.map(|x| (x * 1e5).round() as i64))
                .or_insert(next)
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
    let mut pieces: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (c, &w) in welded.iter().enumerate() {
        let r = find(&mut parent, w);
        pieces.entry(r).or_default().push(c);
    }
    let dense = dense_weights(head, common.bone_names.len());
    let lip = head.pos[common.lip_line];
    let (upper_bone, lower_bone) = (common.bone("head_g"), common.bone("f_jaw_g"));
    let mut best: [Option<Vec<usize>>; 2] = [None, None];
    for corners in pieces.values() {
        let near = corners.iter().all(|&c| {
            let p = head.kpos[c];
            (p[0] - lip[0]).abs() <= 0.035 && (p[1] - lip[1]).abs() <= 0.03 && p[2] - lip[2] <= 0.01
        });
        if corners.len() > 180 || !near {
            continue;
        }
        let mean: Vec<f64> = (0..common.bone_names.len())
            .map(|b| corners.iter().map(|&c| dense[c][b]).sum::<f64>() / corners.len() as f64)
            .collect();
        let top = (0..mean.len())
            .max_by(|a, b| mean[*a].total_cmp(&mean[*b]))
            .unwrap();
        let row = match top {
            b if b == upper_bone => 0,
            b if b == lower_bone => 1,
            _ => continue,
        };
        if mean[top] > 0.95
            && best[row]
                .as_ref()
                .is_none_or(|old| corners.len() > old.len())
        {
            best[row] = Some(corners.clone());
        }
    }
    let strip = |corners: &Vec<usize>| -> Option<Vec<V3>> {
        let mut seen = HashSet::new();
        let mut pts: Vec<V3> = corners
            .iter()
            .map(|&c| head.kpos[c])
            .filter(|p| seen.insert(p.map(|x| (x * 1e6).round() as i64)))
            .collect();
        if pts.len() != 10 {
            return None;
        }
        pts.sort_by(|a, b| a[0].total_cmp(&b[0]));
        for pair in pts.chunks_mut(2) {
            pair.sort_by(|a, b| b[1].total_cmp(&a[1]));
        }
        Some(pts)
    };
    Some([strip(best[0].as_ref()?)?, strip(best[1].as_ref()?)?])
}

fn spread(mean: &[V3], candidates: &[usize], seed: usize) -> Vec<usize> {
    let dist = |a: V3, b: V3| crate::length(crate::sub(a, b));
    let mut chosen = vec![seed];
    let mut d: Vec<f64> = candidates
        .iter()
        .map(|&c| dist(mean[c], mean[seed]))
        .collect();
    while chosen.len() < VERTICES {
        let i = (0..d.len()).max_by(|a, b| d[*a].total_cmp(&d[*b])).unwrap();
        let j = candidates[i];
        chosen.push(j);
        for (k, &c) in candidates.iter().enumerate() {
            d[k] = d[k].min(dist(mean[c], mean[j]));
        }
    }
    chosen
}

/// The weights over the chosen vertices, summing to 1, that best place `targets` from `shapes`, with a ridge of `smoothing`.
fn fit(shapes: &[&Vec<V3>], targets: &[V3], chosen: &[usize], smoothing: f64) -> Vec<f64> {
    let k = chosen.len();
    let a = DMatrix::from_fn(3 * shapes.len(), k, |r, j| shapes[r / 3][chosen[j]][r % 3]);
    let b = DVector::from_fn(3 * shapes.len(), |r, _| targets[r / 3][r % 3]);
    let mut m = DMatrix::zeros(k + 1, k + 1);
    m.view_mut((0, 0), (k, k))
        .copy_from(&(a.transpose() * &a + DMatrix::identity(k, k) * smoothing));
    for j in 0..k {
        m[(j, k)] = 1.0;
        m[(k, j)] = 1.0;
    }
    let mut rhs = DVector::zeros(k + 1);
    rhs.rows_mut(0, k).copy_from(&(a.transpose() * b));
    rhs[k] = 1.0;
    let x = m.lu().solve(&rhs).expect("the regressor's system");
    x.rows(0, k).iter().copied().collect()
}

fn place(shape: &[V3], chosen: &[usize], w: &[f64]) -> V3 {
    std::array::from_fn(|d| chosen.iter().zip(w).map(|(&v, wi)| wi * shape[v][d]).sum())
}

/// The weights of one point and its mean error, in millimetres, on heads left out one at a time.
fn learn_point(shapes: &[&Vec<V3>], targets: &[V3], chosen: &[usize]) -> (Vec<f64>, f64) {
    let mut best = (f64::INFINITY, SMOOTHING[0]);
    for &s in &SMOOTHING {
        let e: f64 = (0..shapes.len())
            .map(|i| {
                let (sh, tg): (Vec<&Vec<V3>>, Vec<V3>) = (0..shapes.len())
                    .filter(|&j| j != i)
                    .map(|j| (shapes[j], targets[j]))
                    .unzip();
                let w = fit(&sh, &tg, chosen, s);
                crate::length(crate::sub(place(shapes[i], chosen, &w), targets[i])) * 1000.0
            })
            .sum::<f64>()
            / shapes.len() as f64;
        if e < best.0 {
            best = (e, s);
        }
    }
    (fit(shapes, targets, chosen, best.1), best.0)
}

/// The regressor learned from `heads`, in the file format of `crate::regressor::Regressor`, and a line of results per point.
pub fn learn(common: &Common, template: &Template, heads: &[Head]) -> (Value, Vec<String>) {
    let n = common.n;
    let mean: Vec<V3> = (0..n)
        .map(|v| {
            std::array::from_fn(|d| {
                heads.iter().map(|h| h.pos[v][d]).sum::<f64>() / heads.len() as f64
            })
        })
        .collect();
    let inside: HashSet<usize> = template.mouth_inside.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut firsts: Vec<(Vec<i64>, usize)> = (0..n)
        .filter(|v| seen.insert(mean[*v].map(|x| (x * 1e5).round() as i64)))
        .map(|v| (mean[v].map(|x| (x * 1e5).round() as i64).to_vec(), v))
        .collect();
    firsts.sort();
    let candidates: Vec<usize> = firsts
        .into_iter()
        .map(|(_, v)| v)
        .filter(|v| !inside.contains(v))
        .collect();
    let chosen = spread(&mean, &candidates, common.lip_line);
    let shapes: Vec<&Vec<V3>> = heads.iter().map(|h| &h.pos).collect();
    let bones: Vec<(usize, &String)> = common
        .bone_names
        .iter()
        .enumerate()
        .filter(|(_, b)| !EYES.contains(&b.as_str()))
        .collect();
    let placed: Vec<(Vec<V3>, Vec<V3>)> = heads.iter().map(kotor_bones).collect();
    let mut points = Vec::new();
    let mut report = Vec::new();
    for &(i, name) in &bones {
        let targets: Vec<V3> = placed.iter().map(|(fit_frame, _)| fit_frame[i]).collect();
        let (w, e) = learn_point(&shapes, &targets, &chosen);
        report.push(format!("{name}: {e:.2} mm on heads left out"));
        points.push(json!({"name": name, "vertices": chosen, "weights": w}));
    }
    let teeth: Vec<Option<[Vec<V3>; 2]>> = heads.iter().map(|h| teeth_points(h, common)).collect();
    let with: Vec<usize> = (0..heads.len()).filter(|&i| teeth[i].is_some()).collect();
    report.push(format!(
        "teeth strips of 10 points on {} of {} heads",
        with.len(),
        heads.len()
    ));
    let teeth_shapes: Vec<&Vec<V3>> = with.iter().map(|&i| &heads[i].pos).collect();
    let mut teeth_error: f64 = 0.0;
    for (row, r) in ["upper", "lower"].iter().enumerate() {
        for col in 0..5 {
            for (k, end) in ["top", "bottom"].iter().enumerate() {
                let targets: Vec<V3> = with
                    .iter()
                    .map(|&i| teeth[i].as_ref().unwrap()[row][2 * col + k])
                    .collect();
                let (w, e) = learn_point(&teeth_shapes, &targets, &chosen);
                teeth_error = teeth_error.max(e);
                points.push(json!({"name": format!("teeth_{r}_{col}_{end}"), "vertices": chosen, "weights": w}));
            }
        }
    }
    report.push(format!(
        "teeth points: worst {teeth_error:.2} mm on heads left out"
    ));
    let reference: serde_json::Map<String, Value> = bones
        .iter()
        .map(|&(i, name)| {
            let p: V3 = std::array::from_fn(|d| {
                placed.iter().map(|(_, m)| m[i][d]).sum::<f64>() / placed.len() as f64
            });
            (name.clone(), json!(p))
        })
        .collect();
    (
        json!({"n": n, "points": points, "reference": reference,
               "note": "Each point is the sum of weight times vertex over the listed vertices of viewer 2's mesh, in the viewer's frame."}),
        report,
    )
}
