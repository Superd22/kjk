//! The forehead lines. The processor runs the geometry stage (`LineField::new`, which grows the lines and cuts them into segments) and
//! packs the segments and a grid of 2 mm cells into the storage buffer. The shader runs the per-pixel stage, which finds the nearest segment of each line and applies the cross-section.
//!
//! Storage buffer, in 32-bit words:
//! - Words 0 to 7, the header: the grid's lowest cell column and row (as `i32`), its width and height in cells, the word where the cells'
//!   index lists start, the word where the segments start, and two unused words.
//! - From word 8, one pair of words for each cell, row by row from the lowest row: the cell's first position in the index lists and the
//!   number of segments in the cell.
//! - Then the index lists: the numbers of the segments that reach each cell, in the order the processor keeps them.
//! - Then the segments, 8 words each: the two ends' (a, v) as `f32` (4 words), the strength at each end as `f32` (2 words), the line's
//!   number and one unused word.
//!
//! Uniform buffer: four `vec4<f32>`. The first holds the forehead's `across` and the depth of its middle, the second `up` and the half
//! width of a line, the third `forward` and the ridge, the fourth the origin and a 1 when the lines paint.

use super::{GpuLayer, LayerInput, LayerWords};
use head_paint::lines::{ForeheadHead, ForeheadLines, LineField};

const CELL_MM: f64 = 2.0;
const HEADER_WORDS: usize = 8;
const SEGMENT_WORDS: usize = 8;

fn cell_of(a: f64, v: f64) -> [i32; 2] {
    [(a / CELL_MM).floor() as i32, (v / CELL_MM).floor() as i32]
}

/// What the packed buffers were made from: the head's frame and the ten sliders as bits, or nothing when the lines are off.
#[derive(PartialEq)]
struct Key(Option<(ForeheadHead, [u64; 10])>);

#[derive(Default)]
pub struct LineLayer {
    key: Option<Key>,
    /// The uniform's words that depend on the head and the sliders, and 1 in the last word when the lines paint.
    uniform: Vec<u32>,
}

fn bits(s: &ForeheadLines) -> [u64; 10] {
    [s.amount, s.spacing, s.continuity, s.waviness, f64::from(s.variation), s.depth, s.sharpness, s.ridge, s.dip_mm, s.reach].map(f64::to_bits)
}

fn f(v: f64) -> u32 {
    (v as f32).to_bits()
}

fn off() -> (Vec<u32>, Vec<u32>) {
    let mut storage = vec![0; HEADER_WORDS];
    storage[2] = 0;
    (vec![0; 16], storage)
}

/// The uniform and storage words of a set of lines on a head.
fn pack(sliders: &ForeheadLines, head: &ForeheadHead) -> (Vec<u32>, Vec<u32>) {
    let field = LineField::new(sliders, head);
    if field.is_empty() || field.segments().is_empty() {
        return off();
    }
    let half_mm = sliders.half_width_mm();
    let reach = 5.5 * half_mm;
    let segments = field.segments();
    let ranges: Vec<([i32; 2], [i32; 2])> = segments
        .iter()
        .map(|s| (cell_of(s.a[0].min(s.b[0]) - reach, s.a[1].min(s.b[1]) - reach), cell_of(s.a[0].max(s.b[0]) + reach, s.a[1].max(s.b[1]) + reach)))
        .collect();
    let lo = [ranges.iter().map(|r| r.0[0]).min().unwrap(), ranges.iter().map(|r| r.0[1]).min().unwrap()];
    let hi = [ranges.iter().map(|r| r.1[0]).max().unwrap(), ranges.iter().map(|r| r.1[1]).max().unwrap()];
    let (w, h) = ((hi[0] - lo[0] + 1) as usize, (hi[1] - lo[1] + 1) as usize);
    let mut lists: Vec<Vec<u32>> = vec![Vec::new(); w * h];
    for (n, r) in ranges.iter().enumerate() {
        for y in r.0[1]..=r.1[1] {
            for x in r.0[0]..=r.1[0] {
                lists[(y - lo[1]) as usize * w + (x - lo[0]) as usize].push(n as u32);
            }
        }
    }
    let index_start = HEADER_WORDS + 2 * w * h;
    let total_indices: usize = lists.iter().map(Vec::len).sum();
    let segment_start = index_start + total_indices;
    let mut storage = vec![0u32; segment_start + SEGMENT_WORDS * segments.len()];
    storage[0] = lo[0] as u32;
    storage[1] = lo[1] as u32;
    storage[2] = w as u32;
    storage[3] = h as u32;
    storage[4] = index_start as u32;
    storage[5] = segment_start as u32;
    let mut at = 0;
    for (c, list) in lists.iter().enumerate() {
        storage[HEADER_WORDS + 2 * c] = at as u32;
        storage[HEADER_WORDS + 2 * c + 1] = list.len() as u32;
        storage[index_start + at..index_start + at + list.len()].copy_from_slice(list);
        at += list.len();
    }
    for (n, s) in segments.iter().enumerate() {
        let o = segment_start + SEGMENT_WORDS * n;
        storage[o..o + 7].copy_from_slice(&[f(s.a[0]), f(s.a[1]), f(s.b[0]), f(s.b[1]), f(s.strength[0]), f(s.strength[1]), s.line]);
    }
    while storage.len() % 4 != 0 {
        storage.push(0);
    }
    let o = head.origin;
    let uniform = vec![
        f(head.across[0]), f(head.across[1]), f(head.across[2]), f(head.middle_depth),
        f(head.up[0]), f(head.up[1]), f(head.up[2]), f(half_mm),
        f(head.forward[0]), f(head.forward[1]), f(head.forward[2]), f(sliders.ridge.clamp(0.0, 1.0)),
        f(o[0]), f(o[1]), f(o[2]), 1.0f32.to_bits(),
    ];
    (uniform, storage)
}

impl GpuLayer for LineLayer {
    fn words(&mut self, input: &LayerInput) -> LayerWords {
        let sliders = &input.recipe.forehead;
        let head = input.head.detail.as_ref().and_then(|d| d.forehead.as_ref());
        let wanted = head.filter(|_| input.layers.wrinkles && !sliders.is_none());
        let key = Key(wanted.map(|h| (*h, bits(sliders))));
        let mut storage = None;
        if input.new_head || self.key.as_ref() != Some(&key) {
            let (uniform, words) = match wanted {
                Some(h) => pack(sliders, h),
                None => off(),
            };
            self.uniform = uniform;
            storage = Some(words);
            self.key = Some(key);
        }
        LayerWords { uniform: self.uniform.clone(), storage }
    }

    fn paints(&self, id: &str) -> bool {
        id.starts_with("lines_")
    }
}
