//! The eyeball's texture, painted on the GPU (`head_paint::eyeball`, `shaders/eyeball.wgsl`).
//!
//! It is a target of its own, apart from the head's layout: the eyeballs are their own mesh, and their texture depends only on the
//! eyeball sliders, not on the head's shape. One compute pass fills 2N by N packed sRGB texels, the eye on the x < 0 side on the left.
//! The processor only turns the sliders' colours into linear sRGB.

use crate::gpu::{Gpu, words_to_bytes};
use head_paint::eyeball::{EYE_PIXELS, Eyeball, k};
use head_paint::eyes::lch_to_linear;

const SHADER: &str = include_str!("../shaders/eyeball.wgsl");

/// The colour of a red eye white at full redness, as lightness, hue and chroma.
const RED: [f64; 3] = [52.0, 28.0, 42.0];

pub struct EyeballPainter {
    pipeline: wgpu::ComputePipeline,
    uniform: wgpu::Buffer,
    texels: wgpu::Buffer,
    bind: wgpu::BindGroup,
    size: u32,
}

fn f(v: f64) -> u32 {
    (v as f32).to_bits()
}

/// The words of the shader's `Ball` uniform for these settings, at a square of `size` texels per eye, for eyeballs of `radii`
/// millimetres (the eye on the x < 0 side first, as `head_paint::eyes::eyeball_radii` gives them).
pub fn uniform_words(b: &Eyeball, centres: [[f64; 2]; 2], radii: [f64; 2], size: u32) -> Vec<u32> {
    let v = |i: usize| b.get(i);
    let iris = lch_to_linear([v(k::IRIS_LIGHTNESS), v(k::IRIS_HUE), v(k::IRIS_CHROMA)]);
    let odd = lch_to_linear([v(k::IRIS_LIGHTNESS), (v(k::IRIS_HUE) + v(k::ODD_HUE)).rem_euclid(360.0), v(k::IRIS_CHROMA)]);
    let inner = lch_to_linear([v(k::INNER_LIGHTNESS), v(k::INNER_HUE), v(k::INNER_CHROMA)]);
    let white = lch_to_linear([v(k::WHITE_LIGHTNESS), v(k::WHITE_HUE), v(k::WHITE_CHROMA)]);
    let red = lch_to_linear(RED);
    let mut w = Vec::with_capacity(32);
    let mut row = |c: [f64; 3], x: f64| w.extend([f(c[0]), f(c[1]), f(c[2]), f(x)]);
    // The texture's left half is the eye on the x > 0 side, the second of `centres` and `radii`.
    row(iris, b.iris_share(radii[1]));
    row(odd, v(k::PUPIL));
    row(inner, v(k::INNER));
    row(white, v(k::WHITE_SHADE));
    row(red, v(k::WHITE_RED));
    row([v(k::FIBRES), v(k::RING), v(k::RING_WIDTH)], v(k::INNER_WIDTH));
    row([v(k::CLOUDING), v(k::CATCHLIGHT), b.iris_share(radii[0])], 0.0);
    row([centres[1][0] / radii[1], centres[1][1] / radii[1], centres[0][0] / radii[0]], centres[0][1] / radii[0]);
    w.extend([b.pattern(), size, 0, 0]);
    w
}

impl EyeballPainter {
    /// A painter of the eyeballs' texture at `EYE_PIXELS` texels per eye.
    pub fn new(gpu: &Gpu) -> Self {
        Self::with_size(gpu, EYE_PIXELS)
    }

    pub fn with_size(gpu: &Gpu, size: u32) -> Self {
        let device = &gpu.device;
        let compute = wgpu::ShaderStages::COMPUTE;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("eyeball"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: compute, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: compute, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("eyeball"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("eyeball"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("eyeball"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor { label: Some("eyeball"), size: 144, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let texels = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("eyeball texels"),
            size: 4 * 2 * u64::from(size) * u64::from(size),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("eyeball"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: texels.as_entire_binding() }],
        });
        EyeballPainter { pipeline, uniform, texels, bind, size }
    }

    /// The texels per eye: the texture is twice this wide and this high.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Paints both eyeballs and submits the work, with each iris centred at `centres` (millimetres across and up from the eyeball's
    /// front, as `head_paint::eyeball::iris_centres` gives them), on eyeballs of `radii` millimetres (`head_paint::eyes::eyeball_radii`).
    /// The texture stays on the GPU, in `texels`, until it is read.
    pub fn paint(&self, gpu: &Gpu, ball: &Eyeball, centres: [[f64; 2]; 2], radii: [f64; 2]) {
        gpu.queue.write_buffer(&self.uniform, 0, &words_to_bytes(&uniform_words(ball, centres, radii, self.size)));
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("eyeball") });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("eyeball"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind, &[]);
            pass.dispatch_workgroups((2 * self.size).div_ceil(8), self.size.div_ceil(8), 1);
        }
        gpu.queue.submit([encoder.finish()]);
    }

    /// The last texture painted: one packed RGBA texel per pixel, rows from the top, red in the low byte.
    pub fn texels(&self) -> &wgpu::Buffer {
        &self.texels
    }

    /// Reads the last texture painted back to the processor, as RGBA bytes row by row from the top.
    pub async fn read_rgba(&self, gpu: &Gpu) -> Vec<u8> {
        gpu.read(&self.texels, 4 * 2 * u64::from(self.size) * u64::from(self.size)).await
    }
}
