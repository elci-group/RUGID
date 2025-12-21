//! Jabulani Ball Demo - The Infamous 2010 World Cup Ball
//!
//! Recreates the controversial Adidas Jabulani with its distinctive
//! 8-panel "Grip'n'Groove" design and South African inspired colors.
//!
//! The Jabulani was infamous for its unpredictable flight path,
//! attributed to its smooth surface and unique panel construction.

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
    println!("⚽ RUGID Jabulani Ball Demo");
    println!("{}", "=".repeat(60));
    println!("The infamous 2010 FIFA World Cup ball");
    println!("'The most hated ball in World Cup history' - goalkeepers");
    
    pollster::block_on(run());
}

async fn run() {
    // Create window and platform
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID Jabulani - 2010 World Cup Ball")
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
    
    println!("✅ Jabulani balls resolved");
    println!("🎨 Colors: White, Gold, Green, Red, Black, Blue (South African palette)");
    println!("📐 8 thermally bonded panels with seam grooves");
    
    // Register Jabulani balls with 3D rotation
    for (name, (sx, sy, sz)) in [
        ("ball1", (0.3, 0.0, 0.0)),  // Slow tumble
        ("ball2", (0.0, 0.8, 0.0)),  // Fast spin (like a goal kick)
        ("ball3", (0.2, 0.5, 0.3)),  // Chaotic "knuckleball" - the infamous Jabulani effect!
    ] {
        if let Some(ball_id) = cells.get(name) {
            if let Some(resolved_cell) = resolved.get(ball_id) {
                let transform = &resolved_cell.transform;
                
                // Create cell with 3D rotation state
                let mut cell = Cell::new(VectorRegion::new(
                    transform.y / 600.0,
                    transform.x / 800.0,
                    transform.height / 600.0,
                    transform.width / 800.0,
                ));
                cell.id = *ball_id;
                
                let rotation_state = Rotation3DState::new(sx, sy, sz).with_opacity(1.0);
                
                // Calculate ball size
                let ball_size = transform.width.min(transform.height) * 0.45;
                
                // Register as 3D Jabulani
                runtime.renderer.register_shape_3d(
                    *ball_id,
                    None,
                    rugid::shapes3d::ShapeType::Jabulani,
                    transform.x,
                    transform.y,
                    transform.width,
                    transform.height,
                    ball_size,
                    rotation_state,
                );
                
                println!("⚽ {} registered: size={:.0}px at ({:.0}, {:.0})",
                    name, ball_size, transform.x + transform.width/2.0, transform.y + transform.height/2.0);
            }
        }
    }
    
    println!("\n🚀 Window open - watch the infamous Jabulani spin!");
    println!("   Ball 3 demonstrates the unpredictable 'knuckleball' effect");
    
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

/// Build the UI declaratively
fn build_declarative_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let ball1_id = CellId::next();
    let ball2_id = CellId::next();
    let ball3_id = CellId::next();
    
    cells.insert("ball1", ball1_id);
    cells.insert("ball2", ball2_id);
    cells.insert("ball3", ball3_id);
    
    // Window with horizontal stack of 3 Jabulani balls
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
                // Ball 1 - Slow tumble
                OntologicalNode::pane(
                    "pane1", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::jabulani_3d(
                        ball1_id, "pane1",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.3, 0.0, 0.0, 1.0
                    )
                )
            )
            .child(
                // Ball 2 - Fast spin
                OntologicalNode::pane(
                    "pane2", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::jabulani_3d(
                        ball2_id, "pane2",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.0, 0.8, 0.0, 1.0
                    )
                )
            )
            .child(
                // Ball 3 - Chaotic knuckleball (the infamous Jabulani effect!)
                OntologicalNode::pane(
                    "pane3", "container", StackDirection::Primary,
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(0.33)
                )
                .child(
                    OntologicalNode::jabulani_3d(
                        ball3_id, "pane3",
                        (0.1, 0.9), (0.1, 0.9), (0.1, 0.9),
                        0.2, 0.5, 0.3, 1.0
                    )
                )
            )
        )
}
