//! Pictures of the facial hair layer (paint track, part C, layer 6). Analysis only.
//!
//! `beard_pictures <label> <out dir> <name>... [--layers 1111111111] [--set <control id>=<value>]... [--hand <json>] [--grid <json>] [--groups <json>] [--preset <id>]... [--size 2048]`
//!
//! For each head it paints the head's recipe from the live page's data on the GPU, with the head's values from the `--hand` file (an
//! object `heads` of control id to value) and then the `--set` values put in, and writes to `<out dir>/<label>/`:
//! - `<name>_front.jpg` and `<name>_three.jpg`: the lower face from the front and from 35 degrees to the head's left, 120 by 115 mm at
//!   6 pixels per millimetre;
//! - `<name>_close.jpg`: the chin and the x > 0 corner of the mouth from the front, 50 by 36 mm at 20 pixels per millimetre;
//! - `<name>_alone.jpg`: the front view with only the facial hair on, on neutral grey skin;
//! - `<name>_regions.jpg`: the front view without the facial hair, with the grid's points drawn as dots coloured by their main region
//!   and sized by their density, a stroke for the direction of growth, and white points on the lips, and `<name>_regions_side.jpg`,
//!   the same seen from 70 degrees to the head's left, 150 mm square, up to the temple;
//! - `<name>_sheet.jpg`: KOTOR's front and three-quarter views above ours.
//!
//! Each head takes its hair groups from the live page's `groups/<name>.json`, or from the `hair_groups::HairGroups` file of `--groups`
//! for every head, and each `--preset` adds a preset of `hair_groups::PRESETS` on top of them. The facial hair's old global sliders
//! (`beard_amount` and the others), given by `--set` or `--hand`, replace the head's stubble groups with one stubble made from them and
//! the recipe's old values. With `--grid`, the values of a `hair_grid::GridEdit` file (as `beard_grid` writes) are laid over the groups.
//!
//! With `--marks`, it also writes `<name>_marks.json`: the place in pixels of every paint grid point seen from the front in
//! `<name>_front.jpg`, `<name>_three.jpg` and a side view from 70 degrees, `<name>_side.jpg` (with KOTOR's in `kotor/`), as
//! `[column, row, x, y]`, for drawing the grid over KOTOR's pictures.
//!
//! It writes the same views of KOTOR's own head, with its hair meshes, to `<out dir>/kotor/` when they are missing.

#[path = "../../../head_paint/src/bin/shared/bundle.rs"]
mod bundle;
#[path = "../../../head_paint/src/bin/shared/fold_common.rs"]
mod fold_common;

use fold_common::{Common, KotorFront};
use head_paint::controls::{controls, layers_from, recipe_from};
use head_paint::folds::Landmarks;
use head_paint::live::Layout;
use head_paint_gpu::gpu::Gpu;
use head_paint_gpu::painter::GpuPainter;
use serde_json::Value;
use std::path::{Path, PathBuf};

const DATA: &str = "/Users/david/.claude/tools/likeness-fit/hq/paint/live";
/// The colours of the regions of `facial_hair::REGIONS` on the region map.
const REGION_RGB: [[u8; 3]; 7] = [
    [230, 60, 60],
    [240, 160, 40],
    [240, 230, 60],
    [60, 200, 90],
    [60, 160, 240],
    [150, 90, 230],
    [240, 110, 200],
];

fn f64s(path: &Path) -> Vec<f64> {
    std::fs::read(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .chunks_exact(8)
        .map(|b| f64::from_le_bytes(b.try_into().expect("8")))
        .collect()
}

fn u32s(path: &Path) -> Vec<u32> {
    std::fs::read(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().expect("4")))
        .collect()
}

fn layout() -> Layout {
    let l = Path::new(DATA).join("layout");
    let triples = |v: Vec<u32>| {
        v.chunks_exact(3)
            .map(|t| [t[0], t[1], t[2]])
            .collect::<Vec<_>>()
    };
    Layout {
        uv: f64s(&l.join("uv.f64"))
            .chunks_exact(2)
            .map(|p| [p[0], p[1]])
            .collect(),
        triangles: triples(u32s(&l.join("tris.u32"))),
        layout_to_mesh: u32s(&l.join("vmap.u32"))
            .into_iter()
            .map(|i| i as usize)
            .collect(),
        mesh_triangles: triples(u32s(&l.join("mesh_tris.u32"))),
    }
}

