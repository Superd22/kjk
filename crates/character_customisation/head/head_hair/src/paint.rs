//! Painted hair: a layer of the head's texture, along the same flow as the mesh hair (decision 35).
//!
//! Inside the hairline the layer is solid hair colour, so the skin never shows through gaps in the cap. Past the hairline it thins out
//! over `band_mm` into short strokes. The style's paint zones add short hair outside the hairline, such as at the temples and over the
//! ears. The strokes come from line integral convolution: at each pixel's point on the head, a fine 3D noise is averaged along the flow,
//! projected onto the skin, for 3.5 mm each way. That draws streaks along the flow that are the same on both sides of a texture seam.
//!
//! This layer lives here for now. It will move into `head_paint`'s layer stack.

use crate::field::Field;
use crate::outline::Outline;
use crate::shell::Shell;
use crate::style::{Paint, bump_weight, parse_colour};
use crate::frame::{angle_between, direction};
use crate::atlas::Image;
use crate::v3::{V3, hash3, length, mad, normalize, reject, smoothstep};

/// The size of the noise's cells, in metres. It sets the width of a painted stroke.
const NOISE_CELL: f64 = 0.00035;
const LIC_STEP: f64 = 0.00035;
const LIC_STEPS: usize = 10;

/// What the painted layer reads at one pixel: the point of the head and its normal.
#[derive(Clone, Copy, Debug)]
pub struct PixelPoint {
    pub x: u16,
    pub y: u16,
    pub position: V3,
    pub normal: V3,
}

/// The streak value at a point: 3D noise averaged along the flow, spread back to an even 0 to 1.
pub fn streak(field: &Field, p: V3, normal: V3, seed: u64) -> f64 {
    let flow = |q: V3| {
        let f = field.at(q).unwrap_or([0.0, -1.0, 0.0]);
        let t = normalize(reject(f, normal));
        if length(t) > 0.0 { t } else { normalize(reject([0.0, -1.0, 0.0], normal)) }
    };
    let noise = |q: V3| hash3((q[0] / NOISE_CELL).floor() as i64, (q[1] / NOISE_CELL).floor() as i64, (q[2] / NOISE_CELL).floor() as i64, seed);
    let mut sum = noise(p);
    let mut count = 1.0;
    for sign in [1.0, -1.0] {
        let mut q = p;
        for _ in 0..LIC_STEPS {
            q = mad(q, flow(q), sign * LIC_STEP);
            sum += noise(q);
            count += 1.0;
        }
    }
    let z = (sum / count - 0.5) / (0.2887 / count.sqrt());
    0.5 + 0.5 * (0.8 * z).tanh()
}

/// The painted layer's colour and opacity at one point. With an outline, the painted hairline is where the mesh hair meets the skin;
/// without one it is the style's hairline. The style's own painted hairline, when it has one, adds to either, for short hair below the
/// mesh hair.
#[allow(clippy::too_many_arguments)]
pub fn paint_at(shell: &Shell, field: &Field, paint: &Paint, outline: Option<&Outline>, colour: [f64; 3], p: V3, normal: V3, seed: u64) -> Option<([f64; 3], f64)> {
    let d = shell.dir(p);
    let mesh = match outline {
        Some(o) if paint.follow_mesh => o.signed_mm(d),
        _ => shell.hairline_mm(d),
    };
    let m = match &paint.hairline {
        Some(knots) => mesh.max(shell.line_mm(knots, d)),
        None => mesh,
    };
    let zone = paint.zones.iter().map(|z| bump_weight(angle_between(d, direction(z.az, z.el)), z.radius_deg) * z.mm.clamp(0.0, 1.0)).fold(0.0, f64::max);
    if m < -paint.band_mm && zone <= 0.0 {
        return None;
    }
    let s = streak(field, p, normal, seed);
    let edge = if m >= 0.0 {
        1.0
    } else {
        let t = 1.0 + m / paint.band_mm.max(0.1);
        if t > 0.0 { smoothstep(1.0 - t - 0.15, 1.0 - t + 0.15, s) * t.sqrt() } else { 0.0 }
    };
    let zoned = if zone > 0.0 { smoothstep(1.0 - zone - 0.15, 1.0 - zone + 0.15, s) * zone.sqrt() } else { 0.0 };
    let alpha = edge.max(zoned) * paint.opacity;
    (alpha > 0.0).then(|| (colour.map(|c| (c * (0.8 + 0.4 * s)).clamp(0.0, 1.0)), alpha))
}

/// Paints the layer for every pixel of a texture of `resolution` pixels, on as many threads as the machine has.
#[allow(clippy::too_many_arguments)]
pub fn paint_layer(shell: &Shell, field: &Field, paint: &Paint, outline: Option<&Outline>, painted_hex: &str, pixels: &[PixelPoint], resolution: usize, seed: u64) -> Image {
    let colour = parse_colour(painted_hex);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let chunk = pixels.len().div_ceil(threads).max(1);
    let results: Vec<Vec<(usize, [f64; 3], f64)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = pixels
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .filter_map(|px| paint_at(shell, field, paint, outline, colour, px.position, px.normal, seed).map(|(c, a)| (px.y as usize * resolution + px.x as usize, c, a)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("a paint thread panicked")).collect()
    });
    let mut img = Image::new(resolution, resolution);
    for list in results {
        for (at, c, a) in list {
            img.px[at] = [c[0] as f32, c[1] as f32, c[2] as f32, a as f32];
        }
    }
    img
}

/// Puts the layer over a texture of RGB bytes of the same size.
pub fn composite(rgb: &mut [u8], layer: &Image) {
    for (i, p) in layer.px.iter().enumerate() {
        let a = p[3] as f64;
        if a <= 0.0 {
            continue;
        }
        for c in 0..3 {
            let under = rgb[3 * i + c] as f64 / 255.0;
            rgb[3 * i + c] = ((under * (1.0 - a) + p[c] as f64 * a).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Grid;

    #[test]
    fn streaks_run_along_the_flow() {
        // A field along x everywhere: values along x should agree more than values across it.
        let grid = Grid::covering([-0.02, -0.02, -0.02], [0.02, 0.02, 0.02], 4);
        let field = Field { grid, values: vec![[1.0, 0.0, 0.0]; grid.len()] };
        let n = [0.0, 0.0, 1.0];
        let (mut along, mut across) = (0.0, 0.0);
        for i in 0..400 {
            let p = [0.0, (i as f64 - 200.0) * 0.00005, 0.0];
            let a = streak(&field, p, n, 3);
            along += (a - streak(&field, [p[0] + 0.0004, p[1], p[2]], n, 3)).abs();
            across += (a - streak(&field, [p[0], p[1] + 0.0004, p[2]], n, 3)).abs();
        }
        assert!(along < across * 0.6, "along {along} across {across}");
    }
}
