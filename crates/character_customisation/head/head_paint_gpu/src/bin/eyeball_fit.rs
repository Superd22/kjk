//! The measure of an eyeball texture and the fit of the eyeball sliders to KOTOR's 57 eyes (paint track, part C, the eye step).
//! Analysis only.
//!
//! `eyeball_fit measure <out json> <name>...|all` measures KOTOR's eye textures (read only as targets).
//! `eyeball_fit fit <out json> <name>...|all` measures them, then searches the sliders of each head so that our own painted texture gives
//! the same measures, and writes the fitted values and the gaps.
//!
//! The measure reads both eyes' squares of a texture as the viewer draws them (`head_paint::eyeball`), 512 pixels for 24.6 mm. KOTOR's
//! lids cover the top and the bottom of its eye textures, so only the pixels within 35 degrees of the horizontal through the pupil are
//! read. Each eye gives a profile of medians in rings 0.1 mm wide round the pupil, and the measures come from the profiles:
//! - the pupil's radius, where the darkest fifth of each ring first rises halfway from the pupil to the iris's middle;
//! - the iris's radius, where the lightness last lies below halfway between the iris's outer ring and the eye white;
//! - the pupil's lightness, the iris's main colour (35 to 80% of the way from the pupil's edge to the iris's edge), the inner colour
//!   (10 to 30%), and the eye white's colour (1.5 to 3.5 mm past the iris's edge), each as L*, a*, b*;
//! - the outer ring's darkness, one minus the luminance of the darkest ring in the iris's outer 15% over the outer colour's luminance;
//! - the fibre contrast, the median over the rings of the iris of the standard deviation of L* round each ring;
//! - the catch-light, the share of the iris's pixels at every angle lighter than 80 L* and 25 L* lighter than the outer colour.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;
#[path = "eyes_shared/common.rs"]
mod common;

use head_paint::controls::{controls, recipe_from};
use head_paint::eyeball::{SLIDERS, k};
use head_paint::pigment::{decode, linear_to_lab};
use head_paint_gpu::eyeball::EyeballPainter;
use head_paint_gpu::gpu::Gpu;
use serde_json::{Value, json};
use std::time::Instant;

const KOTOR_EYES: &str = "/private/tmp/claude-501/-Users-david-perso-kjk/10035d93-2b63-4fbe-8d90-9e81fd5ef380/scratchpad/viewer-data/tex/eyes";
const RINGS: usize = 90;
/// The size of our texture while fitting, per eye. The measure is the same at this size within 0.1 mm.
const FIT_PIXELS: u32 = 256;

#[derive(Clone, Copy, Debug, Default)]
struct Measure {
    pupil_mm: f64,
    iris_mm: f64,
    pupil_l: f64,
    outer: [f64; 3],
    inner: [f64; 3],
    white: [f64; 3],
    ring: f64,
    fibres: f64,
    catch: f64,
}

impl Measure {
    fn json(&self) -> Value {
        let r = |v: f64| (v * 1000.0).round() / 1000.0;
        json!({ "pupil_mm": r(self.pupil_mm), "iris_mm": r(self.iris_mm), "pupil_l": r(self.pupil_l), "outer": self.outer.map(r), "inner": self.inner.map(r), "white": self.white.map(r), "ring": r(self.ring), "fibres": r(self.fibres), "catch": r(self.catch) })
    }

    fn mean(a: &Measure, b: &Measure) -> Measure {
        let m = |x: f64, y: f64| 0.5 * (x + y);
        let m3 = |x: [f64; 3], y: [f64; 3]| [0, 1, 2].map(|i| 0.5 * (x[i] + y[i]));
        Measure { pupil_mm: m(a.pupil_mm, b.pupil_mm), iris_mm: m(a.iris_mm, b.iris_mm), pupil_l: m(a.pupil_l, b.pupil_l), outer: m3(a.outer, b.outer), inner: m3(a.inner, b.inner), white: m3(a.white, b.white), ring: m(a.ring, b.ring), fibres: m(a.fibres, b.fibres), catch: m(a.catch, b.catch) }
    }

