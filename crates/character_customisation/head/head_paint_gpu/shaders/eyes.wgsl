// The eye paint (`head_paint::eyes`): the lash lines, the liner and its wing, the crease, the darker socket and under-eye, red lid rims and
// eye shadow. Every mask comes from the distance to each eye's lid edge, which the processor sends as two polylines per eye in
// millimetres (see `layers/eyes.rs`). It is applied last, on top of the other layers, while its own switch is on.

struct EyeLines {
    // The eyeball's centre in millimetres, and 1 in w when the eye is painted.
    centre: vec4<f32>,
    // The direction toward the temple, in the face's plane, with the eyeball's radius in millimetres in w, and the direction up.
    out: vec4<f32>,
    up: vec4<f32>,
    // The upper lid edge from the inner to the outer corner: points in millimetres, with the share along the lid in w.
    upper: array<vec4<f32>, 16>,
    // The lower lid edge, the same way.
    lower: array<vec4<f32>, 14>,
    // The liner's wing from the outer corner to its tip, seen from the front: points of the face's plane in millimetres from the
    // eyeball's centre, with the half width in w.
    wing: array<vec4<f32>, 8>,
}

struct EyesParams {
    // x: 1 when the layer paints; y: the upper lash amount; z: its width in mm; w: the lower lash amount.
    a: vec4<f32>,
    // x: the lower lash width in mm; y: the liner amount; z: the liner width in mm; w: how much the liner lines the lower lid.
    b: vec4<f32>,
    // x: the crease in L*; y: the crease height in mm; z: the socket in L*; w: the under-eye in L*.
    c: vec4<f32>,
    // x: the shadow amount; y: the shadow height in mm; z: the red rims amount; w: the wing's length in mm.
    d: vec4<f32>,
    lash_colour: vec4<f32>,
    liner_colour: vec4<f32>,
    shadow_colour: vec4<f32>,
    // x: how much lighter the lid skin is in L* (negative for darker); y: the depth of the crease under the bags in L*; z: how far below
    // the lower lid that crease runs, in mm. The natural log of the ratio of the redder skin of the rims to the skin, per channel, follows,
    // with the amount of red lids in w.
    lids: vec4<f32>,
    red_log: vec4<f32>,
    eyes: array<EyeLines, 2>,
}

// How far from the eyeball the skin counts as touching it, in millimetres.
const EYES_CONTACT_MM: f32 = 1.0;

@group(0) @binding(12) var<uniform> eyes_params: EyesParams;
@group(0) @binding(13) var<storage, read> eyes_unused: array<u32>;

// The nearest point of a lid's polyline: x the distance in mm, signed positive above the lid (in the face's plane), y the share along
// the lid from the inner corner, z the unsigned distance.
struct LidHit {
    signed_mm: f32,
    share: f32,
    distance: f32,
}

fn eyes_plane(k: u32, p: vec3<f32>) -> vec2<f32> {
    let r = p - eyes_params.eyes[k].centre.xyz;
    return vec2<f32>(dot(r, eyes_params.eyes[k].out.xyz), dot(r, eyes_params.eyes[k].up.xyz));
}

fn eyes_upper(k: u32, p: vec3<f32>) -> LidHit {
    var best = LidHit(0.0, 0.0, 1e9);
    let q = eyes_plane(k, p);
    for (var i = 0u; i < 15u; i = i + 1u) {
        let a = eyes_params.eyes[k].upper[i];
        let b = eyes_params.eyes[k].upper[i + 1u];
        let ab = b.xyz - a.xyz;
        let l2 = max(dot(ab, ab), 1e-6);
        let t = clamp(dot(p - a.xyz, ab) / l2, 0.0, 1.0);
        let c = a.xyz + t * ab;
        let d = length(p - c);
        if (d < best.distance) {
            let ta = eyes_plane(k, b.xyz) - eyes_plane(k, a.xyz);
            let off = q - eyes_plane(k, c);
            let side = select(-1.0, 1.0, ta.x * off.y - ta.y * off.x >= 0.0);
            best = LidHit(side * d, mix(a.w, b.w, t), d);
        }
    }
    return best;
}

