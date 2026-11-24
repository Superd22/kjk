//! Pictures of the procedural eyeball next to KOTOR's eyes (paint track, part C, the eye step). Analysis only.
//!
//! `eyeball_pictures <out dir> <label> <name>... [--kotor] [--values <json>] [--set id=value]... [--strip id=a,b,c]... [--layers 1111111]`
//!
//! For each head it paints the head's recipe from the live page's data on the GPU, with the skin at 2,048 pixels and the eyeballs at 512
//! pixels per eye, and writes to `<out dir>/<label>/`:
//! - `<name>_ball.png`, the eyeball texture itself;
//! - `<name>_eye.png`, the eye on the x > 0 side, 34 by 24 mm at 20 pixels per millimetre, close up;
//! - `<name>_eyes.png`, both eyes, 104 by 34 mm at 8 pixels per millimetre;
//! - `<name>_face.png`, the whole face, 110 by 120 mm at 4 pixels per millimetre, as at conversation distance.
//!
//! With `--kotor` it writes the same views of KOTOR's head, with KOTOR's eye texture on its eyeballs, to `<out dir>/kotor/`.
//! `--values <json>` reads per-head overrides: an object of head names, each an object of control ids and values. `--set` wins over them.
//! With `--strip id=a,b,c` it writes only `<name>_<id>_<k>.png`, the close-up of our eye at each value.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;
#[path = "eyes_shared/common.rs"]
mod common;

use common::*;
use fold_common::KotorFront;
use head_paint::controls::recipe_from;
use head_paint::folds::Landmarks;
use head_paint_gpu::eyeball::EyeballPainter;
use std::path::PathBuf;
use std::time::Instant;

const KOTOR_EYES: &str = "/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data/tex/eyes";

fn parse(args: &mut Vec<String>, flag: &str) -> Vec<String> {
    let mut out = Vec::new();
    while let Some(k) = args.iter().position(|a| a == flag) {
        out.push(args.remove(k + 1));
        args.remove(k);
    }
    out
}

struct Windows {
    eyes: [f64; 2],
    eye: [f64; 2],
    face: [f64; 2],
}

fn windows(v: &KotorFront, pos: &[[f64; 3]], l: &Landmarks) -> Windows {
    let e = head_paint::eyes::eye_shapes(pos).expect("default mesh");
    let both = mid(e[0].centre, e[1].centre);
    let face = add(mid(l.nose_tip, mid(l.mouth_corner[0], l.mouth_corner[1])), l.frame.up, 8.0);
    let one = add(e[1].centre, l.frame.up, 1.0);
    Windows { eyes: v.project(add(both, l.frame.up, 1.0)).0, eye: v.project(one).0, face: v.project(face).0 }
}

