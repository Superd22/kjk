//! The facial hair, from stubble to short beards, drawn as single hairs from a hair grid on the lower face (paint track, part C, layer 6,
//! decision 60). This first step paints stubble.
//!
//! **The grid.** The hair's values live on `face_grid::FaceGrid`, the paint grid over the whole head, whose points are pinned to the
//! layout, so that a point is the same part of the head on every head. The hair groups (`hair_groups.rs`) are written there from the
//! bottom of their list up, and the editor's values last. A stubble group lays the regions below. The shader reads the grid itself, and finds a texel's place on it through `face_grid::UvLookup`.
//!
//! **The regions.** Each point gets a soft weight for seven regions, from the head's landmarks and the skin's normal: the upper lip (the
//! moustache), the soul patch under the lower lip, the chin, the jaw line, the cheeks, the sideburns and the neck. The jaw's edge is
//! the face grid's jaw row, which was found where the skin turns to face downward, and the jaw line, the cheeks and the neck are measured from it. The weights are
//! shared out so that they add up to 1, and each region's density slider of a stubble group sets the hair there. Four edges bound the hair: the cheek line
//! from the side of the nose to the top of the sideburn, the front of the ear, the neck line under the chin, and the nose. The lips'
//! border is the grid's hard edge: a hair's root never lies on the lips, but a hair may grow over their edge.
//!
//! **The direction of growth.** Down on the cheeks and the sideburns, down and outward from the middle on the moustache, down and
//! forward on the chin, down and a little forward along the jaw, and upward and outward on the neck. Each is laid along the skin at the
//! point, and the regions' directions are mixed by their weights. Off the beard, the grid's zones (`face_grid::ZONES`) set it: back
//! over the scalp and the hairline, turning round the crown's whorl without a parting, back and down over the temples and above the
//! ears, and down the sideburns and the nape.
//!
//! **Two halves.** This file holds the processor's half: the regions, the stubble a stubble group lays on the grid, and the field. The
//! hairs are drawn per pixel by the shader `facial_hair.wgsl` of `head_paint_gpu`, with the grid's own drawing in `hair_grid.wgsl`.

use crate::folds::{Anchor, Anchors, Landmarks};
use crate::face_grid::{FaceBinding, FaceGrid, LONG_RING, NO_ZONE, RING, UvLookup, ZONES};
use crate::hair_grid::{GridEdit, HairField};
use crate::hair_groups::{HairGroups, StubbleLayer};
use crate::layout_points::{dot, normalize, sub};
use crate::lips::LipHead;

/// The densities of the regions are a share of this many hairs per square millimetre.
pub const FULL_DENSITY: f64 = 2.0;
/// The regions, in the order of their density sliders and of `BeardHead::regions`.
pub const REGIONS: [&str; 7] = ["moustache", "soul patch", "chin", "jaw line", "cheeks", "sideburns", "neck"];

/// The settings of a stubble (`hair_groups::GroupKind::Stubble`). Their meaning and ranges are in `hair_groups::GROUP_SLIDERS`. They were
/// the facial hair's global sliders until 2026-11-21.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FacialHair {
    pub amount: f64,
    pub length_mm: f64,
    pub hair_mm: f64,
    pub shadow: f64,
    pub patchy: f64,
    pub seed: f64,
    /// The density of each region, in the order of `REGIONS`.
    pub density: [f64; 7],
    pub cheek_line_mm: f64,
    pub neck_line_mm: f64,
    pub sideburn_top_mm: f64,
    pub edge_mm: f64,
    pub lightness: f64,
    pub hue: f64,
    pub chroma: f64,
    pub tone_var: f64,
    pub grey: f64,
    pub sheen: f64,
}

impl FacialHair {
    /// The old global sliders' defaults.
    pub const DEFAULT: FacialHair = FacialHair {
        amount: 0.0,
        length_mm: 1.2,
        hair_mm: 0.11,
        shadow: 6.0,
        patchy: 0.3,
        seed: 0.0,
        density: [0.85, 0.6, 0.85, 0.75, 0.55, 0.7, 0.4],
        cheek_line_mm: 0.0,
        neck_line_mm: 0.0,
        sideburn_top_mm: 0.0,
        edge_mm: 4.0,
        lightness: 22.0,
        hue: 55.0,
        chroma: 14.0,
        tone_var: 0.4,
        grey: 0.0,
        sheen: 0.2,
    };

