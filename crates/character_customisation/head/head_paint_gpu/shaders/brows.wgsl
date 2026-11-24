// The eyebrows (`head_paint::brows`). The processor sends each brow's middle line on the skin as 33 points, with the distance along the
// brow, the direction up the skin across it and the half thickness (see `layers/brows.rs`). Per pixel this shader finds the point's
// place along the brow, `s`, and its signed distance up from the middle line, `d`, both in millimetres on the skin. It darkens the skin
// under the brow, lays a soft fill of the brow's colour, and draws single hairs on top.
//
// The hairs. Each family of hairs runs along straight stripes in its own coordinates: across the stripes `psi`, along them `chi`, both
// in millimetres. The inner family grows up the skin, leaning toward the temple. Family A grows toward the temple and up, from roots in
// the lower part of the brow, and family B toward the temple and down, from roots in the upper part, so the hairs of the body cross in
// a herringbone as real brow hairs do. Every stripe is cut into cells along `chi`, and each cell may hold one hair: whether it does, where
// its root lies, its length, its lean, its bend and its tone all come from a whole-number hash of the cell. A hair is kept only when its
// root and its tip lie in the brow, so hairs cross the brow's edge only as far as the stray hairs slider lets them. A hair's coverage
// of a pixel is its width boxed by the pixel's size, so a small texture shows the hairs' average darkness instead of flickering lines.

struct BrowSide {
    // The middle line's points in millimetres, with the distance along the brow in w.
    pts: array<vec4<f32>, 33>,
    // The direction up the skin across the brow, with the half thickness in w.
    ups: array<vec4<f32>, 33>,
    // x: the brow's length in millimetres; y: 1 when this brow is painted.
    info: vec4<f32>,
}

struct BrowParams {
    // x: 1 when the layer paints; y: the amount; z: the hair density; w: the hair length in mm.
    a: vec4<f32>,
    // x: the hair thickness in mm; y: the tangent of the hair angle; z: the share of the brow with upright hairs; w: the messiness.
    b: vec4<f32>,
    // x: how far stray hairs grow past the edge, in mm; y: the lower edge trim; z: the inner end's fullness; w: the fill.
    c: vec4<f32>,
    // x: the shadow in L*; y: the tone variation; z: the share of grey hairs.
    d: vec4<f32>,
    // The hair colour and the grey hair colour, in linear sRGB.
    colour: vec4<f32>,
    grey: vec4<f32>,
    // The point between the brows and the direction from the x < 0 side to the x > 0 side.
    middle: vec4<f32>,
    across: vec4<f32>,
    sides: array<BrowSide, 2>,
}

@group(0) @binding(16) var<uniform> brow_params: BrowParams;

// How many pixels of the 2,048-pixel texture cover one millimetre along the brow (measured on the 57 heads: 8 to 12, median 10).
const BROW_PX_PER_MM_2048: f32 = 10.0;
// The stripes' spacing and the cells' length, in millimetres.
const BROW_STRIPE_MM: f32 = 0.33;
const BROW_CELL_MM: f32 = 2.5;
// How much the inner hairs lean toward the temple, as a slope.
const BROW_INNER_LEAN: f32 = 0.6;

// A 32-bit hash (PCG), exact on every GPU.
fn brow_pcg(x: u32) -> u32 {
    let s = x * 747796405u + 2891336453u;
    let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}

fn brow_cell_hash(layer: u32, stripe: i32, cell: i32) -> u32 {
    return brow_pcg(bitcast<u32>(cell) ^ brow_pcg(bitcast<u32>(stripe) ^ brow_pcg(layer + 0x51ED27u)));
}

// The k-th number from 0 to 1 drawn from a cell's hash.
fn brow_rand(h: u32, k: u32) -> f32 {
    return f32(brow_pcg(h + k * 0x9E3779B9u) >> 8u) / 16777216.0;
}

// The half thickness of brow `k` at a distance `s` along it, 0 past its ends.
fn brow_half(k: u32, s: f32) -> f32 {
    let L = brow_params.sides[k].info.x;
    if (s <= 0.0 || s >= L) {
        return 0.0;
    }
    var lo = 0u;
    var hi = 32u;
    for (var it = 0u; it < 6u; it = it + 1u) {
        let m = (lo + hi) / 2u;
        if (brow_params.sides[k].pts[m].w <= s) {
            lo = m;
        } else {
            hi = m;
        }
        if (hi - lo <= 1u) {
            break;
        }
    }
    let a = brow_params.sides[k].pts[lo].w;
    let b = brow_params.sides[k].pts[hi].w;
    let f = clamp((s - a) / max(b - a, 1e-4), 0.0, 1.0);
    return mix(brow_params.sides[k].ups[lo].w, brow_params.sides[k].ups[hi].w, f);
}

