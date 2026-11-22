// Shared helpers of the paint shaders: the frame's settings, a point of the head, 64-bit hashes, solid noise and colour maths.
// Each function mirrors the function of `head_paint` named beside it, in 32-bit floats. The hashes are exact: they give the same bits
// as `noise::hash3`, so every yes-or-no choice taken from a hash is the processor's.

struct Frame {
    // The colour under every layer, in linear sRGB: the base tone, or the neutral grey when the base tone is off.
    base: vec4<f32>,
    // The head's origin in millimetres, a whole number, which the paint pass adds to the interpolated point.
    origin: vec4<f32>,
    // 1 when a layer is on: the shading, the sheen, the wrinkles and the pores.
    on: vec4<u32>,
    // The background's sRGB bytes, packed as the output texels are.
    background: u32,
    resolution: u32,
    pad0: u32,
    pad1: u32,
}

@group(0) @binding(0) var<uniform> frame: Frame;

// What the layers read at one point of the head: the sky light and the bump of the shading, the weights of the sheen's regions and the
// point itself in millimetres.
struct Point {
    sky: f32,
    bump_mm: f32,
    sheen: vec3<f32>,
    position_mm: vec3<f32>,
}

// ---- 64-bit unsigned numbers as two 32-bit halves: x is the low half, y the high half ----

fn u64_of(lo: u32, hi: u32) -> vec2<u32> {
    return vec2<u32>(lo, hi);
}

// A signed 32-bit number widened to 64 bits as `v as u64` does with an i64.
fn u64_from_i32(v: i32) -> vec2<u32> {
    return vec2<u32>(bitcast<u32>(v), select(0u, 0xFFFFFFFFu, v < 0));
}

// The full 64-bit product of two 32-bit numbers.
fn mul_wide(a: u32, b: u32) -> vec2<u32> {
    let a0 = a & 0xFFFFu;
    let a1 = a >> 16u;
    let b0 = b & 0xFFFFu;
    let b1 = b >> 16u;
    let p00 = a0 * b0;
    let p01 = a0 * b1;
    let p10 = a1 * b0;
    let p11 = a1 * b1;
    let mid = (p00 >> 16u) + (p01 & 0xFFFFu) + (p10 & 0xFFFFu);
    let lo = (p00 & 0xFFFFu) | (mid << 16u);
    let hi = p11 + (p01 >> 16u) + (p10 >> 16u) + (mid >> 16u);
    return vec2<u32>(lo, hi);
}

// `a.wrapping_mul(b)`.
fn u64_mul(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let low = mul_wide(a.x, b.x);
    return vec2<u32>(low.x, low.y + a.x * b.y + a.y * b.x);
}

// `a.wrapping_add(b)`.
fn u64_add(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let lo = a.x + b.x;
    return vec2<u32>(lo, a.y + b.y + select(0u, 1u, lo < a.x));
}

// `a >> n` for n from 0 to 63.
fn u64_shr(a: vec2<u32>, n: u32) -> vec2<u32> {
    if (n == 0u) {
        return a;
    }
    if (n >= 32u) {
        return vec2<u32>(a.y >> (n - 32u), 0u);
    }
    return vec2<u32>((a.x >> n) | (a.y << (32u - n)), a.y >> n);
}

// `a % m` for a small m.
fn u64_mod(a: vec2<u32>, m: u32) -> u32 {
    // 2^32 mod m, computed as (2^32 - m) mod m.
    let base = (0u - m) % m;
    return ((a.y % m) * base + a.x % m) % m;
}

// ---- `noise.rs` ----

const HASH_START: vec2<u32> = vec2<u32>(0x7F4A7C15u, 0x9E3779B9u);
const HASH_M1: vec2<u32> = vec2<u32>(0x1CE4E5B9u, 0xBF58476Du);
const HASH_M2: vec2<u32> = vec2<u32>(0x133111EBu, 0x94D049BBu);

fn hash_round(h_in: vec2<u32>, v: i32) -> vec2<u32> {
    var h = h_in ^ u64_mul(u64_from_i32(v), HASH_M1);
    h = u64_mul(h ^ u64_shr(h, 30u), HASH_M1);
    h = u64_mul(h ^ u64_shr(h, 27u), HASH_M2);
    return h ^ u64_shr(h, 31u);
}

