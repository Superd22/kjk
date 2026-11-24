//! The GPU painter: paints a recipe on a head into a texture of the head's layout.
//!
//! A repaint is two passes. The paint pass draws the layout's triangles into an integer texture, one packed sRGB texel per covered pixel,
//! with every layer computed per fragment (`shaders/paint.wgsl`). The finish pass then fills every texel of the output buffer: a covered
//! texel keeps its paint, a texel up to `SEAM_GROWTH` pixels past a patch's edge copies the nearest covered texel, and every other texel
//! gets the background colour (`shaders/finish.wgsl`). Which texel each texel copies depends only on the layout and the size, so it is
//! found once per size, on the processor. The texture keeps exactly the pixels of the processor's pixel map: the few pixels on a patch's
//! edge that the GPU's rasteriser leaves out are drawn as single points, and texels that only the GPU covers are treated as uncovered.
//!
//! The paint pass also draws a second target, the maps of `shaders/maps.wgsl`, and the finish pass copies it the same way into a second
//! output, while the maps are on. While they are off, the second output keeps what it held, and the colour is painted as without it.

use crate::gpu::{Gpu, words_to_bytes};
use crate::head::{PreparedHead, VERTEX_FLOATS};
use crate::layers::{self, GpuLayer, LayerInput};
use head_paint::live::Layout;
use head_paint::paint::{Layers, Recipe, SEAM_GROWTH, srgb8};
use head_paint::pigment::PigmentTable;
use head_paint::pixel_map::nearest_true;

const FINISH_SHADER: &str = include_str!("../shaders/finish.wgsl");
/// The binding of the shared landmarks in `common.wgsl`.
const MARKS_BINDING: u32 = 9;

/// The uniform binding of layer `k`; its storage buffer is the next binding. Binding 9 is the landmarks', so it is skipped.
fn layer_binding(k: u32) -> u32 {
    let b = 1 + 2 * k;
    if b >= MARKS_BINDING { b + 1 } else { b }
}

/// A source of `u32::MAX` in the finish pass's table means the background.
const BACKGROUND: u32 = u32::MAX;

/// Packs sRGB bytes as the shaders pack an output texel: red in the low byte, then green, blue and an opaque alpha.
pub fn pack(rgb: [u8; 3]) -> u32 {
    u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 255])
}

/// One texture size: its paint target, the finish pass's table and the output.
struct Target {
    resolution: u32,
    painted: wgpu::Texture,
    finish_uniform: wgpu::Buffer,
    /// Filled by `prepare`, and `None` before.
    finish_bind: Option<wgpu::BindGroup>,
    /// Which texels the GPU's rasteriser covers, filled by `prepare`.
    covered: Vec<bool>,
    /// The pixels of the processor's pixel map that the GPU's rasteriser leaves out: pixels on a patch's edge, which `PixelMap` keeps
    /// with a small tolerance. They are drawn as points, with their fields interpolated on the processor.
    extra: Vec<head_paint::pixel_map::Pixel>,
    /// The points of `extra` for one head, with the number of that head.
    extra_vertices: Option<(u64, wgpu::Buffer)>,
    texels: wgpu::Buffer,
    /// The maps' paint target and output, as `painted` and `texels` are the colour's.
    maps_painted: wgpu::Texture,
    maps_texels: wgpu::Buffer,
    /// The height of each texel and its length in mm along u and v, which the finish pass turns into the normal.
    heights: wgpu::Texture,
}

/// A layer's two buffers on the GPU, with their sizes in bytes.
struct LayerBuffers {
    uniform: wgpu::Buffer,
    uniform_size: u64,
    storage: wgpu::Buffer,
    storage_size: u64,
}

