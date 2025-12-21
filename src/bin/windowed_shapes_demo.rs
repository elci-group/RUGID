//! Windowed Equilateral Shapes Demo
//!
//! Full GUI application displaying rotating Tetrahedron, Cube, and Octahedron.

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
        
        self.current_rotation.pitch %= 360.0;
        self.current_rotation.yaw %= 360.0;
        self.current_rotation.roll %= 360.0;
    }
}

#[derive(Clone, Copy, Debug)]
enum ShapeType {
    Tetrahedron,
    Cube,
    Octahedron,
}

impl ShapeType {
    fn name(&self) -> &'static str {
        match self {
            ShapeType::Tetrahedron => "Tetrahedron",
            ShapeType::Cube => "Cube",
            ShapeType::Octahedron => "Octahedron",
        }
    }
}

fn main() {
    println!("🎮 RUGID Windowed Shapes Demo");
    println!("{}", "=".repeat(60));
    
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("RUGID Equilateral Shapes Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 600))
        .build(&event_loop)
        .unwrap();
    
    let window = Arc::new(window);
    let mut platform = WgpuPlatform::new(window.clone()).await;
    
    let mut cell_map = HashMap::new();
    let root = build_ui(&mut cell_map);
    
    let screen = ResolvedTransform::screen(1200.0, 600.0);
    let resolved = OntologyResolver::resolve_with_text(&root, screen);
    
    println!("✅ Initializing windowed application...");
    
    // Build UI SVG base
    let mut ui_svg = String::new();
    ui_svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"600\" viewBox=\"0 0 1200 600\">\n");
    ui_svg.push_str("  <rect width=\"1200\" height=\"600\" fill=\"#2C3E50\" />\n");
    ui_svg.push_str("  <text x=\"50%\" y=\"5%\" text-anchor=\"middle\" fill=\"white\" font-size=\"24\">RUGID Equilateral Shapes Demo</text>\n");
    
    // Add layout frames
    for (name, cell_id) in &cell_map {
        if let Some(resolved_cell) = resolved.get(cell_id) {
            let transform = &resolved_cell.transform;
            
            let color = if name.contains("canvas") {
                "#1a1a1a"
            } else {
                "#34495E"
            };
            
            ui_svg.push_str(&format!(
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" />\n",
                transform.x, transform.y, transform.width, transform.height, color
            ));
            
            if !name.contains("canvas") {
                 ui_svg.push_str(&format!(
                    "  <text x=\"{}\" y=\"{}\" fill=\"white\" font-size=\"16\" text-anchor=\"middle\">{}</text>\n",
                    transform.x + transform.width / 2.0,
                    transform.y + transform.height / 2.0 + 5.0,
                    name
                ));
            }
        }
    }
    
    let mut rotation_state = RotationState::new();
    
    // Get canvas IDs
    let tetra_id = cell_map["Tetrahedron_canvas"];
    let cube_id = cell_map["Cube_canvas"];
    let octa_id = cell_map["Octahedron_canvas"];
    
    let tetra_rect = &resolved.get(&tetra_id).unwrap().transform;
    let cube_rect = &resolved.get(&cube_id).unwrap().transform;
    let octa_rect = &resolved.get(&octa_id).unwrap().transform;

    println!("🚀 Window opened!");

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: win_event, .. } => {
                platform.handle_event(&Event::WindowEvent { 
                    window_id: window.id(), 
                    event: win_event.clone() 
                });
                
                match win_event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        rotation_state.update();
                        
                        let mut full_svg = ui_svg.clone();
                        
                        // Render Tetrahedron
                        full_svg.push_str(&render_shape_in_rect(
                            ShapeType::Tetrahedron,
                            &rotation_state,
                            tetra_rect
                        ));
                        
                        // Render Cube
                        full_svg.push_str(&render_shape_in_rect(
                            ShapeType::Cube,
                            &rotation_state,
                            cube_rect
                        ));
                        
                        // Render Octahedron
                        full_svg.push_str(&render_shape_in_rect(
                            ShapeType::Octahedron,
                            &rotation_state,
                            octa_rect
                        ));
                        
                        full_svg.push_str("</svg>");
                        
                        use rugid::platform::Platform;
                        platform.render(full_svg);
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

fn render_shape_in_rect(
    shape_type: ShapeType,
    state: &RotationState,
    rect: &ResolvedTransform
) -> String {
    let size = rect.width.min(rect.height) * 0.5;
    let center_x = rect.x + rect.width / 2.0;
    let center_y = rect.y + rect.height / 2.0;
    
    render_shape_to_svg(shape_type, state, size, center_x, center_y)
}

fn render_shape_to_svg(
    shape_type: ShapeType,
    state: &RotationState,
    size: f32,
    center_x: f32,
    center_y: f32
) -> String {
    let focal_length = 400.0;
    
    let (vertices, faces, colors) = get_shape_data(shape_type, size);
    
    // Precompute normals
    let normals: Vec<Point3D> = faces.iter().map(|indices| {
        let v0 = vertices[indices[0]];
        let v1 = vertices[indices[1]];
        let v2 = vertices[indices[2]];
        compute_normal(v0, v1, v2)
    }).collect();

    // Rotate vertices
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|&v| rotate_3d(v, state.current_rotation))
        .collect();
        
    let light = Point3D::new(1.0, 1.0, 2.0).normalize();
    let view_direction = Point3D::new(0.0, 0.0, 1.0);
    
    let mut face_data: Vec<(f32, String)> = Vec::new();
    
    for (i, indices) in faces.iter().enumerate() {
        let normal = rotate_3d(normals[i], state.current_rotation).normalize();
        
        if normal.dot(&view_direction) <= 0.0 {
            continue;
        }
        
        let face_center_z: f32 = indices.iter()
            .map(|&idx| rotated[idx].z)
            .sum::<f32>() / indices.len() as f32;
            
        let brightness = normal.dot(&light).max(0.0);
        let final_brightness = 0.3 + brightness * 0.7;
        
        let base_color = colors[i % colors.len()];
        let color = (
            (base_color.0 as f32 * final_brightness) as u8,
            (base_color.1 as f32 * final_brightness) as u8,
            (base_color.2 as f32 * final_brightness) as u8,
        );
        
        let projected: Vec<(f32, f32)> = indices.iter()
            .map(|&idx| {
                let (px, py) = rotated[idx].project(focal_length);
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

fn compute_normal(p1: Point3D, p2: Point3D, p3: Point3D) -> Point3D {
    let u = Point3D::new(p2.x - p1.x, p2.y - p1.y, p2.z - p1.z);
    let v = Point3D::new(p3.x - p1.x, p3.y - p1.y, p3.z - p1.z);
    
    let nx = u.y * v.z - u.z * v.y;
    let ny = u.z * v.x - u.x * v.z;
    let nz = u.x * v.y - u.y * v.x;
    
    Point3D::new(nx, ny, nz).normalize()
}

fn get_shape_data(shape_type: ShapeType, size: f32) -> (Vec<Point3D>, Vec<Vec<usize>>, Vec<(u8, u8, u8)>) {
    let half = size / 2.0;
    
    match shape_type {
        ShapeType::Tetrahedron => {
            let s = size * 0.7;
            let vertices = vec![
                Point3D::new( s,  s,  s),
                Point3D::new( s, -s, -s),
                Point3D::new(-s,  s, -s),
                Point3D::new(-s, -s,  s),
            ];
            let faces = vec![
                vec![0, 1, 2],
                vec![0, 3, 1],
                vec![0, 2, 3],
                vec![1, 3, 2],
            ];
            let colors = vec![
                (255, 100, 100), (100, 255, 100), (100, 100, 255), (255, 255, 100),
            ];
            (vertices, faces, colors)
        },
        ShapeType::Cube => {
            let vertices = vec![
                Point3D::new(-half, -half, -half),
                Point3D::new( half, -half, -half),
                Point3D::new( half,  half, -half),
                Point3D::new(-half,  half, -half),
                Point3D::new(-half, -half,  half),
                Point3D::new( half, -half,  half),
                Point3D::new( half,  half,  half),
                Point3D::new(-half,  half,  half),
            ];
            let faces = vec![
                vec![0, 1, 2, 3], vec![4, 7, 6, 5], vec![0, 4, 5, 1],
                vec![3, 2, 6, 7], vec![0, 3, 7, 4], vec![1, 5, 6, 2],
            ];
            let colors = vec![
                (100, 100, 200), (200, 100, 100), (100, 200, 100),
                (200, 200, 100), (200, 100, 200), (100, 200, 200),
            ];
            (vertices, faces, colors)
        },
        ShapeType::Octahedron => {
            let s = size * 0.8;
            let vertices = vec![
                Point3D::new( s,  0.0,  0.0),
                Point3D::new(-s,  0.0,  0.0),
                Point3D::new( 0.0,  s,  0.0),
                Point3D::new( 0.0, -s,  0.0),
                Point3D::new( 0.0,  0.0,  s),
                Point3D::new( 0.0,  0.0, -s),
            ];
            let faces = vec![
                vec![4, 0, 2], vec![4, 2, 1], vec![4, 1, 3], vec![4, 3, 0],
                vec![5, 2, 0], vec![5, 1, 2], vec![5, 3, 1], vec![5, 0, 3],
            ];
            let colors = vec![
                (255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0),
                (255, 0, 255), (0, 255, 255), (255, 128, 0), (128, 0, 255),
            ];
            (vertices, faces, colors)
        }
    }
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
                RelativeSize::Percent(0.1),
                RelativeSize::Percent(1.0),
            )
        )
        .child(
            OntologicalNode::pane(
                "main_area",
                "root",
                StackDirection::Secondary,
                RelativeSize::Percent(0.9),
                RelativeSize::Percent(1.0),
            )
                .child(create_shape_pane("Tetrahedron", cells))
                .child(create_shape_pane("Cube", cells))
                .child(create_shape_pane("Octahedron", cells))
        )
}

fn create_shape_pane(name: &'static str, cells: &mut HashMap<&'static str, CellId>) -> OntologicalNode {
    let canvas_name = Box::leak(format!("{}_canvas", name).into_boxed_str());
    let id = CellId::next();
    cells.insert(canvas_name, id);
    
    let label_name = Box::leak(format!("{}", name).into_boxed_str());
    let label_id = CellId::next();
    cells.insert(label_name, label_id);

    OntologicalNode::pane(
        name,
        "main_area",
        StackDirection::Primary,
        RelativeSize::Percent(1.0),
        RelativeSize::Percent(0.33),
    )
        .child(OntologicalNode::widget(
            label_id,
            name,
            RelativeSize::Percent(0.1),
            RelativeSize::Percent(1.0),
        ))
        .child(OntologicalNode::widget(
            id,
            name,
            RelativeSize::Percent(0.9),
            RelativeSize::Percent(1.0),
        ))
}
