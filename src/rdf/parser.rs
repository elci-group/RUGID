//! RDF Parser - Builds AST from token stream
//!
//! Parses the hierarchical structure based on indentation and
//! attribute syntax.

use super::lexer::{Token, TokenType};
use super::ast::{RdfDocument, RdfNode, ElementClass, AttributeValue, AnimationValue, AnimationModifier};
use super::RdfError;
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    current_line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            current_line: 1,
        }
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    
    fn peek_type(&self) -> Option<TokenType> {
        self.peek().map(|t| t.token_type.clone())
    }
    
    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        if let Some(t) = token {
            self.current_line = t.line;
        }
        self.position += 1;
        token
    }
    
    fn expect(&mut self, expected: TokenType) -> Result<&Token, RdfError> {
        if let Some(token) = self.peek() {
            if token.token_type == expected {
                return Ok(self.advance().unwrap());
            }
            return Err(RdfError::Parser(
                format!("Expected {:?}, found {:?}", expected, token.token_type),
                token.line,
            ));
        }
        Err(RdfError::Parser("Unexpected end of input".to_string(), self.current_line))
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::Newline | TokenType::Comment => {
                    self.advance();
                }
                _ => break,
            }
        }
    }
    
    pub fn parse(&mut self) -> Result<RdfDocument, RdfError> {
        let mut roots = Vec::new();
        
        self.skip_whitespace_and_comments();
        
        while self.peek().is_some() {
            // Parse nodes at root level (indent = 0)
            if let Some(node) = self.parse_node_if_present(0)? {
                roots.push(node);
            } else {
                // Skip lines we can't parse
                self.skip_whitespace_and_comments();
                if self.peek().is_some() && self.peek_type() != Some(TokenType::Indent) 
                   && self.peek_type() != Some(TokenType::Element) {
                    self.advance();
                }
            }
            self.skip_whitespace_and_comments();
        }
        
        Ok(RdfDocument { roots })
    }
    
    fn parse_node_if_present(&mut self, expected_indent: usize) -> Result<Option<RdfNode>, RdfError> {
        self.skip_whitespace_and_comments();
        
        // Check indentation
        let actual_indent = if let Some(token) = self.peek() {
            if token.token_type == TokenType::Indent {
                let indent: usize = token.value.parse().unwrap_or(0);
                self.advance();
                indent
            } else {
                0
            }
        } else {
            return Ok(None);
        };
        
        // If indentation is less than expected, this node belongs to a parent
        if actual_indent < expected_indent {
            // Back up so parent can process
            self.position -= 1;
            return Ok(None);
        }
        
        // If indentation is more than expected, skip (orphaned content)
        if actual_indent > expected_indent {
            // Skip to end of line
            while let Some(token) = self.peek() {
                if token.token_type == TokenType::Newline {
                    break;
                }
                self.advance();
            }
            return self.parse_node_if_present(expected_indent);
        }
        
        // Parse the node at this level
        self.parse_node(actual_indent)
    }
    
    fn parse_node(&mut self, indent: usize) -> Result<Option<RdfNode>, RdfError> {
        // Get element name
        let element_token = match self.peek() {
            Some(t) if t.token_type == TokenType::Element => self.advance().unwrap().clone(),
            Some(t) if t.token_type == TokenType::AttributeName => {
                // Could be a continuation line (just attributes)
                // Skip for now
                while let Some(token) = self.peek() {
                    if token.token_type == TokenType::Newline {
                        break;
                    }
                    self.advance();
                }
                return Ok(None);
            }
            _ => return Ok(None),
        };
        
        let element_class = ElementClass::from_str(&element_token.value)
            .ok_or_else(|| RdfError::Parser(
                format!("Unknown element type: {}", element_token.value),
                element_token.line,
            ))?;
        
        let mut node = RdfNode::new(element_class, element_token.line, indent);
        
        // Expect :::
        if self.peek_type() == Some(TokenType::ElementSeparator) {
            self.advance();
        }
        
        // Parse attributes
        self.parse_attributes(&mut node)?;
        
        // Skip to end of line
        while let Some(token) = self.peek() {
            if token.token_type == TokenType::Newline {
                self.advance();
                break;
            }
            self.advance();
        }
        
        // Parse children (nodes with higher indentation)
        loop {
            self.skip_whitespace_and_comments();
            if let Some(child) = self.parse_node_if_present(indent + 1)? {
                node.children.push(child);
            } else {
                break;
            }
        }
        
        Ok(Some(node))
    }
    
    fn parse_attributes(&mut self, node: &mut RdfNode) -> Result<(), RdfError> {
        loop {
            // Look for attribute name
            let attr_name = match self.peek() {
                Some(t) if t.token_type == TokenType::AttributeName => {
                    self.advance().unwrap().value.clone()
                }
                _ => break,
            };
            
            // Check for value in brackets
            if self.peek_type() == Some(TokenType::OpenBracket) {
                self.advance(); // consume [
                
                let value = if let Some(token) = self.peek() {
                    if token.token_type == TokenType::Value {
                        let v = self.advance().unwrap().value.clone();
                        self.parse_value(&attr_name, &v)?
                    } else {
                        // Empty brackets
                        AttributeValue::String(String::new())
                    }
                } else {
                    AttributeValue::String(String::new())
                };
                
                // Consume closing bracket if present
                if self.peek_type() == Some(TokenType::CloseBracket) {
                    self.advance();
                }
                
                // Check for animation modifiers after this value
                let value = self.check_for_animation_modifier(&attr_name, value)?;
                
                node.attributes.insert(attr_name, value);
            } else if self.peek_type() == Some(TokenType::Tilde) {
                // Continuous rotation like ::rotatez[2.0] without brackets
                // Or animation like ::rotatex~90
                self.advance(); // consume ~
                
                if self.peek_type() == Some(TokenType::OpenBracket) {
                    self.advance();
                    if let Some(token) = self.peek() {
                        if token.token_type == TokenType::Value {
                            let target = token.value.parse::<f32>().unwrap_or(0.0);
                            self.advance();
                            
                            if self.peek_type() == Some(TokenType::CloseBracket) {
                                self.advance();
                            }
                            
                            node.attributes.insert(attr_name, AttributeValue::Animation(AnimationValue {
                                anim_type: AnimationModifier::Singular,
                                from: vec![0.0],
                                to: vec![target],
                                duration: None,
                            }));
                        }
                    }
                }
            }
            
            // Check for :: separator to next attribute
            if self.peek_type() == Some(TokenType::AttributeSeparator) {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    fn parse_value(&self, _attr_name: &str, raw: &str) -> Result<AttributeValue, RdfError> {
        let trimmed = raw.trim();
        
        // Check for "flex" keyword
        if trimmed.eq_ignore_ascii_case("flex") {
            return Ok(AttributeValue::Flex);
        }
        
        // Check for object syntax: key:value,key:value
        if trimmed.contains(':') && !trimmed.starts_with('#') && !trimmed.starts_with('@') {
            let mut map = HashMap::new();
            for pair in trimmed.split(',') {
                if let Some((k, v)) = pair.split_once(':') {
                    if let Ok(f) = v.trim().parse::<f32>() {
                        map.insert(k.trim().to_string(), f);
                    }
                }
            }
            if !map.is_empty() {
                return Ok(AttributeValue::Object(map));
            }
        }
        
        // Check for point syntax: x,y or x,y,z
        if trimmed.contains(',') {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].trim().parse::<f32>(), parts[1].trim().parse::<f32>()) {
                    return Ok(AttributeValue::Point2D(x, y));
                }
            } else if parts.len() == 3 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[0].trim().parse::<f32>(), 
                    parts[1].trim().parse::<f32>(),
                    parts[2].trim().parse::<f32>()
                ) {
                    return Ok(AttributeValue::Point3D(x, y, z));
                }
            }
        }
        
        // Check for single float
        if let Ok(f) = trimmed.parse::<f32>() {
            return Ok(AttributeValue::Float(f));
        }
        
        // Default to string
        Ok(AttributeValue::String(trimmed.to_string()))
    }
    
    fn check_for_animation_modifier(&mut self, _attr_name: &str, value: AttributeValue) -> Result<AttributeValue, RdfError> {
        // Check for animation syntax after a value
        // e.g., [0,0]~[1,1] or [0,0]{~}[1,1] or [0,0][~][1,1]
        
        // Look for ~ {~ or [~
        match self.peek_type() {
            Some(TokenType::Tilde) => {
                self.advance();
                
                // This is singular animation: from ~ to
                if self.peek_type() == Some(TokenType::OpenBracket) {
                    self.advance();
                    if let Some(token) = self.peek() {
                        if token.token_type == TokenType::Value {
                            let to_raw = token.value.clone();
                            self.advance();
                            
                            if self.peek_type() == Some(TokenType::CloseBracket) {
                                self.advance();
                            }
                            
                            let from = self.extract_floats(&value);
                            let to = self.parse_float_list(&to_raw);
                            
                            return Ok(AttributeValue::Animation(AnimationValue {
                                anim_type: AnimationModifier::Singular,
                                from,
                                to,
                                duration: None,
                            }));
                        }
                    }
                }
            }
            Some(TokenType::OpenBrace) => {
                // Check for {~}
                self.advance();
                if self.peek_type() == Some(TokenType::Tilde) {
                    self.advance();
                    if self.peek_type() == Some(TokenType::CloseBrace) {
                        self.advance();
                        
                        // Now get the target
                        if self.peek_type() == Some(TokenType::OpenBracket) {
                            self.advance();
                            if let Some(token) = self.peek() {
                                if token.token_type == TokenType::Value {
                                    let to_raw = token.value.clone();
                                    self.advance();
                                    
                                    if self.peek_type() == Some(TokenType::CloseBracket) {
                                        self.advance();
                                    }
                                    
                                    let from = self.extract_floats(&value);
                                    let to = self.parse_float_list(&to_raw);
                                    
                                    return Ok(AttributeValue::Animation(AnimationValue {
                                        anim_type: AnimationModifier::Rebound,
                                        from,
                                        to,
                                        duration: None,
                                    }));
                                }
                            }
                        }
                    }
                }
            }
            Some(TokenType::OpenBracket) => {
                // Check if this starts [~]
                let saved_pos = self.position;
                self.advance();
                if self.peek_type() == Some(TokenType::Tilde) {
                    self.advance();
                    if self.peek_type() == Some(TokenType::CloseBracket) {
                        self.advance();
                        
                        // Now get the target
                        if self.peek_type() == Some(TokenType::OpenBracket) {
                            self.advance();
                            if let Some(token) = self.peek() {
                                if token.token_type == TokenType::Value {
                                    let to_raw = token.value.clone();
                                    self.advance();
                                    
                                    if self.peek_type() == Some(TokenType::CloseBracket) {
                                        self.advance();
                                    }
                                    
                                    let from = self.extract_floats(&value);
                                    let to = self.parse_float_list(&to_raw);
                                    
                                    return Ok(AttributeValue::Animation(AnimationValue {
                                        anim_type: AnimationModifier::Sawtooth,
                                        from,
                                        to,
                                        duration: None,
                                    }));
                                }
                            }
                        }
                    } else {
                        // Not [~], restore position
                        self.position = saved_pos;
                    }
                } else {
                    // Not [~], restore position
                    self.position = saved_pos;
                }
            }
            _ => {}
        }
        
        Ok(value)
    }
    
    fn extract_floats(&self, value: &AttributeValue) -> Vec<f32> {
        match value {
            AttributeValue::Float(f) => vec![*f],
            AttributeValue::Point2D(x, y) => vec![*x, *y],
            AttributeValue::Point3D(x, y, z) => vec![*x, *y, *z],
            _ => vec![0.0],
        }
    }
    
    fn parse_float_list(&self, raw: &str) -> Vec<f32> {
        raw.split(',')
            .filter_map(|s| s.trim().parse::<f32>().ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lexer::Lexer;
    
    fn parse(input: &str) -> Result<RdfDocument, RdfError> {
        let lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.collect();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    
    #[test]
    fn test_simple_window() {
        let doc = parse("window:::title[Test]::sdims[100,100]").unwrap();
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, ElementClass::Window);
        assert!(doc.roots[0].get_str("title").is_some());
    }
    
    #[test]
    fn test_nested_hierarchy() {
        let input = r#"
window:::title[App]::sdims[100,100]
	pane:::id[main]::parent[App]::origin[0,0]::extent[1.0,1.0]
		widget:::id[btn]::parent[main]::extent[0.1,0.1]
"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].children.len(), 1);
        assert_eq!(doc.roots[0].children[0].children.len(), 1);
    }
    
    #[test]
    fn test_point_parsing() {
        let doc = parse("widget:::origin[0.5,0.3]").unwrap();
        let point = doc.roots[0].get_point("origin");
        assert!(point.is_some());
        let (x, y) = point.unwrap();
        assert!((x - 0.5).abs() < 0.01);
        assert!((y - 0.3).abs() < 0.01);
    }
}
