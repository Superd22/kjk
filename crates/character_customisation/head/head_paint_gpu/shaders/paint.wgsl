// The paint pass: the layout's triangles drawn into the texture, each vertex at its layout point, so that each fragment has its own
// point of the head. It puts the layers together in this order: the base colour, the shading, the sheen, then one
// change of lightness that is the sum of the wrinkles, the forehead lines and the pores. The texel is written as packed sRGB bytes with an opaque alpha;
// texels no triangle covers keep a zero alpha, and the finish pass fills them. While the maps of `maps.wgsl` are on, the second target
// holds their gloss and tile mask, and the third the height in mm that the layers leave, with the skin's length in mm of one texel along
// u and along v. The finish pass turns the height's change between neighbouring texels into the normal.

struct VertexIn {
    @location(0) uv: vec2<f32>,
    // The sky light, the bump in millimetres, and the first two sheen weights.
    @location(1) a: vec4<f32>,
    // The third sheen weight, and the point of the head in millimetres.
    @location(2) b: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) a: vec4<f32>,
    @location(1) b: vec4<f32>,
}

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip = vec4<f32>(v.uv.x * 2.0 - 1.0, v.uv.y * 2.0 - 1.0, 0.0, 1.0);
    out.a = v.a;
    out.b = v.b;
    return out;
}

struct PaintOut {
    @location(0) colour: u32,
    @location(1) maps: u32,
    // The height's bits, and the texel's length in mm along u and v as two 16-bit floats.
    @location(2) height: vec2<u32>,
}

@fragment
fn fs_main(v: VertexOut) -> PaintOut {
    let pt = Point(v.a.x, v.a.y, vec3<f32>(v.a.z, v.a.w, v.b.x), v.b.yzw + frame.origin.xyz);
    let maps = maps_on();
    maps_height = 0.0;
    maps_gloss = 0.0;
    maps_tile = 0.0;
    crater_gleam = 0.0;
    crater_gleam_mean = 0.0;
    crater_shade = 0.0;
    crater_gleam_light = 0.0;
    if (maps) {
        maps_gloss = maps_skin_gloss(pt);
    }
    // The colour under every layer. While the skin's colour zones act it changes from place to place, so the light the layers paint is
    // read against it and not against the frame's single base colour.
    var c = skin_base(pt);
    skin_tint = c;
    if (frame.on.x != 0u) {
        c = shading_apply(c, pt);
    }
    if (frame.on.y != 0u) {
        c = sheen_apply(c, pt);
    }
    // The colour with the light that the shading and the sheen painted, which the pore layer's light grains follow.
    let lit_colour = c;
    lips_cover = 0.0;
    c = lips_apply(c, pt);
    // The lips lie over every pore, crater and grain, so the pore layer paints only the share of the pixel that the lips leave.
    let open_skin = 1.0 - lips_cover;
    let height_before_pores = maps_height;
    let gloss_before_pores = maps_gloss;
    var shift = 0.0;
    if (frame.on.z != 0u) {
        if (maps) {
            // The fine lines are thin strokes 0.4 mm wide. At the creases' relief they stand out as scratches, so they take a smaller share.
            let parts = wrinkle_parts(pt);
            let w = parts.x + parts.y + lines_shift(pt);
            shift = shift + maps_keep(w);
            maps_height = maps_height + maps_params.relief.x * maps_params.relief.y * (w - (1.0 - MAPS_FINE_RELIEF) * parts.y);
        } else {
            shift = shift + wrinkle_shift(pt) + lines_shift(pt);
        }
    }
    if (frame.on.w != 0u) {
        shift = shift + open_skin * pore_shift(pt, skin_tint);
        crater_gleam = crater_gleam * open_skin;
        crater_gleam_mean = crater_gleam_mean * open_skin;
        crater_shade = crater_shade * open_skin;
        maps_tile = maps_tile * open_skin;
    }
    if (shift != 0.0) {
        c = skin_shift(c, shift, 0.0);
    }
    if (frame.on.w != 0u && pore_glint_on()) {
        c = mix(c, pore_glint(c, pt, srgb_to_lab(encode3(lit_colour)).x - srgb_to_lab(encode3(skin_tint)).x), open_skin);
    }
    if (frame.on.w != 0u && pore_gleam_on()) {
        c = pore_gleam(c, srgb_to_lab(encode3(lit_colour)).x - srgb_to_lab(encode3(skin_tint)).x);
    }
    maps_height = mix(height_before_pores, maps_height, open_skin);
    maps_gloss = mix(gloss_before_pores, maps_gloss, open_skin);
    c = eyes_apply(c, pt);
    c = brows_apply(c, pt);
    c = facial_hair_apply(c, pt, v.clip.xy);
    var out: PaintOut;
    out.colour = pack_srgb8(c);
    out.maps = MAPS_FLAT;
    out.height = vec2<u32>(0u);
    // The derivatives are taken while the control flow is the same for every pixel. They give the skin's length of one texel.
    let du = dpdx(pt.position_mm);
    let dv = dpdy(pt.position_mm);
    if (maps) {
        let lu = length(du);
        let lv = length(dv);
        var tile = 0.0;
        // A texel drawn as a single point has no derivatives: its length is left at zero, and it keeps a flat normal and no tile.
        if (lu > 1e-6 && lv > 1e-6) {
            tile = maps_tile;
        }
        out.maps = maps_pack(vec3<f32>(0.0, 0.0, 1.0), maps_gloss, tile);
        out.height = vec2<u32>(bitcast<u32>(maps_height), pack2x16float(vec2<f32>(lu, lv)));
    }
    return out;
}
