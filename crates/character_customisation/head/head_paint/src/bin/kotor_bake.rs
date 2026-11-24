//! Carries each KOTOR head's paint into our head's texture layout, one pixel at a time. This is a tool for making targets and never ships.
//!
//! For each pixel of the layout the tool takes the pixel's point on our wrapped head (`PixelMap`), finds the closest point of the KOTOR face
//! mesh that faces the same way, and reads KOTOR's texture there at its full resolution. The rules for which places count as KOTOR skin, for
//! the lids and for growing the paint over the seams are those of `kotor_paint.paint` and `kotor_bake.py` in the Python tools.
//!
//! Usage: kotor_bake <bundle dir> <out dir> [--png] [--stats] [--no-normals] [--time-only] [<name> ...]
//! The bundle dir is written by `kotor_bake_prep.py`. The tool writes `<out dir>/<name>_ours.jpg` for each head, or for every head in the
//! bundle when no name is given. With `--png` it also writes a lossless `<name>_ours.png`.

use head_paint::closest::Bvh;
use head_paint::mesh::vertex_normals;
use head_paint::pixel_map::{PixelMap, grow_over_seams, nearest_true};
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageEncoder};
use serde_json::Value;
use std::path::Path;
use std::time::Instant;

const RES: usize = 2048;
/// How many pixels the paint grows out of the layout's patches, so that a seam shows no line.
const GROW: f64 = 8.0;
/// The farthest a KOTOR point may lie from our wrapped head to count as the skin under it, in metres.
const REACH: f64 = 0.0025;
/// The smallest dot product between our surface's normal and a KOTOR triangle's normal for the triangle to count as facing the same way.
const NORMAL_AGREEMENT: f64 = 0.2;
/// An unpainted point, as the viewer data draws it: linear 0.2, encoded with gamma 2.2.
const GREY: f64 = 122.9;
/// How much the inside of the mouth is darkened where the rig's page darkens it.
const MOUTH_DARKENING: f64 = 0.85;
/// The faces that turn into the slit between the lips have no KOTOR skin under them, because KOTOR's lips are closed there. They lie within
/// this distance above or below the lip line on the base head, in metres, and are darkened when their normal's y part is above `SLIT_FROM`,
/// fully when it is above `SLIT_TO`.
const SLIT_BAND: f64 = 0.004;
const SLIT_FROM: f64 = 0.6;
const SLIT_TO: f64 = 0.9;
/// Skin this near an eye's centre that the nearest point leaves unpainted is looked up from the front like a lid, in metres.
const EYE_NEIGHBOURHOOD: f64 = 0.022;
/// The half width of the front view around an eye, in metres, and its size in pixels.
const VIEW_HALF: f64 = 0.025;
const VIEW_SIZE: usize = 1000;
/// A front-lid point is looked up from the front only within this distance of the eye's opening, in metres.
const FRONT_LID_MARGIN: f64 = 0.0025;
/// The widening of the eye's opening that the front view avoids, in metres.
const OPENING_MARGIN: f64 = 0.0003;
/// A KOTOR triangle seen from the front counts only when its normal's z is above this share of its length.
const FRONT_FACING: f64 = 0.3;

type Vec3 = [f64; 3];

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(a: Vec3) -> Vec3 {
    let l = dot(a, a).sqrt();
    if l > 0.0 { a.map(|x| x / l) } else { [0.0; 3] }
}

/// The arrays of one bundle folder, written by `kotor_bake_prep.py`.
struct Bundle {
    manifest: Value,
    bytes: Vec<u8>,
}

impl Bundle {
    fn open(dir: &Path) -> Self {
        let text = std::fs::read_to_string(dir.join("bundle.json")).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let bytes = std::fs::read(dir.join("bundle.bin")).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        Bundle { manifest: serde_json::from_str(&text).expect("bundle.json"), bytes }
    }

    fn has(&self, name: &str) -> bool {
        self.manifest["arrays"].get(name).is_some()
    }

