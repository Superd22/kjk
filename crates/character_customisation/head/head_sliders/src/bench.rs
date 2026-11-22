//! The kit's commands for one feature (native builds only): its marks and checks, what each head needs, the named sliders, a clay sheet
//! of their ends, and the bench (the 57 heads solved, the scorecard, the caps, the safety checks, the leak table and the page's data).
//! `sliders <feature> all` runs them in that order.

use crate::check::{Checker, Faults};
use crate::checks::{Marks, check1_and_marks, check3_and_4};
use crate::feature::{Feature, Guard, Scale, Shot, proportion_index};
use crate::heads::{Head, World, parallel, threads};
use crate::maker::{Maker, bending, bumps, fields};
use crate::measure::Frame;
use crate::mesh::input::Input;
use crate::render::{clay, label, sheet};
use crate::set::{SMOOTHING, SliderSet};
use crate::solve::{Evaluator, Smoothed, solve_within};
use crate::verdict::judge;
use crate::{V3, length, sub};
use image::RgbImage;
use serde_json::{Value, json};
use std::path::Path;
use std::time::Instant;

pub const COUNTS: [usize; 6] = [4, 6, 8, 10, 12, 14];
const PRIOR: f64 = 0.01;

fn write(data: &Path, rel: &str, text: String) {
    let path = data.join(rel);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(&path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

fn write_bytes(data: &Path, rel: &str, bytes: Vec<u8>) {
    let path = data.join(rel);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(&path, bytes).unwrap();
}

/// How a slider holds the other measures on our own smooth head: a quarter of the KOTOR heads' spread. The pass marks also cover how
/// much the measure wobbles on KOTOR's coarse meshes, which our head does not have.
fn hold_scale(m: &Marks) -> Scale {
    Scale {
        proportions: m.spread_p.iter().map(|s| 0.25 * s).collect(),
        outlines: m.spread_o.iter().map(|s| 0.25 * s).collect(),
        references: m.marks.references.clone(),
    }
}

fn percentile(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    if v.is_empty() {
        return f64::NAN;
    }
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// Check 1 and the marks, written to the feature's marks file, and checks 3 and 4 to its checks file.
pub fn marks(world: &World, input: &Input, data: &Path) -> Marks {
    let t0 = Instant::now();
    let (check1, m) = check1_and_marks(world, input);
    write(data, &world.feature.files().marks, serde_json::to_string_pretty(&m.to_json()).unwrap());
    let rest = check3_and_4(world, input, &m.marks);
    let unreadable: Vec<Value> = world
        .heads
        .iter()
        .filter_map(|h| {
            let rows: Vec<&str> = world
                .feature
                .proportions()
                .iter()
                .map(|p| p.key)
                .chain(world.feature.outlines().iter().map(|o| o.key))
                .enumerate()
                .filter(|(r, _)| !h.kotor.readable(*r))
                .map(|(_, k)| k)
                .collect();
            (!rows.is_empty()).then(|| json!({"name": h.name, "rows": rows, "error": h.kotor_error}))
        })
        .collect();
    write(
        data,
        &world.feature.files().checks,
        serde_json::to_string_pretty(&json!({"check1": check1, "unreadable": unreadable, "checks": rest, "marks": m.to_json()})).unwrap(),
    );
    eprintln!("marks and checks in {:.1?}", t0.elapsed());
    print_check1(world, &check1, &m);
    m
}

fn print_check1(world: &World, check1: &Value, m: &Marks) {
    let f = world.feature;
    let names: Vec<&str> = f.proportions().iter().map(|p| p.key).chain(f.outlines().iter().map(|o| o.key)).collect();
    for row in check1["summary"].as_array().unwrap() {
        let stats = row["stats"].as_array().unwrap();
        let cells: Vec<String> = names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{n} {:.2}/{:.2}", stats[i]["median"].as_f64().unwrap_or(f64::NAN), stats[i]["p95"].as_f64().unwrap_or(f64::NAN)))
            .collect();
        println!("{:10} {}", row["what"].as_str().unwrap(), cells.join("  "));
    }
    let spreads: Vec<f64> = m.spread_p.iter().chain(&m.spread_o).copied().collect();
    println!("spread     {}", names.iter().zip(&spreads).map(|(n, s)| format!("{n} {s:.2}")).collect::<Vec<_>>().join("  "));
    let marks: Vec<f64> = m.marks.proportions.iter().chain(&m.marks.outlines).copied().collect();
    println!("marks      {}", names.iter().zip(&marks).map(|(n, s)| format!("{n} {s:.2}")).collect::<Vec<_>>().join("  "));
}

/// Per measure: the base head's value, KOTOR's spread, and how far each KOTOR head lies from its own stand-in.
pub fn needs(world: &World) {
    let f = world.feature;
    println!("{:16} {:>7} {:>7} | {:>7} {:>7} {:>7} {:>7} | {:>7} {:>7} {:>7}", "measure", "base", "spread", "k-base lo", "hi", "p5", "p95", "need lo", "hi", "p5/p95");
    for (i, p) in f.proportions().iter().enumerate() {
        let kb: Vec<f64> =
            world.heads.iter().map(|h| h.kotor.proportions[i] - world.base_nose.proportions[i]).filter(|x| !x.is_nan()).collect();
        let ks = world.needs(i);
        let spread = crate::heads::sd(&world.heads.iter().map(|h| h.kotor.proportions[i]).collect::<Vec<_>>());
        let (mut a, mut b) = (kb.clone(), ks.clone());
        println!(
            "{:16} {:7.2} {:7.2} | {:7.2} {:7.2} {:7.2} {:7.2} | {:7.2} {:7.2} {:7.2}/{:.2}",
            p.key,
            world.base_nose.proportions[i],
            spread,
            a.iter().copied().fold(f64::MAX, f64::min),
            a.iter().copied().fold(f64::MIN, f64::max),
            percentile(&mut a, 0.05),
            percentile(&mut a, 0.95),
            b.iter().copied().fold(f64::MAX, f64::min),
            b.iter().copied().fold(f64::MIN, f64::max),
            percentile(&mut b.clone(), 0.05),
            percentile(&mut b, 0.95),
        );
    }
    let missing: Vec<&str> = world.heads.iter().filter(|h| h.start_nose.is_none()).map(|h| h.name.as_str()).collect();
    println!("stand-ins the measure could not read: {missing:?}");
    let unread: Vec<String> = world
        .heads
        .iter()
        .filter(|h| h.kotor_error.is_some() || (0..f.proportions().len()).any(|i| h.kotor.proportions[i].is_nan()))
        .map(|h| format!("{} ({})", h.name, h.kotor_error.unwrap_or("some rows")))
        .collect();
    println!("KOTOR heads with unreadable rows: {unread:?}");
}

/// How far below and above 0 the heads' needs reach: the farthest need on each side, or the feature's quantile of them.
fn reach(f: &dyn Feature, k: usize, needs: &[f64]) -> [f64; 2] {
    let q = f.span_quantile_of(f.proportions()[k].key);
    if q >= 1.0 {
        return [-needs.iter().copied().fold(0.0, f64::min), needs.iter().copied().fold(0.0, f64::max)];
    }
    let mut v = needs.to_vec();
    v.sort_by(f64::total_cmp);
    let at = |p: f64| if v.is_empty() { 0.0 } else { v[((v.len() - 1) as f64 * p).round() as usize] };
    [(-at(1.0 - q)).max(0.0), at(q).max(0.0)]
}

/// The span each slider is asked to reach on each side of the base head: the farthest any KOTOR head lies from its stand-in, plus half
/// a spread, at least one spread and at most four (or the feature's own cap for that slider).
fn spans(world: &World, k: usize, spread: f64) -> [f64; 2] {
    let needs = world.needs(k);
    let [lo, hi] = reach(world.feature, k, &needs);
    let cap = world.feature.span_cap_of(world.feature.proportions()[k].key);
    [lo, hi].map(|n| (n + 0.5 * spread).clamp(spread, cap * spread))
}

/// Makes the feature's named sliders. `SLIDERS_ONLY=key,key` remakes some into `<feature>/sliders_try.json` without touching the set.
pub fn make(world: &World, m: &Marks, data: &Path) -> SliderSet {
    let f = world.feature;
    let only: Option<String> = std::env::var("SLIDERS_ONLY").ok();
    let specs = f.specs();
    let wanted = |i: usize| only.as_deref().is_none_or(|o| o.split(',').any(|x| x == specs[i].measure));
    // the bumps are made only when a slider made of them is asked for
    let any_bumps = (0..specs.len()).any(|i| wanted(i) && f.drawn_step(specs[i].measure).is_none());
    let t0 = Instant::now();
    let b = if any_bumps {
        bumps(f, &world.mesh, &world.zone, &world.base_nose)
    } else {
        crate::maker::Bumps { movements: Vec::new(), parts: Vec::new(), centres: Vec::new() }
    };
    let q = bending(&world.mesh, &b);
    eprintln!("{} bumps in {:.1?}", b.movements.len(), t0.elapsed());
    let hold = hold_scale(m);
    let maker = any_bumps.then(|| Maker::new(f, &world.mesh, &world.smoother, &b, &q, &world.base_nose, &hold));
    let made: Vec<_> = parallel(specs.len(), specs.len(), |i| {
        let spec = &specs[i];
        if !wanted(i) {
            return None;
        }
        let k = proportion_index(f, spec.measure);
        let span = spans(world, k, m.spread_p[k]);
        let t = Instant::now();
        let step = 0.5 * m.spread_p[k];
        let made = match f.drawn_step(spec.measure) {
            // a drawn slider's keys are cheap, so it gets twice as many, and its gain stays steadier between them
            Some(apply) => {
                let starts: Vec<&[V3]> = std::iter::once(world.mesh.base.as_slice()).chain(world.heads.iter().map(|h| h.start.as_slice())).collect();
                crate::maker::make_drawn(f, &world.mesh, &world.smoother, &starts, spec, span, 0.5 * step, apply)
            }
            None => maker.as_ref().expect("bumps for a slider made of them").make(spec, span, step),
        };
        let repaired = made.keys.iter().filter(|k| k["level"].as_u64().unwrap_or(0) > 0).count();
        eprintln!(
            "  {:16} keys {:2} ({} repaired), {:7.2} to {:6.2} (asked {:7.2} to {:6.2}), ends: {} / {} in {:.1?}",
            spec.name,
            made.slider.keys.len(),
            repaired,
            made.slider.low,
            made.slider.high,
            -span[0],
            span[1],
            made.ends[0],
            made.ends[1],
            t.elapsed()
        );
        Some((made, span))
    });
    let report = json!(
        made.iter()
            .zip(specs.iter())
            .filter_map(|(m, spec)| m.as_ref().map(|(m, span)| json!({"name": spec.name, "measure": spec.measure, "hint": spec.hint,
                "group": spec.group, "keys": m.keys, "asked": [-span[0], span[1]], "ends": m.ends})))
            .collect::<Vec<_>>()
    );
    let named = if only.is_some() { format!("{}/named_only.json", f.key()) } else { f.files().named };
    write(data, &named, serde_json::to_string_pretty(&report).unwrap());
    let set = SliderSet {
        vertex_count: world.mesh.base.len(),
        sliders: made.into_iter().flatten().map(|(m, _)| m.slider).collect(),
        fields: fields(&b),
    };
    let out = if only.is_some() { format!("{}/sliders_try.json", f.key()) } else { f.files().sliders };
    write(data, &out, serde_json::to_string(&set).unwrap());
    set
}

/// Clay pictures of a head built from `values`, one per shot of the feature, with its faults against `checker` in red.
fn views(world: &World, set: &SliderSet, checker: &Checker, start: &[V3], frame: &Frame, values: &[f64], shots: &[Shot]) -> Vec<RgbImage> {
    let built = set.build(&world.smoother, start, values);
    let red = checker.marks(&built);
    let local: Vec<V3> = built.iter().map(|&p| frame.local(p)).collect();
    shots.iter().map(|s| clay(&local, &world.mesh.triangles, s, &red, &[])).collect()
}

fn base_checker(world: &World) -> Checker {
    let built = world.smoother.smooth(&world.mesh.base, SMOOTHING);
    let all: Vec<u32> = (0..world.mesh.base.len() as u32).collect();
    Checker::new(&world.mesh, &built, &all)
}

/// Each slider of `set` at its low end, 0 and its high end on the base head, one slider per row, every shot of the feature.
/// `SLIDERS_ONLY=key,key` draws only those sliders.
pub fn slider_sheet(world: &World, set: &SliderSet) -> RgbImage {
    let frame = Frame::from_eyes(world.mesh.eyes);
    let checker = base_checker(world);
    let n = set.sliders.len();
    let only: Vec<usize> = match std::env::var("SLIDERS_ONLY") {
        Ok(o) => (0..n).filter(|&i| o.split(',').any(|x| x == set.sliders[i].measure)).collect(),
        Err(_) => (0..n).collect(),
    };
    let shots = world.feature.shots();
    let tiles: Vec<RgbImage> = parallel(only.len(), only.len(), |r| {
        let i = only[r];
        let s = &set.sliders[i];
        let mut row = Vec::new();
        for (e, v) in [s.low, 0.0, s.high].into_iter().enumerate() {
            let mut values = vec![0.0; n];
            values[i] = v;
            let mut tiles = views(world, set, &checker, &world.mesh.base, &frame, &values, shots);
            if e != 1 {
                label(&mut tiles[0], &format!("{} {:+.1}", s.measure.replace('_', " "), v));
            }
            row.extend(tiles);
        }
        row
    })
    .into_iter()
    .flatten()
    .collect();
    sheet(&tiles, 3 * shots.len())
}

fn read_set(world: &World, data: &Path) -> SliderSet {
    let path = std::env::var("SLIDERS_SET").map(std::path::PathBuf::from).unwrap_or_else(|_| data.join(world.feature.files().sliders));
    let mut set: SliderSet = serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap();
    // a slider without a measure of the feature (the eyes' Inner corner depth, Lower lid flatness, Caruncle width) can be neither
    // solved nor graded, so the bench leaves it at 0
    let known = |s: &crate::set::Slider| world.feature.proportions().iter().any(|p| p.key == s.measure);
    let dropped: Vec<String> = set.sliders.iter().filter(|s| !known(s)).map(|s| s.name.clone()).collect();
    if !dropped.is_empty() {
        eprintln!("left at 0, without a measure: {}", dropped.join(", "));
        set.sliders.retain(|s| known(s));
    }
    set
}

/// Each slider's spread: the KOTOR heads' spread of its measure.
fn slider_spreads(world: &World, set: &SliderSet, m: &Marks) -> Vec<f64> {
    set.sliders.iter().map(|s| m.spread_p[proportion_index(world.feature, &s.measure)]).collect()
}

/// The range each slider would have if head `skip` were not one of the KOTOR heads: the asked span without it, cut to the keys.
fn held_out_bounds(world: &World, set: &SliderSet, m: &Marks, skip: usize) -> Vec<[f64; 2]> {
    set.sliders
        .iter()
        .map(|s| {
            let k = proportion_index(world.feature, &s.measure);
            let spread = m.spread_p[k];
            let needs: Vec<f64> = world
                .heads
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .filter_map(|(_, h)| h.start_nose.as_ref().map(|n| h.kotor.proportions[k] - n.proportions[k]))
                .filter(|d| !d.is_nan())
                .collect();
            let [lo, hi] = reach(world.feature, k, &needs);
            let cap = world.feature.span_cap_of(world.feature.proportions()[k].key);
            let lo = (lo + 0.5 * spread).clamp(spread, cap * spread);
            let hi = (hi + 0.5 * spread).clamp(spread, cap * spread);
            [(-lo).max(s.low), hi.min(s.high)]
        })
        .collect()
}

pub type Solved = (Vec<f64>, Option<crate::feature::Measured>, Vec<V3>, Vec<u32>);

/// Solves one head with `set` (in its own order), within `bounds`, from each slider's own gap and from 0, and keeps the better.
/// `smoothed` holds the movements of `set`'s sliders, smoothed once for every head.
fn solve_head(world: &World, h: &Head, set: &SliderSet, smoothed: &Smoothed, spread: &[f64], marks: &Scale, bounds: &[[f64; 2]]) -> Solved {
    let ev = Evaluator::with(world.feature, set, smoothed, &world.smoother, &h.start, &world.mesh.triangles, h.frame);
    let begin: Vec<f64> = match &h.start_nose {
        Some(n) => set
            .sliders
            .iter()
            .map(|s| {
                let k = proportion_index(world.feature, &s.measure);
                let d = h.kotor.proportions[k] - n.proportions[k];
                if d.is_nan() { 0.0 } else { d }
            })
            .collect(),
        None => vec![],
    };
    // SOLVE_FIX="<slider>;...": with SOLVE_WARM, those sliders stay at the head's value in the earlier page, for trials
    #[cfg(not(target_arch = "wasm32"))]
    if let (Ok(fix), Some(page)) = (std::env::var("SOLVE_FIX"), std::env::var("SOLVE_WARM").ok().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok())) {
        let warm = crate::eyes::tools::solved_values(&page, set, &h.name);
        let mut held = bounds.to_vec();
        for name in fix.split(';') {
            if let Some(j) = set.sliders.iter().position(|x| x.name == name) {
                held[j] = [warm[j], warm[j]];
            }
        }
        let s = solve_within(&ev, &h.kotor, marks, spread, PRIOR, &warm, &held);
        let positions = ev.positions(&s.values);
        return (s.values, s.measured, positions, ev.vertices.clone());
    }
    let a = solve_within(&ev, &h.kotor, marks, spread, PRIOR, &begin, bounds);
    let b = solve_within(&ev, &h.kotor, marks, spread, PRIOR, &[], bounds);
    let mut s = if b.cost < a.cost { b } else { a };
    // SOLVE_WARM=<page.json>: a third start from the head's values in an earlier page, for trials
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(page) = std::env::var("SOLVE_WARM").ok().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
        let warm = crate::eyes::tools::solved_values(&page, set, &h.name);
        let c = solve_within(&ev, &h.kotor, marks, spread, PRIOR, &warm, bounds);
        if c.cost < s.cost {
            s = c;
        }
    }
    // SOLVE_CLIP="<slider>=<low>;...": a solved value below its low is raised to it afterwards, without a new descent, for trials
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(clip) = std::env::var("SOLVE_CLIP") {
        let mut moved = false;
        for (name, low) in clip.split(';').filter_map(|c| c.split_once('=')) {
            let (Some(j), Ok(low)) = (set.sliders.iter().position(|x| x.name == name), low.parse::<f64>()) else { continue };
            if s.values[j] < low {
                s.values[j] = low;
                moved = true;
            }
        }
        if moved {
            s.measured = ev.measure(&s.values).ok();
        }
    }
    let positions = ev.positions(&s.values);
    (s.values, s.measured, positions, ev.vertices.clone())
}

fn faults_json(f: &Faults) -> Value {
    json!({"flipped": f.flipped, "folded": f.folded, "nostrils_flipped": f.nostrils_flipped, "total": f.total()})
}

/// The largest movement, in mm, of the vertices in `mask` between two heads.
fn largest(mask: &[bool], a: &[V3], b: &[V3]) -> f64 {
    (0..a.len()).filter(|&i| mask[i]).map(|i| length(sub(a[i], b[i])) * 1000.0).fold(0.0, f64::max)
}

/// A built head minus its start's best rigid fit over `mask`'s vertices, the largest leftover movement in mm: how far a group is
/// bent rather than carried.
fn bent(mask: &[bool], before: &[V3], after: &[V3]) -> f64 {
    let idx: Vec<usize> = (0..before.len()).filter(|&i| mask[i]).collect();
    if idx.len() < 3 {
        return 0.0;
    }
    let n = idx.len() as f64;
    let mean = |p: &[V3]| -> V3 { std::array::from_fn(|c| idx.iter().map(|&i| p[i][c]).sum::<f64>() / n) };
    let (ca, cb) = (mean(before), mean(after));
    let mut h = nalgebra::Matrix3::<f64>::zeros();
    for &i in &idx {
        let (a, b) = (sub(before[i], ca), sub(after[i], cb));
        for r in 0..3 {
            for c in 0..3 {
                h[(r, c)] += a[r] * b[c];
            }
        }
    }
    let svd = h.svd(true, true);
    let (u, vt) = (svd.u.unwrap(), svd.v_t.unwrap());
    let mut r = vt.transpose() * u.transpose();
    if r.determinant() < 0.0 {
        let mut v = vt.transpose();
        for k in 0..3 {
            v[(k, 2)] = -v[(k, 2)];
        }
        r = v * u.transpose();
    }
    idx.iter()
        .map(|&i| {
            let a = sub(before[i], ca);
            let moved: V3 = std::array::from_fn(|row| (0..3).map(|c| r[(row, c)] * a[c]).sum::<f64>() + cb[row]);
            length(sub(moved, after[i])) * 1000.0
        })
        .fold(0.0, f64::max)
}

/// Solves the 57 heads, scores them, checks the sliders' safety, and writes the report and the page's data.
pub fn bench(world: &World, m: &Marks, data: &Path) {
    let f = world.feature;
    let files = f.files();
    let t0 = Instant::now();
    let set = read_set(world, data);
    let n = set.sliders.len();
    let np = f.proportions().len();
    let no = f.outlines().len();
    let spread = slider_spreads(world, &set, m);
    let marks = &m.marks;
    let aim = if f.solve_by_spread() { hold_scale(m) } else { m.marks.clone() };
    let key_bounds: Vec<[f64; 2]> = set.sliders.iter().map(|s| [s.low, s.high]).collect();
    let threads = threads();
    let counts: Vec<usize> = COUNTS.iter().copied().filter(|&c| c < n).chain([n]).collect::<Vec<_>>();
    let counts: Vec<usize> = { let mut c = counts; c.dedup(); c };

    let smoothed = Smoothed::new(&set, &world.smoother);
    eprintln!("smoothed {} sliders and {} fields once in {:.1?}", n, set.fields.len(), t0.elapsed());

    // all sliders, ranges from every head
    let full: Vec<_> =
        parallel(world.heads.len(), threads, |i| solve_head(world, &world.heads[i], &set, &smoothed, &spread, &aim, &key_bounds));
    // the order of the sliders for the counts: first the measures that most stand-ins miss by more than their mark, then by the mean
    // size of the solved values
    let mut order: Vec<usize> = (0..n).collect();
    let missed = |j: usize| {
        let k = proportion_index(f, &set.sliders[j].measure);
        world
            .heads
            .iter()
            .filter(|h| h.start_nose.as_ref().is_some_and(|nn| (h.kotor.proportions[k] - nn.proportions[k]).abs() > marks.proportions[k]))
            .count()
    };
    let weight = |j: usize| full.iter().map(|f| (f.0[j] / spread[j]).abs()).sum::<f64>();
    order.sort_by(|&a, &b| missed(b).cmp(&missed(a)).then(weight(b).total_cmp(&weight(a))));
    eprintln!("solved with all sliders in {:.1?}; order {:?}", t0.elapsed(), order.iter().map(|&j| set.sliders[j].name.as_str()).collect::<Vec<_>>());
    let ordered =
        SliderSet { vertex_count: set.vertex_count, sliders: order.iter().map(|&j| set.sliders[j].clone()).collect(), fields: set.fields.clone() };
    let ordered_spread: Vec<f64> = order.iter().map(|&j| spread[j]).collect();

    // every head at every count, held out and with ranges from every head
    let solved: Vec<Value> = parallel(world.heads.len(), threads, |i| {
        let h = &world.heads[i];
        let mut modes = serde_json::Map::new();
        for mode in ["held_out", "full"] {
            let bounds = if mode == "held_out" { held_out_bounds(world, &set, m, i) } else { key_bounds.clone() };
            let mut per_count = serde_json::Map::new();
            let mut vertices = Value::Null;
            for &count in &counts {
                let sub_set = ordered.first(count);
                let sub_bounds: Vec<[f64; 2]> = order[..count.min(n)].iter().map(|&j| bounds[j]).collect();
                let t = Instant::now();
                let (values, measured, positions, verts) = solve_head(world, h, &sub_set, &smoothed, &ordered_spread[..count.min(n)], &aim, &sub_bounds);
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                let mut in_order = vec![0.0; n];
                for (o, &j) in order[..count.min(n)].iter().enumerate() {
                    in_order[j] = values[o];
                }
                let at_end: Vec<usize> = (0..n)
                    .filter(|&j| {
                        let w = (bounds[j][1] - bounds[j][0]).max(1e-9);
                        in_order[j] != 0.0 && ((in_order[j] - bounds[j][0]).abs() < 0.01 * w || (in_order[j] - bounds[j][1]).abs() < 0.01 * w)
                    })
                    .collect();
                let verdict = measured.as_ref().map(|nn| judge(f, nn, &h.kotor, marks));
                vertices = json!(verts);
                let mut entry = json!({"values": in_order, "ms": ms, "measured": measured, "verdict": verdict, "at_end": at_end,
                    "positions": positions.iter().map(|p| p.map(|c| c as f32)).collect::<Vec<_>>()});
                if f.solve_by_spread() {
                    // the same head judged against a quarter of KOTOR's spread, a stricter mark than the provisional one
                    entry["verdict_strict"] = json!(measured.as_ref().map(|nn| judge(f, nn, &h.kotor, &aim)));
                }
                per_count.insert(count.to_string(), entry);
            }
            modes.insert(mode.into(), json!({"counts": per_count, "vertices": vertices, "bounds": bounds}));
        }
        json!(modes)
    });
    eprintln!("solved {} heads at every count in {:.1?}", world.heads.len(), t0.elapsed());

    // the scorecard; a head whose KOTOR mesh the measure cannot read at all is left out of it
    let scored: Vec<usize> = (0..world.heads.len()).filter(|&i| world.heads[i].kotor_error.is_none()).collect();
    let mut scorecard = Vec::new();
    for mode in ["held_out", "full"] {
        for &count in &counts {
            let (mut made, mut per) = (0, vec![0usize; np + no]);
            let mut misses = Vec::new();
            for &i in &scored {
                let (h, s) = (&world.heads[i], &solved[i]);
                let v = &s[mode]["counts"][count.to_string()]["verdict"];
                if v["made"].as_bool() == Some(true) {
                    made += 1;
                } else {
                    misses.push(json!({"name": h.name, "title": h.title, "words": v["words"]}));
                }
                for (k, x) in v["pass"].as_array().into_iter().flatten().enumerate() {
                    if x.as_bool() == Some(true) {
                        per[k] += 1;
                    }
                }
            }
            let mut row = json!({"mode": mode, "count": count, "made": made, "of": scored.len(), "per_measure": per, "misses": misses});
            if f.solve_by_spread() {
                let strict = scored
                    .iter()
                    .filter(|&&i| solved[i][mode]["counts"][count.to_string()]["verdict_strict"]["made"].as_bool() == Some(true))
                    .count();
                row["made_strict"] = json!(strict);
                eprintln!("{mode:9} {count:2} sliders: {made} of {} made, {strict} with a quarter of KOTOR's spread as the mark; per measure {per:?}", scored.len());
            } else {
                eprintln!("{mode:9} {count:2} sliders: {made} of {} made; per measure {per:?}", scored.len());
            }
            scorecard.push(row);
        }
    }

    // the stand-ins alone, with every slider at 0: how many heads they already make
    if !world.guards.is_empty() {
        let (mut made, mut strict) = (0, 0);
        let mut misses = Vec::new();
        for &i in &scored {
            let h = &world.heads[i];
            let v = h.start_nose.as_ref().map(|s| judge(f, s, &h.kotor, marks));
            if h.start_nose.as_ref().is_some_and(|s| judge(f, s, &h.kotor, &aim)["made"].as_bool() == Some(true)) {
                strict += 1;
            }
            if v.as_ref().is_some_and(|v| v["made"].as_bool() == Some(true)) {
                made += 1;
            } else {
                misses.push(json!({"name": h.name, "title": h.title, "words": v.map(|v| v["words"].clone())}));
            }
        }
        eprintln!("stand-ins alone: {made} of {} made, {strict} with a quarter of KOTOR's spread as the mark", scored.len());
        scorecard.insert(0, json!({"mode": "standin", "count": 0, "made": made, "made_strict": strict, "of": scored.len(), "misses": misses}));
    }

    // the caps: per slider, how many heads would need a value past its range, and how many solved values sit at an end of the range
    let caps: Vec<Value> = set
        .sliders
        .iter()
        .enumerate()
        .map(|(j, s)| {
            let k = proportion_index(f, &s.measure);
            let beyond: Vec<Value> = world
                .heads
                .iter()
                .filter_map(|h| {
                    let need = h.kotor.proportions[k] - h.start_nose.as_ref()?.proportions[k];
                    (need < s.low || need > s.high).then(|| json!({"name": h.name, "need": need}))
                })
                .collect();
            let at_end: Vec<&str> = world
                .heads
                .iter()
                .zip(&solved)
                .filter(|(_, x)| x["full"]["counts"][n.to_string()]["at_end"].as_array().is_some_and(|a| a.iter().any(|v| v.as_u64() == Some(j as u64))))
                .map(|(h, _)| h.name.as_str())
                .collect();
            json!({"slider": s.name, "low": s.low, "high": s.high, "beyond": beyond, "at_end": at_end})
        })
        .collect();

    // safety on the base head: each slider at both ends, every pair at its four corners, and sampled heads
    let checker = base_checker(world);
    let base_built = world.smoother.smooth(&world.mesh.base, SMOOTHING);
    let build = |values: &[f64]| set.build(&world.smoother, &world.mesh.base, values);
    let standins: Vec<(Vec<V3>, Checker)> = parallel(world.heads.len(), threads, |i| {
        let h = &world.heads[i];
        let before = world.smoother.smooth(&h.start, SMOOTHING);
        let all: Vec<u32> = (0..before.len() as u32).collect();
        let c = Checker::new(&world.mesh, &before, &all);
        (before, c)
    });
    let mut ends = Vec::new();
    let mut guard_worst: Vec<(f64, f64)> = vec![(0.0, 0.0); world.guards.len()];
    for (j, s) in set.sliders.iter().enumerate() {
        for (end, v) in [("low", s.low), ("high", s.high)] {
            let mut values = vec![0.0; n];
            values[j] = v;
            let built = build(&values);
            let fl = checker.faults(&built);
            // the same end on every KOTOR stand-in, against the stand-in itself
            let on_heads: Vec<(usize, Vec<f64>)> = parallel(world.heads.len(), threads, |i| {
                let h = &world.heads[i];
                let b = set.build(&world.smoother, &h.start, &values);
                let g = world.guards.iter().map(|(_, mask)| largest(mask, &b, &standins[i].0)).collect();
                (standins[i].1.faults(&b).total(), g)
            });
            let mut zones = serde_json::Map::new();
            for (zi, zname) in crate::mesh::ZONES.iter().enumerate() {
                let d: Vec<f64> = (0..world.mesh.base.len())
                    .filter(|&i| world.mesh.zone[i] as usize == zi + 1 && !world.zone.contains(world.mesh.group[i] as usize))
                    .map(|i| length(sub(built[i], base_built[i])) * 1000.0)
                    .collect();
                if !d.is_empty() {
                    zones.insert((*zname).into(), json!({"max": d.iter().copied().fold(0.0, f64::max), "mean": d.iter().sum::<f64>() / d.len() as f64}));
                }
            }
            let edge: Vec<f64> = (0..world.mesh.base.len())
                .filter(|&i| {
                    let w = world.zone.weight[world.mesh.group[i] as usize];
                    w > 0.0 && w < 1.0
                })
                .map(|i| length(sub(built[i], base_built[i])) * 1000.0)
                .collect();
            zones.insert(
                "soft edge".into(),
                json!({"max": edge.iter().copied().fold(0.0, f64::max), "mean": edge.iter().sum::<f64>() / edge.len().max(1) as f64}),
            );
            let mut entry = json!({"slider": s.name, "end": end, "value": v, "faults": faults_json(&fl),
                "heads_with_faults": on_heads.iter().filter(|x| x.0 > 0).count(), "zones": zones});
            if !world.guards.is_empty() {
                let mut guards = serde_json::Map::new();
                for (gi, (g, mask)) in world.guards.iter().enumerate() {
                    let on_base = largest(mask, &built, &base_built);
                    let on_standins = on_heads.iter().map(|x| x.1[gi]).fold(0.0, f64::max);
                    let bend = bent(mask, &base_built, &built);
                    guard_worst[gi].0 = guard_worst[gi].0.max(on_base.max(on_standins));
                    guard_worst[gi].1 = guard_worst[gi].1.max(bend);
                    guards.insert(g.label().into(), json!({"base": on_base, "standins": on_standins, "bent": bend}));
                }
                entry["guards"] = json!(guards);
            }
            ends.push(entry);
        }
    }
    let pairs: Vec<Value> = parallel(n * (n - 1) / 2, threads, |p| {
        let (mut a, mut rest) = (0, p);
        while rest >= n - 1 - a {
            rest -= n - 1 - a;
            a += 1;
        }
        let b = a + 1 + rest;
        let mut worst = Vec::new();
        let mut on_standins = 0;
        for va in [set.sliders[a].low, set.sliders[a].high] {
            for vb in [set.sliders[b].low, set.sliders[b].high] {
                let mut values = vec![0.0; n];
                values[a] = va;
                values[b] = vb;
                let fl = checker.faults(&build(&values));
                if fl.total() > 0 {
                    worst.push(json!({"values": [va, vb], "faults": faults_json(&fl)}));
                }
                if !world.guards.is_empty() {
                    // the guarded heads' pairs are checked on the stand-ins too: how many stand-ins fold at this corner
                    on_standins += (0..world.heads.len())
                        .filter(|&i| standins[i].1.faults(&set.build(&world.smoother, &world.heads[i].start, &values)).total() > 0)
                        .count();
                }
            }
        }
        let mut entry = json!({"a": set.sliders[a].name, "b": set.sliders[b].name, "faulty_corners": worst});
        if !world.guards.is_empty() {
            entry["standin_corner_faults"] = json!(on_standins);
        }
        entry
    });
    let faulty_pairs = pairs.iter().filter(|p| !p["faulty_corners"].as_array().unwrap().is_empty()).count();
    let faulty_corners: usize = pairs.iter().map(|p| p["faulty_corners"].as_array().unwrap().len()).sum();
    eprintln!("pairs: {faulty_pairs} of {} pairs have a faulty corner, {faulty_corners} corners of {}", pairs.len(), 4 * pairs.len());

    // sampled heads: like the KOTOR heads (the mean and spread of the solved values, with their links), and anywhere in the ranges
    let solved_values: Vec<Vec<f64>> = solved
        .iter()
        .map(|s| s["full"]["counts"][n.to_string()]["values"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect())
        .collect();
    let mean: Vec<f64> = (0..n).map(|j| solved_values.iter().map(|v| v[j]).sum::<f64>() / solved_values.len() as f64).collect();
    let mut cov = nalgebra::DMatrix::<f64>::zeros(n, n);
    for v in &solved_values {
        for a in 0..n {
            for b in 0..n {
                cov[(a, b)] += (v[a] - mean[a]) * (v[b] - mean[b]) / solved_values.len() as f64;
            }
        }
    }
    for a in 0..n {
        cov[(a, a)] += 1e-6;
    }
    let chol = cov.cholesky().expect("a covariance").l();
    let mut rng = 0x2545F4914F6CDD1Du64;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        (rng >> 11) as f64 / (1u64 << 53) as f64
    };
    let mut draws: Vec<(String, Vec<f64>)> = Vec::new();
    for _ in 0..64 {
        let z: Vec<f64> = (0..n).map(|_| (-2.0 * next().max(1e-12).ln()).sqrt() * (std::f64::consts::TAU * next()).cos()).collect();
        draws.push((
            "like KOTOR".into(),
            (0..n).map(|a| (mean[a] + (0..n).map(|b| chol[(a, b)] * z[b]).sum::<f64>()).clamp(set.sliders[a].low, set.sliders[a].high)).collect(),
        ));
    }
    for _ in 0..64 {
        draws.push(("box".into(), set.sliders.iter().map(|s| s.low + next() * (s.high - s.low)).collect()));
    }
    let base_ev = Evaluator::new(f, &set, &world.smoother, &world.mesh.base, &world.mesh.triangles, Frame::from_eyes(world.mesh.eyes));
    let samples: Vec<Value> = parallel(draws.len(), threads, |i| {
        let (kind, values) = &draws[i];
        let fl = checker.faults(&build(values));
        json!({"kind": kind, "values": values, "faults": faults_json(&fl), "measured": base_ev.measure(values).ok().map(|nn| json!({"proportions": nn.proportions}))})
    });
    for kind in ["like KOTOR", "box"] {
        let faulty = samples.iter().filter(|s| s["kind"] == kind && s["faults"]["total"].as_u64().unwrap_or(0) > 0).count();
        eprintln!("sampled 64 {kind}: {faulty} with a fault");
    }
    if let Ok(dir) = std::env::var("SLIDERS_SHEETS") {
        let frame = Frame::from_eyes(world.mesh.eyes);
        std::fs::create_dir_all(&dir).unwrap();
        for (kind, file) in [("like KOTOR", "samples_kotor"), ("box", "samples_box")] {
            let picked: Vec<&Vec<f64>> = draws.iter().filter(|d| d.0 == kind).map(|d| &d.1).collect();
            for shot in f.shots() {
                let tiles: Vec<RgbImage> = parallel(picked.len(), threads, |i| {
                    let built = build(picked[i]);
                    let red = checker.marks(&built);
                    let local: Vec<V3> = built.iter().map(|&p| frame.local(p)).collect();
                    clay(&local, &world.mesh.triangles, shot, &red, &[])
                });
                sheet(&tiles, 8).save(format!("{dir}/{}_{file}_{}.png", f.key(), shot.name)).unwrap();
            }
        }
    }

    // faults of the solved KOTOR heads, against their own stand-ins
    let kotor_faults: Vec<Value> = parallel(world.heads.len(), threads, |i| {
        let h = &world.heads[i];
        let fl = standins[i].1.faults(&set.build(&world.smoother, &h.start, &solved_values[i]));
        json!({"name": h.name, "faults": faults_json(&fl)})
    });
    eprintln!(
        "solved KOTOR heads with a fault: {} of {}",
        kotor_faults.iter().filter(|k| k["faults"]["total"].as_u64().unwrap_or(0) > 0).count(),
        world.heads.len()
    );
    for (gi, (g, _)) in world.guards.iter().enumerate() {
        eprintln!("{}: moves {:.4} mm at most over every end on the base head and the stand-ins, bent {:.3} mm at most", g.label(), guard_worst[gi].0, guard_worst[gi].1);
    }

    // the page's data
    write_made(world, &solved, &counts, data, &files.made);
    write_starts(world, data, &files.starts);
    let sliders: Vec<Value> = set
        .sliders
        .iter()
        .zip(&spread)
        .map(|(s, sp)| json!({"name": s.name, "hint": s.hint, "group": s.group, "measure": s.measure, "unit": s.unit, "low": s.low, "high": s.high, "spread": sp}))
        .collect();
    let mut report = json!({
        "measure": {
            "proportions": f.proportions().iter().map(|p| json!({"key": p.key, "label": p.label, "low": p.low, "high": p.high, "unit": p.unit})).collect::<Vec<_>>(),
            "outlines": f.outlines().iter().map(|o| json!({"key": o.key, "label": o.label, "points": o.points})).collect::<Vec<_>>(),
            "marks": marks, "provisional": true, "spread_proportions": m.spread_p, "spread_outlines": m.spread_o,
        },
        "sliders": sliders,
        "order": order,
        "counts": counts,
        "heads": world.heads.iter().zip(&solved).map(|(h, s)| {
            let mut s = s.clone();
            for mode in ["held_out", "full"] {
                s[mode].as_object_mut().unwrap().remove("vertices");
                for (_, c) in s[mode]["counts"].as_object_mut().unwrap() {
                    c.as_object_mut().unwrap().remove("positions");
                    if let Some(nose) = c.get_mut("measured").and_then(|x| x.as_object_mut()) {
                        nose.retain(|k, _| k == "proportions" || k == "marks");
                    }
                }
            }
            json!({"name": h.name, "title": h.title, "mod": h.is_mod, "kotor": {"proportions": h.kotor.proportions, "marks": h.kotor.marks},
                   "standin": h.start_nose.as_ref().map(|nn| nn.proportions.clone()), "solved": s})
        }).collect::<Vec<_>>(),
        "scorecard": scorecard,
        "caps": caps,
        "ends": ends,
        "pairs": {"faulty_pairs": faulty_pairs, "faulty_corners": faulty_corners, "of": pairs.len(), "list": pairs},
        "samples": samples,
        "kotor_faults": kotor_faults,
    });
    if !world.guards.is_empty() {
        report["feature"] = json!({"key": f.key(), "title": f.title(), "groups": f.groups()});
        report["guards"] = json!(world.guards.iter().zip(&guard_worst).map(|((g, _), w)| json!({"label": g.label(), "largest": w.0, "bent": w.1})).collect::<Vec<_>>());
        report["unreadable"] = json!(world.heads.iter().filter(|h| (0..np + no).any(|r| !h.kotor.readable(r))).map(|h| {
            let rows: Vec<&str> = f.proportions().iter().map(|p| p.key).chain(f.outlines().iter().map(|o| o.key)).enumerate()
                .filter(|(r, _)| !h.kotor.readable(*r)).map(|(_, k)| k).collect();
            json!({"name": h.name, "rows": rows})
        }).collect::<Vec<_>>());
    }
    write(data, &files.report, serde_json::to_string_pretty(&report).unwrap());
    write(data, &files.page, serde_json::to_string(&report).unwrap());
    eprintln!("done in {:.1?}", t0.elapsed());
}

/// The quick mode (`sliders <f> presets`): the 57 heads solved once, with all the set's sliders and ranges from every head. It writes
/// where `sliders <f> bench` writes, in the same form: the report, the page's data, the made features and the stand-ins. The report
/// holds only the mode "full" at the set's slider count, the stand-ins' row of the scorecard and the caps. It has no held-out mode, no
/// smaller counts, no end, pair or sample checks, and it says `"quick": true`.
pub fn presets(world: &World, m: &Marks, data: &Path) {
    let f = world.feature;
    let files = f.files();
    let t0 = Instant::now();
    let set = &read_set(world, data);
    let n = set.sliders.len();
    let np = f.proportions().len();
    let no = f.outlines().len();
    let spread = slider_spreads(world, set, m);
    let marks = &m.marks;
    let aim = if f.solve_by_spread() { hold_scale(m) } else { m.marks.clone() };
    let bounds: Vec<[f64; 2]> = set.sliders.iter().map(|s| [s.low, s.high]).collect();
    let smoothed = Smoothed::new(set, &world.smoother);
    eprintln!("smoothed {} sliders and {} fields once in {:.1?}", n, set.fields.len(), t0.elapsed());
    let solved = parallel(world.heads.len(), threads(), |i| {
        let t = Instant::now();
        let (values, measured, positions, vertices) = solve_head(world, &world.heads[i], set, &smoothed, &spread, &aim, &bounds);
        (values, measured, positions, vertices, t.elapsed().as_secs_f64() * 1000.0)
    });
    eprintln!("solved {} heads with all sliders in {:.1?}", world.heads.len(), t0.elapsed());
    // the sliders' order, as the bench gives it: first the measures that most stand-ins miss by more than their mark, then by the mean
    // size of the solved values
    let mut order: Vec<usize> = (0..n).collect();
    let missed = |j: usize| {
        let k = proportion_index(f, &set.sliders[j].measure);
        world
            .heads
            .iter()
            .filter(|h| h.start_nose.as_ref().is_some_and(|s| (h.kotor.proportions[k] - s.proportions[k]).abs() > marks.proportions[k]))
            .count()
    };
    let weight = |j: usize| solved.iter().map(|s| (s.0[j] / spread[j]).abs()).sum::<f64>();
    order.sort_by(|&a, &b| missed(b).cmp(&missed(a)).then(weight(b).total_cmp(&weight(a))));
    let count = n.to_string();
    let entries: Vec<Value> = world
        .heads
        .iter()
        .zip(&solved)
        .map(|(h, s)| {
            let at_end: Vec<usize> = (0..n)
                .filter(|&j| {
                    let w = (bounds[j][1] - bounds[j][0]).max(1e-9);
                    s.0[j] != 0.0 && ((s.0[j] - bounds[j][0]).abs() < 0.01 * w || (s.0[j] - bounds[j][1]).abs() < 0.01 * w)
                })
                .collect();
            let mut entry = json!({"values": s.0, "ms": s.4, "measured": s.1, "at_end": at_end,
                "verdict": s.1.as_ref().map(|x| judge(f, x, &h.kotor, marks))});
            if f.solve_by_spread() {
                entry["verdict_strict"] = json!(s.1.as_ref().map(|x| judge(f, x, &h.kotor, &aim)));
            }
            entry
        })
        .collect();
    let scored: Vec<usize> = (0..world.heads.len()).filter(|&i| world.heads[i].kotor_error.is_none()).collect();
    let mut scorecard = Vec::new();
    if !world.guards.is_empty() {
        let (mut made, mut strict) = (0, 0);
        let mut misses = Vec::new();
        for &i in &scored {
            let h = &world.heads[i];
            let v = h.start_nose.as_ref().map(|s| judge(f, s, &h.kotor, marks));
            if h.start_nose.as_ref().is_some_and(|s| judge(f, s, &h.kotor, &aim)["made"].as_bool() == Some(true)) {
                strict += 1;
            }
            if v.as_ref().is_some_and(|v| v["made"].as_bool() == Some(true)) {
                made += 1;
            } else {
                misses.push(json!({"name": h.name, "title": h.title, "words": v.map(|v| v["words"].clone())}));
            }
        }
        eprintln!("stand-ins alone: {made} of {} made, {strict} with a quarter of KOTOR's spread as the mark", scored.len());
        scorecard.push(json!({"mode": "standin", "count": 0, "made": made, "made_strict": strict, "of": scored.len(), "misses": misses}));
    }
    {
        let (mut made, mut per) = (0, vec![0usize; np + no]);
        let mut misses = Vec::new();
        for &i in &scored {
            let (h, v) = (&world.heads[i], &entries[i]["verdict"]);
            if v["made"].as_bool() == Some(true) {
                made += 1;
            } else {
                misses.push(json!({"name": h.name, "title": h.title, "words": v["words"]}));
            }
            for (k, x) in v["pass"].as_array().into_iter().flatten().enumerate() {
                if x.as_bool() == Some(true) {
                    per[k] += 1;
                }
            }
        }
        let mut row = json!({"mode": "full", "count": n, "made": made, "of": scored.len(), "per_measure": per, "misses": misses});
        if f.solve_by_spread() {
            let strict = scored.iter().filter(|&&i| entries[i]["verdict_strict"]["made"].as_bool() == Some(true)).count();
            row["made_strict"] = json!(strict);
            eprintln!("full {n:2} sliders: {made} of {} made, {strict} with a quarter of KOTOR's spread as the mark; per measure {per:?}", scored.len());
        } else {
            eprintln!("full {n:2} sliders: {made} of {} made; per measure {per:?}", scored.len());
        }
        scorecard.push(row);
    }
    let caps: Vec<Value> = set
        .sliders
        .iter()
        .enumerate()
        .map(|(j, s)| {
            let k = proportion_index(f, &s.measure);
            let beyond: Vec<Value> = world
                .heads
                .iter()
                .filter_map(|h| {
                    let need = h.kotor.proportions[k] - h.start_nose.as_ref()?.proportions[k];
                    (need < s.low || need > s.high).then(|| json!({"name": h.name, "need": need}))
                })
                .collect();
            let at_end: Vec<&str> = world
                .heads
                .iter()
                .zip(&entries)
                .filter(|(_, e)| e["at_end"].as_array().is_some_and(|a| a.iter().any(|v| v.as_u64() == Some(j as u64))))
                .map(|(h, _)| h.name.as_str())
                .collect();
            json!({"slider": s.name, "low": s.low, "high": s.high, "beyond": beyond, "at_end": at_end})
        })
        .collect();
    // the made features and the stand-ins, for the page
    let mut made: Vec<f32> = Vec::new();
    let mut index = serde_json::Map::new();
    for (h, s) in world.heads.iter().zip(&solved) {
        let offset = made.len() / 3;
        made.extend(s.2.iter().flat_map(|p| p.map(|c| c as f32)));
        index.insert(h.name.clone(), json!({"full": {"vertices": s.3, "offsets": {count.clone(): offset}}}));
    }
    write_bytes(data, &format!("{}.bin", files.made), made.iter().flat_map(|x| x.to_le_bytes()).collect());
    write(data, &format!("{}.json", files.made), serde_json::to_string(&index).unwrap());
    write_starts(world, data, &files.starts);
    let sliders: Vec<Value> = set
        .sliders
        .iter()
        .zip(&spread)
        .map(|(s, sp)| json!({"name": s.name, "hint": s.hint, "group": s.group, "measure": s.measure, "unit": s.unit, "low": s.low, "high": s.high, "spread": sp}))
        .collect();
    let mut report = json!({
        "quick": true,
        "measure": {
            "proportions": f.proportions().iter().map(|p| json!({"key": p.key, "label": p.label, "low": p.low, "high": p.high, "unit": p.unit})).collect::<Vec<_>>(),
            "outlines": f.outlines().iter().map(|o| json!({"key": o.key, "label": o.label, "points": o.points})).collect::<Vec<_>>(),
            "marks": marks, "provisional": true, "spread_proportions": m.spread_p, "spread_outlines": m.spread_o,
        },
        "sliders": sliders,
        "order": order,
        "counts": [n],
        "heads": world.heads.iter().zip(&entries).map(|(h, e)| {
            let mut e = e.clone();
            if let Some(x) = e.get_mut("measured").and_then(|x| x.as_object_mut()) {
                x.retain(|k, _| k == "proportions" || k == "marks");
            }
            json!({"name": h.name, "title": h.title, "mod": h.is_mod, "kotor": {"proportions": h.kotor.proportions, "marks": h.kotor.marks},
                   "standin": h.start_nose.as_ref().map(|x| x.proportions.clone()), "solved": {"full": {"counts": {count.clone(): e}, "bounds": bounds}}})
        }).collect::<Vec<_>>(),
        "scorecard": scorecard,
        "caps": caps,
    });
    if !world.guards.is_empty() {
        report["feature"] = json!({"key": f.key(), "title": f.title(), "groups": f.groups()});
        report["unreadable"] = json!(world.heads.iter().filter(|h| (0..np + no).any(|r| !h.kotor.readable(r))).map(|h| {
            let rows: Vec<&str> = f.proportions().iter().map(|p| p.key).chain(f.outlines().iter().map(|o| o.key)).enumerate()
                .filter(|(r, _)| !h.kotor.readable(*r)).map(|(_, k)| k).collect();
            json!({"name": h.name, "rows": rows})
        }).collect::<Vec<_>>());
    }
    write(data, &files.report, serde_json::to_string_pretty(&report).unwrap());
    write(data, &files.page, serde_json::to_string(&report).unwrap());
    eprintln!("done in {:.1?}", t0.elapsed());
}

/// The made features for the page: per head, per mode, per count, the positions of the evaluator's vertices.
fn write_made(world: &World, solved: &[Value], counts: &[usize], data: &Path, made_file: &str) {
    let mut made: Vec<f32> = Vec::new();
    let mut index = serde_json::Map::new();
    for (h, s) in world.heads.iter().zip(solved) {
        let mut entry = serde_json::Map::new();
        for mode in ["held_out", "full"] {
            let mut offsets = serde_json::Map::new();
            for &count in counts {
                offsets.insert(count.to_string(), json!(made.len() / 3));
                for q in s[mode]["counts"][count.to_string()]["positions"].as_array().unwrap() {
                    made.extend(q.as_array().unwrap().iter().map(|c| c.as_f64().unwrap() as f32));
                }
            }
            entry.insert(mode.into(), json!({"vertices": s[mode]["vertices"], "offsets": offsets}));
        }
        index.insert(h.name.clone(), json!(entry));
    }
    write_bytes(data, &format!("{made_file}.bin"), made.iter().flat_map(|x| x.to_le_bytes()).collect());
    write(data, &format!("{made_file}.json"), serde_json::to_string(&index).unwrap());
}

/// The stand-ins before smoothing, for the page.
fn write_starts(world: &World, data: &Path, file: &str) {
    let bytes: Vec<u8> = world.heads.iter().flat_map(|h| h.start.iter().flat_map(|p| p.map(|c| (c as f32).to_le_bytes())).flatten()).collect();
    write_bytes(data, file, bytes);
}

/// The guard groups' labels, for the tools.
pub fn guard_labels(g: &[Guard]) -> Vec<&'static str> {
    g.iter().map(|g| g.label()).collect()
}

