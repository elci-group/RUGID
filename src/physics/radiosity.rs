/// Global Illumination - Radiosity Module
///
/// Implements analytic radiosity using geometric form factors.
/// This enables realistic indirect lighting (light bouncing between surfaces).

use crate::geometry3d::Point3D;

/// A surface patch for radiosity computation
#[derive(Clone, Debug)]
pub struct Patch {
    /// Center point of the patch
    pub center: Point3D,
    /// Surface normal (unit vector)
    pub normal: Point3D,
    /// Surface area (m^2)
    pub area: f32,
    /// Albedo (reflectance) color (0.0 - 1.0 per channel)
    pub albedo: (f32, f32, f32),
    /// Direct emission (for light sources)
    pub emission: (f32, f32, f32),
    /// Current radiosity (computed)
    pub radiosity: (f32, f32, f32),
}

impl Patch {
    pub fn new(center: Point3D, normal: Point3D, area: f32, albedo: (f32, f32, f32)) -> Self {
        Self {
            center,
            normal: normal.normalize(),
            area,
            albedo,
            emission: (0.0, 0.0, 0.0),
            radiosity: (0.0, 0.0, 0.0),
        }
    }
}

/// Form Factor Calculator
pub struct FormFactorEngine {
    /// Cached form factors (sparse matrix)
    /// Key: (patch_i, patch_j), Value: form factor
    cache: std::collections::HashMap<(usize, usize), f32>,
}

impl FormFactorEngine {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    /// Calculate form factor between two patches
    /// F_ij = (cos θ_i * cos θ_j) / (π * r^2) * visibility
    pub fn calculate_form_factor(
        &mut self,
        patch_i: &Patch,
        patch_j: &Patch,
        i: usize,
        j: usize,
    ) -> f32 {
        // Check cache first
        if let Some(&cached) = self.cache.get(&(i, j)) {
            return cached;
        }

        // Self-interaction is zero
        if i == j {
            self.cache.insert((i, j), 0.0);
            return 0.0;
        }

        // Vector from i to j
        let r_vec = patch_j.center - patch_i.center;
        let distance = r_vec.length();

        if distance < 1e-6 {
            self.cache.insert((i, j), 0.0);
            return 0.0;
        }

        let r_dir = r_vec.normalize();

        // Angles between normals and connecting vector
        let cos_theta_i = patch_i.normal.dot(&r_dir);
        let cos_theta_j = patch_j.normal.dot(&(r_dir * -1.0));

        // Back-facing check
        if cos_theta_i <= 0.0 || cos_theta_j <= 0.0 {
            self.cache.insert((i, j), 0.0);
            return 0.0;
        }

        // Geometric form factor (simplified, no visibility yet)
        let form_factor = (cos_theta_i * cos_theta_j) / (std::f32::consts::PI * distance * distance);

        // TODO: Multiply by visibility factor (1.0 if visible, 0.0 if occluded)
        // For now, assume full visibility
        let visibility = 1.0;

        let result = form_factor * visibility * patch_j.area;

        self.cache.insert((i, j), result);
        result
    }

    /// Clear the cache (when scene changes)
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// Radiosity Solver using Gauss-Seidel iteration
pub struct RadiositySolver {
    pub form_factor_engine: FormFactorEngine,
    pub max_iterations: usize,
    pub convergence_threshold: f32,
}

impl Default for RadiositySolver {
    fn default() -> Self {
        Self {
            form_factor_engine: FormFactorEngine::new(),
            max_iterations: 10,
            convergence_threshold: 0.001,
        }
    }
}

impl RadiositySolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Solve radiosity for all patches
    /// B_i = E_i + ρ_i * Σ F_ij * B_j
    pub fn solve(&mut self, patches: &mut [Patch]) {
        let n = patches.len();

        for iteration in 0..self.max_iterations {
            let mut max_change = 0.0f32;

            for i in 0..n {
                // Calculate incoming light from all other patches
                let mut incoming = (0.0, 0.0, 0.0);

                for j in 0..n {
                    if i == j {
                        continue;
                    }

                    let form_factor = self.form_factor_engine.calculate_form_factor(
                        &patches[i],
                        &patches[j],
                        i,
                        j,
                    );

                    // Add contribution from patch j
                    incoming.0 += form_factor * patches[j].radiosity.0;
                    incoming.1 += form_factor * patches[j].radiosity.1;
                    incoming.2 += form_factor * patches[j].radiosity.2;
                }

                // New radiosity = emission + reflectance * incoming
                let albedo = patches[i].albedo;
                let new_radiosity = (
                    patches[i].emission.0 + albedo.0 * incoming.0,
                    patches[i].emission.1 + albedo.1 * incoming.1,
                    patches[i].emission.2 + albedo.2 * incoming.2,
                );

                // Calculate change for convergence check
                let change = ((new_radiosity.0 - patches[i].radiosity.0).abs()
                    + (new_radiosity.1 - patches[i].radiosity.1).abs()
                    + (new_radiosity.2 - patches[i].radiosity.2).abs())
                    / 3.0;

                max_change = max_change.max(change);

                // Update radiosity
                patches[i].radiosity = new_radiosity;
            }

            // Check convergence
            if max_change < self.convergence_threshold {
                println!("Radiosity converged after {} iterations", iteration + 1);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_factor_parallel_faces() {
        let mut engine = FormFactorEngine::new();

        // Two parallel faces facing each other
        let patch1 = Patch::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
            1.0,
            (1.0, 1.0, 1.0),
        );

        let patch2 = Patch::new(
            Point3D::new(0.0, 0.0, 2.0),
            Point3D::new(0.0, 0.0, -1.0),
            1.0,
            (1.0, 1.0, 1.0),
        );

        let ff = engine.calculate_form_factor(&patch1, &patch2, 0, 1);

        // Should be non-zero (facing each other)
        assert!(ff > 0.0);
    }

    #[test]
    fn test_radiosity_simple() {
        let mut solver = RadiositySolver::new();

        // Simple test: One emissive patch, one reflective patch
        let mut patches = vec![
            Patch {
                center: Point3D::new(0.0, 0.0, 0.0),
                normal: Point3D::new(0.0, 0.0, 1.0),
                area: 1.0,
                albedo: (0.0, 0.0, 0.0),
                emission: (1.0, 1.0, 1.0), // Light source
                radiosity: (1.0, 1.0, 1.0),
            },
            Patch {
                center: Point3D::new(0.0, 0.0, 2.0),
                normal: Point3D::new(0.0, 0.0, -1.0),
                area: 1.0,
                albedo: (0.8, 0.8, 0.8), // Reflective
                emission: (0.0, 0.0, 0.0),
                radiosity: (0.0, 0.0, 0.0),
            },
        ];

        solver.solve(&mut patches);

        // Second patch should now have non-zero radiosity (received bounced light)
        assert!(patches[1].radiosity.0 > 0.0);
    }
}
