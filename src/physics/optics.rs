use crate::geometry3d::{Point3D, Rotation3D, rotate_3d};
use crate::physics::light::{Ray, LightSource, LightSourceType, OpticalMaterial, HitRecord};
use std::f32::consts::PI;

/// Represents a renderable face in the scene for ray tracing
#[derive(Clone, Debug)]
pub struct RenderFace {
    pub vertices: Vec<Point3D>,
    pub normal: Point3D,
    pub center: Point3D,
    pub material: OpticalMaterial,
    pub parent_id: Option<crate::cell::CellId>,
}

/// The scene context for ray tracing
pub struct Scene {
    pub faces: Vec<RenderFace>,
    pub lights: Vec<LightSource>,
    pub ambient_light: (f32, f32, f32), // RGB intensity 0.0-1.0
}

/// Result of lighting calculation for a single point
#[derive(Clone, Copy, Debug)]
pub struct LightingResult {
    pub color: (u8, u8, u8),
    pub intensity: f32,
}

/// Calculated gradient for a face
#[derive(Clone, Debug)]
pub struct FaceGradient {
    pub is_gradient: bool,
    pub start_point: (f32, f32), // Relative 0.0-1.0 or absolute coords
    pub end_point: (f32, f32),
    pub start_color: (u8, u8, u8),
    pub end_color: (u8, u8, u8),
    pub radial: bool,
    pub center: (f32, f32),
    pub radius: f32,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            faces: Vec::new(),
            lights: Vec::new(),
            ambient_light: (0.1, 0.1, 0.1),
        }
    }

    pub fn add_face(&mut self, face: RenderFace) {
        self.faces.push(face);
    }

    pub fn add_light(&mut self, light: LightSource) {
        self.lights.push(light);
    }
}

/// Calculate lighting at a specific point on a surface
pub fn calculate_lighting(
    point: Point3D,
    normal: Point3D,
    view_dir: Point3D,
    material: &OpticalMaterial,
    scene: &Scene,
) -> LightingResult {
    let mut total_r = scene.ambient_light.0 * material.ambient * material.color.0 as f32;
    let mut total_g = scene.ambient_light.1 * material.ambient * material.color.1 as f32;
    let mut total_b = scene.ambient_light.2 * material.ambient * material.color.2 as f32;

    for light in &scene.lights {
        let (light_dir, distance, attenuation) = match light.light_type {
            LightSourceType::Point => {
                let d_vec = Point3D::new(
                    light.position.x - point.x,
                    light.position.y - point.y,
                    light.position.z - point.z,
                );
                let dist = d_vec.magnitude();
                // Simple linear attenuation - Reduced falloff for better visibility
                let att = 1.0 / (1.0 + 0.0001 * dist + 0.000001 * dist * dist);
                (d_vec.normalize(), dist, att)
            },
            LightSourceType::Directional => {
                // Light position is direction vector pointing TO light
                (light.position.normalize(), f32::INFINITY, 1.0)
            },
            LightSourceType::Spot { cutoff_angle, direction } => {
                let d_vec = Point3D::new(
                    light.position.x - point.x,
                    light.position.y - point.y,
                    light.position.z - point.z,
                );
                let dist = d_vec.magnitude();
                let l_dir = d_vec.normalize();
                
                // Check angle
                let spot_dir = direction.normalize();
                // Dot product of light direction (from source) and vector to point
                // Vector from source to point is -l_dir
                let angle_cos = spot_dir.dot(&Point3D::new(-l_dir.x, -l_dir.y, -l_dir.z));
                
                if angle_cos > cutoff_angle.to_radians().cos() {
                    let att = 1.0 / (1.0 + 0.0001 * dist);
                    (l_dir, dist, att)
                } else {
                    (l_dir, dist, 0.0)
                }
            }
        };

        if attenuation <= 0.001 { continue; }

        // Shadow check
        let shadow_ray = Ray::new(
            Point3D::new(point.x + normal.x * 0.1, point.y + normal.y * 0.1, point.z + normal.z * 0.1),
            light_dir
        );
        
        let mut in_shadow = false;
        for face in &scene.faces {
            if let Some(hit) = intersect_triangle(&shadow_ray, face.vertices[0], face.vertices[1], face.vertices[2]) {
                if hit.t < distance {
                    in_shadow = true;
                    break;
                }
            }
            // For quads, check second triangle
            if face.vertices.len() == 4 {
                if let Some(hit) = intersect_triangle(&shadow_ray, face.vertices[0], face.vertices[2], face.vertices[3]) {
                    if hit.t < distance {
                        in_shadow = true;
                        break;
                    }
                }
            }
        }

        if in_shadow { continue; }

        // Diffuse (Lambertian)
        let diff = normal.dot(&light_dir).max(0.0);
        let diffuse_strength = diff * material.diffuse * light.intensity * attenuation;

        // Specular (Blinn-Phong)
        let half_dir = Point3D::new(
            light_dir.x + view_dir.x,
            light_dir.y + view_dir.y,
            light_dir.z + view_dir.z,
        ).normalize();
        let spec_angle = normal.dot(&half_dir).max(0.0);
        let specular_strength = spec_angle.powf(material.shininess) * material.specular * light.intensity * attenuation;

        // Add contributions
        total_r += diffuse_strength * material.color.0 as f32 + specular_strength * 255.0;
        total_g += diffuse_strength * material.color.1 as f32 + specular_strength * 255.0;
        total_b += diffuse_strength * material.color.2 as f32 + specular_strength * 255.0;
    }

    LightingResult {
        color: (
            total_r.min(255.0) as u8,
            total_g.min(255.0) as u8,
            total_b.min(255.0) as u8,
        ),
        intensity: (total_r + total_g + total_b) / (3.0 * 255.0),
    }
}

