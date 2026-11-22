//! The paint grid: the hair grid's points pinned to the head (paint track, part C, decisions 60 and 63). Each point is bound to a fixed
//! spot of the layout, a triangle and the weights of its corners, so it lies on the same part of the head on every head and moves with
//! the sliders. The grid covers every place hair can grow: the scalp, the hairline, the temples, the sideburns, behind the ears, the
//! nape, the beard and the neck. It is one smooth chart of the skin, laid out as rigidly as it can be, so that its rows and columns run
//! without bends, about 2 mm apart on the face of the mean of the 57 test heads. The middle column runs up the middle of the face, and
//! the x < 0 half has the columns below it; the x > 0 half is its mirror image. Over the top of the head and down the back, the middle
//! line is the grid's seam, made of straight pieces with a turn on the front of the scalp and one on the back of the head. There each
//! half carries on 8 steps past the seam with copies of the other half's points (`FaceBinding::copy_of`), so that its rows and columns
//! run on. The lips, the nose, the eyelids and the ears are holes in it, because hair never grows there. Each point carries a zone (`ZONES`), a label that sets its default stubble and direction of growth. The
//! binding, the zones and a few marks (`MARKS`) were made once on the mean head by the tool `paint_grid` of `head_paint_gpu` and are
//! kept in `data/paint_grid.bin`.
//!
//! The grid is the address of the hair: the sliders' preset, the hair groups and the editor's values are written onto its points, and
//! the shader `hair_grid.wgsl` reads them there. A texel finds its place on the grid through `UvLookup`.

use crate::hair_grid::GridPoint;
use crate::layout_points::{cross, dot, normalize, sub};

const BAKED: &[u8] = include_bytes!("data/paint_grid.bin");

/// Where each point of a face grid lies on the layout.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceBinding {
    pub cols: usize,
    pub rows: usize,
    /// The row of the jaw's edge and the row of the mouth's opening, in the middle of the face.
    pub jaw_row: usize,
    pub mouth_row: usize,
    /// For each point, row by row from the bottom of the neck, its layout triangle and the weights of the triangle's first two corners,
    /// or `None` where no hair grows.
    pub points: Vec<Option<(u32, [f64; 2])>>,
    /// Places on the grid as a column and a row, by the numbers of `MARKS`.
    pub marks: Vec<[f64; 2]>,
    /// The zone of each point, a number of `ZONES`, or `NO_ZONE` where the point is not bound.
    pub zones: Vec<u8>,
    /// For each point past the grid's seam, the point of the other half that lies at the same place on the skin. Over the top of the
    /// head and down the back, each half carries on a few steps onto the other with such copies, so that its rows and columns run on
    /// past the seam. Empty, or one entry per point.
    pub copy_of: Vec<Option<usize>>,
}

/// The zones of the paint grid (decision 63). They are labels that set the default stubble and the default direction of growth, not
/// walls: a hair group paints over any of them. "bare" is skin where no hair grows by default, such as the forehead and the upper cheeks.
pub const ZONES: [&str; 12] = ["bare", "scalp", "hairline", "temple", "sideburn", "cheek", "moustache", "soul patch", "chin", "jaw", "neck", "nape"];
pub const NO_ZONE: u8 = 255;
/// The marks of the paint grid: the crown's whorl, the top and the bottom of the x < 0 sideburn, in front of the ear.
pub const MARKS: [&str; 3] = ["whorl", "sideburn top", "sideburn bottom"];

impl FaceBinding {
    /// The binding kept in the crate.
    pub fn baked() -> Self {
        Self::from_bytes(BAKED).expect("data/paint_grid.bin")
    }

