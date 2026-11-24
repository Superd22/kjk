//! The upper lid piece: one rigid piece per eye, generated from the head,
//! that KOTOR's lid bone turns for the blink, with a lash strip on its edge.
//!
//! The skin round the eye never moves. The piece is a thin shell whose two
//! faces are surfaces of revolution about the blink axis, which is the head's
//! left to right axis through the pivot (the eyeball's centre by default). A
//! turn about that axis slides each face over itself, so the piece can never
//! cut into the eyeball or lift off it, whatever the angle.
//!
//! How the piece meets the skin. The skin's upper lid ends in a roll: from
//! the edge of the opening (`lid_edges.json`, which is the top of the lid's
//! lining, lying on the eyeball) the skin runs down to the lid's margin and
//! then out and up as the front of the lid. The piece's outer face lies just
//! in front of that roll, so the roll is hidden behind it, and the front of
//! the lid crosses the piece a little above the margin. That crossing is the
//! only line where skin and piece meet. The roll's faces that lie wholly
//! behind the piece are not drawn (`EyePiece::hidden_faces`), so a wide open
//! lid shows the eyeball there and not the roll.
//!
//! Everything is computed in one eye's own frame: origin at the eyeball's
//! centre, x toward the temple, y up and z forward. The other eye is the
//! mirror case of the same code (`EyeRig::sign`).

/// A point or a vector.
pub type V3 = [f64; 3];

/// The eyeball's radius on a head whose eyeball size is not changed, in mm.
pub const EYEBALL_RADIUS: f64 = 12.3;
/// The distance of the 8 eyeball-centre vertices from their mean when the eyeball's radius is `EYEBALL_RADIUS`, in mm. The eyeball
/// size slider scales that ring with the radius, so a head's positions alone give its eyeball's radius.
pub const RING_MM: f64 = 6.907;

/// The eyeball's radius of a head (positions in metres), in mm, read from its eyeball-centre vertices: `EYEBALL_RADIUS` times their
/// root mean square distance from their mean over `RING_MM`.
pub fn eyeball_radius(head: &[V3], centre: &[u32]) -> f64 {
    let c = mean(head, centre);
    let n = centre.len().max(1) as f64;
    let ms = centre.iter().map(|&v| (0..3).map(|k| (head[v as usize][k] - c[k]).powi(2)).sum::<f64>()).sum::<f64>() / n;
    EYEBALL_RADIUS * ms.sqrt() * 1000.0 / RING_MM
}
/// The turn of KOTOR's lid bone from rest to closed in the body clips, in
/// degrees: the blinks of `tlknorm` and `tlksad` peak at 35.1 to 35.3
/// degrees from a rest of about 1 degree.
pub const CLIP_TURN_DEG: f64 = 34.0;
/// Rings of skin followed out from each vertex of the opening's edge.
pub const PATH_RINGS: usize = 9;
/// The rings, counted from the edge, in which the lid's margin is looked for.
pub const ROLL_RINGS: usize = 4;
/// The rings in which the lower lid's margin is looked for. 0 takes the
/// opening's own edge, where the lower lid meets the eyeball, so that the
/// closed piece always reaches it.
pub const LOWER_RINGS: usize = 2;
/// The piece's columns along the lid, from the inner corner to the outer one.
pub const COLUMNS: usize = 40;
/// The rows of the piece's outer face, from its edge to its top.
pub const ROWS: usize = 14;

/// The settings of one eye's piece. Lengths are in mm, angles in degrees.
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// How open the eye is at rest: -1 is the widest the skin allows, 0 puts
    /// the piece's edge on the skin's own lid margin, and 1 is sleepy, with
    /// the edge lowered by `SLEEPY_SHARE` of the opening in the middle.
    pub open: f64,
    /// How far the lash strip sticks out from the piece's edge.
    pub lash_length: f64,
    /// How thick the lash strip is.
    pub lash_thickness: f64,
    /// How far the lash strip leans up from straight out.
    pub lash_tilt: f64,
    /// How far behind the eyeball's centre the pivot lies.
    pub pivot_back: f64,
    /// When true, `pivot_back` is chosen so that the piece closes at KOTOR's
    /// clip turn, `CLIP_TURN_DEG`.
    pub match_clip: bool,
    /// The gap between the piece's inner face and the eyeball.
    pub gap: f64,
    /// The least thickness of the piece's edge.
    pub min_thickness: f64,
    /// How far below the skin's lid margin the piece's edge lies at open 0.
    pub cover: f64,
    /// How far past the lower lid's margin the edge goes when closed.
    pub overlap: f64,
    /// How far the open piece's edge is lowered over the eyeball in the middle of the lid, in mm (the slider Upper lid cover). It
    /// fades toward the corners like `open`. A negative value raises it, no further than the skin allows.
    pub lower: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            open: 0.0,
            lash_length: 1.2,
            lash_thickness: 0.25,
            lash_tilt: 35.0,
            pivot_back: 0.0,
            match_clip: false,
            gap: 0.15,
            min_thickness: 0.8,
            cover: 0.2,
            overlap: 0.4,
            lower: 0.0,
        }
    }
}

/// How far in front of the skin's roll the piece's outer face lies, in mm.
const ROLL_CLEARANCE: f64 = 0.15;
/// The most piece that may show at rest between the lid's margin and the
/// skin's crossing, in mm, before the piece goes under the margin instead.
const MAX_BAND: f64 = 1.0;
/// How far in front of the piece the skin stays above the crossing, in mm.
const SKIN_CLEARANCE: f64 = 0.1;
/// The share of the opening, in the middle, that a sleepy lid (open 1) covers.
pub const SLEEPY_SHARE: f64 = 0.45;
/// The band of piece that stays showing between its edge and the skin at the
/// widest setting, in mm.
const MIN_BAND: f64 = 0.15;
/// The piece stops where the lid's margin comes this close to the blink axis, in mm.
pub const END_RHO: f64 = 2.5;
/// Only columns whose margin lies at least this far from the axis set the
/// closing angle; nearer the corners a turn barely moves the lid.
pub const NEED_RHO: f64 = 5.0;
/// How far the piece runs on past each tip of a curve, along the blink axis, in mm.
const CURVE_PAST_TIP: f64 = 3.0;
/// On a curve, how far above the rim the skin the piece wears starts, in mm: past the paint's lash band and liner along the rim.
const CURVE_MIRROR_START: f64 = 2.5;
/// On a curve, over how many mm inside each tip the lash strip fades out.
const CURVE_LASH_FADE: f64 = 1.5;
/// On a curve, how near the axis the upper arch may come before the piece stops, in mm.
const CURVE_END_RHO: f64 = 0.3;
/// On a curve, how far past each tip along the axis a column still sets the closing angle, in mm.
const CURVE_COUNT_PAST: f64 = 1.0;
/// On a curve, the piece's thickness at its ends, as a share of `min_thickness`, and the share of its length at each end over which it
/// thins to that.
const CURVE_END_THICKNESS: f64 = 0.12;
const CURVE_THIN_SHARE: f64 = 0.3;
/// On a curve, how far past the lower arch the closed edge goes, in mm: past the lower lid's rim and the rise of skin just outside it.
const CURVE_OVERLAP: f64 = 0.4;
/// Columns at each end over which the edge rises into the skin.
const END_COLUMNS: usize = 0;
/// How far above the skin's crossing the piece's top stays when closed, in mm.
const TOP_MARGIN: f64 = 1.2;
/// The share of the lid's length, at each end, over which the open setting
/// fades out, so that the corners stay where the skin puts them.
const OPEN_FADE: f64 = 0.3;
/// How many rings above the skin's crossing the piece takes its colour.
const COLOUR_RINGS: usize = 2;
/// How far above the crossing the mirrored skin starts, in mm.
const MIRROR_START: f64 = 0.3;
/// The piece's colour at its edge, as a share of the skin's colour above it.
const EDGE_SHADE: f64 = 0.45;
/// The band, from the edge, over which the colour goes back to the skin's.
const SHADE_BAND: f64 = 1.4;

