//! RDF Loader Service
//!
//! High-level service that orchestrates loading RDF files and converting them
//! to runtime-ready scenes. Designed for extensibility with plugin-style hooks.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::cell::CellId;
use crate::ontology::{OntologicalNode, OntologyResolver, ResolvedTransform, ResolvedCell, NodeContent};
use crate::temporal::DeltaProgram;

use super::{RdfError, RdfDocument, RdfNode, RdfConverter, AttributeValue};

/// Result of loading an RDF file
#[derive(Debug)]
pub struct LoadedScene {
    /// The root ontological node
    pub root: OntologicalNode,
    /// All resolved cells ready for rendering
    pub cells: HashMap<CellId, ResolvedCell>,
    /// Animation programs to add to temporal memory
    pub animations: Vec<DeltaProgram>,
    /// Scene metadata extracted from the window element
    pub metadata: SceneMetadata,
}

/// Metadata about the loaded scene
#[derive(Debug, Clone, Default)]
pub struct SceneMetadata {
    /// Window title from the RDF
    pub title: Option<String>,
    /// Screen dimensions from sdims attribute
    pub dimensions: Option<(f32, f32)>,
    /// Background color
    pub background: Option<String>,
}

/// Action to take when handling a custom attribute
#[derive(Debug, Clone)]
pub enum AttributeAction {
    /// Set a property on the node
    SetProperty { key: String, value: String },
    /// Add a style class
    AddClass(String),
    /// Ignore this attribute
    Ignore,
}

/// Extension trait for adding custom element and attribute handlers
pub trait RdfExtension: Send + Sync {
    /// Called when an unknown element type is encountered
    fn handle_element(&self, _node: &RdfNode) -> Option<OntologicalNode> {
        None
    }
    
    /// Called when an unknown attribute is encountered
    fn handle_attribute(&self, _name: &str, _value: &AttributeValue) -> Option<AttributeAction> {
        None
    }
    
    /// Post-processing hook after conversion
    fn post_process(&self, _root: &mut OntologicalNode) {}
    
    /// Name of this extension for debugging
    fn name(&self) -> &str {
        "unnamed"
    }
}

/// High-level RDF loading service
pub struct RdfLoader {
    /// Registered extension hooks
    extensions: Vec<Box<dyn RdfExtension>>,
    /// Cache of parsed documents (for hot-reload)
    cache: HashMap<PathBuf, CachedDocument>,
}

struct CachedDocument {
    ast: RdfDocument,
    #[allow(dead_code)]
    modified_time: std::time::SystemTime,
}

