//! Paints heads natively on the GPU, as references for the browser's textures (`~/.claude/tools/likeness-fit/paint/gpu/native_browser_check.sh`).
//!
//! `browser_ref <data dir> <out dir> <resolution> <switches> <name>...`
//!
//! - `data dir` is the live paint page's data (`hq/paint/live`): the layout, each head's positions, each head's recipe and its hair groups.
//! - `switches` holds one digit per layer in the order of `controls::GROUPS`, such as `111111111` for every layer with the maps.
//! - For each head it writes `<name>.rgb`, the colour texture as RGB bytes with rows from the top, and `<name>.layers`, one 16-bit
//!   little-endian word per texel. Bit g of the word is set when switching layer g off changes that texel by more than 2 steps of 255, so
//!   the word names the layers that paint there.
//! - It writes `layers.json` with the layers' ids in the order of the bits.

use head_paint::controls::{GROUPS, controls, layers_from, recipe_from, switches_of};
use head_paint::live::Layout;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn read<T>(path: &Path, size: usize, from: impl Fn(&[u8]) -> T) -> Vec<T> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    bytes.chunks_exact(size).map(from).collect()
}

fn f64s(path: &Path) -> Vec<f64> {
    read(path, 8, |b| f64::from_le_bytes(b.try_into().expect("8 bytes")))
}

fn u32s(path: &Path) -> Vec<u32> {
    read(path, 4, |b| u32::from_le_bytes(b.try_into().expect("4 bytes")))
}

fn rgb(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4).flat_map(|t| [t[0], t[1], t[2]]).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 5 {
        eprintln!("Usage: browser_ref <data dir> <out dir> <resolution> <switches> <name>...");
        std::process::exit(2);
    }
    let data = PathBuf::from(&args[0]);
    let out = PathBuf::from(&args[1]);
    std::fs::create_dir_all(&out).expect("out dir");
    let n: u32 = args[2].parse().expect("resolution");
    let switches = switches_of(&layers_from(&args[3].chars().map(|c| c == '1').collect::<Vec<_>>()));
    let recipes: Value = serde_json::from_str(&std::fs::read_to_string(data.join("recipes.json")).expect("recipes.json")).expect("json");
    let l = data.join("layout");
    let triples = |v: Vec<u32>| v.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect::<Vec<_>>();
    let layout = Layout {
        uv: f64s(&l.join("uv.f64")).chunks_exact(2).map(|p| [p[0], p[1]]).collect(),
        triangles: triples(u32s(&l.join("tris.u32"))),
        layout_to_mesh: u32s(&l.join("vmap.u32")).into_iter().map(|i| i as usize).collect(),
        mesh_triangles: triples(u32s(&l.join("mesh_tris.u32"))),
    };
    let mut painter = GpuPainter::new(Gpu::blocking().expect("GPU"), layout);
    let ids: Vec<&str> = GROUPS.iter().map(|g| g.id).collect();
    std::fs::write(out.join("layers.json"), json!({ "layers": ids }).to_string()).expect("layers.json");
    for name in &args[4..] {
        let head = &recipes["heads"][name.as_str()];
        assert!(head.is_object(), "{name} has no recipe");
        let positions: Vec<[f64; 3]> = f64s(&data.join("heads").join(format!("{name}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect();
        painter.set_head(positions);
        // The head's facial hair is its hair groups, from `groups/<name>.json` when there is one, as the page loads them.
        *painter.hair_groups_mut() = std::fs::read_to_string(data.join("groups").join(format!("{name}.json"))).ok().and_then(|t| head_paint::hair_groups::HairGroups::from_json(&t)).unwrap_or_default();
        pollster::block_on(painter.prepare(n));
        let values: Vec<f64> = controls().iter().map(|c| head[&c.id].as_f64().unwrap_or(c.default)).collect();
        let recipe = recipe_from(&values);
        let layers = layers_from(&switches);
        painter.paint(n, &recipe, &layers);
        let full = rgb(&pollster::block_on(painter.read_rgba(n)));
        let mut words = vec![0u16; full.len() / 3];
        for g in 0..GROUPS.len() {
            if !switches[g] {
                continue;
            }
            let mut off = switches.clone();
            off[g] = false;
            painter.paint(n, &recipe, &layers_from(&off));
            let without = rgb(&pollster::block_on(painter.read_rgba(n)));
            for (i, w) in words.iter_mut().enumerate() {
                if (0..3).any(|c| full[3 * i + c].abs_diff(without[3 * i + c]) > 2) {
                    *w |= 1 << g;
                }
            }
        }
        std::fs::write(out.join(format!("{name}.rgb")), &full).expect("rgb");
        std::fs::write(out.join(format!("{name}.layers")), words.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<_>>()).expect("layers");
        println!("{name}: written");
    }
}
