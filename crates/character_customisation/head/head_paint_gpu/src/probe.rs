//! The first proof of the GPU path: a compute shader fills a texture-sized buffer with a known pattern, and it is read back.

use crate::gpu::Gpu;

const SHADER: &str = include_str!("../shaders/probe.wgsl");

/// The pattern's RGBA bytes at a texel: x and y in their low byte, their exclusive or, and an opaque alpha.
pub fn expected(x: u32, y: u32) -> [u8; 4] {
    [(x & 255) as u8, (y & 255) as u8, ((x ^ y) & 255) as u8, 255]
}

/// Fills a `size` by `size` buffer on the GPU and reads it back as RGBA bytes, row by row.
pub async fn fill(gpu: &Gpu, size: u32) -> Vec<u8> {
    let device = &gpu.device;
    let bytes = 4 * u64::from(size) * u64::from(size);
    let out = device.create_buffer(&wgpu::BufferDescriptor { label: Some("probe"), size: bytes, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("probe"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("probe"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("probe"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: out.as_entire_binding() }],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("probe") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("probe"), timestamp_writes: None });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(size.div_ceil(16), size.div_ceil(16), 1);
    }
    gpu.queue.submit([encoder.finish()]);
    gpu.read(&out, bytes).await
}

/// How many texels of a filled buffer differ from the pattern.
pub fn mismatches(rgba: &[u8], size: u32) -> usize {
    rgba.chunks_exact(4).enumerate().filter(|(i, px)| *px != expected(*i as u32 % size, *i as u32 / size)).count()
}
