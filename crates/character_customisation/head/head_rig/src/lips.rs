//! Opening the lip line. Our fitted lips overlap at rest: the upper lip's curl into the mouth hangs up to 2 mm below the top of the lower
//! lip, behind it. KOTOR's lips just meet. When the mouth opens, the hanging curl fills the opening, so ours shows 2 to 3.6 mm less
//! opening than KOTOR's for the same motion. This lifts the upper lip's vertices near the opening until each one sits `gap` above the
//! highest point of the lower lip beneath it, and lifts the vertices above it by less, fading out over `falloff`. The lift fades to
//! nothing toward the mouth corners, where the two lips join.

use crate::V3;

pub struct LipLine<'a> {
    /// The upper lip's vertices near the opening, and the lower lip's.
    pub upper: &'a [usize],
    pub lower: &'a [usize],
    /// The two mouth corners' vertices.
    pub corners: [usize; 2],
}

/// Half the width of the strip of lip that a point along the lip line looks at, in metres.
pub const COLUMN: f64 = 0.0015;
/// The step of the grid along the lip line, and the width of the smoothing along it, in metres.
pub const STEP: f64 = 0.0005;
pub const SIGMA: f64 = 0.0015;

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The upward move of each upper vertex, in the order of `line.upper`. Heights are along +Y, sideways is X.
///
/// Along the lip line, on a grid, it finds the upper lip's lowest point and the lift that puts it `gap` above the lower lip's highest
/// point, and smooths both sideways. Each upper vertex within `falloff` above the lowest point is then moved by squeezing that band:
/// its bottom rises by the full lift and its top stays, so the vertices keep their order and the edge stays smooth.
pub fn open(positions: &[V3], line: &LipLine, gap: f64, falloff: f64) -> Vec<f64> {
    let corner = line
        .corners
        .iter()
        .map(|&c| positions[c][0].abs())
        .sum::<f64>()
        / 2.0;
    let n = (2.0 * corner / STEP).ceil() as usize + 1;
    let xs: Vec<f64> = (0..n).map(|i| -corner + i as f64 * STEP).collect();
    let extreme = |set: &[usize], x: f64, low: bool| {
        let ys = set
            .iter()
            .map(|&v| positions[v])
            .filter(|p| (p[0] - x).abs() < COLUMN)
            .map(|p| p[1]);
        if low {
            ys.fold(f64::INFINITY, f64::min)
        } else {
            ys.fold(f64::NEG_INFINITY, f64::max)
        }
    };
    let raw: Vec<(f64, f64)> = xs
        .iter()
        .map(|&x| {
            let (low, top) = (extreme(line.upper, x, true), extreme(line.lower, x, false));
            if low.is_finite() && top.is_finite() {
                (low, (top + gap - low).max(0.0))
            } else {
                (f64::NAN, 0.0)
            }
        })
        .collect();
    let smoothed: Vec<(f64, f64)> = xs
        .iter()
        .map(|&x| {
            let (mut sw, mut sl, mut sn) = (0.0, 0.0, 0.0);
            for (&xj, &(low, need)) in xs.iter().zip(&raw) {
                if low.is_nan() {
                    continue;
                }
                let w = (-((xj - x) / SIGMA).powi(2) / 2.0).exp();
                sw += w;
                sl += w * low;
                sn += w * need;
            }
            if sw > 0.0 {
                (sl / sw, sn / sw)
            } else {
                (f64::NAN, 0.0)
            }
        })
        .collect();
    let at = |x: f64| -> (f64, f64) {
        let f = ((x + corner) / STEP).clamp(0.0, (n - 1) as f64);
        let i = (f.floor() as usize).min(n - 2);
        let t = f - i as f64;
        let (a, b) = (smoothed[i], smoothed[i + 1]);
        if a.0.is_nan() || b.0.is_nan() {
            return if a.0.is_nan() { b } else { a };
        }
        (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
    };
    line.upper
        .iter()
        .map(|&v| {
            let p = positions[v];
            let (low, lift) = at(p[0]);
            if low.is_nan() || lift <= 0.0 {
                return 0.0;
            }
            let lift = lift.min(0.9 * falloff)
                * (1.0 - smoothstep(corner - 0.008, corner - 0.002, p[0].abs()));
            let t = (p[1] - low).max(0.0);
            if t >= falloff {
                0.0
            } else {
                lift * (falloff - t) / falloff
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hanging_curl_is_lifted_above_the_lower_lip_and_the_lip_above_it_less() {
        // 0, 1: the upper lip's curl and a point 2 mm above it; 2: the lower lip's top; 3, 4: the corners
        let pos = [
            [0.0, -0.001, 0.0],
            [0.0, 0.001, 0.0],
            [0.0, 0.0005, 0.0],
            [0.025, 0.0, 0.0],
            [-0.025, 0.0, 0.0],
        ];
        let line = LipLine {
            upper: &[0, 1],
            lower: &[2],
            corners: [3, 4],
        };
        let lift = open(&pos, &line, 0.0001, 0.0025);
        assert!((pos[0][1] + lift[0] - 0.0006).abs() < 1e-12);
        assert!(lift[1] > 0.0 && lift[1] < lift[0]);
    }

    #[test]
    fn lips_that_already_meet_do_not_move() {
        let pos = [
            [0.0, 0.001, 0.0],
            [0.0, 0.0, 0.0],
            [0.025, 0.0, 0.0],
            [-0.025, 0.0, 0.0],
        ];
        let line = LipLine {
            upper: &[0],
            lower: &[1],
            corners: [2, 3],
        };
        assert_eq!(open(&pos, &line, 0.0001, 0.0025), vec![0.0]);
    }

    #[test]
    fn nothing_moves_at_the_corners() {
        let pos = [
            [0.024, -0.001, 0.0],
            [0.024, 0.0, 0.0],
            [0.025, 0.0, 0.0],
            [-0.025, 0.0, 0.0],
        ];
        let line = LipLine {
            upper: &[0],
            lower: &[1],
            corners: [2, 3],
        };
        assert_eq!(open(&pos, &line, 0.0001, 0.0025), vec![0.0]);
    }
}
