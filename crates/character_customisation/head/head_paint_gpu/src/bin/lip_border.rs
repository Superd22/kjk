//! Finds where KOTOR paints the lip border on our layout, head by head, and from that the lip border the lip layer is placed by (paint
//! track, part C3). Analysis only.
//!
//! `lip_border <out dir>`
//!
//! On each head's KOTOR bake, column by column across the mouth, the upper border is the row above the opening where the redness (a*)
//! falls fastest, and the lower border the same below it. The two halves of the face are read separately, and the x > 0 half is mirrored
//! about the layout's middle column. The layer's own border is the median over the 57 heads, smoothed along the mouth. Each curve is
//! sampled from the mouth corner to the middle line, and each sample becomes an anchor of three mesh vertices on the base head. The x > 0
//! half is the mirror image in 3D, as `fold_anchors` does.
//!
//! It writes `borders.json` (each head's rows per column), prints the constants of `lips.rs`, and writes a sheet of every head's bake
//! with its detected rows and the median curve, to check the detection by eye.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::{Common, RES, bake_path, head_names, parallel};
use head_paint::colour::srgb_to_lab;
use head_paint::folds::{Anchor, SurfaceIndex};
use head_paint::layout_points::{LayoutLocator, uv_of};
use std::path::Path;

/// The layout's middle column, where the hole of the opening is centred.
const MID: usize = 926;
/// The mouth corner, a layout pixel on the x < 0 side: 14 pixels outside the folds' `MOUTH_CORNER` pick, where KOTOR's lips end on
/// most heads.
const CORNER: (f64, f64) = (676.0, 1464.0);
/// The first column read, on the x < 0 half.
const FIRST: usize = 700;
/// Samples per half, from the corner to the middle line, the corner and the middle included.
const SAMPLES: usize = 22;

struct Hole {
    /// For each column: the first and last row of the opening's hole, if the column has one.
    rows: Vec<Option<(usize, usize)>>,
}

impl Hole {
    fn new(covered: &[bool]) -> Self {
        let rows = (0..RES)
            .map(|x| {
                let r: Vec<usize> = (1420..1500).filter(|&y| !covered[y * RES + x]).collect();
                Some((*r.first()?, *r.last()?))
            })
            .collect();
        Hole { rows }
    }

    /// The opening's row at a column of the x < 0 half: the middle of the hole, and past the hole's end a straight line to the corner.
    fn opening(&self, x: usize) -> f64 {
        if let Some((t, b)) = self.rows[x] {
            return 0.5 * (t + b) as f64;
        }
        let end = (x..MID).find(|&c| self.rows[c].is_some()).unwrap_or(MID);
        let (t, b) = self.rows[end].unwrap_or((1463, 1463));
        let e = 0.5 * (t + b) as f64;
        let f = (x as f64 - CORNER.0) / (end as f64 - CORNER.0);
        CORNER.1 + f.clamp(0.0, 1.0) * (e - CORNER.1)
    }
}

/// The a* of a bake around the mouth, blurred over 5 by 5 painted pixels. Other pixels are NaN.
fn redness(img: &image::RgbImage, covered: &[bool]) -> Vec<f32> {
    let (x0, x1, y0, y1) = (600, 1260, 1300, 1640);
    let mut raw = vec![f32::NAN; RES * RES];
    for y in y0..y1 {
        for x in x0..x1 {
            if covered[y * RES + x] {
                raw[y * RES + x] = srgb_to_lab(img.get_pixel(x as u32, y as u32).0.map(|v| f64::from(v) / 255.0))[1] as f32;
            }
        }
    }
    let mut out = vec![f32::NAN; RES * RES];
    for y in y0 + 2..y1 - 2 {
        for x in x0 + 2..x1 - 2 {
            let (mut s, mut n) = (0.0, 0.0);
            for dy in 0..5 {
                for dx in 0..5 {
                    let v = raw[(y + dy - 2) * RES + x + dx - 2];
                    if v.is_finite() {
                        s += v;
                        n += 1.0;
                    }
                }
            }
            if n > 0.0 && covered[y * RES + x] {
                out[y * RES + x] = s / n;
            }
        }
    }
    out
}

/// The row where a* falls fastest going from `start` in direction `dir`, between `near` and `far` pixels away, or NaN.
fn border(a: &[f32], x: usize, start: f64, dir: i64, near: i64, far: i64) -> f64 {
    let get = |y: i64| a[y as usize * RES + x];
    let drop = |y: i64| get(y - 3 * dir) - get(y + 3 * dir);
    let s = start.round() as i64;
    let mut best = (1.0f32, f64::NAN);
    for k in near..far {
        let y = s + dir * k;
        let d = drop(y);
        if d.is_finite() && d > best.0 {
            let (d0, d1) = (drop(y - 1), drop(y + 1));
            let den = d0 - 2.0 * d + d1;
            let off = if d0.is_finite() && d1.is_finite() && den < 0.0 { f64::from(0.5 * (d0 - d1) / den) } else { 0.0 };
            best = (d, y as f64 + off);
        }
    }
    best.1
}

