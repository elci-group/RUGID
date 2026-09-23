//! Physics Engine
//!
//! A comprehensive physics simulation system for RUGID implementing:
//! - Rigid body dynamics (gravity, collisions)
//! - Fluid dynamics (SPH)
//! - Thermodynamics (heat transfer, phase transitions)
//! - Relativity (SR & GR effects)
//! - Spacetime curvature visualization

pub mod aero;
pub mod atmosphere;
pub mod collision;
pub mod core;
pub mod fluid;
pub mod light;
pub mod optics;
pub mod radiosity;
pub mod reflector;
pub mod relativity;
pub mod shadows;
pub mod spacetime;
pub mod system;
pub mod tessellator;
pub mod thermodynamics;
pub mod wind;

// Re-export commonly used types
pub use aero::{AeroBody, DragCoefficient, calculate_drag, calculate_lift};
pub use atmosphere::Atmosphere;
pub use collision::{Collider, CollisionEvent, check_continuous_collision};
pub use core::{MagneticDipole, Material, PhysicsState};
pub use fluid::{FluidDomain, FluidParticle, FluidSystem};
pub use light::{LightSource, OpticalMaterial, Ray};
pub use optics::{Scene, calculate_lighting, compute_face_gradient};
pub use radiosity::{FormFactorEngine, Patch, RadiositySolver};
pub use reflector::{Clipper, Reflector};
pub use relativity::{RelativisticState, RelativityHelpers, RelativitySystem, SPEED_OF_LIGHT};
pub use shadows::ShadowEngine;
pub use spacetime::{MassiveBody, SpacetimeConfig, SpacetimeNode, SpacetimeSystem};
pub use system::PhysicsSystem;
pub use tessellator::{TessellatedTriangle, TessellationConfig, Tessellator};
pub use thermodynamics::{HeatSource, PhaseState, Thermodynamics, ThermodynamicsSystem};
pub use wind::{WindConfig, WindSystem};
