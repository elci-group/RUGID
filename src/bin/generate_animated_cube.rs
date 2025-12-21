//! Generate animated SVG of rotating cube

use rugid::geometry3d::{Point3D, Rotation3D, rotate_3d};
use rugid::svg::path_to_svg_d;

fn main() {
    println!("🎬 Generating animated rotating cube SVG...");
    
    let cube_size = 80.0;
    let center = Point3D::new(400.0, 300.0, 0.0);
    let focal_length = 600.0;
    
    // Generate 60 keyframes for smooth animation
    let keyframes = 60;
    let mut frame_groups = Vec::new();
    
    for frame in 0..keyframes {
        let rotation = Rotation3D::new(
            frame as f32 * 6.0,   // Full 360° rotation over 60 frames
            frame as f32 * 9.0,   // Faster Y rotation
            frame as f32 * 3.0,   // Slower Z rotation
        );
        
        let frame_svg = render_cube_frame(cube_size, center, rotation, focal_length, frame);
        frame_groups.push(frame_svg);
    }
    
    // Create animated SVG
    let animation_duration = 6.0; // seconds
    let frame_duration = animation_duration / keyframes as f32;
    
    let mut animated_svg = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"600\" viewBox=\"0 0 800 600\">\n\
  <rect width=\"800\" height=\"600\" fill=\"#1a1a1a\" />\n\
  <text x=\"400\" y=\"50\" text-anchor=\"middle\" fill=\"white\" font-size=\"24\" font-family=\"Arial\">\n\
    RUGID 3D Rotating Cube\n\
  </text>\n"
    );
    
    // Add all frame groups with animation
    for (i, frame_group) in frame_groups.iter().enumerate() {
        let begin = i as f32 * frame_duration;
        animated_svg.push_str(&format!(
            "  <g opacity=\"0\">\n\
    <animate attributeName=\"opacity\" values=\"0;1;0\" dur=\"{}s\" begin=\"{}s\" repeatCount=\"indefinite\" />\n\
{}\n\
  </g>\n",
            frame_duration, begin, frame_group
        ));
    }
    
    animated_svg.push_str("</svg>");
    
    // Save to file
    std::fs::write("rotating_cube.svg", &animated_svg)
        .expect("Failed to write SVG file");
    
    println!("✅ Generated rotating_cube.svg ({} bytes)", animated_svg.len());
    println!("📁 File: ./rotating_cube.svg");
    println!("🌐 Open in browser to view animation!");
}

fn render_cube_frame(
    size: f32,
    center: Point3D,
    rotation: Rotation3D,
    focal_length: f32,
    _frame: usize,
) -> String {
    let half = size / 2.0;
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
    
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|&v| rotate_3d(v, rotation))
        .collect();
    
    let faces = [
        ([0, 1, 2, 3], Point3D::new(0.0, 0.0, -1.0), (100, 100, 200)),  // Back - blue
        ([4, 7, 6, 5], Point3D::new(0.0, 0.0, 1.0), (200, 100, 100)),   // Front - red
        ([0, 4, 5, 1], Point3D::new(0.0, -1.0, 0.0), (100, 200, 100)),  // Bottom - green
        ([3, 2, 6, 7], Point3D::new(0.0, 1.0, 0.0), (200, 200, 100)),   // Top - yellow
        ([0, 3, 7, 4], Point3D::new(-1.0, 0.0, 0.0), (200, 100, 200)),  // Left - magenta
        ([1, 5, 6, 2], Point3D::new(1.0, 0.0, 0.0), (100, 200, 200)),   // Right - cyan
    ];
    
    let light = Point3D::new(1.0, 1.0, 2.0).normalize();
    let mut face_data: Vec<(f32, String)> = Vec::new();
    
    for (indices, normal_base, base_color) in &faces {
        let normal = rotate_3d(*normal_base, rotation).normalize();
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
                (px + center.x, py + center.y)
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
        .map(|(_, svg)| format!("    {}", svg))
        .collect::<Vec<_>>()
        .join("\n")
}
