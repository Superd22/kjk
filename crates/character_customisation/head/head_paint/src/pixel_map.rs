//! The pixel map: for each pixel of a texture, the triangle of a layout that covers its centre and the pixel's place in that triangle.
//!
//! A layout gives every vertex of a mesh a point in the texture. Any per-vertex value of the mesh, such as the position of the head's surface,
//! can then be read at a pixel from the three corners of its triangle and the pixel's barycentric weights. The bake of KOTOR's paint uses the
//! position, and our own procedural paint will use it to compute its layers at each pixel's point on the head.

/// One pixel that a triangle of the layout covers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pixel {
    /// The column, counted from the left.
    pub x: u16,
    /// The row, counted from the top of the image. The layout's v axis points up, so v = 1 is row 0.
    pub y: u16,
    /// The triangle of the layout that covers the pixel's centre.
    pub triangle: u32,
    /// The weights of the triangle's three corners at the pixel's centre. They add up to 1.
    pub barycentric: [f64; 3],
}

/// Every pixel that the layout covers, each once.
pub struct PixelMap {
    pub resolution: usize,
    pub pixels: Vec<Pixel>,
}

impl PixelMap {
    /// Builds the map for a square texture of `resolution` pixels. `uv` holds one point per vertex, with u and v from 0 to 1, and `triangles`
    /// indexes it. A pixel whose centre lies on an edge shared by several triangles belongs to the one with the lowest index. A triangle
    /// with no area covers nothing.
    pub fn new(uv: &[[f64; 2]], triangles: &[[u32; 3]], resolution: usize) -> Self {
        let res = resolution as f64;
        let mut slot = vec![false; resolution * resolution];
        let mut pixels = Vec::new();
        for (k, t) in triangles.iter().enumerate() {
            let [a, b, c] = t.map(|i| uv[i as usize].map(|x| x * res));
            let den = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
            if den.abs() < 1e-12 {
                continue;
            }
            let lo = [a[0].min(b[0]).min(c[0]).floor().max(0.0) as usize, a[1].min(b[1]).min(c[1]).floor().max(0.0) as usize];
            let hi = [(a[0].max(b[0]).max(c[0]).ceil().max(0.0) as usize).min(resolution), (a[1].max(b[1]).max(c[1]).ceil().max(0.0) as usize).min(resolution)];
            for py in lo[1]..hi[1] {
                for px in lo[0]..hi[0] {
                    let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
                    let w0 = ((b[1] - c[1]) * (x - c[0]) + (c[0] - b[0]) * (y - c[1])) / den;
                    let w1 = ((c[1] - a[1]) * (x - c[0]) + (a[0] - c[0]) * (y - c[1])) / den;
                    let w2 = 1.0 - w0 - w1;
                    if w0 < -1e-6 || w1 < -1e-6 || w2 < -1e-6 {
                        continue;
                    }
                    let row = resolution - 1 - py;
                    if std::mem::replace(&mut slot[row * resolution + px], true) {
                        continue;
                    }
                    pixels.push(Pixel { x: px as u16, y: row as u16, triangle: k as u32, barycentric: [w0, w1, w2] });
                }
            }
        }
        PixelMap { resolution, pixels }
    }

    /// The value at each pixel's centre of a per-vertex quantity, read from the three corners of the pixel's triangle. `triangles` must be the
    /// list the map was built from, and `values` holds one entry per vertex of the layout.
    pub fn interpolate<const N: usize>(&self, triangles: &[[u32; 3]], values: &[[f64; N]]) -> Vec<[f64; N]> {
        self.pixels
            .iter()
            .map(|p| {
                let corners = triangles[p.triangle as usize].map(|i| values[i as usize]);
                let mut out = [0.0; N];
                for (corner, w) in corners.iter().zip(p.barycentric) {
                    for k in 0..N {
                        out[k] += w * corner[k];
                    }
                }
                out
            })
            .collect()
    }
}

