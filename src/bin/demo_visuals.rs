use rugid::runtime::Runtime;
use rugid::cell::Cell;
use rugid::geometry::VectorRegion;
use rugid::platform::{Platform, PlatformEvent};
use rugid::widgets::ButtonProjector;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

// --- Shared State Wrappers ---

struct SharedTuiPlatform {
    width: usize,
    height: usize,
    buffer: Arc<Mutex<String>>,
}

impl Platform for SharedTuiPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> { Vec::new() }
    fn render(&mut self, svg: String) {
        let rect_count = svg.matches("<rect").count();
        let mut buf = self.buffer.lock().unwrap();
        buf.clear();
        // Simple visualization: Draw a box of '#' based on rect count
        // Real TUI would map coordinates.
        let fill_count = (rect_count * 10).min(self.width * self.height);
        for i in 0..self.height {
            for j in 0..self.width {
                if (i * self.width + j) < fill_count {
                    buf.push('#');
                } else {
                    buf.push('.');
                }
            }
            buf.push('\n');
        }
    }
}

struct SharedBitmapPlatform {
    width: usize,
    height: usize,
    buffer: Arc<Mutex<Vec<u8>>>, // RGB
}

impl Platform for SharedBitmapPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> { Vec::new() }
    fn render(&mut self, svg: String) {
        let rect_count = svg.matches("<rect").count();
        let mut buf = self.buffer.lock().unwrap();
        // Fill with black
        for b in buf.iter_mut() { *b = 0; }
        
        // Draw some white pixels based on rect count
        // Simple visualization
        let pixels = (rect_count * 100).min(self.width * self.height);
        for i in 0..pixels {
            let idx = i * 3;
            if idx + 2 < buf.len() {
                buf[idx] = 255;
                buf[idx+1] = 255;
                buf[idx+2] = 255;
            }
        }
    }
}

struct SharedGpuPlatform {
    log: Arc<Mutex<Vec<String>>>,
}

impl Platform for SharedGpuPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> { Vec::new() }
    fn render(&mut self, svg: String) {
        let rect_count = svg.matches("<rect").count();
        let mut log = self.log.lock().unwrap();
        log.clear();
        log.push("BEGIN_FRAME".to_string());
        log.push(format!("  BIND_PIPELINE(Rect)"));
        for i in 0..rect_count {
            log.push(format!("  DRAW_RECT(x={}, y={}, w=10, h=10, color=RED)", i * 10, i * 10));
        }
        log.push("END_FRAME".to_string());
        log.push("PRESENT".to_string());
    }
}

// --- Demo Runners ---

fn run_headless_demo(output_dir: &Path) {
    println!("Generating Headless Demo (SVG)...");
    // HeadlessPlatform is already simple, we can just use Runtime's return value (SVG string)
    // But we need a dummy platform to satisfy the type system.
    struct DummyPlatform;
    impl Platform for DummyPlatform {
        fn poll_events(&mut self) -> Vec<PlatformEvent> { Vec::new() }
        fn render(&mut self, _svg: String) {}
    }

    let mut runtime = Runtime::new(100, Box::new(DummyPlatform));
    let cell = Cell::new(VectorRegion::new(0.0, 0.0, 10.0, 10.0));
    let id = cell.id;
    runtime.register_cell(cell, Box::new(ButtonProjector {
        target: id,
        geometry: VectorRegion::new(0.0, 0.0, 10.0, 10.0),
        label: "Btn".to_string(),
    }), None, None);

    for i in 0..3 {
        let svg = runtime.tick();
        fs::write(output_dir.join(format!("headless_frame_{}.svg", i)), svg).unwrap();
    }
}

fn run_tui_demo(output_dir: &Path) {
    println!("Generating TUI Demo (Text)...");
    let buffer = Arc::new(Mutex::new(String::new()));
    let platform = Box::new(SharedTuiPlatform {
        width: 40,
        height: 10,
        buffer: buffer.clone(),
    });

    let mut runtime = Runtime::new(100, platform);
    let cell = Cell::new(VectorRegion::new(0.0, 0.0, 10.0, 10.0));
    let id = cell.id;
    runtime.register_cell(cell, Box::new(ButtonProjector {
        target: id,
        geometry: VectorRegion::new(0.0, 0.0, 10.0, 10.0),
        label: "Btn".to_string(),
    }), None, None);

    for i in 0..3 {
        runtime.tick();
        let content = buffer.lock().unwrap().clone();
        fs::write(output_dir.join(format!("tui_frame_{}.txt", i)), content).unwrap();
    }
}

fn run_bitmap_demo(output_dir: &Path) {
    println!("Generating Bitmap Demo (PPM)...");
    let width = 100;
    let height = 100;
    let buffer = Arc::new(Mutex::new(vec![0; width * height * 3]));
    let platform = Box::new(SharedBitmapPlatform {
        width,
        height,
        buffer: buffer.clone(),
    });

    let mut runtime = Runtime::new(100, platform);
    let cell = Cell::new(VectorRegion::new(0.0, 0.0, 10.0, 10.0));
    let id = cell.id;
    runtime.register_cell(cell, Box::new(ButtonProjector {
        target: id,
        geometry: VectorRegion::new(0.0, 0.0, 10.0, 10.0),
        label: "Btn".to_string(),
    }), None, None);

    for i in 0..3 {
        runtime.tick();
        let buf = buffer.lock().unwrap();
        // Write PPM (P3)
        let mut ppm = format!("P3\n{} {}\n255\n", width, height);
        for pixel in buf.chunks(3) {
            ppm.push_str(&format!("{} {} {} ", pixel[0], pixel[1], pixel[2]));
        }
        fs::write(output_dir.join(format!("bitmap_frame_{}.ppm", i)), ppm).unwrap();
    }
}

fn run_gpu_demo(output_dir: &Path) {
    println!("Generating GPU Proxy Demo (Log)...");
    let log = Arc::new(Mutex::new(Vec::new()));
    let platform = Box::new(SharedGpuPlatform {
        log: log.clone(),
    });

    let mut runtime = Runtime::new(100, platform);
    let cell = Cell::new(VectorRegion::new(0.0, 0.0, 10.0, 10.0));
    let id = cell.id;
    runtime.register_cell(cell, Box::new(ButtonProjector {
        target: id,
        geometry: VectorRegion::new(0.0, 0.0, 10.0, 10.0),
        label: "Btn".to_string(),
    }), None, None);

    for i in 0..3 {
        runtime.tick();
        let l = log.lock().unwrap();
        let content = l.join("\n");
        fs::write(output_dir.join(format!("gpu_frame_{}.log", i)), content).unwrap();
    }
}

fn main() {
    let output_dir = Path::new("demo_output");
    if !output_dir.exists() {
        fs::create_dir(output_dir).unwrap();
    }

    run_headless_demo(output_dir);
    run_tui_demo(output_dir);
    run_bitmap_demo(output_dir);
    run_gpu_demo(output_dir);

    println!("All demos generated in demo_output/");
}
