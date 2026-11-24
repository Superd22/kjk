// The wrinkle layer (`wrinkles.rs`): the wrinkle sets and the fine lines, as a change of lightness in L*.
// The processor builds the lines, their segments and a grid of 2 mm cells (see `layers/wrinkles.rs` for the layout of the storage
// buffer). This shader reads the cell of the point, finds each line's nearest segment, and sums the lines' cross-sections. It then adds
// the fine lines, which it computes from hashed strokes.

struct WrinkleParams {
    // The fine lines' depth in L*, then three unused words.
    fine: vec4<f32>,
    // The seed's low and high words, 1 if the sets are on, 1 if the fine lines are on.
    flags: vec4<u32>,
}

@group(0) @binding(5) var<uniform> wrinkle_params: WrinkleParams;
@group(0) @binding(6) var<storage, read> wrinkle_data: array<u32>;

const WRINKLE_REACH: f32 = 5.0;
const WRINKLE_RIDGE_BALANCE: f32 = 1.264;
const WRINKLE_CELL_MM: f32 = 2.0;
const WRINKLE_MAX_LINES: u32 = 24u;
const WRINKLE_FINE_CELL_MM: f32 = 8.0;
const WRINKLE_FINE_PER_CELL: u32 = 14u;
const WRINKLE_FINE_WIDTH_MM: f32 = 0.4;

fn wrinkle_f(i: u32) -> f32 {
    return bitcast<f32>(wrinkle_data[i]);
}

fn wrinkle_point(i: u32) -> vec3<f32> {
    return vec3<f32>(wrinkle_f(i), wrinkle_f(i + 1u), wrinkle_f(i + 2u));
}

// `wrinkles::profile`: Bando's cross-section at `x` widths from the furrow, faded to zero between 3 and 5 widths.
fn wrinkle_profile(x: f32) -> f32 {
    if (x >= 5.0) {
        return 0.0;
    }
    let s = -(1.0 - x) * exp(-x);
    if (x <= 1.0) {
        return s;
    }
    if (x <= 3.0) {
        return WRINKLE_RIDGE_BALANCE * s;
    }
    return WRINKLE_RIDGE_BALANCE * s * smooth_step(5.0, 3.0, x);
}

// The change of lightness of the wrinkle sets at a point: for each line near it, the line's depth times its taper and its change along
// its length times the cross-section.
fn wrinkle_sets(p: vec3<f32>) -> f32 {
    let dims = vec3<i32>(i32(wrinkle_data[5]), i32(wrinkle_data[6]), i32(wrinkle_data[7]));
    let rel = vec3<i32>(floor(p / WRINKLE_CELL_MM)) - vec3<i32>(bitcast<i32>(wrinkle_data[2]), bitcast<i32>(wrinkle_data[3]), bitcast<i32>(wrinkle_data[4]));
    if (any(rel < vec3<i32>(0)) || any(rel >= dims)) {
        return 0.0;
    }
    let cell = u32(rel.x + dims.x * (rel.y + dims.y * rel.z));
    let offsets_at = wrinkle_data[8];
    let entries_at = wrinkle_data[9];
    let lines_at = wrinkle_data[10];
    let segments_at = wrinkle_data[11];
    let first = wrinkle_data[offsets_at + cell];
    let last = wrinkle_data[offsets_at + cell + 1u];
    // The nearest point of each line: its distance and its share along the line.
    var ids: array<u32, 24>;
    var dist: array<f32, 24>;
    var share: array<f32, 24>;
    var used = 0u;
    for (var k = first; k < last; k = k + 1u) {
        let at = segments_at + 9u * wrinkle_data[entries_at + k];
        let a = wrinkle_point(at);
        let b = wrinkle_point(at + 3u);
        let s0 = wrinkle_f(at + 6u);
        let s1 = wrinkle_f(at + 7u);
        let line = wrinkle_data[at + 8u];
        let ab = b - a;
        let ap = p - a;
        let len2 = dot(ab, ab);
        var t = 0.0;
        if (len2 > 0.0) {
            t = clamp(dot(ap, ab) / len2, 0.0, 1.0);
        }
        let d = length(ap - t * ab);
        let s = s0 + t * (s1 - s0);
        var found = false;
        for (var j = 0u; j < used; j = j + 1u) {
            if (ids[j] == line) {
                found = true;
                if (d < dist[j]) {
                    dist[j] = d;
                    share[j] = s;
                }
                break;
            }
        }
        if (!found && used < WRINKLE_MAX_LINES) {
            ids[used] = line;
            dist[used] = d;
            share[used] = s;
            used = used + 1u;
        }
    }
    var shift = 0.0;
    for (var j = 0u; j < used; j = j + 1u) {
        let at = lines_at + 8u * ids[j];
        let depth = wrinkle_f(at);
        let width = wrinkle_f(at + 1u);
        let kept = wrinkle_f(at + 2u);
        let total = wrinkle_f(at + 3u);
        let open_anchor = wrinkle_data[at + 4u] != 0u;
        let x = dist[j] / width;
        if (x >= WRINKLE_REACH) {
            continue;
        }
        let s = share[j];
        var fade_in = 1.0;
        if (!open_anchor) {
            fade_in = smooth_step(0.0, 0.12, s);
        }
        let taper = fade_in * (1.0 - smooth_step(kept * 0.66, kept, s));
        let along = 1.0 + 0.35 * line_noise(s * total / 6.0, vec2<u32>(wrinkle_data[at + 5u], wrinkle_data[at + 6u]));
        shift = shift + depth * taper * along * wrinkle_profile(x);
    }
    return shift;
}

