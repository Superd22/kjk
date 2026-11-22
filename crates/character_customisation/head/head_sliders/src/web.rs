//! Building a head from slider values and measuring its nose in the browser, with the same code as the bench.

use crate::V3;
use crate::check::Checker;
use crate::eyes::EYES;
use crate::set::SMOOTHING;
use crate::measure::{Frame, Surface, measure};
use crate::feature::by_key;
use crate::set::{SliderSet, smoother};
use head_shape::smooth::Smoother;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Builder {
    smoother: Smoother,
    set: SliderSet,
    triangles: Vec<[u32; 3]>,
    fold: Option<Checker>,
}

fn points(flat: &[f32]) -> Vec<V3> {
    flat.as_chunks::<3>()
        .0
        .iter()
        .map(|c| c.map(|x| x as f64))
        .collect()
}

#[wasm_bindgen]
impl Builder {
    /// `base` holds the base head's positions (three numbers per vertex, metres), `triangles` three indices per triangle, `keep` one
    /// byte per vertex (not 0 for a vertex the smoothing holds still), and `sliders` the slider set as JSON.
    #[wasm_bindgen(constructor)]
    pub fn new(
        base: &[f32],
        triangles: &[u32],
        keep: &[u8],
        sliders: &str,
    ) -> Result<Builder, JsError> {
        let set: SliderSet = serde_json::from_str(sliders)?;
        let base = points(base);
        let triangles: Vec<[u32; 3]> = triangles.as_chunks::<3>().0.to_vec();
        let keep: Vec<bool> = keep.iter().map(|&k| k != 0).collect();
        crate::eye_curves::init(&base, &triangles);
        crate::set::init_mesh(&triangles, &keep);
        Ok(Builder {
            smoother: smoother(&base, &triangles, &keep),
            set,
            triangles,
            fold: None,
        })
    }

    /// Prepares the eye opening's curve (`head_eye_curve`) for sets with its sliders: `group` each vertex's merged vertex, `first` one
    /// vertex per merged vertex, `hidden` one byte per merged vertex (not 0 inside the nostrils and the mouth).
    pub fn init_curve(&self, group: &[u32], first: &[u32], hidden: &[u8]) {
        crate::set::init_curve(group.to_vec(), first.to_vec(), hidden.iter().map(|&h| h != 0).collect());
    }

    /// Prepares the throat construction (`chin_neck::ThroatShape`) for sets with the throat's sliders: `base` the base head
    /// (metres), `group`, `first` and `hidden` as `init_curve`, `near` six skin merged vertices per hidden merged vertex and `near_w`
    /// their weights (`mesh::HeadMesh`), and `free` one byte per merged vertex (not 0 where the construction may move it).
    #[allow(clippy::too_many_arguments)]
    pub fn init_throat(&self, base: &[f32], group: &[u32], first: &[u32], hidden: &[u8], near: &[u32], near_w: &[f64], free: &[u8]) {
        let mesh = crate::mesh::HeadMesh::new(
            points(base),
            self.triangles.clone(),
            crate::set::kept_flags(),
            group.to_vec(),
            first.to_vec(),
            hidden.iter().map(|&h| h != 0).collect(),
            near.as_chunks::<6>().0.to_vec(),
            near_w.as_chunks::<6>().0.to_vec(),
            Vec::new(),
            [[0.0; 3]; 2],
        );
        crate::set::init_throat(crate::chin_neck::ThroatShape::from_parts(mesh, free.iter().map(|&f| f != 0).collect()));
    }

    /// Eye `e`'s curve of the last build with the curve's sliders, as JSON: `centre` (metres), `axis` (unit), `angle` (degrees, the
    /// turn about the axis that closes the curve's own lid piece), and the `upper` and `lower` arches from the inner tip to the outer
    /// one (metres, three numbers per point). Empty when the last build had no curve.
    pub fn curve_info(&self, e: usize) -> String {
        crate::set::with_curve_lid(e, |l| {
            let flat = |v: &[[f64; 3]]| v.iter().flatten().copied().collect::<Vec<f64>>();
            serde_json::json!({ "centre": l.centre, "axis": l.axis, "angle": l.angle, "upper": flat(&l.upper), "lower": flat(&l.lower) }).to_string()
        })
        .unwrap_or_default()
    }

