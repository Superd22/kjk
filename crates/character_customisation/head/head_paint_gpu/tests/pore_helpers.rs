//! Checks the helpers of `shaders/pores.wgsl` on the GPU of this machine: the net's nearest grid points and their hashes against a Rust
//! twin of its hash, the pore map's regions against `pores::region_factors`, and the share of skin per corner of the net
//! (`CORNER_AREA`) against a count on a flat plane.

use head_paint::noise::unit;
use head_paint::pores::{PORE_BACKGROUND, PoreRegion, region_factors};
use head_paint_gpu::gpu::{Gpu, words_to_bytes};

const HARNESS: &str = r"
@group(0) @binding(20) var<storage, read> probe_in: array<vec4<f32>>;
@group(0) @binding(21) var<storage, read_write> probe_out: array<vec4<u32>>;

@compute @workgroup_size(64)
fn probe_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&probe_in)) {
        return;
    }
    let p = probe_in[i].xyz;
    let seed = vec2<u32>(i * 2654435761u, 3u);
    let t = safe_normalize(vec3<f32>(1.0, 0.3, 0.1));
    let n = safe_normalize(cross(t, vec3<f32>(0.0, 0.0, 1.0)) + vec3<f32>(0.0, 0.0, 0.2));
    let side = cross(n, t);
    let net = pore_net(p, probe_in[i].w, seed, t, side, n, 0.7);
    let f = pore_regions_at(p);
    probe_out[3u * i] = vec4<u32>(net.hs[0].x, net.hs[0].y, net.hs[1].x, net.hs[1].y);
    probe_out[3u * i + 1u] = vec4<u32>(bitcast<u32>(net.d2s[0]), bitcast<u32>(net.d2s[1]), bitcast<u32>(f.x), bitcast<u32>(f.y));
    probe_out[3u * i + 2u] = vec4<u32>(net.hs[2].x, net.hs[2].y, net.hs[3].x, net.hs[3].y);
}
";

fn normalize(a: [f64; 3]) -> [f64; 3] {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    a.map(|x| x / l)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The Rust twin of `net_hash`: pcg3d, with the two words joined low first.
fn net_hash(i: i64, j: i64, k: i64, seed: u64) -> u64 {
    let (lo, hi) = (seed as u32, (seed >> 32) as u32);
    let mut v = [i as i32 as u32 ^ lo, j as i32 as u32 ^ hi, k as i32 as u32 ^ lo ^ 0x9E37_79B9];
    for x in &mut v {
        *x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    }
    let mix = |v: &mut [u32; 3]| {
        v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[2]));
        v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
        v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    };
    mix(&mut v);
    for x in &mut v {
        *x ^= *x >> 16;
    }
    mix(&mut v);
    u64::from(v[0] ^ v[2]) | u64::from(v[1]) << 32
}

/// The Rust twin of `pore_net`: the four nearest grid points as (squeezed offset, squared distance, hash).
fn net(p: [f64; 3], cell: f64, seed: u64, frame: [[f64; 3]; 3], squeeze: f64) -> Vec<([f64; 3], f64, u64)> {
    let c = p.map(|x| (x / cell).floor() as i64);
    let mut all = Vec::with_capacity(27);
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let (i, j, k) = (c[0] + dx, c[1] + dy, c[2] + dz);
                let h = net_hash(i, j, k, seed);
                let q = [(i as f64 + unit(h, 0)) * cell, (j as f64 + unit(h, 16)) * cell, (k as f64 + unit(h, 32)) * cell];
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                let v = [squeeze * dot(d, frame[0]), dot(d, frame[1]), dot(d, frame[2])];
                all.push((v, dot(v, v), h));
            }
        }
    }
    all.sort_by(|a, b| a.1.total_cmp(&b.1));
    all.truncate(4);
    all
}

fn regions() -> Vec<PoreRegion> {
    vec![
        PoreRegion { a: [0.0, 20.0, 60.0], b: [0.0, -10.0, 75.0], reach_mm: 7.0, density: 1.0, size: 1.0 },
        PoreRegion { a: [2.0, 60.0, 55.0], b: [2.0, 60.0, 55.0], reach_mm: 20.0, density: 0.7, size: 0.7 },
        PoreRegion { a: [-30.0, 5.0, 50.0], b: [-30.0, 5.0, 50.0], reach_mm: 9.0, density: 0.8, size: 1.0 },
        PoreRegion { a: [-50.0, 20.0, 30.0], b: [-15.0, 18.0, 40.0], reach_mm: 7.0, density: 0.1, size: 0.4 },
    ]
}

