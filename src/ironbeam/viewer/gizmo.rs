//! IronBeam Gizmo System
//! 
//! Provides visual handles for transforming objects in 3D space.

use crate::geometry3d::Point3D;
use crate::ironbeam::viewer::camera::Camera3D;

#[derive(Clone, Debug, PartialEq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
    Universal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GizmoSpace {
    Local,
    World,
    View,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Axis {
    X, Y, Z,
    XY, XZ, YZ,
    XYZ,
}

pub struct GizmoSystem {
    pub mode: GizmoMode,
    pub space: GizmoSpace,
    pub visible: bool,
    pub size: f32,
    pub active_axis: Option<Axis>,
    pub drag_start: Option<Point3D>,
}

impl Default for GizmoSystem {
    fn default() -> Self {
        Self {
            mode: GizmoMode::Translate,
            space: GizmoSpace::World,
            visible: true,
            size: 100.0,
            active_axis: None,
            drag_start: None,
        }
    }
}

impl GizmoSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&self, _camera: &Camera3D, _selection_center: Point3D) -> String {
        if !self.visible { return String::new(); }
        
        // Gizmo size is constant in screen space
        // We need to project the center to screen space to know where to draw
        // But since we are returning SVG string, we'll just generate SVG elements
        // that represent the gizmo.
        // For a zero z-depth GUI, the gizmo is drawn ON TOP of the 3D scene.
        
        // Placeholder implementation
        // In a real system, we'd project the axes and draw lines/arrows
        
        String::new()
    }
}
