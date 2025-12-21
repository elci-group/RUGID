//! IronBeam Ontology Transition
//! 
//! Handles transitions between different ontological scales.

use crate::ironbeam::ontology::ruleset::OntologicalScale;
use crate::ironbeam::ontology::physics_config::PhysicsConfig;

#[derive(Clone, Debug)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl EasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t },
        }
    }
}

pub struct ScaleTransition {
    pub from: OntologicalScale,
    pub to: OntologicalScale,
    pub progress: f32,  // 0.0 to 1.0
    pub duration: f32,
    pub easing: EasingFunction,
}

impl ScaleTransition {
    pub fn new(from: OntologicalScale, to: OntologicalScale, duration: f32) -> Self {
        Self {
            from,
            to,
            progress: 0.0,
            duration,
            easing: EasingFunction::EaseInOut,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.progress += dt / self.duration;
        if self.progress >= 1.0 {
            self.progress = 1.0;
            return true; // Finished
        }
        false
    }

    pub fn interpolate_physics(&self, from_config: &PhysicsConfig, to_config: &PhysicsConfig) -> PhysicsConfig {
        let t = self.easing.apply(self.progress);
        let t_f64 = t as f64;
        
        // Helper for linear interpolation
        fn lerp(a: f64, b: f64, t: f64) -> f64 {
            if a.is_infinite() && b.is_infinite() { return a; }
            if a.is_infinite() { return b; } // Transition from infinite to finite
            if b.is_infinite() { return a; } // Transition from finite to infinite (simplified)
            a + (b - a) * t
        }

        PhysicsConfig {
            g: lerp(from_config.g, to_config.g, t_f64),
            c: lerp(from_config.c, to_config.c, t_f64),
            h: if t < 0.5 { from_config.h } else { to_config.h },
            relativistic: if t < 0.5 { from_config.relativistic } else { to_config.relativistic },
            quantum_effects: if t < 0.5 { from_config.quantum_effects } else { to_config.quantum_effects },
            default_mass: lerp(from_config.default_mass, to_config.default_mass, t_f64),
            collision_epsilon: lerp(from_config.collision_epsilon, to_config.collision_epsilon, t_f64),
        }
    }
}
