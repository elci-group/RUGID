//! IronBeam Earthquake System
//! 
//! Simulates seismic events and ground displacement.

use crate::geometry3d::Point3D;
use crate::ironbeam::viewer::staging::StagedModel;
use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct SeismicWave {
    pub velocity: f32,  // m/s
    pub amplitude_decay: f32,
    pub frequency: f32,
}

#[derive(Clone, Debug)]
pub struct SurfaceWave {
    pub velocity: f32,
    pub amplitude: f32,
}

pub struct EarthquakeSystem {
    /// Epicenter location
    pub epicenter: Point3D,
    
    /// Magnitude (Richter scale)
    pub magnitude: f32,
    
    /// Focal depth (km)
    pub depth: f32,
    
    /// Wave propagation
    pub p_wave: SeismicWave,
    pub s_wave: SeismicWave,
    pub surface_waves: Vec<SurfaceWave>,
    
    /// Duration of shaking
    pub duration: f32,
    
    /// Current time since event start
    pub elapsed: f32,
}

impl EarthquakeSystem {
    pub fn new(epicenter: Point3D, magnitude: f32) -> Self {
        Self {
            epicenter,
            magnitude,
            depth: 10.0,
            p_wave: SeismicWave { velocity: 6000.0, amplitude_decay: 0.0001, frequency: 5.0 },
            s_wave: SeismicWave { velocity: 3500.0, amplitude_decay: 0.0002, frequency: 3.0 },
            surface_waves: Vec::new(),
            duration: 30.0,
            elapsed: 0.0,
        }
    }

    pub fn calculate_displacement(&self, position: Point3D, _time: f64) -> Point3D {
        let diff = position - self.epicenter;
        let distance = diff.magnitude();
        
        // P-wave (primary, compressional)
        let p_arrival = distance / self.p_wave.velocity;
        let p_displacement = if self.elapsed > p_arrival {
            let local_time = self.elapsed - p_arrival;
            let amplitude = self.magnitude * (-distance * self.p_wave.amplitude_decay).exp();
            Point3D::new(
                amplitude * (local_time * self.p_wave.frequency * 2.0 * PI).sin(),
                0.0,
                amplitude * (local_time * self.p_wave.frequency * 2.0 * PI).cos(),
            )
        } else {
            Point3D::zero()
        };
        
        // S-wave (secondary, shear)
        let s_arrival = distance / self.s_wave.velocity;
        let s_displacement = if self.elapsed > s_arrival {
            let local_time = self.elapsed - s_arrival;
            let amplitude = self.magnitude * 1.5 * (-distance * self.s_wave.amplitude_decay).exp();
            Point3D::new(
                0.0,
                amplitude * (local_time * self.s_wave.frequency * 2.0 * PI).sin(),
                0.0,
            )
        } else {
            Point3D::zero()
        };
        
        p_displacement + s_displacement
    }
    
    pub fn apply_to_model(&self, model: &mut StagedModel, time: f64) {
        let displacement = self.calculate_displacement(model.transform.position, time);
        model.transform.position += displacement;
    }
}
