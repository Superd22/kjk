//! The nose bench: learns the nose sliders from the KOTOR heads, proves the measure with its four checks, solves every KOTOR nose at
//! several slider counts, and writes the scorecard, the leak table, the sampled noses and everything the nose page shows.
//!
//! Its inputs are written once by `~/.claude/tools/likeness-fit/sliders/prep_nose.py`. Usage: `nose_bench [data folder]`, by default
//! `~/.claude/tools/likeness-fit-data/sliders`. It writes `web/` in that folder.

use head_shape::smooth::Smoother;
use head_sliders::round1::learn::{Example, NAMED, learn};
use head_sliders::measure::{
    Frame, N_PROPORTIONS, Nose, OUTLINES, PROPORTIONS, Surface, measure, outline_difference, size,
};
use head_sliders::mesh::HeadMesh;
use head_sliders::mesh::input::Input;
use head_sliders::set::{SMOOTHING, SliderSet, smoother};
use head_sliders::round1::solve::{Evaluator, Scale, gaps, solve};
use head_sliders::round1::verdict::judge;
use head_sliders::zone::Zone;
use head_sliders::{V3, add, cross, length, normalize, scale, sub};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Instant;

const COUNTS: [usize; 5] = [4, 6, 8, 10, 12];
const MAX_COUNT: usize = 12;
const PRIOR: f64 = 0.01;
/// How strongly the learning fields are smoothed (`NOSE_FIELD_SMOOTHING`, 0 by default). Strength 10 halved the faults on the solved
/// KOTOR noses but cost 3 made noses and did not change the faults of the sampled noses, so it is off.
fn field_smoothing() -> f64 {
    std::env::var("NOSE_FIELD_SMOOTHING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

struct Head {
    name: String,
    title: String,
    is_mod: bool,
    frame: Frame,
    eyes: [V3; 2],
    kotor_v: Vec<V3>,
    kotor_f: Vec<[u32; 3]>,
    kotor: Nose,
    /// The stand-in face before smoothing: the fit with its own nose shape taken out.
    start: Vec<V3>,
    /// The nose's own shape in our mesh wrapped onto the KOTOR head, for learning.
    wrap_shape: Vec<V3>,
    wrap: Vec<V3>,
}

fn main() {
    let data = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap())
                .join(".claude/tools/likeness-fit-data/sliders")
        });
    let input = Input::open(&data.join("input"));
    let web = data.join("web");
    std::fs::create_dir_all(&web).unwrap();
    let t0 = Instant::now();
    let mesh = input.head_mesh();
    let sm = smoother(&mesh.base, &mesh.triangles, &mesh.keep);
    let base_frame = Frame::from_eyes(mesh.eyes);
    let base_nose = measure(&Surface::new(
        &sm.smooth(&mesh.base, SMOOTHING),
        &mesh.triangles,
        &base_frame,
    ))
    .expect("the base head's nose");
    let zone = Zone::nose(&mesh, base_nose.marks.tip[1], base_nose.marks.base[1]);
    let own_shape = |positions: &[V3]| -> Vec<V3> {
        let moved: Vec<V3> = mesh
            .merge(positions)
            .iter()
            .zip(mesh.merge(&mesh.base))
            .map(|(p, b)| sub(*p, b))
            .collect();
        mesh.expand(&zone.own_shape(&mesh, &moved))
    };

    // the wrap lies on KOTOR's facets and slides along them, so its nose shapes can carry ripples finer than a nose; they can be
    // smoothed before learning (`field_smoothing`)
    let calm = |field: &[V3]| -> Vec<V3> {
        if field_smoothing() <= 0.0 {
            return field.to_vec();
        }
        let s = sm.smooth(
            field,
            head_shape::smooth::Method::Implicit {
                strength: field_smoothing(),
            },
        );
        mesh.expand(&mesh.merge(&s))
    };
    let heads: Vec<Head> = input.manifest["heads"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            let name = h["name"].as_str().unwrap().to_string();
            let eyes = input.points(&format!("heads/{name}/eyes"));
            let frame = Frame::from_eyes([eyes[0], eyes[1]]);
            let kotor_v = input.points(&format!("heads/{name}/kotor_v"));
            let kotor_f = input.triangles(&format!("heads/{name}/kotor_f"));
            let kotor = measure(&Surface::new(&kotor_v, &kotor_f, &frame))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            let pre = input.points(&format!("heads/{name}/standin_pre"));
            let fit_own = own_shape(&pre);
            let start = pre.iter().zip(&fit_own).map(|(p, o)| sub(*p, *o)).collect();
            let wrap = input.points(&format!("heads/{name}/wrap"));
            let wrap_shape = calm(&own_shape(&wrap));
            Head {
                title: h["title"].as_str().unwrap_or("").to_string(),
                is_mod: h["mod"].as_bool().unwrap_or(false),
                name,
                frame,
                eyes: [eyes[0], eyes[1]],
                kotor_v,
                kotor_f,
                kotor,
                start,
                wrap_shape,
                wrap,
            }
        })
        .collect();
    eprintln!(
        "{} heads read and measured in {:.1?}",
        heads.len(),
        t0.elapsed()
    );

    let references: [f64; 3] = std::array::from_fn(|o| {
        heads
            .iter()
            .map(|h| size(o, &h.kotor.outlines[o]))
            .sum::<f64>()
            / heads.len() as f64
    });

    // check 1: what nobody notices
    let check1 = check_unnoticed(&heads, &mesh, &references);
    if std::env::var("NOSE_CHECK1_ONLY").is_ok() {
        // a quick look at the measure alone: each measure's wobble against KOTOR's spread
        let names: Vec<&str> = PROPORTIONS.iter().map(|p| p.key).chain(OUTLINES.iter().map(|o| o.key)).collect();
        let spread: Vec<f64> = (0..N_PROPORTIONS)
            .map(|i| sd(&heads.iter().map(|h| h.kotor.proportions[i]).collect::<Vec<_>>()))
            .collect();
        for row in check1.0["summary"].as_array().unwrap() {
            let stats = row["stats"].as_array().unwrap();
            let cells: Vec<String> = names
                .iter()
                .enumerate()
                .map(|(i, n)| format!("{n} {:.2}/{:.2}", stats[i]["median"].as_f64().unwrap(), stats[i]["p95"].as_f64().unwrap()))
                .collect();
            println!("{:10} {}", row["what"].as_str().unwrap(), cells.join("  "));
        }
        println!("spread     {}", names.iter().zip(&spread).map(|(n, s)| format!("{n} {s:.2}")).collect::<Vec<_>>().join("  "));
        return;
    }
    // provisional pass marks: a quarter of the KOTOR noses' spread, and never below twice the 95th percentile of the measure's noise
    let spread_p: [f64; N_PROPORTIONS] = std::array::from_fn(|i| {
        sd(&heads
            .iter()
            .map(|h| h.kotor.proportions[i])
            .collect::<Vec<_>>())
    });
    // an outline's spread over the KOTOR noses: the root mean square difference between two of them, over the square root of 2, which
    // is the spread around their mean
    let spread_o: [f64; 3] = std::array::from_fn(|o| {
        let mut sum = 0.0;
        let mut count = 0.0;
        for a in &heads {
            for b in &heads {
                sum +=
                    outline_difference(o, &a.kotor.outlines[o], &b.kotor.outlines[o], references[o])
                        .powi(2);
                count += 1.0;
            }
        }
        (sum / count / 2.0).sqrt()
    });
    let noise_p = &check1.1;
    let noise_o = &check1.2;
    let marks = Scale {
        proportions: std::array::from_fn(|i| (0.25 * spread_p[i]).max(2.0 * noise_p[i])),
        outlines: std::array::from_fn(|o| (0.25 * spread_o[o]).max(2.0 * noise_o[o])),
        references,
    };
    eprintln!("provisional marks {:?}", marks);

    // the marks and spreads, for the round 2 tool (`nose_r2`)
    std::fs::write(
        data.join("marks.json"),
        serde_json::to_string_pretty(&json!({"marks": marks, "spread_proportions": spread_p, "spread_outlines": spread_o}))
            .unwrap(),
    )
    .unwrap();
    if std::env::var("NOSE_MARKS_ONLY").is_ok() {
        return;
    }

    // learning: the set from all game heads, and one set per game head learned without it
    let calibrate = |movement: &[V3], k: usize| -> f64 {
        let up: Vec<V3> = mesh
            .base
            .iter()
            .zip(movement)
            .map(|(b, m)| add(*b, scale(*m, 2.0)))
            .collect();
        let down: Vec<V3> = mesh
            .base
            .iter()
            .zip(movement)
            .map(|(b, m)| add(*b, scale(*m, -2.0)))
            .collect();
        let a = measure(&Surface::new(
            &sm.smooth(&up, SMOOTHING),
            &mesh.triangles,
            &base_frame,
        ))
        .expect("calibration");
        let b = measure(&Surface::new(
            &sm.smooth(&down, SMOOTHING),
            &mesh.triangles,
            &base_frame,
        ))
        .expect("calibration");
        (a.proportions[k] - b.proportions[k]) / 4.0
    };
    let example = |h: &Head| Example {
        shape: h.wrap_shape.clone(),
        proportions: h.kotor.proportions,
    };
    let game: Vec<usize> = (0..heads.len()).filter(|&i| !heads[i].is_mod).collect();
    let (full_set, full_spread) = learn(
        &game.iter().map(|&i| example(&heads[i])).collect::<Vec<_>>(),
        MAX_COUNT - NAMED.len(),
        &calibrate,
    );
    let loo: Vec<Option<(SliderSet, Vec<f64>)>> = (0..heads.len())
        .map(|i| {
            if heads[i].is_mod {
                return None;
            }
            let ex: Vec<Example> = game
                .iter()
                .filter(|&&j| j != i)
                .map(|&j| example(&heads[j]))
                .collect();
            Some(learn(&ex, MAX_COUNT - NAMED.len(), &calibrate))
        })
        .collect();
    eprintln!(
        "learned the full set and {} held-out sets in {:.1?}",
        loo.iter().flatten().count(),
        t0.elapsed()
    );

    // solving every head at every count, held out (mods always are) and with the head learned
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let solved: Vec<Value> = parallel(heads.len(), threads, |i| {
        let h = &heads[i];
        let mut modes = serde_json::Map::new();
        for mode in ["held_out", "learned"] {
            let (set, spread) = match (mode, &loo[i]) {
                ("held_out", Some((s, sp))) => (s, sp),
                ("held_out", None) | ("learned", _) => (&full_set, &full_spread),
                _ => unreachable!(),
            };
            if mode == "learned" && h.is_mod {
                continue;
            }
            let ev = Evaluator::new(set, &sm, &h.start, &mesh.triangles, h.frame);
            let start_nose = ev.measure(&[]).ok();
            let mut per_count = serde_json::Map::new();
            for &count in &COUNTS {
                let e = ev.first(count);
                let t = Instant::now();
                let begin: Vec<f64> = match &start_nose {
                    Some(n) => NAMED
                        .iter()
                        .map(|&k| h.kotor.proportions[k] - n.proportions[k])
                        .collect(),
                    None => vec![],
                };
                // from each proportion's own gap, and from the base nose; the better of the two
                let a = solve(&e, &h.kotor, &marks, &spread[..count], PRIOR, &begin);
                let b = solve(&e, &h.kotor, &marks, &spread[..count], PRIOR, &[]);
                let s = if b.cost < a.cost { b } else { a };
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                let verdict = s.nose.as_ref().map(|n| judge(n, &h.kotor, &marks));
                per_count.insert(
                    count.to_string(),
                    json!({"values": s.values, "cost": s.cost, "rounds": s.rounds, "ms": ms, "nose": s.nose, "verdict": verdict,
                           "positions": e.positions(&s.values).iter().map(|p| p.map(|c| c as f32)).collect::<Vec<_>>()}),
                );
            }
            modes.insert(
                mode.into(),
                json!({"counts": per_count, "vertices": ev.vertices}),
            );
        }
        json!(modes)
    });
    eprintln!("solved {} heads in {:.1?}", heads.len(), t0.elapsed());

    // the optimiser's own time on one thread: the full set, 12 sliders, both starts, on the first 8 heads
    let timing: Vec<f64> = heads
        .iter()
        .take(8)
        .map(|h| {
            let ev = Evaluator::new(&full_set, &sm, &h.start, &mesh.triangles, h.frame);
            let t = Instant::now();
            let n = ev.measure(&[]).ok();
            let begin: Vec<f64> = n
                .map(|n| {
                    NAMED
                        .iter()
                        .map(|&k| h.kotor.proportions[k] - n.proportions[k])
                        .collect()
                })
                .unwrap_or_default();
            let a = solve(&ev, &h.kotor, &marks, &full_spread, PRIOR, &begin);
            let b = solve(&ev, &h.kotor, &marks, &full_spread, PRIOR, &[]);
            let _ = a.cost.min(b.cost);
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    let t = Instant::now();
    let probe = Evaluator::new(
        &full_set,
        &sm,
        &heads[0].start,
        &mesh.triangles,
        heads[0].frame,
    );
    let prepare_ms = t.elapsed().as_secs_f64() * 1000.0;
    let t = Instant::now();
    for _ in 0..50 {
        let _ = probe.measure(&[0.5; MAX_COUNT]);
    }
    let measure_ms = t.elapsed().as_secs_f64() * 1000.0 / 50.0;
    eprintln!(
        "one head, one thread: solve {:?} ms, preparing a head {prepare_ms:.0} ms, one measure {measure_ms:.2} ms",
        timing.iter().map(|x| x.round()).collect::<Vec<_>>()
    );

    let timings = json!({"solve_ms": timing, "prepare_ms": prepare_ms, "measure_ms": measure_ms});
    let report = finish(
        &timings,
        &heads,
        &mesh,
        &zone,
        &sm,
        &full_set,
        &full_spread,
        &marks,
        &spread_p,
        &spread_o,
        &check1.0,
        &solved,
        &input,
        &web,
        &base_nose,
    );
    std::fs::write(
        data.join("report.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
    std::fs::write(
        web.join("page.json"),
        serde_json::to_string(&slim(report)).unwrap(),
    )
    .unwrap();
    eprintln!("done in {:.1?}", t0.elapsed());
}

/// The report without what the page does not draw: the made noses keep their proportions and landmarks, and the sampled noses their
/// proportions.
fn slim(mut report: Value) -> Value {
    let keep = |n: &mut Value, keys: &[&str]| {
        if let Some(o) = n.as_object_mut() {
            o.retain(|k, _| keys.contains(&k.as_str()));
        }
    };
    for h in report["heads"].as_array_mut().unwrap() {
        for (_, mode) in h["solved"].as_object_mut().unwrap() {
            for (_, c) in mode["counts"].as_object_mut().unwrap() {
                keep(&mut c["nose"], &["proportions", "marks"]);
            }
        }
    }
    for s in report["samples"].as_array_mut().unwrap() {
        keep(&mut s["nose"], &["proportions"]);
    }
    report["check1"].as_object_mut().unwrap().remove("heads");
    report
}

/// Runs `f` for 0..n on `threads` threads and returns the results in order.
fn parallel<T: Send>(n: usize, threads: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<Option<T>>> =
        (0..n).map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..threads {
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
    out.into_iter()
        .map(|m| m.into_inner().unwrap().unwrap())
        .collect()
}

fn sd(v: &[f64]) -> f64 {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
}

/// Each proportion's gap and each outline's difference, whether each passes, and the misses in words.
/// Loop's subdivision of a triangle mesh, once: smoother and four times as many triangles.
fn loop_subdivide(v: &[V3], f: &[[u32; 3]]) -> (Vec<V3>, Vec<[u32; 3]>) {
    use std::collections::HashMap;
    let mut edge_faces: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for (fi, t) in f.iter().enumerate() {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            edge_faces
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(fi as u32);
        }
    }
    let n = v.len();
    let mut nb: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut boundary_nb: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (&(a, b), fs) in &edge_faces {
        nb[a as usize].push(b);
        nb[b as usize].push(a);
        if fs.len() == 1 {
            boundary_nb[a as usize].push(b);
            boundary_nb[b as usize].push(a);
        }
    }
    let mut out: Vec<V3> = (0..n)
        .map(|i| {
            if !boundary_nb[i].is_empty() {
                if boundary_nb[i].len() == 2 {
                    let s = add(v[boundary_nb[i][0] as usize], v[boundary_nb[i][1] as usize]);
                    return add(scale(v[i], 0.75), scale(s, 0.125));
                }
                return v[i];
            }
            let k = nb[i].len() as f64;
            let beta = if k > 3.0 { 3.0 / (8.0 * k) } else { 3.0 / 16.0 };
            let mut s = [0.0; 3];
            for &j in &nb[i] {
                s = add(s, v[j as usize]);
            }
            add(scale(v[i], 1.0 - k * beta), scale(s, beta))
        })
        .collect();
    let mut mid: HashMap<(u32, u32), u32> = HashMap::new();
    for (&(a, b), fs) in &edge_faces {
        let p = if fs.len() == 2 {
            let opp = |fi: u32| {
                f[fi as usize]
                    .iter()
                    .copied()
                    .find(|&x| x != a && x != b)
                    .unwrap()
            };
            let (c, d) = (opp(fs[0]), opp(fs[1]));
            add(
                scale(add(v[a as usize], v[b as usize]), 0.375),
                scale(add(v[c as usize], v[d as usize]), 0.125),
            )
        } else {
            scale(add(v[a as usize], v[b as usize]), 0.5)
        };
        mid.insert((a, b), out.len() as u32);
        out.push(p);
    }
    let m = |a: u32, b: u32| mid[&(a.min(b), a.max(b))];
    let mut tris = Vec::with_capacity(f.len() * 4);
    for t in f {
        let (ab, bc, ca) = (m(t[0], t[1]), m(t[1], t[2]), m(t[2], t[0]));
        tris.extend([[t[0], ab, ca], [t[1], bc, ab], [t[2], ca, bc], [ab, bc, ca]]);
    }
    (out, tris)
}

/// The butterfly subdivision of a triangle mesh, once. It keeps every vertex where it is and puts each new point on a smooth surface
/// through them, so the facets go and the shape stays. Where an edge lacks the full stencil (at a border) the new point is the
/// edge's middle.
fn butterfly(v: &[V3], f: &[[u32; 3]]) -> (Vec<V3>, Vec<[u32; 3]>) {
    use std::collections::HashMap;
    let mut edge_faces: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for (fi, t) in f.iter().enumerate() {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            edge_faces
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(fi as u32);
        }
    }
    let opposite = |fi: u32, a: u32, b: u32| {
        f[fi as usize]
            .iter()
            .copied()
            .find(|&x| x != a && x != b)
            .unwrap()
    };
    // the vertex across edge (a, b) from vertex c, if the edge has a second face
    let across = |a: u32, b: u32, c: u32| -> Option<u32> {
        let fs = &edge_faces[&(a.min(b), a.max(b))];
        if fs.len() != 2 {
            return None;
        }
        fs.iter().map(|&fi| opposite(fi, a, b)).find(|&x| x != c)
    };
    let mut out = v.to_vec();
    let mut mid: HashMap<(u32, u32), u32> = HashMap::new();
    for (&(a, b), fs) in &edge_faces {
        let middle = scale(add(v[a as usize], v[b as usize]), 0.5);
        let p = if fs.len() == 2 {
            let (c, d) = (opposite(fs[0], a, b), opposite(fs[1], a, b));
            match (
                across(a, c, b),
                across(b, c, a),
                across(a, d, b),
                across(b, d, a),
            ) {
                (Some(e), Some(g), Some(h), Some(k)) => {
                    let wing = add(
                        add(v[e as usize], v[g as usize]),
                        add(v[h as usize], v[k as usize]),
                    );
                    add(
                        add(middle, scale(add(v[c as usize], v[d as usize]), 0.125)),
                        scale(wing, -0.0625),
                    )
                }
                _ => middle,
            }
        } else {
            middle
        };
        mid.insert((a, b), out.len() as u32);
        out.push(p);
    }
    let m = |a: u32, b: u32| mid[&(a.min(b), a.max(b))];
    let mut tris = Vec::with_capacity(f.len() * 4);
    for t in f {
        let (ab, bc, ca) = (m(t[0], t[1]), m(t[1], t[2]), m(t[2], t[0]));
        tris.extend([[t[0], ab, ca], [t[1], bc, ab], [t[2], ca, bc], [ab, bc, ca]]);
    }
    (out, tris)
}

fn vertex_normals(v: &[V3], f: &[[u32; 3]]) -> Vec<V3> {
    let mut n = vec![[0.0; 3]; v.len()];
    for t in f {
        let c = cross(
            sub(v[t[1] as usize], v[t[0] as usize]),
            sub(v[t[2] as usize], v[t[0] as usize]),
        );
        for &i in t {
            n[i as usize] = add(n[i as usize], c);
        }
    }
    n.into_iter().map(normalize).collect()
}

/// Check 1: the measure ignores what nobody notices. For each head, its KOTOR mesh subdivided twice by the butterfly rule (facets
/// gone, vertices kept), the same with a bump of 0.5 mm on the cheek, our mesh wrapped onto it, and its mesh subdivided once by Loop's
/// rule (which also rounds the shape a little), against its KOTOR mesh. Returns the per-check summary and each proportion's and
/// outline's 95th percentile gap over the subdivided heads.
fn check_unnoticed(
    heads: &[Head],
    mesh: &HeadMesh,
    references: &[f64; 3],
) -> (Value, [f64; N_PROPORTIONS], [f64; 3]) {
    let mut rows: Vec<(String, Vec<[f64; N_PROPORTIONS + 3]>)> = vec![
        ("subdivided".into(), vec![]),
        ("bump".into(), vec![]),
        ("wrap".into(), vec![]),
        ("rounded".into(), vec![]),
    ];
    let mut per_head = Vec::new();
    for h in heads {
        let (sv, sf) = butterfly(&h.kotor_v, &h.kotor_f);
        let (sv, sf) = butterfly(&sv, &sf);
        let sub_nose = measure(&Surface::new(&sv, &sf, &h.frame));
        let (lv, lf) = loop_subdivide(&h.kotor_v, &h.kotor_f);
        let rounded = measure(&Surface::new(&lv, &lf, &h.frame));
        let (bv, bf) = (sv.clone(), sf.clone());
        // a bump of 0.5 mm, 4 mm across (one spread of a Gaussian), on the cheek beside the wing, 28 mm out and 40 mm below the eyes
        let normals = vertex_normals(&bv, &bf);
        let centre_local = [28.0, -40.0, 0.0];
        let local: Vec<V3> = bv.iter().map(|&p| h.frame.local(p)).collect();
        let centre = local
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                ((p[0] - centre_local[0]).powi(2) + (p[1] - centre_local[1]).powi(2)).sqrt() < 4.0
            })
            .max_by(|a, b| a.1[2].total_cmp(&b.1[2]))
            .map(|(i, _)| local[i])
            .unwrap_or(centre_local);
        let bumped: Vec<V3> = bv
            .iter()
            .zip(&local)
            .zip(&normals)
            .map(|((p, q), n)| {
                let d2 = (0..3).map(|c| (q[c] - centre[c]).powi(2)).sum::<f64>();
                add(*p, scale(*n, 0.0005 * (-0.5 * d2 / 16.0).exp()))
            })
            .collect();
        let plain = measure(&Surface::new(&bv, &bf, &h.frame));
        let bump = measure(&Surface::new(&bumped, &bf, &h.frame));
        let wrap = measure(&Surface::new(&h.wrap, &mesh.triangles, &h.frame));
        let gap = |a: &Nose, b: &Nose| -> [f64; N_PROPORTIONS + 3] {
            std::array::from_fn(|i| {
                if i < N_PROPORTIONS {
                    b.proportions[i] - a.proportions[i]
                } else {
                    outline_difference(i - N_PROPORTIONS, &a.outlines[i - N_PROPORTIONS],
                        &b.outlines[i - N_PROPORTIONS],
                        references[i - N_PROPORTIONS],
                    )
                }
            })
        };
        let mut entry = json!({"name": h.name});
        for (k, other, against) in [
            (0usize, &sub_nose, Some(&h.kotor)),
            (1, &bump, plain.as_ref().ok()),
            (2, &wrap, Some(&h.kotor)),
            (3, &rounded, Some(&h.kotor)),
        ] {
            match (other, against) {
                (Ok(o), Some(a)) => {
                    let g = gap(a, o);
                    entry[&rows[k].0] = json!(g.to_vec());
                    rows[k].1.push(g);
                }
                (Err(e), _) => entry[&rows[k].0] = json!(e),
                _ => entry[&rows[k].0] = json!("the plain mesh failed"),
            }
        }
        per_head.push(entry);
    }
    let summary: Vec<Value> = rows
        .iter()
        .map(|(name, gaps)| {
            let stats: Vec<Value> = (0..N_PROPORTIONS + 3)
                .map(|i| {
                    let mut a: Vec<f64> = gaps.iter().map(|g| g[i].abs()).collect();
                    a.sort_by(f64::total_cmp);
                    let q = |p: f64| a[((a.len() - 1) as f64 * p).round() as usize];
                    json!({"median": q(0.5), "p95": q(0.95), "max": a[a.len() - 1]})
                })
                .collect();
            json!({"what": name, "heads": gaps.len(), "stats": stats})
        })
        .collect();
    let p95 = |i: usize| -> f64 {
        let mut a: Vec<f64> = rows[0].1.iter().map(|g| g[i].abs()).collect();
        a.sort_by(f64::total_cmp);
        a[((a.len() - 1) as f64 * 0.95).round() as usize]
    };
    (
        json!({"summary": summary, "heads": per_head}),
        std::array::from_fn(p95),
        std::array::from_fn(|o| p95(N_PROPORTIONS + o)),
    )
}

