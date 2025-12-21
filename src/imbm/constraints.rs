//! IMBM Constraints
//!
//! Defines the fundamental data structures for temporal constraints.

use super::geometry::Transform3D;

/// A mask represents a projection constraint from a specific viewpoint.
/// It encodes per-pixel likelihood of object membership.
#[derive(Debug, Clone)]
pub struct Mask {
    /// Width of the mask in pixels
    pub width: u32,
    /// Height of the mask in pixels
    pub height: u32,
    /// RLE encoded data or raw buffer (simplified here as raw bytes for prototype)
    /// 0 = empty, 255 = occupied
    pub data: Vec<u8>,
}

impl Mask {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self { width, height, data }
    }
    
    /// Sample the mask at a normalized coordinate (0.0-1.0)
    pub fn sample(&self, u: f32, v: f32) -> f32 {
        if u < 0.0 || u >= 1.0 || v < 0.0 || v >= 1.0 {
            return 0.0;
        }
        
        let x = (u * self.width as f32) as u32;
        let y = (v * self.height as f32) as u32;
        
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        
        let idx = (y * self.width + x) as usize;
        if idx < self.data.len() {
            self.data[idx] as f32 / 255.0
        } else {
            0.0
        }
    }
}

/// A packet of constraint information from a single frame
#[derive(Debug, Clone)]
pub struct ConstraintPacket {
    /// The projection mask
    pub mask: Mask,
    /// Camera transform in world space
    pub camera_transform: Transform3D,
    /// Object transform in world space (if known/tracked)
    pub object_transform: Transform3D,
    /// Confidence weight of this constraint (0.0 - 1.0)
    pub confidence: f32,
    /// Timestamp of the frame
    pub timestamp: u64,
}

impl ConstraintPacket {
    pub fn new(mask: Mask, camera_transform: Transform3D, timestamp: u64) -> Self {
        Self {
            mask,
            camera_transform,
            object_transform: Transform3D::default(),
            confidence: 1.0,
            timestamp,
        }
    }
}