#[test]
fn the_pore_shader_helpers_match_their_rust_twins() {
    let gpu = Gpu::blocking().expect("GPU");
    let count = 20_000usize;
    let mut inputs = Vec::with_capacity(4 * count);
    for i in 0..count {
        let x = i as f32;
        inputs.extend([(x * 0.731).sin() * 70.0, (x * 0.377).cos() * 60.0 + 10.0, (x * 0.113).sin() * 40.0 + 40.0, [0.45f32, 0.9, 0.3][i % 3]]);
    }
    let regions = regions();
    let mut storage: Vec<f32> = Vec::new();
    for r in &regions {
        storage.extend([r.a[0], r.a[1], r.a[2], r.reach_mm, r.b[0], r.b[1], r.b[2], 0.0, r.density, r.size, 0.0, 0.0].map(|v| v as f32));
    }
    let mut uniform = vec![0u32; 32];
    uniform[10] = (regions.len() as f32).to_bits();
    for (k, v) in PORE_BACKGROUND.iter().enumerate() {
        uniform[12 + k] = (*v as f32).to_bits();
    }

    let device = &gpu.device;
    let source = format!("{}\n{}\n{}\n{}", include_str!("../shaders/common.wgsl"), include_str!("../shaders/maps.wgsl"), include_str!("../shaders/pores.wgsl"), HARNESS);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(source.into()) });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &module, entry_point: Some("probe_main"), compilation_options: Default::default(), cache: None });
    let buffer = |bytes: &[u8], usage: wgpu::BufferUsages| {
        let b = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: bytes.len().max(16) as u64, usage: usage | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        gpu.queue.write_buffer(&b, 0, bytes);
        b
    };
    let input = buffer(&words_to_bytes(&inputs.iter().map(|v| v.to_bits()).collect::<Vec<_>>()), wgpu::BufferUsages::STORAGE);
    let params = buffer(&words_to_bytes(&uniform), wgpu::BufferUsages::UNIFORM);
    let data = buffer(&words_to_bytes(&storage.iter().map(|v| v.to_bits()).collect::<Vec<_>>()), wgpu::BufferUsages::STORAGE);
    let output = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 48 * count as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let layout = pipeline.get_bind_group_layout(0);
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 7, resource: params.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 8, resource: data.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 20, resource: input.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 21, resource: output.as_entire_binding() },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups((count as u32).div_ceil(64), 1, 1);
    }
    gpu.queue.submit([encoder.finish()]);
    let words: Vec<u32> = pollster::block_on(gpu.read(&output, 48 * count as u64)).chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect();

    let t = normalize([1.0, 0.3, 0.1]);
    let n0 = cross(t, [0.0, 0.0, 1.0]);
    let n = normalize([n0[0], n0[1], n0[2] + 0.2]);
    let frame = [t, cross(n, t), n];
    let (mut order_differs, mut hash_differs, mut distance_worst, mut region_worst) = (0, 0, 0.0f64, 0.0f64);
    for i in 0..count {
        let w = &words[12 * i..12 * i + 12];
        let p = [0, 1, 2].map(|k| f64::from(inputs[4 * i + k]));
        let cell = f64::from(inputs[4 * i + 3]);
        let seed = u64::from((i as u32).wrapping_mul(2_654_435_761)) | (3u64 << 32);
        let want = net(p, cell, seed, frame, 0.7);
        let got = [u64::from(w[0]) | u64::from(w[1]) << 32, u64::from(w[2]) | u64::from(w[3]) << 32, u64::from(w[8]) | u64::from(w[9]) << 32, u64::from(w[10]) | u64::from(w[11]) << 32];
        // Two points at almost the same distance may swap in f32, which is a rounding difference, not a different net.
        let mut sorted_want: Vec<u64> = want.iter().map(|x| x.2).collect();
        let mut sorted_got = got.to_vec();
        if got[0] != want[0].2 || got[1] != want[1].2 {
            order_differs += 1;
        }
        sorted_want.sort_unstable();
        sorted_got.sort_unstable();
        if sorted_want != sorted_got && (want[3].1 - want[2].1).abs() > 1e-4 {
            hash_differs += 1;
        }
        distance_worst = distance_worst.max((f64::from(f32::from_bits(w[4])) - want[0].1).abs()).max((f64::from(f32::from_bits(w[5])) - want[1].1).abs());
        let f = region_factors(&regions, p);
        region_worst = region_worst.max((f64::from(f32::from_bits(w[6])) - f[0]).abs()).max((f64::from(f32::from_bits(w[7])) - f[1]).abs());
    }
    println!("nearest two swapped {order_differs}, sets of four differing {hash_differs}, distance {distance_worst:.2e}, regions {region_worst:.2e}");
    assert!(order_differs < count / 1000, "the nearest points differ in {order_differs} of {count} points");
    assert_eq!(hash_differs, 0, "the four nearest points differ");
    assert!(distance_worst < 1e-4, "the squared distances differ by {distance_worst}");
    assert!(region_worst < 1e-5, "the region map differs by {region_worst}");
}

