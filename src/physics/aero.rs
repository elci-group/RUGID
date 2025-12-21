/// Aerodynamics Module
///
/// Models the interaction between solid bodies and the fluid atmosphere.
/// Implements Drag and Lift equations.

use crate::geometry3d::Point3D;
use crate::physics::atmosphere::Atmosphere;

/// Drag Coefficient (Cd) for common shapes
/// Values based on standard fluid dynamics data (Re ~ 10^4)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragCoefficient {
    Sphere,          // 0.47
    HalfSphere,      // 0.42
    Cone,            // 0.50
    Cube,            // 1.05
    AngledCube,      // 0.80
    LongCylinder,    // 0.82
    ShortCylinder,   // 1.15
    Streamlined,     // 0.04 (Airfoil/Teardrop)
    Custom(f32),
}

impl DragCoefficient {
    pub fn value(&self) -> f32 {
        match self {
            Self::Sphere => 0.47,
            Self::HalfSphere => 0.42,
            Self::Cone => 0.50,
            Self::Cube => 1.05,
            Self::AngledCube => 0.80,
            Self::LongCylinder => 0.82,
            Self::ShortCylinder => 1.15,
            Self::Streamlined => 0.04,
            Self::Custom(v) => *v,
        }
    }
}

/// Properties required for aerodynamic calculations
#[derive(Clone, Debug)]
pub struct AeroBody {
    pub cd: DragCoefficient,
    pub area: f32, // Cross-sectional area (m^2)
    pub cl: f32,   // Lift coefficient (usually 0 for symmetric non-wings)
}

impl Default for AeroBody {
    fn default() -> Self {
        Self {
            cd: DragCoefficient::Sphere,
            area: 1.0,
            cl: 0.0,
        }
    }
}

/// Calculate the Drag Force vector
/// Fd = 1/2 * rho * v^2 * Cd * A * -dir
pub fn calculate_drag(
    velocity: Point3D,
    density: f32,
    body: &AeroBody,
) -> Point3D {
    let speed = velocity.length();
    if speed <= 0.0001 {
        return Point3D::zero();
    }

    let cd = body.cd.value();
    let q = 0.5 * density * speed * speed; // Dynamic Pressure
    let force_magnitude = q * cd * body.area;

    // Drag opposes velocity
    let direction = velocity.normalize();
    
    Point3D::new(
        -direction.x * force_magnitude,
        -direction.y * force_magnitude,
        -direction.z * force_magnitude,
    )
}

/// Calculate the Lift Force vector
/// Fl = 1/2 * rho * v^2 * Cl * A * up_dir
/// Simplified: Lift is perpendicular to velocity and "wing" orientation.
/// For this implementation, we assume lift acts in global +Y (up) if velocity is horizontal,
/// or generally perpendicular to velocity vector in the vertical plane.
pub fn calculate_lift(
    velocity: Point3D,
    density: f32,
    body: &AeroBody,
) -> Point3D {
    if body.cl.abs() < 0.0001 {
        return Point3D::zero();
    }
    
    let speed = velocity.length();
    if speed <= 0.0001 {
        return Point3D::zero();
    }

    let q = 0.5 * density * speed * speed;
    let force_magnitude = q * body.cl * body.area;

    // Simplified Lift Direction: Perpendicular to velocity, generally "Up"
    // Cross product of Velocity and Right-Vector?
    // Let's assume "Up" is +Y.
    // If velocity is (1, 0, 0), lift is (0, 1, 0).
    // If velocity is (0, -1, 0) (falling), lift depends on angle of attack.
    // This is a simplified model. We'll assume lift opposes gravity component perpendicular to velocity?
    // Or just strictly Perpendicular to Velocity in the vertical plane.
    
    let right = velocity.cross(&Point3D::new(0.0, 1.0, 0.0));
    let mut lift_dir = right.cross(&velocity).normalize();
    
    // If velocity is vertical, cross product with Y is zero.
    if lift_dir.length() < 0.001 {
        // Fallback or zero lift
        return Point3D::zero();
    }
    
    // Ensure lift points generally up
    if lift_dir.y < 0.0 {
        lift_dir = lift_dir * -1.0;
    }

    lift_dir * force_magnitude
}
