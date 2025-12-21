use std::collections::HashMap;
use crate::cell::{CellId, Rotation3DState};
use crate::temporal::DeltaFrame;
use crate::svg::{SvgNode, SvgAttribute};

pub struct Renderer {
    // In a real system, this would hold references to DOM nodes or a virtual DOM.
    // Here we store the state of each cell as an SvgNode.
    pub cell_states: HashMap<CellId, SvgNode>,
    render_order: Vec<CellId>,
    // Parent-child relationships for containment enforcement
    parent_map: HashMap<CellId, CellId>,
    // Initial bounds (x, y, width, height) for each cell
    initial_bounds: HashMap<CellId, (f32, f32, f32, f32)>,
    // 3D shape metadata: (shape_type, size, center_x, center_y)
    shape_metadata: HashMap<CellId, (crate::shapes3d::ShapeType, f32, f32, f32)>,
    // 3D rotation states (mutable state for automatic updates)
    rotation_states: HashMap<CellId, Rotation3DState>,
    // 3D translation states
    translation_states: HashMap<CellId, crate::motion3d::Translation3DState>,
    // 3D orbit states
    orbit_states: HashMap<CellId, crate::motion3d::Orbit3DState>,
    // Light sources in the scene
    lights: Vec<crate::physics::light::LightSource>,
    // Post-processing stack
    pub post_process: crate::filters::FilterStack,
}

