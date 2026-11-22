// The eyeball's texture: the iris, the pupil and the eye white of both eyes, per pixel (`head_paint::eyeball`, `src/eyeball.rs`).
//
// The texture is 2N by N texels, one packed sRGB texel per pixel, rows from the top. Each eye is an N by N square seen from the front: the
// texel at (x, y) of a square shows the point of the eyeball whose unit direction has x across and y up. The iris is a flat disc, so its
// radius in the square is its radius over the eyeball's radius, and every ring of the iris is a circle there.
// Every yes-or-no choice comes from a whole-number hash of the lattice cell and the pattern.

struct Ball {
    // The iris colour of each eye in linear sRGB, and the first eye's iris radius as a share of the square's half width.
    iris0: vec4<f32>,
    // The second eye's iris colour, and the pupil radius as a share of the iris radius.
    iris1: vec4<f32>,
    // The second colour round the pupil, and its amount.
    inner: vec4<f32>,
    // The eye white's colour, and how much the lids shade it.
    white: vec4<f32>,
    // The colour the eye white takes where it is red, and the redness.
    red: vec4<f32>,
    // Fibre contrast, outer ring darkness, outer ring width, and the reach of the second colour.
    looks: vec4<f32>,
    // Clouding, catchlight, the second eye's iris radius as a share of the square's half width, and one unused.
    more: vec4<f32>,
    // Where each iris's centre sits on its square: the first eye's x and y, then the second eye's.
    centres: vec4<f32>,
    // The pattern, the square's size N, and two unused.
    pattern: u32,
    size: u32,
    pad0: u32,
    pad1: u32,
}

@group(0) @binding(0) var<uniform> ball: Ball;
@group(0) @binding(1) var<storage, read_write> texels: array<u32>;

const TAU: f32 = 6.2831853;

fn hash(v: vec3<i32>, salt: u32) -> f32 {
    var h = (bitcast<u32>(v.x) * 0x8da6b343u) ^ (bitcast<u32>(v.y) * 0xd8163841u) ^ (bitcast<u32>(v.z) * 0xcb1ab31fu) ^ (salt * 0x9e3779b9u);
    h = h ^ (h >> 16u);
    h = h * 0x7feb352du;
    h = h ^ (h >> 15u);
    h = h * 0x846ca68bu;
    h = h ^ (h >> 16u);
    return f32(h >> 8u) / 16777216.0;
}

// Value noise from 0 to 1 on a unit lattice.
fn noise(p: vec3<f32>, salt: u32) -> f32 {
    let i = vec3<i32>(floor(p));
    let f = p - floor(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(hash(i, salt), hash(i + vec3<i32>(1, 0, 0), salt), u.x), mix(hash(i + vec3<i32>(0, 1, 0), salt), hash(i + vec3<i32>(1, 1, 0), salt), u.x), u.y);
    let b = mix(mix(hash(i + vec3<i32>(0, 0, 1), salt), hash(i + vec3<i32>(1, 0, 1), salt), u.x), mix(hash(i + vec3<i32>(0, 1, 1), salt), hash(i + vec3<i32>(1, 1, 1), salt), u.x), u.y);
    return mix(a, b, u.z);
}

// Noise round the iris: `around` lattice cells per turn of the angle, `along` cells per unit of the iris radius. The angle is wrapped by
// walking a circle, so the pattern has no seam.
fn polar(phi: f32, r: f32, around: f32, along: f32, salt: u32) -> f32 {
    let a = around / TAU;
    return noise(vec3<f32>(a * cos(phi), a * sin(phi), r * along), salt);
}

fn to_srgb(c: f32) -> f32 {
    let x = clamp(c, 0.0, 1.0);
    return select(1.055 * pow(x, 1.0 / 2.4) - 0.055, 12.92 * x, x <= 0.0031308);
}

