//! IronBeam Selection System
//! 
//! Handles raycasting and hit testing for selecting objects in the viewport.

use crate::geometry3d::Point3D;
use crate::ironbeam::viewer::camera::Camera3D;
use crate::ironbeam::viewer::staging::StagingManager;
use crate::cell::CellId;

pub struct Ray3D {
    pub origin: Point3D,
    pub direction: Point3D,
}

pub struct SelectionSystem;

impl SelectionSystem {
    /// Cast a ray from screen coordinates into the scene
    pub fn cast_ray(
        camera: &Camera3D,
        _screen_x: f32,
        _screen_y: f32,
        _viewport_width: f32,
        _viewport_height: f32,
    ) -> Ray3D {
        // Placeholder for unprojection logic
        // Would convert screen coords to NDC, then to World Space using inverse view-projection matrix
        Ray3D {
            origin: camera.position,
            direction: Point3D::new(0.0, 0.0, 1.0), // Dummy forward
        }
    }

    /// Find the closest model intersected by the ray
    pub fn pick(
        _staging: &StagingManager,
        _ray: &Ray3D,
    ) -> Option<CellId> {
        // Iterate over all models and check intersection with their bounds/mesh
        // Return the closest one
        None
    }
}
