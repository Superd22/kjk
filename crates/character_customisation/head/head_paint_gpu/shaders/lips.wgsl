// The lips (`head_paint::lips`), painted after the shading and the sheen and before the fine detail. The processor lays out the lip curves in the
// mouth's frame, every 0.25 mm along `u`, and a map of the head's frontmost surface over the mouth (see `layers/lips.rs`). Each pixel is
// placed in that frame, and its colour and lightness are computed from its distances to the borders and to the opening.

struct LipsParams {
    origin: vec4<f32>,
    across: vec4<f32>,
    up: vec4<f32>,
    forward: vec4<f32>,
    front: vec4<f32>,
    misc: vec4<f32>,
    natural: vec4<f32>,
    stick: vec4<f32>,
    shades: vec4<f32>,
    corners: vec4<f32>,
    halves: vec4<f32>,
    extra: vec4<f32>,
}

@group(0) @binding(14) var<uniform> lips_params: LipsParams;
@group(0) @binding(15) var<storage, read> lips_data: array<f32>;

// How much the lips cover the skin at the pixel, from 0 to 1: the lip colour's share, which is 1 inside the mouth. `lips_apply` sets it,
// and the paint pass sets it to 0 first and then keeps the pores off the lips by it.
var<private> lips_cover: f32;

// How many samples on each side of a pixel's `u` the border distance looks at: 4 mm.
const LIPS_WINDOW: i32 = 16;
const LIPS_SEED: vec2<u32> = vec2<u32>(0x2F6B1D43u, 0x51ED2701u);

fn lips_count() -> i32 {
    return i32(lips_params.forward.w);
}

// A curve's `v` at sample i, clamped to the curve's ends: 0 is the upper border, 1 the lower border, 2 the opening.
fn lips_sample(curve: i32, i: i32) -> f32 {
    let n = lips_count();
    return lips_data[curve * n + clamp(i, 0, n - 1)];
}

fn lips_curve(curve: i32, u: f32) -> f32 {
    let t = (u - lips_params.across.w) / lips_params.up.w;
    let i = i32(floor(t));
    return mix(lips_sample(curve, i), lips_sample(curve, i + 1), t - f32(i));
}

// The distance from (u, v) to a curve, over the samples within 4 mm along `u`, positive on the side where `below` says the lip is.
fn lips_distance(curve: i32, u: f32, v: f32, below: bool) -> f32 {
    let t = (u - lips_params.across.w) / lips_params.up.w;
    let i0 = i32(floor(t));
    let step = lips_params.up.w;
    var best = 1e9;
    for (var i = i0 - LIPS_WINDOW; i <= i0 + LIPS_WINDOW; i = i + 1) {
        let a = vec2<f32>(lips_params.across.w + f32(i) * step, lips_sample(curve, i));
        let b = vec2<f32>(a.x + step, lips_sample(curve, i + 1));
        let ab = b - a;
        let ap = vec2<f32>(u, v) - a;
        let s = clamp(dot(ap, ab) / dot(ab, ab), 0.0, 1.0);
        best = min(best, length(ap - s * ab));
    }
    let side = v - lips_curve(curve, u);
    if ((side < 0.0) == below) {
        return best;
    }
    return -best;
}

// The frontmost surface's `w` at (u, v), or NaN where the map has none.
fn lips_front(u: f32, v: f32) -> f32 {
    let x = (u - lips_params.front.x) / lips_params.front.z;
    let y = (v - lips_params.front.y) / lips_params.front.z;
    let nu = i32(lips_params.front.w);
    let nv = i32(lips_params.misc.x);
    let ix = i32(floor(x));
    let iy = i32(floor(y));
    if (ix < 0 || iy < 0 || ix >= nu - 1 || iy >= nv - 1) {
        return 0.0;
    }
    let base = 3 * lips_count();
    let fx = x - f32(ix);
    let fy = y - f32(iy);
    var sum = 0.0;
    var weight = 0.0;
    for (var k = 0; k < 4; k = k + 1) {
        let dx = k & 1;
        let dy = k >> 1u;
        let w = lips_data[base + (iy + dy) * nu + ix + dx];
        let g = select(1.0 - fx, fx, dx == 1) * select(1.0 - fy, fy, dy == 1);
        if (w == w) {
            sum = sum + g * w;
            weight = weight + g;
        }
    }
    if (weight <= 0.0) {
        return 0.0;
    }
    return sum / weight;
}

