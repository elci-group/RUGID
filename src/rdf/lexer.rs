//! RDF Lexer - Tokenizes RDF files into a stream of tokens
//!
//! Handles the syntax:
//! - ::: separates element from attributes
//! - :: separates attributes
//! - [value] contains attribute values
//! - Indentation (tabs) for hierarchy

/// A token produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

/// Token types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    /// Element class name (window, pane, widget, text, shape3d)
    Element,
    /// ::: separator between element and attributes
    ElementSeparator,
    /// :: separator between attributes
    AttributeSeparator,
    /// Attribute name
    AttributeName,
    /// [ opening bracket
    OpenBracket,
    /// ] closing bracket
    CloseBracket,
    /// Value inside brackets
    Value,
    /// { opening brace (for {~})
    OpenBrace,
    /// } closing brace
    CloseBrace,
    /// ~ tilde (animation modifier)
    Tilde,
    /// Indentation (tabs at start of line)
    Indent,
    /// End of line
    Newline,
    /// End of file
    Eof,
    /// Comment line (starts with #)
    Comment,
}

/// Lexer state machine
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    column: usize,
    at_line_start: bool,
    bracket_depth: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            line: 1,
            column: 0,
            at_line_start: true,
            bracket_depth: 0,
        }
    }
    
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if let Some(c) = ch {
            if c == '\n' {
                self.line += 1;
                self.column = 0;
                self.at_line_start = true;
            } else {
                self.column += 1;
                if c != '\t' && c != ' ' {
                    self.at_line_start = false;
                }
            }
        }
        ch
    }
    
    fn make_token(&self, token_type: TokenType, value: impl Into<String>) -> Token {
        Token {
            token_type,
            value: value.into(),
            line: self.line,
            column: self.column,
        }
    }
    
    fn skip_whitespace_not_indent(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' && !self.at_line_start {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_indent(&mut self) -> Token {
        let mut count = 0;
        while let Some(c) = self.peek() {
            if c == '\t' {
                count += 1;
                self.advance();
            } else if c == ' ' {
                // Convert 4 spaces to 1 tab
                let start = count;
                while let Some(' ') = self.peek() {
                    count += 1;
                    self.advance();
                }
                count = start + (count - start) / 4;
                break;
            } else {
                break;
            }
        }
        self.at_line_start = false;
        self.make_token(TokenType::Indent, count.to_string())
    }
    
    fn read_element_or_attribute(&mut self) -> Token {
        let mut value = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check if this is followed by ::: (element) or :: (attribute)
        if self.peek() == Some(':') {
            // Peek ahead to see if it's ::: or ::
            let remaining: String = self.chars.clone().take(3).collect();
            if remaining.starts_with(":::") {
                self.make_token(TokenType::Element, value)
            } else {
                self.make_token(TokenType::AttributeName, value)
            }
        } else {
            self.make_token(TokenType::AttributeName, value)
        }
    }
    
    fn read_value_content(&mut self) -> Token {
        let mut value = String::new();
        let mut depth = 0;
        
        while let Some(c) = self.peek() {
            if c == '[' {
                depth += 1;
                value.push(c);
                self.advance();
            } else if c == ']' {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                value.push(c);
                self.advance();
            } else if c == '\n' {
                // Values shouldn't span lines typically, but let's allow it or break?
                // If we break, we might leave the parser in a bad state.
                // Let's break for now as RDF is line-based.
                break;
            } else {
                value.push(c);
                self.advance();
            }
        }
        
        self.make_token(TokenType::Value, value)
    }
    
    fn read_comment(&mut self) -> Token {
        let mut value = String::new();
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            value.push(c);
            self.advance();
        }
        self.make_token(TokenType::Comment, value)
    }
    
    fn next_token(&mut self) -> Option<Token> {
        // Handle indentation at start of line
        if self.at_line_start {
            if let Some(c) = self.peek() {
                if c == '\t' || c == ' ' {
                    return Some(self.read_indent());
                } else if c == '\n' {
                    self.advance();
                    return Some(self.make_token(TokenType::Newline, "\n"));
                } else if c == '#' {
                    return Some(self.read_comment());
                }
            }
            self.at_line_start = false;
        }
        
        // If we are inside brackets, read value content
        if self.bracket_depth > 0 {
             if let Some(c) = self.peek() {
                 if c != ']' {
                     return Some(self.read_value_content());
                 }
             }
        }
        
        self.skip_whitespace_not_indent();
        
        let c = self.peek()?;
        
        match c {
            '#' => Some(self.read_comment()),
            '\n' => {
                self.advance();
                Some(self.make_token(TokenType::Newline, "\n"))
            }
            ':' => {
                // Check for ::: or ::
                self.advance();
                if self.peek() == Some(':') {
                    self.advance();
                    if self.peek() == Some(':') {
                        self.advance();
                        Some(self.make_token(TokenType::ElementSeparator, ":::"))
                    } else {
                        Some(self.make_token(TokenType::AttributeSeparator, "::"))
                    }
                } else {
                    // Single colon in value context
                    Some(self.make_token(TokenType::Value, ":"))
                }
            }
            '[' => {
                self.advance();
                self.bracket_depth += 1;
                Some(self.make_token(TokenType::OpenBracket, "["))
            }
            ']' => {
                self.advance();
                if self.bracket_depth > 0 {
                    self.bracket_depth -= 1;
                }
                Some(self.make_token(TokenType::CloseBracket, "]"))
            }
            '{' => {
                self.advance();
                Some(self.make_token(TokenType::OpenBrace, "{"))
            }
            '}' => {
                self.advance();
                Some(self.make_token(TokenType::CloseBrace, "}"))
            }
            '~' => {
                self.advance();
                Some(self.make_token(TokenType::Tilde, "~"))
            }
            _ if c.is_alphabetic() || c == '_' => {
                Some(self.read_element_or_attribute())
            }
            _ => {
                // Skip unknown characters
                self.advance();
                self.next_token()
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_lexing() {
        let input = "window:::title[Test]";
        let lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.collect();
        
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Element && t.value == "window"));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::ElementSeparator));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::AttributeName && t.value == "title"));
    }
    
    #[test]
    fn test_indentation() {
        let input = "\tpane:::id[main]";
        let lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.collect();
        
        let indent = tokens.iter().find(|t| t.token_type == TokenType::Indent);
        assert!(indent.is_some());
        assert_eq!(indent.unwrap().value, "1");
    }
    
    #[test]
    fn test_value_parsing() {
        let input = "widget:::origin[0.5,0.5]";
        let lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.collect();
        
        let value = tokens.iter().find(|t| t.token_type == TokenType::Value);
        assert!(value.is_some());
        assert_eq!(value.unwrap().value, "0.5,0.5");
    }
}
