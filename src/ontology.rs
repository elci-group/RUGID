//! Ontological Indentation System
//!
//! This module implements coordinate-frame-based geometry where:
//! - Each element defines its own local coordinate space
//! - Children are expressed only in parent-relative terms
//! - No child may reference global dimensions directly
//! - Scaling becomes compositional, not procedural
//!
//! This is philosophically closer to scene graphs and relativistic
//! reference frames than CSS box models or constraint solvers.

use crate::cell::CellId;
use std::collections::HashMap;

/// A relative size expressed as a fraction/percentage of parent dimension.
///
/// This is the core encoding that makes ontological indentation work:
/// elements describe their geometry relative to their parent, never globally.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelativeSize {
    /// Fixed percentage of parent (0.0 - 1.0)
    /// Example: 0.08 means "8% of parent"
    Percent(f32),
    
    /// Flex weight - fills remaining space after fixed allocations
    /// Multiple flex children share space proportionally by weight
    Flex(f32),
    
    /// Maintain aspect ratio relative to the resolved other axis
    /// Value is the ratio: width / height
    Aspect(f32),
}

impl Default for RelativeSize {
    fn default() -> Self {
        RelativeSize::Percent(1.0)
    }
}

impl RelativeSize {
    /// Convenience for common percentage values
    pub fn pct(value: f32) -> Self {
        RelativeSize::Percent(value)
    }
    
    /// Convenience for flex with weight 1.0
    pub fn fill() -> Self {
        RelativeSize::Flex(1.0)
    }
}

/// Text alignment within a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
}

/// Vertical text alignment within a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextBaseline {
    Top,
    #[default]
    Middle,
    Bottom,
}

/// Relativistic text style - all sizing relative to parent.
///
/// Font size is expressed as a percentage of the parent's HEIGHT,
/// ensuring text scales compositionally with its container.
#[derive(Debug, Clone, PartialEq)]
pub struct RelativeText {
    /// The text content to render
    pub content: String,
    /// Font size as percentage of parent HEIGHT (0.0 - 1.0)
    /// Example: 0.5 means font is 50% of parent height
    pub font_size: f32,
    /// Horizontal alignment within the cell
    pub align: TextAlign,
    /// Vertical alignment within the cell
    pub baseline: TextBaseline,
    /// Text color (CSS color string)
    pub color: String,
    /// Optional font family
    pub font_family: Option<String>,
    /// Font weight ("normal", "bold", "100"-"900")
    pub font_weight: String,
}

impl Default for RelativeText {
    fn default() -> Self {
        Self {
            content: String::new(),
            font_size: 0.6,       // 60% of parent height
            align: TextAlign::Center,
            baseline: TextBaseline::Middle,
            color: "#FFFFFF".to_string(),
            font_family: None,
            font_weight: "normal".to_string(),
        }
    }
}

impl RelativeText {
    /// Create text with content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }
    
    /// Set font size as percentage of parent height
    pub fn size(mut self, pct: f32) -> Self {
        self.font_size = pct;
        self
    }
    
    /// Set alignment
    pub fn align(mut self, h: TextAlign, v: TextBaseline) -> Self {
        self.align = h;
        self.baseline = v;
        self
    }
    
    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }
    
    /// Set font family
    pub fn font(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }
    
    /// Set font weight
    pub fn weight(mut self, weight: impl Into<String>) -> Self {
        self.font_weight = weight.into();
        self
    }
    
    /// Convenience: left-aligned text
    pub fn left(mut self) -> Self {
        self.align = TextAlign::Start;
        self
    }
    
    /// Convenience: right-aligned text
    pub fn right(mut self) -> Self {
        self.align = TextAlign::End;
        self
    }

    /// Convenience: right-aligned text (alias)
    pub fn end(mut self) -> Self {
        self.align = TextAlign::End;
        self
    }

    /// Convenience: center-aligned text
    pub fn center(mut self) -> Self {
        self.align = TextAlign::Center;
        self
    }
    
    /// Convenience: bold text
    pub fn bold(mut self) -> Self {
        self.font_weight = "bold".to_string();
        self
    }
    
    /// Resolve font size to absolute pixels given parent height
    pub fn resolve_font_size(&self, parent_height: f32) -> f32 {
        parent_height * self.font_size
    }
    
    /// Get SVG text-anchor value
    pub fn svg_text_anchor(&self) -> &'static str {
        match self.align {
            TextAlign::Start => "start",
            TextAlign::Center => "middle",
            TextAlign::End => "end",
        }
    }
    
    /// Get SVG dominant-baseline value
    pub fn svg_baseline(&self) -> &'static str {
        match self.baseline {
            TextBaseline::Top => "hanging",
            TextBaseline::Middle => "central",
            TextBaseline::Bottom => "text-bottom",
        }
    }
}

/// Bounded relativistic constraints for inset positioning.
///
/// Expresses constraints like "20% < x < 80%" which creates a box
/// that starts at 20% of parent and ends at 80% of parent (i.e., 60% wide
/// with 20% margins on each side).
///
/// All values are relative to parent dimensions (0.0 - 1.0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativeBounds {
    /// Minimum position (start) as percentage of parent (0.0 - 1.0)
    pub min: f32,
    /// Maximum position (end) as percentage of parent (0.0 - 1.0)
    pub max: f32,
}

impl RelativeBounds {
    /// Create bounds from min/max percentages
    /// 
    /// Example: `RelativeBounds::new(0.2, 0.8)` means:
    /// - Start at 20% of parent
    /// - End at 80% of parent
    /// - Resulting size is 60% of parent
    /// - Creates 20% margin on each side
    pub fn new(min: f32, max: f32) -> Self {
        debug_assert!(min <= max, "min must be <= max");
        debug_assert!(min >= 0.0 && max <= 1.0, "bounds must be in 0.0-1.0 range");
        Self { min, max }
    }
    
    /// Create bounds that fill the parent (0.0 to 1.0)
    pub fn fill() -> Self {
        Self { min: 0.0, max: 1.0 }
    }
    
    /// Create centered inset bounds with equal margins
    /// 
    /// Example: `RelativeBounds::inset(0.1)` creates 10% margin on each side
    /// (starts at 10%, ends at 90%)
    pub fn inset(margin: f32) -> Self {
        Self::new(margin, 1.0 - margin)
    }
    
    /// Create asymmetric bounds
    /// 
    /// Example: `RelativeBounds::margins(0.1, 0.2)` creates:
    /// - 10% margin at start
    /// - 20% margin at end
    pub fn margins(start: f32, end: f32) -> Self {
        Self::new(start, 1.0 - end)
    }
    
    /// The computed origin (start position) as fraction of parent
    pub fn origin(&self) -> f32 {
        self.min
    }
    
    /// The computed extent (size) as fraction of parent
    pub fn extent(&self) -> f32 {
        self.max - self.min
    }
    
