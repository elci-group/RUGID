//! Declarative Octahedron Rotation Demo
//!
//! Based on cube_rotation_demo.rs, but for Octahedrons.

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId, Rotation3DState};
use rugid::geometry::VectorRegion;
use rugid::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, StackDirection, RelativeSize};
use rugid::platforms::wgpu_platform::WgpuPlatform;
use rugid::shapes3d::ShapeType;
use std::collections::HashMap;
use std::sync::Arc;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

fn main() {
    println!("🎮 RUGID Declarative Octahedron Demo");
    println!("{}", "=".repeat(60));
    
    pollster::block_on(run());
}

async fn run() {
    // Create window and platform
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID Octahedron Rotation Demo")
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
    
    // Initial layout update
    let width = 800.0;
    let height = 600.0;
    update_layout(&mut runtime, &root, &cells, width, height);
    
    println!("✅ Declarative 3D UI resolved");
    println!("📐 3 Octahedrons defined");
    println!("🔄 Rotation: Automatic on X, Y, Z axes");
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

    // Register background panes
    for (name, color) in [
        ("bg1", "rgb(30, 30, 30)"),
        ("bg2", "rgb(40, 40, 40)"),
        ("bg3", "rgb(30, 30, 30)"),
    ] {
        if let Some(cell_id) = cells.get(name) {
            if let Some(resolved_cell) = resolved.get(cell_id) {
                let transform = &resolved_cell.transform;
                let mut cell = Cell::new(VectorRegion::new(
                    transform.y / height,
                    transform.x / width,
                    transform.height / height,
                    transform.width / width,
                ));
                cell.id = *cell_id;
                
                // Register as simple colored rect
                runtime.renderer.register_cell(
                    *cell_id, 
                    None, 
                    rugid::svg::SvgNode::Rect {
                        attributes: vec![rugid::svg::SvgAttribute::Fill(color.to_string())],
                        x: transform.x,
                        y: transform.y,
                        width: transform.width,
                        height: transform.height,
                    }
                );
            }
        }
    }

    // Register shape cells with 3D rotation
    let shapes = [
        ("octa1", ShapeType::Octahedron, (0.5, 0.0, 0.0)),
        ("octa2", ShapeType::Octahedron, (0.0, 0.5, 0.0)),
        ("octa3", ShapeType::Octahedron, (0.0, 0.0, 0.5)),
    ];

    for (name, shape_type, (sx, sy, sz)) in shapes {
        if let Some(cell_id) = cells.get(name) {
            if let Some(resolved_cell) = resolved.get(cell_id) {
                let transform = &resolved_cell.transform;
                
                // Create cell with 3D rotation state
                let mut cell = Cell::new(VectorRegion::new(
                    transform.y / height,
                    transform.x / width,
                    transform.height / height,
                    transform.width / width,
                ));
                cell.id = *cell_id;
                
                let rotation_state = Rotation3DState::new(sx, sy, sz).with_opacity(0.8);
                
                // Calculate size as 40% of smaller dimension
                let size = transform.width.min(transform.height) * 0.4;
                
                // Register as 3D shape
                runtime.renderer.register_shape_3d(
                    *cell_id,
                    None,
                    shape_type,
                    transform.x,
                    transform.y,
                    transform.width,
                    transform.height,
                    size,
                    rotation_state,
                );
            }
        }
    }
}

/// Build the UI declaratively
fn build_declarative_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let o1_id = CellId::next();
    let o2_id = CellId::next();
    let o3_id = CellId::next();
    
    let bg1_id = CellId::next();
    let bg2_id = CellId::next();
    let bg3_id = CellId::next();
    
    cells.insert("octa1", o1_id);
    cells.insert("octa2", o2_id);
    cells.insert("octa3", o3_id);
    
    cells.insert("bg1", bg1_id);
    cells.insert("bg2", bg2_id);
    cells.insert("bg3", bg3_id);
    
    // Window with horizontal stack of 3 panes
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
                    OntologicalNode::widget(
                        bg1_id, "pane1",
                        RelativeSize::Percent(1.0),
                        RelativeSize::Percent(1.0)
                    )
                    .child(
                        OntologicalNode::shape_3d(
                            o1_id, "pane1",
                            ShapeType::Octahedron,
                            (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                            0.5, 0.0, 0.0, 0.8
                        )
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
                    OntologicalNode::widget(
                        bg2_id, "pane2",
                        RelativeSize::Percent(1.0),
                        RelativeSize::Percent(1.0)
                    )
                    .child(
                        OntologicalNode::shape_3d(
                            o2_id, "pane2",
                            ShapeType::Octahedron,
                            (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                            0.0, 0.5, 0.0, 0.8
                        )
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
                    OntologicalNode::widget(
                        bg3_id, "pane3",
                        RelativeSize::Percent(1.0),
                        RelativeSize::Percent(1.0)
                    )
                    .child(
                        OntologicalNode::shape_3d(
                            o3_id, "pane3",
                            ShapeType::Octahedron,
                            (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                            0.0, 0.0, 0.5, 0.8
                        )
                    )
                )
            )
        )
}
