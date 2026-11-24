// The forehead lines (`head_paint::lines`), as a change of lightness in L*. They share the wrinkles'
// switch. The processor grows the lines and packs their segments and a grid of 2 mm cells into `lines_data` (see `layers/lines.rs`).

struct LinesParams {
    across: vec4<f32>,
    up: vec4<f32>,
    forward: vec4<f32>,
    origin: vec4<f32>,
}

@group(0) @binding(10) var<uniform> lines_params: LinesParams;
@group(0) @binding(11) var<storage, read> lines_data: array<u32>;

const LINES_HEADER: u32 = 8u;
const LINES_SEGMENT_WORDS: u32 = 8u;
const LINES_CELL_MM: f32 = 2.0;
const LINES_MAX_LINES: u32 = 16u;
const LINES_POWER: f32 = 1.6;
// ln(2) ^ (1 / 1.6)
const LINES_PROFILE_W: f32 = 0.8003;

fn lines_f(i: u32) -> f32 {
    return bitcast<f32>(lines_data[i]);
}

// `folds::profile`.
fn lines_profile(d: f32, half_mm: f32) -> f32 {
    let x = abs(d) / half_mm;
    if (x >= 3.5) {
        return 0.0;
    }
    let y = x * LINES_PROFILE_W;
    var v = 1.0;
    if (y > 0.0) {
        v = exp(-exp(LINES_POWER * log(y)));
    }
    if (x <= 2.5) {
        return v;
    }
    return v * smooth_step(3.5, 2.5, x);
}

fn lines_shift(pt: Point) -> f32 {
    if (lines_params.origin.w == 0.0) {
        return 0.0;
    }
    let r = pt.position_mm - lines_params.origin.xyz;
    let a = dot(r, lines_params.across.xyz);
    let v = dot(r, lines_params.up.xyz);
    let z = dot(r, lines_params.forward.xyz);
    let cx = i32(floor(a / LINES_CELL_MM)) - bitcast<i32>(lines_data[0]);
    let cy = i32(floor(v / LINES_CELL_MM)) - bitcast<i32>(lines_data[1]);
    let w = i32(lines_data[2]);
    let h = i32(lines_data[3]);
    if (cx < 0 || cy < 0 || cx >= w || cy >= h) {
        return 0.0;
    }
    let depth = lines_params.across.w;
    let facing = smooth_step(depth - 45.0, depth - 30.0, z);
    if (facing == 0.0) {
        return 0.0;
    }
    let cell = LINES_HEADER + 2u * u32(cy * w + cx);
    let first = lines_data[cell];
    let count = lines_data[cell + 1u];
    let index_base = lines_data[4];
    let segment_base = lines_data[5];

    var n_line: array<u32, 16>;
    var n_dist: array<f32, 16>;
    var n_signed: array<f32, 16>;
    var n_strength: array<f32, 16>;
    var used = 0u;
    let p = vec2<f32>(a, v);
    for (var i = 0u; i < count; i = i + 1u) {
        let o = segment_base + LINES_SEGMENT_WORDS * lines_data[index_base + first + i];
        let sa = vec2<f32>(lines_f(o), lines_f(o + 1u));
        let sb = vec2<f32>(lines_f(o + 2u), lines_f(o + 3u));
        let ab = sb - sa;
        let ap = p - sa;
        let len2 = dot(ab, ab);
        var t = 0.0;
        if (len2 > 0.0) {
            t = clamp(dot(ap, ab) / len2, 0.0, 1.0);
        }
        let off = ap - t * ab;
        let d = length(off);
        var signed = -d;
        if (ab.x * off.y - ab.y * off.x >= 0.0) {
            signed = d;
        }
        let s0 = lines_f(o + 4u);
        let s1 = lines_f(o + 5u);
        let strength = s0 + t * (s1 - s0);
        let line = lines_data[o + 6u];
        var slot = LINES_MAX_LINES;
        for (var k = 0u; k < used; k = k + 1u) {
            if (n_line[k] == line) {
                slot = k;
                break;
            }
        }
        if (slot < LINES_MAX_LINES) {
            if (d < n_dist[slot]) {
                n_dist[slot] = d;
                n_signed[slot] = signed;
                n_strength[slot] = strength;
            }
        } else if (used < LINES_MAX_LINES) {
            n_line[used] = line;
            n_dist[used] = d;
            n_signed[used] = signed;
            n_strength[used] = strength;
            used = used + 1u;
        }
    }
    let half_mm = lines_params.up.w;
    let ridge = lines_params.forward.w;
    var shift = 0.0;
    for (var k = 0u; k < used; k = k + 1u) {
        shift = shift + n_strength[k] * (ridge * lines_profile(n_signed[k] + 2.0 * half_mm, half_mm) - lines_profile(n_signed[k], half_mm));
    }
    return facing * shift;
}