    /// No facial hair.
    pub const NONE: FacialHair = FacialHair { amount: 0.0, ..FacialHair::DEFAULT };

    pub fn is_none(&self) -> bool {
        self.amount <= 0.0
    }
}

/// The front edge of each ear, as the mean of four vertices on the ear's front rim, the x < 0 ear first. They were picked on the mean
/// of the 57 test heads, about 66 mm from the middle line and 80 mm behind the mouth.
pub const EAR_FRONT: [[u32; 4]; 2] = [[2204, 2169, 2215, 2393], [4525, 4526, 4491, 4538]];

/// The eyes' outer corners lie this far apart on the mean of the 57 test heads, in millimetres. Distances below are given for it and
/// scaled by each head's own spacing.
const MEAN_EYE_SPACING_MM: f64 = 82.6;

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}

/// The distance in a plane from a point to a closed polygon, negative inside.
pub fn polygon_distance(q: [f64; 2], poly: &[[f64; 2]]) -> f64 {
    let mut best = f64::MAX;
    let mut inside = false;
    for k in 0..poly.len() {
        let a = poly[k];
        let b = poly[(k + 1) % poly.len()];
        let e = [b[0] - a[0], b[1] - a[1]];
        let w = [q[0] - a[0], q[1] - a[1]];
        let t = ((w[0] * e[0] + w[1] * e[1]) / (e[0] * e[0] + e[1] * e[1]).max(1e-12)).clamp(0.0, 1.0);
        let d = [w[0] - t * e[0], w[1] - t * e[1]];
        best = best.min((d[0] * d[0] + d[1] * d[1]).sqrt());
        if (a[1] > q[1]) != (b[1] > q[1]) && q[0] < a[0] + (q[1] - a[1]) / (b[1] - a[1]) * e[0] {
            inside = !inside;
        }
    }
    if inside { -best } else { best }
}

/// What the facial hair keeps of one head: the grid, and at each of its points the regions' weights, the distances to the edges and the
/// default direction of growth.
#[derive(Clone, Debug, PartialEq)]
pub struct BeardHead {
    /// The grid the values are written on, and where it lies on the layout.
    pub grid: FaceGrid,
    pub lookup: UvLookup,
    /// At each point, the weight of each region of `REGIONS`; they add up to 1 where any region reaches, and are all 0 elsewhere.
    pub regions: Vec<[f64; 7]>,
    /// At each point, how much it lies in any region, from 0 to 1.
    pub reach: Vec<f64>,
    /// At each point, the height above the cheek line, how far below the neck line it lies, how far behind the sideburns' back edge it
    /// lies, and how far it lies inside the nose's base, in millimetres. Each is positive on the bare side. The neck line runs 45 mm
    /// below the jaw's edge along the skin.
    pub edges: Vec<[f64; 4]>,
    /// How far the top of the sideburn lies above the cheek line at each point, in millimetres, so that the sideburn height slider can
    /// move the cheek line with it.
    pub sideburn_share: Vec<f64>,
    /// The signed distance to the lips' border, in millimetres, positive off the lips, at the bound points and the points carried on
    /// past them.
    pub lips_mm: Vec<f64>,
    /// The default direction of growth, along the skin.
    pub direction: Vec<[f64; 3]>,
    /// The zone of each point, a number of `face_grid::ZONES`, or `face_grid::NO_ZONE` where it is not bound.
    pub zones: Vec<u8>,
    /// The same head with its grid carried `face_grid::LONG_RING` points past the bound ones instead of `face_grid::RING`, for hair
    /// groups whose hairs hang further over the lips. The grid stays as it was for every other beard, so that they paint as before.
    pub over_lips: Option<Box<BeardHead>>,
}

/// The zones whose direction is the beard regions' own.
const BEARD_ZONES: [&str; 7] = ["cheek", "moustache", "soul patch", "chin", "jaw", "neck", "bare"];
/// The whorl turns the scalp's hair round it within this many millimetres.
const WHORL_MM: f64 = 30.0;

/// One point's place in the face's frame, from the middle of the mouth: across (x), up (y) and forward (z), in millimetres.
fn face_xyz(p: [f64; 3], origin: [f64; 3], m: &Landmarks) -> [f64; 3] {
    let d = sub(p, origin);
    [dot(d, m.frame.across), dot(d, m.frame.up), dot(d, m.frame.forward)]
}

