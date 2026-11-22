//! Strands: grown from roots on the shell along the flow field, then given curl or wave.

use crate::field::{Field, resample_polyline};
use crate::frame::direction;
use crate::shell::{Shell, bumps_at};
use crate::style::{Curl, Style};
use crate::v3::{Rng, V3, any_perpendicular, cross, distance, dot, length, mad, normalize, reject, smoothstep, sub};

/// Fu et al. stop a strand where the field's magnitude falls below this.
pub const STOP_MAGNITUDE: f64 = 0.05;
/// Every strand point is kept at least this far above the shell, in metres.
pub const STRAND_MARGIN: f64 = 0.0002;

#[derive(Clone, Debug)]
pub struct Strand {
    pub points: Vec<V3>,
    /// The lock the strand follows, if any.
    pub lock: Option<usize>,
    /// A number from 0 to 1 that sets the strand's own brightness and curl phase.
    pub seed: f64,
}

impl Strand {
    pub fn length(&self) -> f64 {
        self.points.windows(2).map(|w| distance(w[0], w[1])).sum()
    }
}

/// Root directions spread at random over the part of the head inside the hairline.
pub fn roots(shell: &Shell, count: usize, rng: &mut Rng) -> Vec<V3> {
    let mut out = Vec::with_capacity(count);
    let mut tries = 0;
    while out.len() < count && tries < count * 200 {
        tries += 1;
        let d = rng.direction();
        if shell.hairline_mm(d) > 0.0 {
            out.push(d);
        }
    }
    out
}

/// The length of a strand rooted in a direction, in metres.
pub fn strand_length(style: &Style, d: V3) -> f64 {
    bumps_at(&style.length.zones, d, style.length.default_mm) / 1000.0
}

/// Grows one strand from the shell along the field, in steps of `step`, until the field fades, the strand leaves the grid, or it
/// reaches `max_length`. After every step the point is pushed back out of the shell if it went in.
pub fn grow(field: &Field, shell: &Shell, root: V3, step: f64, max_length: f64) -> Vec<V3> {
    let mut p = shell.point(root, STRAND_MARGIN);
    let mut points = vec![p];
    let mut grown = 0.0;
    while grown + 1e-12 < max_length {
        let Some(v) = field.at(p) else { break };
        if length(v) < STOP_MAGNITUDE {
            break;
        }
        let h = step.min(max_length - grown);
        let next = shell.push_out(mad(p, normalize(v), h), STRAND_MARGIN);
        let moved = distance(next, p);
        if moved < h * 0.1 {
            break;
        }
        grown += moved;
        p = next;
        points.push(p);
    }
    points
}

