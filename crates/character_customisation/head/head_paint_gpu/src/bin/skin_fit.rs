//! The fit of the skin's colour to KOTOR's bakes (paint track, part C, skin colour round of 2026-11-20). Analysis only.
//!
//! `skin_fit <out dir> [name...] [--evals <scale>]`
//!
//! For each head it paints the live page's recipe on the GPU at 512 pixels and compares our paint with KOTOR's bake in the same texels of
//! our layout. KOTOR's bake is read at 2,048 pixels and averaged over each 4 by 4 block; a block counts only when all 16 pixels hold skin.
//! Hair on KOTOR's bake, the lips, the eye openings and the band of the brows are left out, and on a head with a beard or stubble the
//! whole beard area is left out too, so that the fit never takes a beard's shadow for skin colour.
//!
//! What is compared:
//! - the ramp: the face's texels sorted by lightness into 10 bins of equal count, each bin's mean Lab, KOTOR's against ours (ours sorted
//!   by our own lightness), so that the spread of lightness, the colour strength along it and the hue along it all count;
//! - the zones' lightness: each zone's mean L* minus the face's mean, and the middle of the face minus its sides;
//! - in the last pass, each zone's colour beyond the light: its mean a* and b* minus what the face's own ramp gives at the same L*.
//!
//! The passes, each a Nelder-Mead search:
//! A. the base tone (melanin, blood up to 0.08, colour strength, undertone), the shading (strength, highlight whiteness, light from the
//!    front), the sheen's three regions and "Lighter middle of the face", with the light sliders at their values today;
//! B. the four light sliders alone, per head;
//! C. everything of A and B together, with the light sliders held near the median of their tone group (dark: melanin above 0.15,
//!    medium: 0.05 to 0.15, pale: below 0.05);
//! D. the eight colour zones, from the colour beyond the light.
//!
//! It writes `<out dir>/fit.json`: per head, the values of each pass, the costs, the texel counts and whether a beard was masked.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;
#[path = "skin_shared/zones.rs"]
mod zones;

use fold_common::{Common, Lightness, RES, bake_path};
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint::pixel_map::PixelMap;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use zones::*;

const FIT: u32 = 512;
const BLOCK: usize = RES / FIT as usize;
/// The heads whose skin is not a human skin tone, which the fit leaves out: Bao-Dur's markings and the Sith apprentice's grey-violet.
pub const LEFT_OUT: [&str; 2] = ["p_baodurh", "sith_app"];
/// The women's heads, which are never treated as bearded: the hair detector fires on the noise of the 256-pixel textures, and no woman
/// of the set has a beard. The names follow `hq/viewer2/heads.js`.
const WOMEN: [&str; 20] = ["comm_a_f", "comm_a_f2", "comm_b_f", "comm_w_f", "old_a_f", "old_b_f", "old_w_f", "mod_exile_canon", "mod_hp_pfha01", "mod_hp_pfhb02", "mod_improved_bastila", "mod_jaesa", "mod_meetra", "mod_tcs_bastila", "n_daraalah", "n_dodonnah", "n_terenaadareh", "p_bastilah", "p_handmaidenbb", "p_kreiah"];

fn is_woman(name: &str) -> bool {
    WOMEN.contains(&name) || name.starts_with("pf")
}

/// The share of a beard area's skin texels that must be hair on KOTOR's bake for the head to count as bearded.
const BEARD_SHARE: f64 = 0.08;
/// The fewest texels a zone needs to count.
const ZONE_MIN: usize = 150;
/// The zones whose lightness the passes A to C compare (indices into `ZONES`): the cheeks, the nose, the forehead, round the eyes, round
/// the mouth, the chin, the jaw and the temples.
const LIGHT_ZONES: [usize; 8] = [0, 1, 3, 4, 5, 6, 7, 8];

/// One fitted number: its control id, its range, and whether it is searched in cube-root space.
#[derive(Clone, Copy)]
struct Param {
    id: &'static str,
    lo: f64,
    hi: f64,
    cube: bool,
}

const fn p(id: &'static str, lo: f64, hi: f64) -> Param {
    Param { id, lo, hi, cube: false }
}

