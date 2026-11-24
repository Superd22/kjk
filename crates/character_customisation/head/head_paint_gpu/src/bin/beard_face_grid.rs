//! Makes the facial hair's face grid (`head_paint::face_grid`) on the mean of the 57 test heads and writes its binding to the layout
//! (paint track, part C, layer 6). Run it again only to change the grid's design: values drawn on the old grid would land elsewhere.
//!
//! `beard_face_grid <out bin> [<coverage ppm>]`
//!
//! The skin is first sampled finely on the layout's face piece, 1,024 by 1,024 texels. On each of the layout's columns the jaw's edge is found where the skin turns to face downward, and the mouth's opening where the lips meet. The face
//! grid's rows lie 2 mm apart along the skin below the jaw's edge and above the mouth, and are shared out evenly between the two, so that
//! one row runs along the jaw's edge and one through the mouth's corners. Its columns lie 2 mm apart along the skin from the face's
//! middle line. Each grid point is then bound to the nearest spot of the layout. Below the face piece, under the chin, the columns
//! are carried on down the neck in steps of 2 mm along the skin. Last, the points where no beard can grow are dropped:
//! on the lips and the nose, above the cheek line and the sideburns' top, behind the ear and below the neck line, each with room for the
//! sliders that move these edges. The coverage picture shows each grid point before the trim: white where it is kept, and in a colour
//! for the edge that drops it.

use head_paint::closest::closest_on_triangle;
use head_paint::face_grid::FaceBinding;
use head_paint::facial_hair::{BeardHead, polygon_distance};
use head_paint::folds::Landmarks;
use head_paint::layout_points::{cross, dot, normalize, sub};
use std::path::Path;

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
/// The distance between rows and between columns, in millimetres on the mean head.
const SPACING_MM: f64 = 2.0;
/// The fine sampling's size, in nodes along each side of the layout.
const FINE: usize = 1024;
/// Room left past each edge's default for the slider that moves it, plus the edge's softness, in millimetres.
const CHEEK_ROOM_MM: f64 = 12.0;
const NECK_ROOM_MM: f64 = 24.0;

fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().expect("8"))).collect()
}

fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().expect("4"))).collect()
}

fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d).sqrt()
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn curve_at(c: &[[f64; 2]], u: f64) -> f64 {
    if u <= c[0][0] {
        return c[0][1];
    }
    for w in c.windows(2) {
        if u <= w[1][0] {
            let f = (u - w[0][0]) / (w[1][0] - w[0][0]).max(1e-9);
            return w[0][1] + f * (w[1][1] - w[0][1]);
        }
    }
    c[c.len() - 1][1]
}

#[derive(Clone, Copy)]
struct Node {
    valid: bool,
    pos: [f64; 3],
    normal: [f64; 3],
    tri: u32,
}

/// Fills gaps (NaN) by linear interpolation between the nearest values on each side, and holds the ends flat.
fn fill(v: &mut [f64]) {
    let known: Vec<usize> = (0..v.len()).filter(|&i| v[i].is_finite()).collect();
    if known.is_empty() {
        return;
    }
    for i in 0..v.len() {
        if v[i].is_finite() {
            continue;
        }
        let a = known.iter().rev().find(|&&k| k < i).copied();
        let b = known.iter().find(|&&k| k > i).copied();
        v[i] = match (a, b) {
            (Some(a), Some(b)) => v[a] + (v[b] - v[a]) * (i - a) as f64 / (b - a) as f64,
            (Some(a), None) => v[a],
            (None, Some(b)) => v[b],
            (None, None) => 0.0,
        };
    }
}

