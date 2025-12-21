use crate::geometry::AbsoluteRect;

/// 3D point for OCRS
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

impl std::ops::Sub for Point3D {
    type Output = Vector3D;
    
    fn sub(self, other: Self) -> Vector3D {
        Vector3D {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Add<Vector3D> for Point3D {
    type Output = Point3D;
    
    fn add(self, vec: Vector3D) -> Point3D {
        Point3D {
            x: self.x + vec.x,
            y: self.y + vec.y,
            z: self.z + vec.z,
        }
    }
}

/// 3D vector for OCRS
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        } else {
            *self
        }
    }
}

impl std::ops::Mul<f64> for Vector3D {
    type Output = Vector3D;
    
    fn mul(self, scalar: f64) -> Vector3D {
        Vector3D {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

/// Triangle face (indices into vertex array)
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub v0: usize,
    pub v1: usize,
    pub v2: usize,
}

/// Bounding box for spatial queries
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Point3D,
    pub max: Point3D,
}

impl BoundingBox {
    pub fn from_points(points: &[Point3D]) -> Self {
        let mut min = Point3D::new(f64::MAX, f64::MAX, f64::MAX);
        let mut max = Point3D::new(f64::MIN, f64::MIN, f64::MIN);
        
        for p in points {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }
        
        Self { min, max }
    }
    
    pub fn center(&self) -> Point3D {
        Point3D {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
            z: (self.min.z + self.max.z) / 2.0,
        }
    }
    
    pub fn size(&self) -> Vector3D {
        Vector3D {
            x: self.max.x - self.min.x,
            y: self.max.y - self.min.y,
            z: self.max.z - self.min.z,
        }
    }
}

/// Static mesh representation for OCRS PoC
pub struct StaticMesh {
    pub vertices: Vec<Point3D>,
    pub faces: Vec<Triangle>,
    pub bounds: BoundingBox,
}

impl StaticMesh {
    /// Create simple test terrain (100m² with procedural height)
    pub fn test_terrain(size: f64) -> Self {
        let resolution = 50; // 50x50 grid = 2,500 vertices
        let mut vertices = Vec::new();
        
        // Generate height map vertices
        for y in 0..resolution {
            for x in 0..resolution {
                let px = (x as f64 / (resolution - 1) as f64) * size;
                let pz = (y as f64 / (resolution - 1) as f64) * size;
                
                // Procedural height function (rolling hills)
                let py = ((px / 20.0).sin() * (pz / 15.0).cos()) * 5.0
                       + ((px / 10.0 + pz / 8.0).sin()) * 2.0;
                
                vertices.push(Point3D::new(px, py, pz));
            }
        }
        
        // Generate triangle faces
        let mut faces = Vec::new();
        for y in 0..(resolution - 1) {
            for x in 0..(resolution - 1) {
                let idx = y * resolution + x;
                
                // Two triangles per quad
                faces.push(Triangle {
                    v0: idx,
                    v1: idx + resolution,
                    v2: idx + 1,
                });
                faces.push(Triangle {
                    v1: idx + resolution,
                    v2: idx + resolution + 1,
                    v0: idx + 1,
                });
            }
        }
        
        let bounds = BoundingBox::from_points(&vertices);
        
        Self { vertices, faces, bounds }
    }
    
    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
    
    /// Get face count
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }
}

/// Lattice node (simplified for PoC)
#[derive(Debug, Clone)]
pub struct LatticeNode {
    pub position: Point3D,
    pub rest_position: Point3D,
    pub deformation: f64,  // Height displacement from direct observation
    pub reflected_deformations: Vec<f64>,  // Heights from reflections (if in inversion range)
    pub in_inversion_range: bool,  // Whether dual-projection should be used
}

/// 2D lattice layer (simplified for PoC)
#[derive(Debug, Clone)]
pub struct Lattice {
    pub nodes: Vec<LatticeNode>,
    pub width: usize,   // Grid width (X)
    pub height: usize,  // Grid height (Z)
}

impl Lattice {
    /// Create lattice from height map
    pub fn from_height_map(width: usize, height: usize, height_fn: impl Fn(f64, f64) -> f64) -> Self {
        let mut nodes = Vec::new();
        
        for z in 0..height {
            for x in 0..width {
                let px = x as f64;
                let pz = z as f64;
                let py = height_fn(px, pz);
                
                let rest = Point3D::new(px, 0.0, pz);
                let position = Point3D::new(px, py, pz);
                let deformation = py;
                
                nodes.push(LatticeNode {
                    position,
                    rest_position: rest,
                    deformation,
                    reflected_deformations: Vec::new(),  // Default: no reflection data
                    in_inversion_range: false,    // Default: outside inversion range
                });
            }
        }
        
        Self { nodes, width, height }
    }
    
