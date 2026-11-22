//! A slider set as plain data, and building a head from slider values.
//!
//! A slider is a name, the zone it shapes, its range, and a movement of the vertices for each value. A head is the base head (or,
//! while only the nose has sliders, a stand-in for the rest of the face), plus each slider's movement at its value, then the
//! smoothing every built head gets.
//!
//! A slider's movement is either fixed, times its value (`vertices` and `moves`), or set at a few values (`keys`) and blended in
//! between. A key is a mix of the set's shared smooth fields (`fields`). The movements of all sliders add up, so the same values
//! give the same head in any order of edits, and moving a slider up and back returns the same head.

use crate::V3;
use head_shape::smooth::{Method, Smoother};
use serde::{Deserialize, Serialize};

/// The smoothing every head gets after its sliders, as `head_shape` builds the default head.
pub const SMOOTHING: Method = Method::Implicit { strength: 3.0 };
pub const SMOOTHING_RAMP_M: f64 = 0.004;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slider {
    pub name: String,
    /// "proportion" for a slider that sets one named measure, "shape" for one learned from the KOTOR heads.
    pub kind: String,
    pub zone: String,
    /// What one unit is, for a person.
    pub unit: String,
    /// The lowest and highest values the KOTOR heads use.
    pub low: f64,
    pub high: f64,
    /// The vertices it moves, and their movement in metres for one unit, three numbers per vertex. Empty when the slider has keys.
    pub vertices: Vec<u32>,
    pub moves: Vec<f32>,
    /// A sentence for a player.
    #[serde(default)]
    pub hint: String,
    /// The measure this slider sets, as a key of `measure::PROPORTIONS`, when it sets one.
    #[serde(default)]
    pub measure: String,
    /// The movement at a few values, in increasing order of value. Between two keys the movement is blended; past the last key it
    /// stays at the last key's.
    #[serde(default)]
    pub keys: Vec<Key>,
    /// The part of the feature it shapes, for grouping on a page (for the nose: root, bridge, tip, wings or base).
    #[serde(default)]
    pub group: String,
}

/// A slider's movement at one value: a mix of the set's fields.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Key {
    pub value: f64,
    pub fields: Vec<u32>,
    pub weights: Vec<f32>,
}

/// A smooth movement of a few vertices, in metres, three numbers per vertex, shared by the keys of a set's sliders.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub vertices: Vec<u32>,
    pub moves: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliderSet {
    pub vertex_count: usize,
    pub sliders: Vec<Slider>,
    #[serde(default)]
    pub fields: Vec<Field>,
}

impl Field {
    pub fn from_movement(movement: &[V3]) -> Self {
        let mut vertices = Vec::new();
        let mut moves = Vec::new();
        for (i, m) in movement.iter().enumerate() {
            if m.iter().any(|c| c.abs() > 1e-7) {
                vertices.push(i as u32);
                moves.extend(m.iter().map(|&c| c as f32));
            }
        }
        Self { vertices, moves }
    }

    pub fn add_to(&self, positions: &mut [V3], weight: f64) {
        for (&v, m) in self.vertices.iter().zip(self.moves.as_chunks::<3>().0) {
            for (p, d) in positions[v as usize].iter_mut().zip(m) {
                *p += weight * *d as f64;
            }
        }
    }

    pub fn movement(&self, vertex_count: usize) -> Vec<V3> {
        let mut out = vec![[0.0; 3]; vertex_count];
        self.add_to(&mut out, 1.0);
        out
    }
}

