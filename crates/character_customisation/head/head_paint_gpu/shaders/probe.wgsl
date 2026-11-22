@group(0) @binding(0) var<storage, read_write> out: array<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let size = groups.x * 16u;
    if (id.x >= size || id.y >= size) {
        return;
    }
    let r = id.x & 255u;
    let g = id.y & 255u;
    let b = (id.x ^ id.y) & 255u;
    out[id.y * size + id.x] = r | (g << 8u) | (b << 16u) | (255u << 24u);
}
