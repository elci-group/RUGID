//! Declarative 3D Cube GUI using RUGID's ontological system
//!
//! Demonstrates:
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
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

fn main() {
    println!("🎮 RUGID Declarative 3D Cube GUI");
    println!("{}", "=".repeat(60));
    
    pollster::block_on(run());
}

async fn run() {
    // Create window and platform
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID Cube Rotation Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let mut platform = WgpuPlatform::new(window.clone()).await;
    
    // Create runtime
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    // Build UI using ontological system
    let mut cells = HashMap::new();
    let root = build_declarative_ui(&mut cells);
    
    // Resolve ontology
    let screen = ResolvedTransform::screen(800.0, 600.0);
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    println!("✅ Declarative 3D UI resolved");
    println!("📐 3 Cubes defined with bounds (0.1, 0.9)");
    println!("🔄 Rotation: Slow (0.5 deg/frame) on X, Y, Z axes");
    
    // Register cube cells with 3D rotation
    for (name, (sx, sy, sz)) in [
        ("cube1", (0.5, 0.0, 0.0)),
        ("cube2", (0.0, 0.5, 0.0)),
        ("cube3", (0.0, 0.0, 0.5)),
    ] {
        if let Some(cube_id) = cells.get(name) {
            if let Some(resolved_cell) = resolved.get(cube_id) {
                let transform = &resolved_cell.transform;
                
                // Create cell with 3D rotation state
                let mut cell = Cell::new(VectorRegion::new(
                    transform.y / 600.0,
                    transform.x / 800.0,
                    transform.height / 600.0,
                    transform.width / 800.0,
                ));
                cell.id = *cube_id;
                
                let rotation_state = Rotation3DState::new(sx, sy, sz).with_opacity(0.5);
                
                // Calculate cube size as 40% of smaller dimension
                let cube_size = transform.width.min(transform.height) * 0.4;
                
                // Register as 3D cube (renderer will handle projection)
                runtime.renderer.register_cube_3d(
                    *cube_id,
                    None,
                    transform.x,
                    transform.y,
                    transform.width,
                    transform.height,
                    cube_size,
                    rotation_state,
                );
                
                println!("🎲 {} registered: size={:.0}px at ({:.0}, {:.0})",
                    name, cube_size, transform.x + transform.width/2.0, transform.y + transform.height/2.0);
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

/// Build the UI declaratively using OntologicalNode::cube_3d
fn build_declarative_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let cube1_id = CellId::next();
    let cube2_id = CellId::next();
    let cube3_id = CellId::next();
    
    cells.insert("cube1", cube1_id);
    cells.insert("cube2", cube2_id);
    cells.insert("cube3", cube3_id);
    
    // Window with horizontal stack of 3 cubes
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
                // Pane 1
                OntologicalNode::pane(
                    "pane1", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0), // Height
                    RelativeSize::Percent(0.33) // Width
                )
                .child(
                    OntologicalNode::cube_3d(
                        cube1_id, "pane1",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.5, 0.0, 0.0, 0.5
                    )
                )
            )
            .child(
                // Pane 2
                OntologicalNode::pane(
                    "pane2", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::cube_3d(
                        cube2_id, "pane2",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.0, 0.5, 0.0, 0.5
                    )
                )
            )
            .child(
                // Pane 3
                OntologicalNode::pane(
                    "pane3", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::cube_3d(
                        cube3_id, "pane3",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.0, 0.0, 0.5, 0.5
                    )
                )
            )
        )
}