    /// Four 32-bit words of columns, rows, jaw row and mouth row, then per point the triangle (all bits set for none) and two weights as
    /// 32-bit floats, then the marks, one zone byte per point and, from the next whole word, the seam's copies as a count and pairs of
    /// the copy and its point, all little-endian.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        let word = |k: usize| b.get(4 * k..4 * k + 4).map(|w| u32::from_le_bytes(w.try_into().expect("4")));
        let (cols, rows) = (word(0)? as usize, word(1)? as usize);
        let points = (0..cols * rows)
            .map(|k| {
                let t = word(4 + 3 * k)?;
                let w = [word(5 + 3 * k)?, word(6 + 3 * k)?].map(|x| f32::from_bits(x) as f64);
                Some((t != u32::MAX).then_some((t, w)))
            })
            .collect::<Option<Vec<_>>>()?;
        let after = 4 + 3 * cols * rows;
        let marks = match word(after) {
            Some(n) => (0..n as usize).map(|k| Some([word(after + 1 + 2 * k)?, word(after + 2 + 2 * k)?].map(|x| f32::from_bits(x) as f64))).collect::<Option<Vec<_>>>()?,
            None => Vec::new(),
        };
        let zones_at = 4 * (after + 1 + 2 * marks.len());
        let zones = match b.get(zones_at..zones_at + cols * rows) {
            Some(z) => z.to_vec(),
            None => points.iter().map(|p| if p.is_some() { 0 } else { NO_ZONE }).collect(),
        };
        let copies_at = (zones_at + cols * rows).div_ceil(4);
        let mut copy_of = Vec::new();
        if let Some(n) = word(copies_at) {
            copy_of = vec![None; cols * rows];
            for c in 0..n as usize {
                let (k, from) = (word(copies_at + 1 + 2 * c)? as usize, word(copies_at + 2 + 2 * c)? as usize);
                *copy_of.get_mut(k)? = Some(from);
            }
        }
        Some(FaceBinding { cols, rows, jaw_row: word(2)? as usize, mouth_row: word(3)? as usize, points, marks, zones, copy_of })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for w in [self.cols, self.rows, self.jaw_row, self.mouth_row] {
            out.extend((w as u32).to_le_bytes());
        }
        for p in &self.points {
            let (t, w) = p.unwrap_or((u32::MAX, [0.0; 2]));
            out.extend(t.to_le_bytes());
            out.extend((w[0] as f32).to_le_bytes());
            out.extend((w[1] as f32).to_le_bytes());
        }
        out.extend((self.marks.len() as u32).to_le_bytes());
        for m in &self.marks {
            out.extend((m[0] as f32).to_le_bytes());
            out.extend((m[1] as f32).to_le_bytes());
        }
        out.extend(&self.zones);
        let copies: Vec<(usize, usize)> = self.copy_of.iter().enumerate().filter_map(|(k, c)| Some((k, (*c)?))).collect();
        if !copies.is_empty() {
            out.resize(out.len().div_ceil(4) * 4, 0);
            out.extend((copies.len() as u32).to_le_bytes());
            for (k, from) in copies {
                out.extend((k as u32).to_le_bytes());
                out.extend((from as u32).to_le_bytes());
            }
        }
        out
    }

    /// Whether point `k` is a copy of the other half's point past the seam.
    pub fn is_copy(&self, k: usize) -> bool {
        self.copy_of.get(k).is_some_and(|c| c.is_some())
    }
}

/// A face grid laid on one head.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceGrid {
    pub cols: usize,
    pub rows: usize,
    pub jaw_row: usize,
    pub mouth_row: usize,
    /// The face's directions, in which the editor gives a direction of growth.
    pub across: [f64; 3],
    pub up: [f64; 3],
    pub forward: [f64; 3],
    /// The points row by row from the bottom of the neck, `cols` per row, in millimetres.
    pub points: Vec<GridPoint>,
    /// For each point past the seam, the other half's point at the same place, as `FaceBinding::copy_of` holds it.
    pub copy_of: Vec<Option<usize>>,
    /// For a point that is not bound but lies within `RING` points of bound ones, the bound point next to it. Its place is carried on
    /// from the points before it, so that the shader can measure the skin between the last bound points and an edge such as the lips.
    pub source: Vec<Option<usize>>,
    /// For each point, how many points past the bound ones its place was carried on, or 0 for a bound point.
    pub depth: Vec<u8>,
}

/// How many points past the bound ones the grid's places are carried on.
pub const RING: usize = 3;
/// How many points past the bound ones a grid carries its places on when a hair group lets its hairs hang further over the lips
/// (`hair_groups` slider `over_lips`).
pub const LONG_RING: usize = 5;