#[derive(Clone)]
struct SceneObject {
    id: CellId,
    shape_type: crate::shapes3d::ShapeType,
    size: f32,
    center_x: f32,
    center_y: f32,
    position: crate::geometry3d::Point3D,
    rotation: crate::geometry3d::Rotation3D,
    opacity: f32,
    component_angle: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            cell_states: HashMap::new(),
            render_order: Vec::new(),
            parent_map: HashMap::new(),
            initial_bounds: HashMap::new(),
            shape_metadata: HashMap::new(),
            rotation_states: HashMap::new(),
            translation_states: HashMap::new(),
            orbit_states: HashMap::new(),
            lights: Vec::new(),
            post_process: crate::filters::FilterStack::default(),
        }
    }

    pub fn register_cell(&mut self, id: CellId, parent: Option<CellId>, initial_node: SvgNode) {
        // Extract and store initial bounds
        let bounds = match &initial_node {
            SvgNode::Rect { x, y, width, height, .. } => (*x, *y, *width, *height),
            SvgNode::Group { children, .. } => {
                // For groups, use the first rect as bounds (usually the background)
                if let Some(SvgNode::Rect { x, y, width, height, .. }) = children.first() {
                    (*x, *y, *width, *height)
                } else {
                    (0.0, 0.0, 0.0, 0.0)
                }
            },
            _ => (0.0, 0.0, 0.0, 0.0),
        };
        
        self.initial_bounds.insert(id, bounds);
        
        // Store parent relationship
        if let Some(parent_id) = parent {
            self.parent_map.insert(id, parent_id);
        }
        
        self.cell_states.insert(id, initial_node);
        self.render_order.push(id);
    }

    pub fn update_color(&mut self, id: CellId, color: String) {
        if let Some(node) = self.cell_states.get_mut(&id) {
            match node {
                SvgNode::Rect { .. } => node.set_fill(color),
                SvgNode::Group { children, .. } => {
                    // Find the background rect (usually the first child or any Rect)
                    for child in children {
                        if let SvgNode::Rect { .. } = child {
                            child.set_fill(color.clone());
                            // Assuming one background rect per cell
                            break; 
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    pub fn update_text_content(&mut self, id: CellId, new_content: String) {
        if let Some(node) = self.cell_states.get_mut(&id) {
            // Helper to find and update text
            fn update_node(node: &mut SvgNode, content: &str) {
                match node {
                    SvgNode::Text { content: c, .. } => *c = content.to_string(),
                    SvgNode::Group { children, .. } => {
                        for child in children {
                            update_node(child, content);
                        }
                    }
                    _ => {}
                }
            }
            update_node(node, &new_content);
        }
    }
    
    pub fn update_translation(&mut self, id: CellId, offset: crate::geometry3d::Point3D) {
        if let Some(state) = self.translation_states.get_mut(&id) {
            state.current = offset;
        } else {
            // If not registered, register it?
            // Ideally physics bodies should be registered with motion.
            // But if not, we can insert it.
            self.translation_states.insert(id, crate::motion3d::Translation3DState {
                current: offset,
                velocity: crate::geometry3d::Point3D::new(0.0, 0.0, 0.0),
            });
        }
    }

    /// Register a 3D shape cell
    pub fn register_shape_3d(
        &mut self,
        id: CellId,
        parent: Option<CellId>,
        shape_type: crate::shapes3d::ShapeType,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        size: f32,
        rotation_state: Rotation3DState,
    ) {
        // Calculate center of the cell for shape positioning
        let center_x = x + width / 2.0;
        let center_y = y + height / 2.0;
        
        // Store shape metadata
        self.shape_metadata.insert(id, (shape_type, size, center_x, center_y));
        self.rotation_states.insert(id, rotation_state);
        self.initial_bounds.insert(id, (x, y, width, height));
        
        if let Some(parent_id) = parent {
            self.parent_map.insert(id, parent_id);
        }
        
        // Create placeholder node (will be regenerated each frame)
        let placeholder = SvgNode::Group {
            attributes: vec![],
            transform: None,
            children: vec![],
        };
        
        self.cell_states.insert(id, placeholder);
        self.render_order.push(id);
    }

    pub fn register_motion_3d(
        &mut self,
        id: CellId,
        translation: Option<crate::motion3d::Translation3DState>,
        orbit: Option<crate::motion3d::Orbit3DState>,
    ) {
        if let Some(t) = translation {
            self.translation_states.insert(id, t);
        }
        if let Some(o) = orbit {
            self.orbit_states.insert(id, o);
        }
    }

    pub fn add_light(&mut self, light: crate::physics::light::LightSource) {
        self.lights.push(light);
    }

    /// Register a 3D pyramid cell (wrapper for register_shape_3d)
    pub fn register_pyramid_3d(
        &mut self,
        id: CellId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        size: f32,
        rotation_state: Rotation3DState,
    ) {
        self.register_shape_3d(
            id,
            None,
            crate::shapes3d::ShapeType::Pyramid,
            x,
            y,
            width,
            height,
            size,
            rotation_state,
        );
    }

    /// Legacy support for register_cube_3d
    pub fn register_cube_3d(
        &mut self,
        id: CellId,
        parent: Option<CellId>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        size: f32,
        rotation_state: Rotation3DState,
    ) {
        self.register_shape_3d(
            id, parent, 
            crate::shapes3d::ShapeType::Cube, 
            x, y, width, height, size, rotation_state
        )
    }
    
    /// Get mutable access to a 3D shape's properties for animation
    /// Returns (parent, shape_metadata, shape_type, center_x, center_y, width, height, size, rotation_state)
    pub fn get_shape_3d_mut(&mut self, id: CellId) -> Option<(
        Option<CellId>,
        Option<&mut (crate::shapes3d::ShapeType, f32, f32, f32)>,
        crate::shapes3d::ShapeType,
        &mut f32,  // center_x
        &mut f32,  // center_y
        f32,       // width
        f32,       // height
        f32,       // size
        &mut Rotation3DState,
    )> {
        if let Some(meta) = self.shape_metadata.get_mut(&id) {
            if let Some(rot) = self.rotation_states.get_mut(&id) {
                let parent = self.parent_map.get(&id).copied();
                let shape_type = meta.0;
                let size = meta.1;
                let bounds = self.initial_bounds.get(&id).copied().unwrap_or((0.0, 0.0, 100.0, 100.0));
                
                return Some((
                    parent,
                    None,
                    shape_type,
                    &mut meta.2,  // center_x
                    &mut meta.3,  // center_y
                    bounds.2,     // width
                    bounds.3,     // height
                    size,
                    rot,
                ));
            }
        }
        None
    }
    
    /// Update center position of a 3D shape
    pub fn update_shape_position(&mut self, id: CellId, center_x: f32, center_y: f32) {
        if let Some(meta) = self.shape_metadata.get_mut(&id) {
            meta.2 = center_x;
            meta.3 = center_y;
        }
    }
    
    /// Update rotation of a 3D shape (pitch, yaw, roll in degrees)
    pub fn update_shape_rotation(&mut self, id: CellId, pitch: f32, yaw: f32, roll: f32) {
        if let Some(rot) = self.rotation_states.get_mut(&id) {
            rot.current.pitch = pitch;
            rot.current.yaw = yaw;
            rot.current.roll = roll;
        }
    }
    
    /// Update all 3D rotation states (called each frame)
    pub fn update_3d_rotations(&mut self) {
        for rotation_state in self.rotation_states.values_mut() {
            rotation_state.update();
        }
        
        // Update translation states
        for translation_state in self.translation_states.values_mut() {
            translation_state.update();
        }
        
        // Update orbit states
        for orbit_state in self.orbit_states.values_mut() {
            orbit_state.update();
        }
    }

    fn render_scene_recursive(
        &self,
        objects: Vec<SceneObject>,
        scene: &crate::physics::optics::Scene,
        depth: usize,
    ) -> String {
        if depth > 2 { return String::new(); } // Max recursion depth
        
        let mut output = String::new();
        
        // Sort objects by depth (Painter's Algorithm)
        // We need to sort based on the furthest point or center?
        // Simple center-Z sorting for now.
        // Note: In RUGID, +Z is "away" (or "towards"?). 
        // shapes3d.rs says: "Camera is looking down -Z, so +Z is 'away' from camera"??
        // Wait, shapes3d.rs line 64: "View direction (0,0,1)".
        // Line 119: "View pos (0,0,-400)".
        // So Camera is at -400. Looking towards +Z.
        // Objects at 0 are in front. Objects at +100 are further away.
        // Painter's algorithm: Draw furthest first.
        // So sort by Z descending (Large Z first).
        
        let mut sorted_objects = objects.clone();
        sorted_objects.sort_by(|a, b| {
            b.position.z.partial_cmp(&a.position.z).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for obj in sorted_objects {
            // Render the object
            let shape_svg = crate::shapes3d::render_shape_animated(
                obj.shape_type,
                obj.center_x,
                obj.center_y,
                obj.size,
                obj.rotation,
                obj.opacity,
                obj.position,
                obj.component_angle,
                scene,
                None, // Recursion reflection not yet implemented
            );
            
            output.push_str(&shape_svg);
            
            // TODO: Reflection Logic Here
            // If obj is reflective, we need to:
            // 1. Define clip path (obj shape)
            // 2. Reflect all OTHER objects
            // 3. Recurse
        }
        
        output
    }

    pub fn render_tick(
        &mut self, 
        deltas: HashMap<CellId, DeltaFrame>,
        fluid_particles: Option<&Vec<crate::physics::FluidParticle>>
    ) -> String {
        let mut output = String::new();

        // PASS 1: Collect parent bounds for all cells that need clamping
        // This needs to happen *before* any deltas are applied to the parent,
        // so the parent's bounds are still in their pre-delta state.
        let mut cells_to_clamp: Vec<(CellId, (f32, f32, f32, f32))> = Vec::new();
        
        for (cell_id, _) in &deltas {
            if let Some(parent_id) = self.parent_map.get(cell_id).copied() {
                if let Some(parent_bounds) = self.cell_states.get(&parent_id).and_then(|parent_node| {
                    match parent_node {
                        SvgNode::Rect { x, y, width, height, .. } => Some((*x, *y, *width, *height)),
                        SvgNode::Group { children, .. } => {
                            if let Some(SvgNode::Rect { x, y, width, height, .. }) = children.first() {
                                Some((*x, *y, *width, *height))
                            } else {
                                None
                            }
                        },
                        _ => None,
                    }
                }) {
                    cells_to_clamp.push((*cell_id, parent_bounds));
                }
            }
        }

        // PASS 2: Apply deltas
        for (cell_id, delta) in deltas {
            if let Some(node) = self.cell_states.get_mut(&cell_id) {
                // Apply spatial deltas (scale by 100.0 as per Runtime convention)
                match node {
                    SvgNode::Rect { x, y, width, height, .. } => {
                        *x += delta.delta_origin_s * 100.0;
                        *y += delta.delta_origin_p * 100.0;
                        *width += delta.delta_extent_s * 100.0;
                        *height += delta.delta_extent_p * 100.0;
                    },
                    SvgNode::Path { ..} => {
                        // Paths don't have width/height, would need to scale path data
                        // TODO: Implement path scaling if needed
                    },
                    SvgNode::Group { children, .. } => {
                        // Iterate children and update them.
                        for child in children {
                            match child {
                                SvgNode::Rect { x, y, width, height, .. } => {
                                    *x += delta.delta_origin_s * 100.0;
                                    *y += delta.delta_origin_p * 100.0;
                                    *width += delta.delta_extent_s * 100.0;
                                    *height += delta.delta_extent_p * 100.0;
                                },
                                SvgNode::Text { x, y, .. } => {
                                    *x += delta.delta_origin_s * 100.0;
                                    *y += delta.delta_origin_p * 100.0;
                                    // Text doesn't have width/height usually in this model
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }
                
                // Apply color/opacity deltas (simplified)
                if delta.delta_a != 0.0 {
                     node.add_attribute(SvgAttribute::Opacity(1.0 + delta.delta_a)); 
                }
                
                // Apply RGB color deltas
                if delta.delta_r != 0.0 || delta.delta_g != 0.0 || delta.delta_b != 0.0 {
                    // Helper to apply color delta to a node
                    let apply_color_delta = |node: &mut SvgNode, dr: f32, dg: f32, db: f32| {
                        // Find and update the fill attribute
                        let fill_attr_option = match node {
                            SvgNode::Rect { attributes, .. } | 
                            SvgNode::Text { attributes, .. } |
                            SvgNode::Path { attributes, .. } => { // Added Path variant
                                attributes.iter_mut().find(|attr| matches!(attr, SvgAttribute::Fill(_)))
                            }
                            SvgNode::Group { children, .. } => {
                                // For groups, find fill attribute from the first child that has one
                                children.iter_mut().find_map(|child| {
                                    match child {
                                        SvgNode::Rect { attributes, .. } |
                                        SvgNode::Path { attributes, .. } => { // Added Path variant
                                            attributes.iter_mut().find(|a| matches!(a, SvgAttribute::Fill(_)))
                                        }
                                        _ => None
                                    }
                                })
                            }
                            SvgNode::RawSvg(_) => None,
                        };
                        
                        if let Some(SvgAttribute::Fill(color_str)) = fill_attr_option {
                            if let Some((r, g, b)) = crate::svg::parse_rgb_color(color_str) {
                                let new_r = (r + dr).clamp(0.0, 1.0);
                                let new_g = (g + dg).clamp(0.0, 1.0);
                                let new_b = (b + db).clamp(0.0, 1.0);
                                *color_str = crate::svg::rgb_to_string(new_r, new_g, new_b);
                            }
                        }
                    };
                    
                    // Apply to main node
                    apply_color_delta(node, delta.delta_r, delta.delta_g, delta.delta_b);
                    
                    // For groups, also apply to children (especially the background rect)
                    if let SvgNode::Group { children, .. } = node {
                        for child in children.iter_mut() {
                            apply_color_delta(child, delta.delta_r, delta.delta_g, delta.delta_b);
                        }
                    }
                }
            }
        }
        
        // PASS 3: CONTAINMENT ENFORCEMENT - Clamp cells to parent bounds
        for (cell_id, (parent_x, parent_y, parent_width, parent_height)) in cells_to_clamp {
            if let Some(node) = self.cell_states.get_mut(&cell_id) {
                match node {
                    SvgNode::Rect { x, y, width, height, .. } => {
                        // First, clamp size to parent bounds (if box grew too large)
                        if *width > parent_width {
                            *width = parent_width;
                        }
                        if *height > parent_height {
                            *height = parent_height;
                        }
                        
                        // Then clamp position to ensure entire box is within parent
                        *x = x.max(parent_x);
                        *y = y.max(parent_y);
                        *x = x.min(parent_x + parent_width - *width);
                        *y = y.min(parent_y + parent_height - *height);
                    }
                    SvgNode::Path { .. } => {
                        // Paths are already positioned, skip clamping for now
                        // TODO: Implement path bounding box clamping if needed
                    }
                    SvgNode::Group { children, .. } => {
                        // Assume the first rect is the background
                        if let Some(SvgNode::Rect { x, y, width, height, .. }) = children.get_mut(0) {
                            // First, clamp size to parent bounds (if box grew too large)
                            *width = (*width).min(parent_width);
                            *height = (*height).min(parent_height);
                            
                            // Then clamp position to ensure entire box is within parent
                            *x = (*x).max(parent_x);
                            *y = (*y).max(parent_y);
                            *x = (*x).min(parent_x + parent_width - *width);
                            *y = (*y).min(parent_y + parent_height - *height);
                        }
                        
                        // Also update text positions if present
                        for child in children.iter_mut() {
                            if let SvgNode::Text { x, y, .. } = child {
                                // Clamp text X/Y (simplified - just ensure within parent)
                                if *x < parent_x {
                                    *x = parent_x;
                                } else if *x > parent_x + parent_width {
                                    *x = parent_x + parent_width;
                                }
                                
                                if *y < parent_y {
                                    *y = parent_y;
                                } else if *y > parent_y + parent_height {
                                    *y = parent_y + parent_height;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // PASS 4: Render 3D shapes
        // PASS 4: Render 3D shapes
        // Collect all objects first
        let mut scene_objects = Vec::new();
        
        for (cell_id, (shape_type, size, center_x, center_y)) in &self.shape_metadata {
            if let Some(rotation_state) = self.rotation_states.get(cell_id) {
                // Calculate total offset
                let mut offset = crate::geometry3d::Point3D::new(0.0, 0.0, 0.0);
                
                if let Some(t) = self.translation_states.get(cell_id) {
                    offset.x += t.current.x;
                    offset.y += t.current.y;
                    offset.z += t.current.z;
                }
                
                if let Some(o) = self.orbit_states.get(cell_id) {
                    let orbit_offset = o.get_offset();
                    offset.x += orbit_offset.x;
                    offset.y += orbit_offset.y;
                    offset.z += orbit_offset.z;
                }
                
                scene_objects.push(SceneObject {
                    id: *cell_id,
                    shape_type: *shape_type,
                    size: *size,
                    center_x: *center_x,
                    center_y: *center_y,
                    position: offset,
                    rotation: rotation_state.current,
                    opacity: rotation_state.opacity,
                    component_angle: rotation_state.component_angle,
                });
            }
        }

        // Construct Scene for lighting
        let mut scene = crate::physics::optics::Scene::new();
        for light in &self.lights {
            scene.add_light(light.clone());
        }
        if scene.lights.is_empty() {
            scene.add_light(crate::physics::light::LightSource::directional(
                crate::geometry3d::Point3D::new(1.0, 1.0, 2.0),
                (255, 255, 255),
                1.0
            ));
        }

        // Render Scene Recursively
        let rendered_svg = self.render_scene_recursive(scene_objects.clone(), &scene, 0);
        
        // Update cell states with the rendered SVG
        // Wait, render_scene_recursive returns a single String of ALL shapes.
        // But the Renderer architecture expects each cell to have its own SvgNode in `cell_states`.
        // If I merge them into one SVG string, I break the cell structure (e.g. for hit testing or individual updates).
        
        // However, for correct 3D sorting and reflection, they MUST be rendered together.
        // The current architecture (independent cells) prevents correct 3D composition (Painter's algorithm across cells).
        // So, I should probably render them all into a single "Scene Layer" cell?
        // OR, I can just update the individual cell states, but that prevents reflections of one object onto another if they are separate SVG nodes.
        
        // Compromise:
        // For now, `render_scene_recursive` will just return the SVG strings.
        // But `render_tick` needs to put them back into `cell_states`.
        // Actually, if I want reflections, I need to render the reflection *inside* the mirror object's SVG.
        // So `render_scene_recursive` should probably return a map of `CellId -> String`?
        // No, because the reflection of Object A in Mirror B needs to be drawn *inside* Mirror B's SVG.
        
        // Let's stick to the plan:
        // `render_scene_recursive` returns the full SVG string of the scene.
        // But `Renderer` outputs `cell_states`.
        // I can create a "Virtual 3D Layer" cell that contains all 3D shapes?
        // Or I can just update the `cell_states` with the *individual* rendered shapes, but that loses the reflection context.
        
        // Wait, if I use `render_scene_recursive`, I am effectively rendering the whole 3D scene as one block.
        // I can insert this block as a `RawSvg` into a special "Scene" cell, or just append it to output.
        // But `render_tick` iterates `render_order`.
        
        // Let's modify `render_tick` to:
        // 1. Render 2D cells as usual.
        // 2. Render 3D scene as a composite layer.
        // 3. Combine.
        
        // But the user might want 3D objects mixed with 2D UI.
        // RUGID architecture seems to treat 3D shapes as individual cells.
        
        // If I want to support reflections, Object A needs to be rendered *twice*:
        // Once at its position, and once "inside" Mirror B.
        // The "inside Mirror B" version is part of Mirror B's visual representation.
        
        // So, when rendering Mirror B, we need to generate the SVG for Object A (reflected).
        // This means `render_shape_animated` for Mirror B needs to return the SVG for the mirror surface AND the reflected objects.
        
        // Okay, so `render_scene_recursive` is actually just a helper to generate the SVG for the *reflection*.
        // The primary rendering loop should still update individual cells.
        
        // REVISED PLAN for this block:
        // Iterate shapes.
        // For each shape:
        //   Check if it's a mirror.
        //   If yes:
        //     Call `render_scene_recursive` (with reflected objects) to get the "reflection SVG".
        //     Pass this "reflection SVG" to `render_shape_animated` (I need to add a param for it).
        //   If no:
        //     Pass empty string.
        
        // This keeps the cell architecture intact!
        
        for obj in &scene_objects {
             // Check if mirror (CornellBox is the only one for now, or maybe specific surfaces?)
             // Let's assume CornellBox walls are reflective?
             // Or maybe I add a specific "Mirror" shape type?
             // For now, let's say CornellBox is the test case.
             
             let reflection_svg = if obj.shape_type == crate::shapes3d::ShapeType::CornellBox {
                 // Cornell Box has reflective walls? Actually usually matte.
                 // Let's make a new shape "Mirror" or just hack it for now.
                 // Let's assume `Jabulani` is a mirror for testing.
                 String::new() // Placeholder
             } else {
                 String::new()
             };
             
             let shape_svg = crate::shapes3d::render_shape_animated(
                obj.shape_type,
                obj.center_x,
                obj.center_y,
                obj.size,
                obj.rotation,
                obj.opacity,
                obj.position,
                obj.component_angle,
                &scene,
                if !reflection_svg.is_empty() { Some(reflection_svg) } else { None },
            );
            
            let shape_node = SvgNode::RawSvg(shape_svg);
            self.cell_states.insert(obj.id, shape_node);
        }

        // Generate full SVG output
        output.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600">"#);
        
        // Add Definitions (Gradients, Filters)
        output.push_str("<defs>");
        
        // 1. Gooey Filter
        output.push_str(r#"
            <filter id="goo">
                <feGaussianBlur in="SourceGraphic" stdDeviation="10" result="blur" />
                <feColorMatrix in="blur" mode="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 19 -9" result="goo" />
                <feComposite in="SourceGraphic" in2="goo" operator="atop"/>
            </filter>
        "#);
        
        // 2. Fluid Particle Gradient
        output.push_str(r##"
            <radialGradient id="fluidGrad">
                <stop offset="0%" stop-color="#00aaff" stop-opacity="1" />
                <stop offset="100%" stop-color="#0055aa" stop-opacity="0" />
            </radialGradient>
        "##);
        
        output.push_str("</defs>");

        // Render Cells
        for cell_id in &self.render_order {
            if let Some(node) = self.cell_states.get(cell_id) {
                output.push_str(&node.to_string());
            }
        }
        
        // Render Fluid Particles (Overlay)
        if let Some(particles) = fluid_particles {
            output.push_str(r#"<g filter="url(#goo)">"#);
            for p in particles {
                // Render each particle as a circle
                // Radius is roughly half smoothing radius for visual overlap
                let r = 15.0; 
                output.push_str(&format!(
                    r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="url(#fluidGrad)" />"#,
                    p.x, p.y, r
                ));
            }
            output.push_str("</g>");
        }
        
        output.push_str("</svg>");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::CellId;

    #[test]
    fn test_renderer_tick() {
        let mut renderer = Renderer::new();
        let cell_id = CellId::next();
        
        let initial_node = SvgNode::Rect {
            attributes: vec![],
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        
        renderer.register_cell(cell_id, None, initial_node);

        let mut deltas = HashMap::new();
        let mut delta = DeltaFrame::zero(0, 1);
        delta.delta_a = -0.5;
        deltas.insert(cell_id, delta);

        let output = renderer.render_tick(deltas, None);
        
        assert!(output.contains("<svg>"));
        assert!(output.contains("<rect"));
        assert!(output.contains("opacity=\"0.5\"")); // 1.0 + (-0.5)
    }
}
