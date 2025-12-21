//! IronBeam Rain System
//! 
//! Simulates precipitation and droplet dynamics.

use crate::geometry3d::Point3D;
use crate::cad::BoundingBox;
use crate::ironbeam::phenomena::wind::WindSystem;

#[derive(Clone, Debug)]
pub enum DropSizeDistribution {
    Uniform(f32),
    MarshallPalmer { n0: f32, lambda: f32 },
}

#[derive(Clone, Debug)]
pub struct RainDrop {
    pub position: Point3D,
    pub velocity: Point3D,
    pub radius: f32,
    pub lifetime: f32,
}

pub struct RainSystem {
    /// Rain intensity (mm/hour)
    pub intensity: f32,
    
    /// Drop size distribution
    pub drop_size: DropSizeDistribution,
    
    /// Wind influence
    pub wind_influence: f32,
    
    /// Active raindrops (for visualization)
    pub drops: Vec<RainDrop>,
    
    /// Spawn bounds
    pub bounds: BoundingBox,
}

impl RainSystem {
    pub fn new(bounds: BoundingBox) -> Self {
        Self {
            intensity: 10.0,
            drop_size: DropSizeDistribution::Uniform(0.002), // 2mm drops
            wind_influence: 1.0,
            drops: Vec::new(),
            bounds,
        }
    }

    fn calculate_terminal_velocity(radius: f32) -> f32 {
        // Empirical approximation: v = 9.4 * (d/2)^0.5
        // radius in meters
        -9.0 * (radius * 1000.0).sqrt()
    }

    fn spawn_drop(&mut self, _wind: &WindSystem, _time: f64) {
        // Random position within bounds top
        let x = self.bounds.min.x + (self.bounds.max.x - self.bounds.min.x) * 0.5; // TODO: Random
        let z = self.bounds.min.z + (self.bounds.max.z - self.bounds.min.z) * 0.5; // TODO: Random
        let y = self.bounds.max.y;
        
        let radius = match self.drop_size {
            DropSizeDistribution::Uniform(r) => r,
            _ => 0.002,
        };

        self.drops.push(RainDrop {
            position: Point3D::new(x, y, z),
            velocity: Point3D::new(0.0, 0.0, 0.0),
            radius,
            lifetime: 10.0,
        });
    }

    pub fn tick(&mut self, dt: f32, wind: &WindSystem, time: f64) {
        // Spawn new drops
        let drops_per_second = self.intensity * 10.0; // Simplified spawn rate
        let spawn_count = (drops_per_second * dt).ceil() as usize;
        
        for _ in 0..spawn_count {
            self.spawn_drop(wind, time);
        }
        
        // Update existing drops
        for drop in &mut self.drops {
            // Terminal velocity based on size
            let terminal_velocity = Self::calculate_terminal_velocity(drop.radius);
            
            // Wind influence
            let wind_force = wind.sample(drop.position, time) * self.wind_influence;
            
            drop.velocity.y = terminal_velocity;
            drop.velocity.x += wind_force.x * dt;
            drop.velocity.z += wind_force.z * dt;
            
            drop.position += drop.velocity * dt;
            drop.lifetime -= dt;
        }
        
        // Remove dead drops
        self.drops.retain(|d| d.lifetime > 0.0 && d.position.y > self.bounds.min.y);
    }
    
    pub fn render(&self) -> String {
        let mut svg = String::new();
        for drop in &self.drops {
            // Render as elongated ellipse based on velocity
            // Simplified for MVP
            svg.push_str(&format!(
                r#"<circle cx="{}" cy="{}" r="{}" fill="rgba(100,150,255,0.6)" />"#,
                drop.position.x, drop.position.y,
                drop.radius * 1000.0 // Scale for visibility
            ));
        }
        svg
    }
}