pub struct GpuPainter {
    gpu: Gpu,
    layout: Layout,
    table: PigmentTable,
    layers: Vec<Box<dyn GpuLayer>>,
    paint_pipeline: wgpu::RenderPipeline,
    /// The same pass drawn as single points, for the texels that `extra` lists.
    point_pipeline: wgpu::RenderPipeline,
    paint_bind_layout: wgpu::BindGroupLayout,
    finish_pipeline: wgpu::ComputePipeline,
    finish_bind_layout: wgpu::BindGroupLayout,
    indices: wgpu::Buffer,
    index_count: u32,
    vertices: Option<wgpu::Buffer>,
    head: Option<PreparedHead>,
    new_head: bool,
    /// Which vertices the eye paint reads the lids' edges from, on this head and the next ones.
    eye_edge: head_paint::eyes::LidEdge,
    /// Counts the heads set, so that a size knows when its extra points are out of date.
    head_number: u64,
    frame: wgpu::Buffer,
    /// The shared landmarks of `common.wgsl` (binding 9).
    marks: wgpu::Buffer,
    layer_buffers: Vec<LayerBuffers>,
    paint_bind: Option<wgpu::BindGroup>,
    targets: Vec<Target>,
    /// Values written straight onto the facial hair's grid, laid over the sliders' preset.
    hair_edit: Option<head_paint::hair_grid::GridEdit>,
    /// The facial hair's hair groups.
    hair_groups: head_paint::hair_groups::HairGroups,
}

fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }
}

fn storage_entry(binding: u32, visibility: wgpu::ShaderStages, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None }, count: None }
}

