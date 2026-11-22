//! What the fold tools share: the data folders, the layout, a head's positions, and KOTOR's bake read as lightness. The fold tools
//! include it with `#[path]`, after `bundle.rs`, because it never ships.

#![allow(dead_code)]

use crate::bundle::Bundle;
use head_paint::colour::srgb_to_lab;
use head_paint::pixel_map::PixelMap;
use std::path::{Path, PathBuf};

pub const RES: usize = 2048;
pub const SCRATCH: &str = "/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad";
pub const DATA: &str = "/Users/david/.claude/tools/likeness-fit-data/paint/folds";
pub const PAGES: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/folds";

pub fn common_dir() -> PathBuf {
    Path::new(SCRATCH).join("build-smooth-default/prep/common")
}
pub fn heads_dir() -> PathBuf {
    Path::new(SCRATCH).join("build-smooth-default/prep/heads")
}
pub fn bake_path(name: &str) -> PathBuf {
    Path::new(SCRATCH).join(format!("viewer-data/tex/v2/{name}_ours.jpg"))
}
pub fn shading_fit_path() -> PathBuf {
    Path::new(SCRATCH).join("build-smooth-default/fit_shading.json")
}

/// The 57 test heads, in the order of the shading fit.
pub fn head_names() -> Vec<String> {
    let fit: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(shading_fit_path()).expect("shading fit")).expect("json");
    fit["heads"].as_array().expect("heads").iter().map(|h| h["name"].as_str().expect("name").to_string()).collect()
}

/// The layout and the base head, shared by every head.
pub struct Common {
    pub uv: Vec<[f64; 2]>,
    pub tris: Vec<[u32; 3]>,
    pub mesh_tris: Vec<[u32; 3]>,
    pub vmap: Vec<usize>,
    pub base: Vec<[f64; 3]>,
    pub map: PixelMap,
}

impl Common {
    pub fn load() -> Self {
        let b = Bundle::open(&common_dir());
        let uv = b.uv();
        let tris = b.triangles("tris");
        let map = PixelMap::new(&uv, &tris, RES);
        Common { mesh_tris: b.triangles("old_tris"), vmap: b.layout_to_mesh(), base: b.points("base"), uv, tris, map }
    }

    pub fn fitted(&self, name: &str) -> Vec<[f64; 3]> {
        Bundle::open(&heads_dir().join(name)).points("fitted")
    }
}

/// A picture's lightness (L*) on the 2,048-pixel layout, with the pixels that hold paint. Grey pixels (the bake's unpainted colour) and
/// black pixels hold none.
pub struct Lightness {
    pub l: Vec<f32>,
    pub valid: Vec<bool>,
}

impl Lightness {
    pub fn from_rgb(img: &image::RgbImage) -> Self {
        let (w, h) = img.dimensions();
        assert_eq!((w as usize, h as usize), (RES, RES));
        let mut l = vec![0.0f32; RES * RES];
        let mut valid = vec![false; RES * RES];
        for (i, px) in img.pixels().enumerate() {
            let p = px.0;
            let grey = p.iter().all(|&v| (i32::from(v) - 123).abs() <= 2);
            let black = p.iter().all(|&v| v < 6);
            if grey || black {
                continue;
            }
            l[i] = srgb_to_lab(p.map(|v| f64::from(v) / 255.0))[0] as f32;
            valid[i] = true;
        }
        Lightness { l, valid }
    }

    pub fn open(path: &Path) -> Self {
        Self::from_rgb(&image::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).to_rgb8())
    }

    /// The lightness at a point of the layout given as (u, v), read between the four nearest pixels. It is `None` if one of them holds no
    /// paint.
    pub fn at_uv(&self, uv: [f64; 2]) -> Option<f32> {
        let x = uv[0] * RES as f64 - 0.5;
        let y = (1.0 - uv[1]) * RES as f64 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        if x0 < 0.0 || y0 < 0.0 || x0 + 1.0 >= RES as f64 || y0 + 1.0 >= RES as f64 {
            return None;
        }
        let (fx, fy) = ((x - x0) as f32, (y - y0) as f32);
        let at = y0 as usize * RES + x0 as usize;
        let idx = [at, at + 1, at + RES, at + RES + 1];
        if !idx.iter().all(|&i| self.valid[i]) {
            return None;
        }
        let v = idx.map(|i| self.l[i]);
        Some((v[0] * (1.0 - fx) + v[1] * fx) * (1.0 - fy) + (v[2] * (1.0 - fx) + v[3] * fx) * fy)
    }
}

