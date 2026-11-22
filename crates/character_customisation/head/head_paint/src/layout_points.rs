//! Points given in the texture layout, found on the head's surface (paint track, fine skin detail).
//!
//! The wrinkle curves and the pore regions are placed by points of the layout, written as pixels of the 2,048-pixel texture. The layout
//! does not change when the sliders move the vertices, so a point of the layout always lands on the same spot of the skin. Its 3D position
//! is read from the corners of the layout triangle that holds it, so it follows the head's shape.

/// The size of the texture in which the layout points are written.
pub const LAYOUT_PIXELS: f64 = 2048.0;

/// A point of the layout as a pixel of the 2,048-pixel texture: the column from the left and the row from the top.
pub type LayoutPixel = (f64, f64);

/// The layout coordinates (u, v) of a pixel of the 2,048-pixel texture.
pub fn uv_of(pixel: LayoutPixel) -> [f64; 2] {
    [pixel.0 / LAYOUT_PIXELS, 1.0 - pixel.1 / LAYOUT_PIXELS]
}

/// The column of the face's middle line in the 2,048-pixel layout. The layout is close to a mirror image about it.
pub const MIDDLE_COLUMN: f64 = 926.0;

/// The same pixel on the other side of the face.
pub fn mirrored(pixel: LayoutPixel) -> LayoutPixel {
    (2.0 * MIDDLE_COLUMN - pixel.0, pixel.1)
}

/// Finds layout points on the head.
pub struct LayoutLocator<'a> {
    uv: &'a [[f64; 2]],
    triangles: &'a [[u32; 3]],
    cells: Vec<Vec<u32>>,
}

const GRID: usize = 64;

/// A layout point found on the head: its position in millimetres and the unit normal of its triangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSpot {
    pub position_mm: [f64; 3],
    pub normal: [f64; 3],
}

impl<'a> LayoutLocator<'a> {
    /// `uv` holds the layout's vertices and `triangles` indexes them, as for the pixel map.
    pub fn new(uv: &'a [[f64; 2]], triangles: &'a [[u32; 3]]) -> Self {
        let mut cells = vec![Vec::new(); GRID * GRID];
        for (k, t) in triangles.iter().enumerate() {
            let c = t.map(|i| uv[i as usize]);
            let cell = |x: f64| ((x * GRID as f64).floor().max(0.0) as usize).min(GRID - 1);
            let (x0, x1) = (cell(c[0][0].min(c[1][0]).min(c[2][0])), cell(c[0][0].max(c[1][0]).max(c[2][0])));
            let (y0, y1) = (cell(c[0][1].min(c[1][1]).min(c[2][1])), cell(c[0][1].max(c[1][1]).max(c[2][1])));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    cells[y * GRID + x].push(k as u32);
                }
            }
        }
        LayoutLocator { uv, triangles, cells }
    }

    /// The triangle that holds a layout point and the point's barycentric weights in it. The lowest-numbered triangle wins on a shared edge.
    pub fn locate(&self, point: [f64; 2]) -> Option<(usize, [f64; 3])> {
        let x = ((point[0] * GRID as f64).floor().max(0.0) as usize).min(GRID - 1);
        let y = ((point[1] * GRID as f64).floor().max(0.0) as usize).min(GRID - 1);
        let mut best: Option<(usize, [f64; 3])> = None;
        for &k in &self.cells[y * GRID + x] {
            let [a, b, c] = self.triangles[k as usize].map(|i| self.uv[i as usize]);
            let den = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
            if den.abs() < 1e-15 {
                continue;
            }
            let w0 = ((b[1] - c[1]) * (point[0] - c[0]) + (c[0] - b[0]) * (point[1] - c[1])) / den;
            let w1 = ((c[1] - a[1]) * (point[0] - c[0]) + (a[0] - c[0]) * (point[1] - c[1])) / den;
            let w = [w0, w1, 1.0 - w0 - w1];
            if w.iter().all(|&v| v >= -1e-9) && best.is_none_or(|(b, _)| (k as usize) < b) {
                best = Some((k as usize, w));
            }
        }
        best
    }

    /// The spot of the head at a layout point. `positions` holds the head's positions in metres, one per layout vertex.
    pub fn spot(&self, point: [f64; 2], positions: &[[f64; 3]]) -> Option<SurfaceSpot> {
        let (k, w) = self.locate(point)?;
        let corners = self.triangles[k].map(|i| positions[i as usize]);
        let mut p = [0.0; 3];
        for (c, wi) in corners.iter().zip(w) {
            for a in 0..3 {
                p[a] += 1000.0 * wi * c[a];
            }
        }
        let e1 = sub(corners[1], corners[0]);
        let e2 = sub(corners[2], corners[0]);
        Some(SurfaceSpot { position_mm: p, normal: normalize(cross(e1, e2)) })
    }
}

pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn length(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

pub fn normalize(a: [f64; 3]) -> [f64; 3] {
    let l = length(a);
    if l > 0.0 { a.map(|x| x / l) } else { a }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_is_found_with_its_weights_and_position() {
        let uv = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let tris = [[0u32, 1, 2]];
        let locator = LayoutLocator::new(&uv, &tris);
        let positions = [[0.0, 0.0, 0.0], [0.010, 0.0, 0.0], [0.0, 0.020, 0.0]];
        let spot = locator.spot([0.25, 0.5], &positions).unwrap();
        assert!((spot.position_mm[0] - 2.5).abs() < 1e-9 && (spot.position_mm[1] - 10.0).abs() < 1e-9);
        assert!((spot.normal[2] - 1.0).abs() < 1e-9);
        assert!(locator.spot([0.9, 0.9], &positions).is_none());
    }

    #[test]
    fn a_pixel_and_its_mirror_lie_either_side_of_the_middle() {
        let p = (500.0, 800.0);
        let m = mirrored(p);
        assert_eq!(m.1, 800.0);
        assert!((p.0 + m.0) / 2.0 - MIDDLE_COLUMN == 0.0);
        assert_eq!(uv_of((0.0, 0.0)), [0.0, 1.0]);
    }
}
