//! IMBM Constraint Field
//!
//! Sparse voxel storage for the constraint field.
//! Implements the "Thermodynamic Interpretation" where constraints reduce entropy.

use std::collections::HashMap;
use super::geometry::VoxelCoord;

/// State of a single voxel in the constraint field
#[derive(Debug, Clone, Copy)]
pub struct VoxelState {
    /// Probability of occupancy (0.0 = empty, 1.0 = occupied)
    pub occupancy_probability: f32,
    /// Entropy/Uncertainty metric (high = unknown, low = certain)
    pub entropy: f32,
    /// Last update timestamp
    pub last_update: u64,
}

impl Default for VoxelState {
    fn default() -> Self {
        Self {
            occupancy_probability: 0.5, // Maximum entropy (unknown)
            entropy: 1.0,
            last_update: 0,
        }
    }
}

/// Sparse field of constraints
pub struct ConstraintField {
    /// Sparse storage of voxels
    pub voxels: HashMap<VoxelCoord, VoxelState>,
    /// Size of each voxel in world units
    pub voxel_size: f32,
}

impl ConstraintField {
    pub fn new(voxel_size: f32) -> Self {
        Self {
            voxels: HashMap::new(),
            voxel_size,
        }
    }
    
    /// Get the state of a voxel, creating it if it doesn't exist (implicit potential set)
    pub fn get_voxel(&self, coord: VoxelCoord) -> VoxelState {
        self.voxels.get(&coord).copied().unwrap_or_default()
    }
    
    /// Update a voxel with new evidence
    /// 
    /// evidence: 0.0 (empty) to 1.0 (occupied)
    /// weight: confidence of the evidence
    pub fn update_voxel(&mut self, coord: VoxelCoord, evidence: f32, weight: f32, timestamp: u64) {
        let entry = self.voxels.entry(coord).or_default();
        
        // Bayesian-like update (simplified for performance)
        // Move probability towards evidence based on weight
        let current = entry.occupancy_probability;
        let new_prob = current + (evidence - current) * weight;
        
        entry.occupancy_probability = new_prob;
        entry.last_update = timestamp;
        
        // Update entropy: Entropy is low when prob is near 0 or 1
        // E = -p*log2(p) - (1-p)*log2(1-p)
        // approximated as 4 * p * (1-p) which is 1.0 at p=0.5 and 0.0 at p=0/1
        entry.entropy = 4.0 * new_prob * (1.0 - new_prob);
    }
    
    /// Prune low-probability voxels to keep the field sparse
    pub fn prune(&mut self, threshold: f32) {
        self.voxels.retain(|_, state| {
            // Keep if probability is significant OR entropy is high (still unknown)
            state.occupancy_probability > threshold || state.entropy > 0.8
        });
    }
}
