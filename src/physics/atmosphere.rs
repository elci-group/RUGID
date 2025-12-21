/// Atmospheric Physics Module
///
/// Models the properties of the air (Density, Pressure, Temperature) based on altitude and local conditions.
/// Uses the International Standard Atmosphere (ISA) model and Ideal Gas Law.

// Physical Constants
pub const R_DRY: f32 = 287.058; // Specific gas constant for dry air (J/(kg·K))
pub const R_VAPOR: f32 = 461.495; // Specific gas constant for water vapor (J/(kg·K))
pub const G: f32 = 9.80665; // Gravity (m/s^2)
pub const P0: f32 = 101325.0; // Standard Sea Level Pressure (Pa)
pub const T0: f32 = 288.15; // Standard Sea Level Temperature (Kelvin) = 15°C
pub const L: f32 = 0.0065; // Temperature Lapse Rate (K/m) - Standard Troposphere

#[derive(Clone, Debug)]
pub struct Atmosphere {
    /// Sea level temperature offset (Celsius) from standard 15°C
    pub temp_offset: f32,
    /// Global humidity (0.0 - 1.0)
    pub humidity: f32,
}

impl Default for Atmosphere {
    fn default() -> Self {
        Self {
            temp_offset: 0.0,
            humidity: 0.0,
        }
    }
}

impl Atmosphere {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate air density (kg/m^3) at a given altitude (meters)
    pub fn get_density(&self, altitude: f32) -> f32 {
        let (p, t) = self.get_pressure_and_temp(altitude);
        
        // Calculate partial pressure of water vapor (simplified)
        // Saturation vapor pressure (Tetens equation)
        let t_celsius = t - 273.15;
        let es = 6.1078 * 10.0f32.powf((7.5 * t_celsius) / (t_celsius + 237.3)); // hPa
        let es_pa = es * 100.0; // Convert to Pa
        
        let pv = self.humidity * es_pa;
        let pd = p - pv; // Partial pressure of dry air
        
        // Density = (Pd / (Rd * T)) + (Pv / (Rv * T))
        let rho_d = pd / (R_DRY * t);
        let rho_v = pv / (R_VAPOR * t);
        
        rho_d + rho_v
    }

    /// Get Pressure (Pa) and Temperature (K) at altitude
    /// Uses the Barometric Formula for the Troposphere (h < 11km)
    pub fn get_pressure_and_temp(&self, altitude: f32) -> (f32, f32) {
        // Clamp altitude to 0 (sea level) minimum for stability
        let h = altitude.max(0.0);
        
        // Temperature at altitude (T = T0 - L * h)
        // Apply user offset
        let t_base = T0 + self.temp_offset;
        let t = t_base - L * h;
        
        // Ensure T doesn't drop below absolute zero (or reasonable minimum)
        let t = t.max(200.0); 

        // Pressure at altitude
        // P = P0 * (1 - L*h/T0)^(gM/RL)
        // Exponent factor: (g) / (R * L) roughly?
        // Actually derivation: dP/P = -g/RT dh
        // For linear temp: P = P0 * (1 - L*h/T0) ^ (g / (R_DRY * L))
        
        let exponent = G / (R_DRY * L);
        let base = 1.0 - (L * h) / t_base;
        
        let p = P0 * base.powf(exponent);
        
        (p, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sea_level_density() {
        let atmos = Atmosphere::new();
        let rho = atmos.get_density(0.0);
        // Standard sea level density is ~1.225 kg/m^3
        assert!((rho - 1.225).abs() < 0.01);
    }

    #[test]
    fn test_altitude_drop() {
        let atmos = Atmosphere::new();
        let rho_sea = atmos.get_density(0.0);
        let rho_1km = atmos.get_density(1000.0);
        assert!(rho_1km < rho_sea);
    }
    
    #[test]
    fn test_humidity_effect() {
        let mut dry = Atmosphere::new();
        dry.humidity = 0.0;
        let rho_dry = dry.get_density(0.0);
        
        let mut wet = Atmosphere::new();
        wet.humidity = 1.0; // 100% humidity
        let rho_wet = wet.get_density(0.0);
        
        // Moist air is LESS dense than dry air (water vapor is lighter than N2/O2)
        assert!(rho_wet < rho_dry);
    }
}
