//! IronBeam Code Editor
//! 
//! A syntax-highlighting code editor component.

use crate::cell::CellId;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
}

#[derive(Clone, Debug)]
pub enum Language {
    Rust,
    Wgsl,
    Markdown,
    Plain,
}

#[derive(Clone, Debug)]
pub struct EditorConfig {
    pub font_family: String,
    pub font_size: f32,
    pub tab_size: u8,
    pub line_numbers: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_family: "Fira Code, monospace".to_string(),
            font_size: 14.0,
            tab_size: 4,
            line_numbers: true,
        }
    }
}

pub struct CodeEditor {
    pub id: CellId,
    pub content: String, // Using String for MVP, Rope for production
    pub language: Language,
    pub cursors: Vec<Cursor>,
    pub selections: Vec<Selection>,
    pub config: EditorConfig,
    pub path: Option<PathBuf>,
}

impl CodeEditor {
    pub fn new(id: CellId, content: String, language: Language) -> Self {
        Self {
            id,
            content,
            language,
            cursors: vec![Cursor { line: 0, column: 0 }],
            selections: Vec::new(),
            config: EditorConfig::default(),
            path: None,
        }
    }

    pub fn render(&self, width: f32, height: f32) -> String {
        let mut svg = String::new();
        
        // Background
        svg.push_str(&format!(
            r##"<rect x="0" y="0" width="{}" height="{}" fill="#1e1e1e" />"##,
            width, height
        ));
        
        // Line numbers
        if self.config.line_numbers {
            svg.push_str(r##"<rect x="0" y="0" width="40" height="100%" fill="#252526" />"##);
        }
        
        // Text content (simplified rendering)
        let lines: Vec<&str> = self.content.lines().collect();
        let line_height = self.config.font_size * 1.5;
        let start_x = if self.config.line_numbers { 50.0 } else { 10.0 };
        
        for (i, line) in lines.iter().enumerate() {
            let y = (i as f32) * line_height + line_height;
            if y > height { break; }
            
            // Render line number
            if self.config.line_numbers {
                svg.push_str(&format!(
                    r##"<text x="35" y="{}" fill="#858585" text-anchor="end" font-family="{}" font-size="{}">{}</text>"##,
                    y, self.config.font_family, self.config.font_size, i + 1
                ));
            }
            
            // Render text
            // TODO: Syntax highlighting would split this into multiple tspan elements
            svg.push_str(&format!(
                r##"<text x="{}" y="{}" fill="#d4d4d4" font-family="{}" font-size="{}">{}</text>"##,
                start_x, y, self.config.font_family, self.config.font_size, 
                line.replace("<", "&lt;").replace(">", "&gt;")
            ));
        }
        
        // Cursor
        if let Some(cursor) = self.cursors.first() {
            let cursor_y = (cursor.line as f32) * line_height + 4.0; // Offset adjustment
            let cursor_x = start_x + (cursor.column as f32) * (self.config.font_size * 0.6); // Approx char width
            
            svg.push_str(&format!(
                r##"<rect x="{}" y="{}" width="2" height="{}" fill="#007acc" />"##,
                cursor_x, cursor_y, line_height
            ));
        }
        
        svg
    }
}
