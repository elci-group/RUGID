//! Rotating Cube Demo
//!
//! Demonstrates RUGID's 3D rendering capabilities with a rotating cube.
//! Uses differential rotation transformations for true 3D perceptual manipulation.

use rugid::geometry3d::{Point3D, Rotation3D, rotate_3d, surface_normal};
use rugid::svg::path_to_svg_d;
use std::f32::consts::PI;

fn main() {
    println!("🎲 RUGID 3D Rotating Cube Demo");
    println!("{}", "=".repeat(60));
    println!();
    
    // Cube parameters
    let cube_size = 80.0;
    let center = Point3D::new(400.0, 300.0, 0.0);
    let focal_length = 600.0;
    
    println!("📐 Cube Configuration:");
    println!("  Size: {}px", cube_size);
    println!("  Center: ({}, {}, {})", center.x, center.y, center.z);
    println!("  Focal length: {}px", focal_length);
    println!();
    
    // Animation parameters
    let frames = 360;
    let rotation_speed_x = 1.0;  // degrees per frame
    let rotation_speed_y = 1.5;
    let rotation_speed_z = 0.5;
    
    println!("🎬 Animation:");
    println!("  Frames: {}", frames);
    println!("  Rotation speeds: X={:.1}°, Y={:.1}°, Z={:.1}°/frame", 
        rotation_speed_x, rotation_speed_y, rotation_speed_z);
    println!();
    
    // Generate frames
    println!("🎥 Generating {} frames...", frames);
    let mut svg_frames = Vec::new();
    
    for frame in 0..frames {
        let rotation = Rotation3D::new(
            frame as f32 * rotation_speed_x,
            frame as f32 * rotation_speed_y,
            frame as f32 * rotation_speed_z,
        );
        
        let svg = render_cube_frame(cube_size, center, rotation, focal_length);
        svg_frames.push(svg);
        
        if frame % 90 == 0 {
            println!("  Frame {}/{} - Rotation: pitch={:.0}°, yaw={:.0}°, roll={:.0}°",
                frame, frames, rotation.pitch, rotation.yaw, rotation.roll);
        }
    }
    
    println!("\n✅ Generated {} SVG frames", svg_frames.len());
    
    // Show sample frames
    println!("\n📊 Sample Frame Analysis:");
    println!("{}", "=".repeat(60));
    
    for (i, sample_frame) in [0, 90, 180, 270].iter().enumerate() {
        if *sample_frame < svg_frames.len() {
            let frame_size = svg_frames[*sample_frame].len();
            let rotation = Rotation3D::new(
                *sample_frame as f32 * rotation_speed_x,
                *sample_frame as f32 * rotation_speed_y,
                *sample_frame as f32 * rotation_speed_z,
            );
            
            println!("\nFrame {}:", sample_frame);
            println!("  Rotation: ({:.0}°, {:.0}°, {:.0}°)", 
                rotation.pitch, rotation.yaw, rotation.roll);
            println!("  SVG size: {} bytes", frame_size);
            println!("  Visible faces: {}", count_visible_faces(rotation));
        }
    }
    
    // Export sample frame
    println!("\n📄 Sample Frame SVG:");
    println!("{}", "=".repeat(60));
    let sample_svg = &svg_frames[0];
    println!("{}", &sample_svg[..300.min(sample_svg.len())]);
    println!("  ... ({} total bytes)", sample_svg.len());
    
    println!("\n✨ Demo complete!");
    println!("\n💡 3D Capabilities Demonstrated:");
    println!("  ✓ 3-axis rotation (pitch, yaw, roll)");
    println!("  ✓ Perspective projection");
    println!("  ✓ Depth-based face culling");
    println!("  ✓ Lambertian shading");
    println!("  ✓ Depth sorting (painter's algorithm)");
    println!("  ✓ Differential rotation transformations");
    println!("\n🎯 True 3D rendering in 2D SVG: SUCCESS");
}

