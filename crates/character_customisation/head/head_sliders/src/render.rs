//! Clay pictures of a feature for checking sliders by eye (native builds only): an orthographic view of a mesh in the frame of the
//! eyes, lit like clay, cropped to the feature by the shot's window, with chosen triangles painted red and others tinted.

use crate::feature::Shot;
use crate::{V3, cross, dot, normalize, sub};
use image::{Rgb, RgbImage};

/// Pixels per mm: 2.5, or the value of `SLIDERS_PX`.
fn px_per_mm() -> f64 {
    std::env::var("SLIDERS_PX").ok().and_then(|v| v.parse().ok()).unwrap_or(2.5)
}

/// A place turned for a shot: by `yaw` about the up axis, then by `pitch` about the across axis (positive looks from below).
fn turn(shot: &Shot, p: V3) -> V3 {
    let (sy, cy) = shot.yaw.to_radians().sin_cos();
    let (x, z) = (p[0] * cy + p[2] * sy, -p[0] * sy + p[2] * cy);
    let (sp, cp) = shot.pitch.to_radians().sin_cos();
    let y = p[1] - shot.pivot;
    [x, cp * y + sp * z + shot.pivot, -sp * y + cp * z]
}

/// Draws `positions` (in the frame of the eyes, mm) as clay. `red` marks triangles to paint red, and `tint` gives other triangles a
/// colour of their own (for example KOTOR's hair in blue).
pub fn clay(positions: &[V3], triangles: &[[u32; 3]], shot: &Shot, red: &[bool], tint: &[Option<[f64; 3]>]) -> RgbImage {
    let px = px_per_mm();
    let window = shot.window;
    let w = ((window[1] - window[0]) * px) as u32;
    let h = ((window[3] - window[2]) * px) as u32;
    let p: Vec<V3> = positions.iter().map(|&q| turn(shot, q)).collect();
    let mut normals = vec![[0.0; 3]; p.len()];
    for t in triangles {
        let n = cross(sub(p[t[1] as usize], p[t[0] as usize]), sub(p[t[2] as usize], p[t[0] as usize]));
        for &i in t {
            for c in 0..3 {
                normals[i as usize][c] += n[c];
            }
        }
    }
    let light = normalize([-0.35, 0.45, 1.0]);
    let mut img = RgbImage::from_pixel(w, h, Rgb([40, 44, 52]));
    let mut depth = vec![f64::NEG_INFINITY; (w * h) as usize];
    let to_px = |q: V3| [(q[0] - window[0]) * px, (window[3] - q[1]) * px, q[2]];
    for (k, t) in triangles.iter().enumerate() {
        let v = t.map(|i| to_px(p[i as usize]));
        let area = (v[1][0] - v[0][0]) * (v[2][1] - v[0][1]) - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
        if area.abs() < 1e-9 {
            continue;
        }
        let x0 = v.iter().map(|q| q[0]).fold(f64::MAX, f64::min).floor().max(0.0) as i64;
        let x1 = v.iter().map(|q| q[0]).fold(f64::MIN, f64::max).ceil().min(w as f64 - 1.0) as i64;
        let y0 = v.iter().map(|q| q[1]).fold(f64::MAX, f64::min).floor().max(0.0) as i64;
        let y1 = v.iter().map(|q| q[1]).fold(f64::MIN, f64::max).ceil().min(h as f64 - 1.0) as i64;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
                let b0 = ((v[1][0] - px) * (v[2][1] - py) - (v[2][0] - px) * (v[1][1] - py)) / area;
                let b1 = ((v[2][0] - px) * (v[0][1] - py) - (v[0][0] - px) * (v[2][1] - py)) / area;
                let b2 = 1.0 - b0 - b1;
                if b0 < 0.0 || b1 < 0.0 || b2 < 0.0 {
                    continue;
                }
                let z = b0 * v[0][2] + b1 * v[1][2] + b2 * v[2][2];
                let at = (y as u32 * w + x as u32) as usize;
                if z <= depth[at] {
                    continue;
                }
                depth[at] = z;
                let mut n = [0.0; 3];
                for (b, &i) in [b0, b1, b2].iter().zip(t) {
                    for c in 0..3 {
                        n[c] += b * normals[i as usize][c];
                    }
                }
                let n = normalize(n);
                let lit = if n[2] < 0.0 { 0.15 } else { 0.25 + 0.75 * dot(n, light).max(0.0) };
                let base = if red.get(k).copied().unwrap_or(false) {
                    [235.0, 60.0, 50.0]
                } else {
                    tint.get(k).copied().flatten().unwrap_or([222.0, 196.0, 172.0])
                };
                img.put_pixel(x as u32, y as u32, Rgb(base.map(|c| (c * lit).min(255.0) as u8)));
            }
        }
    }
    img
}

