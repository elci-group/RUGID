use rugid::*;
use rugid::ocrs::*;
use std::io::Write;

fn main() {
    println!("=== OCRS Proof-of-Concept Demo ===\n");
    
    // Create static mesh (100m² terrain)
    println!("Generating 100m² procedural terrain...");
    let mesh = StaticMesh::test_terrain(100.0);
    println!("✓ Created mesh: {} vertices, {} faces\n", 
             mesh.vertex_count(), mesh.face_count());
    
    // Project mesh into lattice bundle
    println!("Projecting mesh into lattice layers...");
    let projector = MeshProjector::new(2.0, 5.0);  // 2m cells, 5m layers
    let bundle = projector.project(&mesh);
    println!("✓ Created {} layers with {} total nodes\n", 
             bundle.lattices.len(), bundle.total_node_count());
    
    // Create observer
    let target = Point3D::new(50.0, 0.0, 50.0);  // Center of terrain
    let mut observer = ObserverCone::new(target, 10.0, 90.0);  // Start at 10m, 90° FOV
    
    println!("=== Interactive Demo ===");
    println!("Controls:");
    println!("  + : Move observer farther (increase distance)");
    println!("  - : Move observer closer (decrease distance)");
    println!("  q : Quit\n");
    
    // Interactive loop
    loop {
        // Calculate current metrics
        let confidence = compute_confidence(&observer, &bundle);
        let cone_width = observer.cone_width();
        
        // Display current state
        print!("\r");  // Clear line
        print!("Distance: {:>6.1}m | Cone Width: {:>6.1}m | Confidence: {:>5.1}% | Quality: {}",
               observer.distance,
               cone_width,
               confidence * 100.0,
               quality_description(confidence));
        std::io::stdout().flush().unwrap();
        
        // Wait for input (non-blocking would be better, but this works for PoC)
        print!("\nAction (+/-/q): ");
        std::io::stdout().flush().unwrap();
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        match input {
            "+" => {
                observer.move_away(5.0);
                println!("→ Moved observer farther");
            }
            "-" => {
                observer.move_closer(5.0);
                println!("→ Moved observer closer");
            }
            "q" => {
                println!("\n\nExiting demo...");
                break;
            }
            _ => {
                println!("Invalid input. Use +, -, or q");
            }
        }
        
        println!();  // Blank line for readability
    }
    
    // Final summary
    println!("\n=== Demo Complete ===");
    println!("Thank you for testing OCRS!");
}

fn quality_description(confidence: f64) -> &'static str {
    match confidence {
        c if c >= 0.95 => "Excellent ✨",
        c if c >= 0.85 => "Very Good ✓",
        c if c >= 0.70 => "Good",
        c if c >= 0.50 => "Acceptable",
        c if c >= 0.30 => "Poor",
        _ => "Insufficient ✗",
    }
}