/// KOTOR's own meshes of the 57 heads in the feature's shots, one head per row, with the triangles that are not skin in blue and the
/// head's name on its first picture.
pub fn kotor_sheet(world: &World) -> RgbImage {
    let shots = world.feature.shots();
    let tiles: Vec<RgbImage> = parallel(world.heads.len(), threads(), |i| {
        let h = &world.heads[i];
        let local: Vec<V3> = h.kotor_v.iter().map(|&p| h.frame.local(p)).collect();
        let tint: Vec<Option<[f64; 3]>> = h
            .kotor_f
            .iter()
            .map(|t| match &h.kotor_skin {
                Some(s) if t.iter().any(|&v| !s[v as usize]) => Some([90.0, 140.0, 230.0]),
                _ => None,
            })
            .collect();
        let marked: Vec<V3> = serde_json::from_value(h.kotor.marks["dots"].clone()).unwrap_or_default();
        let mut row: Vec<RgbImage> = shots
            .iter()
            .map(|s| {
                let mut img = clay(&local, &h.kotor_f, s, &[], &tint);
                crate::render::dots(&mut img, s, &marked);
                img
            })
            .collect();
        label(&mut row[0], &h.name.replace('_', " "));
        row
    })
    .into_iter()
    .flatten()
    .collect();
    sheet(&tiles, 2 * shots.len())
}

