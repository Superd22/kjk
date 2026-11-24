// The shading layer (`shading.rs`): darker in creases and lighter on bumps, from the sky light and the bump at a point.
// The lightness shift is the layer's strength times (SKY_GAIN * (sky - SKY_REFERENCE) + BUMP_GAIN * max(bump, 0)), and it is applied with
// `shift_lightness`.

struct ShadingParams {
    // The strength, the highlight whiteness, the sky gain in L* and the sky light at which the layer changes nothing.
    a: vec4<f32>,
    // The bump gain in L* per millimetre.
    b: vec4<f32>,
}

@group(0) @binding(1) var<uniform> shading_params: ShadingParams;
@group(0) @binding(2) var<storage, read> shading_data: array<u32>;

fn shading_apply(c: vec3<f32>, pt: Point) -> vec3<f32> {
    let pattern = shading_params.a.z * (pt.sky - shading_params.a.w) + shading_params.b.x * max(pt.bump_mm, 0.0);
    return skin_shift(c, shading_params.a.x * pattern + skin_front(pt), shading_params.a.y);
}
