//! The nose sliders, learned from the KOTOR heads.
//!
//! Each training head gives its nose's own shape (`zone::Zone::own_shape` of our mesh wrapped onto the KOTOR head) and its measured
//! proportions (on KOTOR's own mesh). The sliders come in two kinds.
//! - Proportion sliders: how the nose's shape changes with one named proportion while the other named proportions stay the same,
//!   by least squares over the training heads. Each is then scaled so one unit is one millimetre of its proportion on the base head.
//! - Shape sliders: the main ways in which the noses' shapes still differ once the proportion sliders have explained what they can
//!   (the leading singular vectors of what is left, without taking out the average, so the average KOTOR nose is reachable from the
//!   base head's nose). One unit is one spread (standard deviation) over the training heads.

use crate::V3;
use crate::measure::{N_PROPORTIONS, PROPORTIONS};
use crate::set::{Slider, SliderSet};
use nalgebra::{DMatrix, SymmetricEigen};

pub struct Example {
    /// The nose's own shape, per vertex of the head, in metres.
    pub shape: Vec<V3>,
    pub proportions: [f64; N_PROPORTIONS],
}

/// The proportions that get a slider of their own, as indices into `PROPORTIONS`, in slider order.
pub const NAMED: [usize; 4] = [0, 3, 4, 1];

/// Learns the sliders: the named proportion sliders, then `shapes` learned shape sliders. `calibrate` gives, for a movement, how much
/// its proportion changes on the base head per unit; the proportion sliders are divided by it. Returns the set and each slider's
/// spread over the training heads in its own units.
pub fn learn(
    examples: &[Example],
    shapes: usize,
    calibrate: &dyn Fn(&[V3], usize) -> f64,
) -> (SliderSet, Vec<f64>) {
    let n = examples.len();
    let vc = examples[0].shape.len();
    let p = 3 * vc;
    let x = DMatrix::from_fn(n, p, |i, j| examples[i].shape[j / 3][j % 3]);
    let q = NAMED.len();
    let mean: Vec<f64> = NAMED
        .iter()
        .map(|&k| examples.iter().map(|e| e.proportions[k]).sum::<f64>() / n as f64)
        .collect();
    let pc = DMatrix::from_fn(n, q, |i, a| examples[i].proportions[NAMED[a]] - mean[a]);
    let xm = DMatrix::from_fn(1, p, |_, j| {
        (0..n).map(|i| x[(i, j)]).sum::<f64>() / n as f64
    });
    let xc = DMatrix::from_fn(n, p, |i, j| x[(i, j)] - xm[(0, j)]);
    let mut ptp = pc.transpose() * &pc;
    for a in 0..q {
        ptp[(a, a)] += 1e-6;
    }
    let b = ptp
        .cholesky()
        .expect("the proportions vary")
        .solve(&(pc.transpose() * &xc));

    let mut sliders = Vec::new();
    let mut spread = Vec::new();
    for (a, &k) in NAMED.iter().enumerate() {
        let movement: Vec<V3> = (0..vc)
            .map(|v| [b[(a, 3 * v)], b[(a, 3 * v + 1)], b[(a, 3 * v + 2)]])
            .collect();
        let per_unit = calibrate(&movement, k);
        let movement: Vec<V3> = movement.iter().map(|m| m.map(|c| c / per_unit)).collect();
        sliders.push(Slider::from_movement(
            PROPORTIONS[k].label,
            "proportion",
            &format!(
                "1 {} of {}",
                PROPORTIONS[k].unit,
                PROPORTIONS[k].label.to_lowercase()
            ),
            &movement,
        ));
        let sd = (pc.column(a).iter().map(|v| v * v).sum::<f64>() / n as f64).sqrt();
        spread.push(sd);
    }

    let rest = &x - &pc * &b;
    let gram = &rest * rest.transpose();
    let eig = SymmetricEigen::new(gram);
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| eig.eigenvalues[j].total_cmp(&eig.eigenvalues[i]));
    for (s, &i) in order.iter().take(shapes).enumerate() {
        let lambda = eig.eigenvalues[i].max(1e-30);
        let a = eig.eigenvectors.column(i);
        let sv = lambda.sqrt();
        // a right singular vector of `rest`, scaled to one spread over the training heads
        let mut dir = rest.transpose() * a;
        dir /= sv;
        let unit = sv / (n as f64).sqrt();
        let flip = if a.iter().sum::<f64>() < 0.0 {
            -1.0
        } else {
            1.0
        };
        let movement: Vec<V3> = (0..vc)
            .map(|v| [dir[3 * v], dir[3 * v + 1], dir[3 * v + 2]].map(|c| c * unit * flip))
            .collect();
        sliders.push(Slider::from_movement(
            &format!("Shape {}", s + 1),
            "shape",
            "1 spread of the KOTOR noses",
            &movement,
        ));
        spread.push(1.0);
    }
    (
        SliderSet {
            vertex_count: vc,
            sliders,
            fields: Vec::new(),
        },
        spread,
    )
}