impl BeardHead {
    /// The facial hair's grid and regions on a head, from its mesh positions in metres, the layout and the head's lips. It is `None` for
    /// a mesh that does not hold the landmarks' vertices.
    pub fn new(positions: &[[f64; 3]], uv: &[[f64; 2]], layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize], lips: Option<&LipHead>) -> Option<Self> {
        Self::with_binding(&FaceBinding::baked(), positions, uv, layout_triangles, layout_to_mesh, lips)
    }

    /// The same, with the face grid bound by `binding`.
    pub fn with_binding(binding: &FaceBinding, positions: &[[f64; 3]], uv: &[[f64; 2]], layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize], lips: Option<&LipHead>) -> Option<Self> {
        let mut head = Self::with_ring(binding, positions, uv, layout_triangles, layout_to_mesh, lips, RING)?;
        head.over_lips = Self::with_ring(binding, positions, uv, layout_triangles, layout_to_mesh, lips, LONG_RING).map(Box::new);
        Some(head)
    }

    /// The same, with the grid's places carried on `ring` points past the bound ones, and no grid for hair over the lips.
    fn with_ring(binding: &FaceBinding, positions: &[[f64; 3]], uv: &[[f64; 2]], layout_triangles: &[[u32; 3]], layout_to_mesh: &[usize], lips: Option<&LipHead>, ring: usize) -> Option<Self> {
        let lips = lips?;
        if binding.cols == 0 || binding.points.iter().flatten().any(|&(t, _)| t as usize >= layout_triangles.len()) {
            return None;
        }
        if EAR_FRONT.iter().flatten().any(|&v| v as usize >= positions.len()) || crate::folds::ANCHORS.all().iter().any(|a| a.vertices.iter().any(|&v| v as usize >= positions.len())) {
            return None;
        }
        let m = Landmarks::new(positions);
        let mouth = [0, 1, 2].map(|k| 0.5 * (m.mouth_corner[0][k] + m.mouth_corner[1][k]));
        let spacing = dot(sub(m.eye_outer[1], m.eye_outer[0]), sub(m.eye_outer[1], m.eye_outer[0])).sqrt();
        let scale = spacing / MEAN_EYE_SPACING_MM;
        // A point well inside the head, toward which no normal of the skin points.
        let centre = add(mouth, m.frame.forward, -80.0 * scale);
        let at = |p: [f64; 3]| face_xyz(p, mouth, &m);
        let ears = EAR_FRONT.map(|vs| {
            let a = Anchor { vertices: [vs[0], vs[1], vs[2]], weights: [1.0 / 3.0; 3] };
            let b = Anchor { vertices: [vs[3], vs[3], vs[3]], weights: [1.0 / 3.0; 3] };
            let (pa, pb) = (Anchors::point(&a, positions), Anchors::point(&b, positions));
            at([0, 1, 2].map(|k| 0.75 * pa[k] + 0.25 * pb[k]))
        });
        let corner_x = 0.5 * (at(m.mouth_corner[1])[0] - at(m.mouth_corner[0])[0]);
        let wings = [at(m.nose_wing[0]), at(m.nose_wing[1])];
        let wing_x = 0.5 * (wings[1][0] - wings[0][0]);
        let nose_base = 0.5 * (wings[0][1] + wings[1][1]) - 8.0 * scale;
        let lip_poly: Vec<[f64; 2]> = lips.upper.iter().chain(lips.lower.iter().rev()).copied().collect();
        let lower_mid = lips.lower[lips.lower.len() / 2];
        let lower_y = at(lips_point(lips, lower_mid))[1];
        let up = m.frame.up;
        let fw = m.frame.forward;
        let grid = FaceGrid::with_ring(binding, [m.frame.across, m.frame.up, m.frame.forward], centre, uv, layout_triangles, layout_to_mesh, positions, ring);
        let n = grid.points.len();
        let jaw = jaw_heights(&grid, 2.0 * scale);
        let mut regions = vec![[0.0; 7]; n];
        let mut reach = vec![0.0; n];
        let mut edges = vec![[1e3; 4]; n];
        let mut sideburn_share = vec![0.0; n];
        let mut lips_mm = vec![1e3; n];
        let mut direction = vec![[0.0; 3]; n];
        for (k, g) in grid.points.iter().enumerate() {
            if !g.valid {
                if grid.known(k) {
                    lips_mm[k] = lip_distance(lips, &lip_poly, g.position, scale);
                }
                continue;
            }
            let q = at(g.position);
            let (x, y, z) = (q[0], q[1], q[2]);
            let s = if x < 0.0 { 0 } else { 1 };
            let sign = if s == 0 { -1.0 } else { 1.0 };
            let ax = x.abs();
            let ny = dot(g.normal, up);
            let ear = ears[s];
            lips_mm[k] = lip_distance(lips, &lip_poly, g.position, scale);
            // The cheek line, from beside the nose wing to the top of the sideburn in front of the ear, sagging 4 mm in its middle.
            let start = [wing_x + 5.0 * scale, nose_base];
            let top = [ear[0].abs() - 3.0 * scale, ear[1] - 24.0 * scale];
            let along = ((ax - start[0]) / (top[0] - start[0])).clamp(0.0, 1.0);
            let line = start[1] + along * (top[1] - start[1]) - 4.0 * scale * 4.0 * along * (1.0 - along);
            let behind_ear = ear[2] + 3.0 * scale - z;
            let jm = jaw[k];
            let in_nose = if ax < wing_x + 3.0 * scale { y - nose_base } else { -1e3 };
            // In front of the ear the sideburn's strip reaches up to the temple, past the cheek line.
            let strip = smooth(ear[2] + 26.0 * scale, ear[2] + 14.0 * scale, z) * smooth(28.0 * scale, 40.0 * scale, ax);
            let sideburn_top = ear[1] + 16.0 * scale;
            let cheek = if ax > start[0] - 6.0 * scale { y - line } else { -1e3 };
            edges[k] = [cheek + strip * ((y - sideburn_top) - cheek), -jm - NECK_DEPTH_MM * scale, behind_ear, in_nose];
            sideburn_share[k] = strip.max(0.5 * along * along);
            // The moustache lies between the upper lip and the nose, and its sides follow the folds from the nose's wings down to the
            // mouth's corners.
            let fold = wing_x + 2.0 * scale + (corner_x + 3.0 * scale - wing_x - 2.0 * scale) * smooth(nose_base, 0.0, y);
            let w = [
                smooth(fold + 4.0 * scale, fold - 2.0 * scale, ax) * smooth(-5.0, 0.5, y) * smooth(-0.6, -0.3, ny),
                smooth(14.0 * scale, 7.0 * scale, ax) * smooth(lower_y - 21.0 * scale, lower_y - 14.0 * scale, y) * smooth(lower_y - 1.0 * scale, lower_y - 4.5 * scale, y),
                smooth(34.0 * scale, 20.0 * scale, ax) * smooth(lower_y - 1.0 * scale, lower_y - 6.0 * scale, y) * smooth(-3.0, 4.0, jm),
                (0.5 + 0.5 * smooth(8.0 * scale, 24.0 * scale, ax)) * smooth(-14.0 * scale, -6.0 * scale, jm) * smooth(26.0 * scale, 14.0 * scale, jm) * smooth(-2.0, -10.0, y),
                smooth(corner_x, corner_x + 9.0 * scale, ax) * smooth(6.0 * scale, 16.0 * scale, jm) * smooth(ear[2] + 20.0 * scale, ear[2] + 30.0 * scale, z),
                smooth(ear[2] + 32.0 * scale, ear[2] + 20.0 * scale, z) * smooth(32.0 * scale, 45.0 * scale, ax) * smooth(4.0 * scale, 12.0 * scale, jm),
                smooth(-4.0 * scale, -12.0 * scale, jm),
            ];
            let total: f64 = w.iter().sum();
            if total <= 1e-6 {
                continue;
            }
            regions[k] = w.map(|v| v / total);
            reach[k] = smooth(0.0, 0.4, total);
            let out = m.frame.across.map(|v| v * sign);
            let dirs = [
                add(up.map(|v| -v), out, 0.15 + 0.6 * (ax / corner_x).min(1.2)),
                up.map(|v| -v),
                add(up.map(|v| -v), fw, 0.7),
                add(up.map(|v| -v), fw, 0.3),
                add(up.map(|v| -v), fw, 0.15),
                up.map(|v| -v),
                add(add(fw, up, 0.6), out, 0.5),
            ];
            let nrm = g.normal;
            let mut d = [0.0; 3];
            for (r, dir) in dirs.iter().enumerate() {
                let t = normalize(add(*dir, nrm, -dot(*dir, nrm)));
                d = add(d, t, regions[k][r]);
            }
            let fallback = normalize(add(up.map(|v| -v), nrm, -dot(up.map(|v| -v), nrm)));
            let l = dot(d, d).sqrt();
            direction[k] = if l > 0.3 { d.map(|v| v / l) } else { fallback };
        }
        let lookup = UvLookup::with_ring(binding, uv, layout_triangles, LOOKUP_SIZE, 6, ring);
        let zones = if binding.zones.len() == n { binding.zones.clone() } else { vec![0; n] };
        zone_directions(&grid, &zones, binding.marks.first().copied(), &mut direction);
        Some(BeardHead { grid, lookup, regions, reach, edges, sideburn_share, lips_mm, direction, zones, over_lips: None })
    }

    /// The field on the grid: the hair groups from the bottom of the list up, then the editor's values, which take the look and the
    /// shadow of the bottom group. A point carried on past the bound points within 3 mm of the lips takes the hair of the bound point
    /// next to it, so that the hair reaches the lips' border, where the edge stops its roots. A copy past the seam takes the hair of its
    /// point on the other half.
    pub fn full_field(&self, groups: &HairGroups, edit: Option<&GridEdit>) -> HairField {
        let mut field = HairField::empty(self.grid.points.len());
        field.direction.copy_from_slice(&self.direction);
        field.edge_mm.copy_from_slice(&self.lips_mm);
        groups.write(&self.grid, &mut field, FULL_DENSITY, 0, &self.direction, &|fh| self.stubble(fh));
        if let Some(e) = edit {
            let (bottom, shadow) = groups.bottom_shadow();
            field.apply(&self.grid, e, 1.0 / FULL_DENSITY, shadow, bottom.unwrap_or(0) as u32);
        }
        for k in 0..self.grid.points.len() {
            let Some(from) = self.grid.source[k] else { continue };
            // Past the edge of a hole other than the lips, such as the nose, an eye or an ear, no root grows.
            let p = sub(self.grid.points[k].position, self.grid.points[from].position);
            let hole = if self.lips_mm[k] > 6.0 { 0.5 - dot(p, p).sqrt() } else { 1e3 };
            field.edge_mm[k] = self.lips_mm[k].min(hole);
            field.direction[k] = field.direction[from];
            field.length_mm[k] = field.length_mm[from];
            field.look[k] = field.look[from];
            if self.lips_mm[k] < 3.0 {
                field.density[k] = field.density[from];
                field.shadow[k] = field.shadow[from];
            }
        }
        let g = &self.grid;
        g.share(&mut field.density);
        g.share(&mut field.shadow);
        g.share(&mut field.direction);
        g.share(&mut field.length_mm);
        g.share(&mut field.look);
        g.share(&mut field.edge_mm);
        field
    }

    /// The words of the grid for the shader (`FaceGrid::words`) for some hair groups and editor's values, with the layer's looks, one
    /// per group.
    /// When a group lets its hairs hang over the lips, the words are those of the grid carried `LONG_RING` points past the bound ones.
    pub fn words(&self, groups: &HairGroups, edit: Option<&GridEdit>, looks: &[[f64; 36]], offset: usize) -> Vec<f32> {
        if let Some(long) = self.over_lips.as_deref().filter(|_| groups.over_lips()) {
            return long.words(groups, edit, looks, offset);
        }
        self.grid.words(&self.full_field(groups, edit), looks, &self.lookup, offset)
    }

    /// The grid's points on the skin for a viewer: for each, eleven numbers, its place and its skin's normal in metres, its density in
    /// hairs per square millimetre, its direction of growth, and the number of its zone in `face_grid::ZONES`, or of its main region in
    /// `REGIONS` on a grid with no zones. `edit` is laid over the hair groups.
    pub fn view_points(&self, groups: &HairGroups, edit: Option<&GridEdit>) -> Vec<f32> {
        let field = self.full_field(groups, edit);
        let mut out = Vec::new();
        for (k, g) in self.grid.points.iter().enumerate() {
            if !g.valid {
                continue;
            }
            let main = (0..7).max_by(|&a, &b| self.regions[k][a].total_cmp(&self.regions[k][b])).unwrap_or(0);
            out.extend(g.position.map(|x| (x / 1000.0) as f32));
            out.extend(g.normal.map(|x| x as f32));
            out.push(field.density[k] as f32);
            out.extend(field.direction[k].map(|x| x as f32));
            out.push(if self.zones[k] == NO_ZONE { main as f32 } else { f32::from(self.zones[k]) });
        }
        out
    }

    /// What a stubble lays on the grid. It covers a point by how much the point lies in the beard's regions and inside its edges. Its
    /// density there is the regions' densities mixed by their weights, and its shadow on the skin from 0 to 1 follows the density.
    pub fn stubble(&self, fh: &FacialHair) -> StubbleLayer {
        let n = self.grid.points.len();
        let mut s = StubbleLayer { cover: vec![0.0; n], density: vec![0.0; n], shadow: vec![0.0; n] };
        let soft = fh.edge_mm.max(0.5);
        let seed = fh.seed.round().max(0.0) as u64;
        for k in 0..n {
            let g = &self.grid.points[k];
            if !g.valid || self.reach[k] <= 0.0 {
                continue;
            }
            let e = self.edges[k];
            let cheek = e[0] - fh.cheek_line_mm - fh.sideburn_top_mm * self.sideburn_share[k];
            let keep = smooth(soft, -soft, cheek) * smooth(soft, -soft, e[1] - fh.neck_line_mm) * smooth(0.5 * soft, -0.5 * soft, e[2]) * smooth(1.5, -1.5, e[3]);
            let wanted: f64 = self.regions[k].iter().zip(fh.density).map(|(w, d)| w * d).sum();
            s.cover[k] = self.reach[k] * keep;
            let base = fh.amount * s.cover[k] * wanted;
            let patch = crate::noise::fractal_noise(g.position, 11.0, 2, 0.5, 4049 + seed);
            let patchy = (1.0 + fh.patchy * 2.2 * patch).clamp(0.0, 1.0 + 0.5 * fh.patchy);
            s.density[k] = FULL_DENSITY * base * patchy;
            // Dense hair darkens the skin more than in proportion, so the beard has dark cores where it is thickest.
            s.shadow[k] = base * (0.4 + 0.6 * base.min(1.0)) * (1.0 + 0.4 * fh.patchy * patch).clamp(0.0, 1.2);
        }
        // The band under the lower lip where no root grows still takes the shadow of the hair round it, so that the beard's shadow
        // reaches the lip and no pale band shows there.
        let around = self.grid.near(&s.shadow, 2);
        for k in 0..n {
            if self.lips_mm[k] < 0.0 && self.lips_mm[k] > -(LOWER_LIP_GAP_MM + 1.5) {
                s.shadow[k] = s.shadow[k].max(0.85 * around[k]);
            }
        }
        s
    }
}

