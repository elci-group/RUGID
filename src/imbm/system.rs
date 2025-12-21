//! IMBM System
//!
//! The main entry point for the Inferential Mask-Based Mapping system.
//! Manages the lifecycle of the constraint field and processes incoming data.

use crate::temporal::DeltaProgram;
use super::field::ConstraintField;
use super::constraints::ConstraintPacket;
use super::inference::InferenceEngine;

pub struct ImbmSystem {
    /// The sparse voxel field
    field: ConstraintField,
    /// Queue of pending constraints to process
    pending_constraints: Vec<ConstraintPacket>,
    /// System time
    current_time: u64,
}

impl ImbmSystem {
    pub fn new() -> Self {
        Self {
            field: ConstraintField::new(0.5), // 0.5 unit voxel size
            pending_constraints: Vec::new(),
            current_time: 0,
        }
    }
    
    /// Feed a new constraint packet into the system
    pub fn feed_packet(&mut self, packet: ConstraintPacket) {
        self.pending_constraints.push(packet);
    }
    
    /// Process pending constraints and update the field
    /// Returns a list of DeltaPrograms to visualize the reconstruction
    pub fn tick(&mut self) -> Vec<DeltaProgram> {
        self.current_time += 1;
        
        // Process a batch of constraints (incremental solving)
        let batch_size = 5;
        let count = self.pending_constraints.len().min(batch_size);
        
        for _ in 0..count {
            if let Some(packet) = self.pending_constraints.pop() {
                InferenceEngine::apply_constraint(&mut self.field, &packet);
            }
        }
        
        // Prune low-probability voxels periodically
        if self.current_time % 100 == 0 {
            self.field.prune(0.1);
        }
        
        // Generate deltas for visualization
        // In a real implementation, this would generate a mesh or point cloud
        // For now, we return an empty list as we don't have a specific cell to update yet
        Vec::new()
    }
    
    /// Get the number of active voxels (for debugging)
    pub fn voxel_count(&self) -> usize {
        self.field.voxels.len()
    }
}

impl Default for ImbmSystem {
    fn default() -> Self {
        Self::new()
    }
}