// How much a point at (s, d) lies in the brow for a hair to grow or end there, from 0 to 1: full inside, falling to 0 at the stray hairs'
// reach past the edge, and over the first and last millimetre of the brow.
// A slow wobble of the brow's thickness along it, from 0.8 to 1.2 times, so that its outline is broken and uneven.
// It is a smooth value noise from the 32-bit hash, every 3 mm along the brow, with its own values above and below the middle line.
fn brow_wobble(k: u32, s: f32, d: f32) -> f32 {
    let x = s / 3.0;
    let i = floor(x);
    let f = x - i;
    let seed = 613u + 2u * k + select(0u, 1u, d > 0.0);
    let a = brow_rand(brow_pcg(bitcast<u32>(i32(i)) ^ brow_pcg(seed)), 0u);
    let b = brow_rand(brow_pcg(bitcast<u32>(i32(i) + 1) ^ brow_pcg(seed)), 0u);
    return 0.8 + 0.4 * mix(a, b, f * f * (3.0 - 2.0 * f));
}

fn brow_room(k: u32, s: f32, d: f32, extra: f32) -> f32 {
    let L = brow_params.sides[k].info.x;
    let hw = brow_half(k, clamp(s, 0.01, L - 0.01)) * brow_wobble(k, s, d);
    let spill = brow_params.c.x + extra;
    let edge = 1.0 - smooth_step(hw - 0.25, hw + spill + 0.3, abs(d));
    return edge * edge * smooth_step(-0.6 - extra, 0.6, s) * smooth_step(L + 0.6 + extra, L - 0.6, s);
}

// Coverage of a pixel of size `fp` by a hair of width `w` whose middle lies `v` away. The hair is spread as a Gaussian of the same
// darkness, at least 0.6 pixel wide, so that a hair about one pixel wide does not turn into a dotted line where it crosses the pixel
// grid at a slant, and so that a small texture keeps the hairs' average darkness.
fn brow_box(v: f32, w: f32, fp: f32) -> f32 {
    let sigma = sqrt(w * w / 5.55 + 0.36 * fp * fp);
    return min(w / (sigma * 2.5066) * exp(-v * v / (2.0 * sigma * sigma)), 1.0);
}

struct BrowHit {
    s: f32,
    d: f32,
    // The 3D distance to the middle line.
    dist: f32,
}

// The direction that halves the bend of brow `k`'s middle line at point `i`: the mean of the directions of the segments on either side,
// or the end segment's direction at an end.
fn brow_bisector(k: u32, i: u32) -> vec3<f32> {
    let b = brow_params.sides[k].pts[i].xyz;
    var t = vec3<f32>(0.0);
    if (i > 0u) {
        let a = b - brow_params.sides[k].pts[i - 1u].xyz;
        t = t + a / max(length(a), 1e-6);
    }
    if (i < 32u) {
        let c = brow_params.sides[k].pts[i + 1u].xyz - b;
        t = t + c / max(length(c), 1e-6);
    }
    return t / max(length(t), 1e-6);
}

