//! Equilateral Shapes Rotation Demo
//!
//! Demonstrates rotation of Tetrahedron, Cube, and Octahedron.

use rugid::geometry3d::{Point3D, Rotation3D, rotate_3d};
use rugid::svg::path_to_svg_d;
use std::f32::consts::PI;
use std::path::Path;
use std::fs;

#[derive(Clone, Copy, Debug)]
enum ShapeType {
    Tetrahedron,
    Cube,
    Octahedron,
}

impl ShapeType {
    fn name(&self) -> &'static str {
        match self {
            ShapeType::Tetrahedron => "tetrahedron",
            ShapeType::Cube => "cube",
            ShapeType::Octahedron => "octahedron",
        }
    }
}

fn main() {
    println!("🎲 RUGID Equilateral Shapes Demo");
    println!("{}", "=".repeat(60));

    let output_dir = Path::new("demo_output");
    if !output_dir.exists() {
        fs::create_dir(output_dir).unwrap();
    }

    generate_demo(ShapeType::Tetrahedron, output_dir);
    generate_demo(ShapeType::Cube, output_dir);
    generate_demo(ShapeType::Octahedron, output_dir);

    println!("\n✨ All demos complete!");
}

fn generate_demo(shape_type: ShapeType, output_dir: &Path) {
    println!("\nGenerating demo for: {:?}", shape_type);
    
    // Parameters
    let size = 100.0;
    let center = Point3D::new(400.0, 300.0, 0.0);
    let focal_length = 600.0;
    let frames = 120; // Shorter than original to save time, but enough to show rotation
    
    let (vertices, faces_indices, face_colors) = get_shape_data(shape_type, size);
    
    // Precompute face normals (assuming rigid body, normals rotate with body)
    // Actually, we compute base normals here.
    let face_normals: Vec<Point3D> = faces_indices.iter().map(|indices| {
        let v0 = vertices[indices[0]];
        let v1 = vertices[indices[1]];
        let v2 = vertices[indices[2]];
        compute_normal(v0, v1, v2)
    }).collect();

    let rotation_speed_x = 1.0;
    let rotation_speed_y = 1.5;
    let rotation_speed_z = 0.5;

    for frame in 0..frames {
        let rotation = Rotation3D::new(
            frame as f32 * rotation_speed_x,
            frame as f32 * rotation_speed_y,
            frame as f32 * rotation_speed_z,
        );

        let svg = render_frame(
            &vertices,
            &faces_indices,
            &face_normals,
            &face_colors,
            center,
            rotation,
            focal_length
        );

        let filename = format!("{}_frame_{:03}.svg", shape_type.name(), frame);
        fs::write(output_dir.join(filename), svg).unwrap();
    }
    
    println!("  Generated {} frames.", frames);
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
            // Tetrahedron vertices (inscribed in a cube for simplicity, or standard coords)
            // Using (1,1,1), (1,-1,-1), (-1,1,-1), (-1,-1,1) scaled
            let s = size * 0.7; // Scale adjustment
            let vertices = vec![
                Point3D::new( s,  s,  s), // 0
                Point3D::new( s, -s, -s), // 1
                Point3D::new(-s,  s, -s), // 2
                Point3D::new(-s, -s,  s), // 3
            ];
            
            // Faces (CCW)
            // 0-1-2 (Normal OUT)
            // 0-3-1
            // 0-2-3
            // 1-3-2
            let faces = vec![
                vec![0, 1, 2],
                vec![0, 3, 1],
                vec![0, 2, 3],
                vec![1, 3, 2],
            ];
            
            let colors = vec![
                (255, 100, 100), // Red
                (100, 255, 100), // Green
                (100, 100, 255), // Blue
                (255, 255, 100), // Yellow
            ];
            
            (vertices, faces, colors)
        },
        ShapeType::Cube => {
            let vertices = vec![
                Point3D::new(-half, -half, -half), // 0
                Point3D::new( half, -half, -half), // 1
                Point3D::new( half,  half, -half), // 2
                Point3D::new(-half,  half, -half), // 3
                Point3D::new(-half, -half,  half), // 4
                Point3D::new( half, -half,  half), // 5
                Point3D::new( half,  half,  half), // 6
                Point3D::new(-half,  half,  half), // 7
            ];
            
            let faces = vec![
                vec![0, 1, 2, 3], // Back
                vec![4, 7, 6, 5], // Front
                vec![0, 4, 5, 1], // Bottom
                vec![3, 2, 6, 7], // Top
                vec![0, 3, 7, 4], // Left
                vec![1, 5, 6, 2], // Right
            ];
            
            let colors = vec![
                (100, 100, 200),
                (200, 100, 100),
                (100, 200, 100),
                (200, 200, 100),
                (200, 100, 200),
                (100, 200, 200),
            ];
            
            (vertices, faces, colors)
        },
        ShapeType::Octahedron => {
            let s = size * 0.8;
            let vertices = vec![
                Point3D::new( s,  0.0,  0.0), // 0: +X
                Point3D::new(-s,  0.0,  0.0), // 1: -X
                Point3D::new( 0.0,  s,  0.0), // 2: +Y
                Point3D::new( 0.0, -s,  0.0), // 3: -Y
                Point3D::new( 0.0,  0.0,  s), // 4: +Z
                Point3D::new( 0.0,  0.0, -s), // 5: -Z
            ];
            
            // Faces
            // Top (Z+)
            // 4-0-2
            // 4-2-1
            // 4-1-3
            // 4-3-0
            // Bottom (Z-)
            // 5-2-0
            // 5-1-2
            // 5-3-1
            // 5-0-3
            let faces = vec![
                vec![4, 0, 2],
                vec![4, 2, 1],
                vec![4, 1, 3],
                vec![4, 3, 0],
                vec![5, 2, 0],
                vec![5, 1, 2],
                vec![5, 3, 1],
                vec![5, 0, 3],
            ];
            
            let colors = vec![
                (255, 0, 0),
                (0, 255, 0),
                (0, 0, 255),
                (255, 255, 0),
                (255, 0, 255),
                (0, 255, 255),
                (255, 128, 0),
                (128, 0, 255),
            ];
            
            (vertices, faces, colors)
        }
    }
}

