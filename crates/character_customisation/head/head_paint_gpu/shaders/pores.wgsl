// The pore layer: a net of fine furrows whose corners are jittered points of a 3D grid, the visible pores on a share of those corners,
// and the mottling, as a change of lightness in L*. This is the design of recommendations 1 to 3 of
// docs/research/2026-11-20-pores-and-grain.md.
//
// The net has two levels, as the sources describe: primary lines around fields twice the grain size, and secondary lines inside
// them.
// Each level is the 3D Voronoi diagram of the grid points, cut by the skin, in a metric squeezed along the flow so that the cells are longer
// along it. The furrows are the cells' walls, and the pores sit where three cells meet. The shader takes the skin's plane from the
// derivatives of the point of the head, which also give the size of one pixel for the band limit.

struct PoreParams {
    // The net's cell edge in mm, the pores' mean radius on the nose in mm, the pores' darkness and the furrows' darkness in L*.
    a: vec4<f32>,
    // The mottling's amplitude in L* and its size in mm, the net's squeeze along the flow (1 is none), and the pores' mean long-to-short
    // axis ratio.
    b: vec4<f32>,
    // How steeply the pores fall away from the nose, the pores per mm² of skin on the nose, the number of regions.
    c: vec4<f32>,
    // The density and size factors of the skin under the regions, and the weight of that background.
    background: vec4<f32>,
    // The seeds as low and high words: the net's points, then the mottling.
    seeds: vec4<u32>,
    // 1 when the pores, the furrows and the mottling are on.
    on: vec4<u32>,
    // The light grains: their brightness in L* (0 when off), their spacing in mm, the dark points' darkness in L* (0 when off), and how
    // much the grains differ from each other.
    g: vec4<f32>,
    // x: how much the grains follow the painted light, from 0 to 1. y: how much each pore is a lit crater, from 0 to 1. z: how many
    // times larger than real the pores are drawn, from 1 to 3. w: how much the craters gleam, from 0 to 1.
    h: vec4<f32>,
}

@group(0) @binding(7) var<uniform> pore_params: PoreParams;
// Three vec4 per region of `head_paint::pores::pore_regions`: the first end and the reach, the second end, the density and size factors.
@group(0) @binding(8) var<storage, read> pore_data: array<vec4<f32>>;

// The size of one pixel where most of the face is painted, in mm. The furrows' darkness is the darkness drawn at this size.
const PORE_PIXEL_MM: f32 = 0.12;
// The narrowest a furrow is drawn: its half width in mm.
const FURROW_HALF_MM: f32 = 0.06;
// The skin lightness at which the furrows and the pores have the darkness their sliders give, in L*. The KOTOR heads' median is about 60.
const TONE_L: f32 = 60.0;
// The heights above the eyes, in mm, between which the furrows and the pores fade out on the forehead.
const FOREHEAD_FADE_MM: vec2<f32> = vec2<f32>(18.0, 32.0);
// The share of a pore's dark core that its light rim reaches, on top of Weiss's profile, whose rim is 13% of the core. No source
// measures it.
const PORE_RIM: f32 = 0.5;
// How soft a pore's edge is, in mm, on top of half a pixel, and how much of its dent is a round bowl instead of Weiss's sharp dent.
const PORE_SOFT_MM: f32 = 0.06;
const PORE_CORE: f32 = 0.6;
// The share of the nose's pores that are plugged, and how much darker those are.
const PORE_PLUGGED: f32 = 0.15;
const PORE_PLUG_DARKER: f32 = 1.8;
// The skin area per corner of the net, as a share of the cell's square. It turns pores per cm² into the chance of a corner to carry one.
// It was counted on a flat plane by `a_corner_of_the_net_covers_the_skin_area_the_shader_assumes` in `tests/pore_helpers.rs`.
const CORNER_AREA: f32 = 0.083;
// The primary fields' size against the grain size.
const PRIMARY_SCALE: f32 = 2.0;
// The secondary furrows' darkness against the primary furrows'.
const SECONDARY: f32 = 0.6;
// The sharp furrow's darkness, the soft valley's darkness and the lightening of a field's top, as shares of the furrows' darkness. The
// field reads as a raised dome lit from the front, with a thin line at its foot.
const CORE: f32 = 0.3;
const VALLEY: f32 = 0.6;
const DOME: f32 = 0.4;

// The density and size factors of the pore map at a point, against the nose's, before the gradient's power.
fn pore_regions_at(p: vec3<f32>) -> vec2<f32> {
    let bg = pore_params.background;
    var w = bg.z;
    var f = bg.z * bg.xy;
    let count = u32(pore_params.c.z);
    for (var r = 0u; r < count; r = r + 1u) {
        let a = pore_data[3u * r];
        let b = pore_data[3u * r + 1u];
        let v = pore_data[3u * r + 2u];
        let g = gauss(distance_to_segment(p, a.xyz, b.xyz), a.w);
        w = w + g;
        f = f + g * v.xy;
    }
    return f / w;
}

// The direction the furrows and pores run along: `marks_flow`, except that below the eyes it turns from straight down to down and out
// towards the ear, by half a right angle at 25 mm or more from the face's middle line. The research found the cheek's furrows and pores
// lined up from the outer eye corner towards the ear.
fn pore_flow(p: vec3<f32>) -> vec3<f32> {
    let left = 0.5 * (marks.eyes[0].xyz + marks.eyes[1].xyz);
    let right = 0.5 * (marks.eyes[2].xyz + marks.eyes[3].xyz);
    let across = safe_normalize(left - right);
    let middle = 0.5 * (left + right);
    let down = safe_normalize(marks.chin.xyz - middle);
    let lateral = dot(p - middle, across);
    let out = across * sign(lateral);
    let turn = smooth_step(5.0, 25.0, abs(lateral));
    let cheek = safe_normalize(down + turn * out);
    let f = marks_flow(p);
    // marks_flow's "down" part is replaced by the turned direction, in the share where it points down.
    let below = max(dot(f, down), 0.0);
    return safe_normalize(f + below * (cheek - down));
}