// The same for the lower lid, signed positive below the lid.
fn eyes_lower(k: u32, p: vec3<f32>) -> LidHit {
    var best = LidHit(0.0, 0.0, 1e9);
    let q = eyes_plane(k, p);
    for (var i = 0u; i < 13u; i = i + 1u) {
        let a = eyes_params.eyes[k].lower[i];
        let b = eyes_params.eyes[k].lower[i + 1u];
        let ab = b.xyz - a.xyz;
        let l2 = max(dot(ab, ab), 1e-6);
        let t = clamp(dot(p - a.xyz, ab) / l2, 0.0, 1.0);
        let c = a.xyz + t * ab;
        let d = length(p - c);
        if (d < best.distance) {
            let ta = eyes_plane(k, b.xyz) - eyes_plane(k, a.xyz);
            let off = q - eyes_plane(k, c);
            let side = select(1.0, -1.0, ta.x * off.y - ta.y * off.x >= 0.0);
            best = LidHit(side * d, mix(a.w, b.w, t), d);
        }
    }
    return best;
}

// The lid's line seen in the face's plane, at the distance x toward the temple from the eyeball's centre. The result's x is the line's
// height there. Its y is the place along the eye, 0 at the inner corner and 1 at the outer corner, which goes on past the corners, where
// the line keeps the corner's height. Its z is how far the point lies past the nearer corner, or 0 between the corners.
fn eyes_profile(k: u32, x: f32, upper: bool) -> vec3<f32> {
    let n = select(14u, 16u, upper);
    var first: vec2<f32>;
    var last: vec2<f32>;
    if (upper) {
        first = eyes_plane(k, eyes_params.eyes[k].upper[0].xyz);
        last = eyes_plane(k, eyes_params.eyes[k].upper[15].xyz);
    } else {
        first = eyes_plane(k, eyes_params.eyes[k].lower[0].xyz);
        last = eyes_plane(k, eyes_params.eyes[k].lower[13].xyz);
    }
    let t = (x - first.x) / max(last.x - first.x, 1.0);
    if (x <= first.x) {
        return vec3<f32>(first.y, t, first.x - x);
    }
    if (x >= last.x) {
        return vec3<f32>(last.y, t, x - last.x);
    }
    var prev = first;
    for (var i = 1u; i < n; i = i + 1u) {
        var cur: vec2<f32>;
        if (upper) {
            cur = eyes_plane(k, eyes_params.eyes[k].upper[i].xyz);
        } else {
            cur = eyes_plane(k, eyes_params.eyes[k].lower[i].xyz);
        }
        if (cur.x > prev.x) {
            if (x <= cur.x) {
                let f = (x - prev.x) / (cur.x - prev.x);
                return vec3<f32>(mix(prev.y, cur.y, f), t, 0.0);
            }
            prev = cur;
        }
    }
    return vec3<f32>(last.y, t, 0.0);
}

