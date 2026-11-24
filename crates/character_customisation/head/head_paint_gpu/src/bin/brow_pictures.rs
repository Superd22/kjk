//! Pictures of the brow layer (paint track, part C, the brows). Analysis only.
//!
//! `brow_pictures <label> <out dir> <name>... [--layers 11111111] [--set <control id>=<value>]... [--hand <json>] [--size 2048]`
//!
//! For each head it paints the head's recipe from the live page's data on the GPU, with the head's values from the `--hand` file (an
//! object `heads` of control id to value) and then the `--set` values put in, and writes to `<out dir>/<label>/`:
//! - `<name>_brows.jpg`: both brows from the front, 130 by 50 mm at 6 pixels per millimetre, framed as the KOTOR wall;
//! - `<name>_close.jpg`: the x > 0 brow, 50 by 24 mm at 20 pixels per millimetre;
//! - `<name>_face.jpg`: the whole face from the front, 110 by 120 mm at 4 pixels per millimetre, as at conversation distance;
//! - `<name>_alone.jpg`: both brows with only the brow layer on, on neutral grey skin.
//!
//! It writes the same views of KOTOR's own head, with its hair meshes, to `<out dir>/kotor/` when they are missing.
//! With `--strip <id>=<a>,<b>,<c>` it paints each value of the control and writes only the brows view, named `<name>_<id>_<k>.jpg`.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::{Common, KotorFront};
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint::folds::{Anchors, Landmarks};
use head_paint::lines::BROW_ANCHORS;
use head_paint::live::Layout;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::Value;
use std::path::{Path, PathBuf};

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";

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

fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [0, 1, 2].map(|k| a[k] + t * (b[k] - a[k]))
}

fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}