impl GpuPainter {
    /// A painter of one layout, drawing with `gpu`'s device and queue.
    pub fn new(gpu: Gpu, layout: Layout) -> Self {
        let device = &gpu.device;
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let all = layers::all();
        let layer_count = all.len() as u32;
        let mut entries = vec![uniform_entry(0, fragment), uniform_entry(MARKS_BINDING, fragment)];
        for (k, layer) in all.iter().enumerate() {
            if layer.has_uniform() {
                entries.push(uniform_entry(layer_binding(k as u32), fragment));
            }
            if layer.has_storage() {
                entries.push(storage_entry(layer_binding(k as u32) + 1, fragment, true));
            }
        }
        let paint_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("paint"), entries: &entries });
        let source: String = layers::PAINT_SHADER.join("\n");
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("paint"), source: wgpu::ShaderSource::Wgsl(source.into()) });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("paint"), bind_group_layouts: &[Some(&paint_bind_layout)], immediate_size: 0 });
        let attributes = [
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 8, shader_location: 1 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 24, shader_location: 2 },
        ];
        let pipeline = |topology: wgpu::PrimitiveTopology| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("paint"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout { array_stride: (4 * VERTEX_FLOATS) as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &attributes }],
            },
            primitive: wgpu::PrimitiveState { topology, cull_mode: None, ..Default::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[
                    Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::R32Uint, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                    Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::R32Uint, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                    Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rg32Uint, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                ],
            }),
            multiview_mask: None,
            cache: None,
        });
        let paint_pipeline = pipeline(wgpu::PrimitiveTopology::TriangleList);
        let point_pipeline = pipeline(wgpu::PrimitiveTopology::PointList);

        let compute = wgpu::ShaderStages::COMPUTE;
        let finish_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("finish"),
            entries: &[
                uniform_entry(0, compute),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: compute,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Uint, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                storage_entry(2, compute, true),
                storage_entry(3, compute, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: compute,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Uint, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                storage_entry(5, compute, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: compute,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Uint, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
            ],
        });
        let finish_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("finish"), source: wgpu::ShaderSource::Wgsl(FINISH_SHADER.into()) });
        let finish_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("finish"), bind_group_layouts: &[Some(&finish_bind_layout)], immediate_size: 0 });
        let finish_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("finish"),
            layout: Some(&finish_layout),
            module: &finish_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let flat: Vec<u32> = layout.triangles.iter().flatten().copied().collect();
        let indices = buffer_with(device, "layout triangles", &words_to_bytes(&flat), wgpu::BufferUsages::INDEX);
        let frame = device.create_buffer(&wgpu::BufferDescriptor { label: Some("frame"), size: 64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let marks = device.create_buffer(&wgpu::BufferDescriptor { label: Some("marks"), size: 176, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let layer_buffers = (0..layer_count).map(|_| LayerBuffers::new(device, 16, 16)).collect();
        GpuPainter {
            index_count: flat.len() as u32,
            gpu,
            layout,
            table: PigmentTable::build(),
            layers: layers::all(),
            paint_pipeline,
            point_pipeline,
            paint_bind_layout,
            finish_pipeline,
            finish_bind_layout,
            indices,
            vertices: None,
            head: None,
            new_head: false,
            eye_edge: Default::default(),
            head_number: 0,
            frame,
            marks,
            layer_buffers,
            paint_bind: None,
            targets: Vec::new(),
            hair_edit: None,
            hair_groups: Default::default(),
        }
    }

    pub fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    /// Puts a head under the paint. `positions` holds the head's mesh positions in metres.
    pub fn set_head(&mut self, positions: Vec<[f64; 3]>) {
        let mut head = PreparedHead::new(&self.layout, positions);
        head.eye_edge = self.eye_edge;
        self.vertices = Some(buffer_with(&self.gpu.device, "head vertices", &words_to_bytes(&head.vertices.iter().map(|v| v.to_bits()).collect::<Vec<_>>()), wgpu::BufferUsages::VERTEX));
        self.gpu.queue.write_buffer(&self.marks, 0, &words_to_bytes(&head.marks_words()));
        self.head = Some(head);
        self.new_head = true;
        self.head_number += 1;
    }

    /// Sets which vertices the eye paint reads the lids' edges from, for the head set and the next ones.
    pub fn set_eye_edge(&mut self, edge: head_paint::eyes::LidEdge) {
        self.eye_edge = edge;
        if let Some(h) = self.head.as_mut() {
            h.eye_edge = edge;
        }
    }

    pub fn eye_edge(&self) -> head_paint::eyes::LidEdge {
        self.eye_edge
    }

    /// Sets or clears the values written straight onto the facial hair's grid, which later paints lay over the sliders' preset.
    pub fn set_hair_edit(&mut self, edit: Option<head_paint::hair_grid::GridEdit>) {
        self.hair_edit = edit;
    }

    /// The facial hair's hair groups, which later paints write over the sliders' preset.
    pub fn hair_groups(&self) -> &head_paint::hair_groups::HairGroups {
        &self.hair_groups
    }

    pub fn hair_groups_mut(&mut self) -> &mut head_paint::hair_groups::HairGroups {
        &mut self.hair_groups
    }

    /// The values written straight onto the facial hair's grid, if any were set.
    pub fn hair_edit(&self) -> Option<&head_paint::hair_grid::GridEdit> {
        self.hair_edit.as_ref()
    }

    /// The prepared head, if one was set.
    pub fn head(&self) -> Option<&PreparedHead> {
        self.head.as_ref()
    }

    /// Makes the target of a size and finds which texels its finish pass copies. It paints once to learn which texels the GPU covers, so
    /// a head must be set. The size must be a multiple of 64.
    pub async fn prepare(&mut self, resolution: u32) {
        if let Some((gpu, buffer, size)) = self.begin_prepare(resolution) {
            let covered = gpu.read(&buffer, size).await;
            self.finish_prepare(resolution, &covered);
        }
    }

    /// Whether a size is ready to paint.
    pub fn is_prepared(&self, resolution: u32) -> bool {
        self.targets.iter().any(|t| t.resolution == resolution && t.finish_bind.is_some())
    }

    /// The first half of `prepare`: paints the coverage of a size and gives the buffer to read back, or `None` when the size is ready.
    /// The browser reads it without holding the painter.
    pub fn begin_prepare(&mut self, resolution: u32) -> Option<(Gpu, wgpu::Buffer, u64)> {
        if self.is_prepared(resolution) {
            return None;
        }
        assert!(resolution % 64 == 0 && resolution > 0, "the texture size must be a multiple of 64");
        if !self.targets.iter().any(|t| t.resolution == resolution) {
            let target = Target::new(&self.gpu.device, resolution);
            self.targets.push(target);
        }
        let off = Layers { base_tone: false, shading: false, sheen: false, wrinkles: false, pores: false, eyes: false, lips: false, brows: false, facial_hair: false, maps: false };
        let recipe = head_paint::controls::default_recipe();
        self.encode(resolution, &recipe, &off, false);
        let n = u64::from(resolution);
        let bytes = 4 * n * n;
        let copy = self.gpu.device.create_buffer(&wgpu::BufferDescriptor { label: Some("coverage"), size: bytes, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let target = self.targets.iter().find(|t| t.resolution == resolution).expect("target");
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("coverage") });
        encoder.copy_texture_to_buffer(
            target.painted.as_image_copy(),
            wgpu::TexelCopyBufferInfo { buffer: &copy, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * resolution), rows_per_image: Some(resolution) } },
            wgpu::Extent3d { width: resolution, height: resolution, depth_or_array_layers: 1 },
        );
        self.gpu.queue.submit([encoder.finish()]);
        Some((self.gpu.clone(), copy, bytes))
    }

    /// The second half of `prepare`, from the coverage texels read back.
    pub fn finish_prepare(&mut self, resolution: u32, covered: &[u8]) {
        let n = resolution as usize;
        let painted: Vec<bool> = covered.chunks_exact(4).map(|t| t[3] != 0).collect();
        // The texture keeps exactly the pixels of the processor's pixel map. Texels that only the GPU covers are treated as uncovered, and
        // pixels that only the processor covers are drawn as points.
        let mut on_map = vec![false; n * n];
        let mut extra = Vec::new();
        head_paint::live::rasterise(&self.layout.uv, &self.layout.triangles, n, |p| {
            let at = p.y as usize * n + p.x as usize;
            on_map[at] = true;
            if !painted[at] {
                extra.push(p);
            }
        });
        let source = finish_sources(&on_map, n);
        let source_buffer = buffer_with(&self.gpu.device, "finish sources", &words_to_bytes(&source), wgpu::BufferUsages::STORAGE);
        let target = self.targets.iter_mut().find(|t| t.resolution == resolution).expect("target");
        target.covered = painted;
        target.extra = extra;
        target.extra_vertices = None;
        let view = target.painted.create_view(&wgpu::TextureViewDescriptor::default());
        let maps_view = target.maps_painted.create_view(&wgpu::TextureViewDescriptor::default());
        let heights_view = target.heights.create_view(&wgpu::TextureViewDescriptor::default());
        target.finish_bind = Some(self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("finish"),
            layout: &self.finish_bind_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: target.finish_uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 2, resource: source_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: target.texels.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&maps_view) },
                wgpu::BindGroupEntry { binding: 5, resource: target.maps_texels.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&heights_view) },
            ],
        }));
    }

    /// Paints a recipe at a prepared size and submits the work. The texture stays on the GPU, in `texels`, until it is read.
    pub fn paint(&mut self, resolution: u32, recipe: &Recipe, layers: &Layers) {
        assert!(self.is_prepared(resolution), "the size {resolution} was not prepared");
        self.encode(resolution, recipe, layers, true);
    }

    /// Which texels of a prepared size the GPU's paint pass covers, row by row from the top.
    pub fn covered(&self, resolution: u32) -> &[bool] {
        &self.targets.iter().find(|t| t.resolution == resolution).expect("the size was not prepared").covered
    }

    /// The output of a size: one packed RGBA texel per pixel, rows from the top, red in the low byte. It can be copied into a texture of
    /// the format `Rgba8UnormSrgb`.
    pub fn texels(&self, resolution: u32) -> &wgpu::Buffer {
        &self.targets.iter().find(|t| t.resolution == resolution).expect("the size was not prepared").texels
    }

    /// The maps of a size, laid out as `texels`: per texel the normal's x and y, the gloss and the detail tile's mask, one byte each.
    /// They hold the last paint made while the maps were on.
    pub fn maps_texels(&self, resolution: u32) -> &wgpu::Buffer {
        &self.targets.iter().find(|t| t.resolution == resolution).expect("the size was not prepared").maps_texels
    }

    /// Reads the maps of the last paint made at a size while they were on, as RGBA bytes row by row from the top.
    pub async fn read_maps(&self, resolution: u32) -> Vec<u8> {
        let n = u64::from(resolution);
        self.gpu.read(self.maps_texels(resolution), 4 * n * n).await
    }

    /// Reads the last texture painted at a size back to the processor, as RGBA bytes row by row from the top.
    pub async fn read_rgba(&self, resolution: u32) -> Vec<u8> {
        let n = u64::from(resolution);
        self.gpu.read(self.texels(resolution), 4 * n * n).await
    }

    fn encode(&mut self, resolution: u32, recipe: &Recipe, layers: &Layers, finish: bool) {
        let head = self.head.as_ref().expect("no head was set");
        let device = &self.gpu.device;
        let queue = &self.gpu.queue;
        let base = recipe.base_colour(layers, &self.table);
        let background = pack(srgb8(base));
        let mut frame = Vec::with_capacity(16);
        frame.extend(base.map(|c| (c as f32).to_bits()));
        frame.push(0);
        frame.extend(head.origin_mm.map(|c| (c as f32).to_bits()));
        frame.push(0);
        frame.extend([layers.shading, layers.sheen, layers.wrinkles, layers.pores].map(u32::from));
        frame.extend([background, resolution, 0, 0]);
        queue.write_buffer(&self.frame, 0, &words_to_bytes(&frame));

        let input = LayerInput { recipe, layers, head, new_head: self.new_head, hair_edit: self.hair_edit.as_ref(), hair_groups: &self.hair_groups };
        let mut rebind = self.paint_bind.is_none();
        for (layer, buffers) in self.layers.iter_mut().zip(&mut self.layer_buffers) {
            let words = layer.words(&input);
            assert!(words.uniform.len() % 4 == 0 && !words.uniform.is_empty(), "a layer's uniform must be a whole number of 16-byte rows");
            rebind |= buffers.write(device, queue, &words.uniform, words.storage.as_deref());
        }
        self.new_head = false;
        if rebind {
            let mut entries = vec![wgpu::BindGroupEntry { binding: 0, resource: self.frame.as_entire_binding() }];
            for (k, (b, layer)) in self.layer_buffers.iter().zip(&self.layers).enumerate() {
                let binding = layer_binding(k as u32);
                if layer.has_uniform() {
                    entries.push(wgpu::BindGroupEntry { binding, resource: b.uniform.as_entire_binding() });
                }
                if layer.has_storage() {
                    entries.push(wgpu::BindGroupEntry { binding: binding + 1, resource: b.storage.as_entire_binding() });
                }
            }
            entries.push(wgpu::BindGroupEntry { binding: MARKS_BINDING, resource: self.marks.as_entire_binding() });
            self.paint_bind = Some(device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("paint"), layout: &self.paint_bind_layout, entries: &entries }));
        }

        let head_number = self.head_number;
        let target = self.targets.iter_mut().find(|t| t.resolution == resolution).expect("target");
        if !target.extra.is_empty() && target.extra_vertices.as_ref().is_none_or(|(h, _)| *h != head_number) {
            let floats = extra_points(&target.extra, resolution, &self.layout.triangles, &head.vertices);
            target.extra_vertices = Some((head_number, buffer_with(device, "extra points", &words_to_bytes(&floats.iter().map(|v| v.to_bits()).collect::<Vec<_>>()), wgpu::BufferUsages::VERTEX)));
        }
        let target = &*target;
        queue.write_buffer(&target.finish_uniform, 0, &words_to_bytes(&[background, resolution, u32::from(layers.maps), 0]));
        let view = target.painted.create_view(&wgpu::TextureViewDescriptor::default());
        let maps_view = target.maps_painted.create_view(&wgpu::TextureViewDescriptor::default());
        let heights_view = target.heights.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("paint") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("paint"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &maps_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &heights_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.paint_pipeline);
            pass.set_bind_group(0, self.paint_bind.as_ref(), &[]);
            pass.set_vertex_buffer(0, self.vertices.as_ref().expect("vertices").slice(..));
            pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.index_count, 0, 0..1);
            if let Some((_, points)) = &target.extra_vertices {
                pass.set_pipeline(&self.point_pipeline);
                pass.set_vertex_buffer(0, points.slice(..));
                pass.draw(0..target.extra.len() as u32, 0..1);
            }
        }
        if finish {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("finish"), timestamp_writes: None });
            pass.set_pipeline(&self.finish_pipeline);
            pass.set_bind_group(0, target.finish_bind.as_ref(), &[]);
            pass.dispatch_workgroups(resolution.div_ceil(16), resolution.div_ceil(16), 1);
        }
        queue.submit([encoder.finish()]);
    }
}

