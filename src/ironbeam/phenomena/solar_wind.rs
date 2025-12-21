//! IronBeam Solar Wind System
//! 
//! Simulates solar wind and radiation pressure.

use crate::geometry3d::Point3D;

#[derive(Clone, Debug)]
pub struct CMEEvent {
    pub start_time: f64,
    pub duration: f64,
    pub intensity_multiplier: f64,
    pub direction: Point3D,
    pub angular_spread: f32,
}

pub struct SolarWindSystem {
    /// Source position (typically the sun)
    pub source: Point3D,
    
    /// Particle density (particles per m³)
    pub density: f64,
    
    /// Velocity (typically 400-800 km/s)
    pub velocity: f64,
    
    /// Electromagnetic field strength
    pub em_field: f64,
    
    /// Coronal mass ejection events
    pub cme_events: Vec<CMEEvent>,
}

impl SolarWindSystem {
    pub fn new() -> Self {
        Self {
            source: Point3D::zero(),
            density: 5.0e6, // 5 particles per cm^3 -> 5e6 per m^3
            velocity: 400_000.0, // 400 km/s
            em_field: 1.0,
            cme_events: Vec::new(),
        }
    }

    pub fn calculate_radiation_pressure(&self, position: Point3D, reflectivity: f32, area: f32) -> Point3D {
        let diff = position - self.source;
        let distance = diff.magnitude() as f64;
        
        if distance < 1.0 { return Point3D::zero(); }

        let direction = diff.normalize();
        
        // Radiation pressure decreases with square of distance
        // Simplified model
        let base_pressure = self.density * self.velocity * self.velocity * 1.67e-27; // mass of proton
        let pressure = base_pressure / (distance * distance * 1e-18); // Scale factor
        
        // Force = pressure * area * (1 + reflectivity)
        let force_magnitude = pressure * (area as f64) * (1.0 + reflectivity as f64);
        
        direction * (force_magnitude as f32)
    }
}
