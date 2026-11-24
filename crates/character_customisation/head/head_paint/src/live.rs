//! The texture layout that every head shares, and the rasteriser that walks its pixels. The GPU painter (`head_paint_gpu`) reads both.

use crate::pixel_map::Pixel;

/// The texture layout of our head, which every head shares.
pub struct Layout {
    pub uv: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
    /// The mesh vertex of each layout vertex.
    pub layout_to_mesh: Vec<usize>,
    /// The triangles of the head's mesh, which the shading reads.
    pub mesh_triangles: Vec<[u32; 3]>,
}

/// Calls `f` with every pixel of `PixelMap::new(uv, triangles, resolution)`, in the same order and with the same values, without
/// holding the whole map. It repeats the loops of `PixelMap::new`, and a test checks that both give the same pixels.
pub fn rasterise(uv: &[[f64; 2]], triangles: &[[u32; 3]], resolution: usize, mut f: impl FnMut(Pixel)) {
    let res = resolution as f64;
    let mut slot = vec![false; resolution * resolution];
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
                f(Pixel { x: px as u16, y: row as u16, triangle: k as u32, barycentric: [w0, w1, w2] });
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel_map::PixelMap;

    /// A bumpy sheet 200 mm wide whose layout covers the whole texture, so every layout point of the wrinkles and the pores lands on it.
    fn sheet() -> (Layout, Vec<[f64; 3]>) {
        // 4,900 vertices, because the sheen's landmarks are vertex numbers of our head of up to 4,322.
        let n = 70;
        let (mut uv, mut positions, mut triangles) = (Vec::new(), Vec::new(), Vec::new());
        for j in 0..n {
            for i in 0..n {
                let (u, v) = (i as f64 / (n - 1) as f64, j as f64 / (n - 1) as f64);
                uv.push([u, v]);
                let (x, y) = ((u - 0.5) * 0.2, (v - 0.5) * 0.2);
                positions.push([x, y, 0.008 * (x * 90.0).sin() * (y * 70.0).cos()]);
            }
        }
        for j in 0..n - 1 {
            for i in 0..n - 1 {
                let a = (j * n + i) as u32;
                let (b, c, d) = (a + 1, a + n as u32, a + n as u32 + 1);
                triangles.push([a, b, d]);
                triangles.push([a, d, c]);
            }
        }
        let layout_to_mesh = (0..uv.len()).collect();
        (Layout { uv, mesh_triangles: triangles.clone(), triangles, layout_to_mesh }, positions)
    }

    #[test]
    fn rasterise_gives_the_pixels_of_the_pixel_map() {
        let (layout, _) = sheet();
        for resolution in [64, 97, 128] {
            let mut pixels = Vec::new();
            rasterise(&layout.uv, &layout.triangles, resolution, |p| pixels.push(p));
            assert_eq!(pixels, PixelMap::new(&layout.uv, &layout.triangles, resolution).pixels);
        }
    }
}
