use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt;

static CELL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellId(pub u64);

impl fmt::Debug for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cell({})", self.0)
    }
}

impl CellId {
    pub fn next() -> Self {
        CellId(CELL_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
    
    pub fn generate() -> Self {
        Self::next()
    }
}

use crate::geometry::VectorRegion;

/// 3D rotation state for cells that represent 3D objects
#[derive(Clone, Debug, PartialEq)]
pub struct Rotation3DState {
    /// Current rotation (pitch, yaw, roll) in degrees
    pub current: crate::geometry3d::Rotation3D,
    /// Rotation velocity (degrees per frame) for each axis
    pub velocity: crate::geometry3d::Rotation3D,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Component animation angle (e.g., wheel rotation) in radians
    pub component_angle: f32,
    /// Component animation speed (radians per frame)
    pub component_speed: f32,
}

impl Rotation3DState {
    /// Create new rotation state with given velocities
    pub fn new(x_speed: f32, y_speed: f32, z_speed: f32) -> Self {
        Self {
            current: crate::geometry3d::Rotation3D::default(),
            velocity: crate::geometry3d::Rotation3D {
                pitch: x_speed,
                yaw: y_speed,
                roll: z_speed,
            },
            opacity: 1.0,
            component_angle: 0.0,
            component_speed: 0.0,
        }
    }
    
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }
    
    /// Set component animation speed (e.g., for spinning wheels)
    pub fn with_component_speed(mut self, speed: f32) -> Self {
        self.component_speed = speed;
        self
    }
    
    /// Update rotation based on velocity (call each frame)
    pub fn update(&mut self) {
        self.current.pitch += self.velocity.pitch;
        self.current.yaw += self.velocity.yaw;
        self.current.roll += self.velocity.roll;
        
        // Keep angles in 0-360 range
        self.current.pitch %= 360.0;
        self.current.yaw %= 360.0;
        self.current.roll %= 360.0;
        
        // Update component animation
        self.component_angle += self.component_speed;
        if self.component_angle > std::f32::consts::TAU {
            self.component_angle -= std::f32::consts::TAU;
        }
    }
}

/// A cell is an addressable, bounded rendering region.
///
/// Each cell defines:
/// * Stable identity
/// * Vector geometry contract (Phase 6)
/// * Layer constraints (Phase 4)
/// * Behaviour flags (Phase 4)
/// * Subscriptions to signals (Phase 2)
/// * Optional morphism state for shape transformations
/// * Optional 3D rotation state for 3D objects
///
/// Cells do not store mutable render state.
#[derive(Clone, Debug)]
pub struct Cell {
    pub id: CellId,
    pub geometry: VectorRegion,
    /// Optional morphism state for shape transformation animations
    pub morphism: Option<crate::morphism::MorphismState>,
    /// Optional 3D rotation state for 3D objects
    pub rotation_3d: Option<Rotation3DState>,
    /// Optional physics state
    pub physics_state: Option<crate::physics::core::PhysicsState>,
}

impl Cell {
    pub fn new(geometry: VectorRegion) -> Self {
        Self {
            id: CellId::next(),
            geometry,
            morphism: None,
            rotation_3d: None,
            physics_state: None,
        }
    }
    
    /// Create a cell with initial morphism state
    pub fn with_shape(geometry: VectorRegion, shape: crate::morphism::ShapeType) -> Self {
        Self {
            id: CellId::next(),
            geometry,
            morphism: Some(crate::morphism::MorphismState::new(shape)),
            rotation_3d: None,
            physics_state: None,
        }
    }
    
    /// Create a cell with 3D rotation
    pub fn with_3d_rotation(geometry: VectorRegion, x_speed: f32, y_speed: f32, z_speed: f32) -> Self {
        Self {
            id: CellId::next(),
            geometry,
            morphism: None,
            rotation_3d: Some(Rotation3DState::new(x_speed, y_speed, z_speed)),
            physics_state: None,
        }
    }

    /// Create a cell with physics
    pub fn with_physics(geometry: VectorRegion, mass: f32) -> Self {
        Self {
            id: CellId::next(),
            geometry,
            morphism: None,
            rotation_3d: None,
            physics_state: Some(crate::physics::core::PhysicsState::new(mass)),
        }
    }

    /// Set a specific ID (for cases where ID comes from ontology resolver)
    pub fn with_id(mut self, id: CellId) -> Self {
        self.id = id;
        self
    }
}
