//! CAD to RUGID Converter
//!
//! Converts parsed CAD meshes into RUGID-compatible 3D shapes
//! that can be rendered using the existing rendering system.

use crate::geometry3d::Point3D;
use crate::cad::CADMesh;
use crate::cell::CellId;

/// Face representation for RUGID rendering
#[derive(Clone, Debug)]
pub struct RugidFace {
    pub vertices: Vec<Point3D>,
    pub normal: Point3D,
    pub color: (u8, u8, u8),
}

/// RUGID-compatible 3D shape converted from CAD
#[derive(Clone, Debug)]
pub struct RugidShape {
    pub name: String,
    pub faces: Vec<RugidFace>,
    pub center: Point3D,
    pub scale: f32,
}

impl RugidShape {
    /// Render the shape to SVG path elements
    pub fn to_svg(
        &self,
        rotation: crate::geometry3d::Rotation3D,
        center_x: f32,
        center_y: f32,
        size: f32,
        focal_length: f32,
    ) -> String {
        use crate::geometry3d::rotate_3d;

        let scale = size * self.scale;
        let light = Point3D::new(1.0, 1.0, 2.0).normalize();

        // Sort faces by depth (painter's algorithm)
        let mut sorted_faces: Vec<(f32, &RugidFace)> = self
            .faces
            .iter()
            .map(|face| {
                // Calculate face center Z after rotation
                let center_z: f32 = face
                    .vertices
                    .iter()
                    .map(|v| rotate_3d(v.scale(scale), rotation).z)
                    .sum::<f32>()
                    / face.vertices.len() as f32;
                (center_z, face)
            })
            .collect();

        sorted_faces.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut svg = String::new();

        for (_, face) in sorted_faces {
            // Rotate normal for backface culling and lighting
            let rotated_normal = rotate_3d(face.normal, rotation).normalize();
            let view_dir = Point3D::new(0.0, 0.0, 1.0);

            // Backface culling
            if rotated_normal.dot(&view_dir) <= 0.0 {
                continue;
            }

            // Calculate lighting
            let brightness = rotated_normal.dot(&light).max(0.0);
            let final_brightness = 0.3 + brightness * 0.7;

            let color = (
                (face.color.0 as f32 * final_brightness) as u8,
                (face.color.1 as f32 * final_brightness) as u8,
                (face.color.2 as f32 * final_brightness) as u8,
            );

            // Project vertices
            let projected: Vec<(f32, f32)> = face
                .vertices
                .iter()
                .map(|v| {
                    let rotated = rotate_3d(v.scale(scale), rotation);
                    let (px, py) = rotated.project(focal_length);
                    (px + center_x, py + center_y)
                })
                .collect();

            // Generate SVG polygon (WgpuPlatform parses <polygon>, not <path>)
            if projected.len() >= 3 {
                // Format points as "x1,y1 x2,y2 x3,y3"
                let points_str: String = projected
                    .iter()
                    .map(|(x, y)| format!("{:.2},{:.2}", x, y))
                    .collect::<Vec<_>>()
                    .join(" ");
                
                svg.push_str(&format!(
                    r#"<polygon points="{}" fill="rgb({},{},{})" />"#,
                    points_str, color.0, color.1, color.2
                ));
                svg.push('\n');
            }
        }

        svg
    }
}

impl CADMesh {
    /// Convert to RUGID shape with default gray coloring
    pub fn to_rugid_shape(&self) -> RugidShape {
        self.to_rugid_shape_colored((180, 180, 200)) // Light steel blue
    }

    /// Convert to RUGID shape with custom base color
    pub fn to_rugid_shape_colored(&self, base_color: (u8, u8, u8)) -> RugidShape {
        let faces = self
            .triangles
            .iter()
            .map(|tri| {
                let vertices = vec![
                    self.vertices[tri.vertices[0]],
                    self.vertices[tri.vertices[1]],
                    self.vertices[tri.vertices[2]],
                ];
                RugidFace {
                    vertices,
                    normal: tri.normal,
                    color: tri.color.unwrap_or(base_color),
                }
            })
            .collect();

        RugidShape {
            name: self.name.clone(),
            faces,
            center: self.bounds.center(),
            scale: 1.0,
        }
    }
}

/// Helper to register a CAD mesh with the RUGID renderer
pub struct CADRenderer {
    pub shapes: std::collections::HashMap<CellId, RugidShape>,
}

impl CADRenderer {
    pub fn new() -> Self {
        Self {
            shapes: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, id: CellId, mesh: &CADMesh) {
        let mut normalized = mesh.clone();
        normalized.normalize();
        self.shapes.insert(id, normalized.to_rugid_shape());
    }

    pub fn register_colored(&mut self, id: CellId, mesh: &CADMesh, color: (u8, u8, u8)) {
        let mut normalized = mesh.clone();
        normalized.normalize();
        self.shapes.insert(id, normalized.to_rugid_shape_colored(color));
    }

    /// Render a shape to SVG
    pub fn render(
        &self,
        id: CellId,
        rotation: crate::geometry3d::Rotation3D,
        center_x: f32,
        center_y: f32,
        size: f32,
    ) -> Option<String> {
        self.shapes
            .get(&id)
            .map(|shape| shape.to_svg(rotation, center_x, center_y, size, 400.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cad::stl::generate_sample_stl;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_convert_to_rugid_shape() {
        let stl_content = generate_sample_stl();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(stl_content.as_bytes()).unwrap();

        let mesh = crate::cad::stl::parse_stl(temp_file.path()).unwrap();
        let shape = mesh.to_rugid_shape();

        assert_eq!(shape.faces.len(), 12);
        assert!(shape.faces.iter().all(|f| f.vertices.len() == 3));
    }
}