// The pore profile of Weiss and colleagues (2023), with x the distance over the pore's radius: -1 at the middle, 0 at the radius, and a
// low rim out to 3 radii.
fn pore_profile(x: f32, core: f32) -> f32 {
    if (x >= 3.0) {
        return 0.0;
    }
    let weiss = (x - 3.0) * (x - 3.0) * (x - 1.0) / 9.0;
    if (x >= 1.0) {
        return weiss;
    }
    // Inside the radius the dent is mixed with a rounder bowl, so that the middle is not a sharp point.
    let bowl = -(1.0 - x * x) * (1.0 - x * x);
    return mix(weiss, bowl, core);
}


// The four grid points nearest to a point in the squeezed metric, as (along the flow, across it, height off the skin's plane), with
// their squared distances and hashes.
struct PoreNet {
    near: array<vec3<f32>, 4>,
    d2s: array<f32, 4>,
    hs: array<vec2<u32>, 4>,
}

// The 64 random bits of a grid point, from the integer hash pcg3d of Jarzynski and Olano (2020). It needs only 32-bit integer
// arithmetic, so it costs a small share of `hash3`, and it is exact on every GPU. `tests/pore_helpers.rs` holds its Rust twin.
fn net_hash(i: i32, j: i32, k: i32, seed: vec2<u32>) -> vec2<u32> {
    var v = vec3<u32>(bitcast<u32>(i), bitcast<u32>(j), bitcast<u32>(k)) ^ vec3<u32>(seed.x, seed.y, seed.x ^ 0x9E3779B9u);
    v = v * 1664525u + 1013904223u;
    v.x = v.x + v.y * v.z;
    v.y = v.y + v.z * v.x;
    v.z = v.z + v.x * v.y;
    v = v ^ (v >> vec3<u32>(16u));
    v.x = v.x + v.y * v.z;
    v.y = v.y + v.z * v.x;
    v.z = v.z + v.x * v.y;
    return vec2<u32>(v.x ^ v.z, v.y);
}

fn pore_net(p: vec3<f32>, cell: f32, seed: vec2<u32>, t: vec3<f32>, side: vec3<f32>, n: vec3<f32>, squeeze: f32) -> PoreNet {
    var net: PoreNet;
    net.d2s = array<f32, 4>(1e9, 1e9, 1e9, 1e9);
    let c = vec3<i32>(floor(p / cell));
    for (var dx = -1; dx <= 1; dx = dx + 1) {
        for (var dy = -1; dy <= 1; dy = dy + 1) {
            for (var dz = -1; dz <= 1; dz = dz + 1) {
                let i = c.x + dx;
                let j = c.y + dy;
                let k = c.z + dz;
                let h = net_hash(i, j, k, seed);
                let q = vec3<f32>(f32(i) + hash_unit(h, 0u), f32(j) + hash_unit(h, 16u), f32(k) + hash_unit(h, 32u)) * cell;
                let d = q - p;
                var v = vec3<f32>(squeeze * dot(d, t), dot(d, side), dot(d, n));
                var d2 = dot(v, v);
                var hh = h;
                for (var m = 0; m < 4; m = m + 1) {
                    if (d2 < net.d2s[m]) {
                        let sv = net.near[m];
                        let sd = net.d2s[m];
                        let sh = net.hs[m];
                        net.near[m] = v;
                        net.d2s[m] = d2;
                        net.hs[m] = hh;
                        v = sv;
                        d2 = sd;
                        hh = sh;
                    }
                }
            }
        }
    }
    return net;
}

// The darkening of one level of furrows at the pixel, times `weight`: a sharp furrow on the wall between the two nearest points, a soft
// valley beside it, and a lighter top in the middle of each field, so that the fields look raised. The level fades out where its
// fields are under `low` pixels across.
fn pore_furrow(net: PoreNet, cell: f32, pixel: f32, weight: f32, low: f32, weakest: f32, squeeze: f32) -> f32 {
    let fade = smooth_step(low, low + 1.5, cell / pixel);
    let wall = (net.d2s[1] - net.d2s[0]) / (2.0 * length(net.near[1] - net.near[0]));
    let strength = weakest + (1.0 - weakest) * hash_unit(net.hs[0] ^ net.hs[1], 8u);
    // A weaker wall is narrower as well as fainter.
    let half = max(FURROW_HALF_MM, pixel) * (0.6 + 0.4 * strength);
    let amplitude = PORE_PIXEL_MM / max(half, PORE_PIXEL_MM);
    let u = min(wall / half, 1.0);
    let core = amplitude * (1.0 - u * u) * (1.0 - u * u);
    let valley = 1.0 - smooth_step(0.0, 0.35 * cell, wall);
    let top = smooth_step(0.1 * cell, 0.4 * cell, wall);
    // Each field has its own dome height, and about 30% of them are nearly flat.
    let height = smooth_step(0.2, 0.7, hash_unit(net.hs[0], 52u));
    let drawn = strength * (CORE * core + VALLEY * valley * valley) - DOME * height * top * top;
    // The level's mean over the skin is taken away, so that the furrows change the skin's tone nowhere on average and a faded level
    // leaves the tone as it is. The means of the cut cell pattern were counted on a flat plane (`net_means` in tests/pore_helpers.rs), at
    // a squeeze of 1 and 0.6; the mean height of a field is 0.55.
    let k = clamp((squeeze - 0.6) / 0.4, 0.0, 1.0);
    let wall_density = mix(5.986, 5.108, k);
    let base_half = max(FURROW_HALF_MM, pixel);
    let mean_strength = weakest + 0.5 * (1.0 - weakest);
    let mean_core = PORE_PIXEL_MM / max(base_half, PORE_PIXEL_MM) * wall_density * (base_half / cell) * (8.0 / 15.0) * (0.6 * mean_strength + 0.4 * (weakest * weakest + weakest * (1.0 - weakest) + (1.0 - weakest) * (1.0 - weakest) / 3.0));
    let mean = CORE * mean_core + mean_strength * VALLEY * mix(0.5667, 0.5021, k) - DOME * 0.55 * mix(0.0959, 0.1515, k);
    return weight * fade * (drawn - mean);
}

// The lit craters (`pore_craters`). While "Pore craters" is above 0 or "Stylised pore size" above 1, the pores are no longer the net's
// corners. They are points of a grid of their own, coarse enough that the 27 cubes round a pixel hold every pore that reaches it, so
// that a pore drawn three times its real size is never cut off. Each point is a ball, and the skin cuts it as a disc: a point far off
// the skin's plane makes a smaller pore, and the size changes smoothly from one pixel to the next.