/// A frame along a polyline that does not twist (the Bishop frame): the tangent, then two normals carried from point to point by the
/// smallest rotation. The first normal starts at a right angle to the first tangent, as close to `up` as it can be.
pub fn bishop_frames(points: &[V3], up: V3) -> Vec<[V3; 3]> {
    let n = points.len();
    let mut out = Vec::with_capacity(n);
    if n < 2 {
        return vec![[[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]; n];
    }
    let tangent = |i: usize| normalize(sub(points[(i + 1).min(n - 1)], points[i.saturating_sub(1)]));
    let t0 = tangent(0);
    let mut nrm = normalize(reject(up, t0));
    if length(nrm) == 0.0 {
        nrm = any_perpendicular(t0);
    }
    let mut prev_t = t0;
    for i in 0..n {
        let t = tangent(i);
        nrm = normalize(reject(nrm, t));
        if length(nrm) == 0.0 {
            nrm = any_perpendicular(t);
        }
        if dot(t, prev_t) < 0.0 {
            nrm = any_perpendicular(t);
        }
        prev_t = t;
        out.push([t, nrm, cross(t, nrm)]);
    }
    out
}

/// Adds curl to a strand: each point moves by `amp(s) * (cos(2 pi s / period + phase) N + sin(2 pi s / period + phase) B)`, where s is the
/// length along the strand and (N, B) is its Bishop frame. With `wave`, only the N part is kept, so the strand swings in one plane. The
/// amplitude grows from 0 at the root to full over `ramp_mm`.
pub fn apply_curl(points: &[V3], curl: &Curl, phase: f64, up: V3) -> Vec<V3> {
    if curl.amplitude_mm <= 0.0 || points.len() < 2 {
        return points.to_vec();
    }
    let frames = bishop_frames(points, up);
    let period = curl.period_mm / 1000.0;
    let amp = curl.amplitude_mm / 1000.0;
    let ramp = curl.ramp_mm / 1000.0;
    let mut s = 0.0;
    let mut out = Vec::with_capacity(points.len());
    for (i, &p) in points.iter().enumerate() {
        if i > 0 {
            s += distance(points[i - 1], p);
        }
        let a = amp * if ramp > 0.0 { smoothstep(0.0, ramp, s) } else { 1.0 };
        let angle = std::f64::consts::TAU * s / period + phase;
        let [_, n, b] = frames[i];
        let offset = if curl.wave { crate::v3::scale(n, a * angle.cos()) } else { crate::v3::add(crate::v3::scale(n, a * angle.cos()), crate::v3::scale(b, a * angle.sin())) };
        out.push(crate::v3::add(p, offset));
    }
    out
}

/// The mean distance from a lock's curve to a strand: for each point of the curve, the distance to the strand's nearest point.
pub fn coverage_distance(curve: &[V3], strand: &[V3]) -> f64 {
    if curve.is_empty() || strand.is_empty() {
        return f64::INFINITY;
    }
    let total: f64 = curve.iter().map(|c| strand.iter().map(|p| distance(*c, *p)).fold(f64::INFINITY, f64::min)).sum();
    total / curve.len() as f64
}

/// Grows every strand of a style: roots, growth, a 2 mm resampling, curl, a last push out of the shell, and which lock each follows.
pub fn grow_all(style: &Style, shell: &Shell, field: &Field, locks: &[crate::curves::LockInfo]) -> Vec<Strand> {
    let mut rng = Rng::new(style.seed);
    let roots = roots(shell, style.strands, &mut rng);
    let step = field.grid.cell * 0.5;
    let lock_samples: Vec<Vec<V3>> = locks.iter().map(|l| resample_polyline(&l.curve, 0.004)).collect();
    let mut out = Vec::with_capacity(roots.len());
    for d in roots {
        let raw = grow(field, shell, d, step, strand_length(style, d));
        if raw.len() < 2 {
            continue;
        }
        let seed = rng.unit();
        let fine = resample_polyline(&raw, 0.002);
        let curled = apply_curl(&fine, &style.curl, seed * std::f64::consts::TAU, shell.dir(fine[0]));
        let points: Vec<V3> = curled.into_iter().map(|p| shell.push_out(p, STRAND_MARGIN)).collect();
        let mut lock = None;
        let mut best = f64::INFINITY;
        for (k, (info, samples)) in locks.iter().zip(&lock_samples).enumerate() {
            let near_start = info.curve.first().map(|c| distance(*c, points[0])).unwrap_or(f64::INFINITY);
            if near_start > info.reach * 2.0 {
                continue;
            }
            let c = coverage_distance(samples, &points);
            if c < info.reach && c < best {
                best = c;
                lock = Some(k);
            }
        }
        let points = match lock {
            Some(k) => end_at_curve_end(&points, &locks[k].curve).into_iter().map(|p| shell.push_out_exact(p, STRAND_MARGIN)).collect(),
            None => hold_on_cap(shell, &points, style.hold_mm / 1000.0, style.cap.fade_mm),
        };
        if points.len() >= 2 {
            out.push(Strand { points, lock, seed });
        }
    }
    out
}

/// A lock strand: cut at its point nearest the end of the lock's curve, so that the curve sets how long the lock is.
pub fn end_at_curve_end(points: &[V3], curve: &[V3]) -> Vec<V3> {
    let Some(&end) = curve.last() else { return points.to_vec() };
    let nearest = (0..points.len()).min_by(|&a, &b| distance(points[a], end).total_cmp(&distance(points[b], end))).unwrap_or(points.len() - 1);
    points[..=nearest.max(1).min(points.len() - 1)].to_vec()
}

/// A cap strand: cut where it first leaves the cap, `fade_mm` past the hairline, and pulled down to at most `hold` above the shell.
pub fn hold_on_cap(shell: &Shell, points: &[V3], hold: f64, fade_mm: f64) -> Vec<V3> {
    let mut out = Vec::with_capacity(points.len());
    for &p in points {
        let d = shell.dir(p);
        if shell.hairline_mm(d) < -fade_mm {
            break;
        }
        out.push(if shell.above_shell(p) > hold { shell.point(d, hold) } else { p });
    }
    out
}

/// A direction from angles, for tests and tools.
pub fn dir(az: f64, el: f64) -> V3 {
    direction(az, el)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{Grid, Guide, solve};
    use crate::shell::tests::sphere_shell;

    #[test]
    fn the_helix_has_the_right_period() {
        let points: Vec<V3> = (0..=400).map(|i| [0.0, 0.0, i as f64 * 0.0005]).collect();
        let curl = Curl { amplitude_mm: 2.0, period_mm: 20.0, wave: false, ramp_mm: 0.0 };
        let curled = apply_curl(&points, &curl, 0.3, [0.0, 1.0, 0.0]);
        let angle = |i: usize| {
            let o = sub(curled[i], points[i]);
            o[1].atan2(o[0])
        };
        // 0.5 mm per point, so 40 points make one period of 20 mm.
        for start in [0, 13, 100] {
            let full = (angle(start + 40) - angle(start)).rem_euclid(std::f64::consts::TAU);
            assert!(full < 1e-6 || (std::f64::consts::TAU - full) < 1e-6, "a full period turns a full circle, got {full}");
            let half = (angle(start + 20) - angle(start)).rem_euclid(std::f64::consts::TAU);
            assert!((half - std::f64::consts::PI).abs() < 1e-6, "half a period turns half a circle, got {half}");
        }
        for i in 0..points.len() {
            assert!((distance(curled[i], points[i]) - 0.002).abs() < 1e-9);
        }
    }

    #[test]
    fn a_wave_stays_in_one_plane() {
        let points: Vec<V3> = (0..=200).map(|i| [0.0, 0.0, i as f64 * 0.0005]).collect();
        let curl = Curl { amplitude_mm: 2.0, period_mm: 20.0, wave: true, ramp_mm: 5.0 };
        let curled = apply_curl(&points, &curl, 0.0, [0.0, 1.0, 0.0]);
        assert!(curled.iter().all(|p| p[0].abs() < 1e-12));
        assert!(curled.iter().any(|p| p[1].abs() > 0.0019));
        assert!(curled[0][1].abs() < 1e-12, "no curl at the root");
    }

    /// A field that points straight into the head everywhere would drive strands into the scalp. The push-out must stop them.
    #[test]
    fn strands_never_enter_the_scalp() {
        let shell = sphere_shell();
        let grid = Grid::covering([-0.15, -0.15, -0.15], [0.15, 0.15, 0.15], 20);
        let mut field = Field { grid, values: vec![[0.0; 3]; grid.len()] };
        for v in 0..grid.len() {
            let p = grid.position(v);
            // Inward and swirling: the worst case for a strand.
            field.values[v] = normalize(crate::v3::add(crate::v3::scale(normalize(p), -1.0), [0.3, -0.2, 0.1]));
        }
        let mut rng = Rng::new(7);
        for d in roots(&shell, 200, &mut rng) {
            let s = grow(&field, &shell, d, grid.cell * 0.5, 0.25);
            let curled = apply_curl(&resample_polyline(&s, 0.002), &Curl { amplitude_mm: 3.0, period_mm: 15.0, wave: false, ramp_mm: 0.0 }, 1.0, d);
            for p in curled.into_iter().map(|p| shell.push_out(p, STRAND_MARGIN)) {
                assert!(shell.above_head(p) > 0.0, "a strand point is inside the head");
                assert!(shell.above_shell(p) >= STRAND_MARGIN - 1e-9, "a strand point is inside the shell");
            }
        }
    }

    #[test]
    fn strands_follow_a_stream_over_the_top_of_the_head() {
        let shell = sphere_shell();
        let grid = Grid::covering([-0.15, -0.15, -0.15], [0.15, 0.15, 0.15], 24);
        let curve: Vec<V3> = (0..=10).map(|i| shell.point(dir(0.0, 30.0 + 12.0 * i as f64), 0.0)).collect();
        let constraints = Guide::stream(&curve, grid.cell * 0.25, grid.cell * 2.0).constraints(&grid, grid.cell * 1.5);
        let (field, _) = solve(&grid, &constraints, None, 1e-5, 4000);
        let s = grow(&field, &shell, dir(0.0, 35.0), grid.cell * 0.5, 0.15);
        assert!(s.len() > 5);
        let end = shell.dir(*s.last().expect("points"));
        assert!(end[2] < 0.0, "the strand went over the top to the back: {end:?}");
    }
}
