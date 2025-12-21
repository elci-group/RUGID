//! 3D Motion Demo
//!
//! Demonstrates:
//! - Panning (Translation)
//! - Zooming (Z-Translation)
//! - Circling (Orbiting)

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId, Rotation3DState};
use rugid::geometry::VectorRegion;
use rugid::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, StackDirection, RelativeSize};
use rugid::platforms::wgpu_platform::WgpuPlatform;
use rugid::motion3d::{Translation3DState, Orbit3DState, OrbitAxis};
use std::collections::HashMap;
use std::sync::Arc;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

fn main() {
    println!("🎮 RUGID 3D Motion Demo");
    println!("{}", "=".repeat(60));
    
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID 3D Motion Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let mut platform = WgpuPlatform::new(window.clone()).await;
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    let mut cells = HashMap::new();
    let root = build_ui(&mut cells);
    
    update_layout(&mut runtime, &root, &cells, 800.0, 600.0);
    
    println!("🚀 Demo running!");
    
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: win_event, .. } => {
                runtime.platform.handle_event(&Event::WindowEvent { 
                    window_id: window.id(), 
                    event: win_event.clone() 
                });
                
                match win_event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::RedrawRequested => {
                        runtime.tick();
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    });
}

fn build_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let pan_id = CellId::next();
    let zoom_id = CellId::next();
    let orbit_id = CellId::next();
    
    cells.insert("pan", pan_id);
    cells.insert("zoom", zoom_id);
    cells.insert("orbit", orbit_id);
    
    OntologicalNode::window("root", StackDirection::Primary)
        .child(
            OntologicalNode::pane("container", "root", StackDirection::Secondary, RelativeSize::Percent(1.0), RelativeSize::Percent(1.0))
                .child(
                    OntologicalNode::pane("col1", "container", StackDirection::Primary, RelativeSize::Percent(1.0), RelativeSize::Percent(0.33))
                        .child(
                            // Panning Cube
                            OntologicalNode::cube_3d(
                                pan_id, "col1",
                                (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                                0.5, 0.5, 0.0, 0.8
                            )
                            .with_motion(1.0, 0.5, 0.0) // Pan Right-Down
                        )
                )
                .child(
                    OntologicalNode::pane("col2", "container", StackDirection::Primary, RelativeSize::Percent(1.0), RelativeSize::Percent(0.33))
                        .child(
                            // Zooming Pyramid
                            OntologicalNode::pyramid_3d(
                                zoom_id, "col2",
                                (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                                0.0, 1.0, 0.0, 0.8
                            )
                            .with_motion(0.0, 0.0, -2.0) // Zoom Away
                        )
                )
                .child(
                    OntologicalNode::pane("col3", "container", StackDirection::Primary, RelativeSize::Percent(1.0), RelativeSize::Percent(0.33))
                        .child(
                            // Orbiting Cube
                            OntologicalNode::cube_3d(
                                orbit_id, "col3",
                                (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                                1.0, 1.0, 1.0, 0.8
                            )
                            .with_orbit(100.0, 2.0, OrbitAxis::Y) // Orbit around Y
                        )
                )
        )
}

fn update_layout(
    runtime: &mut Runtime,
    root: &OntologicalNode,
    cells: &HashMap<&str, CellId>,
    width: f32,
    height: f32,
) {
    let screen = ResolvedTransform::screen(width, height);
    let resolved = OntologyResolver::resolve_with_text(root, screen);

    for (name, id) in cells {
        if let Some(resolved_cell) = resolved.get(id) {
            let t = &resolved_cell.transform;
            let size = t.width.min(t.height) * 0.4;
            
            // Register Shape
            // Note: We need to know the type. For this demo we hardcode based on name.
            let shape_type = if *name == "zoom" {
                rugid::shapes3d::ShapeType::Pyramid
            } else {
                rugid::shapes3d::ShapeType::Cube
            };
            
            let rotation = Rotation3DState::new(0.5, 0.5, 0.0).with_opacity(0.8);
            
            runtime.renderer.register_shape_3d(
                *id, None, shape_type,
                t.x, t.y, t.width, t.height, size, rotation
            );
            
            // Register Motion
            let translation = resolved_cell.motion_3d.map(|v| Translation3DState::new(v.x, v.y, v.z));
            let orbit = resolved_cell.orbit_3d.clone();
            
            runtime.renderer.register_motion_3d(*id, translation, orbit);
        }
    }
}
