//! The nose as a feature of the slider kit: its measure (`measure.rs`), its zone, its five parts (root, bridge, tip, wings, base) and
//! its 14 named sliders.

use crate::V3;
use crate::feature::{BumpRule, Feature, Files, Measured, OutlineDef, Shot, Spec};
use crate::measure::{PROPORTIONS, Proportion, Surface, Window, NOSE_WINDOW, measure};
use crate::mesh::HeadMesh;
use crate::zone::Zone;

pub struct Nose;

pub static NOSE: Nose = Nose;

pub const OUTLINES: [OutlineDef; 3] = [
    OutlineDef { key: "profile", label: "Side profile", points: 40, widths: false },
    OutlineDef { key: "front", label: "Outline from the front", points: 20, widths: true },
    OutlineDef { key: "base", label: "Base seen from below", points: 31, widths: false },
];

const GROUPS: [&str; 5] = ["root", "bridge", "tip", "wings", "base"];

const SHOTS: [Shot; 3] = [
    Shot { name: "front", yaw: 0.0, pitch: 0.0, pivot: 0.0, window: [-30.0, 30.0, -70.0, 8.0] },
    Shot { name: "side", yaw: 90.0, pitch: 0.0, pivot: 0.0, window: [-5.0, 55.0, -70.0, 8.0] },
    Shot { name: "below", yaw: 0.0, pitch: 55.0, pivot: -35.0, window: [-30.0, 30.0, -65.0, 13.0] },
];

/// The nose measured, in the kit's form.
pub fn measured(n: crate::measure::Nose) -> Measured {
    Measured {
        proportions: n.proportions.to_vec(),
        outlines: n.outlines.to_vec(),
        marks: serde_json::to_value(&n.marks).unwrap_or_default(),
        cuts: n.cuts.to_vec(),
    }
}

fn mark(m: &Measured, key: &str) -> V3 {
    serde_json::from_value(m.marks[key].clone()).expect("a nose landmark")
}

fn allowed(p: V3) -> bool {
    p[0] >= -0.75 && p[2] >= 0.0
}

impl Feature for Nose {
    fn key(&self) -> &'static str {
        "nose"
    }

    fn title(&self) -> &'static str {
        "Nose"
    }

    fn proportions(&self) -> &'static [Proportion] {
        &PROPORTIONS
    }

    fn outlines(&self) -> &'static [OutlineDef] {
        &OUTLINES
    }

    fn specs(&self) -> &'static [Spec] {
        &SPECS
    }

    fn groups(&self) -> &'static [&'static str] {
        &GROUPS
    }

    fn window(&self) -> Window {
        NOSE_WINDOW
    }

    fn region(&self) -> Window {
        Window { x: 25.0, y: [-75.0, 10.0], z: 5.0 }
    }

    fn measure(&self, surface: &Surface) -> Result<Measured, &'static str> {
        measure(surface).map(measured)
    }

    fn zone(&self, mesh: &HeadMesh, base: &Measured) -> Zone {
        Zone::nose(mesh, mark(base, "tip")[1], mark(base, "base")[1])
    }

    fn part_of(&self, p: V3, base: &Measured) -> &'static str {
        let tip = mark(base, "tip");
        if p[1] > 0.2 * tip[1] {
            "root"
        } else if p[1] > tip[1] + 8.0 {
            "bridge"
        } else if p[0].abs() >= 9.0 {
            "wings"
        } else if p[2] > tip[2] - 10.0 {
            "tip"
        } else {
            "base"
        }
    }

    fn bumps(&self) -> BumpRule {
        BumpRule { size_mm: 4.0, spacing_mm: 3.5, min_weight: 0.6, allowed }
    }

    fn miss_words(&self, i: usize, g: f64) -> String {
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

    fn check_bump(&self) -> V3 {
        [28.0, -40.0, 0.0]
    }

    fn shots(&self) -> &'static [Shot] {
        &SHOTS
    }

    fn files(&self) -> Files {
        Files {
            sliders: "web/sliders_r2.json".into(),
            page: "web/page_r2.json".into(),
            made: "web/made_r2".into(),
            report: "report_r2.json".into(),
            named: "named.json".into(),
            marks: "marks.json".into(),
            checks: "nose/checks.json".into(),
            starts: "web/starts.bin".into(),
        }
    }
}


