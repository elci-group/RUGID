use rugid::platforms::wgpu_platform::WgpuPlatform;
/// Cornell Box Physics Demonstration
///
/// Showcases RUGID's full potential:
/// - Elasticated floor with material physics
/// - Water simulation with ripples
/// - Realistic bouncing ball with saw-tooth rest pattern
/// - Advanced multi-light setup for definitive lighting
/// - Cornell box classic color scheme (red/green walls)
use rugid::runtime::Runtime;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

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

    let rdf_path = "scenes/cornell_box_physics_demo.rdf";

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("RUGID Cornell Box Physics Demonstration")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    let platform = pollster::block_on(WgpuPlatform::new(window.clone()));

    let mut runtime = match Runtime::from_rdf(rdf_path, Box::new(platform)) {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("✗ Failed to load scene: {}", e);
            eprintln!();
            eprintln!("Make sure the scene file exists at: {}", rdf_path);
            eprintln!("Run this from the RUGID root directory.");
            std::process::exit(1);
        }
    };

    println!("✓ Scene loaded");
    println!();
    println!("Starting rendering...");
    println!("Watch the ball bounce with decreasing height (saw-tooth pattern)!");
    println!();

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            runtime.platform.handle_event(&event);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    target.exit();
                }
                Event::AboutToWait => {
                    runtime.tick();
                }
                _ => {}
            }
        })
        .unwrap();
}
