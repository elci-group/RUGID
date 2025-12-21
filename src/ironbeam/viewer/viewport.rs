//! IronBeam Viewport
//! 
//! The primary 3D view component.

use crate::cell::CellId;
use crate::ironbeam::viewer::camera::{Camera3D, CameraController};
use crate::ironbeam::viewer::staging::StagingManager;
use crate::ironbeam::viewer::gizmo::GizmoSystem;
use crate::input::InputEvent;

#[derive(Clone, Debug)]
pub struct GridConfig {
    pub visible: bool,
    pub size: f32,
    pub subdivisions: usize,
    pub color: String,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            visible: true,
            size: 100.0,
            subdivisions: 10,
            color: "rgba(100, 100, 100, 0.5)".to_string(),
        }
    }
}

pub struct ViewerPane {
    pub id: CellId,
    pub camera: Camera3D,
    pub controller: CameraController,
    pub staging: StagingManager,
    pub gizmo: GizmoSystem,
    pub grid: GridConfig,
    pub width: f32,
    pub height: f32,
}

impl ViewerPane {
    pub fn new(id: CellId) -> Self {
        Self {
            id,
            camera: Camera3D::default(),
            controller: CameraController::default(),
            staging: StagingManager::new(),
            gizmo: GizmoSystem::new(),
            grid: GridConfig::default(),
            width: 800.0,
            height: 600.0,
        }
    }

    pub fn handle_input(&mut self, event: &InputEvent) {
        self.controller.process_input(event);
    }

    pub fn update(&mut self, dt: f32) {
        self.controller.update(&mut self.camera, dt);
    }

    pub fn render(&self) -> String {
        let mut svg = String::new();
        
        // 1. Background (transparent or solid)
        svg.push_str(&format!(
            r##"<rect x="0" y="0" width="{}" height="{}" fill="#1e1e1e" />"##,
            self.width, self.height
        ));
        
        // 2. Grid
        if self.grid.visible {
            // Render grid lines
        }
        
        // 3. Models
        // In a real implementation, we would sort by depth and project vertices
        // For now, we delegate to the existing RUGID renderer logic or implement a simplified one here
        // Since this is "IronBeam", it's a UI overlay. The actual 3D rendering might happen 
        // via the main RUGID renderer, and this pane just controls it.
        // BUT, the requirement is "zero z-depth GUI... using RUGID itself".
        // So this pane IS a RUGID cell that renders 3D content.
        
        // Iterate staged models and render them
        for (_, model) in &self.staging.models {
            if model.visibility {
                // Project and render mesh
            }
        }
        
        // 4. Gizmo
        if let Some(center) = self.staging.get_selection_center() {
            svg.push_str(&self.gizmo.render(&self.camera, center));
        }
        
        svg
    }
}
