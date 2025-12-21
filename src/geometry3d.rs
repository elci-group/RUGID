//! 3D Geometry Module
//!
//! Provides 3D point representation, transformations, and perspective projection

use std::f32::consts::PI;

/// 3D point in space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }
    
    /// Linear interpolation between two 3D points
    pub fn lerp(a: Point3D, b: Point3D, t: f32) -> Point3D {
        Point3D {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
            z: a.z + (b.z - a.z) * t,
        }
    }
    
    /// Project from 3D to 2D using perspective
    /// 
    /// Uses perspective projection: scale = focal_length / (focal_length + z)
    /// Objects farther away (negative Z) appear smaller
    pub fn project(&self, focal_length: f32) -> (f32, f32) {
        let scale = focal_length / (focal_length - self.z);
        (self.x * scale, self.y * scale)
    }
    
    /// Calculate vector length (magnitude)
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Alias for length()
    pub fn magnitude(&self) -> f32 {
        self.length()
    }
    
    /// Normalize vector to unit length
    pub fn normalize(&self) -> Point3D {
        let len = self.length();
        if len > 0.0 {
            Point3D {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    /// Dot product
    pub fn dot(&self, other: &Point3D) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    /// Calculate midpoint between two points
    pub fn midpoint(&self, other: Point3D) -> Point3D {
        Point3D {
            x: (self.x + other.x) * 0.5,
            y: (self.y + other.y) * 0.5,
            z: (self.z + other.z) * 0.5,
        }
    }

    /// Cross product
    pub fn cross(&self, other: &Point3D) -> Point3D {
        Point3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl std::ops::Add for Point3D {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::Sub for Point3D {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Mul<f32> for Point3D {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl std::ops::AddAssign for Point3D {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

/// 3D rotation representation (Euler angles)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation3D {
    pub pitch: f32,  // Rotation around X axis (degrees)
    pub yaw: f32,    // Rotation around Y axis (degrees)
    pub roll: f32,   // Rotation around Z axis (degrees)
}

impl Rotation3D {
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self { pitch, yaw, roll }
    }
    
    pub fn zero() -> Self {
        Self { pitch: 0.0, yaw: 0.0, roll: 0.0 }
    }
}

impl Default for Rotation3D {
    fn default() -> Self {
        Self::zero()
    }
}

/// Rotate point around X axis (pitch)
/// 
/// Matrix form:
/// [1    0      0   ]
/// [0  cos   -sin   ]
/// [0  sin    cos   ]
pub fn rotate_x(point: Point3D, angle_deg: f32) -> Point3D {
    let angle = angle_deg * PI / 180.0;
    let cos = angle.cos();
    let sin = angle.sin();
    
    Point3D {
        x: point.x,
        y: point.y * cos - point.z * sin,
        z: point.y * sin + point.z * cos,
    }
}

/// Rotate point around Y axis (yaw)
/// 
/// Matrix form:
/// [ cos   0   sin]
/// [  0    1    0 ]
/// [-sin   0   cos]
pub fn rotate_y(point: Point3D, angle_deg: f32) -> Point3D {
    let angle = angle_deg * PI / 180.0;
    let cos = angle.cos();
    let sin = angle.sin();
    
    Point3D {
        x: point.x * cos + point.z * sin,
        y: point.y,
        z: -point.x * sin + point.z * cos,
    }
}

/// Rotate point around Z axis (roll)
/// 
/// Matrix form:
/// [cos  -sin   0]
/// [sin   cos   0]
/// [ 0     0    1]
pub fn rotate_z(point: Point3D, angle_deg: f32) -> Point3D {
    let angle = angle_deg * PI / 180.0;
    let cos = angle.cos();
    let sin = angle.sin();
    
    Point3D {
        x: point.x * cos - point.y * sin,
        y: point.x * sin + point.y * cos,
        z: point.z,
    }
}

/// Apply combined 3D rotation (order: Z -> Y -> X)
/// 
/// This implements the differential transformation where we consider
/// how points change through rotation: dP/dθ for each axis.
pub fn rotate_3d(point: Point3D, rotation: Rotation3D) -> Point3D {
    let p1 = rotate_z(point, rotation.roll);
    let p2 = rotate_y(p1, rotation.yaw);
    let p3 = rotate_x(p2, rotation.pitch);
    p3
}

/// Calculate surface normal after rotation
/// 
/// For lighting calculations - determines how surface faces the light
pub fn surface_normal(rotation: Rotation3D) -> Point3D {
    // Initial normal points toward viewer (0, 0, 1)
    let initial = Point3D::new(0.0, 0.0, 1.0);
    rotate_3d(initial, rotation)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rotate_x_90() {
        let p = Point3D::new(1.0, 1.0, 0.0);
        let rotated = rotate_x(p, 90.0);
        
        assert!((rotated.x - 1.0).abs() < 0.01);
        assert!((rotated.y - 0.0).abs() < 0.01);
        assert!((rotated.z - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_rotate_y_90() {
        let p = Point3D::new(1.0, 1.0, 0.0);
        let rotated = rotate_y(p, 90.0);
        
        assert!((rotated.x - 0.0).abs() < 0.01);
        assert!((rotated.y - 1.0).abs() < 0.01);
        assert!((rotated.z - -1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_projection() {
        let p = Point3D::new(10.0, 10.0, -50.0);  // Far away
        let (px, py) = p.project(100.0);
        
        // Should be scaled down (farther = smaller)
        assert!(px < 10.0);
        assert!(py < 10.0);
    }
    
    #[test]
    fn test_normalize() {
        let p = Point3D::new(3.0, 4.0, 0.0);
        let normalized = p.normalize();
        
        let len = normalized.length();
        assert!((len - 1.0).abs() < 0.01);
    }
}
