//! A made nose against its target, in words: which measures pass their marks, and what each miss gets wrong.

use crate::measure::{N_PROPORTIONS, Nose, OUTLINES, PROPORTIONS, outline_difference};
use crate::round1::solve::Scale;
use serde_json::{Value, json};

pub fn judge(nose: &Nose, target: &Nose, marks: &Scale) -> Value {
    let mut pass = Vec::new();
    let mut gap = Vec::new();
    let mut words = Vec::new();
    for i in 0..N_PROPORTIONS {
        let g = nose.proportions[i] - target.proportions[i];
        let ok = g.abs() <= marks.proportions[i];
        pass.push(ok);
        gap.push(g);
        if !ok {
            words.push(miss_words(i, g));
        }
    }
    for (o, outline) in OUTLINES.iter().enumerate() {
        let d = outline_difference(o, &target.outlines[o], &nose.outlines[o], marks.references[o]);
        let ok = d <= marks.outlines[o];
        pass.push(ok);
        gap.push(d);
        if !ok {
            words.push(format!(
                "The {} differs in shape by {d:.2} mm on average (the mark is {:.2} mm).",
                outline.label.to_lowercase(),
                marks.outlines[o]
            ));
        }
    }
    json!({"pass": pass, "gap": gap, "made": pass.iter().all(|&p| p), "words": words})
}

pub fn miss_words(i: usize, g: f64) -> String {
    let a = g.abs();
    let (more, less) = match PROPORTIONS[i].key {
        "width_wings" => ("The nose is too wide at the wings", "The nose is too narrow at the wings"),
        "width_bridge" => ("The bridge is too broad", "The bridge is too thin"),
        "width_tip" => ("The tip is too wide seen from below", "The tip is too narrow seen from below"),
        "length" => ("The nose is too long", "The nose is too short"),
        "projection" => ("The tip stands out too far", "The tip does not stand out enough"),
        "root" => ("The top of the nose between the eyes stands out too far", "The top of the nose between the eyes is too low"),
        "tip_turn" => ("The tip turns up too much", "The tip does not turn up enough"),
        "dorsum" => ("The bridge is too humped", "The bridge is too scooped"),
        "flare" => ("The wings bulge out too much", "The wings do not bulge out enough"),
        "bridge_height" => ("The middle of the bridge stands out too far", "The middle of the bridge is too flat"),
        "round_profile" => ("The tip is too round seen from the side", "The tip is too sharp seen from the side"),
        "round_below" => ("The tip is too blunt seen from below", "The tip is too pointed seen from below"),
        _ => ("Too much", "Too little"),
    };
    let unit = if PROPORTIONS[i].unit == "degrees" { "degrees" } else { "mm" };
    format!("{} by {a:.1} {unit}.", if g > 0.0 { more } else { less })
}