fn ball_texture(p: &Painter, balls: &EyeballPainter, values: &[f64], pos: &[[f64; 3]]) -> BallTex {
    let gpu = p.painter.gpu();
    let ball = recipe_from(values).eyes.ball;
    balls.paint(gpu, &ball, head_paint::eyeball::iris_centres(pos, &ball), head_paint::eyes::eyeball_radii(pos));
    let rgba = pollster::block_on(balls.read_rgba(gpu));
    let n = balls.size();
    image::RgbImage::from_fn(2 * n, n, |x, y| {
        let at = 4 * (y * 2 * n + x) as usize;
        image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
    })
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let kotor = args.iter().position(|a| a == "--kotor").map(|k| args.remove(k)).is_some();
    let overrides: Vec<(String, f64)> = parse(&mut args, "--set")
        .iter()
        .map(|p| {
            let (id, v) = p.split_once('=').expect("--set id=value");
            (id.to_string(), v.parse().expect("number"))
        })
        .collect();
    let per_head: serde_json::Value = parse(&mut args, "--values").pop().map(|p| serde_json::from_str(&std::fs::read_to_string(p).expect("values")).expect("json")).unwrap_or(serde_json::Value::Null);
    let switches: Vec<bool> = parse(&mut args, "--layers").pop().unwrap_or_else(|| "1111111".into()).chars().map(|c| c == '1').collect();
    let strips = parse(&mut args, "--strip");
    if args.len() < 3 {
        eprintln!("usage: eyeball_pictures <out dir> <label> <name>... [--kotor] [--values json] [--set id=value]... [--strip id=a,b,c]... [--layers 1111111]");
        std::process::exit(2);
    }
    let out = PathBuf::from(&args[0]);
    let dir = out.join(&args[1]);
    let kdir = out.join("kotor");
    std::fs::create_dir_all(&dir).expect("out");
    std::fs::create_dir_all(&kdir).expect("out");
    let names: Vec<String> = if args[2] == "all" { head_names() } else { args[2..].to_vec() };
    let mut p = Painter::new();
    let balls = EyeballPainter::new(p.painter.gpu());
    for name in &names {
        let pos = positions(name);
        let marks = Landmarks::new(&pos);
        let f = marks.frame;
        p.set_head(&pos);
        let mut ov: Vec<(String, f64)> = per_head[name.as_str()].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_f64().expect("number"))).collect()).unwrap_or_default();
        ov.extend(overrides.iter().cloned());
        let base = p.values(name, &ov);
        let our_balls = our_balls(&pos);
        let tex = p.texture(&base, &switches);
        let ours = KotorFront::ours(&p.common, &pos, tex, f.across, f.up, f.forward);
        let w = windows(&ours, &pos, &marks);
        if !strips.is_empty() {
            let list = head_paint::controls::controls();
            for s in &strips {
                let (id, vals) = s.split_once('=').expect("--strip id=a,b,c");
                let k = list.iter().position(|c| c.id == id).unwrap_or_else(|| panic!("no control {id}"));
                for (i, x) in vals.split(',').enumerate() {
                    let mut values = base.clone();
                    values[k] = x.parse().expect("number");
                    set_ball_texture(Some(ball_texture(&p, &balls, &values, &pos)), head_paint::eyes::eyeball_radii(&pos));
                    let img = view(&ours, &our_balls, &Vec::new(), f.forward, f.across, f.up, w.eye, 34.0, 24.0, 20.0);
                    img.save(dir.join(format!("{name}_{id}_{i}.png"))).expect("save");
                }
            }
            println!("{name}: strips");
            continue;
        }
        let t0 = Instant::now();
        let bt = ball_texture(&p, &balls, &base, &pos);
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        bt.save(dir.join(format!("{name}_ball.png"))).expect("save");
        set_ball_texture(Some(bt), head_paint::eyes::eyeball_radii(&pos));
        let shot = |v: &KotorFront, b: &[Ball], tris: &Tris| {
            [
                view(v, b, tris, f.forward, f.across, f.up, w.eyes, 104.0, 34.0, 8.0),
                view(v, b, tris, f.forward, f.across, f.up, w.eye, 34.0, 24.0, 20.0),
                view(v, b, tris, f.forward, f.across, f.up, w.face, 110.0, 120.0, 4.0),
            ]
        };
        let [a, b, c] = shot(&ours, &our_balls, &Vec::new());
        a.save(dir.join(format!("{name}_eyes.png"))).expect("save");
        b.save(dir.join(format!("{name}_eye.png"))).expect("save");
        c.save(dir.join(format!("{name}_face.png"))).expect("save");
        if kotor {
            let kt = image::open(format!("{KOTOR_EYES}/{name}_eyes.png")).map(|i| i.to_rgb8()).ok();
            set_ball_texture(kt, [head_paint::eyes::EYEBALL_RADIUS_MM; 2]);
            let k = KotorFront::open(name, f.across, f.up, f.forward);
            let (kb, ktris) = kotor_balls(name);
            let [a, b, c] = shot(&k, &kb, &ktris);
            a.save(kdir.join(format!("{name}_eyes.png"))).expect("save");
            b.save(kdir.join(format!("{name}_eye.png"))).expect("save");
            c.save(kdir.join(format!("{name}_face.png"))).expect("save");
        }
        println!("{name}: done, eyeball painted and read back in {ms:.1} ms");
    }
}
