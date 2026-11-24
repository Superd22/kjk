//! Samples KOTOR's paint baked into our layout next to maps of the head's shape, for the study of KOTOR's painted shading. This is an analysis
//! tool and never ships.
//!
//! For each head the tool computes, at every vertex of the head (by default the wrap of our mesh onto the KOTOR head, so KOTOR's own shape):
//! - the mean curvature at 3, 8 and 20 mm, as how far the vertex stands out of the Gaussian-weighted mean of the surface around it along its
//!   normal, divided by the scale (positive on a bump, negative in a crease);
//! - an ambient occlusion from the horizon around the vertex within 25 mm, in 16 directions (1 is fully open, lower is more occluded);
//! - the normal's up, forward and side parts;
//! - two narrow lobes, the forward part of the normal and the normal's part along a direction 30 degrees above forward, each raised to the
//!   8th power after negative values are set to 0.
//!
//! It then reads the bake in 4 by 4 pixel blocks, keeps the blocks that lie on skin, outside the lids and outside the lip slit's hidden faces,
//! with no grey (unpainted) pixel, and writes one row per block: the block's place, its mean Lab colour, its mean linear RGB colour, the vertex
//! nearest the block's corner pixel and the maps read at that pixel.
//!
//! Usage: shading_maps <common bundle dir> <heads bundle dir> <kotor texture dir> <out dir> [--fitted] [<name> ...]
//! It writes `<out dir>/<name>.f32` (rows of `COLUMNS`, little-endian f32), `<out dir>/<name>_vertices.f32` (per vertex: position, normal and
//! the maps), and `<out dir>/columns.json`.

#[path = "shared/bundle.rs"]
mod bundle;

use bundle::Bundle;
use head_paint::colour::srgb_to_lab;
use head_paint::pixel_map::PixelMap;
use head_paint::surface::Surface;
use serde_json::json;
use std::path::Path;

const RES: usize = 2048;
const BLOCK: usize = 4;
const GREY: i32 = 123;
const SCALES_MM: [f64; 3] = [3.0, 8.0, 20.0];
const AO_RADIUS: f64 = 0.025;
const LOBE_POWER: i32 = 8;
const MAPS: [&str; 12] = ["h3", "h8", "h20", "ao", "n_up", "n_fwd", "n_side", "lobe_fwd", "lobe_upfwd", "area", "sky", "sky_cos"];
const COLUMNS: [&str; 10] = ["px", "py", "L", "a", "b", "r_lin", "g_lin", "b_lin", "vertex", "n_pixels"];

type Vec3 = [f64; 3];

/// The maps of `MAPS` at every vertex.
fn maps(positions: &[Vec3], triangles: &[[u32; 3]]) -> (Vec<Vec3>, Vec<[f64; MAPS.len()]>) {
    let surface = Surface::new(positions, triangles);
    let up_fwd = [0.0, 30f64.to_radians().sin(), 30f64.to_radians().cos()];
    let mut out = vec![[0.0; MAPS.len()]; positions.len()];
    for (i, n) in surface.normals.iter().enumerate() {
        if !surface.is_used(i) {
            continue;
        }
        for (k, &s_mm) in SCALES_MM.iter().enumerate() {
            out[i][k] = surface.bump(i, s_mm / 1000.0) / (s_mm / 1000.0);
        }
        let horizon = surface.horizon(i, AO_RADIUS);
        let sky = horizon.sky();
        out[i][3] = horizon.openness();
        out[i][10] = sky.uniform;
        out[i][11] = sky.cosine;
        out[i][4] = n[1];
        out[i][5] = n[2];
        out[i][6] = n[0];
        out[i][7] = n[2].max(0.0).powi(LOBE_POWER);
        out[i][8] = (n[1] * up_fwd[1] + n[2] * up_fwd[2]).max(0.0).powi(LOBE_POWER);
        out[i][9] = surface.areas[i];
    }
    (surface.normals, out)
}

