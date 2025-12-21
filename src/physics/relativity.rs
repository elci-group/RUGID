//! Relativity Module
//!
//! Implements Special and General Relativity effects:
//! - Lorentz factor (γ)
//! - Relativistic momentum
//! - Length contraction
//! - Time dilation
//! - Gravitational time dilation
//! - Geodesic motion

use crate::geometry3d::Point3D;
use crate::cell::CellId;
use std::collections::HashMap;

/// Speed of light (scaled for UI simulation)
/// In a real simulation this would be ~3e8 m/s
/// Here we use a value that makes relativistic effects visible at UI speeds
pub const SPEED_OF_LIGHT: f32 = 1000.0;
pub const C_SQUARED: f32 = SPEED_OF_LIGHT * SPEED_OF_LIGHT;

/// Relativistic state of a body
#[derive(Clone, Debug, PartialEq)]
pub struct RelativisticState {
    /// Rest mass (kg)
    pub rest_mass: f32,
    /// Velocity (units/frame)
    pub velocity: Point3D,
    /// Position
    pub position: Point3D,
    /// Proper time (τ) - local time experienced by the body
    pub proper_time: f64,
    /// Is this body a massive gravitational source?
    pub is_massive: bool,
    /// Schwarzschild radius (if massive)
    pub schwarzschild_radius: f32,
}

impl Default for RelativisticState {
    fn default() -> Self {
        Self {
            rest_mass: 1.0,
            velocity: Point3D::new(0.0, 0.0, 0.0),
            position: Point3D::new(0.0, 0.0, 0.0),
            proper_time: 0.0,
            is_massive: false,
            schwarzschild_radius: 0.0,
        }
    }
}

impl RelativisticState {
    /// Create a relativistic body
    pub fn new(mass: f32, position: Point3D) -> Self {
        Self {
            rest_mass: mass,
            position,
            ..Default::default()
        }
    }

    /// Create a massive body (star, black hole)
    pub fn massive(mass: f32, position: Point3D) -> Self {
        // r_s = 2GM/c²
        let gravitational_constant = 6.674e-11;
        let schwarzschild = 2.0 * gravitational_constant * mass / C_SQUARED;
        Self {
            rest_mass: mass,
            position,
            is_massive: true,
            schwarzschild_radius: schwarzschild,
            ..Default::default()
        }
    }

    /// Calculate speed as fraction of c (β = v/c)
    pub fn beta(&self) -> f32 {
        let v = self.velocity.magnitude();
        (v / SPEED_OF_LIGHT).min(0.9999) // Cap to avoid NaN
    }

    /// Calculate Lorentz factor: γ = 1/√(1 - v²/c²)
    pub fn lorentz_factor(&self) -> f32 {
        let beta = self.beta();
        1.0 / (1.0 - beta * beta).sqrt()
    }

    /// Relativistic momentum: p = γmv
    pub fn relativistic_momentum(&self) -> Point3D {
        let gamma = self.lorentz_factor();
        Point3D::new(
            gamma * self.rest_mass * self.velocity.x,
            gamma * self.rest_mass * self.velocity.y,
            gamma * self.rest_mass * self.velocity.z,
        )
    }

    /// Relativistic energy: E = γmc²
    pub fn relativistic_energy(&self) -> f32 {
        self.lorentz_factor() * self.rest_mass * C_SQUARED
    }

    /// Length contraction factor (objects appear shorter in motion direction)
    pub fn length_contraction(&self) -> f32 {
        1.0 / self.lorentz_factor()
    }

    /// Time dilation factor (proper time ticks slower)
    pub fn time_dilation(&self) -> f32 {
        1.0 / self.lorentz_factor()
    }
}

/// Helper functions for relativity calculations
pub struct RelativityHelpers;

impl RelativityHelpers {
    /// Lorentz factor from velocity
    pub fn gamma(velocity: &Point3D) -> f32 {
        let v_sq = velocity.x * velocity.x + velocity.y * velocity.y + velocity.z * velocity.z;
        let beta_sq = v_sq / C_SQUARED;
        if beta_sq >= 1.0 {
            return 1000.0; // Very high gamma, approaching infinity
        }
        1.0 / (1.0 - beta_sq).sqrt()
    }

    /// Gravitational time dilation factor at distance r from mass M
    /// γ_grav = 1/√(1 - r_s/r) where r_s = 2GM/c²
    pub fn gravitational_time_dilation(schwarzschild_radius: f32, distance: f32) -> f32 {
        if distance <= schwarzschild_radius {
            return 1000.0; // Inside event horizon - extreme dilation
        }
        let ratio = schwarzschild_radius / distance;
        1.0 / (1.0 - ratio).sqrt()
    }