fn save(img: &image::RgbImage, path: &Path) {
    img.save(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

struct Painter {
    painter: GpuPainter,
    recipes: Value,
    size: u32,
}

impl Painter {
    fn values(&self, name: &str, overrides: &[(String, f64)]) -> Vec<f64> {
        let head = &self.recipes["heads"][name];
        let list = controls();
        let mut v: Vec<f64> = list.iter().map(|c| head[&c.id].as_f64().unwrap_or(c.default)).collect();
        for (id, x) in overrides {
            let k = list.iter().position(|c| &c.id == id).unwrap_or_else(|| panic!("no control {id}"));
            v[k] = *x;
        }
        v
    }

    fn texture(&mut self, values: &[f64], switches: &[bool]) -> image::RgbImage {
        let n = self.size;
        let recipe = recipe_from(values);
        self.painter.paint(n, &recipe, &layers_from(switches));
        let rgba = pollster::block_on(self.painter.read_rgba(n));
        let small = image::RgbImage::from_fn(n, n, |x, y| {
            let at = 4 * (y * n + x) as usize;
            image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
        });
        if n == 2048 { small } else { image::imageops::resize(&small, 2048, 2048, image::imageops::FilterType::Triangle) }
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
    let switches: Vec<bool> = parse(&mut args, "--layers").pop().unwrap_or_else(|| "1".repeat(head_paint::controls::GROUPS.len())).chars().map(|c| c == '1').collect();
    let strips = parse(&mut args, "--strip");
    let timing = args.iter().any(|a| a == "--time");
    args.retain(|a| a != "--time");
    let size: u32 = parse(&mut args, "--size").pop().map_or(2048, |v| v.parse().expect("size"));
    let hand: Value = parse(&mut args, "--hand").pop().map_or(Value::Null, |p| serde_json::from_str(&std::fs::read_to_string(&p).expect("hand file")).expect("json"));
    if args.len() < 3 {
        eprintln!("Usage: brow_pictures <label> <out dir> <name>... [--layers 11111111] [--set id=value]... [--strip id=a,b,c]... [--hand file] [--size n]");
        std::process::exit(2);
    }
    let label = &args[0];
    let out = PathBuf::from(&args[1]);
    let dir = out.join(label);
    let kotor_dir = out.join("kotor");
    std::fs::create_dir_all(&dir).expect("out");
    std::fs::create_dir_all(&kotor_dir).expect("out");
    let lay = layout();
    let common = Common::load();
    assert_eq!(common.uv, lay.uv, "the live page's layout differs from the bundle's");
    let painter = GpuPainter::new(Gpu::blocking().expect("GPU"), lay);
    let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
    let mut p = Painter { painter, recipes, size };
    let list = controls();
    let brow_group = head_paint::controls::GROUPS.iter().position(|g| g.id == "brows").expect("brows group");

    for name in &args[2..] {
        let pos = positions(name);
        let marks = Landmarks::new(&pos);
        let f = marks.frame;
        p.painter.set_head(pos.clone());
        pollster::block_on(p.painter.prepare(size));
        let mut own: Vec<(String, f64)> = hand["heads"][name.as_str()].as_object().map_or(Vec::new(), |o| o.iter().map(|(k, v)| (k.clone(), v.as_f64().expect("number"))).collect());
        own.extend(overrides.iter().cloned());
        let base = p.values(name, &own);
        let b = &BROW_ANCHORS;
        let peak = |s: usize| Anchors::point(&b.brow_peak[s], &pos);
        let mid = Anchors::point(&b.brow_middle, &pos);
        let centre3 = lerp(mid, lerp(peak(0), peak(1), 0.5), 0.5);
        let view_of = |tex: image::RgbImage| KotorFront::ours(&common, &pos, tex, f.across, f.up, f.forward);
        let c = {
            let q = view_of(image::RgbImage::new(1, 1)).project(centre3).0;
            [q[0], q[1] - 2.0]
        };
        let right = {
            let i = Anchors::point(&b.brow_inner[1], &pos);
            let o = Anchors::point(&b.brow_outer[1], &pos);
            let q = view_of(image::RgbImage::new(1, 1)).project(add(lerp(i, o, 0.5), f.up, 2.0)).0;
            q
        };
        if timing {
            let recipe = recipe_from(&base);
            let all = layers_from(&switches);
            for (label, layers) in [("all layers", all), ("all but the brows", head_paint::paint::Layers { brows: false, ..all })] {
                let mut ms = Vec::new();
                for i in 0..9 {
                    let t = std::time::Instant::now();
                    p.painter.paint(size, &recipe, &layers);
                    let _ = pollster::block_on(p.painter.read_rgba(size));
                    if i >= 2 {
                        ms.push(t.elapsed().as_secs_f64() * 1e3);
                    }
                }
                ms.sort_by(f64::total_cmp);
                println!("{name} {size}: {label}: median {:.1} ms, best {:.1} ms (paint and read back)", ms[ms.len() / 2], ms[0]);
            }
            let head = p.painter.head().and_then(|h| h.detail.as_ref()).and_then(|d| d.brows.clone()).expect("brow head");
            let t = std::time::Instant::now();
            for _ in 0..5 {
                let _ = [head.line(&recipe.brows, 0), head.line(&recipe.brows, 1)];
            }
            println!("{name}: laying both brow lines on the skin: {:.1} ms", t.elapsed().as_secs_f64() * 1e3 / 5.0);
            let t = std::time::Instant::now();
            let lay2 = layout();
            let _ = head_paint::brows::BrowHead::new(&pos, &lay2.triangles, &lay2.layout_to_mesh);
            println!("{name}: preparing the brows of a head: {:.1} ms", t.elapsed().as_secs_f64() * 1e3);
            continue;
        }
        if !strips.is_empty() {
            for s in &strips {
                let (id, vals) = s.split_once('=').expect("--strip id=a,b,c");
                let k = list.iter().position(|c| c.id == id).unwrap_or_else(|| panic!("no control {id}"));
                for (i, v) in vals.split(',').enumerate() {
                    let mut values = base.clone();
                    values[k] = v.parse().expect("number");
                    let tex = p.texture(&values, &switches);
                    save(&view_of(tex).tile_at(c, 130.0, 50.0, 6.0), &dir.join(format!("{name}_{id}_{i}.jpg")));
                }
                println!("{name}: strip of {id}");
            }
            continue;
        }
        let tex = p.texture(&base, &switches);
        let ours = view_of(tex);
        save(&ours.tile_at(c, 130.0, 50.0, 6.0), &dir.join(format!("{name}_brows.jpg")));
        save(&ours.tile_at(right, 50.0, 24.0, 20.0), &dir.join(format!("{name}_close.jpg")));
        let face_c = ours.project(add(lerp(marks.nose_tip, centre3, 0.5), f.up, -8.0)).0;
        save(&ours.tile_at(face_c, 110.0, 120.0, 4.0), &dir.join(format!("{name}_face.jpg")));
        let alone_switches: Vec<bool> = (0..head_paint::controls::GROUPS.len()).map(|i| i == brow_group).collect();
        let alone = view_of(p.texture(&base, &alone_switches));
        save(&alone.tile_at(c, 130.0, 50.0, 6.0), &dir.join(format!("{name}_alone.jpg")));
        if !kotor_dir.join(format!("{name}_face.jpg")).exists() {
            let k = KotorFront::open_with_hair(name, f.across, f.up, f.forward);
            save(&k.tile_at(c, 130.0, 50.0, 6.0), &kotor_dir.join(format!("{name}_brows.jpg")));
            save(&k.tile_at(right, 50.0, 24.0, 20.0), &kotor_dir.join(format!("{name}_close.jpg")));
            save(&k.tile_at(face_c, 110.0, 120.0, 4.0), &kotor_dir.join(format!("{name}_face.jpg")));
        }
        println!("{name}: done");
    }
}