const BASE: [Param; 11] = [
    Param { id: "melanin", lo: 0.002, hi: 0.45, cube: true },
    Param { id: "haemoglobin", lo: 0.0005, hi: 0.08, cube: true },
    p("chroma_scale", 0.1, 1.6),
    p("undertone_degrees", -30.0, 30.0),
    p("shading_strength", 0.0, 3.0),
    p("highlight_whiteness", 0.0, 0.6),
    p("sheen_cheeks", 0.0, 15.0),
    p("sheen_nose", 0.0, 15.0),
    p("sheen_chin", 0.0, 10.0),
    p("skin_middle_light", 0.0, 1.5),
    p("skin_front_light", 0.0, 1.5),
];
/// The highlight whiteness and the sheen are searched below their sliders' ends (0.6, and 15, 15 and 10 L*), because the first fit's
/// white highlights on the nose and the chin read as chalk to the judge.
const LIGHT: [Param; 4] = [p("skin_linear_shade", 0.0, 1.0), p("skin_red_shade", 0.0, 1.0), p("skin_warm_light", 0.0, 1.0), p("skin_strong_white", 0.0, 1.0)];
/// The zones are searched in narrower ranges than their sliders: a zone of blood from -0.5 to 1.5, and the yellower forehead and the
/// darker skin round the eyes and the mouth from 0 to 1.5, so that a zone cannot stand in for the light (KOTOR's skin round the eyes is
/// lighter than the face's mean because of its painted light, not its pigment).
const ZONE_SLIDERS: [Param; 8] = [
    p("skin_cheeks_blood", -0.5, 1.5),
    p("skin_nose_blood", -0.5, 1.5),
    p("skin_ears_blood", -0.5, 1.5),
    p("skin_lips_blood", -0.5, 1.5),
    p("skin_chin_blood", -0.5, 1.5),
    p("skin_forehead_yellow", 0.0, 1.5),
    p("skin_eyes_dark", 0.0, 1.5),
    p("skin_mouth_dark", 0.0, 1.5),
];

impl Param {
    fn to_unit(&self, v: f64) -> f64 {
        let v = v.clamp(self.lo, self.hi);
        if self.cube { (v.cbrt() - self.lo.cbrt()) / (self.hi.cbrt() - self.lo.cbrt()) } else { (v - self.lo) / (self.hi - self.lo) }
    }
    fn from_unit(&self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        if self.cube { (self.lo.cbrt() + u * (self.hi.cbrt() - self.lo.cbrt())).powi(3) } else { self.lo + u * (self.hi - self.lo) }
    }
}

/// What is measured on one paint, ours or KOTOR's, over the same texels.
#[derive(Clone, Debug)]
struct Stats {
    bins: [[f64; 3]; 10],
    /// Each zone's mean L* minus the face's, with `NaN` where the zone has too few texels.
    zone_dl: [f64; 10],
    middle_minus_sides: f64,
    /// Each zone's a* and b* beyond the light.
    beyond: [[f64; 2]; 10],
    face_mean: [f64; 3],
}

/// The texels of one head that are compared, and KOTOR's numbers on them.
struct Target {
    texels: Vec<usize>,
    kotor: Vec<[f64; 3]>,
    face: Vec<bool>,
    zone: Vec<Option<usize>>,
    side: Vec<Option<bool>>,
    k: Stats,
    beard: bool,
    zone_count: [usize; 10],
}

fn stats(t: &Target, lab: &[[f64; 3]]) -> Stats {
    let mut face: Vec<[f64; 3]> = Vec::new();
    for (k, c) in lab.iter().enumerate() {
        if t.face[k] {
            face.push(*c);
        }
    }
    let mean_f = mean(&face);
    let mut sorted = face.clone();
    sorted.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let n = sorted.len().max(10);
    let mut bins = [[0.0; 3]; 10];
    for (b, bin) in bins.iter_mut().enumerate() {
        let part = &sorted[(b * n / 10).min(sorted.len().saturating_sub(1))..((b + 1) * n / 10).min(sorted.len()).max((b * n / 10 + 1).min(sorted.len()))];
        *bin = mean(part);
    }
    let ramp = RampFn::new(&face);
    let mut zone_sum = [[0.0f64; 3]; 10];
    let mut zone_beyond = [[0.0f64; 2]; 10];
    let mut zone_n = [0usize; 10];
    let mut sides = [[0.0f64; 2]; 2];
    for (k, c) in lab.iter().enumerate() {
        if let Some(z) = t.zone[k] {
            zone_n[z] += 1;
            for i in 0..3 {
                zone_sum[z][i] += c[i];
            }
            let r = ramp.at(c[0]);
            zone_beyond[z][0] += c[1] - r[0];
            zone_beyond[z][1] += c[2] - r[1];
        }
        if let Some(m) = t.side[k] {
            let s = usize::from(!m);
            sides[s][0] += c[0];
            sides[s][1] += 1.0;
        }
    }
    let mut zone_dl = [f64::NAN; 10];
    let mut beyond = [[f64::NAN; 2]; 10];
    for z in 0..10 {
        if zone_n[z] >= ZONE_MIN {
            zone_dl[z] = zone_sum[z][0] / zone_n[z] as f64 - mean_f[0];
            beyond[z] = [zone_beyond[z][0] / zone_n[z] as f64, zone_beyond[z][1] / zone_n[z] as f64];
        }
    }
    let mms = if sides[0][1] > 0.0 && sides[1][1] > 0.0 { sides[0][0] / sides[0][1] - sides[1][0] / sides[1][1] } else { 0.0 };
    Stats { bins, zone_dl, middle_minus_sides: mms, beyond, face_mean: mean_f }
}

