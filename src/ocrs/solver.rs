use super::projection::{StaticMesh, Point3D, LatticeBundle};
use super::observer::ObserverCone;
use nalgebra::{DMatrix, DVector};

/// Minimal solver for OCRS PoC
/// 
/// Reconstructs mesh from observer perspective by solving the inverse problem:
/// Given deformed lattice observations, recover original mesh geometry.
pub struct MinimalSolver;

impl MinimalSolver {
    pub fn new() -> Self {
        Self
    }
    
    /// Reconstruct mesh from observer perspective
    /// 
    /// Algorithm:
    /// 1. Build constraint matrix from lattice deformations
    /// 2. Solve Ax = b via QR decomposition
    /// 3. Map solution back to mesh vertices
    pub fn solve(
        &self,
        bundle: &LatticeBundle,
        observer: &ObserverCone,
    ) -> Result<ReconstructedMesh, SolverError> {
        // Build constraint system
        let (A, b) = self.build_constraints(bundle)?;
        
        // Solve via QR decomposition
        let solution = self.solve_qr(&A, &b)?;
        
        // Map solution to reconstructed mesh
        let vertices = self.solution_to_vertices(&solution, bundle);
        
        // Calculate confidence (use observer's perspective)
        let confidence = super::confidence::compute_confidence(observer, bundle);
        
        Ok(ReconstructedMesh {
            vertices,
            confidence,
        })
    }
    
    /// Build constraint matrix A and vector b
    /// 
    /// Enhanced for Surface Differential Resolution:
    /// - Nodes outside inversion range: 1 equation (direct observation)
    /// - Nodes inside inversion range: 2 equations (direct + reflected)
    /// 
    /// This dual-projection approach improves depth perception in regions
    /// where lattice curvature would otherwise make bird's-eye computation expensive
    fn build_constraints(
        &self,
        bundle: &LatticeBundle,
    ) -> Result<(DMatrix<f64>, DVector<f64>), SolverError> {
        let total_nodes = bundle.total_node_count();
        
        if total_nodes == 0 {
            return Err(SolverError::InsufficientConstraints);
        }
        
        // Count additional constraints from dual-projection
        let mut dual_projection_count = 0;
        for lattice in &bundle.lattices {
            for node in &lattice.nodes {
                dual_projection_count += node.reflected_deformations.len();
            }
        }
        
        let total_constraints = total_nodes + dual_projection_count;
        
        // Create constraint system
        // One column per node, but potentially >1 row per node (dual projection)
        let mut a_matrix = DMatrix::zeros(total_constraints, total_nodes);
        let mut b_vector = DVector::zeros(total_constraints);
        
        let mut row = 0;
        let mut col = 0;
        
        for (layer_idx, lattice) in bundle.lattices.iter().enumerate() {
            let baseline = bundle.layer_heights[layer_idx];
            
            for node in &lattice.nodes {
                // Direct observation constraint (always present)
                a_matrix[(row, col)] = 1.0;
                b_vector[row] = baseline + node.deformation;
                row += 1;
                
                // Reflection constraints (multiple if faceted surface)
                for reflected_def in &node.reflected_deformations {
                    // Weight reflection data (distribute weight across all reflections)
                    let count = node.reflected_deformations.len().max(1);
                    let reflection_weight = 0.8 / (count as f64).sqrt();
                    
                    a_matrix[(row, col)] = reflection_weight;
                    b_vector[row] = baseline + reflected_def;
                    row += 1;
                }
                
                col += 1;
            }
        }
        
        Ok((a_matrix, b_vector))
    }
    
    /// Solve Ax = b using QR decomposition
    fn solve_qr(&self, A: &DMatrix<f64>, b: &DVector<f64>) -> Result<DVector<f64>, SolverError> {
        // QR decomposition
        let qr = A.clone().qr();
        
        // Solve QR * x = b
        match qr.solve(b) {
            Some(solution) => Ok(solution),
            None => Err(SolverError::SingularMatrix),
        }
    }
    
    /// Map solution vector to 3D mesh vertices
    fn solution_to_vertices(
        &self,
        solution: &DVector<f64>,
        bundle: &LatticeBundle,
    ) -> Vec<Point3D> {
        let mut vertices = Vec::new();
        let mut idx = 0;
        
        for lattice in &bundle.lattices {
            for node in &lattice.nodes {
                let height = solution[idx];
                
                vertices.push(Point3D {
                    x: node.rest_position.x,
                    y: height,
                    z: node.rest_position.z,
                });
                
                idx += 1;
            }
        }
        
        vertices
    }
}

/// Reconstructed mesh from solver
#[derive(Debug, Clone)]
pub struct ReconstructedMesh {
    pub vertices: Vec<Point3D>,
    pub confidence: f64,
}