impl FaceGrid {
    /// The grid on a head from its mesh positions in metres and the layout. `inside` is a point inside the head, toward which no normal
    /// points.
    #[allow(clippy::too_many_arguments)]
    pub fn new(binding: &FaceBinding, frame: [[f64; 3]; 3], inside: [f64; 3], uv: &[[f64; 2]], layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize], positions: &[[f64; 3]]) -> Self {
        Self::with_ring(binding, frame, inside, uv, layout_triangles, layout_to_mesh, positions, RING)
    }

    /// The same, with the places carried on `ring` points past the bound ones.
    #[allow(clippy::too_many_arguments)]
    pub fn with_ring(binding: &FaceBinding, frame: [[f64; 3]; 3], inside: [f64; 3], uv: &[[f64; 2]], layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize], positions: &[[f64; 3]], ring: usize) -> Self {
        let mm = |i: u32| positions[layout_to_mesh[i as usize]].map(|x| 1000.0 * x);
        let mut mesh_normal = vec![[0.0; 3]; positions.len()];
        for t in layout_triangles {
            let v = t.map(mm);
            let n = cross(sub(v[1], v[0]), sub(v[2], v[0]));
            for &i in t {
                let m = layout_to_mesh[i as usize];
                for k in 0..3 {
                    mesh_normal[m][k] += n[k];
                }
            }
        }
        let points = binding
            .points
            .iter()
            .map(|b| {
                let Some((t, w)) = *b else { return GridPoint { valid: false, position: inside, normal: frame[2], mm_per_texel: 0.1 } };
                let tri = layout_triangles[t as usize];
                let w = [w[0], w[1], 1.0 - w[0] - w[1]];
                let v = tri.map(mm);
                let position = [0, 1, 2].map(|k| w[0] * v[0][k] + w[1] * v[1][k] + w[2] * v[2][k]);
                let mut n = [0.0; 3];
                for (c, &i) in tri.iter().enumerate() {
                    let mn = mesh_normal[layout_to_mesh[i as usize]];
                    for k in 0..3 {
                        n[k] += w[c] * mn[k];
                    }
                }
                let n = normalize(n);
                let n = if dot(n, sub(position, inside)) < 0.0 { n.map(|x| -x) } else { n };
                let face = cross(sub(v[1], v[0]), sub(v[2], v[0]));
                let area_mm = 0.5 * dot(face, face).sqrt();
                let q = tri.map(|i| uv[i as usize]);
                let area_uv = 0.5 * ((q[1][0] - q[0][0]) * (q[2][1] - q[0][1]) - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1])).abs() * 2048.0 * 2048.0;
                GridPoint { valid: true, position, normal: n, mm_per_texel: if area_uv > 1e-9 { (area_mm / area_uv).sqrt() } else { 1.0 } }
            })
            .collect();
        let mut grid = FaceGrid { cols: binding.cols, rows: binding.rows, jaw_row: binding.jaw_row, mouth_row: binding.mouth_row, across: frame[0], up: frame[1], forward: frame[2], points, copy_of: binding.copy_of.clone(), source: Vec::new(), depth: Vec::new() };
        grid.carry_on(ring);
        grid
    }

    /// Carries the places of the bound points on past them, `RING` points deep. A point takes the mean of `2 a - b` over each direction
    /// where its neighbour `a` and the next point `b` beyond it are known. Copies past the seam carry nothing on: beyond them, across the
    /// middle column, lie the other half's copies, which are the skin on this half's side.
    fn carry_on(&mut self, ring: usize) {
        let (c, r) = (self.cols as i64, self.rows as i64);
        let mut known: Vec<bool> = self.points.iter().enumerate().map(|(k, p)| p.valid && !self.is_copy(k)).collect();
        self.source = vec![None; self.points.len()];
        self.depth = vec![0; self.points.len()];
        for step in 0..ring {
            let before = known.clone();
            let mut next = self.points.clone();
            for j in 0..r {
                for i in 0..c {
                    let k = (j * c + i) as usize;
                    if before[k] || self.points[k].valid {
                        continue;
                    }
                    let id = |a: i64, b: i64| (a >= 0 && b >= 0 && a < c && b < r && before[(b * c + a) as usize]).then_some((b * c + a) as usize);
                    let mut sum = [0.0; 3];
                    let mut count = 0.0;
                    let mut from = None;
                    for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        if let (Some(a), Some(b)) = (id(i + di, j + dj), id(i + 2 * di, j + 2 * dj)) {
                            let (pa, pb) = (self.points[a].position, self.points[b].position);
                            sum = [0, 1, 2].map(|x| sum[x] + 2.0 * pa[x] - pb[x]);
                            count += 1.0;
                            from = from.or(Some(a));
                        }
                    }
                    let Some(a) = from else { continue };
                    next[k] = GridPoint { valid: false, position: sum.map(|x| x / count), normal: self.points[a].normal, mm_per_texel: self.points[a].mm_per_texel };
                    self.source[k] = Some(self.source[a].unwrap_or(a));
                    self.depth[k] = step as u8 + 1;
                    known[k] = true;
                }
            }
            self.points = next;
        }
    }

    /// Whether point `k` is a copy of the other half's point past the seam.
    pub fn is_copy(&self, k: usize) -> bool {
        self.copy_of.get(k).is_some_and(|c| c.is_some())
    }

    /// Gives each copy past the seam the value of its point, so that the two halves agree on the skin they share.
    pub fn share<T: Copy>(&self, values: &mut [T]) {
        for (k, c) in self.copy_of.iter().enumerate() {
            if let Some(from) = *c {
                values[k] = values[from];
            }
        }
    }

    /// The skin's area of one grid cell at each point, in square millimetres, from the steps to its neighbours along the columns and the
    /// rows. The shader keeps a hair's root with a chance in proportion to it, so that the hairs per square millimetre do not change
    /// where the grid is denser or sparser.
    pub fn cell_area_mm2(&self) -> Vec<f64> {
        let mut out: Vec<f64> = (0..self.points.len())
            .map(|k| {
                if !self.known(k) {
                    return 0.0;
                }
                let n = self.points[k].normal;
                let (du, dv) = (self.step(k, 1, 0), self.step(k, 0, 1));
                let c = cross(du, dv);
                dot(c, n).abs()
            })
            .collect();
        self.share(&mut out);
        out
    }

    /// The skin's step at point `k` to the next point along `(di, dj)`, as a central difference where both neighbours are known.
    fn step(&self, k: usize, di: i64, dj: i64) -> [f64; 3] {
        let (i, j) = ((k % self.cols) as i64, (k / self.cols) as i64);
        let at = |a: i64, b: i64| (a >= 0 && b >= 0 && a < self.cols as i64 && b < self.rows as i64 && self.known((b * self.cols as i64 + a) as usize)).then(|| self.points[(b * self.cols as i64 + a) as usize].position);
        match (at(i + di, j + dj), at(i - di, j - dj)) {
            (Some(a), Some(b)) => sub(a, b).map(|x| 0.5 * x),
            (Some(a), None) => sub(a, self.points[k].position),
            (None, Some(b)) => sub(self.points[k].position, b),
            (None, None) => [0.0; 3],
        }
    }

    /// The point of column `i` and row `j`.
    pub fn at(&self, i: usize, j: usize) -> &GridPoint {
        &self.points[j * self.cols + i]
    }

    /// The distances in millimetres between neighbouring valid points: the shortest, the median and the longest.
    pub fn spacing_mm(&self) -> [f64; 3] {
        let mut d = Vec::new();
        for j in 0..self.rows {
            for i in 0..self.cols {
                let a = self.at(i, j);
                for (di, dj) in [(1, 0), (0, 1)] {
                    if a.valid && i + di < self.cols && j + dj < self.rows && self.at(i + di, j + dj).valid {
                        let e = sub(self.at(i + di, j + dj).position, a.position);
                        d.push(dot(e, e).sqrt());
                    }
                }
            }
        }
        if d.is_empty() {
            return [0.0; 3];
        }
        d.sort_by(f64::total_cmp);
        [d[0], d[d.len() / 2], d[d.len() - 1]]
    }

    /// The largest value within `reach` points of each point, along both directions of the grid.
    pub fn near(&self, values: &[f64], reach: usize) -> Vec<f64> {
        let (c, r) = (self.cols, self.rows);
        let mut across = vec![0.0f64; values.len()];
        for j in 0..r {
            for i in 0..c {
                across[j * c + i] = (i.saturating_sub(reach)..=(i + reach).min(c - 1)).map(|a| values[j * c + a]).fold(0.0, f64::max);
            }
        }
        let mut out = vec![0.0f64; values.len()];
        for j in 0..r {
            for i in 0..c {
                out[j * c + i] = (j.saturating_sub(reach)..=(j + reach).min(r - 1)).map(|b| across[b * c + i]).fold(0.0, f64::max);
            }
        }
        out
    }

    /// 1 when the cross product of the steps along the columns and along the rows points out of the skin, and -1 when it points in, as
    /// most of the bound points say.
    pub fn normal_sign(&self) -> f64 {
        let mut sum = 0.0;
        for j in 1..self.rows.saturating_sub(1) {
            for i in 1..self.cols.saturating_sub(1) {
                let k = j * self.cols + i;
                let ends = [k - 1, k + 1, k - self.cols, k + self.cols];
                if self.depth.get(k).copied().unwrap_or(0) > 0 || !self.points[k].valid || ends.iter().any(|&e| !self.points[e].valid) {
                    continue;
                }
                let du = sub(self.points[k + 1].position, self.points[k - 1].position);
                let dv = sub(self.points[k + self.cols].position, self.points[k - self.cols].position);
                sum += dot(cross(du, dv), self.points[k].normal).signum();
            }
        }
        if sum < 0.0 { -1.0 } else { 1.0 }
    }

    /// The face's down direction laid onto the skin at point `k`. Its length is 1 where the skin is upright, and shrinks to 0 where the
    /// skin faces the ground or the sky.
    pub fn downhill(&self, k: usize) -> [f64; 3] {
        let n = self.points[k].normal;
        let down = self.up.map(|x| -x);
        sub(down, n.map(|x| x * dot(down, n)))
    }

    /// Whether the point has a place on the skin, bound or carried on.
    pub fn known(&self, k: usize) -> bool {
        self.points[k].valid || self.source[k].is_some()
    }

    /// The direction on the skin at point `k` of a step of `t` columns and rows, of unit length, or zero.
    pub fn along(&self, k: usize, t: [f64; 2]) -> [f64; 3] {
        let (du, dv) = (self.step(k, 1, 0), self.step(k, 0, 1));
        let d = [0, 1, 2].map(|x| t[0] * du[x] + t[1] * dv[x]);
        let n = self.points[k].normal;
        let d = sub(d, n.map(|x| x * dot(d, n)));
        if dot(d, d) > 1e-12 { normalize(d) } else { [0.0; 3] }
    }

    /// The place and normal on the skin at a column and row given as fractions, read bilinearly from the known points round it, or
    /// `None` where none is known.
    pub fn place(&self, i: f64, j: f64) -> Option<([f64; 3], [f64; 3])> {
        let (i0, j0) = (i.floor() as i64, j.floor() as i64);
        let f = [i - i0 as f64, j - j0 as f64];
        let mut p = [0.0; 3];
        let mut n = [0.0; 3];
        let mut total = 0.0;
        for (di, dj) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let (a, b) = (i0 + di, j0 + dj);
            if a < 0 || b < 0 || a >= self.cols as i64 || b >= self.rows as i64 {
                continue;
            }
            let k = (b * self.cols as i64 + a) as usize;
            if !self.known(k) {
                continue;
            }
            let w = (if di == 1 { f[0] } else { 1.0 - f[0] }) * (if dj == 1 { f[1] } else { 1.0 - f[1] });
            for x in 0..3 {
                p[x] += w * self.points[k].position[x];
                n[x] += w * self.points[k].normal[x];
            }
            total += w;
        }
        (total > 1e-6).then(|| (p.map(|x| x / total), normalize(n)))
    }

    /// The bound grid point under a ray from `origin` along `dir`, both in millimetres: of the points within 3 mm of the ray whose skin
    /// faces it, the nearest to the ray among those within 6 mm of the first one along it. Returns its column and row. A copy past the
    /// seam is never picked, because its own point lies at the same place.
    pub fn snap(&self, origin: [f64; 3], dir: [f64; 3]) -> Option<(usize, usize)> {
        let d = normalize(dir);
        let hits: Vec<(usize, f64, f64)> = self
            .points
            .iter()
            .enumerate()
            .filter(|(k, p)| p.valid && !self.is_copy(*k) && dot(p.normal, d) < 0.2)
            .filter_map(|(k, p)| {
                let r = sub(p.position, origin);
                let t = dot(r, d);
                let off = sub(r, d.map(|x| x * t));
                let dist = dot(off, off).sqrt();
                (dist < 3.0).then_some((k, t, dist))
            })
            .collect();
        let first = hits.iter().map(|h| h.1).fold(f64::MAX, f64::min);
        let k = hits.iter().filter(|h| h.1 < first + 6.0).min_by(|a, b| a.2.total_cmp(&b.2))?.0;
        Some((k % self.cols, k / self.cols))
    }

    /// The grid, a field on it, the layer's looks and the lookup as 32-bit floats for the shader `hair_grid.wgsl`, starting at row
    /// `offset` of the buffer they go into. A header of two rows: the columns, the rows, the first row of the points and the first row
    /// of the lookup, then the lookup's size, the first row of the looks, their number and the farthest any look's locks and strands reach sideways, in mm. Then four rows per point: its place and the
    /// density, the direction and the length, the edge distance, the shadow, the texture's size and the largest density within reach
    /// of the longest hair, at least two points, and the number of its look with the longest hair within the same reach, the area of skin of one grid cell in square millimetres and how many points past the bound ones its place was carried on. Then nine rows per look, the look's linear colour and its
    /// L*, then the hairs' thickness in mm, the tone variation, the share of grey hairs and the curl, then the sheen, the random pattern, how far the hairs follow the flow and the locks' wave, then the locks' size in mm, gathering, highlight and softness, then the strands' width in mm, straightness, shading and bands, then the strands' light at the root, their layering, packing and shading by lock, then how parallel they run, their fine lines, their feathered edges and how far the hairs hang over the lips, then the clumps' width in mm, their wave, the light zones and the mixed edges, then the combed streaks, their length in mm, their sharpness and a 0, and last the lookup's cells (`UvLookup::words`). A point with no place on the skin sits at the first known point.
    pub fn words(&self, field: &crate::hair_grid::HairField, looks: &[[f64; 36]], lookup: &UvLookup, offset: usize) -> Vec<f32> {
        let n = self.points.len();
        let points_row = offset + 2;
        let looks_row = points_row + 4 * n;
        let lookup_row = looks_row + 9 * looks.len();
        let mut w: Vec<f32> = Vec::with_capacity(4 * (lookup_row - offset) + lookup.size * lookup.size * 2);
        let mut row = |v: [f64; 4]| w.extend(v.map(|x| x as f32));
        row([self.cols as f64, self.rows as f64, points_row as f64, lookup_row as f64]);
        let lock_reach = looks.iter().map(|l| if l[12] > 0.0 { 0.75 * l[12] * l[13] + 1.2 * l[11] } else { 0.0 }).fold(0.0, f64::max)
            + looks.iter().map(|l| 1.3 * l[16] + if l[28] > 0.0 { 0.8 * l[29] } else { 0.0 }).fold(0.0, f64::max);
        row([lookup.size as f64, looks_row as f64, looks.len() as f64, lock_reach]);
        // A pixel draws hairs only where some point within this reach has hair. A hair is at most 1.4 times its typical length, and the
        // points lie about 2 mm apart, so the reach grows with the longest hair and stays 2 points for hairs up to 2 mm long. Without
        // it, long hairs that hang over bare skin stop at a hard line.
        let longest = field.length_mm.iter().cloned().fold(0.0, f64::max);
        let reach = 2usize.max((1.96 * longest / 2.0).ceil() as usize);
        let mut near = self.near(&field.density, reach);
        let mut long = self.near(&field.length_mm, reach);
        self.share(&mut near);
        self.share(&mut long);
        let area = self.cell_area_mm2();
        let fallback = self.points.iter().find(|p| p.valid).map_or([0.0; 3], |p| p.position);
        for (k, p) in self.points.iter().enumerate() {
            let at = if self.known(k) { p.position } else { fallback };
            let d = field.direction[k];
            row([at[0], at[1], at[2], field.density[k]]);
            row([d[0], d[1], d[2], field.length_mm[k]]);
            row([field.edge_mm[k], field.shadow[k], p.mm_per_texel, near[k]]);
            row([f64::from(field.look[k]), long[k], area[k], f64::from(self.depth.get(k).copied().unwrap_or(0))]);
        }
        for l in looks {
            row([l[0], l[1], l[2], l[3]]);
            row([l[4], l[5], l[6], l[7]]);
            row([l[8], l[9], l[10], l[11]]);
            row([l[12], l[13], l[14], l[15]]);
            row([l[16], l[17], l[18], l[19]]);
            row([l[20], l[21], l[22], l[23]]);
            row([l[24], l[25], l[26], l[27]]);
            row([l[28], l[29], l[30], l[31]]);
            row([l[32], l[33], l[34], l[35]]);
        }
        w.extend(lookup.words());
        w
    }
}