// How far a crater's light rim and soft edge reach, against the crater's softened radius.
const CRATER_REACH: f32 = 1.3;
// How far the shadow's edge lies from the crater's own edge, toward the light, against the radius. It sets how wide the shadow is.
const CRATER_SHADOW_SHIFT: f32 = 0.7;
// Pale skin, by the L* of the skin's own colour under the light layers (`skin` of `pore_craters`): from 0 below the first value to 1
// above the second. On pale skin a pore is seen mostly as a break
// in the shine, so its dark middle fades to CRATER_PALE_FLOOR of the dark skin's, its shadow narrows to a thin band along the upper
// edge (the shift falls to CRATER_PALE_SHIFT) and weakens to CRATER_PALE_SHADOW, and its darkening keeps the skin's own colour instead
// of turning red-brown. Dark skin keeps the craters as they were.
const CRATER_PALE_L: vec2<f32> = vec2<f32>(30.0, 48.0);
const CRATER_PALE_FLOOR: f32 = 0.25;
const CRATER_PALE_SHIFT: f32 = 0.35;
const CRATER_PALE_SHADOW: f32 = 0.9;
// Where a crater's soft edge starts and ends, against its radius.
const CRATER_EDGE: vec2<f32> = vec2<f32>(0.55, 1.1);
// The darkness of the shadow on the side nearest the light, against the pores' darkness. At 1 the craters' dark top and lit bottom
// made them read as raised beads.
const CRATER_SHADOW: f32 = 0.6;
// The darkness of a crater's middle, against the pores' darkness. A pore reads as a dent by its dark middle more than by its rim.
const CRATER_FLOOR: f32 = 0.5;
// The light rim's lightening against the shadow's darkening, on skin of L* 60. A lit rim adds light, so on darker skin it is divided by
// the tone factor. A stronger rim made the pores read as raised drops and showed as white specks at face distance.
const CRATER_LIGHT: f32 = 0.3;
// How far a crater's point may lie from its cube's middle, as a share of the cube. Less than the whole cube keeps craters from
// overlapping in pairs.
const CRATER_JITTER: f32 = 0.6;
// The share of the shadow that remains where the light runs along the skin's normal, as under the nose, and the crater has no lit side.
const CRATER_FLAT: f32 = 0.6;
// The depth of a crater in the maps' height, against its radius.
const CRATER_DEPTH: f32 = 0.3;
// The most a crater's long axis may be against its short one. The pores' own stretch made the craters teardrops along the flow.
const CRATER_RATIO_MOST: f32 = 1.25;
// The spread of the craters' radii: a log-normal with this sigma, cut at CRATER_SPREAD_MOST and divided by the cut spread's mean
// (CRATER_SPREAD_MEAN), so that the mean radius is the one the sliders ask for. Most pores are a little smaller than the mean and a few
// are larger. CRATER_SPREAD_SQUARE is the mean of the square of the spread, which the balance needs. The two means were counted on
// 2 million draws. A cut at 1.8 left large pores that read as scars.
const CRATER_SIGMA: f32 = 0.45;
const CRATER_SPREAD_MOST: f32 = 1.5;
const CRATER_SPREAD_MEAN: f32 = 0.948;
const CRATER_SPREAD_SQUARE: f32 = 1.137;
// The integrals over the plane of a crater's shadow (at the full shift and at CRATER_PALE_SHIFT), its middle and its light rim, each with
// weight 1, in units of its softened radius, computed numerically from `crater_profile` with the light along the skin. The mean square
// of the ball's cut is pi / 4. The gleam's integral is 0.167 by the same count, but the skin's mean lightness on pmba01, Carth and Kreia
// stayed level only at about 0.095, so the gleam uses the measured value.
const CRATER_SHADOW_AREA: vec2<f32> = vec2<f32>(1.1582, 0.6514);
const CRATER_FLOOR_AREA: f32 = 0.9803;
const CRATER_RIM_AREA: f32 = 0.411;
const CRATER_GLEAM_AREA: f32 = 0.095;
const CRATER_CUT_SQUARE: f32 = 0.785;
// The gleam: a thin arc on the lit lower rim, at this distance from the crater's middle and this wide, both against the radius, and
// narrowed round the edge by the power of how squarely the wall faces the light. A round spot made the pores look like water drops.
const GLEAM_AT: f32 = 0.85;
const GLEAM_WIDTH: f32 = 0.08;
const GLEAM_NARROW: f32 = 6.0;
// The light a gleam adds at its peak, in linear sRGB, on dark skin and on pale skin, and how much of it takes the skin's own colour
// instead of white. On pale skin a stronger arc than the thin shadow made the pores look like clear drops.
const GLEAM_LIGHT: vec2<f32> = vec2<f32>(0.35, 0.25);
const GLEAM_TINT: f32 = 0.4;
// The least oiliness: dry skin still gleams this much, against the oiliest skin.
const GLEAM_DRY: f32 = 0.2;
// How much the gleam raises the maps' gloss on a crater's lit rim, and how much a crater's inside loses of the skin's gloss.
const GLEAM_RIM_GLOSS: f32 = 0.6;
const CRATER_INSIDE_GLOSS: f32 = 0.9;

// The gleam at the pixel and its mean over the skin, which `pore_shift` leaves for `pore_gleam`. The paint pass sets both to 0 first.
var<private> crater_gleam: f32;
var<private> crater_gleam_mean: f32;
// The share of the craters' darkening, in L*, that the paint pass takes from the colour by a plain factor on its linear light, so that
// it keeps the skin's colour. `pore_shift` leaves it, and the paint pass sets it to 0 first.
var<private> crater_shade: f32;
// The gleam's peak light at the pixel, from GLEAM_LIGHT by how pale the skin is.
var<private> crater_gleam_light: f32;

