use super::projection::{Point3D, Vector3D};

/// Surface geometry type for the metallic weight
#[derive(Debug, Clone)]
pub enum SurfaceGeometry {
    /// Smooth spherical surface (uniform reflections)
    Smooth,
    
    /// Faceted geodesic surface (multiple reflections per point)
    Faceted {
        subdivisions: usize,
        facets: Vec<Facet>,
    },
}

/// A single facet (triangular face) on the weight surface
#[derive(Debug, Clone)]
pub struct Facet {
    /// Three vertices defining the triangle
    pub vertices: [Point3D; 3],
    
    /// Outward-facing normal vector (pre-computed)
    pub normal: Vector3D,
    
    /// Center point of the triangle
    pub center: Point3D,
    
    /// Area of the facet (for information density calculations)
    pub area: f64,
}

impl Facet {
    /// Create a new facet from three vertices
    pub fn new(v0: Point3D, v1: Point3D, v2: Point3D) -> Self {
        let vertices = [v0, v1, v2];
        
        // Calculate center
        let center = Point3D {
            x: (v0.x + v1.x + v2.x) / 3.0,
            y: (v0.y + v1.y + v2.y) / 3.0,
            z: (v0.z + v1.z + v2.z) / 3.0,
        };
        
        // Calculate normal (cross product of two edges)
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = Vector3D {
            x: edge1.y * edge2.z - edge1.z * edge2.y,
            y: edge1.z * edge2.x - edge1.x * edge2.z,
            z: edge1.x * edge2.y - edge1.y * edge2.x,
        }.normalize();
        
        // Calculate area (half the cross product magnitude)
        let area = (edge1.y * edge2.z - edge1.z * edge2.y).powi(2)
            + (edge1.z * edge2.x - edge1.x * edge2.z).powi(2)
            + (edge1.x * edge2.y - edge1.y * edge2.x).powi(2);
        let area = area.sqrt() * 0.5;
        
        Self {
            vertices,
            normal,
            center,
            area,
        }
    }
    
    /// Reflect a point using this facet's surface
    /// Returns None if facet is back-facing or reflection is invalid
    pub fn reflect_point(&self, target: Point3D, distance_scale: f64) -> Option<Point3D> {
        // Check if facet is visible from target (front-facing)
        let to_target = (target - self.center).normalize();
        let facing = to_target.x * self.normal.x + to_target.y * self.normal.y + to_target.z * self.normal.z;
        
        if facing <= 0.01 {  // Small threshold for numerical stability
            return None;  // Back-facing
        }
        
        // Incident ray (from target to facet center)
        let incident = (self.center - target).normalize();
        
        // Reflect using facet normal: R = I - 2(I·N)N
        let dot = incident.x * self.normal.x + incident.y * self.normal.y + incident.z * self.normal.z;
        let reflected = Vector3D {
            x: incident.x - 2.0 * dot * self.normal.x,
            y: incident.y - 2.0 * dot * self.normal.y,
            z: incident.z - 2.0 * dot * self.normal.z,
        };
        
        // Project reflection outward
        Some(self.center + reflected * distance_scale)
    }
}

/// Metallic weight ball that deforms the lattice into a cone shape
/// 
/// The weight acts as both:
/// 1. A gravitational source that creates lattice deformation
/// 2. A reflective surface for dual-projection resolution
#[derive(Debug, Clone)]
pub struct Weight {
    /// Center position in 3D space
    pub position: Point3D,
    
    /// Radius of the spherical weight (can change with mass/density)
    pub radius: f64,
    
    /// Physical mass affecting lattice deformation magnitude
    pub mass: f64,
    
    /// Reflectivity: 0.0 = matte, 1.0 = perfect mirror
    /// Affects the quality of reflection-based data
    pub reflectivity: f64,
    
    /// Surface geometry (smooth or faceted)
    pub surface: SurfaceGeometry,
}

impl Weight {
    /// Create a new weight with specified properties
    pub fn new(position: Point3D, radius: f64, mass: f64, reflectivity: f64) -> Self {
        Self {
            position,
            radius,
            mass,
            reflectivity: reflectivity.clamp(0.0, 1.0),
            surface: SurfaceGeometry::Smooth,
        }
    }
    
    /// Create a default metallic ball suitable for testing
    /// - 5m radius sphere
    /// - 1000kg mass
    /// - 0.95 reflectivity (highly polished metal)
    pub fn default_metallic(position: Point3D) -> Self {
        Self::new(position, 5.0, 1000.0, 0.95)
    }
    
