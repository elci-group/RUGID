//! Ontological Morphism Demo
//!
//! Demonstrates declarative shape definitions in the ontological hierarchy

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId};
use rugid::geometry::VectorRegion;
use rugid::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, StackDirection, RelativeSize};
use rugid::morphism::{ShapeType, MorphismState};
use rugid::temporal::{DeltaFrame, DeltaProgram, DeltaType};
use rugid::widgets::ButtonProjector;
use rugid::platforms::bitmap::BitmapPlatform;
use std::collections::HashMap;

fn main() {
    println!("🏛️ RUGID Ontological Morphism Demo");
    println!("{}", "=".repeat(60));

    let mut runtime = Runtime::new(100, Box::new(BitmapPlatform::new(800, 600)));
    let mut cell_map: HashMap<&str, CellId> = HashMap::new();

    // Build UI with declaratively shaped widgets
    let root = build_shaped_layout(&mut cell_map);
    
    let screen = ResolvedTransform::screen(800.0, 600.0);
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    // Register cells with their declared shapes
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
            
            // Set morphism state based on resolved shape
            // (In full implementation, this would come from LocalSpace.shape)
            if name.contains("circle") {
                cell.morphism = Some(MorphismState::new(ShapeType::Circle));
            } else if name.contains("rounded") {
                cell.morphism = Some(MorphismState::new(ShapeType::RoundedRectangle { radius: 0.15 }));
            } else if name.contains("poly") {
                cell.morphism = Some(MorphismState::new(ShapeType::Polygon { sides: 6 }));
            } else {
                cell.morphism = Some(MorphismState::new(ShapeType::Rectangle));
            }
            
            let color = if name.contains("circle") {
                "#3498DB" // Blue
            } else if name.contains("rounded") {
                "#E74C3C" // Red
            } else if name.contains("poly") {
                "#2ECC71" // Green
            } else {
                "#9B59B6" // Purple
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
    
    println!("\n📊 Registered {} shaped cells", cell_map.len());
    println!("\n🎬 Scheduling morphing animations...\n");
    
    // Demo: Circles morph to squares
    if let Some(&circle_id) = cell_map.get("circle_left") {
        let mut frames = Vec::new();
        for i in 0..50 {
            let mut frame = DeltaFrame::zero(i, i + 1);
            frame.delta_morph_progress = 1.0 / 50.0;
            frame.target_shape = Some(ShapeType::Rectangle);
            frames.push(frame);
        }
        runtime.temporal.add_program(DeltaProgram::new(
            circle_id, frames, DeltaType::Greasy
        ));
        println!("✅ Circle → Rectangle animation scheduled");
    }
    
    // Demo: Squares morph to rounded
    if let Some(&rect_id) = cell_map.get("rect_right") {
        let mut frames = Vec::new();
        for i in 0..40 {
            let mut frame = DeltaFrame::zero(i, i + 1);
            frame.delta_morph_progress = 1.0 / 40.0;
            frame.target_shape = Some(ShapeType::RoundedRectangle { radius: 0.25 });
            frames.push(frame);
        }
        runtime.temporal.add_program(DeltaProgram::new(
            rect_id, frames, DeltaType::Greasy
        ));
        println!("✅ Rectangle → Rounded animation scheduled");
    }
    
    // Demo: Polygons morph to circles
    if let Some(&poly_id) = cell_map.get("poly_center") {
        let mut frames = Vec::new();
        for i in 0..60 {
            let mut frame = DeltaFrame::zero(i, i + 1);
            frame.delta_morph_progress = 1.0 / 60.0;
            frame.target_shape = Some(ShapeType::Circle);
            frames.push(frame);
        }
        runtime.temporal.add_program(DeltaProgram::new(
            poly_id, frames, DeltaType::Greasy
        ));
        println!("✅ Polygon → Circle animation scheduled");
    }
    
    println!("\n🎥 Rendering 100 frames...\n");
    
    for i in 0..100 {
        runtime.tick();
        if i % 25 == 0 {
            println!("Frame {:3}/100", i);
        }
    }
    
    println!("\n✨ Demo complete!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  ✓ Declarative shape definitions in ontological nodes");
    println!("  ✓ Convenience methods: circular_widget(), rounded_widget(), shaped_widget()");
    println!("  ✓ Shape morphing via temporal delta system");
    println!("  ✓ Multiple shapes morphing simultaneously");
}

fn build_shaped_layout(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    macro_rules! cell {
        ($name:expr) => {{
            let id = CellId::next();
            cells.insert($name, id);
            id
        }};
    }
    
    OntologicalNode::window("root", StackDirection::Primary)
        .child(
            // Row 1: Circles
            OntologicalNode::pane(
                "row1",
                "root",
                StackDirection::Secondary,
                RelativeSize::Percent(0.33),
                RelativeSize::Percent(1.0),
            )
                .child(OntologicalNode::circular_widget(
                    cell!("circle_left"),
                    "row1",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                ))
                .child(OntologicalNode::circular_widget(
                    cell!("circle_middle"),
                    "row1",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                ))
                .child(OntologicalNode::circular_widget(
                    cell!("circle_right"),
                    "row1",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                ))
        )
        .child(
            // Row 2: Mixed shapes
            OntologicalNode::pane(
                "row2",
                "root",
                StackDirection::Secondary,
                RelativeSize::Percent(0.33),
                RelativeSize::Percent(1.0),
            )
                .child(OntologicalNode::rounded_widget(
                    cell!("rounded_left"),
                    "row2",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                    0.15,
                ))
                .child(OntologicalNode::shaped_widget(
                    cell!("poly_center"),
                    "row2",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                    ShapeType::Polygon { sides: 6 },
                ))
                .child(OntologicalNode::rounded_widget(
                    cell!("rounded_right"),
                    "row2",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                    0.15,
                ))
        )
        .child(
            // Row 3: Rectangles
            OntologicalNode::pane(
                "row3",
                "root",
                StackDirection::Secondary,
                RelativeSize::Percent(0.33),
                RelativeSize::Percent(1.0),
            )
                .child(OntologicalNode::widget(
                    cell!("rect_left"),
                    "row3",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                ))
                .child(OntologicalNode::widget(
                    cell!("rect_middle"),
                    "row3",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                ))
                .child(OntologicalNode::widget(
                    cell!("rect_right"),
                    "row3",
                    RelativeSize::Percent(0.3),
                    RelativeSize::Percent(0.3),
                ))
        )
}