/// The stand-ins with every feature's own shape taken out, one feature after the other, for the page's live sliders, which build all
/// features at once (`web/starts_all.bin`).
pub fn all_starts(input: &Input, data: &Path) {
    let mesh = input.head_mesh();
    let sm = crate::set::smoother(&mesh.base, &mesh.triangles, &mesh.keep);
    let base_built = sm.smooth(&mesh.base, SMOOTHING);
    let frame = Frame::from_eyes(mesh.eyes);
    let zones: Vec<crate::zone::Zone> = crate::feature::all()
        .iter()
        .map(|f| {
            let base = f.measure(&crate::measure::Surface::within(&base_built, &mesh.triangles, &frame, &f.window())).expect("the base head");
            f.zone(&mesh, &base)
        })
        .collect();
    let base_merged = mesh.merge(&mesh.base);
    let names: Vec<String> = input.manifest["heads"].as_array().unwrap().iter().map(|h| h["name"].as_str().unwrap().to_string()).collect();
    let starts: Vec<Vec<V3>> = parallel(names.len(), threads(), |i| {
        let mut p = input.points(&format!("heads/{}/standin_pre", names[i]));
        for z in &zones {
            let moved: Vec<V3> = mesh.merge(&p).iter().zip(&base_merged).map(|(a, b)| sub(*a, *b)).collect();
            let own = mesh.expand(&z.own_shape(&mesh, &moved));
            p = p.iter().zip(&own).map(|(a, o)| sub(*a, *o)).collect();
        }
        if crate::eyes::paste_on() {
            p = crate::eyes::paste_start_mesh(&mesh, &p);
        }
        p
    });
    let bytes: Vec<u8> = starts.iter().flat_map(|s| s.iter().flat_map(|p| p.map(|c| (c as f32).to_le_bytes())).flatten()).collect();
    write_bytes(data, "web/starts_all.bin", bytes);
    eprintln!("wrote web/starts_all.bin for {} heads", names.len());
}

