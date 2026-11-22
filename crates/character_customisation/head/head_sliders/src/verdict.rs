//! A made feature against its target, in words: which measures pass their marks, and what each miss gets wrong. A measure that cannot
//! be read on the target (hair or cloth over it) neither passes nor fails: its pass is null and it does not count.

use crate::feature::{Feature, Measured, Scale, outline_difference};
use serde_json::{Value, json};

pub fn judge(feature: &dyn Feature, made: &Measured, target: &Measured, marks: &Scale) -> Value {
    let mut pass = Vec::new();
    let mut gap = Vec::new();
    let mut words = Vec::new();
    let mut ok_all = true;
    for i in 0..feature.proportions().len() {
        let g = made.proportions[i] - target.proportions[i];
        if g.is_nan() {
            pass.push(Value::Null);
            gap.push(Value::Null);
            continue;
        }
        let ok = g.abs() <= marks.proportions[i];
        ok_all &= ok;
        pass.push(json!(ok));
        gap.push(json!(g));
        if !ok {
            words.push(feature.miss_words(i, g));
        }
    }
    for (o, outline) in feature.outlines().iter().enumerate() {
        if target.outlines[o].is_empty() || made.outlines[o].is_empty() {
            pass.push(Value::Null);
            gap.push(Value::Null);
            continue;
        }
        let d = outline_difference(outline, &target.outlines[o], &made.outlines[o], marks.references[o]);
        let ok = d <= marks.outlines[o];
        ok_all &= ok;
        pass.push(json!(ok));
        gap.push(json!(d));
        if !ok {
            words.push(format!(
                "The {} differs in shape by {d:.2} mm on average (the mark is {:.2} mm).",
                outline.label.to_lowercase(),
                marks.outlines[o]
            ));
        }
    }
    json!({"pass": pass, "gap": gap, "made": ok_all, "words": words})
}
