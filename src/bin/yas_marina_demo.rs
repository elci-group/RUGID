//! Yas Marina Circuit with Mercedes W14 - Grandstand View Demo
//!
//! Topologically accurate rendering of the Abu Dhabi F1 circuit featuring:
//! - 16 turns (2021+ layout)
//! - 5.281 km track length (scaled)
//! - Key sections: Main Straight, North Hairpin, Marsa Corner, Hotel Section
//! - Grandstand camera view
//! - W14 car following track path in a loop

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId, Rotation3DState};
use rugid::geometry::VectorRegion;
use rugid::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, StackDirection, RelativeSize};
use rugid::platforms::wgpu_platform::WgpuPlatform;
use std::collections::HashMap;
use std::sync::Arc;
use std::f32::consts::PI;
use winit::event_loop::EventLoop;
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

/// Generate Yas Marina Circuit path (topologically accurate 2021+ layout)
/// Returns a vector of (x, z) track centerline points
fn generate_yas_marina_path(scale: f32) -> Vec<(f32, f32)> {
    let mut path = Vec::new();
    let s = scale;
    
    // Yas Marina 2021+ layout - 16 turns
    // Starting from main straight (pit straight)
    
    // Main straight (T16 -> T1) - ~1.14km longest straight
    for i in 0..30 {
        let t = i as f32 / 30.0;
        path.push((s * (-2.0 + t * 2.4), s * 0.0));
    }
    
    // Turn 1 - Heavy braking right-hander
    for i in 0..15 {
        let angle = PI * 0.5 * i as f32 / 15.0;
        path.push((s * (0.4 + 0.3 * angle.sin()), s * (0.0 + 0.3 * (1.0 - angle.cos()))));
    }
    
    // Turn 2-3 - Left-right esses 
    for i in 0..20 {
        let t = i as f32 / 20.0;
        let x = s * (0.7 + t * 0.4);
        let z = s * (0.3 + 0.15 * (t * PI * 2.0).sin());
        path.push((x, z));
    }
    
    // Turn 4-5 - Chicane leading to back straight
    for i in 0..15 {
        let t = i as f32 / 15.0;
        path.push((s * (1.1 + t * 0.2), s * (0.3 - t * 0.1)));
    }
    
    // Turn 6 - Long back straight entry
    for i in 0..25 {
        let t = i as f32 / 25.0;
        path.push((s * (1.3 + t * 0.1), s * (0.2 - t * 0.6)));
    }
    
    // North Hairpin (Turn 7) - Widened hairpin
    for i in 0..25 {
        let angle = PI * i as f32 / 25.0;
        let cx = s * 1.4;
        let cz = s * -0.5;
        path.push((cx + 0.2 * s * angle.sin(), cz - 0.15 * s * angle.cos()));
    }
    
    // Turn 8-9 - Back section
    for i in 0..30 {
        let t = i as f32 / 30.0;
        let x = s * (1.4 - t * 0.9);
        let z = s * (-0.65 + t * 0.1 - 0.05 * (t * PI).sin());
        path.push((x, z));
    }
    
    // Marsa Corner (Turn 10) - Single banked sweeper (replaced old T11-14)
    for i in 0..35 {
        let angle = PI * 0.7 * i as f32 / 35.0;
        let cx = s * 0.4;
        let cz = s * -0.7;
        let r = s * 0.4;
        path.push((cx - r * angle.sin(), cz + r * (1.0 - angle.cos())));
    }
    
    // Turn 11-12 - Marina section
    for i in 0..20 {
        let t = i as f32 / 20.0;
        path.push((s * (0.0 - t * 0.5), s * (-0.3 + t * 0.2)));
    }
    
    // Hotel section (Turn 13-15) - Under the W Hotel
    for i in 0..30 {
        let t = i as f32 / 30.0;
        let x = s * (-0.5 - t * 0.8);
        let z = s * (-0.1 + t * 0.15 + 0.1 * (t * PI * 1.5).sin());
        path.push((x, z));
    }
    
    // Turn 16 - Final corner back to main straight
    for i in 0..20 {
        let angle = PI * 0.5 * i as f32 / 20.0;
        path.push((s * (-1.3 - 0.3 * angle.sin()), s * (0.05 - 0.1 * (1.0 - angle.cos()))));
    }
    
    // Complete back to start
    for i in 0..20 {
        let t = i as f32 / 20.0;
        path.push((s * (-1.6 - t * 0.4), s * (0.0 - t * 0.0)));
    }
    
    path
}

