use crate::geometry3d::{Point3D, Rotation3D, rotate_3d};
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeType {
    Tetrahedron,
    Cube,
    Octahedron,
    Pyramid,
    Torus,
    F1Car,
    Jabulani,
    MercedesW14,
    CornellBox,
    Sphere,
}


/// Render a 3D shape with optional component animation
pub fn render_shape(
    shape_type: ShapeType,
    center_x: f32,
    center_y: f32,
    size: f32,
    rotation: Rotation3D,
    opacity: f32,
    offset: Point3D,
    scene: &crate::physics::optics::Scene,
) -> String {
    render_shape_animated(shape_type, center_x, center_y, size, rotation, opacity, offset, 0.0, scene, None)
}

/// Render a 3D shape with component animation
pub fn render_shape_animated(
    shape_type: ShapeType,
    center_x: f32,
    center_y: f32,
    size: f32,
    rotation: Rotation3D,
    opacity: f32,
    offset: Point3D,
    component_angle: f32,
    scene: &crate::physics::optics::Scene,
    reflection_svg: Option<String>,
) -> String {
    let focal_length = 400.0;
    let (vertices, faces, colors) = get_shape_data_animated(shape_type, size, component_angle);
    
    // Precompute normals
    let normals: Vec<Point3D> = faces.iter().map(|indices| {
        let v0 = vertices[indices[0]];
        let v1 = vertices[indices[1]];
        let v2 = vertices[indices[2]];
        compute_normal(v0, v1, v2)
    }).collect();

    // Rotate vertices
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|&v| rotate_3d(v, rotation))
        .collect();
        
    let mut face_data: Vec<(f32, String)> = Vec::new();
    let mut defs = String::new();
    let mut grad_counter = 0;
    let view_direction = Point3D::new(0.0, 0.0, 1.0); // Camera is looking down -Z, so +Z is "away" from camera
    let view_pos = Point3D::new(0.0, 0.0, -focal_length); 

    // Initialize Tessellator
    let tessellator = crate::physics::Tessellator::new(crate::physics::TessellationConfig::default());

    for (i, indices) in faces.iter().enumerate() {
        let normal = rotate_3d(normals[i], rotation).normalize();
        
        // Back-face culling (only if opaque)
        if opacity >= 1.0 && normal.dot(&view_direction) > 0.0 {
            continue;
        }
        
        // Calculate face center in world space
        let mut face_center_world = Point3D::new(0.0, 0.0, 0.0);
        let mut face_vertices_world = Vec::new();
        
        for &idx in indices {
            let v = rotated[idx];
            let v_world = Point3D::new(
                v.x + offset.x,
                v.y + offset.y,
                v.z + offset.z
            );
            face_center_world.x += v_world.x;
            face_center_world.y += v_world.y;
            face_center_world.z += v_world.z;
            face_vertices_world.push(v_world);
        }
        face_center_world.x /= indices.len() as f32;
        face_center_world.y /= indices.len() as f32;
        face_center_world.z /= indices.len() as f32;

        // Construct RenderFace for lighting engine
        let base_color = colors[i % colors.len()];
        
        // Material selection based on shape type
        let material = match shape_type {
            ShapeType::Jabulani => crate::physics::light::OpticalMaterial::metal(base_color),
            ShapeType::Sphere => crate::physics::light::OpticalMaterial::metal(base_color),
            ShapeType::CornellBox => crate::physics::light::OpticalMaterial::matte(base_color),
            _ => crate::physics::light::OpticalMaterial::plastic(base_color),
        };
        
        let render_face = crate::physics::optics::RenderFace {
            vertices: face_vertices_world.clone(),
            normal: rotate_3d(normals[i], rotation).normalize(), // World normal
            center: face_center_world,
            material: material.clone(),
            parent_id: None,
        };

        // --- VECTOR RAY TRACING: TESSELLATION ---
        // If the material is shiny (Metal/Plastic), use adaptive tessellation
        let use_tessellation = matches!(material.surface, crate::physics::light::SurfaceType::Metal { .. } | crate::physics::light::SurfaceType::Plastic { .. });
        
        if use_tessellation {
            let tessellated = tessellator.tessellate(&render_face, scene, view_pos);
            
            for tri in tessellated {
                grad_counter += 1;
                let grad_id = format!("grad_{}_{}", i, grad_counter);
                
                // Create gradient for this micro-triangle
                // We use the vertex colors calculated by the tessellator
                defs.push_str(&format!(
                    r##"<linearGradient id="{}" x1="0%" y1="0%" x2="100%" y2="0%">
                        <stop offset="0%" stop-color="rgb({},{},{})" />
                        <stop offset="100%" stop-color="rgb({},{},{})" />
                    </linearGradient>"##,
                    grad_id,
                    tri.color_v1.0, tri.color_v1.1, tri.color_v1.2,
                    tri.color_v2.0, tri.color_v2.1, tri.color_v2.2
                ));
                
                // Project vertices
                let mut points_str = String::new();
                for v in &tri.vertices {
                    let (px, py) = v.project(focal_length);
                    points_str.push_str(&format!("{:.1},{:.1} ", px + center_x, py + center_y));
                }
                
                let svg_face = format!(
                    r##"<polygon points="{}" fill="url(#{})" stroke="none" />"##,
                    points_str.trim(), grad_id
                );
                
                // Use the original face depth for sorting (approximation)
                face_data.push((face_center_world.z, svg_face));
            }
        } else {
            // Fallback to standard rendering for Matte/Textured
            
            // Compute Gradient (Legacy Path)
            let gradient = crate::physics::optics::compute_face_gradient(&render_face, scene, view_pos);
            
            // Texture / Gradient Handling
            let fill_attr = match &render_face.material.texture {
                crate::physics::light::TextureType::Checkerboard { color1, color2, scale } => {
                    grad_counter += 1;
                    let pat_id = format!("pat_check_{}_{}", i, grad_counter);
                    let s = scale; // Scale in pixels
                    
                    defs.push_str(&format!(
                        r##"<pattern id="{}" x="0" y="0" width="{}" height="{}" patternUnits="userSpaceOnUse">
                            <rect width="{}" height="{}" fill="rgb({},{},{})" />
                            <rect width="{}" height="{}" fill="rgb({},{},{})" />
                            <rect x="{}" y="{}" width="{}" height="{}" fill="rgb({},{},{})" />
                        </pattern>"##,
                        pat_id, s, s,
                        s, s, color2.0, color2.1, color2.2,
                        s/2.0, s/2.0, color1.0, color1.1, color1.2,
                        s/2.0, s/2.0, s/2.0, s/2.0, color1.0, color1.1, color1.2
                    ));
                    format!("url(#{})", pat_id)
                },
                crate::physics::light::TextureType::Image { path, opacity: img_opacity } => {
                    grad_counter += 1;
                    let pat_id = format!("pat_img_{}_{}", i, grad_counter);
                    
                    defs.push_str(&format!(
                        r##"<pattern id="{}" width="1" height="1" patternContentUnits="objectBoundingBox">
                            <image href="{}" width="1" height="1" preserveAspectRatio="none" opacity="{}" />
                        </pattern>"##,
                        pat_id, path, img_opacity
                    ));
                    format!("url(#{})", pat_id)
                },
                crate::physics::light::TextureType::None => {
                    if gradient.is_gradient {
                        grad_counter += 1;
                        let grad_id = format!("grad_{}_{}", i, grad_counter); // Unique ID
                        
                        if gradient.radial {
                             defs.push_str(&format!(
                                r##"<radialGradient id="{}" cx="50%" cy="50%" r="70%" fx="50%" fy="50%">
                                    <stop offset="0%" stop-color="rgb({},{},{})" />
                                    <stop offset="100%" stop-color="rgb({},{},{})" />
                                </radialGradient>"##,
                                grad_id,
                                gradient.start_color.0, gradient.start_color.1, gradient.start_color.2,
                                gradient.end_color.0, gradient.end_color.1, gradient.end_color.2
                            ));
                        } else {
                            // Linear gradient approximation
                            defs.push_str(&format!(
                                r##"<linearGradient id="{}" x1="0%" y1="0%" x2="100%" y2="100%">
                                    <stop offset="0%" stop-color="rgb({},{},{})" />
                                    <stop offset="100%" stop-color="rgb({},{},{})" />
                                </linearGradient>"##,
                                grad_id,
                                gradient.start_color.0, gradient.start_color.1, gradient.start_color.2,
                                gradient.end_color.0, gradient.end_color.1, gradient.end_color.2
                            ));
                        }
                        format!("url(#{})", grad_id)
                    } else {
                        format!("rgba({},{},{},{})", gradient.start_color.0, gradient.start_color.1, gradient.start_color.2, opacity)
                    }
                }
            };
            
            let projected: Vec<(f32, f32)> = indices.iter()
                .map(|&idx| {
                    let v = rotated[idx];
                    let v_translated = Point3D::new(
                        v.x + offset.x,
                        v.y + offset.y,
                        v.z + offset.z
                    );
                    let (px, py) = v_translated.project(focal_length);
                    (px + center_x, py + center_y)
                })
                .collect();
                
            let points_str: String = projected.iter()
                .map(|(x, y)| format!("{:.1},{:.1}", x, y))
                .collect::<Vec<_>>()
                .join(" ");
                
            let svg_face = format!(
                r##"<polygon points="{}" fill="{}" />"##,
                points_str, fill_attr
            );
            
            face_data.push((face_center_world.z, svg_face));
        }
    }
    
    // Sort by depth (painter's algorithm: far to near)
    face_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    let faces_svg = face_data.iter()
        .map(|(_, svg)| svg.clone())
        .collect::<Vec<_>>()
        .join("\n");
        
    if !defs.is_empty() {
        format!("<defs>{}</defs>{}", defs, faces_svg)
    } else {
        faces_svg
    }
}

fn compute_normal(p1: Point3D, p2: Point3D, p3: Point3D) -> Point3D {
    let u = Point3D::new(p2.x - p1.x, p2.y - p1.y, p2.z - p1.z);
    let v = Point3D::new(p3.x - p1.x, p3.y - p1.y, p3.z - p1.z);
    
    // Cross product (u x v)
    // For CCW winding, this gives Outward/Towards Camera normal
    let nx = u.y * v.z - u.z * v.y;
    let ny = u.z * v.x - u.x * v.z;
    let nz = u.x * v.y - u.y * v.x;
    
    Point3D::new(nx, ny, nz).normalize()
}