/// The default direction of growth off the beard, by zone, laid along the skin, with `beard` the regions' own directions. The head's
/// upright is the mesh's y axis, made square to the face's across. A point next to points of another zone takes the mean of their
/// directions within two steps, so that the zones' directions meet without a seam. The copies past the seam are left out of the mean,
/// and take their own point's direction.
fn zone_directions(grid: &FaceGrid, zones: &[u8], whorl: Option<[f64; 2]>, beard: &mut [[f64; 3]]) {
    let n = grid.points.len();
    let across = grid.across;
    let up = normalize(add([0.0, 1.0, 0.0], across, -across[1]));
    let fw = [across[1] * up[2] - across[2] * up[1], across[2] * up[0] - across[0] * up[2], across[0] * up[1] - across[1] * up[0]];
    let zone = |name: &str| ZONES.iter().position(|z| *z == name).map_or(NO_ZONE, |k| k as u8);
    let beardlike: Vec<u8> = BEARD_ZONES.iter().map(|z| zone(z)).collect();
    let whorl_at = whorl.and_then(|w| grid.place(w[0], w[1])).map(|p| p.0);
    let raw: Vec<[f64; 3]> = (0..n)
        .map(|k| {
            let g = &grid.points[k];
            if !g.valid {
                return beard[k];
            }
            let nrm = g.normal;
            let along = |d: [f64; 3]| {
                let t = add(d, nrm, -dot(d, nrm));
                if dot(t, t) > 1e-12 { normalize(t) } else { [0.0; 3] }
            };
            let down = up.map(|v| -v);
            if zones[k] == NO_ZONE || beardlike.contains(&zones[k]) {
                return if dot(beard[k], beard[k]) > 1e-12 { beard[k] } else { along(down) };
            }
            let z = zones[k];
            if z == zone("sideburn") || z == zone("nape") {
                return along(down);
            }
            if z == zone("temple") {
                return along(add(fw.map(|v| -v), up, -0.9));
            }
            let back = along(add(fw.map(|v| -v), up, -0.35));
            let Some(w) = whorl_at else { return back };
            let r = sub(g.position, w);
            let dist = dot(r, r).sqrt();
            let out = along(r);
            let turn = [nrm[1] * out[2] - nrm[2] * out[1], nrm[2] * out[0] - nrm[0] * out[2], nrm[0] * out[1] - nrm[1] * out[0]];
            let spiral = along(add(out, turn, 0.8));
            let f = smooth(WHORL_MM, 0.3 * WHORL_MM, dist);
            along(add(back.map(|v| v * (1.0 - f)), spiral, f))
        })
        .collect();
    for k in 0..n {
        let g = &grid.points[k];
        if !g.valid || grid.is_copy(k) || zones[k] == NO_ZONE || beardlike.contains(&zones[k]) {
            continue;
        }
        let (i, j) = ((k % grid.cols) as i64, (k / grid.cols) as i64);
        let own = |a: i64, b: i64| (a >= 0 && b >= 0 && a < grid.cols as i64 && b < grid.rows as i64).then_some((b * grid.cols as i64 + a) as usize).filter(|&o| grid.points[o].valid && !grid.is_copy(o));
        let mixed = (-2..=2).any(|dj| (-2..=2).any(|di| own(i + di, j + dj).is_some_and(|o| zones[o] != zones[k])));
        let mut d = raw[k];
        if mixed {
            d = [0.0; 3];
            for dj in -2..=2i64 {
                for di in -2..=2i64 {
                    if let Some(o) = own(i + di, j + dj) {
                        d = add(d, raw[o], 1.0);
                    }
                }
            }
            let nrm = g.normal;
            d = add(d, nrm, -dot(d, nrm));
        }
        if dot(d, d) > 1e-12 {
            beard[k] = normalize(d);
        }
    }
    grid.share(beard);
}

