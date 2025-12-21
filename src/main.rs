use rugid::runtime::Runtime;
use rugid::cell::Cell;
use rugid::geometry::VectorRegion;
use rugid::input::{InputEvent, InputState};
use rugid::platform::{HeadlessPlatform, PlatformEvent};
use rugid::layout::{LayoutNode, LayoutDirection, LayoutConstraint, LayoutSolver};
use rugid::widgets::{ButtonProjector, SliderProjector};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

fn main() {
    let output_dir = Path::new("output");
    if !output_dir.exists() {
        fs::create_dir(output_dir).unwrap();
    }

    // Shared Event Queue
    let event_queue = Arc::new(Mutex::new(Vec::new()));
    
    struct SharedHeadlessPlatform {
        queue: Arc<Mutex<Vec<PlatformEvent>>>,
        last_frame: Option<String>,
    }
    
    impl rugid::platform::Platform for SharedHeadlessPlatform {
        fn poll_events(&mut self) -> Vec<PlatformEvent> {
            let mut q = self.queue.lock().unwrap();
            let events = q.clone();
            q.clear();
            events
        }
        fn render(&mut self, svg: String) {
            self.last_frame = Some(svg);
        }
    }

    let queue_ref = event_queue.clone();
    let platform = Box::new(SharedHeadlessPlatform { 
        queue: event_queue, 
        last_frame: None 
    });

    let mut runtime = Runtime::new(100, platform);

    // Create Cells
    let btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0)); // Placeholders
    let slider_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));

    // Define Layout
    // Root: Column
    //   - Button (Fixed 0.2 height)
    //   - Slider (Fixed 0.2 height)
    let root = LayoutNode::container(
        LayoutDirection::StackSecondary, // Vertical stack
        LayoutConstraint::Fixed(1.0),
        vec![
            LayoutNode::leaf(btn_cell.id, LayoutConstraint::Fixed(0.2)),
            LayoutNode::leaf(slider_cell.id, LayoutConstraint::Fixed(0.2)),
        ]
    );

    // Solve Layout (Screen: 1.0 x 1.0)
    let screen_rect = VectorRegion::new(0.0, 0.0, 1.0, 1.0);
    let layout_map = LayoutSolver::solve(&root, screen_rect);

    // Register Cells with Resolved Geometry
    let btn_geo = *layout_map.get(&btn_cell.id).unwrap();
    let slider_geo = *layout_map.get(&slider_cell.id).unwrap();

    // Update cell geometry before registering? 
    // Cell struct has geometry field but it's immutable public? 
    // Wait, Cell::new takes geometry. We created them with dummy.
    // We should recreate them or update.
    // Cell fields are public? No, `pub struct Cell { pub id: CellId, pub geometry: VectorRegion }`
    // Yes, they are public in `src/cell.rs` (checked previously).
    
    let mut btn_cell = btn_cell;
    btn_cell.geometry = btn_geo;
    
    let mut slider_cell = slider_cell;
    slider_cell.geometry = slider_geo;

    println!("Button Geo: {:?}", btn_geo);
    println!("Slider Geo: {:?}", slider_geo);

    let btn_id = btn_cell.id;
    let slider_id = slider_cell.id;

    runtime.register_cell(btn_cell, Box::new(ButtonProjector {
        target: btn_id,
        geometry: btn_geo,
        label: "Click Me".to_string(),
    }), None, None);

    runtime.register_cell(slider_cell, Box::new(SliderProjector {
        target: slider_id,
        geometry: slider_geo,
        value: 0.5,
    }), None, None);

    // Simulation Loop
    for t in 0..20 {
        // Hover Button at t=5 (Button is at top, 0.0-0.2 Y)
        // Coords: 50, 10 (0.5, 0.1)
        if t == 5 {
            println!("Injecting PointerMove(50, 10) -> Hover Button");
            queue_ref.lock().unwrap().push(PlatformEvent::Input(InputEvent::PointerMove(50.0, 10.0)));
        }
        
        // Hover Slider at t=10 (Slider is below, 0.2-0.4 Y)
        // Coords: 50, 30 (0.5, 0.3)
        if t == 10 {
            println!("Injecting PointerMove(50, 30) -> Hover Slider");
            queue_ref.lock().unwrap().push(PlatformEvent::Input(InputEvent::PointerMove(50.0, 30.0)));
        }

        let svg = runtime.tick();
        let filename = output_dir.join(format!("frame_{:02}.svg", t));
        fs::write(filename, svg).unwrap();
    }

    println!("Simulation complete. Check output/ directory.");
}
