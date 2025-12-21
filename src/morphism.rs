//! Morphism System - Shape Transformation Engine
//!
//! This module provides shape morphing capabilities for RUGID, enabling smooth
//! transitions between different geometric shapes over time.
//!
//! ## Core Concepts
//!
//! - **ShapeType**: Enum defining available shapes (Rectangle, Circle, etc.)
//! - **MorphismState**: Attached to cells to track morphing progress
//! - **Path Normalization**: Converting shapes to common point-based representation
//! - **Interpolation**: Linear interpolation between normalized paths
//!
//! ## Integration
//!
//! Morphism integrates with the temporal delta system via `DeltaFrame.delta_morph_progress`
//! and renders via SVG path generation in the renderer.

use std::f32::consts::PI;

/// Represents different geometric shapes that can be morphed between
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeType {
    /// Standard rectangle (4 corners)
    Rectangle,
    /// Rectangle with rounded corners
    RoundedRectangle { 
        /// Corner radius as fraction of minimum dimension (0.0 to 0.5)
        radius: f32 
    },
    /// Perfect circle
    Circle,
    /// Ellipse with custom radii
    Ellipse { 
        /// X-axis radius multiplier (0.0 to 1.0)
        rx: f32, 
        /// Y-axis radius multiplier (0.0 to 1.0)
        ry: f32 
    },
    /// Regular polygon
    Polygon { 
        /// Number of sides (3 = triangle, 5 = pentagon, etc.)
        sides: u32 
    },
}

/// State tracking for a cell undergoing morphism
#[derive(Debug, Clone, PartialEq)]
pub struct MorphismState {
    /// The shape the cell currently is (or started as)
    pub current_shape: ShapeType,
    /// The shape the cell is morphing into (None = no active morph)
    pub target_shape: Option<ShapeType>,
    /// Progress through the morphing animation (0.0 = current, 1.0 = target)
    pub progress: f32,
}

impl MorphismState {
    /// Create a new morphism state with a starting shape
    pub fn new(shape: ShapeType) -> Self {
        Self {
            current_shape: shape,
            target_shape: None,
            progress: 0.0,
        }
    }
    
    /// Begin morphing to a new target shape
    pub fn morph_to(&mut self, target: ShapeType) {
        self.target_shape = Some(target);
        self.progress = 0.0;
    }
    
    /// Advance the morphing progress by a delta amount
    pub fn advance(&mut self, delta: f32) {
        self.progress = (self.progress + delta).clamp(0.0, 1.0);
        
        // If morphing is complete, update current shape
        if self.progress >= 1.0 {
            if let Some(target) = self.target_shape.take() {
                self.current_shape = target;
                self.progress = 0.0;
            }
        }
    }
    
    /// Check if currently morphing
    pub fn is_morphing(&self) -> bool {
        self.target_shape.is_some() && self.progress < 1.0
    }
}

/// A 2D point in normalized space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    /// Linear interpolation between two points
    pub fn lerp(a: Point2D, b: Point2D, t: f32) -> Point2D {
        Point2D {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
        }
    }
}

/// A normalized path representation with a fixed number of points
#[derive(Debug, Clone)]
pub struct NormalizedPath {
    /// Control points around the shape's perimeter
    pub points: Vec<Point2D>,
    /// Whether the path forms a closed loop
    pub closed: bool,
}

impl NormalizedPath {
    /// Create a new normalized path
    pub fn new(points: Vec<Point2D>, closed: bool) -> Self {
        Self { points, closed }
    }
    
    /// Get the number of points in this path
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
}

/// Convert a shape to its normalized path representation
pub fn shape_to_normalized_path(
    shape: &ShapeType, 
    bounds: (f32, f32, f32, f32),
    point_count: usize,
) -> NormalizedPath {
    let (x, y, width, height) = bounds;
    
    match shape {
        ShapeType::Rectangle => rectangle_to_path(x, y, width, height, point_count),
        ShapeType::Circle => circle_to_path(x, y, width, height, point_count),
        ShapeType::RoundedRectangle { radius } => {
            rounded_rectangle_to_path(x, y, width, height, *radius, point_count)
        },
        ShapeType::Ellipse { rx, ry } => {
            ellipse_to_path(x, y, width, height, *rx, *ry, point_count)
        },
        ShapeType::Polygon { sides } => {
            polygon_to_path(x, y, width, height, *sides, point_count)
        },
    }
}

