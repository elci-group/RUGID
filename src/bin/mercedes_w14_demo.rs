//! Mercedes-AMG W14 E Performance Demo (2023)
//!
//! Photorealistic rendering of the Mercedes W14 Formula 1 car featuring:
//! - Accurate black carbon fiber livery
//! - PETRONAS teal accents
//! - INEOS red highlights
//! - W14-specific "zero-pod" sidepod design
//! - Spinning 18" Pirelli wheels on BBS rims
//! - Glowing brake discs

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
    println!("🏎️  MERCEDES-AMG W14 E PERFORMANCE");
    println!("{}", "=".repeat(60));
    println!("2023 Formula 1 World Championship Car");
    println!("Drivers: Lewis Hamilton #44, George Russell #63");
    println!();
    println!("🎨 Livery: Black carbon fiber + PETRONAS teal");
    println!("🔧 Features: Zero-pod sidepods, Gulley engine cover");
    println!("🛞 Wheels: 18\" Pirelli on BBS forged magnesium");
    
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Mercedes-AMG W14 E Performance (2023)")
        .with_inner_size(winit::dpi::LogicalSize::new(1000, 700))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let platform = WgpuPlatform::new(window.clone()).await;
    
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    let mut cells = HashMap::new();
    let root = build_ui(&mut cells);
    
    let screen = ResolvedTransform::screen(1000.0, 700.0);
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    println!("\n✅ W14 model rendered");
    println!("📊 Estimated triangles: ~800 per car");
    
    // Single large W14 for detail appreciation
    if let Some(car_id) = cells.get("w14") {
        if let Some(resolved_cell) = resolved.get(car_id) {
            let transform = &resolved_cell.transform;
            
            let mut cell = Cell::new(VectorRegion::new(
                transform.y / 700.0,
                transform.x / 1000.0,
                transform.height / 700.0,
                transform.width / 1000.0,
            ));
            cell.id = *car_id;
            
            // Slow rotation to appreciate details + spinning wheels
            let rotation_state = Rotation3DState::new(0.2, 0.35, 0.0)
                .with_opacity(1.0)
                .with_component_speed(0.2);  // Wheels spinning at race pace
            
            let car_size = transform.width.min(transform.height) * 0.85;
            
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
            
            println!("🏎️  W14 registered: size={:.0}px", car_size);
        }
    }
    
    println!("\n🚀 Window open - watch the W14 rotate!");
    println!("   Notice: PETRONAS teal stripes, INEOS red endplates");
    println!("   Wheels spinning independently of car rotation");
    
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

fn build_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let w14_id = CellId::next();
    cells.insert("w14", w14_id);
    
    // Single large W14 showcased in center
    OntologicalNode::window("root", StackDirection::Primary)
        .child(
            OntologicalNode::pane(
                "container",
                "root",
                StackDirection::Primary,
                RelativeSize::Percent(1.0),
                RelativeSize::Percent(1.0),
            )
            .child(
                OntologicalNode::mercedes_w14_3d(
                    w14_id, "container",
                    (0.05, 0.95), (0.05, 0.95), (0.05, 0.95),
                    0.2, 0.35, 0.0, 1.0
                )
            )
        )
}
