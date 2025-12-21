/// Demonstration of Uneven (Faceted) Weight Surfaces
/// 
/// Shows how geodesic faceting increases information density per cm²
/// by providing multiple reflection angles from the same surface area.

use rugid::ocrs::{Weight, Point3D};

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  OCRS: Uneven Weight Surface Demonstration");
    println!("═══════════════════════════════════════════════════════════\n");
    
    let position = Point3D::new(50.0, 10.0, 50.0);
    
    // Create weights with different subdivision levels
    println!("Creating weights with varying facet densities...\n");
    
    let smooth = Weight::default_metallic(position);
    let faceted_1 = Weight::faceted(position, 5.0, 1);
    let faceted_2 = Weight::faceted(position, 5.0, 2);
    
    println!("1. Smooth Sphere:");
    println!("   Facets: {}", smooth.facet_count());
    println!("   Info density: {:.6} reflections/cm²", smooth.information_density());
    println!("   Surface type: Uniform reflections\n");
    
    println!("2. Faceted (Subdivision 1):");
    println!("   Facets: {}", faceted_1.facet_count());
    println!("   Info density: {:.6} reflections/cm²", faceted_1.information_density());
    println!("   Improvement: {:.1}x over smooth", 
             faceted_1.facet_count() as f64 / smooth.facet_count() as f64);
    println!("   Surface type: 80 geodesic facets\n");
    
    println!("3. Faceted (Subdivision 2):");
    println!("   Facets: {}", faceted_2.facet_count());
    println!("   Info density: {:.6} reflections/cm²", faceted_2.information_density());
    println!("   Improvement: {:.1}x over smooth",
             faceted_2.facet_count() as f64 / smooth.facet_count() as f64);
    println!("   Surface type: 320 geodesic facets\n");
    
    // Test multi-reflection capability
    let test_point = Point3D::new(60.0, 15.0, 60.0);
    
    println!("═══════════════════════════════════════════════════════════");
    println!("  Multi-Reflection Test");
    println!("═══════════════════════════════════════════════════════════\n");
    println!("Target point: ({:.1}, {:.1}, {:.1})", test_point.x, test_point.y, test_point.z);
    println!("Distance from weight: {:.2}m\n", 
             (test_point - position).magnitude());
    
    let smooth_reflections = smooth.reflect_point(test_point);
    let faceted_1_reflections = faceted_1.reflect_point(test_point);
    let faceted_2_reflections = faceted_2.reflect_point(test_point);
    
    println!("Smooth sphere reflections: {}", smooth_reflections.len());
    println!("Faceted (sub=1) reflections: {}", faceted_1_reflections.len());
    println!("Faceted (sub=2) reflections: {}", faceted_2_reflections.len());
    
    println!("\n═══════════════════════════════════════════════════════════");
    println!("  Key Benefits");
    println!("═══════════════════════════════════════════════════════════\n");
    println!("• Multiple independent spatial samples from same surface area");
    println!("• {}-{}x more reflection data per target point", 
             faceted_1_reflections.len(), faceted_2_reflections.len());
    println!("• Richer constraint matrix for improved solver conditioning");
    println!("• Scalable: choose subdivision based on scene complexity");
    println!("\n═══════════════════════════════════════════════════════════");
    println!("  Next: Advanced Features");
    println!("═══════════════════════════════════════════════════════════\n");
    println!("See advanced_surface_plan.md for:");
    println!("  • Adaptive Surface Complexity");
    println!("  • Anisotropic Surfaces");
    println!("  • Dynamic Faceting");
    println!("  • Hybrid Materials");
    println!("  • Multi-Weight Systems\n");
}