/// One eye's description from `lid_edges.json`.
#[derive(Clone, Debug)]
pub struct EyeEdge {
    /// The vertices round the opening, in order.
    pub edge: Vec<u32>,
    /// Whether each of them belongs to the upper lid.
    pub upper: Vec<bool>,
    pub inner_corner: u32,
    pub outer_corner: u32,
    /// The vertices whose mean is the eyeball's centre.
    pub centre: Vec<u32>,
    /// Whether the temple lies toward +x in the head's frame.
    pub temple_plus_x: bool,
}

/// What is fixed about one eye on every head: which vertices make up its lids.
#[derive(Clone, Debug)]
pub struct EyeRig {
    /// +1 when the eye's frame has x along the head's +x, -1 for the mirror.
    pub sign: f64,
    pub centre: Vec<u32>,
    /// Paths of skin vertices out from each upper edge vertex, from the inner
    /// corner to the outer one. Each path starts at its edge vertex.
    pub upper: Vec<Vec<u32>>,
    /// The same for the lower lid, from the inner corner to the outer one.
    pub lower: Vec<Vec<u32>>,
    /// The faces whose vertices all lie on the upper paths' first rings: the
    /// candidates for not being drawn.
    pub roll_faces: Vec<u32>,
    /// The faces whose vertices all lie on the paths' first `ROLL_RINGS + 1`
    /// rings, upper and lower: the faces the piece may cover.
    pub coverable_faces: Vec<u32>,
    /// The faces within 45 mm of the eye on the base head, for the checks.
    pub near_faces: Vec<u32>,
}

/// Both eyes' rigs and the head's triangles.
#[derive(Clone, Debug)]
pub struct LidRig {
    pub triangles: Vec<[u32; 3]>,
    pub eyes: Vec<EyeRig>,
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dist(a: V3, b: V3) -> f64 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}
fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl LidRig {
    /// Builds the rig from the head's triangles, the base head's positions
    /// (metres) and the eyes' descriptions. Only the base head's positions
    /// choose the paths, so the rig is the same for every head.
    pub fn new(triangles: &[[u32; 3]], base: &[V3], eyes: &[EyeEdge]) -> Self {
        let n = base.len();
        let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
        for t in triangles {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                if !adj[a as usize].contains(&b) {
                    adj[a as usize].push(b);
                }
                if !adj[b as usize].contains(&a) {
                    adj[b as usize].push(a);
                }
            }
        }
        let rigs = eyes.iter().map(|e| eye_rig(e, triangles, base, &adj)).collect();
        Self { triangles: triangles.to_vec(), eyes: rigs }
    }
}

fn eye_rig(e: &EyeEdge, triangles: &[[u32; 3]], base: &[V3], adj: &[Vec<u32>]) -> EyeRig {
    let n = base.len();
    // rings out from the edge, over the front skin (the edge is the skin's boundary)
    let mut ring = vec![usize::MAX; n];
    let mut front: Vec<u32> = e.edge.clone();
    for &v in &front {
        ring[v as usize] = 0;
    }
    for k in 1..=PATH_RINGS + 6 {
        let mut next = Vec::new();
        for &v in &front {
            for &w in &adj[v as usize] {
                if ring[w as usize] == usize::MAX {
                    ring[w as usize] = k;
                    next.push(w);
                }
            }
        }
        front = next;
    }
    let path = |start: u32| -> Vec<u32> {
        let mut p = vec![start];
        let mut cur = start;
        for k in 1..=PATH_RINGS {
            let best = adj[cur as usize]
                .iter()
                .filter(|&&w| ring[w as usize] == k)
                .min_by(|&&a, &&b| {
                    dist(base[a as usize], base[cur as usize])
                        .total_cmp(&dist(base[b as usize], base[cur as usize]))
                })
                .copied();
            match best {
                Some(w) => {
                    p.push(w);
                    cur = w;
                }
                None => break,
            }
        }
        p
    };
    // the two chains, each from the inner corner to the outer one
    let m = e.edge.len();
    let ic = e.edge.iter().position(|&v| v == e.inner_corner).unwrap_or(0);
    let oc = e.edge.iter().position(|&v| v == e.outer_corner).unwrap_or(m / 2);
    let walk = |step: usize| -> Vec<u32> {
        let mut out = vec![e.edge[ic]];
        let mut i = ic;
        while i != oc {
            i = (i + step) % m;
            out.push(e.edge[i]);
        }
        out
    };
    let a = walk(1);
    let b = walk(m - 1);
    let upper_votes = |c: &[u32]| {
        c.iter()
            .filter(|&&v| e.edge.iter().position(|&w| w == v).is_some_and(|i| e.upper[i]))
            .count() as f64
            / c.len() as f64
    };
    let (up, low) = if upper_votes(&a) >= upper_votes(&b) { (a, b) } else { (b, a) };
    let upper: Vec<Vec<u32>> = up.iter().map(|&v| path(v)).collect();
    let lower: Vec<Vec<u32>> = low.iter().map(|&v| path(v)).collect();
    let mut on_up = vec![false; n];
    let mut on_any = vec![false; n];
    for p in &upper {
        for (k, &v) in p.iter().enumerate() {
            if k <= ROLL_RINGS + 1 {
                on_up[v as usize] = true;
                on_any[v as usize] = true;
            }
        }
    }
    for p in &lower {
        for (k, &v) in p.iter().enumerate() {
            if k <= ROLL_RINGS + 1 {
                on_any[v as usize] = true;
            }
        }
    }
    // faces count as covered by the paths when all three vertices lie within the rings the paths reach
    let in_rings = |v: u32, upto: usize| ring[v as usize] <= upto;
    let mut roll_faces = Vec::new();
    let mut coverable_faces = Vec::new();
    let mut near_faces = Vec::new();
    let c = mean(base, &e.centre);
    for (f, t) in triangles.iter().enumerate() {
        let near = t.iter().all(|&v| dist(base[v as usize], c) < 0.045);
        if near {
            near_faces.push(f as u32);
        }
        if t.iter().all(|&v| in_rings(v, ROLL_RINGS + 1)) {
            let upper_side = t.iter().filter(|&&v| on_up[v as usize]).count() >= 2;
            if upper_side {
                roll_faces.push(f as u32);
            }
            if t.iter().all(|&v| on_any[v as usize] || ring[v as usize] <= ROLL_RINGS) {
                coverable_faces.push(f as u32);
            }
        }
    }
    EyeRig {
        sign: if e.temple_plus_x { 1.0 } else { -1.0 },
        centre: e.centre.clone(),
        upper,
        lower,
        roll_faces,
        coverable_faces,
        near_faces,
    }
}

fn mean(p: &[V3], ids: &[u32]) -> V3 {
    let mut s = [0.0; 3];
    for &i in ids {
        for k in 0..3 {
            s[k] += p[i as usize][k];
        }
    }
    s.map(|x| x / ids.len().max(1) as f64)
}

