//! The teeth: two strips, like KOTOR's own. KOTOR's upper teeth are one strip of 8 triangles on `head_g`, and its lower teeth one strip
//! on `f_jaw_g`. Each strip runs through 5 columns of 2 points, the top and the bottom of the teeth, from the left end of the arch to the
//! right. Our strips go through the same 10 points, placed on each head by the regressor, with more columns so that the arch is round.

use crate::{V3, add, scale, sub};

pub const COLUMNS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Upper,
    Lower,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub positions: Vec<V3>,
    pub uv: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
}

/// The names under which the regressor places a strip's points, top then bottom for each column, left to right.
pub fn point_names(row: Row) -> Vec<String> {
    let r = if row == Row::Upper { "upper" } else { "lower" };
    (0..COLUMNS)
        .flat_map(|i| ["top", "bottom"].map(|e| format!("teeth_{r}_{i}_{e}")))
        .collect()
}

fn catmull_rom(p0: V3, p1: V3, p2: V3, p3: V3, t: f64) -> V3 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mut out = [0.0; 3];
    for k in 0..3 {
        out[k] = 0.5
            * (2.0 * p1[k]
                + (p2[k] - p0[k]) * t
                + (2.0 * p0[k] - 5.0 * p1[k] + 4.0 * p2[k] - p3[k]) * t2
                + (3.0 * p1[k] - p0[k] - 3.0 * p2[k] + p3[k]) * t3);
    }
    out
}

/// A point along a line of control points at t from 0 (the first) to 1 (the last), through every control point.
fn along(line: &[V3], t: f64) -> V3 {
    let n = line.len();
    let x = t.clamp(0.0, 1.0) * (n - 1) as f64;
    let i = (x.floor() as usize).min(n - 2);
    let f = x - i as f64;
    let get = |j: isize| -> V3 {
        if j < 0 {
            add(line[0], sub(line[0], line[1]))
        } else if j as usize >= n {
            add(line[n - 1], sub(line[n - 1], line[n - 2]))
        } else {
            line[j as usize]
        }
    };
    catmull_rom(
        get(i as isize - 1),
        get(i as isize),
        get(i as isize + 1),
        get(i as isize + 2),
        f,
    )
}

/// A strip through the tops and bottoms of the teeth, with `columns` columns. The texture's v is 1 at the gum and 0 at the biting edge
/// on both strips, so one texture serves both. Like KOTOR's and OpenGL's, v = 1 is the top row of the image.
pub fn strip(tops: &[V3], bottoms: &[V3], row: Row, columns: usize) -> Mesh {
    let mut m = Mesh::default();
    for c in 0..columns {
        let t = c as f64 / (columns - 1) as f64;
        let (top, bottom) = (along(tops, t), along(bottoms, t));
        let (gum, edge) = if row == Row::Upper {
            (top, bottom)
        } else {
            (bottom, top)
        };
        m.positions.push(gum);
        m.uv.push([t, 1.0]);
        m.positions.push(edge);
        m.uv.push([t, 0.0]);
    }
    for c in 0..columns as u32 - 1 {
        let (a, b, d, e) = (2 * c, 2 * c + 1, 2 * c + 2, 2 * c + 3);
        m.triangles.push([a, b, d]);
        m.triangles.push([d, b, e]);
    }
    m
}

/// Moves a strip's points up or down so that the middle of its biting edge lies `offset` above `lip_edge` (heights along +Y). The upper
/// strip's biting edge is its bottom row and the lower strip's is its top row. KOTOR's upper teeth end 0.2 mm below its upper lip's edge,
/// and its lower teeth 0.9 mm above its lower lip's edge. Placing ours from our own lips keeps the same share of teeth in view.
pub fn to_lip_edge(tops: &mut [V3], bottoms: &mut [V3], row: Row, lip_edge: f64, offset: f64) {
    let mid = tops.len() / 2;
    let now = if row == Row::Upper {
        bottoms[mid][1]
    } else {
        tops[mid][1]
    };
    let d = lip_edge + offset - now;
    for p in tops.iter_mut().chain(bottoms.iter_mut()) {
        p[1] += d;
    }
}