impl Slider {
    /// A slider from a movement given for every vertex. Vertices that move less than a thousandth of a millimetre are left out.
    pub fn from_movement(name: &str, kind: &str, unit: &str, movement: &[V3]) -> Self {
        let mut vertices = Vec::new();
        let mut moves = Vec::new();
        for (i, m) in movement.iter().enumerate() {
            if m.iter().any(|c| c.abs() > 1e-6) {
                vertices.push(i as u32);
                moves.extend(m.iter().map(|&c| c as f32));
            }
        }
        Self {
            name: name.into(),
            kind: kind.into(),
            zone: "nose".into(),
            unit: unit.into(),
            low: 0.0,
            high: 0.0,
            vertices,
            moves,
            hint: String::new(),
            measure: String::new(),
            keys: Vec::new(),
            group: String::new(),
        }
    }

    /// For a drawn slider (`kind` "drawn"), the amount of its feature's step at `value`: its keys hold the amounts (one weight each,
    /// no fields) at a few values, blended in between and held past the last ones.
    pub fn drawn_amount(&self, value: f64) -> f64 {
        let k = &self.keys;
        if k.is_empty() {
            return 0.0;
        }
        let v = value.clamp(k[0].value, k[k.len() - 1].value);
        let i = k.iter().rposition(|key| key.value <= v).unwrap_or(0).min(k.len().saturating_sub(2));
        let (a, b) = (&k[i], &k[(i + 1).min(k.len() - 1)]);
        let t = if b.value > a.value { (v - a.value) / (b.value - a.value) } else { 0.0 };
        let amount = |key: &Key| key.weights.first().copied().unwrap_or(0.0) as f64;
        amount(a) * (1.0 - t) + amount(b) * t
    }

    /// The mix of the set's fields at `value`, added to `weights` (one per field), for a slider with keys.
    pub fn add_key_weights(&self, value: f64, weights: &mut [f64]) {
        let k = &self.keys;
        if k.is_empty() {
            return;
        }
        let v = value.clamp(k[0].value, k[k.len() - 1].value);
        let i = k.iter().rposition(|key| key.value <= v).unwrap_or(0).min(k.len().saturating_sub(2));
        let (a, b) = (&k[i], &k[(i + 1).min(k.len() - 1)]);
        let t = if b.value > a.value { (v - a.value) / (b.value - a.value) } else { 0.0 };
        for (key, s) in [(a, 1.0 - t), (b, t)] {
            if s != 0.0 {
                for (&f, &w) in key.fields.iter().zip(&key.weights) {
                    weights[f as usize] += s * w as f64;
                }
            }
        }
    }

    pub fn movement(&self, vertex_count: usize) -> Vec<V3> {
        let mut out = vec![[0.0; 3]; vertex_count];
        self.add_to(&mut out, 1.0);
        out
    }

    pub fn add_to(&self, positions: &mut [V3], value: f64) {
        for (&v, m) in self.vertices.iter().zip(self.moves.as_chunks::<3>().0) {
            for (p, d) in positions[v as usize].iter_mut().zip(m) {
                *p += value * *d as f64;
            }
        }
    }
}

impl SliderSet {
    /// The head before smoothing: `start` (the base head, or a stand-in face) plus each value times its slider's movement. Values past
    /// the end of `values` count as 0.
    pub fn unsmoothed(&self, start: &[V3], values: &[f64]) -> Vec<V3> {
        let mut p = start.to_vec();
        let mut weights = vec![0.0; self.fields.len()];
        for (s, &v) in self.sliders.iter().zip(values) {
            if s.kind == "after" {
                continue;
            }
            if s.keys.is_empty() {
                if v != 0.0 {
                    s.add_to(&mut p, v);
                }
            } else {
                s.add_key_weights(v, &mut weights);
            }
        }
        for (f, &w) in self.fields.iter().zip(&weights) {
            if w != 0.0 {
                f.add_to(&mut p, w);
            }
        }
        p
    }

    /// Each field's weight for these values, from the sliders with keys.
    pub fn field_weights(&self, values: &[f64]) -> Vec<f64> {
        let mut weights = vec![0.0; self.fields.len()];
        for (s, &v) in self.sliders.iter().zip(values) {
            s.add_key_weights(v, &mut weights);
        }
        weights
    }

