//! IronBeam Window Manager
//! 
//! Manages the layout, docking, and lifecycle of UI panels.

use crate::cell::CellId;
use crate::ironbeam::shell::theme::Theme;

#[derive(Clone, Debug)]
pub enum DockPosition {
    Left,
    Right,
    Bottom,
    Center,
}

#[derive(Clone, Debug)]
pub struct Panel {
    pub id: CellId,
    pub title: String,
    pub position: DockPosition,
    pub width: f32,  // For Left/Right
    pub height: f32, // For Bottom
    pub visible: bool,
}

pub struct WindowManager {
    pub panels: Vec<Panel>,
    pub active_panel: Option<CellId>,
    pub theme: Theme,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            active_panel: None,
            theme: Theme::default(),
        }
    }

    pub fn register_panel(&mut self, id: CellId, title: &str, position: DockPosition) {
        self.panels.push(Panel {
            id,
            title: title.to_string(),
            position,
            width: 300.0,  // Default width
            height: 200.0, // Default height
            visible: true,
        });
    }

    pub fn toggle_panel(&mut self, id: CellId) {
        if let Some(panel) = self.panels.iter_mut().find(|p| p.id == id) {
            panel.visible = !panel.visible;
        }
    }

    pub fn focus_panel(&mut self, id: CellId) {
        self.active_panel = Some(id);
    }

    /// Calculate the layout rect for a panel based on current state
    pub fn get_panel_rect(&self, id: CellId, window_width: f32, window_height: f32) -> Option<(f32, f32, f32, f32)> {
        let panel = self.panels.iter().find(|p| p.id == id)?;
        if !panel.visible {
            return None;
        }

        // Simple layout logic for MVP
        // In a real system, this would handle complex tiling/splitting
        match panel.position {
            DockPosition::Left => Some((0.0, 0.0, panel.width, window_height)),
            DockPosition::Right => Some((window_width - panel.width, 0.0, panel.width, window_height)),
            DockPosition::Bottom => {
                // Assuming bottom panel spans the full width minus side panels
                // For simplicity, just full width for now
                Some((0.0, window_height - panel.height, window_width, panel.height))
            }
            DockPosition::Center => {
                // Center fills remaining space
                // Simplified: Just a rect in the middle
                Some((300.0, 0.0, window_width - 600.0, window_height - 200.0))
            }
        }
    }
}