    /// The gap between two measures, as a sum of squares of each part over its tolerance.
    fn gap(&self, o: &Measure) -> f64 {
        let lab = |a: [f64; 3], b: [f64; 3], tol: f64| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)) / (tol * tol);
        let sq = |a: f64, b: f64, tol: f64| ((a - b) / tol).powi(2);
        sq(self.pupil_mm, o.pupil_mm, 0.15)
            + sq(self.iris_mm, o.iris_mm, 0.15)
            + sq(self.pupil_l, o.pupil_l, 4.0)
            + lab(self.outer, o.outer, 3.0)
            + lab(self.inner, o.inner, 4.0)
            + lab(self.white, o.white, 4.0)
            + sq(self.ring, o.ring, 0.06)
            + sq(self.fibres, o.fibres, 2.5)
            + sq(self.catch, o.catch, 0.004)
    }
}

fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// The L*, a*, b* of every pixel of one eye's square, from the texture's sRGB bytes.
fn lab_square(img: &image::RgbImage, half: u32) -> Vec<[f64; 3]> {
    let n = img.height();
    let mut out = Vec::with_capacity((n * n) as usize);
    for y in 0..n {
        for x in 0..n {
            let p = img.get_pixel(half * n + x, y).0;
            out.push(linear_to_lab(p.map(|c| decode(f64::from(c) / 255.0))));
        }
    }
    out
}

