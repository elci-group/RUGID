//! IronBeam Tree View Widget

use crate::cell::CellId;

pub struct TreeItem {
    pub label: String,
    pub children: Vec<TreeItem>,
    pub expanded: bool,
}

pub struct TreeView {
    pub id: CellId,
    pub root: TreeItem,
}

impl TreeView {
    pub fn new(id: CellId, root: TreeItem) -> Self {
        Self { id, root }
    }
    
    pub fn render(&self) -> String {
        // Simplified rendering
        format!(
            r##"<text x="10" y="20" fill="#cccccc">{}</text>"##,
            self.root.label
        )
    }
}
