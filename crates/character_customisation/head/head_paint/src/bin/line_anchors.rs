//! Turns the forehead line landmarks, chosen as pixels of the layout on the base head's x < 0 side, into anchors of three mesh vertices
//! and their weights, in the way of `fold_anchors`. It prints the constants of `lines.rs` and draws the anchors on a few bakes. Analysis
//! only.
//!
//! Usage: line_anchors <out dir> [<name> ...]

#[path = "shared/bundle.rs"]
mod bundle;
#[path = "shared/fold_common.rs"]
mod fold_common;

use fold_common::*;
use head_paint::folds::{Anchor, Anchors, SurfaceIndex};
use head_paint::layout_points::{LayoutLocator, uv_of};
use head_paint::lines::BROW_ANCHORS;
use std::path::Path;

/// The landmarks as layout pixels on the x < 0 side, and whether a landmark lies on the face's middle line. They were read from gridded
/// bakes of pmhc04 and Canderous: the brow's inner end, its highest point and its outer end lie on the brow's middle line.
const PICKS: [(&str, (f64, f64), bool); 6] = [
    ("BROW_INNER", (785.0, 640.0), false),
    ("BROW_PEAK", (645.0, 603.0), false),
    ("BROW_OUTER", (440.0, 660.0), false),
    ("BROW_MIDDLE", (926.0, 640.0), true),
    ("FOREHEAD_MIDDLE", (926.0, 480.0), true),
    ("HAIRLINE_MIDDLE", (926.0, 365.0), true),
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
    let fmt = |a: &Anchor| format!("Anchor {{ vertices: {:?}, weights: [{:.5}, {:.5}, {:.5}] }}", a.vertices, a.weights[0], a.weights[1], a.weights[2]);
    println!("pub const BROW_ANCHORS: BrowAnchors = BrowAnchors {{");
    for (name, px, middle) in PICKS {
        let spot = locator.spot(uv_of(px), &per_layout).expect("on the layout").position_mm;
        let field = name.to_lowercase();
        if middle {
            println!("    {field}: {},", fmt(&anchor_of([0.0, spot[1], spot[2]])));
        } else {
            println!("    {field}: [{}, {}],", fmt(&anchor_of(spot)), fmt(&anchor_of([-spot[0], spot[1], spot[2]])));
        }
    }
    println!("}};");
    for name in &args[1..] {
        let positions = c.fitted(name);
        let lay: Vec<[f64; 3]> = c.vmap.iter().map(|&m| positions[m]).collect();
        let idx = SurfaceIndex::new(&c.tris, &lay);
        let mut img = image::open(bake_path(name)).unwrap().to_rgb8();
        for p in BROW_ANCHORS.all() {
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
        image::imageops::crop_imm(&img, 350, 250, 1150, 600).to_image().save(out.join(format!("{name}_brow_anchors.jpg"))).unwrap();
    }
}
