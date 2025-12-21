/// Cornell Box Physics Demonstration
/// 
/// Showcases RUGID's full potential:
/// - Elasticated floor with material physics
/// - Water simulation with ripples
/// - Realistic bouncing ball with saw-tooth rest pattern
/// - Advanced multi-light setup for definitive lighting
/// - Cornell box classic color scheme (red/green walls)

use rugid::runtime::Runtime;
use rugid::rdf::loader::RdfLoader;
use rugid::platforms::wgpu_platform::WgpuPlatform;
use winit::event_loop::EventLoop;

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  RUGID Cornell Box Physics Demonstration");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Features:");
    println!("  ✓ Elasticated floor (85% restitution)");
    println!("  ✓ Water pool with ripple animation");
    println!("  ✓ Bouncing ball with realistic physics");
    println!("  ✓ Saw-tooth bounce pattern upon rest");
    println!("  ✓ Multi-light setup (area + point lights)");
    println!("  ✓ Soft shadows and global illumination");
    println!("  ✓ Classic Cornell box (red/green walls)");
    println!();
    println!("Loading scene...");
    
    // Load RDF scene
    let rdf_path = "scenes/cornell_box_physics_demo.rdf";
    let mut loader = RdfLoader::new();
    
    match loader.load_file(rdf_path) {
        Ok(scene) => {
            println!("✓ Scene loaded: {} objects", scene.cells.len());
            println!("✓ Lights configured: {} sources", scene.lights.len());
            println!();
            println!("Starting rendering...");
            println!("Watch the ball bounce with decreasing height (saw-tooth pattern)!");
            println!();
            
            // Create event loop and platform
            let event_loop = EventLoop::new().expect("Failed to create event loop");
            let platform = WgpuPlatform::new(&event_loop);
            
            // Create runtime with loaded scene
            let mut runtime = Runtime::new(platform);
            runtime.load_scene(scene);
            
            // Run the application
            runtime.run(event_loop);
        },
        Err(e) => {
            eprintln!("✗ Failed to load scene: {}", e);
            eprintln!();
            eprintln!("Make sure the scene file exists at: {}", rdf_path);
            eprintln!("Run this from the RUGID root directory.");
            std::process::exit(1);
        }
    }
}
