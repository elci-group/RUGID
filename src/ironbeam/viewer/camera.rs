//! IronBeam Camera System
//! 
//! Handles 3D camera state, projection, and control logic.

use crate::geometry3d::{Point3D, Rotation3D};
use crate::input::InputEvent;

#[derive(Clone, Debug, PartialEq)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
    Isometric,
}

#[derive(Clone, Debug)]
pub struct Camera3D {
    /// Position in world space
    pub position: Point3D,
    
    /// Euler angles (pitch, yaw, roll) in degrees
    pub rotation: Rotation3D,
    
    /// Field of view in degrees
    pub fov: f32,
    
    /// Near clipping plane
    pub near: f32,
    
    /// Far clipping plane
    pub far: f32,
    
    /// Projection mode
    pub projection: ProjectionMode,
    
    /// Orbit target (for orbit camera mode)
    pub orbit_target: Option<Point3D>,
    
    /// Zoom level (affects orbit distance or orthographic scale)
    pub zoom: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Point3D::new(0.0, 0.0, -10.0),
            rotation: Rotation3D::zero(),
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            projection: ProjectionMode::Perspective,
            orbit_target: Some(Point3D::zero()),
            zoom: 10.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CameraMode {
    Orbit { center: Point3D, distance: f32 },
    Fly { velocity: Point3D },
    Locked,
}

pub struct CameraController {
    /// Sensitivity multipliers
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    
    /// Constraints
    pub pitch_limits: (f32, f32),  // Min/max pitch in degrees
    pub zoom_limits: (f32, f32),   // Min/max zoom distance
    
    /// Smoothing
    pub interpolation_speed: f32,
    pub target_rotation: Rotation3D,
    pub target_position: Point3D,
    pub target_zoom: f32,
    
    /// Mode
    pub mode: CameraMode,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            orbit_sensitivity: 0.5,
            pan_sensitivity: 0.1,
            zoom_sensitivity: 0.1,
            pitch_limits: (-89.0, 89.0),
            zoom_limits: (1.0, 100.0),
            interpolation_speed: 10.0,
            target_rotation: Rotation3D::zero(),
            target_position: Point3D::new(0.0, 0.0, -10.0),
            target_zoom: 10.0,
            mode: CameraMode::Orbit { 
                center: Point3D::zero(), 
                distance: 10.0 
            },
        }
    }
}

impl CameraController {
    pub fn process_input(&mut self, event: &InputEvent) {
        match event {
            InputEvent::PointerMove(dx, dy) => {
                // Simplified input handling for now
                // In a real implementation, we'd check for mouse buttons
                // For now, assume always orbiting if moving
                self.target_rotation.yaw += dx * self.orbit_sensitivity;
                self.target_rotation.pitch += dy * self.orbit_sensitivity;
                self.target_rotation.pitch = self.target_rotation.pitch
                    .clamp(self.pitch_limits.0, self.pitch_limits.1);
            }
            // TODO: Handle scroll for zoom
            _ => {}
        }
    }
    
    pub fn update(&mut self, camera: &mut Camera3D, dt: f32) {
        // Smooth interpolation toward target values
        let t = 1.0 - (-self.interpolation_speed * dt).exp();
        
        // Simple lerp for now
        camera.rotation.pitch += (self.target_rotation.pitch - camera.rotation.pitch) * t;
        camera.rotation.yaw += (self.target_rotation.yaw - camera.rotation.yaw) * t;
        camera.rotation.roll += (self.target_rotation.roll - camera.rotation.roll) * t;
        
        // Update position based on orbit
        if let CameraMode::Orbit { center, distance } = self.mode {
            // Calculate position from rotation and distance
            // This is a simplified orbit calculation
            // Real implementation would use quaternions or matrix math
            // But RUGID uses Euler angles
            
            // For now, just update the camera struct
            camera.zoom = distance;
            camera.orbit_target = Some(center);
        }
    }
}
