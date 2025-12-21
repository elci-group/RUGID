/// Demonstration of Surface Differential Resolution in OCRS
/// 
/// This demo shows how the dual-projection system compares:
/// 1. Direct observation (traditional bird's-eye view)
/// 2. Reflected observation (via metallic weight's surface)
/// 
/// For nodes in the inversion range, both projections are used to
/// improve depth perception without expensive pure-computational approaches.

use rugid::ocrs::{MeshProjector, StaticMesh, ObserverCone, MinimalSolver, Weight, Point3D};

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  OCRS: Surface Differential Resolution Demonstration");
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Create test terrain
    println!("1. Creating test terrain (100m²)...");
    let mesh = StaticMesh::test_terrain(100.0);
    println!("   ✓ Generated {} vertices, {} faces\n", 
             mesh.vertex_count(), mesh.face_count());
    
    // Create weight (metallic ball) at center
    println!("2. Creating metallic weight ball...");
    let weight_pos = Point3D::new(50.0, 10.0, 50.0);
    let weight = Weight::default_metallic(weight_pos);
    println!("   ✓ Radius: {:.1}m", weight.radius);
    println!("   ✓ Mass: {:.0}kg", weight.mass);
    println!("   ✓ Reflectivity: {:.0}%", weight.reflectivity * 100.0);
    println!("   ✓ Inversion range: {:.1}m\n", weight.inversion_range());
    
    // Project with dual-projection
    println!("3. Projecting with dual-projection system...");
    let projector = MeshProjector::new(2.0, 5.0);  // 2m grid, 5m layers
    let bundle_dual = projector.project_with_reflection(&mesh, &weight);
    
    // Count statistics
    let mut nodes_in_inversion = 0;
    let mut nodes_with_reflection = 0;
    let mut total_nodes = 0;
    
    for lattice in &bundle_dual.lattices {
        for node in &lattice.nodes {
            total_nodes += 1;
            if node.in_inversion_range {
                nodes_in_inversion += 1;
                if node.reflected_deformation.is_some() {
                    nodes_with_reflection += 1;
                }
            }
        }
    }
    
    println!("   ✓ Total lattice nodes: {}", total_nodes);
    println!("   ✓ Nodes in inversion range: {} ({:.1}%)", 
             nodes_in_inversion, 
             (nodes_in_inversion as f64 / total_nodes as f64) * 100.0);
    println!("   ✓ Nodes with reflection data: {} ({:.1}%)",
             nodes_with_reflection,
             (nodes_with_reflection as f64 / nodes_in_inversion as f64) * 100.0);
    
    // Calculate additional constraints
    let standard_constraints = total_nodes;
    let dual_constraints = total_nodes + nodes_with_reflection;
    
    println!("\n4. Constraint matrix analysis:");
    println!("   Standard approach: {} constraints", standard_constraints);
    println!("   Dual-projection: {} constraints (+{})", 
             dual_constraints,
             nodes_with_reflection);
    println!("   Improvement: +{:.1}% constraint density\n",
             ((dual_constraints - standard_constraints) as f64 / standard_constraints as f64) * 100.0);
    
    // Create observer and solve
    println!("5. Solving with Observer-Centric Reality Solver...");
    let target = Point3D::new(50.0, 0.0, 50.0);
    let observer = ObserverCone::new(target, 50.0, 90.0);
    
    let solver = MinimalSolver::new();
    let reconstructed = solver.solve(&bundle_dual, &observer).unwrap();
    
    println!("   ✓ Reconstructed {} vertices", reconstructed.vertex_count());
    println!("   ✓ Confidence: {:.1}%\n", reconstructed.confidence * 100.0);
    
    // Summary
    println!("═══════════════════════════════════════════════════════════");
    println!("  Summary: Surface Differential Resolution");
    println!("═══════════════════════════════════════════════════════════");
    println!("\nKey Innovation:");
    println!("  • For targets in the cone's inversion range (high curvature)");
    println!("  • Compare TWO views: direct + reflected (via metallic ball)");
    println!("  • Improves depth perception without expensive bird's-eye compute");
    println!("\nResults:");
    println!("  • {:.1}% of lattice affected by inversion range", 
             (nodes_in_inversion as f64 / total_nodes as f64) * 100.0);
    println!("  • {} additional reflection-based constraints", nodes_with_reflection);
    println!("  • Solver confidence: {:.1}%", reconstructed.confidence * 100.0);
    println!("\nBenefit:");
    println!("  • Richer spatial information in critical regions");
    println!("  • Reduced computational overhead vs full bird's-eye");
    println!("  • Natural integration with metallic weight physics\n");
}