// A crater's darkening, lightening, depth and gleam at the pixel, from `d`, the pixel's offset from the crater's middle over its
// softened radius (stretched so that the crater is round), and `l`, the light's direction along the skin in the same space, whose
// length says how much the light comes from one side.
fn crater_profile(d: vec2<f32>, l: vec2<f32>, pale: f32) -> vec4<f32> {
    let x = length(d);
    // The edge is soft, from 55% to 110% of the radius. A crisp outline made the craters look like water drops on the skin.
    let inside = 1.0 - smooth_step(CRATER_EDGE.x, CRATER_EDGE.y, x);
    let side = smooth_step(0.15, 0.5, length(l));
    let ld = safe_normalize2(l);
    // The floor is lit where a step toward the light stays inside the crater, and in shadow where it leaves it.
    let shifted = 1.0 - smooth_step(CRATER_EDGE.x, CRATER_EDGE.y, length(d + mix(CRATER_SHADOW_SHIFT, CRATER_PALE_SHIFT, pale) * ld));
    let bowl = (1.0 - min(x * x, 1.0)) * (1.0 - min(x * x, 1.0));
    let shadow = inside * (1.0 - shifted);
    let lit_dark = CRATER_SHADOW * mix(1.0, CRATER_PALE_SHADOW, pale) * shadow + CRATER_FLOOR * mix(1.0, CRATER_PALE_FLOOR, pale) * inside * bowl;
    let flat_dark = CRATER_FLAT * inside * bowl;
    // The far wall faces the light and catches a thin crescent along the lower half of the edge.
    let far = max(-dot(safe_normalize2(d), ld), 0.0);
    let rim = far * far * smooth_step(0.6, 0.85, x) * (1.0 - smooth_step(0.95, 1.1, x));
    // The gleam is a thin bright arc on that crescent, where the far wall turns its oily skin to the light.
    let g = x - GLEAM_AT;
    let gleam = exp(-g * g / (2.0 * GLEAM_WIDTH * GLEAM_WIDTH)) * pow(far, GLEAM_NARROW);
    return vec4<f32>(mix(flat_dark * mix(1.0, CRATER_PALE_FLOOR, pale), lit_dark, side), side * rim, inside, side * gleam);
}

fn safe_normalize2(v: vec2<f32>) -> vec2<f32> {
    let l = length(v);
    if (l < 1e-12) {
        return vec2<f32>(0.0);
    }
    return v / l;
}

// The pores' darkening as craters, before the pores' darkness, minus their lightening, plus the mean darkening of pores too small to
// draw, less the craters' own mean so that the skin keeps its tone at a distance. `up` is the direction the light comes from, and
// `oil` how oily the skin is here, from 0 to 1. It leaves the gleam in `crater_gleam` and `crater_gleam_mean`.
fn pore_craters(p: vec3<f32>, pixel: f32, t: vec3<f32>, side: vec3<f32>, n: vec3<f32>, up: vec3<f32>, seed: vec2<u32>, rim_kept: f32, tone: f32, oil: f32, skin_colour: vec3<f32>) -> f32 {
    let crater = pore_params.h.y;
    let scale = pore_params.h.z;
    let gleam_amount = pore_params.h.w * crater;
    let radius = pore_params.a.y * scale;
    let ratio = pore_params.b.w;
    // The pores per mm² fall as the size to the power 1.5, a little slower than the area grows, so that larger pores stay a field and
    // not a scatter.
    let density = pore_params.c.y / (scale * sqrt(scale));
    // The largest pore: the size factor is at most 1, a pore's own size at most 1.3 times the mean (CRATER_SPREAD_MOST with craters),
    // and its long axis sqrt(1.2 ratio) times its radius (at most sqrt(CRATER_RATIO_MOST) with craters). The pixel is taken at 0.3 mm,
    // the most a face texel covers before the pores fade out.
    let largest = select(1.3, CRATER_SPREAD_MOST / CRATER_SPREAD_MEAN, crater > 0.0) * radius;
    let longest = select(1.2 * ratio, min(1.2 * ratio, CRATER_RATIO_MOST), crater > 0.0);
    let reach = CRATER_REACH * sqrt(largest * largest * longest + 0.25 * 0.09 + PORE_SOFT_MM * PORE_SOFT_MM);
    let cell = max(reach, 0.3);
    let half = 0.5 * cell;
    // The light along the skin, as (along the flow, across it).
    let light = vec2<f32>(dot(up, t), dot(up, side));
    let k_rim = CRATER_LIGHT / tone * rim_kept;
    let pale = smooth_step(CRATER_PALE_L.x, CRATER_PALE_L.y, srgb_to_lab(encode3(max(skin_colour, vec3<f32>(0.0)))).x) * crater;
    var dark = 0.0;
    var lit = 0.0;
    var bowl = 0.0;
    var gleam = 0.0;
    var rim_most = 0.0;
    var inside_most = 0.0;
    let c = vec3<i32>(floor(p / cell));
    for (var dx = -1; dx <= 1; dx = dx + 1) {
        for (var dy = -1; dy <= 1; dy = dy + 1) {
            for (var dz = -1; dz <= 1; dz = dz + 1) {
                let i = c.x + dx;
                let j = c.y + dy;
                let k = c.z + dz;
                let h = net_hash(i, j, k, seed);
                let jitter = vec3<f32>(hash_unit(h, 0u), hash_unit(h, 16u), hash_unit(h, 32u));
                let q = (vec3<f32>(f32(i), f32(j), f32(k)) + 0.5 + CRATER_JITTER * (jitter - 0.5)) * cell;
                let e = q - p;
                let height = dot(e, n);
                if (abs(height) >= half) {
                    continue;
                }
                let roll = hash_unit(h, 48u);
                let chance = density * cell * cell;
                if (roll >= chance) {
                    continue;
                }
                let f = pow(pore_regions_at(q), vec2<f32>(pore_params.c.x));
                if (roll >= chance * f.x) {
                    continue;
                }
                let h2 = hash_round(h, 7);
                let h3 = hash_round(h, 11);
                // The ball's cut: the pore shrinks to nothing as its point leaves the slab of the skin.
                let cut = sqrt(sqrt(max(1.0 - (height / half) * (height / half), 0.0)));
                // The spread of sizes: even from 0.7 to 1.3 for the flat dots, log-normal for the craters. The sum of three even numbers
                // stands in for a normal one.
                let z = 2.0 * (hash_unit(h3, 0u) + hash_unit(h3, 16u) + hash_unit(h3, 32u) - 1.5);
                let spread = min(exp(CRATER_SIGMA * z - 0.5 * CRATER_SIGMA * CRATER_SIGMA), CRATER_SPREAD_MOST) / CRATER_SPREAD_MEAN;
                let r = radius * f.y * mix(0.7 + 0.6 * hash_unit(h2, 0u), spread, crater) * cut;
                var kk = ratio * (0.8 + 0.4 * hash_unit(h2, 16u));
                kk = mix(kk, min(kk, CRATER_RATIO_MOST), crater);
                let sq = sqrt(kk);
                // The pixel's offset from the pore's middle, in true mm along the flow and across it.
                let offset = -vec2<f32>(dot(e, t), dot(e, side));
                let soft = sqrt(r * r + 0.25 * pixel * pixel + PORE_SOFT_MM * PORE_SOFT_MM);
                let d = vec2<f32>(offset.x / sq, offset.y * sq) / soft;
                if (dot(d, d) >= CRATER_REACH * CRATER_REACH) {
                    continue;
                }
                let seen = smooth_step(0.5, 1.0, r / pixel);
                var depth = seen * (r * r / (soft * soft));
                if (f.x > 0.9 && hash_unit(h2, 32u) < PORE_PLUGGED) {
                    depth = depth * PORE_PLUG_DARKER;
                }
                // Each crater has its own depth, from 60% to 140% of the mean.
                depth = depth * mix(1.0, 0.6 + 0.8 * hash_unit(h2, 48u), crater);
                // The flat dot of the pores at their real size, with Weiss's profile, and the crater, mixed by "Pore craters".
                let v = pore_profile(length(d), PORE_CORE);
                let cr = crater_profile(d, vec2<f32>(light.x / sq, light.y * sq), pale);
                let dot_dark = max(-v, 0.0);
                let dot_rim = PORE_RIM * rim_kept * max(v, 0.0);
                dark = max(dark, depth * mix(dot_dark, cr.x, crater));
                lit = max(lit, depth * mix(dot_rim, k_rim * cr.y, crater));
                let x = length(d);
                bowl = max(bowl, crater * cr.z * (1.0 - min(x * x, 1.0)) * (1.0 - min(x * x, 1.0)) * CRATER_DEPTH * r);
                gleam = max(gleam, seen * cr.w);
                rim_most = max(rim_most, seen * cr.y);
                inside_most = max(inside_most, seen * cr.z);
            }
        }
    }
    let here = pow(pore_regions_at(p), vec2<f32>(pore_params.c.x));
    let r = radius * here.y;
    let seen_here = smooth_step(0.5, 1.0, r / pixel);
    // The craters' mean over the skin: the pores per mm² here, times the mean square of their radius, times the area integrals.
    let per_mm2 = density * here.x * r * r * CRATER_SPREAD_SQUARE * CRATER_CUT_SQUARE;
    let dark_area = CRATER_SHADOW * mix(1.0, CRATER_PALE_SHADOW, pale) * mix(CRATER_SHADOW_AREA.x, CRATER_SHADOW_AREA.y, pale) + CRATER_FLOOR * mix(1.0, CRATER_PALE_FLOOR, pale) * CRATER_FLOOR_AREA;
    let balance = crater * seen_here * per_mm2 * (dark_area - k_rim * CRATER_RIM_AREA);
    let gleam_strength = gleam_amount * (GLEAM_DRY + (1.0 - GLEAM_DRY) * oil);
    crater_gleam = gleam_strength * gleam;
    crater_gleam_mean = gleam_strength * seen_here * per_mm2 * CRATER_GLEAM_AREA;
    if (maps_on()) {
        maps_height = maps_height - bowl;
        // The rims shine and the holes are dull, so that the viewer's moving highlight catches the rims and breaks round the holes. The
        // whole inside of a crater loses the gloss, not only its middle, so that the broad shine of a nose breaks round each hole.
        maps_gloss = clamp(maps_gloss * (1.0 - CRATER_INSIDE_GLOSS * crater * inside_most) + GLEAM_RIM_GLOSS * gleam_strength * rim_most, 0.0, 1.0);
    }
    let mean = (1.0 - crater) * (1.0 - seen_here) * density * here.x * 0.6 * r * r;
    crater_shade = pale * dark;
    crater_gleam_light = mix(GLEAM_LIGHT.x, GLEAM_LIGHT.y, pale / max(crater, 1e-6));
    return (1.0 - pale) * dark - lit + mean - balance;
}

