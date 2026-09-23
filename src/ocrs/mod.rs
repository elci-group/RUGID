// OCRS (Observer-Centric Reality Solver) - Proof of Concept

pub mod confidence;
pub mod observer;
pub mod projection;
pub mod solver;
pub mod weight;

// Re-exports
pub use confidence::compute_confidence;
pub use observer::ObserverCone;
pub use projection::{
    Lattice, LatticeBundle, LatticeNode, MeshProjector, Point3D, StaticMesh, Vector3D,
};
pub use solver::MinimalSolver;
pub use weight::Weight;

pub mod adaptive;
pub use adaptive::SceneComplexity;
