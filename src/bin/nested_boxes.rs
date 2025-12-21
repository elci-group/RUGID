//! Nested Boxes Demo - Tricolor Movement
//!
//! Demonstrates relativistic movement with three distinct behaviors:
//! - Pink Box: Standard Movement (~)
//! - Orange Box: Yoyo Movement ({~})
//! - Green Box: Simple Cycle ([~])

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId};
use rugid::geometry::VectorRegion;
use rugid::ontology::{
    OntologicalNode, OntologyResolver, RelativeSize, ResolvedTransform, 
    StackDirection, RelativeBounds, PositionMode,
};
use rugid::widgets::ButtonProjector;
use rugid::platforms::wgpu_platform::WgpuPlatform;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::WindowBuilder;
use std::sync::Arc;
use std::collections::HashMap;

fn build_tricolor_demo() -> (OntologicalNode, HashMap<&'static str, CellId>) {
    let mut cells = HashMap::new();
    
    macro_rules! cell {
        ($name:expr) => {{
            let id = CellId::next();
            cells.insert($name, id);
            id
        }};
    }
    
    let root = OntologicalNode::window(
        "root", 
        StackDirection::Primary, 
    )
    .child(
        // Pink Box Container (Top 33%)
        OntologicalNode::bounded_pane(
            "pink_container", 
            "root", 
            RelativeBounds::new(0.0, 0.33), // Top third
            RelativeBounds::new(0.0, 1.0),  // Full width
            StackDirection::Primary
        )
            .child(OntologicalNode::inset_widget(cell!("pink_bg"), "pink_container", 0.0))
            .child(OntologicalNode::bounded_widget(
                cell!("pink_box"), 
                "pink_container",
                RelativeBounds::new(0.2, 0.8),
                RelativeBounds::new(0.1, 0.3),
            ).child(
                OntologicalNode::bounded_widget(
                    cell!("pink_box_inner"), 
                    "pink_box",
                    RelativeBounds::new(0.2, 0.4), // Y: 20% -> 40% (Height 20%)
                    RelativeBounds::new(0.25, 0.75), // X: 25% -> 75% (Width 50%)
                )
            ))
    )
    .child(
        // Orange Box Container (Middle 33%)
        OntologicalNode::bounded_pane(
            "orange_container", 
            "root", 
            RelativeBounds::new(0.33, 0.66), // Middle third
            RelativeBounds::new(0.0, 1.0),
            StackDirection::Primary
        )
            .child(OntologicalNode::inset_widget(cell!("orange_bg"), "orange_container", 0.0))
            .child(OntologicalNode::bounded_widget(
                cell!("orange_box"), 
                "orange_container",
                RelativeBounds::new(0.2, 0.8),
                RelativeBounds::new(0.1, 0.3),
            ).child(
                OntologicalNode::bounded_widget(
                    cell!("orange_box_inner"), 
                    "orange_box",
                    RelativeBounds::new(0.2, 0.4),
                    RelativeBounds::new(0.25, 0.75),
                )
            ))
    )
    .child(
        // Green Box Container (Bottom 33%)
        OntologicalNode::bounded_pane(
            "green_container", 
            "root", 
            RelativeBounds::new(0.66, 1.0), // Bottom third
            RelativeBounds::new(0.0, 1.0),
            StackDirection::Primary
        )
            .child(OntologicalNode::inset_widget(cell!("green_bg"), "green_container", 0.0))
            .child(OntologicalNode::bounded_widget(
                cell!("green_box"), 
                "green_container",
                RelativeBounds::new(0.2, 0.8),
                RelativeBounds::new(0.1, 0.3),
            ).child(
                OntologicalNode::bounded_widget(
                    cell!("green_box_inner"), 
                    "green_box",
                    RelativeBounds::new(0.2, 0.4),
                    RelativeBounds::new(0.25, 0.75),
                )
            ))
    );
    
    (root, cells)
}

