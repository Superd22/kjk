//! Lists the texels of the layout that more than one triangle covers, at one size, and says whether some given texels are among them.
//! `cargo run --release --example overlap -- <data dir> <resolution> [x,y]...`
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let l = std::path::Path::new(&a[0]).join("layout");
    let uv: Vec<[f64; 2]> = std::fs::read(l.join("uv.f64")).unwrap().chunks_exact(16).map(|b| [f64::from_le_bytes(b[..8].try_into().unwrap()), f64::from_le_bytes(b[8..].try_into().unwrap())]).collect();
    let tris: Vec<[u32; 3]> = std::fs::read(l.join("tris.u32")).unwrap().chunks_exact(12).map(|b| [0, 1, 2].map(|k| u32::from_le_bytes(b[4 * k..4 * k + 4].try_into().unwrap()))).collect();
    let n: usize = a[1].parse().unwrap();
    let mut cover = vec![0u8; n * n];
    let mut by = vec![Vec::new(); n * n];
    for (k, t) in tris.iter().enumerate() {
        head_paint::live::rasterise(&uv, std::slice::from_ref(t), n, |p| {
            let at = p.y as usize * n + p.x as usize;
            cover[at] = cover[at].saturating_add(1);
            by[at].push(k);
        });
    }
    let multi = cover.iter().filter(|&&c| c > 1).count();
    println!("{} texels covered, {multi} by more than one triangle", cover.iter().filter(|&&c| c > 0).count());
    for s in &a[2..] {
        let (x, y) = s.split_once(',').unwrap();
        let at = y.parse::<usize>().unwrap() * n + x.parse::<usize>().unwrap();
        println!("{s}: triangles {:?}", by[at]);
    }
}