    /// Calculate reflection of a point in the metallic surface
    /// 
    /// For smooth surfaces: returns single reflection
    /// For faceted surfaces: returns multiple reflections from visible facets
    /// 
    /// Returns empty vec if the point is inside the sphere
    pub fn reflect_point(&self, target: Point3D) -> Vec<Point3D> {
        // Vector from weight center to target
        let to_target = target - self.position;
        let distance = to_target.magnitude();
        
        // Point must be outside sphere to reflect
        if distance <= self.radius {
            return vec![];
        }
        
        match &self.surface {
            SurfaceGeometry::Smooth => {
                // Smooth sphere: single reflection
                self.reflect_point_smooth(target, distance).into_iter().collect()
            },
            SurfaceGeometry::Faceted { facets, .. } => {
                // Faceted surface: multiple reflections from visible facets
                facets.iter()
                    .filter_map(|facet| facet.reflect_point(target, distance))
                    .collect()
            },
        }
    }
    
    /// Compute lattice deformation at a given point based on weight's influence
    /// 
    /// Models a simple inverse-square-like deformation where:
    /// - Closer points experience more deformation
    /// - Mass increases deformation magnitude
    /// - Forms a cone shape around the weight
    pub fn deformation_at(&self, point: Point3D) -> f64 {
        let to_point = point - self.position;
        let distance = to_point.magnitude();
        
        if distance < self.radius {
            // Inside the weight: maximum deformation
            return -self.mass / (self.radius * self.radius);
        }
        
        // Inverse square deformation: F ∝ m/r²
        // Negative because weight "pulls down" the lattice
        -self.mass / (distance * distance)
    }
    
    /// Calculate the radius of the inversion range
    /// 
    /// The inversion range is where the lattice curvature is so severe that
    /// spatial relationships become ambiguous (x>y becomes y>x).
    /// 
    /// This occurs within approximately 2-3 radii from the weight center,
    /// where deformation gradient exceeds 1.0
    pub fn inversion_range(&self) -> f64 {
        // Empirical relationship: inversion occurs at ~2.5 radii
        // where the deformation gradient ≈ 1.0
        self.radius * 2.5
    }
    
    /// Check if a point is within the inversion range
    pub fn is_in_inversion_range(&self, point: Point3D) -> bool {
        let distance = (point - self.position).magnitude();
        distance <= self.inversion_range()
    }
    
    /// Calculate "cone-ness" - how severely curved the lattice is at this point
    /// Returns value in range [0.0, 1.0]:
    /// - 0.0 = flat (far from weight)
    /// - 1.0 = maximum curvature (at sphere surface)
    pub fn cone_severity(&self, point: Point3D) -> f64 {
        let distance = (point - self.position).magnitude();
        
        if distance <= self.radius {
            return 1.0;
        }
        
        let inversion_radius = self.inversion_range();
        
        if distance >= inversion_radius {
            return 0.0;
        }
        
        // Linear falloff from sphere surface to inversion range
        1.0 - ((distance - self.radius) / (inversion_radius - self.radius))
    }
    
    /// Create a faceted weight ball for high information density
    /// 
    /// Subdivisions:
    /// - 0: Icosahedron (20 facets)
    /// - 1: 80 facets (recommended for balanced performance)
    /// - 2: 320 facets (high density, more compute)
    /// - 3: 1280 facets (very high density)
    pub fn faceted(position: Point3D, radius: f64, subdivisions: usize) -> Self {
        let facets = generate_geodesic_facets(position, radius, subdivisions);
        
        Self {
            position,
            radius,
            mass: 1000.0,
            reflectivity: 0.95,
            surface: SurfaceGeometry::Faceted { subdivisions, facets },
        }
    }
    
    /// Get the number of facets (1 for smooth sphere)
    pub fn facet_count(&self) -> usize {
        match &self.surface {
            SurfaceGeometry::Smooth => 1,
            SurfaceGeometry::Faceted { facets, .. } => facets.len(),
        }
    }
    
    /// Calculate information density (reflections per cm²)
    pub fn information_density(&self) -> f64 {
        let surface_area_cm2 = 4.0 * std::f64::consts::PI * (self.radius * 100.0).powi(2);
        self.facet_count() as f64 / surface_area_cm2
    }
    
    /// Single reflection for smooth sphere (internal helper)
    fn reflect_point_smooth(&self, target: Point3D, distance: f64) -> Option<Point3D> {
        // Find intersection point on sphere surface
        let direction_to_center = (self.position - target).normalize();
        let intersection = self.position + direction_to_center * self.radius;
        
        // Surface normal at intersection (points outward from sphere center)
        let normal = (intersection - self.position).normalize();
        
        // Incident ray (from target to intersection)
        let incident = (intersection - target).normalize();
        
        // Reflected ray: R = I - 2(I·N)N
        let dot_product = incident.x * normal.x + incident.y * normal.y + incident.z * normal.z;
        let reflected_dir = Vector3D {
            x: incident.x - 2.0 * dot_product * normal.x,
            y: incident.y - 2.0 * dot_product * normal.y,
            z: incident.z - 2.0 * dot_product * normal.z,
        };
        
        // Project reflected ray
        let reflected_point = intersection + reflected_dir * distance;
        Some(reflected_point)
    }
}

