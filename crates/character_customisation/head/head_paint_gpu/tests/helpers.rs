//! Checks the shared WGSL helpers of `shaders/common.wgsl` against the `head_paint` functions they mirror, on the GPU of this machine.

use head_paint::noise::{fractal_noise, gradient_noise, hash3};
use head_paint::shading::shift_lightness;
use head_paint::pores::PoreLandmarks;
use head_paint_gpu::gpu::{Gpu, words_to_bytes};
use head_paint_gpu::head::marks_words;

const HARNESS: &str = r"
@group(0) @binding(1) var<storage, read> inputs: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> outputs: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read_write> marks_out: array<vec4<f32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&inputs)) {
        return;
    }
    let p = inputs[i].xyz;
    let seed = vec2<u32>(i * 2654435761u, i % 7u);
    let c = vec3<i32>(floor(p * 3.0));
    let h = hash3(c.x, c.y, c.z, seed);
    let g = gradient_noise(p, seed);
    let f = fractal_noise(p, 0.7, 3u, 0.5, seed);
    let colour = vec3<f32>(fract(p.x * 0.13), fract(p.y * 0.07), fract(p.z * 0.11));
    let shifted = shift_lightness(colour, inputs[i].w, 0.3);
    let q = p * 0.6;
    marks_out[i] = vec4<f32>(marks_skin(q), marks_t_zone(q), dot(marks_flow(q), vec3<f32>(0.3, 0.5, 0.81)), marks_origin().y);
    outputs[2u * i] = vec4<u32>(h.x, h.y, bitcast<u32>(g), bitcast<u32>(f));
    outputs[2u * i + 1u] = vec4<u32>(bitcast<u32>(shifted.x), bitcast<u32>(shifted.y), bitcast<u32>(shifted.z), pack_srgb8(shifted));
}
";

fn fract(x: f64) -> f64 {
    x - x.floor()
}

#[test]
fn the_shared_helpers_match_head_paint() {
    let gpu = Gpu::blocking().expect("GPU");
    let count = 20_000usize;
    let mut inputs = Vec::with_capacity(4 * count);
    for i in 0..count {
        let t = i as f32;
        inputs.extend([(t * 0.731).sin() * 90.0, (t * 0.377).cos() * 120.0 - 10.0, (t * 0.113).sin() * 60.0 + 30.0, ((t * 0.5).sin() * 30.0).round() / 2.0]);
    }
    let device = &gpu.device;
    let source = format!("{}\n{}", include_str!("../shaders/common.wgsl"), HARNESS);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(source.into()) });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &module, entry_point: Some("main"), compilation_options: Default::default(), cache: None });
    let input = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 16 * count as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    gpu.queue.write_buffer(&input, 0, &words_to_bytes(&inputs.iter().map(|v| v.to_bits()).collect::<Vec<_>>()));
    let output = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 32 * count as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let marks = PoreLandmarks {
        nose: [[0.0, 20.0, 60.0], [0.0, -10.0, 75.0]],
        forehead: [2.0, 60.0, 55.0],
        chin: [0.0, -70.0, 50.0],
        mouth: [[-25.0, -35.0, 50.0], [24.0, -36.0, 51.0]],
        eyes: [[[-50.0, 20.0, 30.0], [-15.0, 18.0, 40.0]], [[51.0, 21.0, 30.0], [16.0, 19.0, 41.0]]],
    };
    let marks_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 176, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    gpu.queue.write_buffer(&marks_buffer, 0, &words_to_bytes(&marks_words(Some(&marks))));
    let marks_out = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 16 * count as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 1, resource: input.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: output.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: marks_out.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 9, resource: marks_buffer.as_entire_binding() },
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
    let bytes = pollster::block_on(gpu.read(&output, 32 * count as u64));
    let words: Vec<u32> = bytes.chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect();
    let marked: Vec<f32> = pollster::block_on(gpu.read(&marks_out, 16 * count as u64)).chunks_exact(4).map(|b| f32::from_le_bytes(b.try_into().unwrap())).collect();
    let mut marks_worst = 0.0f64;

    let (mut hash_bad, mut noise_worst, mut fractal_worst, mut colour_worst, mut byte_off) = (0, 0.0f64, 0.0f64, 0.0f64, 0);
    for i in 0..count {
        let w = &words[8 * i..8 * i + 8];
        let p = [f64::from(inputs[4 * i]), f64::from(inputs[4 * i + 1]), f64::from(inputs[4 * i + 2])];
        let seed = u64::from((i as u32).wrapping_mul(2_654_435_761)) | (((i % 7) as u64) << 32);
        let c = p.map(|x| (x * 3.0).floor() as i64);
        let h = hash3(c[0], c[1], c[2], seed);
        if (u64::from(w[0]) | (u64::from(w[1]) << 32)) != h {
            hash_bad += 1;
        }
        noise_worst = noise_worst.max((f64::from(f32::from_bits(w[2])) - gradient_noise(p, seed)).abs());
        fractal_worst = fractal_worst.max((f64::from(f32::from_bits(w[3])) - fractal_noise(p, 0.7, 3, 0.5, seed)).abs());
        let colour = [fract(p[0] * 0.13), fract(p[1] * 0.07), fract(p[2] * 0.11)];
        let expected = shift_lightness(colour, f64::from(inputs[4 * i + 3]), 0.3);
        for k in 0..3 {
            colour_worst = colour_worst.max((f64::from(f32::from_bits(w[4 + k])) - expected[k]).abs());
        }
        let q = [0, 1, 2].map(|a| f64::from(inputs[4 * i + a] * 0.6));
        let flow = marks.flow(q);
        let want = [marks.skin(q), marks.t_zone(q), 0.3 * flow[0] + 0.5 * flow[1] + 0.81 * flow[2], marks.origin()[1]];
        for k in 0..4 {
            marks_worst = marks_worst.max((f64::from(marked[4 * i + k]) - want[k]).abs());
        }
        let bytes = head_paint::paint::srgb8(expected);
        let got = w[7].to_le_bytes();
        if (0..3).any(|k| bytes[k].abs_diff(got[k]) > 1) {
            byte_off += 1;
        }
    }
    println!("hash mismatches {hash_bad}, gradient noise {noise_worst:.2e}, fractal noise {fractal_worst:.2e}, colour {colour_worst:.2e}, bytes off by more than 1: {byte_off}, landmarks {marks_worst:.2e}");
    assert!(marks_worst < 1e-4, "the landmarks differ: {marks_worst}");
    assert_eq!(hash_bad, 0, "hash3 differs from noise::hash3");
    assert!(noise_worst < 1e-4 && fractal_worst < 1e-4, "noise differs: {noise_worst} {fractal_worst}");
    assert!(colour_worst < 1e-4 && byte_off == 0, "shift_lightness differs: {colour_worst}, {byte_off} bytes");
}