/// The lookup from the layout to the grid has this many cells along each side, about 0.5 mm of skin each on the face and 3 mm on the scalp.
const LOOKUP_SIZE: usize = 512;

/// No root grows this close to the lower lip's border, in millimetres for the mean head.
const LOWER_LIP_GAP_MM: f64 = 3.0;

/// The signed distance from a point to the lips' border, in millimetres, positive off the lips. Below the opening, no root grows within
/// 3 mm of the lower lip's border, as on a real face. Above it the moustache may start half a millimetre from the upper lip.
fn lip_distance(lips: &LipHead, lip_poly: &[[f64; 2]], p: [f64; 3], scale: f64) -> f64 {
    let uvw = lips.frame.uvw(p);
    let margin = if uvw[1] < curve_at(&lips.opening, uvw[0]) { LOWER_LIP_GAP_MM * scale } else { 0.5 };
    if uvw[2] > -6.0 { polygon_distance([uvw[0], uvw[1]], lip_poly) - margin } else { 1e3 }
}

/// The height `v` of a curve of (u, v) points at `u`, held flat past its ends.
fn curve_at(c: &[[f64; 2]], u: f64) -> f64 {
    if u <= c[0][0] {
        return c[0][1];
    }
    for w in c.windows(2) {
        if u <= w[1][0] {
            let f = (u - w[0][0]) / (w[1][0] - w[0][0]).max(1e-9);
            return w[0][1] + f * (w[1][1] - w[0][1]);
        }
    }
    c[c.len() - 1][1]
}

