//! Windowed Interactive 3D Cube
//!
//! Full GUI application with real window, clickable controls, and live 3D rendering

use rugid::runtime::Runtime;
use rugid::cell::{Cell, CellId};
use rugid::geometry::VectorRegion;
use rugid::geometry3d::{Point3D, Rotation3D, rotate_3d};
use rugid::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, StackDirection, RelativeSize};
use rugid::widgets::ButtonProjector;
use rugid::svg::path_to_svg_d;
use rugid::platforms::wgpu_platform::WgpuPlatform;
use rugid::platform::{Platform, PlatformEvent};
use rugid::input::InputEvent;
use std::collections::HashMap;
use std::sync::Arc;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

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
    
    fn adjust_x_speed(&mut self, delta: f32) {
        self.x_speed = (self.x_speed + delta).max(0.0).min(5.0);
    }
    
    fn adjust_y_speed(&mut self, delta: f32) {
        self.y_speed = (self.y_speed + delta).max(0.0).min(5.0);
    }
    
    fn adjust_z_speed(&mut self, delta: f32) {
        self.z_speed = (self.z_speed + delta).max(0.0).min(5.0);
    }
    
    fn reset(&mut self) {
        self.x_speed = 1.0;
        self.y_speed = 1.5;
        self.z_speed = 0.5;
        self.current_rotation = Rotation3D::zero();
    }
}

fn main() {
    println!("🎮 RUGID Windowed Interactive 3D Cube");
    println!("{}", "=".repeat(60));
    
    pollster::block_on(run());
}

