//! What KOTOR's skin colour varies in across the test heads (paint track, step C2). This is an analysis tool and never ships.
//!
//! Each head's KOTOR paint is read at the points of our split mesh, from the viewer data. Points that the paint transfer left unpainted
//! (grey) or that lie inside the mouth are dropped. For each head the tool finds the base tone, which is the median Lab colour of the
//! cheeks and the forehead, and the regional tint, which is each point's Lab colour minus the base tone, smoothed over the surface.
//! Across heads it finds the distance between base tones, each zone's mean tint, and the principal patterns of the tint field.
//!
//! Usage: skin_tone <viewer data dir> <rig.json> <out dir>
//! The viewer data dir holds common.json, zones.json and heads/<name>.json. The tool writes report.json and patterns.json into the out dir.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use head_paint::colour::{lab_to_srgb, srgb_to_lab};
use head_paint::mesh::{Neighbours, smooth_masked};
use nalgebra::{DMatrix, SymmetricEigen};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use std::time::Instant;

const SMOOTH_ROUNDS: usize = 20;
const PATTERNS_DRAWN: usize = 6;
const OUTSIDE_THE_MOUTH: f64 = 0.05;
// The colour of an unpainted point as the viewer data stores it: linear (0.2, 0.2, 0.2) encoded with gamma 2.2 and rounded to a byte.
const GREY: f64 = 123.0;

#[derive(Deserialize)]
struct Common {
    n: usize,
    tris: String,
    edges: String,
    heads: Vec<HeadName>,
}

#[derive(Deserialize)]
struct HeadName {
    name: String,
}

#[derive(Deserialize)]
struct Zones {
    zones: Vec<String>,
    zone: String,
}

#[derive(Deserialize)]
struct HeadFile {
    ours: Ours,
}

#[derive(Deserialize)]
struct Ours {
    colour: String,
}

#[derive(Deserialize)]
struct Rig {
    #[serde(rename = "mouthInside")]
    mouth_inside: Vec<(usize, f64)>,
}

fn u16s(encoded: &str) -> Vec<u16> {
    let bytes = STANDARD.decode(encoded).expect("base64");
    bytes.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect()
}

fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    let n = values.len();
    if n % 2 == 1 { values[n / 2] } else { (values[n / 2 - 1] + values[n / 2]) / 2.0 }
}

fn round(x: f64, digits: i32) -> f64 {
    let k = 10f64.powi(digits);
    (x * k).round() / k
}

fn rounded(v: &[f64], digits: i32) -> Vec<f64> {
    v.iter().map(|&x| round(x, digits)).collect()
}

fn distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn range(tones: &[[f64; 3]], channel: usize) -> Vec<f64> {
    let values = tones.iter().map(|t| t[channel]);
    vec![round(values.clone().fold(f64::INFINITY, f64::min), 1), round(values.fold(f64::NEG_INFINITY, f64::max), 1)]
}