/// One eye's piece, in the head's frame (metres), at rest.
#[derive(Clone, Debug, Default)]
pub struct EyePiece {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    /// For each vertex, four skin vertices and their weights: the piece's
    /// colour is the skin's colour there (its texture coordinates are the
    /// weighted sum of theirs).
    pub colour_from: Vec<[u32; 4]>,
    pub colour_weights: Vec<[f32; 4]>,
    /// For each vertex, how much of the skin's colour it keeps (1 all, 0 black).
    pub shade: Vec<f32>,
    /// Which part each vertex belongs to: 0 outer face, 1 the middle of the
    /// rounded edge, 2 inner face, 3 the inner face's row at the edge.
    pub part: Vec<u8>,
    /// Each vertex's column, from the inner corner (0) to the outer one.
    pub column: Vec<u16>,
    pub lash_positions: Vec<[f32; 3]>,
    pub lash_triangles: Vec<[u32; 3]>,
    /// The point the piece turns about, and the axis (the head's +x).
    pub pivot: V3,
    pub axis: V3,
    /// The turn about `axis`, in degrees, at which the piece is closed. A
    /// positive turn closes it, on both eyes.
    pub closed_angle: f64,
    /// The pivot's distance behind the eyeball's centre, in mm.
    pub pivot_back: f64,
    /// The eyeball's radius in mm, read from the head's eyeball-centre vertices.
    pub eyeball_radius: f64,
    /// Skin faces not to draw: the roll behind the piece.
    pub hidden_faces: Vec<u32>,
    pub info: PieceInfo,
}

/// Numbers about one eye's piece, for reports and the page.
#[derive(Clone, Copy, Debug, Default)]
pub struct PieceInfo {
    /// The edge's thickness, least and most, in mm.
    pub thickness_min: f64,
    pub thickness_max: f64,
    /// The open setting's ends in degrees of turn in the middle of the lid
    /// (wide is the most the edge can rise, sleepy the most it drops).
    pub wide_deg: f64,
    pub sleepy_deg: f64,
    /// The opening's height in the middle of the lid, edge to lower margin, mm.
    pub opening_mm: f64,
    /// The visible band of piece, from the skin's crossing down to the edge,
    /// in the middle of the lid, mm.
    pub band_mm: f64,
    /// Columns whose skin never rises in front of the piece within the paths.
    pub columns_without_crossing: u32,
    /// The closing angle the eye needs with the pivot at the eyeball's centre.
    pub closed_angle_centred: f64,
}

/// A point in the eye's frame, read in the pivot's cylinder: x along the
/// axis, `rho` from the axis, `beta` the angle from forward, up positive.
#[derive(Clone, Copy, Debug)]
struct Cyl {
    x: f64,
    rho: f64,
    beta: f64,
}

/// The frame of one eye: local = R M (p - c) in mm, with M = diag(sign, 1, 1) and R a turn whose rows are the frame's axes (the
/// identity unless the blink turns about the tips' line, when x runs along that line).
#[derive(Clone, Copy, Debug)]
struct Frame {
    c: V3,
    sign: f64,
    rot: [V3; 3],
}

impl Frame {
    fn local(&self, p: V3) -> V3 {
        let m = [self.sign * (p[0] - self.c[0]) * 1000.0, (p[1] - self.c[1]) * 1000.0, (p[2] - self.c[2]) * 1000.0];
        self.rot.map(|r| r[0] * m[0] + r[1] * m[1] + r[2] * m[2])
    }
    fn dir(&self, q: V3) -> V3 {
        let r = &self.rot;
        let m = [0, 1, 2].map(|k| r[0][k] * q[0] + r[1][k] * q[1] + r[2][k] * q[2]);
        [self.sign * m[0], m[1], m[2]]
    }
    fn head(&self, q: V3) -> V3 {
        let d = self.dir(q);
        [self.c[0] + d[0] / 1000.0, self.c[1] + d[1] / 1000.0, self.c[2] + d[2] / 1000.0]
    }
}

fn cyl(q: V3, pivot: [f64; 2]) -> Cyl {
    let y = q[1] - pivot[0];
    let z = q[2] - pivot[1];
    Cyl { x: q[0], rho: y.hypot(z), beta: y.atan2(z) }
}
fn point(x: f64, rho: f64, beta: f64, pivot: [f64; 2]) -> V3 {
    [x, pivot[0] + rho * beta.sin(), pivot[1] + rho * beta.cos()]
}

/// Piecewise linear interpolation of `ys` over increasing `xs`, held at the ends.
fn interp(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    if x <= xs[0] {
        return ys[0];
    }
    for i in 1..xs.len() {
        if x <= xs[i] {
            let t = if xs[i] > xs[i - 1] { (x - xs[i - 1]) / (xs[i] - xs[i - 1]) } else { 1.0 };
            return ys[i - 1] + t * (ys[i] - ys[i - 1]);
        }
    }
    ys[ys.len() - 1]
}

/// Interpolates an angle about the axis over increasing `xs` through the
/// points it names with `rhos`, so that near the axis, where angles mean
/// little, the line between two points is followed and not their angles.
fn interp_angle(xs: &[f64], betas: &[f64], rhos: &[f64], x: f64) -> f64 {
    let ys: Vec<f64> = betas.iter().zip(rhos).map(|(b, r)| r * b.sin()).collect();
    let zs: Vec<f64> = betas.iter().zip(rhos).map(|(b, r)| r * b.cos()).collect();
    interp(xs, &ys, x).atan2(interp(xs, &zs, x))
}

/// What one lid's chain says at each of its edge vertices, in the eye's frame.
struct Chain {
    /// The x of each sample, made strictly increasing.
    x: Vec<f64>,
    /// The margin's angle (lowest point of the upper roll, highest of the lower).
    margin: Vec<f64>,
    /// The roll's largest distance from the axis, up to the margin.
    roll_rho: Vec<f64>,
    /// The margin's own distance from the axis.
    margin_rho: Vec<f64>,
}

