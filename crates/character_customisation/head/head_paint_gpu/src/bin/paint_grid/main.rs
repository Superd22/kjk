//! Makes the paint grid (`head_paint::face_grid`, decision 63) on the mean of the 57 test heads and writes its binding to the layout,
//! its zones and its marks, and the hair group presets that are laid on it (paint track, part C, with part D). Run it again only to
//! change the grid's design: values drawn on the old grid would land elsewhere.
//!
//! `paint_grid <out bin> <out presets json> [<points csv>]`, and with `HEADS_CSV=<folder>` it also writes the grid's points on a few
//! heads, and the points of the grid in `head_paint`, for pictures.
//!
//! The paint grid covers every place hair can grow: the scalp, the hairline, the temples, the sideburns, behind the ears, the nape, the
//! beard and the neck. It is one smooth chart of the skin (`chart`), laid out as rigidly as it can be, so that its columns and rows run
//! about 2 mm apart without bends. Column 0 runs up the middle of the face, the x < 0 half has the columns below it, and row 0 passes
//! through the chin. Over the top of the head and down the back the middle line is the grid's seam, where each half carries on a few
//! steps onto the other. The ears, the pockets of the layout (the inside of the mouth and the linings of the eyes) and the shoulders are
//! left out. The lips, the nose and the eyelids are holes in the grid, because hair never grows there. The CSV, when asked for, lists
//! each point's column, row, place in millimetres, zone and whether it lies past the seam, for pictures.

mod chart;

use head_paint::closest::closest_on_triangle;
use head_paint::face_grid::{FaceBinding, NO_ZONE, ZONES};
use head_paint::facial_hair::{BeardHead, polygon_distance, EAR_FRONT};
use head_paint::folds::{Anchor, Anchors, Landmarks};
use head_paint::hair_groups::{ControlPoint, GroupKind, HairGroup, HairGroups, default_values};
use head_paint::layout_points::{cross, dot, normalize, sub};
use std::path::Path;

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
/// The distance between rows and between columns, in millimetres on the mean head.
const SPACING_MM: f64 = 2.0;
/// The grid before it is cropped: columns from -170 to 170 and rows from -60 to 200, in steps from the face's middle and the chin.
const SMAX: i64 = 170;
const TLO: i64 = -60;
const THI: i64 = 200;
/// The cones of the chart, in degrees round the ears' axis from the front: one on the front of the scalp and one on the back of the
/// head. Between them the middle line runs straight across the grid, so the columns cross the top of the head from side to side.
const CONES_DEG: [f64; 2] = [73.0, 151.0];
/// Holes whose outline is shorter than this are pockets of skin where hair may grow, such as at the jaw's corners and under the chin.
const SMALL_HOLE_MM: f64 = 80.0;
/// How far each half carries on past the seam, in steps.
const OVERLAP: f64 = 8.0;
/// How much the chart's triangles hold their shape: the neck below a line from the throat to the nape gives, and so does the underside
/// of the jaw, while the front of the face holds.
const NECK_WEIGHT: f64 = 0.15;
const THROAT_Y: f64 = 600.0;
const NAPE_Y: f64 = 660.0;
const UNDER_JAW_WEIGHT: f64 = 0.5;
const FACE_WEIGHT: f64 = 2.0;

/// The height of the scalp's hairline on the mean head, in millimetres of the mesh's height, against the angle round the upright axis
/// from the front, in degrees: high on the forehead, receding at the temples, down to the top of the sideburn in front of the ear, over
/// the ear, and down behind it to the nape.
const HAIRLINE: [[f64; 2]; 12] = [[0.0, 790.0], [25.0, 788.0], [40.0, 781.0], [50.0, 776.0], [58.0, 764.0], [65.0, 750.0], [72.0, 745.0], [80.0, 750.0], [98.0, 752.0], [115.0, 728.0], [140.0, 690.0], [180.0, 668.0]];
/// The sideburn lies in front of the ear between these angles, and reaches down to this height.
const SIDEBURN_DEG: [f64; 2] = [62.0, 79.0];
const SIDEBURN_LOW: f64 = 676.0;
fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().expect("8"))).collect()
}

fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().expect("4"))).collect()
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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args.first().expect("Usage: paint_grid <out bin> <out presets json> [<points csv>]");
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

    let mm: Vec<[f64; 3]> = vmap.iter().map(|&i| mean[i].map(|x| 1000.0 * x)).collect();

    // ---- The whole head. ----
    let presets_out = args.get(1).expect("Usage: paint_grid <out bin> <out presets json> [<points csv>]");
    let nl = uv.len();
    let ear_pt = EAR_FRONT.map(|vs| Anchors::point(&Anchor { vertices: [vs[0], vs[1], vs[2]], weights: [1.0 / 3.0; 3] }, &mean));
    // The middle of the head's upright axis, at the height of the ears and 15 mm behind their front.
    let axis = [0.5 * (ear_pt[0][0] + ear_pt[1][0]), 0.5 * (ear_pt[0][1] + ear_pt[1][1]), 0.5 * (ear_pt[0][2] + ear_pt[1][2]) - 15.0];
    let azimuth = |p: [f64; 3]| (p[0] - axis[0]).atan2(p[2] - axis[2]).to_degrees();
    // The layout's pieces: the ears are the small pieces near the ear's place, and the shoulders lie far below the chin.
    let mut parent: Vec<usize> = (0..nl).collect();
    fn root2(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for t in &tris {
        let a = root2(&mut parent, t[0] as usize);
        for &o in &t[1..] {
            let b = root2(&mut parent, o as usize);
            parent[b] = a;
        }
    }
    let roots: Vec<usize> = (0..nl).map(|i| root2(&mut parent, i)).collect();
    let in_ear_box = |p: [f64; 3]| p[0].abs() > 60.0 && p[1] > 680.0 && p[1] < 742.0 && p[2] > 10.0 && p[2] < 55.0;
    let mut piece_count = vec![[0usize; 2]; nl];
    for i in 0..nl {
        piece_count[roots[i]][0] += 1;
        piece_count[roots[i]][1] += usize::from(in_ear_box(mm[i]));
    }
    let chin_y = m.chin_middle[1];
    let ear_tri: Vec<bool> = tris.iter().map(|t| { let c = piece_count[roots[t[0] as usize]]; c[0] < 150 && c[1] as f64 >= 0.6 * c[0] as f64 }).collect();
    let shoulder_tri: Vec<bool> = tris.iter().map(|t| { let c = [0, 1, 2].map(|a| t.iter().map(|&i| mm[i as usize][a]).sum::<f64>() / 3.0); c[1] < chin_y - 90.0 || (c[0].abs() > 85.0 && c[1] < 650.0) }).collect();
    // A pocket is skin that the head hides from its axis: a ray from the axis's middle through the triangle's middle meets the head
    // again further out. Only the head above the neck hides skin this way, so that the shoulders' slope does not hide the neck.
    let corners: Vec<[[f64; 3]; 3]> = tris.iter().map(|t| t.map(|i| mm[i as usize])).collect();
    let outer: Vec<bool> = (0..tris.len()).map(|t| !ear_tri[t] && !shoulder_tri[t] && (corners[t][0][1] + corners[t][1][1] + corners[t][2][1]) / 3.0 > chin_y - 25.0).collect();
    let pocket: Vec<bool> = (0..tris.len())
        .map(|t| {
            if ear_tri[t] || shoulder_tri[t] {
                return false;
            }
            let c = [0, 1, 2].map(|a| (corners[t][0][a] + corners[t][1][a] + corners[t][2][a]) / 3.0);
            let d = sub(c, axis);
            let len = dot(d, d).sqrt();
            let dir = d.map(|x| x / len);
            (0..tris.len()).any(|o| o != t && outer[o] && ray_hit(axis, dir, corners[o]).is_some_and(|h| h > len + 0.5))
        })
        .collect();
    // Skin inside a cavity, such as the throat, sees mostly the head round it: of 32 rays spread over the side its normal faces, more than
    // 70% meet the head within 150 mm.
    let cavity: Vec<bool> = {
        let n = tris.len();
        let mut out = vec![false; n];
        let threads = 8;
        let chunk = n.div_ceil(threads);
        std::thread::scope(|sc| {
            for (part, slot) in out.chunks_mut(chunk).enumerate() {
                let (corners, ear_tri, shoulder_tri) = (&corners, &ear_tri, &shoulder_tri);
                sc.spawn(move || {
                    for (o, flag) in slot.iter_mut().enumerate() {
                        let t = part * chunk + o;
                        if ear_tri[t] || shoulder_tri[t] {
                            continue;
                        }
                        let v = corners[t];
                        let c = [0, 1, 2].map(|a| (v[0][a] + v[1][a] + v[2][a]) / 3.0);
                        let nrm = normalize(cross(sub(v[1], v[0]), sub(v[2], v[0])));
                        let helper = if nrm[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
                        let e1 = normalize(cross(nrm, helper));
                        let e2 = cross(nrm, e1);
                        let mut hits = 0;
                        for r in 0..32 {
                            let (u1, u2) = ((r as f64 + 0.5) / 32.0, (r as f64 * 0.618034).fract());
                            let (rad, ang) = (u1.sqrt(), std::f64::consts::TAU * u2);
                            let h = (1.0 - u1).sqrt();
                            let d = [0, 1, 2].map(|a| rad * ang.cos() * e1[a] + rad * ang.sin() * e2[a] + h * nrm[a]);
                            let o = [0, 1, 2].map(|a| c[a] + 0.05 * nrm[a]);
                            if (0..corners.len()).any(|q| q != t && !shoulder_tri[q] && !ear_tri[q] && ray_hit(o, d, corners[q]).is_some_and(|x| x < 150.0)) {
                                hits += 1;
                            }
                        }
                        *flag = hits > 22;
                    }
                });
            }
        });
        out
    };
    println!("{} triangles lie in cavities", cavity.iter().filter(|&&x| x).count());
    let pocket: Vec<bool> = (0..tris.len()).map(|t| pocket[t] || cavity[t]).collect();
    // A piece of the layout that is mostly hidden, such as the inside of the mouth down to the throat, is a pocket as a whole.
    let mut hidden = vec![[0usize; 2]; nl];
    for t in 0..tris.len() {
        let r = roots[tris[t][0] as usize];
        hidden[r][0] += 1;
        hidden[r][1] += usize::from(pocket[t]);
    }
    let pocket: Vec<bool> = (0..tris.len()).map(|t| { let h = hidden[roots[tris[t][0] as usize]]; pocket[t] || 2 * h[1] > h[0] }).collect();
    let usable: Vec<bool> = (0..tris.len()).map(|t| !ear_tri[t] && !shoulder_tri[t] && !pocket[t]).collect();
    if let Ok(path) = std::env::var("TRIS_CSV") {
        let mut text = String::from("x,y,z,class\n");
        for t in 0..tris.len() {
            let c = [0, 1, 2].map(|a| (corners[t][0][a] + corners[t][1][a] + corners[t][2][a]) / 3.0);
            let class = if ear_tri[t] { 1 } else if shoulder_tri[t] { 2 } else if pocket[t] { 3 } else { 0 };
            text += &format!("{:.2},{:.2},{:.2},{class}\n", c[0], c[1], c[2]);
        }
        std::fs::write(path, text).expect("tris csv");
    }
    println!("triangles: {} ears, {} shoulders, {} pockets, {} usable", ear_tri.iter().filter(|&&x| x).count(), shoulder_tri.iter().filter(|&&x| x).count(), pocket.iter().filter(|&&x| x).count(), usable.iter().filter(|&&x| x).count());

    // The holes, where hair never grows: the lips, the nose and the eyelids, and the underside of the nose and the skin round the ear's
    // opening, which face along the head rather than out of it. They are left out of the chart, so their bumps do not bend its lines.
    let lip_poly: Vec<[f64; 2]> = lips.upper.iter().chain(lips.lower.iter().rev()).copied().collect();
    let wings = [at(m.nose_wing[0]), at(m.nose_wing[1])];
    let wing_x = 0.5 * (wings[1][0] - wings[0][0]);
    let wing_y = 0.5 * (wings[0][1] + wings[1][1]);
    let eyes = [0, 1].map(|s| at([0, 1, 2].map(|a| 0.5 * (m.eye_inner[s][a] + m.eye_outer[s][a]))));
    let eye_half = [0, 1].map(|s| 0.5 * dist(m.eye_inner[s], m.eye_outer[s]));
    let hole = |p: [f64; 3]| -> bool {
        let uvw = lips.frame.uvw(p);
        if uvw[2] > -6.0 && polygon_distance([uvw[0], uvw[1]], &lip_poly) < 0.0 {
            return true;
        }
        let q = at(p);
        let (ax, y) = (q[0].abs(), q[1]);
        let eye_y = 0.5 * (eyes[0][1] + eyes[1][1]);
        let half = curve_at(&[[wing_y - 4.0, wing_x - 3.0], [wing_y, wing_x + 1.0], [wing_y + 12.0, wing_x - 2.0], [eye_y - 8.0, 10.0], [eye_y + 6.0, 7.0]], y);
        if y > wing_y - 4.0 && y < eye_y + 6.0 && ax < half && q[2] > -12.0 {
            return true;
        }
        q[2] > -25.0 && eyes.iter().zip(eye_half).any(|(e, h)| ((q[0] - e[0]) / (h + 5.0)).powi(2) + ((q[1] - e[1]) / 11.0).powi(2) < 1.0)
    };
    let sideways = |t: usize| {
        let v = corners[t];
        let c = [0, 1, 2].map(|a| (v[0][a] + v[1][a] + v[2][a]) / 3.0);
        let n = normalize(cross(sub(v[1], v[0]), sub(v[2], v[0])));
        let q = at(c);
        let under_nose = q[0].abs() < wing_x + 3.0 && q[1] > wing_y - 9.0 && q[1] < wing_y + 2.0 && q[2] > -8.0 && dot(n, up) < -0.55;
        let ear_opening = c[0].abs() > 55.0 && c[1] > 680.0 && c[1] < 735.0 && c[2] > 25.0 && c[2] < 52.0 && n[0].abs() < 0.55;
        under_nose || ear_opening
    };
    let skin: Vec<bool> = (0..tris.len()).map(|t| usable[t] && !sideways(t) && !hole([0, 1, 2].map(|a| (corners[t][0][a] + corners[t][1][a] + corners[t][2][a]) / 3.0))).collect();

    // The chart. The front of the face holds its shape most, and the neck, which is narrower than the head but has as many columns,
    // and the underside of the jaw give most.
    let welded: Vec<[usize; 3]> = tris.iter().map(|t| t.map(|i| vmap[i as usize])).collect();
    let mean_mm: Vec<[f64; 3]> = mean.iter().map(|p| p.map(|x| 1000.0 * x)).collect();
    let weight = |c: [f64; 3], n: [f64; 3]| {
        let y0 = NAPE_Y - (c[2] + 40.0) * (NAPE_Y - THROAT_Y) / 140.0;
        let w = NECK_WEIGHT + (1.0 - NECK_WEIGHT) * smooth(y0 - 25.0, y0, c[1]);
        let q = at(c);
        if q[1] < -25.0 && dot(n, up) < -0.3 {
            w.min(UNDER_JAW_WEIGHT)
        } else if q[2] > -45.0 && q[1] > -45.0 {
            w * FACE_WEIGHT
        } else {
            w
        }
    };
    let face = |c: [f64; 3]| c[2] > 60.0 && c[1] > 630.0 && c[1] < 720.0;
    let cones: Vec<f64> = std::env::var("CONES").map_or(CONES_DEG.to_vec(), |c| c.split(',').map(|x| x.trim().parse().expect("cone")).collect());
    let settings = chart::Settings { cones_deg: &cones, axis, chin: m.chin_middle, subdivisions: 1, rigid_steps: 100, weight: &weight, face: &face, spacing_mm: SPACING_MM };
    let ch = chart::solve(&mean_mm, &welded, &skin, &settings);

    // Each whole column and row found in the chart's skin triangles, bound to its layout triangle. Column 0 runs up the middle of the face;
    // the x < 0 half has the columns below 0.
    let gc = (2 * SMAX + 1) as usize;
    let gr = (THI - TLO + 1) as usize;
    let mut bind: Vec<Option<(u32, [f64; 2])>> = vec![None; gc * gr];
    let g_of = |s: i64, t: i64| (s.abs() <= SMAX && (TLO..=THI).contains(&t)).then(|| (t - TLO) as usize * gc + (s + SMAX) as usize);
    // Inside the small holes, such as the pockets at the jaw's corners and under the chin, hair can grow: a point there takes the skin
    // under the patch.
    let pocket_skin: Vec<u32> = (0..tris.len()).filter(|&t| !ear_tri[t] && !shoulder_tri[t]).map(|t| t as u32).collect();
    let bvh = head_paint::closest::Bvh::new(mm.clone(), tris.clone());
    for (k, t) in ch.mesh.t.iter().enumerate() {
        let small = ch.mesh.parent[k].is_none() && ch.mesh.hole_mm[k] < SMALL_HOLE_MM;
        if ch.mesh.parent[k].is_none() && !small {
            continue;
        }
        let h = ch.side[k];
        let c = t.map(|i| ch.uv[i][h]);
        let den = (c[1][1] - c[2][1]) * (c[0][0] - c[2][0]) + (c[2][0] - c[1][0]) * (c[0][1] - c[2][1]);
        if den.abs() < 1e-14 {
            continue;
        }
        let lo = [(c[0][0].min(c[1][0]).min(c[2][0]) - 1e-6).ceil() as i64, (c[0][1].min(c[1][1]).min(c[2][1]) - 1e-6).ceil() as i64];
        let hi = [(c[0][0].max(c[1][0]).max(c[2][0]) + 1e-6).floor() as i64, (c[0][1].max(c[1][1]).max(c[2][1]) + 1e-6).floor() as i64];
        for gt in lo[1]..=hi[1] {
            for gs in lo[0]..=hi[0] {
                if (h == 0 && gs > 0) || (h == 1 && gs < 0) {
                    continue;
                }
                let Some(g) = g_of(gs, gt) else { continue };
                if bind[g].is_some() {
                    continue;
                }
                let (x, y) = (gs as f64, gt as f64);
                let w0 = ((c[1][1] - c[2][1]) * (x - c[2][0]) + (c[2][0] - c[1][0]) * (y - c[2][1])) / den;
                let w1 = ((c[2][1] - c[0][1]) * (x - c[2][0]) + (c[0][0] - c[2][0]) * (y - c[2][1])) / den;
                let w = [w0, w1, 1.0 - w0 - w1];
                if w.iter().any(|&v| v < -1e-7) {
                    continue;
                }
                let p = [0, 1, 2].map(|a| w[0] * ch.mesh.p[t[0]][a] + w[1] * ch.mesh.p[t[1]][a] + w[2] * ch.mesh.p[t[2]][a]);
                bind[g] = match ch.mesh.parent[k] {
                    Some(parent) => {
                        let v = tris[parent].map(|i| mm[i as usize]);
                        let b = closest_on_triangle(p, v[0], v[1], v[2]);
                        Some((parent as u32, [b[0], b[1]]))
                    }
                    None => bvh.closest_with(p, |t| pocket_skin.binary_search(&t).is_ok()).filter(|c| c.distance < 10.0 && !hole(c.point)).map(|c| (c.triangle, [c.barycentric[0], c.barycentric[1]])),
                };
            }
        }
    }
    let place = |b: (u32, [f64; 2])| {
        let v = tris[b.0 as usize].map(|i| mm[i as usize]);
        let w = [b.1[0], b.1[1], 1.0 - b.1[0] - b.1[1]];
        [0, 1, 2].map(|a| w[0] * v[0][a] + w[1] * v[1][a] + w[2] * v[2][a])
    };
    // The neck stops 75 mm below the chin, and the shoulders are left out.
    for g in 0..gc * gr {
        if bind[g].is_some_and(|b| { let p = place(b); p[1] < chin_y - 75.0 || p[0].abs() > 95.0 }) {
            bind[g] = None;
        }
    }
    // A point stays only if its mirror image does, so that the two halves match where the mean head is not quite symmetric, such as
    // round the ears.
    let before = bind.clone();
    for t in TLO..=THI {
        for s in 1..=SMAX {
            let (Some(a), Some(b)) = (g_of(-s, t), g_of(s, t)) else { continue };
            if before[a].is_none() || before[b].is_none() {
                bind[a] = None;
                bind[b] = None;
            }
        }
    }
    // Past the middle line over the top and down the back, each half carries on `OVERLAP` steps onto the other: a point there is the point
    // of the other half whose mirror image is its own reflection in the piece of the middle line, so that the lines run on.
    let own = bind.clone();
    let mut overlap = vec![false; gc * gr];
    let mut copy_src: Vec<Option<usize>> = vec![None; gc * gr];
    for t in TLO..=THI {
        for s in -SMAX..=SMAX {
            let Some(g) = g_of(s, t) else { continue };
            if own[g].is_some() || s == 0 {
                continue;
            }
            let e = [-(s.abs() as f64), t as f64];
            let mut best: Option<(f64, [i64; 2])> = None;
            for piece in &ch.segments[1..] {
                let d = piece.inside(e);
                if d >= 0.0 || d < -OVERLAP || !piece.beside(e) {
                    continue;
                }
                let r = piece.reflect(e);
                let l = [r[0].round() as i64, r[1].round() as i64];
                if l[0] > 0 || g_of(l[0], l[1]).is_none_or(|k| own[k].is_none()) {
                    continue;
                }
                if best.is_none_or(|b| d > b.0) {
                    best = Some((d, l));
                }
            }
            let Some((_, l)) = best else { continue };
            let from = if s < 0 { g_of(-l[0], l[1]) } else { g_of(l[0], l[1]) };
            if let Some(k) = from.filter(|&k| own[k].is_some()) {
                bind[g] = own[k];
                overlap[g] = true;
                copy_src[g] = Some(k);
            }
        }
    }
    println!("{} points, {} of them past the middle line", bind.iter().flatten().count(), overlap.iter().filter(|&&o| o).count());

    // Patches of fewer than 40 points are dropped.
    let mut seen = vec![false; gc * gr];
    for start in 0..gc * gr {
        if bind[start].is_none() || seen[start] {
            continue;
        }
        let mut patch = vec![start];
        seen[start] = true;
        let mut k0 = 0;
        while k0 < patch.len() {
            let k = patch[k0];
            k0 += 1;
            let (i, j) = ((k % gc) as i64, (k / gc) as i64);
            for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (a, b) = (i + di, j + dj);
                if a >= 0 && b >= 0 && (a as usize) < gc && (b as usize) < gr {
                    let n = b as usize * gc + a as usize;
                    if bind[n].is_some() && !seen[n] {
                        seen[n] = true;
                        patch.push(n);
                    }
                }
            }
        }
        if patch.len() < 40 {
            for k in patch {
                bind[k] = None;
            }
        }
    }


    // The crop, the same on both sides of the middle column.
    let ks: Vec<usize> = (0..gc * gr).filter(|&k| bind[k].is_some()).collect();
    let smax = ks.iter().map(|&k| ((k % gc) as i64 - SMAX).abs()).max().expect("points");
    let (j0, j1) = (ks.iter().map(|k| k / gc).min().expect("points"), ks.iter().map(|k| k / gc).max().expect("points"));
    let (i0, i1) = ((SMAX - smax) as usize, (SMAX + smax) as usize);
    let (nc, nr) = (i1 - i0 + 1, j1 - j0 + 1);
    let points: Vec<Option<(u32, [f64; 2])>> = (j0..=j1).flat_map(|j| (i0..=i1).map(move |i| (i, j))).map(|(i, j)| bind[j * gc + i]).collect();
    let mid_col = smax as usize;
    // A copy keeps its point of the other half by that point's index in the cropped grid. A copy whose point was dropped goes too.
    let crop = |g: usize| { let (i, j) = (g % gc, g / gc); (i >= i0 && i <= i1 && j >= j0 && j <= j1).then(|| (j - j0) * nc + (i - i0)) };
    let mut points = points;
    let mut copy_of: Vec<Option<usize>> = vec![None; nc * nr];
    for j in j0..=j1 {
        for i in i0..=i1 {
            let Some(src) = copy_src[j * gc + i] else { continue };
            let k = (j - j0) * nc + (i - i0);
            match crop(src).filter(|&o| points[o].is_some()) {
                Some(o) if points[k].is_some() => copy_of[k] = Some(o),
                _ => points[k] = None,
            }
        }
    }
    // The jaw's row is where the middle column, going down from the mouth past the lower lip's underside and down the chin's front,
    // first meets skin that faces more than 60 degrees downward; the mouth's row is the row nearest the mouth's corners.
    let normal_at = |b: (u32, [f64; 2])| {
        let v = tris[b.0 as usize].map(|i| mm[i as usize]);
        let n = normalize(cross(sub(v[1], v[0]), sub(v[2], v[0])));
        if dot(n, sub(v[0], axis)) < 0.0 { n.map(|x| -x) } else { n }
    };
    let mut on_chin = false;
    let mut jaw_row = 0;
    for j in (0..nr).rev() {
        let Some(b) = points[j * nc + mid_col] else { continue };
        if place(b)[1] > m.mouth_corner[0][1] {
            continue;
        }
        let n = normal_at(b);
        on_chin |= dot(n, fw) > 0.8;
        if on_chin && dot(n, up) < -0.5 {
            jaw_row = j + 1;
            break;
        }
    }
    let mouth_row = {
        let k = (0..nc * nr).filter(|&k| points[k].is_some()).min_by(|&a, &b| dist(place(points[a].expect("point")), m.mouth_corner[0]).total_cmp(&dist(place(points[b].expect("point")), m.mouth_corner[0]))).expect("points");
        k / nc
    };
    let mut binding = FaceBinding { cols: nc, rows: nr, jaw_row, mouth_row, points: points.clone(), marks: Vec::new(), zones: points.iter().map(|p| if p.is_some() { 0 } else { NO_ZONE }).collect(), copy_of };
    println!("grid {nc} by {nr}, {} points, middle column {mid_col}, jaw row {jaw_row}, mouth row {mouth_row}", ks.len());


    // The zones, on the mean head.
    let beard = BeardHead::with_binding(&binding, &mean, &uv, &tris, &vmap, Some(&lips)).expect("beard");
    let hairline = |phi: f64| curve_at(&HAIRLINE, phi.abs());
    let zone_of = |k: usize| -> u8 {
        let g = &beard.grid.points[k];
        let p = g.position;
        let phi = azimuth(p).abs();
        let yh = hairline(phi);
        let y = p[1];
        let z = |name: &str| ZONES.iter().position(|n| *n == name).expect("zone") as u8;
        if y > yh + 5.0 {
            return if phi > 45.0 && phi < 105.0 && y < yh + 28.0 { z("temple") } else { z("scalp") };
        }
        if y > yh - 4.0 {
            return z("hairline");
        }
        if phi > SIDEBURN_DEG[0] && phi < SIDEBURN_DEG[1] && y > SIDEBURN_LOW {
            return z("sideburn");
        }
        if phi > 100.0 {
            return z("nape");
        }
        let e = beard.edges[k];
        if beard.reach[k] > 0.3 && e[0] < 2.0 && e[2] < 0.0 && e[3] < 0.0 {
            let r = (0..7).max_by(|&a, &b| beard.regions[k][a].total_cmp(&beard.regions[k][b])).unwrap_or(0);
            return z(["moustache", "soul patch", "chin", "jaw", "cheek", "sideburn", "neck"][r]);
        }
        if beard.regions[k][6] > 0.5 || (phi > 60.0 && y < 680.0) {
            return z("neck");
        }
        z("bare")
    };
    for k in 0..nc * nr {
        if points[k].is_some() {
            binding.zones[k] = zone_of(k);
        }
    }

    // The marks, on the x < 0 side: the whorl, behind the top of the head, and the sideburn's top and bottom, in front of the ear.
    let nearest = |want: &dyn Fn([f64; 3]) -> f64| -> [f64; 2] {
        let k = (0..nc * nr).filter(|&k| points[k].is_some() && (k % nc) <= mid_col && !overlap[(k / nc + j0) * gc + (k % nc + i0)]).min_by(|&a, &b| want(beard.grid.points[a].position).total_cmp(&want(beard.grid.points[b].position))).expect("points");
        [(k % nc) as f64, (k / nc) as f64]
    };
    let top_y = mm.iter().map(|p| p[1]).fold(f64::MIN, f64::max);
    let whorl = nearest(&|p| (p[0] - axis[0]).abs() * 3.0 + (p[2] - (axis[2] - 40.0)).abs() - p[1] * 0.0 + if p[1] < top_y - 25.0 { 1e3 } else { 0.0 });
    let side_dir = |deg: f64, y: f64| move |p: [f64; 3]| (azimuth(p) + deg).abs() * 1.5 + (p[1] - y).abs() + if p[0] > 0.0 { 1e3 } else { 0.0 };
    let sb_top = nearest(&side_dir(70.0, hairline(70.0) + 6.0));
    let sb_bottom = nearest(&side_dir(72.0, SIDEBURN_LOW - 2.0));
    binding.marks = vec![whorl, sb_top, sb_bottom];
    println!("marks: whorl {whorl:?}, sideburn top {sb_top:?}, bottom {sb_bottom:?}");
    std::fs::write(out, binding.to_bytes()).expect("write");

    // The presets: the moustache of 2026-11-20 at the same places on the mean head, as offsets from the middle column and the mouth's
    // row, a sideburn and a buzz cut.
    let mid = mid_col as f64;
    let mrow = mouth_row as f64;
    let moustache = HairGroup {
        name: "Moustache".into(),
        kind: GroupKind::Stroke,
        points: [(0.0, 6.2, 11.0), (-2.95, 5.1, 10.0), (-6.95, 2.45, 8.0), (-9.25, 1.1, 5.0)].map(|(di, dj, w)| ControlPoint::new(mid + di, mrow + dj, w)).to_vec(),
        mirror: true,
        hidden: false,
        pull: false,
        flow: Vec::new(),
        values: first_values([3.0, 0.9, 0.12, 0.15, 20.0, 55.0, 14.0, 0.0, 0.5]),
    };
    let sideburn = HairGroup {
        name: "Sideburn".into(),
        kind: GroupKind::Stroke,
        points: [0.0, 0.35, 0.7, 1.0]
            .iter()
            .zip([20.0, 19.0, 17.0, 13.0])
            .map(|(&f, w)| {
                let y = hairline(70.0) + 6.0 + f * (SIDEBURN_LOW - 2.0 - hairline(70.0) - 6.0);
                let q = nearest(&side_dir(70.0 + 2.0 * f, y));
                ControlPoint::new(q[0], q[1], w)
            })
            .collect(),
        mirror: true,
        hidden: false,
        pull: false,
        flow: Vec::new(),
        values: first_values([6.0, 1.0, 0.13, 0.2, 14.0, 45.0, 12.0, 0.0, 1.0]),
    };
    // The buzz cut's outline runs along the hairline from just past the front middle to just past the back middle, then back to the
    // front along the seam, 1.5 steps past it, so that it and its mirror image cover the whole scalp.
    let mut outline: Vec<ControlPoint> = [0.0, 25.0, 42.0, 55.0, 65.0, 72.0, 88.0, 108.0, 130.0, 155.0, 178.0]
        .iter()
        .map(|&deg| {
            let q = nearest(&side_dir(deg, hairline(deg) - 2.0));
            ControlPoint::new(q[0], q[1], 4.0)
        })
        .collect();
    outline[0].i = mid + 1.5;
    let to_grid = |q: [f64; 2]| [q[0] + mid, q[1] - (TLO + j0 as i64) as f64];
    let pieces = &ch.segments;
    let back = pieces[pieces.len() - 1];
    let last = outline.len() - 1;
    let q = [outline[last].i - mid, outline[last].j + (TLO + j0 as i64) as f64];
    let d = back.inside(q) + 1.5;
    let past = to_grid([q[0] - d * back.normal[0], q[1] - d * back.normal[1]]);
    outline[last].i = past[0];
    outline[last].j = past[1];
    for w in pieces.windows(2).rev() {
        let (a, b) = (w[0].normal, w[1].normal);
        let miter = 1.5 / (1.0 + a[0] * b[0] + a[1] * b[1]).max(0.3);
        let c = to_grid([w[0].ends[1][0] - miter * (a[0] + b[0]), w[0].ends[1][1] - miter * (a[1] + b[1])]);
        outline.push(ControlPoint::new(c[0], c[1], 4.0));
    }
    println!("seam pieces in the grid: {:?}", pieces.iter().map(|p| [to_grid(p.ends[0]), to_grid(p.ends[1])]).collect::<Vec<_>>());
    let buzz = HairGroup { name: "Buzz cut".into(), kind: GroupKind::Area, points: outline, mirror: true, hidden: false, pull: false, flow: Vec::new(), values: first_values([3.0, 0.95, 0.07, 0.1, 15.0, 50.0, 10.0, 0.0, 0.0]) };
    std::fs::write(presets_out, HairGroups { groups: vec![moustache, sideburn, buzz] }.to_json()).expect("presets");

    if let Some(csv) = args.get(2) {
        let mut text = String::from("i,j,x,y,z,zone,overlap\n");
        for k in 0..nc * nr {
            if points[k].is_some() {
                let p = beard.grid.points[k].position;
                let g = (k / nc + j0) * gc + (k % nc + i0);
                text += &format!("{},{},{:.2},{:.2},{:.2},{},{}\n", k % nc, k / nc, p[0], p[1], p[2], binding.zones[k], u8::from(overlap[g]));
            }
        }
        std::fs::write(csv, text).expect("csv");
    }
    let sp = beard.grid.spacing_mm();
    println!("spacing on the mean head {:.2} / {:.2} / {:.2} mm", sp[0], sp[1], sp[2]);
    if let Ok(dir) = std::env::var("HEADS_CSV") {
        let dir = Path::new(&dir);
        std::fs::create_dir_all(dir).expect("folder");
        let old = FaceBinding::baked();
        for head in ["mean", "p_carthh", "pmba01", "p_bastilah"] {
            let pos: Vec<[f64; 3]> = if head == "mean" { mean_mm.clone() } else { f64s(&Path::new(DATA).join("heads").join(format!("{head}.f64"))).chunks_exact(3).map(|p| [1000.0 * p[0], 1000.0 * p[1], 1000.0 * p[2]]).collect() };
            for (name, b, flags) in [("new", &binding, Some((nc, i0, j0))), ("old", &old, None)] {
                let mut text = String::from("i,j,x,y,z,overlap\n");
                for (k, p) in b.points.iter().enumerate() {
                    let Some((t, w)) = *p else { continue };
                    let v = welded[t as usize].map(|i| pos[i]);
                    let w = [w[0], w[1], 1.0 - w[0] - w[1]];
                    let q = [0, 1, 2].map(|a| w[0] * v[0][a] + w[1] * v[1][a] + w[2] * v[2][a]);
                    let o = flags.is_some_and(|(nc, i0, j0)| overlap[(k / nc + j0) * gc + (k % nc + i0)]);
                    text += &format!("{},{},{:.3},{:.3},{:.3},{}\n", k % b.cols, k / b.cols, q[0], q[1], q[2], u8::from(o));
                }
                std::fs::write(dir.join(format!("{name}_{head}.csv")), text).expect("csv");
            }
        }
    }
}

/// The distance along a ray from `o` in the unit direction `d` to a triangle, if the ray meets it.
fn ray_hit(o: [f64; 3], d: [f64; 3], v: [[f64; 3]; 3]) -> Option<f64> {
    let e1 = sub(v[1], v[0]);
    let e2 = sub(v[2], v[0]);
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-12 {
        return None;
    }
    let s = sub(o, v[0]);
    let u = dot(s, p) / det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, e1);
    let w = dot(d, q) / det;
    if w < 0.0 || u + w > 1.0 {
        return None;
    }
    let t = dot(e2, q) / det;
    (t > 0.0).then_some(t)
}

/// A preset's values: its first nine sliders as given, and the rest at their defaults.
fn first_values(first: [f64; 9]) -> Vec<f64> {
    let mut v = default_values();
    v[..9].copy_from_slice(&first);
    v
}
