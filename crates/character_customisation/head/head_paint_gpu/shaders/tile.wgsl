// The detail tile (`tile.rs`): one small square of skin, `TILE_MM` on a side, whose heights repeat at its edges, written as a
// tangent-space normal. A viewer repeats it over the face and adds it to the head's own normal map. It holds the net of fine furrows on
// two levels, as the pore layer draws it in colour, and pores on a share of a coarser grid. Row 0 is v = 0 and each row goes up in v.
// Every random choice comes from a whole-number hash, so the tile is the same on every GPU.

struct TileParams {
    // The tile's size in texels, then unused.
    size: vec4<u32>,
    // The tile's side in mm, the furrows' depth and half width in mm, the pores' depth in mm.
    a: vec4<f32>,
    // The cells per side of the primary furrows, of the secondary furrows and of the pores' grid, and the share of the pores' cells that
    // hold a pore.
    b: vec4<f32>,
}

@group(0) @binding(0) var<uniform> tile: TileParams;
@group(0) @binding(1) var<storage, read_write> tile_out: array<u32>;

fn tile_hash(x: u32) -> u32 {
    var h = x * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

fn tile_unit(h: u32) -> f32 {
    return f32(h & 0xFFFFFFu) / 16777216.0;
}

// The jittered point of a cell of a grid of `n` cells per side, in cells, with `level` choosing the grid's own random numbers.
fn tile_point(cell: vec2<i32>, n: i32, level: u32) -> vec2<f32> {
    let c = vec2<u32>(u32((cell.x % n + n) % n), u32((cell.y % n + n) % n));
    let h = tile_hash(c.x + 4099u * c.y + 16777619u * level);
    return vec2<f32>(cell) + 0.15 + 0.7 * vec2<f32>(tile_unit(h), tile_unit(tile_hash(h)));
}

// The distance in mm from `p` (in cells) to the nearest wall between two cells of a grid: half the difference of the two nearest points'
// distances, measured along the line between them.
fn tile_wall(p: vec2<f32>, n: i32, level: u32, cell_mm: f32) -> f32 {
    let base = vec2<i32>(floor(p));
    var d1 = 1e9;
    var d2 = 1e9;
    var q1 = vec2<f32>(0.0);
    var q2 = vec2<f32>(0.0);
    for (var j = -2; j <= 2; j = j + 1) {
        for (var i = -2; i <= 2; i = i + 1) {
            let q = tile_point(base + vec2<i32>(i, j), n, level);
            let d = length(p - q);
            if (d < d1) {
                d2 = d1;
                q2 = q1;
                d1 = d;
                q1 = q;
            } else if (d < d2) {
                d2 = d;
                q2 = q;
            }
        }
    }
    let m = 0.5 * (q1 + q2);
    let along = q2 - q1;
    return abs(dot(p - m, along / max(length(along), 1e-6))) * cell_mm;
}

// The skin's height in mm at a point of the tile given in mm.
fn tile_height(p_mm: vec2<f32>) -> f32 {
    let side = tile.a.x;
    let depth = tile.a.y;
    let half_w = tile.a.z;
    var h = 0.0;
    // The furrows: a groove along every wall, deeper on the primary level, with the fields between them a little domed.
    let n1 = i32(tile.b.x);
    let c1 = side / tile.b.x;
    let w1 = tile_wall(p_mm / c1, n1, 1u, c1);
    h = h - depth * exp(-(w1 * w1) / (half_w * half_w)) + 0.25 * depth * min(w1 / (0.5 * c1), 1.0);
    let n2 = i32(tile.b.y);
    let c2 = side / tile.b.y;
    let w2 = tile_wall(p_mm / c2, n2, 2u, c2);
    h = h - 0.5 * depth * exp(-(w2 * w2) / (0.6 * half_w * 0.6 * half_w));
    // The pores: round pits on a share of the cells of a coarser grid, each with its own radius and depth.
    let n3 = i32(tile.b.z);
    let c3 = side / tile.b.z;
    let p3 = p_mm / c3;
    let base = vec2<i32>(floor(p3));
    for (var j = -1; j <= 1; j = j + 1) {
        for (var i = -1; i <= 1; i = i + 1) {
            let cell = base + vec2<i32>(i, j);
            let wrapped = vec2<u32>(u32((cell.x % n3 + n3) % n3), u32((cell.y % n3 + n3) % n3));
            let hh = tile_hash(wrapped.x + 7919u * wrapped.y + 3u * 16777619u);
            if (tile_unit(hh) >= tile.b.w) {
                continue;
            }
            let at = tile_point(cell, n3, 3u);
            let r = (0.07 + 0.07 * tile_unit(tile_hash(hh ^ 0x51EDu))) ;
            let d = length(p3 - at) * c3 / r;
            if (d < 1.0) {
                let bowl = (1.0 - d * d) * (1.0 - d * d);
                h = h - tile.a.w * (0.6 + 0.4 * tile_unit(tile_hash(hh ^ 0xBEEFu))) * bowl;
            }
        }
    }
    return h;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let n = tile.size.x;
    if (id.x >= n || id.y >= n) {
        return;
    }
    let px = tile.a.x / f32(n);
    let p = (vec2<f32>(id.xy) + 0.5) * px;
    let dx = tile_height(p + vec2<f32>(px, 0.0)) - tile_height(p - vec2<f32>(px, 0.0));
    let dy = tile_height(p + vec2<f32>(0.0, px)) - tile_height(p - vec2<f32>(0.0, px));
    let normal = normalize(vec3<f32>(-dx / (2.0 * px), -dy / (2.0 * px), 1.0));
    let h = tile_height(p);
    let b = vec4<u32>(round(clamp(vec4<f32>(normal.xy * 0.5 + 0.5, 0.5 + h / (4.0 * tile.a.y), 1.0), vec4<f32>(0.0), vec4<f32>(1.0)) * 255.0));
    tile_out[id.y * n + id.x] = b.x | (b.y << 8u) | (b.z << 16u) | (b.w << 24u);
}
