//! IMBM Geometry Types

/// A 3D transform (position, rotation, scale)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3D {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32), // Euler angles in degrees
    pub scale: (f32, f32, f32),
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
        }
    }
}

impl Transform3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: (x, y, z),
            ..Default::default()
        }
    }
}

/// A coordinate in the sparse voxel grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl VoxelCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}