/// The cost of the base tone, the shading and the light: the ramp without its darkest and lightest tenth, the face's mean lightness, then
/// the zones' lightness at half the weight. The colour of the ramp counts most in its lit half: KOTOR's skin reads warm from its lit
/// parts, and its shade is nearly colourless because it is dark. The first fit, which counted every bin's colour alike and the darkest
/// tenth too, greyed the warm and dark skins to reach that shade.
fn cost_light(t: &Target, o: &Stats) -> f64 {
    let mut ramp = 0.0;
    for b in 1..9 {
        let (k, s) = (t.k.bins[b], o.bins[b]);
        let w = if b >= 5 { 3.0 } else { 0.5 };
        ramp += (k[0] - s[0]).powi(2) + w * ((k[1] - s[1]).powi(2) + (k[2] - s[2]).powi(2));
    }
    ramp /= 8.0;
    ramp += (t.k.face_mean[0] - o.face_mean[0]).powi(2);
    let mut zones = 0.0;
    let mut n = 0.0;
    for &z in &LIGHT_ZONES {
        if t.k.zone_dl[z].is_finite() && o.zone_dl[z].is_finite() {
            zones += (t.k.zone_dl[z] - o.zone_dl[z]).powi(2);
            n += 1.0;
        }
    }
    zones = zones / f64::max(n, 1.0) + (t.k.middle_minus_sides - o.middle_minus_sides).powi(2);
    ramp + 0.5 * zones
}

/// The cost of the zones: each zone's colour beyond the light, and its lightness at a tenth of the weight.
fn cost_zones(t: &Target, o: &Stats) -> f64 {
    let mut c = 0.0;
    for z in 0..9 {
        let (k, s) = (t.k.beyond[z], o.beyond[z]);
        if k[0].is_finite() && s[0].is_finite() {
            let w = (t.zone_count[z] as f64 / 2000.0).sqrt().min(1.0);
            c += w * ((k[0] - s[0]).powi(2) + (k[1] - s[1]).powi(2) + 0.1 * (t.k.zone_dl[z] - o.zone_dl[z]).powi(2));
        }
    }
    c
}

struct Fitter {
    painter: GpuPainter,
    list: Vec<head_paint::controls::Control>,
    evals: usize,
}

impl Fitter {
    fn index(&self, id: &str) -> usize {
        self.list.iter().position(|c| c.id == id).unwrap_or_else(|| panic!("no control {id}"))
    }

    fn paint(&mut self, t: &Target, values: &[f64]) -> Stats {
        let mut switches = vec![true; 9];
        switches[8] = false;
        self.painter.paint(FIT, &recipe_from(values), &layers_from(&switches));
        let rgba = pollster::block_on(self.painter.read_rgba(FIT));
        self.evals += 1;
        let lab: Vec<[f64; 3]> = t.texels.iter().map(|&i| lab_of([rgba[4 * i], rgba[4 * i + 1], rgba[4 * i + 2]])).collect();
        stats(t, &lab)
    }