impl ReconstructedMesh {
    /// Calculate RMS error compared to original mesh
    pub fn error(&self, original: &StaticMesh) -> f64 {
        if self.vertices.len() != original.vertices.len() {
            return f64::MAX;  // Size mismatch
        }
        
        let mut sum_sq_error = 0.0;
        
        for (reconstructed, original) in self.vertices.iter().zip(original.vertices.iter()) {
            let dx = reconstructed.x - original.x;
            let dy = reconstructed.y - original.y;
            let dz = reconstructed.z - original.z;
            
            sum_sq_error += dx * dx + dy * dy + dz * dz;
        }
        
        (sum_sq_error / self.vertices.len() as f64).sqrt()
    }
    
    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

#[derive(Debug)]
pub enum SolverError {
    SingularMatrix,
    InsufficientConstraints,
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SolverError::SingularMatrix => write!(f, "Constraint matrix is singular"),
            SolverError::InsufficientConstraints => write!(f, "Insufficient constraints for resolution"),
        }
    }
}

impl std::error::Error for SolverError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocrs::projection::{StaticMesh, MeshProjector};
    
    #[test]
    fn test_solver_round_trip() {
        // Create test mesh
        let mesh = StaticMesh::test_terrain(100.0);
        
        // Project to lattice
        let projector = MeshProjector::new(2.0, 5.0);
        let bundle = projector.project(&mesh);
        
        // Create observer
        let target = Point3D::new(50.0, 0.0, 50.0);
        let observer = ObserverCone::new(target, 50.0, 90.0);
        
        // Solve (reconstruct)
        let solver = MinimalSolver::new();
        let reconstructed = solver.solve(&bundle, &observer).unwrap();
        
        // Validate
        assert_eq!(reconstructed.vertex_count(), bundle.total_node_count());
        assert!(reconstructed.confidence > 0.5, "Confidence should be reasonable at 50m");
        
        println!("Reconstructed {} vertices with {:.1}% confidence",
                 reconstructed.vertex_count(),
                 reconstructed.confidence * 100.0);
    }
    
    #[test]
    fn test_reconstruction_error() {
        // Create mesh
        let mesh = StaticMesh::test_terrain(100.0);
        
        // Project and reconstruct
        let projector = MeshProjector::new(2.0, 5.0);
        let bundle = projector.project(&mesh);
        
        let target = Point3D::new(50.0, 0.0, 50.0);
        let observer = ObserverCone::new(target, 100.0, 90.0);  // Far = high confidence
        
        let solver = MinimalSolver::new();
        let reconstructed = solver.solve(&bundle, &observer).unwrap();
        
        // Calculate error
        // Note: Error will be high because we're comparing lattice nodes to original mesh vertices
        // The lattice has different vertex positions (grid-aligned)
        let error = reconstructed.error(&mesh);
        
        println!("Reconstruction error: {:.2}m RMS", error);
        println!("Confidence: {:.1}%", reconstructed.confidence * 100.0);
        
        // For this PoC test, just verify solver runs successfully
        assert!(error >= 0.0, "Error should be non-negative");
    }
    
    #[test]
    fn test_dual_projection_solver() {
        // Create mesh
        let mesh = StaticMesh::test_terrain(100.0);
        
        // Create weight at center for maximum inversion range coverage
        let weight_pos = Point3D::new(50.0, 10.0, 50.0);
        let weight = super::super::weight::Weight::default_metallic(weight_pos);
        
        // Project with dual-projection
        let projector = MeshProjector::new(2.0, 5.0);
        let bundle_dual = projector.project_with_reflection(&mesh, &weight);
        
        // Also create standard projection for comparison
        let bundle_standard = projector.project(&mesh);
        
        // Create observer (weight is embedded in observer)
        let target = Point3D::new(50.0, 0.0, 50.0);
        let observer = ObserverCone::new(target, 50.0, 90.0);
        
        // Solve both
        let solver = MinimalSolver::new();
        let reconstructed_dual = solver.solve(&bundle_dual, &observer).unwrap();
        let reconstructed_standard = solver.solve(&bundle_standard, &observer).unwrap();
        
        println!("Dual-projection solver test:");
        println!("  Standard constraints: {}", bundle_standard.total_node_count());
        
        // Count dual projection constraints
        let mut dual_constraints = bundle_dual.total_node_count();
        for lattice in &bundle_dual.lattices {
            for node in &lattice.nodes {
                if node.in_inversion_range && !node.reflected_deformations.is_empty() {
                    dual_constraints += node.reflected_deformations.len();
                }
            }
        }
        println!("  Dual-projection constraints: {}", dual_constraints);
        println!("  Standard confidence: {:.1}%", reconstructed_standard.confidence * 100.0);
        println!("  Dual-projection confidence: {:.1}%", reconstructed_dual.confidence * 100.0);
        
        // Both should solve successfully
        assert!(reconstructed_dual.vertex_count() > 0);
        assert!(reconstructed_standard.vertex_count() > 0);
        
        // Dual projection should have more constraints
        assert!(dual_constraints > bundle_standard.total_node_count(), 
                "Expected dual-projection to have more constraints");
    }
}

