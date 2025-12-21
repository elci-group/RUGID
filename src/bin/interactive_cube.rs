//! Interactive 3D Cube GUI Demo
//!
//! Demonstrates RUGID's architecture for interactive 3D applications

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId};
use rugid::geometry::VectorRegion;
use rugid::geometry3d::Rotation3D;
use rugid::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, StackDirection, RelativeSize};
use rugid::widgets::ButtonProjector;
use rugid::platforms::bitmap::BitmapPlatform;
use std::collections::HashMap;

// Rotation state
struct RotationState {
    x_speed: f32,
    y_speed: f32,
    z_speed: f32,
    current_rotation: Rotation3D,
}

impl RotationState {
    fn new() -> Self {
        Self {
            x_speed: 1.0,
            y_speed: 1.5,
            z_speed: 0.5,
            current_rotation: Rotation3D::zero(),
        }
    }
    
    fn update(&mut self) {
        self.current_rotation.pitch += self.x_speed;
        self.current_rotation.yaw += self.y_speed;
        self.current_rotation.roll += self.z_speed;
        
        // Keep angles in reasonable range
        self.current_rotation.pitch %= 360.0;
        self.current_rotation.yaw %= 360.0;
        self.current_rotation.roll %= 360.0;
    }
}

fn main() {
    println!("🎮 RUGID Interactive 3D Cube Demo");
    println!("{}", "=".repeat(60));
    println!();
    
    // Create runtime with Bitmap platform
    let platform = BitmapPlatform::new(800, 600);
    let mut runtime = Runtime::new(60, Box::new(platform));
    
    // Build UI
    let mut cell_map = HashMap::new();
    let root = build_ui(&mut cell_map);
    
    // Resolve ontology
    let screen = ResolvedTransform::screen(800.0, 600.0);
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    println!("📐 UI Layout:");
    println!("  Cube canvas: 65% width (left side)");
    println!("  Control panel: 35% width (right side)");
    println!("    - X rotation controls (red)");
    println!("    - Y rotation controls (green)");
    println!("    - Z rotation controls (blue)");
    println!();
    
    // Register UI cells
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
            
            // Set colors for control widgets
            let color = if name.contains("x_") {
                "#E74C3C" // Red for X
            } else if name.contains("y_") {
                "#2ECC71" // Green for Y
            } else if name.contains("z_") {
                "#3498DB" // Blue for Z
            } else if *name == "cube_canvas" {
                "#1a1a1a" // Dark for cube area
            } else {
                "#34495E" // Gray for other controls
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
    
    println!("✅ UI initialized with {} widgets", cell_map.len());
    println!();
    println!("🎮 Simulated Controls:");
    println!(" Frame 150: Increase X speed → 2.0°/frame");
    println!("  Frame 300: Decrease Y speed → 0.5°/frame");
    println!("  Frame 450: Increase Z speed → 1.5°/frame");
    println!();
    println!("🎲 Starting interactive simulation...\n");
    
    // Initialize rotation state
    let mut rotation_state = RotationState::new();
    
    // Main loop - simulate 10 seconds at 60fps = 600 frames
    for frame in 0..600 {
        // Update rotation based on current speeds
        rotation_state.update();
        
        // Simulate user interactions
        let mut speed_changed = false;
        if frame == 150 {
            rotation_state.x_speed = 2.0;
            println!("⚡ User clicked X+ button");
            println!("   → X speed increased to 2.0°/frame\n");
            speed_changed = true;
        }
        if frame == 300 {
            rotation_state.y_speed = 0.5;
            println!("⚡ User clicked Y- button");
            println!("   → Y speed decreased to 0.5°/frame\n");
            speed_changed = true;
        }
        if frame == 450 {
            rotation_state.z_speed = 1.5;
            println!("⚡ User clicked Z+ button");
            println!("   → Z speed increased to 1.5°/frame\n");
            speed_changed = true;
        }
        
        // Log progress every second (60 frames)
        if frame % 60 == 0 || speed_changed {
            println!("Frame {:3}/600 | Rotation: {:6.1}° {:6.1}° {:6.1}° | Speeds: {:.1} {:.1} {:.1}",
                frame,
                rotation_state.current_rotation.pitch,
                rotation_state.current_rotation.yaw,
                rotation_state.current_rotation.roll,
                rotation_state.x_speed,
                rotation_state.y_speed,
                rotation_state.z_speed
            );
        }
        
        // Render cube (would update actual SVG in full implementation)
        // For now, just tick the runtime
        runtime.tick();
    }
    
    println!("\n✨ Simulation complete!");
    println!("\n📊 Final state:");
    println!("  Total rotation: {:.0}°, {:.0}°, {:.0}°",
        rotation_state.current_rotation.pitch,
        rotation_state.current_rotation.yaw,
        rotation_state.current_rotation.roll
    );
    println!("  Final speeds: {:.1}°, {:.1}°, {:.1}° per frame",
        rotation_state.x_speed,
        rotation_state.y_speed,
        rotation_state.z_speed
    );
    
    println!("\n💡 Full Implementation Would Include:");
    println!("  ✓ Real-time cube rendering in canvas");
    println!("  ✓ Click handlers on +/- buttons");
    println!("  ✓ Live speed value displays");
    println!("  ✓ Reset button functionality");
    println!("  ✓ Smooth transitions");
    println!("\n🎯 Architecture demonstrates RUGID's capability for");
    println!("   dynamic, interactive 3D applications!");
}

fn build_ui(cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    macro_rules! cell {
        ($name:expr) => {{
            let id = CellId::next();
            cells.insert($name, id);
            id
        }};
    }
    
    OntologicalNode::window("root", StackDirection::Primary)
        // Title bar
        .child(
            OntologicalNode::pane(
                "title_bar",
                "root",
                StackDirection::Secondary,
                RelativeSize::Percent(0.08),
                RelativeSize::Percent(1.0),
            )
                .child(OntologicalNode::widget(
                    cell!("title"),
                    "title_bar",
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(1.0),
                ))
        )
        // Main content area
        .child(
            OntologicalNode::pane(
                "main",
                "root",
                StackDirection::Secondary,
                RelativeSize::Percent(0.84),
                RelativeSize::Percent(1.0),
            )
                // Cube display area (left side - 65%)
                .child(
                    OntologicalNode::pane(
                        "cube_area",
                        "main",
                        StackDirection::Primary,
                        RelativeSize::Percent(1.0),
                        RelativeSize::Percent(0.65),
                    )
                        .child(OntologicalNode::widget(
                            cell!("cube_canvas"),
                            "cube_area",
                            RelativeSize::Percent(1.0),
                            RelativeSize::Percent(1.0),
                        ))
                )
                // Control panel (right side - 35%)
                .child(
                    OntologicalNode::pane(
                        "controls",
                        "main",
                        StackDirection::Primary,
                        RelativeSize::Percent(1.0),
                        RelativeSize::Percent(0.35),
                    )
                        // X rotation controls
                        .child(
                            OntologicalNode::pane(
                                "x_controls",
                                "controls",
                                StackDirection::Secondary,
                                RelativeSize::Percent(0.28),
                                RelativeSize::Percent(1.0),
                            )
                                .child(OntologicalNode::widget(
                                    cell!("x_label"),
                                    "x_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.2),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("x_decrease"),
                                    "x_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.3),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("x_value"),
                                    "x_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.2),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("x_increase"),
                                    "x_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.3),
                                ))
                        )
                        // Y rotation controls
                        .child(
                            OntologicalNode::pane(
                                "y_controls",
                                "controls",
                                StackDirection::Secondary,
                                RelativeSize::Percent(0.28),
                                RelativeSize::Percent(1.0),
                            )
                                .child(OntologicalNode::widget(
                                    cell!("y_label"),
                                    "y_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.2),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("y_decrease"),
                                    "y_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.3),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("y_value"),
                                    "y_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.2),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("y_increase"),
                                    "y_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.3),
                                ))
                        )
                        // Z rotation controls
                        .child(
                            OntologicalNode::pane(
                                "z_controls",
                                "controls",
                                StackDirection::Secondary,
                                RelativeSize::Percent(0.28),
                                RelativeSize::Percent(1.0),
                            )
                                .child(OntologicalNode::widget(
                                    cell!("z_label"),
                                    "z_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.2),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("z_decrease"),
                                    "z_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.3),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("z_value"),
                                    "z_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.2),
                                ))
                                .child(OntologicalNode::widget(
                                    cell!("z_increase"),
                                    "z_controls",
                                    RelativeSize::Percent(1.0),
                                    RelativeSize::Percent(0.3),
                                ))
                        )
                        // Reset button
                        .child(
                            OntologicalNode::pane(
                                "reset_area",
                                "controls",
                                StackDirection::Primary,
                                RelativeSize::Percent(0.16),
                                RelativeSize::Percent(1.0),
                            )
                                .child(OntologicalNode::widget(
                                    cell!("reset_button"),
                                    "reset_area",
                                    RelativeSize::Percent(0.6),
                                    RelativeSize::Percent(0.8),
                                ))
                        )
                )
        )
        // Status bar
        .child(
            OntologicalNode::pane(
                "status_bar",
                "root",
                StackDirection::Secondary,
                RelativeSize::Percent(0.08),
                RelativeSize::Percent(1.0),
            )
                .child(OntologicalNode::widget(
                    cell!("status"),
                    "status_bar",
                    RelativeSize::Percent(1.0),
                    RelativeSize::Percent(1.0),
                ))
        )
}