    fn raw(&self, name: &str, size: usize) -> &[u8] {
        let entry = &self.manifest["arrays"][name];
        let count: usize = entry["shape"].as_array().unwrap_or_else(|| panic!("no array {name}")).iter().map(|d| d.as_u64().unwrap_or(0) as usize).product();
        let offset = entry["offset"].as_u64().unwrap_or(0) as usize;
        &self.bytes[offset..offset + count * size]
    }

    fn f64s(&self, name: &str) -> Vec<f64> {
        self.raw(name, 8).as_chunks::<8>().0.iter().map(|c| f64::from_le_bytes(*c)).collect()
    }

    fn u32s(&self, name: &str) -> Vec<u32> {
        self.raw(name, 4).as_chunks::<4>().0.iter().map(|c| u32::from_le_bytes(*c)).collect()
    }

    fn u8s(&self, name: &str) -> Vec<u8> {
        self.raw(name, 1).to_vec()
    }

    fn vec3s(&self, name: &str) -> Vec<Vec3> {
        self.f64s(name).as_chunks::<3>().0.to_vec()
    }

    fn tris(&self, name: &str) -> Vec<[u32; 3]> {
        self.u32s(name).as_chunks::<3>().0.to_vec()
    }
}

struct Texture {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

impl Texture {
    /// The colour at texture coordinates (u, v), read between the four nearest texels, in encoded 0 to 255 values. The texture repeats.
    fn sample(&self, uv: [f64; 2]) -> [f64; 3] {
        let x = uv[0].rem_euclid(1.0) * self.width as f64 - 0.5;
        let y = (1.0 - uv[1].rem_euclid(1.0)) * self.height as f64 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let at = |xx: f64, yy: f64| {
            let (xx, yy) = ((xx.max(0.0) as usize).min(self.width - 1), (yy.max(0.0) as usize).min(self.height - 1));
            let i = (yy * self.width + xx) * 3;
            [f64::from(self.rgb[i]), f64::from(self.rgb[i + 1]), f64::from(self.rgb[i + 2])]
        };
        let (a, b, c, d) = (at(x0, y0), at(x0 + 1.0, y0), at(x0, y0 + 1.0), at(x0 + 1.0, y0 + 1.0));
        [0, 1, 2].map(|k| (a[k] * (1.0 - fx) + b[k] * fx) * (1.0 - fy) + (c[k] * (1.0 - fx) + d[k] * fx) * fy)
    }
}

/// What every head shares: the layout, which of our vertices are skin or lid, and those read at each pixel.
struct Common {
    triangles: Vec<[u32; 3]>,
    vmap: Vec<usize>,
    old_triangles: Vec<[u32; 3]>,
    map: PixelMap,
    on_skin: Vec<bool>,
    lid: Vec<bool>,
    front: Vec<bool>,
    dark: Vec<f64>,
    /// Each pixel's point on the base head, and the height of the base head's lip line and the half width of its mouth.
    base: Vec<Vec3>,
    mouth: [f64; 2],
}

fn to_new_points(v: &[Vec3], vmap: &[usize]) -> Vec<Vec3> {
    vmap.iter().map(|&o| v[o]).collect()
}

impl Common {
    fn load(dir: &Path) -> Self {
        let b = Bundle::open(dir);
        let uv: Vec<[f64; 2]> = b.f64s("uv").as_chunks::<2>().0.to_vec();
        let triangles = b.tris("tris");
        let vmap: Vec<usize> = b.u32s("vmap").into_iter().map(|i| i as usize).collect();
        let started = Instant::now();
        let map = PixelMap::new(&uv, &triangles, RES);
        println!("pixel map: {} of {} pixels lie on our head ({:.0}%), built in {:.2} s", map.pixels.len(), RES * RES, 100.0 * map.pixels.len() as f64 / (RES * RES) as f64, started.elapsed().as_secs_f64());
        let per_vertex = |name: &str| -> Vec<[f64; 1]> { b.u8s(name).iter().map(|&x| [f64::from(x)]).collect() };
        let read = |values: Vec<[f64; 1]>| -> Vec<f64> {
            let new: Vec<[f64; 1]> = vmap.iter().map(|&o| values[o]).collect();
            map.interpolate(&triangles, &new).into_iter().map(|v| v[0]).collect()
        };
        let on_skin = read(per_vertex("skin")).into_iter().map(|v| v > 0.999).collect();
        let lid = read(per_vertex("lid")).into_iter().map(|v| v > 0.0).collect();
        let front = read(per_vertex("front")).into_iter().map(|v| v > 0.999).collect();
        let dark = read(b.f64s("dark").into_iter().map(|d| [d]).collect());
        let base = map.interpolate(&triangles, &to_new_points(&b.vec3s("base"), &vmap));
        let mouth = b.f64s("mouth");
        Common { base, mouth: [mouth[0], mouth[1]], old_triangles: b.tris("old_tris"), triangles, vmap, map, on_skin, lid, front, dark }
    }
}

/// A view of triangles from the front (+z), orthographic, over a square around `centre`: the label of the frontmost triangle at each pixel.
struct Raster {
    label: Vec<u32>,
}

fn rasterise(size: usize, centre: [f64; 2], half: f64, triangles: impl Iterator<Item = ([Vec3; 3], u32)>) -> Raster {
    let px = 2.0 * half / size as f64;
    let mut label = vec![0u32; size * size];
    let mut depth = vec![f64::NEG_INFINITY; size * size];
    for (t, l) in triangles {
        let p = t.map(|v| [(v[0] - centre[0] + half) / px, (half - (v[1] - centre[1])) / px, v[2]]);
        let den = (p[1][1] - p[2][1]) * (p[0][0] - p[2][0]) + (p[2][0] - p[1][0]) * (p[0][1] - p[2][1]);
        if den.abs() < 1e-12 {
            continue;
        }
        let lo = |k: usize| p[0][k].min(p[1][k]).min(p[2][k]).floor().max(0.0) as usize;
        let hi = |k: usize| (p[0][k].max(p[1][k]).max(p[2][k]).ceil().max(0.0) as usize).min(size);
        for row in lo(1)..hi(1) {
            for col in lo(0)..hi(0) {
                let (x, y) = (col as f64 + 0.5, row as f64 + 0.5);
                let w0 = ((p[1][1] - p[2][1]) * (x - p[2][0]) + (p[2][0] - p[1][0]) * (y - p[2][1])) / den;
                let w1 = ((p[2][1] - p[0][1]) * (x - p[2][0]) + (p[0][0] - p[2][0]) * (y - p[2][1])) / den;
                let w2 = 1.0 - w0 - w1;
                if w0 < -1e-9 || w1 < -1e-9 || w2 < -1e-9 {
                    continue;
                }
                let z = w0 * p[0][2] + w1 * p[1][2] + w2 * p[2][2];
                let i = row * size + col;
                if z > depth[i] {
                    depth[i] = z;
                    label[i] = l;
                }
            }
        }
    }
    Raster { label }
}

/// `mask` grown by `radius` pixels in every direction, as a square.
fn dilate(mask: &[bool], size: usize, radius: usize) -> Vec<bool> {
    let n = size + 1;
    let mut sum = vec![0u32; n * n];
    for y in 0..size {
        for x in 0..size {
            sum[(y + 1) * n + x + 1] = u32::from(mask[y * size + x]) + sum[y * n + x + 1] + sum[(y + 1) * n + x] - sum[y * n + x];
        }
    }
    let mut out = vec![false; size * size];
    for y in 0..size {
        for x in 0..size {
            let (x0, x1, y0, y1) = (x.saturating_sub(radius), (x + radius + 1).min(size), y.saturating_sub(radius), (y + radius + 1).min(size));
            out[y * size + x] = sum[y1 * n + x1] + sum[y0 * n + x0] > sum[y0 * n + x1] + sum[y1 * n + x0];
        }
    }
    out
}

/// The opening of one eye in the KOTOR face mesh, seen from the front: where the eyeball shows through the face mesh. KOTOR's face mesh has no
/// hole there. It folds back into a socket behind a slit, and the eyeball fills the slit.
struct Opening {
    centre: Vec3,
    mask: Vec<bool>,
}

impl Opening {
    fn new(face: &[[Vec3; 3]], eyeballs: &[[Vec3; 3]], centre: Vec3) -> Self {
        let both = face.iter().map(|&t| (t, 1)).chain(eyeballs.iter().map(|&t| (t, 2)));
        let r = rasterise(VIEW_SIZE, [centre[0], centre[1]], VIEW_HALF, both);
        let middle = (VIEW_SIZE as f64 / 2.0) as isize;
        let start = (0..VIEW_SIZE * VIEW_SIZE)
            .filter(|&i| r.label[i] == 2)
            .min_by_key(|&i| ((i / VIEW_SIZE) as isize - middle).pow(2) + ((i % VIEW_SIZE) as isize - middle).pow(2));
        let mut mask = vec![false; VIEW_SIZE * VIEW_SIZE];
        if let Some(s) = start {
            let mut stack = vec![s];
            mask[s] = true;
            while let Some(i) = stack.pop() {
                let (y, x) = ((i / VIEW_SIZE) as isize, (i % VIEW_SIZE) as isize);
                for (dy, dx) in [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)] {
                    let (yy, xx) = (y + dy, x + dx);
                    if yy < 0 || xx < 0 || yy >= VIEW_SIZE as isize || xx >= VIEW_SIZE as isize {
                        continue;
                    }
                    let j = yy as usize * VIEW_SIZE + xx as usize;
                    if r.label[j] == 2 && !mask[j] {
                        mask[j] = true;
                        stack.push(j);
                    }
                }
            }
        }
        Opening { centre, mask }
    }

