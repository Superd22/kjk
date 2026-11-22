// The hair grid (`head_paint::hair_grid` and `head_paint::face_grid`): the shared primitive that draws single hairs from a grid of points
// on the skin, each holding the hair's density, length, direction, a hard edge and the look of its hairs. A layer puts a grid into its
// storage buffer `hair_data` (see `FaceGrid::words`) and passes the row where the grid's header starts.
//
// A texel finds its place on the grid, a column and a row as fractions, through the grid's lookup over the layout. The hairs are drawn
// on the skin in millimetres, through the skin's own lengths between neighbouring grid points at the texel, so they keep their size
// where the grid is stretched. Each hair's root comes from a cell of a finer lattice laid on the grid's columns and rows, in
// `HAIR_LAYERS` layers with shifted lattices, and whether a cell holds a hair, its root, length, lean and tone come from a whole-number
// hash of the cell. A cell keeps its hair with the chance that the density at the root gives for the cell's area on the skin at the
// root, so the density is in hairs per square millimetre wherever the grid stretches, and whether a hair exists does not depend on the
// pixel that draws it. A hair takes its length and its look (colour, thickness,
// greying and curl) from the grid point nearest its root. A hair's coverage of a pixel is its width boxed by the pixel's size, as for
// the brows.

// The finer lattice's cells, in grid steps, and the number of shifted lattices.
const HAIR_CELL: f32 = 0.34;
const HAIR_LAYERS: u32 = 3u;
// Grid coordinates below this in the lookup mark a cell the grid does not reach.
const HAIR_NO_CELL: f32 = -500.0;
// Two cells of the lookup agree when their places lie within this many columns and rows (`face_grid::AGREE_STEPS`).
const HAIR_AGREE: f32 = 6.0;

struct HairAt {
    // The pixel's place in grid steps, and 1 when the grid reaches it.
    c: vec2<f32>,
    on: f32,
    // The hairs per square millimetre, the typical length in mm, the edge distance in mm and the shadow, at the pixel.
    density: f32,
    length_mm: f32,
    edge_mm: f32,
    shadow: f32,
    // The largest density and the longest hair within reach of the longest hair, at least two grid steps, for skipping pixels no hair
    // can reach and for the reach of the search for roots.
    near: f32,
    long_mm: f32,
    // The skin's millimetres per texel of a 2,048-pixel texture.
    mm_per_texel: f32,
    // The look of the nearest grid point.
    look: u32,
    // The direction of growth along the skin, and the skin's millimetres per grid step along the columns and the rows.
    dir: vec3<f32>,
    // The pixel's place on the skin in millimetres, read from the grid unless the layer gives the texel's own place.
    pos: vec3<f32>,
    du: vec3<f32>,
    dv: vec3<f32>,
    // How many grid points past the bound ones the pixel lies, read bilinearly, or 0 on the bound points.
    depth: f32,
}

struct HairLook {
    grey: vec3<f32>,
    // The seed of the lattices' hashes, the size of a texel in mm, the L* of the skin under the hairs, against which dark hair is kept
    // dark, and the L* of the head's base skin tone, which says whether the head's skin is dark.
    seed: u32,
    fp: f32,
    skin_l: f32,
    base_l: f32,
    // How opaque the hairs are, as where the lower lip covers them.
    opacity: f32,
    // The most that any look's hairs follow the flow, which widens the search for hairs that bend into the pixel.
    flow_max: f32,
    // The widest painted strand of any look in mm, or 0 without strands.
    strand_max: f32,
    // The most that any look's strands are layered, or 0 when the hairs lie over each other in the order they are found.
    layer_max: f32,
    // The direction from which the light zones are lit, and the sign that turns the grid's own normal out of the skin.
    light: vec3<f32>,
    normal_sign: f32,
}

// A hair that follows the flow turns by at most this many radians from the direction at the pixel it covers.
const HAIR_FLOW_TURN: f32 = 0.6;

// The colour of the light a hair catches, in linear sRGB: a warm grey of about 39 L*. It shows on dark skin and hardly on pale skin.
const HAIR_GLINT: vec3<f32> = vec3<f32>(0.11, 0.1, 0.09);

fn hair_pcg(x: u32) -> u32 {
    let s = x * 747796405u + 2891336453u;
    let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}

fn hair_rand(h: u32, k: u32) -> f32 {
    return f32(hair_pcg(h + k * 0x9E3779B9u) >> 8u) / 16777216.0;
}

fn hair_box(v: f32, w: f32, fp: f32) -> f32 {
    let sigma = sqrt(w * w / 5.55 + 0.36 * fp * fp);
    return min(w / (sigma * 2.5066) * exp(-v * v / (2.0 * sigma * sigma)), 1.0);
}

fn hair_cols(g: u32) -> i32 {
    return i32(hair_data[g].x);
}

fn hair_rows(g: u32) -> i32 {
    return i32(hair_data[g].y);
}

// Row `k` of the point at column `i` and row `j`, clamped into the grid.
fn hair_row(g: u32, i: i32, j: i32, k: u32) -> vec4<f32> {
    let ii = clamp(i, 0, hair_cols(g) - 1);
    let jj = clamp(j, 0, hair_rows(g) - 1);
    return hair_data[u32(hair_data[g].z) + 4u * u32(jj * hair_cols(g) + ii) + k];
}

// The nine rows of look `k`: the linear colour and its L*, then the thickness in mm, the tone variation, the share of grey hairs and
// the curl, then how strongly each hair catches the light along its middle, the look's own random pattern, how far the hairs follow
// the flow and the locks' wave, then the locks' size in mm, how tightly their hairs gather, how much they catch the light and their
// softness, then the painted strands' width in mm, straightness, shading across their width and bands, then their light at the root,
// their layering, their packing and their shading by lock, then how parallel they run, their fine lines, their feathered edges and how
// far the hairs hang over the lips, then the clumps' width in mm, their wave, the light zones and the mixed edges, then the combed
// streaks, their length in mm and their sharpness.
fn hair_look_a(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k];
}

fn hair_look_b(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k + 1u];
}

fn hair_look_c(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k + 2u];
}

fn hair_look_d(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k + 3u];
}

fn hair_look_e(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k + 4u];
}

fn hair_look_f(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k + 5u];
}

fn hair_look_g(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k + 6u];
}

fn hair_look_h(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k + 7u];
}

fn hair_look_i(g: u32, k: u32) -> vec4<f32> {
    return hair_data[u32(hair_data[g + 1u].y) + 9u * k + 8u];
}

// Painted strands are this many times as many as the hairs' width alone gives, because a single hair's soft, thin stroke covers the
// skin more than its width says at the texture's resolution, and the strands narrow at their ends.
const HAIR_STRAND_FILL: f32 = 1.6;