/// Two places on the grid agree when they lie within this many columns and this many rows of each other. Places that disagree come from
/// the two halves of the grid along the seam over the top and down the back, or from two pieces of the layout that touch.
pub const AGREE_STEPS: f64 = 6.0;

/// The weighted mean of the largest group, by weight, of weighted guesses that agree, or `None` without guesses. Guesses that disagree
/// are never mixed, because their mean is a place on neither side.
fn agreeing_mean(guesses: &[(f64, [f64; 2])]) -> Option<[f64; 2]> {
    let agree = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() <= AGREE_STEPS && (a[1] - b[1]).abs() <= AGREE_STEPS;
    let pick = guesses.iter().max_by(|x, y| {
        let score = |c: &(f64, [f64; 2])| guesses.iter().filter(|o| agree(o.1, c.1)).map(|o| o.0).sum::<f64>();
        score(x).total_cmp(&score(y))
    })?;
    let group: Vec<&(f64, [f64; 2])> = guesses.iter().filter(|o| agree(o.1, pick.1)).collect();
    let w: f64 = group.iter().map(|o| o.0).sum();
    (w > 0.0).then(|| [group.iter().map(|o| o.0 * o.1[0]).sum::<f64>() / w, group.iter().map(|o| o.0 * o.1[1]).sum::<f64>() / w])
}

