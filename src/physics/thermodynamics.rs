//! Thermodynamics and Physical State Change
//!
//! Implements heat transfer, phase transitions (solid ↔ liquid ↔ gas),
//! and temperature-dependent material properties.

use crate::geometry3d::Point3D;
use crate::cell::CellId;
use std::collections::HashMap;

/// Physical state of matter
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseState {
    Solid,
    Liquid,
    Gas,
    Plasma, // For extreme temperatures
}

/// Thermodynamic properties of a body
#[derive(Clone, Debug, PartialEq)]
pub struct Thermodynamics {
    /// Current temperature (Kelvin)
    pub temperature: f32,
    /// Specific heat capacity (J/kg·K)
    pub heat_capacity: f32,
    /// Thermal conductivity (W/m·K)
    pub thermal_conductivity: f32,
    /// Melting point (K)
    pub melting_point: f32,
    /// Boiling point (K)
    pub boiling_point: f32,
    /// Latent heat of fusion (J/kg)
    pub latent_heat_fusion: f32,
    /// Latent heat of vaporization (J/kg)
    pub latent_heat_vaporization: f32,
    /// Current phase
    pub phase: PhaseState,
    /// Accumulated latent heat during transition
    pub transition_energy: f32,
    /// Mass (kg)
    pub mass: f32,
}

impl Default for Thermodynamics {
    fn default() -> Self {
        // Default to water properties
        Self {
            temperature: 293.15, // 20°C
            heat_capacity: 4186.0, // Water
            thermal_conductivity: 0.6, // Water
            melting_point: 273.15, // 0°C
            boiling_point: 373.15, // 100°C
            latent_heat_fusion: 334000.0, // Water
            latent_heat_vaporization: 2260000.0, // Water
            phase: PhaseState::Liquid,
            transition_energy: 0.0,
            mass: 1.0,
        }
    }
}

impl Thermodynamics {
    /// Create ice (solid water)
    pub fn ice(mass: f32) -> Self {
        Self {
            temperature: 263.15, // -10°C
            phase: PhaseState::Solid,
            mass,
            ..Default::default()
        }
    }

    /// Create water (liquid)
    pub fn water(mass: f32) -> Self {
        Self {
            temperature: 293.15, // 20°C
            phase: PhaseState::Liquid,
            mass,
            ..Default::default()
        }
    }

    /// Create steam (gas)
    pub fn steam(mass: f32) -> Self {
        Self {
            temperature: 383.15, // 110°C
            phase: PhaseState::Gas,
            mass,
            ..Default::default()
        }
    }

    /// Create metal (iron-like)
    pub fn metal(mass: f32) -> Self {
        Self {
            temperature: 293.15,
            heat_capacity: 450.0, // Iron
            thermal_conductivity: 80.0, // Iron
            melting_point: 1811.0, // Iron
            boiling_point: 3134.0, // Iron
            latent_heat_fusion: 247000.0,
            latent_heat_vaporization: 6090000.0,
            phase: PhaseState::Solid,
            transition_energy: 0.0,
            mass,
        }
    }
}

/// Heat source that affects nearby bodies
#[derive(Clone, Debug)]
pub struct HeatSource {
    pub position: Point3D,
    pub power: f32, // Watts
    pub radius: f32,
}

/// The Thermodynamics System
pub struct ThermodynamicsSystem {
    /// Thermodynamic states of bodies
    pub states: HashMap<CellId, Thermodynamics>,
    /// Positions (for heat transfer calculations)
    pub positions: HashMap<CellId, Point3D>,
    /// External heat sources
    pub heat_sources: Vec<HeatSource>,
    /// Ambient temperature
    pub ambient_temperature: f32,
    /// Stefan-Boltzmann constant (for radiation)
    pub stefan_boltzmann: f32,
}

