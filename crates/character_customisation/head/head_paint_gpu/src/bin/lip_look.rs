//! Looks at KOTOR's mouths before the lip layer is built (paint track, part C3). Analysis only.
//!
//! `lip_look <out dir> [name...]`
//!
//! It finds our mesh's lip border and opening as chains of mesh edges through the MediaPipe lip landmarks of `lip_vertices.json`, prints
//! the chains, and for each head writes KOTOR's own mouth seen from the front, 70 by 44 mm at 8 pixels per millimetre, with and without
//! our chains drawn on it.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::{Common, KotorFront};
use head_paint::folds::Landmarks;
use std::collections::{BinaryHeap, HashMap};
use std::path::{Path, PathBuf};

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
const LIPS: &str = "/Users/david/.claude/tools/likeness-fit/lip_vertices.json";

const UPPER_OUTER: [u32; 11] = [61, 185, 40, 39, 37, 0, 267, 269, 270, 409, 291];
const LOWER_OUTER: [u32; 11] = [61, 146, 91, 181, 84, 17, 314, 405, 321, 375, 291];
const UPPER_INNER: [u32; 11] = [78, 191, 80, 81, 82, 13, 312, 311, 310, 415, 308];
const LOWER_INNER: [u32; 11] = [78, 95, 88, 178, 87, 14, 317, 402, 318, 324, 308];

fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap().chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().unwrap())).collect()
}

fn positions(name: &str) -> Vec<[f64; 3]> {
    f64s(&Path::new(DATA).join("heads").join(format!("{name}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect()
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// The shortest chain of mesh edges from `a` to `b`, both ends included.
fn path(adj: &HashMap<u32, Vec<u32>>, pos: &[[f64; 3]], a: u32, b: u32) -> Vec<u32> {
    #[derive(PartialEq)]
    struct Item(f64, u32);
    impl Eq for Item {}
    impl PartialOrd for Item {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for Item {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            o.0.partial_cmp(&self.0).unwrap()
        }
    }
    let mut best: HashMap<u32, (f64, u32)> = HashMap::new();
    let mut heap = BinaryHeap::new();
    best.insert(a, (0.0, a));
    heap.push(Item(0.0, a));
    while let Some(Item(d, v)) = heap.pop() {
        if v == b {
            break;
        }
        if d > best[&v].0 {
            continue;
        }
        for &w in &adj[&v] {
            let nd = d + dist(pos[v as usize], pos[w as usize]);
            if best.get(&w).is_none_or(|e| nd < e.0) {
                best.insert(w, (nd, v));
                heap.push(Item(nd, w));
            }
        }
    }
    if !best.contains_key(&b) {
        panic!("no path from {a} to {b}: {} reached", best.len());
    }
    let mut out = vec![b];
    while *out.last().unwrap() != a {
        out.push(best[out.last().unwrap()].1);
    }
    out.reverse();
    out
}

fn chain(adj: &HashMap<u32, Vec<u32>>, pos: &[[f64; 3]], lips: &serde_json::Value, marks: &[u32]) -> Vec<u32> {
    let v: Vec<u32> = marks.iter().map(|m| lips[m.to_string()].as_u64().unwrap() as u32).collect();
    let mut out = vec![v[0]];
    for w in v.windows(2) {
        out.extend(path(adj, pos, w[0], w[1]).into_iter().skip(1));
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = PathBuf::from(&args[0]);
    std::fs::create_dir_all(&out).unwrap();
    let common = Common::load();
    let lips: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(LIPS).unwrap()).unwrap();
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for t in &common.mesh_tris {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            adj.entry(a).or_default().push(b);
            adj.entry(b).or_default().push(a);
        }
    }
    let base = common.base.clone();
    let chains: Vec<(&str, Vec<u32>)> = [("upper_outer", &UPPER_OUTER), ("lower_outer", &LOWER_OUTER), ("upper_inner", &UPPER_INNER), ("lower_inner", &LOWER_INNER)]
        .into_iter()
        .map(|(n, m)| (n, chain(&adj, &base, &lips, m)))
        .collect();
    for (n, c) in &chains {
        let len: f64 = c.windows(2).map(|w| dist(base[w[0] as usize], base[w[1] as usize])).sum();
        println!("{n}: {} vertices, {:.1} mm: {:?}", c.len(), 1000.0 * len, c);
    }
    let names: Vec<String> = if args.len() > 1 { args[1..].to_vec() } else { fold_common::head_names() };
    let colours = [[0u8, 255, 0], [0, 200, 255], [255, 255, 0], [255, 0, 255]];
    for name in &names {
        let pos = positions(name);
        let m = Landmarks::new(&pos);
        let f = m.frame;
        let kotor = KotorFront::open(name, f.across, f.up, f.forward);
        let centre_3d = [0, 1, 2].map(|k| 0.5 * (m.mouth_corner[0][k] + m.mouth_corner[1][k]) - 1.0 * f.up[k]);
        let q = kotor.project(centre_3d).0;
        let (w, h, per) = (70.0, 44.0, 8.0);
        let img = kotor.tile_at(q, w, h, per);
        img.save(out.join(format!("{name}_kotor.jpg"))).unwrap();
        let mut over = img.clone();
        for ((_, c), col) in chains.iter().zip(colours) {
            for pair in c.windows(2) {
                let a = kotor.project(pos[pair[0] as usize].map(|x| 1000.0 * x)).0;
                let b = kotor.project(pos[pair[1] as usize].map(|x| 1000.0 * x)).0;
                for s in 0..=40 {
                    let t = f64::from(s) / 40.0;
                    let p = [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])];
                    let x = ((p[0] - (q[0] - 0.5 * w)) * per) as i64;
                    let y = (((q[1] + 0.5 * h) - p[1]) * per) as i64;
                    if x >= 0 && y >= 0 && x < i64::from(over.width()) && y < i64::from(over.height()) {
                        over.put_pixel(x as u32, y as u32, image::Rgb(col));
                    }
                }
            }
        }
        over.save(out.join(format!("{name}_overlay.jpg"))).unwrap();
        println!("{name}");
    }
}