// `noise::hash3`: three cell coordinates and a seed mixed into 64 random bits.
fn hash3(i: i32, j: i32, k: i32, seed: vec2<u32>) -> vec2<u32> {
    var h = seed ^ HASH_START;
    h = hash_round(h, i);
    h = hash_round(h, j);
    h = hash_round(h, k);
    return h;
}

// `noise::unit`: a number from 0 to 1 from the 16 bits of a hash that start at bit `shift`.
fn hash_unit(h: vec2<u32>, shift: u32) -> f32 {
    return f32(u64_shr(h, shift).x & 0xFFFFu) / 65536.0;
}

fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn gradient(h: vec2<u32>, d: vec3<f32>) -> f32 {
    switch (u64_mod(h, 12u)) {
        case 0u: { return d.x + d.y; }
        case 1u: { return -d.x + d.y; }
        case 2u: { return d.x - d.y; }
        case 3u: { return -d.x - d.y; }
        case 4u: { return d.x + d.z; }
        case 5u: { return -d.x + d.z; }
        case 6u: { return d.x - d.z; }
        case 7u: { return -d.x - d.z; }
        case 8u: { return d.y + d.z; }
        case 9u: { return -d.y + d.z; }
        case 10u: { return d.y - d.z; }
        default: { return -d.y - d.z; }
    }
}

// `noise::gradient_noise`: Perlin's gradient noise with a feature size of one unit.
fn gradient_noise(p: vec3<f32>, seed: vec2<u32>) -> f32 {
    let cell = floor(p);
    let f = p - cell;
    let c = vec3<i32>(cell);
    let u = vec3<f32>(fade(f.x), fade(f.y), fade(f.z));
    var corners: array<f32, 8>;
    for (var n = 0u; n < 8u; n = n + 1u) {
        let o = vec3<i32>(i32(n & 1u), i32((n >> 1u) & 1u), i32((n >> 2u) & 1u));
        let h = hash3(c.x + o.x, c.y + o.y, c.z + o.z, seed);
        corners[n] = gradient(h, f - vec3<f32>(o));
    }
    let x00 = mix(corners[0], corners[1], u.x);
    let x10 = mix(corners[2], corners[3], u.x);
    let x01 = mix(corners[4], corners[5], u.x);
    let x11 = mix(corners[6], corners[7], u.x);
    return mix(mix(x00, x10, u.y), mix(x01, x11, u.y), u.z);
}

// `noise::fractal_noise`: gradient noise summed over octaves, the first with features of `wavelength` millimetres.
fn fractal_noise(p: vec3<f32>, wavelength: f32, octaves: u32, gain: f32, seed: vec2<u32>) -> f32 {
    var sum = 0.0;
    var total = 0.0;
    var amplitude = 1.0;
    var scale = 1.0 / wavelength;
    for (var o = 0u; o < octaves; o = o + 1u) {
        sum = sum + amplitude * gradient_noise(p * scale, u64_add(seed, u64_of(o * 7919u, 0u)));
        total = total + amplitude;
        amplitude = amplitude * gain;
        scale = scale * 2.0;
    }
    return sum / total;
}

// `noise::line_noise`: gradient noise along a line.
fn line_noise(t: f32, seed: vec2<u32>) -> f32 {
    return gradient_noise(vec3<f32>(t, 0.37, 0.61), seed);
}

// ---- `pigment.rs`, `colour.rs` and `shading.rs` ----

// `pigment::encode`: a linear sRGB channel encoded, keeping its sign.
fn encode(c: f32) -> f32 {
    let m = abs(c);
    var e: f32;
    if (m > 0.0031308) {
        e = 1.055 * pow(m, 1.0 / 2.4) - 0.055;
    } else {
        e = 12.92 * m;
    }
    return select(e, -e, c < 0.0);
}

// `pigment::decode`.
fn decode(c: f32) -> f32 {
    if (c > 0.04045) {
        return pow((c + 0.055) / 1.055, 2.4);
    }
    return c / 12.92;
}

fn encode3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(encode(c.x), encode(c.y), encode(c.z));
}

fn decode3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(decode(c.x), decode(c.y), decode(c.z));
}