// Whether a point lies inside the lid edge's loop, seen in the face's plane.
fn eyes_inside(k: u32, p: vec3<f32>) -> bool {
    let q = eyes_plane(k, p);
    var inside = false;
    for (var i = 0u; i < 28u; i = i + 1u) {
        var a: vec2<f32>;
        var b: vec2<f32>;
        if (i < 15u) {
            a = eyes_plane(k, eyes_params.eyes[k].upper[i].xyz);
            b = eyes_plane(k, eyes_params.eyes[k].upper[i + 1u].xyz);
        } else {
            let j = 28u - i;
            a = eyes_plane(k, eyes_params.eyes[k].lower[j].xyz);
            b = eyes_plane(k, eyes_params.eyes[k].lower[j - 1u].xyz);
        }
        if ((a.y > q.y) != (b.y > q.y)) {
            let x = a.x + (q.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if (q.x < x) {
                inside = !inside;
            }
        }
    }
    return inside;
}

// How much a point lies on the liner's wing, from 0 to 1. The wing is a line in the face's plane, so it is measured there.
fn eyes_wing(k: u32, p: vec3<f32>) -> f32 {
    let q = eyes_plane(k, p);
    var best = 0.0;
    for (var i = 0u; i < 7u; i = i + 1u) {
        let a = eyes_params.eyes[k].wing[i];
        let b = eyes_params.eyes[k].wing[i + 1u];
        let ab = b.xy - a.xy;
        let l2 = max(dot(ab, ab), 1e-6);
        let t = clamp(dot(q - a.xy, ab) / l2, 0.0, 1.0);
        let d = length(q - (a.xy + t * ab));
        let hw = mix(a.w, b.w, t);
        if (hw > 0.0) {
            let soft = 0.12 + 0.15 * hw;
            best = max(best, smooth_step(hw + soft, hw - soft, d));
        }
    }
    return best;
}

// A band from `inner` mm inside the lid edge to `width` mm outside it, with soft sides.
fn eyes_band(s: f32, inner: f32, width: f32) -> f32 {
    if (width <= 0.0) {
        return 0.0;
    }
    let soft = 0.15 + 0.3 * width;
    return smooth_step(width + soft, width - soft, s) * smooth_step(-inner - 0.4, -inner + 0.2, s);
}

struct EyeMasks {
    upper_lash: f32,
    lower_lash: f32,
    liner: f32,
    shadow: f32,
    rims: f32,
    // The red of the lid skin, from 0 to 1.
    lids: f32,
    // The darkening of the skin in L*, negative where the skin is lighter.
    dark: f32,
    // The share of `dark` that is the crease above the upper lid and the bags under the eye, which the maps turn into relief.
    relief: f32,
    // How wet the skin is: the rim of the lower lid and the pink corner of the eye, from 0 to 1.
    wet: f32,
}

fn eyes_masks(k: u32, p: vec3<f32>) -> EyeMasks {
    var m = EyeMasks(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    if (eyes_params.eyes[k].centre.w == 0.0 || length(p - eyes_params.eyes[k].centre.xyz) > 48.0) {
        return m;
    }
    let pa = eyes_params;
    let up = eyes_upper(k, p);
    let lo = eyes_lower(k, p);
    // Inside the opening, between the lids: the lining of the lids, which hangs in front of the eyeball near the edge.
    let inside = eyes_inside(k, p);
    let su = up.share;
    let sl = lo.share;

    // The broad parts use the height above the upper lid's line and below the lower lid's line, seen in the face's plane, which is smooth
    // everywhere, also past the corners. `t` runs from 0 at the inner corner to 1 at the outer corner.
    let q = eyes_plane(k, p);
    let pu = eyes_profile(k, q.x, true);
    let pl = eyes_profile(k, q.x, false);
    // Past a corner the space between the two lines closes at about 70 degrees: a point there counts as higher above the upper lid and
    // lower below the lower lid by 1.5 times its distance past the corner, so that the socket and the skin under the eye meet there.
    let hu = q.y - pu.x + 1.5 * pu.z;
    let hl = pl.x - q.y + 1.5 * pl.z;
    let t = pu.y;
    let near = 1.0 - smooth_step(22.0, 30.0, min(up.distance, lo.distance));


    // The upper lash line: thin at the inner corner, widest over the outer half, and over the lining inside the edge.
    let uw = pa.a.z * (0.35 + 0.65 * smooth_step(0.0, 0.55, su));
    // The lining inside the edge belongs to the lid on its side of the line halfway between the lids. It is painted whole, because the part of it a viewer sees is the strip that
    // hangs in front of the eyeball, and its width varies from head to head.
    let upper_side = q.y > 0.5 * (pu.x + pl.x);
    let reach_up = select(0.0, 8.0, inside && upper_side);
    let reach_lo = select(0.0, 8.0, inside && !upper_side);
    // Over the outer part of the band the lashes show as strands: the darkness breaks up along the lid, about 4 strands per millimetre.
    let strands = smooth_step(-0.35, 0.45, line_noise(su * 110.0, vec2<u32>(17u + k, 3u)));
    let tips = mix(1.0, strands, smooth_step(0.35 * uw, uw + 0.3, up.signed_mm));
    m.upper_lash = pa.a.y * eyes_band(up.signed_mm, reach_up, uw) * tips * select(1.0, 0.0, hl > 0.6 && !inside);
    // The skin that touches the eyeball is the edge a viewer sees, which can lie a few millimetres from the lid edge's vertices, most of
    // all at the outer corner. It takes the lash colour of the nearer lid.
    let g = length(p - eyes_params.eyes[k].centre.xyz) - eyes_params.eyes[k].out.w;
    let near_edge = 1.0 - smooth_step(4.0, 7.0, min(up.distance, lo.distance));
    let contact = smooth_step(EYES_CONTACT_MM + 0.5, EYES_CONTACT_MM - 0.3, g) * near_edge;
    let upper_half = upper_side;
    // Only the skin next to each lid's line counts, because the lid skin further out also lies on the eyeball.
    let contact_up = contact * select(0.0, 1.0, upper_half) * smooth_step(2.0, 1.0, hu);
    let contact_lo = contact * select(1.0, 0.0, upper_half) * smooth_step(1.2, 0.5, hl) * (1.0 - smooth_step(0.95, 1.15, t));
    m.upper_lash = max(m.upper_lash, pa.a.y * contact_up);
    // Between the two lids' lines near the outer corner, the skin a viewer sees is the fold of the corner, which the lashes cover. Past the
    // corner the two lines close, so this fill ends in a point.
    let corner_fill = smooth_step(0.8, 0.95, t) * (1.0 - smooth_step(1.05, 1.3, t)) * smooth_step(pa.a.z + 1.2, pa.a.z * 0.5, hu) * smooth_step(0.6, -0.2, hl);
    m.upper_lash = max(m.upper_lash, 0.85 * pa.a.y * corner_fill);
    // The lower lash line: faint at the inner corner, widest toward the outer corner.
    let lw = pa.b.x * (0.25 + 0.75 * smooth_step(0.05, 0.75, sl));
    m.lower_lash = pa.a.w * eyes_band(lo.signed_mm, reach_lo, lw) * (0.35 + 0.65 * smooth_step(0.0, 0.5, sl)) * select(1.0, 0.0, hu > 0.6 && !inside);
    m.lower_lash = max(m.lower_lash, pa.a.w * contact_lo * (0.35 + 0.65 * smooth_step(0.0, 0.5, t)));

    // The liner: thicker toward the outer corner, with its wing, and a share of it along the lower lid.
    let liw = pa.b.z * (0.3 + 0.7 * smooth_step(0.1, 0.95, su));
    var liner = eyes_band(up.signed_mm, reach_up, liw) * select(1.0, 0.0, hl > 0.6 && !inside);
    liner = max(liner, eyes_wing(k, p));
    liner = max(liner, pa.b.w * eyes_band(lo.signed_mm, reach_lo, 0.9 * pa.b.z * (0.35 + 0.65 * smooth_step(0.1, 0.9, sl))) * select(1.0, 0.0, hu > 0.6 && !inside));
    m.liner = pa.b.y * liner;

    // Eye shadow over the upper lid, up to its height, higher toward the outer end.
    let sh = pa.d.y * (0.8 + 0.3 * smooth_step(0.2, 1.0, t));
    let shadow_ends = smooth_step(-0.15, 0.2, t) * (1.0 - smooth_step(1.05, 1.4, t));
    // It reaches down to the lashes a viewer sees, which lie below the lid edge's line where the lid hangs in front of the eyeball, and it
    // never reaches below the lower lid's line past the corners.
    m.shadow = pa.d.x * smooth_step(sh * 1.3 + 0.5, sh * 0.7, hu) * smooth_step(-3.5, -1.5, hu) * (1.0 - 0.4 * smooth_step(0.0, sh, hu)) * shadow_ends * smooth_step(1.0, -0.5, hl) * near;

    // Red rims: the lining and the first millimetre of skin round the lower lid, and less round the upper lid.
    // The inner corner's pink flesh, between the lids, counts with the rims.
    let inner = eyes_params.eyes[k].upper[0].xyz;
    let caruncle = select(0.0, 1.0, inside) * smooth_step(4.0, 1.5, length(p - inner));
    m.rims = pa.d.z * max(max(caruncle, contact_lo), max(eyes_band(lo.signed_mm, reach_lo, 0.7) * smooth_step(0.0, 0.25, sl), 0.4 * eyes_band(up.signed_mm, reach_up, 0.3)));
    m.wet = max(max(caruncle, contact_lo), eyes_band(lo.signed_mm, reach_lo, 0.7) * smooth_step(0.0, 0.25, sl));

    // Red lids: the skin of the upper lid up to the crease, and of the lower lid down to about 5 mm.
    // Red lids: the skin next to each lid's edge, up to 2.5 mm above the upper lid and 2 mm below the lower lid, so that it reads as a rim
    // of sore skin and not as a rash.
    let lid_ends = smooth_step(-0.3, 0.1, t) * (1.0 - smooth_step(1.0, 1.3, t)) * near;
    let lid_up = smooth_step(-0.5, 0.5, hu) * smooth_step(2.5, 0.8, hu);
    let lid_lo = smooth_step(-0.5, 0.5, hl) * smooth_step(2.0, 0.6, hl);
    m.lids = eyes_params.red_log.w * max(lid_up, lid_lo) * lid_ends;
    // The lid skin's own lightness: the upper lid up to the crease and the lower lid down to 4 mm. Old or tired eyes have pale lids.
    let lid_skin_up = smooth_step(-0.5, 0.5, hu) * smooth_step(pa.c.y + 2.0, pa.c.y * 0.3, hu);
    let lid_skin_lo = smooth_step(-0.5, 0.5, hl) * smooth_step(4.5, 2.5, hl);
    // Past the corners it fades out within 4 mm, so that it leaves no edge along the lines that close there.
    let past = 1.0 - smooth_step(0.0, 4.0, pu.z);
    let lid_light = eyes_params.lids.x * max(lid_skin_up, lid_skin_lo) * lid_ends * past;

    // The crease: a soft valley above the upper lid, fading at both ends.
    var dark = 0.0;
    let h = pa.c.y;
    let sigma = 0.8 + 0.18 * h;
    let crease_ends = smooth_step(-0.05, 0.3, t) * (1.0 - smooth_step(0.95, 1.3, t));
    let crease = pa.c.x * gauss(hu - h, sigma) * crease_ends;
    dark = dark + crease;
    // The socket under the brow: darkest next to the lid, fading out over the crease and a little above it.
    // It sits on the lid and the fold, and fades out a little above the crease, so that it does not read as a band under the brow.
    let reach = h + 1.0;
    let socket_ends = smooth_step(-0.4, 0.05, t) * (1.0 - smooth_step(1.0, 1.45, t));
    let fall = 1.0 - smooth_step(0.0, reach, hu);
    dark = dark + pa.c.z * fall * fall * smooth_step(-3.0, 1.0, hu) * socket_ends * smooth_step(1.0, -0.5, hl);
    // The skin under the eye: darkest about 3.5 mm below the lower lid, more on the inner half.
    let under_ends = smooth_step(-0.4, 0.05, t) * (1.0 - smooth_step(0.95, 1.4, t));
    let tone = gauss(hl - 3.5, 3.2) + 0.5 * (1.0 - smooth_step(0.0, 2.0, hl));
    dark = dark + pa.c.w * min(tone, 1.0) * smooth_step(-3.0, 1.0, hl) * (1.0 - 0.45 * smooth_step(0.3, 1.0, t)) * under_ends;
    // Bags: a soft crease under the puffy skin below the lower lid, at its set distance, with the puffy skin above it a little lighter.
    let bags = eyes_params.lids.y;
    let bag_mm = eyes_params.lids.z;
    var bag = 0.0;
    if (bags > 0.0) {
        let bag_ends = smooth_step(0.0, 0.25, t) * (1.0 - smooth_step(0.8, 1.05, t));
        let curve = bag_mm * (1.0 - 0.25 * pow(2.0 * t - 1.0, 2.0));
        bag = bags * (gauss(hl - curve, 0.9) - 0.35 * gauss(hl - 0.55 * curve, 0.3 * curve)) * bag_ends * past;
        dark = dark + bag;
    }
    dark = (dark - lid_light) * near;
    m.dark = dark;
    m.relief = (crease + bag) * near;
    return m;
}

fn eyes_apply(c_in: vec3<f32>, pt: Point) -> vec3<f32> {
    if (eyes_params.a.x == 0.0) {
        return c_in;
    }
    let p = pt.position_mm;
    let m0 = eyes_masks(0u, p);
    let m1 = eyes_masks(1u, p);
    var c = c_in;
    // The two eyes' masks never overlap, so the one further from zero is the one that applies.
    let first = abs(m0.dark) > abs(m1.dark);
    var dark = select(m1.dark, m0.dark, first);
    // While the light comes from the maps, the crease and the bags become relief, and the colour keeps only a share of their darker and
    // lighter parts. `dark` darkens, so its darker part is its positive part. The wet rims shine in the gloss mask.
    if (maps_on()) {
        let relief = select(m1.relief, m0.relief, first);
        dark = dark - relief + maps_params.kept.w * max(relief, 0.0) + maps_params.kept.z * min(relief, 0.0);
        maps_height = maps_height - maps_params.relief.x * maps_params.relief.w * relief;
        maps_gloss = max(maps_gloss, maps_params.gloss2.x * max(m0.wet, m1.wet));
    }
    // The darker skin is the skin's colour scaled down, as in a shadow, so that its chroma falls with its lightness. Lighter skin is scaled up.
    if (dark != 0.0) {
        let l = srgb_to_lab(encode3(c)).x;
        let y0 = lightness_to_luminance(l);
        if (y0 > 1e-6) {
            c = c * (lightness_to_luminance(clamp(l - dark, 0.0, 100.0)) / y0);
        }
        // A strong shadow is also greyer than the skin, as KOTOR paints it: up to 35% of its colour goes at 25 L* of darkening.
        if (dark > 0.0) {
            let grey = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
            c = mix(c, vec3<f32>(grey), 0.35 * clamp(dark / 25.0, 0.0, 1.0));
        }
    }
    let rims = max(max(m0.rims, m1.rims), 0.3 * max(m0.lids, m1.lids));
    if (rims > 0.0) {
        c = c * exp(eyes_params.red_log.xyz * rims);
    }
    c = mix(c, eyes_params.shadow_colour.xyz, clamp(max(m0.shadow, m1.shadow), 0.0, 1.0));
    c = mix(c, eyes_params.liner_colour.xyz, clamp(max(m0.liner, m1.liner), 0.0, 1.0));
    let lash = clamp(max(max(m0.upper_lash, m1.upper_lash), max(m0.lower_lash, m1.lower_lash)), 0.0, 1.0);
    c = mix(c, eyes_params.lash_colour.xyz, lash);
    if (maps_on()) {
        maps_gloss = maps_gloss * (1.0 - 0.8 * lash);
    }
    return c;
}
