//! The style's curves placed on a head, and the field constraints they make.

use crate::field::{Constraint, Field, Grid, Guide, resample_polyline};
use crate::frame::direction;
use crate::shell::Shell;
use crate::style::Style;
use crate::v3::{V3, cross, dot, length, mad, normalize, reject, scale, sub};

/// Fu et al.'s parting: the Gaussian through each point of the dividing curve has a width of 1 cell and a height of half a cell, and the
/// vector on the curve itself has a magnitude of 0.3.
const PARTING_SIGMA_CELLS: f64 = 1.0;
const PARTING_HEIGHT_CELLS: f64 = 0.5;
const PARTING_CENTRE: f64 = 0.3;
/// Fu et al.'s rule against hair entering the scalp sets the corrected vectors to this magnitude.
const TANGENT_RULE_MAGNITUDE: f64 = 0.3;

/// A style's curves turned into points on one head.
pub struct PlacedCurves {
    /// Every stream curve, and whether it is a lock and which.
    pub streams: Vec<(Vec<V3>, f64, Option<usize>)>,
    /// Every dividing curve, as points on the shell.
    pub partings: Vec<Vec<V3>>,
    /// Every ponytail: its spine, its gather radius, its tightness and its lock.
    pub ponytails: Vec<(Vec<V3>, f64, f64, usize)>,
    /// The locks, in the order their numbers refer to: the style's stream locks, then its ponytails.
    pub locks: Vec<LockInfo>,
}

#[derive(Clone, Debug)]
pub struct LockInfo {
    pub curve: Vec<V3>,
    pub cards: usize,
    pub width_mm: [f64; 2],
    pub segments: usize,
    pub crossed: bool,
    pub lean_deg: f64,
    /// Strands closer than this to the curve, on average, follow it.
    pub reach: f64,
}

/// A curve point `[az, el, lift_mm]` as a point in the head's frame.
pub fn place(shell: &Shell, p: [f64; 3]) -> V3 {
    shell.point(direction(p[0], p[1]), p[2] / 1000.0)
}

impl PlacedCurves {
    /// `cell` is the size of the finest grid's cells, which sets how far a lock reaches.
    pub fn new(style: &Style, shell: &Shell, cell: f64) -> Self {
        let mut locks = Vec::new();
        let mut streams = Vec::new();
        for s in &style.streams {
            let pts: Vec<V3> = s.points.iter().map(|&p| place(shell, p)).collect();
            let lock = s.lock.as_ref().map(|l| {
                locks.push(LockInfo { curve: pts.clone(), cards: l.cards, width_mm: l.width_mm, segments: l.segments, crossed: l.crossed, lean_deg: l.lean_deg, reach: s.reach_cells.max(1.0) * cell });
                locks.len() - 1
            });
            streams.push((pts, s.reach_cells, lock));
        }
        let partings = style.partings.iter().map(|p| p.points.iter().map(|q| place(shell, [q[0], q[1], 0.0])).collect()).collect();
        let mut ponytails = Vec::new();
        for p in &style.ponytails {
            let spine: Vec<V3> = p.spine.iter().map(|&q| place(shell, q)).collect();
            locks.push(LockInfo { curve: spine.clone(), cards: p.lock.cards, width_mm: p.lock.width_mm, segments: p.lock.segments, crossed: p.lock.crossed, lean_deg: p.lock.lean_deg, reach: (p.gather_mm / 1000.0).max(cell) });
            ponytails.push((spine, p.gather_mm / 1000.0, p.tightness, locks.len() - 1));
        }
        PlacedCurves { streams, partings, ponytails, locks }
    }