/// The measure of one eye's square. `known` is the iris's centre in pixels when it is known, as on our own texture.
fn measure_square(lab: &[[f64; 3]], known: Option<(f64, f64)>) -> Measure {
    let n = (lab.len() as f64).sqrt().round() as usize;
    let mm = n as f64 / 24.6;
    let c0 = 0.5 * n as f64 - 0.5;
    // The iris's centre: the mean of the dark disc within 7 mm of the square's centre, the pixels darker than halfway between the
    // darkest 3% and the lightest 10%. On a light iris that is the pupil, on a dark one the whole iris, and both share the centre.
    let mut near: Vec<(f64, usize)> = (0..lab.len()).filter(|&i| ((i % n) as f64 - c0).hypot((i / n) as f64 - c0) < 7.0 * mm).map(|i| (lab[i][0], i)).collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let dark_l = near[..near.len() * 3 / 100].iter().map(|d| d.0).sum::<f64>() / (near.len() * 3 / 100) as f64;
    let light_l = near[near.len() * 9 / 10].0;
    let disc: Vec<usize> = near.iter().filter(|d| d.0 < 0.5 * (dark_l + light_l)).map(|d| d.1).collect();
    let (cx, cy) = match known {
        Some(c) => c,
        None if dark_l < 45.0 && !disc.is_empty() => (disc.iter().map(|&i| (i % n) as f64).sum::<f64>() / disc.len() as f64, disc.iter().map(|&i| (i / n) as f64).sum::<f64>() / disc.len() as f64),
        None => (c0, c0),
    };
    // Rings 0.1 mm wide, within 35 degrees of the horizontal.
    let mut rings: Vec<Vec<[f64; 3]>> = vec![Vec::new(); RINGS];
    let tan = 35f64.to_radians().tan();
    for (i, c) in lab.iter().enumerate() {
        let (dx, dy) = ((i % n) as f64 - cx, (i / n) as f64 - cy);
        if dy.abs() > tan * dx.abs() {
            continue;
        }
        let r = dx.hypot(dy) / mm;
        let k = (r / 0.1) as usize;
        if k < RINGS {
            rings[k].push(*c);
        }
    }
    let ring_median = |k: usize, ch: usize| median(&mut rings[k].iter().map(|c| c[ch]).collect::<Vec<_>>());
    let l: Vec<f64> = (0..RINGS).map(|k| ring_median(k, 0)).collect();
    let mid_mm = |a: f64, b: f64| ((a / 0.1) as usize..((b / 0.1) as usize).min(RINGS)).collect::<Vec<_>>();
    let zone = |ks: &[usize]| -> [f64; 3] {
        let mut ch = [Vec::new(), Vec::new(), Vec::new()];
        for &k in ks {
            for c in &rings[k] {
                for i in 0..3 {
                    ch[i].push(c[i]);
                }
            }
        }
        [0, 1, 2].map(|i| median(&mut ch[i]))
    };
    // The pupil's lightness: the darkest ring within 1.5 mm, since KOTOR often paints its catch-light in the middle of the pupil.
    let low: Vec<f64> = (0..RINGS)
        .map(|k| {
            let mut v: Vec<f64> = rings[k].iter().map(|c| c[0]).collect();
            v.sort_by(f64::total_cmp);
            if v.is_empty() { f64::NAN } else { v[v.len() / 5] }
        })
        .collect();
    let darkest = (0..15).min_by(|&a, &b| low[a].total_cmp(&low[b])).unwrap_or(0);
    let pupil_l = l[darkest].min(low[darkest] + 10.0);
    if std::env::var("PROFILE").is_ok() {
        eprintln!("centre ({cx:.1}, {cy:.1}) of {n}, profile {:?}", l.iter().map(|v| v.round()).collect::<Vec<_>>());
    }
    let white_l = median(&mut (75..90).map(|k| l[k]).collect::<Vec<_>>());
    // The iris's edge: the last ring, out to 7.5 mm, darker than halfway between the darkest ring past 3 mm and the white.
    let ring_min = (30..75).map(|k| l[k]).fold(f64::INFINITY, f64::min);
    let half = 0.5 * (ring_min + white_l);
    let edge_k = (20..75).rev().find(|&k| l[k] < half).unwrap_or(57);
    let iris_r = (edge_k as f64 + 0.5) * 0.1;
    let iris_mid = median(&mut mid_mm(0.4 * iris_r, 0.75 * iris_r).iter().map(|&k| l[k]).collect::<Vec<_>>());
    // The pupil's edge, on the profile of each ring's darkest fifth, which a catch-light covering part of a ring does not move.
    let low_mid = median(&mut mid_mm(0.4 * iris_r, 0.75 * iris_r).iter().map(|&k| low[k]).collect::<Vec<_>>());
    let pupil_half = low[darkest] + 0.5 * (low_mid - low[darkest]);
    let pupil_k = (darkest..edge_k).find(|&k| low[k] > pupil_half).unwrap_or(darkest);
    let pupil_r = (pupil_k as f64 + 0.5) * 0.1;
    let at = |s: f64| pupil_r + s * (iris_r - pupil_r);
    let outer = zone(&mid_mm(at(0.35), at(0.8)));
    let inner = zone(&mid_mm(at(0.1), at(0.3)));
    let white = zone(&mid_mm(iris_r + 1.5, iris_r + 3.5));
    let y = |l: f64| if l > 8.0 { ((l + 16.0) / 116.0).powi(3) } else { l / 903.3 };
    let ring_dark = mid_mm(at(0.85), iris_r).iter().map(|&k| l[k]).fold(f64::INFINITY, f64::min);
    let ring = (1.0 - y(ring_dark) / y(outer[0]).max(1e-4)).clamp(0.0, 1.0);
    let mut sds: Vec<f64> = mid_mm(at(0.2), at(0.8))
        .iter()
        .map(|&k| {
            let v: Vec<f64> = rings[k].iter().map(|c| c[0]).collect();
            let m = v.iter().sum::<f64>() / v.len().max(1) as f64;
            (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len().max(1) as f64).sqrt()
        })
        .collect();
    let fibres = median(&mut sds);
    // The catch-light is read at every angle, since it often sits above the pupil.
    let iris_px: Vec<f64> = (0..lab.len()).filter(|&i| ((i % n) as f64 - cx).hypot((i / n) as f64 - cy) < iris_r * mm).map(|i| lab[i][0]).collect();
    let catch = iris_px.iter().filter(|&&x| x > 80.0 && x > outer[0] + 25.0).count() as f64 / iris_px.len().max(1) as f64;
    Measure { pupil_mm: 2.0 * pupil_r, iris_mm: 2.0 * iris_r, pupil_l, outer, inner, white, ring, fibres, catch }
}