/// The RGB colour at a layout point, read between the four nearest pixels.
pub fn rgb_at_uv(img: &image::RgbImage, uv: [f64; 2]) -> [f32; 3] {
    let x = (uv[0] * RES as f64 - 0.5).clamp(0.0, RES as f64 - 1.001);
    let y = ((1.0 - uv[1]) * RES as f64 - 0.5).clamp(0.0, RES as f64 - 1.001);
    let (x0, y0) = (x.floor() as u32, y.floor() as u32);
    let (fx, fy) = ((x - f64::from(x0)) as f32, (y - f64::from(y0)) as f32);
    let p = |dx: u32, dy: u32| img.get_pixel(x0 + dx, y0 + dy).0.map(f32::from);
    let (a, b, c, d) = (p(0, 0), p(1, 0), p(0, 1), p(1, 1));
    [0, 1, 2].map(|k| (a[k] * (1.0 - fx) + b[k] * fx) * (1.0 - fy) + (c[k] * (1.0 - fx) + d[k] * fx) * fy)
}

pub fn median(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

pub fn sd(v: &[f64]) -> f64 {
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len().max(1) as f64).sqrt()
}

pub fn correlation(a: &[f64], b: &[f64]) -> f64 {
    let (ma, mb) = (mean(a), mean(b));
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let va: f64 = a.iter().map(|x| (x - ma).powi(2)).sum();
    let vb: f64 = b.iter().map(|y| (y - mb).powi(2)).sum();
    if va > 0.0 && vb > 0.0 { cov / (va * vb).sqrt() } else { 0.0 }
}

/// Runs `f` on every item, on `threads` threads, and keeps the results in order.
pub fn parallel<T: Sync, R: Send + Default + Clone>(items: &[T], threads: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out = std::sync::Mutex::new(vec![R::default(); items.len()]);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if i >= items.len() {
                        break;
                    }
                    let r = f(&items[i]);
                    out.lock().unwrap()[i] = r;
                }
            });
        }
    });
    out.into_inner().unwrap()
}

fn box_blur(values: &[f32], valid: &[bool], r: usize) -> Vec<f32> {
    let mut num: Vec<f64> = values.iter().zip(valid).map(|(v, ok)| if *ok { f64::from(*v) } else { 0.0 }).collect();
    let mut den: Vec<f64> = valid.iter().map(|ok| if *ok { 1.0 } else { 0.0 }).collect();
    for rows in [true, false] {
        for arr in [&mut num, &mut den] {
            let mut prefix = vec![0.0f64; RES + 1];
            let src = arr.clone();
            for line in 0..RES {
                let at = |i: usize| if rows { line * RES + i } else { i * RES + line };
                for i in 0..RES {
                    prefix[i + 1] = prefix[i] + src[at(i)];
                }
                for i in 0..RES {
                    arr[at(i)] = prefix[(i + r + 1).min(RES)] - prefix[i.saturating_sub(r)];
                }
            }
        }
    }
    num.iter().zip(&den).map(|(n, d)| if *d > 0.0 { (n / d) as f32 } else { 0.0 }).collect()
}

/// How many times the face's median fine-detail energy a pixel needs to count as hair.
pub const HAIR_FACTOR: f32 = 4.0;

impl Lightness {
    /// The pixels that hold hair: where the energy of the finest detail (the lightness minus its blur over 5 pixels, about 0.6 mm),
    /// averaged over 13 pixels (about 1.5 mm), is more than `HAIR_FACTOR` times its median over the painted pixels, and the
    /// pixels where such pixels fill more than 12% of the 13 by 13 square around them. Stubble, beards
    /// and brows have far more fine detail than skin or a painted crease.
    pub fn hair(&self) -> Vec<bool> {
        let blur = box_blur(&self.l, &self.valid, 2);
        let hp2: Vec<f32> = self.l.iter().zip(&blur).zip(&self.valid).map(|((a, b), ok)| if *ok { (a - b) * (a - b) } else { 0.0 }).collect();
        let energy = box_blur(&hp2, &self.valid, 6);
        let mut v: Vec<f32> = energy.iter().zip(&self.valid).filter(|(_, ok)| **ok).map(|(e, _)| *e).collect();
        v.sort_by(f32::total_cmp);
        let med = v[v.len() / 2];
        let raw: Vec<bool> = energy.iter().zip(&self.valid).map(|(e, ok)| *ok && *e > HAIR_FACTOR * med).collect();
        // Grown by 6 pixels (about 0.7 mm), so that the inside of a dense moustache, whose strokes merge, is covered too.
        let grown = box_blur(&raw.iter().map(|&h| if h { 1.0 } else { 0.0 }).collect::<Vec<f32>>(), &vec![true; RES * RES], 6);
        grown.iter().zip(&self.valid).map(|(g, ok)| *ok && *g > 0.12).collect()
    }
}

