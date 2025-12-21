use super::observer::ObserverCone;
use super::projection::LatticeBundle;

/// Compute confidence score for observer → target resolution
/// 
/// Confidence represents how well the solver can reconstruct the target
/// geometry from the current observer position. Higher distance → more
/// separable lattice intersections → higher confidence.
///
/// Returns: Value in [0.0, 1.0] where:
/// - 0.0 = No confidence (insufficient constraints)
/// - 1.0 = Perfect confidence (fully over-determined system)
pub fn compute_confidence(observer: &ObserverCone, bundle: &LatticeBundle) -> f64 {
    // Simplified confidence model for PoC
    // Real implementation would analyze actual lattice intersections
    
    let cone_width = observer.cone_width();
    let node_count = bundle.total_node_count();
    
    // Estimate separable nodes based on cone width
    // Wider cone = more nodes can be distinguished
    let grid_cell_size = cone_width / 50.0;  // Assume 50x50 resolution
    let estimated_separable = (cone_width / grid_cell_size).powf(2.0);
    
    // Constraint ratio: separable nodes / total nodes
    let constraint_ratio = estimated_separable / node_count as f64;
    
    // Asymptotic confidence function
    // Approaches 1.0 as distance increases
    // Reference distance: 10.0 meters
    const REFERENCE_DISTANCE: f64 = 10.0;
    
    let base_confidence = 1.0 - (-constraint_ratio * observer.distance / REFERENCE_DISTANCE).exp();
    
    // Clamp to valid range
    base_confidence.clamp(0.0, 1.0)
}

/// Simplified confidence for testing (just based on distance)
pub fn compute_confidence_simple(observer: &ObserverCone) -> f64 {
    // Very simple: confidence increases logarithmically with distance
    let normalized_distance = observer.distance / 100.0;  // 100m = reference
    let conf = normalized_distance.ln().max(0.0) / 5.0_f64.ln();  // log_5(dist/100)
    conf.clamp(0.0, 0.99)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocrs::projection::{Point3D, StaticMesh, MeshProjector};
    
    #[test]
    fn test_confidence_increases_with_distance() {
        // Create test lattice bundle
        let mesh = StaticMesh::test_terrain(100.0);
        let projector = MeshProjector::new(2.0, 5.0);
        let bundle = projector.project(&mesh);
        
        let target = Point3D::new(50.0, 0.0, 50.0);
        
        let observer_1m = ObserverCone::new(target, 1.0, 90.0);
        let observer_10m = ObserverCone::new(target, 10.0, 90.0);
        let observer_50m = ObserverCone::new(target, 50.0, 90.0);
        let observer_100m = ObserverCone::new(target, 100.0, 90.0);
        
        let conf_1m = compute_confidence(&observer_1m, &bundle);
        let conf_10m = compute_confidence(&observer_10m, &bundle);
        let conf_50m = compute_confidence(&observer_50m, &bundle);
        let conf_100m = compute_confidence(&observer_100m, &bundle);
        
        println!("Confidence at distances:");
        println!("  1m:   {:.1}%", conf_1m * 100.0);
        println!("  10m:  {:.1}%", conf_10m * 100.0);
        println!("  50m:  {:.1}%", conf_50m * 100.0);
        println!("  100m: {:.1}%", conf_100m * 100.0);
        
        assert!(conf_1m < conf_10m, "Confidence should increase with distance");
        assert!(conf_10m < conf_50m, "Confidence should increase with distance");
        assert!(conf_50m < conf_100m, "Confidence should increase with distance");
        
        // At 1m, confidence should be low
        assert!(conf_1m < 0.5, "Close distance should have low confidence");
        
        // At 100m, confidence should be high
        assert!(conf_100m > 0.7, "Far distance should have high confidence");
    }
}
