//! What the eye paint tools share: the live page's data, a GPU painter of a head's recipe, and front views of the eyes with a plain
//! eyeball drawn in, the same on KOTOR's head and on ours. Analysis only, never shipped. Included with `#[path]` after `bundle.rs` and
//! `fold_common.rs`.

#![allow(dead_code)]

use crate::bundle::Bundle;
use crate::fold_common::{Common, KotorFront, heads_dir};
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint::live::Layout;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::Value;
use std::path::Path;

pub const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
pub const N: u32 = 2048;

pub fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().expect("8"))).collect()
}

pub fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().expect("4"))).collect()
}

pub fn layout() -> Layout {
    let l = Path::new(DATA).join("layout");
    let triples = |v: Vec<u32>| v.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect::<Vec<_>>();
    Layout {
        uv: f64s(&l.join("uv.f64")).chunks_exact(2).map(|p| [p[0], p[1]]).collect(),
        triangles: triples(u32s(&l.join("tris.u32"))),
        layout_to_mesh: u32s(&l.join("vmap.u32")).into_iter().map(|i| i as usize).collect(),
        mesh_triangles: triples(u32s(&l.join("mesh_tris.u32"))),
    }
}

pub fn positions(name: &str) -> Vec<[f64; 3]> {
    f64s(&Path::new(DATA).join("heads").join(format!("{name}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect()
}

pub fn head_names() -> Vec<String> {
    let r: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
    let mut v: Vec<String> = r["heads"].as_object().expect("heads").keys().cloned().collect();
    v.sort();
    v
}

pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}
pub fn mid(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|k| 0.5 * (a[k] + b[k]))
}

pub struct Painter {
    pub painter: GpuPainter,
    pub common: Common,
    pub recipes: Value,
}

impl Painter {
    pub fn new() -> Self {
        let layout = layout();
        let common = Common::load();
        assert_eq!(common.uv, layout.uv, "the live page's layout differs from the bundle's");
        let painter = GpuPainter::new(Gpu::blocking().expect("GPU"), layout);
        let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
        Painter { painter, common, recipes }
    }

    /// The head's values from the live page's data, with `overrides` put in. A control the data lacks takes its default.
    pub fn values(&self, name: &str, overrides: &[(String, f64)]) -> Vec<f64> {
        let head = &self.recipes["heads"][name];
        let list = controls();
        let mut v: Vec<f64> = list.iter().map(|c| head[&c.id].as_f64().unwrap_or(c.default)).collect();
        for (id, x) in overrides {
            let k = list.iter().position(|c| &c.id == id).unwrap_or_else(|| panic!("no control {id}"));
            v[k] = *x;
        }
        v
    }

    pub fn set_head(&mut self, pos: &[[f64; 3]]) {
        self.painter.set_head(pos.to_vec());
        pollster::block_on(self.painter.prepare(N));
    }

    pub fn texture(&mut self, values: &[f64], switches: &[bool]) -> image::RgbImage {
        let recipe = recipe_from(values);
        self.painter.paint(N, &recipe, &layers_from(switches));
        let rgba = pollster::block_on(self.painter.read_rgba(N));
        image::RgbImage::from_fn(N, N, |x, y| {
            let at = 4 * (y * N + x) as usize;
            image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
        })
    }
}

/// An eyeball: its centre in millimetres and its radius.
#[derive(Clone, Copy)]
pub struct Ball {
    pub centre: [f64; 3],
    pub radius: f64,
}

/// KOTOR's eyeball triangles, in millimetres. KOTOR's eyeballs are not whole spheres, so they are drawn from their triangles.
pub type Tris = Vec<[[f64; 3]; 3]>;

/// KOTOR's eyeballs on its own head, in the frame of our fitted head: their centres and a radius from their triangles.
pub fn kotor_balls(name: &str) -> (Vec<Ball>, Tris) {
    let b = Bundle::open(&heads_dir().join(name));
    if !b.manifest["arrays"].as_object().is_some_and(|a| a.contains_key("k_eyes")) {
        return (Vec::new(), Vec::new());
    }
    let centres: Vec<[f64; 3]> = b.points("k_eyes").into_iter().map(|p| p.map(|x| 1000.0 * x)).collect();
    let tris: Vec<[f64; 3]> = b.points("eye_tris").into_iter().map(|p| p.map(|x| 1000.0 * x)).collect();
    let balls = centres
        .iter()
        .map(|&c| {
            let d: Vec<f64> = tris.iter().filter(|p| centres.iter().all(|o| dot(sub(**p, *o), sub(**p, *o)) >= dot(sub(**p, c), sub(**p, c)))).map(|p| dot(sub(*p, c), sub(*p, c)).sqrt()).collect();
            let r = if d.is_empty() { 12.3 } else { d.iter().sum::<f64>() / d.len() as f64 };
            Ball { centre: c, radius: r }
        })
        .collect();
    (balls, tris.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect())
}

/// Our eyeballs, from the lid edge data's centres and radii.
pub fn our_balls(pos: &[[f64; 3]]) -> Vec<Ball> {
    head_paint::eyes::eye_shapes(pos).map(|e| e.iter().map(|s| Ball { centre: s.centre, radius: s.radius }).collect()).unwrap_or_default()
}

/// A plain eyeball's colour where a ray along `-forward` meets it, the same on every head: a pale eye white, a brown iris 6 mm in
/// radius with a darker rim, and a pupil 2.3 mm in radius, looking straight ahead.
fn ball_colour(normal: [f64; 3], forward: [f64; 3], radius: f64) -> [f64; 3] {
    let c = dot(normal, forward).clamp(-1.0, 1.0);
    let off = radius * (1.0 - c * c).max(0.0).sqrt();
    let light = 0.55 + 0.45 * c.max(0.0);
    if c > 0.0 && off < 2.3 {
        return [12.0, 10.0, 10.0];
    }
    if c > 0.0 && off < 6.0 {
        let rim = ((off - 4.8) / 1.2).clamp(0.0, 1.0);
        let base = [110.0, 78.0, 52.0];
        return base.map(|v| v * (1.0 - 0.55 * rim) * light);
    }
    [222.0, 212.0, 200.0].map(|v| v * light)
}

/// An eyeball texture as the viewer draws it: both eyes side by side, each a square seen from the front (`head_paint::eyeball`), in sRGB.
pub type BallTex = image::RgbImage;

thread_local! {
    static BALL_TEX: std::cell::RefCell<Option<(BallTex, [f64; 2])>> = const { std::cell::RefCell::new(None) };
}

/// Sets the texture that the next views draw on the eyeballs, or `None` for the plain eyeball. `radii` are the radii of our eyeballs
/// the texture was painted for, in millimetres, the eye on the x < 0 side first (`head_paint::eyes::eyeball_radii`).
pub fn set_ball_texture(t: Option<BallTex>, radii: [f64; 2]) {
    BALL_TEX.with(|b| *b.borrow_mut() = t.map(|t| (t, radii)));
}

/// The colour of a ball's point from the texture set by `set_ball_texture`, with the same soft light as the plain eyeball. `side` is the
/// half of the texture.
fn textured(normal: [f64; 3], scale: f64, forward: [f64; 3], across: [f64; 3], up: [f64; 3], side: usize) -> Option<[f64; 3]> {
    BALL_TEX.with(|b| {
        let b = b.borrow();
        let (t, radii) = b.as_ref()?;
        let n = f64::from(t.height());
        let c = dot(normal, forward);
        // The texture's square spans the radius of our eyeball it was painted for each way from the ball's centre, on KOTOR's balls too,
        // whatever their own radius. Its axes are the world's x and y, as on the viewer's eyeball mesh, not the face's frame, which is
        // pitched. The texture's left half, side 0, is the eye on the x > 0 side, the second of `radii`.
        let r = radii[1 - side];
        let (_, _) = (across, up);
        let (x, y) = (normal[0] * scale / r, normal[1] * scale / r);
        let fx = ((x + 1.0) * 0.5 * n - 0.5).clamp(0.0, n - 1.001) + side as f64 * n;
        let fy = ((1.0 - y) * 0.5 * n - 0.5).clamp(0.0, n - 1.001);
        let (x0, y0) = (fx.floor() as u32, fy.floor() as u32);
        let (ax, ay) = (fx - f64::from(x0), fy - f64::from(y0));
        let px = |x: u32, y: u32| t.get_pixel(x.min(t.width() - 1), y.min(t.height() - 1)).0.map(f64::from);
        let (p00, p10, p01, p11) = (px(x0, y0), px(x0 + 1, y0), px(x0, y0 + 1), px(x0 + 1, y0 + 1));
        let light = 0.8 + 0.2 * c.max(0.0);
        Some([0, 1, 2].map(|k| ((p00[k] * (1.0 - ax) + p10[k] * ax) * (1.0 - ay) + (p01[k] * (1.0 - ax) + p11[k] * ax) * ay) * light))
    })
}

/// Which half of the texture each ball takes: 0, the left half, for the ball on the x > 0 side, as on the viewer's eyeball mesh.
fn sides(balls: &[Ball], _across: [f64; 3]) -> Vec<usize> {
    balls.iter().map(|b| usize::from(balls.iter().any(|o| o.centre[0] > b.centre[0]))).collect()
}

/// A front view of a head with eyeballs drawn in where they stand in front of the head's own surface, `per_mm` pixels per millimetre,
/// centred on a point of the face's plane. Each pixel averages 3 by 3 points.
pub fn view(v: &KotorFront, balls: &[Ball], tris: &Tris, forward: [f64; 3], across: [f64; 3], up: [f64; 3], centre: [f64; 2], w_mm: f64, h_mm: f64, per_mm: f64) -> image::RgbImage {
    view_masked(v, balls, tris, forward, across, up, centre, w_mm, h_mm, per_mm).0
}

/// The same view, with the share of each pixel, row by row, where an eyeball is seen.
#[allow(clippy::too_many_arguments)]
pub fn view_masked(v: &KotorFront, balls: &[Ball], tris: &Tris, forward: [f64; 3], across: [f64; 3], up: [f64; 3], centre: [f64; 2], w_mm: f64, h_mm: f64, per_mm: f64) -> (image::RgbImage, Vec<f32>) {
    let origin = [centre[0] - 0.5 * w_mm, centre[1] + 0.5 * h_mm];
    let step = 1.0 / per_mm;
    let (w, h) = ((w_mm * per_mm) as u32, (h_mm * per_mm) as u32);
    let mut mask = vec![0.0f32; (w * h) as usize];
    let side = sides(balls, across);
    let img = image::RgbImage::from_fn(w, h, |x, y| {
        let mut sum = [0.0f64; 3];
        let mut on_ball = 0.0f32;
        for j in 0..3 {
            for i in 0..3 {
                let q = [origin[0] + (f64::from(x) + (f64::from(i) + 0.5) / 3.0) * step, origin[1] - (f64::from(y) + (f64::from(j) + 0.5) / 3.0) * step];
                let face = v.front_rgb(q).map(|c| c.map(f64::from));
                let face_depth = v.front_point(q).map(|p| dot(p, forward));
                let mut colour = face.unwrap_or([40.0, 40.0, 40.0]);
                let mut best = face_depth.unwrap_or(f64::NEG_INFINITY);
                let mut hit_ball = false;
                for t in tris.iter() {
                    let a = t.map(|p| [dot(p, across), dot(p, up)]);
                    let den = (a[1][1] - a[2][1]) * (a[0][0] - a[2][0]) + (a[2][0] - a[1][0]) * (a[0][1] - a[2][1]);
                    if den.abs() < 1e-12 {
                        continue;
                    }
                    let w0 = ((a[1][1] - a[2][1]) * (q[0] - a[2][0]) + (a[2][0] - a[1][0]) * (q[1] - a[2][1])) / den;
                    let w1 = ((a[2][1] - a[0][1]) * (q[0] - a[2][0]) + (a[0][0] - a[2][0]) * (q[1] - a[2][1])) / den;
                    let w = [w0, w1, 1.0 - w0 - w1];
                    if w.iter().any(|&x| x < 0.0) {
                        continue;
                    }
                    let p = [0, 1, 2].map(|k| w[0] * t[0][k] + w[1] * t[1][k] + w[2] * t[2][k]);
                    let depth = dot(p, forward);
                    if depth > best {
                        let b = (0..balls.len()).min_by(|&x, &y| dot(sub(p, balls[x].centre), sub(p, balls[x].centre)).total_cmp(&dot(sub(p, balls[y].centre), sub(p, balls[y].centre))));
                        if let Some(i) = b {
                            let b = &balls[i];
                            best = depth;
                            hit_ball = true;
                            let d = sub(p, b.centre);
                            let l = dot(d, d).sqrt().max(1e-9);
                            let n = d.map(|x| x / l);
                            colour = textured(n, l, forward, across, up, side[i]).unwrap_or_else(|| ball_colour(n, forward, l));
                        }
                    }
                }
                for (i, b) in balls.iter().enumerate().filter(|_| tris.is_empty()) {
                    let bq = [dot(b.centre, across), dot(b.centre, up)];
                    let r2 = (q[0] - bq[0]).powi(2) + (q[1] - bq[1]).powi(2);
                    if r2 >= b.radius * b.radius {
                        continue;
                    }
                    let h = (b.radius * b.radius - r2).sqrt();
                    let depth = dot(b.centre, forward) + h;
                    if depth > best {
                        best = depth;
                        hit_ball = true;
                        let n = [0, 1, 2].map(|k| ((q[0] - bq[0]) * across[k] + (q[1] - bq[1]) * up[k] + h * forward[k]) / b.radius);
                        colour = textured(n, b.radius, forward, across, up, side[i]).unwrap_or_else(|| ball_colour(n, forward, b.radius));
                    }
                }
                for k in 0..3 {
                    sum[k] += colour[k] / 9.0;
                }
                if hit_ball {
                    on_ball += 1.0 / 9.0;
                }
            }
        }
        mask[(y * w + x) as usize] = on_ball;
        image::Rgb(sum.map(|v| v.round().clamp(0.0, 255.0) as u8))
    });
    (img, mask)
}

/// A label written in the top left corner of a picture, in a 5 by 7 pixel font scaled by 2: only digits, lower case letters, `_`, `-`
/// and spaces are drawn.
pub fn label(img: &mut image::RgbImage, text: &str) {
    let w = 12 * text.len() as u32 + 4;
    for y in 0..18.min(img.height()) {
        for x in 0..w.min(img.width()) {
            img.put_pixel(x, y, image::Rgb([0, 0, 0]));
        }
    }
    for (n, ch) in text.chars().enumerate() {
        let g = glyph(ch);
        for (row, bits) in g.iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let (px, py) = (2 + 12 * n as u32 + 2 * col + dx, 2 + 2 * row as u32 + dy);
                            if px < img.width() && py < img.height() {
                                img.put_pixel(px, py, image::Rgb([255, 230, 60]));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn glyph(c: char) -> [u8; 7] {
    match c.to_ascii_lowercase() {
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [31, 2, 4, 2, 1, 17, 14],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 30, 1, 1, 17, 14],
        '6' => [6, 8, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 2, 12],
        'a' => [14, 17, 17, 31, 17, 17, 17],
        'b' => [30, 17, 17, 30, 17, 17, 30],
        'c' => [14, 17, 16, 16, 16, 17, 14],
        'd' => [28, 18, 17, 17, 17, 18, 28],
        'e' => [31, 16, 16, 30, 16, 16, 31],
        'f' => [31, 16, 16, 30, 16, 16, 16],
        'g' => [14, 17, 16, 23, 17, 17, 15],
        'h' => [17, 17, 17, 31, 17, 17, 17],
        'i' => [14, 4, 4, 4, 4, 4, 14],
        'j' => [7, 2, 2, 2, 2, 18, 12],
        'k' => [17, 18, 20, 24, 20, 18, 17],
        'l' => [16, 16, 16, 16, 16, 16, 31],
        'm' => [17, 27, 21, 21, 17, 17, 17],
        'n' => [17, 17, 25, 21, 19, 17, 17],
        'o' => [14, 17, 17, 17, 17, 17, 14],
        'p' => [30, 17, 17, 30, 16, 16, 16],
        'q' => [14, 17, 17, 17, 21, 18, 13],
        'r' => [30, 17, 17, 30, 20, 18, 17],
        's' => [15, 16, 16, 14, 1, 1, 30],
        't' => [31, 4, 4, 4, 4, 4, 4],
        'u' => [17, 17, 17, 17, 17, 17, 14],
        'v' => [17, 17, 17, 17, 17, 10, 4],
        'w' => [17, 17, 17, 21, 21, 21, 10],
        'x' => [17, 17, 10, 4, 10, 17, 17],
        'y' => [17, 17, 17, 10, 4, 4, 4],
        'z' => [31, 1, 2, 4, 8, 16, 31],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        _ => [0; 7],
    }
}

/// Pictures side by side in one row, with a gap of `gap` pixels.
pub fn row(images: &[image::RgbImage], gap: u32) -> image::RgbImage {
    let w: u32 = images.iter().map(|i| i.width()).sum::<u32>() + gap * (images.len() as u32).saturating_sub(1);
    let h = images.iter().map(|i| i.height()).max().unwrap_or(0);
    let mut out = image::RgbImage::from_pixel(w, h, image::Rgb([20, 20, 20]));
    let mut x = 0;
    for i in images {
        image::imageops::replace(&mut out, i, i64::from(x), 0);
        x += i.width() + gap;
    }
    out
}

/// Pictures stacked in a column, with a gap of `gap` pixels.
pub fn column(images: &[image::RgbImage], gap: u32) -> image::RgbImage {
    let h: u32 = images.iter().map(|i| i.height()).sum::<u32>() + gap * (images.len() as u32).saturating_sub(1);
    let w = images.iter().map(|i| i.width()).max().unwrap_or(0);
    let mut out = image::RgbImage::from_pixel(w, h, image::Rgb([20, 20, 20]));
    let mut y = 0;
    for i in images {
        image::imageops::replace(&mut out, i, 0, i64::from(y));
        y += i.height() + gap;
    }
    out
}
