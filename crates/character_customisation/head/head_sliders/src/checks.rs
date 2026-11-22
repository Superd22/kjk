//! The checks of a feature's measure, and its provisional pass marks (native builds only).
//!
//! - Check 1: the measure ignores what nobody notices. Each KOTOR mesh is subdivided twice by the butterfly rule (facets gone, vertices
//!   kept), the same gets a bump of 0.5 mm on a part of the face the measure should not see, our mesh wrapped onto it is measured, and
//!   the mesh subdivided once by Loop's rule (which also rounds the shape a little) is measured. Each is compared with KOTOR's own mesh.
//! - Check 3: copies of a test head that BioWare shipped score as that head, and heads that share one mesh score the same.
//! - Check 4: the pairs of heads the measure calls most alike and most different, to be looked at.
//!
//! The provisional pass marks are a quarter of KOTOR's spread, or twice the 95th percentile of check 1's subdivided wobble if larger.

use crate::feature::{Feature, KotorExtras, Measured, Scale, outline_difference, outline_size};
use crate::heads::{World, parallel, sd, threads};
use crate::measure::Surface;
use crate::mesh::input::Input;
use crate::solve::gaps;
use crate::verdict::judge;
use crate::{V3, add, cross, length, normalize, scale, sub};
use serde_json::{Value, json};
use std::collections::HashMap;

/// The marks and spreads of a feature, as `marks.json` holds them.
pub struct Marks {
    pub marks: Scale,
    pub spread_p: Vec<f64>,
    pub spread_o: Vec<f64>,
}

impl Marks {
    pub fn read(path: &std::path::Path) -> Self {
        let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap();
        Self {
            marks: serde_json::from_value(v["marks"].clone()).unwrap(),
            spread_p: serde_json::from_value(v["spread_proportions"].clone()).unwrap(),
            spread_o: serde_json::from_value(v["spread_outlines"].clone()).unwrap(),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({"marks": self.marks, "spread_proportions": self.spread_p, "spread_outlines": self.spread_o})
    }
}

/// The butterfly subdivision of a triangle mesh, once. It keeps every vertex where it is and puts each new point on a smooth surface
/// through them, so the facets go and the shape stays. Where an edge lacks the full stencil (at a border) the new point is the
/// edge's middle. Also returns the two ends of the edge each new vertex was made on.
pub fn butterfly(v: &[V3], f: &[[u32; 3]]) -> (Vec<V3>, Vec<[u32; 3]>, Vec<[u32; 2]>) {
    let mut edge_faces: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for (fi, t) in f.iter().enumerate() {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(fi as u32);
        }
    }
    let opposite = |fi: u32, a: u32, b: u32| f[fi as usize].iter().copied().find(|&x| x != a && x != b).unwrap();
    let across = |a: u32, b: u32, c: u32| -> Option<u32> {
        let fs = &edge_faces[&(a.min(b), a.max(b))];
        if fs.len() != 2 {
            return None;
        }
        fs.iter().map(|&fi| opposite(fi, a, b)).find(|&x| x != c)
    };
    let mut out = v.to_vec();
    let mut parents = Vec::new();
    let mut mid: HashMap<(u32, u32), u32> = HashMap::new();
    for (&(a, b), fs) in &edge_faces {
        let middle = scale(add(v[a as usize], v[b as usize]), 0.5);
        let p = if fs.len() == 2 {
            let (c, d) = (opposite(fs[0], a, b), opposite(fs[1], a, b));
            match (across(a, c, b), across(b, c, a), across(a, d, b), across(b, d, a)) {
                (Some(e), Some(g), Some(h), Some(k)) => {
                    let wing = add(add(v[e as usize], v[g as usize]), add(v[h as usize], v[k as usize]));
                    add(add(middle, scale(add(v[c as usize], v[d as usize]), 0.125)), scale(wing, -0.0625))
                }
                _ => middle,
            }
        } else {
            middle
        };
        mid.insert((a, b), out.len() as u32);
        out.push(p);
        parents.push([a, b]);
    }
    let m = |a: u32, b: u32| mid[&(a.min(b), a.max(b))];
    let mut tris = Vec::with_capacity(f.len() * 4);
    for t in f {
        let (ab, bc, ca) = (m(t[0], t[1]), m(t[1], t[2]), m(t[2], t[0]));
        tris.extend([[t[0], ab, ca], [t[1], bc, ab], [t[2], ca, bc], [ab, bc, ca]]);
    }
    (out, tris, parents)
}