    fn pixel_of(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let px = 2.0 * VIEW_HALF / VIEW_SIZE as f64;
        let col = ((x - self.centre[0] + VIEW_HALF) / px).floor();
        let row = ((VIEW_HALF - y + self.centre[1]) / px).floor();
        (col >= 0.0 && row >= 0.0 && col < VIEW_SIZE as f64 && row < VIEW_SIZE as f64).then_some((row as usize, col as usize))
    }

    /// Whether each point (x, y) lies in the opening widened by `margin`.
    fn grown(&self, margin: f64) -> Vec<bool> {
        let px = 2.0 * VIEW_HALF / VIEW_SIZE as f64;
        dilate(&self.mask, VIEW_SIZE, (margin / px) as usize)
    }
}

/// The KOTOR triangle and the texture coordinates that the face mesh shows at each point (x, y) as seen from the front, for the lids. A
/// point that lies in the eye's opening, where the eyeball shows, reads the skin mirrored across the opening's edge as far out as it lies in,
/// so KOTOR's lash line and liner go on into our lid instead of one pixel of them being drawn out into a streak. A triangle seen nearly edge
/// on would smear its texture, so such places are avoided as well.
fn front_lookup(points: &[[f64; 2]], face: &[[Vec3; 3]], uv: &[[[f64; 2]; 3]], opening: &Opening) -> Vec<(u32, [f64; 2])> {
    let size = VIEW_SIZE;
    let px = 2.0 * VIEW_HALF / size as f64;
    let r = rasterise(size, [opening.centre[0], opening.centre[1]], VIEW_HALF, face.iter().enumerate().map(|(k, &t)| (t, k as u32 + 1)));
    let facing: Vec<bool> = face.iter().map(|t| {
        let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
        n[2] / dot(n, n).sqrt().max(1e-20) > FRONT_FACING
    }).collect();
    let avoid = opening.grown(OPENING_MARGIN);
    let bad: Vec<bool> = (0..size * size).map(|i| r.label[i] == 0 || !facing[r.label[i] as usize - 1] || avoid[i]).collect();
    let nearest = nearest_true(&bad.iter().map(|&b| !b).collect::<Vec<_>>(), size, size);
    points
        .iter()
        .map(|&xy| {
            let Some((row, col)) = opening.pixel_of(xy[0], xy[1]) else { return (u32::MAX, [0.0; 2]) };
            let i = nearest[row * size + col];
            if i == u32::MAX {
                return (u32::MAX, [0.0; 2]);
            }
            let (rr, cc) = ((i as usize / size) as isize, (i as usize % size) as isize);
            let (r2, c2) = (2 * rr - row as isize, 2 * cc - col as isize);
            let mirrored = r2 >= 0 && c2 >= 0 && (r2 as usize) < size && (c2 as usize) < size && !bad[r2 as usize * size + c2 as usize];
            let (row, col, xy) = if mirrored {
                let (r2, c2) = (r2 as usize, c2 as usize);
                (r2, c2, [opening.centre[0] + (c2 as f64 + 0.5) * px - VIEW_HALF, opening.centre[1] + VIEW_HALF - (r2 as f64 + 0.5) * px])
            } else {
                (rr as usize, cc as usize, xy)
            };
            let label = r.label[row * size + col];
            if label == 0 {
                return (u32::MAX, [0.0; 2]);
            }
            let t = label as usize - 1;
            let [a, b, d] = face[t].map(|v| [v[0], v[1]]);
            let den = (b[0] - a[0]) * (d[1] - a[1]) - (d[0] - a[0]) * (b[1] - a[1]);
            let wb = ((xy[0] - a[0]) * (d[1] - a[1]) - (d[0] - a[0]) * (xy[1] - a[1])) / den;
            let wd = ((b[0] - a[0]) * (xy[1] - a[1]) - (xy[0] - a[0]) * (b[1] - a[1])) / den;
            let mut w = [(1.0 - wb - wd).max(0.0), wb.max(0.0), wd.max(0.0)];
            let total: f64 = w.iter().sum();
            w = w.map(|x| x / total.max(1e-20));
            let c = uv[t];
            (t as u32, [0, 1].map(|k| w[0] * c[0][k] + w[1] * c[1][k] + w[2] * c[2][k]))
        })
        .collect()
}