impl ThermodynamicsSystem {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            positions: HashMap::new(),
            heat_sources: Vec::new(),
            ambient_temperature: 293.15, // 20°C
            stefan_boltzmann: 5.67e-8,
        }
    }

    pub fn register(&mut self, id: CellId, thermo: Thermodynamics, position: Point3D) {
        self.states.insert(id, thermo);
        self.positions.insert(id, position);
    }

    pub fn add_heat_source(&mut self, source: HeatSource) {
        self.heat_sources.push(source);
    }

    /// Main simulation tick
    pub fn tick(&mut self, dt: f32) {
        let ids: Vec<CellId> = self.states.keys().cloned().collect();

        // Step 1: Heat transfer from sources
        for id in &ids {
            let pos = match self.positions.get(id) {
                Some(p) => *p,
                None => continue,
            };

            let mut heat_in = 0.0;

            // Heat from external sources
            for source in &self.heat_sources {
                let dx = pos.x - source.position.x;
                let dy = pos.y - source.position.y;
                let dz = pos.z - source.position.z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();

                if dist < source.radius && dist > 0.001 {
                    // Inverse square law
                    heat_in += source.power / (4.0 * std::f32::consts::PI * dist * dist);
                }
            }

            // Radiation to/from environment
            if let Some(thermo) = self.states.get(id) {
                let surface_area = thermo.mass.powf(2.0 / 3.0); // Approximate
                let emissivity = 0.9; // Assume high emissivity
                let radiation = emissivity * self.stefan_boltzmann * surface_area
                    * (thermo.temperature.powi(4) - self.ambient_temperature.powi(4));
                heat_in -= radiation * dt;
            }

            // Apply heat
            if let Some(thermo) = self.states.get_mut(id) {
                let delta_q = heat_in * dt;
                Self::apply_heat_to(thermo, delta_q);
            }
        }

        // Step 2: Conduction between touching bodies
        // (Simplified: all bodies within a threshold distance conduct)
        let conduction_threshold = 0.1;
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let id_a = ids[i];
                let id_b = ids[j];

                let pos_a = match self.positions.get(&id_a) {
                    Some(p) => *p,
                    None => continue,
                };
                let pos_b = match self.positions.get(&id_b) {
                    Some(p) => *p,
                    None => continue,
                };

                let dx = pos_a.x - pos_b.x;
                let dy = pos_a.y - pos_b.y;
                let dz = pos_a.z - pos_b.z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();

                if dist < conduction_threshold && dist > 0.001 {
                    // Fourier's Law: Q = -k * A * dT/dx
                    let (temp_a, temp_b, k_avg) = {
                        let a = &self.states[&id_a];
                        let b = &self.states[&id_b];
                        let k_avg = (a.thermal_conductivity + b.thermal_conductivity) / 2.0;
                        (a.temperature, b.temperature, k_avg)
                    };

                    let contact_area = 0.01; // Simplified
                    let heat_flow = k_avg * contact_area * (temp_a - temp_b) / dist * dt;

                    if let Some(a) = self.states.get_mut(&id_a) {
                        Self::apply_heat_to(a, -heat_flow);
                    }
                    if let Some(b) = self.states.get_mut(&id_b) {
                        Self::apply_heat_to(b, heat_flow);
                    }
                }
            }
        }
    }

    /// Apply heat to a body, handling phase transitions
    fn apply_heat_to(thermo: &mut Thermodynamics, delta_q: f32) {
        match thermo.phase {
            PhaseState::Solid => {
                // Check for melting
                if thermo.temperature >= thermo.melting_point {
                    thermo.transition_energy += delta_q;
                    let required = thermo.latent_heat_fusion * thermo.mass;
                    if thermo.transition_energy >= required {
                        thermo.phase = PhaseState::Liquid;
                        thermo.transition_energy = 0.0;
                    }
                } else {
                    // Normal heating: ΔT = Q / (m * c)
                    thermo.temperature += delta_q / (thermo.mass * thermo.heat_capacity);
                }
            }
            PhaseState::Liquid => {
                // Check for freezing
                if thermo.temperature <= thermo.melting_point && delta_q < 0.0 {
                    thermo.transition_energy -= delta_q;
                    let required = thermo.latent_heat_fusion * thermo.mass;
                    if thermo.transition_energy >= required {
                        thermo.phase = PhaseState::Solid;
                        thermo.transition_energy = 0.0;
                    }
                }
                // Check for boiling
                else if thermo.temperature >= thermo.boiling_point {
                    thermo.transition_energy += delta_q;
                    let required = thermo.latent_heat_vaporization * thermo.mass;
                    if thermo.transition_energy >= required {
                        thermo.phase = PhaseState::Gas;
                        thermo.transition_energy = 0.0;
                    }
                } else {
                    thermo.temperature += delta_q / (thermo.mass * thermo.heat_capacity);
                }
            }
            PhaseState::Gas => {
                // Check for condensation
                if thermo.temperature <= thermo.boiling_point && delta_q < 0.0 {
                    thermo.transition_energy -= delta_q;
                    let required = thermo.latent_heat_vaporization * thermo.mass;
                    if thermo.transition_energy >= required {
                        thermo.phase = PhaseState::Liquid;
                        thermo.transition_energy = 0.0;
                    }
                } else {
                    thermo.temperature += delta_q / (thermo.mass * thermo.heat_capacity);
                }
            }
            PhaseState::Plasma => {
                // Plasma cooling/heating
                thermo.temperature += delta_q / (thermo.mass * thermo.heat_capacity);
                if thermo.temperature < 10000.0 {
                    thermo.phase = PhaseState::Gas;
                }
            }
        }

        // Check for plasma transition
        if thermo.temperature > 10000.0 && thermo.phase == PhaseState::Gas {
            thermo.phase = PhaseState::Plasma;
        }
    }

    /// Get the current phase of a body
    pub fn get_phase(&self, id: CellId) -> Option<PhaseState> {
        self.states.get(&id).map(|s| s.phase)
    }

    /// Get temperature
    pub fn get_temperature(&self, id: CellId) -> Option<f32> {
        self.states.get(&id).map(|s| s.temperature)
    }
}