/// A face grid quad whose corners lie further apart than this on the layout, in texels of a 2,048-pixel texture, spans a seam of the
/// layout, such as the one down the middle of the neck, and is not laid on the lookup.
const SEAM_TEXELS: f64 = 64.0;

/// Where the face grid lies on the layout: for each cell of a square of `size` by `size` cells over the texture, the face grid's column
/// and row there, as fractions, or `None` where the grid does not reach. The shader finds a texel's place on the grid from it. It
/// depends on the binding and the layout only, so it is the same on every head.
#[derive(Clone, Debug, PartialEq)]
pub struct UvLookup {
    pub size: usize,
    /// Row by row from v = 0, `size` cells per row.
    pub cells: Vec<Option<[f32; 2]>>,
}

/// Carries the places on the layout of the bound points of `binding` on past them, `ring` points deep, as `FaceGrid::carry_on` carries
/// their places on the skin: a point takes the mean of `2 a - b` over each direction where its neighbour `a` and the next point `b`
/// beyond it are known. Copies past the seam carry nothing on. Returns how many points past the bound ones each point was carried, or
/// 0 for a point that was not.
fn carry_layout_places(at: &mut [Option<[f64; 2]>], binding: &FaceBinding, ring: usize) -> Vec<u8> {
    let (c, r) = (binding.cols as i64, binding.rows as i64);
    let mut carried = vec![0u8; at.len()];
    let mut known: Vec<bool> = (0..at.len()).map(|k| at[k].is_some() && !binding.is_copy(k)).collect();
    for step in 0..ring {
        let before = known.clone();
        let mut next = at.to_vec();
        for j in 0..r {
            for i in 0..c {
                let k = (j * c + i) as usize;
                if before[k] || at[k].is_some() {
                    continue;
                }
                let id = |a: i64, b: i64| (a >= 0 && b >= 0 && a < c && b < r && before[(b * c + a) as usize]).then_some((b * c + a) as usize);
                let mut sum = [0.0; 2];
                let mut count = 0.0;
                for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if let (Some(a), Some(b)) = (id(i + di, j + dj), id(i + 2 * di, j + 2 * dj)) {
                        let (pa, pb) = (at[a].expect("known"), at[b].expect("known"));
                        sum = [0, 1].map(|x| sum[x] + 2.0 * pa[x] - pb[x]);
                        count += 1.0;
                    }
                }
                if count > 0.0 {
                    next[k] = Some(sum.map(|x| x / count));
                    carried[k] = step as u8 + 1;
                    known[k] = true;
                }
            }
        }
        at.copy_from_slice(&next);
    }
    carried
}

