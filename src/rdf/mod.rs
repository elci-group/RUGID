//! RDF (RUGID Definition File) Extrapolator
//!
//! This module parses `.rdf` files and converts them to the RUGID ontology system.
//!
//! # Architecture
//!
//! ```text
//! .rdf file → Lexer → Tokens → Parser → RdfAst → Converter → OntologicalNode
//! ```

mod lexer;
mod parser;
mod ast;
mod converter;
mod animation;
pub mod loader;

pub use lexer::{Lexer, Token, TokenType};
pub use parser::Parser;
pub use ast::{RdfNode, RdfDocument, ElementClass, AttributeValue};
pub use converter::RdfConverter;
pub use animation::{AnimationType, AnimationSpec, parse_animation};
pub use loader::{RdfLoader, LoadedScene, SceneMetadata, RdfExtension, AttributeAction, extract_animations_from_rdf};

use std::path::Path;
use std::fs;

/// Parse an RDF file and convert to OntologicalNode tree
pub fn parse_file(path: impl AsRef<Path>) -> Result<crate::ontology::OntologicalNode, RdfError> {
    let content = fs::read_to_string(path.as_ref())
        .map_err(|e| RdfError::Io(e.to_string()))?;
    parse_string(&content)
}

/// Parse RDF content from a string
pub fn parse_string(content: &str) -> Result<crate::ontology::OntologicalNode, RdfError> {
    let lexer = Lexer::new(content);
    let tokens: Vec<Token> = lexer.collect();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    let converter = RdfConverter::new();
    converter.convert(ast)
}

/// Errors that can occur during RDF processing
#[derive(Debug, Clone)]
pub enum RdfError {
    Io(String),
    Lexer(String, usize),      // message, line
    Parser(String, usize),     // message, line
    Validation(String, usize), // message, line
    Conversion(String),
}

impl std::fmt::Display for RdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RdfError::Io(msg) => write!(f, "IO Error: {}", msg),
            RdfError::Lexer(msg, line) => write!(f, "Lexer Error (line {}): {}", line, msg),
            RdfError::Parser(msg, line) => write!(f, "Parser Error (line {}): {}", line, msg),
            RdfError::Validation(msg, line) => write!(f, "Validation Error (line {}): {}", line, msg),
            RdfError::Conversion(msg) => write!(f, "Conversion Error: {}", msg),
        }
    }
}

impl std::error::Error for RdfError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_parse() {
        let rdf = r#"
window:::title[Test]::sdims[100,100]
	pane:::id[main]::parent[Test]::origin[0,0]::extent[1.0,1.0]
"#;
        let result = parse_string(rdf);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    }
}
