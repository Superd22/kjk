//! Hair for customized heads (roadmap part D). A hairstyle is a few curves and sliders in a small file. From them this crate builds a
//! hair shell over the scalp, solves a flow field around the head (Fu et al. 2007), grows strands along it, and bakes them into what
//! KOTOR can draw: a hair cap, a few cards and a texture of our own, plus hair painted into the head's texture. The plan is in
//! `docs/research/2026-11-19-head-roadmap.md`, part D, and the method in `docs/research/2026-11-19-drawn-and-procedural-hair.md`.

pub mod atlas;
pub mod cap;
pub mod cards;
pub mod curves;
pub mod field;
pub mod frame;
pub mod outline;
pub mod paint;
pub mod shell;
pub mod strands;
pub mod style;
pub mod v3;

use cards::{Mesh, SAMPLES, card_strip, cluster_half_widths, even_points, kmeans, mean_strand};
use field::{Field, Grid, SolveStats, solve, solve_multilevel};
use frame::{HeadFrame, RadialDepth};
use shell::Shell;
use std::time::Instant;
use strands::Strand;
use style::Style;
use v3::{Rng, V3};

/// The tolerance of the field's solves: the residual divided by the right-hand side. The constraint rows dominate both, so the far
/// field only settles, to within 0.005 of the exact answer, below about 1e-8.
pub const TOLERANCE: f64 = 1e-8;
/// The step of the table of distances from the head's centre to its surface, in degrees.
pub const DEPTH_STEP_DEG: f64 = 1.0;
/// How far cards are kept above the shell, in metres.
pub const CARD_MARGIN: f64 = 0.0015;

/// Timings of the steps, in seconds, and the solves' costs.
#[derive(Clone, Debug, Default)]
pub struct Timings {
    pub depth: f64,
    pub constraints: f64,
    /// The three solves of the multilevel start, coarsest first, then the solve with the rule against entering the scalp.
    pub solves: Vec<SolveStats>,
    pub field_total: f64,
    pub strands: f64,
    pub cap_and_cards: f64,
    pub atlas: f64,
}

/// Everything a hairstyle makes on one head, except the painted layer, which needs the head's texture layout.
pub struct Hair {
    pub shell: Shell,
    pub field: Field,
    pub strands: Vec<Strand>,
    pub locks: Vec<curves::LockInfo>,
    pub cap: Mesh,
    pub cards: Mesh,
    pub card_count: usize,
    /// How many of the cards have a second, crossed strip.
    pub crossed_count: usize,
    pub atlas: atlas::Image,
    /// Where the cap and the cards meet the skin, for the painted hair.
    pub outline: outline::Outline,
    pub timings: Timings,
}

/// Lays the cards of one lock along the lock's own curve, which sets their shape and length. The lock's strands set how the cards
/// spread: they are clustered by k-means into `lock.cards` groups, and each card is the curve moved by its group's mean offset from the
/// whole lock's mean strand, with a width from the group's sideways spread. A lock that caught no strand still gets one card on its
/// curve, at the narrowest width. A lock of 0 cards, such as a bun made by the shell's volume, adds none. Returns the number of cards and of crossed cards added.
pub fn add_lock_cards(shell: &Shell, strands: &[Strand], k: usize, lock: &curves::LockInfo, rng: &mut Rng, cards: &mut Mesh) -> (usize, usize) {
    if lock.cards == 0 {
        return (0, 0);
    }
    let out = |p: V3| shell.dir(p);
    let curve = even_points(&lock.curve, SAMPLES);
    let members: Vec<Vec<V3>> = strands.iter().filter(|s| s.lock == Some(k)).map(|s| even_points(&s.points, SAMPLES)).collect();
    let mut placed: Vec<(Vec<V3>, f64, f64)> = Vec::new();
    if members.is_empty() {
        let half = lock.width_mm[0] / 2000.0;
        placed.push((curve.clone(), half, half));
    } else {
        let all: Vec<&Vec<V3>> = members.iter().collect();
        let lock_mean = mean_strand(&all);
        let groups = kmeans(&members, lock.cards, rng);
        for g in 0..lock.cards.min(members.len()) {
            let cluster: Vec<&Vec<V3>> = members.iter().zip(&groups).filter(|(_, x)| **x == g).map(|(s, _)| s).collect();
            if cluster.is_empty() {
                continue;
            }
            let mean = mean_strand(&cluster);
            let (root, tip) = cluster_half_widths(&cluster, &mean, out, lock.width_mm);
            let centre: Vec<V3> = (0..SAMPLES).map(|i| v3::add(curve[i], v3::sub(mean[i], lock_mean[i]))).collect();
            placed.push((centre, root, tip));
        }
    }
    let mut crossed = 0;
    for (centre, root, tip) in &placed {
        let strip = (rng.unit() * atlas::STRIPS as f64) as usize % atlas::STRIPS;
        let mut card = card_strip(centre, lock.segments, *root, *tip, out, atlas::strip_rect(strip), lock.lean_deg);
        if lock.crossed {
            let second = (strip + 1) % atlas::STRIPS;
            card.append(&card_strip(centre, lock.segments, *root, *tip, out, atlas::strip_rect(second), lock.lean_deg + 90.0));
            crossed += 1;
        }
        for p in &mut card.positions {
            *p = shell.push_out_exact(*p, CARD_MARGIN);
        }
        cards.append(&card);
    }
    (placed.len(), crossed)
}

