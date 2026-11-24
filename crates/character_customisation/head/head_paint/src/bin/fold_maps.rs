//! Pictures of a head's surface valleys over KOTOR's bake, in the layout, with a grid every 50 pixels. They are used to place the folds'
//! anchors on the base head and to look at the valleys that step 2 of the fold work compares with KOTOR's folds. Analysis only.
//!
//! Usage: fold_maps <out dir> <scale mm> <x0> <y0> <x1> <y1> [<name> ...]   ("base" is the base head over a grey skin)

#[path = "shared/bundle.rs"]
mod bundle;
#[path = "shared/fold_common.rs"]
mod fold_common;

use fold_common::*;
use head_paint::surface::Surface;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = Path::new(&args[0]);
    std::fs::create_dir_all(out).unwrap();
    let scale_mm: f64 = args[1].parse().unwrap();
    let crop: Vec<usize> = args[2..6].iter().map(|a| a.parse().unwrap()).collect();
    let c = Common::load();
    for name in &args[6..] {
        let positions = if name == "base" { c.base.clone() } else { c.fitted(name) };
        let surface = Surface::new(&positions, &c.mesh_tris);
        let bump: Vec<f64> = (0..positions.len()).map(|i| if surface.is_used(i) { 1000.0 * surface.bump(i, scale_mm / 1000.0) } else { 0.0 }).collect();
        let per_layout: Vec<[f64; 1]> = c.vmap.iter().map(|&m| [bump[m]]).collect();
        let values = c.map.interpolate(&c.tris, &per_layout);
        let bake = if name == "base" { None } else { Some(image::open(bake_path(name)).unwrap().to_rgb8()) };
        let (w, h) = ((crop[2] - crop[0]) as u32, (crop[3] - crop[1]) as u32);
        let mut img = image::RgbImage::new(w, h);
        for (p, v) in c.map.pixels.iter().zip(&values) {
            let (x, y) = (p.x as usize, p.y as usize);
            if x < crop[0] || x >= crop[2] || y < crop[1] || y >= crop[3] {
                continue;
            }
            let g = bake.as_ref().map_or(150.0, |b| {
                let q = b.get_pixel(x as u32, y as u32).0;
                0.3 * f64::from(q[0]) + 0.59 * f64::from(q[1]) + 0.11 * f64::from(q[2])
            });
            let t = (v[0] / 0.6).clamp(-1.0, 1.0);
            let rgb = if t < 0.0 { [g + (255.0 - g) * -t, g * (1.0 + t), g * (1.0 + t)] } else { [g * (1.0 - t), g * (1.0 - t), g + (255.0 - g) * t] };
            img.put_pixel((x - crop[0]) as u32, (y - crop[1]) as u32, image::Rgb(rgb.map(|v| v.clamp(0.0, 255.0) as u8)));
        }
        for y in 0..h {
            for x in 0..w {
                let (gx, gy) = (x as usize + crop[0], y as usize + crop[1]);
                if gx % 50 == 0 || gy % 50 == 0 {
                    let strong = gx % 100 == 0 || gy % 100 == 0;
                    img.put_pixel(x, y, image::Rgb(if strong { [0, 255, 0] } else { [0, 120, 0] }));
                }
            }
        }
        img.save(out.join(format!("{name}_valleys.png"))).unwrap();
        println!("{name}");
    }
}
