//! The KOTOR test heads as the tools read them (native builds only): each head's frame, KOTOR's own mesh and its measured feature, and
//! the stand-in face the sliders start from.

use crate::feature::{Feature, Guard, KotorExtras, Measured};
use crate::measure::{Frame, Surface};
use crate::mesh::HeadMesh;
use crate::mesh::input::Input;
use crate::set::{SMOOTHING, smoother};
use crate::zone::Zone;
use crate::{V3, sub};
use head_shape::smooth::Smoother;

pub struct Head {
    pub name: String,
    pub title: String,
    pub is_mod: bool,
    pub frame: Frame,
    pub eyes: [V3; 2],
    /// KOTOR's own face mesh, in the fit's frame (metres).
    pub kotor_v: Vec<V3>,
    pub kotor_f: Vec<[u32; 3]>,
    /// Which of KOTOR's vertices are skin, when the inputs say.
    pub kotor_skin: Option<Vec<bool>>,
    /// The feature measured on KOTOR's own mesh.
    pub kotor: Measured,
    /// Why the measure failed on KOTOR's mesh, when it did. Such a head is left out of everything that needs its target.
    pub kotor_error: Option<&'static str>,
    /// The stand-in face before smoothing: the fit with the feature's own shape taken out.
    pub start: Vec<V3>,
    /// The stand-in's feature, measured after the smoothing.
    pub start_nose: Option<Measured>,
}

/// Everything a feature's tools share: the head mesh, its smoothing, the base head's feature, the feature's zone and the heads.
pub struct World {
    pub feature: &'static dyn Feature,
    pub mesh: HeadMesh,
    pub smoother: Smoother,
    pub base_nose: Measured,
    pub zone: Zone,
    pub heads: Vec<Head>,
    /// The vertices of each guarded group, when the inputs hold them.
    pub guards: Vec<(Guard, Vec<bool>)>,
}

/// An unreadable target for a head whose KOTOR mesh the measure could not read at all.
fn unreadable(feature: &dyn Feature) -> Measured {
    Measured {
        proportions: vec![f64::NAN; feature.proportions().len()],
        outlines: vec![Vec::new(); feature.outlines().len()],
        marks: serde_json::Value::Null,
        cuts: Vec::new(),
    }
}

impl World {
    pub fn load(input: &Input, feature: &'static dyn Feature) -> Self {
        let mesh = input.head_mesh();
        let sm = smoother(&mesh.base, &mesh.triangles, &mesh.keep);
        crate::eye_curves::init(&mesh.base, &mesh.triangles);
        crate::set::init_mesh(&mesh.triangles, &mesh.keep);
        let base_frame = Frame::from_eyes(mesh.eyes);
        let base_nose = feature
            .measure(&Surface::within(&sm.smooth(&mesh.base, SMOOTHING), &mesh.triangles, &base_frame, &feature.window()))
            .expect("the base head's feature");
        let zone = feature.zone(&mesh, &base_nose);
        let base_merged = mesh.merge(&mesh.base);
        let names: Vec<(String, String, bool)> = input.manifest["heads"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| {
                (h["name"].as_str().unwrap().to_string(), h["title"].as_str().unwrap_or("").to_string(), h["mod"].as_bool().unwrap_or(false))
            })
            .collect();
        let window = feature.window();
        let heads = parallel(names.len(), threads(), |i| {
            let (name, title, is_mod) = names[i].clone();
            let eyes = input.points(&format!("heads/{name}/eyes"));
            let frame = crate::measure::head_frame([eyes[0], eyes[1]]);
            let kotor_v = input.points(&format!("heads/{name}/kotor_v"));
            let kotor_f = input.triangles(&format!("heads/{name}/kotor_f"));
            let skin: Option<Vec<bool>> =
                input.has(&format!("extra/heads/{name}/kotor_skin")).then(|| input.u8s(&format!("extra/heads/{name}/kotor_skin")).iter().map(|&s| s != 0).collect());
            let colour: Option<Vec<V3>> =
                input.has(&format!("extra/heads/{name}/kotor_colour")).then(|| input.points(&format!("extra/heads/{name}/kotor_colour")));
            let fit: Option<Vec<V3>> = input.has(&format!("heads/{name}/fit_smooth")).then(|| input.points(&format!("heads/{name}/fit_smooth")));
            let extras = KotorExtras {
                name: Some(&name),
                triangles: Some(&kotor_f),
                fit_triangles: Some(&mesh.triangles),
                skin: skin.as_deref(),
                colour: colour.as_deref(),
                fit: fit.as_deref(),
            };
            let surface = Surface::within(&kotor_v, &kotor_f, &frame, &window);
            let (kotor, kotor_error) = match feature.measure_kotor(&surface, &extras, &frame, &kotor_v) {
                Ok(m) => (m, None),
                Err(e) => (unreadable(feature), Some(e)),
            };
            let pre = input.points(&format!("heads/{name}/standin_pre"));
            let moved: Vec<V3> = mesh.merge(&pre).iter().zip(&base_merged).map(|(p, b)| sub(*p, *b)).collect();
            let own = mesh.expand(&zone.own_shape(&mesh, &moved));
            let mut start: Vec<V3> = pre.iter().zip(&own).map(|(p, o)| sub(*p, *o)).collect();
            // the brow and the cheeks border the eye area, so they are solved on the stand-ins the page shows
            if matches!(feature.key(), "eyes" | "brow" | "cheeks") && crate::eyes::paste_on() {
                start = crate::eyes::paste_start_mesh(&mesh, &start);
            }
            let start_nose = if feature.key() == "eyes" {
                // the stand-in as the page shows it at 0: pressed, with the lid pieces
                let mut p = sm.smooth(&start, SMOOTHING);
                crate::eye_curves::press(&mut p, &start);
                feature.measure(&crate::eyes::shown_surface(&p, &mesh.triangles, &frame, [0.0; 2])).ok()
            } else {
                feature.measure(&Surface::within(&sm.smooth(&start, SMOOTHING), &mesh.triangles, &frame, &window)).ok()
            };
            Head { name, title, is_mod, frame, eyes: [eyes[0], eyes[1]], kotor_v, kotor_f, kotor_skin: skin, kotor, kotor_error, start, start_nose }
        });
        let guards = feature
            .guards()
            .iter()
            .filter_map(|&g| guard_mask(input, &mesh, g).map(|m| (g, m)))
            .collect();
        Self { feature, mesh, smoother: sm, base_nose, zone, heads, guards }
    }

