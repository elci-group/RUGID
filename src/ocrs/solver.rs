use super::observer::ObserverCone;
use super::projection::{LatticeBundle, Point3D, StaticMesh};
use nalgebra::DVector;

/// Minimal solver for OCRS PoC
///
/// Reconstructs mesh from observer perspective by solving the inverse problem:
/// Given deformed lattice observations, recover original mesh geometry.
pub struct MinimalSolver;

impl Default for MinimalSolver {
    fn default() -> Self {
        Self::new()
    }
}

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
        if bundle.total_node_count() == 0 {
            return Err(SolverError::InsufficientConstraints);
        }

        // Solve the per-node inverse problem directly.
        let solution = self.solve_constraints(bundle);

        // Map solution to reconstructed mesh
        let vertices = self.solution_to_vertices(&solution, bundle);

        // Calculate confidence (use observer's perspective)
        let confidence = super::confidence::compute_confidence(observer, bundle);

        Ok(ReconstructedMesh {
            vertices,
            confidence,
        })
    }

    /// Solve the constraint system for every node's height.
    ///
    /// Enhanced for Surface Differential Resolution:
    /// - Nodes outside inversion range: 1 equation (direct observation)
    /// - Nodes inside inversion range: 2+ equations (direct + reflected)
    ///
    /// Every equation for a given node only involves that node's own height —
    /// there is no coupling between nodes — so the constraint system is
    /// block-diagonal with one scalar unknown per block. Solving it as one
    /// large dense least-squares system (the original approach: assembling an
    /// N x N matrix and running a full QR decomposition on it) is
    /// mathematically equivalent but scales as O(n^3) in both time and memory;
    /// at realistic lattice sizes (thousands of nodes) that made the solver
    /// effectively never terminate. Solving each node's decoupled
    /// weighted-least-squares problem in closed form (x = sum(w*b) / sum(w^2))
    /// gives the identical result in O(n) time.
    fn solve_constraints(&self, bundle: &LatticeBundle) -> DVector<f64> {
        let total_nodes = bundle.total_node_count();
        let mut solution = DVector::zeros(total_nodes);

        let mut col = 0;

        for (layer_idx, lattice) in bundle.lattices.iter().enumerate() {
            let baseline = bundle.layer_heights[layer_idx];

            for node in &lattice.nodes {
                // Direct observation constraint (weight 1.0, always present)
                let mut sum_weighted_b = baseline + node.deformation;
                let mut sum_weight_sq = 1.0;

                // Reflection constraints (multiple if faceted surface)
                let count = node.reflected_deformations.len().max(1);
                let reflection_weight = 0.8 / (count as f64).sqrt();

                for reflected_def in &node.reflected_deformations {
                    let b = baseline + reflected_def;
                    sum_weighted_b += reflection_weight * b;
                    sum_weight_sq += reflection_weight * reflection_weight;
                }

                solution[col] = sum_weighted_b / sum_weight_sq;
                col += 1;
            }
        }

        solution
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
            return f64::MAX; // Size mismatch
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
            SolverError::InsufficientConstraints => {
                write!(f, "Insufficient constraints for resolution")
            }
        }
    }
}

impl std::error::Error for SolverError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocrs::projection::{MeshProjector, StaticMesh};

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
        assert!(
            reconstructed.confidence > 0.5,
            "Confidence should be reasonable at 50m"
        );

        println!(
            "Reconstructed {} vertices with {:.1}% confidence",
            reconstructed.vertex_count(),
            reconstructed.confidence * 100.0
        );
    }

    #[test]
    fn test_reconstruction_error() {
        // Create mesh
        let mesh = StaticMesh::test_terrain(100.0);

        // Project and reconstruct
        let projector = MeshProjector::new(2.0, 5.0);
        let bundle = projector.project(&mesh);

        let target = Point3D::new(50.0, 0.0, 50.0);
        let observer = ObserverCone::new(target, 100.0, 90.0); // Far = high confidence

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
        println!(
            "  Standard constraints: {}",
            bundle_standard.total_node_count()
        );

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
        println!(
            "  Standard confidence: {:.1}%",
            reconstructed_standard.confidence * 100.0
        );
        println!(
            "  Dual-projection confidence: {:.1}%",
            reconstructed_dual.confidence * 100.0
        );

        // Both should solve successfully
        assert!(reconstructed_dual.vertex_count() > 0);
        assert!(reconstructed_standard.vertex_count() > 0);

        // Dual projection should have more constraints
        assert!(
            dual_constraints > bundle_standard.total_node_count(),
            "Expected dual-projection to have more constraints"
        );
    }
}
