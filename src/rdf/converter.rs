//! RDF to Ontology Converter
//!
//! Converts parsed RDF AST into the RUGID OntologicalNode tree.

use super::ast::{RdfDocument, RdfNode, ElementClass, AttributeValue};
use super::RdfError;
use crate::cell::CellId;
use crate::ontology::{
    OntologicalNode, RelativeSize, RelativeText,
    StackDirection, TextAlign, TextBaseline,
};
use std::collections::HashMap;

pub struct RdfConverter {
    /// Track generated IDs for parent lookups
    id_map: HashMap<String, CellId>,
}

impl RdfConverter {
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
        }
    }
    
    pub fn convert(&self, doc: RdfDocument) -> Result<OntologicalNode, RdfError> {
        if doc.roots.is_empty() {
            return Err(RdfError::Conversion("No root elements found".to_string()));
        }
        
        // For now, return the first window as root
        // In a multi-window scenario, we'd need a different approach
        let mut converter = RdfConverter::new();
        converter.convert_node(&doc.roots[0], None)
    }
    
    fn convert_node(&mut self, node: &RdfNode, parent_name: Option<&str>) -> Result<OntologicalNode, RdfError> {
        match node.element {
            ElementClass::Window => self.convert_window(node),
            ElementClass::Pane => self.convert_pane(node, parent_name),
            ElementClass::Widget => self.convert_widget(node, parent_name),
            ElementClass::Text => self.convert_text(node, parent_name),
            ElementClass::Shape3D => self.convert_shape3d(node, parent_name),
        }
    }
    
    fn convert_window(&mut self, node: &RdfNode) -> Result<OntologicalNode, RdfError> {
        let title = node.get_str("title").unwrap_or("Untitled");
        
        // Get stack direction
        let stack_dir = self.parse_stack_direction(node);
        
        let mut window = OntologicalNode::window(title, stack_dir);
        
        // Store ID for children to reference
        self.id_map.insert(title.to_string(), CellId::next());
        
        // Convert children
        for child in &node.children {
            let child_node = self.convert_node(child, Some(title))?;
            window = window.child(child_node);
        }
        
        Ok(window)
    }
    
    fn convert_pane(&mut self, node: &RdfNode, default_parent: Option<&str>) -> Result<OntologicalNode, RdfError> {
        let id = node.get_str("id").unwrap_or("unnamed_pane");
        let parent = node.get_str("parent").or(default_parent).unwrap_or("root");
        let stack_dir = self.parse_stack_direction(node);
        
        // Parse geometry
        let (extent_p, extent_s) = self.parse_extents(node);
        
        let mut pane = OntologicalNode::pane(id, parent, stack_dir, extent_p, extent_s);
        
        // Apply origin if specified
        if let Some((ox, oy)) = node.get_point("origin") {
            pane.local_space.origin_s = ox;
            pane.local_space.origin_p = oy;
        }
        
        // Store ID
        self.id_map.insert(id.to_string(), CellId::next());
        
        // Apply fill color via a background widget
        if let Some(fill) = node.get_str("fill") {
            let bg_id = CellId::next();
            let mut bg = OntologicalNode::widget(
                bg_id, 
                id, 
                RelativeSize::Percent(1.0), 
                RelativeSize::Percent(1.0)
            );
            bg.local_space.fill = Some(fill.to_string());
            bg.position_mode = crate::ontology::PositionMode::Absolute;
            pane = pane.child(bg);
        }
        
        // Convert children
        for child in &node.children {
            let child_node = self.convert_node(child, Some(id))?;
            pane = pane.child(child_node);
        }
        
        Ok(pane)
    }
    
    fn convert_widget(&mut self, node: &RdfNode, default_parent: Option<&str>) -> Result<OntologicalNode, RdfError> {
        let id = node.get_str("id").unwrap_or("unnamed_widget");
        let parent = node.get_str("parent").or(default_parent).unwrap_or("root");
        
        // Generate cell ID
        let cell_id = CellId::next();
        self.id_map.insert(id.to_string(), cell_id);
        
        // Parse geometry
        let (extent_p, extent_s) = self.parse_extents(node);
        
        let mut widget = OntologicalNode::widget(cell_id, parent, extent_p, extent_s);
        
        // Apply origin if specified (for absolute positioning)
        if let Some((ox, oy)) = node.get_point("origin") {
            widget.local_space.origin_s = ox;
            widget.local_space.origin_p = oy;
            widget.position_mode = crate::ontology::PositionMode::Absolute;
        }
        
        // Apply fill color
        if let Some(fill) = node.get_str("fill") {
            widget.local_space.fill = Some(fill.to_string());
        }
        
        // Apply shape
        if let Some(shape_str) = node.get_str("shape") {
            widget.local_space.shape = self.parse_shape(shape_str);
        }
        
        // Apply corner radius
        if let Some(radius) = node.get_float("radius") {
            widget.local_space.shape = Some(crate::morphism::ShapeType::RoundedRectangle { radius });
        }
        
        // Apply opacity
        if let Some(opacity) = node.get_float("opacity") {
            widget.local_space.opacity = opacity;
        }
        
        // Convert children (typically text content)
        for child in &node.children {
            let child_node = self.convert_node(child, Some(id))?;
            widget = widget.child(child_node);
        }
        
        Ok(widget)
    }
    
    fn convert_text(&mut self, node: &RdfNode, default_parent: Option<&str>) -> Result<OntologicalNode, RdfError> {
        let parent = node.get_str("parent").or(default_parent).unwrap_or("root");
        let cell_id = CellId::next();
        
        // Build RelativeText
        let content = node.get_str("value").unwrap_or("");
        let mut text = RelativeText::new(content);
        
        // Font size (as percentage of parent height)
        if let Some(size) = node.get_float("size") {
            text = text.size(size);
        }
        
        // Font family
        if let Some(font) = node.get_str("font") {
            text = text.font(font);
        }
        
        // Color
        if let Some(color) = node.get_str("color") {
            text = text.color(color);
        }
        
        // Weight
        if let Some(weight) = node.get_str("weight") {
            if weight.eq_ignore_ascii_case("bold") {
                text = text.bold();
            } else {
                text = text.weight(weight);
            }
        }
        
        // Alignment
        if let Some(align) = node.get_str("align") {
            let (h_align, v_align) = self.parse_alignment(align, node.get_str("baseline"));
            text = text.align(h_align, v_align);
        }
        
        // Create text content node
        // Text typically fills its parent
        let extent_p = node.get_float("extent_p")
            .map(RelativeSize::Percent)
            .unwrap_or(RelativeSize::Percent(1.0));
        let extent_s = node.get_float("extent_s")
            .map(RelativeSize::Percent)
            .unwrap_or(RelativeSize::Percent(1.0));
        
        Ok(OntologicalNode::text_content(cell_id, parent, text, extent_p, extent_s))
    }
    
    fn convert_shape3d(&mut self, node: &RdfNode, default_parent: Option<&str>) -> Result<OntologicalNode, RdfError> {
        let id = node.get_str("id").unwrap_or("unnamed_shape");
        let parent = node.get_str("parent").or(default_parent).unwrap_or("root");
        let cell_id = CellId::next();
        
        self.id_map.insert(id.to_string(), cell_id);
        
        // Parse shape type
        let shape_type = node.get_str("type")
            .map(|s| self.parse_3d_shape_type(s))
            .unwrap_or(crate::shapes3d::ShapeType::Cube);
        
        // Parse size
        let size = node.get_float("size").unwrap_or(0.2);
        
        // Parse opacity
        let opacity = node.get_float("opacity").unwrap_or(1.0);
        
        // Parse rotation speeds
        let rot_x = self.parse_rotation_speed(node, "rotatex");
        let rot_y = self.parse_rotation_speed(node, "rotatey");
        let rot_z = self.parse_rotation_speed(node, "rotatez");
        
        // Create the shape node
        let (extent_p, extent_s) = self.parse_extents(node);
        
        let mut shape_node = OntologicalNode::shape3d_rotating(
            cell_id,
            parent,
            shape_type,
            extent_p,
            extent_s,
            size,
            rot_x,
            rot_y,
            rot_z,
        );
        
        // Set opacity
        shape_node.local_space.opacity = opacity;
        
        // Handle origin for positioning
        if let Some((ox, oy)) = node.get_point("origin") {
            shape_node.local_space.origin_s = ox;
            shape_node.local_space.origin_p = oy;
            shape_node.position_mode = crate::ontology::PositionMode::Absolute;
        }
        
        // Handle orbit animation
        if let Some(AttributeValue::Object(orbit_params)) = node.attributes.get("orbit") {
            let radius = orbit_params.get("radius").copied().unwrap_or(0.1);
            let speed = orbit_params.get("speed").copied().unwrap_or(0.01);
            let axis_str = node.get_str("orbit_axis").unwrap_or("y");
            let axis = match axis_str {
                "x" => crate::motion3d::OrbitAxis::X,
                "z" => crate::motion3d::OrbitAxis::Z,
                _ => crate::motion3d::OrbitAxis::Y,
            };
            shape_node = shape_node.with_orbit(radius, speed, axis);
        }
        
        Ok(shape_node)
    }
    
    // Helper methods
    
    fn parse_extents(&self, node: &RdfNode) -> (RelativeSize, RelativeSize) {
        // Check for point-style extent
        if let Some((ex, ey)) = node.get_point("extent") {
            return (RelativeSize::Percent(ey), RelativeSize::Percent(ex));
        }
        
        // Check for flex
        if let Some(AttributeValue::Flex) = node.attributes.get("extent") {
            return (RelativeSize::Flex(1.0), RelativeSize::Flex(1.0));
        }
        
        // Check for individual extent values
        let extent_s = match node.attributes.get("extent_s") {
            Some(AttributeValue::Flex) => RelativeSize::Flex(1.0),
            Some(AttributeValue::Float(f)) => RelativeSize::Percent(*f),
            _ => {
                // Try to get from point
                if let Some((w, _)) = node.get_point("extent") {
                    RelativeSize::Percent(w)
                } else {
                    RelativeSize::Percent(1.0)
                }
            }
        };
        
        let extent_p = match node.attributes.get("extent_p") {
            Some(AttributeValue::Flex) => RelativeSize::Flex(1.0),
            Some(AttributeValue::Float(f)) => RelativeSize::Percent(*f),
            _ => {
                if let Some((_, h)) = node.get_point("extent") {
                    RelativeSize::Percent(h)
                } else {
                    RelativeSize::Percent(1.0)
                }
            }
        };
        
        (extent_p, extent_s)
    }
    
    fn parse_stack_direction(&self, node: &RdfNode) -> StackDirection {
        match node.get_str("stack") {
            Some("s") | Some("secondary") | Some("horizontal") => StackDirection::Secondary,
            _ => StackDirection::Primary,
        }
    }
    
    fn parse_shape(&self, shape_str: &str) -> Option<crate::morphism::ShapeType> {
        match shape_str.to_lowercase().as_str() {
            "circle" => Some(crate::morphism::ShapeType::Circle),
            "rounded" => Some(crate::morphism::ShapeType::RoundedRectangle { radius: 0.1 }),
            "rectangle" | "rect" => None, // Default
            _ => None,
        }
    }
    
    fn parse_3d_shape_type(&self, type_str: &str) -> crate::shapes3d::ShapeType {
        match type_str.to_lowercase().as_str() {
            "cube" => crate::shapes3d::ShapeType::Cube,
            "pyramid" => crate::shapes3d::ShapeType::Pyramid,
            "tetrahedron" => crate::shapes3d::ShapeType::Tetrahedron,
            "octahedron" => crate::shapes3d::ShapeType::Octahedron,
            "torus" => crate::shapes3d::ShapeType::Torus,
            "f1car" | "f1" => crate::shapes3d::ShapeType::F1Car,
            "jabulani" | "ball" => crate::shapes3d::ShapeType::Jabulani,
            "mercedes" | "w14" => crate::shapes3d::ShapeType::MercedesW14,
            "cornellbox" | "cornell" => crate::shapes3d::ShapeType::CornellBox,
            "sphere" => crate::shapes3d::ShapeType::Sphere,
            "icosahedron" | "dodecahedron" => crate::shapes3d::ShapeType::Jabulani,
            _ => crate::shapes3d::ShapeType::Cube,
        }
    }
    
    fn parse_rotation_speed(&self, node: &RdfNode, attr: &str) -> f32 {
        match node.attributes.get(attr) {
            Some(AttributeValue::Float(f)) => *f,
            Some(AttributeValue::Animation(anim)) => {
                // For continuous rotation, use the 'to' value as speed
                anim.to.first().copied().unwrap_or(0.0)
            }
            _ => 0.0,
        }
    }
    
    fn parse_alignment(&self, align: &str, baseline: Option<&str>) -> (TextAlign, TextBaseline) {
        let h_align = match align.to_lowercase().as_str() {
            "start" | "left" => TextAlign::Start,
            "center" | "middle" => TextAlign::Center,
            "end" | "right" => TextAlign::End,
            _ => TextAlign::Start,
        };
        
        let v_align = match baseline.unwrap_or("middle").to_lowercase().as_str() {
            "top" | "hanging" => TextBaseline::Top,
            "middle" | "central" => TextBaseline::Middle,
            "bottom" | "text-bottom" => TextBaseline::Bottom,
            _ => TextBaseline::Middle,
        };
        
        (h_align, v_align)
    }
}

impl Default for RdfConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Lexer, Parser};
    
    fn parse_and_convert(input: &str) -> Result<OntologicalNode, RdfError> {
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        let mut parser = Parser::new(tokens);
        let doc = parser.parse()?;
        let converter = RdfConverter::new();
        converter.convert(doc)
    }
    
    #[test]
    fn test_window_conversion() {
        let rdf = "window:::title[Test App]::sdims[100,100]";
        let result = parse_and_convert(rdf);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_nested_conversion() {
        let rdf = r#"
window:::title[App]::sdims[100,100]
	pane:::id[main]::parent[App]::extent[1.0,1.0]::stack[p]
		widget:::id[btn]::parent[main]::extent[0.1,0.05]::fill[#007acc]
"#;
        let result = parse_and_convert(rdf);
        assert!(result.is_ok(), "Conversion failed: {:?}", result.err());
        
        let root = result.unwrap();
        assert!(!root.children.is_empty(), "Window should have children");
    }
}