impl UvLookup {
    /// Lays each square of four bound points on the layout as two triangles, and spreads the grid's coordinates `spread` cells further
    /// past the edges it reaches, so that a texel between the last points and an edge, such as the lips' border or a seam, still finds
    /// its place. The squares of each half's own points are laid first. A square that holds copies past the seam only fills the cells
    /// left empty, which is the strip along the middle line between the two halves' last own points. Last, the places of the bound
    /// points on the layout are carried on `LAYOUT_RING` points past them, as `FaceGrid` carries their places on the skin, and the squares
    /// that hold such points fill the cells still empty. A texel on the lips then finds its place, so that a moustache can hang over the
    /// upper lip.
    pub fn new(binding: &FaceBinding, uv: &[[f64; 2]], layout_triangles: &[[u32; 3]], size: usize, spread: usize) -> Self {
        Self::with_ring(binding, uv, layout_triangles, size, spread, RING)
    }

    /// The same, with the places on the layout carried on `ring` points past the bound ones. The squares that hold points carried
    /// further than `RING` fill the cells left empty last, so that the cells of a lookup carried `RING` deep stay as they are.
    pub fn with_ring(binding: &FaceBinding, uv: &[[f64; 2]], layout_triangles: &[[u32; 3]], size: usize, spread: usize, ring: usize) -> Self {
        let at: Vec<Option<[f64; 2]>> = binding
            .points
            .iter()
            .map(|p| {
                p.map(|(t, w)| {
                    let tri = layout_triangles[t as usize];
                    let w = [w[0], w[1], 1.0 - w[0] - w[1]];
                    [0, 1].map(|k| (0..3).map(|c| w[c] * uv[tri[c] as usize][k]).sum::<f64>() * size as f64 - 0.5)
                })
            })
            .collect();
        let cols = binding.cols;
        let mut at = at;
        let carried = carry_layout_places(&mut at, binding, ring);
        let seam = SEAM_TEXELS * size as f64 / 2048.0;
        let mut cells: Vec<Option<[f32; 2]>> = vec![None; size * size];
        // The bound squares are laid first, then spread past their edges, and the squares with carried points fill what is left, those
        // with points carried further than `RING` last.
        for pass in [0, 1, 3, 2, 4] {
            if pass == 3 {
                // Past an edge a cell takes the coordinates carried on from the two cells before it, along each direction where both are known.
                let n = size as i64;
                for _ in 0..spread {
                    let before = cells.clone();
                    for y in 0..n {
                        for x in 0..n {
                            if before[(y * n + x) as usize].is_some() {
                                continue;
                            }
                            let get = |a: i64, b: i64| (a >= 0 && b >= 0 && a < n && b < n).then(|| before[(b * n + a) as usize]).flatten();
                            let mut guesses: Vec<(f64, [f64; 2])> = Vec::new();
                            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                                if let (Some(p1), Some(p2)) = (get(x + dx, y + dy), get(x + 2 * dx, y + 2 * dy)) {
                                    guesses.push((1.0, [2.0 * p1[0] as f64 - p2[0] as f64, 2.0 * p1[1] as f64 - p2[1] as f64]));
                                }
                            }
                            if let Some(c) = agreeing_mean(&guesses) {
                                cells[(y * n + x) as usize] = Some(c.map(|v| v as f32));
                            }
                        }
                    }
                }
                continue;
            }
            let copies = pass > 0;
            for j in 0..binding.rows.saturating_sub(1) {
                for i in 0..cols.saturating_sub(1) {
                    let q = [(i, j), (i + 1, j), (i, j + 1), (i + 1, j + 1)];
                    let has_copy = q.iter().any(|&(a, b)| binding.is_copy(b * cols + a));
                    let has_carried = q.iter().any(|&(a, b)| carried[b * cols + a] > 0);
                    let deep = q.iter().any(|&(a, b)| carried[b * cols + a] as usize > RING);
                    if (pass == 0 && (has_copy || has_carried)) || (pass == 1 && (!has_copy || has_carried)) || (pass == 2 && (has_copy || !has_carried || deep)) || (pass == 4 && (has_copy || !deep)) {
                        continue;
                    }
                    let Some(c) = q.iter().map(|&(a, b)| at[b * cols + a]).collect::<Option<Vec<_>>>() else { continue };
                    if (0..4).any(|x| (0..4).any(|y| (c[x][0] - c[y][0]).hypot(c[x][1] - c[y][1]) > seam)) {
                        continue;
                    }
                    let g = q.map(|(a, b)| [a as f64, b as f64]);
                    for t in [[0, 1, 3], [0, 3, 2]] {
                        let (p, v) = (t.map(|k| c[k]), t.map(|k| g[k]));
                        let den = (p[1][1] - p[2][1]) * (p[0][0] - p[2][0]) + (p[2][0] - p[1][0]) * (p[0][1] - p[2][1]);
                        if den.abs() < 1e-12 {
                            continue;
                        }
                        let lo = [0, 1].map(|k| p[0][k].min(p[1][k]).min(p[2][k]).ceil().max(0.0) as usize);
                        let hi = [0, 1].map(|k| p[0][k].max(p[1][k]).max(p[2][k]).floor().min(size as f64 - 1.0));
                        if hi[0] < 0.0 || hi[1] < 0.0 {
                            continue;
                        }
                        for y in lo[1]..=hi[1] as usize {
                            for x in lo[0]..=hi[0] as usize {
                                let (qx, qy) = (x as f64, y as f64);
                                let w0 = ((p[1][1] - p[2][1]) * (qx - p[2][0]) + (p[2][0] - p[1][0]) * (qy - p[2][1])) / den;
                                let w1 = ((p[2][1] - p[0][1]) * (qx - p[2][0]) + (p[0][0] - p[2][0]) * (qy - p[2][1])) / den;
                                let w = [w0, w1, 1.0 - w0 - w1];
                                if w.iter().any(|&e| e < -1e-9) || (copies && cells[y * size + x].is_some()) {
                                    continue;
                                }
                                cells[y * size + x] = Some([0, 1].map(|k| (w[0] * v[0][k] + w[1] * v[1][k] + w[2] * v[2][k]) as f32));
                            }
                        }
                    }
                }
            }
        }
        UvLookup { size, cells }
    }

    /// The face grid's column and row at a point of the layout, read bilinearly from the cells the grid reaches, as the shader reads it.
    pub fn at(&self, uv: [f64; 2]) -> Option<[f64; 2]> {
        let q = [uv[0] * self.size as f64 - 0.5, uv[1] * self.size as f64 - 0.5];
        let (i, j) = (q[0].floor() as i64, q[1].floor() as i64);
        let f = [q[0] - i as f64, q[1] - j as f64];
        let mut total = 0.0;
        let mut found: Vec<(f64, [f64; 2])> = Vec::new();
        for (di, dj) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let (a, b) = ((i + di).clamp(0, self.size as i64 - 1), (j + dj).clamp(0, self.size as i64 - 1));
            let w = (if di == 1 { f[0] } else { 1.0 - f[0] }) * (if dj == 1 { f[1] } else { 1.0 - f[1] });
            if let Some(c) = self.cells[(b * self.size as i64 + a) as usize] {
                total += w;
                found.push((w, [c[0] as f64, c[1] as f64]));
            }
        }
        if total <= 1e-4 {
            return None;
        }
        agreeing_mean(&found)
    }

    /// The cells as 32-bit floats for the shader, two cells per row of four, with -1,000 where the grid does not reach.
    pub fn words(&self) -> Vec<f32> {
        let mut w: Vec<f32> = self.cells.iter().flat_map(|c| c.unwrap_or([-1000.0, -1000.0])).collect();
        w.resize(w.len().div_ceil(4) * 4, -1000.0);
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_baked_binding_reads_back_the_same() {
        let b = FaceBinding::baked();
        assert!(b.cols > 0 && b.rows > b.mouth_row && b.mouth_row > b.jaw_row);
        assert!(b.points.iter().flatten().count() > 1000);
        assert_eq!(FaceBinding::from_bytes(&b.to_bytes()), Some(b));
    }

    #[test]
    fn each_copy_past_the_seam_is_its_point_and_mirrors_a_copy() {
        let b = FaceBinding::baked();
        let copies: Vec<(usize, usize)> = b.copy_of.iter().enumerate().filter_map(|(k, c)| Some((k, (*c)?))).collect();
        assert!(copies.len() > 1000);
        for (k, from) in copies {
            assert_eq!(b.points[k], b.points[from]);
            assert!(!b.is_copy(from));
            let (i, j) = (k % b.cols, k / b.cols);
            assert!(b.is_copy(j * b.cols + b.cols - 1 - i));
        }
    }
}