/// The "Novel" Gradient-Space Ray Tracing method
/// Instead of rasterizing, we sample lighting at vertices and fit an SVG gradient
pub fn compute_face_gradient(
    face: &RenderFace,
    scene: &Scene,
    view_pos: Point3D,
) -> FaceGradient {
    // 1. Calculate lighting at all vertices
    let mut vertex_colors = Vec::new();
    let mut vertex_intensities = Vec::new();
    
    for v in &face.vertices {
        let view_dir = Point3D::new(
            view_pos.x - v.x,
            view_pos.y - v.y,
            view_pos.z - v.z,
        ).normalize();
        
        let result = calculate_lighting(*v, face.normal, view_dir, &face.material, scene);
        vertex_colors.push(result.color);
        vertex_intensities.push(result.intensity);
    }

    // 2. Analyze intensity distribution
    let min_i = vertex_intensities.iter().fold(1.0f32, |a, &b| a.min(b));
    let max_i = vertex_intensities.iter().fold(0.0f32, |a, &b| a.max(b));
    
    // If flat lighting, return flat color
    if (max_i - min_i).abs() < 0.05 {
        let avg_r = vertex_colors.iter().map(|c| c.0 as f32).sum::<f32>() / vertex_colors.len() as f32;
        let avg_g = vertex_colors.iter().map(|c| c.1 as f32).sum::<f32>() / vertex_colors.len() as f32;
        let avg_b = vertex_colors.iter().map(|c| c.2 as f32).sum::<f32>() / vertex_colors.len() as f32;
        
        return FaceGradient {
            is_gradient: false,
            start_point: (0.0, 0.0),
            end_point: (0.0, 0.0),
            start_color: (avg_r as u8, avg_g as u8, avg_b as u8),
            end_color: (avg_r as u8, avg_g as u8, avg_b as u8),
            radial: false,
            center: (0.0, 0.0),
            radius: 0.0,
        };
    }

    // 3. Determine Gradient Vector (Linear Regression in 2D projected space)
    // Simplified: Find vertex with max intensity and min intensity
    let mut max_idx = 0;
    let mut min_idx = 0;
    for (i, val) in vertex_intensities.iter().enumerate() {
        if *val > vertex_intensities[max_idx] { max_idx = i; }
        if *val < vertex_intensities[min_idx] { min_idx = i; }
    }
    
    // Check for specular highlight (Radial Gradient candidate)
    // If center is brighter than all vertices, it's likely a highlight
    let center_view_dir = Point3D::new(
        view_pos.x - face.center.x,
        view_pos.y - face.center.y,
        view_pos.z - face.center.z,
    ).normalize();
    let center_light = calculate_lighting(face.center, face.normal, center_view_dir, &face.material, scene);
    
    if center_light.intensity > max_i + 0.1 {
        // Radial gradient for highlight
        return FaceGradient {
            is_gradient: true,
            start_point: (0.0, 0.0),
            end_point: (0.0, 0.0),
            start_color: center_light.color, // Highlight color
            end_color: vertex_colors[min_idx], // Falloff color
            radial: true,
            center: (0.5, 0.5), // Approximate center of face
            radius: 0.7,
        };
    }

    // Linear Gradient
    // Project vertices to 2D for gradient coordinates (0.0-1.0 relative to bounding box)
    // This is complex to map perfectly to SVG polygon space without a transform.
    // Approximation: Use the projected screen coordinates if available, or just use relative index logic.
    // For now, we'll return the colors and let the renderer map them.
    
    FaceGradient {
        is_gradient: true,
        start_point: (0.0, 0.0), // Placeholder, renderer will calculate based on vertex positions
        end_point: (1.0, 1.0),
        start_color: vertex_colors[max_idx], // Lightest
        end_color: vertex_colors[min_idx],   // Darkest
        radial: false,
        center: (0.0, 0.0),
        radius: 0.0,
    }
}

// Möller–Trumbore intersection algorithm
fn intersect_triangle(ray: &Ray, v0: Point3D, v1: Point3D, v2: Point3D) -> Option<HitRecord> {
    let epsilon = 0.0000001;
    let edge1 = Point3D::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
    let edge2 = Point3D::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
    
    let h = ray.direction.cross(&edge2);
    let a = edge1.dot(&h);
    
    if a > -epsilon && a < epsilon {
        return None; // Parallel
    }
    
    let f = 1.0 / a;
    let s = Point3D::new(ray.origin.x - v0.x, ray.origin.y - v0.y, ray.origin.z - v0.z);
    let u = f * s.dot(&h);
    
    if u < 0.0 || u > 1.0 {
        return None;
    }
    
    let q = s.cross(&edge1);
    let v = f * ray.direction.dot(&q);
    
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    
    let t = f * edge2.dot(&q);
    
    if t > epsilon {
        let hit_point = ray.at(t);
        let normal = edge1.cross(&edge2).normalize();
        
        Some(HitRecord {
            t,
            point: hit_point,
            normal, // Should check facing
            material: OpticalMaterial::default(), // Placeholder
        })
    } else {
        None
    }
}
