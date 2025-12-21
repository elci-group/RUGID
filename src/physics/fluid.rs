use rayon::prelude::*;
use std::collections::HashMap;
use crate::cell::CellId;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct FluidParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub fx: f32,
    pub fy: f32,
    pub density: f32,
    pub pressure: f32,
}

impl FluidParticle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x, y,
            vx: 0.0, vy: 0.0,
            fx: 0.0, fy: 0.0,
            density: 0.0,
            pressure: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FluidConfig {
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub smoothing_radius: f32, // h
    pub target_density: f32,   // rho0
    pub pressure_multiplier: f32, // k
    pub viscosity: f32,        // mu
    pub damping: f32,
}

impl Default for FluidConfig {
    fn default() -> Self {
        Self {
            gravity_x: 0.0,
            gravity_y: 980.0, // Pixel units approx
            smoothing_radius: 20.0,
            target_density: 0.5,
            pressure_multiplier: 2000.0,
            viscosity: 50.0,
            damping: 0.5,
        }
    }
}

pub struct FluidDomain {
    pub particles: Vec<FluidParticle>,
    pub config: FluidConfig,
    pub width: f32,
    pub height: f32,
}

impl FluidDomain {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            particles: Vec::new(),
            config: FluidConfig::default(),
            width,
            height,
        }
    }

    pub fn add_particle(&mut self, x: f32, y: f32) {
        self.particles.push(FluidParticle::new(x, y));
    }

    /// SPH Simulation Step
    pub fn tick(&mut self, dt: f32) {
        let h = self.config.smoothing_radius;
        let h2 = h * h;
        let poly6_coeff = 315.0 / (64.0 * PI * h.powi(9));
        let spiky_grad_coeff = -45.0 / (PI * h.powi(6));
        let viscosity_coeff = 45.0 / (PI * h.powi(6));

        // 1. Compute Density & Pressure
        // We can't easily mutate in parallel while reading all, so we compute densities into a separate vec first?
        // Or use interior mutability? Or just collect.
        // For simplicity in this iteration, we'll do a multi-pass approach.
        
        // Pass 1: Density
        let densities: Vec<f32> = self.particles.par_iter().map(|p_i| {
            let mut density = 0.0;
            for p_j in &self.particles {
                let dx = p_i.x - p_j.x;
                let dy = p_i.y - p_j.y;
                let r2 = dx*dx + dy*dy;
                
                if r2 < h2 {
                    let diff = h2 - r2;
                    density += diff * diff * diff;
                }
            }
            density * poly6_coeff
        }).collect();

        // Update particles with density and pressure
        for (i, p) in self.particles.iter_mut().enumerate() {
            p.density = densities[i].max(self.config.target_density);
            p.pressure = self.config.pressure_multiplier * (p.density - self.config.target_density);
        }

        // Pass 2: Forces (Pressure + Viscosity)
        // We need read access to all particles, write access to forces.
        // We can compute forces in parallel and collect them.
        let forces: Vec<(f32, f32)> = self.particles.par_iter().enumerate().map(|(i, p_i)| {
            let mut fx = 0.0;
            let mut fy = 0.0;

            for (j, p_j) in self.particles.iter().enumerate() {
                if i == j { continue; }

                let dx = p_i.x - p_j.x;
                let dy = p_i.y - p_j.y;
                let r = (dx*dx + dy*dy).sqrt();

                if r > 0.0 && r < h {
                    // Pressure Force
                    // Fp = -normalize(r) * mass * (pi + pj) / (2 * rho_j) * W_spiky
                    // Simplified SPH:
                    let force_pressure = (p_i.pressure + p_j.pressure) / (2.0 * p_j.density);
                    let diff = h - r;
                    let spiky = spiky_grad_coeff * diff * diff;
                    
                    fx += -dx / r * force_pressure * spiky;
                    fy += -dy / r * force_pressure * spiky;

                    // Viscosity Force
                    // Fv = mu * (vj - vi) / rho_j * W_visc
                    let visc_term = self.config.viscosity / p_j.density * viscosity_coeff * (h - r);
                    fx += (p_j.vx - p_i.vx) * visc_term;
                    fy += (p_j.vy - p_i.vy) * visc_term;
                }
            }
            (fx, fy)
        }).collect();

        // Pass 3: Integration & Boundary Conditions
        for (i, p) in self.particles.iter_mut().enumerate() {
            let (force_x, force_y) = forces[i];
            
            // Apply forces
            p.fx = force_x + self.config.gravity_x * p.density;
            p.fy = force_y + self.config.gravity_y * p.density;

            // Verlet / Euler Integration
            p.vx += (p.fx / p.density) * dt;
            p.vy += (p.fy / p.density) * dt;

            p.x += p.vx * dt;
            p.y += p.vy * dt;

            // Boundary Conditions (Simple Box)
            let damping = self.config.damping;
            if p.x < 0.0 { p.x = 0.0; p.vx *= -damping; }
            if p.x > self.width { p.x = self.width; p.vx *= -damping; }
            if p.y < 0.0 { p.y = 0.0; p.vy *= -damping; }
            if p.y > self.height { p.y = self.height; p.vy *= -damping; }
        }
    }
}

pub struct FluidSystem {
    pub domains: HashMap<CellId, FluidDomain>,
}

impl FluidSystem {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: CellId, width: f32, height: f32) {
        let mut domain = FluidDomain::new(width, height);
        // Add some test particles
        for i in 0..10 {
            for j in 0..10 {
                domain.add_particle(100.0 + i as f32 * 10.0, 100.0 + j as f32 * 10.0);
            }
        }
        self.domains.insert(id, domain);
    }

    pub fn tick(&mut self) {
        // Parallel update of all domains
        self.domains.par_iter_mut().for_each(|(_, domain)| {
            domain.tick(0.016); // Fixed dt
        });
    }
    
    pub fn get_particles(&self, id: CellId) -> Option<&Vec<FluidParticle>> {
        self.domains.get(&id).map(|d| &d.particles)
    }
}