    /// Resolve to absolute values given parent size
    pub fn resolve(&self, parent_size: f32) -> (f32, f32) {
        let origin = self.min * parent_size;
        let extent = (self.max - self.min) * parent_size;
        (origin, extent)
    }
}

/// Local coordinate space definition for a node.
///
/// Defines how a node is positioned and sized relative to its parent:
/// - Position: origin_p (primary axis), origin_s (secondary axis), origin_z (depth axis)
/// - Size: extent_p (primary axis), extent_s (secondary axis), extent_z (depth axis)
/// - Rotation: rotation_deg (degrees, 2D), rotation_3d (3-axis rotation)
/// - Shape: shape (optional morphable shape definition)
///
/// All values are relativistic to parent dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalSpace {
    /// Position on primary axis (0.0 - 1.0, of parent's primary dimension)
    pub origin_p: f32,
    /// Position on secondary axis (0.0 - 1.0, of parent's secondary dimension)
    pub origin_s: f32,
    /// Position on depth axis (0.0 - 1.0, of parent's depth)
    pub origin_z: f32,
    /// Size on primary axis as relative size
    pub extent_p: RelativeSize,
    /// Size on secondary axis as relative size
    pub extent_s: RelativeSize,
    /// Size on depth axis as relative size
    pub extent_z: RelativeSize,
    /// Rotation in degrees (clockwise, 2D)
    pub rotation_deg: f32,
    /// 3D rotation (pitch, yaw, roll) in degrees
    pub rotation_3d: Option<crate::geometry3d::Rotation3D>,
    /// Optional shape definition for morphism
    pub shape: Option<crate::morphism::ShapeType>,
    /// Whether this element clips its children to its bounds
    pub clips_children: bool,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
    /// 3D translation velocity (x, y, z)
    pub motion_3d: Option<crate::geometry3d::Point3D>,
    /// 3D orbit configuration
    pub orbit_3d: Option<crate::motion3d::Orbit3DState>,
    /// Physics configuration
    pub physics: Option<crate::physics::core::PhysicsState>,
    /// Fill color (for RDF support)
    pub fill: Option<String>,
}

impl Default for LocalSpace {
    fn default() -> Self {
        Self {
            origin_p: 0.0,
            origin_s: 0.0,
            origin_z: 0.0,
            extent_p: RelativeSize::Percent(1.0),
            extent_s: RelativeSize::Percent(1.0),
            extent_z: RelativeSize::Percent(1.0),
            rotation_deg: 0.0,
            rotation_3d: None,
            shape: None,
            clips_children: false,
            opacity: 1.0,
            motion_3d: None,
            orbit_3d: None,
            physics: None,
            fill: None,
        }
    }
}

impl LocalSpace {
    /// Create a local space that fills the parent
    pub fn fill() -> Self {
        Self::default()
    }
    
    /// Convenience: Position this space at origin
    pub fn origin() -> Self {
        Self {
            origin_p: 0.0,
            origin_s: 0.0,
            ..Default::default()
        }
    }
    
    /// Create a local space with fixed percentage extents
    pub fn sized(extent_p: f32, extent_s: f32) -> Self {
        Self {
            extent_p: RelativeSize::Percent(extent_p),
            extent_s: RelativeSize::Percent(extent_s),
            ..Default::default()
        }
    }
    
    /// Set the shape for this local space
    pub fn with_shape(mut self, shape: crate::morphism::ShapeType) -> Self {
        self.shape = Some(shape);
        self
    }
    
    /// Create a local space from RelativeBounds on both axes.
    ///
    /// Example: `LocalSpace::bounded(RelativeBounds::inset(0.2), RelativeBounds::inset(0.2))`
    /// Creates a box with 20% margins on all sides.
    pub fn bounded(bounds_p: RelativeBounds, bounds_s: RelativeBounds) -> Self {
        Self {
            origin_p: bounds_p.origin(),
            origin_s: bounds_s.origin(),
            extent_p: RelativeSize::Percent(bounds_p.extent()),
            extent_s: RelativeSize::Percent(bounds_s.extent()),
            ..Default::default()
        }
    }
    
    /// Create a local space with equal inset margins on all sides.
    ///
    /// Example: `LocalSpace::inset(0.1)` creates a box with 10% margin on all sides.
    /// This is equivalent to: 10% < p < 90% AND 10% < s < 90%
    pub fn inset(margin: f32) -> Self {
        Self::bounded(RelativeBounds::inset(margin), RelativeBounds::inset(margin))
    }
    
    /// Create a local space with specified margins.
    ///
    /// Example: `LocalSpace::margins(0.1, 0.2, 0.1, 0.2)` creates:
    /// - 10% margin at top (primary start)
    /// - 20% margin at bottom (primary end)
    /// - 10% margin at left (secondary start)
    /// - 20% margin at right (secondary end)
    pub fn margins(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self::bounded(
            RelativeBounds::margins(top, bottom),
            RelativeBounds::margins(left, right),
        )
    }
    
    /// Set the origin position
    pub fn at(mut self, origin_p: f32, origin_s: f32) -> Self {
        self.origin_p = origin_p;
        self.origin_s = origin_s;
        self
    }
    
    /// Enable clipping
    pub fn clipped(mut self) -> Self {
        self.clips_children = true;
        self
    }
    
    /// Set rotation
    pub fn rotated(mut self, degrees: f32) -> Self {
        self.rotation_deg = degrees;
        self
    }
}

/// Content type for ontological nodes
#[derive(Debug, Clone)]
pub enum NodeContent {
    /// A visual cell that gets rendered (no text)
    Cell(CellId),
    /// A cell with associated text content
    TextCell {
        cell_id: CellId,
        text: RelativeText,
    },
    /// A 3D shape object
    Shape3D {
        cell_id: CellId,
        /// Shape type (Tetrahedron, Cube, Octahedron)
        shape_type: crate::shapes3d::ShapeType,
        /// Shape size as percentage of parent's minimum dimension
        size: f32,
    },
    /// A container with a label (for debugging)
    Container(String),
    /// Empty structural node
    Structural,
}

/// Direction for stacking children
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    /// Stack along primary axis (vertical in portrait, horizontal in landscape)
    Primary,
    /// Stack along secondary axis
    Secondary,
}

/// How a node is positioned within its parent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PositionMode {
    /// Participates in parent's stacking layout
    /// Origin values are ignored; position determined by stacking order
    #[default]
    Stacked,
    /// Positioned absolutely using LocalSpace origin values
    /// Does not affect sibling positions
    Absolute,
}

/// Asset classification for strict hierarchy enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssetClass {
    /// Class 1: The root window
    Window = 1,
    /// Class 2: A structural pane/container
    Pane = 2,
    /// Class 3: An interactive widget (button, field, etc.)
    Widget = 3,
    /// Class 4: Content (text, media)
    Content = 4,
}