/// The nearest pixel of a grid whose `good` is true, for every pixel, as an index into the grid. A grid with no good pixel gives `u32::MAX`.
/// It is an exact Euclidean distance transform: a pass down each column, then the lower envelope of parabolas along each row.
pub fn nearest_true(good: &[bool], width: usize, height: usize) -> Vec<u32> {
    // The nearest good pixel in the same column.
    let mut column = vec![u32::MAX; width * height];
    for x in 0..width {
        let mut last: Option<usize> = None;
        let mut seen = vec![usize::MAX; height];
        for y in 0..height {
            if good[y * width + x] {
                last = Some(y);
            }
            seen[y] = last.unwrap_or(usize::MAX);
        }
        last = None;
        for y in (0..height).rev() {
            if good[y * width + x] {
                last = Some(y);
            }
            let up = seen[y];
            let best = match (up != usize::MAX, last) {
                (true, Some(down)) => if y - up <= down - y { up } else { down },
                (true, None) => up,
                (false, Some(down)) => down,
                (false, None) => usize::MAX,
            };
            if best != usize::MAX {
                column[y * width + x] = best as u32;
            }
        }
    }
    // Along each row, the lower envelope of the parabolas that those columns' nearest pixels make.
    let mut out = vec![u32::MAX; width * height];
    let mut starts: Vec<f64> = Vec::new();
    let mut owners: Vec<usize> = Vec::new();
    let height_at = |y: usize, x: usize| -> f64 {
        let r = column[y * width + x];
        if r == u32::MAX { f64::INFINITY } else { (y as f64 - f64::from(r)).powi(2) }
    };
    for y in 0..height {
        starts.clear();
        owners.clear();
        for q in 0..width {
            let fq = height_at(y, q);
            if fq.is_infinite() {
                continue;
            }
            let s = loop {
                let Some(&p) = owners.last() else { break f64::NEG_INFINITY };
                let (fp, qf, pf) = (height_at(y, p), q as f64, p as f64);
                let s = ((fq + qf * qf) - (fp + pf * pf)) / (2.0 * qf - 2.0 * pf);
                if s <= *starts.last().unwrap_or(&f64::NEG_INFINITY) {
                    owners.pop();
                    starts.pop();
                } else {
                    break s;
                }
            };
            owners.push(q);
            starts.push(s);
        }
        if owners.is_empty() {
            continue;
        }
        let mut k = 0;
        for x in 0..width {
            while k + 1 < owners.len() && starts[k + 1] <= x as f64 {
                k += 1;
            }
            let c = owners[k];
            out[y * width + x] = column[y * width + c] * width as u32 + c as u32;
        }
    }
    out
}


