//! Inferential Mask-Based Mapping (IMBM)
//!
//! A system for 2.5D/3D inferential reconstruction from video streams using
//! a constraint-based, entropy-reduction approach.
//!
//! # Architecture
//!
//! The system operates on the principle of "Constraint-First, Geometry-Second".
//! It accumulates temporal projection constraints to bound spatial uncertainty.

pub mod constraints;
pub mod field;
pub mod geometry;
pub mod inference;
pub mod system;
#[cfg(test)]
mod tests;

pub use constraints::{ConstraintPacket, Mask};
pub use field::ConstraintField;
pub use system::ImbmSystem;