/// Whether a place (eye frame, mm) lies under the eyeball's surface, where the eyeball hides it from every side. Skin sunk there (the
/// slider Outer lid opening) is no part of the lid's margin.
fn sunk(q: V3, radius: f64) -> bool {
    (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() < radius - 0.05
}

fn chain(paths: &[Vec<u32>], pts: &dyn Fn(u32) -> V3, pivot: [f64; 2], upper: bool, radius: f64) -> Chain {
    let last = paths.len() - 1;
    let mut x = Vec::new();
    let mut margin = Vec::new();
    let mut roll_rho = Vec::new();
    let mut margin_rho = Vec::new();
    for (i, p) in paths.iter().enumerate() {
        let cs: Vec<Cyl> = p.iter().map(|&v| cyl(pts(v), pivot)).collect();
        // at the corners the edge vertex itself is the margin
        // the lower lid's margin is looked for nearer its edge, where its path cannot wander up into the upper lid
        let rings = if upper { ROLL_RINGS } else { LOWER_RINGS };
        let reach = if i == 0 || i == last { 0 } else { rings.min(cs.len() - 1) };
        let mut k_m = 0;
        for k in 0..=reach {
            // points next to the axis have no meaningful angle
            if cs[k].rho < 3.0 && k > 0 || k > 0 && sunk(pts(p[k]), radius) {
                continue;
            }
            let better = if upper { cs[k].beta < cs[k_m].beta } else { cs[k].beta > cs[k_m].beta };
            if better {
                k_m = k;
            }
        }
        x.push(cs[k_m].x);
        margin.push(cs[k_m].beta);
        roll_rho.push(cs[..=k_m].iter().map(|c| c.rho).fold(0.0, f64::max));
        margin_rho.push(cs[k_m].rho);
    }
    // keep the samples that run strictly from the inner corner to the outer one
    let (xa, xb) = (x[0], x[last]);
    let mut keep = vec![0];
    for i in 1..last {
        let prev = x[*keep.last().unwrap()];
        if x[i] > prev + 0.02 && x[i] < xb - 0.02 && x[i] > xa && margin_rho[i] >= 3.0 {
            keep.push(i);
        }
    }
    keep.push(last);
    if x[last] <= x[keep[keep.len() - 2]] {
        x[last] = x[keep[keep.len() - 2]] + 0.02;
    }
    let pick = |v: &[f64]| keep.iter().map(|&i| v[i]).collect::<Vec<f64>>();
    Chain { x: pick(&x), margin: pick(&margin), roll_rho: pick(&roll_rho), margin_rho: pick(&margin_rho) }
}

/// An eye opening drawn as a curve on the eyeball (`head_eye_curve`): its upper and lower arches from the inner tip to the outer one,
/// in the eye's frame (mm). The lid's margins are the arches themselves.
pub struct Arches {
    pub upper: Vec<V3>,
    pub lower: Vec<V3>,
}

/// An arch run on past each tip along the eyeball, by half of `CURVE_PAST_TIP` and by all of it. Each extra point lies further along
/// the blink axis (the frame's x) than its tip, at the tip's angle about the axis, put back on the sphere through the tip, so that the
/// piece built on it covers the corners when closed.
fn past_tips(arch: &[V3]) -> Vec<V3> {
    let n = arch.len();
    if n < 2 {
        return arch.to_vec();
    }
    let len = |v: V3| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let on = |tip: V3, sign: f64, mm: f64| -> V3 {
        let q = [tip[0] + sign * mm, tip[1], tip[2]];
        let s = len(tip) / len(q).max(1e-9);
        q.map(|c| c * s)
    };
    let (first, last) = (arch[0], arch[n - 1]);
    let sign = if last[0] >= first[0] { 1.0 } else { -1.0 };
    let mut out = vec![on(first, -sign, CURVE_PAST_TIP), on(first, -sign, 0.5 * CURVE_PAST_TIP)];
    out.extend_from_slice(arch);
    out.push(on(last, sign, 0.5 * CURVE_PAST_TIP));
    out.push(on(last, sign, CURVE_PAST_TIP));
    out
}

/// A chain read from an arch: each point is a margin sample, kept where x runs strictly from the inner tip to the outer one.
fn arch_chain(arch: &[V3], pivot: [f64; 2]) -> Chain {
    let mut out = Chain { x: Vec::new(), margin: Vec::new(), roll_rho: Vec::new(), margin_rho: Vec::new() };
    for &q in arch {
        let c = cyl(q, pivot);
        if out.x.last().is_some_and(|&x| c.x <= x + 0.02) {
            continue;
        }
        out.x.push(c.x);
        out.margin.push(c.beta);
        out.roll_rho.push(c.rho);
        out.margin_rho.push(c.rho);
    }
    out
}

/// Where a path's skin first rises in front of the shell after its margin:
/// the angle, and the two path vertices and the weight of the second.
fn crossing(path: &[u32], pts: &dyn Fn(u32) -> V3, pivot: [f64; 2], shell: &dyn Fn(f64) -> f64, upper: bool) -> (f64, u32, u32, f64, bool, f64) {
    let cs: Vec<Cyl> = path.iter().map(|&v| cyl(pts(v), pivot)).collect();
    let reach = ROLL_RINGS.min(cs.len() - 1);
    let mut k_m = 0;
    for k in 0..=reach {
        let better = if upper { cs[k].beta < cs[k_m].beta } else { cs[k].beta > cs[k_m].beta };
        if better {
            k_m = k;
        }
    }
    for k in k_m + 1..cs.len() {
        let s1 = cs[k].rho - shell(cs[k].x);
        if s1 >= 0.0 {
            let s0 = cs[k - 1].rho - shell(cs[k - 1].x);
            let t = if s1 - s0 > 1e-9 { (-s0 / (s1 - s0)).clamp(0.0, 1.0) } else { 1.0 };
            let beta = cs[k - 1].beta + t * (cs[k].beta - cs[k - 1].beta);
            let above = (k + COLOUR_RINGS).min(path.len() - 1);
            return (beta, path[above - 1], path[above], 0.5, true, cs[k_m].x);
        }
    }
    let k = cs.len() - 1;
    (cs[k].beta, path[k], path[k], 0.0, false, cs[k_m].x)
}

/// The shell's distance from the axis that keeps a circle about the pivot at
/// least `clear` mm outside the eyeball at angle `beta`, in slice `x`.
fn clear_of_eyeball(x: f64, beta: f64, pivot: [f64; 2], clear: f64, radius: f64) -> f64 {
    let r = (radius * radius - x * x).max(0.0).sqrt();
    if r <= 0.0 {
        return 0.0;
    }
    let s = r + clear;
    // the eyeball's centre seen from the pivot, in the y-z plane
    let c = [-pivot[0], -pivot[1]];
    let u = [beta.sin(), beta.cos()];
    let uc = u[0] * c[0] + u[1] * c[1];
    let disc = s * s - (c[0] * c[0] + c[1] * c[1]) + uc * uc;
    if disc < 0.0 { 0.0 } else { uc + disc.sqrt() }
}

/// The piece's shape in the eye's frame before it is turned into meshes.
struct Plan {
    pivot: [f64; 2],
    col_x: Vec<f64>,
    edge: Vec<f64>,
    top: Vec<f64>,
    rho_out: Vec<f64>,
    rho_in: Vec<f64>,
    close: f64,
    ladders: Vec<Vec<(f64, u32)>>,
    col_src: Vec<(usize, usize, f64)>,
    hidden: Vec<u32>,
    info: PieceInfo,
    /// Per column, the angle the piece's colour is mirrored about: the skin's crossing, or on a curve the upper arch.
    col_join: Vec<f64>,
    /// How far above that angle the mirrored skin starts, in mm.
    mirror_start: f64,
    /// Per column, how much of the lash strip it carries: 1 over the opening, falling to 0 at a curve's tips.
    lash: Vec<f64>,
}

#[allow(clippy::too_many_arguments)]
fn plan(rig: &EyeRig, triangles: &[[u32; 3]], pts: &dyn Fn(u32) -> V3, s: &Settings, pivot: [f64; 2], least_close: f64, radius: f64, arches: Option<&Arches>) -> Plan {
    let (up, low) = match arches {
        // only the upper arch runs on past the tips: the piece is built along it, and past the tips the lower margin is held at the tip's
        Some(a) => (arch_chain(&past_tips(&a.upper), pivot), arch_chain(&a.lower, pivot)),
        None => (chain(&rig.upper, pts, pivot, true, radius), chain(&rig.lower, pts, pivot, false, radius)),
    };
    // Near the corners the lid runs close to the blink axis, where a turn
    // barely moves it and angles mean little, so the piece stops where the
    // margin comes within `END_RHO` of the axis and the skin draws the rest.
    // on a curve the tips of a long eye lie near the axis, and the piece must still reach them
    let end_rho = if arches.is_some() { CURVE_END_RHO } else { END_RHO };
    let usable = |i: usize| up.margin_rho[i] >= end_rho;
    let first = (0..up.x.len()).find(|&i| usable(i)).unwrap_or(0);
    let last = (0..up.x.len()).rev().find(|&i| usable(i)).unwrap_or(up.x.len() - 1);
    let end_x = |a: usize, b: usize| -> f64 {
        // where the margin's distance from the axis crosses END_RHO between samples a (too close) and b
        let (ra, rb) = (up.margin_rho[a], up.margin_rho[b]);
        let t = if rb > ra { ((end_rho - ra) / (rb - ra)).clamp(0.0, 1.0) } else { 1.0 };
        up.x[a] + t * (up.x[b] - up.x[a])
    };
    let x0 = if first > 0 { end_x(first - 1, first) } else { up.x[0] };
    let x1 = if last + 1 < up.x.len() { end_x(last + 1, last) } else { up.x[up.x.len() - 1] };
    // columns spread evenly along the upper margin, by its length in mm
    let mut acc = vec![0.0];
    for i in 1..up.x.len() {
        let rho = up.margin_rho[i].max(1.0);
        let d = (up.x[i] - up.x[i - 1]).hypot(rho * (up.margin[i] - up.margin[i - 1]));
        acc.push(acc[i - 1] + d);
    }
    let a0 = interp(&up.x, &acc, x0);
    let a1 = interp(&up.x, &acc, x1);
    let mut col_x: Vec<f64> = (0..=COLUMNS).map(|j| interp(&acc, &up.x, a0 + (a1 - a0) * j as f64 / COLUMNS as f64)).collect();
    col_x[0] = x0;
    col_x[COLUMNS] = x1;
    for j in 1..=COLUMNS {
        if col_x[j] <= col_x[j - 1] + 0.02 {
            col_x[j] = col_x[j - 1] + 0.02;
        }
    }
    let share: Vec<f64> = (0..=COLUMNS).map(|j| j as f64 / COLUMNS as f64).collect();
    let fade: Vec<f64> = share.iter().map(|&t| smoothstep(0.0, OPEN_FADE, t) * smoothstep(0.0, OPEN_FADE, 1.0 - t)).collect();
    let margin_up: Vec<f64> = col_x.iter().map(|&x| interp_angle(&up.x, &up.margin, &up.margin_rho, x)).collect();
    let margin_low: Vec<f64> = col_x.iter().map(|&x| interp_angle(&low.x, &low.margin, &low.margin_rho, x)).collect();
    // The outer face's distance from the axis, chosen per upper path. The
    // skin must cross it once, a little above the lid's margin (or on the
    // margin's underside where the lid hangs in front of the eyeball), and
    // stay in front of it from there on.
    let sweep_lo = margin_low.iter().fold(f64::INFINITY, |a, &b| a.min(b)) - 0.6;
    let sweep_hi = 2.2;
    let need_eye = |x: f64, clear: f64| -> f64 {
        (0..=24)
            .map(|k| clear_of_eyeball(x, sweep_lo + (sweep_hi - sweep_lo) * k as f64 / 24.0, pivot, clear, radius))
            .fold(0.0, f64::max)
    };
    let rho_lo = |x: f64| need_eye(x, s.gap + s.min_thickness);
    let mut pref_x: Vec<f64> = Vec::new();
    let mut pref: Vec<f64> = Vec::new();
    for path in &rig.upper {
        let cs: Vec<Cyl> = path.iter().map(|&v| cyl(pts(v), pivot)).collect();
        let reach = ROLL_RINGS.min(cs.len() - 1);
        let k_m = (0..=reach).filter(|&k| k == 0 || !sunk(pts(path[k]), radius)).min_by(|&a, &b| cs[a].beta.total_cmp(&cs[b].beta)).unwrap_or(0);
        let x = cs[k_m].x;
        if x <= pref_x.last().copied().unwrap_or(f64::NEG_INFINITY) + 0.02 {
            continue;
        }
        let roll_max = cs[..=k_m].iter().map(|c| c.rho).fold(0.0, f64::max);
        let lo = rho_lo(x);
        let valid = |r: f64| -> bool {
            let k_c = if cs[k_m].rho >= r { Some(k_m) } else { (k_m + 1..cs.len()).find(|&k| cs[k].rho >= r) };
            let Some(k_c) = k_c else { return false };
            if k_c > k_m && (cs[k_c].beta - cs[k_m].beta) * r > MAX_BAND {
                return false;
            }
            cs[k_c..].iter().all(|c| c.rho >= r + SKIN_CLEARANCE) || (k_c == k_m && cs[k_c + 1..].iter().all(|c| c.rho >= r + SKIN_CLEARANCE))
        };
        let mut r = (roll_max + ROLL_CLEARANCE).max(lo);
        let mut chosen = r;
        while r >= lo - 1e-9 {
            if valid(r) {
                chosen = r;
                break;
            }
            r -= 0.05;
            chosen = lo;
        }
        pref_x.push(x);
        pref.push(chosen);
    }
    let mut rho_out: Vec<f64> = col_x.iter().map(|&x| interp(&pref_x, &pref, x)).collect();
    // smoothed along the lid, so the edge's thickness does not wobble
    for _ in 0..4 {
        let prev = rho_out.clone();
        for j in 0..=COLUMNS {
            let a = prev[j.saturating_sub(1)];
            let b = prev[(j + 1).min(COLUMNS)];
            rho_out[j] = (0.25 * a + 0.5 * prev[j] + 0.25 * b).max(rho_lo(col_x[j]));
        }
    }
    let mut rho_in: Vec<f64> = col_x
        .iter()
        .zip(&rho_out)
        .map(|(&x, &ro)| need_eye(x, s.gap).max(ro - s.min_thickness - 3.0).min(ro - s.min_thickness * 0.5).max(0.3))
        .collect();
    if arches.is_some() {
        // on a curve the piece lies on the eyeball, and thins toward both ends so that it stays under the skin near the tips
        for j in 0..=COLUMNS {
            let t = j as f64 / COLUMNS as f64;
            let thick = s.min_thickness * (CURVE_END_THICKNESS + (1.0 - CURVE_END_THICKNESS) * smoothstep(0.0, CURVE_THIN_SHARE, t.min(1.0 - t)));
            rho_in[j] = need_eye(col_x[j], s.gap).max(0.3);
            rho_out[j] = rho_in[j] + thick;
        }
    }
    let shell = |x: f64| interp(&col_x, &rho_out, x);
    // where the front of each lid rises in front of the shell
    let mut up_join_x: Vec<f64> = Vec::new();
    let mut up_join = Vec::new();
    let mut up_src = Vec::new();
    let mut up_ladder: Vec<Vec<(f64, u32)>> = Vec::new();
    let mut up_under = Vec::new();
    let mut missing = 0;
    let last_path = rig.upper.len() - 1;
    for (i, p) in rig.upper.iter().enumerate() {
        let (b, a, c, t, ok, x) = crossing(p, pts, pivot, &shell, true);
        if !ok && i != 0 && i != last_path {
            missing += 1;
        }
        // the same clean run from corner to corner as the chains
        if i != 0 && i != last_path && (x <= up_join_x.last().copied().unwrap_or(f64::NEG_INFINITY) + 0.02 || x >= up.x[up.x.len() - 1] - 0.02) {
            continue;
        }
        up_join_x.push(x.max(up_join_x.last().copied().unwrap_or(f64::NEG_INFINITY) + 0.02));
        up_join.push(b);
        up_src.push((a, c, t));
        // the skin above the crossing, by its distance from the crossing, for the piece's texture
        {
            let cs: Vec<Cyl> = p.iter().map(|&v| cyl(pts(v), pivot)).collect();
            let reach = ROLL_RINGS.min(cs.len() - 1);
            let k_m = (0..=reach).filter(|&k| k == 0 || !sunk(pts(p[k]), radius)).min_by(|&a, &b| cs[a].beta.total_cmp(&cs[b].beta)).unwrap_or(0);
            // on a curve the skin crosses the shell far above the rim, near the brow, so the piece wears the skin just above the rim
            let k_c = if arches.is_some() { (k_m + 1).min(p.len() - 1) } else { (k_m + 1..p.len()).find(|&k| cs[k].rho >= shell(cs[k].x)).unwrap_or(p.len() - 1) };
            let mut ladder = vec![(0.0, p[k_c])];
            for k in k_c + 1..p.len() {
                let d = dist(pts(p[k]), pts(p[k - 1]));
                ladder.push((ladder.last().unwrap().0 + d, p[k]));
            }
            up_ladder.push(ladder);
        }
        // where the roll, from the edge out to the margin, first comes in front of the shell
        let cs: Vec<Cyl> = p.iter().map(|&v| cyl(pts(v), pivot)).collect();
        let reach = ROLL_RINGS.min(cs.len() - 1);
        let k_m = (0..=reach).filter(|&k| k == 0 || !sunk(pts(p[k]), radius)).min_by(|&a, &b| cs[a].beta.total_cmp(&cs[b].beta)).unwrap_or(0);
        let mut under = cs[k_m].beta;
        for k in 1..=k_m {
            let (s0, s1) = (cs[k - 1].rho - shell(cs[k - 1].x), cs[k].rho - shell(cs[k].x));
            if s0 < 0.0 && s1 >= 0.0 {
                let t = -s0 / (s1 - s0);
                under = under.max(cs[k - 1].beta + t * (cs[k].beta - cs[k - 1].beta));
                break;
            }
        }
        up_under.push(under);
    }
    let join_rho: Vec<f64> = up_join_x.iter().map(|&x| shell(x)).collect();
    let join: Vec<f64> = col_x.iter().map(|&x| interp_angle(&up_join_x, &up_join, &join_rho, x)).collect();
    let under: Vec<f64> = col_x.iter().map(|&x| interp_angle(&up_join_x, &up_under, &join_rho, x)).collect();
    // the open setting, faded toward the corners
    let mid = COLUMNS / 2;
    let opening_mid = (margin_up[mid] - margin_low[mid]).max(0.0);
    let mut wide = f64::INFINITY;
    for j in 1..COLUMNS {
        if fade[j] > 0.05 {
            let room = join[j] - (margin_up[j].max(under[j]) - s.cover / rho_out[j]) - MIN_BAND / rho_out[j];
            wide = wide.min(room.max(0.0) / fade[j]);
        }
    }
    if !wide.is_finite() {
        wide = 0.0;
    }
    let sleepy = SLEEPY_SHARE * opening_mid;
    let alpha = if s.open >= 0.0 { s.open.min(1.0) * sleepy } else { s.open.max(-1.0) * wide };
    let alpha = (alpha + s.lower / rho_out[mid].max(1.0)).max(-wide);
    let mut edge: Vec<f64> = (0..=COLUMNS)
        .map(|j| margin_up[j].max(under[j]) - s.cover / rho_out[j] - alpha * fade[j])
        .collect();
    for j in 0..=COLUMNS {
        if END_COLUMNS == 0 {
            break;
        }
        let from_end = j.min(COLUMNS - j) as f64 / END_COLUMNS as f64;
        let e = 1.0 - smoothstep(0.0, 1.0, from_end);
        edge[j] += e * (join[j].max(edge[j]) - edge[j]);
    }
    // closed: the edge goes `overlap` past the lower margin, or behind the lower skin where it rises first
    let overlap = if arches.is_some() { CURVE_OVERLAP } else { s.overlap };
    let tips_x = arches.map(|a| {
        let (u, v) = (cyl(a.upper[0], pivot).x, cyl(a.upper[a.upper.len() - 1], pivot).x);
        (u.min(v), u.max(v))
    });
    let close_to: Vec<f64> = (0..=COLUMNS).map(|j| margin_low[j] - overlap / rho_out[j]).collect();
    let mut close = least_close;
    let margin_rho: Vec<f64> = col_x.iter().map(|&x| interp(&up.x, &up.margin_rho, x)).collect();
    for j in END_COLUMNS..=COLUMNS - END_COLUMNS {
        // on a curve every column between the tips counts, and those that run on past the tips count up to `CURVE_COUNT_PAST`
        let counts = match tips_x {
            Some((a, b)) => col_x[j] > a - CURVE_COUNT_PAST && col_x[j] < b + CURVE_COUNT_PAST,
            None => margin_rho[j] >= NEED_RHO,
        };
        if counts {
            close = close.max(edge[j] - close_to[j]);
        }
    }
    if std::env::var("LID_DEBUG").is_ok() {
        for (i, path) in rig.upper.iter().enumerate() {
            let line: Vec<String> = path.iter().map(|&v| { let c = cyl(pts(v), pivot); format!("{:.1}/{:.1}", c.rho, c.beta.to_degrees()) }).collect();
            eprintln!("path {i:2} x {:6.2} shell {:5.2}: {}", cyl(pts(path[0]), pivot).x, shell(cyl(pts(path[0]), pivot).x), line.join(" "));
        }
        for (name, c) in [("up", &up), ("low", &low)] {
            for i in 0..c.x.len() {
                eprintln!("{name} {i:2} x {:6.2} margin {:6.1} rho {:5.2} roll {:5.2}", c.x[i], c.margin[i].to_degrees(), c.margin_rho[i], c.roll_rho[i]);
            }
        }
        for j in 0..=COLUMNS {
            eprintln!(
                "col {j:2} x {:6.2} up {:6.1} low {:6.1} edge {:6.1} join {:6.1} rho {:5.2} in {:5.2} need {:5.1}",
                col_x[j], margin_up[j].to_degrees(), margin_low[j].to_degrees(), edge[j].to_degrees(), join[j].to_degrees(),
                rho_out[j], rho_in[j], (edge[j] - close_to[j]).to_degrees()
            );
        }
    }
    let mut top: Vec<f64> = (0..=COLUMNS)
        .map(|j| (join[j] + close + TOP_MARGIN / rho_out[j]).max(edge[j] + close + 0.1))
        .collect();
    // colour: each column reads the skin above the crossing from its two nearest upper paths
    let col_src: Vec<(usize, usize, f64)> = col_x
        .iter()
        .map(|&x| {
            let i = up_join_x.partition_point(|&v| v < x).clamp(1, up_join_x.len() - 1);
            let (x_a, x_b) = (up_join_x[i - 1], up_join_x[i]);
            let w = if x_b > x_a { ((x - x_a) / (x_b - x_a)).clamp(0.0, 1.0) } else { 0.5 };
            (i - 1, i, w)
        })
        .collect();
    let _ = &up_src;
    // the roll's faces that lie wholly behind the shell, before the skin's crossing, are not drawn
    let mut behind = std::collections::HashSet::new();
    for path in &rig.upper {
        let cs: Vec<Cyl> = path.iter().map(|&v| cyl(pts(v), pivot)).collect();
        let reach = ROLL_RINGS.min(cs.len() - 1);
        let k_m = (0..=reach).filter(|&k| k == 0 || !sunk(pts(path[k]), radius)).min_by(|&a, &b| cs[a].beta.total_cmp(&cs[b].beta)).unwrap_or(0);
        for k in 0..cs.len() {
            if k > k_m && cs[k].rho >= shell(cs[k].x) {
                break;
            }
            if cs[k].x > col_x[0] && cs[k].x < col_x[COLUMNS] && cs[k].rho < shell(cs[k].x) - 0.02 {
                behind.insert(path[k]);
            }
        }
    }
    let hidden: Vec<u32> = rig
        .roll_faces
        .iter()
        .copied()
        .filter(|&f| triangles[f as usize].iter().all(|v| behind.contains(v)))
        .collect();
    // the piece's top stays above every hidden face, also when closed, or the hole would show
    for &f in &hidden {
        for &v in &triangles[f as usize] {
            let c = cyl(pts(v), pivot);
            for j in 0..=COLUMNS {
                if (c.x - col_x[j]).abs() < 2.0 {
                    top[j] = top[j].max(c.beta + close + TOP_MARGIN / rho_out[j]);
                }
            }
        }
    }
    let mut tmin = f64::INFINITY;
    let mut tmax = 0.0_f64;
    for j in 0..=COLUMNS {
        tmin = tmin.min(rho_out[j] - rho_in[j]);
        tmax = tmax.max(rho_out[j] - rho_in[j]);
    }
    let info = PieceInfo {
        thickness_min: tmin,
        thickness_max: tmax,
        wide_deg: (wide * fade[mid]).to_degrees(),
        sleepy_deg: sleepy.to_degrees(),
        opening_mm: (edge[mid] - margin_low[mid]).max(0.0) * rho_out[mid],
        band_mm: (join[mid] - edge[mid]) * rho_out[mid],
        columns_without_crossing: missing,
        closed_angle_centred: 0.0,
    };
    // on a curve the lash strip ends at the tips, fading over the last `CURVE_LASH_FADE` mm, and is not drawn on the piece past them
    let lash = col_x
        .iter()
        .map(|&x| match tips_x {
            Some((a, b)) => smoothstep(a, a + CURVE_LASH_FADE, x) * smoothstep(b, b - CURVE_LASH_FADE, x),
            None => 1.0,
        })
        .collect();
    let col_join = if arches.is_some() { margin_up.clone() } else { join.clone() };
    Plan { pivot, col_x, edge, top, rho_out, rho_in, close, ladders: up_ladder, col_src, hidden, info, lash, col_join, mirror_start: if arches.is_some() { CURVE_MIRROR_START } else { MIRROR_START } }
}

/// Builds one eye's piece on a head.
pub fn build_eye(rig: &EyeRig, triangles: &[[u32; 3]], head: &[V3], s: &Settings) -> EyePiece {
    build_eye_with(rig, triangles, head, s, None)
}

/// Builds one eye's piece on an opening drawn as a curve on the eyeball: `curve` holds its upper and lower arches (metres, the head's
/// frame), each from the inner tip to the outer one. The piece turns about the line through the eyeball's centre parallel to the line
/// through the two tips, its open edge lies along the upper arch and it closes past the lower arch.
pub fn build_eye_on_curve(rig: &EyeRig, triangles: &[[u32; 3]], head: &[V3], s: &Settings, upper: &[V3], lower: &[V3]) -> EyePiece {
    build_eye_with(rig, triangles, head, s, Some((upper, lower)))
}

fn build_eye_with(rig: &EyeRig, triangles: &[[u32; 3]], head: &[V3], s: &Settings, curve: Option<(&[V3], &[V3])>) -> EyePiece {
    let mut frame = Frame { c: mean(head, &rig.centre), sign: rig.sign, rot: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] };
    let base = [0.0, 0.0];
    if let Some((up, _)) = curve {
        // x along the line from the inner tip to the outer one, z forward across it
        let (a, b) = (frame.local(up[0]), frame.local(up[up.len() - 1]));
        let unit = |v: V3| { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(); [v[0] / l, v[1] / l, v[2] / l] };
        let e1 = unit(sub(b, a));
        let e3 = unit([-e1[2] * e1[0], -e1[2] * e1[1], 1.0 - e1[2] * e1[2]]);
        let e2 = [e3[1] * e1[2] - e3[2] * e1[1], e3[2] * e1[0] - e3[0] * e1[2], e3[0] * e1[1] - e3[1] * e1[0]];
        frame.rot = [e1, e2, e3];
    }
    let radius = eyeball_radius(head, &rig.centre);
    let pts = |v: u32| frame.local(head[v as usize]);
    let arches = curve.map(|(u, l)| Arches { upper: u.iter().map(|&q| frame.local(q)).collect(), lower: l.iter().map(|&q| frame.local(q)).collect() });
    let arches = arches.as_ref();
    let mut p = plan(rig, triangles, &pts, s, base, 0.0, radius, arches);
    let centred = p.close.to_degrees();
    let mut back = s.pivot_back.max(0.0);
    if s.match_clip && centred > CLIP_TURN_DEG {
        // the closing angle falls as the pivot moves back; find where it meets the clip's turn
        let (mut lo, mut hi) = (0.0, 30.0);
        for _ in 0..22 {
            let m = 0.5 * (lo + hi);
            let q = plan(rig, triangles, &pts, s, [base[0], base[1] - m], 0.0, radius, arches);
            if q.close.to_degrees() > CLIP_TURN_DEG { lo = m } else { hi = m }
        }
        back = hi;
    } else if s.match_clip {
        back = 0.0;
    }
    if s.match_clip {
        // the clip always turns the lid by its full turn, so the piece is made for it, going further past the lower lid on heads
        // that need less
        p = plan(rig, triangles, &pts, s, [base[0], base[1] - back], CLIP_TURN_DEG.to_radians(), radius, arches);
    } else if back > 0.0 {
        p = plan(rig, triangles, &pts, s, [base[0], base[1] - back], 0.0, radius, arches);
    }
    p.info.closed_angle_centred = centred;
    let mut out = mesh(&p, s, &frame);
    out.hidden_faces = p.hidden.clone();
    out.closed_angle = p.close.to_degrees();
    out.pivot_back = back;
    out.eyeball_radius = radius;
    out.info = p.info;
    out
}

