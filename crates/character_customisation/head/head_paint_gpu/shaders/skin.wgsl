// The skin's colour under light and its colour zones (`skin.rs`). `skin_base` gives the colour under every layer, with the zones' melanin
// and blood. `skin_shift` changes a colour's lightness as `shift_lightness` does, and moves its colour along the skin's own colours: the
// shade dims the light, turns toward the skin's reddest hue, the light gets more colourful, and only strong light goes toward white. With
// every slider at zero both give exactly the colours of before.

const SKIN_TABLE_U: u32 = 9u;
const SKIN_TABLE_B: u32 = 13u;
const SKIN_BLOOD_LOW: f32 = -0.04;
const SKIN_BLOOD_HIGH: f32 = 0.08;
const SKIN_ZONE_BLOOD: f32 = 0.03;
const SKIN_ZONE_MELANIN_LOG2: f32 = 0.3;

struct SkinParams {
    on: vec4<u32>,
    light: vec4<f32>,
    zones_a: vec4<f32>,
    zones_b: vec4<f32>,
    zones_c: vec4<f32>,
    points: array<vec4<f32>, 18>,
    table: array<vec4<f32>, 117>,
}

@group(0) @binding(20) var<uniform> skin_params: SkinParams;

// The colour under the shading at the pixel being painted, which `skin_shift` takes its tint from. The paint pass sets it.
var<private> skin_tint: vec3<f32>;

fn skin_point(k: u32) -> vec3<f32> {
    return skin_params.points[k].xyz;
}

fn skin_table(i: u32, j: u32) -> vec3<f32> {
    return skin_params.table[j * SKIN_TABLE_U + i].xyz;
}

// How much a point lies in a ring round a segment: nothing within `inner` mm, all of it at `inner + 4` mm, fading outward over `sigma`.
fn skin_ring(d: f32, inner: f32, sigma: f32) -> f32 {
    return smooth_step(inner, inner + 4.0, d) * gauss(max(d - inner - 4.0, 0.0), sigma);
}

// The colour under every layer at a point: the base tone, with the zones' melanin and blood while they act.
fn skin_base(pt: Point) -> vec3<f32> {
    if (skin_params.on.y == 0u) {
        return frame.base.xyz;
    }
    let p = pt.position_mm;
    let s = skin_params.zones_c.y;
    let cheeks = max(gauss(length(p - skin_point(0u)), 13.0 * s), gauss(length(p - skin_point(1u)), 13.0 * s));
    let ears = max(gauss(length(p - skin_point(2u)), 14.0 * s), gauss(length(p - skin_point(3u)), 14.0 * s));
    let chin = gauss(length(p - skin_point(4u)), 10.0 * s);
    let forehead = gauss(length(p - (skin_point(5u) + 15.0 * skin_point(16u))), 22.0 * s);
    let nose = max(gauss(distance_to_segment(p, skin_point(6u), skin_point(7u)), 6.0 * s), gauss(length(p - skin_point(7u)), 9.0 * s));
    // The mouth's ring lies outside an ellipse through the two corners, 9 mm high above and below the mouth line, so that it stays off
    // the lips, whose natural colour the lip layer takes from the skin under them.
    let corners = skin_point(9u) - skin_point(8u);
    let half = max(0.5 * length(corners), 1.0);
    let rm = p - 0.5 * (skin_point(8u) + skin_point(9u));
    let ellipse = 9.0 * (length(vec2<f32>(dot(rm, corners / (2.0 * half)) / half, dot(rm, skin_point(16u)) / 9.0)) - 1.0);
    let mouth = skin_ring(ellipse, 0.0, 6.0 * s);
    let eye = min(distance_to_segment(p, skin_point(10u), skin_point(11u)), distance_to_segment(p, skin_point(12u), skin_point(13u)));
    let eyes = skin_ring(eye, 0.0, 8.0 * s);
    let r = p - skin_point(14u);
    let x = dot(r, skin_point(15u));
    let y = dot(r, skin_point(16u));
    let middle = gauss(x, 16.0 * s) * smooth_step(-135.0, -110.0, y) * (1.0 - smooth_step(45.0, 70.0, y));
    let a = skin_params.zones_a;
    let b = skin_params.zones_b;
    let m = skin_params.zones_c.x;
    let blood = SKIN_ZONE_BLOOD * (a.x * cheeks + a.y * nose + a.z * ears + a.w * mouth + b.x * chin - b.y * forehead + m * middle / 3.0);
    let u = SKIN_ZONE_MELANIN_LOG2 * (b.z * eyes + b.w * mouth + 0.5 * b.y * forehead - m * middle);
    let fu = clamp((u + 1.0) * 0.5, 0.0, 1.0) * f32(SKIN_TABLE_U - 1u);
    let fb = clamp((blood - SKIN_BLOOD_LOW) / (SKIN_BLOOD_HIGH - SKIN_BLOOD_LOW), 0.0, 1.0) * f32(SKIN_TABLE_B - 1u);
    let i0 = min(u32(floor(fu)), SKIN_TABLE_U - 2u);
    let j0 = min(u32(floor(fb)), SKIN_TABLE_B - 2u);
    let tu = fu - f32(i0);
    let tb = fb - f32(j0);
    let low = mix(skin_table(i0, j0), skin_table(i0 + 1u, j0), tu);
    let high = mix(skin_table(i0, j0 + 1u), skin_table(i0 + 1u, j0 + 1u), tu);
    return mix(low, high, tb);
}