// A grid carries its places this many points past the bound ones (`face_grid::RING`). Hairs reach further over the lips only as far as
// their look's "over the lips" allows, up to 2 points more.
const HAIR_RING: f32 = 3.0;

// The fine lines inside a strand lie this many millimetres apart, and each is about a third of that wide.
const HAIR_LINE_MM: f32 = 0.5;

// A step of the painted bands, as a ratio of lightness: each band is about 60% lighter than the one below it.
const HAIR_BAND_STEP: f32 = 0.47;

// The locks' lattice: its cells are `HAIR_LOCK_STEP_MM` millimetres of skin per grid step, the grid's spacing on the face.
const HAIR_LOCK_STEP_MM: f32 = 2.0;

// The grid's column and row in a cell of the lookup, clamped into it.
fn hair_cell(g: u32, i: i32, j: i32) -> vec2<f32> {
    let n = i32(hair_data[g + 1u].x);
    let k = clamp(j, 0, n - 1) * n + clamp(i, 0, n - 1);
    let r = hair_data[u32(hair_data[g].w) + u32(k) / 2u];
    return select(r.xy, r.zw, (k % 2) == 1);
}

// A texel's column and row on the grid from its place on the layout, read bilinearly from the lookup's cells that the grid reaches, and
// in z 1 when any of them does.
fn hair_lookup(g: u32, uv: vec2<f32>) -> vec3<f32> {
    let q = uv * hair_data[g + 1u].x - vec2<f32>(0.5);
    let i = i32(floor(q.x));
    let j = i32(floor(q.y));
    let f = q - floor(q);
    let a = hair_cell(g, i, j);
    let b = hair_cell(g, i + 1, j);
    let c = hair_cell(g, i, j + 1);
    let d = hair_cell(g, i + 1, j + 1);
    let wa = (1.0 - f.x) * (1.0 - f.y) * select(0.0, 1.0, a.x > HAIR_NO_CELL);
    let wb = f.x * (1.0 - f.y) * select(0.0, 1.0, b.x > HAIR_NO_CELL);
    let wc = (1.0 - f.x) * f.y * select(0.0, 1.0, c.x > HAIR_NO_CELL);
    let wd = f.x * f.y * select(0.0, 1.0, d.x > HAIR_NO_CELL);
    let total = wa + wb + wc + wd;
    if (total < 1e-4) {
        return vec3<f32>(0.0);
    }
    // Cells that hold places far apart on the grid, as from the two halves along the seam over the top and down the back, or where two
    // pieces of the layout touch, are not mixed. The texel takes the group of cells that agree, within 6 rows and 6 columns, with the
    // most weight, and mixes only those.
    var cq = array<vec2<f32>, 4>(a, b, c, d);
    var cw = array<f32, 4>(wa, wb, wc, wd);
    var pick = 0;
    var most = -1.0;
    for (var k = 0; k < 4; k = k + 1) {
        if (cw[k] <= 0.0) {
            continue;
        }
        var score = 0.0;
        for (var o = 0; o < 4; o = o + 1) {
            if (cw[o] > 0.0 && abs(cq[o].x - cq[k].x) <= HAIR_AGREE && abs(cq[o].y - cq[k].y) <= HAIR_AGREE) {
                score = score + cw[o];
            }
        }
        if (score > most) {
            most = score;
            pick = k;
        }
    }
    var sum = vec2<f32>(0.0);
    for (var o = 0; o < 4; o = o + 1) {
        if (cw[o] > 0.0 && abs(cq[o].x - cq[pick].x) <= HAIR_AGREE && abs(cq[o].y - cq[pick].y) <= HAIR_AGREE) {
            sum = sum + cq[o] * cw[o];
        }
    }
    return vec3<f32>(sum / most, 1.0);
}