// A point's place along brow `k` and its distance up from the middle line. Each segment of the line holds the skin between the planes
// that halve the bends at its two ends, and the place along the brow goes evenly from one plane to the other. So the place changes
// smoothly from segment to segment. The nearest point of each segment, which the place came from before, gives every pixel past a
// bend's outer side the same place, and a pixel on its inner side a jump. The hairs, cut from cells of that place, then fanned out
// straight from the line or were cut off. A pixel that no segment holds, far inside a sharp bend, still takes its nearest point.
fn brow_locate(k: u32, p: vec3<f32>) -> BrowHit {
    var slab = BrowHit(0.0, 0.0, 1e9);
    var m0 = brow_bisector(k, 0u);
    for (var i = 0u; i < 32u; i = i + 1u) {
        let a = brow_params.sides[k].pts[i];
        let b = brow_params.sides[k].pts[i + 1u];
        let m1 = brow_bisector(k, i + 1u);
        let fa = dot(p - a.xyz, m0);
        let fb = dot(p - b.xyz, m1);
        var c = vec3<f32>(0.0);
        var s = 0.0;
        var up = vec3<f32>(0.0);
        var held = false;
        if (i == 0u && fa < 0.0) {
            c = a.xyz + fa * m0;
            s = a.w + fa;
            up = brow_params.sides[k].ups[0].xyz;
            held = true;
        } else if (i == 31u && fb > 0.0) {
            c = b.xyz + fb * m1;
            s = b.w + fb;
            up = brow_params.sides[k].ups[32].xyz;
            held = true;
        } else if (fa >= 0.0 && fb <= 0.0) {
            let t = fa / max(fa - fb, 1e-6);
            c = mix(a.xyz, b.xyz, t);
            s = mix(a.w, b.w, t);
            up = normalize(mix(brow_params.sides[k].ups[i].xyz, brow_params.sides[k].ups[i + 1u].xyz, t));
            held = true;
        }
        if (held) {
            let dist = length(p - c);
            if (dist < slab.dist) {
                // The distance from the line is measured straight across the segment, and only its side comes from the direction up the
                // skin. Where the line lies under the brow ridge, that direction points nearly forward, and a pixel on the forehead
                // above would otherwise seem close to the line.
                let ab = b.xyz - a.xyz;
                let e = p - c - dot(p - c, ab) / max(dot(ab, ab), 1e-6) * ab;
                slab = BrowHit(s, select(-1.0, 1.0, dot(p - c, up) >= 0.0) * length(e), dist);
            }
        }
        m0 = m1;
    }
    if (slab.dist < 1e9) {
        return slab;
    }
    var best = BrowHit(0.0, 0.0, 1e9);
    for (var i = 0u; i < 32u; i = i + 1u) {
        let a = brow_params.sides[k].pts[i];
        let b = brow_params.sides[k].pts[i + 1u];
        let ab = b.xyz - a.xyz;
        let l2 = max(dot(ab, ab), 1e-6);
        let raw = dot(p - a.xyz, ab) / l2;
        let t = clamp(raw, 0.0, 1.0);
        let c = a.xyz + t * ab;
        let dist = length(p - c);
        if (dist < best.dist) {
            var s = mix(a.w, b.w, t);
            // Past the ends, the distance along the brow goes on along the end segment.
            if (i == 0u && raw < 0.0) {
                s = a.w + raw * sqrt(l2);
            }
            if (i == 31u && raw > 1.0) {
                s = a.w + raw * sqrt(l2);
            }
            let up = normalize(mix(brow_params.sides[k].ups[i].xyz, brow_params.sides[k].ups[i + 1u].xyz, t));
            best = BrowHit(s, dot(p - c, up), dist);
        }
    }
    return best;
}