    /// Relativistic Doppler shift factor
    /// For approaching: f_obs = f_src * √((1 + β)/(1 - β))
    /// For receding:    f_obs = f_src * √((1 - β)/(1 + β))
    pub fn doppler_factor(relative_velocity: f32) -> f32 {
        let beta = (relative_velocity / SPEED_OF_LIGHT).clamp(-0.9999, 0.9999);
        ((1.0 + beta) / (1.0 - beta)).sqrt()
    }

    /// Relativistic velocity addition: u' = (u + v)/(1 + uv/c²)
    pub fn velocity_addition(u: f32, v: f32) -> f32 {
        (u + v) / (1.0 + u * v / C_SQUARED)
    }
}

/// The Relativity System
pub struct RelativitySystem {
    /// All relativistic bodies
    pub bodies: HashMap<CellId, RelativisticState>,
    /// Global time (coordinate time)
    pub coordinate_time: f64,
    /// Massive bodies that create gravitational wells
    pub massive_bodies: Vec<(Point3D, f32, f32)>, // (position, mass, schwarzschild_radius)
}

impl RelativitySystem {
    pub fn new() -> Self {
        Self {
            bodies: HashMap::new(),
            coordinate_time: 0.0,
            massive_bodies: Vec::new(),
        }
    }

    pub fn register(&mut self, id: CellId, state: RelativisticState) {
        if state.is_massive {
            self.massive_bodies.push((
                state.position,
                state.rest_mass,
                state.schwarzschild_radius,
            ));
        }
        self.bodies.insert(id, state);
    }

    /// Main simulation tick
    pub fn tick(&mut self, dt: f32) {
        self.coordinate_time += dt as f64;

        let ids: Vec<CellId> = self.bodies.keys().cloned().collect();

        for id in &ids {
            if let Some(body) = self.bodies.get_mut(id) {
                // Calculate total time dilation (kinematic + gravitational)
                let gamma_kinematic = body.lorentz_factor();

                let mut gamma_gravitational = 1.0;
                for (massive_pos, _mass, r_s) in &self.massive_bodies {
                    let dx = body.position.x - massive_pos.x;
                    let dy = body.position.y - massive_pos.y;
                    let dz = body.position.z - massive_pos.z;
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                    gamma_gravitational *= RelativityHelpers::gravitational_time_dilation(*r_s, dist);
                }

                let total_gamma = gamma_kinematic * gamma_gravitational;
                let local_dt = dt / total_gamma;

                // Update proper time
                body.proper_time += local_dt as f64;

                // Update position (using proper time for integration)
                body.position.x += body.velocity.x * local_dt;
                body.position.y += body.velocity.y * local_dt;
                body.position.z += body.velocity.z * local_dt;
            }
        }
    }

    /// Apply force using relativistic momentum
    pub fn apply_force(&mut self, id: CellId, force: Point3D, dt: f32) {
        if let Some(body) = self.bodies.get_mut(&id) {
            // dp = F * dt
            // p = γmv, so dv requires accounting for γ changing with v
            // Simplified: dv ≈ F*dt / (γm)
            let gamma = body.lorentz_factor();
            let accel = Point3D::new(
                force.x / (gamma * body.rest_mass),
                force.y / (gamma * body.rest_mass),
                force.z / (gamma * body.rest_mass),
            );
            body.velocity.x += accel.x * dt;
            body.velocity.y += accel.y * dt;
            body.velocity.z += accel.z * dt;

            // Cap velocity at c
            let v = body.velocity.magnitude();
            if v > SPEED_OF_LIGHT * 0.9999 {
                let scale = SPEED_OF_LIGHT * 0.9999 / v;
                body.velocity.x *= scale;
                body.velocity.y *= scale;
                body.velocity.z *= scale;
            }
        }
    }

    /// Get the current proper time of a body (for animations)
    pub fn get_proper_time(&self, id: CellId) -> Option<f64> {
        self.bodies.get(&id).map(|b| b.proper_time)
    }

    /// Get length contraction factor for rendering
    pub fn get_length_contraction(&self, id: CellId) -> Option<(f32, Point3D)> {
        self.bodies.get(&id).map(|b| {
            let contraction = b.length_contraction();
            let direction = if b.velocity.magnitude() > 0.001 {
                Point3D::new(
                    b.velocity.x / b.velocity.magnitude(),
                    b.velocity.y / b.velocity.magnitude(),
                    b.velocity.z / b.velocity.magnitude(),
                )
            } else {
                Point3D::new(1.0, 0.0, 0.0)
            };
            (contraction, direction)
        })
    }

    /// Get Doppler shift factor for a body relative to viewer at origin
    pub fn get_doppler_shift(&self, id: CellId) -> Option<f32> {
        self.bodies.get(&id).map(|b| {
            // Radial velocity (towards/away from origin)
            let dist = b.position.magnitude();
            if dist < 0.001 {
                return 1.0;
            }
            let radial_velocity = (b.position.x * b.velocity.x
                + b.position.y * b.velocity.y
                + b.position.z * b.velocity.z)
                / dist;
            RelativityHelpers::doppler_factor(radial_velocity)
        })
    }
}