/// Loop's subdivision of a triangle mesh, once: smoother and four times as many triangles. Also returns the new vertices' edges.
pub fn loop_subdivide(v: &[V3], f: &[[u32; 3]]) -> (Vec<V3>, Vec<[u32; 3]>, Vec<[u32; 2]>) {
    let mut edge_faces: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for (fi, t) in f.iter().enumerate() {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(fi as u32);
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
    let mut parents = Vec::new();
    let mut mid: HashMap<(u32, u32), u32> = HashMap::new();
    for (&(a, b), fs) in &edge_faces {
        let p = if fs.len() == 2 {
            let opp = |fi: u32| f[fi as usize].iter().copied().find(|&x| x != a && x != b).unwrap();
            let (c, d) = (opp(fs[0]), opp(fs[1]));
            add(scale(add(v[a as usize], v[b as usize]), 0.375), scale(add(v[c as usize], v[d as usize]), 0.125))
        } else {
            scale(add(v[a as usize], v[b as usize]), 0.5)
        };
        mid.insert((a, b), out.len() as u32);
        out.push(p);
        parents.push([a, b]);
    }
    let m = |a: u32, b: u32| mid[&(a.min(b), a.max(b))];
    let mut tris = Vec::with_capacity(f.len() * 4);
    for t in f {
        let (ab, bc, ca) = (m(t[0], t[1]), m(t[1], t[2]), m(t[2], t[0]));
        tris.extend([[t[0], ab, ca], [t[1], bc, ab], [t[2], ca, bc], [ab, bc, ca]]);
    }
    (out, tris, parents)
}

pub fn vertex_normals(v: &[V3], f: &[[u32; 3]]) -> Vec<V3> {
    let mut n = vec![[0.0; 3]; v.len()];
    for t in f {
        let c = cross(sub(v[t[1] as usize], v[t[0] as usize]), sub(v[t[2] as usize], v[t[0] as usize]));
        for &i in t {
            n[i as usize] = add(n[i as usize], c);
        }
    }
    n.into_iter().map(normalize).collect()
}

/// A KOTOR head's extras carried onto a subdivided copy of its mesh: a new vertex is skin when both ends of its edge are, and each
/// new triangle has its parent's colour.
struct Carried {
    skin: Option<Vec<bool>>,
    colour: Option<Vec<V3>>,
}

fn carry(skin: Option<&[bool]>, colour: Option<&[V3]>, parents: &[[u32; 2]]) -> Carried {
    Carried {
        skin: skin.map(|s| s.iter().copied().chain(parents.iter().map(|p| s[p[0] as usize] && s[p[1] as usize])).collect()),
        colour: colour.map(|c| c.iter().flat_map(|&x| [x; 4]).collect()),
    }
}

/// The per-head data check 1 needs besides the world: the KOTOR head's skin and colours, its smoothed fit and our wrap.
struct HeadInputs {
    skin: Option<Vec<bool>>,
    colour: Option<Vec<V3>>,
    fit: Option<Vec<V3>>,
    wrap: Vec<V3>,
}

fn head_inputs(input: &Input, name: &str) -> HeadInputs {
    HeadInputs {
        skin: input
            .has(&format!("extra/heads/{name}/kotor_skin"))
            .then(|| input.u8s(&format!("extra/heads/{name}/kotor_skin")).iter().map(|&s| s != 0).collect()),
        colour: input.has(&format!("extra/heads/{name}/kotor_colour")).then(|| input.points(&format!("extra/heads/{name}/kotor_colour"))),
        fit: input.has(&format!("heads/{name}/fit_smooth")).then(|| input.points(&format!("heads/{name}/fit_smooth"))),
        wrap: input.points(&format!("heads/{name}/wrap")),
    }
}

/// The gap between two measured features, row by row: each proportion's difference (b minus a), then each outline's difference.
/// NaN where either cannot be read.
fn row_gaps(feature: &dyn Feature, a: &Measured, b: &Measured, references: &[f64]) -> Vec<f64> {
    let np = feature.proportions().len();
    (0..np)
        .map(|i| b.proportions[i] - a.proportions[i])
        .chain(feature.outlines().iter().enumerate().map(|(o, def)| {
            if a.outlines[o].is_empty() || b.outlines[o].is_empty() {
                f64::NAN
            } else {
                outline_difference(def, &a.outlines[o], &b.outlines[o], references[o])
            }
        }))
        .collect()
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted[((sorted.len() - 1) as f64 * q).round() as usize]
}

/// Check 1, and the marks it gives. Returns the report and the marks.
pub fn check1_and_marks(world: &World, input: &Input) -> (Value, Marks) {
    let feature = world.feature;
    let np = feature.proportions().len();
    let no = feature.outlines().len();
    let heads = &world.heads;
    let readable: Vec<usize> = (0..heads.len()).filter(|&i| heads[i].kotor_error.is_none()).collect();
    let references: Vec<f64> = feature
        .outlines()
        .iter()
        .enumerate()
        .map(|(o, def)| {
            let s: Vec<f64> =
                readable.iter().filter(|&&i| !heads[i].kotor.outlines[o].is_empty()).map(|&i| outline_size(def, &heads[i].kotor.outlines[o])).collect();
            s.iter().sum::<f64>() / s.len() as f64
        })
        .collect();
    let window = feature.window();
    let names = ["subdivided", "bump", "wrap", "rounded"];
    let per_head: Vec<(Value, Vec<Option<Vec<f64>>>)> = parallel(heads.len(), threads(), |hi| {
        let h = &heads[hi];
        let inp = head_inputs(input, &h.name);
        let measure_k = |v: &[V3], f: &[[u32; 3]], c: &Carried| -> Result<Measured, &'static str> {
            // the subdivided copies are read whole: the rows hair covers on KOTOR's own mesh are left out of the comparison anyway
            let extras = KotorExtras {
                name: Some(&h.name),
                triangles: None,
                fit_triangles: Some(&world.mesh.triangles),
                skin: c.skin.as_deref(),
                colour: c.colour.as_deref(),
                fit: inp.fit.as_deref(),
            };
            feature.measure_kotor(&Surface::within(v, f, &h.frame, &window), &extras, &h.frame, v)
        };
        let (sv, sf, p1) = butterfly(&h.kotor_v, &h.kotor_f);
        let c1 = carry(inp.skin.as_deref(), inp.colour.as_deref(), &p1);
        let (sv, sf, p2) = butterfly(&sv, &sf);
        let c2 = carry(c1.skin.as_deref(), c1.colour.as_deref(), &p2);
        let sub_m = measure_k(&sv, &sf, &c2);
        let (lv, lf, pl) = loop_subdivide(&h.kotor_v, &h.kotor_f);
        let cl = carry(inp.skin.as_deref(), inp.colour.as_deref(), &pl);
        let rounded = measure_k(&lv, &lf, &cl);
        // a bump of 0.5 mm, 4 mm across (one spread of a Gaussian), where the feature says
        let normals = vertex_normals(&sv, &sf);
        let centre_local = feature.check_bump();
        let local: Vec<V3> = sv.iter().map(|&p| h.frame.local(p)).collect();
        let centre = local
            .iter()
            .enumerate()
            .filter(|(_, p)| ((p[0] - centre_local[0]).powi(2) + (p[1] - centre_local[1]).powi(2)).sqrt() < 4.0)
            .max_by(|a, b| a.1[2].total_cmp(&b.1[2]))
            .map(|(i, _)| local[i])
            .unwrap_or(centre_local);
        let bumped: Vec<V3> = sv
            .iter()
            .zip(&local)
            .zip(&normals)
            .map(|((p, q), n)| {
                let d2 = (0..3).map(|c| (q[c] - centre[c]).powi(2)).sum::<f64>();
                add(*p, scale(*n, 0.0005 * (-0.5 * d2 / 16.0).exp()))
            })
            .collect();
        let plain = measure_k(&sv, &sf, &c2);
        let bump = measure_k(&bumped, &sf, &c2);
        let wrap = feature.measure(&Surface::within(&inp.wrap, &world.mesh.triangles, &h.frame, &window));
        let kotor = if h.kotor_error.is_none() { Some(&h.kotor) } else { None };
        let mut entry = json!({"name": h.name});
        let mut rows = Vec::new();
        for (k, other, against) in
            [(0usize, &sub_m, kotor), (1, &bump, plain.as_ref().ok()), (2, &wrap, kotor), (3, &rounded, kotor)]
        {
            match (other, against) {
                (Ok(o), Some(a)) => {
                    let g = row_gaps(feature, a, o, &references);
                    entry[names[k]] = json!(g.iter().map(|x| if x.is_nan() { Value::Null } else { json!(x) }).collect::<Vec<_>>());
                    rows.push(Some(g));
                }
                (Err(e), _) => {
                    entry[names[k]] = json!(e);
                    rows.push(None);
                }
                _ => {
                    entry[names[k]] = json!("the plain mesh failed");
                    rows.push(None);
                }
            }
        }
        (entry, rows)
    });
    let stats_of = |k: usize, i: usize| -> Vec<f64> {
        let mut a: Vec<f64> =
            per_head.iter().filter_map(|(_, r)| r[k].as_ref().map(|g| g[i].abs())).filter(|x| !x.is_nan()).collect();
        a.sort_by(f64::total_cmp);
        a
    };
    let summary: Vec<Value> = names
        .iter()
        .enumerate()
        .map(|(k, name)| {
            let stats: Vec<Value> = (0..np + no)
                .map(|i| {
                    let a = stats_of(k, i);
                    json!({"median": quantile(&a, 0.5), "p95": quantile(&a, 0.95), "max": a.last().copied().unwrap_or(f64::NAN), "heads": a.len()})
                })
                .collect();
            json!({"what": name, "heads": per_head.iter().filter(|(_, r)| r[k].is_some()).count(), "stats": stats})
        })
        .collect();
    let noise: Vec<f64> = (0..np + no).map(|i| quantile(&stats_of(0, i), 0.95)).collect();
    let spread_p: Vec<f64> =
        (0..np).map(|i| sd(&heads.iter().filter(|h| h.kotor_error.is_none()).map(|h| h.kotor.proportions[i]).collect::<Vec<_>>())).collect();
    // an outline's spread over the KOTOR heads: the root mean square difference between two of them, over the square root of 2
    let spread_o: Vec<f64> = feature
        .outlines()
        .iter()
        .enumerate()
        .map(|(o, def)| {
            let (mut sum, mut count) = (0.0, 0.0);
            for a in heads {
                for b in heads {
                    if a.kotor.outlines[o].is_empty() || b.kotor.outlines[o].is_empty() {
                        continue;
                    }
                    sum += outline_difference(def, &a.kotor.outlines[o], &b.kotor.outlines[o], references[o]).powi(2);
                    count += 1.0;
                }
            }
            (sum / count / 2.0).sqrt()
        })
        .collect();
    let marks = Scale {
        proportions: (0..np).map(|i| (0.25 * spread_p[i]).max(2.0 * noise[i])).collect(),
        outlines: (0..no).map(|o| (0.25 * spread_o[o]).max(2.0 * noise[np + o])).collect(),
        references,
    };
    let report = json!({"summary": summary, "heads": per_head.iter().map(|(e, _)| e.clone()).collect::<Vec<_>>()});
    (report, Marks { marks, spread_p, spread_o })
}