/// The measure of our texture while fitting: our two eyes differ only in their fine pattern, so one is enough.
fn measure_one(img: &image::RgbImage) -> Measure {
    let c = 0.5 * f64::from(img.height()) - 0.5;
    measure_square(&lab_square(img, 0), Some((c, c)))
}

fn measure(img: &image::RgbImage) -> Measure {
    Measure::mean(&measure_square(&lab_square(img, 0), None), &measure_square(&lab_square(img, 1), None))
}

fn kotor(name: &str) -> Measure {
    measure(&image::open(format!("{KOTOR_EYES}/{name}_eyes.png")).expect("KOTOR eye texture").to_rgb8())
}

struct Ours {
    gpu: Gpu,
    painter: EyeballPainter,
    base: Vec<f64>,
    first: usize,
}

impl Ours {
    fn new() -> Self {
        let gpu = Gpu::blocking().expect("GPU");
        let painter = EyeballPainter::with_size(&gpu, FIT_PIXELS);
        let list = controls();
        let first = list.iter().position(|c| c.id == format!("eye_{}", SLIDERS[0].id)).expect("eyeball controls");
        Ours { gpu, painter, base: list.iter().map(|c| c.default).collect(), first }
    }

    fn texture(&self, ball: &[f64]) -> image::RgbImage {
        let mut v = self.base.clone();
        v[self.first..self.first + SLIDERS.len()].copy_from_slice(ball);
        self.painter.paint(&self.gpu, &recipe_from(&v).eyes.ball, [[0.0; 2]; 2], [head_paint::eyes::EYEBALL_RADIUS_MM; 2]);
        let rgba = pollster::block_on(self.painter.read_rgba(&self.gpu));
        let n = FIT_PIXELS;
        image::RgbImage::from_fn(2 * n, n, |x, y| {
            let at = 4 * (y * 2 * n + x) as usize;
            image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
        })
    }
}

/// The sliders the fit searches, in the order it searches them. The others keep their defaults, except the second colour's amount and
/// reach, which the fit sets to 0.8 and 0.35 so that the inner colour is the measured one.
const SEARCHED: [usize; 15] = [k::IRIS_MM, k::PUPIL, k::IRIS_LIGHTNESS, k::IRIS_CHROMA, k::IRIS_HUE, k::INNER_LIGHTNESS, k::INNER_CHROMA, k::INNER_HUE, k::RING, k::FIBRES, k::WHITE_LIGHTNESS, k::WHITE_CHROMA, k::WHITE_HUE, k::CATCHLIGHT, k::CLOUDING];

fn fit(ours: &Ours, target: &Measure) -> (Vec<f64>, Measure, f64, usize) {
    let mut v: Vec<f64> = SLIDERS.iter().map(|s| s.default).collect();
    v[k::INNER] = 0.8;
    v[k::INNER_WIDTH] = 0.35;
    // Start from the measured colours.
    let lch = |c: [f64; 3]| [c[0], c[2].atan2(c[1]).to_degrees().rem_euclid(360.0), c[1].hypot(c[2])];
    let (o, i, w) = (lch(target.outer), lch(target.inner), lch(target.white));
    v[k::IRIS_LIGHTNESS] = o[0].clamp(3.0, 95.0);
    v[k::IRIS_HUE] = o[1];
    v[k::IRIS_CHROMA] = o[2].min(60.0);
    v[k::INNER_LIGHTNESS] = i[0].clamp(3.0, 95.0);
    v[k::INNER_HUE] = i[1];
    v[k::INNER_CHROMA] = i[2].min(60.0);
    v[k::WHITE_LIGHTNESS] = w[0].clamp(40.0, 98.0);
    v[k::WHITE_HUE] = w[1];
    v[k::WHITE_CHROMA] = w[2].min(30.0);
    v[k::IRIS_MM] = target.iris_mm.clamp(8.0, 15.0);
    v[k::PUPIL] = (target.pupil_mm / target.iris_mm).clamp(0.15, 0.7);
    let mut evals = 0;
    let mut cost = |v: &[f64]| {
        evals += 1;
        measure_one(&ours.texture(v)).gap(target)
    };
    let mut best = cost(&v);
    for round in 0..3 {
        for &s in &SEARCHED {
            let sl = &SLIDERS[s];
            let span = (sl.max - sl.min) / [6.0, 24.0, 80.0][round];
            let centre = v[s];
            for j in -4..=4 {
                if j == 0 {
                    continue;
                }
                let mut t = v.clone();
                t[s] = if s == k::IRIS_HUE || s == k::INNER_HUE || s == k::WHITE_HUE { (centre + f64::from(j) * span).rem_euclid(360.0) } else { (centre + f64::from(j) * span).clamp(sl.min, sl.max) };
                let c = cost(&t);
                if c < best {
                    best = c;
                    v = t;
                }
            }
        }
    }
    let m = measure(&ours.texture(&v));
    (v, m, best, evals)
}

