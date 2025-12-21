use crate::geometry3d::Point3D;
use crate::physics::optics::{RenderFace, Scene, calculate_lighting, LightingResult};

pub struct TessellationConfig {
    pub intensity_threshold: f32, // Max allowed difference in intensity
    pub max_depth: usize,
}

impl Default for TessellationConfig {
    fn default() -> Self {
        Self {
            intensity_threshold: 0.1,
            max_depth: 3,
        }
    }
}

/// A tessellated triangle ready for rendering
pub struct TessellatedTriangle {
    pub vertices: [Point3D; 3],
    pub color_v1: (u8, u8, u8),
    pub color_v2: (u8, u8, u8),
    pub color_v3: (u8, u8, u8),
}

pub struct Tessellator {
    config: TessellationConfig,
}

impl Tessellator {
    pub fn new(config: TessellationConfig) -> Self {
        Self { config }
    }

    pub fn tessellate(
        &self,
        face: &RenderFace,
        scene: &Scene,
        view_pos: Point3D,
    ) -> Vec<TessellatedTriangle> {
        let mut result = Vec::new();
        
        // We assume the face is a triangle or convex polygon. 
        // For now, let's assume it's a triangle or fan it.
        if face.vertices.len() < 3 { return result; }
        
        // Simple fan triangulation for initial face
        for i in 1..face.vertices.len()-1 {
            let v0 = face.vertices[0];
            let v1 = face.vertices[i];
            let v2 = face.vertices[i+1];
            
            self.recursive_tessellate(
                v0, v1, v2,
                face, scene, view_pos,
                0,
                &mut result
            );
        }
        
        result
    }

    fn recursive_tessellate(
        &self,
        v0: Point3D, v1: Point3D, v2: Point3D,
        face: &RenderFace,
        scene: &Scene,
        view_pos: Point3D,
        depth: usize,
        output: &mut Vec<TessellatedTriangle>,
    ) {
        // 1. Calculate lighting at vertices
        let l0 = self.compute_light(v0, face, scene, view_pos);
        let l1 = self.compute_light(v1, face, scene, view_pos);
        let l2 = self.compute_light(v2, face, scene, view_pos);

        // 2. Check if we need to subdivide
        if depth < self.config.max_depth {
            // Test Midpoints
            let m01 = v0.midpoint(v1);
            let m12 = v1.midpoint(v2);
            let m20 = v2.midpoint(v0);
            
            let l_m01_actual = self.compute_light(m01, face, scene, view_pos);
            let l_m01_interp = (l0.intensity + l1.intensity) * 0.5;
            
            let l_m12_actual = self.compute_light(m12, face, scene, view_pos);
            let l_m12_interp = (l1.intensity + l2.intensity) * 0.5;
            
            let l_m20_actual = self.compute_light(m20, face, scene, view_pos);
            let l_m20_interp = (l2.intensity + l0.intensity) * 0.5;
            
            let err01 = (l_m01_actual.intensity - l_m01_interp).abs();
            let err12 = (l_m12_actual.intensity - l_m12_interp).abs();
            let err20 = (l_m20_actual.intensity - l_m20_interp).abs();
            
            let max_err = err01.max(err12).max(err20);
            
            if max_err > self.config.intensity_threshold {
                // Subdivide into 4 triangles (Edge Midpoints)
                //      v0
                //     /  \
                //   m01--m20
                //   / \  / \
                // v1--m12--v2
                
                self.recursive_tessellate(v0, m01, m20, face, scene, view_pos, depth + 1, output);
                self.recursive_tessellate(v1, m12, m01, face, scene, view_pos, depth + 1, output);
                self.recursive_tessellate(v2, m20, m12, face, scene, view_pos, depth + 1, output);
                self.recursive_tessellate(m01, m12, m20, face, scene, view_pos, depth + 1, output);
                return;
            }
        }

        // 3. Leaf Node - Push Triangle
        output.push(TessellatedTriangle {
            vertices: [v0, v1, v2],
            color_v1: l0.color,
            color_v2: l1.color,
            color_v3: l2.color,
        });
    }
    
    fn compute_light(&self, p: Point3D, face: &RenderFace, scene: &Scene, view_pos: Point3D) -> LightingResult {
        let view_dir = Point3D::new(
            view_pos.x - p.x,
            view_pos.y - p.y,
            view_pos.z - p.z,
        ).normalize();
        
        calculate_lighting(p, face.normal, view_dir, &face.material, scene)
    }
}
