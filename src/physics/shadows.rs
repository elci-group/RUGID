use crate::geometry3d::Point3D;
use crate::physics::optics::RenderFace;

pub struct ShadowEngine;

impl ShadowEngine {
    /// Detect the silhouette edges of a mesh relative to a light source
    /// Returns a list of edges (start, end) that form the silhouette
    pub fn detect_silhouette(
        faces: &[RenderFace],
        light_pos: Point3D,
    ) -> Vec<(Point3D, Point3D)> {
        // 1. Identify all edges of lit faces
        let mut candidate_edges = Vec::new();
        
        for face in faces {
            // Check if face is facing the light
            let face_center = face.center;
            let light_dir = (light_pos - face_center).normalize();
            let is_lit = face.normal.dot(&light_dir) > 0.0;

            if is_lit {
                let v = &face.vertices;
                let n = v.len();
                for i in 0..n {
                    let p1 = v[i];
                    let p2 = v[(i + 1) % n];
                    candidate_edges.push((p1, p2));
                }
            }
        }
        

        
        // Now remove shared edges (duplicates in reverse order usually)
        // If (A,B) exists and (B,A) exists, they cancel out (internal edge).
        // The remaining edges form the outline.
        
        let mut final_edges = Vec::new();
        let mut i = 0;
        while i < candidate_edges.len() {
            let e1 = candidate_edges[i];
            let mut shared = false;
            let mut j = 0;
            while j < candidate_edges.len() {
                if i != j {
                    let e2 = candidate_edges[j];
                    // Check if e2 is the reverse of e1 (approximate equality)
                    if (e1.0 - e2.1).length() < 0.001 && (e1.1 - e2.0).length() < 0.001 {
                        shared = true;
                        break;
                    }
                }
                j += 1;
            }
            
            if !shared {
                final_edges.push(e1);
            }
            i += 1;
        }
        
        final_edges
    }

    /// Extrude a silhouette edge to form a shadow volume quad
    /// Returns 4 points defining the quad (v1, v2, v2_extruded, v1_extruded)
    pub fn extrude_shadow_volume(
        edge: (Point3D, Point3D),
        light_pos: Point3D,
        length: f32,
    ) -> [Point3D; 4] {
        let (v1, v2) = edge;
        
        let dir1 = (v1 - light_pos).normalize();
        let dir2 = (v2 - light_pos).normalize();
        
        let v1_ext = v1 + dir1 * length;
        let v2_ext = v2 + dir2 * length;
        
        [v1, v2, v2_ext, v1_ext]
    }
    
    /// Generate Penumbra Wedge (for soft shadows)
    /// This requires a light radius.
    /// Returns a polygon representing the penumbra fade region.
    pub fn generate_penumbra_wedge(
        edge: (Point3D, Point3D),
        light_pos: Point3D,
        light_radius: f32,
        length: f32,
    ) -> Vec<Point3D> {
        // Simplified Penumbra:
        // Extrude the edge from (light_pos + radius) and (light_pos - radius)
        // This creates a wedge.
        // For a full implementation, we need the edge normal to know which way is "out".
        // Placeholder implementation returning a simple quad for now.
        Self::extrude_shadow_volume(edge, light_pos, length).to_vec()
    }
}
