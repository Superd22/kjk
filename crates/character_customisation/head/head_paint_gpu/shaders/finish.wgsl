// The finish pass: every texel takes the painted texel that `source` names, which is itself where a triangle covered it, the nearest
// painted texel up to `paint::SEAM_GROWTH` pixels past a patch's edge, or none, where the background colour goes. While the maps are on,
// their texels are copied the same way, with a flat normal and no gloss for the background. Their normal is made here, from the height
// of the source texel's neighbours on both sides along each axis: a central difference where both are painted, one side where only one is.

struct Finish {
    background: u32,
    resolution: u32,
    // 1 while the maps are on.
    maps: u32,
    pad1: u32,
}

@group(0) @binding(0) var<uniform> finish: Finish;
@group(0) @binding(1) var painted: texture_2d<u32>;
@group(0) @binding(2) var<storage, read> source: array<u32>;
@group(0) @binding(3) var<storage, read_write> texels: array<u32>;
@group(0) @binding(4) var maps_painted: texture_2d<u32>;
@group(0) @binding(5) var<storage, read_write> maps_texels: array<u32>;
@group(0) @binding(6) var heights: texture_2d<u32>;

fn painted_at(p: vec2<i32>) -> bool {
    let n = i32(finish.resolution);
    if (p.x < 0 || p.y < 0 || p.x >= n || p.y >= n) {
        return false;
    }
    return (textureLoad(painted, vec2<u32>(p), 0).x >> 24u) != 0u;
}

fn height_at(p: vec2<i32>) -> f32 {
    return bitcast<f32>(textureLoad(heights, vec2<u32>(p), 0).x);
}

// The change of height per texel along `step`, from the painted neighbours on either side.
fn height_change(p: vec2<i32>, step: vec2<i32>) -> f32 {
    let ahead = painted_at(p + step);
    let behind = painted_at(p - step);
    if (ahead && behind) {
        return 0.5 * (height_at(p + step) - height_at(p - step));
    }
    if (ahead) {
        return height_at(p + step) - height_at(p);
    }
    if (behind) {
        return height_at(p) - height_at(p - step);
    }
    return 0.0;
}

// The maps' texel of the painted texel `p`: its gloss and tile bytes, with the normal's two bytes made from the heights. Rows run down
// the texture while v runs up, so a height that grows down the rows tilts the normal toward +v.
fn maps_texel(p: vec2<i32>) -> u32 {
    let m = textureLoad(maps_painted, vec2<u32>(p), 0).x;
    let length_mm = unpack2x16float(textureLoad(heights, vec2<u32>(p), 0).y);
    if (length_mm.x <= 1e-6 || length_mm.y <= 1e-6) {
        return (m & 0xFFFF0000u) | 0x8080u;
    }
    let n = normalize(vec3<f32>(-height_change(p, vec2<i32>(1, 0)) / length_mm.x, height_change(p, vec2<i32>(0, 1)) / length_mm.y, 1.0));
    let b = vec2<u32>(round(clamp(n.xy * 0.5 + 0.5, vec2<f32>(0.0), vec2<f32>(1.0)) * 255.0));
    return (m & 0xFFFF0000u) | b.x | (b.y << 8u);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let n = finish.resolution;
    if (id.x >= n || id.y >= n) {
        return;
    }
    let at = id.y * n + id.x;
    let s = source[at];
    if (s == 0xFFFFFFFFu) {
        texels[at] = finish.background;
    } else {
        texels[at] = textureLoad(painted, vec2<u32>(s % n, s / n), 0).x;
    }
    if (finish.maps != 0u) {
        if (s == 0xFFFFFFFFu) {
            maps_texels[at] = 0x00008080u;
        } else {
            maps_texels[at] = maps_texel(vec2<i32>(i32(s % n), i32(s / n)));
        }
    }
}