/// The skin area per corner of the net, as a share of the cell's square, counted on a flat plane: each corner is the point of the plane
/// as far from three grid points as the shader finds it, kept when no fourth of the four nearest is nearer.
#[test]
fn a_corner_of_the_net_covers_the_skin_area_the_shader_assumes() {
    let cell = 0.5;
    let side = 30.0;
    let frame = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut corners = std::collections::HashSet::new();
    let steps = (side / (0.05 * cell)) as usize;
    for a in 0..steps {
        for b in 0..steps {
            let p = [a as f64 * side / steps as f64, b as f64 * side / steps as f64, 0.37];
            let near = net(p, cell, 11, frame, 1.0);
            for skip in 0..4 {
                let tri: Vec<usize> = (0..4).filter(|&q| q != skip).collect();
                let (x, y, z) = (near[tri[0]].0, near[tri[1]].0, near[tri[2]].0);
                let m1 = [2.0 * (y[0] - x[0]), 2.0 * (y[1] - x[1])];
                let m2 = [2.0 * (z[0] - x[0]), 2.0 * (z[1] - x[1])];
                let det = m1[0] * m2[1] - m1[1] * m2[0];
                if det.abs() < 1e-9 {
                    continue;
                }
                let r1 = dot(y, y) - dot(x, x);
                let r2 = dot(z, z) - dot(x, x);
                let corner = [(r1 * m2[1] - r2 * m1[1]) / det, (m1[0] * r2 - m2[0] * r1) / det];
                let f = near[skip].0;
                let dist = |q: [f64; 3]| (corner[0] - q[0]).powi(2) + (corner[1] - q[1]).powi(2) + q[2] * q[2];
                if dist(f) < dist(x) {
                    continue;
                }
                let world = [p[0] + corner[0], p[1] + corner[1]];
                if world[0] < 0.0 || world[1] < 0.0 || world[0] >= side || world[1] >= side {
                    continue;
                }
                let mut key = [near[tri[0]].2, near[tri[1]].2, near[tri[2]].2];
                key.sort_unstable();
                corners.insert(key);
            }
        }
    }
    let area = side * side / corners.len() as f64 / (cell * cell);
    println!("{} corners on {side} by {side} mm: {area:.3} of a cell's square per corner", corners.len());
    assert!((area - 0.083).abs() < 0.01, "a corner covers {area} of a cell's square, and the shader assumes 0.083");
}

/// Prints the means of the cut cell pattern that `pore_furrow` takes away, at a squeeze of 1 and 0.6.
/// `cargo test --release -p head_paint_gpu --test pore_helpers net_means -- --ignored --nocapture`
#[test]
#[ignore]
fn net_means() {
    for squeeze in [1.0, 0.6] {
        let frame = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let (mut valley, mut top, mut near_wall, mut n) = (0.0, 0.0, 0.0, 0.0);
        let x = 0.02;
        for a in 0..1500 {
            for b in 0..1500 {
                let p = [a as f64 * 0.0137, b as f64 * 0.0137, 0.37];
                let m = net(p, 1.0, 5, frame, squeeze);
                let d = [m[1].0[0] - m[0].0[0], m[1].0[1] - m[0].0[1], m[1].0[2] - m[0].0[2]];
                let wall = (m[1].1 - m[0].1) / (2.0 * dot(d, d).sqrt());
                let sm = |a: f64, b: f64, v: f64| { let t = ((v - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
                let v = 1.0 - sm(0.0, 0.35, wall);
                let t = sm(0.1, 0.4, wall);
                valley += v * v;
                top += t * t;
                if wall < x { near_wall += 1.0; }
                n += 1.0;
            }
        }
        println!("squeeze {squeeze}: valley^2 {:.4}, top^2 {:.4}, wall density at 0 {:.4} per cell", valley / n, top / n, near_wall / n / x);
    }
}

/// Prints the means over a flat plane of the light grains' speck and of their dark points, which `pore_glint` puts in where the grains
/// fade, at the grains' squeeze of 1 and 0.8.
/// `cargo test --release -p head_paint_gpu --test pore_helpers glint_means -- --ignored --nocapture`
#[test]
#[ignore]
fn glint_means() {
    for squeeze in [1.0, 0.8] {
        let frame = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let (mut speck, mut point, mut n) = (0.0, 0.0, 0.0);
        let sm = |a: f64, b: f64, v: f64| { let t = ((v - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
        for a in 0..1500 {
            for b in 0..1500 {
                let p = [a as f64 * 0.0137, b as f64 * 0.0137, 0.37];
                let m = net(p, 1.0, 5, frame, squeeze);
                let wall = |k: usize| {
                    let d = [m[k].0[0] - m[0].0[0], m[k].0[1] - m[0].0[1], m[k].0[2] - m[0].0[2]];
                    (m[k].1 - m[0].1) / (2.0 * dot(d, d).sqrt())
                };
                let x = m[0].1 / (0.75 * 0.75);
                speck += (1.0 - x).max(0.0).powi(2);
                point += 1.0 - sm(0.0, 0.08, wall(1).max(wall(2)));
                n += 1.0;
            }
        }
        println!("squeeze {squeeze}: speck {:.4}, dark point {:.4}", speck / n, point / n);
    }
}
