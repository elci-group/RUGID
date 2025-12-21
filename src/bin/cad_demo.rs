//! CAD File Viewer Demo
//!
//! Demonstrates loading and rendering CAD files (STL) in RUGID.
//! Uses the same declarative pattern as cube_rotation_demo.rs.

use rugid::cad::stl::{parse_stl, generate_sample_stl};
use rugid::cad::converter::CADRenderer;
use rugid::geometry3d::Rotation3D;
use rugid::cell::CellId;
use rugid::platforms::wgpu_platform::WgpuPlatform;
use rugid::platform::Platform;
use std::sync::Arc;
use std::io::Write;
use winit::event_loop::EventLoop;
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

fn main() {
    println!("🔧 RUGID CAD Viewer Demo");
    println!("{}", "=".repeat(60));
    
    pollster::block_on(run());
}

async fn run() {
    // Create window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID CAD Viewer")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let mut platform = WgpuPlatform::new(window.clone()).await;
    
    // Generate sample STL file
    let stl_content = generate_sample_stl();
    let temp_path = std::env::temp_dir().join("rugid_sample.stl");
    {
        let mut file = std::fs::File::create(&temp_path).unwrap();
        file.write_all(stl_content.as_bytes()).unwrap();
    }
    
    println!("📄 Generated sample STL: {}", temp_path.display());
    
    // Parse STL
    let mesh = parse_stl(&temp_path).expect("Failed to parse STL");
    println!("{}", mesh.stats());
    
    // Create CAD renderer
    let mut cad_renderer = CADRenderer::new();
    let shape_id = CellId::next();
    cad_renderer.register_colored(shape_id, &mesh, (100, 150, 220)); // Steel blue
    
    println!("✅ CAD mesh registered with RUGID");
    println!("🎲 Shape: {} triangles, {} vertices", 
        mesh.triangles.len(), mesh.vertices.len());
    
    // Rotation state
    let mut rotation = Rotation3D::zero();
    let rotation_speed = (0.5, 0.75, 0.25); // Degrees per frame
    
    println!("\n🚀 Window open - rotating CAD model!");
    
    // Event loop
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: win_event, .. } => {
                platform.handle_event(&Event::WindowEvent { 
                    window_id: window.id(), 
                    event: win_event.clone() 
                });
                
                match win_event {
                    WindowEvent::CloseRequested => {
                        println!("\n👋 Closing...");
                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        // Update rotation
                        rotation.pitch += rotation_speed.0;
                        rotation.yaw += rotation_speed.1;
                        rotation.roll += rotation_speed.2;
                        
                        // Keep angles reasonable
                        rotation.pitch %= 360.0;
                        rotation.yaw %= 360.0;
                        rotation.roll %= 360.0;
                        
                        // Render CAD shape
                        let shape_svg = cad_renderer
                            .render(shape_id, rotation, 400.0, 300.0, 200.0)
                            .unwrap_or_default();
                        
                        // Build full SVG
                        let svg = format!(
                            r##"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600" viewBox="0 0 800 600">
  <rect width="800" height="600" fill="#1a1a2e" />
  <text x="400" y="40" text-anchor="middle" fill="#ecf0f1" font-size="24" font-family="sans-serif">RUGID CAD Viewer</text>
  <text x="400" y="70" text-anchor="middle" fill="#7f8c8d" font-size="14" font-family="sans-serif">{} triangles</text>
  {}
  <text x="400" y="580" text-anchor="middle" fill="#7f8c8d" font-size="12" font-family="sans-serif">Rotation: {:.1} / {:.1} / {:.1}</text>
</svg>"##,
                            mesh.triangles.len(),
                            shape_svg,
                            rotation.pitch, rotation.yaw, rotation.roll
                        );
                        
                        platform.render(svg);
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
