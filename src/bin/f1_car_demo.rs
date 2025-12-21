//! Declarative 3D F1 Car GUI using RUGID's ontological system
//!
//! Demonstrates:
//! - Multi-surface 3D rendering (body, wheels, wings, cockpit)
//! - Declarative 3D object specification with bounds
//! - Automatic rotation via DeltaFrame
//! - Ontological layout
//! - No manual rotation logic needed!

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId, Rotation3DState};
use rugid::geometry::VectorRegion;
use rugid::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, StackDirection, RelativeSize};
use rugid::platforms::wgpu_platform::WgpuPlatform;
use std::collections::HashMap;
use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

fn main() {
    println!("🏎️  RUGID Declarative 3D F1 Car GUI");
    println!("{}", "=".repeat(60));
    
    pollster::block_on(run());
}

async fn run() {
    // Create window and platform
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID F1 Car Rotation Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let platform = WgpuPlatform::new(window.clone()).await;
    
    // Create runtime
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    // Build UI using ontological system
    let mut cells = HashMap::new();
    let root = build_declarative_ui(&mut cells);
    
    // Resolve ontology
    let screen = ResolvedTransform::screen(800.0, 600.0);
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    println!("✅ Declarative 3D UI resolved");
    println!("📐 3 F1 Cars defined with bounds (0.1, 0.9)");
    println!("🔄 Rotation: Slow (0.5 deg/frame) on X, Y, Z axes");
    println!("🎨 Multi-surface: Body (red), Wheels (black), Wings (dark red), Cockpit (carbon)");
    println!("🛞 Component rotation: Wheels spin independently of car orientation!");
    
    // Register F1 car cells with 3D rotation
    for (name, (sx, sy, sz)) in [
        ("f1car1", (0.5, 0.0, 0.0)),
        ("f1car2", (0.0, 0.5, 0.0)),
        ("f1car3", (0.0, 0.0, 0.5)),
    ] {
        if let Some(car_id) = cells.get(name) {
            if let Some(resolved_cell) = resolved.get(car_id) {
                let transform = &resolved_cell.transform;
                
                // Create cell with 3D rotation state
                let mut cell = Cell::new(VectorRegion::new(
                    transform.y / 600.0,
                    transform.x / 800.0,
                    transform.height / 600.0,
                    transform.width / 800.0,
                ));
                cell.id = *car_id;
                
                let rotation_state = Rotation3DState::new(sx, sy, sz)
                    .with_opacity(0.95)
                    .with_component_speed(0.15);  // Spinning wheels!
                
                // Calculate car size as 75% of smaller dimension (bigger!)
                let car_size = transform.width.min(transform.height) * 0.75;
                
                // Register as 3D F1 Car (renderer will handle projection)
                runtime.renderer.register_shape_3d(
                    *car_id,
                    None,
                    rugid::shapes3d::ShapeType::F1Car,
                    transform.x,
                    transform.y,
                    transform.width,
                    transform.height,
                    car_size,
                    rotation_state,
                );
                
                println!("🏎️  {} registered: size={:.0}px at ({:.0}, {:.0})",
                    name, car_size, transform.x + transform.width/2.0, transform.y + transform.height/2.0);
            }
        }
    }
    
    println!("\n🚀 Window open - rotating declaratively!");
    
    // Event loop
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
                        // Runtime automatically updates rotation and renders
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

/// Build the UI declaratively using OntologicalNode::f1_car_3d
fn build_declarative_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let car1_id = CellId::next();
    let car2_id = CellId::next();
    let car3_id = CellId::next();
    
    cells.insert("f1car1", car1_id);
    cells.insert("f1car2", car2_id);
    cells.insert("f1car3", car3_id);
    
    // Window with horizontal stack of 3 F1 cars
    OntologicalNode::window("root", StackDirection::Primary)
        .child(
            OntologicalNode::pane(
                "container",
                "root",
                StackDirection::Secondary, // Horizontal
                RelativeSize::Percent(1.0),
                RelativeSize::Percent(1.0),
            )
            .child(
                // Pane 1 - X rotation (pitch - car flips forward/back)
                OntologicalNode::pane(
                    "pane1", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0), // Height
                    RelativeSize::Percent(0.33) // Width
                )
                .child(
                    OntologicalNode::f1_car_3d(
                        car1_id, "pane1",
                        (0.05, 0.95), (0.05, 0.95), (0.05, 0.95),
                        0.5, 0.0, 0.0, 0.95
                    )
                )
            )
            .child(
                // Pane 2 - Y rotation (yaw - car spins like a top)
                OntologicalNode::pane(
                    "pane2", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::f1_car_3d(
                        car2_id, "pane2",
                        (0.05, 0.95), (0.05, 0.95), (0.05, 0.95),
                        0.0, 0.5, 0.0, 0.95
                    )
                )
            )
            .child(
                // Pane 3 - Z rotation (roll - car tilts side to side)
                OntologicalNode::pane(
                    "pane3", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::f1_car_3d(
                        car3_id, "pane3",
                        (0.05, 0.95), (0.05, 0.95), (0.05, 0.95),
                        0.0, 0.0, 0.5, 0.95
                    )
                )
            )
        )
}
