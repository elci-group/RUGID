//! RDF AST (Abstract Syntax Tree) definitions
//!
//! Represents the parsed structure of an RDF document before conversion to ontology.

use std::collections::HashMap;

/// The root document containing all parsed elements
#[derive(Debug, Clone)]
pub struct RdfDocument {
    /// All top-level nodes (typically windows)
    pub roots: Vec<RdfNode>,
}

/// A node in the RDF AST
#[derive(Debug, Clone)]
pub struct RdfNode {
    /// Element class (window, pane, widget, text, shape3d)
    pub element: ElementClass,
    /// Attributes parsed from the line
    pub attributes: HashMap<String, AttributeValue>,
    /// Child nodes (determined by indentation)
    pub children: Vec<RdfNode>,
    /// Source line number for error reporting
    pub line: usize,
    /// Indentation level
    pub indent: usize,
}

impl RdfNode {
    pub fn new(element: ElementClass, line: usize, indent: usize) -> Self {
        Self {
            element,
            attributes: HashMap::new(),
            children: Vec::new(),
            line,
            indent,
        }
    }
    
    /// Get a required string attribute
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).and_then(|v| {
            if let AttributeValue::String(s) = v {
                Some(s.as_str())
            } else {
                None
            }
        })
    }
    
    /// Get a required float attribute
    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.attributes.get(key).and_then(|v| {
            if let AttributeValue::Float(f) = v {
                Some(*f)
            } else {
                None
            }
        })
    }
    
    /// Get a point (x, y) attribute
    pub fn get_point(&self, key: &str) -> Option<(f32, f32)> {
        self.attributes.get(key).and_then(|v| {
            if let AttributeValue::Point2D(x, y) = v {
                Some((*x, *y))
            } else {
                None
            }
        })
    }
    
    /// Get a 3D point attribute
    pub fn get_point3d(&self, key: &str) -> Option<(f32, f32, f32)> {
        self.attributes.get(key).and_then(|v| {
            if let AttributeValue::Point3D(x, y, z) = v {
                Some((*x, *y, *z))
            } else {
                None
            }
        })
    }
    
    /// Check if this node has an animation
    pub fn has_animation(&self) -> bool {
        self.attributes.contains_key("move") ||
        self.attributes.contains_key("rotatex") ||
        self.attributes.contains_key("rotatey") ||
        self.attributes.contains_key("rotatez") ||
        self.attributes.contains_key("fade")
    }
}

/// Element class types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementClass {
    Window,
    Pane,
    Widget,
    Text,
    Shape3D,
}

impl ElementClass {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "window" => Some(Self::Window),
            "pane" => Some(Self::Pane),
            "widget" => Some(Self::Widget),
            "text" => Some(Self::Text),
            "shape3d" => Some(Self::Shape3D),
            _ => None,
        }
    }
}

/// Attribute value types
#[derive(Debug, Clone)]
pub enum AttributeValue {
    /// Simple string value
    String(String),
    /// Float value
    Float(f32),
    /// 2D point (x, y)
    Point2D(f32, f32),
    /// 3D point (x, y, z)
    Point3D(f32, f32, f32),
    /// Animation specification
    Animation(AnimationValue),
    /// Object with named properties (e.g., orbit[radius:0.2,speed:0.1])
    Object(HashMap<String, f32>),
    /// Flex sizing
    Flex,
}

/// Animation value with type and parameters
#[derive(Debug, Clone)]
pub struct AnimationValue {
    /// Singular (~), Rebound ({~}), or Sawtooth ([~])
    pub anim_type: AnimationModifier,
    /// Starting value(s)
    pub from: Vec<f32>,
    /// Ending value(s)
    pub to: Vec<f32>,
    /// Duration in frames (optional, for continuous animations)
    pub duration: Option<u64>,
}

/// Animation modifier type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationModifier {
    /// ~ : Move and stay
    Singular,
    /// {~} : Bounce back and forth
    Rebound,
    /// [~] : Teleport back to start
    Sawtooth,
    /// Continuous (for rotation speeds)
    Continuous,
}

impl std::fmt::Display for ElementClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Window => write!(f, "window"),
            Self::Pane => write!(f, "pane"),
            Self::Widget => write!(f, "widget"),
            Self::Text => write!(f, "text"),
            Self::Shape3D => write!(f, "shape3d"),
        }
    }
}