/// Get shape data with optional component animation angle
/// For F1Car, this rotates the wheels around their axle
fn get_shape_data_animated(shape_type: ShapeType, size: f32, component_angle: f32) -> (Vec<Point3D>, Vec<Vec<usize>>, Vec<(u8, u8, u8)>) {
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
                vec![0, 2, 1],
                vec![0, 1, 3],
                vec![0, 3, 2],
                vec![1, 2, 3],
            ];
            let colors = vec![
                (255, 100, 100), (100, 255, 100), (100, 100, 255), (255, 255, 100),
            ];
            (vertices, faces, colors)
        },
        ShapeType::CornellBox => {
            let vertices = vec![
                Point3D::new(-half, -half, -half), // 0: Top-Left-Near
                Point3D::new( half, -half, -half), // 1: Top-Right-Near
                Point3D::new( half,  half, -half), // 2: Bottom-Right-Near
                Point3D::new(-half,  half, -half), // 3: Bottom-Left-Near
                Point3D::new(-half, -half,  half), // 4: Top-Left-Far
                Point3D::new( half, -half,  half), // 5: Top-Right-Far
                Point3D::new( half,  half,  half), // 6: Bottom-Right-Far
                Point3D::new(-half,  half,  half), // 7: Bottom-Left-Far
            ];
            
            // We want to render the INSIDE of the box.
            // 1. Floor (Bottom, Y=+half). Normal Up (-Y).
            // 2. Back Wall (Far, Z=+half). Normal Near (-Z).
            // 3. Left Wall (Left, X=-half). Normal Right (+X).
            
            let faces = vec![
                vec![3, 2, 6, 7], // Floor (Bottom)
                vec![1, 5, 6, 2], // Right Wall (Red) - Inward Normal (-X)
                vec![4, 7, 6, 5], // Back Wall (Far)
            ];
            
            let colors = vec![
                (10, 10, 10),    // Black Floor
                (200, 50, 50),   // Red Wall
                (50, 50, 200),   // Blue Wall
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
                vec![4, 2, 0], vec![4, 1, 2], vec![4, 3, 1], vec![4, 0, 3],
                vec![5, 0, 2], vec![5, 2, 1], vec![5, 1, 3], vec![5, 3, 0],
            ];
            let colors = vec![
                (255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0),
                (255, 0, 255), (0, 255, 255), (255, 128, 0), (128, 0, 255),
            ];
            (vertices, faces, colors)
        },
        ShapeType::Pyramid => {
            let s = size * 0.7;
            let h = size * 0.7;
            let vertices = vec![
                Point3D::new( 0.0,  0.0,  h), // Apex (0)
                Point3D::new( s,  s, -h),     // Base 1 (1)
                Point3D::new( s, -s, -h),     // Base 2 (2)
                Point3D::new(-s, -s, -h),     // Base 3 (3)
                Point3D::new(-s,  s, -h),     // Base 4 (4)
            ];
            let faces = vec![
                vec![0, 2, 1], // Side 1
                vec![0, 3, 2], // Side 2
                vec![0, 4, 3], // Side 3
                vec![0, 1, 4], // Side 4
                vec![1, 2, 3, 4], // Base (Square)
            ];
            let colors = vec![
                (255, 50, 50),   // Red
                (50, 255, 50),   // Green
                (50, 50, 255),   // Blue
                (255, 255, 50),  // Yellow
                (100, 100, 100), // Gray Base
            ];
            (vertices, faces, colors)
        },
        ShapeType::Torus => {
            // Torus parameters
            let major_radius = size * 0.5;
            let minor_radius = size * 0.2;
            let major_segments = 24;
            let minor_segments = 12;
            
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            let mut colors = Vec::new();
            
            // Generate vertices
            for i in 0..=major_segments {
                let theta = 2.0 * PI * i as f32 / major_segments as f32;
                for j in 0..=minor_segments {
                    let phi = 2.0 * PI * j as f32 / minor_segments as f32;
                    let x = (major_radius + minor_radius * phi.cos()) * theta.cos();
                    let y = minor_radius * phi.sin();
                    let z = (major_radius + minor_radius * phi.cos()) * theta.sin();
                    vertices.push(Point3D::new(x, y, z));
                }
            }
            
            // Generate faces (quads split into triangles)
            for i in 0..major_segments {
                for j in 0..minor_segments {
                    let ring_size = minor_segments + 1;
                    let i0 = i * ring_size + j;
                    let i1 = i * ring_size + j + 1;
                    let i2 = (i + 1) * ring_size + j;
                    let i3 = (i + 1) * ring_size + j + 1;
                    
                    // Triangle 1
                    faces.push(vec![i0, i2, i1]);
                    // Triangle 2
                    faces.push(vec![i1, i2, i3]);
                    
                    // Copper/bronze color with slight variation
                    let base = (184, 115, 51);
                    colors.push(base);
                    colors.push(base);
                }
            }
            
            (vertices, faces, colors)
        },
        ShapeType::Sphere => {
            let radius = size * 0.5;
            let segments = 24;
            let rings = 12;
            
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            let mut colors = Vec::new();
            
            // Generate vertices
            for i in 0..=rings {
                let v = i as f32 / rings as f32;
                let phi = v * PI;
                
                for j in 0..=segments {
                    let u = j as f32 / segments as f32;
                    let theta = u * 2.0 * PI;
                    
                    let x = radius * phi.sin() * theta.cos();
                    let y = radius * phi.cos();
                    let z = radius * phi.sin() * theta.sin();
                    
                    vertices.push(Point3D::new(x, y, z));
                }
            }
            
            // Generate faces
            for i in 0..rings {
                for j in 0..segments {
                    let next_i = i + 1;
                    let next_j = j + 1;
                    
                    let v0 = i * (segments + 1) + j;
                    let v1 = i * (segments + 1) + next_j;
                    let v2 = next_i * (segments + 1) + next_j;
                    let v3 = next_i * (segments + 1) + j;
                    
                    // Two triangles per quad
                    faces.push(vec![v0, v1, v2]);
                    faces.push(vec![v0, v2, v3]);
                    
                    // Silver/Grey color
                    colors.push((200, 200, 200));
                    colors.push((200, 200, 200));
                }
            }
            (vertices, faces, colors)
        },
        ShapeType::F1Car => {
            // ============================================================
            // F1 CAR - ULTRA-DETAILED 100+ PART MODEL
            // ============================================================
            let s = size * 0.5;
            
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            let mut colors = Vec::new();
            
            // === COLOR PALETTE (14 distinct materials) ===
            let body_color = (220, 30, 30);       // Ferrari red
            let body_accent = (180, 20, 20);      // Darker red
            let cockpit_color = (40, 40, 45);     // Carbon fiber
            let wheel_color = (25, 25, 25);       // Tire black
            let wheel_rim = (180, 180, 185);      // Silver alloy
            let wing_color = (200, 20, 20);       // Wing red
            let floor_color = (50, 50, 55);       // Dark gray
            let suspension_color = (60, 60, 65);  // Wishbone gray
            let halo_color = (30, 30, 35);        // Titanium black
            let helmet_color = (240, 200, 50);    // Driver helmet gold
            let visor_color = (20, 20, 25);       // Visor tint
            let brake_color = (200, 100, 30);     // Brake glow orange
            let exhaust_color = (80, 80, 85);     // Steel
            let light_color = (255, 50, 50);      // LED red
            
            // === DIMENSIONS ===
            let body_l = s * 2.2;   // Total length
            let body_w = s * 0.45;  // Body width
            let body_h = s * 0.28;  // Body height
            let ground_y = -body_h * 0.5;
            
            // ============================================================
            // PART 1-8: MONOCOQUE & CHASSIS
            // ============================================================
            let b_start = vertices.len();
            
            // Nose cone (tapered)
            vertices.push(Point3D::new(-body_l * 0.55, ground_y + body_h * 0.15, 0.0)); // Nose tip (0)
            vertices.push(Point3D::new(-body_l * 0.45, ground_y, -body_w * 0.15)); // (1)
            vertices.push(Point3D::new(-body_l * 0.45, ground_y,  body_w * 0.15)); // (2)
            vertices.push(Point3D::new(-body_l * 0.45, ground_y + body_h * 0.25, -body_w * 0.12)); // (3)
            vertices.push(Point3D::new(-body_l * 0.45, ground_y + body_h * 0.25,  body_w * 0.12)); // (4)
            
            // Front bulkhead
            vertices.push(Point3D::new(-body_l * 0.35, ground_y, -body_w * 0.25)); // (5)
            vertices.push(Point3D::new(-body_l * 0.35, ground_y,  body_w * 0.25)); // (6)
            vertices.push(Point3D::new(-body_l * 0.35, ground_y + body_h * 0.35, -body_w * 0.22)); // (7)
            vertices.push(Point3D::new(-body_l * 0.35, ground_y + body_h * 0.35,  body_w * 0.22)); // (8)
            
            // Cockpit opening
            vertices.push(Point3D::new(-body_l * 0.15, ground_y, -body_w * 0.35)); // (9)
            vertices.push(Point3D::new(-body_l * 0.15, ground_y,  body_w * 0.35)); // (10)
            vertices.push(Point3D::new(-body_l * 0.15, ground_y + body_h * 0.45, -body_w * 0.18)); // (11)
            vertices.push(Point3D::new(-body_l * 0.15, ground_y + body_h * 0.45,  body_w * 0.18)); // (12)
            
            // Sidepod area
            vertices.push(Point3D::new(0.0, ground_y, -body_w * 0.5)); // (13)
            vertices.push(Point3D::new(0.0, ground_y,  body_w * 0.5)); // (14)
            vertices.push(Point3D::new(0.0, ground_y + body_h * 0.5, -body_w * 0.35)); // (15)
            vertices.push(Point3D::new(0.0, ground_y + body_h * 0.5,  body_w * 0.35)); // (16)
            
            // Engine cover area
            vertices.push(Point3D::new(body_l * 0.2, ground_y, -body_w * 0.4)); // (17)
            vertices.push(Point3D::new(body_l * 0.2, ground_y,  body_w * 0.4)); // (18)
            vertices.push(Point3D::new(body_l * 0.2, ground_y + body_h * 0.55, -body_w * 0.25)); // (19)
            vertices.push(Point3D::new(body_l * 0.2, ground_y + body_h * 0.55,  body_w * 0.25)); // (20)
            
            // Rear crash structure
            vertices.push(Point3D::new(body_l * 0.4, ground_y + body_h * 0.1, -body_w * 0.2)); // (21)
            vertices.push(Point3D::new(body_l * 0.4, ground_y + body_h * 0.1,  body_w * 0.2)); // (22)
            vertices.push(Point3D::new(body_l * 0.4, ground_y + body_h * 0.35, -body_w * 0.15)); // (23)
            vertices.push(Point3D::new(body_l * 0.4, ground_y + body_h * 0.35,  body_w * 0.15)); // (24)
            
            // Nose cone faces
            faces.push(vec![b_start, b_start+1, b_start+3]); // Nose left
            faces.push(vec![b_start, b_start+4, b_start+2]); // Nose right
            faces.push(vec![b_start, b_start+3, b_start+4]); // Nose top
            faces.push(vec![b_start, b_start+2, b_start+1]); // Nose bottom
            for _ in 0..4 { colors.push(body_color); }
            
            // Front section faces
            faces.push(vec![b_start+1, b_start+5, b_start+7, b_start+3]); // Left
            faces.push(vec![b_start+2, b_start+4, b_start+8, b_start+6]); // Right
            faces.push(vec![b_start+3, b_start+7, b_start+8, b_start+4]); // Top
            faces.push(vec![b_start+1, b_start+2, b_start+6, b_start+5]); // Bottom
            for _ in 0..4 { colors.push(body_color); }
            
            // Mid section faces
            faces.push(vec![b_start+5, b_start+9, b_start+11, b_start+7]); // Left
            faces.push(vec![b_start+6, b_start+8, b_start+12, b_start+10]); // Right
            faces.push(vec![b_start+7, b_start+11, b_start+12, b_start+8]); // Top
            faces.push(vec![b_start+5, b_start+6, b_start+10, b_start+9]); // Bottom
            for _ in 0..4 { colors.push(body_color); }
            
            // Sidepod section faces
            faces.push(vec![b_start+9, b_start+13, b_start+15, b_start+11]); // Left
            faces.push(vec![b_start+10, b_start+12, b_start+16, b_start+14]); // Right
            faces.push(vec![b_start+11, b_start+15, b_start+16, b_start+12]); // Top
            faces.push(vec![b_start+9, b_start+10, b_start+14, b_start+13]); // Bottom
            for _ in 0..4 { colors.push(body_color); }
            
            // Engine cover section
            faces.push(vec![b_start+13, b_start+17, b_start+19, b_start+15]); // Left
            faces.push(vec![b_start+14, b_start+16, b_start+20, b_start+18]); // Right
            faces.push(vec![b_start+15, b_start+19, b_start+20, b_start+16]); // Top
            faces.push(vec![b_start+13, b_start+14, b_start+18, b_start+17]); // Bottom
            for _ in 0..4 { colors.push(body_accent); }
            
            // Rear section
            faces.push(vec![b_start+17, b_start+21, b_start+23, b_start+19]); // Left
            faces.push(vec![b_start+18, b_start+20, b_start+24, b_start+22]); // Right
            faces.push(vec![b_start+19, b_start+23, b_start+24, b_start+20]); // Top
            faces.push(vec![b_start+21, b_start+22, b_start+24, b_start+23]); // Rear
            for _ in 0..4 { colors.push(body_accent); }
            
            // ============================================================
            // PART 9-16: FRONT WING (Multi-element)
            // ============================================================
            let fw_x = -body_l * 0.55;
            let fw_y = ground_y - s * 0.02;
            let fw_w = s * 0.9;
            
            // Main plane
            let fwm = vertices.len();
            vertices.push(Point3D::new(fw_x - s*0.12, fw_y, -fw_w * 0.5));
            vertices.push(Point3D::new(fw_x - s*0.12, fw_y,  fw_w * 0.5));
            vertices.push(Point3D::new(fw_x - s*0.12, fw_y + s*0.03, -fw_w * 0.5));
            vertices.push(Point3D::new(fw_x - s*0.12, fw_y + s*0.03,  fw_w * 0.5));
            vertices.push(Point3D::new(fw_x + s*0.08, fw_y + s*0.02, -fw_w * 0.48));
            vertices.push(Point3D::new(fw_x + s*0.08, fw_y + s*0.02,  fw_w * 0.48));
            vertices.push(Point3D::new(fw_x + s*0.08, fw_y + s*0.05, -fw_w * 0.48));
            vertices.push(Point3D::new(fw_x + s*0.08, fw_y + s*0.05,  fw_w * 0.48));
            
            faces.push(vec![fwm+2, fwm+3, fwm+7, fwm+6]); // Top
            faces.push(vec![fwm, fwm+4, fwm+5, fwm+1]); // Bottom
            for _ in 0..2 { colors.push(wing_color); }
            
            // Flap element 1
            let fwf1 = vertices.len();
            for i in 0..8 {
                let v = vertices[fwm + i];
                vertices.push(Point3D::new(v.x + s*0.02, v.y + s*0.025, v.z * 0.95));
            }
            faces.push(vec![fwf1+2, fwf1+3, fwf1+7, fwf1+6]);
            faces.push(vec![fwf1, fwf1+4, fwf1+5, fwf1+1]);
            for _ in 0..2 { colors.push(wing_color); }
            
            // Flap element 2
            let fwf2 = vertices.len();
            for i in 0..8 {
                let v = vertices[fwf1 + i];
                vertices.push(Point3D::new(v.x + s*0.015, v.y + s*0.02, v.z * 0.92));
            }
            faces.push(vec![fwf2+2, fwf2+3, fwf2+7, fwf2+6]);
            faces.push(vec![fwf2, fwf2+4, fwf2+5, fwf2+1]);
            for _ in 0..2 { colors.push(wing_color); }
            
            // Endplates (left and right)
            for side in [-1.0, 1.0] {
                let ep = vertices.len();
                let ep_z = side * fw_w * 0.5;
                vertices.push(Point3D::new(fw_x - s*0.15, fw_y - s*0.01, ep_z));
                vertices.push(Point3D::new(fw_x + s*0.1, fw_y, ep_z));
                vertices.push(Point3D::new(fw_x - s*0.15, fw_y + s*0.12, ep_z));
                vertices.push(Point3D::new(fw_x + s*0.1, fw_y + s*0.08, ep_z));
                
                faces.push(vec![ep, ep+1, ep+3, ep+2]);
                colors.push(cockpit_color);
            }
            
            // ============================================================
            // PART 17-24: REAR WING (DRS + Beam wing)
            // ============================================================
            let rw_x = body_l * 0.42;
            let rw_y = ground_y + body_h * 0.35;
            let rw_w = s * 0.45;
            let rw_h = s * 0.28;
            
            // Main plane
            let rwm = vertices.len();
            vertices.push(Point3D::new(rw_x, rw_y + rw_h * 0.6, -rw_w));
            vertices.push(Point3D::new(rw_x, rw_y + rw_h * 0.6,  rw_w));
            vertices.push(Point3D::new(rw_x, rw_y + rw_h * 0.7, -rw_w));
            vertices.push(Point3D::new(rw_x, rw_y + rw_h * 0.7,  rw_w));
            vertices.push(Point3D::new(rw_x + s*0.06, rw_y + rw_h * 0.65, -rw_w));
            vertices.push(Point3D::new(rw_x + s*0.06, rw_y + rw_h * 0.65,  rw_w));
            vertices.push(Point3D::new(rw_x + s*0.06, rw_y + rw_h * 0.75, -rw_w));
            vertices.push(Point3D::new(rw_x + s*0.06, rw_y + rw_h * 0.75,  rw_w));
            
            faces.push(vec![rwm+2, rwm+3, rwm+7, rwm+6]); // Top
            faces.push(vec![rwm, rwm+4, rwm+5, rwm+1]); // Bottom
            for _ in 0..2 { colors.push(wing_color); }
            
            // DRS flap
            let drs = vertices.len();
            vertices.push(Point3D::new(rw_x - s*0.02, rw_y + rw_h * 0.75, -rw_w * 0.95));
            vertices.push(Point3D::new(rw_x - s*0.02, rw_y + rw_h * 0.75,  rw_w * 0.95));
            vertices.push(Point3D::new(rw_x - s*0.02, rw_y + rw_h * 0.82, -rw_w * 0.95));
            vertices.push(Point3D::new(rw_x - s*0.02, rw_y + rw_h * 0.82,  rw_w * 0.95));
            vertices.push(Point3D::new(rw_x + s*0.04, rw_y + rw_h * 0.78, -rw_w * 0.95));
            vertices.push(Point3D::new(rw_x + s*0.04, rw_y + rw_h * 0.78,  rw_w * 0.95));
            vertices.push(Point3D::new(rw_x + s*0.04, rw_y + rw_h * 0.85, -rw_w * 0.95));
            vertices.push(Point3D::new(rw_x + s*0.04, rw_y + rw_h * 0.85,  rw_w * 0.95));
            
            faces.push(vec![drs+2, drs+3, drs+7, drs+6]);
            faces.push(vec![drs, drs+4, drs+5, drs+1]);
            for _ in 0..2 { colors.push(wing_color); }
            
            // Rear wing endplates
            for side in [-1.0, 1.0] {
                let rep = vertices.len();
                let ep_z = side * rw_w;
                vertices.push(Point3D::new(rw_x - s*0.04, rw_y, ep_z));
                vertices.push(Point3D::new(rw_x + s*0.08, rw_y + rw_h * 0.1, ep_z));
                vertices.push(Point3D::new(rw_x - s*0.04, rw_y + rw_h, ep_z));
                vertices.push(Point3D::new(rw_x + s*0.08, rw_y + rw_h * 0.9, ep_z));
                
                faces.push(vec![rep, rep+1, rep+3, rep+2]);
                colors.push(cockpit_color);
            }
            
            // Beam wing
            let bw = vertices.len();
            vertices.push(Point3D::new(rw_x - s*0.02, rw_y + s*0.08, -rw_w * 0.8));
            vertices.push(Point3D::new(rw_x - s*0.02, rw_y + s*0.08,  rw_w * 0.8));
            vertices.push(Point3D::new(rw_x + s*0.04, rw_y + s*0.1, -rw_w * 0.8));
            vertices.push(Point3D::new(rw_x + s*0.04, rw_y + s*0.1,  rw_w * 0.8));
            faces.push(vec![bw, bw+1, bw+3, bw+2]);
            colors.push(wing_color);
            
            // ============================================================
            // PART 25-32: SIDEPODS & BARGEBOARDS
            // ============================================================
            // Sidepod inlets (left)
            let spl = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.1, ground_y + body_h * 0.15, -body_w * 0.5));
            vertices.push(Point3D::new(-body_l * 0.05, ground_y + body_h * 0.1, -body_w * 0.52));
            vertices.push(Point3D::new(-body_l * 0.1, ground_y + body_h * 0.45, -body_w * 0.48));
            vertices.push(Point3D::new(-body_l * 0.05, ground_y + body_h * 0.4, -body_w * 0.5));
            faces.push(vec![spl, spl+1, spl+3, spl+2]);
            colors.push(cockpit_color);
            
            // Sidepod inlet (right)
            let spr = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.1, ground_y + body_h * 0.15, body_w * 0.5));
            vertices.push(Point3D::new(-body_l * 0.05, ground_y + body_h * 0.1, body_w * 0.52));
            vertices.push(Point3D::new(-body_l * 0.1, ground_y + body_h * 0.45, body_w * 0.48));
            vertices.push(Point3D::new(-body_l * 0.05, ground_y + body_h * 0.4, body_w * 0.5));
            faces.push(vec![spr, spr+2, spr+3, spr+1]);
            colors.push(cockpit_color);
            
            // Bargeboards (left)
            let bbl = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.25, ground_y + s*0.02, -body_w * 0.35));
            vertices.push(Point3D::new(-body_l * 0.12, ground_y + s*0.02, -body_w * 0.48));
            vertices.push(Point3D::new(-body_l * 0.25, ground_y + body_h * 0.3, -body_w * 0.32));
            vertices.push(Point3D::new(-body_l * 0.12, ground_y + body_h * 0.25, -body_w * 0.45));
            faces.push(vec![bbl, bbl+1, bbl+3, bbl+2]);
            colors.push(cockpit_color);
            
            // Bargeboards (right)
            let bbr = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.25, ground_y + s*0.02, body_w * 0.35));
            vertices.push(Point3D::new(-body_l * 0.12, ground_y + s*0.02, body_w * 0.48));
            vertices.push(Point3D::new(-body_l * 0.25, ground_y + body_h * 0.3, body_w * 0.32));
            vertices.push(Point3D::new(-body_l * 0.12, ground_y + body_h * 0.25, body_w * 0.45));
            faces.push(vec![bbr, bbr+2, bbr+3, bbr+1]);
            colors.push(cockpit_color);
            
            // ============================================================
            // PART 33-40: HALO & COCKPIT DETAILS
            // ============================================================
            let halo = vertices.len();
            let halo_y = ground_y + body_h * 0.5;
            let halo_r = s * 0.12;
            
            // Halo titanium frame (simplified arc)
            for i in 0..12 {
                let angle = PI * i as f32 / 11.0;
                let hx = -body_l * 0.1 + halo_r * 1.5 * angle.cos();
                let hy = halo_y + halo_r * angle.sin();
                vertices.push(Point3D::new(hx, hy, -s * 0.02));
                vertices.push(Point3D::new(hx, hy,  s * 0.02));
            }
            
            for i in 0..11 {
                let i0 = halo + i * 2;
                faces.push(vec![i0, i0+2, i0+3, i0+1]);
                colors.push(halo_color);
            }
            
            // Headrest
            let hr = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.02, halo_y - s*0.05, -s * 0.06));
            vertices.push(Point3D::new(-body_l * 0.02, halo_y - s*0.05,  s * 0.06));
            vertices.push(Point3D::new(-body_l * 0.02, halo_y + s*0.08, -s * 0.05));
            vertices.push(Point3D::new(-body_l * 0.02, halo_y + s*0.08,  s * 0.05));
            vertices.push(Point3D::new(body_l * 0.05, halo_y - s*0.03, -s * 0.05));
            vertices.push(Point3D::new(body_l * 0.05, halo_y - s*0.03,  s * 0.05));
            vertices.push(Point3D::new(body_l * 0.05, halo_y + s*0.06, -s * 0.04));
            vertices.push(Point3D::new(body_l * 0.05, halo_y + s*0.06,  s * 0.04));
            
            faces.push(vec![hr, hr+1, hr+3, hr+2]); // Front
            faces.push(vec![hr+2, hr+3, hr+7, hr+6]); // Top
            faces.push(vec![hr+4, hr+6, hr+7, hr+5]); // Back
            for _ in 0..3 { colors.push(body_color); }
            
            // ============================================================
            // PART 41-48: DRIVER (Helmet & Shoulders)
            // ============================================================
            let helm = vertices.len();
            let helm_x = -body_l * 0.18;
            let helm_y = halo_y + s * 0.02;
            let helm_r = s * 0.08;
            
            // Simplified helmet (octagonal)
            for i in 0..8 {
                let angle = 2.0 * PI * i as f32 / 8.0;
                vertices.push(Point3D::new(
                    helm_x + helm_r * 0.3 * angle.cos(),
                    helm_y + helm_r * angle.sin(),
                    helm_r * 0.8 * angle.cos()
                ));
            }
            // Helmet top
            vertices.push(Point3D::new(helm_x, helm_y + helm_r * 1.1, 0.0));
            
            for i in 0..8 {
                faces.push(vec![helm + i, helm + (i+1)%8, helm + 8]);
                colors.push(helmet_color);
            }
            
            // Visor
            let vis = vertices.len();
            vertices.push(Point3D::new(helm_x - helm_r * 0.4, helm_y + helm_r * 0.3, -helm_r * 0.5));
            vertices.push(Point3D::new(helm_x - helm_r * 0.4, helm_y + helm_r * 0.3,  helm_r * 0.5));
            vertices.push(Point3D::new(helm_x - helm_r * 0.3, helm_y + helm_r * 0.8, -helm_r * 0.4));
            vertices.push(Point3D::new(helm_x - helm_r * 0.3, helm_y + helm_r * 0.8,  helm_r * 0.4));
            faces.push(vec![vis, vis+1, vis+3, vis+2]);
            colors.push(visor_color);
            
            // ============================================================
            // PART 49-64: SUSPENSION (Wishbones)
            // ============================================================
            let wheel_positions = [
                (-body_l * 0.38, ground_y, -body_w * 0.55, true),   // FL
                (-body_l * 0.38, ground_y,  body_w * 0.55, true),   // FR
                (body_l * 0.28, ground_y, -body_w * 0.58, false),  // RL
                (body_l * 0.28, ground_y,  body_w * 0.58, false),  // RR
            ];
            
            for (wx, wy, wz, is_front) in wheel_positions {
                // Upper wishbone
                let uw = vertices.len();
                let inner_z = if wz > 0.0 { body_w * 0.3 } else { -body_w * 0.3 };
                let chassis_x = if is_front { -body_l * 0.3 } else { body_l * 0.15 };
                
                vertices.push(Point3D::new(chassis_x - s*0.05, wy + s*0.12, inner_z));
                vertices.push(Point3D::new(chassis_x + s*0.05, wy + s*0.12, inner_z));
                vertices.push(Point3D::new(wx, wy + s*0.1, wz * 0.9));
                
                faces.push(vec![uw, uw+1, uw+2]);
                colors.push(suspension_color);
                
                // Lower wishbone
                let lw = vertices.len();
                vertices.push(Point3D::new(chassis_x - s*0.06, wy + s*0.02, inner_z));
                vertices.push(Point3D::new(chassis_x + s*0.06, wy + s*0.02, inner_z));
                vertices.push(Point3D::new(wx, wy + s*0.03, wz * 0.92));
                
                faces.push(vec![lw, lw+1, lw+2]);
                colors.push(suspension_color);
            }
            
            // ============================================================
            // PART 65-80: WHEELS & BRAKES (with component rotation!)
            // ============================================================
            let wheel_r = s * 0.16;
            let wheel_w = s * 0.08;
            let wheel_segments = 12;
            
            // Use component_angle for wheel rotation around axle (Z-axis in wheel space)
            let wheel_angle = component_angle;
            
            for (wx, wy, wz, _is_front) in wheel_positions {
                let w_start = vertices.len();
                
                // Tire outer - apply wheel rotation!
                for i in 0..wheel_segments {
                    // Add wheel_angle to make wheels spin
                    let angle = 2.0 * PI * i as f32 / wheel_segments as f32 + wheel_angle;
                    let dx = wheel_r * angle.cos();
                    let dy = wheel_r * angle.sin();
                    
                    vertices.push(Point3D::new(wx + dx, wy + dy, wz - wheel_w/2.0));
                    vertices.push(Point3D::new(wx + dx, wy + dy, wz + wheel_w/2.0));
                }
                
                // Tire sidewall faces
                for i in 0..wheel_segments {
                    let i0 = w_start + i * 2;
                    let i1 = w_start + i * 2 + 1;
                    let i2 = w_start + ((i + 1) % wheel_segments) * 2;
                    let i3 = w_start + ((i + 1) % wheel_segments) * 2 + 1;
                    
                    faces.push(vec![i0, i2, i3, i1]);
                    colors.push(wheel_color);
                }
                
                // Wheel rim (inner circle) - also rotates!
                let rim_start = vertices.len();
                let rim_r = wheel_r * 0.65;
                for i in 0..wheel_segments {
                    let angle = 2.0 * PI * i as f32 / wheel_segments as f32 + wheel_angle;
                    vertices.push(Point3D::new(wx + rim_r * angle.cos(), wy + rim_r * angle.sin(), wz - wheel_w * 0.4));
                    vertices.push(Point3D::new(wx + rim_r * angle.cos(), wy + rim_r * angle.sin(), wz + wheel_w * 0.4));
                }
                
                for i in 0..wheel_segments {
                    let i0 = rim_start + i * 2;
                    let i1 = rim_start + i * 2 + 1;
                    let i2 = rim_start + ((i + 1) % wheel_segments) * 2;
                    let i3 = rim_start + ((i + 1) % wheel_segments) * 2 + 1;
                    
                    faces.push(vec![i0, i1, i3, i2]);
                    colors.push(wheel_rim);
                }
                
                // Brake disc glow (stationary - attached to car)
                let bd = vertices.len();
                let bd_r = wheel_r * 0.5;
                vertices.push(Point3D::new(wx - bd_r, wy - bd_r * 0.5, wz - wheel_w * 0.3));
                vertices.push(Point3D::new(wx + bd_r, wy - bd_r * 0.5, wz - wheel_w * 0.3));
                vertices.push(Point3D::new(wx - bd_r, wy + bd_r * 0.5, wz - wheel_w * 0.3));
                vertices.push(Point3D::new(wx + bd_r, wy + bd_r * 0.5, wz - wheel_w * 0.3));
                faces.push(vec![bd, bd+1, bd+3, bd+2]);
                colors.push(brake_color);
            }
            
            // ============================================================
            // PART 81-88: DIFFUSER & FLOOR
            // ============================================================
            let diff = vertices.len();
            let diff_x = body_l * 0.25;
            
            // Diffuser channels
            for i in 0..5 {
                let z_pos = -body_w * 0.4 + i as f32 * body_w * 0.2;
                let d_start = vertices.len();
                vertices.push(Point3D::new(diff_x, ground_y - s*0.01, z_pos - s*0.03));
                vertices.push(Point3D::new(diff_x, ground_y - s*0.01, z_pos + s*0.03));
                vertices.push(Point3D::new(body_l * 0.42, ground_y + s*0.08, z_pos - s*0.02));
                vertices.push(Point3D::new(body_l * 0.42, ground_y + s*0.08, z_pos + s*0.02));
                faces.push(vec![d_start, d_start+1, d_start+3, d_start+2]);
                colors.push(floor_color);
            }
            
            // Stepped floor
            let sf = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.5, ground_y - s*0.01, -body_w * 0.45));
            vertices.push(Point3D::new(-body_l * 0.5, ground_y - s*0.01,  body_w * 0.45));
            vertices.push(Point3D::new(body_l * 0.35, ground_y - s*0.01, -body_w * 0.5));
            vertices.push(Point3D::new(body_l * 0.35, ground_y - s*0.01,  body_w * 0.5));
            faces.push(vec![sf, sf+2, sf+3, sf+1]);
            colors.push(floor_color);
            
            // ============================================================
            // PART 89-96: ENGINE COVER & AIR INTAKE
            // ============================================================
            // Air intake (above driver)
            let ai = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.05, halo_y + s*0.1, -s * 0.08));
            vertices.push(Point3D::new(-body_l * 0.05, halo_y + s*0.1,  s * 0.08));
            vertices.push(Point3D::new(-body_l * 0.05, halo_y + s*0.2, -s * 0.06));
            vertices.push(Point3D::new(-body_l * 0.05, halo_y + s*0.2,  s * 0.06));
            vertices.push(Point3D::new(body_l * 0.1, halo_y + s*0.12, -s * 0.07));
            vertices.push(Point3D::new(body_l * 0.1, halo_y + s*0.12,  s * 0.07));
            vertices.push(Point3D::new(body_l * 0.1, halo_y + s*0.18, -s * 0.05));
            vertices.push(Point3D::new(body_l * 0.1, halo_y + s*0.18,  s * 0.05));
            
            faces.push(vec![ai, ai+1, ai+3, ai+2]); // Front
            faces.push(vec![ai+2, ai+3, ai+7, ai+6]); // Top
            faces.push(vec![ai, ai+2, ai+6, ai+4]); // Left
            faces.push(vec![ai+1, ai+5, ai+7, ai+3]); // Right
            for _ in 0..4 { colors.push(cockpit_color); }
            
            // Shark fin
            let fin = vertices.len();
            vertices.push(Point3D::new(body_l * 0.1, ground_y + body_h * 0.55, 0.0));
            vertices.push(Point3D::new(body_l * 0.38, ground_y + body_h * 0.4, 0.0));
            vertices.push(Point3D::new(body_l * 0.38, ground_y + body_h * 0.4, s * 0.01));
            vertices.push(Point3D::new(body_l * 0.1, ground_y + body_h * 0.55, s * 0.01));
            faces.push(vec![fin, fin+1, fin+2, fin+3]);
            colors.push(body_accent);
            
            // ============================================================
            // PART 97-100: LIGHTS & ELECTRONICS
            // ============================================================
            // Rear rain light
            let rl = vertices.len();
            vertices.push(Point3D::new(body_l * 0.42, ground_y + body_h * 0.25, -s * 0.04));
            vertices.push(Point3D::new(body_l * 0.42, ground_y + body_h * 0.25,  s * 0.04));
            vertices.push(Point3D::new(body_l * 0.42, ground_y + body_h * 0.32, -s * 0.04));
            vertices.push(Point3D::new(body_l * 0.42, ground_y + body_h * 0.32,  s * 0.04));
            faces.push(vec![rl, rl+1, rl+3, rl+2]);
            colors.push(light_color);
            
            // T-cam (on roll hoop)
            let tcam = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.02, halo_y + s*0.22, -s * 0.015));
            vertices.push(Point3D::new(-body_l * 0.02, halo_y + s*0.22,  s * 0.015));
            vertices.push(Point3D::new(-body_l * 0.02, halo_y + s*0.26, -s * 0.015));
            vertices.push(Point3D::new(-body_l * 0.02, halo_y + s*0.26,  s * 0.015));
            vertices.push(Point3D::new(body_l * 0.02, halo_y + s*0.22, -s * 0.015));
            vertices.push(Point3D::new(body_l * 0.02, halo_y + s*0.22,  s * 0.015));
            vertices.push(Point3D::new(body_l * 0.02, halo_y + s*0.26, -s * 0.015));
            vertices.push(Point3D::new(body_l * 0.02, halo_y + s*0.26,  s * 0.015));
            
            faces.push(vec![tcam, tcam+1, tcam+3, tcam+2]);
            faces.push(vec![tcam+2, tcam+3, tcam+7, tcam+6]);
            faces.push(vec![tcam+4, tcam+6, tcam+7, tcam+5]);
            for _ in 0..3 { colors.push(cockpit_color); }
            
            // Pitot tubes (nose) - thin triangles
            for side in [-1.0, 1.0] {
                let pt = vertices.len();
                vertices.push(Point3D::new(-body_l * 0.56, ground_y + body_h * 0.2, side * s * 0.03));
                vertices.push(Point3D::new(-body_l * 0.52, ground_y + body_h * 0.18, side * s * 0.03));
                vertices.push(Point3D::new(-body_l * 0.54, ground_y + body_h * 0.19, side * s * 0.035));
                faces.push(vec![pt, pt+1, pt+2]);
                colors.push(cockpit_color);
            }
            
            // Exhaust pipe
            let exh = vertices.len();
            let exh_segs = 6;
            for i in 0..exh_segs {
                let angle = 2.0 * PI * i as f32 / exh_segs as f32;
                let r = s * 0.03;
                vertices.push(Point3D::new(body_l * 0.38, ground_y + body_h * 0.2 + r * angle.sin(), r * angle.cos()));
                vertices.push(Point3D::new(body_l * 0.44, ground_y + body_h * 0.22 + r * angle.sin(), r * angle.cos()));
            }
            for i in 0..exh_segs {
                let i0 = exh + i * 2;
                let i1 = exh + i * 2 + 1;
                let i2 = exh + ((i + 1) % exh_segs) * 2;
                let i3 = exh + ((i + 1) % exh_segs) * 2 + 1;
                faces.push(vec![i0, i2, i3, i1]);
                colors.push(exhaust_color);
            }
            
            (vertices, faces, colors)
        },
        ShapeType::Jabulani => {
            // ============================================================
            // JABULANI - 2010 FIFA World Cup Ball
            // The infamous ball with unpredictable flight characteristics
            // 8 thermally bonded 3D panels with "Grip'n'Groove" texture
            // ============================================================
            
            let r = size * 0.5;  // Ball radius
            
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            let mut colors = Vec::new();
            
            // Jabulani color palette (South African inspired)
            let white = (255, 255, 255);           // Main white
            let gold = (255, 203, 5);              // Metallic gold
            let green = (0, 122, 51);              // South African green
            let red = (222, 56, 49);               // Red accent
            let black = (30, 30, 30);              // Black panels
            let blue = (0, 56, 168);               // Blue accent
            
            // Generate sphere vertices using UV sphere method
            let lat_segments = 24;
            let lon_segments = 32;
            
            // Generate vertices
            for lat in 0..=lat_segments {
                let theta = PI * lat as f32 / lat_segments as f32;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();
                
                for lon in 0..=lon_segments {
                    let phi = 2.0 * PI * lon as f32 / lon_segments as f32;
                    let x = r * sin_theta * phi.cos();
                    let y = r * cos_theta;
                    let z = r * sin_theta * phi.sin();
                    vertices.push(Point3D::new(x, y, z));
                }
            }
            
            // Generate faces with Jabulani panel pattern
            // The Jabulani has 8 panels in a unique curved arrangement
            for lat in 0..lat_segments {
                for lon in 0..lon_segments {
                    let ring_size = lon_segments + 1;
                    let i0 = lat * ring_size + lon;
                    let i1 = lat * ring_size + lon + 1;
                    let i2 = (lat + 1) * ring_size + lon;
                    let i3 = (lat + 1) * ring_size + lon + 1;
                    
                    // Determine panel color based on position
                    // Jabulani's distinctive 8-panel pattern
                    let lat_norm = lat as f32 / lat_segments as f32;
                    let lon_norm = lon as f32 / lon_segments as f32;
                    
                    // Create the distinctive panel pattern
                    let panel_id = ((lon_norm * 4.0) as usize + (lat_norm * 2.0) as usize) % 8;
                    
                    let color = match panel_id {
                        0 => white,
                        1 => gold,
                        2 => green,
                        3 => white,
                        4 => black,
                        5 => red,
                        6 => white,
                        7 => blue,
                        _ => white,
                    };
                    
                    // Add curved panel edge effect - darken at boundaries
                    let edge_dist = ((lon_norm * 4.0).fract() - 0.5).abs() * 2.0;
                    let lat_edge = ((lat_norm * 4.0).fract() - 0.5).abs() * 2.0;
                    let is_edge = edge_dist > 0.85 || lat_edge > 0.85;
                    
                    let final_color = if is_edge {
                        // Panel seam - darker groove
                        ((color.0 as f32 * 0.6) as u8,
                         (color.1 as f32 * 0.6) as u8,
                         (color.2 as f32 * 0.6) as u8)
                    } else {
                        color
                    };
                    
                    // South pole = single tri
                    if lat == 0 {
                        faces.push(vec![i0, i2, i3]);
                        colors.push(final_color);
                    }
                    // North pole = single tri
                    else if lat == lat_segments - 1 {
                        faces.push(vec![i0, i2, i1]);
                        colors.push(final_color);
                    }
                    // Regular quads split into triangles
                    else {
                        faces.push(vec![i0, i2, i1]);
                        faces.push(vec![i1, i2, i3]);
                        colors.push(final_color);
                        colors.push(final_color);
                    }
                }
            }
            
            // Add the "adidas" logo stripe effect (3 stripes on equator)
            // These overwrite some faces with black
            let stripe_lat_start = (lat_segments as f32 * 0.42) as usize;
            let stripe_lat_end = (lat_segments as f32 * 0.58) as usize;
            
            for stripe in 0..3 {
                let stripe_lon_start = (lon_segments as f32 * (0.1 + stripe as f32 * 0.08)) as usize;
                let stripe_lon_end = stripe_lon_start + 2;
                
                for lat in stripe_lat_start..stripe_lat_end {
                    for lon in stripe_lon_start..stripe_lon_end.min(lon_segments) {
                        let ring_size = lon_segments + 1;
                        let i0 = lat * ring_size + lon;
                        let i1 = lat * ring_size + lon + 1;
                        let i2 = (lat + 1) * ring_size + lon;
                        let i3 = (lat + 1) * ring_size + lon + 1;
                        
                        faces.push(vec![i0, i2, i1]);
                        faces.push(vec![i1, i2, i3]);
                        colors.push(black);
                        colors.push(black);
                    }
                }
            }
            
            (vertices, faces, colors)
        },
        ShapeType::MercedesW14 => {
            // ============================================================
            // MERCEDES-AMG W14 E PERFORMANCE (2023)
            // Photorealistic model with accurate livery and geometry
            // ============================================================
            
            let s = size * 0.5;
            
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            let mut colors = Vec::new();
            
            // === W14 EXACT COLOR PALETTE ===
            let carbon_fiber = (30, 30, 32);        // Exposed CF #1E1E20
            let matte_black = (20, 20, 22);         // Painted black #141416
            let gloss_black = (10, 10, 12);         // Accent black #0A0A0C
            let petronas_teal = (0, 162, 146);      // PETRONAS #00A292
            let ineos_red = (222, 35, 39);          // INEOS #DE2327
            let amg_silver = (192, 192, 198);       // AMG #C0C0C6
            let star_silver = (210, 210, 215);      // Mercedes star #D2D2D7
            let brake_glow = (255, 120, 40);        // Hot brakes #FF7828
            let tire_rubber = (35, 35, 38);         // Pirelli #232326
            let wheel_bbs = (215, 210, 200);        // BBS rim #D7D2C8
            let visor_tint = (40, 45, 50);          // Helmet visor #282D32
            let driver_suit = (15, 15, 18);         // Black suit #0F0F12
            let halo_titanium = (80, 82, 85);       // Titanium halo
            
            // === W14 DIMENSIONS (scaled) ===
            let body_l = s * 2.4;    // Length ~5m scaled
            let body_w = s * 0.48;   // Width ~2m scaled  
            let body_h = s * 0.24;   // Height ~0.95m scaled
            let ground_y = -body_h * 0.5;
            
            // ============================================================
            // W14 MONOCOQUE - Narrow "zero-pod" design
            // ============================================================
            let mono = vertices.len();
            
            // Nose cone (W14 rounder profile)
            vertices.push(Point3D::new(-body_l * 0.52, ground_y + body_h * 0.18, 0.0)); // Nose tip
            vertices.push(Point3D::new(-body_l * 0.46, ground_y + body_h * 0.05, -body_w * 0.12));
            vertices.push(Point3D::new(-body_l * 0.46, ground_y + body_h * 0.05,  body_w * 0.12));
            vertices.push(Point3D::new(-body_l * 0.46, ground_y + body_h * 0.28, -body_w * 0.10));
            vertices.push(Point3D::new(-body_l * 0.46, ground_y + body_h * 0.28,  body_w * 0.10));
            
            // Front bulkhead
            vertices.push(Point3D::new(-body_l * 0.36, ground_y + body_h * 0.02, -body_w * 0.22));
            vertices.push(Point3D::new(-body_l * 0.36, ground_y + body_h * 0.02,  body_w * 0.22));
            vertices.push(Point3D::new(-body_l * 0.36, ground_y + body_h * 0.38, -body_w * 0.18));
            vertices.push(Point3D::new(-body_l * 0.36, ground_y + body_h * 0.38,  body_w * 0.18));
            
            // Cockpit section (survival cell)
            vertices.push(Point3D::new(-body_l * 0.18, ground_y + body_h * 0.02, -body_w * 0.32));
            vertices.push(Point3D::new(-body_l * 0.18, ground_y + body_h * 0.02,  body_w * 0.32));
            vertices.push(Point3D::new(-body_l * 0.18, ground_y + body_h * 0.48, -body_w * 0.16));
            vertices.push(Point3D::new(-body_l * 0.18, ground_y + body_h * 0.48,  body_w * 0.16));
            
            // Zero-pod sidepod start (NARROW - W14 signature)
            vertices.push(Point3D::new(-body_l * 0.02, ground_y + body_h * 0.02, -body_w * 0.38));
            vertices.push(Point3D::new(-body_l * 0.02, ground_y + body_h * 0.02,  body_w * 0.38));
            vertices.push(Point3D::new(-body_l * 0.02, ground_y + body_h * 0.52, -body_w * 0.22));
            vertices.push(Point3D::new(-body_l * 0.02, ground_y + body_h * 0.52,  body_w * 0.22));
            
            // Sidepod maximum width (still narrow)
            vertices.push(Point3D::new(body_l * 0.08, ground_y + body_h * 0.02, -body_w * 0.42));
            vertices.push(Point3D::new(body_l * 0.08, ground_y + body_h * 0.02,  body_w * 0.42));
            vertices.push(Point3D::new(body_l * 0.08, ground_y + body_h * 0.54, -body_w * 0.24));
            vertices.push(Point3D::new(body_l * 0.08, ground_y + body_h * 0.54,  body_w * 0.24));
            
            // Engine cover (gulley design - carved in)
            vertices.push(Point3D::new(body_l * 0.22, ground_y + body_h * 0.08, -body_w * 0.32));
            vertices.push(Point3D::new(body_l * 0.22, ground_y + body_h * 0.08,  body_w * 0.32));
            vertices.push(Point3D::new(body_l * 0.22, ground_y + body_h * 0.58, -body_w * 0.18));
            vertices.push(Point3D::new(body_l * 0.22, ground_y + body_h * 0.58,  body_w * 0.18));
            
            // Rear crash structure
            vertices.push(Point3D::new(body_l * 0.38, ground_y + body_h * 0.12, -body_w * 0.18));
            vertices.push(Point3D::new(body_l * 0.38, ground_y + body_h * 0.12,  body_w * 0.18));
            vertices.push(Point3D::new(body_l * 0.38, ground_y + body_h * 0.38, -body_w * 0.12));
            vertices.push(Point3D::new(body_l * 0.38, ground_y + body_h * 0.38,  body_w * 0.12));
            
            // Nose cone faces (matte black with Mercedes-AMG branding area)
            faces.push(vec![mono, mono+1, mono+3]);
            faces.push(vec![mono, mono+4, mono+2]);
            faces.push(vec![mono, mono+3, mono+4]);
            faces.push(vec![mono, mono+2, mono+1]);
            for _ in 0..4 { colors.push(matte_black); }
            
            // Front section (carbon fiber visible)
            faces.push(vec![mono+1, mono+5, mono+7, mono+3]);
            faces.push(vec![mono+2, mono+4, mono+8, mono+6]);
            faces.push(vec![mono+3, mono+7, mono+8, mono+4]);
            faces.push(vec![mono+1, mono+2, mono+6, mono+5]);
            for _ in 0..4 { colors.push(carbon_fiber); }
            
            // Cockpit section (exposed carbon)
            faces.push(vec![mono+5, mono+9, mono+11, mono+7]);
            faces.push(vec![mono+6, mono+8, mono+12, mono+10]);
            faces.push(vec![mono+7, mono+11, mono+12, mono+8]);
            faces.push(vec![mono+5, mono+6, mono+10, mono+9]);
            for _ in 0..4 { colors.push(carbon_fiber); }
            
            // Zero-pod sidepod section (with PETRONAS teal stripe)
            faces.push(vec![mono+9, mono+13, mono+15, mono+11]);
            colors.push(carbon_fiber);
            faces.push(vec![mono+10, mono+12, mono+16, mono+14]);
            colors.push(carbon_fiber);
            faces.push(vec![mono+11, mono+15, mono+16, mono+12]);
            colors.push(carbon_fiber);
            faces.push(vec![mono+9, mono+10, mono+14, mono+13]);
            colors.push(petronas_teal); // Floor edge teal stripe!
            
            // Sidepod outer (carbon with teal accent)
            faces.push(vec![mono+13, mono+17, mono+19, mono+15]);
            colors.push(carbon_fiber);
            faces.push(vec![mono+14, mono+16, mono+20, mono+18]);
            colors.push(carbon_fiber);
            faces.push(vec![mono+15, mono+19, mono+20, mono+16]);
            colors.push(carbon_fiber);
            faces.push(vec![mono+13, mono+14, mono+18, mono+17]);
            colors.push(petronas_teal); // Underbody teal
            
            // Engine cover (gulley - matte black)
            faces.push(vec![mono+17, mono+21, mono+23, mono+19]);
            faces.push(vec![mono+18, mono+20, mono+24, mono+22]);
            faces.push(vec![mono+19, mono+23, mono+24, mono+20]);
            faces.push(vec![mono+17, mono+18, mono+22, mono+21]);
            for _ in 0..4 { colors.push(matte_black); }
            
            // Rear section
            faces.push(vec![mono+21, mono+25, mono+27, mono+23]);
            faces.push(vec![mono+22, mono+24, mono+28, mono+26]);
            faces.push(vec![mono+23, mono+27, mono+28, mono+24]);
            faces.push(vec![mono+25, mono+26, mono+28, mono+27]);
            for _ in 0..4 { colors.push(carbon_fiber); }
            
            // ============================================================
            // W14 FRONT WING (Multi-element with teal tips)
            // ============================================================
            let fw_x = -body_l * 0.52;
            let fw_y = ground_y + body_h * 0.02;
            let fw_w = s * 0.95;
            
            // Main plane
            let fwm = vertices.len();
            vertices.push(Point3D::new(fw_x - s*0.14, fw_y, -fw_w * 0.5));
            vertices.push(Point3D::new(fw_x - s*0.14, fw_y,  fw_w * 0.5));
            vertices.push(Point3D::new(fw_x - s*0.14, fw_y + s*0.025, -fw_w * 0.5));
            vertices.push(Point3D::new(fw_x - s*0.14, fw_y + s*0.025,  fw_w * 0.5));
            vertices.push(Point3D::new(fw_x + s*0.08, fw_y + s*0.015, -fw_w * 0.48));
            vertices.push(Point3D::new(fw_x + s*0.08, fw_y + s*0.015,  fw_w * 0.48));
            vertices.push(Point3D::new(fw_x + s*0.08, fw_y + s*0.04, -fw_w * 0.48));
            vertices.push(Point3D::new(fw_x + s*0.08, fw_y + s*0.04,  fw_w * 0.48));
            
            faces.push(vec![fwm+2, fwm+3, fwm+7, fwm+6]);
            faces.push(vec![fwm, fwm+4, fwm+5, fwm+1]);
            for _ in 0..2 { colors.push(carbon_fiber); }
            
            // Flap elements
            for flap in 0..3 {
                let ff = vertices.len();
                let offset = (flap + 1) as f32 * s * 0.018;
                for i in 0..8 {
                    let v = vertices[fwm + i];
                    vertices.push(Point3D::new(v.x + offset * 0.5, v.y + offset, v.z * (0.96 - flap as f32 * 0.02)));
                }
                faces.push(vec![ff+2, ff+3, ff+7, ff+6]);
                faces.push(vec![ff, ff+4, ff+5, ff+1]);
                for _ in 0..2 { colors.push(carbon_fiber); }
            }
            
            // Front wing endplates with AMG branding
            for side in [-1.0, 1.0] {
                let ep = vertices.len();
                let ep_z = side * fw_w * 0.5;
                vertices.push(Point3D::new(fw_x - s*0.16, fw_y - s*0.01, ep_z));
                vertices.push(Point3D::new(fw_x + s*0.1, fw_y + s*0.01, ep_z));
                vertices.push(Point3D::new(fw_x - s*0.16, fw_y + s*0.1, ep_z));
                vertices.push(Point3D::new(fw_x + s*0.1, fw_y + s*0.08, ep_z));
                
                faces.push(vec![ep, ep+1, ep+3, ep+2]);
                colors.push(gloss_black); // AMG logo area
            }
            
            // Teal wing tips
            for side in [-1.0, 1.0] {
                let tip = vertices.len();
                let tip_z = side * fw_w * 0.52;
                vertices.push(Point3D::new(fw_x - s*0.08, fw_y + s*0.02, tip_z));
                vertices.push(Point3D::new(fw_x + s*0.06, fw_y + s*0.03, tip_z));
                vertices.push(Point3D::new(fw_x - s*0.08, fw_y + s*0.06, tip_z * 0.98));
                vertices.push(Point3D::new(fw_x + s*0.06, fw_y + s*0.05, tip_z * 0.98));
                faces.push(vec![tip, tip+1, tip+3, tip+2]);
                colors.push(petronas_teal); // PETRONAS branded tips
            }
            
            // ============================================================
            // W14 REAR WING (PETRONAS branding)
            // ============================================================
            let rw_x = body_l * 0.40;
            let rw_y = ground_y + body_h * 0.38;
            let rw_w = s * 0.42;
            let rw_h = s * 0.26;
            
            let rwm = vertices.len();
            vertices.push(Point3D::new(rw_x, rw_y + rw_h * 0.55, -rw_w));
            vertices.push(Point3D::new(rw_x, rw_y + rw_h * 0.55,  rw_w));
            vertices.push(Point3D::new(rw_x, rw_y + rw_h * 0.68, -rw_w));
            vertices.push(Point3D::new(rw_x, rw_y + rw_h * 0.68,  rw_w));
            vertices.push(Point3D::new(rw_x + s*0.055, rw_y + rw_h * 0.62, -rw_w));
            vertices.push(Point3D::new(rw_x + s*0.055, rw_y + rw_h * 0.62,  rw_w));
            vertices.push(Point3D::new(rw_x + s*0.055, rw_y + rw_h * 0.75, -rw_w));
            vertices.push(Point3D::new(rw_x + s*0.055, rw_y + rw_h * 0.75,  rw_w));
            
            faces.push(vec![rwm+2, rwm+3, rwm+7, rwm+6]); // Top - PETRONAS text
            colors.push(carbon_fiber);
            faces.push(vec![rwm, rwm+4, rwm+5, rwm+1]);
            colors.push(carbon_fiber);
            
            // DRS flap
            let drs = vertices.len();
            vertices.push(Point3D::new(rw_x - s*0.015, rw_y + rw_h * 0.72, -rw_w * 0.92));
            vertices.push(Point3D::new(rw_x - s*0.015, rw_y + rw_h * 0.72,  rw_w * 0.92));
            vertices.push(Point3D::new(rw_x - s*0.015, rw_y + rw_h * 0.82, -rw_w * 0.92));
            vertices.push(Point3D::new(rw_x - s*0.015, rw_y + rw_h * 0.82,  rw_w * 0.92));
            vertices.push(Point3D::new(rw_x + s*0.035, rw_y + rw_h * 0.76, -rw_w * 0.92));
            vertices.push(Point3D::new(rw_x + s*0.035, rw_y + rw_h * 0.76,  rw_w * 0.92));
            vertices.push(Point3D::new(rw_x + s*0.035, rw_y + rw_h * 0.86, -rw_w * 0.92));
            vertices.push(Point3D::new(rw_x + s*0.035, rw_y + rw_h * 0.86,  rw_w * 0.92));
            
            faces.push(vec![drs+2, drs+3, drs+7, drs+6]);
            faces.push(vec![drs, drs+4, drs+5, drs+1]);
            for _ in 0..2 { colors.push(carbon_fiber); }
            
            // Rear wing endplates (INEOS + AMG)
            for side in [-1.0, 1.0] {
                let rep = vertices.len();
                let ep_z = side * rw_w;
                vertices.push(Point3D::new(rw_x - s*0.035, rw_y + s*0.02, ep_z));
                vertices.push(Point3D::new(rw_x + s*0.07, rw_y + rw_h * 0.12, ep_z));
                vertices.push(Point3D::new(rw_x - s*0.035, rw_y + rw_h * 0.95, ep_z));
                vertices.push(Point3D::new(rw_x + s*0.07, rw_y + rw_h * 0.88, ep_z));
                
                faces.push(vec![rep, rep+1, rep+3, rep+2]);
                colors.push(ineos_red); // INEOS branded endplate
            }
            
            // Beam wing
            let bw = vertices.len();
            vertices.push(Point3D::new(rw_x - s*0.02, rw_y + s*0.06, -rw_w * 0.75));
            vertices.push(Point3D::new(rw_x - s*0.02, rw_y + s*0.06,  rw_w * 0.75));
            vertices.push(Point3D::new(rw_x + s*0.035, rw_y + s*0.08, -rw_w * 0.75));
            vertices.push(Point3D::new(rw_x + s*0.035, rw_y + s*0.08,  rw_w * 0.75));
            faces.push(vec![bw, bw+1, bw+3, bw+2]);
            colors.push(carbon_fiber);
            
            // ============================================================
            // W14 HALO (Titanium with Monster Energy branding)
            // ============================================================
            let halo = vertices.len();
            let halo_y = ground_y + body_h * 0.52;
            let halo_r = s * 0.11;
            
            for i in 0..14 {
                let angle = PI * i as f32 / 13.0;
                let hx = -body_l * 0.12 + halo_r * 1.4 * angle.cos();
                let hy = halo_y + halo_r * angle.sin();
                vertices.push(Point3D::new(hx, hy, -s * 0.018));
                vertices.push(Point3D::new(hx, hy,  s * 0.018));
            }
            
            for i in 0..13 {
                let i0 = halo + i * 2;
                faces.push(vec![i0, i0+2, i0+3, i0+1]);
                colors.push(halo_titanium);
            }
            
            // ============================================================
            // W14 AIR INTAKE (Triangular with INEOS red accent)
            // ============================================================
            let ai = vertices.len();
            let ai_y = halo_y + s * 0.08;
            
            vertices.push(Point3D::new(-body_l * 0.06, ai_y, -s * 0.07));
            vertices.push(Point3D::new(-body_l * 0.06, ai_y,  s * 0.07));
            vertices.push(Point3D::new(-body_l * 0.06, ai_y + s*0.12, 0.0)); // Peak
            vertices.push(Point3D::new(body_l * 0.08, ai_y + s*0.04, -s * 0.055));
            vertices.push(Point3D::new(body_l * 0.08, ai_y + s*0.04,  s * 0.055));
            vertices.push(Point3D::new(body_l * 0.08, ai_y + s*0.10, 0.0));
            
            faces.push(vec![ai, ai+1, ai+2]); // Front triangle (INEOS)
            colors.push(ineos_red);
            faces.push(vec![ai, ai+2, ai+5, ai+3]); // Left
            colors.push(carbon_fiber);
            faces.push(vec![ai+1, ai+4, ai+5, ai+2]); // Right
            colors.push(carbon_fiber);
            faces.push(vec![ai+3, ai+5, ai+4]); // Rear
            colors.push(matte_black);
            
            // ============================================================
            // W14 DRIVER (Hamilton/Russell helmet + suit)
            // ============================================================
            let helm = vertices.len();
            let helm_x = -body_l * 0.2;
            let helm_y = halo_y + s * 0.01;
            let helm_r = s * 0.075;
            
            for i in 0..10 {
                let angle = 2.0 * PI * i as f32 / 10.0;
                vertices.push(Point3D::new(
                    helm_x + helm_r * 0.35 * angle.cos(),
                    helm_y + helm_r * angle.sin(),
                    helm_r * 0.75 * angle.cos()
                ));
            }
            vertices.push(Point3D::new(helm_x, helm_y + helm_r * 1.05, 0.0));
            
            for i in 0..10 {
                faces.push(vec![helm + i, helm + (i+1)%10, helm + 10]);
                colors.push(star_silver); // Silver Mercedes helmet
            }
            
            // Visor (dark tint)
            let vis = vertices.len();
            vertices.push(Point3D::new(helm_x - helm_r * 0.38, helm_y + helm_r * 0.25, -helm_r * 0.45));
            vertices.push(Point3D::new(helm_x - helm_r * 0.38, helm_y + helm_r * 0.25,  helm_r * 0.45));
            vertices.push(Point3D::new(helm_x - helm_r * 0.28, helm_y + helm_r * 0.75, -helm_r * 0.38));
            vertices.push(Point3D::new(helm_x - helm_r * 0.28, helm_y + helm_r * 0.75,  helm_r * 0.38));
            faces.push(vec![vis, vis+1, vis+3, vis+2]);
            colors.push(visor_tint);
            
            // ============================================================
            // W14 SUSPENSION (Raised pushrod front, pullrod rear)
            // ============================================================
            let wheel_positions = [
                (-body_l * 0.36, ground_y, -body_w * 0.58, true),
                (-body_l * 0.36, ground_y,  body_w * 0.58, true),
                (body_l * 0.30, ground_y, -body_w * 0.60, false),
                (body_l * 0.30, ground_y,  body_w * 0.60, false),
            ];
            
            for (wx, wy, wz, is_front) in wheel_positions {
                let inner_z = if wz > 0.0 { body_w * 0.28 } else { -body_w * 0.28 };
                let chassis_x = if is_front { -body_l * 0.28 } else { body_l * 0.18 };
                
                // Upper wishbone
                let uw = vertices.len();
                let upper_y = if is_front { wy + s*0.14 } else { wy + s*0.11 }; // Front raised higher
                vertices.push(Point3D::new(chassis_x - s*0.045, upper_y, inner_z));
                vertices.push(Point3D::new(chassis_x + s*0.045, upper_y, inner_z));
                vertices.push(Point3D::new(wx, wy + s*0.10, wz * 0.88));
                faces.push(vec![uw, uw+1, uw+2]);
                colors.push(carbon_fiber);
                
                // Lower wishbone
                let lw = vertices.len();
                vertices.push(Point3D::new(chassis_x - s*0.055, wy + s*0.025, inner_z));
                vertices.push(Point3D::new(chassis_x + s*0.055, wy + s*0.025, inner_z));
                vertices.push(Point3D::new(wx, wy + s*0.035, wz * 0.90));
                faces.push(vec![lw, lw+1, lw+2]);
                colors.push(carbon_fiber);
            }
            
            // ============================================================
            // W14 WHEELS (18" Pirelli on BBS rims)
            // ============================================================
            let wheel_r = s * 0.17;  // 18" larger profile
            let wheel_w = s * 0.085;
            let wheel_segments = 14;
            
            for (wx, wy, wz, _is_front) in wheel_positions {
                let w_start = vertices.len();
                
                // Tire tread (with spinning animation)
                for i in 0..wheel_segments {
                    let angle = 2.0 * PI * i as f32 / wheel_segments as f32 + component_angle;
                    let dx = wheel_r * angle.cos();
                    let dy = wheel_r * angle.sin();
                    vertices.push(Point3D::new(wx + dx, wy + dy, wz - wheel_w/2.0));
                    vertices.push(Point3D::new(wx + dx, wy + dy, wz + wheel_w/2.0));
                }
                
                for i in 0..wheel_segments {
                    let i0 = w_start + i * 2;
                    let i1 = w_start + i * 2 + 1;
                    let i2 = w_start + ((i + 1) % wheel_segments) * 2;
                    let i3 = w_start + ((i + 1) % wheel_segments) * 2 + 1;
                    faces.push(vec![i0, i2, i3, i1]);
                    colors.push(tire_rubber);
                }
                
                // BBS forged magnesium rim (spinning)
                let rim_start = vertices.len();
                let rim_r = wheel_r * 0.62;
                for i in 0..wheel_segments {
                    let angle = 2.0 * PI * i as f32 / wheel_segments as f32 + component_angle;
                    vertices.push(Point3D::new(wx + rim_r * angle.cos(), wy + rim_r * angle.sin(), wz - wheel_w * 0.38));
                    vertices.push(Point3D::new(wx + rim_r * angle.cos(), wy + rim_r * angle.sin(), wz + wheel_w * 0.38));
                }
                
                for i in 0..wheel_segments {
                    let i0 = rim_start + i * 2;
                    let i1 = rim_start + i * 2 + 1;
                    let i2 = rim_start + ((i + 1) % wheel_segments) * 2;
                    let i3 = rim_start + ((i + 1) % wheel_segments) * 2 + 1;
                    faces.push(vec![i0, i1, i3, i2]);
                    colors.push(wheel_bbs);
                }
                
                // Brake disc (glowing hot - stationary)
                let bd = vertices.len();
                let bd_r = wheel_r * 0.48;
                vertices.push(Point3D::new(wx - bd_r, wy - bd_r * 0.4, wz - wheel_w * 0.28));
                vertices.push(Point3D::new(wx + bd_r, wy - bd_r * 0.4, wz - wheel_w * 0.28));
                vertices.push(Point3D::new(wx - bd_r, wy + bd_r * 0.4, wz - wheel_w * 0.28));
                vertices.push(Point3D::new(wx + bd_r, wy + bd_r * 0.4, wz - wheel_w * 0.28));
                faces.push(vec![bd, bd+1, bd+3, bd+2]);
                colors.push(brake_glow);
            }
            
            // ============================================================
            // W14 FLOOR + DIFFUSER (with PETRONAS edge)
            // ============================================================
            // Main floor (teal edge stripe)
            let fl = vertices.len();
            vertices.push(Point3D::new(-body_l * 0.48, ground_y - s*0.008, -body_w * 0.48));
            vertices.push(Point3D::new(-body_l * 0.48, ground_y - s*0.008,  body_w * 0.48));
            vertices.push(Point3D::new(body_l * 0.32, ground_y - s*0.008, -body_w * 0.52));
            vertices.push(Point3D::new(body_l * 0.32, ground_y - s*0.008,  body_w * 0.52));
            faces.push(vec![fl, fl+2, fl+3, fl+1]);
            colors.push(matte_black);
            
            // PETRONAS edge stripes
            for side in [-1.0, 1.0] {
                let edge = vertices.len();
                let edge_z = side * body_w * 0.5;
                vertices.push(Point3D::new(-body_l * 0.35, ground_y - s*0.005, edge_z - side * s*0.02));
                vertices.push(Point3D::new(body_l * 0.28, ground_y - s*0.005, edge_z - side * s*0.025));
                vertices.push(Point3D::new(-body_l * 0.35, ground_y - s*0.005, edge_z));
                vertices.push(Point3D::new(body_l * 0.28, ground_y - s*0.005, edge_z));
                faces.push(vec![edge, edge+1, edge+3, edge+2]);
                colors.push(petronas_teal);
            }
            
            // Diffuser channels
            for i in 0..6 {
                let z_off = -body_w * 0.42 + i as f32 * body_w * 0.168;
                let df = vertices.len();
                vertices.push(Point3D::new(body_l * 0.24, ground_y - s*0.008, z_off - s*0.025));
                vertices.push(Point3D::new(body_l * 0.24, ground_y - s*0.008, z_off + s*0.025));
                vertices.push(Point3D::new(body_l * 0.40, ground_y + s*0.07, z_off - s*0.018));
                vertices.push(Point3D::new(body_l * 0.40, ground_y + s*0.07, z_off + s*0.018));
                faces.push(vec![df, df+1, df+3, df+2]);
                colors.push(carbon_fiber);
            }
            
            // ============================================================
            // W14 REAR LIGHT + T-CAM
            // ============================================================
            // Rear rain light (FIA mandated)
            let rl = vertices.len();
            vertices.push(Point3D::new(body_l * 0.40, ground_y + body_h * 0.28, -s * 0.035));
            vertices.push(Point3D::new(body_l * 0.40, ground_y + body_h * 0.28,  s * 0.035));
            vertices.push(Point3D::new(body_l * 0.40, ground_y + body_h * 0.36, -s * 0.035));
            vertices.push(Point3D::new(body_l * 0.40, ground_y + body_h * 0.36,  s * 0.035));
            faces.push(vec![rl, rl+1, rl+3, rl+2]);
            colors.push(ineos_red); // Red LED
            
            // T-cam (black - team-specific)
            let tcam = vertices.len();
            let tcam_y = halo_y + s * 0.20;
            vertices.push(Point3D::new(-body_l * 0.04, tcam_y, -s * 0.012));
            vertices.push(Point3D::new(-body_l * 0.04, tcam_y,  s * 0.012));
            vertices.push(Point3D::new(-body_l * 0.04, tcam_y + s*0.035, -s * 0.012));
            vertices.push(Point3D::new(-body_l * 0.04, tcam_y + s*0.035,  s * 0.012));
            vertices.push(Point3D::new(body_l * 0.01, tcam_y, -s * 0.012));
            vertices.push(Point3D::new(body_l * 0.01, tcam_y,  s * 0.012));
            vertices.push(Point3D::new(body_l * 0.01, tcam_y + s*0.035, -s * 0.012));
            vertices.push(Point3D::new(body_l * 0.01, tcam_y + s*0.035,  s * 0.012));
            
            faces.push(vec![tcam, tcam+1, tcam+3, tcam+2]);
            faces.push(vec![tcam+2, tcam+3, tcam+7, tcam+6]);
            faces.push(vec![tcam+4, tcam+6, tcam+7, tcam+5]);
            for _ in 0..3 { colors.push(gloss_black); }
            
            // Mercedes three-pointed star (on engine cover)
            let star = vertices.len();
            let star_x = body_l * 0.12;
            let star_y = ground_y + body_h * 0.58;
            let star_r = s * 0.045;
            
            for i in 0..8 {
                let angle = 2.0 * PI * i as f32 / 8.0;
                vertices.push(Point3D::new(star_x, star_y + star_r * angle.sin(), star_r * angle.cos()));
            }
            vertices.push(Point3D::new(star_x + s*0.01, star_y, 0.0)); // Center point
            
            for i in 0..8 {
                faces.push(vec![star + i, star + (i+1)%8, star + 8]);
                colors.push(star_silver);
            }
            
            (vertices, faces, colors)
        }
    }
}

