use crate::input::InputEvent;

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    Input(InputEvent),
    Resize(f32, f32),
    RedrawRequest,
    Quit,
}

use winit::event::Event;

pub trait Platform {
    fn poll_events(&mut self) -> Vec<PlatformEvent>;
    fn render(&mut self, svg: String);
    fn handle_event(&mut self, _event: &Event<()>) {}
    
    /// Get the current viewport dimensions (width, height)
    fn dimensions(&self) -> (f32, f32) {
        (800.0, 600.0) // Default dimensions
    }
    
    /// Set the window title
    fn set_title(&mut self, _title: &str) {}
}

pub struct HeadlessPlatform {
    events: Vec<PlatformEvent>,
    pub last_frame: Option<String>,
    width: f32,
    height: f32,
}

impl HeadlessPlatform {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            last_frame: None,
            width: 800.0,
            height: 600.0,
        }
    }
    
    pub fn with_dimensions(width: f32, height: f32) -> Self {
        Self {
            events: Vec::new(),
            last_frame: None,
            width,
            height,
        }
    }

    pub fn inject_event(&mut self, event: PlatformEvent) {
        self.events.push(event);
    }
}

impl Platform for HeadlessPlatform {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        let events = self.events.clone();
        self.events.clear();
        events
    }

    fn render(&mut self, svg: String) {
        self.last_frame = Some(svg);
    }
    
    fn dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_platform() {
        let mut platform = HeadlessPlatform::new();
        platform.inject_event(PlatformEvent::Quit);
        
        let events = platform.poll_events();
        assert_eq!(events.len(), 1);
        match events[0] {
            PlatformEvent::Quit => {},
            _ => panic!("Wrong event type"),
        }

        platform.render("<svg></svg>".to_string());
        assert_eq!(platform.last_frame.as_deref(), Some("<svg></svg>"));
    }
}