// The fine lines: short strokes scattered through space, 14 for each cube of 8 mm, along the flow of the face.
fn wrinkle_fine(p: vec3<f32>, seed: vec2<u32>) -> f32 {
    let c = vec3<i32>(floor(p / WRINKLE_FINE_CELL_MM));
    let reach = WRINKLE_REACH * WRINKLE_FINE_WIDTH_MM;
    var furrow = 0.0;
    var ridge = 0.0;
    for (var dx = -1; dx <= 1; dx = dx + 1) {
        for (var dy = -1; dy <= 1; dy = dy + 1) {
            for (var dz = -1; dz <= 1; dz = dz + 1) {
                let cc = c + vec3<i32>(dx, dy, dz);
                for (var k = 0u; k < WRINKLE_FINE_PER_CELL; k = k + 1u) {
                    let h = hash3(cc.x, cc.y, cc.z, vec2<u32>(seed.x ^ (0xF1E0u + k), seed.y));
                    let q = vec3<f32>(cc) * WRINKLE_FINE_CELL_MM + vec3<f32>(hash_unit(h, 0u), hash_unit(h, 16u), hash_unit(h, 32u)) * WRINKLE_FINE_CELL_MM;
                    let half_length = 2.5 + 3.0 * hash_unit(h, 48u);
                    let qp = p - q;
                    let near = half_length + reach;
                    if (dot(qp, qp) > near * near) {
                        continue;
                    }
                    let t = marks_flow(q);
                    let along = dot(qp, t);
                    if (abs(along) >= half_length) {
                        continue;
                    }
                    let d = length(qp - along * t);
                    if (d >= reach) {
                        continue;
                    }
                    let f = 1.0 - (along / half_length) * (along / half_length);
                    let fade = f * f;
                    let strength = 0.4 + 0.6 * hash_unit(hash3(cc.x, cc.y, cc.z, vec2<u32>(seed.x ^ (0xBEEFu + k), seed.y)), 0u);
                    let v = strength * fade * wrinkle_profile(d / WRINKLE_FINE_WIDTH_MM);
                    furrow = min(furrow, v);
                    ridge = max(ridge, v);
                }
            }
        }
    }
    return furrow + ridge;
}

// The two parts of `wrinkle_shift` apart, for the maps: x the sets and y the fine lines. Their sum is `wrinkle_shift`.
fn wrinkle_parts(pt: Point) -> vec2<f32> {
    let p = pt.position_mm;
    var parts = vec2<f32>(0.0);
    if (wrinkle_params.flags.z != 0u) {
        parts.x = wrinkle_sets(p);
    }
    if (wrinkle_params.flags.w != 0u) {
        let skin = marks_skin(p);
        if (skin != 0.0) {
            parts.y = wrinkle_params.fine.x * skin * wrinkle_fine(p, wrinkle_params.flags.xy);
        }
    }
    return parts;
}

// the sum of the wrinkle layer, without the folds and the forehead lines: the sets, plus the fine lines' depth times the skin texture
// times the fine lines.
fn wrinkle_shift(pt: Point) -> f32 {
    let p = pt.position_mm;
    var shift = 0.0;
    if (wrinkle_params.flags.z != 0u) {
        shift = wrinkle_sets(p);
    }
    if (wrinkle_params.flags.w != 0u) {
        let skin = marks_skin(p);
        if (skin != 0.0) {
            shift = shift + wrinkle_params.fine.x * skin * wrinkle_fine(p, wrinkle_params.flags.xy);
        }
    }
    return shift;
}