    /// A Nelder-Mead search over `params` from `values`, in the unit cube of each parameter's range. `cost` adds to the paint's cost
    /// whatever penalty it wants from the unit values. Returns the best values and their cost.
    fn search(&mut self, t: &Target, values: &[f64], params: &[Param], max: usize, cost: &dyn Fn(&Stats, &[f64]) -> f64) -> (Vec<f64>, f64) {
        let idx: Vec<usize> = params.iter().map(|p| self.index(p.id)).collect();
        let n = params.len();
        let start: Vec<f64> = params.iter().zip(&idx).map(|(p, &i)| p.to_unit(values[i])).collect();
        let mut eval = |u: &[f64], me: &mut Self| -> f64 {
            let mut v = values.to_vec();
            for k in 0..n {
                v[idx[k]] = params[k].from_unit(u[k]);
            }
            let s = me.paint(t, &v);
            cost(&s, u)
        };
        let mut simplex: Vec<(Vec<f64>, f64)> = Vec::with_capacity(n + 1);
        let c0 = eval(&start, self);
        simplex.push((start.clone(), c0));
        for k in 0..n {
            let mut u = start.clone();
            u[k] = if u[k] > 0.85 { u[k] - 0.15 } else { u[k] + 0.15 };
            let c = eval(&u, self);
            simplex.push((u, c));
        }
        let mut used = n + 1;
        let clamp = |u: Vec<f64>| -> Vec<f64> { u.into_iter().map(|x| x.clamp(0.0, 1.0)).collect() };
        while used < max {
            simplex.sort_by(|a, b| a.1.total_cmp(&b.1));
            if (simplex[n].1 - simplex[0].1).abs() < 1e-4 * (1.0 + simplex[0].1.abs()) {
                break;
            }
            let centre: Vec<f64> = (0..n).map(|k| simplex[..n].iter().map(|s| s.0[k]).sum::<f64>() / n as f64).collect();
            let worst = simplex[n].0.clone();
            let along = |t: f64| -> Vec<f64> { clamp((0..n).map(|k| centre[k] + t * (worst[k] - centre[k])).collect()) };
            let r = along(-1.0);
            let cr = eval(&r, self);
            used += 1;
            if cr < simplex[0].1 {
                let e = along(-2.0);
                let ce = eval(&e, self);
                used += 1;
                simplex[n] = if ce < cr { (e, ce) } else { (r, cr) };
            } else if cr < simplex[n - 1].1 {
                simplex[n] = (r, cr);
            } else {
                let c = along(if cr < simplex[n].1 { -0.5 } else { 0.5 });
                let cc = eval(&c, self);
                used += 1;
                if cc < simplex[n].1.min(cr) {
                    simplex[n] = (c, cc);
                } else {
                    let best = simplex[0].0.clone();
                    for s in simplex.iter_mut().skip(1) {
                        s.0 = clamp((0..n).map(|k| best[k] + 0.5 * (s.0[k] - best[k])).collect());
                        s.1 = eval(&s.0, self);
                        used += 1;
                    }
                }
            }
        }
        simplex.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut v = values.to_vec();
        for k in 0..n {
            v[idx[k]] = params[k].from_unit(simplex[0].0[k]);
        }
        (v, simplex[0].1)
    }
}

fn target(name: &str, common: &Common, map: &PixelMap, painter: &GpuPainter, pos: &[[f64; 3]]) -> Target {
    let detail = painter.head().and_then(|h| h.detail.as_ref()).expect("detail");
    let lm = detail.folds.as_ref().expect("folds").landmarks;
    let mm: Vec<[f64; 3]> = pos.iter().map(|p| p.map(|x| 1000.0 * x)).collect();
    let zones = Zones::new(lm, detail.landmarks, &mm);
    let bake_img = image::open(bake_path(name)).expect("bake").to_rgb8();
    let bake = Lightness::from_rgb(&bake_img);
    let hair = bake.hair();
    let layout_mm: Vec<[f64; 3]> = common.vmap.iter().map(|&m| mm[m]).collect();
    // The points of the texels at 512 pixels, and whether the 2,048-pixel block under each holds skin.
    let mut raw = Vec::new();
    let (mut beard_skin, mut beard_hair) = (0usize, 0usize);
    for px in &map.pixels {
        let t = common.tris[px.triangle as usize];
        let p = [0, 1, 2].map(|k| (0..3).map(|c| px.barycentric[c] * layout_mm[t[c] as usize][k]).sum::<f64>());
        if zones.excluded(p) {
            continue;
        }
        let (x0, y0) = (px.x as usize * BLOCK, px.y as usize * BLOCK);
        let mut ok = true;
        let mut any_hair = false;
        let mut sum = [0.0f64; 3];
        for dy in 0..BLOCK {
            for dx in 0..BLOCK {
                let i = (y0 + dy) * RES + x0 + dx;
                if !bake.valid[i] {
                    ok = false;
                } else if hair[i] {
                    any_hair = true;
                }
                let c = bake_img.get_pixel((x0 + dx) as u32, (y0 + dy) as u32).0;
                for k in 0..3 {
                    sum[k] += f64::from(c[k]);
                }
            }
        }
        let is_beard = zones.beard(p);
        if ok && is_beard {
            if any_hair {
                beard_hair += 1;
            } else {
                beard_skin += 1;
            }
        }
        if ok && !any_hair {
            let n = (BLOCK * BLOCK) as f64;
            let rgb = sum.map(|s| (s / n).round() as u8);
            raw.push((px.y as usize * FIT as usize + px.x as usize, p, lab_of(rgb), is_beard));
        }
    }
    let beard = !is_woman(name) && beard_hair as f64 / (beard_hair + beard_skin).max(1) as f64 > BEARD_SHARE;
    let mut t = Target { texels: vec![], kotor: vec![], face: vec![], zone: vec![], side: vec![], k: Stats { bins: [[0.0; 3]; 10], zone_dl: [0.0; 10], middle_minus_sides: 0.0, beyond: [[0.0; 2]; 10], face_mean: [0.0; 3] }, beard, zone_count: [0; 10] };
    for (i, p, lab, is_beard) in raw {
        if beard && is_beard {
            continue;
        }
        let z = zones.zone(p);
        t.texels.push(i);
        t.kotor.push(lab);
        t.face.push(zones.face(p) && z != Some(2));
        t.zone.push(z);
        t.side.push(if z == Some(2) { None } else { zones.middle_or_side(p) });
        if let Some(z) = z {
            t.zone_count[z] += 1;
        }
    }
    t.k = stats(&t, &t.kotor);
    t
}