// The colour with the pale skin's share of the craters' darkening, then the craters' gleam: a near-white light added on each crater's
// lit rim, which follows the painted light as the light grains do (`lit`, in L* against the base colour). The skin gives up the gleam's mean light, so that its tone stays at a distance.
// The paint pass calls it only under a test of uniforms.
fn pore_gleam(c_in: vec3<f32>, lit: f32) -> vec3<f32> {
    var c = c_in;
    // The pale skin's share of the craters' darkening, by a plain factor on the linear light.
    if (crater_shade > 0.0) {
        let l = srgb_to_lab(encode3(max(c, vec3<f32>(0.0)))).x;
        let y0 = lightness_to_luminance(l);
        if (y0 > GLINT_DARKEST) {
            c = c * (lightness_to_luminance(max(l - crater_shade, 0.0)) / y0);
        }
    }
    if (crater_gleam_mean == 0.0 && crater_gleam == 0.0) {
        return c;
    }
    let light = clamp(1.0 + pore_params.h.x * lit / GLINT_LIT_L, 0.0, GLINT_MOST);
    let positive = max(c, vec3<f32>(0.0));
    let y = luminance(positive);
    var tint = vec3<f32>(1.0);
    if (y > GLINT_DARKEST) {
        tint = min(positive / y, vec3<f32>(GLINT_TINT_MOST));
    }
    let colour = mix(vec3<f32>(1.0), tint, GLEAM_TINT);
    let keep = max(1.0 - light * crater_gleam_light * crater_gleam_mean * luminance(colour) / max(y, GLINT_DARKEST), GLINT_FLOOR);
    return clamp(c * keep + light * crater_gleam_light * crater_gleam * colour, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn pore_gleam_on() -> bool {
    return pore_params.h.y != 0.0 && pore_params.on.x != 0u;
}

// `skin_colour` is the skin's own colour under the light layers, which the craters read to tell pale skin from dark.
fn pore_shift(pt: Point, skin_colour: vec3<f32>) -> f32 {
    let p = pt.position_mm;
    // The derivatives are taken first, while the control flow is the same for every pixel.
    let du = dpdx(p);
    let dv = dpdy(p);
    if (marks.present.x == 0u) {
        return 0.0;
    }
    let skin = marks_skin(p);
    if (skin == 0.0) {
        return 0.0;
    }
    var pixel = max(length(du), length(dv));
    var n = safe_normalize(cross(du, dv));
    if (pixel < 1e-5) {
        // A pixel drawn as a single point has no derivatives: the face's pixel size and the direction from the head's middle stand in.
        pixel = PORE_PIXEL_MM;
        n = safe_normalize(p - frame.origin.xyz);
    }
    // A furrow or a pore darkens by shadow, which takes away a share of the light, so its change of L* is smaller on darker skin. The
    // square root keeps more contrast on dark skin than a plain share would, which looked like plastic.
    let tone = clamp(sqrt(srgb_to_lab(encode3(frame.base.xyz)).x / TONE_L), 0.5, 1.0);
    var shift = 0.0;
    if (pore_params.on.z != 0u) {
        let s = vec2<u32>(pore_params.seeds.z, pore_params.seeds.w);
        shift = pore_params.b.x * fractal_noise(p, pore_params.b.y, 2u, 0.5, s);
    }
    // The upper forehead of the layout has pixels that grow in steps from 0.12 mm to more than 0.3 mm. Each level of the net would fade
    // at one of those steps and draw a hard edge, so the furrows and the pores fade out by height above the eyes instead, between 18 and
    // 32 mm, before the steps begin.
    let eyes_middle = 0.25 * (marks.eyes[0].xyz + marks.eyes[1].xyz + marks.eyes[2].xyz + marks.eyes[3].xyz);
    let face_down = safe_normalize(marks.chin.xyz - eyes_middle);
    let forehead_fade = 1.0 - smooth_step(FOREHEAD_FADE_MM.x, FOREHEAD_FADE_MM.y, -dot(p - eyes_middle, face_down));
    // The detail tile of the maps follows the pores' map of regions, most on the nose and least round the eyes. It fades out up the
    // forehead with the pores, and from 50 to 75 mm behind the eyes, measured level with the face, toward the ears and the back of the head. There the layout's texels
    // grow up to 0.6 mm, too large for a tile repeated over the texture.
    if (maps_on()) {
        let to_nose = marks.nose_tip.xyz - eyes_middle;
        let ahead = dot(p - eyes_middle, safe_normalize(to_nose - dot(to_nose, face_down) * face_down));
        maps_tile = skin * forehead_fade * smooth_step(-75.0, -50.0, ahead) * pow(pore_regions_at(p), vec2<f32>(pore_params.c.x)).x;
    }
    if ((pore_params.on.x == 0u && pore_params.on.y == 0u) || forehead_fade == 0.0) {
        return skin * shift;
    }
    var detail = 0.0;

    let cell = pore_params.a.x;
    var t = pore_flow(p);
    t = safe_normalize(t - dot(t, n) * n);
    let side = cross(n, t);
    let squeeze = pore_params.b.z;
    let seed = vec2<u32>(pore_params.seeds.x, pore_params.seeds.y);

    let fine = pore_net(p, cell, seed, t, side, n, squeeze);
    let coarse = pore_net(p, PRIMARY_SCALE * cell, u64_add(seed, vec2<u32>(0x9E37u, 0u)), t, side, n, squeeze);
    let near = fine.near;
    let d2s = fine.d2s;
    let hs = fine.hs;

    // The furrows are the cells' walls, on two levels: the deeper primary lines around fields twice the grain size, and the secondary
    // lines inside them. A level whose fields are under a few pixels cannot be drawn without blocks, so it fades out.
    if (pore_params.on.y != 0u) {
        detail = detail - tone * pore_params.a.w * (pore_furrow(fine, cell, pixel, SECONDARY, 1.8, 0.15, squeeze) + pore_furrow(coarse, PRIMARY_SCALE * cell, pixel, 1.0, 3.0, 0.45, squeeze));
    }

    // The pores sit where three cells meet. Each triple of the four nearest points meets at the point of the skin's plane that lies as far
    // from all three, and a share of those corners carry a pore.
    if (pore_params.on.x != 0u && (pore_params.h.y != 0.0 || pore_params.h.z != 1.0)) {
        // The light comes from above, straight down the face, as the shading layer's sky is brightest straight up.
        let rim_kept = select(1.0, maps_params.pores.x, maps_on());
        // The oily skin: the T-zone, the sheen's regions and the pore map's density, which is highest on the nose and the inner cheeks.
        let oil = clamp(max(max(marks_t_zone(p), max(pt.sheen.x, max(pt.sheen.y, pt.sheen.z))), pow(pore_regions_at(p), vec2<f32>(pore_params.c.x)).x), 0.0, 1.0);
        detail = detail - tone * pore_params.a.z * pore_craters(p, pixel, t, side, n, -face_down, u64_add(seed, vec2<u32>(0xC7A7u, 0u)), rim_kept, tone, oil, skin_colour);
        crater_shade = crater_shade * tone * pore_params.a.z * skin * forehead_fade;
    } else if (pore_params.on.x != 0u) {
        let radius = pore_params.a.y;
        let ratio = pore_params.b.w;
        var dark = 0.0;
        var rim = 0.0;
        for (var skip = 0; skip < 4; skip = skip + 1) {
            var tri: array<u32, 3>;
            var m = 0;
            for (var q = 0; q < 4; q = q + 1) {
                if (q != skip) {
                    tri[m] = u32(q);
                    m = m + 1;
                }
            }
            let a = near[tri[0]];
            let b = near[tri[1]];
            let e = near[tri[2]];
            // |x - q|^2 is the same for the three points, with x in the plane (height 0): 2 x . (b - a) = |b|^2 - |a|^2, and so for e.
            let m1 = 2.0 * (b.xy - a.xy);
            let m2 = 2.0 * (e.xy - a.xy);
            let det = m1.x * m2.y - m1.y * m2.x;
            if (abs(det) < 1e-6 * cell * cell) {
                continue;
            }
            let r1 = dot(b, b) - dot(a, a);
            let r2 = dot(e, e) - dot(a, a);
            let corner = vec2<f32>(r1 * m2.y - r2 * m1.y, m1.x * r2 - m2.x * r1) / det;
            // The corner is real only when no fourth point is nearer to it than the three.
            let fourth = near[u32(skip)];
            let to_a = vec3<f32>(corner, 0.0) - a;
            let to_f = vec3<f32>(corner, 0.0) - fourth;
            if (dot(to_f, to_f) < dot(to_a, to_a)) {
                continue;
            }
            let h = hs[tri[0]] ^ hs[tri[1]] ^ hs[tri[2]];
            // The region map is never above 1, so most corners are turned away before it is read.
            let chance = pore_params.c.y * CORNER_AREA * cell * cell;
            let roll = hash_unit(h, 48u);
            if (roll >= chance) {
                continue;
            }
            // The corner in true millimetres, from the pixel.
            let offset = vec2<f32>(corner.x / squeeze, corner.y);
            let world = p + offset.x * t + offset.y * side;
            let f = pow(pore_regions_at(world), vec2<f32>(pore_params.c.x));
            if (roll >= chance * f.x) {
                continue;
            }
            let h2 = hash_round(h, 7);
            let r = radius * f.y * (0.7 + 0.6 * hash_unit(h2, 0u));
            let k = ratio * (0.8 + 0.4 * hash_unit(h2, 16u));
            // The long axis runs along the flow, and the pore keeps the area of a round pore of radius r.
            let sq = sqrt(k);
            let d = vec2<f32>(offset.x / sq, offset.y * sq);
            let soft2 = r * r + 0.25 * pixel * pixel + PORE_SOFT_MM * PORE_SOFT_MM;
            let x = length(d) / sqrt(soft2);
            var depth = smooth_step(0.5, 1.0, r / pixel) * (r * r / soft2);
            if (f.x > 0.9 && hash_unit(h2, 32u) < PORE_PLUGGED) {
                depth = depth * PORE_PLUG_DARKER;
            }
            let v = pore_profile(x, PORE_CORE);
            if (v < 0.0) {
                dark = max(dark, -v * depth);
            } else {
                rim = max(rim, v * depth);
            }
        }
        // Pores too small to draw darken the skin on average instead: the chance of a corner to carry a pore, times the share of the skin
        // that the pore's core darkens.
        let here = pow(pore_regions_at(p), vec2<f32>(pore_params.c.x));
        let r = radius * here.y;
        let mean = (1.0 - smooth_step(0.5, 1.0, r / pixel)) * pore_params.c.y * here.x * 0.6 * r * r;
        // While the detail tile gives the pores their relief, the colour keeps only a share of each pore's light rim.
        detail = detail - tone * pore_params.a.z * (dark - PORE_RIM * select(1.0, maps_params.pores.x, maps_on()) * rim + mean);
    }
    shift = shift + forehead_fade * detail;
    return skin * shift;
}

// The light grains (`pore_glint`): the small bumps of skin between the furrows, each catching a little light, with a dark point where three
// of them meet. The fine bands of KOTOR's high-detail heads hold as many light spots as dark ones, and on dark skin more light than dark,
// because their photographs carry the light of each bump. A lit bump adds light, so the grains are added to the linear colour, and they
// show more on dark skin than on pale skin. The dark points take a share of the light, so they show more on pale skin.

// The skin lightness at which a grain's brightness is the L* its slider gives, and the one for the dark points.
const GLINT_REF_L: f32 = 50.0;
const GLINT_POINT_L: f32 = 60.0;
// How much of a grain's added light takes the skin's own colour instead of white.
const GLINT_TINT: f32 = 0.9;
// The painted light, in L*, at which "Grains follow the light" at 1 doubles the grains, or takes them away when it is a shade.
const GLINT_LIT_L: f32 = 5.0;
// The most that the painted light multiplies the grains by. More draws a crackled net on a strongly lit nose.
const GLINT_MOST: f32 = 1.6;
// The share of the grains' squeeze along the flow that the pore net has.
const GLINT_SQUEEZE: f32 = 0.5;
// A grain's radius against the grains' spacing.
const GLINT_RADIUS: f32 = 0.75;
// The means over the skin of a grain's speck (before its own strength) and of the dark points, at a squeeze between 1 and 0.8. The
// dark points' mean replaces them where a grain is under a few pixels across. They were counted on a flat plane by `glint_means` in
// tests/pore_helpers.rs (0.333 to 0.386, and 0.033 to 0.036).
const GLINT_MEAN: f32 = 0.36;
const GLINT_POINT_MEAN: f32 = 0.035;
// The least share of its light that the skin between the grains keeps.
const GLINT_FLOOR: f32 = 0.4;
// The luminance under which the skin is too dark to give the grains its own colour, and the largest a tint may be: 1 over the smallest
// weight of a channel in the luminance, blue's 0.0722.
const GLINT_DARKEST: f32 = 1e-4;
const GLINT_TINT_MOST: f32 = 13.85;
// The reach of a dark point from where three grains meet, against the grains' spacing. A wider reach joins the points into a net.
const GLINT_POINT: f32 = 0.08;

// The three grid points nearest to a point in the squeezed metric, for the light grains: their offsets (along the flow, across it, height
// off the skin's plane), their squared distances, and the nearest one's hash. It finds what `pore_net` finds, but keeps the three
// points in plain variables. With `pore_net`'s arrays a second time in the paint pass, Chrome's compiler painted the brows' hairs
// in a wrong colour on some texels of the browser's texture (green specks on Male B1), and natively it did not.
struct GlintNet {
    q0: vec3<f32>,
    q1: vec3<f32>,
    q2: vec3<f32>,
    d: vec3<f32>,
    h0: vec2<u32>,
}

fn glint_net(p: vec3<f32>, cell: f32, seed: vec2<u32>, t: vec3<f32>, side: vec3<f32>, n: vec3<f32>, squeeze: f32) -> GlintNet {
    var r: GlintNet;
    r.d = vec3<f32>(1e9);
    let c = vec3<i32>(floor(p / cell));
    for (var dx = -1; dx <= 1; dx = dx + 1) {
        for (var dy = -1; dy <= 1; dy = dy + 1) {
            for (var dz = -1; dz <= 1; dz = dz + 1) {
                let i = c.x + dx;
                let j = c.y + dy;
                let k = c.z + dz;
                let h = net_hash(i, j, k, seed);
                let q = vec3<f32>(f32(i) + hash_unit(h, 0u), f32(j) + hash_unit(h, 16u), f32(k) + hash_unit(h, 32u)) * cell;
                let e = q - p;
                let v = vec3<f32>(squeeze * dot(e, t), dot(e, side), dot(e, n));
                let d2 = dot(v, v);
                if (d2 < r.d.x) {
                    r.q2 = r.q1;
                    r.q1 = r.q0;
                    r.q0 = v;
                    r.d = vec3<f32>(d2, r.d.x, r.d.y);
                    r.h0 = h;
                } else if (d2 < r.d.y) {
                    r.q2 = r.q1;
                    r.q1 = v;
                    r.d = vec3<f32>(r.d.x, d2, r.d.y);
                } else if (d2 < r.d.z) {
                    r.q2 = v;
                    r.d.z = d2;
                }
            }
        }
    }
    return r;
}

fn pore_glint_on() -> bool {
    return pore_params.g.x != 0.0 || pore_params.g.z != 0.0;
}

// The colour with the light grains and their dark points, from the colour after the other layers' changes of lightness, the point, and
// `lit`, the light that the shading and the sheen painted at the point, in L* against the base colour. The paint pass calls it only
// under a test of uniforms, so the control flow is the same for every pixel when it starts.
fn pore_glint(c: vec3<f32>, pt: Point, lit: f32) -> vec3<f32> {
    // The texel's two sides on the skin, taken here before any return. They used to be taken at the top of the paint pass and passed in.
    // Natively that painted the same, but Chrome's shader compiler then painted some texels black or green, even with the pore layer off:
    // 3,268 texels on Bao-Dur and about 10 to 20 on many other heads.
    let du = dpdx(pt.position_mm);
    let dv = dpdy(pt.position_mm);
    let p = pt.position_mm;
    if (marks.present.x == 0u) {
        return c;
    }
    let skin = marks_skin(p);
    if (skin == 0.0) {
        return c;
    }
    // The texel's size is the mean of its two sides, so that a stretched texel does not fade a whole triangle of the layout.
    var pixel = sqrt(length(du) * length(dv));
    var n = safe_normalize(cross(du, dv));
    if (pixel < 1e-5) {
        pixel = PORE_PIXEL_MM;
        n = safe_normalize(p - frame.origin.xyz);
    }
    let eyes_middle = 0.25 * (marks.eyes[0].xyz + marks.eyes[1].xyz + marks.eyes[2].xyz + marks.eyes[3].xyz);
    let face_down = safe_normalize(marks.chin.xyz - eyes_middle);
    let forehead_fade = 1.0 - smooth_step(FOREHEAD_FADE_MM.x, FOREHEAD_FADE_MM.y, -dot(p - eyes_middle, face_down));
    let k = skin * forehead_fade;
    if (k == 0.0) {
        return c;
    }
    let light = clamp(1.0 + pore_params.h.x * lit / GLINT_LIT_L, 0.0, GLINT_MOST);
    let cell = pore_params.g.y;
    var t = pore_flow(p);
    t = safe_normalize(t - dot(t, n) * n);
    let side = cross(n, t);
    let squeeze = mix(1.0, pore_params.b.z, GLINT_SQUEEZE);
    let seed = u64_add(vec2<u32>(pore_params.seeds.x, pore_params.seeds.y), vec2<u32>(0x51A7u, 0u));
    let net = glint_net(p, cell, seed, t, side, n, squeeze);
    // The distances to the wall with the second nearest point and with the third: both are small where three grains meet.
    let w1 = (net.d.y - net.d.x) / max(2.0 * length(net.q1 - net.q0), 1e-6);
    let w2 = (net.d.z - net.d.x) / max(2.0 * length(net.q2 - net.q0), 1e-6);
    let fade = smooth_step(1.5, 2.5, cell / pixel);
    let strength = 1.0 - pore_params.g.w * hash_unit(net.h0, 20u);
    // Each grain is a round speck about its point. The points lie above and below the skin, so the skin cuts each speck at its own size,
    // and the specks of points far off the skin are missing, which keeps the grains from tiling the skin like cobbles.
    let x = net.d.x / (GLINT_RADIUS * GLINT_RADIUS * cell * cell);
    let bump = strength * max(1.0 - x, 0.0) * max(1.0 - x, 0.0);
    let corner = 1.0 - smooth_step(0.0, max(GLINT_POINT * cell, 0.6 * pixel), max(w1, w2));
    var kept = 1.0;
    if (maps_on()) {
        // While the maps are on, the grains stand up in the normal map and the colour keeps only a share of them.
        kept = maps_params.pores.y;
        maps_height = maps_height + k * fade * maps_params.pores.z * (pore_params.g.x / 10.0) * (bump - corner);
    }
    let mean = GLINT_MEAN * (1.0 - 0.5 * pore_params.g.w);
    // Where the grains fade, the skin keeps its tone: the grains' light is spread evenly, and the skin under them darkened by as much.
    let g = k * light * kept * fade * bump;
    let g_mean = k * light * kept * mean;
    let d = k * light * kept * mix(GLINT_POINT_MEAN, corner, fade);
    var out = c;
    if (pore_params.g.z != 0.0) {
        let y0 = lightness_to_luminance(GLINT_POINT_L);
        out = out * (lightness_to_luminance(max(GLINT_POINT_L - pore_params.g.z * d, 0.0)) / y0);
    }
    if (pore_params.g.x != 0.0) {
        let unit = lightness_to_luminance(min(GLINT_REF_L + pore_params.g.x, 100.0)) - lightness_to_luminance(GLINT_REF_L);
        // The colour that comes in can hold a channel a little under 0 on very dark or saturated skin. Divided by a luminance near 0,
        // such a channel made the tint huge and drew green specks in the browser, so the tint is taken from the colour with its channels
        // at 0 or more, and it is bounded by the largest a colour of that luminance can hold.
        let positive = max(out, vec3<f32>(0.0));
        let y = luminance(positive);
        var tint = vec3<f32>(1.0);
        if (y > GLINT_DARKEST) {
            tint = min(positive / y, vec3<f32>(GLINT_TINT_MOST));
        }
        // The grains add light where they stand and the skin between them gives up as much, so that the skin's mean lightness, which
        // the tone's fit set, stays.
        let keep = max(1.0 - unit * g_mean / max(y, GLINT_DARKEST), GLINT_FLOOR);
        out = clamp(out * keep + unit * (g + (1.0 - fade) * g_mean) * mix(vec3<f32>(1.0), tint, GLINT_TINT), vec3<f32>(0.0), vec3<f32>(1.0));
    }
    return out;
}