// The hairs of one layer over a pixel, laid onto the colour `col`. Family 0 is the inner family, 1 is family A and 2 is family B.
fn brow_layer(col_in: vec3<f32>, k: u32, family: u32, layer: u32, s: f32, d: f32, fp: f32) -> vec3<f32> {
    var col = col_in;
    let pa = brow_params;
    let L = pa.sides[k].info.x;
    var slope = pa.b.y;
    if (family == 2u) {
        slope = -0.8 * pa.b.y;
    }
    // The stripes' direction: along (ax, ay) in (s, d), across n = (ay, -ax).
    var along = normalize(vec2<f32>(1.0, slope));
    if (family == 0u) {
        along = normalize(vec2<f32>(BROW_INNER_LEAN, 1.0));
    }
    let n = vec2<f32>(along.y, -along.x);
    let psi = dot(vec2<f32>(s, d), n);
    let chi = dot(vec2<f32>(s, d), along);
    let h0 = brow_pcg(layer * 7919u + k * 104729u + 17u);
    let phase_psi = brow_rand(h0, 0u);
    let phase_chi = brow_rand(h0, 1u);
    // The upright inner hairs are shorter, as on real brows, so that they stay inside a thin brow.
    let length_mm = select(pa.a.w, 0.55 * pa.a.w, family == 0u);
    let cells = i32(ceil(1.4 * length_mm / BROW_CELL_MM)) + 1;
    let i0 = i32(floor(psi / BROW_STRIPE_MM + phase_psi));
    let c0 = i32(floor(chi / BROW_CELL_MM + phase_chi));
    let zone = max(pa.b.z * L, 0.5);
    let messy = pa.b.w;
    for (var j = -2; j <= 2; j = j + 1) {
        let stripe = i0 + j;
        for (var m = 0; m < cells; m = m + 1) {
            let cell = c0 - m;
            let h = brow_cell_hash(layer + 8u * k, stripe, cell);
            let lh = length_mm * (0.6 + 0.8 * brow_rand(h, 3u));
            let root_chi = (f32(cell) - phase_chi + brow_rand(h, 2u)) * BROW_CELL_MM;
            let u = chi - root_chi;
            if (u < 0.0 || u > lh) {
                continue;
            }
            let root_psi = (f32(stripe) - phase_psi + 0.5 + 0.8 * (brow_rand(h, 1u) - 0.5)) * BROW_STRIPE_MM;
            let lean = (messy + select(0.0, 0.15, family == 0u)) * min(0.25, 0.6 / lh) * (2.0 * brow_rand(h, 4u) - 1.0);
            let bend = messy * (0.4 / (lh * lh)) * (2.0 * brow_rand(h, 5u) - 1.0);
            let v = psi - root_psi - lean * u - bend * u * u;
            let w = pa.b.x * (0.7 + 0.7 * brow_rand(h, 8u)) * (1.0 - 0.65 * u / lh);
            if (abs(v) > 0.5 * w + 2.5 * fp) {
                continue;
            }
            // The root and the tip, in (s, d).
            let root = vec2<f32>(s, d) - u * along - v * n;
            let tip = root + lh * along;
            let hw_root = max(brow_half(k, root.x), 0.15);
            let e = root.y / hw_root;
            // Hairs near the brow's edges are shorter.
            if (u > lh * (1.0 - 0.45 * clamp(abs(e), 0.0, 1.0))) {
                continue;
            }
            var weight = brow_room(k, root.x, root.y, 0.0) * sqrt(brow_room(k, tip.x, tip.y, 0.8));
            // Where each family grows.
            let inner = 1.0 - smooth_step(0.7 * zone, 1.3 * zone, root.x);
            if (family == 0u) {
                weight = weight * inner * 0.75 * smooth_step(-0.5, 3.0, root.x);
            } else {
                weight = weight * (1.0 - inner);
                if (family == 1u) {
                    weight = weight * smooth_step(0.55, -0.35, e);
                } else {
                    weight = weight * smooth_step(-0.55, 0.35, e);
                }
            }
            // A sparse inner end unless it is full, and a fuller middle.
            let t = 2.0 * root.x / L - 1.0;
            weight = weight * mix(smooth_step(-0.3, 0.25 * L, root.x), 1.0, pa.c.z) * (0.75 + 0.25 * (1.0 - t * t));
            if (brow_rand(h, 0u) >= pa.a.z * weight) {
                continue;
            }
            var cover = brow_box(v, w, fp) * smooth_step(0.0, 0.35, u) * (1.0 - 0.7 * smooth_step(0.55 * lh, lh, u));
            // Plucked lower edge.
            let hw_here = brow_half(k, s);
            cover = cover * mix(1.0, smooth_step(-hw_here - 0.7, -hw_here + 0.1, d), pa.c.y);
            var hair = pa.colour.xyz * exp(pa.d.y * 1.6 * (brow_rand(h, 6u) - 0.5));
            if (brow_rand(h, 7u) < pa.d.z) {
                hair = pa.grey.xyz * exp(0.3 * (brow_rand(h, 6u) - 0.5));
            }
            hair = hair * (1.0 + 0.35 * u / lh);
            let strength = (0.6 + 0.4 * brow_rand(h, 9u)) * select(1.0, 0.65, family == 0u);
            col = mix(col, clamp(hair, vec3<f32>(0.0), vec3<f32>(1.0)), clamp(cover * pa.a.y, 0.0, 1.0) * strength);
        }
    }
    return col;
}

