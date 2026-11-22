//! Moves the facial hair's old global sliders of the live page's recipes into hair groups (2026-11-21).
//!
//! `old_beard_groups <recipes.json> <groups dir>`
//!
//! For each head whose recipe has `beard_amount` above 0, it makes a stubble group from the head's `beard_*` values
//! (`HairGroup::stubble_from_old_sliders`) and adds it at the bottom of the head's groups in `<groups dir>/<head>.json`, which it makes
//! when it is missing. A head that already has a stubble group is left as it is. It does not change the recipes.

use head_paint::hair_groups::{GroupKind, HairGroup, HairGroups};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("Usage: old_beard_groups <recipes.json> <groups dir>");
        std::process::exit(2);
    }
    let recipes: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[0]).expect("recipes")).expect("json");
    let dir = PathBuf::from(&args[1]);
    std::fs::create_dir_all(&dir).expect("groups dir");
    let heads = recipes["heads"].as_object().expect("heads");
    for (name, values) in heads {
        let Some(stubble) = HairGroup::stubble_from_old_sliders(|id| values.get(id).and_then(serde_json::Value::as_f64)) else { continue };
        let path = dir.join(format!("{name}.json"));
        let mut groups = std::fs::read_to_string(&path).ok().map_or_else(HairGroups::default, |t| HairGroups::from_json(&t).expect("hair groups"));
        if groups.groups.iter().any(|g| g.kind == GroupKind::Stubble) {
            println!("{name}: already has a stubble group");
            continue;
        }
        groups.groups.push(stubble);
        std::fs::write(&path, groups.to_json()).expect("write");
        println!("{name}: wrote {} with {} groups", path.display(), groups.groups.len());
    }
}