/// The fine grid of a style around a head.
pub fn field_grid(style: &Style, frame: &HeadFrame) -> Grid {
    let reach = frame.radius + 0.06;
    let c = frame.centre;
    Grid::covering([c[0] - reach, c[1] - style.grid.below_mm / 1000.0, c[2] - reach], [c[0] + reach, c[1] + reach, c[2] + reach], style.grid.cells)
}

/// Solves a style's field on a head: the multilevel solve of the curves' constraints, then one more solve with Fu et al.'s rule against
/// hair entering the scalp added.
pub fn solve_field(style: &Style, shell: &Shell, timings: &mut Timings) -> (Field, Vec<curves::LockInfo>) {
    let started = Instant::now();
    let grid = field_grid(style, &shell.frame);
    let placed = curves::PlacedCurves::new(style, shell, grid.cell);
    let t = Instant::now();
    let constraints = placed.constraints(&grid, shell);
    timings.constraints = t.elapsed().as_secs_f64();
    let (first, levels) = solve_multilevel(&grid, |g| if *g == grid { constraints.clone() } else { placed.constraints(g, shell) }, TOLERANCE);
    timings.solves = levels;
    let mut all = constraints;
    all.extend(curves::tangent_rule(&first, shell));
    let (field, last) = solve(&grid, &all, Some(first.values), TOLERANCE, 4000);
    timings.solves.push(last);
    timings.field_total = started.elapsed().as_secs_f64();
    (field, placed.locks)
}

/// Builds a hairstyle on a head. `positions` and `triangles` are the head's mesh, and `eye_y` the height of its eyes, below which the
/// sphere of the head's frame is not fitted.
pub fn build(style: &Style, positions: &[V3], triangles: &[[u32; 3]], eye_y: f64) -> Hair {
    let style = style.expanded();
    let mut timings = Timings::default();
    let t = Instant::now();
    let frame = HeadFrame::fit(positions, eye_y + 0.01);
    let depth = RadialDepth::build(frame.centre, positions, triangles, DEPTH_STEP_DEG);
    let mut shell = Shell::new(frame, depth, &style);
    shell.skin = Some(shell::Skin::new(positions, triangles));
    timings.depth = t.elapsed().as_secs_f64();

    let (field, locks) = solve_field(&style, &shell, &mut timings);

    let t = Instant::now();
    let strands = strands::grow_all(&style, &shell, &field, &locks);
    timings.strands = t.elapsed().as_secs_f64();

    let t = Instant::now();
    let mut rng = Rng::new(style.seed ^ 0xC4AD);
    let region: Vec<V3> = (0..20000).map(|_| rng.direction()).filter(|d| shell.hairline_mm(*d) > -style.cap.fade_mm).collect();
    let projection = cap::Projection::fitting(&region, 4.0);
    let cap = cap::cap_mesh(&shell, &projection, style.cap.cell_mm, style.cap.fade_mm, atlas::cap_rect());
    let mut cards = Mesh::default();
    let mut card_count = 0;
    let mut crossed_count = 0;
    for (k, lock) in locks.iter().enumerate() {
        let (count, crossed) = add_lock_cards(&shell, &strands, k, lock, &mut rng, &mut cards);
        card_count += count;
        crossed_count += crossed;
    }
    timings.cap_and_cards = t.elapsed().as_secs_f64();

    let t = Instant::now();
    let mut atlas_img = atlas::Image::new(atlas::SIZE, atlas::SIZE);
    atlas::bake_cap(&mut atlas_img, &shell, &projection, &strands, &style.colour, style.cap.fade_mm, style.seed);
    for k in 0..atlas::STRIPS {
        atlas::bake_strip(&mut atlas_img, k, &style.colour, style.seed);
    }
    let outline = outline::Outline::new(&shell, &projection, &atlas_img, &cards);
    timings.atlas = t.elapsed().as_secs_f64();

    Hair { shell, field, strands, locks, cap, cards, card_count, crossed_count, atlas: atlas_img, outline, timings }
}
