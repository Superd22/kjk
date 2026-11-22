//! The lip measure and the lip fit (paint track, part C3). Analysis only.
//!
//! `lip_fit measure <out.json> [--hand <json>] <name>...` measures KOTOR's lips and ours, painted from the live page's recipe with the
//! hand values put in, and writes both.
//! `lip_fit fit <out.json> <name>...` fits each head's lip sliders so that our lips measure like KOTOR's, and writes the values.
//! `lip_fit hair <ignored> <name>...` prints what `read_kotor` finds on each head: the hair in the bands round the lips, whether the head
//! counts as bearded, and the lip colour it can read. With `LIP_DEBUG` set it also prints the colours of the middle column.
//!
//! **The measure.** Everything is read on the texture in our layout, KOTOR's bake and our paint alike, in the same pixels. Column by
//! column across the mouth, the upper border is the row above the opening where the colour passes halfway from the lip's colour to the
//! skin's, coming in from the skin, and the lower border the same below it. From those rows and the opening's hole it reads:
//! - the border's place: the median row of each border over the middle columns, and the bow, which is how far the middle of the upper
//!   border dips below its highest point beside it;
//! - the lips' colour: the median Lab of the upper and of the lower lip, 3 pixels inside the borders and the opening;
//! - the skin beside the lips: the median Lab of a band 8 to 25 pixels above the upper border, on the middle columns;
//! - the mouth line: how much darker the darkest pixel next to the opening is than the lips, as a median over the columns;
//! - the light on the lower lip: its lightest middle band against the band next to the border, each band a median over the middle
//!   columns;
//! - the border's softness: the distance over which the colour goes from 20% to 80% of the way from the skin's colour to the lip's,
//!   across the upper border, as a median over the columns.
//! Rows are turned into millimetres with the head's own scale at the mouth.
//!
//! **The fit.** It starts from the defaults and repeats ten times: paint, measure, and move each slider by the gap of its own measure.
//! The border's place sets the upper and lower lip size and the bow. The lower lip's colour sets the redness, the darkness and the tint. Its
//! lightness is taken against the skin above the lips, and its a* and b* are KOTOR's own. Where KOTOR's skin there is more than 8 L*
//! darker than ours, KOTOR's lower lip colour itself is the target. The upper lip is left out of the colour, because KOTOR's upper lip is
//! often under a moustache or mixed with KOTOR's mouth inside. The upper lip against the lower sets the darker upper lip, only where KOTOR's
//! two lips are within 15 L* of each other, and the light and the mouth line set their own sliders, the light by half steps. If the natural
//! colour cannot come within 8 of KOTOR's lip colour (CIE76), it fits again with lipstick, and keeps the lipstick only if it comes at least 2
//! closer. A natural lip's lightness target is never lighter than our skin. Lipstick is not tried on a bearded head, nor where KOTOR's lower lip reads more than 3 L* lighter than our skin, because
//! that is KOTOR's teeth or mouth inside baked onto our lower lip. The fit keeps redness under 0.9, tint within 25 degrees, darkness from -30 to 30 L*, the darker upper lip from -6 to 14 L*, the
//! light from 2 to 14 L* (above 14 the lower lip's edge turns into a dark band), the lipstick's strength under 60, the upper lip size from -2 to 3 mm and the lower from -1 to 5 mm, and the bow
//! between -1.2 and 1.2 mm. A natural lip gets a border softness from its contrast with the skin: 1.2 mm at a colour gap of 14 or more, up
//! to 2.2 mm at 6 or less, because a lip close to the skin's colour has a barely visible border.
//!
//! **Beards.** `read_kotor` reads KOTOR's bake in our own lips' rows, painted with the default lips, so that a beard cannot move them. A
//! pixel is hair when it is dark and has lost most of the skin's chroma, when it is yellower than the skin, or when it is near black. Above
//! the lips, a pixel far darker than the skin and not redder than it counts too. A head is bearded when more than a third of the band 0.8 to
//! 4 mm above the upper lip is hair, or more than 60% of the band 1.5 to 4 mm under the lower lip while some hair shows above and its lip can still be read. On a
//! bearded head the borders, the light, the mouth line and the darker upper lip are not fitted, and the lower lip's colour is read only from
//! pixels that are surely lip: not hair, redder than the skin, and at most 20 L* darker than it. If fewer than a quarter of the pixels are
//! left, or what is left is more than 8 L* darker than the skin, the head keeps the default natural lip for its skin tone.
//!
//! The softness, the gloss, the width, the liner, the light rim and the shadow under the lip are measured but not fitted. The lopsidedness
//! and the lip lines are not measured. These stay at their defaults.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::{Common, RES, bake_path, head_names};
use head_paint::colour::srgb_to_lab;
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint::layout_points::{LayoutLocator, uv_of};
use head_paint::lips::{Lips, SLIDERS};
use head_paint::live::Layout;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::{Value, json};
use std::path::Path;

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
const N: u32 = 2048;
const MID: usize = 926;
const COLS: std::ops::RangeInclusive<usize> = 790..=1062;

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