fn positions(name: &str) -> Vec<[f64; 3]> {
    f64s(&Path::new(DATA).join("heads").join(format!("{name}.f64")))
        .chunks_exact(3)
        .map(|p| [p[0], p[1], p[2]])
        .collect()
}

fn add(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]]
}

fn save(img: &image::RgbImage, path: &Path) {
    img.save(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

fn parse(args: &mut Vec<String>, flag: &str) -> Vec<String> {
    let mut out = Vec::new();
    while let Some(k) = args.iter().position(|a| a == flag) {
        out.push(args.remove(k + 1));
        args.remove(k);
    }
    out
}

/// Pictures side by side in rows, with a gap.
fn sheet(rows: &[Vec<&image::RgbImage>]) -> image::RgbImage {
    let gap = 8;
    let w = rows
        .iter()
        .map(|r| r.iter().map(|i| i.width() + gap).sum::<u32>())
        .max()
        .unwrap_or(0);
    let h: u32 = rows
        .iter()
        .map(|r| r.iter().map(|i| i.height()).max().unwrap_or(0) + gap)
        .sum();
    let mut out = image::RgbImage::from_pixel(w, h, image::Rgb([20, 20, 20]));
    let mut y = 0;
    for r in rows {
        let mut x = 0;
        for i in r {
            image::imageops::overlay(&mut out, *i, i64::from(x), i64::from(y));
            x += i.width() + gap;
        }
        y += r.iter().map(|i| i.height()).max().unwrap_or(0) + gap;
    }
    out
}

struct View {
    across: [f64; 3],
    up: [f64; 3],
    forward: [f64; 3],
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let overrides: Vec<(String, f64)> = parse(&mut args, "--set")
        .iter()
        .map(|p| {
            let (id, v) = p.split_once('=').expect("--set id=value");
            (id.to_string(), v.parse().expect("number"))
        })
        .collect();
    let switches: Vec<bool> = parse(&mut args, "--layers")
        .pop()
        .unwrap_or_else(|| "1".repeat(head_paint::controls::GROUPS.len()))
        .chars()
        .map(|c| c == '1')
        .collect();
    let size: u32 = parse(&mut args, "--size")
        .pop()
        .map_or(2048, |v| v.parse().expect("size"));
    let hand: Value = parse(&mut args, "--hand").pop().map_or(Value::Null, |p| {
        serde_json::from_str(&std::fs::read_to_string(&p).expect("hand file")).expect("json")
    });
    let edit = parse(&mut args, "--grid").pop().map(|p| {
        head_paint::hair_grid::GridEdit::from_json(&std::fs::read_to_string(&p).expect("grid file"))
            .expect("a grid edit")
    });
    let given_groups = parse(&mut args, "--groups").pop().map(|p| {
        head_paint::hair_groups::HairGroups::from_json(&std::fs::read_to_string(&p).expect("groups file")).expect("hair groups")
    });
    let presets = parse(&mut args, "--preset");
    let marks_out = args.iter().any(|a| a == "--marks");
    args.retain(|a| a != "--marks");
    let timing = args.iter().any(|a| a == "--time");
    args.retain(|a| a != "--time");
    if args.len() < 3 {
        eprintln!(
            "Usage: beard_pictures <label> <out dir> <name>... [--layers 1111111111] [--set id=value]... [--hand file] [--size n] [--time]"
        );
        std::process::exit(2);
    }
    let label = &args[0];
    let out = PathBuf::from(&args[1]);
    let dir = out.join(label);
    let kotor_dir = out.join("kotor");
    std::fs::create_dir_all(&dir).expect("out");
    std::fs::create_dir_all(&kotor_dir).expect("out");
    let lay = layout();
    let common = Common::load();
    assert_eq!(
        common.uv, lay.uv,
        "the live page's layout differs from the bundle's"
    );
    let mut painter = GpuPainter::new(Gpu::blocking().expect("GPU"), lay);
    painter.set_hair_edit(edit.clone());
    let recipes: Value = serde_json::from_str(
        &std::fs::read_to_string(Path::new(DATA).join("recipes.json")).expect("recipes"),
    )
    .expect("json");
    let list = controls();
    let group = head_paint::controls::GROUPS
        .iter()
        .position(|g| g.id == "facial_hair")
        .expect("facial hair group");

    for name in &args[2..] {
        let pos = positions(name);
        let marks = Landmarks::new(&pos);
        let f = marks.frame;
        painter.set_head(pos.clone());
        pollster::block_on(painter.prepare(size));
        let head = &recipes["heads"][name.as_str()];
        let mut values: Vec<f64> = list
            .iter()
            .map(|c| head[&c.id].as_f64().unwrap_or(c.default))
            .collect();
        let own: Vec<(String, f64)> =
            hand["heads"][name.as_str()]
                .as_object()
                .map_or(Vec::new(), |o| {
                    o.iter()
                        .map(|(k, v)| (k.clone(), v.as_f64().expect("number")))
                        .collect()
                });
        let mut groups = given_groups.clone().unwrap_or_else(|| {
            std::fs::read_to_string(Path::new(DATA).join("groups").join(format!("{name}.json")))
                .ok()
                .and_then(|t| head_paint::hair_groups::HairGroups::from_json(&t))
                .unwrap_or_default()
        });
        for id in &presets {
            groups.groups.splice(0..0, head_paint::hair_groups::HairGroups::preset(id).expect("a preset"));
        }
        let old: Vec<&(String, f64)> = own.iter().chain(&overrides).filter(|(id, _)| id.starts_with("beard_")).collect();
        if !old.is_empty() {
            groups.groups.retain(|g| g.kind != head_paint::hair_groups::GroupKind::Stubble);
            let from = |id: &str| old.iter().rev().find(|(k, _)| k == id).map(|(_, v)| *v).or_else(|| head[id].as_f64());
            groups.groups.extend(head_paint::hair_groups::HairGroup::stubble_from_old_sliders(from));
        }
        *painter.hair_groups_mut() = groups.clone();
        for (id, x) in own.iter().chain(&overrides).filter(|(id, _)| !id.starts_with("beard_")) {
            let k = list
                .iter()
                .position(|c| &c.id == id)
                .unwrap_or_else(|| panic!("no control {id}"));
            values[k] = *x;
        }
        let mut texture = |values: &[f64], switches: &[bool]| {
            let n = size;
            painter.paint(n, &recipe_from(values), &layers_from(switches));
            let rgba = pollster::block_on(painter.read_rgba(n));
            let img = image::RgbImage::from_fn(n, n, |x, y| {
                let at = 4 * (y * n + x) as usize;
                image::Rgb([rgba[at], rgba[at + 1], rgba[at + 2]])
            });
            if n == 2048 {
                img
            } else {
                image::imageops::resize(&img, 2048, 2048, image::imageops::FilterType::Triangle)
            }
        };
        if timing {
            let recipe = recipe_from(&values);
            let all = layers_from(&switches);
            for (what, layers) in [
                ("all layers", all),
                (
                    "all but the facial hair",
                    head_paint::paint::Layers {
                        facial_hair: false,
                        ..all
                    },
                ),
            ] {
                let mut ms = Vec::new();
                for i in 0..9 {
                    let t = std::time::Instant::now();
                    painter.paint(size, &recipe, &layers);
                    let _ = pollster::block_on(painter.read_rgba(size));
                    if i >= 2 {
                        ms.push(t.elapsed().as_secs_f64() * 1e3);
                    }
                }
                ms.sort_by(f64::total_cmp);
                println!(
                    "{name} {size}: {what}: median {:.1} ms, best {:.1} ms (paint and read back)",
                    ms[ms.len() / 2],
                    ms[0]
                );
            }
            let t = std::time::Instant::now();
            let lay2 = layout();
            let lips = head_paint::lips::LipHead::new(&pos, &lay2.triangles, &lay2.layout_to_mesh);
            let t_lips = t.elapsed().as_secs_f64() * 1e3;
            let beard = head_paint::facial_hair::BeardHead::new(
                &pos,
                &lay2.uv,
                &lay2.triangles,
                &lay2.layout_to_mesh,
                lips.as_ref(),
            )
            .expect("beard");
            println!(
                "{name}: building the grid and its regions: {:.1} ms",
                t.elapsed().as_secs_f64() * 1e3 - t_lips
            );
            let t = std::time::Instant::now();
            let _ = beard.full_field(&groups, None);
            println!(
                "{name}: writing the hair groups onto the grid: {:.1} ms",
                t.elapsed().as_secs_f64() * 1e3
            );
            continue;
        }
        let tex = texture(&values, &switches);
        let mouth = [0, 1, 2].map(|k| 0.5 * (marks.mouth_corner[0][k] + marks.mouth_corner[1][k]));
        let centre3 = add(mouth, f.up, -14.0);
        let a = 35f64.to_radians();
        let views = [
            View {
                across: f.across,
                up: f.up,
                forward: f.forward,
            },
            View {
                across: [0, 1, 2].map(|k| a.cos() * f.across[k] - a.sin() * f.forward[k]),
                up: f.up,
                forward: [0, 1, 2].map(|k| a.cos() * f.forward[k] + a.sin() * f.across[k]),
            },
        ];
        let ours: Vec<KotorFront> = views
            .iter()
            .map(|v| KotorFront::ours(&common, &pos, tex.clone(), v.across, v.up, v.forward))
            .collect();
        let shots: Vec<image::RgbImage> = ours
            .iter()
            .map(|o| o.tile_at(o.project(centre3).0, 120.0, 115.0, 6.0))
            .collect();
        save(&shots[0], &dir.join(format!("{name}_front.jpg")));
        save(&shots[1], &dir.join(format!("{name}_three.jpg")));
        let close_c = add(add(mouth, f.up, -12.0), f.across, 12.0);
        save(
            &ours[0].tile_at(ours[0].project(close_c).0, 50.0, 36.0, 20.0),
            &dir.join(format!("{name}_close.jpg")),
        );
        let alone_switches: Vec<bool> = (0..head_paint::controls::GROUPS.len())
            .map(|i| i == group)
            .collect();
        let alone = KotorFront::ours(
            &common,
            &pos,
            texture(&values, &alone_switches),
            f.across,
            f.up,
            f.forward,
        );
        save(
            &alone.tile_at(alone.project(centre3).0, 120.0, 115.0, 6.0),
            &dir.join(format!("{name}_alone.jpg")),
        );

        let mut bare_switches = switches.clone();
        bare_switches.resize(head_paint::controls::GROUPS.len(), true);
        bare_switches[group] = false;
        let bare_tex = texture(&values, &bare_switches);
        let detail = painter
            .head()
            .and_then(|h| h.detail.as_ref())
            .and_then(|d| d.beard.clone());
        let side = 70f64.to_radians();
        let side_view = View {
            across: [0, 1, 2].map(|k| side.cos() * f.across[k] - side.sin() * f.forward[k]),
            up: f.up,
            forward: [0, 1, 2].map(|k| side.cos() * f.forward[k] + side.sin() * f.across[k]),
        };
        let side_centre = add(add(mouth, f.up, 20.0), f.forward, -45.0);
        if marks_out {
            if let Some(beard) = &detail {
                let side_o = KotorFront::ours(&common, &pos, tex.clone(), side_view.across, side_view.up, side_view.forward);
                let side_k = KotorFront::open_with_hair(name, side_view.across, side_view.up, side_view.forward);
                save(&side_o.tile_at(side_o.project(side_centre).0, 150.0, 150.0, 6.0), &dir.join(format!("{name}_side.jpg")));
                let kside = kotor_dir.join(format!("{name}_side.jpg"));
                if !kside.exists() {
                    save(&side_k.tile_at(side_k.project(side_centre).0, 150.0, 150.0, 6.0), &kside);
                }
                let mut out = serde_json::Map::new();
                for (o, file, centre, (w, h)) in [(&ours[0], "front", centre3, (120.0, 115.0)), (&ours[1], "three", centre3, (120.0, 115.0)), (&side_o, "side", side_centre, (150.0, 150.0))] {
                    let origin = o.project(centre).0;
                    let mut list = Vec::new();
                    for (k, g) in beard.grid.points.iter().enumerate() {
                        if !g.valid {
                            continue;
                        }
                        let (q, depth) = o.project(g.position);
                        if o.front_point(q).is_some_and(|p| o.project(p).1 > depth + 3.0) {
                            continue;
                        }
                        let x = (q[0] - origin[0] + 0.5 * w) * 6.0;
                        let y = (origin[1] - q[1] + 0.5 * h) * 6.0;
                        if (0.0..w * 6.0).contains(&x) && (0.0..h * 6.0).contains(&y) {
                            list.push(serde_json::json!([k % beard.grid.cols, k / beard.grid.cols, x, y]));
                        }
                    }
                    out.insert(file.to_string(), Value::Array(list));
                }
                std::fs::write(dir.join(format!("{name}_marks.json")), Value::Object(out).to_string()).expect("marks");
            }
        }
        for (file, view, centre, (w, h)) in [
            ("regions", &views[0], centre3, (120.0, 115.0)),
            ("regions_side", &side_view, side_centre, (150.0, 150.0)),
        ] {
            let bare = KotorFront::ours(
                &common,
                &pos,
                bare_tex.clone(),
                view.across,
                view.up,
                view.forward,
            );
            let origin = bare.project(centre).0;
            let mut map = bare.tile_at(origin, w, h, 6.0);
            if let Some(beard) = &detail {
                let field = beard.full_field(&groups, edit.as_ref());
                let spacing = beard.grid.spacing_mm();
                if file == "regions" {
                    println!(
                        "{name}: grid of {} by {} points, {} on the skin, spacing {:.2} / {:.2} / {:.2} mm (shortest, median, longest)",
                        beard.grid.cols,
                        beard.grid.rows,
                        beard.grid.points.iter().filter(|p| p.valid).count(),
                        spacing[0],
                        spacing[1],
                        spacing[2]
                    );
                }
                for (k, g) in beard.grid.points.iter().enumerate() {
                    if !g.valid || (beard.reach[k] <= 0.0 && field.density[k] <= 0.0) {
                        continue;
                    }
                    let (q, depth) = bare.project(g.position);
                    // Points hidden behind the skin seen from the front are left out.
                    if bare
                        .front_point(q)
                        .is_some_and(|p| bare.project(p).1 > depth + 3.0)
                    {
                        continue;
                    }
                    if beard.lips_mm[k] < 0.0 {
                        let x = ((q[0] - origin[0] + 0.5 * w) * 6.0) as i64;
                        let y = ((origin[1] - q[1] + 0.5 * h) * 6.0) as i64;
                        if x >= 0
                            && y >= 0
                            && x < i64::from(map.width())
                            && y < i64::from(map.height())
                        {
                            map.put_pixel(x as u32, y as u32, image::Rgb([255, 255, 255]));
                        }
                    }
                    if field.density[k] < 0.01 {
                        continue;
                    }
                    let main = (0..7)
                        .max_by(|&a, &b| beard.regions[k][a].total_cmp(&beard.regions[k][b]))
                        .expect("regions");
                    let x = ((q[0] - origin[0] + 0.5 * w) * 6.0) as i64;
                    let y = ((origin[1] - q[1] + 0.5 * h) * 6.0) as i64;
                    let r = 1 + (3.0 * field.density[k].sqrt()).round() as i64;
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy <= r * r
                                && x + dx >= 0
                                && y + dy >= 0
                                && (x + dx) < i64::from(map.width())
                                && (y + dy) < i64::from(map.height())
                            {
                                map.put_pixel(
                                    (x + dx) as u32,
                                    (y + dy) as u32,
                                    image::Rgb(REGION_RGB[main]),
                                );
                            }
                        }
                    }
                    let d = field.direction[k];
                    let tip = bare.project(add(g.position, d, 2.2)).0;
                    for s in 0..12 {
                        let t = f64::from(s) / 11.0;
                        let px = ((q[0] + t * (tip[0] - q[0]) - origin[0] + 0.5 * w) * 6.0) as i64;
                        let py =
                            ((origin[1] - (q[1] + t * (tip[1] - q[1])) + 0.5 * h) * 6.0) as i64;
                        if px >= 0
                            && py >= 0
                            && px < i64::from(map.width())
                            && py < i64::from(map.height())
                        {
                            map.put_pixel(px as u32, py as u32, image::Rgb([20, 20, 20]));
                        }
                    }
                }
            }
            save(&map, &dir.join(format!("{name}_{file}.jpg")));
        }

        let kfront = kotor_dir.join(format!("{name}_front.jpg"));
        if !kfront.exists() {
            for (v, file) in views.iter().zip(["front", "three"]) {
                let k = KotorFront::open_with_hair(name, v.across, v.up, v.forward);
                save(
                    &k.tile_at(k.project(centre3).0, 120.0, 115.0, 6.0),
                    &kotor_dir.join(format!("{name}_{file}.jpg")),
                );
            }
            let k = KotorFront::open_with_hair(name, f.across, f.up, f.forward);
            save(
                &k.tile_at(k.project(close_c).0, 50.0, 36.0, 20.0),
                &kotor_dir.join(format!("{name}_close.jpg")),
            );
        }
        let open = |p: PathBuf| {
            image::open(&p)
                .unwrap_or_else(|e| panic!("{}: {e}", p.display()))
                .to_rgb8()
        };
        let kf = open(kfront);
        let kt = open(kotor_dir.join(format!("{name}_three.jpg")));
        save(
            &sheet(&[vec![&kf, &kt], vec![&shots[0], &shots[1]]]),
            &dir.join(format!("{name}_sheet.jpg")),
        );
        println!("{name}: done");
    }
}