/// Checks 3 and 4: copies of a test head and heads that share a mesh, how many kinds of the feature KOTOR has, and the pairs the
/// measure calls most alike and most different.
pub fn check3_and_4(world: &World, input: &Input, marks: &Scale) -> Value {
    let feature = world.feature;
    let heads = &world.heads;
    let np = feature.proportions().len();
    let no = feature.outlines().len();
    let window = feature.window();
    let region = feature.region();
    let twins: Vec<Value> = input.manifest["twins"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| {
            let name = t["name"].as_str()?;
            let of = t["of"].as_str()?;
            let h = heads.iter().find(|h| h.name == of)?;
            let v = input.points(&format!("twins/{name}/kotor_v"));
            let f = input.triangles(&format!("twins/{name}/kotor_f"));
            let extras = KotorExtras { name: None, triangles: None, fit_triangles: None, skin: None, colour: None, fit: None };
            let n = feature.measure_kotor(&Surface::within(&v, &f, &h.frame, &window), &extras, &h.frame, &v);
            // how far the copy's feature lies from the test head's: the largest gap from a vertex of the copy in the feature's region to
            // the nearest vertex of the test head, in the frame of the eyes
            let own: Vec<V3> = h.kotor_v.iter().map(|&p| h.frame.local(p)).collect();
            let gap = v
                .iter()
                .map(|&p| h.frame.local(p))
                .filter(|&p| region.contains(p))
                .map(|p| own.iter().map(|&q| length(sub(p, q))).fold(f64::INFINITY, f64::min))
                .fold(0.0, f64::max);
            Some(json!({"name": name, "of": of, "gap_mm": t["median_gap_mm"], "feature_gap_mm": gap, "same_topology": t["same_topology"],
                        "verdict": n.as_ref().ok().map(|n| judge(feature, n, &h.kotor, marks)), "error": n.err()}))
        })
        .collect();
    // heads among the 57 whose KOTOR meshes are the same: same vertex count, and every vertex within 0.5 mm in the frame of the eyes
    let mut shared = Vec::new();
    for a in 0..heads.len() {
        for b in a + 1..heads.len() {
            let (ha, hb) = (&heads[a], &heads[b]);
            if ha.kotor_v.len() != hb.kotor_v.len() || ha.kotor_f.len() != hb.kotor_f.len() {
                continue;
            }
            let worst = ha
                .kotor_v
                .iter()
                .zip(&hb.kotor_v)
                .filter(|(p, _)| region.contains(ha.frame.local(**p)))
                .map(|(p, q)| length(sub(ha.frame.local(*p), hb.frame.local(*q))))
                .fold(0.0, f64::max);
            if worst < 0.5 {
                shared.push(json!({"a": ha.name, "b": hb.name, "mesh_gap_mm": worst, "verdict": judge(feature, &hb.kotor, &ha.kotor, marks)}));
            }
        }
    }
    let n = heads.len();
    let dist = |a: &Measured, b: &Measured| -> (f64, f64) {
        let g = gaps(feature, a, b, marks);
        let mut worst: f64 = (0..np).map(|i| g[i].abs()).fold(0.0, f64::max);
        let mut count = (0..np).filter(|&i| a.readable(i) && b.readable(i)).count();
        for (o, def) in feature.outlines().iter().enumerate() {
            if a.outlines[o].is_empty() || b.outlines[o].is_empty() {
                continue;
            }
            count += 1;
            worst = worst.max(outline_difference(def, &a.outlines[o], &b.outlines[o], marks.references[o]) / marks.outlines[o]);
        }
        if count < (np + no) / 2 {
            return (f64::INFINITY, f64::NAN);
        }
        let rms = (g.iter().map(|x| x * x).sum::<f64>() / count.max(1) as f64).sqrt();
        (worst, rms)
    };
    let usable: Vec<usize> = (0..n).filter(|&i| heads[i].kotor_error.is_none()).collect();
    let mut d = vec![vec![(0.0, 0.0); n]; n];
    for &i in &usable {
        for &j in &usable {
            d[i][j] = dist(&heads[i].kotor, &heads[j].kotor);
        }
    }
    // kinds: complete linkage, two heads join a kind when every proportion and outline is within its mark of every member's
    let mut clusters: Vec<Vec<usize>> = usable.iter().map(|&i| vec![i]).collect();
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
    let kinds: Vec<Vec<&str>> = clusters.iter().map(|c| c.iter().map(|&i| heads[i].name.as_str()).collect()).collect();
    let mut all_pairs: Vec<(f64, usize, usize)> =
        usable.iter().flat_map(|&i| usable.iter().filter(move |&&j| j > i).map(move |&j| (i, j))).map(|(i, j)| (d[i][j].1, i, j)).filter(|p| !p.0.is_nan()).collect();
    all_pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    let pair = |p: &(f64, usize, usize)| {
        json!({"a": heads[p.1].name, "b": heads[p.2].name, "distance": p.0, "verdict": judge(feature, &heads[p.2].kotor, &heads[p.1].kotor, marks)})
    };
    let alike: Vec<Value> = all_pairs.iter().take(8).map(pair).collect();
    let different: Vec<Value> = all_pairs.iter().rev().take(8).map(pair).collect();
    let _ = no;
    json!({"check3": {"twins": twins, "shared_meshes": shared, "kinds": kinds}, "check4": {"alike": alike, "different": different}})
}