/// How far the neck's hair reaches below the jaw's edge, along the skin, in millimetres, before the neck line slider moves it.
const NECK_DEPTH_MM: f64 = 45.0;

/// The jaw's edge in each column of the grid, as a row with a fraction. Going down a column from above the jaw row, the edge is where
/// the skin first faces more than 30 degrees below the face's forward direction. The rows do not follow the jaw's edge out to the
/// ears, so each column finds its own. The edges are then moved together so that the middle column's is the grid's jaw row. A column
/// where the skin never turns down near the jaw row, as behind the ear, takes the edge of the nearest column that has one.
fn jaw_edges(grid: &FaceGrid) -> Vec<f64> {
    let jr = grid.jaw_row;
    let facing = |i: usize, j: usize| { let p = grid.at(i, j); (p.valid && !grid.is_copy(j * grid.cols + i)).then(|| dot(p.normal, grid.up)) };
    let found: Vec<Option<f64>> = (0..grid.cols)
        .map(|i| {
            let top = (jr + JAW_SEARCH_ROWS[1]).min(grid.rows - 1);
            let bottom = jr.saturating_sub(JAW_SEARCH_ROWS[0]);
            (bottom..top).rev().find_map(|j| match (facing(i, j), facing(i, j + 1)) {
                (Some(a), Some(b)) if a < -0.5 && b >= -0.5 => Some(j as f64 + (-0.5 - a) / (b - a)),
                _ => None,
            })
        })
        .collect();
    let mid = (grid.cols - 1) / 2;
    let shift = found[mid].map_or(0.5, |e| jr as f64 - e);
    (0..grid.cols)
        .map(|i| {
            let near = (0..grid.cols).filter(|&a| found[a].is_some()).min_by_key(|&a| a.abs_diff(i));
            near.map_or(jr as f64, |a| {
                let around: Vec<f64> = (a.saturating_sub(2)..=(a + 2).min(grid.cols - 1)).filter_map(|b| found[b]).collect();
                around.iter().sum::<f64>() / around.len() as f64 + shift
            })
        })
        .collect()
}

