//! The hair's own texture (decision 37), made by our code: one RGBA atlas with the cap's strands and a few card strips.
//!
//! The atlas is `SIZE` pixels square. The cap's disc fills the top left square of three quarters of the side. The right quarter holds
//! `STRIPS` vertical strips for the cards, root at the top. The bottom left is unused.
//!
//! The cap's texture is drawn from the strands themselves: every strand point that lies on the shell is projected into the disc, and the
//! strand is drawn there as a thin line over a base colour. So the cap's texture follows the flow, partings included. The strips are
//! generated: many thin hairs with gaps between them, a denser core, and transparency at the edges and the tip.

use crate::cap::Projection;
use crate::shell::Shell;
use crate::strands::Strand;
use crate::style::{Colours, parse_colour};
use crate::v3::{Rng, hash3, smoothstep};

pub const SIZE: usize = 1024;
pub const CAP_PIXELS: usize = 768;
pub const STRIPS: usize = 4;
pub const STRIP_WIDTH: usize = (SIZE - CAP_PIXELS) / STRIPS;

/// An image with straight (not premultiplied) alpha, as sRGB values from 0 to 1, row by row from the top.
#[derive(Clone, Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub px: Vec<[f32; 4]>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Image { width, height, px: vec![[0.0; 4]; width * height] }
    }

    /// Paints a colour over a pixel with the "over" rule.
    pub fn over(&mut self, x: usize, y: usize, rgb: [f64; 3], alpha: f64) {
        if x >= self.width || y >= self.height || alpha <= 0.0 {
            return;
        }
        let d = &mut self.px[y * self.width + x];
        let a = alpha.min(1.0) as f32;
        let out = a + d[3] * (1.0 - a);
        if out <= 0.0 {
            return;
        }
        for c in 0..3 {
            d[c] = (rgb[c] as f32 * a + d[c] * d[3] * (1.0 - a)) / out;
        }
        d[3] = out;
    }

    /// Draws a line with soft edges from `a` to `b`, in pixels.
    pub fn line(&mut self, a: [f64; 2], b: [f64; 2], width: f64, rgb: [f64; 3], alpha: f64) {
        let r = width / 2.0 + 1.0;
        let x0 = (a[0].min(b[0]) - r).floor().max(0.0) as usize;
        let x1 = ((a[0].max(b[0]) + r).ceil().max(0.0) as usize).min(self.width.saturating_sub(1));
        let y0 = (a[1].min(b[1]) - r).floor().max(0.0) as usize;
        let y1 = ((a[1].max(b[1]) + r).ceil().max(0.0) as usize).min(self.height.saturating_sub(1));
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len2 = dx * dx + dy * dy;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
                let t = if len2 > 0.0 { (((px - a[0]) * dx + (py - a[1]) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
                let (cx, cy) = (a[0] + t * dx - px, a[1] + t * dy - py);
                let dist = (cx * cx + cy * cy).sqrt();
                let cover = (width / 2.0 + 0.5 - dist).clamp(0.0, 1.0);
                if cover > 0.0 {
                    self.over(x, y, rgb, alpha * cover);
                }
            }
        }
    }

    /// The image as RGBA bytes, with every alpha set to 0 or 1 at `cut`. A card or cap drawn with an alpha test at `cut` looks the
    /// same, but a viewer whose canvas has alpha no longer shows the page through texels that pass the test.
    pub fn rgba8_cut(&self, cut: f32) -> Vec<u8> {
        self.px.iter().flat_map(|p| [p[0], p[1], p[2], if p[3] >= cut { 1.0 } else { 0.0 }].map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)).collect()
    }

    /// The image as RGBA bytes.
    pub fn rgba8(&self) -> Vec<u8> {
        self.px.iter().flat_map(|p| p.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)).collect()
    }
}

/// The atlas's texture coordinates of a pixel position: u to the right, v up.
pub fn uv_of_pixel(x: f64, y: f64) -> [f64; 2] {
    [x / SIZE as f64, 1.0 - y / SIZE as f64]
}

pub fn pixel_of_uv(uv: [f64; 2]) -> [f64; 2] {
    [uv[0] * SIZE as f64, (1.0 - uv[1]) * SIZE as f64]
}

/// The cap's place on the atlas, as texture coordinates.
pub fn cap_rect() -> crate::cap::CapRect {
    let f = CAP_PIXELS as f64 / SIZE as f64;
    crate::cap::CapRect { u: [0.0, f], v: [1.0 - f, 1.0] }
}