    /// The finer triangles below the lower lids (`head_eye_curve::refine`), the same for every head. Per new vertex (vertex
    /// `vertex count + j`), the three vertices of the head triangle it lies in.
    pub fn refine_parents(&self) -> Vec<u32> {
        crate::set::with_curve(|m| m.refine.parents.iter().flat_map(|p| p.0).collect()).unwrap_or_default()
    }

    /// Per new vertex, its weights on the three vertices of `refine_parents`.
    pub fn refine_weights(&self) -> Vec<f32> {
        crate::set::with_curve(|m| m.refine.parents.iter().flat_map(|p| p.1.map(|w| w as f32)).collect()).unwrap_or_default()
    }

    /// The triangles that replace the split head triangles (`refine_split`), three indices each.
    pub fn refine_triangles(&self) -> Vec<u32> {
        crate::set::with_curve(|m| m.refine.triangles.iter().flatten().copied().collect()).unwrap_or_default()
    }

    /// Per triangle of `refine_triangles`, the head triangle it lies in.
    pub fn refine_parent(&self) -> Vec<u32> {
        crate::set::with_curve(|m| m.refine.parent.clone()).unwrap_or_default()
    }

    /// The head triangles that the refinement splits.
    pub fn refine_split(&self) -> Vec<u32> {
        crate::set::with_curve(|m| m.refine.split.clone()).unwrap_or_default()
    }

    /// The new vertices' places in the last build with the curve's sliders (three numbers per vertex, metres), to be put after the
    /// head's own. Read right after that build, as `curve_info`.
    pub fn refine_extra(&self) -> Vec<f32> {
        crate::set::with_curve(|m| m.extra.iter().flat_map(|q| q.map(|x| x as f32)).collect()).unwrap_or_default()
    }

    /// The built head: `start` (the base head or a stand-in face, positions before smoothing) plus the slider values, smoothed.
    pub fn build(&self, start: &[f32], values: &[f64]) -> Vec<f32> {
        self.set
            .build(&self.smoother, &points(start), values)
            .iter()
            .flat_map(|p| p.map(|c| c as f32))
            .collect()
    }

    /// Prepares the fold check (`check::Checker`, the one the eye audit uses) for one head: `start` is its start shape, `eyes` its two
    /// eye centres (six numbers, metres) and `hidden` one byte per vertex (not 0 for the insides of the nostrils and the mouth). The
    /// faults are counted against the start smoothed, among the vertices in the eyes' window.
    pub fn fold_setup(&mut self, start: &[f32], eyes: &[f64], hidden: &[u8]) {
        let frame = crate::measure::head_frame([[eyes[0], eyes[1], eyes[2]], [eyes[3], eyes[4], eyes[5]]]);
        let before = self.smoother.smooth(&points(start), SMOOTHING);
        let window = crate::feature::Feature::window(&EYES);
        let near: Vec<u32> = (0..before.len() as u32).filter(|&v| window.contains(frame.local(before[v as usize]))).collect();
        let hidden: Vec<bool> = hidden.iter().map(|&h| h != 0).collect();
        self.fold = Some(Checker::from_parts(&self.triangles, &hidden, &before, &near));
    }

    /// The faulty triangles of the head built from `start` and `values` (after `fold_setup` for that start), as JSON: `count` is the
    /// kit's number (`Faults::total`), and `tris` the triangles drawn as faulty (`Checker::marks`).
    pub fn fold_marks(&self, start: &[f32], values: &[f64]) -> String {
        let Some(c) = &self.fold else { return serde_json::json!({ "error": "no fold_setup" }).to_string() };
        let built = self.set.build(&self.smoother, &points(start), values);
        serde_json::json!({ "count": c.faults(&built).total(), "tris": marked(&c.marks(&built)) }).to_string()
    }

