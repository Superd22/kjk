// The sheen layer (`sheen.rs`): lighter cheekbones, nose ridge and chin.

// The lightness each region adds at its centre, in L*, for the cheeks, the nose and the chin, then the share of the lightening that is white.
struct SheenParams {
    v: vec4<f32>,
}

@group(0) @binding(3) var<uniform> sheen_params: SheenParams;
@group(0) @binding(4) var<storage, read> sheen_data: array<u32>;

fn sheen_apply(c: vec3<f32>, pt: Point) -> vec3<f32> {
    let shift = sheen_params.v.x * pt.sheen.x + sheen_params.v.y * pt.sheen.y + sheen_params.v.z * pt.sheen.z;
    // While the light comes from the maps, the colour keeps only a share of the painted sheen, and the gloss mask shines there instead.
    return skin_shift(c, shift * select(1.0, maps_params.kept.x, maps_on()), sheen_params.v.w);
}