#[derive(Default)]
struct Stats {
    covered: usize,
    on_skin: usize,
    painted: usize,
    hair: usize,
    too_far: usize,
    rejected_by_normal: usize,
    lid: usize,
}

struct Baked {
    rgb: Vec<u8>,
    stats: Stats,
}

struct Options {
    normals: bool,
    compare: bool,
}

fn bake(common: &Common, bundle: &Bundle, options: &Options) -> Baked {
    let mut stats = Stats { covered: common.map.pixels.len(), ..Stats::default() };
    let n_pixels = common.map.pixels.len();
    let to_new = |v: &[Vec3]| to_new_points(v, &common.vmap);

    let wrapped = bundle.vec3s("wrapped");
    let points = common.map.interpolate(&common.triangles, &to_new(&wrapped));
    let normals: Vec<Vec3> = common
        .map
        .interpolate(&common.triangles, &to_new(&vertex_normals(&wrapped, &common.old_triangles)))
        .into_iter()
        .map(unit)
        .collect();

    let kv = bundle.vec3s("kv");
    let kf = bundle.tris("kf");
    let kuv: Vec<[[f64; 2]; 3]> = bundle.f64s("kuv").as_chunks::<6>().0.iter().map(|c| [[c[0], c[1]], [c[2], c[3]], [c[4], c[5]]]).collect();
    let kmat = bundle.u32s("kmat");
    let khair = bundle.u8s("khair");
    let face: Vec<[Vec3; 3]> = kf.iter().map(|t| t.map(|i| kv[i as usize])).collect();
    let kn: Vec<Vec3> = face.iter().map(|t| unit(cross(sub(t[1], t[0]), sub(t[2], t[0])))).collect();
    let bvh = Bvh::new(kv.clone(), kf.clone());

    // KOTOR's triangles may wind either way, so the sign of the normal agreement is the one most nearby matches have.
    let sample: Vec<Vec3> = points.iter().step_by(13).copied().collect();
    let mut vote = 0.0;
    for (i, c) in bvh.closest_all(&sample).into_iter().enumerate() {
        if let Some(c) = c.filter(|c| c.distance < REACH) {
            vote += dot(normals[i * 13], kn[c.triangle as usize]);
        }
    }
    let sign = if vote < 0.0 { -1.0 } else { 1.0 };
    let found = if options.normals {
        bvh.closest_all_with(&points, &|i, t| sign * dot(normals[i], kn[t as usize]) > NORMAL_AGREEMENT)
    } else {
        bvh.closest_all(&points)
    };
    if options.compare {
        let plain = bvh.closest_all(&points);
        stats.rejected_by_normal = plain.iter().zip(&found).filter(|(p, f)| p.map(|c| c.triangle) != f.map(|c| c.triangle)).count();
    }

    let textures: Vec<Option<Texture>> = bundle.manifest["textures"]
        .as_array()
        .map_or(&[][..], |a| a.as_slice())
        .iter()
        .map(|p| {
            p.as_str().map(|path| {
                let img = image::open(path).unwrap_or_else(|e| panic!("{path}: {e}")).to_rgb8();
                Texture { width: img.width() as usize, height: img.height() as usize, rgb: img.into_raw() }
            })
        })
        .collect();
    let read = |triangle: usize, uv: [f64; 2]| -> Option<[f64; 3]> { textures[kmat[triangle] as usize].as_ref().map(|t| t.sample(uv)) };

    let mut colour = vec![[GREY; 3]; n_pixels];
    let mut painted = vec![false; n_pixels];
    for i in 0..n_pixels {
        stats.on_skin += usize::from(common.on_skin[i]);
        let Some(c) = found[i] else { continue };
        let t = c.triangle as usize;
        if !common.on_skin[i] {
            continue;
        }
        if c.distance >= REACH {
            stats.too_far += 1;
        } else if khair[t] != 0 {
            stats.hair += 1;
        } else {
            let w = c.barycentric;
            let uv = [0, 1].map(|k| w[0] * kuv[t][0][k] + w[1] * kuv[t][1][k] + w[2] * kuv[t][2][k]);
            if let Some(rgb) = read(t, uv) {
                colour[i] = rgb;
                painted[i] = true;
            }
        }
    }

    // The lids, and any skin near an eye that the nearest point left unpainted, are looked up from the front.
    if bundle.has("k_eyes") {
        let eyes = bundle.vec3s("k_eyes");
        let eyeballs: Vec<[Vec3; 3]> = bundle.vec3s("eye_tris").as_chunks::<3>().0.to_vec();
        let fitted = common.map.interpolate(&common.triangles, &to_new(&bundle.vec3s("fitted")));
        let lid: Vec<bool> = (0..n_pixels)
            .map(|i| common.lid[i] || (!painted[i] && eyes.iter().map(|&e| dot(sub(points[i], e), sub(points[i], e)).sqrt()).fold(f64::INFINITY, f64::min) < EYE_NEIGHBOURHOOD))
            .collect();
        for &eye in &eyes {
            let opening = Opening::new(&face, &eyeballs, eye);
            let near_front = opening.grown(FRONT_LID_MARGIN);
            let chosen: Vec<usize> = (0..n_pixels)
                .filter(|&i| {
                    if !lid[i] || dot(sub(fitted[i], eye), sub(fitted[i], eye)).sqrt() >= VIEW_HALF {
                        return false;
                    }
                    !(common.front[i] && painted[i]) || opening.pixel_of(fitted[i][0], fitted[i][1]).is_some_and(|(r, c)| near_front[r * VIEW_SIZE + c])
                })
                .collect();
            let queries: Vec<[f64; 2]> = chosen.iter().map(|&i| [fitted[i][0], fitted[i][1]]).collect();
            for (&i, (t, uv)) in chosen.iter().zip(front_lookup(&queries, &face, &kuv, &opening)) {
                if t != u32::MAX && let Some(rgb) = read(t as usize, uv) {
                    colour[i] = rgb;
                    painted[i] = true;
                    stats.lid += 1;
                }
            }
        }
    }
    stats.painted = painted.iter().filter(|&&p| p).count();

    // The inside of the mouth is darkened as the rig's page darkens it, then the paint grows over the layout's seams.
    let mut rgb = vec![0u8; RES * RES * 3];
    let mut filled = vec![false; RES * RES];
    for (i, p) in common.map.pixels.iter().enumerate() {
        let near_lip_line = (common.base[i][1] - common.mouth[0]).abs() < SLIT_BAND && common.base[i][0].abs() < common.mouth[1];
        let slit = if near_lip_line { ((normals[i][1].abs() - SLIT_FROM) / (SLIT_TO - SLIT_FROM)).clamp(0.0, 1.0) } else { 0.0 };
        let d = common.dark[i].max(slit);
        let k = if d > 0.0 { (1.0 - MOUTH_DARKENING * d).max(0.0).powf(1.0 / 2.2) } else { 1.0 };
        let at = p.y as usize * RES + p.x as usize;
        filled[at] = true;
        for c in 0..3 {
            rgb[at * 3 + c] = (colour[i][c] * k).round().clamp(0.0, 255.0) as u8;
        }
    }
    grow_over_seams(&mut rgb, &filled, RES, GROW);
    Baked { rgb, stats }
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut flag = |name: &str| args.iter().position(|a| a == name).map(|i| args.remove(i)).is_some();
    let (png, stats, no_normals, time_only) = (flag("--png"), flag("--stats"), flag("--no-normals"), flag("--time-only"));
    if args.len() < 2 {
        eprintln!("Usage: kotor_bake <bundle dir> <out dir> [--png] [--stats] [--no-normals] [--time-only] [<name> ...]");
        std::process::exit(2);
    }
    let (bundle_dir, out) = (Path::new(&args[0]), Path::new(&args[1]));
    std::fs::create_dir_all(out).expect("out dir");
    let started = Instant::now();
    let common = Common::load(&bundle_dir.join("common"));
    let mut names: Vec<String> = args[2..].to_vec();
    if names.is_empty() {
        let mut all: Vec<String> = std::fs::read_dir(bundle_dir.join("heads")).expect("heads").filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        all.sort();
        names = all;
    }
    let options = Options { normals: !no_normals, compare: stats };
    let mut total = 0.0;
    for name in &names {
        let t = Instant::now();
        let baked = bake(&common, &Bundle::open(&bundle_dir.join("heads").join(name)), &options);
        let seconds = t.elapsed().as_secs_f64();
        total += seconds;
        let s = &baked.stats;
        println!(
            "{name}: {seconds:.2} s; {} pixels, {} on skin, {} painted ({} through the lid lookup), hair {}, too far {}, other side {}",
            s.covered, s.on_skin, s.painted, s.lid, s.hair, s.too_far, s.rejected_by_normal
        );
        if time_only {
            continue;
        }
        let file = std::fs::File::create(out.join(format!("{name}_ours.jpg"))).expect("jpg");
        JpegEncoder::new_with_quality(std::io::BufWriter::new(file), 90).write_image(&baked.rgb, RES as u32, RES as u32, ExtendedColorType::Rgb8).expect("jpg");
        if png {
            image::save_buffer(out.join(format!("{name}_ours.png")), &baked.rgb, RES as u32, RES as u32, ExtendedColorType::Rgb8).expect("png");
        }
    }
    println!("{} heads baked in {total:.1} s ({:.2} s each), {:.1} s with the setup and the files", names.len(), total / names.len().max(1) as f64, started.elapsed().as_secs_f64());
}
