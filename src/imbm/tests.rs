#[cfg(test)]
mod tests {
    use super::*;
    use crate::imbm::field::ConstraintField;
    use crate::imbm::geometry::VoxelCoord;

    #[test]
    fn test_entropy_reduction() {
        let mut field = ConstraintField::new(1.0);
        let coord = VoxelCoord::new(0, 0, 0);
        
        // Initial state: max entropy (0.5 probability)
        let initial = field.get_voxel(coord);
        assert_eq!(initial.occupancy_probability, 0.5);
        assert_eq!(initial.entropy, 1.0);
        
        // Apply evidence of occupancy (0.8 probability) with 0.5 confidence
        field.update_voxel(coord, 0.8, 0.5, 1);
        
        let state1 = field.get_voxel(coord);
        // New prob = 0.5 + (0.8 - 0.5) * 0.5 = 0.5 + 0.15 = 0.65
        assert!((state1.occupancy_probability - 0.65).abs() < 0.001);
        // Entropy should be lower than 1.0
        assert!(state1.entropy < 1.0);
        
        // Apply more evidence of occupancy (0.9 probability) with 0.5 confidence
        field.update_voxel(coord, 0.9, 0.5, 2);
        
        let state2 = field.get_voxel(coord);
        // New prob = 0.65 + (0.9 - 0.65) * 0.5 = 0.65 + 0.125 = 0.775
        assert!((state2.occupancy_probability - 0.775).abs() < 0.001);
        // Entropy should be lower than previous state
        assert!(state2.entropy < state1.entropy);
        
        println!("Initial Entropy: 1.0");
        println!("Step 1 Entropy: {}", state1.entropy);
        println!("Step 2 Entropy: {}", state2.entropy);
    }
}
