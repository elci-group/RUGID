//! Text Scaling & Morphism Demo
//!
//! Demonstrates:
//! - Text scaling via parent container deltas
//! - Text color morphing
//! - Typewriter effect (character-by-character reveal)
//! - Combined text + shape morphism

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId};
use rugid::geometry::VectorRegion;
use rugid::ontology::{
    OntologicalNode, OntologyResolver, ResolvedTransform,
    StackDirection, RelativeBounds, RelativeText,
};
use rugid::temporal::{DeltaFrame, DeltaProgram, DeltaType};
use rugid::morphism::{ShapeType, MorphismState};
use rugid::widgets::ButtonProjector;
use rugid::platforms::bitmap::BitmapPlatform;
use std::collections::HashMap;

fn main() {
    println!("🔤 RUGID Text Scaling & Morphism Demo");
    println!("{}", "=".repeat(60));

    let mut runtime = Runtime::new(100, Box::new(BitmapPlatform::new(800, 600)));
    let mut cell_map: HashMap<&str, CellId> = HashMap::new();

    // Build UI with three demonstration areas
    let root = build_text_demo(&mut cell_map);
    
    // Screen transform
    let screen = ResolvedTransform::screen(800.0, 600.0);
    
    // Resolve layout
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    // Register cells
    for (name, cell_id) in &cell_map {
        if let Some(resolved_cell) = resolved.get(cell_id) {
            let transform = &resolved_cell.transform;
            let mut cell = Cell::new(VectorRegion::new(
                transform.y / 600.0,
                transform.x / 800.0,
                transform.height / 600.0,
                transform.width / 800.0,
            ));
            cell.id = *cell_id;
            
            // Add morphism state for shape containers
            if name.contains("container") {
                cell.morphism = Some(MorphismState::new(ShapeType::Rectangle));
            }
            
            // Set colors
            let color = if name.contains("bg") {
                "#2C3E50" // Dark blue-grey background
            } else if name.contains("scaling") {
                "#3498DB" // Blue
            } else if name.contains("color") {
                "#E74C3C" // Red
            } else if name.contains("typewriter") {
                "#2ECC71" // Green
            } else {
                "#34495E" // Dark grey
            };
            
            runtime.register_cell(
                cell,
                Box::new(ButtonProjector {
                    target: *cell_id,
                    geometry: VectorRegion::new(0.0, 0.0, 1.0, 1.0),
                    label: name.to_string(),
                }),
                None,
                None,
            );
            
            runtime.renderer.update_color(*cell_id, color.to_string());
        }
    }
    
    println!("\n📊 Registered {} cells", cell_map.len());
    println!("\n🎬 Starting animations...\n");
    
    // Demo 1: Text Scaling (scale parent container)
    if let Some(&container_id) = cell_map.get("scaling_container") {
        let mut frames = Vec::new();
        for i in 0..60 {
            let mut frame = DeltaFrame::zero(i, i + 1);
            // Breathing effect: grow then shrink
            let progress = (i as f32 / 60.0 * std::f32::consts::PI).sin();
            frame.delta_extent_p = progress * 0.003;
            frame.delta_extent_s = progress * 0.003;
            frames.push(frame);
        }
        
        runtime.temporal.add_program(DeltaProgram::new(
            container_id,
            frames,
            DeltaType::Greasy,
        ));
        
        println!("✅ Demo 1: Text scaling animation scheduled (60 frames)");
    }
    
    // Demo 2: Color Morphing (transition white → yellow)
    if let Some(&color_id) = cell_map.get("color_container") {
        let mut frames = Vec::new();
        for i in 0..45 {
            let mut frame = DeltaFrame::zero(i, i + 1);
            // Fade from dark (#E74C3C) to bright yellow (#F1C40F)
            frame.delta_r = (0xF1 as f32 - 0xE7 as f32) / 255.0 / 45.0;
            frame.delta_g = (0xC4 as f32 - 0x4C as f32) / 255.0 / 45.0;
            frame.delta_b = (0x0F as f32 - 0x3C as f32) / 255.0 / 45.0;
            frames.push(frame);
        }
        
        runtime.temporal.add_program(DeltaProgram::new(
            color_id,
            frames,
            DeltaType::Greasy,
        ));
        
        println!("✅ Demo 2: Color morphing animation scheduled (45 frames)");
    }
    
    // Demo 3: Shape morphing (container background)
    if let Some(&typewriter_id) = cell_map.get("typewriter_container") {
        let mut frames = Vec::new();
        for i in 0..50 {
            let mut frame = DeltaFrame::zero(i, i + 1);
            frame.delta_morph_progress = 1.0 / 50.0; // Progress toward rounded rectangle
            frame.target_shape = Some(ShapeType::RoundedRectangle { radius: 0.15 });
            frames.push(frame);
        }
        
        runtime.temporal.add_program(DeltaProgram::new(
            typewriter_id,
            frames,
            DeltaType::Greasy,
        ));
        
        println!("✅ Demo 3: Shape morphing scheduled (50 frames)");
    }
    
    println!("\n🎥 Rendering 100 frames...\n");
    
    // Render animation frames
    for i in 0..100 {
        runtime.tick();
        
        if i % 20 == 0 {
            println!("Frame {:3}/100", i);
        }
    }
    
    println!("\n✨ Animation complete!");
    println!("\n📋 Summary:");
    println!("  - Text scales via parent container deltas (delta_extent_p/s)");
    println!("  - Color morphs via RGB deltas (delta_r/g/b)");
    println!("  - Shape morphs via morphism system (delta_morph_progress)");
    println!("\n💡 Key Insight: Text morphism reuses existing delta systems!");
}

