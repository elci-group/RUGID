//! Minimal Text Scaling Test
//!
//! Layout:
//! - Window
//!   - Blue Box (20% < d < 80%)
//!     - Text "Hello World!" (20% < d < 80%)

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId};
use rugid::ontology::{
    OntologicalNode, OntologyResolver, RelativeSize, StackDirection, RelativeBounds, RelativeText, ResolvedTransform,
};
use rugid::platforms::wgpu_platform::WgpuPlatform;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rugid::platform::{Platform, PlatformEvent};

fn build_ui_ontology() -> (OntologicalNode, HashMap<&'static str, CellId>) {
    let mut cells = HashMap::new();
    
    macro_rules! cell {
        ($name:expr) => {{
            let id = CellId::next();
            cells.insert($name, id);
            id
        }};
    }

    let root = OntologicalNode::window("root", StackDirection::Primary)
        .child(
            OntologicalNode::bounded_pane(
                "blue_box",
                "root",
                RelativeBounds::margins(0.2, 0.2), // 20% < d < 80% (20% margins)
                RelativeBounds::margins(0.2, 0.2),
                StackDirection::Primary
            )
            .child(
                OntologicalNode::inset_widget(cell!("bg"), "blue_box", 0.0) // Fill blue box with color
            )
            .child(
                OntologicalNode::bounded_text(
                    cell!("text"),
                    "blue_box",
                    RelativeText::new("Hello World!").color("#FFFFFF"),
                    RelativeBounds::margins(0.2, 0.2), // 20% < d < 80% relative to blue box
                    RelativeBounds::margins(0.2, 0.2),
                )
            )
        );

    (root, cells)
}

fn main() {
    println!("🎬 RUGID Text Test");
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("RUGID Text Test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap());

    let platform = pollster::block_on(WgpuPlatform::new(window.clone()));
    
    struct SharedWgpuPlatform {
        inner: Arc<Mutex<WgpuPlatform>>,
    }
    
    impl Platform for SharedWgpuPlatform {
        fn poll_events(&mut self) -> Vec<PlatformEvent> {
            self.inner.lock().unwrap().poll_events()
        }
        fn render(&mut self, svg: String) {
            self.inner.lock().unwrap().render(svg);
        }
    }

    let wgpu_platform = Arc::new(Mutex::new(platform));
    let runtime_platform = Box::new(SharedWgpuPlatform { inner: wgpu_platform.clone() });

    let mut runtime = Runtime::new(100, runtime_platform);

    let (ontology_root, cell_names) = build_ui_ontology();
    
    // Initial layout
    // Initial layout
    let screen = ResolvedTransform { 
        x: 0.0, 
        y: 0.0, 
        width: 800.0, 
        height: 600.0,
        rotation: 0.0,
        clip: None,
    };
    let resolved_cells_map = OntologyResolver::resolve_with_text(&ontology_root, screen);
    
    for (cell_id, resolved_cell) in resolved_cells_map {
        let geometry = rugid::geometry::VectorRegion::new(
            resolved_cell.transform.y / 100.0,
            resolved_cell.transform.x / 100.0,
            resolved_cell.transform.height / 100.0,
            resolved_cell.transform.width / 100.0,
        );

        let projector = Box::new(rugid::widgets::ButtonProjector {
            target: cell_id,
            geometry,
            label: "".to_string(), // Label handled by text cell registration
        });

        if let Some(text_info) = resolved_cell.text {
            runtime.register_text_cell(
                Cell::new(geometry).with_id(cell_id),
                projector,
                &text_info.content,
                text_info.x,
                text_info.y,
                text_info.font_size_px,
                &text_info.color,
                &text_info.text_anchor,
                None,
                None,
            );
        } else {
            runtime.register_cell(Cell::new(geometry).with_id(cell_id), projector, None, None);
        }
    }

    event_loop.run(move |event, target| {
        target.set_control_flow(winit::event_loop::ControlFlow::Poll);
        
        match event {
            winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
                target.exit();
            }
            winit::event::Event::AboutToWait => {
                let svg = runtime.tick();
                wgpu_platform.lock().unwrap().render(svg);
            }
            winit::event::Event::WindowEvent { event, .. } => {
                wgpu_platform.lock().unwrap().handle_event(&winit::event::Event::WindowEvent { window_id: unsafe { winit::window::WindowId::dummy() }, event });
            }
            _ => {}
        }
    }).unwrap();
}
