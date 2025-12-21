use crate::geometry3d::Point3D;

pub struct Reflector;

impl Reflector {
    /// Reflect a point across a plane defined by a point on the plane and a normal
    pub fn reflect_point(point: Point3D, plane_point: Point3D, plane_normal: Point3D) -> Point3D {
        let normal = plane_normal.normalize();
        let v = point - plane_point; // Vector from plane to point
        let dist = v.dot(&normal);   // Perpendicular distance to plane
        point - normal * (2.0 * dist)
    }

    /// Reflect a vector (direction) across a plane normal
    pub fn reflect_vector(vector: Point3D, plane_normal: Point3D) -> Point3D {
        let normal = plane_normal.normalize();
        vector - normal * (2.0 * vector.dot(&normal))
    }
}

/// A simple 3D polygon clipper (Sutherland-Hodgman)
/// Clips a subject polygon against a convex clip polygon (the mirror face)
/// Note: This assumes all points are coplanar (projected to screen space or on the mirror plane)
/// For 3D mirror reflections, we usually clip in Screen Space (2D) after projection, 
/// OR we clip in 3D against the prism defined by the mirror edge and the view point.
/// For RUGID's SVG approach, Screen Space clipping (2D) is often sufficient and easier.
pub struct Clipper;

impl Clipper {
    /// Clip a 2D polygon against a convex 2D clipping polygon
    pub fn clip_polygon(subject: &[(f32, f32)], clipper: &[(f32, f32)]) -> Vec<(f32, f32)> {
        let mut output = subject.to_vec();
        
        // Iterate over each edge of the clipper
        for i in 0..clipper.len() {
            let c1 = clipper[i];
            let c2 = clipper[(i + 1) % clipper.len()];
            
            let input = output.clone();
            output.clear();
            
            if input.is_empty() { break; }
            
            let mut s = input[input.len() - 1];
            
            for &e in &input {
                if Self::is_inside(e, c1, c2) {
                    if !Self::is_inside(s, c1, c2) {
                        output.push(Self::intersection(s, e, c1, c2));
                    }
                    output.push(e);
                } else if Self::is_inside(s, c1, c2) {
                    output.push(Self::intersection(s, e, c1, c2));
                }
                s = e;
            }
        }
        
        output
    }
    
    // Check if point p is inside the edge defined by c1->c2 (assuming clockwise winding?)
    // Actually, let's assume standard mathematical convention (counter-clockwise is positive area).
    // "Inside" means to the left of the line? Or right?
    // Let's assume standard screen coordinates: X right, Y down.
    fn is_inside(p: (f32, f32), c1: (f32, f32), c2: (f32, f32)) -> bool {
        // Cross product (c2-c1) x (p-c1)
        let val = (c2.0 - c1.0) * (p.1 - c1.1) - (c2.1 - c1.1) * (p.0 - c1.0);
        // If winding is consistent, sign determines side.
        // Assuming convex hull is ordered such that "inside" is always on one side.
        // For screen space (Y down), let's say "right" side is inside if clockwise.
        val >= 0.0 
    }
    
    fn intersection(s: (f32, f32), e: (f32, f32), c1: (f32, f32), c2: (f32, f32)) -> (f32, f32) {
        let dc = (c1.0 - c2.0, c1.1 - c2.1);
        let dp = (s.0 - e.0, s.1 - e.1);
        let n1 = c1.0 * c2.1 - c1.1 * c2.0;
        let n2 = s.0 * e.1 - s.1 * e.0;
        let n3 = 1.0 / (dc.0 * dp.1 - dc.1 * dp.0);
        
        (
            (n1 * dp.0 - n2 * dc.0) * n3,
            (n1 * dp.1 - n2 * dc.1) * n3
        )
    }
}
