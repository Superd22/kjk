//! Pictures of the pore layer for the pores and grain round (paint track, part C, fine detail). Analysis only.
//!
//! `pore_pictures <label> <out dir> <name>... [--layers 11111] [--set <control id>=<value>]...`
//!
//! For each head it paints the head's recipe from the live page's data on the GPU at 2,048 pixels, with the `--set` values put in, and
//! writes to `<out dir>/<label>/`:
//! - `<name>.png`, the texture;
//! - `<name>_<region>.jpg` for the cheek, the forehead and the nose: a front view 30 by 30 mm at 32 pixels per millimetre, which is about
//!   4 screen pixels per texel;
//! - `<name>_face.jpg`, the whole face from the front, 110 by 120 mm at 4 pixels per millimetre, as at conversation distance.
//!
//! With `--pore-defaults`, every pore control takes its default instead of the head's fitted value, as the live page's
//! data now does.
//!
//! With `--time` it writes nothing and prints, per head, the repaint time at 2,048 pixels of the pore layer alone, of every layer but the
//! pores, and of every layer: 2 repaints to warm up, then the median and the best of 10, each from the request until the GPU has
//! finished.
//!
//! It writes the same views of KOTOR's own head with its own texture to `<out dir>/kotor/` when they are missing.
//! The label `strip` is special: it paints one head with each value given by `--strip <id>=<a>,<b>,<c>` and writes only cheek crops,
//! named `<name>_<id>_<k>.jpg`.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::{Common, KotorFront};
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint::folds::Landmarks;
use head_paint::live::Layout;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::Value;
use std::path::{Path, PathBuf};

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
const N: u32 = 2048;
const CROP_MM: f64 = 30.0;
const CROP_PER_MM: f64 = 32.0;

fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().expect("8"))).collect()
}

fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().expect("4"))).collect()
}

fn layout() -> Layout {
    let l = Path::new(DATA).join("layout");
    let triples = |v: Vec<u32>| v.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect::<Vec<_>>();
    Layout {
        uv: f64s(&l.join("uv.f64")).chunks_exact(2).map(|p| [p[0], p[1]]).collect(),
        triangles: triples(u32s(&l.join("tris.u32"))),
        layout_to_mesh: u32s(&l.join("vmap.u32")).into_iter().map(|i| i as usize).collect(),
        mesh_triangles: triples(u32s(&l.join("mesh_tris.u32"))),
    }
}