#[allow(clippy::too_many_arguments)]
fn finish(
    timings: &Value,
    heads: &[Head],
    mesh: &HeadMesh,
    zone: &Zone,
    sm: &Smoother,
    set: &SliderSet,
    spread: &[f64],
    marks: &Scale,
    spread_p: &[f64; N_PROPORTIONS],
    spread_o: &[f64; 3],
    check1: &Value,
    solved: &[Value],
    input: &Input,
    web: &std::path::Path,
    base_nose: &Nose,
) -> Value {
    // the scorecard
    let mut scorecard = Vec::new();
    for mode in ["held_out", "learned"] {
        for &count in &COUNTS {
            let mut made = 0;
            let mut total = 0;
            let mut misses = Vec::new();
            let mut per_descriptor = vec![0usize; N_PROPORTIONS + 3];
            for (h, s) in heads.iter().zip(solved) {
                let Some(m) = s.get(mode) else { continue };
                total += 1;
                let v = &m["counts"][count.to_string()]["verdict"];
                if v["made"].as_bool() == Some(true) {
                    made += 1;
                } else {
                    misses.push(json!({"name": h.name, "title": h.title, "words": v["words"]}));
                }
                if let Some(p) = v["pass"].as_array() {
                    for (k, x) in p.iter().enumerate() {
                        if x.as_bool() == Some(true) {
                            per_descriptor[k] += 1;
                        }
                    }
                }
            }
            eprintln!(
                "{mode:9} {count:2} sliders: {made} of {total} made; per measure {:?}",
                per_descriptor
            );
            scorecard.push(json!({"mode": mode, "count": count, "made": made, "of": total, "per_measure": per_descriptor, "misses": misses}));
        }
    }

    // slider ranges: the lowest and highest values the solved noses use with all sliders, held-out for the mods
    let mut set = set.clone();
    for (k, s) in set.sliders.iter_mut().enumerate() {
        let vals: Vec<f64> = solved
            .iter()
            .filter_map(|x| x.get("learned").or_else(|| x.get("held_out")))
            .filter_map(|m| {
                m["counts"][MAX_COUNT.to_string()]["values"]
                    .get(k)
                    .and_then(|v| v.as_f64())
            })
            .collect();
        s.low = vals.iter().copied().fold(f64::INFINITY, f64::min);
        s.high = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    }

    let base_built = sm.smooth(&mesh.base, SMOOTHING);
    let base_ev = Evaluator::new(
        &set,
        sm,
        &mesh.base,
        &mesh.triangles,
        Frame::from_eyes(mesh.eyes),
    );
    let mut rng = 0x2545F4914F6CDD1Du64;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        (rng >> 11) as f64 / (1u64 << 53) as f64
    };
    let base_normals = face_normals(&base_built, &mesh.triangles);
    let nostrils: Vec<bool> = mesh
        .triangles
        .iter()
        .map(|t| {
            t.iter().all(|&i| {
                mesh.hidden[mesh.group[i as usize] as usize] && base_built[i as usize][1] > 0.6
            })
        })
        .collect();
    // the leak table: each slider at each end of its range on the base head, how far each zone moves
    let mut leaks = Vec::new();
    for s in &set.sliders {
        for (end, value) in [("low", s.low), ("high", s.high)] {
            let built = set_one(&set, s, value, sm, &mesh.base);
            let mut zones = serde_json::Map::new();
            // only the vertices outside the nose's zone and its soft edge count, because the edge belongs to the nose
            for (zi, zname) in head_sliders::mesh::ZONES.iter().enumerate() {
                let d: Vec<f64> = (0..mesh.base.len())
                    .filter(|&i| {
                        mesh.zone[i] as usize == zi + 1 && !zone.contains(mesh.group[i] as usize)
                    })
                    .map(|i| length(sub(built[i], base_built[i])) * 1000.0)
                    .collect();
                if d.is_empty() {
                    continue;
                }
                let max = d.iter().copied().fold(0.0, f64::max);
                zones.insert(
                    (*zname).into(),
                    json!({"max": max, "mean": d.iter().sum::<f64>() / d.len() as f64}),
                );
            }
            let edge: Vec<f64> = (0..mesh.base.len())
                .filter(|&i| {
                    zone.weight[mesh.group[i] as usize] > 0.0
                        && zone.weight[mesh.group[i] as usize] < 1.0
                })
                .map(|i| length(sub(built[i], base_built[i])) * 1000.0)
                .collect();
            zones.insert("soft edge".into(), json!({"max": edge.iter().copied().fold(0.0, f64::max), "mean": edge.iter().sum::<f64>() / edge.len().max(1) as f64}));
            let f = faults(
                &built,
                &base_built,
                &base_normals,
                &mesh.triangles,
                &nostrils,
                &base_ev.vertices,
            );
            leaks.push(
                json!({"slider": s.name, "end": end, "value": value, "zones": zones, "faults": f}),
            );
        }
    }

    // sampled noses on the base head, and their checks
    // two ways of sampling inside the ranges: each slider anywhere in its range on its own ("box"), and like the KOTOR noses, from
    // a normal spread with the mean and covariance of the 57 solved values, clipped to the ranges ("like KOTOR")
    let solved_values: Vec<Vec<f64>> = solved
        .iter()
        .filter_map(|x| x.get("learned").or_else(|| x.get("held_out")))
        .filter_map(|m| {
            m["counts"][MAX_COUNT.to_string()]["values"]
                .as_array()
                .map(|a| a.iter().map(|v| v.as_f64().unwrap()).collect())
        })
        .collect();
    let k = set.sliders.len();
    let mean: Vec<f64> = (0..k)
        .map(|j| solved_values.iter().map(|v| v[j]).sum::<f64>() / solved_values.len() as f64)
        .collect();
    let mut cov = nalgebra::DMatrix::<f64>::zeros(k, k);
    for v in &solved_values {
        for a in 0..k {
            for b in 0..k {
                cov[(a, b)] += (v[a] - mean[a]) * (v[b] - mean[b]) / solved_values.len() as f64;
            }
        }
    }
    for a in 0..k {
        cov[(a, a)] += 1e-9;
    }
    let chol = cov.cholesky().expect("a covariance").l();
    let mut sample = |kind: &str| -> Value {
        let values: Vec<f64> = if kind == "box" {
            set.sliders
                .iter()
                .map(|s| s.low + next() * (s.high - s.low))
                .collect()
        } else {
            let z: Vec<f64> = (0..k)
                .map(|_| {
                    (-2.0 * next().max(1e-12).ln()).sqrt() * (std::f64::consts::TAU * next()).cos()
                })
                .collect();
            (0..k)
                .map(|a| {
                    (mean[a] + (0..k).map(|b| chol[(a, b)] * z[b]).sum::<f64>())
                        .clamp(set.sliders[a].low, set.sliders[a].high)
                })
                .collect()
        };
        let built = set.build(sm, &mesh.base, &values);
        json!({"kind": kind, "values": values, "faults": faults(&built, &base_built, &base_normals, &mesh.triangles, &nostrils, &base_ev.vertices), "nose": base_ev.measure(&values).ok()})
    };
    let mut samples: Vec<Value> = (0..64).map(|_| sample("like KOTOR")).collect();
    samples.extend((0..64).map(|_| sample("box")));
    for kind in ["like KOTOR", "box"] {
        let faulty = samples
            .iter()
            .filter(|s| {
                s["kind"] == kind
                    && s["faults"]["flipped"].as_u64().unwrap_or(0)
                        + s["faults"]["folded"].as_u64().unwrap_or(0)
                        + s["faults"]["nostrils_flipped"].as_u64().unwrap_or(0)
                        > 0
            })
            .count();
        eprintln!("sampled 64 noses {kind}, {faulty} with a fault");
    }
    let kotor_faults: Vec<Value> = heads
        .iter()
        .zip(solved)
        .filter_map(|(h, s)| {
            let m = s.get("learned").or_else(|| s.get("held_out"))?;
            let values: Vec<f64> = m["counts"][MAX_COUNT.to_string()]["values"].as_array()?.iter().map(|v| v.as_f64().unwrap()).collect();
            let start_built = sm.smooth(&h.start, SMOOTHING);
            let built = set.build(sm, &h.start, &values);
            let n = face_normals(&start_built, &mesh.triangles);
            Some(json!({"name": h.name, "faults": faults(&built, &start_built, &n, &mesh.triangles, &nostrils, &base_ev.vertices)}))
        })
        .collect();

    // the same checks on the earlier fits themselves, against the base head, as a yardstick
    let fit_faults: Vec<Value> = heads
        .iter()
        .map(|h| {
            let fit = input.points(&format!("heads/{}/fit_smooth", h.name));
            json!({"name": h.name, "faults": faults(&fit, &base_built, &base_normals, &mesh.triangles, &nostrils, &base_ev.vertices)})
        })
        .collect();

    // the noticing strips: on the base head, one proportion changed in even steps while the other proportions stay put
    let strips = noticing_strips(&base_ev, spread, spread_p);

    // check 3: copies of a head score as that head; and how many kinds of nose KOTOR has
    let twins: Vec<Value> = input.manifest["twins"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| {
            let name = t["name"].as_str()?;
            let of = t["of"].as_str()?;
            let h = heads.iter().find(|h| h.name == of)?;
            let v = input.points(&format!("twins/{name}/kotor_v"));
            let f = input.triangles(&format!("twins/{name}/kotor_f"));
            let n = measure(&Surface::new(&v, &f, &h.frame));
            // how far the copy's nose lies from the test head's: the largest gap from a vertex of the copy near the nose to the
            // nearest vertex of the test head, in the frame of the eyes
            let near_nose = |p: V3| p[0].abs() < 25.0 && p[1] > -75.0 && p[1] < 10.0 && p[2] > 5.0;
            let own: Vec<V3> = h.kotor_v.iter().map(|&p| h.frame.local(p)).collect();
            let nose_gap = v
                .iter()
                .map(|&p| h.frame.local(p))
                .filter(|&p| near_nose(p))
                .map(|p| own.iter().map(|&q| length(sub(p, q))).fold(f64::INFINITY, f64::min))
                .fold(0.0, f64::max);
            Some(json!({"name": name, "of": of, "gap_mm": t["median_gap_mm"], "nose_gap_mm": nose_gap, "same_topology": t["same_topology"],
                        "verdict": n.as_ref().ok().map(|n| judge(n, &h.kotor, marks)), "error": n.err()}))
        })
        .collect();
    let named_pairs = [
        ("mod_tcs_bastila", "p_bastilah"),
        ("mod_improved_bastila", "p_bastilah"),
        ("comm_a_f2", "comm_a_f"),
    ];
    let pairs_known: Vec<Value> = named_pairs
        .iter()
        .filter_map(|(a, b)| {
            let ha = heads.iter().find(|h| h.name == *a)?;
            let hb = heads.iter().find(|h| h.name == *b)?;
            Some(json!({"name": a, "of": b, "verdict": judge(&ha.kotor, &hb.kotor, marks)}))
        })
        .collect();
    let n = heads.len();
    let dist = |a: &Nose, b: &Nose| -> (f64, f64) {
        let g = gaps(a, b, marks);
        let mut worst: f64 = (0..N_PROPORTIONS).map(|i| g[i].abs()).fold(0.0, f64::max);
        for o in 0..3 {
            worst = worst.max(
                outline_difference(o, &a.outlines[o], &b.outlines[o], marks.references[o])
                    / marks.outlines[o],
            );
        }
        let rms = (g.iter().map(|x| x * x).sum::<f64>() / (N_PROPORTIONS + 3) as f64).sqrt();
        (worst, rms)
    };
    let mut d = vec![vec![(0.0, 0.0); n]; n];
    for i in 0..n {
        for j in 0..n {
            d[i][j] = dist(&heads[i].kotor, &heads[j].kotor);
        }
    }
    // kinds: complete linkage, two noses join a kind when every proportion and outline is within its mark of every member's
    let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    loop {
        let mut best = (f64::INFINITY, 0, 0);
        for a in 0..clusters.len() {
            for b in a + 1..clusters.len() {
                let link = clusters[a]
                    .iter()
                    .flat_map(|&i| clusters[b].iter().map(move |&j| (i, j)))
                    .map(|(i, j)| d[i][j].0)
                    .fold(0.0, f64::max);
                if link < best.0 {
                    best = (link, a, b);
                }
            }
        }
        if best.0 > 1.0 {
            break;
        }
        let merged = clusters.remove(best.2);
        clusters[best.1].extend(merged);
    }
    clusters.sort_by_key(|c| std::cmp::Reverse(c.len()));
    let kinds: Vec<Vec<&str>> = clusters
        .iter()
        .map(|c| c.iter().map(|&i| heads[i].name.as_str()).collect())
        .collect();
    eprintln!("{} kinds of nose among {} heads", kinds.len(), n);
    let mut all_pairs: Vec<(f64, usize, usize)> = (0..n)
        .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
        .map(|(i, j)| (d[i][j].1, i, j))
        .collect();
    all_pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    let pair = |p: &(f64, usize, usize)| json!({"a": heads[p.1].name, "b": heads[p.2].name, "distance": p.0, "verdict": judge(&heads[p.2].kotor, &heads[p.1].kotor, marks)});
    let alike: Vec<Value> = all_pairs.iter().take(8).map(pair).collect();
    let different: Vec<Value> = all_pairs.iter().rev().take(8).map(pair).collect();

    // the web data
    write_web(heads, mesh, &set, solved, web);

    json!({
        "measure": {
            "proportions": PROPORTIONS.iter().map(|p| json!({"key": p.key, "label": p.label, "low": p.low, "high": p.high, "unit": p.unit})).collect::<Vec<_>>(),
            "outlines": OUTLINES.iter().map(|o| json!({"key": o.key, "label": o.label, "points": o.points})).collect::<Vec<_>>(),
            "marks": marks, "provisional": true, "spread_proportions": spread_p, "spread_outlines": spread_o,
        },
        "base_nose": base_nose,
        "base_eyes": mesh.eyes,
        "base_frame": {"origin": Frame::from_eyes(mesh.eyes).origin, "axes": Frame::from_eyes(mesh.eyes).axes, "scale": Frame::from_eyes(mesh.eyes).scale},
        "heads": heads.iter().zip(solved).map(|(h, s)| {
            let mut s = s.clone();
            for mode in ["held_out", "learned"] {
                if let Some(m) = s.get_mut(mode) {
                    m.as_object_mut().unwrap().remove("vertices");
                    for (_, c) in m["counts"].as_object_mut().unwrap() {
                        c.as_object_mut().unwrap().remove("positions");
                    }
                }
            }
            json!({"name": h.name, "title": h.title, "mod": h.is_mod, "kotor": h.kotor,
                   "frame": {"origin": h.frame.origin, "axes": h.frame.axes, "scale": h.frame.scale}, "eyes": h.eyes, "solved": s})
        }).collect::<Vec<_>>(),
        "scorecard": scorecard,
        "timings": timings,
        "sliders": set.sliders.iter().map(|s| json!({"name": s.name, "kind": s.kind, "unit": s.unit, "low": s.low, "high": s.high})).collect::<Vec<_>>(),
        "spread": spread,
        "leaks": leaks,
        "samples": samples,
        "kotor_faults": kotor_faults,
        "fit_faults": fit_faults,
        "strips": strips,
        "check1": check1,
        "check3": {"twins": twins, "pairs": pairs_known, "kinds": kinds},
        "check4": {"alike": alike, "different": different},
    })
}

