//! IronBeam Wind System
//! 
//! Simulates wind dynamics and turbulence.

use crate::geometry3d::Point3D;
use crate::physics::core::PhysicsState;

#[derive(Clone, Debug)]
pub struct GustConfig {
    pub enabled: bool,
    pub frequency: f32,      // Gusts per second
    pub intensity_range: (f32, f32),
    pub duration_range: (f32, f32),
}

impl Default for GustConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency: 0.1,
            intensity_range: (1.5, 3.0),
            duration_range: (1.0, 5.0),
        }
    }
}

pub struct WindSystem {
    /// Base wind vector (direction and speed)
    pub base_velocity: Point3D,
    
    /// Turbulence intensity (0.0 = laminar, 1.0 = highly turbulent)
    pub turbulence: f32,
    
    /// Gust configuration
    pub gusts: GustConfig,
}

impl WindSystem {
    pub fn new() -> Self {
        Self {
            base_velocity: Point3D::new(1.0, 0.0, 0.0),
            turbulence: 0.5,
            gusts: GustConfig::default(),
        }
    }

    /// Simple pseudo-noise function
    fn noise(&self, x: f32, y: f32, z: f32) -> f32 {
        let n = (x * 12.9898 + y * 78.233 + z * 37.719).sin() * 43758.5453;
        n - n.floor()
    }

    pub fn sample(&self, position: Point3D, time: f64) -> Point3D {
        let base = self.base_velocity;
        
        // Add turbulence
        let noise_scale = 0.1;
        let t = time as f32;
        let turbulence_offset = Point3D::new(
            self.noise(position.x * noise_scale, position.y * noise_scale, t),
            self.noise(position.y * noise_scale, position.z * noise_scale, t + 100.0),
            self.noise(position.z * noise_scale, position.x * noise_scale, t + 200.0),
        ) * self.turbulence * base.magnitude();
        
        base + turbulence_offset
    }
    
    pub fn apply_force(&self, body: &mut PhysicsState, position: Point3D, area: f32, drag_coefficient: f32, time: f64) {
        let wind = self.sample(position, time);
        let relative_velocity = wind - body.velocity;
        let speed = relative_velocity.magnitude();
        
        // Drag force: F = 0.5 * ρ * v² * Cd * A
        let air_density = 1.225;  // kg/m³ at sea level
        let force_magnitude = 0.5 * air_density * speed * speed * drag_coefficient * area;
        
        if speed > 0.001 {
            let force = relative_velocity.normalize() * force_magnitude;
            body.apply_force(force);
        }
    }
}