/// KOTOR's own head with its own texture, seen along the face's forward direction. A point of our head reads the KOTOR skin that a
/// viewer in front of the face sees at the same place, which is what the eye work found more reliable near the eyes than the bake.
pub struct KotorFront {
    vertices: Vec<[f64; 3]>,
    faces: Vec<[u32; 3]>,
    uvs: Vec<[[f64; 2]; 3]>,
    material: Vec<u32>,
    textures: Vec<(Vec<f32>, u32, u32)>,
    rgb: Vec<image::RgbImage>,
    across: [f64; 3],
    up: [f64; 3],
    forward: [f64; 3],
    lo: [f64; 2],
    cells: Vec<Vec<u32>>,
    dims: [usize; 2],
}

const FRONT_CELL_MM: f64 = 2.0;

impl KotorFront {
    /// `frame` holds the face's across, up and forward directions.
    pub fn open(name: &str, across: [f64; 3], up: [f64; 3], forward: [f64; 3]) -> Self {
        let b = Bundle::open(&heads_dir().join(name));
        let vertices: Vec<[f64; 3]> = b.points("kv").into_iter().map(|p| p.map(|x| 1000.0 * x)).collect();
        let faces = b.triangles("kf");
        let kuv = b.f64s("kuv");
        let uvs: Vec<[[f64; 2]; 3]> = (0..faces.len()).map(|f| [0, 1, 2].map(|c| [kuv[f * 6 + c * 2], kuv[f * 6 + c * 2 + 1]])).collect();
        let material = b.u32s("kmat");
        let hair = b.u8s("khair");
        let rgb: Vec<image::RgbImage> = b.manifest["textures"].as_array().unwrap().iter().map(|t| image::open(t.as_str().unwrap()).unwrap().to_rgb8()).collect();
        Self::build(vertices, faces, uvs, material, &hair, rgb, across, up, forward)
    }

    /// KOTOR's own head with its hair meshes kept, so that a fringe or a hood shows.
    pub fn open_with_hair(name: &str, across: [f64; 3], up: [f64; 3], forward: [f64; 3]) -> Self {
        let b = Bundle::open(&heads_dir().join(name));
        let vertices: Vec<[f64; 3]> = b.points("kv").into_iter().map(|p| p.map(|x| 1000.0 * x)).collect();
        let faces = b.triangles("kf");
        let kuv = b.f64s("kuv");
        let uvs: Vec<[[f64; 2]; 3]> = (0..faces.len()).map(|f| [0, 1, 2].map(|c| [kuv[f * 6 + c * 2], kuv[f * 6 + c * 2 + 1]])).collect();
        let material = b.u32s("kmat");
        let hair = vec![0u8; faces.len()];
        let rgb: Vec<image::RgbImage> = b.manifest["textures"].as_array().unwrap().iter().map(|t| image::open(t.as_str().unwrap()).unwrap().to_rgb8()).collect();
        Self::build(vertices, faces, uvs, material, &hair, rgb, across, up, forward)
    }

    /// Our own head with our texture, seen the same way. `positions` holds the head's mesh positions in metres.
    pub fn ours(c: &Common, positions: &[[f64; 3]], texture: image::RgbImage, across: [f64; 3], up: [f64; 3], forward: [f64; 3]) -> Self {
        let vertices: Vec<[f64; 3]> = c.vmap.iter().map(|&m| positions[m].map(|x| 1000.0 * x)).collect();
        let uvs: Vec<[[f64; 2]; 3]> = c.tris.iter().map(|t| t.map(|i| c.uv[i as usize])).collect();
        let n = c.tris.len();
        Self::build(vertices, c.tris.clone(), uvs, vec![0; n], &vec![0; n], vec![texture], across, up, forward)
    }