fn parse(args: &mut Vec<String>, flag: &str) -> Vec<String> {
    let mut out = Vec::new();
    while let Some(k) = args.iter().position(|a| a == flag) {
        out.push(args.remove(k + 1));
        args.remove(k);
    }
    out
}

fn r4(x: f64) -> f64 {
    (x * 10000.0).round() / 10000.0
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let scale: f64 = parse(&mut args, "--evals").pop().map_or(1.0, |s| s.parse().expect("number"));
    if args.is_empty() {
        eprintln!("Usage: skin_fit <out dir> [name...] [--evals scale]");
        std::process::exit(2);
    }
    let out = PathBuf::from(&args[0]);
    std::fs::create_dir_all(&out).expect("out");
    let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
    let mut names: Vec<String> = args[1..].to_vec();
    if names.is_empty() {
        names = recipes["heads"].as_object().expect("heads").keys().cloned().collect();
        names.sort();
    }
    names.retain(|n| !LEFT_OUT.contains(&n.as_str()));
    let lay = layout();
    let common = Common::load();
    assert_eq!(common.uv, lay.uv, "the live page's layout differs from the bundle's");
    let map = PixelMap::new(&common.uv, &common.tris, FIT as usize);
    let painter = GpuPainter::new(Gpu::blocking().expect("GPU"), lay);
    let list = controls();
    let mut f = Fitter { painter, list, evals: 0 };
    let n_evals = |base: usize| ((base as f64) * scale) as usize;

    struct Head {
        name: String,
        target: Target,
        values: Vec<f64>,
        before: f64,
        a: (Vec<f64>, f64),
        b: Option<(Vec<f64>, f64)>,
        c: Option<(Vec<f64>, f64)>,
        d: Option<(Vec<f64>, f64)>,
    }
    let mut heads: Vec<Head> = Vec::new();
    let t0 = std::time::Instant::now();
    // Pass A, then B, per head. The head is prepared again for each pass, so the targets are kept.
    for name in &names {
        let pos = positions(name);
        f.painter.set_head(pos.clone());
        pollster::block_on(f.painter.prepare(FIT));
        let target = target(name, &common, &map, &f.painter, &pos);
        let head = &recipes["heads"][name.as_str()];
        let values: Vec<f64> = f.list.iter().map(|c| head[&c.id].as_f64().unwrap_or(c.default)).collect();
        let before = cost_light(&target, &f.paint(&target, &values));
        let a = f.search(&target, &values, &BASE, n_evals(260), &|s, _| cost_light(&target, s));
        let b = f.search(&target, &a.0, &LIGHT, n_evals(70), &|s, _| cost_light(&target, s));
        println!("{name}: {} texels, beard {}; cost {before:.1} -> A {:.1} -> B {:.1} ({} evals, {:.0} s)", target.texels.len(), target.beard, a.1, b.1, f.evals, t0.elapsed().as_secs_f64());
        heads.push(Head { name: name.clone(), target, values, before, a, b: Some(b), c: None, d: None });
    }
    // The tone groups and their medians of the light sliders after pass B.
    let mel = f.index("melanin");
    let group = |v: &[f64]| if v[mel] > 0.15 { "dark" } else if v[mel] > 0.05 { "medium" } else { "pale" };
    let mut medians: std::collections::BTreeMap<&str, Vec<f64>> = Default::default();
    for g in ["dark", "medium", "pale"] {
        let rows: Vec<&Vec<f64>> = heads.iter().filter(|h| group(&h.a.0) == g).map(|h| &h.b.as_ref().unwrap().0).collect();
        let med: Vec<f64> = LIGHT
            .iter()
            .map(|p| {
                let i = f.index(p.id);
                let mut v: Vec<f64> = rows.iter().map(|r| r[i]).collect();
                v.sort_by(f64::total_cmp);
                if v.is_empty() { 0.0 } else { v[v.len() / 2] }
            })
            .collect();
        println!("group {g}: {} heads, light sliders median {:?}", rows.len(), med.iter().map(|x| r4(*x)).collect::<Vec<_>>());
        medians.insert(g, med);
    }
    // Pass C and D per head.
    let all: Vec<Param> = BASE.iter().chain(LIGHT.iter()).copied().collect();
    for h in heads.iter_mut() {
        let pos = positions(&h.name);
        f.painter.set_head(pos);
        pollster::block_on(f.painter.prepare(FIT));
        let med = medians[group(&h.a.0)].clone();
        let mut start = h.a.0.clone();
        for (k, p) in LIGHT.iter().enumerate() {
            start[f.index(p.id)] = med[k];
        }
        let target = &h.target;
        let pull = |s: &Stats, u: &[f64]| {
            let pen: f64 = (0..LIGHT.len()).map(|k| (u[BASE.len() + k] - LIGHT[k].to_unit(med[k])).powi(2)).sum();
            cost_light(target, s) + 40.0 * pen
        };
        // Two searches, the second from the first's best with a fresh simplex, because one search over 14 numbers often stops early.
        let c1 = f.search(target, &start, &all, n_evals(300), &pull);
        let c = f.search(target, &c1.0, &all, n_evals(250), &pull);
        let d = f.search(target, &c.0, &ZONE_SLIDERS, n_evals(160), &|s, u| {
            // A small pull toward zero.
            let pen: f64 = u.iter().zip(&ZONE_SLIDERS).map(|(x, p)| (x - p.to_unit(0.0)).powi(2)).sum();
            cost_zones(target, s) + 2.0 * pen
        });
        let check = cost_light(target, &f.paint(target, &d.0));
        println!("{}: C {:.1}, zones {:.1}, light after zones {check:.1} ({} evals, {:.0} s)", h.name, c.1, d.1, f.evals, t0.elapsed().as_secs_f64());
        h.c = Some(c);
        h.d = Some(d);
    }
    // The report.
    let ids: Vec<&str> = BASE.iter().chain(LIGHT.iter()).chain(ZONE_SLIDERS.iter()).map(|p| p.id).collect();
    let pick = |v: &[f64]| -> Value { Value::Object(ids.iter().map(|id| (id.to_string(), json!(r4(v[f.index(id)])))).collect()) };
    let report: Vec<Value> = heads
        .iter()
        .map(|h| {
            let d = h.d.as_ref().unwrap();
            json!({
                "name": h.name, "beard_masked": h.target.beard, "texels": h.target.texels.len(), "zone_texels": h.target.zone_count.to_vec(),
                "group": group(&h.a.0),
                "cost": { "before": r4(h.before), "a": r4(h.a.1), "b": r4(h.b.as_ref().unwrap().1), "c": r4(h.c.as_ref().unwrap().1), "zones": r4(d.1) },
                "before": pick(&h.values), "a": pick(&h.a.0), "final": pick(&d.0),
            })
        })
        .collect();
    let groups: Value = Value::Object(medians.iter().map(|(g, m)| (g.to_string(), json!(m.iter().map(|x| r4(*x)).collect::<Vec<_>>()))).collect());
    std::fs::write(out.join("fit.json"), serde_json::to_string_pretty(&json!({ "left_out": LEFT_OUT, "light_medians": groups, "heads": report })).unwrap()).expect("write");
    println!("{} evaluations in {:.0} s", f.evals, t0.elapsed().as_secs_f64());
}
