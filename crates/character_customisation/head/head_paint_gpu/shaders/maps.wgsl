// Light from maps (`head_paint::maps`, `layers/maps.rs`): the second texture the paint pass writes while the maps are on. Its red and
// green bytes are a tangent-space normal along the texture's u and v, its blue byte the gloss mask and its alpha byte the mask of the
// repeated detail tile. The layers leave their share in the three private numbers below while they paint a pixel, and the paint pass
// turns the height into a normal from its change between neighbouring texels.

struct MapsParams {
    // x: 1 when the maps are written; y: the skin grain's strength from 0 to 1; z and w: unused.
    on: vec4<f32>,
    // The skin's gloss, the T-zone's, the sheen regions' and the lips'.
    gloss: vec4<f32>,
    // x: the wet lid rims' gloss; y: how much the grain takes from the skin's gloss; z: the share of the gloss kept under the brows.
    gloss2: vec4<f32>,
    // The shares kept in the colour: the painted sheen, the lips' painted shine, the light band and the dark line of a crease.
    kept: vec4<f32>,
    // x: the height in mm of 1 L* of a crease's painted darkness; then the wrinkles', the lips' and the eye's share of the relief.
    relief: vec4<f32>,
    // x: the share of a pore's painted light rim kept in the colour; y: the share of the light grains and their dark points kept;
    // z: the light grains' height in mm at 10 L*.
    pores: vec4<f32>,
}

@group(0) @binding(18) var<uniform> maps_params: MapsParams;

// The height of the skin in mm, its gloss from 0 to 1, and the detail tile's mask from 0 to 1, at the pixel being painted.
var<private> maps_height: f32;
var<private> maps_gloss: f32;
var<private> maps_tile: f32;

// The share of the creases' relief that the fine lines of the wrinkle layer get.
const MAPS_FINE_RELIEF: f32 = 0.3;

// The texel the maps hold where no layer wrote anything: a normal straight out of the skin, no gloss and no tile.
const MAPS_FLAT: u32 = 0x00008080u;

fn maps_on() -> bool {
    return maps_params.on.x != 0.0;
}

// A change of lightness in L* with its darker part scaled by the share of the dark line kept and its lighter part by the share of the
// light band kept.
fn maps_keep(shift: f32) -> f32 {
    return maps_params.kept.w * min(shift, 0.0) + maps_params.kept.z * max(shift, 0.0);
}

// The skin's gloss at a point before the lips, the eyes and the brows change it: the whole skin's, more on the T-zone and the sheen
// regions, less where the grain is strong.
fn maps_skin_gloss(pt: Point) -> f32 {
    let g = maps_params.gloss;
    var t_zone = 0.0;
    if (marks.present.x != 0u) {
        t_zone = marks_t_zone(pt.position_mm);
    }
    let sheen = clamp(max(pt.sheen.x, max(pt.sheen.y, pt.sheen.z)), 0.0, 1.0);
    let dry = 1.0 - maps_params.gloss2.y * maps_params.on.y;
    return clamp((g.x + g.y * t_zone + g.z * sheen) * dry, 0.0, 1.0);
}

// The texel of the maps: the normal, the gloss and the tile's mask as bytes, red first.
fn maps_pack(n: vec3<f32>, gloss: f32, tile: f32) -> u32 {
    let b = vec4<u32>(round(clamp(vec4<f32>(n.xy * 0.5 + 0.5, gloss, tile), vec4<f32>(0.0), vec4<f32>(1.0)) * 255.0));
    return b.x | (b.y << 8u) | (b.z << 16u) | (b.w << 24u);
}