/// An element that exists in its parent's coordinate ontology.
///
/// This is the core building block of ontological indentation:
/// - Each node defines a local coordinate space
/// - Children are expressed only within that space
/// - The hierarchy encodes the geometry relationship
#[derive(Debug, Clone)]
pub struct OntologicalNode {
    /// The classification of this asset
    pub class: AssetClass,
    /// Explicitly declared parent identifier (Name or CellId string)
    pub parent_id: Option<String>,
    /// Optional cell ID for leaf nodes
    pub content: NodeContent,
    /// How this node is positioned/sized within parent
    pub local_space: LocalSpace,
    /// How this node is positioned (stacked or absolute)
    pub position_mode: PositionMode,
    /// How children are arranged within this node
    pub stack_direction: StackDirection,
    /// Child nodes (exist in THIS node's coordinate space)
    pub children: Vec<OntologicalNode>,
}

impl OntologicalNode {
    /// Create a Class 1 Window (Root)
    pub fn window(name: &str, direction: StackDirection) -> Self {
        Self {
            class: AssetClass::Window,
            parent_id: None,
            content: NodeContent::Container(name.to_string()),
            local_space: LocalSpace {
                extent_p: RelativeSize::Percent(1.0),
                extent_s: RelativeSize::Percent(1.0),
                ..Default::default()
            },
            position_mode: PositionMode::Stacked,
            stack_direction: direction,
            children: Vec::new(),
        }
    }

    /// Create a Class 2 Pane (Container)
    pub fn pane(name: &str, parent: &str, direction: StackDirection, extent_p: RelativeSize, extent_s: RelativeSize) -> Self {
        Self {
            class: AssetClass::Pane,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Container(name.to_string()),
            local_space: LocalSpace {
                extent_p,
                extent_s,
                ..Default::default()
            },
            position_mode: PositionMode::Stacked,
            stack_direction: direction,
            children: Vec::new(),
        }
    }