/// Render a CAD mesh with the same rendering pipeline as built-in shapes
/// 
/// # Arguments
/// * `vertices` - List of all vertices in the mesh
/// * `triangles` - List of triangles, each containing (vertex indices, normal, color)
/// * `center_x` - Screen X center position
/// * `center_y` - Screen Y center position
/// * `size` - Scale factor for the mesh
/// * `rotation` - Current rotation angles
/// * `opacity` - Transparency (0.0 - 1.0)
pub fn render_cad_mesh(
    vertices: &[Point3D],
    triangles: &[(Vec<usize>, Point3D, (u8, u8, u8))],
    center_x: f32,
    center_y: f32,
    size: f32,
    rotation: Rotation3D,
    opacity: f32,
) -> String {
    let focal_length = 400.0;
    
    // Scale and rotate vertices
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|v| rotate_3d(v.scale(size), rotation))
        .collect();
    
    let light = Point3D::new(1.0, 1.0, 2.0).normalize();
    let view_direction = Point3D::new(0.0, 0.0, 1.0);
    
    let mut face_data: Vec<(f32, String)> = Vec::new();
    
    for (indices, normal, base_color) in triangles {
        // Rotate normal
        let rotated_normal = rotate_3d(*normal, rotation).normalize();
        
        // Back-face culling
        if opacity >= 1.0 && rotated_normal.dot(&view_direction) <= 0.0 {
            continue;
        }
        
        // Compute face center Z for depth sorting
        let face_center_z: f32 = indices.iter()
            .map(|&idx| rotated[idx].z)
            .sum::<f32>() / indices.len() as f32;
        
        // Compute lighting
        let brightness = rotated_normal.dot(&light).max(0.0);
        let final_brightness = 0.3 + brightness * 0.7;
        
        let color = (
            (base_color.0 as f32 * final_brightness) as u8,
            (base_color.1 as f32 * final_brightness) as u8,
            (base_color.2 as f32 * final_brightness) as u8,
        );
        
        // Project vertices
        let projected: Vec<(f32, f32)> = indices.iter()
            .map(|&idx| {
                let v = rotated[idx];
                let (px, py) = v.project(focal_length);
                (px + center_x, py + center_y)
            })
            .collect();
        
        // Format as polygon
        let points_str: String = projected.iter()
            .map(|(x, y)| format!("{:.1},{:.1}", x, y))
            .collect::<Vec<_>>()
            .join(" ");
        
        let svg_face = format!(
            r##"<polygon points="{}" fill="rgba({},{},{},{})" />"##,
            points_str, color.0, color.1, color.2, opacity
        );
        
        face_data.push((face_center_z, svg_face));
    }
    
    // Sort by depth (painter's algorithm)
    face_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    face_data.iter()
        .map(|(_, svg)| svg.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