    /// The built head: the head before smoothing, smoothed.
    pub fn build(&self, smoother: &Smoother, start: &[V3], values: &[f64]) -> Vec<V3> {
        let mut p = smoother.smooth(&self.unsmoothed(start, values), SMOOTHING);
        // a slider of kind "after" is a movement added after the smoothing, so a narrow shape (the lips' corners) keeps its form
        for (s, &v) in self.sliders.iter().zip(values) {
            if s.kind == "after" && v != 0.0 {
                s.add_to(&mut p, v);
            }
        }
        // a set with eye sliders ends with the lids' edges pressed onto the eyeballs (`eye_curves::press`)
        if self.sliders.iter().any(|s| s.zone == "eyes") {
            crate::eye_curves::press(&mut p, start);
        }
        let steps = self.drawn_steps(values);
        if let Some(mesh) = MESH.get().filter(|m| m.keep.len() == p.len() && !steps.is_empty()) {
            let ids: Vec<u32> = (0..p.len() as u32).collect();
            for (step, amount) in steps {
                step(&mut p, &ids, &mesh.triangles, amount);
            }
        }
        if let Some(h) = self.curve_handles(values) {
            if let Some(maker) = CURVE.lock().unwrap().as_mut().filter(|m| m.vertex_count() == p.len()) {
                p = maker.build(&p, &h);
            }
        }
        // a set with the throat's sliders ends with the throat built on the outline they set (`chin_neck::ThroatShape`)
        if let Some(t) = self.throat_values(values) {
            if let Some(shape) = THROAT.get().filter(|s| s.vertex_count() == p.len()) {
                p = shape.apply(&p, &t);
            }
        }
        p
    }

    /// The throat's four values (`chin_neck::THROAT_KEYS`), when the set has the throat's sliders (`kind` "throat", each named by its
    /// key in `measure`). Each value is held within its slider's range, and a value the set lacks is the base head's.
    pub fn throat_values(&self, values: &[f64]) -> Option<crate::chin_neck::Throat> {
        let mut out = crate::chin_neck::THROAT_SLIDERS.map(|s| s.3);
        let mut any = false;
        for (s, &v) in self.sliders.iter().zip(values.iter().chain(std::iter::repeat(&0.0))) {
            if s.kind == "throat" {
                if let Some(k) = crate::chin_neck::THROAT_KEYS.iter().position(|m| *m == s.measure) {
                    out[k] = v.clamp(s.low.min(s.high), s.high.max(s.low));
                    any = true;
                }
            }
        }
        any.then(|| crate::chin_neck::Throat::from_slice(&out))
    }

    /// The ten values of the eye opening's curve (`head_eye_curve::curve::HANDLES`, in that order), when the set has the curve's
    /// sliders (`kind` "curve", each named by its handle in `measure`). Each value is held within its slider's range, and a handle the
    /// set lacks is 0.
    pub fn curve_handles(&self, values: &[f64]) -> Option<[f64; 10]> {
        let mut out = [f64::NAN; 10];
        let mut any = false;
        for (s, &v) in self.sliders.iter().zip(values.iter().chain(std::iter::repeat(&0.0))) {
            if s.kind == "curve" {
                if let Some(k) = head_eye_curve::curve::HANDLES.iter().position(|m| *m == s.measure) {
                    out[k] = v.clamp(s.low.min(s.high), s.high.max(s.low));
                    any = true;
                }
            }
        }
        if !any {
            return None;
        }
        Some(out.map(|x| if x.is_nan() { 0.0 } else { x }))
    }

    /// The steps of the set's drawn sliders, with their amounts at `values`, for those whose amount is not 0.
    pub fn drawn_steps(&self, values: &[f64]) -> Vec<(DrawnStep, f64)> {
        self.sliders
            .iter()
            .zip(values)
            .filter(|(s, _)| s.kind == "drawn")
            .filter_map(|(s, &v)| Some((drawn_step(&s.measure)?, s.drawn_amount(v))))
            .filter(|(_, a)| *a != 0.0)
            .collect()
    }

