use super::projection::{Point3D, Vector3D};
use super::weight::Weight;

/// Observer cone for OCRS resolution
#[derive(Debug, Clone)]
pub struct ObserverCone {
    pub position: Point3D,
    pub target: Point3D,
    pub fov: f64,  // Field of view in radians
    pub distance: f64,  // Current distance from target
    pub weight: Weight,  // The metallic ball causing lattice deformation
}

impl ObserverCone {
    pub fn new(target: Point3D, distance: f64, fov_degrees: f64) -> Self {
        let fov = fov_degrees.to_radians();
        let _direction = Vector3D::new(0.0, 1.0, 0.0);  // Default: above target
        let position = Point3D {
            x: target.x,
            y: target.y + distance,
            z: target.z,
        };
        
        // Create weight at target position (the ball that deforms the lattice)
        let weight = Weight::default_metallic(target);
        
        Self {
            position,
            target,
            fov,
            distance,
            weight,
        }
    }
    
    /// Compute cone width at current distance
    pub fn cone_width(&self) -> f64 {
        2.0 * self.distance * (self.fov / 2.0).tan()
    }
    
    /// Get viewing direction (unit vector from observer to target)
    pub fn direction(&self) -> Vector3D {
        (self.target - self.position).normalize()
    }
    
    /// Move observer away from target (increases distance)
    pub fn move_away(&mut self, delta: f64) {
        self.distance += delta;
        self.update_position();
    }
    
    /// Move observer closer to target (decreases distance)
    pub fn move_closer(&mut self, delta: f64) {
        self.distance = (self.distance - delta).max(0.1);
        self.update_position();
    }
    
    /// Update position based on current distance
    fn update_position(&mut self) {
        // Keep observer directly above target for simplicity
        self.position = Point3D {
            x: self.target.x,
            y: self.target.y + self.distance,
            z: self.target.z,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cone_width_increases_with_distance() {
        let target = Point3D::new(50.0, 0.0, 50.0);
        let mut observer = ObserverCone::new(target, 10.0, 90.0);
        
        let width_10m = observer.cone_width();
        
        observer.move_away(40.0);  // Now at 50m
        let width_50m = observer.cone_width();
        
        assert!(width_50m > width_10m);
        assert!((width_50m - width_10m).abs() > 1.0);  // Substantial difference
    }
    
    #[test]
    fn test_move_away_and_closer() {
        let target = Point3D::new(0.0, 0.0, 0.0);
        let mut observer = ObserverCone::new(target, 10.0, 45.0);
        
        assert!((observer.distance - 10.0).abs() < 1e-10);
        
        observer.move_away(5.0);
        assert!((observer.distance - 15.0).abs() < 1e-10);
        
        observer.move_closer(10.0);
        assert!((observer.distance - 5.0).abs() < 1e-10);
        
        // Can't go below minimum
        observer.move_closer(100.0);
        assert!(observer.distance >= 0.1);
    }
}
