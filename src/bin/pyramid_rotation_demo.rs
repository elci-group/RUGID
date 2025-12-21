//! Declarative Pyramid Rotation Demo
//!
//! Implements the user's requested "Pyramid Rotation Demo" following the specified
//! ontological and architectural philosophy.

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
    println!("🎮 RUGID Pyramid Rotation Demo");
    println!("{}", "=".repeat(60));
    
    pollster::block_on(run());
}

async fn run() {
    // Create window and platform
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID Pyramid Rotation Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let mut platform = WgpuPlatform::new(window.clone()).await;
    
    // Create runtime
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    // Build UI using ontological system
    let mut cells = HashMap::new();
    let root = build_pyramid_rotation_demo(&mut cells);
    
    // Initial layout update
    let width = 800.0;
    let height = 600.0;
    update_layout(&mut runtime, &root, &cells, width, height);
    
    println!("✅ Declarative Pyramid UI resolved");
    println!("📐 Pyramid defined as ontological object");
    println!("🔄 Rotation: Attached to Pyramid, not faces");
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
                    WindowEvent::Resized(size) => {
                        if size.width > 0 && size.height > 0 {
                            update_layout(
                                &mut runtime, 
                                &root, 
                                &cells, 
                                size.width as f32, 
                                size.height as f32
                            );
                        }
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

fn update_layout(
    runtime: &mut Runtime,
    root: &OntologicalNode,
    cells: &HashMap<&str, CellId>,
    width: f32,
    height: f32,
) {
    // Resolve ontology
    let screen = ResolvedTransform::screen(width, height);
    let resolved = OntologyResolver::resolve_with_text(root, screen);

    // Register 3 pyramids with different rotations
    let pyramids = [
        ("pyramid1", 0.5, 0.0, 0.0), // X rotation
        ("pyramid2", 0.0, 0.5, 0.0), // Y rotation
        ("pyramid3", 0.0, 0.0, 0.5), // Z rotation
    ];

    for (name, rx, ry, rz) in pyramids {
        if let Some(pyramid_id) = cells.get(name) {
            if let Some(resolved_cell) = resolved.get(pyramid_id) {
                let t = &resolved_cell.transform;

                let rotation = Rotation3DState::new(rx, ry, rz)
                    .with_opacity(0.8);

                let size = t.width.min(t.height) * 0.45;

                runtime.renderer.register_pyramid_3d(
                    *pyramid_id,
                    t.x,
                    t.y,
                    t.width,
                    t.height,
                    size,
                    rotation,
                );
            }
        }
    }
}

/// Composition function: build_pyramid_rotation_demo
fn build_pyramid_rotation_demo(
    cells: &mut HashMap<&'static str, CellId>
) -> OntologicalNode {
    let p1_id = CellId::next();
    let p2_id = CellId::next();
    let p3_id = CellId::next();
    
    cells.insert("pyramid1", p1_id);
    cells.insert("pyramid2", p2_id);
    cells.insert("pyramid3", p3_id);

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
                // Pane 1 (X Axis)
                OntologicalNode::pane(
                    "pane1", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::pyramid_3d(
                        p1_id, "pane1",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.5, 0.0, 0.0, 0.8
                    )
                )
            )
            .child(
                // Pane 2 (Y Axis)
                OntologicalNode::pane(
                    "pane2", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::pyramid_3d(
                        p2_id, "pane2",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.0, 0.5, 0.0, 0.8
                    )
                )
            )
            .child(
                // Pane 3 (Z Axis)
                OntologicalNode::pane(
                    "pane3", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::pyramid_3d(
                        p3_id, "pane3",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.0, 0.0, 0.5, 0.8
                    )
                )
            )
        )
}
