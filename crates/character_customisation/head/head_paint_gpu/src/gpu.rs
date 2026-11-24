//! The GPU device the painter draws with, and reading a buffer back to the processor.
//!
//! The painter takes a device and a queue from outside, so that the engine can hand it its own. `Gpu::request` makes a device of its own
//! for the tools and the browser. It asks for WebGPU's default limits only, which every card of the minimum spec (a GTX 1050 Ti) meets.

/// A device and its queue. Both are cheap handles, so cloning one shares the same device.
#[derive(Clone)]
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Gpu {
    /// Makes a device on the first high-performance adapter, with WebGPU's default limits and no optional feature.
    pub async fn request() -> Result<Gpu, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })
            .await
            .map_err(|e| format!("no GPU adapter: {e}"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor { label: Some("head paint"), required_limits: wgpu::Limits::default(), ..Default::default() })
            .await
            .map_err(|e| format!("no GPU device: {e}"))?;
        Ok(Gpu { device, queue })
    }

    /// `request`, waiting for the answer. Native builds only.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn blocking() -> Result<Gpu, String> {
        pollster::block_on(Self::request())
    }

    /// Waits until the GPU has done all the work submitted so far. In the browser this returns at once, and a read waits instead.
    pub fn wait(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.device.poll(wgpu::PollType::wait_indefinitely()).expect("the GPU stopped");
    }

    /// Resolves once the GPU has finished the work submitted so far, natively and in the browser.
    pub async fn done(&self) {
        let (tx, rx) = futures_channel::oneshot::channel();
        self.queue.on_submitted_work_done(move || {
            let _ = tx.send(());
        });
        self.wait();
        let _ = rx.await;
    }

    /// Copies `size` bytes of `buffer` (which needs `COPY_SRC`) to the processor.
    pub async fn read(&self, buffer: &wgpu::Buffer, size: u64) -> Vec<u8> {
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read back"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("read back") });
        encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, size);
        self.queue.submit([encoder.finish()]);
        let (tx, rx) = futures_channel::oneshot::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.wait();
        rx.await.expect("the GPU dropped the read").expect("the read failed");
        let bytes = staging.slice(..).get_mapped_range().to_vec();
        staging.unmap();
        bytes
    }
}

/// Bytes of 32-bit words, in the GPU's little-endian order.
pub fn words_to_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}
