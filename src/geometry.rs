#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// Geometry expressed in orientation-relative axes.
/// P = Primary axis (flow direction)
/// S = Secondary axis (orthogonal)
///
/// All values are normalized [0.0 - 1.0].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorRegion {
    pub origin_p: f32,
    pub origin_s: f32,
    pub extent_p: f32,
    pub extent_s: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbsoluteRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl VectorRegion {
    pub fn new(origin_p: f32, origin_s: f32, extent_p: f32, extent_s: f32) -> Self {
        Self {
            origin_p,
            origin_s,
            extent_p,
            extent_s,
        }
    }

    /// Resolve to absolute coordinates based on orientation.
    /// In Portrait: Primary = Y, Secondary = X
    /// In Landscape: Primary = X, Secondary = Y
    /// Note: This is a simplification. Real layout might be more complex.
    pub fn resolve(&self, orientation: Orientation, width: f32, height: f32) -> AbsoluteRect {
        match orientation {
            Orientation::Portrait => {
                // P -> Y, S -> X
                AbsoluteRect {
                    x: self.origin_s * width,
                    y: self.origin_p * height,
                    width: self.extent_s * width,
                    height: self.extent_p * height,
                }
            }
            Orientation::Landscape => {
                // P -> X, S -> Y
                AbsoluteRect {
                    x: self.origin_p * width,
                    y: self.origin_s * height,
                    width: self.extent_p * width,
                    height: self.extent_s * height,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_resolution_portrait() {
        let region = VectorRegion::new(0.1, 0.2, 0.5, 0.3); // P=0.1, S=0.2
        let rect = region.resolve(Orientation::Portrait, 100.0, 200.0);

        // Portrait: P=Y, S=X
        // x = S * width = 0.2 * 100 = 20
        // y = P * height = 0.1 * 200 = 20
        // w = S_ext * width = 0.3 * 100 = 30
        // h = P_ext * height = 0.5 * 200 = 100
        
        assert!((rect.x - 20.0).abs() < 1e-5);
        assert!((rect.y - 20.0).abs() < 1e-5);
        assert!((rect.width - 30.0).abs() < 1e-5);
        assert!((rect.height - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_geometry_resolution_landscape() {
        let region = VectorRegion::new(0.1, 0.2, 0.5, 0.3); // P=0.1, S=0.2
        let rect = region.resolve(Orientation::Landscape, 200.0, 100.0);

        // Landscape: P=X, S=Y
        // x = P * width = 0.1 * 200 = 20
        // y = S * height = 0.2 * 100 = 20
        // w = P_ext * width = 0.5 * 200 = 100
        // h = S_ext * height = 0.3 * 100 = 30

        assert!((rect.x - 20.0).abs() < 1e-5);
        assert!((rect.y - 20.0).abs() < 1e-5);
        assert!((rect.width - 100.0).abs() < 1e-5);
        assert!((rect.height - 30.0).abs() < 1e-5);
    }
}
