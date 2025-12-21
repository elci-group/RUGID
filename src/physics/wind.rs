/// Wind Physics Module
///
/// Models the movement of air as a vector field.
/// Includes Global Wind and Turbulence (Gusts).

use crate::geometry3d::Point3D;

/// Configuration for the Wind System
#[derive(Clone, Debug)]
pub struct WindConfig {
    /// Base wind velocity (m/s)
    pub base_velocity: Point3D,
    /// Turbulence intensity (0.0 - 1.0)
    pub turbulence: f32,
    /// Frequency of gusts (Hz)
    pub gust_frequency: f32,
}

impl Default for WindConfig {
    fn default() -> Self {
        Self {
            base_velocity: Point3D::zero(),
            turbulence: 0.0,
            gust_frequency: 0.1,
        }
    }
}

pub struct WindSystem {
    pub config: WindConfig,
    /// Time accumulator for turbulence animation
    time: f32,
}

impl WindSystem {
    pub fn new(config: WindConfig) -> Self {
        Self {
            config,
            time: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
    }

    /// Get the wind vector at a specific position
    /// V_wind = V_base + V_turbulence(pos, time)
    pub fn get_wind_at(&self, position: Point3D) -> Point3D {
        let base = self.config.base_velocity;
        
        if self.config.turbulence <= 0.001 {
            return base;
        }

        // Generate deterministic turbulence using sine wave superposition
        // This simulates a "rolling" wind field without needing external noise libs.
        
        let t = self.time * self.config.gust_frequency;
        let scale = 0.1; // Spatial scale
        
        // Wave 1: Main gust
        let w1 = (position.x * scale + t).sin();
        let w2 = (position.y * scale + t * 1.3).cos();
        let w3 = (position.z * scale + t * 0.7).sin();
        
        // Wave 2: High frequency detail
        let w4 = (position.x * scale * 2.0 - t * 2.0).cos();
        let w5 = (position.z * scale * 2.0 + t * 2.5).sin();
        
        let noise_x = w1 + w4 * 0.5;
        let noise_y = w2 * 0.5; // Less vertical turbulence usually
        let noise_z = w3 + w5 * 0.5;
        
        let turbulence_vec = Point3D::new(noise_x, noise_y, noise_z);
        
        // Scale by turbulence intensity
        // Assuming intensity is in m/s magnitude roughly
        let gust = turbulence_vec * (self.config.turbulence * 5.0); 
        
        base + gust
    }
}