fn render_frame(
    vertices: &[Point3D],
    faces: &[Vec<usize>],
    normals: &[Point3D],
    colors: &[(u8, u8, u8)],
    center: Point3D,
    rotation: Rotation3D,
    focal_length: f32
) -> String {
    // Rotate vertices
    let rotated_vertices: Vec<Point3D> = vertices.iter()
        .map(|&v| rotate_3d(v, rotation))
        .collect();
        
    // Light source
    let light = Point3D::new(1.0, 1.0, 2.0).normalize();
    let view_direction = Point3D::new(0.0, 0.0, 1.0);
    
    struct RenderFace {
        z: f32,
        svg: String,
    }
    
    let mut render_list: Vec<RenderFace> = Vec::new();
    
    for (i, indices) in faces.iter().enumerate() {
        // Rotate normal
        let normal = rotate_3d(normals[i], rotation).normalize();
        
        // Back-face culling
        if normal.dot(&view_direction) <= 0.0 {
            continue;
        }
        
        // Depth sorting (centroid Z)
        let face_center_z: f32 = indices.iter()
            .map(|&idx| rotated_vertices[idx].z)
            .sum::<f32>() / indices.len() as f32;
            
        // Shading
        let brightness = normal.dot(&light).max(0.0);
        let base_brightness = 0.3;
        let final_brightness = base_brightness + brightness * 0.7;
        
        let base_color = colors[i % colors.len()];
        let color = (
            (base_color.0 as f32 * final_brightness) as u8,
            (base_color.1 as f32 * final_brightness) as u8,
            (base_color.2 as f32 * final_brightness) as u8,
        );
        
        // Project
        let projected: Vec<(f32, f32)> = indices.iter()
            .map(|&idx| {
                let (px, py) = rotated_vertices[idx].project(focal_length);
                (px + center.x, py + center.y)
            })
            .collect();
            
        let path_d = path_to_svg_d(&projected, true);
        let svg_face = format!(
            r#"<path d="{}" fill="rgb({},{},{})" stroke="black" stroke-width="1" />"#,
            path_d, color.0, color.1, color.2
        );
        
        render_list.push(RenderFace { z: face_center_z, svg: svg_face });
    }
    
    // Sort far to near
    render_list.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
    
    let faces_svg: String = render_list.iter()
        .map(|f| f.svg.clone())
        .collect::<Vec<_>>()
        .join("\n  ");
        
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"600\" viewBox=\"0 0 800 600\">\n  <rect width=\"800\" height=\"600\" fill=\"#1a1a1a\" />\n  {}\n</svg>",
        faces_svg
    )
}