/// The base head in the feature's shots: first tinted by the zone's weight (white is 0, red is 1), then by the part each triangle's
/// nearest bump would belong to (blue, green, orange, purple, teal in the order of the parts met).
pub fn zone_sheet(world: &World) -> RgbImage {
    let frame = Frame::from_eyes(world.mesh.eyes);
    let built = world.smoother.smooth(&world.mesh.base, SMOOTHING);
    let local: Vec<V3> = built.iter().map(|&p| frame.local(p)).collect();
    let base_local: Vec<V3> = world.mesh.base.iter().map(|&p| frame.local(p)).collect();
    let tris = &world.mesh.triangles;
    let w = |i: u32| world.zone.weight[world.mesh.group[i as usize] as usize];
    let weight_tint: Vec<Option<[f64; 3]>> = tris
        .iter()
        .map(|t| {
            let a = t.iter().map(|&i| w(i)).sum::<f64>() / 3.0;
            (a > 0.0).then(|| [222.0 + a * 13.0, 196.0 * (1.0 - 0.7 * a), 172.0 * (1.0 - 0.7 * a)])
        })
        .collect();
    let colours = [[90.0, 140.0, 230.0], [90.0, 200.0, 110.0], [240.0, 160.0, 60.0], [170.0, 100.0, 220.0], [60.0, 190.0, 190.0]];
    let mut names: Vec<&str> = Vec::new();
    let part_tint: Vec<Option<[f64; 3]>> = tris
        .iter()
        .map(|t| {
            if t.iter().any(|&i| w(i) < 0.6) {
                return None;
            }
            let c = t.iter().fold([0.0; 3], |a, &i| crate::add(a, crate::scale(base_local[i as usize], 1.0 / 3.0)));
            let part = world.feature.part_of(c, &world.base_nose);
            let k = names.iter().position(|n| *n == part).unwrap_or_else(|| {
                names.push(part);
                names.len() - 1
            });
            Some(colours[k % colours.len()])
        })
        .collect();
    eprintln!("parts in colour order: {names:?}");
    let mut tiles = Vec::new();
    for tint in [&weight_tint, &part_tint] {
        for s in world.feature.shots() {
            tiles.push(clay(&local, tris, s, &[], tint));
        }
    }
    sheet(&tiles, world.feature.shots().len())
}