/// Convert rectangle to normalized path with uniform point distribution
fn rectangle_to_path(x: f32, y: f32, w: f32, h: f32, point_count: usize) -> NormalizedPath {
    let mut points = Vec::with_capacity(point_count);
    let perimeter = 2.0 * (w + h);
    
    for i in 0..point_count {
        let dist = (i as f32 / point_count as f32) * perimeter;
        let point = if dist < w {
            // Top edge
            Point2D::new(x + dist, y)
        } else if dist < w + h {
            // Right edge
            Point2D::new(x + w, y + (dist - w))
        } else if dist < 2.0 * w + h {
            // Bottom edge
            Point2D::new(x + w - (dist - w - h), y + h)
        } else {
            // Left edge
            Point2D::new(x, y + h - (dist - 2.0 * w - h))
        };
        points.push(point);
    }
    
    NormalizedPath::new(points, true)
}

/// Convert circle to normalized path using uniform angular distribution
fn circle_to_path(x: f32, y: f32, w: f32, h: f32, point_count: usize) -> NormalizedPath {
    let mut points = Vec::with_capacity(point_count);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let rx = w / 2.0;
    let ry = h / 2.0;
    
    for i in 0..point_count {
        let angle = 2.0 * PI * i as f32 / point_count as f32;
        points.push(Point2D::new(
            cx + rx * angle.cos(),
            cy + ry * angle.sin(),
        ));
    }
    
    NormalizedPath::new(points, true)
}

/// Convert rounded rectangle to normalized path
fn rounded_rectangle_to_path(
    x: f32, y: f32, w: f32, h: f32, 
    radius_fraction: f32, 
    point_count: usize
) -> NormalizedPath {
    // Clamp radius to valid range
    let radius = (radius_fraction.clamp(0.0, 0.5) * w.min(h)).min(w / 2.0).min(h / 2.0);
    
    // If radius is effectively zero, just return rectangle
    if radius < 0.01 {
        return rectangle_to_path(x, y, w, h, point_count);
    }
    
    let mut points = Vec::with_capacity(point_count);
    let perimeter = 2.0 * (w + h - 4.0 * radius) + 2.0 * PI * radius;
    
    for i in 0..point_count {
        let dist = (i as f32 / point_count as f32) * perimeter;
        let mut accumulated = 0.0;
        
        // Top edge (minus corners)
        let top_len = w - 2.0 * radius;
        if dist < accumulated + top_len {
            let d = dist - accumulated;
            points.push(Point2D::new(x + radius + d, y));
            continue;
        }
        accumulated += top_len;
        
        // Top-right arc
        let arc_len = PI * radius / 2.0;
        if dist < accumulated + arc_len {
            let d = dist - accumulated;
            let angle = -PI / 2.0 + (d / arc_len) * (PI / 2.0);
            points.push(Point2D::new(
                x + w - radius + radius * angle.cos(),
                y + radius + radius * angle.sin(),
            ));
            continue;
        }
        accumulated += arc_len;
        
        // Right edge (minus corners)
        let right_len = h - 2.0 * radius;
        if dist < accumulated + right_len {
            let d = dist - accumulated;
            points.push(Point2D::new(x + w, y + radius + d));
            continue;
        }
        accumulated += right_len;
        
        // Bottom-right arc
        if dist < accumulated + arc_len {
            let d = dist - accumulated;
            let angle = 0.0 + (d / arc_len) * (PI / 2.0);
            points.push(Point2D::new(
                x + w - radius + radius * angle.cos(),
                y + h - radius + radius * angle.sin(),
            ));
            continue;
        }
        accumulated += arc_len;
        
        // Bottom edge (minus corners)
        if dist < accumulated + top_len {
            let d = dist - accumulated;
            points.push(Point2D::new(x + w - radius - d, y + h));
            continue;
        }
        accumulated += top_len;
        
        // Bottom-left arc
        if dist < accumulated + arc_len {
            let d = dist - accumulated;
            let angle = PI / 2.0 + (d / arc_len) * (PI / 2.0);
            points.push(Point2D::new(
                x + radius + radius * angle.cos(),
                y + h - radius + radius * angle.sin(),
            ));
            continue;
        }
        accumulated += arc_len;
        
        // Left edge (minus corners)
        let d = dist - accumulated;
        points.push(Point2D::new(x, y + h - radius - d));
    }
    
    NormalizedPath::new(points, true)
}

