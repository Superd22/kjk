//! Step 2 of the fold work: what decides where KOTOR places a fold (paint track, fine detail). Analysis only.
//!
//! For each head, fold and side where the measure finds KOTOR's fold, the tool compares KOTOR's found path with four paths computed
//! from our fitted head:
//! - the search path, placed from the landmarks (the found path's offset from it is the measure's track);
//! - the old fixed line of `wrinkles.rs`, drawn by hand in the layout;
//! - the valley of our head's surface: on each line across the fold, the point that sinks most below the surface around it at a 3 mm
//!   scale;
//! - the darkest point of our shading layer's sky light on that line.
//! It writes `<data dir>/placement.json` with the numbers per head and side and the summary over heads.
//!
//! Usage: fold_place

#[path = "shared/bundle.rs"]
mod bundle;
#[path = "shared/fold_common.rs"]
mod fold_common;
#[path = "shared/fold_measure.rs"]
mod fold_measure;

use fold_common::*;
use fold_measure::{Frame, KotorSkin, STATION_MM};
use head_paint::folds::{FoldHead, FoldKind, KINDS};
use head_paint::layout_points::{LayoutLocator, length, sub};
use head_paint::shading::ShapeMaps;
use head_paint::surface::Surface;
use head_paint::wrinkles::HeadLines;
use serde_json::{Value, json};
use std::path::Path;

/// The wrinkle set of `wrinkles.rs` that drew each fold before.
fn old_set(kind: FoldKind) -> usize {
    match kind {
        FoldKind::Nasolabial => 2,
        FoldKind::Marionette => 3,
        FoldKind::Chin => 7,
        FoldKind::UnderEye => 1,
    }
}

