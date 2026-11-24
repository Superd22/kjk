//! The wrinkle layer, with the fine lines.
//!
//! This half builds, on the processor, every drawn line with its depth, width, kept share and wobbled
//! points, the segments of those lines, and a grid of 2 mm cells that lists the segments near each cell. It packs them into the storage
//! buffer. The shader (`wrinkles.wgsl`) does the lookup in the grid, the cross-section of each line and the fine lines.
//!
//! Storage buffer, in 32-bit words:
//!
//! | Words                  | Content                                                                                              |
//! |------------------------|------------------------------------------------------------------------------------------------------|
//! | 0 to 15                | Header: line count, segment count, the grid's first cell (3 `i32`), its size in cells (3), then the    |
//! |                        | word index where the cell offsets start, where the segment lists start, where the lines start and      |
//! |                        | where the segments start.                                                                              |
//! | cell offsets           | `cells + 1` numbers. Cell `x + nx * (y + ny * z)` lists the entries from its offset to the next one.    |
//! | segment lists          | One segment number per entry.                                                                          |
//! | lines, 8 words each    | depth, width in mm, kept share, total length in mm (all `f32`), 1 if the line is open-anchored, the     |
//! |                        | seed of the line's strength noise (low then high word), 0.                                             |
//! | segments, 9 words each | start point (3 `f32`), end point (3), the line's share at the start, at the end, the line's number.     |
//!
//! Uniform (8 words): the fine lines' depth, 3 unused words, the seed (low then high word), 1 if the sets are on, 1 if the fine lines
//! are on.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::layout_points::{cross, normalize, sub};
use head_paint::lines::LineField;
use head_paint::noise::{hash3, line_noise, unit};
use head_paint::wrinkles::{HeadLines, REACH, SETS, Wrinkles};

/// The edge of a grid cell in millimetres.
const CELL_MM: f64 = 2.0;
const HEADER: usize = 16;
const LINE_WORDS: usize = 8;
const SEGMENT_WORDS: usize = 9;

struct Line {
    depth: f64,
    width_mm: f64,
    kept: f64,
    total_mm: f64,
    open_anchor: bool,
    along_seed: u64,
}

struct Segment {
    a: [f64; 3],
    b: [f64; 3],
    s: [f64; 2],
    line: u32,
}

#[derive(Default)]
pub struct WrinkleLayer {
    /// The wrinkle settings, without the fine lines' depth, and the forehead lines' effect on the sets, that the storage was built for.
    built: Option<(Wrinkles, bool)>,
    /// Whether the forehead lines paint anything, for the forehead sliders it was found for. Building the lines costs about 1 ms, so it
    /// is not done again while those sliders stay the same.
    lines_paint: Option<(head_paint::lines::ForeheadLines, bool)>,
}

fn cell_of(p: [f64; 3]) -> [i32; 3] {
    p.map(|x| (x / CELL_MM).floor() as i32)
}

/// What the layer keeps of a recipe and a head, as lines and segments.
fn build(recipe: &Wrinkles, head: &HeadLines) -> (Vec<Line>, Vec<Segment>) {
    let (mut lines, mut segments) = (Vec::new(), Vec::new());
    for line in &head.lines {
        let set = &recipe.sets[line.set];
        let share = (set.count - line.rank as f64).clamp(0.0, 1.0);
        let depth = recipe.set_depth(line.set) * share;
        if depth == 0.0 {
            continue;
        }
        let h = hash3(line.key as i64, 11, 0, recipe.seed);
        let kept = set.length.clamp(0.05, 1.0) * (0.8 + 0.2 * unit(h, 0));
        let depth = depth * (0.6 + 0.4 * unit(h, 16));
        let width_mm = SETS[line.set].width_mm * set.width * recipe.width;
        let total_mm = *line.along_mm.last().unwrap_or(&0.0);
        let id = lines.len() as u32;
        let open_anchor = SETS[line.set].open_anchor;
        lines.push(Line { depth, width_mm, kept, total_mm, open_anchor, along_seed: recipe.seed ^ line.key.wrapping_mul(97) ^ 0x55 });
        let wobbled: Vec<[f64; 3]> = line
            .points
            .iter()
            .enumerate()
            .map(|(i, &p)| {
                let next = line.points[(i + 1).min(line.points.len() - 1)];
                let prev = line.points[i.saturating_sub(1)];
                let tangent = normalize(sub(next, prev));
                let side = normalize(cross(tangent, line.normals[i]));
                let mirror = if line.key & 1 == 1 { -1.0 } else { 1.0 };
                let w = mirror * set.shift_mm + recipe.wobble_mm * line_noise(line.along_mm[i] / 7.0, recipe.seed ^ line.key.wrapping_mul(31));
                [p[0] + w * side[0], p[1] + w * side[1], p[2] + w * side[2]]
            })
            .collect();
        for i in 0..wobbled.len() - 1 {
            let mut s = [line.along_mm[i] / total_mm.max(1e-9), line.along_mm[i + 1] / total_mm.max(1e-9)];
            if open_anchor {
                s = s.map(|x| (2.0 * x - 1.0).abs());
            }
            if s[0].min(s[1]) > kept {
                continue;
            }
            segments.push(Segment { a: wobbled[i], b: wobbled[i + 1], s, line: id });
        }
    }
    (lines, segments)
}