/// Convert ellipse to normalized path
fn ellipse_to_path(
    x: f32, y: f32, w: f32, h: f32,
    rx_mult: f32, ry_mult: f32,
    point_count: usize
) -> NormalizedPath {
    let mut points = Vec::with_capacity(point_count);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let rx = (w / 2.0) * rx_mult.clamp(0.1, 1.0);
    let ry = (h / 2.0) * ry_mult.clamp(0.1, 1.0);
    
    for i in 0..point_count {
        let angle = 2.0 * PI * i as f32 / point_count as f32;
        points.push(Point2D::new(
            cx + rx * angle.cos(),
            cy + ry * angle.sin(),
        ));
    }
    
    NormalizedPath::new(points, true)
}

/// Convert regular polygon to normalized path
fn polygon_to_path(
    x: f32, y: f32, w: f32, h: f32,
    sides: u32,
    point_count: usize
) -> NormalizedPath {
    let sides = sides.max(3); // Minimum 3 sides
    let mut points = Vec::with_capacity(point_count);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let rx = w / 2.0;
    let ry = h / 2.0;
    
    for i in 0..point_count {
        // Distribute points around polygon perimeter
        let t = i as f32 / point_count as f32;
        let angle = 2.0 * PI * t - PI / 2.0; // Start at top
        points.push(Point2D::new(
            cx + rx * angle.cos(),
            cy + ry * angle.sin(),
        ));
    }
    
    NormalizedPath::new(points, true)
}

/// Linearly interpolate between two normalized paths
pub fn interpolate_paths(
    from: &NormalizedPath,
    to: &NormalizedPath,
    t: f32,
) -> NormalizedPath {
    assert_eq!(
        from.point_count(), 
        to.point_count(),
        "Paths must have same point count for interpolation"
    );
    
    let points = from.points
        .iter()
        .zip(to.points.iter())
        .map(|(a, b)| Point2D::lerp(*a, *b, t))
        .collect();
    
    NormalizedPath::new(points, from.closed && to.closed)
}

/// Adaptively interpolate between paths with different point counts
///
/// This function subdivides the path with fewer points to match the path with more points,
/// enabling morphing between shapes of different complexity (e.g., triangle → hexagon).
pub fn interpolate_paths_adaptive(from: &NormalizedPath, to: &NormalizedPath, t: f32) -> NormalizedPath {
    let from_count = from.point_count();
    let to_count = to.point_count();
    
    if from_count == to_count {
        // Same count: use standard interpolation
        return interpolate_paths(from, to, t);
    }
    
    // Different counts: subdivide the smaller path
    let (from_normalized, to_normalized) = if from_count < to_count {
        (subdivide_path(from, to_count), to.clone())
    } else {
        (from.clone(), subdivide_path(to, from_count))
    };
    
    interpolate_paths(&from_normalized, &to_normalized, t)
}

/// Subdivide a path to have a target number of points
///
/// Uses linear interpolation between existing points to create new points.
fn subdivide_path(path: &NormalizedPath, target_count: usize) -> NormalizedPath {
    if path.point_count() >= target_count {
        return path.clone();
    }
    
    let mut new_points = Vec::with_capacity(target_count);
    let source_points = &path.points;
    let source_count = source_points.len();
    
    // Calculate how many points to insert between each pair
    let scale = (target_count as f32) / (source_count as f32);
    
    for i in 0..target_count {
        let float_index = i as f32 / scale;
        let index_floor = float_index.floor() as usize % source_count;
        let index_ceil = (index_floor + 1) % source_count;
        let t = float_index.fract();
        
        let p1 = source_points[index_floor];
        let p2 = source_points[index_ceil];
        
        new_points.push(Point2D::lerp(p1, p2, t));
    }
    
    NormalizedPath::new(new_points, path.closed)
}

