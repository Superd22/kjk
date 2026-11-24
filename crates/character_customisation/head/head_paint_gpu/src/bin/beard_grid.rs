//! Turns a drawn mask into values on the facial hair's grid of one head, as the grid editor will (paint track, part C, layer 6). Analysis
//! only.
//!
//! `beard_grid <mask json> <out json>`
//!
//! The mask names a head and draws shapes on its face seen from the front, in millimetres from the middle of the mouth: `x` across the
//! face toward the head's x > 0 side and `y` up. It holds `polygons` (each a list of [x, y] corners), the hairs' `density` in hairs per
//! square millimetre and `length_mm`, `soft_mm`, over which the density fades out at a shape's edge, and `front_mm`: only points less
//! than this far behind the front of the chin are written, so a shape drawn over the chin does not reach down the neck. The output is a
//! `hair_grid::GridEdit` with `only` set, so every point outside the shapes has no hair. It also prints the head's landmarks in the same
//! millimetres, to draw a mask from.

use head_paint::facial_hair::{BeardHead, polygon_distance};
use head_paint::folds::Landmarks;
use head_paint::hair_grid::{EditPoint, GridEdit};
use head_paint::layout_points::{dot, sub};
use head_paint::live::Layout;
use serde_json::Value;
use std::path::Path;

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";

fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .chunks_exact(8)
        .map(|b| f64::from_le_bytes(b.try_into().expect("8")))
        .collect()
}

fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().expect("4")))
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("Usage: beard_grid <mask json> <out json>");
        std::process::exit(2);
    }
    let mask: Value =
        serde_json::from_str(&std::fs::read_to_string(&args[0]).expect("mask")).expect("json");
    let name = mask["head"].as_str().expect("head");
    let l = Path::new(DATA).join("layout");
    let triples = |v: Vec<u32>| {
        v.chunks_exact(3)
            .map(|t| [t[0], t[1], t[2]])
            .collect::<Vec<_>>()
    };
    let lay = Layout {
        uv: f64s(&l.join("uv.f64"))
            .chunks_exact(2)
            .map(|p| [p[0], p[1]])
            .collect(),
        triangles: triples(u32s(&l.join("tris.u32"))),
        layout_to_mesh: u32s(&l.join("vmap.u32"))
            .into_iter()
            .map(|i| i as usize)
            .collect(),
        mesh_triangles: triples(u32s(&l.join("mesh_tris.u32"))),
    };
    let pos: Vec<[f64; 3]> = f64s(&Path::new(DATA).join("heads").join(format!("{name}.f64")))
        .chunks_exact(3)
        .map(|p| [p[0], p[1], p[2]])
        .collect();
    let lips = head_paint::lips::LipHead::new(&pos, &lay.triangles, &lay.layout_to_mesh);
    let beard = BeardHead::new(
        &pos,
        &lay.uv,
        &lay.triangles,
        &lay.layout_to_mesh,
        lips.as_ref(),
    )
    .expect("the head has no facial hair grid");
    let m = Landmarks::new(&pos);
    let mouth = [0, 1, 2].map(|k| 0.5 * (m.mouth_corner[0][k] + m.mouth_corner[1][k]));
    let at = |p: [f64; 3]| {
        let d = sub(p, mouth);
        [
            dot(d, m.frame.across),
            dot(d, m.frame.up),
            dot(d, m.frame.forward),
        ]
    };
    let r = |v: [f64; 3]| v.map(|x| (x * 10.0).round() / 10.0);
    println!("{name}, in mm from the middle of the mouth (across, up, forward):");
    println!(
        "  mouth corners {:?} {:?}",
        r(at(m.mouth_corner[0])),
        r(at(m.mouth_corner[1]))
    );
    println!(
        "  nose wings {:?} {:?}",
        r(at(m.nose_wing[0])),
        r(at(m.nose_wing[1]))
    );
    println!(
        "  lower lip middle {:?}, chin middle {:?}",
        r(at(m.lower_lip_middle)),
        r(at(m.chin_middle))
    );
    let chin = at(m.chin_middle);
    let polys: Vec<Vec<[f64; 2]>> = mask["polygons"]
        .as_array()
        .expect("polygons")
        .iter()
        .map(|p| {
            p.as_array()
                .expect("polygon")
                .iter()
                .map(|c| [c[0].as_f64().expect("x"), c[1].as_f64().expect("y")])
                .collect()
        })
        .collect();
    let density = mask["density"].as_f64().expect("density");
    let length = mask["length_mm"].as_f64().expect("length_mm");
    let soft = mask["soft_mm"].as_f64().unwrap_or(1.0).max(0.05);
    let front = mask["front_mm"].as_f64().unwrap_or(20.0);
    let g = &beard.grid;
    let mut points = Vec::new();
    for j in 0..g.rows {
        for i in 0..g.cols {
            let p = g.at(i, j);
            if !p.valid {
                continue;
            }
            let q = at(p.position);
            if q[2] < chin[2] - front {
                continue;
            }
            let dist = polys
                .iter()
                .map(|poly| polygon_distance([q[0], q[1]], poly))
                .fold(f64::MAX, f64::min);
            let t = ((soft - dist) / (2.0 * soft)).clamp(0.0, 1.0);
            let w = t * t * (3.0 - 2.0 * t);
            if w > 0.0 {
                points.push(EditPoint {
                    i,
                    j,
                    density: density * w,
                    length_mm: length,
                    direction: None,
                });
            }
        }
    }
    let edit = GridEdit {
        cols: g.cols,
        rows: g.rows,
        only: true,
        points,
    };
    std::fs::write(&args[1], edit.to_json()).expect("write");
    println!(
        "{}: {} of {} grid points hold hair",
        args[1],
        edit.points.len(),
        g.points.len()
    );
}