const LAB_WHITE: vec3<f32> = vec3<f32>(0.95047, 1.0, 1.08883);
const LAB_EPSILON: f32 = 216.0 / 24389.0;
const LAB_KAPPA: f32 = 24389.0 / 27.0;

fn lab_f(t: f32) -> f32 {
    if (t > LAB_EPSILON) {
        return pow(t, 1.0 / 3.0);
    }
    return (LAB_KAPPA * t + 16.0) / 116.0;
}

// `colour::srgb_to_lab`: an encoded sRGB colour to Lab.
fn srgb_to_lab(rgb: vec3<f32>) -> vec3<f32> {
    let l = decode3(rgb);
    let xyz = vec3<f32>(
        0.4124 * l.x + 0.3576 * l.y + 0.1805 * l.z,
        0.2126 * l.x + 0.7152 * l.y + 0.0722 * l.z,
        0.0193 * l.x + 0.1192 * l.y + 0.9505 * l.z,
    );
    let f = vec3<f32>(lab_f(xyz.x / LAB_WHITE.x), lab_f(xyz.y / LAB_WHITE.y), lab_f(xyz.z / LAB_WHITE.z));
    return vec3<f32>(116.0 * f.y - 16.0, 500.0 * (f.x - f.y), 200.0 * (f.y - f.z));
}

fn lab_f_inverse(f: f32) -> f32 {
    let cube = f * f * f;
    if (cube > LAB_EPSILON) {
        return cube;
    }
    return (116.0 * f - 16.0) / LAB_KAPPA;
}

// `colour::lab_to_srgb`: a Lab colour to encoded sRGB, clamped to 0..1.
fn lab_to_srgb(lab: vec3<f32>) -> vec3<f32> {
    let fy = (lab.x + 16.0) / 116.0;
    let xyz = vec3<f32>(lab_f_inverse(fy + lab.y / 500.0), lab_f_inverse(fy), lab_f_inverse(fy - lab.z / 200.0)) * LAB_WHITE;
    let lin = vec3<f32>(
        3.2406 * xyz.x - 1.5372 * xyz.y - 0.4986 * xyz.z,
        -0.9689 * xyz.x + 1.8758 * xyz.y + 0.0415 * xyz.z,
        0.0557 * xyz.x - 0.2040 * xyz.y + 1.0570 * xyz.z,
    );
    var out: vec3<f32>;
    for (var i = 0; i < 3; i = i + 1) {
        let c = lin[i];
        var e: f32;
        if (c > 0.0031308) {
            e = 1.055 * pow(max(c, 0.0), 1.0 / 2.4) - 0.055;
        } else {
            e = 12.92 * c;
        }
        out[i] = clamp(e, 0.0, 1.0);
    }
    return out;
}

// `shading::luminance`.
fn luminance(c: vec3<f32>) -> f32 {
    return 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
}

// `shading::lightness_to_luminance`.
fn lightness_to_luminance(l: f32) -> f32 {
    let f = (l + 16.0) / 116.0;
    if (f * f * f > LAB_EPSILON) {
        return f * f * f;
    }
    return l * 27.0 / 24389.0;
}

// `shading::shift_lightness`: shifts a linear sRGB colour's lightness by `delta_l` in L*. A darker colour keeps its a* and b*. A lighter
// colour mixes the colour made brighter with the colour plus white light, `whiteness` being the share of the second.
fn shift_lightness(c: vec3<f32>, delta_l: f32, whiteness: f32) -> vec3<f32> {
    if (delta_l == 0.0) {
        return c;
    }
    if (delta_l < 0.0) {
        let lab = srgb_to_lab(encode3(c));
        return decode3(lab_to_srgb(vec3<f32>(max(lab.x + delta_l, 0.0), lab.y, lab.z)));
    }
    let y = luminance(c);
    let l = srgb_to_lab(encode3(c)).x;
    let target_y = lightness_to_luminance(min(l + delta_l, 100.0));
    let w = clamp(whiteness, 0.0, 1.0);
    var scale = 1.0;
    var added = target_y;
    if (y > 1e-9) {
        scale = target_y / y;
        added = target_y - y;
    }
    return clamp((1.0 - w) * c * scale + w * (c + added), vec3<f32>(0.0), vec3<f32>(1.0));
}

