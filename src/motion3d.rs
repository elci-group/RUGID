//! 3D Motion Module
//!
//! Defines states for 3D movement:
//! - Translation (Panning/Zooming)
//! - Orbiting (Circling)

use crate::geometry3d::Point3D;

/// State for linear translation (Panning/Zooming)
#[derive(Clone, Debug, PartialEq)]
pub struct Translation3DState {
    /// Current position offset
    pub current: Point3D,
    /// Velocity per frame
    pub velocity: Point3D,
}

impl Translation3DState {
    pub fn new(vx: f32, vy: f32, vz: f32) -> Self {
        Self {
            current: Point3D::new(0.0, 0.0, 0.0),
            velocity: Point3D::new(vx, vy, vz),
        }
    }
    
    pub fn update(&mut self) {
        self.current.x += self.velocity.x;
        self.current.y += self.velocity.y;
        self.current.z += self.velocity.z;
    }
}

/// Axis of rotation for orbiting
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrbitAxis {
    X, // Orbit around X axis (vertical circle)
    Y, // Orbit around Y axis (horizontal circle)
    Z, // Orbit around Z axis (screen plane circle)
}

/// State for orbital motion (Circling)
#[derive(Clone, Debug, PartialEq)]
pub struct Orbit3DState {
    pub angle: f32,       // Current angle in degrees
    pub speed: f32,       // Angular velocity (deg/frame)
    pub radius: f32,      // Orbit radius
    pub center: Point3D,  // Center of orbit (relative to shape center)
    pub axis: OrbitAxis,  // Axis of rotation
}

impl Orbit3DState {
    pub fn new(radius: f32, speed: f32, axis: OrbitAxis) -> Self {
        Self {
            angle: 0.0,
            speed,
            radius,
            center: Point3D::new(0.0, 0.0, 0.0),
            axis,
        }
    }
    
    pub fn update(&mut self) {
        self.angle += self.speed;
        self.angle %= 360.0;
    }
    
    /// Calculate current offset based on orbit
    pub fn get_offset(&self) -> Point3D {
        let rad = self.angle.to_radians();
        let sin = rad.sin();
        let cos = rad.cos();
        let r = self.radius;
        
        match self.axis {
            OrbitAxis::X => Point3D::new(
                0.0,
                r * cos,
                r * sin
            ),
            OrbitAxis::Y => Point3D::new(
                r * sin,
                0.0,
                r * cos
            ),
            OrbitAxis::Z => Point3D::new(
                r * cos,
                r * sin,
                0.0
            ),
        }
    }
}