impl Target {
    fn new(device: &wgpu::Device, resolution: u32) -> Self {
        let painted = device.create_texture(&painted_descriptor(resolution));
        let output = |label: &str| device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: 4 * u64::from(resolution) * u64::from(resolution),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let texels = output("texels");
        let maps_texels = output("maps texels");
        let maps_painted = device.create_texture(&wgpu::TextureDescriptor { label: Some("maps painted"), ..painted_descriptor(resolution) });
        let heights = device.create_texture(&wgpu::TextureDescriptor { label: Some("heights"), format: wgpu::TextureFormat::Rg32Uint, ..painted_descriptor(resolution) });
        let finish_uniform = device.create_buffer(&wgpu::BufferDescriptor { label: Some("finish"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        Target { resolution, painted, finish_uniform, finish_bind: None, covered: Vec::new(), extra: Vec::new(), extra_vertices: None, texels, maps_painted, maps_texels, heights }
    }
}

/// A paint target of one size: one packed texel per pixel.
fn painted_descriptor(resolution: u32) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some("painted"),
        size: wgpu::Extent3d { width: resolution, height: resolution, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Uint,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    }
}

fn layer_uniform(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: Some("layer uniform"), size, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

fn layer_storage(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: Some("layer storage"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

impl LayerBuffers {
    fn new(device: &wgpu::Device, uniform_size: u64, storage_size: u64) -> Self {
        LayerBuffers { uniform: layer_uniform(device, uniform_size), uniform_size, storage: layer_storage(device, storage_size), storage_size }
    }

    /// Writes a layer's words, making a buffer anew when its size changed. Returns whether a buffer was made anew.
    fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, uniform: &[u32], storage: Option<&[u32]>) -> bool {
        let mut remade = false;
        let u = 4 * uniform.len() as u64;
        if u != self.uniform_size {
            (self.uniform, self.uniform_size) = (layer_uniform(device, u), u);
            remade = true;
        }
        queue.write_buffer(&self.uniform, 0, &words_to_bytes(uniform));
        if let Some(words) = storage {
            let s = (4 * words.len() as u64).max(16);
            if s != self.storage_size {
                (self.storage, self.storage_size) = (layer_storage(device, s), s);
                remade = true;
            }
            if !words.is_empty() {
                queue.write_buffer(&self.storage, 0, &words_to_bytes(words));
            }
        }
        remade
    }
}

fn buffer_with(device: &wgpu::Device, label: &str, bytes: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: (bytes.len() as u64).max(16), usage, mapped_at_creation: true });
    buffer.slice(..bytes.len() as u64).get_mapped_range_mut().copy_from_slice(bytes);
    buffer.unmap();
    buffer
}

/// The texel each texel of a finished texture copies: itself where it was covered, the nearest covered texel within `SEAM_GROWTH`
/// pixels, as `pixel_map::grow_over_seams` picks it, or `BACKGROUND`.
pub fn finish_sources(painted: &[bool], n: usize) -> Vec<u32> {
    let nearest = nearest_true(painted, n, n);
    (0..n * n)
        .map(|at| {
            if painted[at] {
                return at as u32;
            }
            let s = nearest[at];
            if s == u32::MAX {
                return BACKGROUND;
            }
            let s_us = s as usize;
            let (dy, dx) = ((at / n) as f64 - (s_us / n) as f64, (at % n) as f64 - (s_us % n) as f64);
            if (dx * dx + dy * dy).sqrt() <= SEAM_GROWTH { s } else { BACKGROUND }
        })
        .collect()
}

/// The vertices of the extra points: each at its texel's centre, with the fields of its triangle's corners mixed by its barycentric
/// weights, in f64 as `PixelMap::interpolate` mixes them.
fn extra_points(pixels: &[head_paint::pixel_map::Pixel], resolution: u32, triangles: &[[u32; 3]], vertices: &[f32]) -> Vec<f32> {
    let n = f64::from(resolution);
    let mut out = Vec::with_capacity(VERTEX_FLOATS * pixels.len());
    for p in pixels {
        let corners = triangles[p.triangle as usize];
        out.push(((f64::from(p.x) + 0.5) / n) as f32);
        out.push(((n - 1.0 - f64::from(p.y) + 0.5) / n) as f32);
        for f in 2..VERTEX_FLOATS {
            let v: f64 = (0..3).map(|k| p.barycentric[k] * f64::from(vertices[VERTEX_FLOATS * corners[k] as usize + f])).sum();
            out.push(v as f32);
        }
    }
    out
}
