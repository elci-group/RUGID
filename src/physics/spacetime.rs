//! Spacetime Curvature Visualization
//!
//! Implements a deformable spacetime grid that visualizes gravitational
//! curvature caused by massive bodies. Includes embedding diagrams
//! and geodesic path calculation.

use crate::geometry3d::Point3D;
use crate::cell::CellId;

/// A single node in the spacetime grid
#[derive(Clone, Debug)]
pub struct SpacetimeNode {
    /// Current position (deformed)
    pub position: Point3D,
    /// Rest position (flat spacetime)
    pub rest_position: Point3D,
    /// Velocity (for spring dynamics)
    pub velocity: Point3D,
    /// Local time dilation factor
    pub time_dilation: f32,
    /// Metric tensor component g_tt (simplified)
    pub metric_tt: f32,
}

impl SpacetimeNode {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Point3D::new(x, y, z),
            rest_position: Point3D::new(x, y, z),
            velocity: Point3D::new(0.0, 0.0, 0.0),
            time_dilation: 1.0,
            metric_tt: -1.0, // Flat spacetime
        }
    }
}

/// Configuration for the spacetime grid
#[derive(Clone, Debug)]
pub struct SpacetimeConfig {
    /// Grid resolution (nodes per axis)
    pub resolution: usize,
    /// Physical size of the grid
    pub size: f32,
    /// Spring stiffness (for recovery to rest)
    pub stiffness: f32,
    /// Damping factor
    pub damping: f32,
    /// Maximum deformation
    pub max_deformation: f32,
}

impl Default for SpacetimeConfig {
    fn default() -> Self {
        Self {
            resolution: 20,
            size: 100.0,
            stiffness: 50.0,
            damping: 0.9,
            max_deformation: 50.0,
        }
    }
}

/// Represents a massive body that warps spacetime
#[derive(Clone, Debug)]
pub struct MassiveBody {
    pub id: CellId,
    pub position: Point3D,
    pub mass: f32,
    /// Schwarzschild radius: r_s = 2GM/c²
    pub schwarzschild_radius: f32,
}

impl MassiveBody {
    pub fn new(id: CellId, position: Point3D, mass: f32) -> Self {
        // Using scaled units where G = 1 and c = 1000
        let c_sq = 1_000_000.0;
        let schwarzschild = 2.0 * mass / c_sq;
        Self {
            id,
            position,
            mass,
            schwarzschild_radius: schwarzschild,
        }
    }
}

/// The Spacetime Curvature System
pub struct SpacetimeSystem {
    /// Grid of spacetime nodes
    pub grid: Vec<Vec<SpacetimeNode>>,
    /// Configuration
    pub config: SpacetimeConfig,
    /// Massive bodies causing curvature
    pub massive_bodies: Vec<MassiveBody>,
    /// Gravitational constant (scaled)
    pub g_constant: f32,
}

impl SpacetimeSystem {
    pub fn new(config: SpacetimeConfig) -> Self {
        let mut grid = Vec::with_capacity(config.resolution);
        let step = config.size / config.resolution as f32;
        let half = config.size / 2.0;

        for i in 0..config.resolution {
            let mut row = Vec::with_capacity(config.resolution);
            for j in 0..config.resolution {
                let x = i as f32 * step - half;
                let y = j as f32 * step - half;
                row.push(SpacetimeNode::new(x, y, 0.0));
            }
            grid.push(row);
        }

        Self {
            grid,
            config,
            massive_bodies: Vec::new(),
            g_constant: 1.0, // Scaled units
        }
    }

    /// Register a massive body
    pub fn register_mass(&mut self, body: MassiveBody) {
        self.massive_bodies.push(body);
    }

    /// Update mass position
    pub fn update_mass_position(&mut self, id: CellId, new_position: Point3D) {
        for body in &mut self.massive_bodies {
            if body.id == id {
                body.position = new_position;
                break;
            }
        }
    }

    /// Main simulation tick
    pub fn tick(&mut self, dt: f32) {
        let res = self.config.resolution;

        // Calculate gravitational potential and deformation
        for i in 0..res {
            for j in 0..res {
                let node = &self.grid[i][j];
                let rest = node.rest_position;

                // Calculate potential from all massive bodies
                let mut potential: f32 = 0.0;
                for body in &self.massive_bodies {
                    let dx = rest.x - body.position.x;
                    let dy = rest.y - body.position.y;
                    let dist = (dx * dx + dy * dy).sqrt().max(0.1);

                    // Φ = -GM/r
                    potential -= self.g_constant * body.mass / dist;
                }

                // Deformation in Z (embedding diagram "dip")
                // z = Φ (scaled for visualization)
                let target_z = (potential * 0.01).clamp(-self.config.max_deformation, 0.0);

                // Spring force towards target
                let node = &mut self.grid[i][j];
                let force_z = self.config.stiffness * (target_z - node.position.z);
                node.velocity.z += force_z * dt;
                node.velocity.z *= self.config.damping;
                node.position.z += node.velocity.z * dt;

                // Calculate time dilation at this point
                // γ = 1/√(1 - 2GM/(rc²)) ≈ 1 + GM/(rc²) for weak fields
                let mut dilation = 1.0;
                for body in &self.massive_bodies {
                    let dx = rest.x - body.position.x;
                    let dy = rest.y - body.position.y;
                    let dist = (dx * dx + dy * dy).sqrt().max(body.schwarzschild_radius + 0.1);
                    let ratio = body.schwarzschild_radius / dist;
                    if ratio < 1.0 {
                        dilation *= 1.0 / (1.0 - ratio).sqrt();
                    }
                }
                node.time_dilation = dilation;
                node.metric_tt = -1.0 / (dilation * dilation); // Simplified metric component
            }
        }
    }