/// The strip `k`'s place on the atlas.
pub fn strip_rect(k: usize) -> crate::cards::StripRect {
    let x0 = (CAP_PIXELS + k * STRIP_WIDTH) as f64;
    crate::cards::StripRect { u: [(x0 + 1.0) / SIZE as f64, (x0 + STRIP_WIDTH as f64 - 1.0) / SIZE as f64], v_root: 1.0 - 1.0 / SIZE as f64, v_tip: 1.0 / SIZE as f64 }
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t)
}

fn shade(c: [f64; 3], f: f64) -> [f64; 3] {
    c.map(|x| (x * f).clamp(0.0, 1.0))
}

/// The direction halfway between the light and the eye that the cap's painted highlight assumes: in front of the face and above it.
const HALFWAY: [f64; 3] = [0.0, 0.707, 0.707];
const HIGHLIGHT_EXPONENT: f64 = 80.0;
const HIGHLIGHT_STRENGTH: f64 = 0.55;

/// Kajiya and Kay's highlight for a strand with a unit tangent: the sine of the angle between the tangent and `HALFWAY`, raised to
/// `HIGHLIGHT_EXPONENT`. It paints a lighter band where strands run across that direction, as KOTOR's hair textures do.
pub fn kajiya(tangent: [f64; 3]) -> f64 {
    let c = crate::v3::dot(tangent, HALFWAY);
    (1.0 - c * c).max(0.0).powf(HIGHLIGHT_EXPONENT / 2.0)
}

/// How much darker the hair is painted at a direction: down to 70% low on the back and sides, where little light reaches, and down to
/// 80% right at the hairline, where the hair is thin over the darker scalp.
pub fn occlusion(shell: &Shell, d: [f64; 3]) -> f64 {
    let low = 0.7 + 0.3 * smoothstep(-0.4, 0.6, d[1]);
    let edge = 0.8 + 0.2 * smoothstep(0.0, 12.0, shell.hairline_mm(d));
    low * edge
}

/// Draws the cap: a base colour over the region inside the hairline that frays out over `fade_mm`, then every strand's part that lies on
/// the shell.
pub fn bake_cap(img: &mut Image, shell: &Shell, projection: &Projection, strands: &[Strand], colours: &Colours, fade_mm: f64, seed: u64) {
    let rect = cap_rect();
    let (root, tip) = (parse_colour(&colours.root), parse_colour(&colours.tip));
    let base = shade(mix(root, tip, 0.2), 0.75);
    let to_px = |q: [f64; 2]| pixel_of_uv(rect.uv(q));
    let cap = CAP_PIXELS as f64;
    for y in 0..CAP_PIXELS {
        for x in 0..CAP_PIXELS {
            let q = [(x as f64 + 0.5) / cap * 2.0 - 1.0, 1.0 - (y as f64 + 0.5) / cap * 2.0];
            if q[0] * q[0] + q[1] * q[1] > 1.0 {
                continue;
            }
            let d = projection.from_plane(q);
            let m = shell.hairline_mm(d);
            let n = hash3((x / 3) as i64, (y / 3) as i64, 0, seed);
            let a = smoothstep(0.0, 1.0, 0.5 + m / fade_mm.max(0.1) + 0.5 * (n - 0.5));
            let fine = hash3(x as i64, y as i64, 1, seed);
            img.over(x, y, shade(base, (0.9 + 0.2 * fine) * occlusion(shell, d)), a);
        }
    }
    let mut rng = Rng::new(seed ^ 0xCAFE);
    let on_cap = 0.003;
    let highlight = tip.map(|c| (c * 1.9 + 0.08).min(1.0));
    for s in strands {
        // One strand in five is a highlight strand, lighter and thinner, as KOTOR's painted hair has.
        let bright = rng.unit() < 0.2;
        let f = (1.0 + colours.variation * (2.0 * s.seed - 1.0)) * if bright { 1.35 } else { 1.0 };
        let alpha = if bright { rng.range(0.6, 0.9) } else { rng.range(0.45, 0.85) };
        let width = if bright { rng.range(0.8, 1.2) } else { rng.range(1.0, 2.0) };
        let total = s.length().max(1e-6);
        let mut along = 0.0;
        let mut last: Option<[f64; 2]> = None;
        for (i, &p) in s.points.iter().enumerate() {
            if i > 0 {
                along += crate::v3::distance(s.points[i - 1], p);
            }
            if shell.above_shell(p) > on_cap {
                last = None;
                continue;
            }
            let q = projection.to_plane(shell.dir(p));
            if q[0] * q[0] + q[1] * q[1] > 1.0 {
                last = None;
                continue;
            }
            let px = to_px(q);
            if let Some(prev) = last
                && ((px[0] - prev[0]).powi(2) + (px[1] - prev[1]).powi(2)).sqrt() < 20.0
            {
                let tangent = crate::v3::normalize(crate::v3::sub(p, s.points[i - 1]));
                let c = shade(mix(root, tip, (along / total).min(1.0)), f * occlusion(shell, shell.dir(p)));
                let c = mix(c, highlight, HIGHLIGHT_STRENGTH * kajiya(tangent));
                img.line(prev, px, width, c, alpha);
            }
            last = Some(px);
        }
    }
}