    /// The first `count` sliders.
    pub fn first(&self, count: usize) -> SliderSet {
        SliderSet {
            vertex_count: self.vertex_count,
            sliders: self.sliders[..count.min(self.sliders.len())].to_vec(),
            fields: self.fields.clone(),
        }
    }
}

impl SliderSet {
    /// Several features' sets as one: their sliders in order, and their fields in order with each key's field numbers moved along.
    /// A head built from the merged set with every feature's values is the head built from each set's values in turn.
    pub fn merge(sets: &[SliderSet]) -> SliderSet {
        let mut out = SliderSet { vertex_count: sets.first().map_or(0, |s| s.vertex_count), sliders: Vec::new(), fields: Vec::new() };
        for s in sets {
            let offset = out.fields.len() as u32;
            out.sliders.extend(s.sliders.iter().map(|sl| {
                let mut sl = sl.clone();
                for k in &mut sl.keys {
                    for f in &mut k.fields {
                        *f += offset;
                    }
                }
                sl
            }));
            out.fields.extend(s.fields.iter().cloned());
        }
        out
    }
}

/// A step that a feature applies to the built head for a drawn slider, a slider that is not a movement of the vertices but a shape
/// found on the head as it is built (the brow's edge, wherever each head has it). It moves `p`, the head after the smoothing and the
/// press, by `amount`. `ids` gives the head's vertex for each place in `p` (all of them for a whole head, fewer in the solver), and
/// `triangles` index `p`.
pub type DrawnStep = fn(&mut [V3], &[u32], &[[u32; 3]], f64);

/// The step of the drawn slider that sets `measure`, from the feature that owns it.
pub fn drawn_step(measure: &str) -> Option<DrawnStep> {
    crate::feature::all().iter().find_map(|f| f.drawn_step(measure))
}

/// The head mesh's triangles and the vertices the smoothing keeps, for the drawn sliders' steps when a whole head is built.
pub struct MeshInfo {
    pub triangles: Vec<[u32; 3]>,
    pub keep: Vec<bool>,
}

static MESH: std::sync::OnceLock<MeshInfo> = std::sync::OnceLock::new();

/// Records the head mesh for the drawn sliders (the first call wins). The tools' `World::load` and the page's `Builder` call it.
pub fn init_mesh(triangles: &[[u32; 3]], keep: &[bool]) {
    let _ = MESH.get_or_init(|| MeshInfo { triangles: triangles.to_vec(), keep: keep.to_vec() });
}

/// The vertices the smoothing keeps (empty before `init_mesh`).
pub fn kept_flags() -> Vec<bool> {
    MESH.get().map(|m| m.keep.clone()).unwrap_or_default()
}

/// Whether the smoothing keeps head vertex `v` still (false before `init_mesh`).
pub fn kept(v: u32) -> bool {
    MESH.get().is_some_and(|m| m.keep.get(v as usize).copied().unwrap_or(false))
}

static THROAT: std::sync::OnceLock<crate::chin_neck::ThroatShape> = std::sync::OnceLock::new();

/// Records the throat construction for sets with the throat's sliders (the first call wins).
pub fn init_throat(shape: crate::chin_neck::ThroatShape) {
    let _ = THROAT.set(shape);
}

static CURVE: std::sync::Mutex<Option<head_eye_curve::maker::Maker>> = std::sync::Mutex::new(None);

/// Records the merged head mesh for the eye opening's curve (`head_eye_curve`): each vertex's merged vertex, one vertex per merged
/// vertex and the hidden merged vertices. A set with the curve's sliders builds its openings on the curve only after this call.
pub fn init_curve(group: Vec<u32>, first: Vec<u32>, hidden: Vec<bool>) {
    let triangles = MESH.get().map(|m| m.triangles.clone()).unwrap_or_default();
    let mesh = head_eye_curve::maker::Mesh::new(triangles, group, first, hidden);
    *CURVE.lock().unwrap() = Some(head_eye_curve::maker::Maker::new(mesh));
}