pub const SPECS: [Spec; 14] = [
    Spec {
        measure: "root",
        name: "Root height",
        hint: "How far the top of the nose stands out between the eyes. The upper part of the bridge follows it, so the bridge's line tilts.",
        group: "root",
        parts: &["root", "bridge"],
        repair_parts: &[],
        free: &[],
        loose: &["bridge_height", "front"],
    },
    Spec {
        measure: "root_dip",
        name: "Root dip",
        hint: "How deep the dip is between the brow and the bridge, seen from the side.",
        group: "root",
        parts: &["root"],
        repair_parts: &["bridge"],
        free: &["root", "profile"],
        loose: &[],
    },
    Spec {
        measure: "bridge_height",
        name: "Bridge height",
        hint: "How far the middle of the bridge stands out, from flat to high. The top of the nose between the eyes follows part of the way.",
        group: "bridge",
        parts: &["bridge", "root"],
        repair_parts: &[],
        free: &[],
        loose: &["front", "width_bridge", "root", "root_dip", "bend", "projection", "dorsum"],
    },
    Spec {
        measure: "width_bridge",
        name: "Bridge width",
        hint: "How broad the bridge is between the eyes and the tip.",
        group: "bridge",
        parts: &["bridge"],
        repair_parts: &["root"],
        free: &["front"],
        loose: &[],
    },
    Spec {
        measure: "dorsum",
        name: "Bridge line",
        hint: "Whether the bridge dips in (scooped) or bulges out (humped), seen from the side.",
        group: "bridge",
        parts: &["bridge"],
        repair_parts: &[],
        free: &["profile"],
        loose: &[],
    },
    Spec {
        measure: "bend",
        name: "Bridge bend",
        hint: "Seen from the side, whether the bridge bulges more near the eyes or more near the tip.",
        group: "bridge",
        parts: &["bridge"],
        repair_parts: &["root", "tip"],
        free: &["profile"],
        loose: &[],
    },
    Spec {
        measure: "projection",
        name: "Tip projection",
        hint: "How far the tip stands out from the face.",
        group: "tip",
        parts: &["tip", "base", "wings", "bridge"],
        repair_parts: &[],
        free: &[],
        loose: &["profile", "base", "bridge_height", "dorsum"],
    },
    Spec {
        measure: "tip_turn",
        name: "Tip turn",
        hint: "Whether the tip points up or down.",
        group: "tip",
        parts: &["tip", "base"],
        repair_parts: &["wings", "bridge"],
        free: &["profile"],
        loose: &[],
    },
    Spec {
        measure: "width_tip",
        name: "Tip width",
        hint: "How wide the tip is, seen from below.",
        group: "tip",
        parts: &["tip"],
        repair_parts: &["base"],
        free: &["base"],
        loose: &[],
    },
    Spec {
        measure: "round_below",
        name: "Tip bluntness",
        hint: "Whether the tip is pointed or blunt, seen from below.",
        group: "tip",
        parts: &["tip"],
        repair_parts: &["base"],
        free: &["base"],
        loose: &[],
    },
    Spec {
        measure: "round_profile",
        name: "Tip roundness",
        hint: "Whether the tip is sharp or round, seen from the side.",
        group: "tip",
        parts: &["tip"],
        repair_parts: &["base"],
        free: &["profile"],
        loose: &[],
    },
    Spec {
        measure: "width_wings",
        name: "Wing width",
        hint: "How wide the nose is across its wings, where the nostrils are. The sides of the nose above the wings widen with them.",
        group: "wings",
        parts: &["wings"],
        repair_parts: &["base"],
        free: &[],
        loose: &["base"],
    },
    Spec {
        measure: "flare",
        name: "Wing flare",
        hint: "How far the wings bulge out beyond the sides of the nose above them. More flare also makes the nose wider at the wings.",
        group: "wings",
        parts: &["wings"],
        repair_parts: &["base"],
        free: &["width_wings", "base"],
        loose: &[],
    },
    Spec {
        measure: "length",
        name: "Nose length",
        hint: "How far below the eyes the bottom of the nose sits.",
        group: "base",
        parts: &["tip", "base", "wings", "bridge"],
        repair_parts: &[],
        free: &[],
        loose: &["profile"],
    },
];