fn to_bytes(rgb: [f64; 3]) -> [u8; 3] {
    rgb.map(|c| (c * 255.0).round() as u8)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: skin_tone <viewer data dir> <rig.json> <out dir>");
        std::process::exit(2);
    }
    let (data, out) = (Path::new(&args[1]), Path::new(&args[3]));
    let started = Instant::now();

    let common: Common = read(&data.join("common.json"));
    let zone_file: Zones = read(&data.join("zones.json"));
    let rig: Rig = read(Path::new(&args[2]));
    let triangles: Vec<[u32; 3]> = u16s(&common.tris).as_chunks::<3>().0.iter().map(|t| t.map(u32::from)).collect();
    let edges: Vec<[usize; 2]> = u16s(&common.edges).as_chunks::<2>().0.iter().map(|e| e.map(usize::from)).collect();
    let zone = STANDARD.decode(&zone_file.zone).expect("zones");
    let points = zone.len();
    assert_eq!(points, common.n + edges.len(), "zones.json does not fit common.json");

    let mut inside = vec![0.0; common.n];
    for &(v, d) in &rig.mouth_inside {
        inside[v] = d;
    }
    let in_mouth: Vec<bool> = (0..points)
        .map(|i| if i < common.n { inside[i] } else { (inside[edges[i - common.n][0]] + inside[edges[i - common.n][1]]) / 2.0 } > OUTSIDE_THE_MOUTH)
        .collect();
    let neighbours = Neighbours::new(&triangles, points);
    let zone_id = |name: &str| zone_file.zones.iter().position(|z| z == name).map(|i| i as u8);
    let base_zones = [zone_id("cheeks").expect("cheeks"), zone_id("forehead").expect("forehead")];

    let names: Vec<&str> = common.heads.iter().map(|h| h.name.as_str()).collect();
    let mut tones = Vec::new();
    let mut tints: Vec<Vec<[f64; 3]>> = Vec::new();
    let mut painted: Vec<Vec<bool>> = Vec::new();
    for name in &names {
        let head: HeadFile = read(&data.join("heads").join(format!("{name}.json")));
        let bytes = STANDARD.decode(&head.ours.colour).expect("colour");
        assert_eq!(bytes.len(), points * 3, "{name}: colour does not fit the mesh");
        let ok: Vec<bool> = bytes.as_chunks::<3>().0.iter().zip(&in_mouth).map(|(c, &m)| !m && c.iter().any(|&b| (f64::from(b) - GREY).abs() > 2.0)).collect();
        let lab: Vec<[f64; 3]> = bytes.as_chunks::<3>().0.iter().map(|c| srgb_to_lab(c.map(|b| f64::from(b) / 255.0))).collect();
        let skin: Vec<usize> = (0..points).filter(|&i| ok[i] && base_zones.contains(&zone[i])).collect();
        assert!(!skin.is_empty(), "{name}: no painted cheek or forehead point");
        let base: [f64; 3] = std::array::from_fn(|c| median(skin.iter().map(|&i| lab[i][c]).collect()));
        let offset: Vec<[f64; 3]> = lab.iter().map(|l| std::array::from_fn(|c| l[c] - base[c])).collect();
        tints.push(smooth_masked(&offset, &ok, &neighbours, SMOOTH_ROUNDS));
        println!("{name} {:.1} {:.1} {:.1}", base[0], base[1], base[2]);
        tones.push(base);
        painted.push(ok);
    }
    let heads = names.len();

    let mut delta_sum = 0.0;
    let mut pairs = 0;
    for i in 0..heads {
        for j in i + 1..heads {
            delta_sum += distance(&tones[i], &tones[j]);
            pairs += 1;
        }
    }
    let mean_delta = delta_sum / f64::from(pairs);

    // The points painted on at least nine heads in ten.
    let common_points: Vec<usize> = (0..points).filter(|&i| painted.iter().filter(|p| p[i]).count() as f64 / heads as f64 > 0.9).collect();
    let mut zone_tints = serde_json::Map::new();
    for (id, zone_name) in zone_file.zones.iter().enumerate() {
        let selected: Vec<usize> = common_points.iter().copied().filter(|&i| zone[i] == id as u8).collect();
        if selected.len() < 20 {
            continue;
        }
        let per_head: Vec<[f64; 3]> = (0..heads)
            .filter_map(|h| {
                let used: Vec<usize> = selected.iter().copied().filter(|&i| painted[h][i]).collect();
                (!used.is_empty()).then(|| std::array::from_fn(|c| used.iter().map(|&i| tints[h][i][c]).sum::<f64>() / used.len() as f64))
            })
            .collect();
        let mean: [f64; 3] = std::array::from_fn(|c| per_head.iter().map(|t| t[c]).sum::<f64>() / per_head.len() as f64);
        let spread = per_head.iter().map(|t| distance(t, &mean)).sum::<f64>() / per_head.len() as f64;
        zone_tints.insert(
            zone_name.clone(),
            json!({"mean tint (L, a, b)": rounded(&mean, 2), "spread between heads (delta E)": round(spread, 2), "points": selected.len(), "heads counted": per_head.len()}),
        );
    }

    // The principal patterns. The matrix is heads by (points times three), so its singular values come from the heads-by-heads Gram matrix.
    let width = common_points.len() * 3;
    let mut x = DMatrix::<f64>::zeros(heads, width);
    for h in 0..heads {
        for (k, &i) in common_points.iter().enumerate() {
            for c in 0..3 {
                x[(h, k * 3 + c)] = tints[h][i][c];
            }
        }
    }
    for k in 0..width {
        let mean = x.column(k).sum() / heads as f64;
        x.column_mut(k).add_scalar_mut(-mean);
    }
    let eigen = SymmetricEigen::new(&x * x.transpose());
    let mut order: Vec<usize> = (0..heads).collect();
    order.sort_by(|&a, &b| eigen.eigenvalues[b].total_cmp(&eigen.eigenvalues[a]));
    let total: f64 = eigen.eigenvalues.iter().map(|v| v.max(0.0)).sum();
    let share: Vec<f64> = order.iter().map(|&k| eigen.eigenvalues[k].max(0.0) / total).collect();

    let mut pattern_json = Vec::new();
    let mut loadings = serde_json::Map::new();
    let centre = [62.0, 14.0, 18.0];
    for (rank, &k) in order.iter().take(PATTERNS_DRAWN).enumerate() {
        // The pattern at each point is the head-score-weighted mean tint, scaled to one standard deviation across the heads.
        let singular = eigen.eigenvalues[k].max(0.0).sqrt();
        let direction = x.transpose() * eigen.eigenvectors.column(k) / singular.max(1e-12);
        let field = direction * singular / (heads as f64).sqrt();
        let mut by_zone = serde_json::Map::new();
        for (id, zone_name) in zone_file.zones.iter().enumerate() {
            let sel: Vec<usize> = common_points.iter().enumerate().filter(|&(_, &i)| zone[i] == id as u8).map(|(slot, _)| slot).collect();
            if sel.len() >= 20 {
                let mean: Vec<f64> = (0..3).map(|c| sel.iter().map(|&slot| field[slot * 3 + c]).sum::<f64>() / sel.len() as f64).collect();
                by_zone.insert(zone_name.clone(), json!(rounded(&mean, 2)));
            }
        }
        loadings.insert(format!("pattern {}", rank + 1), Value::Object(by_zone));
        let mut plus = vec![(0.35f64 * 255.0).round() as u8; points * 3];
        let mut minus = plus.clone();
        for (slot, &i) in common_points.iter().enumerate() {
            for (sign, colours) in [(1.0, &mut plus), (-1.0, &mut minus)] {
                let lab = std::array::from_fn(|c| centre[c] + sign * 2.0 * field[slot * 3 + c]);
                colours[i * 3..i * 3 + 3].copy_from_slice(&to_bytes(lab_to_srgb(lab)));
            }
        }
        pattern_json.push(json!({"share": round(share[rank], 4), "plus": STANDARD.encode(&plus), "minus": STANDARD.encode(&minus)}));
    }

    let report = json!({
        "heads": names,
        "base tones (Lab)": tones.iter().map(|t| rounded(t, 2)).collect::<Vec<_>>(),
        "base tone, mean distance between two heads (delta E)": round(mean_delta, 2),
        "base tone range": {"L": range(&tones, 0), "a": range(&tones, 1), "b": range(&tones, 2)},
        "zone tints": zone_tints,
        "tint patterns, share of variety": rounded(&share[..10.min(heads)], 3),
        "points": points,
        "points painted on nine heads in ten": common_points.len(),
        "pattern means by zone (L, a, b at one standard deviation)": loadings,
    });
    std::fs::create_dir_all(out).expect("out dir");
    std::fs::write(out.join("report.json"), serde_json::to_string_pretty(&report).expect("json")).expect("report.json");
    let patterns = json!({"points": points, "patterns": pattern_json});
    std::fs::write(out.join("patterns.json"), serde_json::to_string(&patterns).expect("json")).expect("patterns.json");
    println!("{}", serde_json::to_string_pretty(&report).expect("json"));
    println!("took {:.2} s", started.elapsed().as_secs_f64());
}