fn main() {
    println!("🏁 YAS MARINA CIRCUIT - ABU DHABI");
    println!("{}", "=".repeat(60));
    println!("2021+ Layout: 5.281 km, 16 turns");
    println!("Key Feature: W Abu Dhabi Hotel, Marsa Corner");
    println!();
    println!("🎥 Camera: Grandstand view (Main Straight)");
    println!("🏎️  Vehicle: Mercedes-AMG W14 E Performance");
    
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Yas Marina Circuit - Mercedes W14 Lap")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let platform = WgpuPlatform::new(window.clone()).await;
    
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    let mut cells = HashMap::new();
    let root = build_ui(&mut cells);
    
    let screen = ResolvedTransform::screen(1200.0, 800.0);
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    // Generate track path
    let track_path = generate_yas_marina_path(300.0);
    println!("📍 Track path generated: {} waypoints", track_path.len());
    
    // Track state
    let mut path_index: usize = 0;
    let path_speed = 2;  // Points per frame
    
    // Register W14
    if let Some(car_id) = cells.get("w14") {
        if let Some(resolved_cell) = resolved.get(car_id) {
            let transform = &resolved_cell.transform;
            
            let mut cell = Cell::new(VectorRegion::new(
                transform.y / 800.0,
                transform.x / 1200.0,
                transform.height / 800.0,
                transform.width / 1200.0,
            ));
            cell.id = *car_id;
            
            // Initial position - will be updated each frame
            let rotation_state = Rotation3DState::new(0.0, 0.0, 0.0)
                .with_opacity(1.0)
                .with_component_speed(0.25);  // Wheels spinning fast
            
            let car_size = 60.0;  // Small car for track view
            
            runtime.renderer.register_shape_3d(
                *car_id,
                None,
                rugid::shapes3d::ShapeType::MercedesW14,
                transform.x,
                transform.y,
                transform.width,
                transform.height,
                car_size,
                rotation_state,
            );
        }
    }
    
    println!("\n🚀 Circuit simulation running!");
    println!("   W14 driving around Yas Marina");
    println!("   Watch it navigate all 16 turns");
    
    let center_x = 600.0;  // Screen center
    let center_y = 400.0;
    
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: win_event, .. } => {
                runtime.platform.handle_event(&Event::WindowEvent { 
                    window_id: window.id(), 
                    event: win_event.clone() 
                });
                
                match win_event {
                    WindowEvent::CloseRequested => {
                        println!("\n👋 Closing...");
                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        // Update car position along track
                        path_index = (path_index + path_speed) % track_path.len();
                        let next_index = (path_index + 1) % track_path.len();
                        
                        let (track_x, track_z) = track_path[path_index];
                        let (next_x, next_z) = track_path[next_index];
                        
                        // Calculate car heading (yaw) from path direction
                        let dx = next_x - track_x;
                        let dz = next_z - track_z;
                        let heading = dz.atan2(dx) * 180.0 / PI;
                        
                        // Update car position in renderer
                        if let Some(car_id) = cells.get("w14") {
                            // Update position
                            runtime.renderer.update_shape_position(
                                *car_id,
                                center_x + track_x,
                                center_y + track_z * 0.7,  // Perspective scaling
                            );
                            
                            // Update rotation
                            runtime.renderer.update_shape_rotation(
                                *car_id,
                                8.0,                    // Slight tilt for 3D feel
                                heading + 90.0,         // Point along track
                                0.0,
                            );
                        }
                        
                        runtime.tick();
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

fn build_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let w14_id = CellId::next();
    cells.insert("w14", w14_id);
    
    OntologicalNode::window("root", StackDirection::Primary)
        .child(
            OntologicalNode::pane(
                "circuit_view",
                "root",
                StackDirection::Primary,
                RelativeSize::Percent(1.0),
                RelativeSize::Percent(1.0),
            )
            .child(
                OntologicalNode::mercedes_w14_3d(
                    w14_id, "circuit_view",
                    (0.4, 0.6), (0.4, 0.6), (0.4, 0.6),
                    0.0, 0.0, 0.0, 1.0
                )
            )
        )
}