/// Render a single frame of the rotating cube
fn render_cube_frame(
    size: f32,
    center: Point3D,
    rotation: Rotation3D,
    focal_length: f32
) -> String {
    // Define cube vertices (centered at origin)
    let half = size / 2.0;
    let vertices = [
        Point3D::new(-half, -half, -half), // 0: back-bottom-left
        Point3D::new( half, -half, -half), // 1: back-bottom-right
        Point3D::new( half,  half, -half), // 2: back-top-right
        Point3D::new(-half,  half, -half), // 3: back-top-left
        Point3D::new(-half, -half,  half), // 4: front-bottom-left
        Point3D::new( half, -half,  half), // 5: front-bottom-right
        Point3D::new( half,  half,  half), // 6: front-top-right
        Point3D::new(-half,  half,  half), // 7: front-top-left
    ];
    
    // Rotate vertices
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|&v| rotate_3d(v, rotation))
        .collect();
    
    // Define faces (counter-clockwise winding)
    let faces = [
        ([0, 1, 2, 3], Point3D::new(0.0, 0.0, -1.0)), // Back
        ([4, 7, 6, 5], Point3D::new(0.0, 0.0, 1.0)),  // Front
        ([0, 4, 5, 1], Point3D::new(0.0, -1.0, 0.0)), // Bottom
        ([3, 2, 6, 7], Point3D::new(0.0, 1.0, 0.0)),  // Top
        ([0, 3, 7, 4], Point3D::new(-1.0, 0.0, 0.0)), // Left
        ([1, 5, 6, 2], Point3D::new(1.0, 0.0, 0.0)),  // Right
    ];
    
    // Light source (from upper-right-front)
    let light = Point3D::new(1.0, 1.0, 2.0).normalize();
    
    // Render faces with depth sorting
    let mut face_data: Vec<(f32, String)> = Vec::new();
    
    for (indices, normal_base) in &faces {
        // Rotate normal
        let normal = rotate_3d(*normal_base, rotation).normalize();
        
        // Back-face culling: only render if facing camera
        let view_direction = Point3D::new(0.0, 0.0, 1.0);
        if normal.dot(&view_direction) <= 0.0 {
            continue;
        }
        
        // Calculate face center Z for depth sorting
        let face_center_z: f32 = indices.iter()
            .map(|&i| rotated[i].z)
            .sum::<f32>() / 4.0;
        
        // Calculate shading (Lambertian)
        let brightness = normal.dot(&light).max(0.0);
        let base_brightness = 0.3;  // Ambient light
        let final_brightness = base_brightness + brightness * 0.7;
        
        // Face color based on which face it is
        let base_color = match indices {
            [0, 1, 2, 3] => (100, 100, 200),  // Back - blue
            [4, 7, 6, 5] => (200, 100, 100),  // Front - red
            [0, 4, 5, 1] => (100, 200, 100),  // Bottom - green
            [3, 2, 6, 7] => (200, 200, 100),  // Top - yellow
            [0, 3, 7, 4] => (200, 100, 200),  // Left - magenta
            _ => (100, 200, 200),             // Right - cyan
        };
        
        let color = (
            (base_color.0 as f32 * final_brightness) as u8,
            (base_color.1 as f32 * final_brightness) as u8,
            (base_color.2 as f32 * final_brightness) as u8,
        );
        
        // Project vertices to 2D
        let projected: Vec<(f32, f32)> = indices.iter()
            .map(|&i| {
                let (px, py) = rotated[i].project(focal_length);
                (px + center.x, py + center.y)
            })
            .collect();
        
        // Generate SVG path
        let path_d = path_to_svg_d(&projected, true);
        let svg_face = format!(
            r#"<path d="{}" fill="rgb({},{},{})" stroke="black" stroke-width="1" />"#,
            path_d, color.0, color.1, color.2
        );
        
        face_data.push((face_center_z, svg_face));
    }
    
    // Sort by depth (painter's algorithm: far to near)
    face_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    // Combine into SVG
    let faces_svg: String = face_data.iter()
        .map(|(_, svg)| svg.clone())
        .collect::<Vec<_>>()
        .join("\n  ");
    
    let svg_output = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"600\" viewBox=\"0 0 800 600\">\n  <rect width=\"800\" height=\"600\" fill=\"#1a1a1a\" />\n  {}\n</svg>",
        faces_svg
    );
    
    svg_output
}

/// Count how many faces would be visible at given rotation
fn count_visible_faces(rotation: Rotation3D) -> usize {
    let face_normals = [
        Point3D::new(0.0, 0.0, -1.0),  // Back
        Point3D::new(0.0, 0.0, 1.0),   // Front
        Point3D::new(0.0, -1.0, 0.0),  // Bottom
        Point3D::new(0.0, 1.0, 0.0),   // Top
        Point3D::new(-1.0, 0.0, 0.0),  // Left
        Point3D::new(1.0, 0.0, 0.0),   // Right
    ];
    
    let view = Point3D::new(0.0, 0.0, 1.0);
    
    face_normals.iter()
        .map(|&normal| rotate_3d(normal, rotation).normalize())
        .filter(|rotated_normal| rotated_normal.dot(&view) > 0.0)
        .count()
}
