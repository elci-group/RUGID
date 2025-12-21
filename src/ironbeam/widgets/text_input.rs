//! IronBeam Text Input Widget

use crate::cell::CellId;

pub struct TextInput {
    pub id: CellId,
    pub value: String,
    pub placeholder: String,
}

impl TextInput {
    pub fn new(id: CellId, value: &str) -> Self {
        Self {
            id,
            value: value.to_string(),
            placeholder: String::new(),
        }
    }
    
    pub fn render(&self) -> String {
        format!(
            r##"<rect width="200" height="30" fill="#252526" stroke="#454545" rx="4" />
               <text x="10" y="20" fill="#cccccc">{}</text>"##,
            self.value
        )
    }
}