    /// Get node at grid position
    pub fn node_at(&self, x: usize, z: usize) -> Option<&LatticeNode> {
        if x < self.width && z < self.height {
            Some(&self.nodes[z * self.width + x])
        } else {
            None
        }
    }
    
    /// Total node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Bundle of lattice layers with color encoding (simplified)
#[derive(Debug, Clone)]
pub struct LatticeBundle {
    pub lattices: Vec<Lattice>,
    pub layer_heights: Vec<f64>,  // Y-coordinate for each layer
}

impl LatticeBundle {
    pub fn total_node_count(&self) -> usize {
        self.lattices.iter().map(|l| l.node_count()).sum()
    }
}

/// Project 3D mesh onto 2D lattice layers
pub struct MeshProjector {
    pub grid_resolution: f64,   // Meters per grid cell (e.g., 2.0)
    pub layer_spacing: f64,     // Meters between layers (e.g., 5.0)
}

impl MeshProjector {
    pub fn new(grid_resolution: f64, layer_spacing: f64) -> Self {
        Self {
            grid_resolution,
            layer_spacing,
        }
    }
    
    /// Project mesh into lattice bundle
    pub fn project(&self, mesh: &StaticMesh) -> LatticeBundle {
        // 1. Determine layer count based on mesh height
        let mesh_height = mesh.bounds.max.y - mesh.bounds.min.y;
        let num_layers = ((mesh_height / self.layer_spacing).ceil() as usize).max(1);
        
        // 2. Calculate lattice grid dimensions
        let width = ((mesh.bounds.max.x - mesh.bounds.min.x) / self.grid_resolution).ceil() as usize;
        let height = ((mesh.bounds.max.z - mesh.bounds.min.z) / self.grid_resolution).ceil() as usize;
        
        // 3. Create lattices for each height band
        let mut lattices = Vec::new();
        let mut layer_heights = Vec::new();
        
        for layer_idx in 0..num_layers {
            let layer_y = mesh.bounds.min.y + (layer_idx as f64 * self.layer_spacing);
            layer_heights.push(layer_y);
            
            // Sample mesh at this height
            let lattice = self.create_lattice_at_height(mesh, layer_y, width, height);
            lattices.push(lattice);
        }
        
        LatticeBundle {
            lattices,
            layer_heights,
        }
    }
    
    /// Project mesh with dual-projection resolution using metallic weight
    /// 
    /// Enhances the lattice bundle with reflection data for nodes in the inversion range
    pub fn project_with_reflection(&self, mesh: &StaticMesh, weight: &super::weight::Weight) -> LatticeBundle {
        // Start with standard projection
        let mut bundle = self.project(mesh);
        
        // Enhance nodes in inversion range with reflection data
        for (layer_idx, lattice) in bundle.lattices.iter_mut().enumerate() {
            let layer_y = bundle.layer_heights[layer_idx];
            
            for node in &mut lattice.nodes {
                let node_3d = Point3D {
                    x: node.rest_position.x + mesh.bounds.min.x,
                    y: layer_y + node.deformation,
                    z: node.rest_position.z + mesh.bounds.min.z,
                };
                
                // Check if node is in inversion range
                if weight.is_in_inversion_range(node_3d) {
                    node.in_inversion_range = true;
                    
                    // Calculate reflected views of this point (may be multiple for faceted surfaces)
                    let reflected_pts = weight.reflect_point(node_3d);
                    
                    node.reflected_deformations = reflected_pts.iter()
                        .map(|pt| self.sample_mesh_height(mesh, pt.x, pt.z, layer_y))
                        .collect();
                }
            }
        }
        
        bundle
    }
    
    fn create_lattice_at_height(
        &self,
        mesh: &StaticMesh,
        target_y: f64,
        width: usize,
        height: usize,
    ) -> Lattice {
        // Create height map by sampling mesh
        let height_fn = |grid_x: f64, grid_z: f64| {
            let world_x = mesh.bounds.min.x + (grid_x * self.grid_resolution);
            let world_z = mesh.bounds.min.z + (grid_z * self.grid_resolution);
            
            // Find mesh height at this XZ position (simplified: use nearest vertex)
            self.sample_mesh_height(mesh, world_x, world_z, target_y)
        };
        
        Lattice::from_height_map(width, height, height_fn)
    }
    
