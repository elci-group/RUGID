//! IronBeam Ontological Ruleset
//! 
//! Defines the rulesets for different scales of existence.

use crate::ironbeam::ontology::physics_config::PhysicsConfig;

// Constants
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;
pub const GRAVITATIONAL_CONSTANT: f64 = 6.674e-11;
pub const PLANCK_CONSTANT: f64 = 6.626e-34;
pub const ELECTRON_MASS: f64 = 9.109e-31;
pub const SOLAR_MASS: f64 = 1.989e30;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum OntologicalScale {
    Quantum,
    SubAtomic,
    Atomic,
    Molecular,
    Microscopic,
    Macroscopic,
    Planetary,
    Cosmic,
}

#[derive(Clone, Debug)]
pub enum LengthUnit {
    Femtometer,
    Picometer,
    Nanometer,
    Micrometer,
    Millimeter,
    Meter,
    Kilometer,
    AstronomicalUnit,
    LightYear,
    Parsec,
}

#[derive(Clone, Debug)]
pub struct ScaleRenderConfig {
    pub show_grid: bool,
    pub show_axes: bool,
    pub fog_density: f32,
    pub particle_visualization: bool,
    pub wave_function_visualization: bool,
}

impl ScaleRenderConfig {
    pub fn quantum_default() -> Self {
        Self {
            show_grid: false,
            show_axes: true,
            fog_density: 0.1,
            particle_visualization: false,
            wave_function_visualization: true,
        }
    }

    pub fn macroscopic_default() -> Self {
        Self {
            show_grid: true,
            show_axes: true,
            fog_density: 0.0,
            particle_visualization: false,
            wave_function_visualization: false,
        }
    }

    pub fn cosmic_default() -> Self {
        Self {
            show_grid: true,
            show_axes: false,
            fog_density: 0.0,
            particle_visualization: true, // Stars as particles
            wave_function_visualization: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OntologicalRuleset {
    pub scale: OntologicalScale,
    pub physics_config: PhysicsConfig,
    pub render_config: ScaleRenderConfig,
    pub time_multiplier: f64,
    pub length_unit: LengthUnit,
}

impl OntologicalScale {
    pub fn default_ruleset(&self) -> OntologicalRuleset {
        match self {
            Self::Quantum => OntologicalRuleset {
                scale: Self::Quantum,
                physics_config: PhysicsConfig {
                    g: 0.0,
                    c: SPEED_OF_LIGHT,
                    h: Some(PLANCK_CONSTANT),
                    relativistic: true,
                    quantum_effects: true,
                    default_mass: ELECTRON_MASS,
                    collision_epsilon: 1e-18,
                },
                render_config: ScaleRenderConfig::quantum_default(),
                time_multiplier: 1e-15,
                length_unit: LengthUnit::Femtometer,
            },
            Self::SubAtomic => OntologicalRuleset {
                scale: Self::SubAtomic,
                physics_config: PhysicsConfig {
                    g: 0.0,
                    c: SPEED_OF_LIGHT,
                    h: Some(PLANCK_CONSTANT),
                    relativistic: true,
                    quantum_effects: true,
                    default_mass: 1.67e-27, // Proton mass
                    collision_epsilon: 1e-15,
                },
                render_config: ScaleRenderConfig::quantum_default(),
                time_multiplier: 1e-12,
                length_unit: LengthUnit::Femtometer,
            },
            Self::Atomic => OntologicalRuleset {
                scale: Self::Atomic,
                physics_config: PhysicsConfig {
                    g: 0.0,
                    c: SPEED_OF_LIGHT,
                    h: Some(PLANCK_CONSTANT),
                    relativistic: false,
                    quantum_effects: true,
                    default_mass: 1.67e-27,
                    collision_epsilon: 1e-12,
                },
                render_config: ScaleRenderConfig::quantum_default(),
                time_multiplier: 1e-9,
                length_unit: LengthUnit::Picometer,
            },
            Self::Molecular => OntologicalRuleset {
                scale: Self::Molecular,
                physics_config: PhysicsConfig {
                    g: 0.0,
                    c: SPEED_OF_LIGHT,
                    h: None,
                    relativistic: false,
                    quantum_effects: false,
                    default_mass: 1e-25,
                    collision_epsilon: 1e-9,
                },
                render_config: ScaleRenderConfig::macroscopic_default(), // Similar to macro but smaller
                time_multiplier: 1e-6,
                length_unit: LengthUnit::Nanometer,
            },
            Self::Microscopic => OntologicalRuleset {
                scale: Self::Microscopic,
                physics_config: PhysicsConfig {
                    g: 9.81, // Gravity starts to matter but surface tension dominates
                    c: f64::INFINITY,
                    h: None,
                    relativistic: false,
                    quantum_effects: false,
                    default_mass: 1e-9,
                    collision_epsilon: 1e-6,
                },
                render_config: ScaleRenderConfig::macroscopic_default(),
                time_multiplier: 1e-3,
                length_unit: LengthUnit::Micrometer,
            },
            Self::Macroscopic => OntologicalRuleset {
                scale: Self::Macroscopic,
                physics_config: PhysicsConfig {
                    g: 9.81,
                    c: f64::INFINITY,
                    h: None,
                    relativistic: false,
                    quantum_effects: false,
                    default_mass: 1.0,
                    collision_epsilon: 1e-3,
                },
                render_config: ScaleRenderConfig::macroscopic_default(),
                time_multiplier: 1.0,
                length_unit: LengthUnit::Meter,
            },
            Self::Planetary => OntologicalRuleset {
                scale: Self::Planetary,
                physics_config: PhysicsConfig {
                    g: GRAVITATIONAL_CONSTANT, // Use G for orbital mechanics
                    c: SPEED_OF_LIGHT,
                    h: None,
                    relativistic: false,
                    quantum_effects: false,
                    default_mass: 5.97e24, // Earth mass
                    collision_epsilon: 1000.0,
                },
                render_config: ScaleRenderConfig::cosmic_default(),
                time_multiplier: 3600.0, // 1 sec = 1 hour
                length_unit: LengthUnit::Kilometer,
            },
            Self::Cosmic => OntologicalRuleset {
                scale: Self::Cosmic,
                physics_config: PhysicsConfig {
                    g: GRAVITATIONAL_CONSTANT,
                    c: SPEED_OF_LIGHT,
                    h: None,
                    relativistic: true,
                    quantum_effects: false,
                    default_mass: SOLAR_MASS,
                    collision_epsilon: 1e9,
                },
                render_config: ScaleRenderConfig::cosmic_default(),
                time_multiplier: 3.154e7, // 1 sec = 1 year
                length_unit: LengthUnit::LightYear,
            },
        }
    }
}
