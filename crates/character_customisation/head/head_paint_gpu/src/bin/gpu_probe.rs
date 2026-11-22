//! `gpu_probe [size]` fills a texture on the GPU natively, reads it back and counts the texels that differ from the pattern.

fn main() {
    let size: u32 = std::env::args().nth(1).map_or(2048, |s| s.parse().expect("size"));
    let gpu = head_paint_gpu::gpu::Gpu::blocking().expect("GPU");
    let start = std::time::Instant::now();
    let rgba = pollster::block_on(head_paint_gpu::probe::fill(&gpu, size));
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    let bad = head_paint_gpu::probe::mismatches(&rgba, size);
    println!("{size} by {size}: {} bytes back, {bad} texels differ from the pattern, {ms:.1} ms", rgba.len());
    std::process::exit(i32::from(bad != 0));
}