/// Builds both eyes' pieces on a head (positions in metres).
pub fn build(rig: &LidRig, head: &[V3], s: &Settings) -> Vec<EyePiece> {
    rig.eyes.iter().map(|e| build_eye(e, &rig.triangles, head, s)).collect()
}

fn mesh(p: &Plan, s: &Settings, frame: &Frame) -> EyePiece {
    let mut out = EyePiece::default();
    let n = COLUMNS + 1;
    let mut local: Vec<V3> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    // The piece wears the skin above the crossing, mirrored about the
    // crossing: a point some way below the crossing takes the colour of the
    // skin as far above it, so the two meet without a seam.
    let ladder_at = |l: &[(f64, u32)], d: f64| -> (u32, u32, f64) {
        for k in 1..l.len() {
            if d <= l[k].0 {
                let t = ((d - l[k - 1].0) / (l[k].0 - l[k - 1].0).max(1e-9)).clamp(0.0, 1.0);
                return (l[k - 1].1, l[k].1, t);
            }
        }
        let last = l[l.len() - 1].1;
        (last, last, 0.0)
    };
    let source = |col: usize, beta: f64| -> ([u32; 4], [f32; 4]) {
        let d = ((p.col_join[col] - beta) * p.rho_out[col]).max(0.0) + p.mirror_start;
        let (ia, ib, w) = p.col_src[col];
        let (a0, a1, ta) = ladder_at(&p.ladders[ia], d);
        let (b0, b1, tb) = ladder_at(&p.ladders[ib], d);
        let ws = [(1.0 - w) * (1.0 - ta), (1.0 - w) * ta, w * (1.0 - tb), w * tb];
        ([a0, a1, b0, b1], ws.map(|v| v as f32))
    };
    let mut push = |q: V3, col: usize, shade: f64, part: u8, out: &mut EyePiece| -> u32 {
        local.push(q);
        let beta = (q[1] - p.pivot[0]).atan2(q[2] - p.pivot[1]);
        let (from, w) = source(col, beta);
        out.colour_from.push(from);
        out.colour_weights.push(w);
        out.shade.push(shade as f32);
        out.part.push(part);
        out.column.push(col as u16);
        (local.len() - 1) as u32
    };
    // rows of the outer face: denser near the edge
    let row_beta = |j: usize, r: usize| -> f64 {
        let t = (r as f64 / ROWS as f64).powf(1.6);
        p.edge[j] + t * (p.top[j] - p.edge[j])
    };
    let mut outer = vec![vec![0u32; ROWS + 1]; n];
    let mut inner = vec![vec![0u32; ROWS + 1]; n];
    let mut wall = vec![0u32; n];
    // the piece thins toward both corners, so its ends meet the lids' edges instead of standing off them as a flat block: the inner
    // face comes out to the outer one over the last 15% of the lid at each end, down to a tenth of the thickness
    let rho_in: Vec<f64> = (0..n)
        .map(|j| {
            let t = j as f64 / COLUMNS as f64;
            let end = smoothstep(0.0, 0.15, t.min(1.0 - t));
            let keep = 0.1 + 0.9 * end;
            p.rho_out[j] - (p.rho_out[j] - p.rho_in[j]) * keep
        })
        .collect();
    for j in 0..n {
        for r in 0..=ROWS {
            let b = row_beta(j, r);
            let d = (b - p.edge[j]) * p.rho_out[j];
            let shade = 1.0 - (1.0 - EDGE_SHADE) * (1.0 - smoothstep(0.0, SHADE_BAND, d));
            outer[j][r] = push(point(p.col_x[j], p.rho_out[j], b, p.pivot), j, shade, 0, &mut out);
        }
        // the edge: a rounded margin between the two faces
        let t = p.rho_out[j] - rho_in[j];
        let mid_rho = 0.5 * (p.rho_out[j] + rho_in[j]);
        let drop = (0.3 * t).min(0.5) / mid_rho;
        wall[j] = push(point(p.col_x[j], mid_rho, p.edge[j] - drop, p.pivot), j, EDGE_SHADE * 0.75, 1, &mut out);
        for r in 0..=ROWS {
            let part = if r == 0 { 3 } else { 2 };
            inner[j][r] = push(point(p.col_x[j], rho_in[j], row_beta(j, r), p.pivot), j, EDGE_SHADE * 0.6, part, &mut out);
        }
    }
    let quad = |t: &mut Vec<[u32; 3]>, a: u32, b: u32, c: u32, d: u32| {
        t.push([a, b, c]);
        t.push([a, c, d]);
    };
    // winding: outward faces seen from outside are counter-clockwise in the eye's frame
    for j in 0..COLUMNS {
        for r in 0..ROWS {
            quad(&mut tris, outer[j][r], outer[j + 1][r], outer[j + 1][r + 1], outer[j][r + 1]);
            quad(&mut tris, inner[j][r], inner[j][r + 1], inner[j + 1][r + 1], inner[j + 1][r]);
        }
        tris.push([outer[j][0], wall[j], wall[j + 1]]);
        tris.push([outer[j][0], wall[j + 1], outer[j + 1][0]]);
        tris.push([wall[j], inner[j][0], inner[j + 1][0]]);
        tris.push([wall[j], inner[j + 1][0], wall[j + 1]]);
        // the top
        quad(&mut tris, outer[j][ROWS], outer[j + 1][ROWS], inner[j + 1][ROWS], inner[j][ROWS]);
    }
    // the two ends
    for (j, flip) in [(0usize, false), (COLUMNS, true)] {
        let mut ring: Vec<u32> = vec![wall[j]];
        ring.extend((0..=ROWS).map(|r| outer[j][r]));
        ring.extend((0..=ROWS).rev().map(|r| inner[j][r]));
        let c = ring[0];
        for k in 1..ring.len() - 1 {
            if flip { tris.push([c, ring[k], ring[k + 1]]) } else { tris.push([c, ring[k + 1], ring[k]]) }
        }
    }
    // the lash strip along the outer edge
    let mut lash: Vec<V3> = Vec::new();
    let mut lt: Vec<[u32; 3]> = Vec::new();
    let tilt = s.lash_tilt.to_radians();
    for j in 0..n {
        let t = j as f64 / COLUMNS as f64;
        // the strip thins to almost nothing at both corners, so it never runs past a corner as a flat wedge
        let taper = (0.03 + 0.97 * (std::f64::consts::PI * t).sin().powf(0.8)) * p.lash[j];
        let len = s.lash_length * taper;
        let thick = s.lash_thickness * (0.2 + 0.8 * taper);
        let rho = p.rho_out[j] - 0.05;
        let b = p.edge[j] + 0.1 / rho;
        let root = point(p.col_x[j], rho, b, p.pivot);
        let radial = [0.0, b.sin(), b.cos()];
        let upward = [0.0, b.cos(), -b.sin()];
        let dir = [0.0, radial[1] * tilt.cos() + upward[1] * tilt.sin(), radial[2] * tilt.cos() + upward[2] * tilt.sin()];
        let tip = [root[0], root[1] + len * dir[1], root[2] + len * dir[2]];
        // the strip's thickness lies across its direction, in the y-z plane
        let across = [0.0, -dir[2], dir[1]];
        let lift = |q: V3| [q[0], q[1] + thick * across[1], q[2] + thick * across[2]];
        lash.extend([root, tip, lift(tip), lift(root)]);
    }
    for j in 0..COLUMNS {
        let a = (4 * j) as u32;
        let b = (4 * (j + 1)) as u32;
        for k in 0..4u32 {
            let k2 = (k + 1) % 4;
            quad(&mut lt, a + k, b + k, b + k2, a + k2);
        }
    }
    for (j, flip) in [(0u32, true), (COLUMNS as u32, false)] {
        let a = 4 * j;
        if flip { quad(&mut lt, a, a + 3, a + 2, a + 1) } else { quad(&mut lt, a, a + 1, a + 2, a + 3) }
    }
    // to the head's frame; the mirror flips the winding
    let fix = |t: [u32; 3]| if frame.sign < 0.0 { [t[0], t[2], t[1]] } else { t };
    out.positions = local.iter().map(|&q| frame.head(q).map(|v| v as f32)).collect();
    out.triangles = tris.into_iter().map(fix).collect();
    out.lash_positions = lash.iter().map(|&q| frame.head(q).map(|v| v as f32)).collect();
    out.lash_triangles = lt.into_iter().map(fix).collect();
    out.pivot = frame.head([0.0, p.pivot[0], p.pivot[1]]);
    // the axis in the head's frame, kept pointing toward the head's +x so that a positive turn closes the lid on both eyes
    let a = frame.dir([1.0, 0.0, 0.0]);
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    out.axis = if a[0] < 0.0 { [-a[0] / l, -a[1] / l, -a[2] / l] } else { [a[0] / l, a[1] / l, a[2] / l] };
    out
}

