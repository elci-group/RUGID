use crate::physics::core::PhysicsState;
use crate::geometry3d::Point3D;
use std::collections::HashMap;
use crate::cell::CellId;

pub struct PhysicsSystem {
    pub gravity: Point3D,
    pub states: HashMap<CellId, PhysicsState>,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            gravity: Point3D::new(0.0, 9.81, 0.0), // Default gravity (down is +Y in screen coords usually, but let's assume +Y is down)
            states: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: CellId, state: PhysicsState) {
        self.states.insert(id, state);
    }

    pub fn tick(&mut self, dt: f32) {
        // 1. Apply Forces & Integrate
        for state in self.states.values_mut() {
            if state.is_static {
                continue;
            }

            // Apply Gravity (F = mg)
            // a += g
            state.acceleration.x += self.gravity.x;
            state.acceleration.y += self.gravity.y;
            state.acceleration.z += self.gravity.z;

            // Integrate Velocity: v += a * dt
            state.velocity.x += state.acceleration.x * dt;
            state.velocity.y += state.acceleration.y * dt;
            state.velocity.z += state.acceleration.z * dt;

            // Integrate Position: p += v * dt
            state.position.x += state.velocity.x * dt;
            state.position.y += state.velocity.y * dt;
            state.position.z += state.velocity.z * dt;

            // Reset acceleration (forces are applied per frame)
            state.acceleration = Point3D::new(0.0, 0.0, 0.0);
        }
        
        // 2. Resolve Collisions (Placeholder)
        // self.resolve_collisions();
    }
    
    /// Get the calculated translation offset for a body (displacement from origin)
    pub fn get_deltas(&self) -> HashMap<CellId, Point3D> {
        let mut deltas = HashMap::new();
        for (id, state) in &self.states {
            if !state.is_static {
                deltas.insert(*id, state.position);
            }
        }
        deltas
    }
}
