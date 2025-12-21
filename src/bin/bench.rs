use rugid::runtime::Runtime;
use rugid::cell::Cell;
use rugid::geometry::VectorRegion;
use rugid::platform::{Platform, HeadlessPlatform};
use rugid::platforms::tui::TuiPlatform;
use rugid::platforms::bitmap::BitmapPlatform;
use rugid::platforms::gpu_proxy::GpuProxyPlatform;
use rugid::widgets::ButtonProjector;
use std::time::Instant;

fn run_benchmark(name: &str, mut platform: Box<dyn Platform>) {
    let mut runtime = Runtime::new(100, platform);

    // Setup Heavy Scene: 1000 Buttons
    for i in 0..1000 {
        let cell = Cell::new(VectorRegion::new(0.0, 0.0, 10.0, 10.0));
        let id = cell.id;
        runtime.register_cell(cell, Box::new(ButtonProjector {
            target: id,
            geometry: VectorRegion::new(0.0, 0.0, 10.0, 10.0),
            label: format!("Btn {}", i),
        }), None, None);
    }

    let start = Instant::now();
    let frames = 100;
    
    for _ in 0..frames {
        runtime.tick();
    }
    
    let duration = start.elapsed();
    let fps = frames as f64 / duration.as_secs_f64();
    let time_per_frame = duration.as_micros() as f64 / frames as f64;

    println!("| {:<15} | {:<10.2} | {:<10.2} |", name, fps, time_per_frame);
}

fn main() {
    println!("| Platform        | FPS        | Time/Frame (us) |");
    println!("|-----------------|------------|-----------------|");
    
    run_benchmark("Headless", Box::new(HeadlessPlatform::new()));
    run_benchmark("TUI (80x24)", Box::new(TuiPlatform::new(80, 24)));
    run_benchmark("Bitmap (1080p)", Box::new(BitmapPlatform::new(1920, 1080)));
    run_benchmark("GPU Proxy", Box::new(GpuProxyPlatform::new()));
}