fn positions(name: &str) -> Vec<[f64; 3]> {
    f64s(&Path::new(DATA).join("heads").join(format!("{name}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect()
}

fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}

fn mid(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|k| 0.5 * (a[k] + b[k]))
}

/// The centres of the crops on the face, in 3D millimetres: the cheek of the x < 0 side, the forehead and the nose.
fn regions(l: &Landmarks) -> [(&'static str, [f64; 3]); 3] {
    let f = l.frame;
    let cheek = add(add(l.lower_lid[0], f.up, -17.0), f.out(0), 2.0);
    let eyes = mid(mid(l.eye_inner[0], l.eye_outer[0]), mid(l.eye_inner[1], l.eye_outer[1]));
    let forehead = add(eyes, f.up, 38.0);
    let nose = add(l.nose_tip, f.up, 8.0);
    [("cheek", cheek), ("forehead", forehead), ("nose", nose)]
}

fn face_centre(l: &Landmarks) -> [f64; 3] {
    add(mid(l.nose_tip, mid(l.mouth_corner[0], l.mouth_corner[1])), l.frame.up, 8.0)
}

fn save(img: &image::RgbImage, path: &Path) {
    img.save(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

struct Painter {
    painter: GpuPainter,
    common: Common,
    recipes: Value,
    /// Whether the pore controls take their defaults instead of the head's values.
    pore_defaults: bool,
}

impl Painter {
    fn new() -> Self {
        let layout = layout();
        let common = Common::load();
        assert_eq!(common.uv, layout.uv, "the live page's layout differs from the bundle's");
        let painter = GpuPainter::new(Gpu::blocking().expect("GPU"), layout);
        let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
        Painter { painter, common, recipes, pore_defaults: false }
    }

    fn values(&self, name: &str, overrides: &[(String, f64)]) -> Vec<f64> {
        let head = &self.recipes["heads"][name];
        let list = controls();
        let mut v: Vec<f64> = list.iter().map(|c| if self.pore_defaults && c.group == "pores" { c.default } else { head[&c.id].as_f64().unwrap_or(c.default) }).collect();
        for (id, x) in overrides {
            let k = list.iter().position(|c| &c.id == id).unwrap_or_else(|| panic!("no control {id}"));
            v[k] = *x;
        }
        v
    }

    fn texture(&mut self, values: &[f64], switches: &[bool]) -> image::RgbImage {
        let recipe = recipe_from(values);
        self.painter.paint(N, &recipe, &layers_from(switches));
        let rgba = pollster::block_on(self.painter.read_rgba(N));
        image::RgbImage::from_fn(N, N, |x, y| {
            let at = 4 * (y * N + x) as usize;
            image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
        })
    }
}

fn parse(args: &mut Vec<String>, flag: &str) -> Vec<String> {
    let mut out = Vec::new();
    while let Some(k) = args.iter().position(|a| a == flag) {
        out.push(args.remove(k + 1));
        args.remove(k);
    }
    out
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let overrides: Vec<(String, f64)> = parse(&mut args, "--set")
        .iter()
        .map(|p| {
            let (id, v) = p.split_once('=').expect("--set id=value");
            (id.to_string(), v.parse().expect("number"))
        })
        .collect();
    let switches: Vec<bool> = parse(&mut args, "--layers").pop().unwrap_or_else(|| "11111".into()).chars().map(|c| c == '1').collect();
    let strips = parse(&mut args, "--strip");
    let pore_defaults = args.iter().any(|a| a == "--pore-defaults");
    let timing = args.iter().any(|a| a == "--time");
    args.retain(|a| a != "--pore-defaults" && a != "--time");
    if args.len() < 3 {
        eprintln!("Usage: pore_pictures <label> <out dir> <name>... [--layers 11111] [--set id=value]... [--strip id=a,b,c]...");
        std::process::exit(2);
    }
    let label = &args[0];
    let out = PathBuf::from(&args[1]);
    let dir = out.join(label);
    let kotor_dir = out.join("kotor");
    std::fs::create_dir_all(&dir).expect("out");
    std::fs::create_dir_all(&kotor_dir).expect("out");
    let mut p = Painter::new();
    p.pore_defaults = pore_defaults;

    for name in &args[2..] {
        let pos = positions(name);
        let marks = Landmarks::new(&pos);
        let f = marks.frame;
        p.painter.set_head(pos.clone());
        pollster::block_on(p.painter.prepare(N));
        let base = p.values(name, &overrides);
        if timing {
            let recipe = recipe_from(&base);
            for set in ["00001", "11110", "11111"] {
                let layers = layers_from(&set.chars().map(|c| c == '1').collect::<Vec<_>>());
                let mut ms = Vec::new();
                for k in 0..12 {
                    let t0 = std::time::Instant::now();
                    p.painter.paint(N, &recipe, &layers);
                    p.painter.gpu().wait();
                    if k >= 2 {
                        ms.push(t0.elapsed().as_secs_f64() * 1e3);
                    }
                }
                ms.sort_by(f64::total_cmp);
                println!("{name} {set}: median {:.2} ms, best {:.2} ms", ms[ms.len() / 2], ms[0]);
            }
            continue;
        }

        if !strips.is_empty() {
            let list = controls();
            let centre = regions(&marks)[0].1;
            for s in &strips {
                let (id, vals) = s.split_once('=').expect("--strip id=a,b,c");
                let k = list.iter().position(|c| c.id == id).unwrap_or_else(|| panic!("no control {id}"));
                for (i, v) in vals.split(',').enumerate() {
                    let mut values = base.clone();
                    values[k] = v.parse().expect("number");
                    let tex = p.texture(&values, &switches);
                    let ours = KotorFront::ours(&p.common, &pos, tex, f.across, f.up, f.forward);
                    let q = ours.project(centre).0;
                    save(&ours.tile_at(q, CROP_MM, CROP_MM, CROP_PER_MM), &dir.join(format!("{name}_{id}_{i}.jpg")));
                }
                println!("{name}: strip of {id}");
            }
            continue;
        }

        let tex = p.texture(&base, &switches);
        save(&tex, &dir.join(format!("{name}.png")));
        let ours = KotorFront::ours(&p.common, &pos, tex, f.across, f.up, f.forward);
        let need_kotor = !kotor_dir.join(format!("{name}_face.jpg")).exists();
        let kotor = need_kotor.then(|| KotorFront::open(name, f.across, f.up, f.forward));
        for (region, centre) in regions(&marks) {
            let q = ours.project(centre).0;
            save(&ours.tile_at(q, CROP_MM, CROP_MM, CROP_PER_MM), &dir.join(format!("{name}_{region}.jpg")));
            if let Some(k) = &kotor {
                save(&k.tile_at(q, CROP_MM, CROP_MM, CROP_PER_MM), &kotor_dir.join(format!("{name}_{region}.jpg")));
            }
        }
        let q = ours.project(face_centre(&marks)).0;
        save(&ours.tile_at(q, 110.0, 120.0, 4.0), &dir.join(format!("{name}_face.jpg")));
        if let Some(k) = &kotor {
            save(&k.tile_at(q, 110.0, 120.0, 4.0), &kotor_dir.join(format!("{name}_face.jpg")));
        }
        println!("{name}: done");
    }
}
