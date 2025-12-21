//! IronBeam Phenomena Compositor
//! 
//! Aggregates and manages all environmental simulations.

use crate::ironbeam::phenomena::wind::WindSystem;
use crate::ironbeam::phenomena::solar_wind::SolarWindSystem;
use crate::ironbeam::phenomena::rain::RainSystem;
use crate::ironbeam::phenomena::earthquake::EarthquakeSystem;
use crate::cad::BoundingBox;

pub struct PhenomenaCompositor {
    pub wind: WindSystem,
    pub solar_wind: SolarWindSystem,
    pub rain: Option<RainSystem>,
    pub earthquake: Option<EarthquakeSystem>,
    pub time: f64,
}

impl PhenomenaCompositor {
    pub fn new() -> Self {
        Self {
            wind: WindSystem::new(),
            solar_wind: SolarWindSystem::new(),
            rain: None,
            earthquake: None,
            time: 0.0,
        }
    }

    pub fn enable_rain(&mut self, bounds: BoundingBox) {
        self.rain = Some(RainSystem::new(bounds));
    }

    pub fn trigger_earthquake(&mut self, system: EarthquakeSystem) {
        self.earthquake = Some(system);
    }

    pub fn tick(&mut self, dt: f32) {
        self.time += dt as f64;
        
        if let Some(rain) = &mut self.rain {
            rain.tick(dt, &self.wind, self.time);
        }
        
        if let Some(eq) = &mut self.earthquake {
            eq.elapsed += dt;
            if eq.elapsed > eq.duration {
                self.earthquake = None;
            }
        }
    }
}