/// The storage words for some lines and segments, with the grid of the segments near each cell.
fn pack(lines: &[Line], segments: &[Segment]) -> Vec<u32> {
    let reach_of = |s: &Segment| REACH * lines[s.line as usize].width_mm + 0.01;
    let range = |s: &Segment| {
        let reach = reach_of(s);
        (cell_of([0, 1, 2].map(|c| s.a[c].min(s.b[c]) - reach)), cell_of([0, 1, 2].map(|c| s.a[c].max(s.b[c]) + reach)))
    };
    let mut lo = [i32::MAX; 3];
    let mut hi = [i32::MIN; 3];
    for s in segments {
        let (l, h) = range(s);
        for c in 0..3 {
            lo[c] = lo[c].min(l[c]);
            hi[c] = hi[c].max(h[c]);
        }
    }
    if segments.is_empty() {
        lo = [0; 3];
        hi = [-1; 3];
    }
    let dims = [0, 1, 2].map(|c| (hi[c] - lo[c] + 1) as usize);
    let cells = dims[0] * dims[1] * dims[2];
    let linear = |x: i32, y: i32, z: i32| (x - lo[0]) as usize + dims[0] * ((y - lo[1]) as usize + dims[1] * (z - lo[2]) as usize);
    let mut offsets = vec![0u32; cells + 1];
    for s in segments {
        let (l, h) = range(s);
        for x in l[0]..=h[0] {
            for y in l[1]..=h[1] {
                for z in l[2]..=h[2] {
                    offsets[linear(x, y, z) + 1] += 1;
                }
            }
        }
    }
    for i in 0..cells {
        offsets[i + 1] += offsets[i];
    }
    let mut fill = offsets.clone();
    let mut entries = vec![0u32; offsets[cells] as usize];
    for (k, s) in segments.iter().enumerate() {
        let (l, h) = range(s);
        for x in l[0]..=h[0] {
            for y in l[1]..=h[1] {
                for z in l[2]..=h[2] {
                    let c = linear(x, y, z);
                    entries[fill[c] as usize] = k as u32;
                    fill[c] += 1;
                }
            }
        }
    }
    let offsets_at = HEADER;
    let entries_at = offsets_at + offsets.len();
    let lines_at = entries_at + entries.len();
    let segments_at = lines_at + LINE_WORDS * lines.len();
    let mut words = Vec::with_capacity(segments_at + SEGMENT_WORDS * segments.len());
    words.extend([lines.len() as u32, segments.len() as u32, lo[0] as u32, lo[1] as u32, lo[2] as u32, dims[0] as u32, dims[1] as u32, dims[2] as u32]);
    words.extend([offsets_at as u32, entries_at as u32, lines_at as u32, segments_at as u32, 0, 0, 0, 0]);
    words.extend(offsets);
    words.extend(entries);
    for l in lines {
        words.extend([(l.depth as f32).to_bits(), (l.width_mm as f32).to_bits(), (l.kept as f32).to_bits(), (l.total_mm as f32).to_bits()]);
        words.extend([u32::from(l.open_anchor), l.along_seed as u32, (l.along_seed >> 32) as u32, 0]);
    }
    for s in segments {
        words.extend(s.a.map(|v| (v as f32).to_bits()));
        words.extend(s.b.map(|v| (v as f32).to_bits()));
        words.extend(s.s.map(|v| (v as f32).to_bits()));
        words.push(s.line);
    }
    words
}

impl GpuLayer for WrinkleLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let w = &input.recipe.wrinkles;
        let detail = input.head.detail.as_ref();
        let on = input.layers.wrinkles && detail.is_some();
        let sets_on = on && w.depth != 0.0;
        let fine_on = on && w.fine != 0.0;
        if input.new_head {
            self.built = None;
            self.lines_paint = None;
        }
        let mut storage = None;
        if sets_on {
            // Where the forehead lines paint, the old forehead set is turned off.
            let d = detail.expect("detail");
            let forehead = input.recipe.forehead;
            let lines_replace = match self.lines_paint {
                Some((k, paints)) if k == forehead => paints,
                _ => {
                    let paints = !forehead.is_none() && d.forehead.as_ref().is_some_and(|head| !LineField::new(&forehead, head).is_empty());
                    self.lines_paint = Some((forehead, paints));
                    paints
                }
            };
            let key = (Wrinkles { fine: 0.0, ..*w }, lines_replace);
            if self.built.as_ref() != Some(&key) {
                let mut recipe = key.0;
                if lines_replace {
                    for (set, def) in SETS.iter().enumerate() {
                        if def.id == "forehead" {
                            recipe.sets[set].strength = 0.0;
                        }
                    }
                }
                let (lines, segments) = build(&recipe, &d.lines);
                storage = Some(pack(&lines, &segments));
                self.built = Some(key);
            }
        }
        let seed = w.seed;
        let uniform = vec![(w.fine as f32).to_bits(), 0, 0, 0, seed as u32, (seed >> 32) as u32, u32::from(sets_on), u32::from(fine_on)];
        LayerWords { uniform, storage }
    }

    fn paints(&self, id: &str) -> bool {
        let controls = head_paint::controls::controls();
        controls.iter().any(|c| c.id == id && c.group == "wrinkles" && !id.starts_with("lines_"))
    }
}
