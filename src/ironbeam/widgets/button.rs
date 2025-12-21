//! IronBeam Button Widget

use crate::cell::CellId;

pub struct Button {
    pub id: CellId,
    pub label: String,
    pub clicked: bool,
}

impl Button {
    pub fn new(id: CellId, label: &str) -> Self {
        Self {
            id,
            label: label.to_string(),
            clicked: false,
        }
    }
    
    pub fn render(&self) -> String {
        format!(
            r##"<rect width="100" height="30" fill="#007acc" rx="4" />
               <text x="50" y="20" text-anchor="middle" fill="white">{}</text>"##,
            self.label
        )
    }
}