fn set_one(
    set: &SliderSet,
    s: &head_sliders::set::Slider,
    value: f64,
    sm: &Smoother,
    start: &[V3],
) -> Vec<V3> {
    let one = SliderSet {
        vertex_count: set.vertex_count,
        sliders: vec![s.clone()],
        fields: set.fields.clone(),
    };
    one.build(sm, start, &[value])
}

fn face_normals(p: &[V3], tris: &[[u32; 3]]) -> Vec<V3> {
    tris.iter()
        .map(|t| {
            normalize(cross(
                sub(p[t[1] as usize], p[t[0] as usize]),
                sub(p[t[2] as usize], p[t[0] as usize]),
            ))
        })
        .collect()
}

/// Faults of a built head against the head it started from, among the triangles near the nose: triangles turned over (their normal
/// turned by more than 90 degrees), folds (neighbouring triangles meeting at more than 90 degrees where they met at under 45), and
/// nostril triangles turned over.
fn faults(
    built: &[V3],
    before: &[V3],
    before_normals: &[V3],
    tris: &[[u32; 3]],
    nostrils: &[bool],
    near: &[u32],
) -> Value {
    let near_set: std::collections::HashSet<u32> = near.iter().copied().collect();
    let n = face_normals(built, tris);
    let mut flipped = 0;
    let mut nostrils_flipped = 0;
    let mut at: Vec<V3> = Vec::new();
    let centre = |t: &[u32; 3]| {
        scale(
            add(
                add(before[t[0] as usize], before[t[1] as usize]),
                before[t[2] as usize],
            ),
            1.0 / 3.0,
        )
    };
    for (k, t) in tris.iter().enumerate() {
        if !t.iter().all(|i| near_set.contains(i)) {
            continue;
        }
        if head_sliders::dot(n[k], before_normals[k]) < 0.0 {
            if nostrils[k] {
                nostrils_flipped += 1;
            } else {
                flipped += 1;
                at.push(centre(t));
            }
        }
    }
    let mut edges: std::collections::HashMap<(u32, u32), Vec<usize>> =
        std::collections::HashMap::new();
    for (k, t) in tris.iter().enumerate() {
        if !t.iter().all(|i| near_set.contains(i)) || nostrils[k] {
            continue;
        }
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            edges.entry((a.min(b), a.max(b))).or_default().push(k);
        }
    }
    let mut folded = 0;
    for fs in edges.values() {
        if fs.len() == 2 {
            let now = head_sliders::dot(n[fs[0]], n[fs[1]]);
            let was = head_sliders::dot(before_normals[fs[0]], before_normals[fs[1]]);
            if now < 0.0 && was > 0.707 {
                folded += 1;
                at.push(centre(&tris[fs[0]]));
            }
        }
    }
    at.truncate(40);
    json!({"flipped": flipped, "folded": folded, "nostrils_flipped": nostrils_flipped, "at": at})
}