fn pack(c: vec3<f32>) -> u32 {
    let b = vec3<u32>(round(vec3<f32>(to_srgb(c.x), to_srgb(c.y), to_srgb(c.z)) * 255.0));
    return b.x | (b.y << 8u) | (b.z << 16u) | (255u << 24u);
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// The colour of one point of one eye. `q` is the point in the eye's square, from -1 to 1, with y up.
fn eye_colour(q: vec2<f32>, eye: u32, px: f32) -> vec3<f32> {
    let rho = length(q);
    let share = select(ball.iris0.w, ball.more.z, eye == 1u);
    let pupil = ball.iris1.w;
    // The iris is measured from its own centre, which sits in the middle of the eye's opening.
    let iq = q - select(ball.centres.xy, ball.centres.zw, eye == 1u);
    let r = length(iq) / share;
    let phi = atan2(iq.y, iq.x);
    let dr = px / share;
    let salt = ball.pattern * 16u;
    // The second eye's fine detail takes its own salt, so that the two irises differ a little.
    let fine_salt = salt + 1u + eye * 7u;

    // --- the eye white ---
    var white = ball.white.rgb;
    // Redness toward the left and right corners, and thin veins that run in from the corners.
    let corner = smoothstep(0.3, 0.95, abs(q.x)) * smoothstep(1.05, 1.6, r);
    // The veins are contour lines of a noise stretched along the radius, so they run in from the edge, kept where a coarse mask lets them.
    let wphi = atan2(q.y, q.x);
    let warp = polar(wphi, rho, 5.0, 2.0, salt + 3u) - 0.5;
    let vein_line = abs(polar(wphi + 0.3 * warp, rho, 40.0, 1.3, fine_salt + 4u) - 0.5);
    let vein_mask = smoothstep(0.45, 0.7, polar(wphi, rho, 9.0, 1.5, fine_salt + 5u));
    let veins = (1.0 - smoothstep(0.004, 0.022, vein_line)) * vein_mask;
    let red = ball.red.w;
    white = mix(white, ball.red.rgb, clamp(red * (0.6 * corner + 0.15 * smoothstep(1.1, 1.8, r) + veins * (0.4 + corner)), 0.0, 1.0));

    // --- the iris ---
    let iris_colour = select(ball.iris0.rgb, ball.iris1.rgb, eye == 1u);
    let t = clamp((r - pupil) / max(1.0 - pupil, 1e-3), 0.0, 1.0);
    // The collarette: the wavy edge of the zone round the pupil.
    let wobble = 0.06 * (polar(phi, 0.0, 14.0, 1.0, salt + 6u) - 0.5) + 0.04 * (polar(phi, 0.0, 40.0, 1.0, fine_salt + 7u) - 0.5);
    let reach = ball.looks.w + wobble;
    let inner_share = 1.0 - smoothstep(reach - 0.05, reach + 0.05, t);
    var c = mix(iris_colour, ball.inner.rgb, ball.inner.w * inner_share);
    // A lighter rim on the collarette.
    let collar = exp(-pow((t - reach) / 0.05, 2.0));
    c = c * (1.0 + 0.35 * ball.inner.w * collar);
    // Fibres: streaks that run from the pupil to the edge, coarse and fine.
    let coarse = polar(phi, t, 26.0, 1.6, salt + 8u);
    let fine = polar(phi, t, 110.0, 2.5, fine_salt + 9u);
    let finest = polar(phi, t, 230.0, 4.0, fine_salt + 14u);
    let streaks = 0.45 * (coarse - 0.5) + 0.4 * (fine - 0.5) + 0.25 * (finest - 0.5);
    let fibres = ball.looks.x;
    c = c * exp(1.3 * fibres * streaks);
    // Mottling: broad blotches of the colour, and small dark specks.
    let blotch = polar(phi, t, 9.0, 2.5, fine_salt + 15u) - 0.5;
    c = c * exp(1.3 * fibres * blotch);
    let speck = smoothstep(0.78, 0.9, polar(phi, t, 70.0, 12.0, fine_salt + 16u));
    // The specks and crypts stop growing past a fibre contrast of 1, so a coarse iris does not fill with black dots.
    let gaps = min(fibres, 1.0);
    c = c * (1.0 - 0.3 * gaps * speck);
    // Crypts: a few soft dark gaps on both sides of the collarette.
    let crypt_noise = polar(phi, t, 18.0, 5.0, fine_salt + 10u);
    let crypts = smoothstep(0.66, 0.8, crypt_noise) * exp(-pow((t - reach) / 0.25, 2.0));
    c = c * (1.0 - 0.45 * gaps * crypts);
    // Furrows: thin broken arcs in the outer third.
    let arc = abs(fract(t * 5.0 + 0.6 * polar(phi, t, 10.0, 2.0, salt + 11u)) - 0.5);
    let furrows = (1.0 - smoothstep(0.0, 0.08, arc)) * smoothstep(0.55, 0.75, t) * smoothstep(0.45, 0.7, polar(phi, t, 22.0, 2.0, fine_salt + 12u));
    c = c * (1.0 - 0.35 * fibres * furrows);
    // A thin dark ruff at the pupil's edge.
    c = c * (1.0 - 0.35 * exp(-pow(t / 0.05, 2.0)));
    // The outer ring.
    let ring_width = ball.looks.z;
    let ring = smoothstep(1.0 - ring_width, 1.0, r) ;
    c = c * (1.0 - 0.9 * ball.looks.y * ring);

    // --- the pupil ---
    let pupil_colour = vec3<f32>(0.0022, 0.002, 0.0018);
    let in_pupil = 1.0 - smoothstep(pupil - 0.5 * dr - 0.03, pupil + 0.5 * dr + 0.03, r);
    c = mix(c, pupil_colour, in_pupil);

    // --- clouding: a milky haze over the iris and the pupil ---
    let cloud = ball.more.x;
    let haze = ball.white.rgb * (0.9 + 0.08 * (polar(phi, t, 30.0, 2.0, fine_salt + 13u) - 0.5));
    c = mix(c, haze * (1.0 - 0.1 * ring * (1.0 - cloud)), sqrt(cloud));

    // The iris's edge against the eye white, about one texel soft and a little more.
    let edge = 1.0 - smoothstep(1.0 - 0.5 * dr - 0.01, 1.0 + 0.5 * dr + 0.01, r);
    var col = mix(white, c, edge);
    col = mix(col, mix(white, haze, edge), cloud * cloud);

    // --- shading: the lids shade the top and the edges of the eyeball, the iris too ---
    let shade = ball.white.w;
    let dark = 0.5 * smoothstep(0.4, 1.0, rho) + 0.35 * smoothstep(0.05, 0.6, q.y);
    col = col * (1.0 - shade * clamp(dark, 0.0, 0.7));

    // --- the catchlight: a painted reflection of a light, above the pupil and toward the head's x > 0 side ---
    let spot = iq - vec2<f32>(0.07, 0.1) * share / 0.48;
    let glint = exp(-dot(spot, spot) / (0.0016 * share * share / 0.23));
    col = mix(col, vec3<f32>(0.95, 0.95, 0.95), ball.more.y * 0.95 * glint * (1.0 - cloud));
    return col;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let n = ball.size;
    if (id.x >= 2u * n || id.y >= n) {
        return;
    }
    let eye = id.x / n;
    let x = id.x % n;
    let px = 2.0 / f32(n);
    let q = vec2<f32>((f32(x) + 0.5) * px - 1.0, 1.0 - (f32(id.y) + 0.5) * px);
    texels[id.y * 2u * n + id.x] = pack(eye_colour(q, eye, px));
}
