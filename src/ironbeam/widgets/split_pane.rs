//! IronBeam Split Pane Widget

use crate::cell::CellId;

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct SplitPane {
    pub id: CellId,
    pub orientation: Orientation,
    pub split_ratio: f32,
}

impl SplitPane {
    pub fn new(id: CellId, orientation: Orientation) -> Self {
        Self {
            id,
            orientation,
            split_ratio: 0.5,
        }
    }
    
    pub fn render(&self, width: f32, height: f32) -> String {
        match self.orientation {
            Orientation::Horizontal => {
                let split = width * self.split_ratio;
                format!(
                    r##"<rect x="{}" y="0" width="4" height="{}" fill="#1e1e1e" />"##,
                    split - 2.0, height
                )
            }
            Orientation::Vertical => {
                let split = height * self.split_ratio;
                format!(
                    r##"<rect x="0" y="{}" width="{}" height="4" fill="#1e1e1e" />"##,
                    split - 2.0, width
                )
            }
        }
    }
}