    #[allow(clippy::too_many_arguments)]
    fn build(vertices: Vec<[f64; 3]>, faces: Vec<[u32; 3]>, uvs: Vec<[[f64; 2]; 3]>, material: Vec<u32>, hair: &[u8], rgb: Vec<image::RgbImage>, across: [f64; 3], up: [f64; 3], forward: [f64; 3]) -> Self {
        let textures = rgb
            .iter()
            .map(|img| {
                let l = img.pixels().map(|p| srgb_to_lab(p.0.map(|v| f64::from(v) / 255.0))[0] as f32).collect();
                (l, img.width(), img.height())
            })
            .collect();
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let plane: Vec<[f64; 2]> = vertices.iter().map(|v| [dot(*v, across), dot(*v, up)]).collect();
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for p in &plane {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        let dims = [0, 1].map(|k| ((hi[k] - lo[k]) / FRONT_CELL_MM) as usize + 1);
        let mut cells = vec![Vec::new(); dims[0] * dims[1]];
        for (f, t) in faces.iter().enumerate() {
            if hair[f] != 0 {
                continue;
            }
            let c = t.map(|i| plane[i as usize]);
            let a = [0, 1].map(|k| ((c[0][k].min(c[1][k]).min(c[2][k]) - lo[k]) / FRONT_CELL_MM) as usize);
            let z = [0, 1].map(|k| (((c[0][k].max(c[1][k]).max(c[2][k]) - lo[k]) / FRONT_CELL_MM) as usize).min(dims[k] - 1));
            for x in a[0]..=z[0] {
                for y in a[1]..=z[1] {
                    cells[x * dims[1] + y].push(f as u32);
                }
            }
        }
        KotorFront { vertices, faces, uvs, material, textures, rgb, across, up, forward, lo, cells, dims }
    }

    /// KOTOR's lightness seen from the front at a point of our head, in millimetres. Of the KOTOR faces in front of or behind the point
    /// along the forward direction, the one nearest to it in depth is read, within 6 mm.
    pub fn lightness(&self, p: [f64; 3]) -> Option<f64> {
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let (f, uv) = self.hit([dot(p, self.across), dot(p, self.up)], Some(dot(p, self.forward)))?;
        let (tex, tw, th) = &self.textures[(self.material[f] as usize).min(self.textures.len() - 1)];
        let x = (uv[0].rem_euclid(1.0) * f64::from(*tw) - 0.5).clamp(0.0, f64::from(*tw) - 1.001);
        let y = ((1.0 - uv[1].rem_euclid(1.0)) * f64::from(*th) - 0.5).clamp(0.0, f64::from(*th) - 1.001);
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (fx, fy) = (x - x0 as f64, y - y0 as f64);
        let at = |xx: usize, yy: usize| f64::from(tex[yy * *tw as usize + xx]);
        Some((at(x0, y0) * (1.0 - fx) + at(x0 + 1, y0) * fx) * (1.0 - fy) + (at(x0, y0 + 1) * (1.0 - fx) + at(x0 + 1, y0 + 1) * fx) * fy)
    }

    /// The colour of the frontmost KOTOR face at a point of the face's plane (across, up), in millimetres.
    pub fn front_rgb(&self, q: [f64; 2]) -> Option<[u8; 3]> {
        let (f, uv) = self.hit(q, None)?;
        let img = &self.rgb[(self.material[f] as usize).min(self.rgb.len() - 1)];
        let x = ((uv[0].rem_euclid(1.0) * f64::from(img.width())) as u32).min(img.width() - 1);
        let y = (((1.0 - uv[1].rem_euclid(1.0)) * f64::from(img.height())) as u32).min(img.height() - 1);
        Some(img.get_pixel(x, y).0)
    }

    /// The 3D point of the frontmost face at a point of the face's plane, in millimetres.
    pub fn front_point(&self, q: [f64; 2]) -> Option<[f64; 3]> {
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let (f, _) = self.hit(q, None)?;
        let t = self.faces[f].map(|i| self.vertices[i as usize]);
        let a = t.map(|v| [dot(v, self.across), dot(v, self.up)]);
        let den = (a[1][1] - a[2][1]) * (a[0][0] - a[2][0]) + (a[2][0] - a[1][0]) * (a[0][1] - a[2][1]);
        let w0 = ((a[1][1] - a[2][1]) * (q[0] - a[2][0]) + (a[2][0] - a[1][0]) * (q[1] - a[2][1])) / den;
        let w1 = ((a[2][1] - a[0][1]) * (q[0] - a[2][0]) + (a[0][0] - a[2][0]) * (q[1] - a[2][1])) / den;
        let w = [w0, w1, 1.0 - w0 - w1];
        Some([0, 1, 2].map(|k| w[0] * t[0][k] + w[1] * t[1][k] + w[2] * t[2][k]))
    }

    /// The point's place in the plane (across, up) and its depth along forward, in millimetres.
    pub fn project(&self, p: [f64; 3]) -> ([f64; 2], f64) {
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        ([dot(p, self.across), dot(p, self.up)], dot(p, self.forward))
    }

    /// The KOTOR face seen at a point of the plane and its texture point: the one nearest `depth` within 6 mm, or the frontmost.
    fn hit(&self, q: [f64; 2], depth: Option<f64>) -> Option<(usize, [f64; 2])> {
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let cx = ((q[0] - self.lo[0]) / FRONT_CELL_MM).floor();
        let cy = ((q[1] - self.lo[1]) / FRONT_CELL_MM).floor();
        if cx < 0.0 || cy < 0.0 || cx as usize >= self.dims[0] || cy as usize >= self.dims[1] {
            return None;
        }
        let mut best: Option<(f64, usize, [f64; 3])> = None;
        for &f in &self.cells[cx as usize * self.dims[1] + cy as usize] {
            let t = self.faces[f as usize].map(|i| self.vertices[i as usize]);
            let a = t.map(|v| [dot(v, self.across), dot(v, self.up)]);
            let den = (a[1][1] - a[2][1]) * (a[0][0] - a[2][0]) + (a[2][0] - a[1][0]) * (a[0][1] - a[2][1]);
            if den.abs() < 1e-12 {
                continue;
            }
            let w0 = ((a[1][1] - a[2][1]) * (q[0] - a[2][0]) + (a[2][0] - a[1][0]) * (q[1] - a[2][1])) / den;
            let w1 = ((a[2][1] - a[0][1]) * (q[0] - a[2][0]) + (a[0][0] - a[2][0]) * (q[1] - a[2][1])) / den;
            let w = [w0, w1, 1.0 - w0 - w1];
            if w.iter().any(|&v| v < -1e-9) {
                continue;
            }
            let d = w[0] * dot(t[0], self.forward) + w[1] * dot(t[1], self.forward) + w[2] * dot(t[2], self.forward);
            let gap = match depth {
                Some(z) => (d - z).abs(),
                None => -d,
            };
            if (depth.is_none() || gap < 6.0) && best.is_none_or(|b| gap < b.0) {
                best = Some((gap, f as usize, w));
            }
        }
        let (_, f, w) = best?;
        Some((f, [0, 1].map(|k| w[0] * self.uvs[f][0][k] + w[1] * self.uvs[f][1][k] + w[2] * self.uvs[f][2][k])))
    }
}

impl KotorFront {
    /// A front view, 10 pixels per millimetre, `w_mm` by `h_mm`, centred on a point of the face's plane. Pixels where no face is seen
    /// are dark grey.
    pub fn tile(&self, centre: [f64; 2], w_mm: f64, h_mm: f64) -> image::RgbImage {
        self.tile_at(centre, w_mm, h_mm, 10.0)
    }