/// Turns a point of a piece (head frame, metres) by `angle` degrees about its
/// pivot and axis, as the lid bone does. A positive angle closes the lid.
pub fn turn(q: V3, pivot: V3, angle: f64) -> V3 {
    let a = angle.to_radians();
    let (s, c) = a.sin_cos();
    let y = q[1] - pivot[1];
    let z = q[2] - pivot[2];
    [q[0], pivot[1] + y * c - z * s, pivot[2] + y * s + z * c]
}

/// Turns a point of a piece (head frame, metres) by `angle` degrees about the line through `pivot` along `axis` (a unit vector), as
/// the lid bone does. A positive angle closes the lid when the axis points toward the head's +x.
pub fn turn_about(q: V3, pivot: V3, axis: V3, angle: f64) -> V3 {
    let (s, c) = angle.to_radians().sin_cos();
    let v = sub(q, pivot);
    let k = axis;
    let kv = [k[1] * v[2] - k[2] * v[1], k[2] * v[0] - k[0] * v[2], k[0] * v[1] - k[1] * v[0]];
    let kd = k[0] * v[0] + k[1] * v[1] + k[2] * v[2];
    [0, 1, 2].map(|i| pivot[i] + v[i] * c + kv[i] * s + k[i] * kd * (1.0 - c))
}

/// The pivot, axis, closing angle and the numbers of `PieceInfo`, as JSON.
pub fn info_json(p: &EyePiece) -> String {
    let i = &p.info;
    format!(
        "{{\"pivot\":[{},{},{}],\"axis\":[{},{},{}],\"closed_angle\":{:.3},\"closed_angle_centred\":{:.3},\"pivot_back\":{:.3},\"thickness_min\":{:.3},\"thickness_max\":{:.3},\"wide_deg\":{:.3},\"sleepy_deg\":{:.3},\"opening_mm\":{:.3},\"band_mm\":{:.3},\"columns_without_crossing\":{},\"hidden_faces\":{},\"eyeball_radius_mm\":{:.4}}}",
        p.pivot[0], p.pivot[1], p.pivot[2], p.axis[0], p.axis[1], p.axis[2], p.closed_angle, i.closed_angle_centred, p.pivot_back,
        i.thickness_min, i.thickness_max, i.wide_deg, i.sleepy_deg, i.opening_mm, i.band_mm, i.columns_without_crossing, p.hidden_faces.len(), p.eyeball_radius
    )
}
