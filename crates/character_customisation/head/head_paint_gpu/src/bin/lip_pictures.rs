//! Pictures of the lip layer (paint track, part C3). Analysis only.
//!
//! `lip_pictures <label> <out dir> <name>... [--layers 111111] [--set <control id>=<value>]... [--hand <json>]`
//!
//! For each head it paints the head's recipe from the live page's data on the GPU at 2,048 pixels, with the head's values from the
//! `--hand` file (an object `heads` of control id to value) and then the `--set` values put in, and writes to `<out dir>/<label>/`:
//! - `<name>_mouth.jpg`: the mouth from the front, 64 by 40 mm at 10 pixels per millimetre;
//! - `<name>_face.jpg`: the whole face from the front, 110 by 120 mm at 4 pixels per millimetre, as at conversation distance;
//! - `<name>_alone.jpg`: the mouth with only the lip layer on, on neutral grey skin.
//!
//! It writes the same views of KOTOR's own head with its own texture to `<out dir>/kotor/` when they are missing.
//! With `--strip <id>=<a>,<b>,<c>` it paints each value of the control on each head and writes only mouth views, named
//! `<name>_<id>_<k>.jpg`.

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
const MOUTH: (f64, f64, f64) = (64.0, 40.0, 10.0);

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

/// The middle of the mouth view: between the corners, 1 mm up.
fn mouth_centre(l: &Landmarks) -> [f64; 3] {
    add(mid(l.mouth_corner[0], l.mouth_corner[1]), l.frame.up, 1.0)
}

fn face_centre(l: &Landmarks) -> [f64; 3] {
    add(mid(l.nose_tip, mid(l.mouth_corner[0], l.mouth_corner[1])), l.frame.up, 8.0)
}