fn distance_to_polyline(p: [f64; 3], line: &[[f64; 3]]) -> f64 {
    let mut best = f64::INFINITY;
    for w in line.windows(2) {
        let ab = sub(w[1], w[0]);
        let ap = sub(p, w[0]);
        let l2 = head_paint::layout_points::dot(ab, ab);
        let t = if l2 > 0.0 { (head_paint::layout_points::dot(ap, ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
        best = best.min(length([ap[0] - t * ab[0], ap[1] - t * ab[1], ap[2] - t * ab[2]]));
    }
    best
}

fn main() {
    let c = Common::load();
    let names = head_names();
    let locator = LayoutLocator::new(&c.uv, &c.tris);
    let rows: Vec<Value> = parallel(&names, 8, |name| {
        let positions = c.fitted(name);
        let head = FoldHead::new(&c.tris, &c.vmap, &positions).expect("our head holds the landmarks");
        let skin = KotorSkin::open(name, &head);
        let surface = Surface::new(&positions, &c.mesh_tris);
        let valley: Vec<f64> = (0..positions.len()).map(|i| if surface.is_used(i) { 1000.0 * surface.bump(i, 0.003) } else { 0.0 }).collect();
        let sky = ShapeMaps::compute(&positions, &c.mesh_tris).sky;
        let per_layout: Vec<[f64; 3]> = c.vmap.iter().map(|&m| positions[m]).collect();
        let old = HeadLines::new(&locator, &per_layout);
        let at = |values: &[f64], s: &fold_measure::Sample| -> f64 { (0..3).map(|k| s.barycentric[k] * values[c.vmap[c.tris[s.triangle][k] as usize]]).sum() };
        let mut out = Vec::new();
        for kind in KINDS {
            for side in 0..2 {
                let frame = Frame::new(&head, &c.uv, &c.tris, kind, side);
                let m = frame.measure(&head, &|s| skin.lightness(kind, s));
                let Some((a, b)) = m.run.filter(|_| m.present) else { continue };
                let old_lines: Vec<&[[f64; 3]]> = old.lines.iter().filter(|l| l.set == old_set(kind) && (l.key & 1) as usize == side).map(|l| l.points.as_slice()).collect();
                let (mut d_old, mut d_valley, mut d_sky, mut u_valley, mut u_sky) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
                for s in a..=b {
                    let k = m.track[s];
                    let argmin = |values: &[f64]| -> Option<f64> {
                        (0..frame.width())
                            .filter(|&u| frame.offsets[u].abs() <= fold_measure::SEARCH_MM)
                            .filter_map(|u| frame.sample(s, u).map(|x| (frame.offsets[u], at(values, &x))))
                            .min_by(|p, q| p.1.total_cmp(&q.1))
                            .map(|p| p.0)
                    };
                    if let Some(v) = argmin(&valley) {
                        d_valley.push((v - k).abs());
                        u_valley.push(v);
                    }
                    if let Some(v) = argmin(&sky) {
                        d_sky.push((v - k).abs());
                        u_sky.push(v);
                    }
                    let (p, _, across) = frame.path.at(s as f64 * STATION_MM);
                    let q = [p[0] + k * across[0], p[1] + k * across[1], p[2] + k * across[2]];
                    if let Some(d) = old_lines.iter().map(|l| distance_to_polyline(q, l)).min_by(f64::total_cmp) {
                        d_old.push(d);
                    }
                }
                let track: Vec<f64> = m.track[a..=b].to_vec();
                let n = track.len();
                let fractions: Vec<f64> = [0.0, 0.25, 0.5, 0.75, 1.0].iter().map(|f| track[((n - 1) as f64 * f).round() as usize]).collect();
                let l = &head.landmarks;
                out.push(json!({
                    "name": name, "fold": kind.id(), "side": side,
                    "depth": m.depth, "length_mm": m.length_mm,
                    "start_station_mm": a as f64 * STATION_MM, "end_station_mm": b as f64 * STATION_MM,
                    "start_from_mark": m.start_from_mark, "end_from_mark": m.end_from_mark,
                    "mean_offset": mean(&track),
                    "offset_at_fractions": fractions,
                    "shape_spread": sd(&track),
                    "mean_abs_offset_search": mean(&track.iter().map(|x| x.abs()).collect::<Vec<_>>()),
                    "mean_distance_old_line": mean(&d_old),
                    "mean_distance_valley": mean(&d_valley),
                    "mean_distance_sky": mean(&d_sky),
                    "mean_valley_offset": mean(&u_valley),
                    "mean_sky_offset": mean(&u_sky),
                    "wing_to_corner_out_mm": head_paint::layout_points::dot(sub(l.mouth_corner[side], l.nose_wing[side]), l.frame.out(side)),
                    "mouth_half_width_mm": 0.5 * length(sub(l.mouth_corner[1], l.mouth_corner[0])),
                }));
            }
        }
        println!("{name}");
        Value::Array(out)
    });
    let rows: Vec<Value> = rows.into_iter().flat_map(|v| v.as_array().unwrap().clone()).collect();
    let mut summary = serde_json::Map::new();
    for kind in KINDS {
        let r: Vec<&Value> = rows.iter().filter(|x| x["fold"] == kind.id()).collect();
        let col = |k: &str| -> Vec<f64> { r.iter().map(|x| x[k].as_f64().unwrap_or(f64::NAN)).filter(|v| v.is_finite()).collect() };
        let off = col("mean_offset");
        let pair_diff: Vec<f64> = r
            .iter()
            .filter(|x| x["side"] == 0)
            .filter_map(|x| r.iter().find(|y| y["side"] == 1 && y["name"] == x["name"]).map(|y| x["mean_offset"].as_f64().unwrap() - y["mean_offset"].as_f64().unwrap()))
            .collect();
        let both = |a: &str, b: &str| -> (Vec<f64>, Vec<f64>) {
            r.iter().filter_map(|x| Some((x[a].as_f64()?, x[b].as_f64()?))).filter(|(p, q)| p.is_finite() && q.is_finite()).unzip()
        };
        let (o1, v1) = both("mean_offset", "mean_valley_offset");
        let (o2, s2) = both("mean_offset", "mean_sky_offset");
        let (o3, w3) = both("mean_offset", "wing_to_corner_out_mm");
        let (o4, h4) = both("mean_offset", "mouth_half_width_mm");
        let (o5, d5) = both("mean_offset", "depth");
        let fr: Vec<Vec<f64>> = (0..5).map(|i| r.iter().filter_map(|x| x["offset_at_fractions"][i].as_f64()).collect()).collect();
        summary.insert(
            kind.id().into(),
            json!({
                "sides found": r.len(),
                "KOTOR's mean offset from the search path, mean over sides (mm)": mean(&off),
                "the same, spread between sides (sd, mm)": sd(&off),
                "left minus right, sd over heads with both (mm)": sd(&pair_diff),
                "heads with both sides": pair_diff.len(),
                "spread of the offset along one fold (mean sd, mm)": mean(&col("shape_spread")),
                "offset at 0, 25, 50, 75 and 100% of the fold: mean": fr.iter().map(|v| mean(v)).collect::<Vec<_>>(),
                "offset at 0, 25, 50, 75 and 100% of the fold: sd": fr.iter().map(|v| sd(v)).collect::<Vec<_>>(),
                "mean distance to the search path (mm)": mean(&col("mean_abs_offset_search")),
                "mean distance to the old fixed line (mm)": mean(&col("mean_distance_old_line")),
                "mean distance to the surface valley (mm)": mean(&col("mean_distance_valley")),
                "mean distance to the darkest sky light (mm)": mean(&col("mean_distance_sky")),
                "correlation of the offset with the valley's offset": correlation(&o1, &v1),
                "correlation of the offset with the darkest sky light's offset": correlation(&o2, &s2),
                "correlation of the offset with the nose wing to mouth corner distance outward": correlation(&o3, &w3),
                "correlation of the offset with the mouth's half width": correlation(&o4, &h4),
                "correlation of the offset with the fold's depth": correlation(&o5, &d5),
                "start from its landmark (out, up), mean": [mean(&r.iter().map(|x| x["start_from_mark"][0].as_f64().unwrap()).collect::<Vec<_>>()), mean(&r.iter().map(|x| x["start_from_mark"][1].as_f64().unwrap()).collect::<Vec<_>>())],
                "start from its landmark (out, up), sd": [sd(&r.iter().map(|x| x["start_from_mark"][0].as_f64().unwrap()).collect::<Vec<_>>()), sd(&r.iter().map(|x| x["start_from_mark"][1].as_f64().unwrap()).collect::<Vec<_>>())],
                "end from its landmark (out, up), mean": [mean(&r.iter().map(|x| x["end_from_mark"][0].as_f64().unwrap()).collect::<Vec<_>>()), mean(&r.iter().map(|x| x["end_from_mark"][1].as_f64().unwrap()).collect::<Vec<_>>())],
                "end from its landmark (out, up), sd": [sd(&r.iter().map(|x| x["end_from_mark"][0].as_f64().unwrap()).collect::<Vec<_>>()), sd(&r.iter().map(|x| x["end_from_mark"][1].as_f64().unwrap()).collect::<Vec<_>>())],
                "length (mm), mean and sd": [mean(&col("length_mm")), sd(&col("length_mm"))],
                "start along the search path (mm), mean and sd": [mean(&col("start_station_mm")), sd(&col("start_station_mm"))],
            }),
        );
    }
    let doc = json!({ "summary": summary, "rows": rows });
    std::fs::write(Path::new(DATA).join("placement.json"), serde_json::to_string_pretty(&doc).unwrap()).unwrap();
    println!("{}", serde_json::to_string_pretty(&doc["summary"]).unwrap());
}