/// Draws strip `k`: hairs from the root (top) down, denser and longer in the middle, with a solid core that narrows toward the tip.
pub fn bake_strip(img: &mut Image, k: usize, colours: &Colours, seed: u64) {
    let (root, tip) = (parse_colour(&colours.root), parse_colour(&colours.tip));
    let x0 = (CAP_PIXELS + k * STRIP_WIDTH) as f64;
    let w = STRIP_WIDTH as f64;
    let h = SIZE as f64;
    let mut rng = Rng::new(seed ^ (k as u64 + 1).wrapping_mul(0x51_7CC1));
    for y in 0..SIZE {
        let s = y as f64 / h;
        let half = w * (0.44 - 0.26 * s);
        for x in 0..STRIP_WIDTH {
            let off = (x as f64 + 0.5 - w / 2.0).abs();
            let a = smoothstep(half + 2.0, half - 2.0, off) * smoothstep(1.0, 0.8, s);
            img.over(x0 as usize + x, y, shade(mix(root, tip, 0.5 * s), 0.75), a);
        }
    }
    for _ in 0..90 {
        let centre = (w / 2.0 + rng.normal() * w * 0.24).clamp(3.0, w - 3.0);
        let edge = (centre - w / 2.0).abs() / (w / 2.0);
        let end = h * (0.55 + 0.45 * (1.0 - edge).sqrt() * rng.range(0.75, 1.0));
        let start = h * rng.range(0.0, 0.03);
        let width = rng.range(1.2, 2.4);
        let wobble = rng.range(0.5, 2.5);
        let period = rng.range(180.0, 500.0);
        let phase = rng.range(0.0, std::f64::consts::TAU);
        let f = 1.0 + colours.variation * rng.range(-1.0, 1.0);
        let steps = 48;
        let mut prev: Option<[f64; 2]> = None;
        for i in 0..=steps {
            let y = start + (end - start) * i as f64 / steps as f64;
            let x = x0 + centre + wobble * (std::f64::consts::TAU * y / period + phase).sin();
            let p = [x, y];
            if let Some(q) = prev {
                let s = y / h;
                let fade = smoothstep(end, end - 0.15 * (end - start), y);
                img.line(q, p, width, shade(mix(root, tip, 0.6 * s), f), 0.95 * fade);
            }
            prev = Some(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_paints_its_pixels_and_no_others() {
        let mut img = Image::new(16, 16);
        img.line([2.0, 8.0], [14.0, 8.0], 2.0, [1.0, 0.0, 0.0], 1.0);
        assert!(img.px[8 * 16 + 8][3] > 0.99);
        assert_eq!(img.px[2 * 16 + 8][3], 0.0);
    }

    #[test]
    fn strips_are_solid_at_the_root_and_clear_at_the_side_of_the_tip() {
        let mut img = Image::new(SIZE, SIZE);
        let c = Colours { root: "#201008".into(), tip: "#806040".into(), painted: "#201008".into(), variation: 0.2 };
        bake_strip(&mut img, 1, &c, 5);
        let x0 = CAP_PIXELS + STRIP_WIDTH;
        assert!(img.px[20 * SIZE + x0 + STRIP_WIDTH / 2][3] > 0.9);
        assert!(img.px[(SIZE - 2) * SIZE + x0 + 2][3] < 0.1);
    }
}