/// For each named head: KOTOR's own mesh, then our head made by the sliders (ranges from every head, every slider), in the feature's
/// shots, from the bench's page data. `mode` is "full" or "held_out".
pub fn made_sheet(world: &World, data: &Path, names: &[String], mode: &str) -> RgbImage {
    let set = read_set(world, data);
    let page: Value = serde_json::from_str(&std::fs::read_to_string(data.join(world.feature.files().page)).unwrap()).unwrap();
    let n = set.sliders.len();
    let shots = world.feature.shots();
    let mut tiles = Vec::new();
    for name in names {
        let Some(i) = world.heads.iter().position(|h| &h.name == name) else { continue };
        let h = &world.heads[i];
        let values: Vec<f64> = page["heads"][i]["solved"][mode]["counts"][n.to_string()]["values"]
            .as_array()
            .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect())
            .unwrap_or_else(|| vec![0.0; n]);
        let kl: Vec<V3> = h.kotor_v.iter().map(|&p| h.frame.local(p)).collect();
        let tint: Vec<Option<[f64; 3]>> = h
            .kotor_f
            .iter()
            .map(|t| match &h.kotor_skin {
                Some(s) if t.iter().any(|&v| !s[v as usize]) => Some([200.0, 180.0, 165.0]),
                _ => None,
            })
            .collect();
        for (k, s) in shots.iter().enumerate() {
            let mut img = clay(&kl, &h.kotor_f, s, &[], &tint);
            if k == 0 {
                label(&mut img, &format!("{} kotor", name.replace('_', " ")));
            }
            tiles.push(img);
        }
        let built = set.build(&world.smoother, &h.start, &values);
        let local: Vec<V3> = built.iter().map(|&p| h.frame.local(p)).collect();
        for (k, s) in shots.iter().enumerate() {
            let mut img = clay(&local, &world.mesh.triangles, s, &[], &[]);
            if k == 0 {
                label(&mut img, "ours");
            }
            tiles.push(img);
        }
    }
    sheet(&tiles, 2 * shots.len())
}