    fn sample_mesh_height(&self, mesh: &StaticMesh, x: f64, z: f64, layer_y: f64) -> f64 {
        // Simplified sampling: find closest vertex in XZ plane
        let mut min_dist_sq = f64::MAX;
        let mut sampled_y = layer_y;
        
        for vertex in &mesh.vertices {
            // Only consider vertices near this layer
            if (vertex.y - layer_y).abs() < self.layer_spacing {
                let dx = vertex.x - x;
                let dz = vertex.z - z;
                let dist_sq = dx * dx + dz * dz;
                
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    sampled_y = vertex.y;
                }
            }
        }
        
        // Return height relative to layer
        sampled_y - layer_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_static_mesh_creation() {
        let mesh = StaticMesh::test_terrain(100.0);
        
        assert_eq!(mesh.vertex_count(), 50 * 50);
        assert_eq!(mesh.face_count(), (50 - 1) * (50 - 1) * 2);
        
        // Bounds should encompass 100x100 area
        assert!((mesh.bounds.max.x - mesh.bounds.min.x - 100.0).abs() < 0.1);
        assert!((mesh.bounds.max.z - mesh.bounds.min.z - 100.0).abs() < 0.1);
    }
    
    #[test]
    fn test_mesh_projection() {
        let mesh = StaticMesh::test_terrain(100.0);
        let projector = MeshProjector::new(2.0, 5.0);  // 2m cells, 5m layer spacing
        
        let bundle = projector.project(&mesh);
        
        // Should have multiple layers for the terrain height
        assert!(bundle.lattices.len() >= 1);
        assert!(bundle.lattices.len() <= 5);  // Terrain is ~10m tall
        
        // Each lattice should have reasonable dimensions
        for lattice in &bundle.lattices {
            assert!(lattice.width >= 40);  // 100m / 2m ≈ 50
            assert!(lattice.height >= 40);
            assert!(lattice.node_count() > 0);
        }
        
        println!("Projected {} layers with {} total nodes",
                 bundle.lattices.len(),
                 bundle.total_node_count());
    }
    
    #[test]
    fn test_lattice_node_access() {
        let lattice = Lattice::from_height_map(10, 10, |x, z| x + z);
        
        assert_eq!(lattice.node_count(), 100);
        
        let node = lattice.node_at(5, 5).unwrap();
        assert!((node.deformation - 10.0).abs() < 1e-10);  // 5 + 5 = 10
        
        assert!(lattice.node_at(100, 100).is_none());  // Out of bounds
    }
    
    #[test]
    fn test_dual_projection_with_reflection() {
        use super::super::weight::Weight;
        
        let mesh = StaticMesh::test_terrain(100.0);
        let projector = MeshProjector::new(2.0, 5.0);
        
        // Create weight at center of terrain
        let weight_pos = Point3D::new(50.0, 10.0, 50.0);
        let weight = Weight::default_metallic(weight_pos);
        
        // Project with dual-projection
        let bundle = projector.project_with_reflection(&mesh, &weight);
        
        // Count nodes with reflection data
        let mut nodes_in_inversion = 0;
        let mut nodes_with_reflection = 0;
        
        for lattice in &bundle.lattices {
            for node in &lattice.nodes {
                if node.in_inversion_range {
                    nodes_in_inversion += 1;
                    
                    if !node.reflected_deformations.is_empty() {
                        nodes_with_reflection += 1;
                    }
                }
            }
        }
        
        println!("Dual-projection stats:");
        println!("  Nodes in inversion range: {}", nodes_in_inversion);
        println!("  Nodes with reflection data: {}", nodes_with_reflection);
        println!("  Total nodes: {}", bundle.total_node_count());
        
        // We expect some nodes to be in inversion range (within ~12.5m of weight)
        assert!(nodes_in_inversion > 0, "Expected some nodes in inversion range");
        
        // Most/all nodes in inversion range should have reflection data
        assert!(nodes_with_reflection > 0, "Expected reflection data for inversion range nodes");
        
        // Reflection data should account for significant portion of inversion range
        let reflection_ratio = nodes_with_reflection as f64 / nodes_in_inversion as f64;
        assert!(reflection_ratio > 0.5, "Expected >50% of inversion nodes to have reflection data");
    }
}

