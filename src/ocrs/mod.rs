// OCRS (Observer-Centric Reality Solver) - Proof of Concept

pub mod observer;
pub mod projection;
pub mod confidence;
pub mod solver;
pub mod weight;

// Re-exports
pub use observer::ObserverCone;
pub use projection::{
    MeshProjector, StaticMesh, Point3D, Vector3D, 
    Lattice, LatticeBundle, LatticeNode
};
pub use confidence::compute_confidence;
pub use solver::MinimalSolver;
pub use weight::Weight;


pub mod adaptive;
pub use adaptive::SceneComplexity;