/// For four proportions, slider values that change that proportion in even steps on the base head while the other seven stay put:
/// the smallest change of the values (in spreads) whose linear effect is one unit of that proportion and none of the others.
fn noticing_strips(ev: &Evaluator, spread: &[f64], spread_p: &[f64; N_PROPORTIONS]) -> Value {
    let k = ev.slider_count();
    let n0 = ev.measure(&vec![0.0; k]).expect("the base nose");
    let mut jac = nalgebra::DMatrix::<f64>::zeros(N_PROPORTIONS, k);
    for j in 0..k {
        let h = 0.25 * spread[j];
        let mut up = vec![0.0; k];
        up[j] = h;
        let mut down = vec![0.0; k];
        down[j] = -h;
        let a = ev.measure(&up).expect("probe");
        let b = ev.measure(&down).expect("probe");
        for i in 0..N_PROPORTIONS {
            jac[(i, j)] = (a.proportions[i] - b.proportions[i]) / (2.0 * h) * spread[j];
        }
    }
    let mut out = Vec::new();
    for key in ["width_wings", "length", "projection", "dorsum"] {
        let i = PROPORTIONS.iter().position(|p| p.key == key).unwrap();
        let mut e = nalgebra::DVector::<f64>::zeros(N_PROPORTIONS);
        e[i] = 1.0;
        // minimum norm solution of J c = e, in spreads
        let jjt = &jac * jac.transpose()
            + nalgebra::DMatrix::<f64>::identity(N_PROPORTIONS, N_PROPORTIONS) * 1e-6;
        let c = jac.transpose() * jjt.cholesky().expect("independent proportions").solve(&e);
        let per: Vec<f64> = (0..k).map(|j| c[j] * spread[j]).collect();
        // ten steps of an eighth of KOTOR's spread, rounded to 0.05, so the strip ends 1.25 spreads from the base head
        let step = ((spread_p[i] / 8.0 * 20.0).round() / 20.0).max(0.05);
        let steps: Vec<Value> = (0..=10)
            .map(|s| {
                let values: Vec<f64> = per.iter().map(|x| x * step * s as f64).collect();
                let nose = ev.measure(&values).ok();
                let got = nose.as_ref().map(|n| n.proportions.iter().zip(&n0.proportions).map(|(a, b)| a - b).collect::<Vec<_>>());
                json!({"step": s, "asked": step * s as f64, "values": values, "measured_change": got})
            })
            .collect();
        out.push(json!({"key": key, "label": PROPORTIONS[i].label, "unit": PROPORTIONS[i].unit, "step": step, "steps": steps}));
    }
    json!(out)
}

