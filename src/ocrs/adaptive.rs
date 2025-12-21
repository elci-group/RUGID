use super::projection::Point3D;
use super::weight::Weight;

/// Scene complexity metrics for adaptive surface optimization
#[derive(Debug, Clone)]
pub struct SceneComplexity {
    /// Total number of lattice nodes in scene
    pub total_nodes: usize,
    
    /// Number of nodes in inversion range (high curvature)
    pub inversion_nodes: usize,
    
    /// Mesh detail level (variance in heights, 0.0-1.0)
    pub mesh_detail_level: f64,
    
    /// Observer distance from target (meters)
    pub observer_distance: f64,
    
    /// Target frame time budget (milliseconds)
    pub target_frame_time_ms: f64,
}

impl SceneComplexity {
    /// Create complexity metrics from scene state
    pub fn new(
        total_nodes: usize,
        inversion_nodes: usize,
        mesh_variance: f64,
        observer_distance: f64,
    ) -> Self {
        Self {
            total_nodes,
            inversion_nodes,
            mesh_detail_level: mesh_variance.clamp(0.0, 1.0),
            observer_distance,
            target_frame_time_ms: 16.67, // 60 FPS default
        }
    }
    
    /// Recommend subdivision level based on complexity heuristics
    ///
    /// Heuristic rules:
    /// - Far observer + few nodes → Low subdivision (faster)
    /// - Close observer + many nodes → High subdivision (quality)
    /// - Consider mesh detail for adaptive quality
    pub fn recommend_subdivision(&self) -> usize {
        // Base recommendation on node count and distance
        let base_subdivision = match (self.inversion_nodes, self.observer_distance) {
            // Very far or very few nodes → Icosahedron (20 facets)
            (n, d) if n < 50 || d > 100.0 => 0,
            
            // Far or few nodes → Sub 1 (80 facets)
            (n, d) if n < 200 || d > 50.0 => 1,
            
            // Mid-range → Sub 2 (320 facets)
            (n, d) if n < 1000 || d > 25.0 => 2,
            
            // Close and dense → Sub 3 (1280 facets)
            _ => 3,
        };
        
        // Adjust for mesh detail
        let detail_adjustment = if self.mesh_detail_level > 0.7 && base_subdivision < 3 {
            1 // Increase subdivision for high-detail meshes
        } else if self.mesh_detail_level < 0.3 && base_subdivision > 0 {
            -1 // Decrease for low-detail meshes
        } else {
            0
        };
        
        (base_subdivision as i32 + detail_adjustment).clamp(0, 3) as usize
    }
    
    /// Estimate performance cost for a given subdivision level
    pub fn estimate_cost(&self, subdivisions: usize) -> f64 {
        let facet_count = 20 * 4_usize.pow(subdivisions as u32);
        let reflection_cost = self.inversion_nodes as f64 * facet_count as f64 * 0.001;
        let solver_cost = (self.total_nodes + self.inversion_nodes * facet_count / 4) as f64 * 0.002;
        
        reflection_cost + solver_cost
    }
    
    /// Check if subdivision level fits within performance budget
    pub fn fits_budget(&self, subdivisions: usize) -> bool {
        self.estimate_cost(subdivisions) <= self.target_frame_time_ms
    }
}

impl Weight {
    /// Create weight with adaptive subdivision based on scene complexity
    pub fn adaptive(
        position: Point3D,
        radius: f64,
        scene: &SceneComplexity,
    ) -> Self {
        let subdivisions = scene.recommend_subdivision();
        Self::faceted(position, radius, subdivisions)
    }
    
    /// Adapt existing weight to new scene complexity
    ///
    /// Only regenerates facets if subdivision level changes significantly
    pub fn adapt_to_scene(&mut self, scene: &SceneComplexity) {
        let recommended = scene.recommend_subdivision();
        
        let current_subdivisions = match &self.surface {
            super::weight::SurfaceGeometry::Smooth => 0,
            super::weight::SurfaceGeometry::Faceted { subdivisions, .. } => *subdivisions,
        };
        
        // Only regenerate if recommendation differs
        if current_subdivisions != recommended {
            let facets = super::weight::generate_geodesic_facets(
                self.position,
                self.radius,
                recommended
            );
            
            self.surface = super::weight::SurfaceGeometry::Faceted {
                subdivisions: recommended,
                facets,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scene_complexity_recommendation() {
        // Far observer, few nodes → Low subdivision
        let simple_scene = SceneComplexity::new(1000, 10, 0.5, 150.0);
        assert_eq!(simple_scene.recommend_subdivision(), 0);
        
        // Close observer, many nodes → High subdivision
        let complex_scene = SceneComplexity::new(10000, 2000, 0.8, 20.0);
        assert!(complex_scene.recommend_subdivision() >= 2);
        
        // Mid-range
        let mid_scene = SceneComplexity::new(5000, 500, 0.5, 40.0);
        assert_eq!(mid_scene.recommend_subdivision(), 2);
    }
    
    #[test]
    fn test_mesh_detail_adjustment() {
        // High detail should increase subdivision
        let high_detail = SceneComplexity::new(1000, 100, 0.9, 60.0);
        let base_detail = SceneComplexity::new(1000, 100, 0.5, 60.0);
        
        assert!(high_detail.recommend_subdivision() >= base_detail.recommend_subdivision());
        
        // Low detail should decrease subdivision
        let low_detail = SceneComplexity::new(1000, 100, 0.2, 30.0);
        assert!(low_detail.recommend_subdivision() <= base_detail.recommend_subdivision());
    }
    
    #[test]
    fn test_performance_budget() {
        let scene = SceneComplexity::new(5000, 500, 0.5, 40.0);
        
        // Lower subdivisions should fit budget
        assert!(scene.fits_budget(0));
        assert!(scene.fits_budget(1));
        
        // Cost should increase with subdivision
        let cost_0 = scene.estimate_cost(0);
        let cost_1 = scene.estimate_cost(1);
        let cost_2 = scene.estimate_cost(2);
        
        assert!(cost_1 > cost_0);
        assert!(cost_2 > cost_1);
    }
    
    #[test]
    fn test_adaptive_weight_creation() {
        let scene = SceneComplexity::new(5000, 500, 0.6, 40.0);
        let weight = Weight::adaptive(Point3D::zero(), 5.0, &scene);
        
        let expected_sub = scene.recommend_subdivision();
        assert_eq!(weight.facet_count(), 20 * 4_usize.pow(expected_sub as u32));
    }
}
