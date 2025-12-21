use crate::platform::{Platform, PlatformEvent};

pub struct BitmapPlatform {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
}

impl BitmapPlatform {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }
}

impl Platform for BitmapPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        Vec::new()
    }

    fn render(&mut self, svg: String) {
        // Mock Rasterization
        let rect_count = svg.matches("<rect").count();
        
        // Simulate iterating pixels for each rect
        // Assume average rect covers 100 pixels
        let pixels_touched = rect_count * 100;
        
        for i in 0..pixels_touched {
            if i < self.buffer.len() {
                self.buffer[i] = 0xFFFFFFFF;
            }
        }
    }
}
