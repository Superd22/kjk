// The facial hair (`head_paint::facial_hair`): stubble and hair groups drawn from the face grid of the lower face with the shared
// drawing of `hair_grid.wgsl`. The processor writes the layer's settings in the first four rows of the storage buffer and the grid after
// them (see `layers/facial_hair.rs`). Per pixel this shader darkens the skin where the hair grows and gives it some of the hair's colour, as the
// hairs under a shaved skin and between the stubble show through it, then draws the hairs on top. The paint pass calls it after the lips, the eye paint and the brows.

@group(0) @binding(23) var<storage, read> hair_data: array<vec4<f32>>;

// The row where the facial hair's grid starts.
const BEARD_GRID: u32 = 4u;

// How much a point lies on the lower lip, from 0 to 1, in x, and 1 in y when it lies below the middle of the mouth's opening between the
// corners. The lip's shape is the lip layer's after its sliders, in the lips' frame: `u` round a cylinder of radius 50 mm, `v` up.
fn beard_lower_lip(p: vec3<f32>) -> vec2<f32> {
    let base = u32(hair_data[3].z);
    if (base == 0u) {
        return vec2<f32>(0.0);
    }
    let info = hair_data[base + 4u];
    let r = p - hair_data[base].xyz;
    let x = dot(r, hair_data[base + 1u].xyz);
    let z = dot(r, hair_data[base + 3u].xyz) + info.w;
    let w = sqrt(x * x + z * z) - info.w;
    if (w < -8.0 || w > 8.0) {
        return vec2<f32>(0.0);
    }
    let t = (info.w * atan2(x, z) - info.x) / info.y;
    let count = info.z;
    if (t < 0.0 || t > count - 1.001) {
        return vec2<f32>(0.0);
    }
    let i = u32(floor(t));
    let f = t - f32(i);
    let a = hair_data[base + 5u + i / 2u];
    let b = hair_data[base + 5u + (i + 1u) / 2u];
    let pa = select(a.xy, a.zw, i % 2u == 1u);
    let pb = select(b.xy, b.zw, (i + 1u) % 2u == 1u);
    let lower = mix(pa.x, pb.x, f);
    let opening = mix(pa.y, pb.y, f);
    if (opening <= lower) {
        return vec2<f32>(0.0);
    }
    let v = dot(r, hair_data[base + 2u].xyz);
    let below = select(0.0, 1.0, v < opening);
    return vec2<f32>(smooth_step(lower - 0.12, lower + 0.12, v) * smooth_step(opening + 0.1, opening - 0.1, v), below);
}

// The locks' highlight of the looks of the four grid points round `c`, mixed bilinearly, with 0 for a look without locks. A look whose
// painted strands are shaded across their width counts its shading instead, when that is larger, so that the skin between the strands
// lies in their shadow.
fn beard_lock_shadow(c_in: vec2<f32>) -> f32 {
    let c = clamp(c_in, vec2<f32>(0.0), vec2<f32>(f32(hair_cols(BEARD_GRID)) - 1.001, f32(hair_rows(BEARD_GRID)) - 1.001));
    let i = i32(floor(c.x));
    let j = i32(floor(c.y));
    let f = c - vec2<f32>(f32(i), f32(j));
    var k = vec4<f32>(0.0);
    for (var n = 0; n < 4; n = n + 1) {
        let look = u32(hair_row(BEARD_GRID, i + (n % 2), j + (n / 2), 3u).x);
        let d = hair_look_d(BEARD_GRID, look);
        k[n] = max(select(0.0, d.z, d.x > 0.0), hair_look_e(BEARD_GRID, look).z);
    }
    return mix(mix(k.x, k.y, f.x), mix(k.z, k.w, f.x), f.y);
}

// `pixel` is the texel's place in the texture, in texels from its top left corner, as the paint pass's fragment position gives it.
fn facial_hair_apply(c_in: vec3<f32>, pt: Point, pixel: vec2<f32>) -> vec3<f32> {
    let s0 = hair_data[0];
    if (s0.x == 0.0) {
        return c_in;
    }
    let res = f32(max(frame.resolution, 1u));
    let place = hair_lookup(BEARD_GRID, vec2<f32>(pixel.x / res, 1.0 - pixel.y / res));
    if (place.z == 0.0) {
        return c_in;
    }
    var at = hair_at(BEARD_GRID, place.xy);
    if (at.on == 0.0) {
        return c_in;
    }
    // When any group has painted strands, the hairs are measured in steps that change smoothly between cells and from the texel's own
    // place on the skin, so that long, wide strands do not bend or jump sideways at each cell's side. Beards without strands keep the
    // steps of their own cell.
    if (s0.z > 0.0) {
        let steps = hair_smooth_steps(BEARD_GRID, at.c);
        at.du = steps[0];
        at.dv = steps[1];
        at.pos = pt.position_mm;
    }
    let pixel_look = hair_look_a(BEARD_GRID, at.look);
    var c = c_in;
    let s2 = hair_data[2];
    let s3 = hair_data[3];
    // Everything is measured against the skin's own lightness here, so that the beard reads on dark skin as on pale skin. The shadow
    // keeps off the lips, as the hairs' roots do, and takes at most 45% of the skin's lightness.
    let skin_l = srgb_to_lab(encode3(c)).x;
    // Under the mouth the shadow reaches the lower lip's border, so that no pale band shows between the lip and the beard. The lip
    // itself covers it.
    let lip = beard_lower_lip(pt.position_mm);
    let off_lips = mix(smooth_step(-0.3, 0.8, at.edge_mm), 1.0, lip.y) * (1.0 - lip.x);
    var dark = min(s2.w * at.shadow * off_lips, 0.45 * skin_l);
    // Under hair in locks, or in shaded painted strands, the skin between them lies in their shadow, up to 65% of its lightness with the
    // full highlight or shading. It
    // grows with the beard's own shadow up to 12 L*, so that it fades out at the beard's soft edges. The
    // highlight is read from the looks of the four grid points round the pixel, so that it fades between groups instead of stepping.
    let lock_shadow = beard_lock_shadow(place.xy);
    if (lock_shadow > 0.0) {
        dark = max(dark, 0.65 * lock_shadow * skin_l * clamp(at.shadow / 12.0, 0.0, 1.0) * off_lips);
    }
    if (dark > 0.0) {
        let y0 = lightness_to_luminance(skin_l);
        if (y0 > 1e-6) {
            c = c * (lightness_to_luminance(skin_l - dark) / y0);
        }
        // The skin also takes some of the hair's colour, as a beard seen from a little way off does, but never lighter than the
        // darkened skin.
        let tint = pixel_look.xyz * 2.2 + vec3<f32>(0.004);
        let tint_y = max(luminance(tint), 1e-6);
        let tint_kept = tint * min(1.0, 0.9 * luminance(c) / tint_y);
        c = mix(c, tint_kept, clamp(0.02 * dark, 0.0, 0.4));
    }
    var look: HairLook;
    look.grey = s2.xyz;
    look.seed = u32(s3.x);
    look.skin_l = skin_l;
    look.base_l = hair_data[1].x;
    look.flow_max = s0.y;
    look.strand_max = s0.z;
    look.layer_max = s0.w;
    look.light = hair_data[1].yzw;
    look.normal_sign = s3.y;
    // The lower lip is drawn over the hairs, as a real lip covers the hair that reaches it.
    look.opacity = 1.0 - lip.x;
    look.fp = at.mm_per_texel * 2048.0 / res;
    return hair_draw(c, BEARD_GRID, at, look);
}