/// Writes the files the nose page reads.
fn write_web(
    heads: &[Head],
    mesh: &HeadMesh,
    set: &SliderSet,
    solved: &[Value],
    web: &std::path::Path,
) {
    let f32s = |v: &[f32]| -> Vec<u8> { v.iter().flat_map(|x| x.to_le_bytes()).collect() };
    let flat = |p: &[V3]| -> Vec<f32> { p.iter().flat_map(|q| q.map(|c| c as f32)).collect() };
    std::fs::write(web.join("base.bin"), f32s(&flat(&mesh.base))).unwrap();
    std::fs::write(
        web.join("triangles.bin"),
        mesh.triangles
            .iter()
            .flat_map(|t| t.iter().flat_map(|i| i.to_le_bytes()))
            .collect::<Vec<u8>>(),
    )
    .unwrap();
    std::fs::write(
        web.join("keep.bin"),
        mesh.keep.iter().map(|&k| k as u8).collect::<Vec<u8>>(),
    )
    .unwrap();
    let mut starts = Vec::new();
    for h in heads {
        starts.extend(flat(&h.start));
    }
    std::fs::write(web.join("starts.bin"), f32s(&starts)).unwrap();
    std::fs::write(
        web.join("sliders.json"),
        serde_json::to_string(set).unwrap(),
    )
    .unwrap();
    // the made noses near the nose: per head, per mode, per count, the positions of the evaluator's vertices
    let mut made = Vec::new();
    let mut index = serde_json::Map::new();
    for (h, s) in heads.iter().zip(solved) {
        let mut entry = serde_json::Map::new();
        for mode in ["held_out", "learned"] {
            let Some(m) = s.get(mode) else { continue };
            let vertices = m["vertices"].clone();
            let mut counts = serde_json::Map::new();
            for &count in &COUNTS {
                let p = &m["counts"][count.to_string()]["positions"];
                let offset = made.len() / 3;
                for q in p.as_array().unwrap() {
                    for c in q.as_array().unwrap() {
                        made.push(c.as_f64().unwrap() as f32);
                    }
                }
                counts.insert(count.to_string(), json!(offset));
            }
            entry.insert(
                mode.into(),
                json!({"vertices": vertices, "offsets": counts}),
            );
        }
        index.insert(h.name.clone(), json!(entry));
    }
    std::fs::write(web.join("made.bin"), f32s(&made)).unwrap();
    std::fs::write(
        web.join("made.json"),
        serde_json::to_string(&index).unwrap(),
    )
    .unwrap();
    std::fs::write(
        web.join("heads.json"),
        serde_json::to_string(
            &heads
                .iter()
                .map(|h| json!({"name": h.name, "title": h.title, "mod": h.is_mod}))
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
    .unwrap();
}