/// Eye `e`'s lid of the last build on the curve: the turn that closes it and the curve's arches (`head_eye_curve::maker::Lid`).
pub fn with_curve_lid<T>(e: usize, f: impl FnOnce(&head_eye_curve::maker::Lid) -> T) -> Option<T> {
    CURVE.lock().unwrap().as_ref().and_then(|m| m.lids.get(e).map(f))
}

/// What `f` reads from the eye opening's curve maker (`head_eye_curve::maker::Maker`), after `init_curve`.
pub fn with_curve<T>(f: impl FnOnce(&head_eye_curve::maker::Maker) -> T) -> Option<T> {
    CURVE.lock().unwrap().as_ref().map(f)
}

/// The smoothing prepared for a head mesh.
pub fn smoother(base: &[V3], triangles: &[[u32; 3]], keep: &[bool]) -> Smoother {
    Smoother::new(base, triangles, keep, SMOOTHING_RAMP_M)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> SliderSet {
        let a = Slider::from_movement(
            "a",
            "shape",
            "1",
            &[[0.001, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.002, 0.0]],
        );
        let b = Slider::from_movement(
            "b",
            "shape",
            "1",
            &[[0.0, 0.0, 0.003], [0.001, 0.0, 0.0], [0.0, 0.0, 0.0]],
        );
        SliderSet {
            vertex_count: 3,
            sliders: vec![a, b],
            fields: Vec::new(),
        }
    }

    #[test]
    fn a_head_is_its_start_plus_its_values() {
        let s = set();
        let p = s.unsmoothed(&[[0.0; 3]; 3], &[2.0, -1.0]);
        let want = [[0.002, 0.0, -0.003], [-0.001, 0.0, 0.0], [0.0, 0.004, 0.0]];
        for i in 0..3 {
            for c in 0..3 {
                // the movements are stored as 32-bit numbers
                assert!((p[i][c] - want[i][c]).abs() < 1e-9, "{p:?}");
            }
        }
    }

    #[test]
    fn up_and_back_returns_the_same_head_in_any_order() {
        let s = set();
        let start = [[0.1, 0.2, 0.3]; 3];
        let one = s.unsmoothed(&s.unsmoothed(&start, &[1.5, 0.0]), &[0.0, -0.5]);
        let other = s.unsmoothed(&s.unsmoothed(&start, &[0.0, -0.5]), &[1.5, 0.0]);
        let together = s.unsmoothed(&start, &[1.5, -0.5]);
        for i in 0..3 {
            for c in 0..3 {
                assert!(
                    (one[i][c] - other[i][c]).abs() < 1e-12
                        && (one[i][c] - together[i][c]).abs() < 1e-12
                );
            }
        }
        let back = s.unsmoothed(&s.unsmoothed(&start, &[1.5, -0.5]), &[-1.5, 0.5]);
        for i in 0..3 {
            for c in 0..3 {
                assert!((back[i][c] - start[i][c]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn the_set_survives_its_plain_data_file() {
        let s = set();
        let text = serde_json::to_string(&s).unwrap();
        let again: SliderSet = serde_json::from_str(&text).unwrap();
        assert_eq!(again.sliders.len(), 2);
        assert_eq!(again.sliders[1].vertices, vec![0, 1]);
    }

    /// Two sliders made of keys over two shared fields, like the named nose sliders.
    fn keyed() -> SliderSet {
        let f0 = Field::from_movement(&[[0.001, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.002, 0.0]]);
        let f1 = Field::from_movement(&[[0.0, 0.0, 0.003], [0.001, 0.0, 0.0], [0.0, 0.0, 0.0]]);
        let key = |value: f64, w0: f32, w1: f32| Key { value, fields: vec![0, 1], weights: vec![w0, w1] };
        let mut a = Slider::from_movement("a", "named", "1", &[]);
        a.keys = vec![key(-2.0, -1.5, 0.2), key(0.0, 0.0, 0.0), key(1.0, 0.8, 0.1), key(3.0, 1.9, 0.6)];
        let mut b = Slider::from_movement("b", "named", "1", &[]);
        b.keys = vec![key(-1.0, 0.3, -1.2), key(0.0, 0.0, 0.0), key(2.0, -0.2, 2.5)];
        SliderSet { vertex_count: 3, sliders: vec![a, b], fields: vec![f0, f1] }
    }

    #[test]
    fn two_features_merged_build_the_same_head_in_any_order() {
        // two features, each with keyed sliders over its own fields, merged into one set
        let nose = keyed();
        let mut jaw = keyed();
        for f in &mut jaw.fields {
            for m in &mut f.moves {
                *m *= -0.5;
            }
        }
        let both = SliderSet::merge(&[nose.clone(), jaw.clone()]);
        assert_eq!(both.sliders.len(), 4);
        assert_eq!(both.fields.len(), 4);
        let start = [[0.1, 0.2, 0.3]; 3];
        let values = [0.4, -0.7, 2.1, 1.3];
        // the merged head is the nose's head with the jaw's movement added, and the other way round
        let a = jaw.unsmoothed(&nose.unsmoothed(&start, &values[..2]), &values[2..]);
        let b = nose.unsmoothed(&jaw.unsmoothed(&start, &values[2..]), &values[..2]);
        let c = both.unsmoothed(&start, &values);
        for i in 0..3 {
            for k in 0..3 {
                assert!((a[i][k] - c[i][k]).abs() < 1e-12 && (b[i][k] - c[i][k]).abs() < 1e-12);
            }
        }
        // edits of one feature then the other, in any order and through other values, give the same head
        let mut v = [0.0; 4];
        for (i, x) in [(2, 0.5), (0, 1.1), (3, 1.3), (0, 0.4), (1, -0.7), (2, 2.1)] {
            v[i] = x;
        }
        assert_eq!(both.unsmoothed(&start, &v), c);
        // and every feature moved up and back gives the start again
        assert_eq!(both.unsmoothed(&start, &[0.0; 4]), start.to_vec());
    }

    #[test]
    fn a_head_made_of_keys_depends_only_on_its_values() {
        let s = keyed();
        let start = [[0.1, 0.2, 0.3]; 3];
        // edits in two orders, through other values on the way, end on the same values and the same head
        let mut v = [0.0, 0.0];
        for (i, x) in [(0, 2.5), (1, -0.7), (0, 0.4), (1, 1.3)] {
            v[i] = x;
        }
        let mut w = [0.0, 0.0];
        for (i, x) in [(1, 1.3), (0, -1.9), (0, 0.4)] {
            w[i] = x;
        }
        assert_eq!(v, w);
        assert_eq!(s.unsmoothed(&start, &v), s.unsmoothed(&start, &w));
        // moving a slider up and back to 0 gives the start again
        assert_eq!(s.unsmoothed(&start, &[0.0, 0.0]), start.to_vec());
        // the movement is continuous at a key, and stays at the last key's past the end
        let near = |x: f64, y: f64| {
            let (p, q) = (s.unsmoothed(&start, &[x, 0.0]), s.unsmoothed(&start, &[y, 0.0]));
            p.iter().zip(&q).all(|(a, b)| (0..3).all(|c| (a[c] - b[c]).abs() < 1e-6))
        };
        assert!(near(1.0 - 1e-6, 1.0 + 1e-6));
        assert!(near(3.0, 7.0));
        assert!(!near(1.0, 2.0));
    }
}