fn main() {
    println!("📦 RUGID Tricolor Movement Demo");
    println!("==============================================================");
    println!("Pink Box:   Standard (~)");
    println!("Orange Box: Yoyo ({{~}})");
    println!("Green Box:  Simple Cycle ([~])\n");

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("RUGID Tricolor Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap());

    // Initialize WGPU Platform
    let platform = pollster::block_on(WgpuPlatform::new(window.clone()));
    
    use std::sync::Mutex;
    use rugid::platform::{Platform, PlatformEvent};
    
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

    let (ontology_root, cell_names) = build_tricolor_demo();
    
    // Get initial window size
    let size = window.inner_size();
    let screen = ResolvedTransform::screen(size.width as f32, size.height as f32);
    
    // Resolve the ontological tree
    let resolved = OntologyResolver::resolve(&ontology_root, screen);
    
    // Register cells with runtime by traversing the tree to ensure correct Z-order
    // (Backgrounds are defined first in children list, so they should be registered first)
    fn register_recursive(
        node: &OntologicalNode, 
        resolved: &HashMap<CellId, ResolvedTransform>, 
        runtime: &mut Runtime,
        cell_names: &HashMap<&'static str, CellId>,
        screen: ResolvedTransform,
        inherited_anchor: Option<(f32, f32)>,
        parent_cell_id: Option<CellId>,
    ) {
        // Determine anchor for children
        let mut child_anchor = inherited_anchor;
        
        // If this is a container, try to find a background cell to use as anchor
        if let rugid::ontology::NodeContent::Container(_) = &node.content {
            for child in &node.children {
                if let rugid::ontology::NodeContent::Cell(id) = &child.content {
                    // Check if this is a background cell
                    let mut is_bg = false;
                    for (n, cid) in cell_names {
                        if cid == id && n.contains("bg") {
                            is_bg = true;
                            break;
                        }
                    }
                    
                    if is_bg {
                        if let Some(transform) = resolved.get(id) {
                            let x = transform.x + transform.width / 2.0;
                            let y = transform.y + transform.height / 2.0;
                            child_anchor = Some((x, y));
                        }
                        break;
                    }
                }
            }
        }

        // Register current node if it has a cell
        match &node.content {
            rugid::ontology::NodeContent::Cell(id) | rugid::ontology::NodeContent::TextCell { cell_id: id, .. } => {
                 if let Some(transform) = resolved.get(id) {
                    let mut cell = Cell::new(VectorRegion::new(
                        transform.y / screen.height,
                        transform.x / screen.width,
                        transform.height / screen.height,
                        transform.width / screen.width,
                    ));
                    cell.id = *id;
                    
                    // Reverse lookup name for color logic (inefficient but fine for demo)
                    let mut name = "unknown";
                    for (n, cid) in cell_names {
                        if cid == id {
                            name = n;
                            break;
                        }
                    }

                    // Assign colors based on name
                    let color = if name.contains("pink_box_inner") {
                        "#FF69B4" // HotPink (Darker Pink)
                    } else if name.contains("pink_box") {
                        "#FFC0CB" // Pink
                    } else if name.contains("orange_box_inner") {
                        "#FF8C00" // DarkOrange
                    } else if name.contains("orange_box") {
                        "#FFA500" // Orange
                    } else if name.contains("green_box_inner") {
                        "#006400" // DarkGreen
                    } else if name.contains("green_box") {
                        "#008000" // Green
                    } else if name.contains("bg") {
                        "#333333" // Dark grey background
                    } else {
                        "#FFFFFF"
                    };
                    
                    runtime.register_cell(cell, Box::new(ButtonProjector {
                        target: *id,
                        geometry: VectorRegion::new(
                            transform.y / 100.0,
                            transform.x / 100.0,
                            transform.height / 100.0,
                            transform.width / 100.0,
                        ),
                        label: format!("{} ({})", name, color),
                    }), child_anchor, parent_cell_id); // Pass both anchor and parent!
                    
                    // Override procedural color with our explicit color
                    runtime.renderer.update_color(*id, color.to_string());
                }
            }
            _ => {}
        }
        
        // Determine the current cell ID to pass as parent to children
        let current_cell_id = match &node.content {
            rugid::ontology::NodeContent::Cell(id) | rugid::ontology::NodeContent::TextCell { cell_id: id, .. } => Some(*id),
            _ => parent_cell_id,
        };
        
        // Recurse children
        for child in &node.children {
            register_recursive(child, resolved, runtime, cell_names, screen, child_anchor, current_cell_id);
        }
    }
    
    // We need to pass the resolved map (which is HashMap<CellId, ResolvedCell>)
    // But register_recursive expects HashMap<CellId, ResolvedTransform> or we adapt it.
    // OntologyResolver::resolve returns HashMap<CellId, ResolvedTransform>.
    // Let's use that.
    let resolved_transforms = OntologyResolver::resolve(&ontology_root, screen);
    
    register_recursive(&ontology_root, &resolved_transforms, &mut runtime, &cell_names, screen, None, None);
    
    println!("Starting event loop...\n");

    let mut frame_count: u64 = 0;
    let target_fps = 60;
    let frame_duration = std::time::Duration::from_micros(1_000_000 / target_fps);
    let mut next_frame_time = std::time::Instant::now();

    event_loop.run(move |event, target| {
        let now = std::time::Instant::now();
        
        if now >= next_frame_time {
            target.set_control_flow(ControlFlow::Poll); // Process immediately if behind
        } else {
            target.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
        }

        if let Ok(mut platform) = wgpu_platform.lock() {
            platform.handle_event(&event);
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                target.exit();
            }
            Event::AboutToWait => {
                // Check if it's time for the next frame
                if std::time::Instant::now() >= next_frame_time {
                    // Advance frame time
                    next_frame_time = std::time::Instant::now() + frame_duration;
                    
                    // Trigger movements periodically
                    frame_count += 1;
                    
                    // Trigger every 120 frames (approx 2 seconds at 60fps)
                    if frame_count % 120 == 0 {
                        println!("Triggering movements at frame {}", frame_count);
                            
                            // Pink Box: Standard ~ (Move 10% -> 80%)
                            if let Some(id) = cell_names.get("pink_box") {
                                // "10,20<md<80,20~60" (Move horizontal)
                                let input = "10,20<md<80,20~60"; 
                                if let Ok(frames) = rugid::temporal::parse_movement(input, runtime.temporal.current_time()) {
                                    runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                        *id, frames.clone(), rugid::temporal::DeltaType::Greasy
                                    ));
                                    
                                    // Sync inner box
                                    if let Some(inner_id) = cell_names.get("pink_box_inner") {
                                        // 1. Horizontal Sync
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                        
                                        // 2. Vertical Oscillation: 20<y<80~2.5 (150 frames)
                                        let vert_input = "0,20<md<0,80~150{~}";
                                        if let Ok(mut v_frames) = rugid::temporal::parse_movement(vert_input, runtime.temporal.current_time()) {
                                            // Scale by parent height
                                            if let Some(parent_id) = cell_names.get("pink_box") {
                                                if let Some(parent_transform) = resolved_transforms.get(parent_id) {
                                                    let scale = parent_transform.height / 600.0; // Screen height 600
                                                    for frame in &mut v_frames {
                                                        frame.delta_origin_p *= scale;
                                                    }
                                                }
                                            }
                                            
                                            runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                                *inner_id, v_frames, rugid::temporal::DeltaType::Greasy
                                            ));
                                        }
                                        
                                        // 3. Color Pulse (R channel) - Creates a red pulsing effect
                                        let mut color_frames = Vec::new();
                                        let current_time = runtime.temporal.current_time();
                                        
                                        // Brighten phase (0-30 frames)
                                        for i in 0..30 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_r = 0.005; // +0.5% red per frame
                                            color_frames.push(frame);
                                        }
                                        
                                        // Dim phase (30-60 frames)
                                        for i in 30..60 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_r = -0.005; // -0.5% red per frame
                                            color_frames.push(frame);
                                        }
                                        
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, color_frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                        
                                        // 4. Proportional Scaling (Breathing) - Grows and shrinks uniformly
                                        let mut size_frames = Vec::new();
                                        let current_time = runtime.temporal.current_time();
                                        
                                        // Grow phase (0-40 frames)
                                        for i in 0..40 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_extent_s = 0.002; // +0.2% width per frame
                                            frame.delta_extent_p = 0.002; // +0.2% height per frame
                                            size_frames.push(frame);
                                        }
                                        
                                        // Shrink phase (40-80 frames)
                                        for i in 40..80 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_extent_s = -0.002; // -0.2% width per frame
                                            frame.delta_extent_p = -0.002; // -0.2% height per frame
                                            size_frames.push(frame);
                                        }
                                        
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, size_frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                    }
                                }
                            }
                            
                            // Orange Box: Yoyo {~} (Move 10% -> 80% -> 10%)
                            if let Some(id) = cell_names.get("orange_box") {
                                let input = "10,20<md<80,20~60{~}";
                                if let Ok(frames) = rugid::temporal::parse_movement(input, runtime.temporal.current_time()) {
                                    runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                        *id, frames.clone(), rugid::temporal::DeltaType::Greasy
                                    ));
                                    
                                    // Sync inner box
                                    if let Some(inner_id) = cell_names.get("orange_box_inner") {
                                        // 1. Horizontal Sync
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                        
                                        // 2. Vertical Oscillation
                                        let vert_input = "0,20<md<0,80~150{~}";
                                        if let Ok(mut v_frames) = rugid::temporal::parse_movement(vert_input, runtime.temporal.current_time()) {
                                            // Scale by parent height
                                            if let Some(parent_id) = cell_names.get("orange_box") {
                                                if let Some(parent_transform) = resolved_transforms.get(parent_id) {
                                                    let scale = parent_transform.height / 600.0;
                                                    for frame in &mut v_frames {
                                                        frame.delta_origin_p *= scale;
                                                    }
                                                }
                                            }

                                            runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                                *inner_id, v_frames, rugid::temporal::DeltaType::Greasy
                                            ));
                                        }
                                        
                                        // 3. Color Pulse (G channel) - Creates a green pulsing effect
                                        let mut color_frames = Vec::new();
                                        let current_time = runtime.temporal.current_time();
                                        
                                        for i in 0..30 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_g = 0.005;
                                            color_frames.push(frame);
                                        }
                                        
                                        for i in 30..60 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_g = -0.005;
                                            color_frames.push(frame);
                                        }
                                        
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, color_frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                        
                                        // 4. Horizontal Scaling (Stretching) - Width changes only
                                        let mut size_frames = Vec::new();
                                        let current_time = runtime.temporal.current_time();
                                        
                                        // Widen phase (0-40 frames)
                                        for i in 0..40 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_extent_s = 0.003; // +0.3% width per frame
                                            frame.delta_extent_p = 0.0;   // No height change
                                            size_frames.push(frame);
                                        }
                                        
                                        // Narrow phase (40-80 frames)
                                        for i in 40..80 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_extent_s = -0.003; // -0.3% width per frame
                                            frame.delta_extent_p = 0.0;    // No height change
                                            size_frames.push(frame);
                                        }
                                        
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, size_frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                    }
                                }
                            }
                            
                            // Green Box: Simple Cycle [~] (Move 10% -> 80%, then snap back)
                            if let Some(id) = cell_names.get("green_box") {
                                let input = "10,20<md<80,20~60[~]";
                                if let Ok(frames) = rugid::temporal::parse_movement(input, runtime.temporal.current_time()) {
                                    runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                        *id, frames.clone(), rugid::temporal::DeltaType::Greasy
                                    ));
                                    
                                    // Sync inner box
                                    if let Some(inner_id) = cell_names.get("green_box_inner") {
                                        // 1. Horizontal Sync
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                        
                                        // 2. Vertical Oscillation
                                        let vert_input = "0,20<md<0,80~150{~}";
                                        if let Ok(mut v_frames) = rugid::temporal::parse_movement(vert_input, runtime.temporal.current_time()) {
                                            // Scale by parent height
                                            if let Some(parent_id) = cell_names.get("green_box") {
                                                if let Some(parent_transform) = resolved_transforms.get(parent_id) {
                                                    let scale = parent_transform.height / 600.0;
                                                    for frame in &mut v_frames {
                                                        frame.delta_origin_p *= scale;
                                                    }
                                                }
                                            }

                                            runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                                *inner_id, v_frames, rugid::temporal::DeltaType::Greasy
                                            ));
                                        }
                                        
                                        // 3. Color Pulse (B channel) - Creates a blue pulsing effect
                                        let mut color_frames = Vec::new();
                                        let current_time = runtime.temporal.current_time();
                                        
                                        for i in 0..30 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_b = 0.005;
                                            color_frames.push(frame);
                                        }
                                        
                                        for i in 30..60 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_b = -0.005;
                                            color_frames.push(frame);
                                        }
                                        
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, color_frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                        
                                        // 4. Vertical Scaling (Squashing) - Height changes only
                                        let mut size_frames = Vec::new();
                                        let current_time = runtime.temporal.current_time();
                                        
                                        // Heighten phase (0-40 frames)
                                        for i in 0..40 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_extent_s = 0.0;    // No width change
                                            frame.delta_extent_p = 0.003;  // +0.3% height per frame
                                            size_frames.push(frame);
                                        }
                                        
                                        // Shorten phase (40-80 frames)
                                        for i in 40..80 {
                                            let mut frame = rugid::temporal::DeltaFrame::zero(current_time + i, current_time + i + 1);
                                            frame.delta_extent_s = 0.0;     // No width change
                                            frame.delta_extent_p = -0.003;  // -0.3% height per frame
                                            size_frames.push(frame);
                                        }
                                        
                                        runtime.temporal.add_program(rugid::temporal::DeltaProgram::new(
                                            *inner_id, size_frames, rugid::temporal::DeltaType::Greasy
                                        ));
                                    }
                                }
                            }
                    }
                    
                    runtime.tick();
                }
            }
            _ => {}
        }
    }).unwrap();
}