    /// The same at `per_mm` pixels per millimetre. Each pixel averages 3 by 3 points, so a small view is not jagged.
    pub fn tile_at(&self, centre: [f64; 2], w_mm: f64, h_mm: f64, per_mm: f64) -> image::RgbImage {
        let origin = [centre[0] - 0.5 * w_mm, centre[1] + 0.5 * h_mm];
        let step = 1.0 / per_mm;
        image::RgbImage::from_fn((w_mm * per_mm) as u32, (h_mm * per_mm) as u32, |x, y| {
            let mut sum = [0.0f64; 3];
            for j in 0..3 {
                for i in 0..3 {
                    let q = [origin[0] + (f64::from(x) + (f64::from(i) + 0.5) / 3.0) * step, origin[1] - (f64::from(y) + (f64::from(j) + 0.5) / 3.0) * step];
                    let c = self.front_rgb(q).unwrap_or([40, 40, 40]);
                    for k in 0..3 {
                        sum[k] += f64::from(c[k]) / 9.0;
                    }
                }
            }
            image::Rgb(sum.map(|v| v.round() as u8))
        })
    }
}

/// A fold side's sliders as JSON, rounded to a thousandth.
pub fn side_to_json(s: &head_paint::folds::FoldSide) -> serde_json::Value {
    let r = |x: f64| (x * 1000.0).round() / 1000.0;
    serde_json::json!({ "strength": r(s.strength), "highlight": r(s.highlight), "width_mm": r(s.width_mm), "edge_mm": r(s.edge_mm), "start_mm": r(s.start_mm), "length_mm": r(s.length_mm), "shift_mm": r(s.shift_mm) })
}
