//! Physics Engine
//!
//! A comprehensive physics simulation system for RUGID implementing:
//! - Rigid body dynamics (gravity, collisions)
//! - Fluid dynamics (SPH)
//! - Thermodynamics (heat transfer, phase transitions)
//! - Relativity (SR & GR effects)
//! - Spacetime curvature visualization

pub mod core;
pub mod system;
pub mod fluid;
pub mod tessellator;
pub mod reflector;
pub mod shadows;
pub mod atmosphere;
pub mod aero;
pub mod wind;
pub mod collision;
pub mod radiosity;
pub mod thermodynamics;
pub mod relativity;
pub mod spacetime;
pub mod light;
pub mod optics;

// Re-export commonly used types
pub use core::{PhysicsState, Material, MagneticDipole};
pub use system::PhysicsSystem;
pub use fluid::{FluidSystem, FluidParticle, FluidDomain};
pub use thermodynamics::{Thermodynamics, ThermodynamicsSystem, PhaseState, HeatSource};
pub use relativity::{RelativisticState, RelativitySystem, RelativityHelpers, SPEED_OF_LIGHT};
pub use spacetime::{SpacetimeSystem, SpacetimeNode, SpacetimeConfig, MassiveBody};
pub use light::{LightSource, OpticalMaterial, Ray};
pub use optics::{Scene, calculate_lighting, compute_face_gradient};
pub use tessellator::{Tessellator, TessellationConfig, TessellatedTriangle};
pub use reflector::{Reflector, Clipper};
pub use shadows::ShadowEngine;
pub use atmosphere::Atmosphere;
pub use aero::{AeroBody, DragCoefficient, calculate_drag, calculate_lift};
pub use wind::{WindSystem, WindConfig};
pub use collision::{Collider, CollisionEvent, check_continuous_collision};
pub use radiosity::{Patch, FormFactorEngine, RadiositySolver};