fn median(mut v: Vec<f64>) -> f64 {
    v.retain(|x| x.is_finite());
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn quantile(mut v: Vec<f64>, q: f64) -> f64 {
    v.retain(|x| x.is_finite());
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// What the measure needs of the layout: the covered pixels and the opening's hole per column.
struct Frame {
    covered: Vec<bool>,
    hole: Vec<Option<(usize, usize)>>,
}

impl Frame {
    fn new(c: &Common) -> Self {
        let mut covered = vec![false; RES * RES];
        for p in &c.map.pixels {
            covered[p.y as usize * RES + p.x as usize] = true;
        }
        let hole = (0..RES)
            .map(|x| {
                let r: Vec<usize> = (1420..1500).filter(|&y| !covered[y * RES + x]).collect();
                Some((*r.first()?, *r.last()?))
            })
            .collect();
        Frame { covered, hole }
    }
}

/// A texture's Lab around the mouth, and its a* blurred over 5 by 5 painted pixels. Other pixels are NaN.
struct Mouth {
    lab: Vec<[f64; 3]>,
    a: Vec<f64>,
}

const BOX: (usize, usize, usize, usize) = (600, 1260, 1300, 1640);

impl Mouth {
    fn new(rgb: &image::RgbImage, frame: &Frame) -> Self {
        let mut lab = vec![[f64::NAN; 3]; RES * RES];
        for y in BOX.2..BOX.3 {
            for x in BOX.0..BOX.1 {
                if frame.covered[y * RES + x] {
                    lab[y * RES + x] = srgb_to_lab(rgb.get_pixel(x as u32, y as u32).0.map(|v| f64::from(v) / 255.0));
                }
            }
        }
        let mut a = vec![f64::NAN; RES * RES];
        for y in BOX.2 + 2..BOX.3 - 2 {
            for x in BOX.0 + 2..BOX.1 - 2 {
                let (mut s, mut n) = (0.0, 0.0);
                for dy in 0..5 {
                    for dx in 0..5 {
                        let v = lab[(y + dy - 2) * RES + x + dx - 2][1];
                        if v.is_finite() {
                            s += v;
                            n += 1.0;
                        }
                    }
                }
                if n > 0.0 && frame.covered[y * RES + x] {
                    a[y * RES + x] = s / n;
                }
            }
        }
        Mouth { lab, a }
    }

    /// A column's colour at a row: the mean Lab of 5 by 3 pixels, or NaN where none is painted.
    fn colour(&self, x: usize, y: i64) -> [f64; 3] {
        let mut sum = [0.0; 3];
        let mut n = 0.0;
        for dy in -1..=1i64 {
            for dx in -2..=2i64 {
                let p = self.lab[(y + dy) as usize * RES + (x as i64 + dx) as usize];
                if p[0].is_finite() {
                    for k in 0..3 {
                        sum[k] += p[k];
                    }
                    n += 1.0;
                }
            }
        }
        if n == 0.0 { [f64::NAN; 3] } else { sum.map(|v| v / n) }
    }

    /// The lip border on a column, going from the opening at `start` in direction `dir`. The lip's colour is the median of the 5 to 40
    /// rows past the opening, and the skin's the median of the rows 95 to 115 past it above the lips, or 120 to 140 below them. Each row
    /// gets its share of the way from the skin's colour to the lip's, and the border is the first row, coming in from the skin's side,
    /// whose share passes one half. So lips of any colour count, and a dark band inside the lip, such as KOTOR's mouth inside baked onto
    /// our inner lip, does not count as the border.
    fn border(&self, x: usize, start: f64, dir: i64) -> f64 {
        let s = start.round() as i64;
        let lip = lab_median(&(5..40).map(|k| self.colour(x, s + dir * k)).collect::<Vec<_>>());
        let far = if dir < 0 { 95..115 } else { 120..140 };
        let skin = lab_median(&far.clone().map(|k| self.colour(x, s + dir * k)).collect::<Vec<_>>());
        let d = [0, 1, 2].map(|k| lip[k] - skin[k]);
        let dd = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if !dd.is_finite() || dd < 9.0 {
            return f64::NAN;
        }
        let share = |y: i64| { let c = self.colour(x, y); ((c[0] - skin[0]) * d[0] + (c[1] - skin[1]) * d[1] + (c[2] - skin[2]) * d[2]) / dd };
        let mut prev = f64::NAN;
        for k in (3..far.start).rev() {
            let v = share(s + dir * k);
            if v.is_finite() && v > 0.5 {
                let off = if prev.is_finite() && prev < 0.5 { (v - 0.5) / (v - prev) } else { 0.0 };
                return (s + dir * k) as f64 + dir as f64 * off;
            }
            if v.is_finite() {
                prev = v;
            }
        }
        f64::NAN
    }
}

/// The measures of one texture. Rows are layout rows; colours are CIELAB.
#[derive(Clone, Debug, Default)]
struct Measure {
    upper_row: f64,
    lower_row: f64,
    bow_px: f64,
    upper: [f64; 3],
    lower: [f64; 3],
    skin: [f64; 3],
    line: f64,
    light: f64,
    soft_px: f64,
    /// How much darker the band just inside both borders is than the middle of the lips, in L*.
    liner: f64,
    /// How much lighter the band just outside the upper border is than the skin above it, in L*.
    rim: f64,
    /// The spread of the lower lip's brightest pixels: its 97th percentile of L* against its 75th, in its middle third.
    gloss: f64,
    /// How much darker the skin 3 to 10 pixels under the lower border is than the skin 15 to 30 pixels under it, in L*.
    under: f64,
    /// How far the lips reach from the middle column, in pixels: the mean of both sides.
    reach_px: f64,
    /// The lower lip's median L* in ten bands from the opening to the border.
    profile: Vec<f64>,
    /// The rows of each column's borders, smoothed, from `COLS`.
    rows: Vec<[f64; 2]>,
}

fn smooth_rows(raw: &[f64], fallback: f64) -> Vec<f64> {
    let n = raw.len();
    let med: Vec<f64> = (0..n).map(|i| median((i.saturating_sub(7)..(i + 8).min(n)).map(|j| raw[j]).collect())).collect();
    med.into_iter().map(|v| if v.is_finite() { v } else { fallback }).collect()
}

fn lab_median(v: &[[f64; 3]]) -> [f64; 3] {
    [0, 1, 2].map(|k| median(v.iter().map(|p| p[k]).collect()))
}

/// Measures a texture. `rows` gives the borders to read the colours in; when `None`, the texture's own detected borders are used.
fn measure(m: &Mouth, frame: &Frame, rows: Option<&[[f64; 2]]>) -> Measure {
    let cols: Vec<usize> = COLS.collect();
    let opening = |x: usize| frame.hole[x].map_or((1460.0, 1460.0), |(t, b)| (t as f64, b as f64));
    let raw_up: Vec<f64> = cols.iter().map(|&x| { let (t, _) = opening(x); m.border(x, t - 1.0, -1) }).collect();
    let raw_lo: Vec<f64> = cols.iter().map(|&x| { let (_, b) = opening(x); m.border(x, b + 1.0, 1) }).collect();
    let up = smooth_rows(&raw_up, median(raw_up.clone()));
    let lo = smooth_rows(&raw_lo, median(raw_lo.clone()));
    let own: Vec<[f64; 2]> = up.iter().zip(&lo).map(|(&u, &l)| [u, l]).collect();
    let r = rows.unwrap_or(&own);
    let middle: Vec<usize> = (0..cols.len()).filter(|&i| (cols[i] as i64 - MID as i64).abs() <= 80).collect();
    let mut out = Measure { rows: own.clone(), ..Default::default() };
    out.upper_row = median(middle.iter().map(|&i| own[i][0]).collect());
    out.lower_row = median(middle.iter().map(|&i| own[i][1]).collect());
    let dip = median((0..cols.len()).filter(|&i| (cols[i] as i64 - MID as i64).abs() <= 6).map(|i| own[i][0]).collect());
    let peak = (0..cols.len()).filter(|&i| { let d = (cols[i] as i64 - MID as i64).abs(); (15..=70).contains(&d) }).map(|i| own[i][0]).fold(f64::INFINITY, f64::min);
    out.bow_px = dip - peak;
    let (mut upper, mut lower, mut skin, mut lines, mut lower_mid) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut edge_in, mut edge_out) = (Vec::new(), Vec::new());
    let (mut under_near, mut under_far) = (Vec::new(), Vec::new());
    for (i, &x) in cols.iter().enumerate() {
        let (t, b) = opening(x);
        let [ru, rl] = r[i];
        // The middle of each lip: from 35% to 85% of the way from the opening to the border.
        if !(ru.is_finite() && rl.is_finite()) {
            continue;
        }
        for y in (t - 0.85 * (t - ru)).ceil() as usize..=(t - 0.35 * (t - ru)) as usize {
            upper.push(m.lab[y * RES + x]);
        }
        for y in (b + 0.35 * (rl - b)).ceil() as usize..=(b + 0.85 * (rl - b)) as usize {
            let p = m.lab[y * RES + x];
            lower.push(p);
            if (x as i64 - MID as i64).abs() <= 45 {
                lower_mid.push(p[0]);
            }
        }
        if (x as i64 - MID as i64).abs() <= 66 {
            for y in (ru - 25.0) as usize..=(ru - 8.0) as usize {
                skin.push(m.lab[y * RES + x]);
            }
        }
        if (x as i64 - MID as i64).abs() <= 100 {
            for k in 4..=8 {
                edge_in.push(m.lab[(ru + k as f64).round() as usize * RES + x][0]);
                edge_in.push(m.lab[(rl - k as f64).round() as usize * RES + x][0]);
            }
            for k in 3..=10 {
                under_near.push(m.lab[(rl + k as f64).round() as usize * RES + x][0]);
            }
            for k in 15..=30 {
                under_far.push(m.lab[(rl + k as f64).round() as usize * RES + x][0]);
            }
            for k in 1..=3 {
                edge_out.push(m.lab[(ru - 1.0 - k as f64).round() as usize * RES + x][0]);
            }
        }
        let near: Vec<f64> = ((t - 3.0) as usize..=(b + 3.0) as usize).map(|y| m.lab[y * RES + x][0]).filter(|v| v.is_finite()).collect();
        if !near.is_empty() {
            lines.push(near.iter().copied().fold(f64::INFINITY, f64::min));
        }
    }
    out.upper = lab_median(&upper);
    out.lower = lab_median(&lower);
    out.skin = lab_median(&skin);
    out.line = 0.5 * (out.upper[0] + out.lower[0]) - median(lines);
    out.liner = 0.5 * (out.upper[0] + out.lower[0]) - median(edge_in);
    out.rim = median(edge_out) - out.skin[0];
    out.gloss = quantile(lower_mid.clone(), 0.97) - quantile(lower_mid.clone(), 0.75);
    out.under = median(under_far) - median(under_near);
    // The reach: from the middle outward along the opening, the last column where the lips 5 pixels above or below the opening are
    // still redder than halfway between the lips and the skin.
    let thr = 0.5 * (0.5 * (out.upper[1] + out.lower[1]) + out.skin[1]);
    let mut reach = Vec::new();
    for side in [-1i64, 1] {
        let (mut top, mut bottom) = (1460.0, 1460.0);
        let mut last = 0;
        for k in 0..300i64 {
            let x = (MID as i64 + side * k) as usize;
            if let Some((t, b)) = frame.hole[x] {
                (top, bottom) = (t as f64, b as f64);
            } else {
                let mid = 0.5 * (top + bottom);
                (top, bottom) = (mid, mid);
            }
            let a = m.a[(top - 5.0).round() as usize * RES + x].max(m.a[(bottom + 5.0).round() as usize * RES + x]);
            if !(a.is_finite() && a > thr) {
                if k > 60 {
                    break;
                }
                continue;
            }
            last = k;
        }
        reach.push(last as f64);
    }
    out.reach_px = 0.5 * (reach[0] + reach[1]);
    // The light: the lower lip's lightness across its height, in ten bands from the opening to the border, each the median over the
    // middle columns so that lip lines and specks do not count. It is the lightest of the four middle bands against the band next to the
    // border. The three bands next to the opening are left out, because KOTOR's dark mouth inside lands there on our inner lip.
    let mut bands = vec![Vec::new(); 10];
    for (i, &x) in cols.iter().enumerate() {
        let (_, b) = opening(x);
        let rl = r[i][1];
        if !rl.is_finite() || (x as i64 - MID as i64).abs() > 45 || rl - b < 5.0 {
            continue;
        }
        for (k, band) in bands.iter_mut().enumerate() {
            let y = (b + (k as f64 + 0.5) / 10.0 * (rl - b)).round() as usize;
            band.push(m.lab[y * RES + x][0]);
        }
    }
    let prof: Vec<f64> = bands.into_iter().map(median).collect();
    out.light = prof[3].max(prof[4]).max(prof[5]).max(prof[6]) - prof[8];
    out.profile = prof.clone();
    let _ = lower_mid;
    // The softness: across the upper border at each middle column, the share of the way from the skin's colour to the lip's colour, and
    // the rows where it passes 20% and 80%. Colours are averaged over 3 columns.
    let mut widths = Vec::new();
    for &i in &middle {
        let x = cols[i];
        let row = own[i][0];
        if !row.is_finite() {
            continue;
        }
        let at = |y: f64| -> [f64; 3] {
            let yy = y.round() as usize;
            let v: Vec<[f64; 3]> = (x - 1..=x + 1).map(|xx| m.lab[yy * RES + xx]).collect();
            [0, 1, 2].map(|k| v.iter().map(|p| p[k]).sum::<f64>() / 3.0)
        };
        let skin = lab_median(&(8..14).map(|k| at(row - k as f64)).collect::<Vec<_>>());
        let lip = lab_median(&(5..9).map(|k| at(row + k as f64)).collect::<Vec<_>>());
        let d = [0, 1, 2].map(|k| lip[k] - skin[k]);
        let dd = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if !dd.is_finite() || dd < 16.0 {
            continue;
        }
        let share = |y: f64| { let c = at(y); ((c[0] - skin[0]) * d[0] + (c[1] - skin[1]) * d[1] + (c[2] - skin[2]) * d[2]) / dd };
        let level = |q: f64| -> Option<f64> {
            let rows: Vec<f64> = (-14..9).map(|k| row + f64::from(k)).collect();
            rows.windows(2).find(|w| share(w[0]) < q && share(w[1]) >= q).map(|w| { let (a, b) = (share(w[0]), share(w[1])); w[0] + (q - a) / (b - a) })
        };
        if let (Some(a), Some(b)) = (level(0.2), level(0.8)) {
            widths.push((b - a).max(0.3));
        }
    }
    out.soft_px = median(widths);
    out
}

/// Millimetres per layout pixel at the middle of the mouth, along the column.
fn mm_per_px(c: &Common, pos: &[[f64; 3]]) -> f64 {
    let per_layout: Vec<[f64; 3]> = c.vmap.iter().map(|&m| pos[m]).collect();
    let locator = LayoutLocator::new(&c.uv, &c.tris);
    let a = locator.spot(uv_of((926.5, 1390.5)), &per_layout).expect("spot").position_mm;
    let b = locator.spot(uv_of((926.5, 1430.5)), &per_layout).expect("spot").position_mm;
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() / 40.0
}

struct Painter {
    painter: GpuPainter,
    common: Common,
    recipes: Value,
}

impl Painter {
    fn texture(&mut self, values: &[f64]) -> image::RgbImage {
        let recipe = recipe_from(values);
        self.painter.paint(N, &recipe, &layers_from(&vec![true; head_paint::controls::GROUPS.len()]));
        let rgba = pollster::block_on(self.painter.read_rgba(N));
        image::RgbImage::from_fn(N, N, |x, y| {
            let at = 4 * (y * N + x) as usize;
            image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
        })
    }

    fn values(&self, name: &str, lips: &Lips) -> Vec<f64> {
        let head = &self.recipes["heads"][name];
        let list = controls();
        let mut v: Vec<f64> = list.iter().map(|c| head[&c.id].as_f64().unwrap_or(c.default)).collect();
        let mut l = *lips;
        for (k, s) in SLIDERS.iter().enumerate() {
            let at = list.iter().position(|c| c.id == format!("lips_{}", s.id)).expect("lip control");
            v[at] = *l.field(k);
        }
        v
    }
}

fn to_json(m: &Measure, mmpp: f64) -> Value {
    let r = |x: f64| (x * 100.0).round() / 100.0;
    let diff = [0, 1, 2].map(|k| r(0.5 * (m.upper[k] + m.lower[k]) - m.skin[k]));
    json!({
        "upper_row": r(m.upper_row), "lower_row": r(m.lower_row), "bow_mm": r(m.bow_px * mmpp), "soft_mm": r(m.soft_px * mmpp),
        "upper": m.upper.map(r), "lower": m.lower.map(r), "skin": m.skin.map(r), "line": r(m.line), "light": r(m.light), "liner": r(m.liner), "rim": r(m.rim), "gloss": r(m.gloss), "under": r(m.under), "reach_mm": r(m.reach_px * mmpp), "profile": m.profile.iter().map(|&v| r(v)).collect::<Vec<_>>(),
        "lip_minus_skin": diff,
        "upper_minus_lower_l": r(m.upper[0] - m.lower[0]),
    })
}

fn lips_from(v: &Value) -> Lips {
    let mut l = Lips::DEFAULT;
    for (k, s) in SLIDERS.iter().enumerate() {
        if let Some(x) = v[format!("lips_{}", s.id)].as_f64().or_else(|| v[s.id].as_f64()) {
            *l.field(k) = x;
        }
    }
    l
}

fn clamp(l: &mut Lips) {
    for (k, s) in SLIDERS.iter().enumerate() {
        let f = l.field(k);
        *f = f.clamp(s.min, s.max);
    }
}

fn de(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}


/// Whether a KOTOR pixel is hair rather than skin or lip, against the skin colour `skin`: a dark pixel that has lost most of the skin's
/// chroma (black, grey or dark brown hair), a pixel turned yellower than the skin (blond or brown hair), or a near black one. Shaded skin
/// keeps its hue and chroma, and lips are redder than the skin, so neither counts. With `dark_counts`, a pixel far darker than the skin
/// and not redder than it also counts, which catches a moustache of the skin's own hue but also the chin's painted shadow, so it is
/// used above the lips only.
fn is_hair(p: [f64; 3], skin: [f64; 3], dark_counts: bool) -> bool {
    if !p[0].is_finite() {
        return false;
    }
    let (cp, cs) = (p[1].hypot(p[2]), skin[1].hypot(skin[2]));
    let turn = (p[2].atan2(p[1]) - skin[2].atan2(skin[1])).to_degrees();
    let lip_like = p[1] > skin[1] + 4.0 && p[1] > p[2];
    let much_darker = p[0] < skin[0] - 18.0 && p[0] < 0.75 * skin[0];
    p[0] < 12.0 || (p[0] < skin[0] - 10.0 && p[0] < 0.7 * skin[0] && cp < 0.65 * cs) || (turn > 12.0 && de(p, skin) > 8.0) || (dark_counts && much_darker && !lip_like)
}

/// What can be trusted of KOTOR's lips on one head, read in our own lips' rows so that a beard cannot move them.
struct KotorRead {
    /// The share of hair pixels in the skin band above the upper lip and below the lower lip.
    hair_above: f64,
    hair_below: f64,
    /// A beard or a moustache touches the lips: KOTOR's borders are not trusted, and the shape keeps its defaults.
    bearded: bool,
    /// The lower lip's colour from the pixels that are not hair, or `None` where too few are left or what is left is not lip-coloured.
    lip: Option<[f64; 3]>,
    /// The share of the lower lip's pixels that were kept.
    kept: f64,
}

fn read_kotor(m: &Mouth, frame: &Frame, ours: &Measure, skin: [f64; 3]) -> KotorRead {
    let cols: Vec<usize> = COLS.collect();
    let opening = |x: usize| frame.hole[x].map_or((1460.0, 1460.0), |(t, b)| (t as f64, b as f64));
    let (mut above, mut below, mut core) = (Vec::new(), Vec::new(), Vec::new());
    for (i, &x) in cols.iter().enumerate() {
        let [ru, rl] = ours.rows[i];
        if !(ru.is_finite() && rl.is_finite()) || (x as i64 - MID as i64).abs() > 90 {
            continue;
        }
        let (_, b) = opening(x);
        for y in (ru - 40.0) as usize..=(ru - 8.0) as usize {
            above.push(m.lab[y * RES + x]);
        }
        for y in (rl + 15.0) as usize..=(rl + 40.0) as usize {
            below.push(m.lab[y * RES + x]);
        }
        if (x as i64 - MID as i64).abs() <= 70 {
            for y in (b + 0.3 * (rl - b)).ceil() as usize..=(b + 0.7 * (rl - b)) as usize {
                core.push(m.lab[y * RES + x]);
            }
        }
    }
    let share = |v: &[[f64; 3]], dark: bool| v.iter().filter(|p| is_hair(**p, skin, dark)).count() as f64 / v.iter().filter(|p| p[0].is_finite()).count().max(1) as f64;
    let (hair_above, hair_below) = (share(&above, true), share(&below, false));
    // The bands start 0.8 mm above the upper lip and 1.5 mm under the lower lip, past KOTOR's dark lip edges and the shadow under the
    // lower lip, and run 3 to 4 mm out. A band that is more than a third hair above, or more than 60% hair below, means a beard.
    let bearded = hair_above > 0.35 || hair_below > 0.6;
    let total = core.iter().filter(|p| p[0].is_finite()).count().max(1) as f64;
    // On a bearded head only the pixels that are surely lip count: redder than the skin and not far darker than it, which leaves out the
    // hair and KOTOR's dark mouth inside where it lands on our lower lip.
    let sure = |p: &[f64; 3]| !bearded || (p[1] > skin[1] + 4.0 && p[1] > p[2] && p[0] > skin[0] - 20.0);
    let clean: Vec<[f64; 3]> = core.iter().copied().filter(|p| p[0].is_finite() && !is_hair(*p, skin, true) && sure(p)).collect();
    let kept = clean.len() as f64 / total;
    // A trimmed median: the pixels within 12 L* of the median lightness, which leaves out stray hair and the mouth's inside.
    let mid = median(clean.iter().map(|p| p[0]).collect());
    let trimmed: Vec<[f64; 3]> = clean.into_iter().filter(|p| (p[0] - mid).abs() < 12.0).collect();
    let lip = lab_median(&trimmed);
    // What is left must be lip: at least as red as the skin, and not yellower than it.
    let lip_like = lip.iter().all(|v| v.is_finite()) && lip[1] >= skin[1] - 2.0 && lip[2].atan2(lip[1]) <= skin[2].atan2(skin[1]) + 0.1;
    // Under a beard, a lip read much darker than the skin is the beard's shadow or KOTOR's mouth inside, not the lip. A natural lip there
    // is at most 8 L* darker than the skin.
    let lip_like = lip_like && (!bearded || lip[0] >= skin[0] - 8.0);
    let lip = (kept >= if bearded { 0.25 } else { 0.4 } && lip_like).then_some(lip);
    // Hair under the lips alone is less sure than a moustache: the chin's painted shadow and KOTOR's lower lip edge read as hair too. Such
    // a head is only taken as bearded when a little hair also shows above the lips and its lip can still be read.
    let bearded = bearded && (hair_above > 0.35 || (hair_above > 0.05 && lip.is_some()));
    KotorRead { hair_above, hair_below, bearded, lip, kept }
}

/// One fit from a start, with lipstick or the natural colour. Returns the sliders, the colour gap left and our last measure. `ours0` is
/// our paint measured with the default lips, whose rows come from our mesh and so cannot be moved by a beard.
#[allow(clippy::too_many_arguments)]
fn fit_from(p: &mut Painter, name: &str, start: Lips, stick: bool, k: &Measure, read: &KotorRead, ours0: &Measure, frame: &Frame, mmpp: f64) -> (Lips, f64, Measure) {
    let mut l = start;
    let first = p.texture(&p.values(name, &l));
    let our_skin = measure(&Mouth::new(&first, frame), frame, Some(&ours0.rows)).skin;
    // On a bearded head, KOTOR's borders are not trusted, so the colours are read in our own rows, on the pixels that are not hair.
    let rows: &[[f64; 2]] = if read.bearded { &ours0.rows } else { &k.rows };
    let our_lip = |tex: &image::RgbImage| -> (Measure, [f64; 3]) {
        let m = Mouth::new(tex, frame);
        let o = measure(&m, frame, Some(rows));
        let lip = if read.bearded { read_kotor(&m, frame, ours0, our_skin).lip.unwrap_or(o.lower) } else { o.lower };
        (o, lip)
    };
    // The lower lip sets the colour: KOTOR's upper lip is often hidden under a moustache, or mixed with the inside of KOTOR's mouth
    // where it is baked onto our inner lip.
    let kotor_lip = if read.bearded { read.lip.unwrap_or([f64::NAN; 3]) } else { k.lower };
    // A moustache makes KOTOR's skin above the lips darker than ours; a lighter KOTOR skin is only a base tone that differs.
    let skin_ok = !read.bearded && k.skin[0] > our_skin[0] - 8.0;
    // Only the lightness is taken against the skin: a* and b* are KOTOR's own, so a lip keeps its hue when our skin's hue differs.
    // Half of the difference between KOTOR's skin and ours is carried into the lip's lightness: the full difference made lips black
    // where our skin is much darker than KOTOR's, and none made them vanish where it is much lighter.
    let target = if skin_ok { [kotor_lip[0] + 0.5 * (our_skin[0] - k.skin[0]), kotor_lip[1], kotor_lip[2]] } else { kotor_lip };
    // A natural lip is not lighter than the skin round it. A lighter read is KOTOR's teeth or mouth inside baked onto our lower lip.
    let target = if stick { target } else { [target[0].min(our_skin[0]), target[1], target[2]] };
    // The upper lip against the lower is fitted only where the skin above matches and KOTOR's two lips are within 15 L* of each other.
    let shade_ok = skin_ok && (k.upper[0] - k.lower[0]).abs() < 15.0;
    if stick {
        l.stick = 1.0;
        let c = (target[1].powi(2) + target[2].powi(2)).sqrt();
        l.stick_lightness = target[0];
        l.stick_chroma = c;
        l.stick_hue = target[2].atan2(target[1]).to_degrees().rem_euclid(360.0);
    }
    for _ in 0..10 {
        let tex = p.texture(&p.values(name, &l));
        let (o, ours) = our_lip(&tex);
        // The shape, from our own detected borders, where KOTOR's are trusted.
        if !read.bearded {
            let own = measure(&Mouth::new(&tex, frame), frame, None);
            if own.upper_row.is_finite() && k.upper_row.is_finite() {
                l.upper_mm += 0.8 * (own.upper_row - k.upper_row) * mmpp;
            }
            if own.lower_row.is_finite() && k.lower_row.is_finite() {
                l.lower_mm += 0.8 * (k.lower_row - own.lower_row) * mmpp;
            }
            if own.bow_px.is_finite() && k.bow_px.is_finite() {
                l.bow_mm += 0.6 * (k.bow_px - own.bow_px) * mmpp;
            }
        }
        if std::env::var("LIP_DEBUG").is_ok() {
            eprintln!("  stick {stick} target {:?} ours {:?} blood {:.3} dark {:.2} tint {:.1}", target.map(f64::round), ours.map(f64::round), l.blood, l.darkness, l.tint_degrees);
        }
        let colour_ok = target.iter().chain(&ours).all(|v| v.is_finite());
        if !colour_ok {
        } else if stick {
            l.stick_lightness += target[0] - ours[0];
            let (ct, co) = ((target[1].powi(2) + target[2].powi(2)).sqrt(), (ours[1].powi(2) + ours[2].powi(2)).sqrt());
            l.stick_chroma += ct - co;
            let dh = (target[2].atan2(target[1]) - ours[2].atan2(ours[1])).to_degrees();
            l.stick_hue = (l.stick_hue + 0.8 * ((dh + 180.0).rem_euclid(360.0) - 180.0)).rem_euclid(360.0);
        } else {
            // Half steps: near little blood, a* moves by about 100 per unit of blood, and full steps swung the lips between red and skin.
            l.blood += 0.5 * (target[1] - ours[1]) / 60.0;
            let dh = (target[2].atan2(target[1]) - ours[2].atan2(ours[1])).to_degrees();
            l.tint_degrees += 0.4 * ((dh + 180.0).rem_euclid(360.0) - 180.0);
            l.darkness += 0.8 * (ours[0] - target[0]);
        }
        if !read.bearded {
            let shade = (o.lower[0] - o.upper[0]) - (k.lower[0] - k.upper[0]);
            if shade.is_finite() && shade_ok {
                l.upper_shade -= 0.8 * shade;
            }
            // The light: half steps, because the measure's percentile moves less than the slider near the top of its range.
            if k.light.is_finite() && o.light.is_finite() {
                l.highlight += 0.5 * (k.light - o.light);
            }
            if k.line.is_finite() && o.line.is_finite() {
                l.mouth_line += 0.8 * (k.line - o.line);
            }
        }
        l.bow_mm = l.bow_mm.clamp(-1.2, 1.2);
        clamp(&mut l);
        // The fit keeps to the ranges that natural lips and the measure's reliability allow. A measure misread by KOTOR's mouth inside
        // otherwise drives the sliders to plum, magenta or black lips.
        l.blood = l.blood.min(0.9);
        l.tint_degrees = l.tint_degrees.clamp(-25.0, 25.0);
        l.darkness = l.darkness.clamp(-30.0, 30.0);
        l.upper_shade = l.upper_shade.clamp(-6.0, 14.0);
        l.highlight = l.highlight.clamp(2.0, 14.0);
        l.stick_chroma = l.stick_chroma.min(60.0);
        l.upper_mm = l.upper_mm.clamp(-2.0, 3.0);
        l.lower_mm = l.lower_mm.clamp(-1.0, 5.0);
    }
    let tex = p.texture(&p.values(name, &l));
    let (o, ours) = our_lip(&tex);
    (l, de(ours, target), o)
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let hand: Value = match args.iter().position(|a| a == "--hand") {
        Some(k) => {
            let p = args.remove(k + 1);
            args.remove(k);
            serde_json::from_str(&std::fs::read_to_string(p).expect("hand")).expect("json")
        }
        None => Value::Null,
    };
    let mode = args[0].clone();
    let out = args[1].clone();
    let names: Vec<String> = if args.len() > 2 { args[2..].to_vec() } else { head_names() };
    let common = Common::load();
    let frame = Frame::new(&common);
    let recipes: Value = serde_json::from_str(&std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes")).expect("json");
    let mut p = Painter { painter: GpuPainter::new(Gpu::blocking().expect("GPU"), layout()), common, recipes };
    let mut result = serde_json::Map::new();
    for name in &names {
        let pos = positions(name);
        let mmpp = mm_per_px(&p.common, &pos);
        let kotor_img = image::open(bake_path(name)).unwrap().to_rgb8();
        let k = measure(&Mouth::new(&kotor_img, &frame), &frame, None);
        p.painter.set_head(pos);
        pollster::block_on(p.painter.prepare(N));
        if mode == "hair" {
            let def = p.texture(&p.values(name, &Lips::DEFAULT));
            let ours0 = measure(&Mouth::new(&def, &frame), &frame, None);
            let read = read_kotor(&Mouth::new(&kotor_img, &frame), &frame, &ours0, ours0.skin);
            if std::env::var("LIP_DEBUG").is_ok() {
                let km = Mouth::new(&kotor_img, &frame);
                let i = MID - *COLS.start();
                let [ru, rl] = ours0.rows[i];
                for dy in [-40i64, -30, -25, -20, -15, -10, -6, -3, 0, 3, 6, 10, 15, 20, 25, 30] {
                    let y = if dy <= 0 { ru as i64 + dy } else { rl as i64 + dy };
                    let c = km.lab[y as usize * RES + MID];
                    println!("  {} {dy:4} row {y} {:?} hair {}", if dy <= 0 { "above" } else { "below" }, c.map(f64::round), is_hair(c, ours0.skin, dy <= 0));
                }
                println!("  rows {ru:.1} {rl:.1} hole {:?}", frame.hole[MID]);
            }
            println!("{name:24} above {:.2} below {:.2} bearded {} kept {:.2} lip {:?} skin {:?}", read.hair_above, read.hair_below, read.bearded, read.kept, read.lip.map(|c| c.map(|v| v.round())), ours0.skin.map(f64::round));
            continue;
        }
        if mode == "measure" {
            let lips = lips_from(&hand["heads"][name.as_str()]);
            let tex = p.texture(&p.values(name, &lips));
            let o = measure(&Mouth::new(&tex, &frame), &frame, Some(&k.rows));
            let own = measure(&Mouth::new(&tex, &frame), &frame, None);
            let mut oj = to_json(&o, mmpp);
            for key in ["upper_row", "lower_row", "bow_mm", "soft_mm"] {
                oj[key] = to_json(&own, mmpp)[key].clone();
            }
            result.insert(name.clone(), json!({ "mm_per_px": mmpp, "kotor": to_json(&k, mmpp), "ours": oj }));
            println!("{name}: KOTOR {} | ours {}", to_json(&k, mmpp), oj);
        } else {
            let def = p.texture(&p.values(name, &Lips::DEFAULT));
            let ours0 = measure(&Mouth::new(&def, &frame), &frame, None);
            let read = read_kotor(&Mouth::new(&kotor_img, &frame), &frame, &ours0, ours0.skin);
            // Where a beard leaves no lip that can be read, the head keeps a plain natural lip for its skin tone.
            let (best, gap, gap_n, o) = if read.bearded && read.lip.is_none() {
                (Lips::DEFAULT, f64::NAN, f64::NAN, ours0.clone())
            } else {
                let (nat, gap_n, on) = fit_from(&mut p, name, Lips::DEFAULT, false, &k, &read, &ours0, &frame, mmpp);
                // Lipstick is tried only where the lip read is sure: not on a bearded head, and not where KOTOR's lip reads more than 3 L*
                // lighter than our skin, which is KOTOR's teeth or mouth inside baked onto our lower lip rather than a pale lipstick.
                let stick_ok = !read.bearded && k.lower[0] <= ours0.skin[0] + 3.0;
                let (best, gap, o) = if gap_n > 8.0 && stick_ok {
                    let (st, gap_s, os) = fit_from(&mut p, name, nat, true, &k, &read, &ours0, &frame, mmpp);
                    if gap_s < gap_n - 2.0 { (st, gap_s, os) } else { (nat, gap_n, on) }
                } else {
                    (nat, gap_n, on)
                };
                (best, gap, gap_n, o)
            };
            // A natural lip whose colour is close to the skin's gets a softer border, up to 2.2 mm, as a man's barely visible upper border.
            let mut best = best;
            if best.stick == 0.0 {
                let contrast = de(o.lower, o.skin);
                best.softness_mm = if contrast.is_finite() { 1.2 + ((14.0 - contrast) / 8.0).clamp(0.0, 1.0) } else { 2.0 };
            }
            let mut values = serde_json::Map::new();
            let mut b = best;
            for (i, s) in SLIDERS.iter().enumerate() {
                values.insert(s.id.to_string(), json!((*b.field(i) * 1000.0).round() / 1000.0));
            }
            println!("{name}: colour gap {gap:.1} (natural {gap_n:.1}), {}", Value::Object(values.clone()));
            result.insert(name.clone(), json!({ "values": values, "colour_gap": (gap * 10.0).round() / 10.0, "natural_gap": (gap_n * 10.0).round() / 10.0, "kotor": to_json(&k, mmpp), "ours": to_json(&o, mmpp), "bearded": read.bearded, "hair_above": (read.hair_above * 100.0).round() / 100.0, "hair_below": (read.hair_below * 100.0).round() / 100.0, "kept": (read.kept * 100.0).round() / 100.0, "kotor_lip_read": read.lip.map(|c| c.map(|v| (v * 10.0).round() / 10.0)) }));
        }
    }
    std::fs::write(&out, serde_json::to_string_pretty(&json!({ "heads": result })).unwrap()).unwrap();
}
