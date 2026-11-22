//! Turns the folds' landmark points, chosen as pixels of the layout on the base head's x < 0 side, into anchors made of three mesh
//! vertices and their weights, for both sides of the face. The other side's anchor is the base head's surface point nearest the mirror
//! image of the first, so the two sides match in 3D even though the layout and the mesh are not exact mirror images. It prints the
//! constants of `folds.rs` and a picture of the anchors on a few bakes. Analysis only.
//!
//! Usage: fold_anchors <out dir> [<name> ...]

#[path = "shared/bundle.rs"]
mod bundle;
#[path = "shared/fold_common.rs"]
mod fold_common;

use fold_common::*;
use head_paint::folds::{ANCHORS, Anchor, Anchors, SurfaceIndex};
use head_paint::layout_points::{LayoutLocator, uv_of};
use std::path::Path;

/// The landmarks as layout pixels on the x < 0 side, and whether a landmark lies on the face's middle line.
const PICKS: [(&str, (f64, f64), bool); 8] = [
    ("NOSE_WING", (697.0, 1212.0), false),
    ("MOUTH_CORNER", (688.0, 1466.0), false),
    ("CHIN_MIDDLE", (926.0, 1600.0), true),
    ("LOWER_LIP_MIDDLE", (926.0, 1552.0), true),
    ("EYE_INNER", (655.0, 828.0), false),
    ("EYE_OUTER", (446.0, 854.0), false),
    ("LOWER_LID", (575.0, 878.0), false),
    ("NOSE_TIP", (926.0, 1118.0), true),
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = Path::new(&args[0]);
    std::fs::create_dir_all(out).unwrap();
    let c = Common::load();
    let per_layout: Vec<[f64; 3]> = c.vmap.iter().map(|&m| c.base[m]).collect();
    let locator = LayoutLocator::new(&c.uv, &c.tris);
    let index = SurfaceIndex::new(&c.tris, &per_layout);
    let anchor_of = |p_mm: [f64; 3]| -> Anchor {
        let hit = index.closest(p_mm).expect("on the head");
        let t = c.tris[hit.triangle];
        Anchor { vertices: t.map(|i| c.vmap[i as usize] as u32), weights: hit.barycentric }
    };
    println!("pub const ANCHORS: Anchors = Anchors {{");
    for (name, px, middle) in PICKS {
        let spot = locator.spot(uv_of(px), &per_layout).expect("on the layout").position_mm;
        let a = anchor_of(spot);
        let mirror = if middle { [0.0, spot[1], spot[2]] } else { [-spot[0], spot[1], spot[2]] };
        let a0 = if middle { anchor_of(mirror) } else { a };
        let b = anchor_of(mirror);
        let field = name.to_lowercase();
        let fmt = |a: &Anchor| format!("Anchor {{ vertices: {:?}, weights: [{:.5}, {:.5}, {:.5}] }}", a.vertices, a.weights[0], a.weights[1], a.weights[2]);
        if middle {
            println!("    {field}: {},", fmt(&a0));
        } else {
            println!("    {field}: [{}, {}],", fmt(&a), fmt(&b));
        }
    }
    println!("}};");
    // The anchors on the bakes, to check them.
    for name in &args[1..] {
        let positions = c.fitted(name);
        let lay: Vec<[f64; 3]> = c.vmap.iter().map(|&m| positions[m]).collect();
        let idx = SurfaceIndex::new(&c.tris, &lay);
        let mut img = image::open(bake_path(name)).unwrap().to_rgb8();
        for p in ANCHORS.all() {
            let at = Anchors::point(&p, &positions);
            if let Some(hit) = idx.closest(at) {
                let uv = idx.uv_of(hit, &c.uv, &c.tris);
                let (x, y) = ((uv[0] * RES as f64) as i64, ((1.0 - uv[1]) * RES as f64) as i64);
                for d in -6i64..=6 {
                    for (dx, dy) in [(d, 0), (0, d)] {
                        let (px, py) = (x + dx, y + dy);
                        if px >= 0 && py >= 0 && px < RES as i64 && py < RES as i64 {
                            img.put_pixel(px as u32, py as u32, image::Rgb([0, 255, 0]));
                        }
                    }
                }
            }
        }
        image::imageops::crop_imm(&img, 350, 700, 1150, 1050).to_image().save(out.join(format!("{name}_anchors.png"))).unwrap();
    }
}
