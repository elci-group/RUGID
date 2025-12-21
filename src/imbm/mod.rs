//! Inferential Mask-Based Mapping (IMBM)
//!
//! A system for 2.5D/3D inferential reconstruction from video streams using
//! a constraint-based, entropy-reduction approach.
//!
//! # Architecture
//!
//! The system operates on the principle of "Constraint-First, Geometry-Second".
//! It accumulates temporal projection constraints to bound spatial uncertainty.

pub mod system;
pub mod constraints;
pub mod field;
pub mod inference;
pub mod geometry;
#[cfg(test)]
mod tests;

pub use system::ImbmSystem;
pub use constraints::{ConstraintPacket, Mask};
pub use field::ConstraintField;
