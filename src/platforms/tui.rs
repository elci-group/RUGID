use crate::platform::{Platform, PlatformEvent};

pub struct TuiPlatform {
    width: usize,
    height: usize,
    buffer: String,
}

impl TuiPlatform {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: String::with_capacity(width * height),
        }
    }
}

impl Platform for TuiPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        Vec::new() // No input for benchmark
    }

    fn render(&mut self, svg: String) {
        // Mock SVG parsing: Count "rect" tags to simulate work
        // In a real TUI, we'd map rects to character cells.
        let rect_count = svg.matches("<rect").count();
        
        self.buffer.clear();
        // Simulate filling buffer
        for _ in 0..rect_count {
            self.buffer.push('#');
        }
    }
}