impl RdfLoader {
    /// Create a new RDF loader
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            cache: HashMap::new(),
        }
    }
    
    /// Register a custom extension
    pub fn register_extension(&mut self, ext: Box<dyn RdfExtension>) {
        self.extensions.push(ext);
    }
    
    /// Load an RDF file and resolve it against a screen transform
    pub fn load(&mut self, path: &Path, screen: ResolvedTransform) -> Result<LoadedScene, RdfError> {
        // Parse the file
        let content = std::fs::read_to_string(path)
            .map_err(|e| RdfError::Io(format!("{}: {}", path.display(), e)))?;
        
        let ast = self.parse_content(&content)?;
        
        // Cache for potential hot-reload
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.cache.insert(path.to_path_buf(), CachedDocument {
                    ast: ast.clone(),
                    modified_time: modified,
                });
            }
        }
        
        self.convert_and_resolve(ast, screen)
    }
    
    /// Load from a string (useful for testing and embedded RDF)
    pub fn load_string(&self, content: &str, screen: ResolvedTransform) -> Result<LoadedScene, RdfError> {
        let ast = self.parse_content(content)?;
        self.convert_and_resolve(ast, screen)
    }
    
    /// Parse RDF content to AST
    fn parse_content(&self, content: &str) -> Result<RdfDocument, RdfError> {
        use super::lexer::Lexer;
        use super::parser::Parser;
        
        let lexer = Lexer::new(content);
        let tokens: Vec<_> = lexer.collect();
        
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    
    /// Convert AST and resolve to cells
    fn convert_and_resolve(&self, ast: RdfDocument, screen: ResolvedTransform) -> Result<LoadedScene, RdfError> {
        // Extract metadata from window elements
        let metadata = self.extract_metadata(&ast);
        
        // Convert to ontological tree
        let converter = RdfConverter::new();
        let mut root = converter.convert(ast)?;
        
        // Run post-processing extensions
        for ext in &self.extensions {
            ext.post_process(&mut root);
        }
        
        // Resolve to cells
        let cells = OntologyResolver::resolve_with_text(&root, screen);
        
        // Extract animations from the tree
        let animations = self.extract_all_animations(&root);
        
        Ok(LoadedScene {
            root,
            cells,
            animations,
            metadata,
        })
    }
    
    /// Extract metadata from the RDF document
    fn extract_metadata(&self, ast: &RdfDocument) -> SceneMetadata {
        let mut metadata = SceneMetadata::default();
        
        for root in &ast.roots {
            if root.element == super::ast::ElementClass::Window {
                metadata.title = root.get_str("title").map(|s| s.to_string());
                metadata.dimensions = root.get_point("sdims");
                metadata.background = root.get_str("fill").map(|s| s.to_string());
            }
        }
        
        metadata
    }
    
    /// Recursively extract animations from the ontology tree
    fn extract_all_animations(&self, node: &OntologicalNode) -> Vec<DeltaProgram> {
        let mut programs = Vec::new();
        
        // Get cell ID if this node has one
        let _cell_id = match &node.content {
            NodeContent::Cell(id) => Some(*id),
            NodeContent::TextCell { cell_id, .. } => Some(*cell_id),
            NodeContent::Shape3D { cell_id, .. } => Some(*cell_id),
            _ => None,
        };
        
        // Note: 3D shape rotation is handled by the renderer directly
        // Motion is handled by the renderer via local_space.motion_3d
        // Future: Convert motion_3d to DeltaPrograms for more control
        
        // Check for motion animations
        if node.local_space.motion_3d.is_some() {
            // Motion is continuous, rendered frame-by-frame
            // Could convert to DeltaPrograms for temporal control
        }
        
        // Recurse into children
        for child in &node.children {
            programs.extend(self.extract_all_animations(child));
        }
        
        programs
    }
}

impl Default for RdfLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract animations from an RDF node's attributes
pub fn extract_animations_from_rdf(node: &RdfNode, cell_id: CellId) -> Vec<DeltaProgram> {
    let mut programs = Vec::new();
    
    for (name, value) in &node.attributes {
        if let Some(spec) = super::animation::parse_animation(name, value) {
            programs.push(spec.to_delta_program(cell_id));
        }
    }
    
    programs
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_loader_basic() {
        let loader = RdfLoader::new();
        let screen = ResolvedTransform::new(0.0, 0.0, 800.0, 600.0);
        
        let rdf = r#"
window:::title[Test App]::sdims[800,600]
    pane:::id[main]::parent[Test App]::origin[0,0]::extent[1.0,1.0]
"#;
        
        let scene = loader.load_string(rdf, screen);
        assert!(scene.is_ok(), "Load failed: {:?}", scene.err());
        
        let scene = scene.unwrap();
        assert_eq!(scene.metadata.title.as_deref(), Some("Test App"));
    }
    
    #[test]
    fn test_metadata_extraction() {
        let loader = RdfLoader::new();
        let screen = ResolvedTransform::new(0.0, 0.0, 1920.0, 1080.0);
        
        let rdf = r#"
window:::title[My Application]::sdims[1920,1080]::fill[#1a1a2e]
"#;
        
        let scene = loader.load_string(rdf, screen).unwrap();
        assert_eq!(scene.metadata.title.as_deref(), Some("My Application"));
        assert_eq!(scene.metadata.dimensions, Some((1920.0, 1080.0)));
        assert_eq!(scene.metadata.background.as_deref(), Some("#1a1a2e"));
    }
}
