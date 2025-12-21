//! IronBeam Physics Configuration
//! 
//! Defines physics parameters for different ontological scales.

#[derive(Clone, Debug)]
pub struct PhysicsConfig {
    /// Gravitational constant (scaled for the ruleset)
    pub g: f64,
    
    /// Speed of light (scaled or infinite for non-relativistic)
    pub c: f64,
    
    /// Planck constant (only relevant for quantum/atomic)
    pub h: Option<f64>,
    
    /// Enable relativistic effects
    pub relativistic: bool,
    
    /// Enable quantum effects (probability waves, tunneling)
    pub quantum_effects: bool,
    
    /// Default particle/body mass for the scale
    pub default_mass: f64,
    
    /// Collision detection precision
    pub collision_epsilon: f64,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            g: 9.81,
            c: f64::INFINITY,
            h: None,
            relativistic: false,
            quantum_effects: false,
            default_mass: 1.0,
            collision_epsilon: 1e-6,
        }
    }
}
