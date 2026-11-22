//! The lid piece on the 57 KOTOR heads: builds it, times it, checks it and
//! writes the page's data.
//!
//! `lid_piece export` writes the heads' positions for the page.
//! `lid_piece report` builds and checks every head and writes `report.json`.
//! Both read `~/.claude/tools/likeness-fit-data/sliders/` and write to
//! `~/.claude/tools/likeness-fit-data/lids/`.

use head_eyes::piece::{EyeEdge, LidRig, Settings, V3, build, info_json};
use std::path::PathBuf;
use std::time::Instant;

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME"))
}

fn read_f64(p: &PathBuf) -> Vec<f64> {
    let b = std::fs::read(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    b.chunks_exact(8).map(|c| f64::from_le_bytes(c.try_into().unwrap())).collect()
}
fn read_f32(p: &PathBuf) -> Vec<f32> {
    let b = std::fs::read(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    b.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect()
}
fn read_u32(p: &PathBuf) -> Vec<u32> {
    let b = std::fs::read(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    b.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect()
}

pub struct Inputs {
    pub rig: LidRig,
    pub names: Vec<String>,
    pub heads: Vec<Vec<V3>>,
}

fn inputs() -> Inputs {
    let data = home().join(".claude/tools/likeness-fit-data/sliders");
    let input = data.join("input");
    let base: Vec<V3> = read_f64(&input.join("base.bin")).chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    let tris: Vec<[u32; 3]> = read_u32(&input.join("tris.bin")).chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    let edges: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("lid_edges.json")).unwrap()).unwrap();
    let mut eyes = Vec::new();
    for key in ["L", "R"] {
        let e = &edges["eyes"][key];
        let u = |v: &serde_json::Value| v.as_u64().unwrap() as u32;
        eyes.push(EyeEdge {
            edge: e["edge"].as_array().unwrap().iter().map(u).collect(),
            upper: e["lid"].as_array().unwrap().iter().map(|v| v.as_str() == Some("upper")).collect(),
            inner_corner: u(&e["inner_corner"]),
            outer_corner: u(&e["outer_corner"]),
            centre: e["eyeball_centre_vertices"].as_array().unwrap().iter().map(u).collect(),
            temple_plus_x: e["temple_is_plus_x"].as_bool().unwrap(),
        });
    }
    let rig = LidRig::new(&tris, &base, &eyes);
    let mut names: Vec<String> = std::fs::read_dir(input.join("heads"))
        .unwrap()
        .filter_map(|d| d.ok().map(|d| d.file_name().to_string_lossy().to_string()))
        .collect();
    names.sort();
    let heads: Vec<Vec<V3>> = names
        .iter()
        .map(|n| read_f32(&input.join("heads").join(n).join("fit_smooth.bin")).chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect())
        .collect();
    let mut names2 = vec!["base".to_string()];
    names2.extend(names);
    let mut heads2 = vec![base];
    heads2.extend(heads);
    Inputs { rig, names: names2, heads: heads2 }
}

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "report".into());
    let out = home().join(".claude/tools/likeness-fit-data/lids");
    std::fs::create_dir_all(&out).unwrap();
    let t0 = Instant::now();
    let inp = inputs();
    eprintln!("read {} heads in {:.1} s", inp.heads.len(), t0.elapsed().as_secs_f64());
    match cmd.as_str() {
        "export" => {
            let mut bytes = Vec::new();
            for h in &inp.heads {
                for p in h {
                    for v in p {
                        bytes.extend_from_slice(&(*v as f32).to_le_bytes());
                    }
                }
            }
            std::fs::write(out.join("heads.bin"), bytes).unwrap();
            std::fs::write(out.join("heads.json"), serde_json::to_string(&inp.names).unwrap()).unwrap();
            println!("wrote {} heads", inp.names.len());
        }
        "debug" => {
            let name = std::env::args().nth(2).unwrap();
            let i = inp.names.iter().position(|n| *n == name).unwrap();
            let eye: usize = std::env::args().nth(3).map(|e| e.parse().unwrap()).unwrap_or(0);
            let p = head_eyes::piece::build_eye(&inp.rig.eyes[eye], &inp.rig.triangles, &inp.heads[i], &Settings::default());
            println!("{}", info_json(&p));
            let c = head_eyes::piece_check::check_eye(&inp.rig.eyes[eye], &inp.rig.triangles, &inp.heads[i], &p);
            println!("{c:?}");
        }
        _ => {
            // `report [open] [match]`: every head at one open setting, turning about the eyeball's centre or by KOTOR's clip
            let open: f64 = std::env::args().nth(2).map(|v| v.parse().unwrap()).unwrap_or(0.0);
            let match_clip = std::env::args().nth(3).is_some_and(|v| v == "match");
            let s = Settings { open, match_clip, ..Settings::default() };
            let mut lines = Vec::new();
            let mut times = Vec::new();
            let mut check_times = Vec::new();
            let work: Vec<(usize, String)> = inp.names.iter().cloned().enumerate().collect();
            let results: Vec<(String, Vec<String>, f64, f64, Vec<head_eyes::piece_check::EyeCheck>, Vec<f64>)> = std::thread::scope(|sc| {
                let chunks: Vec<_> = work.chunks(work.len().div_ceil(8)).collect();
                let handles: Vec<_> = chunks
                    .into_iter()
                    .map(|chunk| {
                        let inp = &inp;
                        sc.spawn(move || {
                            chunk
                                .iter()
                                .map(|(i, n)| {
                                    let h = &inp.heads[*i];
                                    let t = Instant::now();
                                    let mut pieces = build(&inp.rig, h, &s);
                                    for _ in 0..19 {
                                        pieces = build(&inp.rig, h, &s);
                                    }
                                    let ms = t.elapsed().as_secs_f64() * 1000.0 / 20.0;
                                    let t = Instant::now();
                                    let checks: Vec<_> = pieces
                                        .iter()
                                        .zip(&inp.rig.eyes)
                                        .map(|(p, e)| head_eyes::piece_check::check_eye(e, &inp.rig.triangles, h, p))
                                        .collect();
                                    let cms = t.elapsed().as_secs_f64() * 1000.0;
                                    let infos = pieces.iter().map(info_json).collect();
                                    let angles = pieces.iter().map(|p| p.closed_angle).collect();
                                    (n.clone(), infos, ms, cms, checks, angles)
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect();
                handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
            });
            for (n, infos, ms, cms, checks, angles) in &results {
                times.push(*ms);
                check_times.push(*cms);
                let cj: Vec<String> = checks
                    .iter()
                    .map(|c| {
                        format!(
                            "{{\"into_eyeball\":{:.3},\"edge_gap\":{:.3},\"poke_points\":{},\"poke_depth\":{:.3},\"lash_poke_points\":{},\"lash_poke_depth\":{:.3},\"shown_closed_front\":{:.3},\"shown_closed_three\":{:.3},\"shown_open_front\":{:.2}}}",
                            c.into_eyeball, c.edge_gap, c.poke_points, c.poke_depth, c.lash_poke_points, c.lash_poke_depth, c.shown_closed_front, c.shown_closed_three, c.shown_open_front
                        )
                    })
                    .collect();
                lines.push(format!("\"{n}\":{{\"pieces\":[{}],\"checks\":[{}],\"build_ms\":{ms:.3}}}", infos.join(","), cj.join(",")));
                let c = |f: fn(&head_eyes::piece_check::EyeCheck) -> f64| checks.iter().map(f).fold(f64::NAN, |a: f64, b| if a.is_nan() { b } else { a.max(b) });
                println!(
                    "{n:22} angle {:5.1} {:5.1} | into eye {:6.2} | edge gap {:5.2} | poke {:4} pts {:5.2} mm | lash {:4} pts {:5.2} mm | shown closed front {:5.2} three {:5.2} mm2 | open {:5.1}",
                    angles[0], angles[1],
                    checks.iter().map(|c| c.into_eyeball).fold(f64::INFINITY, f64::min),
                    c(|c| c.edge_gap), checks.iter().map(|c| c.poke_points).sum::<u32>(), c(|c| c.poke_depth),
                    checks.iter().map(|c| c.lash_poke_points).sum::<u32>(), c(|c| c.lash_poke_depth),
                    c(|c| c.shown_closed_front), c(|c| c.shown_closed_three), c(|c| c.shown_open_front)
                );
            }
            times.sort_by(f64::total_cmp);
            check_times.sort_by(f64::total_cmp);
            println!("build ms per head (both eyes): median {:.3}, max {:.3}", times[times.len() / 2], times[times.len() - 1]);
            println!("check ms per head: median {:.0}, max {:.0}", check_times[check_times.len() / 2], check_times[check_times.len() - 1]);
            let name = if match_clip { "report_clip.json" } else { "report.json" };
            std::fs::write(out.join(name), format!("{{{}}}", lines.join(",\n"))).unwrap();
        }
    }
}
