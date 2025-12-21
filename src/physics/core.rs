use crate::geometry3d::Point3D;

/// Physical properties of a material
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    /// Bounciness (0.0 = no bounce, 1.0 = perfect elastic)
    pub restitution: f32,
    /// Surface friction coefficient (0.0 = ice, 1.0 = rubber)
    pub friction: f32,
    /// Surface tension / Adhesion strength (force per unit distance)
    pub surface_tension: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            restitution: 0.5,
            friction: 0.5,
            surface_tension: 0.0,
        }
    }
}

impl Material {
    pub fn rubber() -> Self {
        Self { restitution: 0.8, friction: 0.9, surface_tension: 0.1 }
    }
    
    pub fn metal() -> Self {
        Self { restitution: 0.2, friction: 0.4, surface_tension: 0.0 }
    }
    
    pub fn sticky() -> Self {
        Self { restitution: 0.0, friction: 1.0, surface_tension: 2.0 }
    }
}

/// Magnetic properties
#[derive(Clone, Debug, PartialEq)]
pub struct MagneticDipole {
    /// Strength of the magnet
    pub strength: f32,
    /// Orientation of North pole relative to body rotation
    pub orientation: Point3D,
}

/// State for physics simulation
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsState {
    /// Mass in kg (0.0 means static/immovable)
    pub mass: f32,
    /// Position offset (from initial layout)
    pub position: Point3D,
    /// Linear velocity (units/frame)
    pub velocity: Point3D,
    /// Linear acceleration (units/frame^2)
    pub acceleration: Point3D,
    /// Material properties
    pub material: Material,
    /// Optional magnetic properties
    pub magnetic_properties: Option<MagneticDipole>,
    /// Is this body static? (Optimization flag)
    pub is_static: bool,
}

impl PhysicsState {
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            position: Point3D::new(0.0, 0.0, 0.0),
            velocity: Point3D::new(0.0, 0.0, 0.0),
            acceleration: Point3D::new(0.0, 0.0, 0.0),
            material: Material::default(),
            magnetic_properties: None,
            is_static: mass <= 0.0,
        }
    }
    
    pub fn with_material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }
    
    pub fn with_magnetism(mut self, strength: f32, axis: Point3D) -> Self {
        self.magnetic_properties = Some(MagneticDipole {
            strength,
            orientation: axis,
        });
        self
    }
    
    /// Apply a force (F = ma -> a += F/m)
    pub fn apply_force(&mut self, force: Point3D) {
        if !self.is_static && self.mass > 0.0 {
            self.acceleration.x += force.x / self.mass;
            self.acceleration.y += force.y / self.mass;
            self.acceleration.z += force.z / self.mass;
        }
    }
}