fn write_f32(path: &Path, values: &[f32]) {
    let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    std::fs::write(path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let fitted = args.iter().any(|a| a == "--fitted");
    args.retain(|a| a != "--fitted");
    if args.len() < 4 {
        eprintln!("Usage: shading_maps <common bundle dir> <heads bundle dir> <kotor texture dir> <out dir> [--fitted] [<name> ...]");
        std::process::exit(2);
    }
    let common = Bundle::open(Path::new(&args[0]));
    let out_dir = Path::new(&args[3]);
    std::fs::create_dir_all(out_dir).expect("out dir");
    let uv = common.uv();
    let triangles = common.triangles("tris");
    let old_triangles = common.triangles("old_tris");
    let vmap = common.layout_to_mesh();
    let map = PixelMap::new(&uv, &triangles, RES);
    let per_pixel = |values: Vec<f64>| -> Vec<f64> {
        let new: Vec<[f64; 1]> = vmap.iter().map(|&o| [values[o]]).collect();
        map.interpolate(&triangles, &new).into_iter().map(|v| v[0]).collect()
    };
    let to_f64 = |v: Vec<u8>| v.into_iter().map(f64::from).collect::<Vec<_>>();
    let on_skin = per_pixel(to_f64(common.u8s("skin")));
    let on_lid = per_pixel(to_f64(common.u8s("lid")));
    let hidden = per_pixel(common.f64s("dark"));
    let mut where_pixel = vec![u32::MAX; RES * RES];
    for (k, p) in map.pixels.iter().enumerate() {
        where_pixel[p.y as usize * RES + p.x as usize] = k as u32;
    }
    let good = |k: u32| k != u32::MAX && on_skin[k as usize] >= 0.999 && on_lid[k as usize] <= 0.0 && hidden[k as usize] <= 0.0;

    let names: Vec<String> = if args.len() > 4 {
        args[4..].to_vec()
    } else {
        let mut v: Vec<String> = std::fs::read_dir(&args[1]).expect("heads dir").filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        v
    };
    for name in &names {
        let started = std::time::Instant::now();
        let head = Bundle::open(&Path::new(&args[1]).join(name));
        let positions = head.points(if fitted { "fitted" } else { "wrapped" });
        let (normals, vertex_maps) = maps(&positions, &old_triangles);
        let mut per_vertex = Vec::with_capacity(positions.len() * (6 + MAPS.len()));
        for i in 0..positions.len() {
            per_vertex.extend(positions[i].iter().chain(&normals[i]).chain(&vertex_maps[i]).map(|&x| x as f32));
        }
        write_f32(&out_dir.join(format!("{name}_vertices.f32")), &per_vertex);

        let img = image::open(Path::new(&args[2]).join(format!("{name}_ours.jpg"))).unwrap_or_else(|e| panic!("{name}: {e}")).to_rgb8();
        let mut rows: Vec<f32> = Vec::new();
        for by in (0..RES).step_by(BLOCK) {
            for bx in (0..RES).step_by(BLOCK) {
                let mut ok = true;
                let (mut lin, mut count) = ([0.0; 3], 0.0);
                for y in by..by + BLOCK {
                    for x in bx..bx + BLOCK {
                        let k = where_pixel[y * RES + x];
                        if !good(k) {
                            ok = false;
                            continue;
                        }
                        let px = img.get_pixel(x as u32, y as u32).0;
                        if px.iter().all(|&v| (i32::from(v) - GREY).abs() <= 2) {
                            ok = false;
                            continue;
                        }
                        for c in 0..3 {
                            let e = f64::from(px[c]) / 255.0;
                            lin[c] += if e > 0.04045 { ((e + 0.055) / 1.055).powf(2.4) } else { e / 12.92 };
                        }
                        count += 1.0;
                    }
                }
                if !ok || count == 0.0 {
                    continue;
                }
                let lin = lin.map(|x| x / count);
                let encoded = lin.map(|c| if c > 0.0031308 { 1.055 * c.powf(1.0 / 2.4) - 0.055 } else { 12.92 * c });
                let lab = srgb_to_lab(encoded);
                let corner = &map.pixels[where_pixel[by * RES + bx] as usize];
                let tri = triangles[corner.triangle as usize];
                let best = (0..3).max_by(|&a, &b| corner.barycentric[a].total_cmp(&corner.barycentric[b])).unwrap();
                let vertex = vmap[tri[best] as usize];
                rows.extend([bx as f64, by as f64, lab[0], lab[1], lab[2], lin[0], lin[1], lin[2], vertex as f64, count].map(|x| x as f32));
                let corners = tri.map(|t| vmap[t as usize]);
                let mut values = [0.0; MAPS.len()];
                for (c, &vertex) in corners.iter().enumerate() {
                    for (value, map) in values.iter_mut().zip(vertex_maps[vertex]) {
                        *value += corner.barycentric[c] * map;
                    }
                }
                rows.extend(values.map(|v| v as f32));
            }
        }
        write_f32(&out_dir.join(format!("{name}.f32")), &rows);
        println!("{name}: {} blocks in {:.2} s", rows.len() / (COLUMNS.len() + MAPS.len()), started.elapsed().as_secs_f64());
    }
    let columns: Vec<&str> = COLUMNS.iter().chain(MAPS.iter()).copied().collect();
    let vertex_columns: Vec<&str> = ["x", "y", "z", "nx", "ny", "nz"].iter().chain(MAPS.iter()).copied().collect();
    std::fs::write(out_dir.join("columns.json"), serde_json::to_string_pretty(&json!({ "rows": columns, "vertices": vertex_columns, "fitted": fitted })).unwrap()).expect("columns");
}
