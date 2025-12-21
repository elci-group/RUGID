//! IronBeam Dropdown Widget

use crate::cell::CellId;

pub struct Dropdown {
    pub id: CellId,
    pub options: Vec<String>,
    pub selected_index: usize,
}

impl Dropdown {
    pub fn new(id: CellId, options: Vec<String>) -> Self {
        Self {
            id,
            options,
            selected_index: 0,
        }
    }
    
    pub fn render(&self) -> String {
        let text = self.options.get(self.selected_index).map(|s| s.as_str()).unwrap_or("");
        format!(
            r##"<rect width="150" height="30" fill="#252526" stroke="#454545" rx="4" />
               <text x="10" y="20" fill="#cccccc">{}</text>
               <path d="M130 12 L140 12 L135 20 Z" fill="#cccccc" />"##,
            text
        )
    }
}