fn hair_mix(a: vec4<f32>, b: vec4<f32>, c: vec4<f32>, d: vec4<f32>, f: vec2<f32>) -> vec4<f32> {
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// The grid read at a pixel whose place on the grid is `c`.
fn hair_at(g: u32, c_in: vec2<f32>) -> HairAt {
    var out: HairAt;
    out.on = 0.0;
    let cols = f32(hair_cols(g));
    let rows = f32(hair_rows(g));
    // A texel up to half a step past the grid's edge is read at the edge.
    if (c_in.x < -0.5 || c_in.y < -0.5 || c_in.x > cols - 0.5 || c_in.y > rows - 0.5) {
        out.c = c_in;
        return out;
    }
    let c = clamp(c_in, vec2<f32>(0.0), vec2<f32>(cols - 1.001, rows - 1.001));
    out.c = c;
    let i = i32(floor(c.x));
    let j = i32(floor(c.y));
    let f = c - vec2<f32>(f32(i), f32(j));
    let p00 = hair_row(g, i, j, 0u);
    let p10 = hair_row(g, i + 1, j, 0u);
    let p01 = hair_row(g, i, j + 1, 0u);
    let p11 = hair_row(g, i + 1, j + 1, 0u);
    let pos = hair_mix(p00, p10, p01, p11, f);
    let dir = hair_mix(hair_row(g, i, j, 1u), hair_row(g, i + 1, j, 1u), hair_row(g, i, j + 1, 1u), hair_row(g, i + 1, j + 1, 1u), f);
    let m00 = hair_row(g, i, j, 2u);
    let m10 = hair_row(g, i + 1, j, 2u);
    let m01 = hair_row(g, i, j + 1, 2u);
    let m11 = hair_row(g, i + 1, j + 1, 2u);
    let more = hair_mix(m00, m10, m01, m11, f);
    let l00 = hair_row(g, i, j, 3u);
    let l10 = hair_row(g, i + 1, j, 3u);
    let l01 = hair_row(g, i, j + 1, 3u);
    let l11 = hair_row(g, i + 1, j + 1, 3u);
    out.on = 1.0;
    out.density = pos.w;
    out.pos = pos.xyz;
    out.dir = dir.xyz;
    out.length_mm = dir.w;
    out.edge_mm = more.x;
    out.shadow = more.y;
    out.mm_per_texel = more.z;
    out.near = max(max(m00.w, m10.w), max(m01.w, m11.w));
    out.depth = mix(mix(l00.w, l10.w, f.x), mix(l01.w, l11.w, f.x), f.y);
    out.long_mm = max(max(l00.y, l10.y), max(l01.y, l11.y));
    out.look = u32(hair_row(g, i + select(0, 1, f.x > 0.5), j + select(0, 1, f.y > 0.5), 3u).x);
    out.du = mix(p10.xyz - p00.xyz, p11.xyz - p01.xyz, f.y);
    out.dv = mix(p01.xyz - p00.xyz, p11.xyz - p10.xyz, f.x);
    return out;
}

// The skin's millimetres per grid step along the columns and along the rows at `c`, from differences centred on each grid point, read
// bilinearly. Unlike the steps of `hair_at`, which are the sides of the pixel's own cell, they change smoothly from one cell to the
// next, so that a long painted strand measured in them runs on without a step where it crosses a cell's side.
fn hair_smooth_steps(g: u32, c: vec2<f32>) -> array<vec3<f32>, 2> {
    let i = i32(floor(c.x));
    let j = i32(floor(c.y));
    let f = clamp(c - vec2<f32>(f32(i), f32(j)), vec2<f32>(0.0), vec2<f32>(1.0));
    var du = vec3<f32>(0.0);
    var dv = vec3<f32>(0.0);
    for (var n = 0; n < 4; n = n + 1) {
        let a = i + (n % 2);
        let b = j + (n / 2);
        let w = select(1.0 - f.x, f.x, (n % 2) == 1) * select(1.0 - f.y, f.y, (n / 2) == 1);
        let ea = select(0.5, 1.0, a <= 0 || a >= hair_cols(g) - 1);
        let eb = select(0.5, 1.0, b <= 0 || b >= hair_rows(g) - 1);
        du = du + w * ea * (hair_row(g, a + 1, b, 0u).xyz - hair_row(g, a - 1, b, 0u).xyz);
        dv = dv + w * eb * (hair_row(g, a, b + 1, 0u).xyz - hair_row(g, a, b - 1, 0u).xyz);
    }
    return array<vec3<f32>, 2>(du, dv);
}

// The skin's area of one grid cell at a root, in square millimetres, read bilinearly.
fn hair_root_area(g: u32, c: vec2<f32>) -> f32 {
    let i = i32(floor(c.x));
    let j = i32(floor(c.y));
    let f = clamp(c - vec2<f32>(f32(i), f32(j)), vec2<f32>(0.0), vec2<f32>(1.0));
    return mix(mix(hair_row(g, i, j, 3u).z, hair_row(g, i + 1, j, 3u).z, f.x), mix(hair_row(g, i, j + 1, 3u).z, hair_row(g, i + 1, j + 1, 3u).z, f.x), f.y);
}

// The place on the skin in millimetres at `c`, read bilinearly.
fn hair_pos(g: u32, c: vec2<f32>) -> vec3<f32> {
    let i = i32(floor(c.x));
    let j = i32(floor(c.y));
    let f = clamp(c - vec2<f32>(f32(i), f32(j)), vec2<f32>(0.0), vec2<f32>(1.0));
    return hair_mix(hair_row(g, i, j, 0u), hair_row(g, i + 1, j, 0u), hair_row(g, i, j + 1, 0u), hair_row(g, i + 1, j + 1, 0u), f).xyz;
}

// The lock of a root, for a look whose locks are `lock_mm` wide: its middle in grid steps and its hash. The locks lie on a coarser
// lattice whose cells are as wide as a lock, every second row shifted by half a cell, and each lock's middle moves a little of its own.
struct HairLock {
    centre: vec2<f32>,
    h: u32,
}

fn hair_lock(root: vec2<f32>, lock_mm: f32, h0: u32) -> HairLock {
    var out: HairLock;
    let lock_cell = lock_mm / HAIR_LOCK_STEP_MM;
    let lq = root / lock_cell;
    let lrow = floor(lq.y);
    let lcol = floor(lq.x + 0.5 * (lrow - 2.0 * floor(lrow / 2.0)));
    out.h = hair_pcg(bitcast<u32>(i32(lcol)) ^ hair_pcg(bitcast<u32>(i32(lrow)) ^ (h0 + 0x51ED27u)));
    out.centre = (vec2<f32>(lcol + 0.5 - 0.5 * (lrow - 2.0 * floor(lrow / 2.0)), lrow + 0.5) + 0.5 * (vec2<f32>(hair_rand(out.h, 1u), hair_rand(out.h, 2u)) - 0.5)) * lock_cell;
    return out;
}

// The direction of growth at a root, read bilinearly.
fn hair_root_dir(g: u32, c: vec2<f32>) -> vec3<f32> {
    let i = i32(floor(c.x));
    let j = i32(floor(c.y));
    let f = clamp(c - vec2<f32>(f32(i), f32(j)), vec2<f32>(0.0), vec2<f32>(1.0));
    return hair_mix(hair_row(g, i, j, 1u), hair_row(g, i + 1, j, 1u), hair_row(g, i, j + 1, 1u), hair_row(g, i + 1, j + 1, 1u), f).xyz;
}

// At a root, in grid steps: the density and the edge distance read bilinearly, and the look and the length of the nearest point.
fn hair_root(g: u32, c: vec2<f32>) -> vec4<f32> {
    let i = i32(floor(c.x));
    let j = i32(floor(c.y));
    let f = clamp(c - vec2<f32>(f32(i), f32(j)), vec2<f32>(0.0), vec2<f32>(1.0));
    let d = mix(mix(hair_row(g, i, j, 0u).w, hair_row(g, i + 1, j, 0u).w, f.x), mix(hair_row(g, i, j + 1, 0u).w, hair_row(g, i + 1, j + 1, 0u).w, f.x), f.y);
    let e = mix(mix(hair_row(g, i, j, 2u).x, hair_row(g, i + 1, j, 2u).x, f.x), mix(hair_row(g, i, j + 1, 2u).x, hair_row(g, i + 1, j + 1, 2u).x, f.x), f.y);
    let ni = i + select(0, 1, f.x > 0.5);
    let nj = j + select(0, 1, f.y > 0.5);
    return vec4<f32>(d, e, hair_row(g, ni, nj, 3u).x, hair_row(g, ni, nj, 1u).w);
}

// The light and dark bands of shading by lock at a pixel, from 0 to 1 around 0.5: a sparse noise of soft streaks that run along the
// flow at the pixel, each as long as about two locks and a quarter of a lock wide, with their middles on a lattice about half a lock
// apart, so that the bands span several strands.
fn hair_bands(g: u32, at: HairAt, t: vec3<f32>, b: vec3<f32>, lock_mm: f32, seed: u32) -> f32 {
    let sa = lock_mm;
    let sc = 0.25 * lock_mm;
    let cell = 0.55 * lock_mm / HAIR_LOCK_STEP_MM;
    let q = at.c / cell;
    let base = vec2<i32>(floor(q));
    let h0 = hair_pcg(seed * 977u + g * 31u + 0xBA4Du);
    var sum = 0.0;
    var wsum = 0.0;
    for (var y = -4; y <= 4; y = y + 1) {
        for (var x = -4; x <= 4; x = x + 1) {
            let k = base + vec2<i32>(x, y);
            let hk = hair_pcg(bitcast<u32>(k.x) ^ hair_pcg(bitcast<u32>(k.y) ^ h0));
            let centre = (vec2<f32>(k) + vec2<f32>(hair_rand(hk, 1u), hair_rand(hk, 2u))) * cell;
            let dc = at.c - centre;
            let dm = at.du * dc.x + at.dv * dc.y;
            let a = dot(dm, t) / sa;
            let c = dot(dm, b) / sc;
            let w = exp(-0.5 * (a * a + c * c));
            sum = sum + w * (2.0 * hair_rand(hk, 3u) - 1.0);
            wsum = wsum + w * w;
        }
    }
    // The sum's spread grows with the square root of the kernels' summed squares, so the bands keep their contrast where the kernels
    // happen to lie sparse.
    return clamp(0.5 + 0.5 * sum / max(sqrt(wsum) * 1.2, 0.3), 0.0, 1.0);
}

// The light zones at `c`, from -1 in the darkest to 1 in the lightest: broad zones from the face's shape, measured over 3 grid steps,
// about 7 mm, to each side. Skin that faces the light is lighter, and skin in a hollow, whose neighbours lie in front of it, is darker.
fn hair_zone(g: u32, c: vec2<f32>, light: vec3<f32>, normal_sign: f32) -> f32 {
    let s = 3.0;
    let p = hair_pos(g, c);
    let xa = hair_pos(g, c + vec2<f32>(s, 0.0));
    let xb = hair_pos(g, c - vec2<f32>(s, 0.0));
    let ya = hair_pos(g, c + vec2<f32>(0.0, s));
    let yb = hair_pos(g, c - vec2<f32>(0.0, s));
    let du = xa - xb;
    let dv = ya - yb;
    let n = normal_sign * safe_normalize(cross(du, dv));
    let h2 = max(0.0625 * (dot(du, du) + dot(dv, dv)), 1.0);
    // The mean curvature in 1/mm, positive in a hollow.
    let bend = 2.0 * dot(0.25 * (xa + xb + ya + yb) - p, n) / h2;
    return tanh(HAIR_ZONE_FACING * (dot(n, light) - HAIR_ZONE_MID) - HAIR_ZONE_HOLLOW * max(bend, 0.0));
}

// How the light zones weigh the skin's facing, the facing that is neither light nor dark, and how much a hollow darkens, per 1/mm of
// curvature.
const HAIR_ZONE_FACING: f32 = 2.5;
const HAIR_ZONE_MID: f32 = 0.0;
const HAIR_ZONE_HOLLOW: f32 = 50.0;

// How many times lighter the light zones make the hair in a zone `z`, for a look with light zones `amount`: from 2 times darker
// to 1.5 times as light at 1.
fn hair_zone_light(z: f32, amount: f32) -> f32 {
    return exp(amount * select(0.405, 0.693, z < 0.0) * z);
}

// The combed streaks are traced from two noises whose cells are this many millimetres wide, so the broad streaks are about 0.9 mm
// wide across the flow and the fine ones inside them about half that. Each half of the trace takes this many steps.
const HAIR_STREAK_CELL: f32 = 0.9;
const HAIR_STREAK_FINE: f32 = 0.45;
const HAIR_STREAK_STEPS: i32 = 20;

// A smooth value noise from 0 to 1 at a place on the skin in mm, in cells `cell` mm wide.
fn hair_noise3(p: vec3<f32>, cell: f32, seed: u32) -> f32 {
    let q = p / cell;
    let i = vec3<i32>(floor(q));
    let f0 = q - floor(q);
    let f = f0 * f0 * (3.0 - 2.0 * f0);
    var v = 0.0;
    for (var n = 0; n < 8; n = n + 1) {
        let o = vec3<i32>(n & 1, (n >> 1u) & 1, (n >> 2u) & 1);
        let k = i + o;
        let h = hair_pcg(bitcast<u32>(k.x) ^ hair_pcg(bitcast<u32>(k.y) ^ hair_pcg(bitcast<u32>(k.z) ^ seed)));
        let w = select(1.0 - f.x, f.x, o.x == 1) * select(1.0 - f.y, f.y, o.y == 1) * select(1.0 - f.z, f.z, o.z == 1);
        v = v + w * hair_rand(h, 0u);
    }
    return v;
}

// The combed streaks at `c`, as a spread around 0 of about 1 either way: the noise averaged along the flow through the pixel, over
// `len_mm` in all, half before the pixel and half after it. The pixels along one flow line average nearly the same noise, so their
// light and dark run on unbroken along the flow, and pixels side by side across it average different noise.
fn hair_streaks(g: u32, c0: vec2<f32>, len_mm: f32, seed: u32) -> f32 {
    let h = 0.5 * len_mm / f32(HAIR_STREAK_STEPS);
    let s0 = hair_pcg(seed * 613u + g * 59u + 0x5713Au);
    let s1 = hair_pcg(s0 ^ 0x9E3779B9u);
    let p0 = hair_pos(g, c0);
    var sum = vec2<f32>(hair_noise3(p0, HAIR_STREAK_CELL, s0), hair_noise3(p0, HAIR_STREAK_FINE, s1));
    var wsum = 1.0;
    let top = vec2<f32>(f32(hair_cols(g) - 1), f32(hair_rows(g) - 1));
    for (var side = 0; side < 2; side = side + 1) {
        let sgn = select(1.0, -1.0, side == 1);
        var c = c0;
        for (var k = 1; k <= HAIR_STREAK_STEPS; k = k + 1) {
            let st = hair_smooth_steps(g, c);
            let du = st[0];
            let dv = st[1];
            let n = safe_normalize(cross(du, dv));
            let d = hair_root_dir(g, c);
            let t = safe_normalize(d - n * dot(n, d));
            let g11 = dot(du, du);
            let g12 = dot(du, dv);
            let g22 = dot(dv, dv);
            let det = max(g11 * g22 - g12 * g12, 1e-6);
            let rt = vec2<f32>(dot(du, t), dot(dv, t));
            c = clamp(c + sgn * h * vec2<f32>(g22 * rt.x - g12 * rt.y, g11 * rt.y - g12 * rt.x) / det, vec2<f32>(0.0), top);
            let x = f32(k) / f32(HAIR_STREAK_STEPS + 1);
            let w = 1.0 - x * x;
            let p = hair_pos(g, c);
            sum = sum + w * vec2<f32>(hair_noise3(p, HAIR_STREAK_CELL, s0), hair_noise3(p, HAIR_STREAK_FINE, s1));
            wsum = wsum + w;
        }
    }
    // Each average's spread, from a simulation of this noise, is about 0.227 times the square root of its cell over the length. The
    // broad and the fine streaks are mixed 3 to 2 in spread, so that the sum still spreads about 1 either way.
    let z = (sum / wsum - 0.5) / (0.227 * sqrt(vec2<f32>(HAIR_STREAK_CELL, HAIR_STREAK_FINE) / max(len_mm, 1.0)));
    return 0.83 * z.x + 0.55 * z.y;
}

// How many times lighter the combed streaks `z` make the hair, for a look with sharp streaks `sharp`. Soft streaks go from about 2.2
// times darker to 2.2 times as light at two spreads either way. Sharp streaks leave the hair 30% darker and light only the streaks more
// than about one spread above the middle, up to 2.4 times as light as that darker base.
fn hair_streak_light(z: f32, sharp: f32) -> f32 {
    let soft = exp(0.4 * clamp(z, -2.5, 2.5));
    let hard = 0.75 * (1.0 + 1.3 * smooth_step(0.5, 1.4, z));
    return mix(soft, hard, sharp);
}

// The colour of look `k` under packed strands, before it is darkened: the look's colour, kept no lighter than `dark_l` if it is dark.
fn hair_under(g: u32, k: u32, dark_l: f32) -> vec3<f32> {
    let la = hair_look_a(g, k);
    return la.xyz * mix(min(1.0, dark_l / max(luminance(la.xyz), 1e-6)), 1.0, smooth_step(40.0, 60.0, la.w));
}

// The hairs of grid `g` over a pixel read as `at`, laid onto the colour `col_in`.
fn hair_draw(col_in: vec3<f32>, g: u32, at: HairAt, look: HairLook) -> vec3<f32> {
    var col = col_in;
    if (at.on == 0.0 || at.near <= 0.0) {
        return col;
    }
    let n = safe_normalize(cross(at.du, at.dv));
    let t = safe_normalize(at.dir - n * dot(n, at.dir));
    let b = cross(n, t);
    // The grid steps that one millimetre along the hair crosses: [du dv] a = t, solved in the plane of the skin.
    let g11 = dot(at.du, at.du);
    let g12 = dot(at.du, at.dv);
    let g22 = dot(at.dv, at.dv);
    let det = max(g11 * g22 - g12 * g12, 1e-6);
    let rt = vec2<f32>(dot(at.du, t), dot(at.dv, t));
    let along = vec2<f32>(g22 * rt.x - g12 * rt.y, g11 * rt.y - g12 * rt.x) / det;
    let lmax = 1.4 * at.long_mm;
    let seg = along * lmax;
    // The pixel's own look bounds how wide and how curled the hairs that reach it are.
    let here_b = hair_look_b(g, at.look);
    let wmax = 1.3 * here_b.x;
    let spread = 0.85 + 3.2 * here_b.w;
    // A hair's width, its root dot and its curl reach about this far sideways, in grid steps along the columns and along the rows.
    // Hairs gathered into a lock move sideways by up to about half a lock toward its middle, and a painted strand is wider than a hair.
    // The reach is the largest of every look's, because the hairs that reach a pixel can come from a neighbouring group.
    let lock_reach = hair_data[g + 1u].w;
    // A hair that follows the flow reaches the pixel from the side, by up to half the tangent of its largest turn per millimetre along.
    let flow_side = 0.5 * tan(HAIR_FLOW_TURN) * look.flow_max;
    let reach_mm = wmax + 3.0 * look.fp + 0.15 * here_b.w * lmax + lock_reach + flow_side * lmax;
    let side = vec2<f32>(reach_mm / sqrt(max(g11, 1e-8)), reach_mm / sqrt(max(g22, 1e-8)));
    // A painted strand runs along the direction at its root, which can differ from the pixel's, so with strands the search reaches
    // forward as far as back.
    let back = at.c - seg;
    let ahead = select(at.c, at.c + seg, look.strand_max > 0.0);
    let lo = min(min(at.c, back), ahead) - side;
    let hi = max(max(at.c, back), ahead) + side;
    // On a head with dark skin, dark hair is kept at most 22% as light as the skin under it, so that it shows there. The limit fades out
    // as the head's base skin tone goes from 33 to 40 L*, and on a pale head dark hair keeps its own colour. The base tone is one value
    // for the whole head, so the shading of the skin does not make the hair darker in patches. Of the 57 test heads, `comm_b_m` has a
    // base tone of 31 L* and the pale bearded heads from 41 to 55 L*.
    let dark_l = lightness_to_luminance(mix(0.22 * look.skin_l, 100.0, smooth_step(33.0, 40.0, look.base_l)));
    // Shading by lock: the light and dark bands at the pixel, which the strands over it take instead of a shade of their own.
    let lf_here = hair_look_f(g, at.look);
    let ld_here = hair_look_d(g, at.look);
    var band = 0.5;
    if (lf_here.w > 0.0 && ld_here.x > 0.0) {
        band = hair_bands(g, at, t, b, ld_here.x, look.seed);
    }
    // The light zones, at the pixel, for every hair over it.
    let zones_here = hair_look_h(g, at.look).z;
    var zone = 0.0;
    if (zones_here > 0.0) {
        zone = hair_zone(g, at.c, look.light, look.normal_sign);
    }
    // The combed streaks at the pixel, which every combed strand over it takes instead of a shade of its own, so that no strand's start
    // or tip shows where the strands overlap.
    let li_here = hair_look_i(g, at.look);
    var streak = 1.0;
    if (li_here.x > 0.0) {
        streak = hair_streak_light(hair_streaks(g, at.c, li_here.y, look.seed), li_here.z);
    }
    // Packed strands: the skin under the beard is first painted over in the hair's colour, 25% darker and in the same bands, so that
    // no skin shows between the strands. It thins out with the hair, against the most hair within reach, at the beard's edges and at
    // bare patches.
    let packing = lf_here.z;
    if (packing > 0.0 && hair_look_e(g, at.look).x > here_b.x && at.depth <= HAIR_RING + 2.0 * hair_look_g(g, at.look).w) {
        var under = hair_under(g, at.look, dark_l);
        let tone_here = mix(0.5, band, lf_here.w);
        // Feathered, the under colour fades over the whole edge instead of its inner half, and it is mixed from the looks of the four
        // grid points round the pixel, so that it does not change in blocks where two groups of different colours meet.
        let feather_here = hair_look_g(g, at.look).z;
        var full = smooth_step(0.1, 0.6, at.density / max(at.near, 1e-4));
        if (feather_here > 0.0) {
            let i = i32(floor(at.c.x));
            let j = i32(floor(at.c.y));
            let f = at.c - vec2<f32>(f32(i), f32(j));
            let u00 = hair_under(g, u32(hair_row(g, i, j, 3u).x), dark_l);
            let u10 = hair_under(g, u32(hair_row(g, i + 1, j, 3u).x), dark_l);
            let u01 = hair_under(g, u32(hair_row(g, i, j + 1, 3u).x), dark_l);
            let u11 = hair_under(g, u32(hair_row(g, i + 1, j + 1, 3u).x), dark_l);
            under = mix(mix(u00, u10, f.x), mix(u01, u11, f.x), f.y);
            full = pow(smooth_step(0.1 - 0.1 * feather_here, 0.6 + 0.4 * feather_here, at.density / max(at.near, 1e-4)), 1.0 + feather_here);
        }
        if (li_here.x > 0.0) {
            under = under * mix(1.0, streak, li_here.x);
        }
        col = mix(col, 0.75 * under * exp(here_b.y * 1.6 * (tone_here - 0.5)) * hair_zone_light(zone, zones_here), packing * full * look.opacity);
    }
    // With layered strands, the hairs over the pixel are mixed by how far in front each lies, whatever order they are found in: a hair
    // 5 mm further in front counts about 7 times as much. `acc` sums the hairs' colours by their weight and coverage, and `clear` is the
    // share of the skin that no hair covers.
    var acc = vec4<f32>(0.0);
    var clear = 1.0;
    for (var l = 0u; l < HAIR_LAYERS; l = l + 1u) {
        let h0 = hair_pcg(look.seed * 131u + l * 7919u + g * 104729u + 5u);
        let phase = vec2<f32>(hair_rand(h0, 0u), hair_rand(h0, 1u));
        let c0 = vec2<i32>(floor(lo / HAIR_CELL + phase));
        let c1 = vec2<i32>(floor(hi / HAIR_CELL + phase));
        let nx = min(c1.x - c0.x, 64);
        let ny = min(c1.y - c0.y, 64);
        for (var cy = 0; cy <= ny; cy = cy + 1) {
            for (var cx = 0; cx <= nx; cx = cx + 1) {
                let cell = c0 + vec2<i32>(cx, cy);
                let hc = hair_pcg(bitcast<u32>(cell.x) ^ hair_pcg(bitcast<u32>(cell.y) ^ h0));
                let root = (vec2<f32>(cell) + vec2<f32>(hair_rand(hc, 1u), hair_rand(hc, 2u)) - phase) * HAIR_CELL;
                // A root past the grid's last column or row would repeat the hairs of the points on its edge.
                if (root.x < 0.0 || root.y < 0.0 || root.x > f32(hair_cols(g) - 1) || root.y > f32(hair_rows(g) - 1)) {
                    continue;
                }
                let dc = at.c - root;
                let dm = at.du * dc.x + at.dv * dc.y;
                var u = dot(dm, t);
                var v0 = dot(dm, b);
                var ht = t;
                var hb = b;
                let far = look.strand_max > 0.0 && dot(dm, dm) > (lmax + reach_mm) * (lmax + reach_mm);
                if (look.strand_max > 0.0) {
                    if (far) {
                        continue;
                    }
                } else if (u < -wmax || u > lmax || abs(v0) > wmax + 2.5 * look.fp + (spread + flow_side) * max(u, 0.0) + lock_reach) {
                    continue;
                }
                var r = hair_root(g, root);
                // Mixed edges: a root takes the look of the grid point nearest a place up to 2 grid steps, about 4 mm, to one side, and
                // a painted strand also takes its direction from there, so that two groups fray into each other where they meet.
                let mix_edges = hair_look_h(g, u32(r.z)).w;
                var jitter = vec2<f32>(0.0);
                if (mix_edges > 0.0) {
                    jitter = 2.0 * mix_edges * (vec2<f32>(hair_rand(hc, 20u), hair_rand(hc, 21u)) - 0.5) * 2.0;
                    let q = clamp(root + jitter, vec2<f32>(0.0), vec2<f32>(f32(hair_cols(g) - 1), f32(hair_rows(g) - 1)));
                    jitter = q - root;
                    r.z = hair_row(g, i32(round(q.x)), i32(round(q.y)), 3u).x;
                }
                // A look's own random pattern picks other hairs from the same roots, and leaves them as they are at 0.
                let lc = hair_look_c(g, u32(r.z));
                let h = select(hc, hair_pcg(hc ^ (u32(lc.y) * 0x85EBCA6Bu)), lc.y > 0.5);
                let keep = smooth_step(-0.1, 0.25, r.y);
                let cell_area = HAIR_CELL * HAIR_CELL * hair_root_area(g, root);
                let lb = hair_look_b(g, u32(r.z));
                // A painted strand wider than the hairs stands for several of them, so there are fewer strands and the beard covers
                // the skin about as much as with single hairs.
                let le = hair_look_e(g, u32(r.z));
                let lf = hair_look_f(g, u32(r.z));
                let lg = hair_look_g(g, u32(r.z));
                let strand = le.x > lb.x;
                // Combed, a strand takes its light and dark from the streaks at the pixel instead of from itself (`comb` of 1), and loses
                // the shading that would show where it starts and ends.
                let comb = select(0.0, hair_look_i(g, u32(r.z)).x, strand);
                if (at.depth > HAIR_RING + 2.0 * lg.w) {
                    continue;
                }
                // Packed strands are up to 3.5 times as many, so that they overlap.
                let fewer = select(1.0, min(HAIR_STRAND_FILL * lb.x / max(le.x, 1e-4), 1.0) * (1.0 + 2.5 * lf.z), strand);
                if (hair_rand(h, 0u) >= r.x * keep * cell_area / f32(HAIR_LAYERS) * fewer) {
                    continue;
                }
                let la = hair_look_a(g, u32(r.z));
                let lh = max(r.w, 0.05) * (0.6 + 0.8 * hair_rand(h, 3u));
                // A painted strand runs straight. As far as it follows the flow, its direction is the mean of the directions at its root
                // and 60% of the way along it, so that it turns with the flow lines without bending. It is measured from the root's own
                // place on the skin to the pixel's. A single hair is measured in the pixel's own direction and steps instead, which
                // change a little from cell to cell, and a long strand would show each change as a bend or a step sideways.
                // The strands of a lock all take the direction at the lock's middle, so that each lock runs one way. Parallel strands
                // take the direction at their own root instead, which changes smoothly from one strand to the next.
                if (strand) {
                    let ld_s = hair_look_d(g, u32(r.z));
                    let source = select(root, mix(hair_lock(root, ld_s.x, h0).centre, root, lg.x), ld_s.x > 0.0);
                    let dr = hair_root_dir(g, source + jitter);
                    let rt = safe_normalize(dr - n * dot(n, dr));
                    let rs = vec2<f32>(dot(at.du, rt), dot(at.dv, rt));
                    let further = source + jitter + 0.6 * lh * vec2<f32>(g22 * rs.x - g12 * rs.y, g11 * rs.y - g12 * rs.x) / det;
                    let dm_ = hair_root_dir(g, further);
                    let mt = safe_normalize(dm_ - n * dot(n, dm_));
                    ht = safe_normalize(rt + lc.z * mt);
                    hb = cross(n, ht);
                    let dm = at.pos - hair_pos(g, root);
                    u = dot(dm, ht);
                    v0 = dot(dm, hb);
                }
                if (u < -max(lb.x, le.x) || u > lh) {
                    continue;
                }
                // Straightness takes away the random lean, bend and wave of each hair and the random lean and bend of each lock.
                let crooked = 1.0 - le.y;
                let lean = 0.35 * (2.0 * hair_rand(h, 4u) - 1.0) * crooked;
                let bend = ((0.5 + 3.0 * lb.w) / max(lh, 0.3)) * (2.0 * hair_rand(h, 10u) - 1.0) * crooked;
                let uu = max(u, 0.0);
                let wave = lb.w * 0.12 * lh * sin(6.2832 * (uu / max(0.5 * lh, 0.3) + hair_rand(h, 12u))) * crooked;
                // A hair that follows the flow leaves its root along the direction there and passes the pixel along the pixel's
                // direction. Between the two it curves, and its line from the root to the pixel turns by half the angle between them,
                // as a streamline does to second order.
                var turn = 0.0;
                if (lc.z > 0.0 && !strand) {
                    let dr = hair_root_dir(g, root);
                    turn = 0.5 * lc.z * tan(clamp(atan2(dot(dr, b), max(dot(dr, t), 1e-3)), -HAIR_FLOW_TURN, HAIR_FLOW_TURN));
                }
                let v_own = select(v0 - lean * uu - bend * uu * uu - wave, v0 - lean * uu - bend * uu * uu - wave - turn * uu, lc.z > 0.0);
                var v_lock = v_own;
                // A lock: the hairs whose roots share a cell of a coarser lattice gather along their length toward a line through the
                // cell's middle, which leans a little of its own. `across` is the pixel's place across the lock, from its middle at 0 to
                // its edges at 1.
                let ld = hair_look_d(g, u32(r.z));
                var across = 0.0;
                var lock_h = 0u;
                // How far along the hair its lock's middle lies, in mm.
                var lock_u = 0.0;
                if (ld.x > 0.0) {
                    let lock = hair_lock(root, ld.x, h0);
                    lock_h = lock.h;
                    let centre = lock.centre;
                    let cd = centre - root;
                    let cm = at.du * cd.x + at.dv * cd.y;
                    let cu = dot(cm, ht);
                    lock_u = cu;
                    let cv = dot(cm, hb);
                    let lock_lean = 0.25 * (2.0 * hair_rand(lock_h, 3u) - 1.0) * crooked;
                    let lock_bend = (0.12 / max(lh, 0.3)) * (2.0 * hair_rand(lock_h, 8u) - 1.0) * crooked;
                    // The lock's middle line bends with the flow as its hairs do, and sways from side to side with the wave.
                    let sway = select(0.0, 1.2 * lc.w * sin(6.2832 * (uu / (8.0 + 6.0 * hair_rand(lock_h, 9u)) + hair_rand(lock_h, 10u))), lc.w > 0.0);
                    let line = cv + (lock_lean + turn) * (uu - cu) + lock_bend * uu * uu + sway;
                    let s = clamp(uu / max(0.55 * lh, 0.3), 0.0, 1.0);
                    let pull = ld.y * s * (2.0 - s) * (1.0 - lg.x);
                    let own = lean * uu + bend * uu * uu + wave + turn * uu;
                    v_lock = v0 - mix(own, line, pull);
                    let half = max(0.5 * ld.x * (0.7 + 0.6 * hair_rand(lock_h, 7u)) * (1.0 - 0.75 * pull), 0.2);
                    across = clamp(abs(v0 - line) / half, 0.0, 2.0);
                }
                var v = select(v_own, v_lock, ld.x > 0.0);
                // A clump: the strands whose roots share a cell of a lattice as wide as the clump are shaded as one piece and wave
                // together. `clump_v` is the pixel's place across the clump from its middle line, in mm.
                let lh_ = hair_look_h(g, u32(r.z));
                let clumped = strand && lh_.x > 0.0;
                var clump_v = 0.0;
                var clump_h = 0u;
                if (clumped) {
                    // The clumps' lattice is the same in every layer of roots, so that a clump holds strands of all of them.
                    let clump = hair_lock(root, lh_.x, hair_pcg(look.seed * 131u + g * 104729u + 0xC1D3A7u));
                    clump_h = clump.h;
                    let dmc = at.pos - hair_pos(g, clump.centre);
                    let uc = dot(dmc, ht);
                    // The wave is measured along the flow from the clump's middle, and each clump shifts it only a little of its own, so
                    // that neighbouring clumps wave nearly in step, as combed hair does.
                    let sway = 0.8 * lh_.y * sin(6.2832 * (uc / (10.0 + 2.0 * hair_rand(clump_h, 1u)) + 0.15 * hair_rand(clump_h, 2u)));
                    v = v - sway;
                    clump_v = dot(dmc, hb) - sway;
                }
                let w_hair = lb.x * (0.7 + 0.6 * hair_rand(h, 5u));
                let w = select(w_hair, le.x * (0.85 + 0.3 * hair_rand(h, 5u)), strand);
                // A painted strand is a brush stroke that narrows to a point at both ends, so that strands lying over each other blend
                // instead of ending in a step.
                let at_len = clamp(u / lh, 0.0, 1.0);
                let lens = pow(max(4.0 * at_len * (1.0 - at_len), 0.0), 0.2);
                let w_here = select(w * (1.0 - 0.55 * at_len), w * max(lens, 0.15), strand);
                if (abs(v) > w + 2.5 * look.fp) {
                    continue;
                }
                // The stroke along the hair, and the dark dot where it leaves the skin. A painted strand is a flat brush stroke with a
                // firm edge, and keeps the dot of a single hair.
                var cover = hair_box(v, w_here, look.fp);
                let half_w = 0.5 * w_here;
                let across_w = abs(v) / max(half_w, 1e-4);
                if (strand) {
                    cover = 1.0 - smooth_step(0.75 - 0.5 * look.fp / max(half_w, 1e-4), 1.0 + look.fp / max(half_w, 1e-4), across_w);
                }
                // A strand also fades out where the hair at the pixel thins out, as at a bare patch, the lips or the beard's edge, so
                // that it ends with the hair's own edge instead of lying over bare skin. A layered strand starts more sharply, because
                // its start is the lit edge that lies over the strands above it.
                if (strand) {
                    // Feathered, each strand fades out from a place of its own, and grows see-through over the whole edge.
                    let thin = at.density / max(r.x, 1e-4);
                    var fade = smooth_step(0.05, 0.5, thin);
                    if (lg.z > 0.0) {
                        let start = 0.05 + 0.5 * lg.z * hair_rand(h, 14u);
                        fade = smooth_step(start, start + 0.45 + 0.05 * lg.z, thin) * mix(1.0, smooth_step(0.0, 1.0, thin), lg.z);
                    }
                    cover = cover * smooth_step(0.0, mix(0.2, 0.08, lf.y), at_len) * (1.0 - smooth_step(0.7, 1.0, at_len)) * fade;
                } else {
                    cover = cover * smooth_step(-0.02, 0.12, u) * (1.0 - 0.6 * smooth_step(0.55 * lh, lh, u));
                }
                let dot_cover = hair_box(length(vec2<f32>(u, v)), 1.3 * w_hair, look.fp);
                cover = max(cover, 0.9 * dot_cover);
                // Dark hair is kept no lighter than `dark_l`. Light hair, blond or grey, keeps its own colour.
                let darker = min(1.0, dark_l / max(luminance(la.xyz), 1e-6));
                let colour = la.xyz * mix(darker, 1.0, smooth_step(40.0, 60.0, la.w));
                // Shaded by lock, a strand takes its tone from its lock's hash instead of its own.
                // A clump's strands share the clump's own shade.
                let own_tone = select(hair_rand(h, 6u), hair_rand(clump_h, 3u), clumped);
                var tone = select(own_tone, mix(own_tone, band, lf.w), ld.x > 0.0 && lf_here.w > 0.0);
                if (comb > 0.0) {
                    tone = mix(tone, 0.5, comb);
                }
                var hair = colour * exp(lb.y * 1.6 * (tone - 0.5));
                if (hair_rand(h, 7u) < lb.z) {
                    hair = look.grey * exp(0.3 * (hair_rand(h, 6u) - 0.5));
                }
                if (comb > 0.0) {
                    hair = hair * (1.0 + 0.3 * (1.0 - comb) * clamp(u / lh, 0.0, 1.0));
                } else {
                    hair = hair * (1.0 + 0.3 * clamp(u / lh, 0.0, 1.0));
                }
                // The per-strand shading below fades out as far as the strand is combed.
                let kept = 1.0 - comb;
                // A lock's highlight: the lock has a shade of its own, its middle catches the light over a stretch of its length, and
                // its edges are in the shadow of the locks beside it.
                let lock_light = select(ld.z, ld.z * kept, comb > 0.0);
                if (ld.x > 0.0 && lock_light > 0.0) {
                    let shade = exp(lock_light * 2.4 * (hair_rand(lock_h, 4u) - 0.5));
                    let stretch = exp(-pow((u / lh - (0.2 + 0.6 * hair_rand(lock_h, 5u))) / 0.25, 2.0));
                    // A soft lock spreads its light over more of its width and darkens only its outer edges.
                    let middle = exp(-across * across / (0.45 + 1.2 * ld.w)) * stretch * (0.3 + hair_rand(lock_h, 6u));
                    let edge = smooth_step(0.4 + 0.5 * ld.w, 1.1 + 0.5 * ld.w, across);
                    hair = hair * shade * (1.0 + 1.1 * lock_light * middle) * (1.0 - 0.9 * lock_light * edge);
                    hair = mix(hair, vec3<f32>(luminance(hair)), 0.6 * lock_light * clamp(middle, 0.0, 1.0));
                }
                // Light at the root: a strand is up to 1.6 times as light as its colour at its root and 2 times darker at its tip.
                let root_light = select(lf.x, lf.x * kept, comb > 0.0);
                if (root_light > 0.0) {
                    hair = hair * exp(root_light * 1.2 * (0.4 - at_len));
                }
                // Shading across a strand's width, as on a small cylinder: up to 1.7 times as light along its middle and 4 times darker
                // at its edges.
                // A clump is shaded across its whole width instead.
                let strand_shade = select(le.z, le.z * kept, comb > 0.0);
                if (strand_shade > 0.0) {
                    let across_s = select(across_w, abs(clump_v) / max(0.5 * lh_.x, 1e-4), clumped);
                    let round_off = sqrt(max(1.0 - min(across_s, 1.0) * min(across_s, 1.0), 0.0));
                    // A clump is shaded more softly than a single strand: up to 1.35 times as light along its middle and 2 times darker
                    // at its edges.
                    let shade_s = select(0.25 + 1.45 * round_off, 0.5 + 0.85 * round_off, clumped);
                    hair = hair * mix(1.0, shade_s, strand_shade);
                }
                if (comb > 0.0) {
                    hair = hair * mix(1.0, streak, comb);
                }
                // The painted bands: the strand's lightness against its colour falls onto a few steps, with short slopes between them.
                if (le.w > 0.0) {
                    let y = log(max(luminance(hair), 1e-6) / max(luminance(colour), 1e-6)) / HAIR_BAND_STEP;
                    let stepped = y - le.w * sin(6.2832 * y) / 6.2832;
                    hair = hair * exp((stepped - y) * HAIR_BAND_STEP);
                }
                // Fine lines: narrow light lines along the strand, `HAIR_LINE_MM` apart across it, each strand's at a place of its own. They come
                // after the bands, so that the bands' steps do not swallow them.
                let fine = select(lg.y, lg.y * kept, comb > 0.0);
                if (strand && fine > 0.0) {
                    // In a clump the lines run on across the whole clump.
                    let line_v = select(v / HAIR_LINE_MM + hair_rand(h, 15u), clump_v / HAIR_LINE_MM + hair_rand(clump_h, 4u), clumped);
                    let k = 0.5 + 0.5 * cos(6.2832 * line_v);
                    hair = hair * mix(1.0, 0.75 + 1.05 * k * k * k * k, fine);
                }
                if (lh_.z > 0.0) {
                    hair = hair * hair_zone_light(zone, lh_.z);
                }
                let glint = lc.x * (0.4 + 0.6 * hair_rand(h, 11u)) * exp(-pow((u / lh - 0.45) / 0.14, 2.0));
                hair = mix(hair, HAIR_GLINT, clamp(glint, 0.0, 1.0));
                let strength = mix(0.65 + 0.35 * hair_rand(h, 9u), 1.0, lf.z) * look.opacity;
                let shown = clamp(hair, vec3<f32>(0.0), vec3<f32>(1.0));
                let a = clamp(cover * strength, 0.0, 1.0);
                if (look.layer_max > 0.0) {
                    // A layered strand lies further in front the nearer its lock's middle is to the pixel, as the rows of a thatched
                    // roof, so the locks rooted further along the flow cover the tips of those above them, and the strands of one lock
                    // lie together. Without layering a hair's place in front is random.
                    let z = mix(lmax * hair_rand(h, 13u), lock_u - u, lf.y);
                    let wz = a * exp(clamp(z / 2.5, -30.0, 30.0));
                    acc = acc + vec4<f32>(shown * wz, wz);
                    clear = clear * (1.0 - a);
                } else {
                    col = mix(col, shown, a);
                }
            }
        }
    }
    if (look.layer_max > 0.0 && acc.w > 0.0) {
        col = mix(acc.xyz / acc.w, col, clear);
    }
    return col;
}