fn blur(v: &[f64], reach: usize) -> Vec<f64> {
    (0..v.len()).map(|i| {
        let lo = i.saturating_sub(reach);
        let hi = (i + reach).min(v.len() - 1);
        v[lo..=hi].iter().sum::<f64>() / (hi - lo + 1) as f64
    }).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args.first().expect("Usage: beard_face_grid <out bin>");
    let l = Path::new(DATA).join("layout");
    let triples = |v: Vec<u32>| v.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect::<Vec<_>>();
    let uv: Vec<[f64; 2]> = f64s(&l.join("uv.f64")).chunks_exact(2).map(|p| [p[0], p[1]]).collect();
    let tris = triples(u32s(&l.join("tris.u32")));
    let vmap: Vec<usize> = u32s(&l.join("vmap.u32")).into_iter().map(|i| i as usize).collect();
    let mut names: Vec<String> = std::fs::read_dir(Path::new(DATA).join("heads")).expect("heads").filter_map(|e| e.ok()?.file_name().into_string().ok()?.strip_suffix(".f64").map(String::from)).collect();
    names.sort();
    let mut mean: Vec<[f64; 3]> = Vec::new();
    for n in &names {
        let p: Vec<[f64; 3]> = f64s(&Path::new(DATA).join("heads").join(format!("{n}.f64"))).chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect();
        if mean.is_empty() {
            mean = vec![[0.0; 3]; p.len()];
        }
        for (m, q) in mean.iter_mut().zip(&p) {
            for k in 0..3 {
                m[k] += q[k] / names.len() as f64;
            }
        }
    }
    println!("mean of {} heads", names.len());
    let lips = head_paint::lips::LipHead::new(&mean, &tris, &vmap).expect("lips");
    let m = Landmarks::new(&mean);
    let (across, up, fw) = (m.frame.across, m.frame.up, m.frame.forward);
    let mouth = [0, 1, 2].map(|k| 0.5 * (m.mouth_corner[0][k] + m.mouth_corner[1][k]));
    let at = |p: [f64; 3]| {
        let d = sub(p, mouth);
        [dot(d, across), dot(d, up), dot(d, fw)]
    };
    let scale = dist(m.eye_outer[1], m.eye_outer[0]) / 82.6;
    let centre = add(mouth, fw, -80.0 * scale);
    let chin = at(m.chin_middle);

    // The fine sampling of the skin on the layout's face piece, the one that holds the chin, `FINE` by `FINE` texels of the layout.
    let mm: Vec<[f64; 3]> = vmap.iter().map(|&i| mean[i].map(|x| 1000.0 * x)).collect();
    let mut mesh_normal = vec![[0.0; 3]; mean.len()];
    for t in &tris {
        let v = t.map(|i| mm[i as usize]);
        let n = cross(sub(v[1], v[0]), sub(v[2], v[0]));
        for &i in t {
            for k in 0..3 {
                mesh_normal[vmap[i as usize]][k] += n[k];
            }
        }
    }
    let piece = {
        let mut parent: Vec<usize> = (0..uv.len()).collect();
        fn root(p: &mut [usize], mut i: usize) -> usize {
            while p[i] != i {
                p[i] = p[p[i]];
                i = p[i];
            }
            i
        }
        for t in &tris {
            let a = root(&mut parent, t[0] as usize);
            for &o in &t[1..] {
                let b = root(&mut parent, o as usize);
                parent[b] = a;
            }
        }
        let chin_vertex = (0..mm.len()).min_by(|&a, &b| dist(mm[a], m.chin_middle).total_cmp(&dist(mm[b], m.chin_middle))).expect("vertices");
        let r = root(&mut parent, chin_vertex);
        (0..uv.len()).map(|i| root(&mut parent, i) == r).collect::<Vec<bool>>()
    };
    let (cols, rows) = (FINE, FINE);
    let mut nodes = vec![Node { valid: false, pos: centre, normal: fw, tri: 0 }; cols * rows];
    for (ti, t) in tris.iter().enumerate() {
        if !piece[t[0] as usize] {
            continue;
        }
        let v = t.map(|i| mm[i as usize]);
        let c = t.map(|i| [uv[i as usize][0] * (cols - 1) as f64, uv[i as usize][1] * (rows - 1) as f64]);
        let den = (c[1][1] - c[2][1]) * (c[0][0] - c[2][0]) + (c[2][0] - c[1][0]) * (c[0][1] - c[2][1]);
        if den.abs() < 1e-14 {
            continue;
        }
        let i0 = c[0][0].min(c[1][0]).min(c[2][0]).ceil().max(0.0) as usize;
        let i1 = c[0][0].max(c[1][0]).max(c[2][0]).floor().min(cols as f64 - 1.0) as usize;
        let j0 = c[0][1].min(c[1][1]).min(c[2][1]).ceil().max(0.0) as usize;
        let j1 = c[0][1].max(c[1][1]).max(c[2][1]).floor().min(rows as f64 - 1.0) as usize;
        for j in j0..=j1 {
            for i in i0..=i1 {
                let q = [i as f64, j as f64];
                let w0 = ((c[1][1] - c[2][1]) * (q[0] - c[2][0]) + (c[2][0] - c[1][0]) * (q[1] - c[2][1])) / den;
                let w1 = ((c[2][1] - c[0][1]) * (q[0] - c[2][0]) + (c[0][0] - c[2][0]) * (q[1] - c[2][1])) / den;
                let w = [w0, w1, 1.0 - w0 - w1];
                if w.iter().any(|&x| x < -1e-9) {
                    continue;
                }
                let pos = [0, 1, 2].map(|a| w[0] * v[0][a] + w[1] * v[1][a] + w[2] * v[2][a]);
                let mut n = [0.0; 3];
                for (e, &vi) in t.iter().enumerate() {
                    for a in 0..3 {
                        n[a] += w[e] * mesh_normal[vmap[vi as usize]][a];
                    }
                }
                let n = normalize(n);
                let n = if dot(n, sub(pos, centre)) < 0.0 { n.map(|x| -x) } else { n };
                nodes[j * cols + i] = Node { valid: true, pos, normal: n, tri: ti as u32 };
            }
        }
    }

    // Along each meridian, from the top: the arc length, the jaw's edge and the mouth's opening.
    let ic = (0..cols * rows).filter(|&k| nodes[k].valid).min_by(|&a, &b| dist(nodes[a].pos, m.chin_middle).total_cmp(&dist(nodes[b].pos, m.chin_middle))).expect("chin") % cols;
    let mut arc = vec![f64::NAN; cols * rows];
    let mut jaw = vec![f64::NAN; cols];
    let mut mouth_arc = vec![f64::NAN; cols];
    // The underside of the lower lip faces down too, so in front of the mouth the jaw's edge is looked for only below the chin's front.
    let corner_x = 0.5 * (at(m.mouth_corner[1])[0] - at(m.mouth_corner[0])[0]);
    let below = |x: f64| if x.abs() < corner_x + 8.0 * scale { chin[1] + 2.0 * scale } else { 0.0 };
    let (u_lo, u_hi) = (lips.opening[0][0], lips.opening[lips.opening.len() - 1][0]);
    for i in 0..cols {
        let mut s = 0.0;
        let mut last: Option<usize> = None;
        for j in (0..rows).rev() {
            let k = j * cols + i;
            if !nodes[k].valid {
                continue;
            }
            if let Some(p) = last {
                let (a, b) = (&nodes[p], &nodes[k]);
                let step = dist(a.pos, b.pos);
                if jaw[i].is_nan() && at(b.pos)[1] < below(at(b.pos)[0]) && dot(a.normal, up) >= -0.5 && dot(b.normal, up) < -0.5 {
                    let (na, nb) = (dot(a.normal, up), dot(b.normal, up));
                    jaw[i] = s + step * ((na + 0.5) / (na - nb).max(1e-9)).clamp(0.0, 1.0);
                }
                if mouth_arc[i].is_nan() {
                    let (ua, ub) = (lips.frame.uvw(a.pos), lips.frame.uvw(b.pos));
                    let near = ua[2] > -6.0 && ub[2] > -6.0 && ua[0] > u_lo && ua[0] < u_hi;
                    let (da, db) = (ua[1] - curve_at(&lips.opening, ua[0]), ub[1] - curve_at(&lips.opening, ub[0]));
                    if near && da >= 0.0 && db < 0.0 {
                        mouth_arc[i] = s + step * (da / (da - db)).clamp(0.0, 1.0);
                    }
                }
                s += step;
            }
            arc[k] = s;
            last = Some(k);
        }
    }
    // The mouth's columns run from corner to corner. Beyond the corners the rows between the jaw and the mouth's height fade to 2 mm apart
    // over 20 mm.
    let mut gap: Vec<f64> = (0..cols).map(|i| if mouth_arc[i].is_finite() && jaw[i].is_finite() { jaw[i] - mouth_arc[i] } else { f64::NAN }).collect();
    let mouth_cols: Vec<usize> = (0..cols).filter(|&i| gap[i].is_finite()).collect();
    let (cl, cr) = (*mouth_cols.first().expect("mouth"), *mouth_cols.last().expect("mouth"));
    fill(&mut jaw);
    let jaw = blur(&jaw, 12);
    let n_jm = (gap[ic] / SPACING_MM).round();
    println!("jaw's edge to mouth in the middle: {:.1} mm, {} rows; mouth's columns {}..{} of {}", gap[ic], n_jm, cl, cr, cols);
    let col_mm = {
        let j = (0..rows).filter(|&j| nodes[j * cols + ic].valid && mouth_arc[ic].is_finite() && arc[j * cols + ic] >= mouth_arc[ic]).max().expect("mouth row");
        dist(nodes[j * cols + ic].pos, nodes[j * cols + ic + 1].pos)
    };
    for i in 0..cols {
        if i < cl || i > cr {
            let corner = if i < cl { cl } else { cr };
            let f = smooth(0.0, 20.0, (i as f64 - corner as f64).abs() * col_mm);
            gap[i] = gap[corner] * (1.0 - f) + n_jm * SPACING_MM * f;
        }
    }
    let gap = blur(&gap, 9);

    // The rows (t, from the jaw's edge up) and the columns (s, from the middle line out) of each fine node, in grid steps. The skin is
    // longer across the lips than across the mouth's opening, so between the corners each row's length is shared out evenly, and past
    // the corners the length is counted from the corner. Then the columns do not shift at the mouth's height.
    let mut ra = vec![f64::NAN; cols * rows];
    let mut at_corner = vec![[f64::NAN; 2]; rows];
    for j in 0..rows {
        for (side, dir) in [(0usize, -1i64), (1, 1)] {
            let corner = if side == 0 { cl } else { cr };
            let mut s = 0.0;
            let mut last: Option<usize> = None;
            let mut i = ic as i64;
            while i >= 0 && (i as usize) < cols {
                let k = j * cols + i as usize;
                if nodes[k].valid {
                    if let Some(p) = last {
                        s += dist(nodes[p].pos, nodes[k].pos);
                    }
                    last = Some(k);
                    ra[k] = s;
                    if at_corner[j][side].is_nan() && (i - corner as i64) * dir >= 0 {
                        at_corner[j][side] = s;
                    }
                }
                i += dir;
            }
        }
    }
    let corner_mm = [0, 1].map(|side| {
        let mut v: Vec<f64> = at_corner.iter().map(|c| c[side]).filter(|x| x.is_finite()).collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    });
    let mut st = vec![[f64::NAN; 2]; cols * rows];
    for j in 0..rows {
        for i in 0..cols {
            let k = j * cols + i;
            if !nodes[k].valid {
                continue;
            }
            let side = if i < ic { 0 } else { 1 };
            let (c, r) = (corner_mm[side], at_corner[j][side]);
            let along = if !r.is_finite() { ra[k] } else if ra[k] <= r { c * ra[k] / r.max(1e-6) } else { c + ra[k] - r };
            let a = jaw[i] - arc[k];
            let g = gap[i].max(4.0);
            let t = if a <= 0.0 { a / SPACING_MM } else if a <= g { n_jm * a / g } else { n_jm + (a - g) / SPACING_MM };
            st[k] = [if side == 0 { -along } else { along } / SPACING_MM, t];
        }
    }

    // Each whole (s, t) found among the fine nodes' triangles, and bound to the nearest spot of the layout. The grid spans s from -70 to
    // 70 and t from -40 to 60 before it is trimmed.
    let (smax, tlo, thi) = (70i64, -40i64, 60i64);
    let gc = (2 * smax + 1) as usize;
    let gr = (thi - tlo + 1) as usize;
    let mut found: Vec<Vec<(u32, [f64; 2], [f64; 3])>> = vec![Vec::new(); gc * gr];
    for j in 0..rows - 1 {
        for i in 0..cols - 1 {
            let q = [j * cols + i, j * cols + i + 1, (j + 1) * cols + i, (j + 1) * cols + i + 1];
            for tri in [[q[0], q[1], q[3]], [q[0], q[3], q[2]]] {
                if tri.iter().any(|&k| !nodes[k].valid || st[k][0].is_nan()) {
                    continue;
                }
                // A triangle across a silhouette, such as the lips' edge seen from below, joins skin far apart.
                if (0..3).any(|e| dist(nodes[tri[e]].pos, nodes[tri[(e + 1) % 3]].pos) > 10.0) {
                    continue;
                }
                let c = tri.map(|k| st[k]);
                let den = (c[1][1] - c[2][1]) * (c[0][0] - c[2][0]) + (c[2][0] - c[1][0]) * (c[0][1] - c[2][1]);
                if den.abs() < 1e-12 {
                    continue;
                }
                let lo = [c[0][0].min(c[1][0]).min(c[2][0]).ceil() as i64, c[0][1].min(c[1][1]).min(c[2][1]).ceil() as i64];
                let hi = [c[0][0].max(c[1][0]).max(c[2][0]).floor() as i64, c[0][1].max(c[1][1]).max(c[2][1]).floor() as i64];
                for gt in lo[1].max(tlo)..=hi[1].min(thi) {
                    for gs in lo[0].max(-smax)..=hi[0].min(smax) {
                        let (x, y) = (gs as f64, gt as f64);
                        let w0 = ((c[1][1] - c[2][1]) * (x - c[2][0]) + (c[2][0] - c[1][0]) * (y - c[2][1])) / den;
                        let w1 = ((c[2][1] - c[0][1]) * (x - c[2][0]) + (c[0][0] - c[2][0]) * (y - c[2][1])) / den;
                        let w = [w0, w1, 1.0 - w0 - w1];
                        if w.iter().any(|&v| v < -1e-9) {
                            continue;
                        }
                        let g = (gt - tlo) as usize * gc + (gs + smax) as usize;
                        let p = [0, 1, 2].map(|a| w[0] * nodes[tri[0]].pos[a] + w[1] * nodes[tri[1]].pos[a] + w[2] * nodes[tri[2]].pos[a]);
                        let mut best = (f64::MAX, 0u32, [0.0; 2]);
                        for &k in &tri {
                            let t = nodes[k].tri;
                            let v = tris[t as usize].map(|i| mm[i as usize]);
                            let b = closest_on_triangle(p, v[0], v[1], v[2]);
                            let on = [0, 1, 2].map(|a| b[0] * v[0][a] + b[1] * v[1][a] + b[2] * v[2][a]);
                            let d = dist(on, p);
                            if d < best.0 {
                                best = (d, t, [b[0], b[1]]);
                            }
                        }
                        found[g].push((best.1, best.2, p));
                    }
                }
            }
        }
    }
    // Where the rows and columns fold over on the skin, a whole (s, t) is found at more than one place. The place nearest to the
    // middle of its neighbours' places is kept.
    let first: Vec<Option<[f64; 3]>> = found.iter().map(|f| f.first().map(|c| c.2)).collect();
    let multi = found.iter().filter(|f| f.len() > 1).count();
    let mut bind: Vec<Option<(u32, [f64; 2])>> = vec![None; gc * gr];
    for g in 0..gc * gr {
        let (i, j) = ((g % gc) as i64, (g / gc) as i64);
        let near: Vec<[f64; 3]> = [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter_map(|&(di, dj)| {
                let (a, b) = (i + di, j + dj);
                (a >= 0 && b >= 0 && (a as usize) < gc && (b as usize) < gr).then(|| first[b as usize * gc + a as usize]).flatten()
            })
            .collect();
        let mid = if near.is_empty() { None } else { Some([0, 1, 2].map(|a| near.iter().map(|p| p[a]).sum::<f64>() / near.len() as f64)) };
        bind[g] = found[g].iter().min_by(|x, y| mid.map_or(std::cmp::Ordering::Equal, |m| dist(x.2, m).total_cmp(&dist(y.2, m)))).map(|c| (c.0, c.1));
    }
    println!("{multi} grid points were found at more than one place");

    // The layout's face piece ends a little below the chin, so each column is carried on down the neck from its lowest point, 2 mm at a
    // time along the skin, keeping its direction.
    let bvh = head_paint::closest::Bvh::new(mm.clone(), tris.clone());
    let place = |b: (u32, [f64; 2])| {
        let v = tris[b.0 as usize].map(|i| mm[i as usize]);
        let w = [b.1[0], b.1[1], 1.0 - b.1[0] - b.1[1]];
        [0, 1, 2].map(|a| w[0] * v[0][a] + w[1] * v[1][a] + w[2] * v[2][a])
    };
    // Row by row downward, each column whose point above has no point below takes a step of 2 mm along the skin, in the mean
    // direction of its neighbours' last steps, so that the columns run parallel. A column that comes within 1.2 mm of the next column
    // toward the middle, or crosses it, stops there, so the columns thin out as the neck narrows.
    let mid_col = smax as usize;
    for j in (0..gr - 2).rev() {
        let mut steps: Vec<Option<[f64; 3]>> = vec![None; gc];
        for gi in 0..gc {
            if bind[j * gc + gi].is_none() {
                if let (Some(a), Some(b)) = (bind[(j + 1) * gc + gi], bind[(j + 2) * gc + gi]) {
                    steps[gi] = Some(normalize(sub(place(a), place(b))));
                }
            }
        }
        let mut new_row: Vec<Option<((u32, [f64; 2]), [f64; 3])>> = vec![None; gc];
        for gi in 0..gc {
            if steps[gi].is_none() {
                continue;
            }
            let mut d = [0.0; 3];
            for k in gi.saturating_sub(4)..=(gi + 4).min(gc - 1) {
                if let Some(v) = steps[k] {
                    d = add(d, v, 1.0);
                }
            }
            let p = place(bind[(j + 1) * gc + gi].expect("bound"));
            let Some(c) = bvh.closest(add(p, normalize(d), SPACING_MM)) else { continue };
            if c.distance > 3.0 || at(c.point)[1] < chin[1] - 75.0 * scale {
                continue;
            }
            new_row[gi] = Some(((c.triangle, [c.barycentric[0], c.barycentric[1]]), c.point));
        }
        for dir in [1i64, -1] {
            let mut inner: Option<[f64; 3]> = None;
            let mut gi = mid_col as i64;
            while gi >= 0 && (gi as usize) < gc {
                let k = j * gc + gi as usize;
                let here = match (bind[k], new_row[gi as usize]) {
                    (Some(b), _) => Some(place(b)),
                    (None, Some(n)) => {
                        let ok = inner.is_none_or(|q| dist(q, n.1) > 1.2 && (at(n.1)[0] - at(q)[0]) * dir as f64 > 0.0);
                        if ok {
                            bind[k] = Some(n.0);
                            Some(n.1)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if here.is_some() {
                    inner = here;
                }
                gi += dir;
            }
        }
    }
    // A point that was not found but has bound points on three or four sides, or on two opposite sides, takes the skin nearest to the
    // middle of theirs.
    for _ in 0..3 {
        let before = bind.clone();
        for g in 0..gc * gr {
            if before[g].is_some() {
                continue;
            }
            let (i, j) = ((g % gc) as i64, (g / gc) as i64);
            let side = |di: i64, dj: i64| {
                let (a, b) = (i + di, j + dj);
                (a >= 0 && b >= 0 && (a as usize) < gc && (b as usize) < gr).then(|| before[b as usize * gc + a as usize].map(place)).flatten()
            };
            let s4 = [side(1, 0), side(-1, 0), side(0, 1), side(0, -1)];
            let n = s4.iter().flatten().count();
            if !(n >= 3 || (s4[0].is_some() && s4[1].is_some()) || (s4[2].is_some() && s4[3].is_some())) {
                continue;
            }
            let mid = [0, 1, 2].map(|a| s4.iter().flatten().map(|p| p[a]).sum::<f64>() / n as f64);
            if let Some(c) = bvh.closest(mid) {
                bind[g] = Some((c.triangle, [c.barycentric[0], c.barycentric[1]]));
            }
        }
    }
    let full = FaceBinding { cols: gc, rows: gr, jaw_row: (-tlo) as usize, mouth_row: (n_jm as i64 - tlo) as usize, points: bind, marks: Vec::new(), zones: Vec::new(), copy_of: Vec::new() };

    // The trim, from the facial hair's own regions and edges on the mean head.
    let beard = BeardHead::with_binding(&full, &mean, &uv, &tris, &vmap, Some(&lips)).expect("beard");
    let lip_poly: Vec<[f64; 2]> = lips.upper.iter().chain(lips.lower.iter().rev()).copied().collect();
    let mut keep: Vec<bool> = (0..gc * gr)
        .map(|k| {
            let g = &beard.grid.points[k];
            if !g.valid || beard.reach[k] <= 0.0 {
                return false;
            }
            let uvw = lips.frame.uvw(g.position);
            let on_lips = uvw[2] > -6.0 && polygon_distance([uvw[0], uvw[1]], &lip_poly) < 0.0;
            let e = beard.edges[k];
            !on_lips && e[0] < CHEEK_ROOM_MM && e[1] < NECK_ROOM_MM && e[2] < 3.0 && e[3] < 2.5
        })
        .collect();
    // Patches of fewer than 40 points, such as specks inside the nostrils, are dropped.
    let mut seen = vec![false; gc * gr];
    for start in 0..gc * gr {
        if !keep[start] || seen[start] {
            continue;
        }
        let mut patch = vec![start];
        seen[start] = true;
        let mut at = 0;
        while at < patch.len() {
            let k = patch[at];
            at += 1;
            let (i, j) = ((k % gc) as i64, (k / gc) as i64);
            for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (a, b) = (i + di, j + dj);
                if a >= 0 && b >= 0 && (a as usize) < gc && (b as usize) < gr {
                    let n = b as usize * gc + a as usize;
                    if keep[n] && !seen[n] {
                        seen[n] = true;
                        patch.push(n);
                    }
                }
            }
        }
        if patch.len() < 40 {
            for k in patch {
                keep[k] = false;
            }
        }
    }
    if let Some(debug) = args.get(1) {
        let mut ppm = format!("P3 {gc} {gr} 255\n");
        for j in (0..gr).rev() {
            for i in 0..gc {
                let k = j * gc + i;
                let c = if j == full.jaw_row { [255, 0, 0] } else if j == full.mouth_row { [0, 255, 0] } else if keep[k] { [255, 255, 255] } else if beard.grid.points[k].valid {
                    let e = beard.edges[k];
                    if beard.reach[k] <= 0.0 { [40, 40, 160] } else if e[2] >= 3.0 { [0, 200, 200] } else if e[1] >= NECK_ROOM_MM { [200, 0, 200] } else if e[0] >= CHEEK_ROOM_MM { [200, 200, 0] } else if e[3] >= 1.5 { [255, 140, 0] } else { [110, 110, 110] }
                } else { [0, 0, 0] };
                ppm += &format!("{} {} {} ", c[0], c[1], c[2]);
            }
            ppm += "\n";
        }
        std::fs::write(debug, ppm).expect("debug");
    }
    let ks: Vec<usize> = (0..gc * gr).filter(|&k| keep[k]).collect();
    let (i0, i1) = (ks.iter().map(|k| k % gc).min().expect("points"), ks.iter().map(|k| k % gc).max().expect("points"));
    let (j0, j1) = (ks.iter().map(|k| k / gc).min().expect("points"), ks.iter().map(|k| k / gc).max().expect("points"));
    let (nc, nr) = (i1 - i0 + 1, j1 - j0 + 1);
    let points = (j0..=j1).flat_map(|j| (i0..=i1).map(move |i| (i, j))).map(|(i, j)| if keep[j * gc + i] { full.points[j * gc + i] } else { None }).collect();
    let binding = FaceBinding { cols: nc, rows: nr, jaw_row: full.jaw_row - j0, mouth_row: full.mouth_row - j0, points, marks: Vec::new(), zones: Vec::new(), copy_of: Vec::new() };
    std::fs::write(out, binding.to_bytes()).expect("write");
    let check = BeardHead::with_binding(&binding, &mean, &uv, &tris, &vmap, Some(&lips)).expect("beard");
    let sp = check.grid.spacing_mm();
    {
        let g = &check.grid;
        let mut long = Vec::new();
        for j in 0..g.rows {
            for i in 0..g.cols {
                for (di, dj) in [(1, 0), (0, 1)] {
                    if i + di < g.cols && j + dj < g.rows && g.at(i, j).valid && g.at(i + di, j + dj).valid {
                        let d = dist(g.at(i, j).position, g.at(i + di, j + dj).position);
                        if d > 4.0 {
                            long.push(d);
                        }
                    }
                }
            }
        }
        println!("{} neighbour gaps over 4 mm", long.len());
    }
    println!(
        "{out}: {nc} by {nr} points, {} kept, jaw row {}, mouth row {}, middle column {}; spacing on the mean head {:.2} / {:.2} / {:.2} mm",
        ks.len(),
        binding.jaw_row,
        binding.mouth_row,
        smax as usize - i0,
        sp[0],
        sp[1],
        sp[2]
    );
}
