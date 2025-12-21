/// Continuous Collision Detection (CCD) Module
///
/// Implements analytic collision detection using swept volumes to prevent tunneling.
/// Focuses on Sphere vs Plane and Sphere vs Sphere interactions.

use crate::geometry3d::Point3D;

#[derive(Clone, Debug, PartialEq)]
pub struct CollisionEvent {
    /// Time of impact (0.0 to 1.0, where 1.0 is full timestep)
    pub t: f32,
    /// Point of impact
    pub point: Point3D,
    /// Normal of the surface hit
    pub normal: Point3D,
}

#[derive(Clone, Debug)]
pub enum Collider {
    /// A static plane defined by a point and a normal
    Plane { point: Point3D, normal: Point3D },
    /// A sphere defined by a center (relative to body) and radius
    Sphere { radius: f32 },
}

/// Check for collision between a moving sphere and a static collider
/// start: Position at t=0
/// end: Position at t=1
/// radius: Radius of the moving sphere
/// collider: The static object to check against
pub fn check_continuous_collision(
    start: Point3D,
    end: Point3D,
    radius: f32,
    collider: &Collider,
) -> Option<CollisionEvent> {
    match collider {
        Collider::Plane { point, normal } => {
            check_swept_sphere_plane(start, end, radius, *point, *normal)
        }
        Collider::Sphere { .. } => {
            // TODO: Implement Sphere-Sphere CCD
            None
        }
    }
}

/// Ray-Plane Intersection
/// Returns t (0.0 - 1.0) if intersection occurs within the segment
fn intersect_ray_plane(
    ray_origin: Point3D,
    ray_dir: Point3D, // Full displacement vector (end - start)
    plane_point: Point3D,
    plane_normal: Point3D,
) -> Option<f32> {
    let denom = plane_normal.dot(&ray_dir);
    
    // Check if ray is parallel to plane
    if denom.abs() < 1e-6 {
        return None;
    }
    
    let vec_to_plane = plane_point - ray_origin;
    let t = plane_normal.dot(&vec_to_plane) / denom;
    
    if t >= 0.0 && t <= 1.0 {
        Some(t)
    } else {
        None
    }
}

/// Swept Sphere vs Plane Intersection
/// A swept sphere collides with a plane when the distance from the center to the plane is equal to the radius.
/// This is equivalent to intersecting a Ray (path of center) with a Plane shifted by the radius along the normal.
fn check_swept_sphere_plane(
    start: Point3D,
    end: Point3D,
    radius: f32,
    plane_point: Point3D,
    plane_normal: Point3D,
) -> Option<CollisionEvent> {
    let displacement = end - start;
    let normal = plane_normal.normalize();
    
    // 1. Check if we are already intersecting or behind the plane at start
    // Distance from start to plane
    let dist_start = (start - plane_point).dot(&normal);
    
    // If we are moving AWAY from the plane, ignore (backface culling / separating)
    if displacement.dot(&normal) > 0.0 {
        return None;
    }
    
    // If we started inside/behind, that's a static collision (t=0)
    // But for CCD, we usually care about entering.
    // If dist_start < radius, we are already colliding.
    if dist_start < radius {
        // Already penetrating. Return t=0.
        return Some(CollisionEvent {
            t: 0.0,
            point: start - normal * dist_start, // Project to plane
            normal,
        });
    }
    
    // 2. We want to find t where distance(P(t), plane) = radius
    // P(t) = start + displacement * t
    // (P(t) - plane_point) . normal = radius
    // (start + disp*t - plane_point) . normal = radius
    // (start - plane_point).normal + (disp.normal)*t = radius
    // dist_start + (disp.normal)*t = radius
    // t = (radius - dist_start) / (disp.normal)
    
    let denom = displacement.dot(&normal);
    
    if denom.abs() < 1e-6 {
        return None; // Parallel motion
    }
    
    let t = (radius - dist_start) / denom;
    
    if t >= 0.0 && t <= 1.0 {
        let impact_center = start + displacement * t;
        let impact_point = impact_center - normal * radius;
        
        Some(CollisionEvent {
            t,
            point: impact_point,
            normal,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swept_sphere_plane_hit() {
        let start = Point3D::new(0.0, 10.0, 0.0);
        let end = Point3D::new(0.0, -10.0, 0.0); // Moving down
        let radius = 1.0;
        
        // Plane at y=0, normal pointing up (0, 1, 0)
        let plane_point = Point3D::new(0.0, 0.0, 0.0);
        let plane_normal = Point3D::new(0.0, 1.0, 0.0);
        
        let result = check_swept_sphere_plane(start, end, radius, plane_point, plane_normal);
        
        assert!(result.is_some());
        let event = result.unwrap();
        
        // Should hit when center is at y=1.0 (radius distance)
        // Start y=10, End y=-10. Total dist 20.
        // Hit at y=1. Travel dist = 9.
        // t = 9 / 20 = 0.45
        assert!((event.t - 0.45).abs() < 0.001);
        assert!((event.point.y - 0.0).abs() < 0.001); // Touches plane surface
    }

    #[test]
    fn test_swept_sphere_plane_miss() {
        let start = Point3D::new(0.0, 10.0, 0.0);
        let end = Point3D::new(0.0, 5.0, 0.0); // Stops before hit
        let radius = 1.0;
        
        let plane_point = Point3D::new(0.0, 0.0, 0.0);
        let plane_normal = Point3D::new(0.0, 1.0, 0.0);
        
        let result = check_swept_sphere_plane(start, end, radius, plane_point, plane_normal);
        
        assert!(result.is_none());
    }
}