/// The head with its mouth opened by `open_mm` at the middle, for checking the inner lip: every vertex of the lower lip and the chin
/// moves down, fading out toward the corners and 25 to 55 mm below the opening. Near the opening a vertex counts as the lower lip's when
/// its normal faces up, so the upper lip's curl stays with the upper lip. A rough stand-in for the jaw, for pictures only.
fn open_mouth(pos: &[[f64; 3]], layout: &Layout, open_mm: f64) -> Vec<[f64; 3]> {
    use head_paint::lips::{LipHead, LipShape, Lips, STEP_MM};
    let head = LipHead::new(pos, &layout.triangles, &layout.layout_to_mesh).expect("lip anchors");
    let shape = LipShape::new(&Lips::DEFAULT, &head);
    let mut normals = vec![[0.0f64; 3]; pos.len()];
    for t in &layout.mesh_triangles {
        let [a, b, c] = t.map(|i| pos[i as usize]);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
        for &i in t {
            for k in 0..3 {
                normals[i as usize][k] += n[k];
            }
        }
    }
    let f = head.frame;
    let half = 0.5 * (shape.corners[1][0] - shape.corners[0][0]);
    let smooth = |a: f64, b: f64, x: f64| { let t = ((x - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
    pos.iter()
        .zip(&normals)
        .map(|(p, n)| {
            let q = f.uvw(p.map(|x| 1000.0 * x));
            let i = ((q[0] - shape.u0) / STEP_MM).round().clamp(0.0, (shape.opening.len() - 1) as f64) as usize;
            let dv = q[1] - shape.opening[i];
            let up_n = n[0] * f.up[0] + n[1] * f.up[1] + n[2] * f.up[2];
            let lower = if dv < -3.0 { true } else if dv > 3.0 { false } else if q[2] < -1.0 { up_n > 0.0 } else { dv < 0.0 };
            if !lower || q[2] < -30.0 {
                return *p;
            }
            let w = (1.0 - smooth(0.2 * half, 1.0 * half, q[0].abs())) * (1.0 - smooth(25.0, 55.0, -dv));
            let d = open_mm * w / 1000.0;
            [p[0] - d * f.up[0], p[1] - d * f.up[1], p[2] - d * f.up[2]]
        })
        .collect()
}

fn save(img: &image::RgbImage, path: &Path) {
    img.save(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

struct Painter {
    painter: GpuPainter,
    common: Common,
    recipes: Value,
}

impl Painter {
    fn new() -> Self {
        let layout = layout();
        let common = Common::load();
        assert_eq!(common.uv, layout.uv, "the live page's layout differs from the bundle's");
        let painter = GpuPainter::new(Gpu::blocking().expect("GPU"), layout);
        let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
        Painter { painter, common, recipes }
    }

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
    let switches: Vec<bool> = parse(&mut args, "--layers").pop().unwrap_or_else(|| "1".repeat(head_paint::controls::GROUPS.len())).chars().map(|c| c == '1').collect();
    let strips = parse(&mut args, "--strip");
    let open: Option<f64> = parse(&mut args, "--open").pop().map(|v| v.parse().expect("mm"));
    let hand: Value = parse(&mut args, "--hand").pop().map_or(Value::Null, |p| serde_json::from_str(&std::fs::read_to_string(&p).expect("hand file")).expect("json"));
    if args.len() < 3 {
        eprintln!("Usage: lip_pictures <label> <out dir> <name>... [--layers 111111] [--set id=value]... [--strip id=a,b,c]... [--hand file]");
        std::process::exit(2);
    }
    let label = &args[0];
    let out = PathBuf::from(&args[1]);
    let dir = out.join(label);
    let kotor_dir = out.join("kotor");
    std::fs::create_dir_all(&dir).expect("out");
    std::fs::create_dir_all(&kotor_dir).expect("out");
    let mut p = Painter::new();
    let list = controls();

    for name in &args[2..] {
        let pos = positions(name);
        let marks = Landmarks::new(&pos);
        let f = marks.frame;
        p.painter.set_head(pos.clone());
        pollster::block_on(p.painter.prepare(N));
        let mut own: Vec<(String, f64)> = hand["heads"][name.as_str()].as_object().map_or(Vec::new(), |o| o.iter().map(|(k, v)| (k.clone(), v.as_f64().expect("number"))).collect());
        own.extend(overrides.iter().cloned());
        let base = p.values(name, &own);
        let centre = mouth_centre(&marks);

        if !strips.is_empty() {
            for s in &strips {
                let (id, vals) = s.split_once('=').expect("--strip id=a,b,c");
                let k = list.iter().position(|c| c.id == id).unwrap_or_else(|| panic!("no control {id}"));
                for (i, v) in vals.split(',').enumerate() {
                    let mut values = base.clone();
                    values[k] = v.parse().expect("number");
                    let tex = p.texture(&values, &switches);
                    let ours = KotorFront::ours(&p.common, &pos, tex, f.across, f.up, f.forward);
                    let q = ours.project(centre).0;
                    save(&ours.tile_at(q, MOUTH.0, MOUTH.1, MOUTH.2), &dir.join(format!("{name}_{id}_{i}.jpg")));
                }
                println!("{name}: strip of {id}");
            }
            continue;
        }

        let tex = p.texture(&base, &switches);
        if let Some(mm) = open {
            let opened = open_mouth(&pos, &layout(), mm);
            let view = KotorFront::ours(&p.common, &opened, tex.clone(), f.across, f.up, f.forward);
            let q = view.project(add(centre, f.up, -0.4 * mm)).0;
            save(&view.tile_at(q, 40.0, 30.0, 16.0), &dir.join(format!("{name}_open.jpg")));
            save(&tex, &dir.join(format!("{name}_texture.png")));
            println!("{name}: open");
            continue;
        }
        save(&image::imageops::crop_imm(&tex, 600, 1300, 660, 340).to_image(), &dir.join(format!("{name}_layout.jpg")));
        let bake = fold_common::bake_path(name);
        if !kotor_dir.join(format!("{name}_layout.jpg")).exists() {
            let k = image::open(&bake).expect("bake").to_rgb8();
            save(&image::imageops::crop_imm(&k, 600, 1300, 660, 340).to_image(), &kotor_dir.join(format!("{name}_layout.jpg")));
        }
        let ours = KotorFront::ours(&p.common, &pos, tex, f.across, f.up, f.forward);
        let q = ours.project(centre).0;
        save(&ours.tile_at(q, MOUTH.0, MOUTH.1, MOUTH.2), &dir.join(format!("{name}_mouth.jpg")));
        let qf = ours.project(face_centre(&marks)).0;
        save(&ours.tile_at(qf, 110.0, 120.0, 4.0), &dir.join(format!("{name}_face.jpg")));
        let lip_group = head_paint::controls::GROUPS.iter().position(|g| g.id == "lips").expect("lips group");
        let alone_switches: Vec<bool> = (0..head_paint::controls::GROUPS.len()).map(|i| i == lip_group).collect();
        let alone_tex = p.texture(&base, &alone_switches);
        let alone = KotorFront::ours(&p.common, &pos, alone_tex, f.across, f.up, f.forward);
        save(&alone.tile_at(q, MOUTH.0, MOUTH.1, MOUTH.2), &dir.join(format!("{name}_alone.jpg")));
        if !kotor_dir.join(format!("{name}_face.jpg")).exists() {
            let k = KotorFront::open(name, f.across, f.up, f.forward);
            save(&k.tile_at(q, MOUTH.0, MOUTH.1, MOUTH.2), &kotor_dir.join(format!("{name}_mouth.jpg")));
            save(&k.tile_at(qf, 110.0, 120.0, 4.0), &kotor_dir.join(format!("{name}_face.jpg")));
        }
        println!("{name}: done");
    }
}