// The fine vertical lines across a lip: dark strokes about 1 mm apart, each with its own place, strength and length from a hash.
fn lips_lines(u: f32, f: f32, lower: bool) -> f32 {
    let spacing = 1.05;
    let t = (u * (1.0 - 0.18 * f)) / spacing;
    let k = i32(floor(t));
    let lip = select(1, 0, lower);
    var sum = 0.0;
    for (var j = -1; j <= 1; j = j + 1) {
        let h = hash3(k + j, lip, 7, LIPS_SEED);
        if (hash_unit(h, 0u) < 0.2) {
            continue;
        }
        let centre = f32(k + j) + 0.5 + 0.7 * (hash_unit(h, 16u) - 0.5);
        let strength = 0.4 + 0.6 * hash_unit(h, 32u);
        let start = 0.05 + 0.35 * hash_unit(h, 48u);
        let end = 0.55 + 0.4 * hash_unit(hash3(k + j, lip, 11, LIPS_SEED), 0u);
        let across = gauss((t - centre) * spacing, 0.16);
        let along = smooth_step(start, start + 0.12, f) * (1.0 - smooth_step(end - 0.12, end, f));
        sum = sum + strength * across * along;
    }
    return min(sum, 1.0);
}

fn lips_apply(c: vec3<f32>, pt: Point) -> vec3<f32> {
    if (lips_params.misc.y == 0.0) {
        return c;
    }
    let r = pt.position_mm - lips_params.origin.xyz;
    let x = dot(r, lips_params.across.xyz);
    let z = dot(r, lips_params.forward.xyz) + lips_params.origin.w;
    if (z <= 0.0) {
        return c;
    }
    let u = lips_params.origin.w * atan2(x, z);
    let v = dot(r, lips_params.up.xyz);
    let w = sqrt(x * x + z * z) - lips_params.origin.w;
    let first = lips_params.across.w;
    let last = first + f32(lips_count() - 1) * lips_params.up.w;
    if (u < first || u > last) {
        return c;
    }
    let v_up = lips_curve(0, u);
    let v_lo = lips_curve(1, u);
    let v_op = lips_curve(2, u);
    let soft = lips_params.misc.z;
    let margin = 6.0 + soft;
    if (v > v_up + margin || v < v_lo - margin) {
        return c;
    }
    let d_up = lips_distance(0, u, v, true);
    let d_lo = lips_distance(1, u, v, false);
    let d = min(d_up, d_lo);
    let c0 = lips_params.corners.xy;
    let c1 = lips_params.corners.zw;
    let past = max(c0.x - u, u - c1.x);

    // Inside the mouth: points behind the frontmost surface, between the borders, take the lip colour wherever they are.
    let depth = lips_front(u, v) - w;
    let between = smooth_step(0.5, 1.5, min(v_up - v, v - v_lo)) * (1.0 - smooth_step(-1.0, 1.0, past));
    let inside = smooth_step(1.0, 3.0, depth) * between;
    // The border softens toward the corners, to twice its width at the corner, where the lips fade into the skin.
    let half_w = select(lips_params.halves.y, lips_params.halves.x, u < 0.0);
    let soft_here = soft * (1.0 + smooth_step(0.55, 1.0, abs(u) / max(half_w, 1.0)));
    let alpha = max(smooth_step(-0.5 * soft_here, 0.5 * soft_here, d), inside);
    lips_cover = alpha;
    if (alpha <= 0.0 && d < -(soft + 3.0) && past > 4.0) {
        return c;
    }

    // Which lip, and how far across it: 0 at the opening and 1 at the border.
    let m = v - v_op;
    let is_up = smooth_step(-0.4, 0.4, m);
    let f_up = clamp(m / max(v_up - v_op, 0.5), 0.0, 1.0);
    let f_lo = clamp(-m / max(v_op - v_lo, 0.5), 0.0, 1.0);
    let f = mix(f_lo, f_up, is_up);
    let half = select(lips_params.halves.y, lips_params.halves.x, u < 0.0);
    let share = clamp(abs(u) / max(half, 1.0), 0.0, 1.0);
    let front_only = 1.0 - inside;
    // How far toward a corner, where both lips turn away from the light.
    let to_corner = smooth_step(0.4, 0.97, share);

    // The colour: the shaded skin colour turned by the natural lip shift, then the lipstick over it. The natural shift fades over the outer
    // 40% of each lip, to 50% at the upper border and 85% at the lower one, so a natural lip melts into the skin and only a lipstick has a
    // painted edge. The lipstick keeps half of the shading's change of lightness, so it still shows the lips' form.
    let lab = srgb_to_lab(encode3(c));
    let fade = 1.0 - mix(0.15, 0.5, is_up) * smooth_step(0.6, 1.0, f) * front_only;
    let natural = decode3(lab_to_srgb(lab + fade * lips_params.natural.xyz));
    let shade = lab.x - srgb_to_lab(encode3(frame.base.xyz)).x;
    let stick = decode3(lab_to_srgb(lips_params.stick.xyz + vec3<f32>(0.5 * shade, 0.0, 0.0)));
    let lip = mix(natural, stick, lips_params.misc.w);
    var out = mix(c, lip, alpha);

    let lip_shade = lips_params.natural.w;
    var dark = -max(lip_shade, 0.0) * alpha * is_up - max(-lip_shade, 0.0) * alpha * (1.0 - is_up);
    // The share of `dark` that is a crease or a ridge of the lips and not a broad shade: the mouth line, the liner, the light rim of the
    // border, the shadow under the lower lip and the fine lines. With the lips' roll it is the lips' relief in the maps.
    var relief = 0.0;
    // The mouth line along the opening, fading out 4 mm past each corner, and a shadow round each corner.
    let taper = 1.0 - smooth_step(-0.5, 1.0, past);
    // The line widens a little toward the corners and stops at them, with a small soft shadow in each corner.
    let corner = min(length(vec2<f32>(u, v) - c0), length(vec2<f32>(u, v) - c1));
    // It is a soft shadow band about 1.5 mm tall with a darker core, not a drawn line.
    let line_w = 0.35 + 0.25 * share * share;
    let band = 0.55 * exp(-(m * m) / (line_w * line_w)) + 0.45 * exp(-(m * m) / (4.0 * line_w * line_w));
    let mouth_line = lips_params.shades.x * (1.0 + 0.15 * share) * max(band * taper, 0.35 * gauss(corner, 1.3));
    dark = dark - mouth_line;
    relief = relief - mouth_line;
    // Deeper inside the mouth the lip darkens, to 35 L* darker 10 mm in, with the mouth line's darkness on top.
    dark = dark - (35.0 + 0.4 * lips_params.shades.x) * smooth_step(2.0, 10.0, depth) * between;
    // The liner: a thin dark band just inside the border.
    let liner_w = max(0.3, 0.4 * soft);
    let liner = lips_params.shades.y * gauss(d - liner_w, liner_w) * front_only;
    dark = dark - liner;
    relief = relief - liner;
    // The light rim just outside the border, strongest along the upper lip.
    let rim = gauss(d + 0.5 * soft + 0.5, 0.45) * select(0.5, 1.0, d_up < d_lo) * taper;
    let rim_light = lips_params.shades.z * rim * front_only;
    dark = dark + rim_light;
    relief = relief + rim_light;
    // The soft shadow under the lower lip.
    let under = lips_params.extra.y * gauss(d_lo + 1.5 + 0.5 * soft, 1.2) * select(0.0, 1.0, d_lo < 0.5) * (1.0 - smooth_step(0.5, 1.0, share)) * front_only;
    // It is a shadow cast by the lower lip with a hard edge where the lip ends, so it stays out of the relief.
    dark = dark - under;
    // The fine lines, stronger on the lower lip, and deeper on a wet lip.
    let gloss = lips_params.extra.x;
    let lines = select(0.6 * lips_lines(u, f_up, false), lips_lines(u, f_lo, true), m < 0.0);
    let fine = lips_params.shades.w * (1.0 + 0.6 * gloss) * lines * alpha * front_only * (1.0 - smooth_step(0.75, 1.0, share));
    dark = dark - fine;
    relief = relief - fine;

    // The form. Each lip is a roll lit from the front and above. The lower lip is lightest a little below its middle and darker toward
    // the opening, its border and the corners. The upper lip faces down, so the darker upper lip slider darkens it as a whole, and a soft
    // lighter roll runs along its upper half, with a darker band just above the opening. `hl` is the lower lip's light, from its darkest
    // edge to its lightest middle.
    let hl = lips_params.stick.w;
    let lower_form = hl * (1.3 * gauss(f_lo - 0.48, 0.22) * (1.0 - 0.6 * to_corner) - 0.3 - 0.5 * (1.0 - smooth_step(0.0, 0.3, f_lo)) - 0.3 * smooth_step(0.75, 1.0, f_lo));
    let upper_form = hl * (0.45 * gauss(f_up - 0.62, 0.18) * (1.0 - to_corner) - 0.15 - 0.45 * (1.0 - smooth_step(0.0, 0.4, f_up)));
    var form = (mix(lower_form, upper_form, is_up) - (0.3 * hl + 2.0) * to_corner) * alpha * front_only;
    // While the light comes from the maps, the creases and the roll become the lips' relief, and the colour keeps only a share of their
    // painted light and shade. The lips take their own gloss in the mask, more on a wet lip and less toward the corners.
    let maps = maps_on();
    if (maps) {
        maps_height = maps_height + maps_params.relief.x * maps_params.relief.z * (relief + form);
        dark = dark - relief + maps_keep(relief);
        form = maps_keep(form);
        maps_gloss = mix(maps_gloss, clamp(maps_params.gloss.w * (0.5 + gloss), 0.0, 1.0), clamp(alpha * front_only * (1.0 - 0.6 * to_corner), 0.0, 1.0));
    }
    dark = dark + min(form, 0.0);
    out = shift_lightness(out, dark, 0.0);
    if (form > 0.0) {
        out = shift_lightness(out, form, 0.15);
    }

    // The gloss: whitish light where the lower lip's fullest part faces the viewer, mostly on either side of the middle, broken up by the
    // lip lines into thin vertical streaks and small bright specks, and a weaker light on the upper lip's roll. At 0 the lips are matte.
    if (gloss > 0.0) {
        let streak_lo = lips_lines(u + 0.52, f_lo, true);
        let streak_up = lips_lines(u + 0.52, f_up, false);
        let cell = vec2<f32>(u, v) / 0.6;
        let ci = vec2<i32>(floor(cell));
        var speck = 0.0;
        for (var k = 0; k < 4; k = k + 1) {
            let cc = ci + vec2<i32>(k & 1, k >> 1u);
            let h = hash3(cc.x, cc.y, 23, LIPS_SEED);
            if (hash_unit(h, 0u) < 0.35) {
                let at = vec2<f32>(cc) + vec2<f32>(hash_unit(h, 16u), hash_unit(h, 32u)) - 0.5;
                speck = max(speck, (0.5 + 0.5 * hash_unit(h, 48u)) * gauss(length(cell - at) * 0.6, 0.17));
            }
        }
        let sides = gauss(share - 0.33, 0.2) + 0.45 * gauss(share, 0.12);
        let zone_lo = gauss(f_lo - 0.45, 0.2) * sides * (1.0 - is_up);
        let zone_up = 0.45 * gauss(f_up - 0.62, 0.1) * gauss(share - 0.28, 0.2) * is_up;
        // The streaks and specks grow faster than the broad light, so a little gloss is a soft sheen and only a wet lip sparkles.
        let sharp = gloss * gloss;
        let wet = 40.0 * gloss * (zone_lo * (0.45 + 0.9 * sharp * streak_lo + 0.45 * sharp * speck) + zone_up * (0.45 + 0.6 * sharp * streak_up + 0.3 * sharp * speck));
        let spec = wet * alpha * front_only * (1.0 - to_corner) * select(1.0, maps_params.kept.y, maps);
        if (spec > 0.0) {
            out = shift_lightness(out, spec, 0.4 + 0.3 * gloss);
        }
    }
    return out;
}
