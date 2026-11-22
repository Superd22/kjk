//! Checks the port against the prototype: `eye_curve_check <input dir> <handles.json> <dump dir> <heads...>`. The dump dir holds
//! `<head>_in.bin` (the live head) and `<head>_out.bin` (the prototype's head), float32 metres. Prints the largest and the mean gap per
//! head, and the time of each build.

use head_eye_curve::V3;
use head_eye_curve::maker::{Maker, Mesh};
use std::path::Path;
use std::time::Instant;

fn read(p: &Path) -> Vec<u8> {
    std::fs::read(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn u32s(p: &Path) -> Vec<u32> {
    read(p).chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

fn points(p: &Path) -> Vec<V3> {
    let f: Vec<f32> = read(p).chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
    f.chunks_exact(3).map(|c| [c[0] as f64, c[1] as f64, c[2] as f64]).collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let input = Path::new(&a[1]);
    let tris: Vec<[u32; 3]> = u32s(&input.join("tris.bin")).as_chunks::<3>().0.to_vec();
    let mesh = Mesh::new(tris, u32s(&input.join("group.bin")), u32s(&input.join("first.bin")), read(&input.join("hidden.bin")).iter().map(|&x| x != 0).collect());
    let t0 = Instant::now();
    let mut maker = Maker::new(mesh);
    println!("prepare: {:?}", t0.elapsed());
    let handles: serde_json::Value = serde_json::from_str(&String::from_utf8(read(Path::new(&a[2]))).unwrap()).unwrap();
    let dump = Path::new(&a[3]);
    for name in &a[4..] {
        let h: Vec<f64> = handles[name.as_str()].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect();
        let built = points(&dump.join(format!("{name}_in.bin")));
        let want = points(&dump.join(format!("{name}_out.bin")));
        let t0 = Instant::now();
        let got = maker.build(&built, &h);
        let ms = t0.elapsed();
        let gaps: Vec<f64> = got.iter().zip(&want).map(|(g, w)| head_eye_curve::length(head_eye_curve::sub(*g, *w)) * 1000.0).collect();
        let moved = got.iter().zip(&built).filter(|(g, b)| head_eye_curve::length(head_eye_curve::sub(**g, **b)) > 1e-7).count();
        let max = gaps.iter().copied().fold(0.0, f64::max);
        let over = gaps.iter().filter(|&&g| g > 0.01).count();
        let t1 = Instant::now();
        let mut h2 = h.clone();
        h2[4] += 0.5;
        let _ = maker.build(&built, &h2);
        let again = t1.elapsed();
        let back = maker.build(&built, &h);
        let same = back.iter().zip(&got).all(|(a, b)| a == b) && maker.build(&built.iter().map(|p| [p[0] + 0.0, p[1], p[2]]).collect::<Vec<_>>(), &h) == got;
        println!("{name}: {ms:?}, a handle moved {again:?}, the same again {same}, {moved} vertices moved, largest gap to the prototype {max:.4} mm, {over} vertices over 0.01 mm, closing angles {:.1} and {:.1}", maker.lids[0].angle, maker.lids[1].angle);
    }
}