    /// Each head's need for proportion `k`: KOTOR's value minus the stand-in's, where both can be read.
    pub fn needs(&self, k: usize) -> Vec<f64> {
        self.heads
            .iter()
            .filter_map(|h| h.start_nose.as_ref().map(|s| h.kotor.proportions[k] - s.proportions[k]))
            .filter(|d| !d.is_nan())
            .collect()
    }
}

/// The vertices of a guarded group, from the extra inputs.
fn guard_mask(input: &Input, mesh: &HeadMesh, g: Guard) -> Option<Vec<bool>> {
    let n = mesh.base.len();
    let mask = |name: &str| -> Option<Vec<bool>> { input.has(name).then(|| input.u8s(name).iter().map(|&v| v != 0).collect()) };
    match g {
        Guard::Rim => {
            let rim = input.has("extra/rim").then(|| input.u32s("extra/rim"))?;
            let below = mask("extra/below")?;
            let mut groups = vec![false; mesh.merged_count()];
            for &v in &rim {
                groups[mesh.group[v as usize] as usize] = true;
            }
            Some((0..n).map(|i| below[i] || groups[mesh.group[i] as usize]).collect())
        }
        Guard::LipLine => mask("extra/lip_line"),
        Guard::MouthBag => mask("extra/mouth_bag"),
        Guard::Ears => mask("extra/ears"),
        Guard::Lips => {
            let mouth = 1 + crate::mesh::ZONES.iter().position(|z| *z == "mouth")? as u8;
            Some(mesh.zone.iter().map(|&z| z == mouth).collect())
        }
        Guard::Lids => {
            let text = std::fs::read_to_string(input.dir.parent()?.join("lid_edges.json")).ok()?;
            let lids: serde_json::Value = serde_json::from_str(&text).ok()?;
            let mut groups = vec![false; mesh.merged_count()];
            for eye in lids["eyes"].as_object()?.values() {
                for key in ["edge", "eyeball_centre_vertices"] {
                    for v in eye[key].as_array()? {
                        groups[mesh.group[v.as_u64()? as usize] as usize] = true;
                    }
                }
            }
            Some((0..n).map(|i| groups[mesh.group[i] as usize]).collect())
        }
    }
}

pub fn threads() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
}

/// Runs `f` for 0..n on `threads` threads and returns the results in order.
pub fn parallel<T: Send>(n: usize, threads: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<Option<T>>> = (0..n).map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..threads.min(n.max(1)) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if i >= n {
                        break;
                    }
                    *out[i].lock().unwrap() = Some(f(i));
                }
            });
        }
    });
    out.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect()
}

/// The spread of the values that are not NaN.
pub fn sd(v: &[f64]) -> f64 {
    let v: Vec<f64> = v.iter().copied().filter(|x| !x.is_nan()).collect();
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
}
