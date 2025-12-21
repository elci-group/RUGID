//! 3D Cube Rendering Module
//!
//! Handles projection and rendering of 3D cube primitives

use crate::geometry3d::{Point3D, Rotation3D, rotate_3d};

/// Render a 3D cube to SVG paths
///
/// # Arguments
/// * `size` - Cube size in pixels
/// * `center_x`, `center_y` - Center position in 2D
/// * `rotation` - Current rotation state
///
/// # Returns
/// SVG markup for the cube faces
pub fn render_cube(
    center_x: f32,
    center_y: f32,
    size: f32,
    rotation: Rotation3D,
    opacity: f32,
) -> String {
    let focal_length = 400.0;
    
    // Define vertices of a cube (-1 to 1)
    let vertices = [
        Point3D::new(-1.0, -1.0, -1.0), // 0
        Point3D::new( 1.0, -1.0, -1.0), // 1
        Point3D::new( 1.0,  1.0, -1.0), // 2
        Point3D::new(-1.0,  1.0, -1.0), // 3
        Point3D::new(-1.0, -1.0,  1.0), // 4
        Point3D::new( 1.0, -1.0,  1.0), // 5
        Point3D::new( 1.0,  1.0,  1.0), // 6
        Point3D::new(-1.0,  1.0,  1.0), // 7
    ];
    
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|v| v.scale(size))
        .map(|v| rotate_3d(v, rotation))
        .collect();
    
    // Define faces with normals and colors
    let faces = [
        ([0, 1, 2, 3], Point3D::new(0.0, 0.0, -1.0), (100, 100, 200)), // Back - blue
        ([4, 7, 6, 5], Point3D::new(0.0, 0.0, 1.0), (200, 100, 100)),  // Front - red
        ([0, 4, 5, 1], Point3D::new(0.0, -1.0, 0.0), (100, 200, 100)), // Bottom - green
        ([3, 2, 6, 7], Point3D::new(0.0, 1.0, 0.0), (200, 200, 100)),  // Top - yellow
        ([0, 3, 7, 4], Point3D::new(-1.0, 0.0, 0.0), (200, 100, 200)), // Left - magenta
        ([1, 5, 6, 2], Point3D::new(1.0, 0.0, 0.0), (100, 200, 200)),  // Right - cyan
    ];
    
    let light = Point3D::new(1.0, 1.0, 2.0).normalize();
    
    // Project all vertices
    let projected: Vec<(f32, f32)> = rotated.iter()
        .map(|v| {
            let (px, py) = v.project(focal_length);
            (px + center_x, py + center_y)
        })
        .collect();

    let mut face_data: Vec<(f32, String)> = Vec::new();
    
    for (indices, normal_base, base_color) in &faces {
        // Transform normal
        let normal = rotate_3d(*normal_base, rotation).normalize();
        let view_direction = Point3D::new(0.0, 0.0, 1.0);
        
        // Back-face culling (only if opaque)
        if opacity >= 1.0 && normal.dot(&view_direction) <= 0.0 {
            continue;
        }
        
        // Calculate depth for sorting
        let face_center_z: f32 = indices.iter()
            .map(|&i| rotated[i].z)
            .sum::<f32>() / 4.0;
        
        // Lambertian shading
        let brightness = normal.dot(&light).max(0.0);
        let final_brightness = 0.3 + brightness * 0.7;
        
        let color = (
            (base_color.0 as f32 * final_brightness) as u8,
            (base_color.1 as f32 * final_brightness) as u8,
            (base_color.2 as f32 * final_brightness) as u8,
        );
        
        let mut svg_parts = String::new();
        
        // 1. Polygon (Filled Face)
        let points_str: String = indices.iter()
            .map(|&i| format!("{:.1},{:.1}", projected[i].0, projected[i].1))
            .collect::<Vec<_>>()
            .join(" ");
            
        svg_parts.push_str(&format!(
            r##"<polygon points="{}" fill="rgba({},{},{},{})" />"##,
            points_str, color.0, color.1, color.2, opacity
        ));
        
        // 2. Edges (Wireframe) - REMOVED
        
        face_data.push((face_center_z, svg_parts));
    }
    
    // Sort by depth (painter's algorithm: far to near)
    face_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    // Return concatenated SVG
    face_data.iter()
        .map(|(_, svg)| svg.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_cube() {
        let svg = render_cube(100.0, 250.0, 250.0, Rotation3D::zero(), 1.0);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("rgba("));
    }
}