    /// Which sliders cause the faults of the head built from `start` and `values`, as JSON. `singles` has, for each slider with a
    /// value other than 0, the number of faults that go away when only that slider is set to 0 (`saved`) and the triangles that stop
    /// being faulty (`tris`). `pairs` is the same for each pair among the `top` sliders that save most (ties by the largest value
    /// against its range), with `extra`, what the pair saves beyond its better slider alone. Only rows that save something are kept.
    pub fn fold_blame(&self, start: &[f32], values: &[f64], top: usize) -> String {
        let Some(c) = &self.fold else { return serde_json::json!({ "error": "no fold_setup" }).to_string() };
        let start = points(start);
        let at = |v: &[f64]| {
            let built = self.set.build(&self.smoother, &start, v);
            (c.faults(&built).total(), c.marks(&built))
        };
        let (count, marks) = at(values);
        let gone = |after: &[bool]| -> Vec<usize> { (0..marks.len()).filter(|&k| marks[k] && !after[k]).collect() };
        let mut singles: Vec<(usize, usize, Vec<usize>)> = Vec::new();
        let mut size: Vec<(usize, f64)> = Vec::new();
        if count > 0 {
            for k in 0..values.len().min(self.set.sliders.len()) {
                if values[k] == 0.0 {
                    continue;
                }
                let s = &self.set.sliders[k];
                size.push((k, values[k].abs() / s.low.abs().max(s.high.abs()).max(1e-12)));
                let mut v = values.to_vec();
                v[k] = 0.0;
                let (n, after) = at(&v);
                singles.push((k, count.saturating_sub(n), gone(&after)));
            }
        }
        let saved_of = |k: usize| singles.iter().find(|s| s.0 == k).map_or(0, |s| s.1);
        let mut order: Vec<usize> = size.iter().map(|s| s.0).collect();
        order.sort_by(|&a, &b| {
            let w = |k: usize| size.iter().find(|s| s.0 == k).map_or(0.0, |s| s.1);
            saved_of(b).cmp(&saved_of(a)).then(w(b).partial_cmp(&w(a)).unwrap_or(std::cmp::Ordering::Equal))
        });
        order.truncate(top);
        let mut pairs = Vec::new();
        for (i, &a) in order.iter().enumerate() {
            for &b in &order[i + 1..] {
                let mut v = values.to_vec();
                v[a] = 0.0;
                v[b] = 0.0;
                let (n, after) = at(&v);
                let saved = count.saturating_sub(n);
                if saved > 0 {
                    let extra = saved.saturating_sub(saved_of(a).max(saved_of(b)));
                    pairs.push(serde_json::json!({ "a": a, "b": b, "saved": saved, "extra": extra, "tris": gone(&after) }));
                }
            }
        }
        pairs.sort_by_key(|p| std::cmp::Reverse(p["saved"].as_u64().unwrap_or(0)));
        let mut singles: Vec<_> = singles.into_iter().filter(|s| s.1 > 0).collect();
        singles.sort_by_key(|s| std::cmp::Reverse(s.1));
        let singles: Vec<_> = singles.iter().map(|s| serde_json::json!({ "k": s.0, "saved": s.1, "tris": s.2 })).collect();
        serde_json::json!({ "count": count, "tris": marked(&marks), "singles": singles, "pairs": pairs }).to_string()
    }

    /// A built head's feature as JSON, measured in the frame of the head (`measure::head_frame`; `eyes` only matter for the old frame).
    pub fn measure_feature(&self, feature: &str, positions: &[f32], eyes: &[f64]) -> String {
        let Some(f) = by_key(feature) else { return serde_json::json!({ "error": "unknown feature" }).to_string() };
        let frame = crate::measure::head_frame([[eyes[0], eyes[1], eyes[2]], [eyes[3], eyes[4], eyes[5]]]);
        match f.measure(&Surface::within(&points(positions), &self.triangles, &frame, &f.window())) {
            Ok(m) => serde_json::to_string(&m).unwrap_or_default(),
            Err(e) => serde_json::json!({ "error": e }).to_string(),
        }
    }

    /// The nose of a built head as JSON, measured in the frame of the head (`eyes`: the left eye's centre, then the right's, metres, for the old frame).
    pub fn measure(&self, positions: &[f32], eyes: &[f64]) -> String {
        let frame = crate::measure::head_frame([[eyes[0], eyes[1], eyes[2]], [eyes[3], eyes[4], eyes[5]]]);
        match measure(&Surface::new(&points(positions), &self.triangles, &frame)) {
            Ok(n) => serde_json::to_string(&n).unwrap_or_default(),
            Err(e) => serde_json::json!({ "error": e }).to_string(),
        }
    }
}

fn marked(marks: &[bool]) -> Vec<usize> {
    (0..marks.len()).filter(|&k| marks[k]).collect()
}

/// Several slider sets (a JSON array of sets, one per feature) merged into one set, as JSON, for a `Builder` of every feature at once.
#[wasm_bindgen]
pub fn merge_sets(sets: &str) -> Result<String, JsError> {
    let sets: Vec<SliderSet> = serde_json::from_str(sets)?;
    Ok(serde_json::to_string(&SliderSet::merge(&sets))?)
}