fn names(args: &[String]) -> Vec<String> {
    if args.first().is_some_and(|a| a == "all") { common::head_names() } else { args.to_vec() }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: eyeball_fit measure|fit <out json> <name>...|all");
        std::process::exit(2);
    }
    let list = names(&args[2..]);
    let mut out = serde_json::Map::new();
    match args[0].as_str() {
        "measure" => {
            for n in &list {
                out.insert(n.clone(), kotor(n).json());
            }
        }
        "fit" => {
            let ours = Ours::new();
            for n in &list {
                let t0 = Instant::now();
                let target = kotor(n);
                let (v, m, gap, evals) = fit(&ours, &target);
                let values: serde_json::Map<String, Value> = SLIDERS.iter().zip(&v).map(|(s, x)| (format!("eye_{}", s.id), json!((x * 1000.0).round() / 1000.0))).collect();
                println!("{n}: gap {gap:.2} after {evals} paints in {:.1} s", t0.elapsed().as_secs_f64());
                out.insert(n.clone(), json!({ "values": values, "kotor": target.json(), "ours": m.json(), "gap": gap }));
            }
        }
        "check" => {
            // The measure of our own texture at 512 and at 256 pixels per eye, for the defaults and the six hand-set heads' looks.
            let gpu = Gpu::blocking().expect("GPU");
            let big = EyeballPainter::new(&gpu);
            let ours = Ours::new();
            let hand: Value = serde_json::from_str(&std::fs::read_to_string("/Users/david/.claude/tools/likeness-fit-data/paint/eyeball/hand.json").expect("hand")).expect("json");
            for n in &list {
                let mut v: Vec<f64> = SLIDERS.iter().map(|s| s.default).collect();
                for (i, s) in SLIDERS.iter().enumerate() {
                    if let Some(x) = hand[n.as_str()][format!("eye_{}", s.id)].as_f64() {
                        v[i] = x;
                    }
                }
                let mut all = ours.base.clone();
                all[ours.first..ours.first + SLIDERS.len()].copy_from_slice(&v);
                big.paint(&gpu, &recipe_from(&all).eyes.ball, [[0.0; 2]; 2], [head_paint::eyes::EYEBALL_RADIUS_MM; 2]);
                let rgba = pollster::block_on(big.read_rgba(&gpu));
                let img = image::RgbImage::from_fn(1024, 512, |x, y| {
                    let at = 4 * (y * 1024 + x) as usize;
                    image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
                });
                let t0 = Instant::now();
                let small = measure_one(&ours.texture(&v));
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                out.insert(n.clone(), json!({ "at_512": measure_one(&img).json(), "at_256": small.json(), "paint_and_measure_ms_at_256": ms }));
            }
        }
        other => panic!("unknown mode {other}"),
    }
    std::fs::write(&args[1], serde_json::to_string_pretty(&Value::Object(out)).expect("json")).expect("write");
}