// The sRGB bytes of a linear colour, packed as an output texel: red in the low byte, then green, blue and an opaque alpha.
fn pack_srgb8(c: vec3<f32>) -> u32 {
    let b = vec3<u32>(round(clamp(encode3(c), vec3<f32>(0.0), vec3<f32>(1.0)) * 255.0));
    return b.x | (b.y << 8u) | (b.z << 16u) | (255u << 24u);
}

// ---- small vector helpers ----

// `wrinkles::smoothstep`, written out because WGSL's own is undefined when a >= b.
fn smooth_step(a: f32, b: f32, x: f32) -> f32 {
    let t = clamp((x - a) / (b - a), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// `layout_points::normalize`: a zero vector stays zero.
fn safe_normalize(a: vec3<f32>) -> vec3<f32> {
    let l = length(a);
    if (l > 0.0) {
        return a / l;
    }
    return a;
}

fn distance_to_segment(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let l2 = dot(ab, ab);
    var t = 0.0;
    if (l2 > 0.0) {
        t = clamp(dot(ap, ab) / l2, 0.0, 1.0);
    }
    return length(ap - t * ab);
}

fn gauss(d: f32, sigma: f32) -> f32 {
    return exp(-d * d / (2.0 * sigma * sigma));
}

// ---- `pores::PoreLandmarks`, which the wrinkle and the pore layers share ----

// The landmarks of the head, in millimetres. `present` is 0 on a mesh without them, where the fine detail layers paint nothing.
struct Marks {
    nose_root: vec4<f32>,
    nose_tip: vec4<f32>,
    forehead: vec4<f32>,
    chin: vec4<f32>,
    mouth: array<vec4<f32>, 2>,
    // The outer then the inner corner of the first eye, then of the second.
    eyes: array<vec4<f32>, 4>,
    present: vec4<u32>,
}

@group(0) @binding(9) var<uniform> marks: Marks;

// `PoreLandmarks::t_zone`.
fn marks_t_zone(p: vec3<f32>) -> f32 {
    let nose = gauss(distance_to_segment(p, marks.nose_root.xyz, marks.nose_tip.xyz), 9.0);
    let forehead = gauss(length(p - marks.forehead.xyz), 26.0);
    let chin = gauss(length(p - marks.chin.xyz), 12.0);
    return max(max(nose, forehead), chin);
}

// `PoreLandmarks::origin`: a point between the eyes.
fn marks_origin() -> vec3<f32> {
    return 0.25 * (marks.eyes[0].xyz + marks.eyes[1].xyz + marks.eyes[2].xyz + marks.eyes[3].xyz);
}

// `PoreLandmarks::flow`: the unit direction the fine lines run along at a point.
fn marks_flow(p: vec3<f32>) -> vec3<f32> {
    let left = 0.5 * (marks.eyes[0].xyz + marks.eyes[1].xyz);
    let right = 0.5 * (marks.eyes[2].xyz + marks.eyes[3].xyz);
    let across = safe_normalize(left - right);
    let middle = 0.5 * (left + right);
    let down = safe_normalize(marks.chin.xyz - middle);
    let height = -dot(p - middle, down);
    let forehead = smooth_step(10.0, 20.0, height);
    var near = length(p - right);
    var eye = right;
    if (length(p - left) < length(p - right)) {
        near = length(p - left);
        eye = left;
    }
    let around_eye = (1.0 - smooth_step(22.0, 35.0, near)) * (1.0 - forehead);
    let radial = safe_normalize(p - eye);
    let rest = (1.0 - forehead) * (1.0 - around_eye);
    return safe_normalize(forehead * across + around_eye * radial + rest * down);
}

// `PoreLandmarks::skin`: how much skin texture a point has, from 0 on the lips and the eye openings to 1 elsewhere.
fn marks_skin(p: vec3<f32>) -> f32 {
    let lips = distance_to_segment(p, marks.mouth[0].xyz, marks.mouth[1].xyz);
    let eye = min(distance_to_segment(p, marks.eyes[0].xyz, marks.eyes[1].xyz), distance_to_segment(p, marks.eyes[2].xyz, marks.eyes[3].xyz));
    return smooth_step(6.0, 11.0, lips) * smooth_step(2.5, 7.5, eye);
}