/// Copies the paint of the nearest painted pixel into every unpainted pixel within `radius` pixels of it, so that a texture filtered across
/// the edge of a patch of the layout shows no line. `rgb` holds three bytes per pixel of a square texture of `resolution` pixels, and
/// `painted` is true where the layout put paint.
pub fn grow_over_seams(rgb: &mut [u8], painted: &[bool], resolution: usize, radius: f64) {
    let nearest = nearest_true(painted, resolution, resolution);
    for at in 0..resolution * resolution {
        let source = nearest[at];
        if painted[at] || source == u32::MAX {
            continue;
        }
        let s = source as usize;
        let (dy, dx) = ((at / resolution) as f64 - (s / resolution) as f64, (at % resolution) as f64 - (s % resolution) as f64);
        if (dx * dx + dy * dy).sqrt() <= radius {
            rgb.copy_within(s * 3..s * 3 + 3, at * 3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit square split along its diagonal, covering a 4 by 4 texture. Its 3D points are (u, v, 2u + 3v).
    type Square = (Vec<[f64; 2]>, Vec<[u32; 3]>, Vec<[f64; 3]>);

    fn square() -> Square {
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let position = uv.iter().map(|p| [p[0], p[1], 2.0 * p[0] + 3.0 * p[1]]).collect();
        (uv, vec![[0, 1, 2], [0, 2, 3]], position)
    }

    #[test]
    fn a_square_covers_every_pixel_once() {
        let (uv, tris, _) = square();
        let map = PixelMap::new(&uv, &tris, 4);
        assert_eq!(map.pixels.len(), 16);
        let mut seen = [false; 16];
        for p in &map.pixels {
            assert!(!std::mem::replace(&mut seen[p.y as usize * 4 + p.x as usize], true));
        }
    }

    #[test]
    fn a_pixel_on_the_shared_diagonal_goes_to_the_lower_triangle_index() {
        let (uv, tris, _) = square();
        let map = PixelMap::new(&uv, &tris, 4);
        let on_diagonal = map.pixels.iter().find(|p| p.x == 0 && p.y == 3).unwrap();
        assert_eq!(on_diagonal.triangle, 0);
        assert!(on_diagonal.barycentric.iter().all(|&w| w >= -1e-9));
    }

    #[test]
    fn row_zero_is_the_top_where_v_is_one() {
        let (uv, tris, position) = square();
        let map = PixelMap::new(&uv, &tris, 4);
        let points = map.interpolate(&tris, &position);
        for (p, q) in map.pixels.iter().zip(points) {
            let (u, v) = ((p.x as f64 + 0.5) / 4.0, 1.0 - (p.y as f64 + 0.5) / 4.0);
            assert!((q[0] - u).abs() < 1e-12 && (q[1] - v).abs() < 1e-12, "{p:?} gives {q:?}");
            assert!((q[2] - (2.0 * u + 3.0 * v)).abs() < 1e-12);
        }
    }

    #[test]
    fn pixels_outside_the_layout_are_not_in_the_map() {
        let uv = vec![[0.0, 0.0], [0.5, 0.0], [0.0, 0.5]];
        let map = PixelMap::new(&uv, &[[0, 1, 2]], 8);
        assert!(!map.pixels.is_empty() && map.pixels.len() < 16);
        assert!(map.pixels.iter().all(|p| p.x < 4 && p.y >= 4));
    }

    #[test]
    fn a_triangle_with_no_area_covers_nothing() {
        let uv = vec![[0.1, 0.1], [0.5, 0.5], [0.9, 0.9]];
        assert!(PixelMap::new(&uv, &[[0, 1, 2]], 8).pixels.is_empty());
    }

    #[test]
    fn a_vertex_value_is_read_back_at_a_pixel_next_to_it() {
        let (uv, tris, _) = square();
        let map = PixelMap::new(&uv, &tris, 64);
        let values: Vec<[f64; 1]> = vec![[10.0], [20.0], [30.0], [40.0]];
        let read = map.interpolate(&tris, &values);
        let corner = map.pixels.iter().position(|p| p.x == 63 && p.y == 63).unwrap();
        assert!((read[corner][0] - 20.0).abs() < 0.5);
    }

    #[test]
    fn the_nearest_true_pixel_is_found_in_both_directions() {
        let mut good = vec![false; 25];
        good[5 * 2 + 2] = true;
        good[0] = true;
        let near = nearest_true(&good, 5, 5);
        assert_eq!(near[1], 0);
        assert_eq!(near[5 * 4 + 4], 12);
        assert_eq!(near[5 * 2 + 3], 12);
        assert!(nearest_true(&[false; 4], 2, 2).iter().all(|&n| n == u32::MAX));
    }

    #[test]
    fn paint_grows_only_as_far_as_the_radius() {
        let mut rgb = vec![0u8; 3 * 64];
        let mut painted = vec![false; 64];
        painted[0] = true;
        rgb[..3].copy_from_slice(&[200, 100, 50]);
        grow_over_seams(&mut rgb, &painted, 8, 2.0);
        assert_eq!(&rgb[3 * 2..3 * 2 + 3], &[200, 100, 50]);
        assert_eq!(&rgb[3 * 9..3 * 9 + 3], &[200, 100, 50]);
        assert_eq!(&rgb[3 * 3..3 * 3 + 3], &[0, 0, 0]);
    }
}