/// Generate geodesic sphere facets using icosahedron subdivision
pub fn generate_geodesic_facets(center: Point3D, radius: f64, subdivisions: usize) -> Vec<Facet> {
    // Start with icosahedron vertices
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;  // Golden ratio
    let a = 1.0;
    let b = 1.0 / phi;
    
    // Normalize to unit sphere
    let len = (a * a + b * b).sqrt();
    let a = a / len;
    let b = b / len;
    
    // 12 vertices of icosahedron
    let vertices = vec![
        Vector3D::new(0.0, b, -a), Vector3D::new(b, a, 0.0), Vector3D::new(-b, a, 0.0),
        Vector3D::new(0.0, b, a), Vector3D::new(0.0, -b, a), Vector3D::new(-a, 0.0, b),
        Vector3D::new(0.0, -b, -a), Vector3D::new(a, 0.0, -b), Vector3D::new(a, 0.0, b),
        Vector3D::new(-a, 0.0, -b), Vector3D::new(b, -a, 0.0), Vector3D::new(-b, -a, 0.0),
    ];
    
    // 20 faces of icosahedron (indices into vertices)
    let mut triangles = vec![
        (0, 1, 2), (3, 2, 1), (3, 4, 5), (3, 8, 4), (0, 6, 7),
        (0, 9, 6), (4, 10, 11), (6, 11, 10), (2, 5, 9), (11, 9, 5),
        (1, 7, 8), (10, 8, 7), (3, 5, 2), (3, 1, 8), (0, 2, 9),
        (0, 7, 1), (6, 9, 11), (6, 10, 7), (4, 11, 5), (4, 8, 10),
    ];
    
    // Subdivide
    let mut vertex_list = vertices;
    for _ in 0..subdivisions {
        let mut new_triangles = Vec::new();
        
        for &(i0,  i1, i2) in &triangles {
            // Get midpoints
            let v0 = vertex_list[i0];
            let v1 = vertex_list[i1];
            let v2 = vertex_list[i2];
            
            let m01 = midpoint_normalized(v0, v1);
            let m12 = midpoint_normalized(v1, v2);
            let m20 = midpoint_normalized(v2, v0);
            
            // Add new vertices
            let i_m01 = add_unique_vertex(&mut vertex_list, m01);
            let i_m12 = add_unique_vertex(&mut vertex_list, m12);
            let i_m20 = add_unique_vertex(&mut vertex_list, m20);
            
            // Create 4 new triangles
            new_triangles.push((i0, i_m01, i_m20));
            new_triangles.push((i1, i_m12, i_m01));
            new_triangles.push((i2, i_m20, i_m12));
            new_triangles.push((i_m01, i_m12, i_m20));
        }
        
        triangles = new_triangles;
    }
    
    // Convert to facets
    triangles.iter().map(|&(i0, i1, i2)| {
        let v0 = center + vertex_list[i0] * radius;
        let v1 = center + vertex_list[i1] * radius;
        let v2 = center + vertex_list[i2] * radius;
        Facet::new(v0, v1, v2)
    }).collect()
}

fn midpoint_normalized(v0: Vector3D, v1: Vector3D) -> Vector3D {
    let mid = Vector3D {
        x: (v0.x + v1.x) / 2.0,
        y: (v0.y + v1.y) / 2.0,
        z: (v0.z + v1.z) / 2.0,
    };
    mid.normalize()
}