/// Writes a short label in the top left corner of a picture, in a 3 by 5 pixel font scaled by 2 (digits, capitals and a few signs).
pub fn label(img: &mut RgbImage, text: &str) {
    const GLYPHS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ-+.: _";
    const BITS: [u16; 42] = [
        0x7B6F, 0x2C97, 0x73E7, 0x73CF, 0x5BC9, 0x79CF, 0x79EF, 0x7249, 0x7BEF, 0x7BCF, 0x2BED, 0x6BAE, 0x7927, 0x6B6E, 0x79A7, 0x79A4,
        0x796F, 0x5BED, 0x7497, 0x126A, 0x5BAD, 0x4927, 0x5FED, 0x7B6D, 0x2B6A, 0x7BE4, 0x2B6B, 0x7BAD, 0x388E, 0x7492, 0x5B6F, 0x5B52,
        0x5BFD, 0x5AAD, 0x5A92, 0x72A7, 0x01C0, 0x05D0, 0x0002, 0x0410, 0x0000, 0x0007,
    ];
    let mut x0 = 3u32;
    for ch in text.to_uppercase().chars() {
        let Some(i) = GLYPHS.find(ch) else { x0 += 8; continue };
        for row in 0..5u32 {
            for col in 0..3u32 {
                if BITS[i] >> (14 - (row * 3 + col)) & 1 == 1 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let (x, y) = (x0 + col * 2 + dx, 3 + row * 2 + dy);
                            if x < img.width() && y < img.height() {
                                img.put_pixel(x, y, Rgb([255, 255, 120]));
                            }
                        }
                    }
                }
            }
        }
        x0 += 8;
    }
}

/// Puts pictures of the same size in a grid of `columns`, with a gap of 4 pixels.
pub fn sheet(tiles: &[RgbImage], columns: usize) -> RgbImage {
    let (w, h) = (tiles[0].width(), tiles[0].height());
    let rows = tiles.len().div_ceil(columns);
    let mut out = RgbImage::from_pixel(columns as u32 * (w + 4), rows as u32 * (h + 4), Rgb([15, 15, 18]));
    for (i, t) in tiles.iter().enumerate() {
        let (cx, cy) = ((i % columns) as u32 * (w + 4), (i / columns) as u32 * (h + 4));
        image::imageops::overlay(&mut out, t, cx as i64, cy as i64);
    }
    out
}

/// Draws `dots` (in the frame of the eyes, mm) as small red squares on a picture made with `shot`.
pub fn dots(img: &mut RgbImage, shot: &Shot, dots: &[V3]) {
    let px = px_per_mm();
    for &d in dots {
        let q = turn(shot, d);
        let (x, y) = ((q[0] - shot.window[0]) * px, (shot.window[3] - q[1]) * px);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (i, j) = (x as i64 + dx, y as i64 + dy);
                if i >= 0 && j >= 0 && (i as u32) < img.width() && (j as u32) < img.height() {
                    img.put_pixel(i as u32, j as u32, Rgb([230, 30, 30]));
                }
            }
        }
    }
}