    /// Create a Class 3 Widget (Leaf)
    pub fn widget(cell_id: CellId, parent: &str, extent_p: RelativeSize, extent_s: RelativeSize) -> Self {
        Self {
            class: AssetClass::Widget,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Cell(cell_id),
            local_space: LocalSpace {
                extent_p,
                extent_s,
                ..Default::default()
            },
            position_mode: PositionMode::Stacked,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }
    
    /// Create a Class 4 Content (Text)
    pub fn text_content(cell_id: CellId, parent: &str, text: RelativeText, extent_p: RelativeSize, extent_s: RelativeSize) -> Self {
        Self {
            class: AssetClass::Content,
            parent_id: Some(parent.to_string()),
            content: NodeContent::TextCell { cell_id, text },
            local_space: LocalSpace {
                extent_p,
                extent_s,
                ..Default::default()
            },
            position_mode: PositionMode::Stacked,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }

    /// Helper for inset text content (Class 4)
    pub fn text_inset(cell_id: CellId, parent: &str, text: RelativeText, margin: f32) -> Self {
        Self {
            class: AssetClass::Content,
            parent_id: Some(parent.to_string()),
            content: NodeContent::TextCell { cell_id, text },
            local_space: LocalSpace::inset(margin),
            position_mode: PositionMode::Absolute, // Insets are usually absolute relative to parent
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }
    
    /// Create a 3D shape widget with rotation
    /// 
    /// # Arguments
    /// * `cell_id` - The cell ID for this shape
    /// * `parent` - Parent node name
    /// * `shape_type` - Type of shape (Tetrahedron, Cube, Octahedron)
    /// * `bounds_x` - X-axis bounds (0.0-1.0 range)
    /// * `bounds_y` - Y-axis bounds
    /// * `bounds_z` - Z-axis bounds  
    /// * `x_speed` - X-axis rotation speed (degrees/frame)
    /// * `y_speed` - Y-axis rotation speed
    /// * `z_speed` - Z-axis rotation speed
    pub fn shape_3d(
        cell_id: CellId,
        parent: &str,
        shape_type: crate::shapes3d::ShapeType,
        bounds_x: (f32, f32),
        bounds_y: (f32, f32),
        bounds_z: (f32, f32),
        x_speed: f32,
        y_speed: f32,
        z_speed: f32,
        opacity: f32,
    ) -> Self {
        let x_min = bounds_x.0;
        let x_max = bounds_x.1;
        let y_min = bounds_y.0;
        let y_max = bounds_y.1;
        let z_min = bounds_z.0;
        let z_max = bounds_z.1;
        
        // Calculate size as the extent of the bounds
        let size_x = x_max - x_min;
        let size_y = y_max - y_min;
        let size_z = z_max - z_min;
        
        // Use minimum dimension as the definitive size
        let size = size_x.min(size_y).min(size_z);
        
        Self {
            class: AssetClass::Widget,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Shape3D { cell_id, shape_type, size },
            local_space: LocalSpace {
                origin_p: y_min,  // Primary = Y
                origin_s: x_min,  // Secondary = X
                origin_z: z_min,  // Depth = Z
                extent_p: RelativeSize::Percent(size_y),
                extent_s: RelativeSize::Percent(size_x),
                extent_z: RelativeSize::Percent(size_z),
                rotation_3d: Some(crate::geometry3d::Rotation3D {
                    pitch: x_speed,
                    yaw: y_speed,
                    roll: z_speed,
                }),
                opacity,
                ..Default::default()
            },
            position_mode: PositionMode::Absolute,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }

    /// Create a Pyramid 3D node
    pub fn pyramid_3d(
        cell_id: CellId,
        parent: &str,
        bounds_x: (f32, f32),
        bounds_y: (f32, f32),
        bounds_z: (f32, f32),
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        opacity: f32,
    ) -> Self {
        Self::shape_3d(
            cell_id,
            parent,
            crate::shapes3d::ShapeType::Pyramid,
            bounds_x,
            bounds_y,
            bounds_z,
            rot_x,
            rot_y,
            rot_z,
            opacity,
        )
    }

    /// Legacy support for cube_3d (wraps shape_3d)
    pub fn cube_3d(
        cell_id: CellId,
        parent: &str,
        bounds_x: (f32, f32),
        bounds_y: (f32, f32),
        bounds_z: (f32, f32),
        x_speed: f32,
        y_speed: f32,
        z_speed: f32,
        opacity: f32,
    ) -> Self {
        Self::shape_3d(
            cell_id, parent, 
            crate::shapes3d::ShapeType::Cube, 
            bounds_x, bounds_y, bounds_z, 
            x_speed, y_speed, z_speed, opacity
        )
    }

    /// Create a Torus 3D node
    pub fn torus_3d(
        cell_id: CellId,
        parent: &str,
        bounds_x: (f32, f32),
        bounds_y: (f32, f32),
        bounds_z: (f32, f32),
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        opacity: f32,
    ) -> Self {
        Self::shape_3d(
            cell_id,
            parent,
            crate::shapes3d::ShapeType::Torus,
            bounds_x,
            bounds_y,
            bounds_z,
            rot_x,
            rot_y,
            rot_z,
            opacity,
        )
    }

    /// Create a 3D shape with rotation speeds using extent-based dimensions.
    /// This is a simpler interface for RDF support.
    pub fn shape3d_rotating(
        cell_id: CellId,
        parent: &str,
        shape_type: crate::shapes3d::ShapeType,
        extent_p: RelativeSize,
        extent_s: RelativeSize,
        size: f32,
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
    ) -> Self {
        Self {
            class: AssetClass::Widget,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Shape3D { cell_id, shape_type, size },
            local_space: LocalSpace {
                extent_p,
                extent_s,
                rotation_3d: Some(crate::geometry3d::Rotation3D {
                    pitch: rot_x,
                    yaw: rot_y,
                    roll: rot_z,
                }),
                ..Default::default()
            },
            position_mode: PositionMode::Stacked,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }

    /// Create an F1 Car 3D node
    pub fn f1_car_3d(
        cell_id: CellId,
        parent: &str,
        bounds_x: (f32, f32),
        bounds_y: (f32, f32),
        bounds_z: (f32, f32),
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        opacity: f32,
    ) -> Self {
        Self::shape_3d(
            cell_id,
            parent,
            crate::shapes3d::ShapeType::F1Car,
            bounds_x,
            bounds_y,
            bounds_z,
            rot_x,
            rot_y,
            rot_z,
            opacity,
        )
    }

    /// Create a Jabulani ball (2010 World Cup) 3D node
    pub fn jabulani_3d(
        cell_id: CellId,
        parent: &str,
        bounds_x: (f32, f32),
        bounds_y: (f32, f32),
        bounds_z: (f32, f32),
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        opacity: f32,
    ) -> Self {
        Self::shape_3d(
            cell_id,
            parent,
            crate::shapes3d::ShapeType::Jabulani,
            bounds_x,
            bounds_y,
            bounds_z,
            rot_x,
            rot_y,
            rot_z,
            opacity,
        )
    }

    /// Create a Mercedes-AMG W14 (2023) 3D node
    pub fn mercedes_w14_3d(
        cell_id: CellId,
        parent: &str,
        bounds_x: (f32, f32),
        bounds_y: (f32, f32),
        bounds_z: (f32, f32),
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        opacity: f32,
    ) -> Self {
        Self::shape_3d(
            cell_id,
            parent,
            crate::shapes3d::ShapeType::MercedesW14,
            bounds_x,
            bounds_y,
            bounds_z,
            rot_x,
            rot_y,
            rot_z,
            opacity,
        )
    }
    
    /// Helper for inset pane (Class 2)
    pub fn inset_pane(name: &str, parent: &str, margin: f32, direction: StackDirection) -> Self {
        Self {
            class: AssetClass::Pane,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Container(name.to_string()),
            local_space: LocalSpace::inset(margin),
            position_mode: PositionMode::Absolute,
            stack_direction: direction,
            children: Vec::new(),
        }
    }

    /// Helper for inset widget (Class 3)
    pub fn inset_widget(cell_id: CellId, parent: &str, margin: f32) -> Self {
        Self {
            class: AssetClass::Widget,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Cell(cell_id),
            local_space: LocalSpace::inset(margin),
            position_mode: PositionMode::Absolute,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }
    
    /// Create a leaf node with explicit bounds on both axes.
    /// Uses ABSOLUTE positioning.
    ///
    /// Example: `OntologicalNode::bounded_leaf(cell_id, bounds_p, bounds_s)`
    pub fn bounded_widget(cell_id: CellId, parent: &str, bounds_p: RelativeBounds, bounds_s: RelativeBounds) -> Self {
        Self {
            class: AssetClass::Widget,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Cell(cell_id),
            local_space: LocalSpace::bounded(bounds_p, bounds_s),
            position_mode: PositionMode::Absolute,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }
    
    /// Create a container node with explicit bounds.
    /// Uses ABSOLUTE positioning.
    pub fn bounded_pane(name: &str, parent: &str, bounds_p: RelativeBounds, bounds_s: RelativeBounds, direction: StackDirection) -> Self {
        Self {
            class: AssetClass::Pane,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Container(name.to_string()),
            local_space: LocalSpace::bounded(bounds_p, bounds_s),
            position_mode: PositionMode::Absolute,
            stack_direction: direction,
            children: Vec::new(),
        }
    }
    
    /// Create a bounded text leaf (absolute positioning with explicit bounds).
    pub fn bounded_text(cell_id: CellId, parent: &str, text: RelativeText, bounds_p: RelativeBounds, bounds_s: RelativeBounds) -> Self {
        Self {
            class: AssetClass::Content,
            parent_id: Some(parent.to_string()),
            content: NodeContent::TextCell { cell_id, text },
            local_space: LocalSpace::bounded(bounds_p, bounds_s),
            position_mode: PositionMode::Absolute,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }
    
    /// Create a circular widget (convenience for common shape)
    pub fn circular_widget(cell_id: CellId, parent: &str, extent_p: RelativeSize, extent_s: RelativeSize) -> Self {
        Self {
            class: AssetClass::Widget,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Cell(cell_id),
            local_space: LocalSpace {
                extent_p,
                extent_s,
                shape: Some(crate::morphism::ShapeType::Circle),
                ..Default::default()
            },
            position_mode: PositionMode::Stacked,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }
    
    /// Create a rounded rectangle widget
    pub fn rounded_widget(cell_id: CellId, parent: &str, extent_p: RelativeSize, extent_s: RelativeSize, radius: f32) -> Self {
        Self {
            class: AssetClass::Widget,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Cell(cell_id),
            local_space: LocalSpace {
                extent_p,
                extent_s,
                shape: Some(crate::morphism::ShapeType::RoundedRectangle { radius }),
                ..Default::default()
            },
            position_mode: PositionMode::Stacked,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }
    
    /// Create a widget with custom shape
    pub fn shaped_widget(cell_id: CellId, parent: &str, extent_p: RelativeSize, extent_s: RelativeSize, shape: crate::morphism::ShapeType) -> Self {
        Self {
            class: AssetClass::Widget,
            parent_id: Some(parent.to_string()),
            content: NodeContent::Cell(cell_id),
            local_space: LocalSpace {
                extent_p,
                extent_s,
                shape: Some(shape),
                ..Default::default()
            },
            position_mode: PositionMode::Stacked,
            stack_direction: StackDirection::Primary,
            children: Vec::new(),
        }
    }

    pub fn with_motion(mut self, dx: f32, dy: f32, dz: f32) -> Self {
        self.local_space.motion_3d = Some(crate::geometry3d::Point3D::new(dx, dy, dz));
        self
    }
    
    pub fn with_orbit(mut self, radius: f32, speed: f32, axis: crate::motion3d::OrbitAxis) -> Self {
        self.local_space.orbit_3d = Some(crate::motion3d::Orbit3DState::new(radius, speed, axis));
        self
    }

    pub fn with_physics(mut self, mass: f32) -> Self {
        self.local_space.physics = Some(crate::physics::core::PhysicsState::new(mass));
        self
    }

    pub fn with_material(mut self, restitution: f32, friction: f32, tension: f32) -> Self {
        if let Some(physics) = &mut self.local_space.physics {
            physics.material = crate::physics::core::Material {
                restitution,
                friction,
                surface_tension: tension,
            };
        }
        self
    }

    pub fn with_magnetism(mut self, strength: f32, axis: crate::geometry3d::Point3D) -> Self {
        if let Some(physics) = &mut self.local_space.physics {
            physics.magnetic_properties = Some(crate::physics::core::MagneticDipole {
                strength,
                orientation: axis,
            });
        }
        self
    }

    /// Transform a generic widget into a text cell
    pub fn with_text(mut self, text: RelativeText) -> Self {
        if let NodeContent::Cell(cell_id) = self.content {
            self.content = NodeContent::TextCell { cell_id, text };
            self.class = AssetClass::Content;
        } else {
            // If it's already a TextCell, just update the text
            if let NodeContent::TextCell { cell_id, .. } = self.content {
                self.content = NodeContent::TextCell { cell_id, text };
            } else {
                eprintln!("Warning: with_text called on non-cell node {:?}", self.content);
            }
        }
        self
    }

    /// Add a child node with validation
    pub fn child(mut self, child: OntologicalNode) -> Self {
        // Validate hierarchy
        // Class 1 -> Class 2
        // Class 2 -> Class 2 or Class 3
        // Class 3 -> Class 4
        // Class 4 -> None (Leaf)
        
        let valid = match self.class {
            AssetClass::Window => child.class == AssetClass::Pane,
            AssetClass::Pane => child.class == AssetClass::Pane || child.class == AssetClass::Widget,
            AssetClass::Widget => child.class == AssetClass::Content || child.class == AssetClass::Widget, // Allow composite widgets
            AssetClass::Content => false, // Content cannot have children
        };
        
        if !valid {
            // We'll just print a warning for now to avoid crashing during development, 
            // but strictly this should be an error.
            eprintln!("WARNING: Invalid hierarchy! {:?} cannot contain {:?}", self.class, child.class);
        }
        
        // Validate explicit parent declaration
        if let Some(parent_id) = &child.parent_id {
            let name = match &self.content {
                NodeContent::Container(name) => name.clone(),
                NodeContent::Cell(id) => format!("cell_{}", id.0), // Approximate
                NodeContent::TextCell { cell_id, .. } => format!("cell_{}", cell_id.0),
                NodeContent::Shape3D { cell_id, .. } => format!("shape3d_{}", cell_id.0),
                NodeContent::Structural => "structural".to_string(),
            };
            
            // Note: This check is tricky because CellIds are generated. 
            // For Containers (Panes), we can check names.
            if let NodeContent::Container(my_name) = &self.content {
                if my_name != parent_id {
                     eprintln!("WARNING: Parent mismatch! Child declares parent '{}' but is being added to '{}'", parent_id, my_name);
                }
            }
        }

        self.children.push(child);
        self
    }
    
    /// Add multiple children
    pub fn children(mut self, nodes: Vec<OntologicalNode>) -> Self {
        self.children.extend(nodes);
        self
    }
    
    /// Set clipping on this node
    pub fn clipped(mut self) -> Self {
        self.local_space.clips_children = true;
        self
    }
}

/// Resolved absolute transform computed from ontological hierarchy.
///
/// This is the OUTPUT of the resolution process - never stored in nodes,
/// only computed when needed for rendering or hit testing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedTransform {
    /// Absolute X position in screen coordinates
    pub x: f32,
    /// Absolute Y position in screen coordinates
    pub y: f32,
    /// Absolute width in screen coordinates
    pub width: f32,
    /// Absolute height in screen coordinates
    pub height: f32,
    /// Total accumulated rotation
    pub rotation: f32,
    /// Clipping rectangle if any ancestor clips
    pub clip: Option<ClipRect>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ResolvedTransform {
    /// Create a new resolved transform with explicit values
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            rotation: 0.0,
            clip: None,
        }
    }
    
    /// Create a screen-space root transform
    pub fn screen(width: f32, height: f32) -> Self {
        Self::new(0.0, 0.0, width, height)
    }
    
    /// Get primary axis size (assumes portrait: primary = height)
    pub fn primary(&self) -> f32 {
        self.height
    }
    
    /// Get secondary axis size (assumes portrait: secondary = width)
    pub fn secondary(&self) -> f32 {
        self.width
    }
}

/// Resolved text with absolute font size (calculated from parent height)
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedText {
    /// The text content
    pub content: String,
    /// Font size in absolute pixels
    pub font_size_px: f32,
    /// X position for text (based on alignment)
    pub x: f32,
    /// Y position for text (based on baseline)
    pub y: f32,
    /// Horizontal alignment: "start", "middle", "end"
    pub text_anchor: String,
    /// Vertical alignment: "hanging", "central", "text-bottom"
    pub dominant_baseline: String,
    /// Text color
    pub color: String,
    /// Font family (optional)
    pub font_family: Option<String>,
    /// Font weight
    pub font_weight: String,
}

/// A fully resolved cell with transform and optional text
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCell {
    /// The cell's absolute transform
    pub transform: ResolvedTransform,
    /// Optional resolved text content
    pub text: Option<ResolvedText>,
    /// Optional 3D motion state
    pub motion_3d: Option<crate::geometry3d::Point3D>,
    /// Optional 3D orbit state
    pub orbit_3d: Option<crate::motion3d::Orbit3DState>,
    /// Optional physics state
    pub physics: Option<crate::physics::core::PhysicsState>,
    /// Optional fill color
    pub fill: Option<String>,
}

impl ResolvedCell {
    /// Create a cell without text
    pub fn from_transform(transform: ResolvedTransform) -> Self {
        Self { 
            transform, 
            text: None,
            motion_3d: None,
            orbit_3d: None,
            physics: None,
            fill: None,
        }
    }
    
    /// Create a cell with motion
    pub fn with_motion(
        transform: ResolvedTransform, 
        motion: Option<crate::geometry3d::Point3D>,
        orbit: Option<crate::motion3d::Orbit3DState>,
        physics: Option<crate::physics::core::PhysicsState>
    ) -> Self {
        Self {
            transform,
            text: None,
            motion_3d: motion,
            orbit_3d: orbit,
            physics,
            fill: None,
        }
    }
    
    /// Create a cell with text, resolving font size from the node's geometric height
    /// This strictly enforces l < fs < u logic where fs is determined by the bounds
    pub fn with_text(transform: ResolvedTransform, relative_text: &RelativeText, _parent_height: f32) -> Self {
        let font_size_px = transform.height;
        
        // Calculate text position based on alignment
        let x = match relative_text.align {
            TextAlign::Start => transform.x,
            TextAlign::Center => transform.x + transform.width / 2.0,
            TextAlign::End => transform.x + transform.width,
        };
        
        let y = match relative_text.baseline {
            TextBaseline::Top => transform.y,
            TextBaseline::Middle => transform.y + transform.height / 2.0,
            TextBaseline::Bottom => transform.y + transform.height,
        };
        
        Self {
            transform,
            text: Some(ResolvedText {
                content: relative_text.content.clone(),
                font_size_px,
                x,
                y,
                text_anchor: relative_text.svg_text_anchor().to_string(),
                dominant_baseline: relative_text.svg_baseline().to_string(),
                color: relative_text.color.clone(),
                font_family: relative_text.font_family.clone(),
                font_weight: relative_text.font_weight.clone(),
            }),
            motion_3d: None,
            orbit_3d: None,
            physics: None,
            fill: None,
        }
    }
}

/// Resolver for the ontological tree
pub struct OntologyResolver;

impl OntologyResolver {
    /// Resolve the entire tree, producing absolute transforms for all cells
    /// (Legacy method - does not include text information)
    pub fn resolve(
        root: &OntologicalNode,
        screen: ResolvedTransform,
    ) -> HashMap<CellId, ResolvedTransform> {
        let cells = Self::resolve_with_text(root, screen);
        cells.into_iter().map(|(id, cell)| (id, cell.transform)).collect()
    }
    
    /// Resolve the entire tree, producing ResolvedCells with text info
    pub fn resolve_with_text(
        root: &OntologicalNode,
        screen: ResolvedTransform,
    ) -> HashMap<CellId, ResolvedCell> {
        let mut results = HashMap::new();
        Self::resolve_node_with_text(root, &screen, &mut results);
        results
    }
    
    fn resolve_node_with_text(
        node: &OntologicalNode,
        parent: &ResolvedTransform,
        results: &mut HashMap<CellId, ResolvedCell>,
    ) {
        // Compute this node's transform
        let this_transform = Self::compute_transform(node, parent);
        
        // Record this node if it has a cell
        match &node.content {
            NodeContent::Cell(cell_id) => {
                let mut cell = ResolvedCell::from_transform(this_transform);
                cell.fill = node.local_space.fill.clone();
                results.insert(*cell_id, cell);
            }
            NodeContent::TextCell { cell_id, text } => {
                results.insert(*cell_id, ResolvedCell::with_text(this_transform, text, parent.height));
            }
            NodeContent::Shape3D { cell_id, .. } => {
                // Shape3D uses regular transform resolution
                results.insert(*cell_id, ResolvedCell::from_transform(this_transform));
            }
            _ => {}
        }
        
        // If no children, we're done
        if node.children.is_empty() {
            return;
        }
        
        // Separate children by position mode
        let (stacked_children, absolute_children): (Vec<_>, Vec<_>) = 
            node.children.iter().partition(|c| c.position_mode == PositionMode::Stacked);
        
        // Resolve stacked children with proper layout
        if !stacked_children.is_empty() {
            Self::resolve_stacked_children_with_text(&stacked_children, node.stack_direction, &this_transform, results);
        }
        
        // Resolve absolute children directly
        for child in absolute_children {
            Self::resolve_node_with_text(child, &this_transform, results);
        }
    }
    
    fn compute_transform(node: &OntologicalNode, parent: &ResolvedTransform) -> ResolvedTransform {
        let space = &node.local_space;
        
        // Resolve extents relative to parent
        let width = Self::resolve_size(&space.extent_s, parent.secondary(), parent.primary());
        let height = Self::resolve_size(&space.extent_p, parent.primary(), parent.secondary());
        
        // Compute position in parent space
        let x = parent.x + space.origin_s * parent.secondary();
        let y = parent.y + space.origin_p * parent.primary();
        
        // Accumulate rotation
        let rotation = parent.rotation + space.rotation_deg;
        
        // Handle clipping
        let clip = if space.clips_children {
            Some(ClipRect { x, y, width, height })
        } else {
            parent.clip
        };
        
        ResolvedTransform {
            x,
            y,
            width,
            height,
            rotation,
            clip,
        }
    }
    
    fn resolve_size(size: &RelativeSize, parent_size: f32, other_axis: f32) -> f32 {
        match size {
            RelativeSize::Percent(pct) => parent_size * pct,
            RelativeSize::Flex(_) => parent_size, // Flex resolved during stacking
            RelativeSize::Aspect(ratio) => other_axis * ratio,
        }
    }
    
    fn resolve_stacked_children_with_text(
        children: &[&OntologicalNode],
        stack_direction: StackDirection,
        parent: &ResolvedTransform,
        results: &mut HashMap<CellId, ResolvedCell>,
    ) {
        // Calculate space allocation for flex vs fixed children
        let axis_size = match stack_direction {
            StackDirection::Primary => parent.primary(),
            StackDirection::Secondary => parent.secondary(),
        };
        
        // First pass: calculate fixed allocations and flex weights
        let mut fixed_total = 0.0;
        let mut flex_total = 0.0;
        
        for child in children {
            let extent = match stack_direction {
                StackDirection::Primary => &child.local_space.extent_p,
                StackDirection::Secondary => &child.local_space.extent_s,
            };
            
            match extent {
                RelativeSize::Percent(pct) => fixed_total += axis_size * pct,
                RelativeSize::Flex(weight) => flex_total += weight,
                RelativeSize::Aspect(_) => {
                    // Aspect-ratio items need special handling
                    let other = match stack_direction {
                        StackDirection::Primary => parent.secondary(),
                        StackDirection::Secondary => parent.primary(),
                    };
                    let child_other = match stack_direction {
                        StackDirection::Primary => &child.local_space.extent_s,
                        StackDirection::Secondary => &child.local_space.extent_p,
                    };
                    let resolved_other = Self::resolve_size(child_other, other, axis_size);
                    if let RelativeSize::Aspect(ratio) = extent {
                        fixed_total += resolved_other * ratio;
                    }
                }
            }
        }
        
        let remaining = (axis_size - fixed_total).max(0.0);
        
        // Second pass: position children
        let mut current_pos = 0.0;
        
        for child in children {
            let extent = match stack_direction {
                StackDirection::Primary => &child.local_space.extent_p,
                StackDirection::Secondary => &child.local_space.extent_s,
            };
            
            // Calculate child size along stack axis
            let child_size = match extent {
                RelativeSize::Percent(pct) => axis_size * pct,
                RelativeSize::Flex(weight) => {
                    if flex_total > 0.0 {
                        remaining * (weight / flex_total)
                    } else {
                        0.0
                    }
                }
                RelativeSize::Aspect(ratio) => {
                    let other = match stack_direction {
                        StackDirection::Primary => parent.secondary(),
                        StackDirection::Secondary => parent.primary(),
                    };
                    let child_other = match stack_direction {
                        StackDirection::Primary => &child.local_space.extent_s,
                        StackDirection::Secondary => &child.local_space.extent_p,
                    };
                    let resolved_other = Self::resolve_size(child_other, other, axis_size);
                    resolved_other * ratio
                }
            };
            
            // Cross-axis size
            let cross_axis_size = match stack_direction {
                StackDirection::Primary => parent.secondary(),
                StackDirection::Secondary => parent.primary(),
            };
            
            let cross_extent = match stack_direction {
                StackDirection::Primary => &child.local_space.extent_s,
                StackDirection::Secondary => &child.local_space.extent_p,
            };
            
            let child_cross_size = Self::resolve_size(cross_extent, cross_axis_size, child_size);
            
            // Build child transform - stacked children ignore origin values
            let (child_x, child_y, child_w, child_h) = match stack_direction {
                StackDirection::Primary => {
                    (parent.x, parent.y + current_pos, child_cross_size, child_size)
                }
                StackDirection::Secondary => {
                    (parent.x + current_pos, parent.y, child_size, child_cross_size)
                }
            };
            
            let child_transform = ResolvedTransform {
                x: child_x,
                y: child_y,
                width: child_w,
                height: child_h,
                rotation: parent.rotation + child.local_space.rotation_deg,
                clip: if child.local_space.clips_children {
                    Some(ClipRect {
                        x: child_x,
                        y: child_y,
                        width: child_w,
                        height: child_h,
                    })
                } else {
                    parent.clip
                },
            };
            
            // Record cell with text info if applicable
            match &child.content {
                NodeContent::Cell(cell_id) => {
                    results.insert(*cell_id, ResolvedCell::with_motion(
                        child_transform, 
                        child.local_space.motion_3d.clone(),
                        child.local_space.orbit_3d.clone(),
                        child.local_space.physics.clone()
                    ));
                }
                NodeContent::TextCell { cell_id, text } => {
                    results.insert(*cell_id, ResolvedCell::with_text(child_transform, text, parent.height));
                }
                NodeContent::Shape3D { cell_id, .. } => {
                    results.insert(*cell_id, ResolvedCell::with_motion(
                        child_transform,
                        child.local_space.motion_3d.clone(),
                        child.local_space.orbit_3d.clone(),
                        child.local_space.physics.clone()
                    ));
                }
                _ => {}
            }
            
            // Recursively resolve grandchildren
            if !child.children.is_empty() {
                let (stacked_gc, absolute_gc): (Vec<_>, Vec<_>) = 
                    child.children.iter().partition(|c| c.position_mode == PositionMode::Stacked);
                
                if !stacked_gc.is_empty() {
                    Self::resolve_stacked_children_with_text(&stacked_gc, child.stack_direction, &child_transform, results);
                }
                
                for gc in absolute_gc {
                    Self::resolve_node_with_text(gc, &child_transform, results);
                }
            }
            
            current_pos += child_size;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_hierarchy() {
        // Screen -> Panel (50% height) -> Button (25% of panel)
        let button_id = CellId::next();
        
        let root = OntologicalNode::window("root", StackDirection::Primary)
            .child(
                OntologicalNode::pane("panel", "root", StackDirection::Primary, RelativeSize::Percent(0.5), RelativeSize::Percent(1.0))
                    .child(
                        OntologicalNode::widget(button_id, "panel", RelativeSize::Percent(0.25), RelativeSize::Percent(1.0))
                    )
            );
        
        let screen = ResolvedTransform::screen(1000.0, 800.0);
        let resolved = OntologyResolver::resolve(&root, screen);
        
        let button = resolved.get(&button_id).unwrap();
        
        // Panel is 50% of screen height (400px)
        // Button is 25% of panel height (100px)
        assert!((button.height - 100.0).abs() < 0.1, "Button height: {}", button.height);
        assert!((button.width - 1000.0).abs() < 0.1, "Button width: {}", button.width);
    }
    
    #[test]
    fn test_flex_allocation() {
        let cell1 = CellId::next();
        let cell2 = CellId::next();
        let cell3 = CellId::next();
        
        let root = OntologicalNode::window("root", StackDirection::Primary)
            .child(OntologicalNode::widget(cell1, "root", RelativeSize::Percent(0.1), RelativeSize::Percent(1.0))) // 10%
            .child(OntologicalNode::widget(cell2, "root", RelativeSize::Flex(1.0), RelativeSize::Percent(1.0)))    // Flex
            .child(OntologicalNode::widget(cell3, "root", RelativeSize::Percent(0.1), RelativeSize::Percent(1.0))); // 10%
        
        let screen = ResolvedTransform::screen(1000.0, 1000.0);
        let resolved = OntologyResolver::resolve(&root, screen);
        
        // cell1: 10% = 100px
        // cell3: 10% = 100px
        // cell2: remaining 80% = 800px
        
        assert!((resolved.get(&cell1).unwrap().height - 100.0).abs() < 0.1);
        assert!((resolved.get(&cell2).unwrap().height - 800.0).abs() < 0.1);
        assert!((resolved.get(&cell3).unwrap().height - 100.0).abs() < 0.1);
    }
    
    #[test]
    fn test_compositional_scaling() {
        // Demonstrate: child size is 10% of parent, parent is 50% of screen
        // Result: child is 5% of screen - but we never say that explicitly
        
        let inner_id = CellId::next();
        
        let root = OntologicalNode::window("root", StackDirection::Primary)
            .child(
                OntologicalNode::pane("outer", "root", StackDirection::Primary, RelativeSize::Percent(0.5), RelativeSize::Percent(1.0))
                    .child(
                        OntologicalNode::widget(inner_id, "outer", RelativeSize::Percent(0.1), RelativeSize::Percent(1.0))
                    )
            );
        
        let screen = ResolvedTransform::screen(1000.0, 1000.0);
        let resolved = OntologyResolver::resolve(&root, screen);
        
        let inner = resolved.get(&inner_id).unwrap();
        
        // 0.5 * 0.1 = 0.05 = 5% of 1000 = 50px
        assert!((inner.height - 50.0).abs() < 0.1, "Compositional: {}", inner.height);
    }
    
    #[test]
    fn test_relative_bounds() {
        // Test RelativeBounds calculations
        let bounds = RelativeBounds::new(0.2, 0.8);
        assert!((bounds.origin() - 0.2).abs() < 0.001);
        assert!((bounds.extent() - 0.6).abs() < 0.001);
        
        let (origin, extent) = bounds.resolve(1000.0);
        assert!((origin - 200.0).abs() < 0.1);
        assert!((extent - 600.0).abs() < 0.1);
    }
    
    #[test]
    fn test_relative_bounds_inset() {
        // Test inset convenience method
        let bounds = RelativeBounds::inset(0.1);
        assert!((bounds.min - 0.1).abs() < 0.001);
        assert!((bounds.max - 0.9).abs() < 0.001);
        assert!((bounds.extent() - 0.8).abs() < 0.001);
    }
    
    #[test]
    fn test_inset_leaf() {
        // Test inset_leaf: 20% margin on all sides
        // Parent is 1000x1000, so inner box should be:
        // - Position: (200, 200)
        // - Size: 600x600
        
        let inner_id = CellId::next();
        
        let root = OntologicalNode::window("root", StackDirection::Primary)
            .child(OntologicalNode::inset_widget(inner_id, "root", 0.2));
        
        let screen = ResolvedTransform::screen(1000.0, 1000.0);
        let resolved = OntologyResolver::resolve(&root, screen);
        
        let inner = resolved.get(&inner_id).unwrap();
        
        // Position should be at 20% = 200px
        assert!((inner.x - 200.0).abs() < 0.1, "Inner x: {}", inner.x);
        assert!((inner.y - 200.0).abs() < 0.1, "Inner y: {}", inner.y);
        
        // Size should be 60% = 600px
        assert!((inner.width - 600.0).abs() < 0.1, "Inner width: {}", inner.width);
        assert!((inner.height - 600.0).abs() < 0.1, "Inner height: {}", inner.height);
    }
    
    #[test]
    fn test_nested_inset() {
        // Nested insets: outer box has 10% margin, inner box has 20% margin within outer
        // Screen: 1000x1000
        // Outer: starts at 100, size 800 (10% margin)
        // Inner: starts at 100 + 160 = 260, size 480 (20% of 800 margin = 160)
        
        let inner_id = CellId::next();
        
        let root = OntologicalNode::window("root", StackDirection::Primary)
            .child(
                OntologicalNode::inset_pane("outer", "root", 0.1, StackDirection::Primary)
                    .child(OntologicalNode::inset_widget(inner_id, "outer", 0.2))
            );
        
        let screen = ResolvedTransform::screen(1000.0, 1000.0);
        let resolved = OntologyResolver::resolve(&root, screen);
        
        let inner = resolved.get(&inner_id).unwrap();
        
        // Outer: 10% margin = position 100, size 800
        // Inner within outer: 20% margin = position 160 (of 800) + 100 = 260
        // Inner size: 60% of 800 = 480
        
        assert!((inner.x - 260.0).abs() < 0.1, "Nested inner x: {}", inner.x);
        assert!((inner.y - 260.0).abs() < 0.1, "Nested inner y: {}", inner.y);
        assert!((inner.width - 480.0).abs() < 0.1, "Nested inner width: {}", inner.width);
        assert!((inner.height - 480.0).abs() < 0.1, "Nested inner height: {}", inner.height);
    }
    
    #[test]
    fn test_bounded_asymmetric() {
        // Test asymmetric bounds: 10% < p < 70%, 20% < s < 90%
        // This creates a box that's not centered
        
        let cell_id = CellId::next();
        
        let bounds_p = RelativeBounds::new(0.1, 0.7); // 10% to 70% = 60% height
        let bounds_s = RelativeBounds::new(0.2, 0.9); // 20% to 90% = 70% width
        
        let root = OntologicalNode::window("root", StackDirection::Primary)
            .child(OntologicalNode::bounded_widget(cell_id, "root", bounds_p, bounds_s));
        
        let screen = ResolvedTransform::screen(1000.0, 1000.0);
        let resolved = OntologyResolver::resolve(&root, screen);
        
        let cell = resolved.get(&cell_id).unwrap();
        
        // x: 20% of 1000 = 200
        // y: 10% of 1000 = 100
        // width: 70% of 1000 = 700
        // height: 60% of 1000 = 600
        
        assert!((cell.x - 200.0).abs() < 0.1, "Asymmetric x: {}", cell.x);
        assert!((cell.y - 100.0).abs() < 0.1, "Asymmetric y: {}", cell.y);
        assert!((cell.width - 700.0).abs() < 0.1, "Asymmetric width: {}", cell.width);
        assert!((cell.height - 600.0).abs() < 0.1, "Asymmetric height: {}", cell.height);
    }
    
    #[test]
    fn test_text_relative_sizing() {
        // Test that text font size is resolved relative to parent height
        // Parent height: 100px, font_size: 0.5 (50%) = 50px font
        
        let text_id = CellId::next();
        
        let root = OntologicalNode::window("root", StackDirection::Primary)
            .child(
                OntologicalNode::pane("main", "root", StackDirection::Primary, RelativeSize::Percent(1.0), RelativeSize::Percent(1.0))
                    .child(
                        OntologicalNode::text_content(
                            text_id,
                            "main",
                            RelativeText::new("Hello World")
                                .size(0.5)  // 50% of parent height
                                .bold()
                                .color("#FF0000"),
                            RelativeSize::Percent(0.1),  // 10% of screen height = 100px
                            RelativeSize::Percent(1.0),
                        )
                    )
            );
        
        let screen = ResolvedTransform::screen(1000.0, 1000.0);
        let resolved = OntologyResolver::resolve_with_text(&root, screen);
        
        let cell = resolved.get(&text_id).unwrap();
        
        // Cell should be 100px tall (10% of 1000)
        assert!((cell.transform.height - 100.0).abs() < 0.1, "Height: {}", cell.transform.height);
        
        // Text should be present
        let text = cell.text.as_ref().expect("Should have text");
        
        // Font size should be 50% of 100px = 50px
        assert!((text.font_size_px - 50.0).abs() < 0.1, "Font size: {}", text.font_size_px);
        assert_eq!(text.content, "Hello World");
        assert_eq!(text.font_weight, "bold");
        assert_eq!(text.color, "#FF0000");
    }
    
    #[test]
    fn test_text_inset_with_bounded_sizing() {
        // Test text in a bounded container - font scales with container size
        // Screen: 1000x1000
        // Bounds: 20%-80% on both axes = 600x600 container at position (200, 200)
        // Font: 0.4 = 40% of 600 = 240px
        
        let text_id = CellId::next();
        
        let bounds = RelativeBounds::new(0.2, 0.8);
        
        let root = OntologicalNode::window("root", StackDirection::Primary)
            .child(
                OntologicalNode::pane("main", "root", StackDirection::Primary, RelativeSize::Percent(1.0), RelativeSize::Percent(1.0))
                    .child(
                        OntologicalNode::bounded_text(
                            text_id,
                            "main",
                            RelativeText::new("Bounded Text").size(0.4),
                            bounds,
                            bounds,
                        )
                    )
            );
        
        let screen = ResolvedTransform::screen(1000.0, 1000.0);
        let resolved = OntologyResolver::resolve_with_text(&root, screen);
        
        let cell = resolved.get(&text_id).unwrap();
        let text = cell.text.as_ref().expect("Should have text");
        
        // FontSize should be 40% of 600px = 240px
        assert!((text.font_size_px - 240.0).abs() < 0.1, "Bounded font size: {}", text.font_size_px);
    }
}