fn add_unique_vertex(vertices: &mut Vec<Vector3D>, v: Vector3D) -> usize {
    // Check if vertex already exists (within tolerance)
    for (i, existing) in vertices.iter().enumerate() {
        let dx = existing.x - v.x;
        let dy = existing.y - v.y;
        let dz = existing.z - v.z;
        if (dx * dx + dy * dy + dz * dz).sqrt() < 1e-6 {
            return i;
        }
    }
    
    // Add new vertex
    vertices.push(v);
    vertices.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_weight_creation() {
        let pos = Point3D::new(0.0, 10.0, 0.0);
        let weight = Weight::new(pos, 5.0, 1000.0, 0.95);
        
        assert_eq!(weight.radius, 5.0);
        assert_eq!(weight.mass, 1000.0);
        assert_eq!(weight.reflectivity, 0.95);
    }
    
    #[test]
    fn test_default_metallic() {
        let pos = Point3D::new(50.0, 20.0, 50.0);
        let weight = Weight::default_metallic(pos);
        
        assert_eq!(weight.position, pos);
        assert_eq!(weight.radius, 5.0);
        assert_eq!(weight.mass, 1000.0);
        assert_eq!(weight.reflectivity, 0.95);
    }
    
    #[test]
    fn test_reflection_outside_sphere() {
        let weight = Weight::default_metallic(Point3D::zero());
        let target = Point3D::new(10.0, 0.0, 0.0);  // 10m away on X axis
        
        let reflected = weight.reflect_point(target);
        assert!(!reflected.is_empty(), "Should get reflections from outside sphere");
        
        let reflected_pt = reflected[0];
        // Reflection should not be at the same point
        assert_ne!(reflected_pt, target);
    }
    
    #[test]
    fn test_reflection_inside_sphere() {
        let weight = Weight::default_metallic(Point3D::zero());
        let inside = Point3D::new(2.0, 0.0, 0.0);  // 2m away, inside 5m radius
        
        let reflected = weight.reflect_point(inside);
        assert!(reflected.is_empty(), "Can't reflect from inside sphere");
    }
    
    #[test]
    fn test_deformation_decreases_with_distance() {
        let weight = Weight::default_metallic(Point3D::zero());
        
        let close = Point3D::new(10.0, 0.0, 0.0);
        let far = Point3D::new(50.0, 0.0, 0.0);
        
        let deform_close = weight.deformation_at(close);
        let deform_far = weight.deformation_at(far);
        
        // Both should be negative (pulling down)
        assert!(deform_close < 0.0);
        assert!(deform_far < 0.0);
        
        // Closer point should have more deformation (more negative)
        assert!(deform_close < deform_far);
    }
    
    #[test]
    fn test_inversion_range() {
        let weight = Weight::default_metallic(Point3D::zero());
        let range = weight.inversion_range();
        
        // Should be ~2.5 * radius = 12.5m
        assert!((range - 12.5).abs() < 0.1);
    }
    
    #[test]
    fn test_is_in_inversion_range() {
        let weight = Weight::default_metallic(Point3D::zero());
        
        let close = Point3D::new(10.0, 0.0, 0.0);  // Within inversion range
        let far = Point3D::new(50.0, 0.0, 0.0);    // Outside inversion range
        
        assert!(weight.is_in_inversion_range(close));
        assert!(!weight.is_in_inversion_range(far));
    }
    
    #[test]
    fn test_cone_severity() {
        let weight = Weight::default_metallic(Point3D::zero());
        
        // At surface: maximum severity
        let at_surface = Point3D::new(5.0, 0.0, 0.0);
        assert!((weight.cone_severity(at_surface) - 1.0).abs() < 0.1);
        
        // Inside sphere: maximum severity
        let inside = Point3D::new(2.0, 0.0, 0.0);
        assert_eq!(weight.cone_severity(inside), 1.0);
        
        // At inversion range edge: zero severity
        let edge = Point3D::new(12.5, 0.0, 0.0);
        assert!(weight.cone_severity(edge) < 0.1);
        
        // Far beyond: zero severity
        let far = Point3D::new(100.0, 0.0, 0.0);
        assert_eq!(weight.cone_severity(far), 0.0);
        
        // Midpoint: intermediate severity
        let mid = Point3D::new(8.75, 0.0, 0.0);  // Halfway between surface and edge
        let severity = weight.cone_severity(mid);
        assert!(severity > 0.3 && severity < 0.7);
    }
}
    
    #[test]
    fn test_faceted_weight_creation() {
        let pos = Point3D::new(0.0, 10.0, 0.0);
        let weight = Weight::faceted(pos, 5.0, 1);
        
        assert_eq!(weight.position, pos);
        assert_eq!(weight.radius, 5.0);
        assert_eq!(weight.facet_count(), 80); // 1 subdivision
    }
    
    #[test]
    fn test_multi_reflection() {
        let smooth = Weight::default_metallic(Point3D::zero());
        let faceted = Weight::faceted(Point3D::zero(), 5.0, 2);
        
        let target = Point3D::new(10.0, 5.0, 10.0);
        
        let smooth_reflections = smooth.reflect_point(target);
        let faceted_reflections = faceted.reflect_point(target);
        
        assert_eq!(smooth_reflections.len(), 1);
        assert!(faceted_reflections.len() > 1);
        
        println!("Smooth: {}, Faceted: {}", smooth_reflections.len(), faceted_reflections.len());
    }