    /// Generate vertices for rendering the grid as a mesh
    pub fn generate_mesh_vertices(&self) -> Vec<Point3D> {
        let mut vertices = Vec::new();
        let res = self.config.resolution;

        for i in 0..res {
            for j in 0..res {
                vertices.push(self.grid[i][j].position);
            }
        }

        vertices
    }

    /// Generate indices for a triangle mesh (for rendering)
    pub fn generate_mesh_indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        let res = self.config.resolution as u32;

        for i in 0..(res - 1) {
            for j in 0..(res - 1) {
                let tl = i * res + j;
                let tr = i * res + j + 1;
                let bl = (i + 1) * res + j;
                let br = (i + 1) * res + j + 1;

                // First triangle
                indices.push(tl);
                indices.push(bl);
                indices.push(tr);

                // Second triangle
                indices.push(tr);
                indices.push(bl);
                indices.push(br);
            }
        }

        indices
    }

    /// Calculate a geodesic path through the curved spacetime
    pub fn trace_geodesic(
        &self,
        start: Point3D,
        direction: Point3D,
        steps: usize,
        step_size: f32,
    ) -> Vec<Point3D> {
        let mut path = Vec::with_capacity(steps);
        let mut pos = start;
        let mut vel = direction.normalize();

        for _ in 0..steps {
            path.push(pos);

            // Calculate gradient of potential at current position
            let mut gradient = Point3D::new(0.0, 0.0, 0.0);
            for body in &self.massive_bodies {
                let dx = pos.x - body.position.x;
                let dy = pos.y - body.position.y;
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt().max(0.1);

                // ∇Φ = GM * r / r³
                let factor = self.g_constant * body.mass / (dist_sq * dist);
                gradient.x += factor * dx;
                gradient.y += factor * dy;
            }

            // Geodesic equation (simplified): acceleration = -∇Φ
            vel.x -= gradient.x * step_size;
            vel.y -= gradient.y * step_size;

            // Normalize to maintain constant "speed of light"
            let v_mag = (vel.x * vel.x + vel.y * vel.y).sqrt();
            if v_mag > 0.001 {
                vel.x /= v_mag;
                vel.y /= v_mag;
            }

            // Update position
            pos.x += vel.x * step_size;
            pos.y += vel.y * step_size;

            // Get Z from grid interpolation (for 3D visualization)
            if let Some(z) = self.sample_z(pos.x, pos.y) {
                pos.z = z;
            }
        }

        path
    }

    /// Sample the Z value at a point (bilinear interpolation)
    fn sample_z(&self, x: f32, y: f32) -> Option<f32> {
        let half = self.config.size / 2.0;
        let step = self.config.size / self.config.resolution as f32;

        let xi = ((x + half) / step) as usize;
        let yi = ((y + half) / step) as usize;

        if xi >= self.config.resolution - 1 || yi >= self.config.resolution - 1 {
            return None;
        }

        // Bilinear interpolation
        let fx = ((x + half) / step) - xi as f32;
        let fy = ((y + half) / step) - yi as f32;

        let z00 = self.grid[xi][yi].position.z;
        let z10 = self.grid[xi + 1][yi].position.z;
        let z01 = self.grid[xi][yi + 1].position.z;
        let z11 = self.grid[xi + 1][yi + 1].position.z;

        let z = z00 * (1.0 - fx) * (1.0 - fy)
            + z10 * fx * (1.0 - fy)
            + z01 * (1.0 - fx) * fy
            + z11 * fx * fy;

        Some(z)
    }

    /// Get time dilation at a point (for rendering clock effects)
    pub fn get_time_dilation_at(&self, x: f32, y: f32) -> f32 {
        let half = self.config.size / 2.0;
        let step = self.config.size / self.config.resolution as f32;

        let xi = ((x + half) / step) as usize;
        let yi = ((y + half) / step) as usize;

        if xi >= self.config.resolution || yi >= self.config.resolution {
            return 1.0;
        }

        self.grid[xi][yi].time_dilation
    }
}
