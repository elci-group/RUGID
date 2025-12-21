//! RUGID Media Player - Ontological Indentation Exemplar
//!
//! This demonstrates the ontological indentation principle:
//! - Each element exists in its parent's coordinate space
//! - All sizes are expressed as percentages of parent
//! - No absolute dimensions anywhere in the layout definition
//! - Code structure mirrors visual hierarchy
//! - Nested boxes use min/max bounded constraints (20% < x < 80%)
//! - Text rendering with parent-relative font sizing

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId};
use rugid::geometry::VectorRegion;
use rugid::ontology::{
    OntologicalNode, OntologyResolver, RelativeSize, ResolvedTransform, 
    StackDirection, RelativeBounds, RelativeText, ResolvedCell,
};
use rugid::widgets::{ButtonProjector, SliderProjector};
use rugid::platforms::wgpu_platform::WgpuPlatform;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::WindowBuilder;
use std::sync::Arc;
use std::collections::HashMap;

/// Builds the ontological tree for the media player UI.
/// 
/// Notice: NO absolute pixel values. ALL dimensions are relative to parent.
/// The hierarchy in code exactly mirrors the visual containment hierarchy.
/// 
/// NEW: Demonstrates NESTED BOXES using bounded constraints:
/// - Video overlay controls (inset within video)
/// - Sidebar items with padding (bounded within sidebar)
/// - Modal-style elements using bounded positioning
fn build_ui_ontology() -> (OntologicalNode, HashMap<&'static str, CellId>) {
    let mut cells = HashMap::new();
    
    // Helper to create a named cell
    macro_rules! cell {
        ($name:expr) => {{
            let id = CellId::next();
            cells.insert($name, id);
            id
        }};
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // ROOT (Screen Space)
    // └── Header (8% of screen height)
    // │   ├── Logo (12% of header width)
    // │   ├── SearchBar (fills remaining) ← NESTED: search icon + input field
    // │   └── ButtonGroup (25% of header width)
    // │       ├── MenuBtn (1/3 of group)
    // │       ├── SettingsBtn (1/3 of group)
    // │       └── ProfileBtn (1/3 of group)
    // ├── MainArea (fills remaining height minus control bar)
    // │   ├── Sidebar (18% of main width)
    // │   │   ├── SidebarHeader (10% of sidebar height)
    // │   │   ├── Playlists (bounded with padding)
    // │   │   │   ├── Playlist1 
    // │   │   │   ├── Playlist2 
    // │   │   │   └── Playlist3 
    // │   │   └── Actions (bounded with padding)
    // │   │       ├── LibraryBtn 
    // │   │       └── DownloadsBtn 
    // │   └── VideoSection (fills remaining width)
    // │       ├── VideoPlayer (fills, minus info area)
    // │       │   └── VideoOverlay (INSET: centered controls within video)
    // │       │       ├── BigPlayBtn (centered, 15% of video dims)
    // │       │       └── ProgressOverlay (bottom 10%, 80% width centered)
    // │       ├── VideoTitle (5% of section height)
    // │       └── VideoInfo (4% of section height)
    // └── ControlBar (8% of screen height)
    //     ├── PlaybackControls (15% of bar width)
    //     │   ├── PrevBtn (25% of controls)
    //     │   ├── PlayBtn (50% of controls)
    //     │   └── NextBtn (25% of controls)
    //     ├── Timeline (fills remaining)
    //     ├── TimeDisplay (8% of bar width)
    //     ├── VolumeIcon (4% of bar width)
    //     ├── VolumeSlider (10% of bar width)
    //     └── FullscreenBtn (4% of bar width)
    // ═══════════════════════════════════════════════════════════════════════════

    // ═══════════════════════════════════════════════════════════════════════════
    // MINIMAL TEST CASE: Single Label
    // ═══════════════════════════════════════════════════════════════════════════

    let root = OntologicalNode::window("root", StackDirection::Primary)
        .child(
            OntologicalNode::pane("test_pane", "root", StackDirection::Primary, RelativeSize::Percent(0.5), RelativeSize::Percent(0.5))
                .child(
                    OntologicalNode::widget(cell!("test_widget"), "test_pane", RelativeSize::Percent(1.0), RelativeSize::Percent(1.0))
                        .child(
                            OntologicalNode::text_inset(
                                cell!("test_label"),
                                "test_widget", // Explicit parent is the widget
                                RelativeText::new("TESTING 123").bold().color("#FFFFFF"),
                                0.0, // 0 margin = 100% height of widget
                            )
                        )
                )
        );

    (root, cells)
}


fn main() {
    println!("🎬 RUGID Media Player - Ontological Indentation Demo");
    println!("=====================================================");
    println!("Demonstrating: compositional scaling via hierarchy\n");

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("RUGID Media Player (Ontological)")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
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

    // ═══════════════════════════════════════════════════════════════════════════
    // BUILD THE ONTOLOGICAL TREE
    // ═══════════════════════════════════════════════════════════════════════════
    let (ontology_root, cell_names) = build_ui_ontology();
    
    // Initial layout update
    let size = window.inner_size();
    update_layout(&mut runtime, &ontology_root, &cell_names, size.width as f32, size.height as f32);
    
    println!("\n✓ UI initialized with ontological indentation");
    println!("  - All sizes relative to parent");
    println!("  - No absolute pixel values in layout definition");
    println!("  - Scaling is compositional\n");
    println!("Starting event loop...\n");

    // Event Loop
    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

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
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                // On resize, re-resolve the ontological tree
                // This ensures true relativistic scaling where text respects its parent's actual dimensions
                if new_size.width > 0 && new_size.height > 0 {
                    update_layout(&mut runtime, &ontology_root, &cell_names, new_size.width as f32, new_size.height as f32);
                }
            }
            Event::AboutToWait => {
                runtime.tick();
            }
            _ => {}
        }
    }).unwrap();
}