/// The teeth texture, after KOTOR's: a thin, muted pink gum band at the top, then wide teeth with rounded tops and thin dark gaps,
/// darker toward the back of the mouth, where little light reaches. RGB, `w` by `h`, the gum in the first rows.
pub fn texture(w: usize, h: usize) -> Vec<u8> {
    const GUM: f64 = 0.17;
    const TEETH: usize = 10;
    let gum: V3 = [0.66, 0.46, 0.43];
    let gum_line: V3 = [0.42, 0.26, 0.25];
    let tooth: V3 = [0.92, 0.90, 0.84];
    let mut px = vec![0u8; w * h * 3];
    for y in 0..h {
        let v = (y as f64 + 0.5) / h as f64;
        for x in 0..w {
            let u = (x as f64 + 0.5) / w as f64;
            // the arch's angle, so the front teeth come out wider than the back ones on the strip
            let a = (2.0 * u - 1.0).clamp(-1.0, 1.0).asin() / std::f64::consts::FRAC_PI_2;
            let pos = (a * 0.5 + 0.5) * TEETH as f64;
            let f = pos - pos.floor();
            let top = GUM + 0.09 * (1.0 - (std::f64::consts::PI * f).sin().powf(0.5));
            let gap = 1.0 - smoothstep(0.0, 0.07, f.min(1.0 - f));
            let depth = 1.0 - (2.0 * u - 1.0).abs();
            let light = 0.30 + 0.70 * depth.powf(0.6);
            let colour: V3 = if v < top {
                let line = smoothstep(top - 0.05, top, v);
                [0, 1, 2].map(|k| gum[k] + (gum_line[k] - gum[k]) * line)
            } else {
                let shade = 1.0
                    - 0.6 * gap
                    - 0.12 * smoothstep(0.75, 1.0, v)
                    - 0.15 * (1.0 - smoothstep(top, top + 0.12, v));
                scale(tooth, shade)
            };
            let c = scale(colour, light);
            let o = 3 * (y * w + x);
            for k in 0..3 {
                px[o + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    }
    px
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arch(y: f64) -> Vec<V3> {
        (0..COLUMNS)
            .map(|i| {
                let x = -0.02 + 0.01 * i as f64;
                [x, y, -400.0 * x * x]
            })
            .collect()
    }

    #[test]
    fn the_strip_passes_through_every_control_point() {
        let (tops, bottoms) = (arch(0.012), arch(0.002));
        let m = strip(&tops, &bottoms, Row::Upper, 9);
        for (i, top) in tops.iter().enumerate() {
            let gum = m.positions[2 * (2 * i)];
            assert!(crate::length(sub(gum, *top)) < 1e-12, "column {i}");
        }
        assert_eq!(m.triangles.len(), 16);
    }

    #[test]
    fn the_lower_strip_puts_its_gum_at_the_bottom_and_the_image_top() {
        let m = strip(&arch(0.0), &arch(-0.01), Row::Lower, 5);
        assert_eq!(m.uv[0], [0.0, 1.0]);
        assert!(m.positions[0][1] < m.positions[1][1]);
    }

    #[test]
    fn the_biting_edge_lands_on_the_lip_edge_plus_the_offset() {
        let (mut tops, mut bottoms) = (arch(0.012), arch(0.002));
        to_lip_edge(&mut tops, &mut bottoms, Row::Upper, 0.0, -0.0002);
        assert!((bottoms[2][1] + 0.0002).abs() < 1e-12);
        assert!((tops[2][1] - 0.0098).abs() < 1e-12);
    }

    #[test]
    fn the_gum_is_at_the_top_of_the_image() {
        let (w, h) = (64, 32);
        let px = texture(w, h);
        let x = w / 2 + w / 20;
        let at = |y: usize| px[3 * (y * w + x)];
        let g = 3 * x;
        assert!(px[g] > px[g + 1] + 30, "the first row is pink");
        assert!(at(h - 4) > 200, "the lower rows are teeth");
    }

    #[test]
    fn the_back_teeth_are_darker_than_the_front() {
        let (w, h) = (64, 16);
        let px = texture(w, h);
        let at = |x: usize| px[3 * (12 * w + x)] as u32;
        assert!(at(1) < at(w / 2 + w / 20));
    }
}