/// The jaw's edge is searched for this many rows below and above the jaw row.
const JAW_SEARCH_ROWS: [usize; 2] = [4, 14];

/// The height of each point of the face grid above the jaw's edge, in millimetres along the skin, positive above it. The jaw's edge in
/// each column comes from `jaw_edges`. Along each column the height is summed from the valid point nearest the edge, which is taken to
/// lie `step` times its distance in rows from the edge.
fn jaw_heights(grid: &FaceGrid, step: f64) -> Vec<f64> {
    let mut out = vec![1e3; grid.points.len()];
    let edges = jaw_edges(grid);
    for i in 0..grid.cols {
        let jr = edges[i];
        let Some(start) = (0..grid.rows).filter(|&j| grid.at(i, j).valid).min_by(|&a, &b| (a as f64 - jr).abs().total_cmp(&(b as f64 - jr).abs())) else { continue };
        out[start * grid.cols + i] = (start as f64 - jr) * step;
        for dir in [1i64, -1] {
            let mut j = start as i64;
            let mut h = out[start * grid.cols + i];
            loop {
                let n = j + dir;
                if n < 0 || n >= grid.rows as i64 || !grid.at(i, n as usize).valid {
                    break;
                }
                let d = sub(grid.at(i, n as usize).position, grid.at(i, j as usize).position);
                h += dir as f64 * dot(d, d).sqrt();
                out[n as usize * grid.cols + i] = h;
                j = n;
            }
        }
    }
    out
}

/// The 3D point of the lips' frame at (u, v) on its cylinder.
fn lips_point(lips: &LipHead, q: [f64; 2]) -> [f64; 3] {
    let fr = &lips.frame;
    let a = q[0] / crate::lips::RADIUS_MM;
    let (x, z) = (crate::lips::RADIUS_MM * a.sin(), crate::lips::RADIUS_MM * a.cos() - crate::lips::RADIUS_MM);
    [0, 1, 2].map(|k| fr.origin[k] + x * fr.across[k] + q[1] * fr.up[k] + z * fr.forward[k])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_square_is_inside_and_outside() {
        let sq = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
        assert!((polygon_distance([1.0, 1.0], &sq) + 1.0).abs() < 1e-12);
        assert!((polygon_distance([3.0, 1.0], &sq) - 1.0).abs() < 1e-12);
    }
}