fn update_layout(
    runtime: &mut Runtime,
    ontology_root: &OntologicalNode,
    cell_names: &HashMap<&str, CellId>,
    width: f32,
    height: f32
) {
    let screen = ResolvedTransform::screen(width, height);
    
    // Resolve the entire ontological tree to absolute positions WITH TEXT
    let resolved = OntologyResolver::resolve_with_text(ontology_root, screen);
    
    for (name, cell_id) in cell_names {
        if let Some(resolved_cell) = resolved.get(cell_id) {
            let transform = &resolved_cell.transform;
            
            // Create cell with geometry from ontological resolution
            let mut cell = Cell::new(VectorRegion::new(
                transform.y / height, // origin_p (normalized)
                transform.x / width,  // origin_s (normalized)
                transform.height / height, // extent_p
                transform.width / width,   // extent_s
            ));
            cell.id = *cell_id;
            
            // Use slider projector for timeline and volume
            if *name == "timeline" || *name == "volume_slider" {
                let value = if *name == "timeline" { 0.35 } else { 0.75 };
                runtime.register_cell(cell, Box::new(SliderProjector {
                    target: *cell_id,
                    geometry: VectorRegion::new(
                        transform.y / 100.0,
                        transform.x / 100.0,
                        transform.height / 100.0,
                        transform.width / 100.0,
                    ),
                    value,
                }), None, None);
            } else if let Some(text) = &resolved_cell.text {
                // Register with text - using RESOLVED PIXEL font size
                // The ontology resolver has already calculated this based on the parent's height
                // We re-resolve on resize, so this pixel value will be updated dynamically
                
                runtime.register_text_cell(
                    cell, 
                    Box::new(ButtonProjector {
                        target: *cell_id,
                        geometry: VectorRegion::new(
                            transform.y / 100.0,
                            transform.x / 100.0,
                            transform.height / 100.0,
                            transform.width / 100.0,
                        ),
                        label: text.content.clone(),
                    }),
                    &text.content,
                    text.x / width * 100.0,
                    text.y / height * 100.0,
                    text.font_size_px,
                    &text.color,
                    &text.text_anchor,
                    None,
                    None
                );
            } else {
                runtime.register_cell(cell, Box::new(ButtonProjector {
                    target: *cell_id,
                    geometry: VectorRegion::new(
                        transform.y / 100.0,
                        transform.x / 100.0,
                        transform.height / 100.0,
                        transform.width / 100.0,
                    ),
                    label: name.to_string(),
                }), None, None);
            }
        }
    }
}
