//! The detail tile of the light from maps (`shaders/tile.wgsl`): one small square of skin with furrows and pores, as a tangent-space
//! normal map whose edges join, so that a viewer can repeat it over the face. It does not depend on the head, so it is made once and
//! every head shares it.

use crate::gpu::{Gpu, words_to_bytes};

const SHADER: &str = include_str!("../shaders/tile.wgsl");

/// The tile's side in mm. A face 24 cm tall holds it about 30 times.
pub const TILE_MM: f64 = 8.0;
/// The tile's size in texels: 0.016 mm per texel, so that a pore of 0.1 mm radius is 6 texels wide.
pub const TILE_PIXELS: u32 = 512;
/// The depth of the furrows and their half width, in mm. Real cheek furrows are 20 to 50 µm deep and 50 to 100 µm wide.
const FURROW_DEPTH_MM: f64 = 0.02;
const FURROW_HALF_MM: f64 = 0.025;
/// The depth of a pore in mm.
const PORE_DEPTH_MM: f64 = 0.05;
/// The cells per side of the primary furrows (0.44 mm apart), of the secondary ones (0.25 mm) and of the pores' grid (0.8 mm), and the
/// share of the pores' cells that hold a pore: about 78 pores per cm², the nose's density in the pore layer's map.
const CELLS: [f64; 3] = [18.0, 32.0, 10.0];
const PORE_SHARE: f64 = 0.5;

fn f(v: f64) -> u32 {
    (v as f32).to_bits()
}

/// Paints the tile at `size` texels per side, a multiple of 16, and gives its RGBA bytes: the normal's x and y, the height mapped to the
/// blue byte, and an opaque alpha. Row 0 is v = 0.
pub async fn detail_tile(gpu: &Gpu, size: u32) -> Vec<u8> {
    let device = &gpu.device;
    let compute = wgpu::ShaderStages::COMPUTE;
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("tile"),
        entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: compute, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: compute, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
        ],
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("tile"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("tile"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("tile"), layout: Some(&pipeline_layout), module: &module, entry_point: Some("main"), compilation_options: Default::default(), cache: None });
    let words = [size, 0, 0, 0, f(TILE_MM), f(FURROW_DEPTH_MM), f(FURROW_HALF_MM), f(PORE_DEPTH_MM), f(CELLS[0]), f(CELLS[1]), f(CELLS[2]), f(PORE_SHARE)];
    let uniform = device.create_buffer(&wgpu::BufferDescriptor { label: Some("tile"), size: 48, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    gpu.queue.write_buffer(&uniform, 0, &words_to_bytes(&words));
    let bytes = 4 * u64::from(size) * u64::from(size);
    let out = device.create_buffer(&wgpu::BufferDescriptor { label: Some("tile texels"), size: bytes, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("tile"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: out.as_entire_binding() }],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("tile") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("tile"), timestamp_writes: None });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(size.div_ceil(16), size.div_ceil(16), 1);
    }
    gpu.queue.submit([encoder.finish()]);
    gpu.read(&out, bytes).await
}