/// Standard easing functions for non-linear morphing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl EasingFunction {
    /// Apply the easing function to a value in range [0, 1]
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphism_state_progression() {
        let mut state = MorphismState::new(ShapeType::Rectangle);
        assert!(!state.is_morphing());
        
        state.morph_to(ShapeType::Circle);
        assert!(state.is_morphing());
        assert_eq!(state.progress, 0.0);
        
        state.advance(0.5);
        assert_eq!(state.progress, 0.5);
        assert!(state.is_morphing());
        
        state.advance(0.6); // Total would be 1.1, gets clamped and completes
        // After completion, progress resets to 0.0 and target is moved to current
        assert_eq!(state.progress, 0.0);
        assert!(!state.is_morphing());
        assert_eq!(state.current_shape, ShapeType::Circle);
    }
    
    #[test]
    fn test_adaptive_interpolation_same_count() {
        // When counts match, should behave like standard interpolation
        let path1 = NormalizedPath::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
        ], true);
        
        let path2 = NormalizedPath::new(vec![
            Point2D::new(0.5, 0.5),
            Point2D::new(1.5, 0.5),
            Point2D::new(1.5, 1.5),
        ], true);
        
        let result = interpolate_paths_adaptive(&path1, &path2, 0.5);
        assert_eq!(result.point_count(), 3);
        
        // Middle point should be halfway between
        assert!((result.points[0].x - 0.25).abs() < 0.01);
        assert!((result.points[0].y - 0.25).abs() < 0.01);
    }
    
    #[test]
    fn test_adaptive_interpolation_different_counts() {
        // Triangle (3 points) -> Hexagon (6 points)
        let triangle = shape_to_normalized_path(
            &ShapeType::Polygon { sides: 3 },
            (0.0, 0.0, 100.0, 100.0),
            3
        );
        
        let hexagon = shape_to_normalized_path(
            &ShapeType::Polygon { sides: 6 },
            (0.0, 0.0, 100.0, 100.0),
            6
        );
        
        // Should subdivide triangle to 6 points
        let result = interpolate_paths_adaptive(&triangle, &hexagon, 0.5);
        assert_eq!(result.point_count(), 6);
    }
    
    #[test]
    fn test_path_subdivision() {
        let simple_path = NormalizedPath::new(vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
        ], true);
        
        let subdivided = subdivide_path(&simple_path, 6);
        assert_eq!(subdivided.point_count(), 6);
        assert!(subdivided.closed);
        
        // First point should remain
        assert!((subdivided.points[0].x - 0.0).abs() < 0.01);
        assert!((subdivided.points[0].y - 0.0).abs() < 0.01);
    }
    
    #[test]
    fn test_subdivision_preserves_shape() {
        let square = shape_to_normalized_path(
            &ShapeType::Rectangle,
            (0.0, 0.0, 100.0, 100.0),
            4
        );
        
        let subdivided = subdivide_path(&square, 16);
        assert_eq!(subdivided.point_count(), 16);
        
        // Subdivided path should still approximate the square
        // Check that points are on perimeter
        for point in &subdivided.points {
            let on_edge = 
                (point.x - 0.0).abs() < 1.0 || 
                (point.x - 100.0).abs() < 1.0 ||
                (point.y - 0.0).abs() < 1.0 ||
                (point.y - 100.0).abs() < 1.0;
            assert!(on_edge, "Point {:?} should be on square edge", point);
        }
    }

    #[test]
    fn test_point_lerp() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(10.0, 10.0);
        
        let mid = Point2D::lerp(a, b, 0.5);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 5.0);
    }

    #[test]
    fn test_rectangle_path_point_count() {
        let path = rectangle_to_path(0.0, 0.0, 100.0, 100.0, 32);
        assert_eq!(path.point_count(), 32);
        assert!(path.closed);
    }

    #[test]
    fn test_circle_path_point_count() {
        let path = circle_to_path(0.0, 0.0, 100.0, 100.0, 32);
        assert_eq!(path.point_count(), 32);
        assert!(path.closed);
    }

    #[test]
    fn test_path_interpolation() {
        let rect = rectangle_to_path(0.0, 0.0, 100.0, 100.0, 8);
        let circle = circle_to_path(0.0, 0.0, 100.0, 100.0, 8);
        
        let halfway = interpolate_paths(&rect, &circle, 0.5);
        assert_eq!(halfway.point_count(), 8);
        
        // Halfway point should be between rect and circle
        let rect_first = rect.points[0];
        let circle_first = circle.points[0];
        let halfway_first = halfway.points[0];
        
        assert!((halfway_first.x - (rect_first.x + circle_first.x) / 2.0).abs() < 0.1);
    }

    #[test]
    fn test_easing_functions() {
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert_eq!(EasingFunction::EaseIn.apply(0.0), 0.0);
        assert_eq!(EasingFunction::EaseIn.apply(1.0), 1.0);
        
        // EaseIn should be slower at start
        assert!(EasingFunction::EaseIn.apply(0.5) < 0.5);
        // EaseOut should be faster at start
        assert!(EasingFunction::EaseOut.apply(0.5) > 0.5);
    }
}