fn brows_apply(c_in: vec3<f32>, pt: Point) -> vec3<f32> {
    let pa = brow_params;
    if (pa.a.x == 0.0) {
        return c_in;
    }
    let p = pt.position_mm;
    var k = 1u;
    if (dot(p - pa.middle.xyz, pa.across.xyz) < 0.0) {
        k = 0u;
    }
    if (pa.sides[k].info.y == 0.0 || length(p - pa.sides[k].pts[16].xyz) > 40.0) {
        return c_in;
    }
    let hit = brow_locate(k, p);
    if (hit.dist > 14.0) {
        return c_in;
    }
    let s = hit.s;
    let d = hit.d;
    let L = pa.sides[k].info.x;
    let hw = brow_half(k, s);
    var c = c_in;
    let ends = smooth_step(-2.5, 1.0, s) * smooth_step(L + 2.5, L - 1.0, s);
    // The hairs and the fill are matte, so the gloss under the brow falls to a fifth of the skin's.
    if (maps_on()) {
        let cover = clamp(pa.a.y * ends * smooth_step(hw + 1.0, max(hw - 0.5, 0.0), abs(d)), 0.0, 1.0);
        maps_gloss = maps_gloss * (1.0 - (1.0 - maps_params.gloss2.z) * cover);
    }

    // The shadow: the skin under and around the hairs, a little below the middle line, darker, as in a shadow.
    let dark = pa.d.x * pa.a.y * smooth_step(hw + 2.2, max(hw - 0.8, 0.0), abs(d + 0.4)) * ends;
    if (dark > 0.0) {
        let l = srgb_to_lab(encode3(c)).x;
        let y0 = lightness_to_luminance(l);
        if (y0 > 1e-6) {
            c = c * (lightness_to_luminance(clamp(l - dark, 0.0, 100.0)) / y0);
        }
    }
    // The fill: a lighter, greyer version of the hair colour on the skin between the hairs, as hand-painted brows have under their
    // strokes. Its lower edge is firm and its upper edge soft, its inner end lighter, and it is broken into faint streaks that run with
    // the body's hairs. At full fill the core is opaque, so the brow keeps its darkness at face distance, where single hairs blur away.
    let flow = normalize(vec2<f32>(1.0, 0.1 * pa.b.y));
    let streak = gradient_noise(vec3<f32>(3.5 * dot(vec2<f32>(s, d), vec2<f32>(flow.y, -flow.x)), 0.35 * dot(vec2<f32>(s, d), flow), f32(k) * 7.0), vec2<u32>(911u, 5u));
    // The fill keeps to the core, so that the brow's edges are made of hairs with skin between them.
    let hwf = hw * brow_wobble(k, s, d);
    let lower = smooth_step(-hwf - 0.2, -0.8 * hwf + 0.4, d);
    let upper = smooth_step(0.95 * hwf + 0.4, 0.15 * hwf, d) * smooth_step(L + 0.3, L - 2.5, s);
    let front = mix(mix(0.5, 1.0, pa.c.z), 1.0, smooth_step(0.0, 0.3 * L, s));
    let fill = pa.c.w * pa.a.y * lower * upper * front * smooth_step(0.05, 0.8, hw) * smooth_step(-0.3, 2.0, s) * clamp(0.95 + 0.45 * streak, 0.6, 1.0);
    let lighter = pa.colour.xyz * 1.25 + vec3<f32>(0.002);
    let fill_colour = mix(lighter, vec3<f32>(luminance(lighter)), 0.2);
    c = mix(c, fill_colour, clamp(fill, 0.0, 1.0));

    // The hairs, two layers per family.
    let mm_per_px = 2048.0 / (BROW_PX_PER_MM_2048 * f32(max(frame.resolution, 1u)));
    if (abs(d) > hw + pa.c.x + pa.a.w + 1.0) {
        return c;
    }
    let zone = max(pa.b.z * L, 0.5);
    if (s < 1.3 * zone + 1.4 * pa.a.w) {
        c = brow_layer(c, k, 0u, 0u, s, d, mm_per_px);
        c = brow_layer(c, k, 0u, 1u, s, d, mm_per_px);
    }
    if (s > 0.7 * zone - 1.4 * pa.a.w) {
        c = brow_layer(c, k, 1u, 2u, s, d, mm_per_px);
        c = brow_layer(c, k, 2u, 3u, s, d, mm_per_px);
        c = brow_layer(c, k, 1u, 4u, s, d, mm_per_px);
        c = brow_layer(c, k, 2u, 5u, s, d, mm_per_px);
    }
    return c;
}
