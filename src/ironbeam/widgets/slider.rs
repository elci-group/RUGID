//! IronBeam Slider Widget

use crate::cell::CellId;

pub struct Slider {
    pub id: CellId,
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl Slider {
    pub fn new(id: CellId, value: f32, min: f32, max: f32) -> Self {
        Self { id, value, min, max }
    }
    
    pub fn render(&self) -> String {
        let pct = (self.value - self.min) / (self.max - self.min);
        let width = 200.0;
        let pos = pct * width;
        
        format!(
            r##"<rect width="{}" height="4" fill="#454545" y="13" rx="2" />
               <rect width="{}" height="4" fill="#007acc" y="13" rx="2" />
               <circle cx="{}" cy="15" r="8" fill="white" />"##,
            width, pos, pos
        )
    }
}
