use crate::platform::{Platform, PlatformEvent};

pub enum GpuCommand {
    DrawRect(f32, f32, f32, f32, u32),
    SetColor(u32),
}

pub struct GpuProxyPlatform {
    command_buffer: Vec<GpuCommand>,
}

impl GpuProxyPlatform {
    pub fn new() -> Self {
        Self {
            command_buffer: Vec::new(),
        }
    }
}

impl Platform for GpuProxyPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        Vec::new()
    }

    fn render(&mut self, svg: String) {
        self.command_buffer.clear();
        
        // Mock Command Generation
        let rect_count = svg.matches("<rect").count();
        
        for _ in 0..rect_count {
            self.command_buffer.push(GpuCommand::SetColor(0xFF0000FF));
            self.command_buffer.push(GpuCommand::DrawRect(0.0, 0.0, 100.0, 100.0, 0));
        }
    }
}