// The painted light from in front of the face, in L*: the head is taken as a cylinder about the face's up direction, whose radius is 0.78
// times the distance between the eyes' outer corners. Where the skin faces the front it is up to 12 L* lighter at 1, where it is turned
// halfway it is unchanged, and the far sides and the back of the head are up to 24 L* darker.
fn skin_front(pt: Point) -> f32 {
    let k = skin_params.zones_c.z;
    if (k == 0.0) {
        return 0.0;
    }
    let r = pt.position_mm - skin_point(14u);
    let x = dot(r, skin_point(15u));
    let z = dot(r, skin_point(17u));
    let radius = max(0.78 * length(skin_point(11u) - skin_point(13u)), 1.0);
    let facing = sqrt(max(1.0 - (x / radius) * (x / radius), 0.0));
    let front = clamp((facing - 0.75) / 0.25, -2.0, 1.0);
    return k * 12.0 * mix(-2.0, front, smooth_step(-90.0, -40.0, z));
}

// The skin's own colour direction from its tint: 1 on its strongest channel, 0 on its weakest, and 0 everywhere on a grey skin.
fn skin_direction() -> vec3<f32> {
    let top = max(max(skin_tint.x, skin_tint.y), skin_tint.z);
    if (top <= 1e-6) {
        return vec3<f32>(0.0);
    }
    let t = skin_tint / top;
    let low = min(min(t.x, t.y), t.z);
    if (low >= 0.9999) {
        return vec3<f32>(0.0);
    }
    return (t - vec3<f32>(low)) / (1.0 - low);
}

// Scales a colour so that its luminance is `y`.
fn skin_keep_luminance(c: vec3<f32>, y: f32) -> vec3<f32> {
    let now = luminance(c);
    if (now <= 1e-9) {
        return c;
    }
    return c * (y / now);
}

// `shift_lightness`, moved along the skin's own colours by the light sliders. The lightness of the result is the same.
fn skin_shift(c: vec3<f32>, delta_l: f32, whiteness: f32) -> vec3<f32> {
    if (skin_params.on.x == 0u || delta_l == 0.0) {
        return shift_lightness(c, delta_l, whiteness);
    }
    let k = skin_params.light;
    let y = luminance(c);
    if (y <= 1e-9) {
        return shift_lightness(c, delta_l, whiteness);
    }
    let l = srgb_to_lab(encode3(c)).x;
    let target_y = lightness_to_luminance(clamp(l + delta_l, 0.0, 100.0));
    let m = target_y / y;
    let dir = skin_direction();
    if (delta_l < 0.0) {
        let lab_way = shift_lightness(c, delta_l, whiteness);
        var out = mix(lab_way, c * m, k.x);
        if (k.y > 0.0) {
            let y_out = luminance(out);
            out = skin_keep_luminance(out * pow(vec3<f32>(m), -0.7 * k.y * dir), y_out);
        }
        return clamp(out, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    let w = whiteness * mix(1.0, clamp((delta_l - 4.0) / 20.0, 0.0, 1.0), k.w);
    var out = shift_lightness(c, delta_l, w);
    if (k.z > 0.0) {
        // More colour at the same luminance: the colour moves away from the grey of its own luminance, by more where the light adds more.
        let y_out = luminance(out);
        out = vec3<f32>(y_out) + (out - vec3<f32>(y_out)) * pow(m, 0.6 * k.z);
    }
    return clamp(out, vec3<f32>(0.0), vec3<f32>(1.0));
}