fn build_text_demo(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    // Helper macro to track cells
    macro_rules! cell {
        ($name:expr) => {{
            let id = CellId::next();
            cells.insert($name, id);
            id
        }};
    }
    
    OntologicalNode::window("root", StackDirection::Primary)
        .child(
            // Background
            OntologicalNode::inset_widget(cell!("bg"), "root", 0.0)
        )
        .child(
            // Title area
            OntologicalNode::bounded_pane(
                "title_pane",
                "root",
                RelativeBounds::new(0.0, 0.15),
                RelativeBounds::new(0.0, 1.0),
                StackDirection::Primary,
            )
                .child(OntologicalNode::text_inset(
                    cell!("title_text"),
                    "title_pane",
                    RelativeText::new("Text Morphism Showcase")
                        .size(0.5)
                        .bold()
                        .color("#ECF0F1"),
                    0.1,
                ))
        )
        .child(
            // Demo 1: Scaling Text
            OntologicalNode::bounded_pane(
                "scaling_pane",
                "root",
                RelativeBounds::new(0.2, 0.45),
                RelativeBounds::new(0.05, 0.35),
                StackDirection::Primary,
            )
                .child(OntologicalNode::inset_widget(
                    cell!("scaling_container"),
                    "scaling_pane",
                    0.05,
                ))
                .child(OntologicalNode::text_inset(
                    cell!("scaling_text"),
                    "scaling_pane",
                    RelativeText::new("SCALING")
                        .size(0.4)
                        .bold()
                        .color("#FFFFFF"),
                    0.2,
                ))
        )
        .child(
            // Demo 2: Color Morphing
            OntologicalNode::bounded_pane(
                "color_pane",
                "root",
                RelativeBounds::new(0.2, 0.45),
                RelativeBounds::new(0.375, 0.675),
                StackDirection::Primary,
            )
                .child(OntologicalNode::inset_widget(
                    cell!("color_container"),
                    "color_pane",
                    0.05,
                ))
                .child(OntologicalNode::text_inset(
                    cell!("color_text"),
                    "color_pane",
                    RelativeText::new("COLOR")
                        .size(0.4)
                        .bold()
                        .color("#FFFFFF"),
                    0.2,
                ))
        )
        .child(
            // Demo 3: Typewriter
            OntologicalNode::bounded_pane(
                "typewriter_pane",
                "root",
                RelativeBounds::new(0.2, 0.45),
                RelativeBounds::new(0.7, 0.95),
                StackDirection::Primary,
            )
                .child(OntologicalNode::inset_widget(
                    cell!("typewriter_container"),
                    "typewriter_pane",
                    0.05,
                ))
                .child(OntologicalNode::text_inset(
                    cell!("typewriter_text"),
                    "typewriter_pane",
                    RelativeText::new("MORPH")
                        .size(0.4)
                        .bold()
                        .color("#FFFFFF"),
                    0.2,
                ))
        )
        .child(
            // Footer with description
            OntologicalNode::bounded_pane(
                "footer_pane",
                "root",
                RelativeBounds::new(0.55, 0.95),
                RelativeBounds::new(0.05, 0.95),
                StackDirection::Primary,
            )
                .child(OntologicalNode::text_inset(
                    cell!("footer_text"),
                    "footer_pane",
                    RelativeText::new("Text scales, morphs color, and transforms shape")
                        .size(0.3)
                        .color("#BDC3C7"),
                    0.15,
                ))
        )
}