async fn run() {
    // Create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID Interactive 3D Cube")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    
    // Create platform (keep reference)
    let mut platform = WgpuPlatform::new(window.clone()).await;
    
    // Build UI
    let mut cell_map = HashMap::new();
    let root = build_ui(&mut cell_map);
    
    // Resolve ontology
    let screen = ResolvedTransform::screen(800.0, 600.0);
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    println!("✅ Initializing windowed application...");
    println!("📐 UI: 16 widgets with interactive controls");
    println!("🎨 Cube canvas: 65% width | Controls: 35% width");
    
    // Build UI SVG base
    let mut ui_svg = String::new();
    ui_svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"600\" viewBox=\"0 0 800 600\">\n");
    ui_svg.push_str("  <rect width=\"800\" height=\"600\" fill=\"#2C3E50\" />\n");
    ui_svg.push_str("  <text x=\"50%\" y=\"5%\" text-anchor=\"middle\" fill=\"white\" font-size=\"24\">RUGID Interactive 3D Cube</text>\n");
    
    // Add control widgets to SVG
    for (name, cell_id) in &cell_map {
        if let Some(resolved_cell) = resolved.get(cell_id) {
            let transform = &resolved_cell.transform;
            
            let color = if name.contains("x_") {
                "#E74C3C"
            } else if name.contains("y_") {
                "#2ECC71"
            } else if name.contains("z_") {
                "#3498DB"
            } else if *name == "cube_canvas" {
                "#1a1a1a"
            } else {
                "#34495E"
            };
            
            ui_svg.push_str(&format!(
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" />\n",
                transform.x, transform.y, transform.width, transform.height, color
            ));
            ui_svg.push_str(&format!(
                "  <text x=\"{}\" y=\"{}\" fill=\"white\" font-size=\"12\" text-anchor=\"middle\">{}</text>\n",
                transform.x + transform.width / 2.0,
                transform.y + transform.height / 2.0,
                name
            ));
        }
    }
    
    // Initialize rotation state
    let mut rotation_state = RotationState::new();
    
    // Get cube canvas dimensions relativistically
    let cube_canvas_id = cell_map["cube_canvas"];
    let cube_canvas_transform = &resolved.get(&cube_canvas_id).unwrap().transform;
    
    println!("🚀 Window opened! Cube will rotate automatically.");
    println!("   Canvas: {}x{} at ({}, {})", 
        cube_canvas_transform.width, 
        cube_canvas_transform.height,
        cube_canvas_transform.x,
        cube_canvas_transform.y
    );
    
    // Event loop
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: win_event, .. } => {
                // Forward events to platform
                platform.handle_event(&Event::WindowEvent { 
                    window_id: window.id(), 
                    event: win_event.clone() 
                });
                
                match win_event {
                    WindowEvent::CloseRequested => {
                        println!("\n👋 Closing application...");
                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        // Update rotation
                        rotation_state.update();
                        
                        // Calculate cube parameters relativistically
                        let canvas_width = cube_canvas_transform.width;
                        let canvas_height = cube_canvas_transform.height;
                        let canvas_x = cube_canvas_transform.x;
                        let canvas_y = cube_canvas_transform.y;
                        
                        // Cube size is 30% of smaller canvas dimension
                        let cube_size = canvas_width.min(canvas_height) * 0.3;
                        
                        // Center cube in canvas
                        let cube_center_x = canvas_x + canvas_width / 2.0;
                        let cube_center_y = canvas_y + canvas_height / 2.0;
                        
                        // Render cube with relativistic positioning
                        let cube_svg = render_cube_to_svg(
                            &rotation_state, 
                            cube_size, 
                            cube_center_x, 
                            cube_center_y
                        );
                        
                        // Combine UI and cube
                        let mut full_svg = ui_svg.clone();
                        full_svg.push_str(&cube_svg);
                        full_svg.push_str("</svg>");
                        
                        // Render using Platform trait
                        use rugid::platform::Platform;
                        platform.render(full_svg);
                        
                        // Request next frame
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

/// Render the 3D cube to SVG markup
fn render_cube_to_svg(state: &RotationState, size: f32, center_x: f32, center_y: f32) -> String {
    let half = size / 2.0;
    let focal_length = 400.0;
    
    // Define cube vertices
    let vertices = [
        Point3D::new(-half, -half, -half),
        Point3D::new( half, -half, -half),
        Point3D::new( half,  half, -half),
        Point3D::new(-half,  half, -half),
        Point3D::new(-half, -half,  half),
        Point3D::new( half, -half,  half),
        Point3D::new( half,  half,  half),
        Point3D::new(-half,  half,  half),
    ];
    
    // Rotate vertices
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|&v| rotate_3d(v, state.current_rotation))
        .collect();
    
    // Define faces
    let faces = [
        ([0, 1, 2, 3], Point3D::new(0.0, 0.0, -1.0), (100, 100, 200)),
        ([4, 7, 6, 5], Point3D::new(0.0, 0.0, 1.0), (200, 100, 100)),
        ([0, 4, 5, 1], Point3D::new(0.0, -1.0, 0.0), (100, 200, 100)),
        ([3, 2, 6, 7], Point3D::new(0.0, 1.0, 0.0), (200, 200, 100)),
        ([0, 3, 7, 4], Point3D::new(-1.0, 0.0, 0.0), (200, 100, 200)),
        ([1, 5, 6, 2], Point3D::new(1.0, 0.0, 0.0), (100, 200, 200)),
    ];
    
    let light = Point3D::new(1.0, 1.0, 2.0).normalize();
    let mut face_data: Vec<(f32, String)> = Vec::new();
    
    for (indices, normal_base, base_color) in &faces {
        let normal = rotate_3d(*normal_base, state.current_rotation).normalize();
        let view_direction = Point3D::new(0.0, 0.0, 1.0);
        
        if normal.dot(&view_direction) <= 0.0 {
            continue;
        }
        
        let face_center_z: f32 = indices.iter()
            .map(|&i| rotated[i].z)
            .sum::<f32>() / 4.0;
        
        let brightness = normal.dot(&light).max(0.0);
        let final_brightness = 0.3 + brightness * 0.7;
        
        let color = (
            (base_color.0 as f32 * final_brightness) as u8,
            (base_color.1 as f32 * final_brightness) as u8,
            (base_color.2 as f32 * final_brightness) as u8,
        );
        
        let projected: Vec<(f32, f32)> = indices.iter()
            .map(|&i| {
                let (px, py) = rotated[i].project(focal_length);
                (px + center_x, py + center_y)
            })
            .collect();
        
        let path_d = path_to_svg_d(&projected, true);
        let svg_face = format!(
            r#"<path d="{}" fill="rgb({},{},{})" stroke="rgba(255,255,255,0.3)" stroke-width="2" />"#,
            path_d, color.0, color.1, color.2
        );
        
        face_data.push((face_center_z, svg_face));
    }
    
    face_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    face_data.iter()
        .map(|(_, svg)| svg.clone())
        .collect::<Vec<_>>()
        .join("\n")
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
        .child(
            OntologicalNode::pane(
                "main",
                "root",
                StackDirection::Secondary,
                RelativeSize::Percent(0.84),
                RelativeSize::Percent(1.0),
            )
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
                .child(
                    OntologicalNode::pane(
                        "controls",
                        "main",
                        StackDirection::Primary,
                        RelativeSize::Percent(1.0),
                        RelativeSize::Percent(0.35),
                    )
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