fn median(mut v: Vec<f64>) -> f64 {
    v.retain(|x| x.is_finite());
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = Path::new(&args[0]);
    std::fs::create_dir_all(out).unwrap();
    let c = Common::load();
    let mut covered = vec![false; RES * RES];
    for p in &c.map.pixels {
        covered[p.y as usize * RES + p.x as usize] = true;
    }
    let hole = Hole::new(&covered);
    let names = head_names();
    let cols: Vec<usize> = (FIRST..=MID).collect();
    let per_head: Vec<Vec<[f64; 4]>> = parallel(&names, 8, |n| {
        let img = image::open(bake_path(n)).unwrap().to_rgb8();
        let a = redness(&img, &covered);
        cols.iter()
            .map(|&x| {
                let o = hole.opening(x);
                let (t, b) = hole.rows[x].map_or((o, o), |(t, b)| (t as f64 - 1.0, b as f64 + 1.0));
                let m = 2 * MID - x;
                let (tm, bm) = hole.rows[m].map_or((o, o), |(t, b)| (t as f64 - 1.0, b as f64 + 1.0));
                [border(&a, x, t, -1, 3, 70), border(&a, x, b, 1, 3, 110), border(&a, m, tm, -1, 3, 70), border(&a, m, bm, 1, 3, 110)]
            })
            .collect()
    });
    let raw: Vec<[f64; 2]> = (0..cols.len())
        .map(|i| [median(per_head.iter().flat_map(|h| [h[i][0], h[i][2]]).collect()), median(per_head.iter().flat_map(|h| [h[i][1], h[i][3]]).collect())])
        .collect();
    let smooth = |k: usize| -> Vec<f64> {
        let n = raw.len();
        let med: Vec<f64> = (0..n).map(|i| median((i.saturating_sub(6)..(i + 7).min(n)).map(|j| raw[j][k]).collect())).collect();
        (0..n)
            .map(|i| {
                let r = i.saturating_sub(4)..(i + 5).min(n);
                let l = r.len() as f64;
                r.map(|j| med[j]).sum::<f64>() / l
            })
            .collect()
    };
    let (up, lo) = (smooth(0), smooth(1));
    for (i, &x) in cols.iter().enumerate().step_by(10) {
        eprintln!("x {x}: opening {:.1}, upper {:.1}, lower {:.1}", hole.opening(x), up[i], lo[i]);
    }
    let json: serde_json::Map<String, serde_json::Value> = names
        .iter()
        .zip(&per_head)
        .map(|(n, h)| (n.clone(), serde_json::json!(h.iter().map(|r| r.map(|v| if v.is_finite() { (v * 10.0).round() / 10.0 } else { -1.0 })).collect::<Vec<_>>())))
        .collect();
    let doc = serde_json::json!({
        "note": "Rows of KOTOR's painted lip border on each head's bake, per column of the x < 0 half from `first` to 926: upper and lower on that side, then upper and lower on the x > 0 side mirrored about column 926. -1 where none was found.",
        "first": FIRST, "median_upper": up, "median_lower": lo,
        "opening": cols.iter().map(|&x| hole.opening(x)).collect::<Vec<_>>(),
        "heads": json,
    });
    std::fs::write(out.join("borders.json"), serde_json::to_string(&doc).unwrap()).unwrap();

    let xs: Vec<f64> = (0..SAMPLES).map(|i| CORNER.0 + (MID as f64 - CORNER.0) * (1.0 - (1.0 - i as f64 / (SAMPLES - 1) as f64).powf(1.5))).collect();
    let at = |v: &[f64], x: f64| -> f64 {
        let f = (x - FIRST as f64).max(0.0);
        let i = (f.floor() as usize).min(v.len() - 2);
        let t = f - i as f64;
        v[i] * (1.0 - t) + v[i + 1] * t
    };
    // Near the corner the borders close onto it.
    let row = |v: &[f64], x: f64| -> f64 {
        // From the corner the border opens as a wedge that curves out, so that the corner is a point, as KOTOR paints it.
        let end = 775.0;
        if x >= end {
            return at(v, x);
        }
        let f = ((x - CORNER.0) / (end - CORNER.0)).clamp(0.0, 1.0);
        CORNER.1 + (at(v, end) - CORNER.1) * (0.35 * f + 0.65 * f * f * (3.0 - 2.0 * f))
    };
    let per_layout: Vec<[f64; 3]> = c.vmap.iter().map(|&m| c.base[m]).collect();
    let locator = LayoutLocator::new(&c.uv, &c.tris);
    let index = SurfaceIndex::new(&c.tris, &per_layout);
    let anchor_of = |p_mm: [f64; 3]| -> Anchor {
        let hit = index.closest(p_mm).expect("on the head");
        let t = c.tris[hit.triangle];
        Anchor { vertices: t.map(|i| c.vmap[i as usize] as u32), weights: hit.barycentric }
    };
    let spot = |x: f64, y: f64, toward: f64| -> [f64; 3] {
        let mut py = y;
        for _ in 0..12 {
            if let Some(s) = locator.spot(uv_of((x + 0.5, py + 0.5)), &per_layout) {
                return s.position_mm;
            }
            py += (toward - py).signum() * 0.5;
        }
        panic!("no spot at {x}, {y}");
    };
    let fmt = |a: &Anchor| format!("Anchor {{ vertices: {:?}, weights: [{:.5}, {:.5}, {:.5}] }}", a.vertices, a.weights[0], a.weights[1], a.weights[2]);
    let open_edge = |x: f64, upper: bool| -> f64 {
        let xi = x.round() as usize;
        match hole.rows[xi] {
            Some((t, b)) => if upper { t as f64 - 0.5 } else { b as f64 + 0.5 },
            None => hole.opening(xi),
        }
    };
    let curves: Vec<(&str, Vec<(f64, f64, f64)>)> = vec![
        ("UPPER", xs.iter().map(|&x| (x, row(&up, x), 1500.0)).collect()),
        ("LOWER", xs.iter().map(|&x| (x, row(&lo, x), 1400.0)).collect()),
        ("OPENING_UPPER", xs.iter().map(|&x| (x, open_edge(x, true), 1300.0)).collect()),
        ("OPENING_LOWER", xs.iter().map(|&x| (x, open_edge(x, false), 1600.0)).collect()),
    ];
    for (name, pts) in &curves {
        let left: Vec<[f64; 3]> = pts.iter().map(|&(x, y, t)| spot(x, y, t)).collect();
        let n = left.len();
        let mut all: Vec<Anchor> = left.iter().take(n - 1).map(|&p| anchor_of(p)).collect();
        all.push(anchor_of([0.0, left[n - 1][1], left[n - 1][2]]));
        for p in left.iter().take(n - 1).rev() {
            all.push(anchor_of([-p[0], p[1], p[2]]));
        }
        println!("pub const {name}: [Anchor; {}] = [", all.len());
        for a in &all {
            println!("    {},", fmt(a));
        }
        println!("];");
    }

    // A sheet: every head's bake around the mouth with its own rows (yellow) and the median curves (green), 4 heads a row.
    let (bx, by, bw, bh) = (640u32, 1340u32, 572u32, 260u32);
    let per_row = 4;
    let rows = names.len().div_ceil(per_row) as u32;
    let mut sheet = image::RgbImage::new(bw * per_row as u32, bh * rows);
    for (k, n) in names.iter().enumerate() {
        let img = image::open(bake_path(n)).unwrap().to_rgb8();
        let mut tile = image::imageops::crop_imm(&img, bx, by, bw, bh).to_image();
        let mut put = |x: f64, y: f64, col: [u8; 3]| {
            let (px, py) = (x - f64::from(bx), y - f64::from(by));
            if px >= 0.0 && py >= 0.0 && px < f64::from(bw) && py < f64::from(bh) {
                tile.put_pixel(px as u32, py as u32, image::Rgb(col));
            }
        };
        for (i, &x) in cols.iter().enumerate() {
            let h = per_head[k][i];
            let m = (2 * MID - x) as f64;
            for (xx, y) in [(x as f64, h[0]), (x as f64, h[1]), (m, h[2]), (m, h[3])] {
                if y.is_finite() {
                    put(xx, y, [255, 255, 0]);
                }
            }
        }
        for (_, pts) in curves.iter().take(2) {
            for w in pts.windows(2) {
                for s in 0..=30 {
                    let t = f64::from(s) / 30.0;
                    let (x, y) = (w[0].0 + t * (w[1].0 - w[0].0), w[0].1 + t * (w[1].1 - w[0].1));
                    put(x, y, [0, 255, 0]);
                    put(2.0 * MID as f64 - x, y, [0, 255, 0]);
                }
            }
        }
        image::imageops::replace(&mut sheet, &tile, i64::from((k % per_row) as u32 * bw), i64::from((k / per_row) as u32 * bh));
    }
    sheet.save(out.join("detect_sheet.jpg")).unwrap();
}