    /// The constraints of every curve on a grid.
    pub fn constraints(&self, grid: &Grid, shell: &Shell) -> Vec<Constraint> {
        let cell = grid.cell;
        let mut out = Vec::new();
        for (pts, reach, _) in &self.streams {
            out.extend(Guide::stream(pts, cell * 0.25, cell * 2.0).constraints(grid, reach * cell));
        }
        for parting in &self.partings {
            out.extend(parting_constraints(grid, shell, parting));
        }
        for (spine, gather, tightness, _) in &self.ponytails {
            out.extend(Guide::stream(spine, cell * 0.25, cell * 2.0).constraints(grid, cell * 1.2));
            if let Some(&start) = spine.first() {
                grid.near(start, *gather + cell, |v, d| {
                    if d > cell * 0.5 {
                        out.push(Constraint { vertex: v, target: scale(normalize(sub(start, grid.position(v))), *tightness), weight: 0.5 });
                    }
                });
            }
        }
        out
    }
}

/// The constraints of one dividing curve. At each point of the curve, with the curve's tangent `t`, the shell's normal `n` and the side
/// `s = n x t`, the grid points at 1 and 2 cells on each side point away from the curve along the slope of a Gaussian bump centred on it.
pub fn parting_constraints(grid: &Grid, shell: &Shell, curve: &[V3]) -> Vec<Constraint> {
    let cell = grid.cell;
    let pts = resample_polyline(curve, cell * 0.5);
    let mut samples: Vec<(V3, V3)> = Vec::new();
    for i in 0..pts.len() {
        let (a, b) = (pts[i.saturating_sub(1)], pts[(i + 1).min(pts.len() - 1)]);
        let p = pts[i];
        let n = shell.normal(p);
        let t = normalize(reject(sub(b, a), n));
        if length(t) == 0.0 {
            continue;
        }
        let s = normalize(cross(n, t));
        samples.push((mad(p, n, cell * 0.5), scale(n, PARTING_CENTRE)));
        for x in [-2.0f64, -1.0, 1.0, 2.0] {
            let slope = -x.abs() / (PARTING_SIGMA_CELLS * PARTING_SIGMA_CELLS) * PARTING_HEIGHT_CELLS * (-x * x / (2.0 * PARTING_SIGMA_CELLS * PARTING_SIGMA_CELLS)).exp();
            let q = shell.point(shell.dir(mad(p, s, x * cell)), cell * 0.5);
            samples.push((q, normalize(mad(scale(s, x.signum()), n, slope))));
        }
    }
    nearest_sample_constraints(grid, &samples, cell * 0.75, 1.0)
}

/// Each vertex within `reach` of a sample takes the target of its nearest sample.
fn nearest_sample_constraints(grid: &Grid, samples: &[(V3, V3)], reach: f64, weight: f64) -> Vec<Constraint> {
    let mut best: std::collections::HashMap<usize, (f64, V3)> = std::collections::HashMap::new();
    for &(p, target) in samples {
        grid.near(p, reach, |v, d| {
            let e = best.entry(v).or_insert((f64::INFINITY, [0.0; 3]));
            if d < e.0 {
                *e = (d, target);
            }
        });
    }
    let mut out: Vec<Constraint> = best.into_iter().map(|(vertex, (_, target))| Constraint { vertex, target, weight }).collect();
    out.sort_by_key(|c| c.vertex);
    out
}

/// Fu et al.'s rule against hair diving into the scalp: every grid vertex near or inside the shell whose vector points inward gets a
/// constraint toward that vector's projection on the shell's tangent plane, with a magnitude of 0.3.
pub fn tangent_rule(field: &Field, shell: &Shell) -> Vec<Constraint> {
    let grid = &field.grid;
    let mut out = Vec::new();
    for v in 0..grid.len() {
        let p = grid.position(v);
        let above = shell.above_shell(p);
        if !(-2.0 * grid.cell..=1.5 * grid.cell).contains(&above) {
            continue;
        }
        let t = field.values[v];
        let n = shell.normal(p);
        if dot(t, n) < 0.0 {
            let along = normalize(reject(t, n));
            if length(along) > 0.0 {
                out.push(Constraint { vertex: v, target: scale(along, TANGENT_RULE_MAGNITUDE), weight: 1.0 });
            }
        }
    }
    out
}
