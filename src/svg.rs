use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SvgAttribute {
    Fill(String),
    Stroke(String),
    Opacity(f32),
    Transform(String),
    /// Font size in pixels (resolved from relative sizing)
    FontSize(f32),
    /// Font family
    FontFamily(String),
    /// Font weight (100-900 or "bold", "normal")
    FontWeight(String),
    /// Text anchor for horizontal alignment: "start", "middle", "end"
    TextAnchor(String),
    /// Dominant baseline for vertical alignment: "auto", "middle", "hanging", "text-top"
    DominantBaseline(String),
    /// Letter spacing
    LetterSpacing(f32),
    /// Filter ID reference (e.g., "url(#goo)")
    Filter(String),
}

impl fmt::Display for SvgAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SvgAttribute::Fill(val) => write!(f, "fill=\"{}\"", val),
            SvgAttribute::Stroke(val) => write!(f, "stroke=\"{}\"", val),
            SvgAttribute::Opacity(val) => write!(f, "opacity=\"{}\"", val),
            SvgAttribute::Transform(val) => write!(f, "transform=\"{}\"", val),
            SvgAttribute::FontSize(val) => write!(f, "font-size=\"{:.1}px\"", val),
            SvgAttribute::FontFamily(val) => write!(f, "font-family=\"{}\"", val),
            SvgAttribute::FontWeight(val) => write!(f, "font-weight=\"{}\"", val),
            SvgAttribute::TextAnchor(val) => write!(f, "text-anchor=\"{}\"", val),
            SvgAttribute::DominantBaseline(val) => write!(f, "dominant-baseline=\"{}\"", val),
            SvgAttribute::LetterSpacing(val) => write!(f, "letter-spacing=\"{:.2}px\"", val),
            SvgAttribute::Filter(val) => write!(f, "filter=\"{}\"", val),
        }
    }
}

/// Transform for ontological coordinate space composition
#[derive(Debug, Clone, PartialEq)]
pub struct SvgTransform {
    pub translate: (f32, f32),
    pub scale: (f32, f32),
    pub rotate: f32,
}

impl Default for SvgTransform {
    fn default() -> Self {
        Self {
            translate: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotate: 0.0,
        }
    }
}

impl SvgTransform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            translate: (x, y),
            ..Default::default()
        }
    }
    
    pub fn with_scale(mut self, sx: f32, sy: f32) -> Self {
        self.scale = (sx, sy);
        self
    }
    
    pub fn with_rotation(mut self, degrees: f32) -> Self {
        self.rotate = degrees;
        self
    }
    
    /// Generate SVG transform attribute value
    pub fn to_svg_string(&self) -> String {
        let mut parts = Vec::new();
        
        if self.translate != (0.0, 0.0) {
            parts.push(format!("translate({:.2}, {:.2})", self.translate.0, self.translate.1));
        }
        
        if self.scale != (1.0, 1.0) {
            parts.push(format!("scale({:.4}, {:.4})", self.scale.0, self.scale.1));
        }
        
        if self.rotate != 0.0 {
            parts.push(format!("rotate({:.2})", self.rotate));
        }
        
        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SvgNode {
    Group {
        attributes: Vec<SvgAttribute>,
        children: Vec<SvgNode>,
        /// Ontological transform - establishes local coordinate space
        transform: Option<SvgTransform>,
    },
    Rect {
        attributes: Vec<SvgAttribute>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// SVG path element for morphed shapes
    Path {
        attributes: Vec<SvgAttribute>,
        /// SVG path data string (e.g., "M 0 0 L 100 0 L 100 100 Z")
        d: String,
    },
    Text {
        attributes: Vec<SvgAttribute>,
        x: f32,
        y: f32,
        content: String,
    },
    /// Raw SVG markup (for pre-rendered content like 3D objects)
    RawSvg(String),
    // Add more primitives as needed
}

impl SvgNode {
    pub fn add_attribute(&mut self, attr: SvgAttribute) {
        match self {
            SvgNode::Group { attributes, .. } => attributes.push(attr),
            SvgNode::Rect { attributes, .. } => attributes.push(attr),
            SvgNode::Path { attributes, .. } => attributes.push(attr),
            SvgNode::Text { attributes, .. } => attributes.push(attr),
            SvgNode::RawSvg(_) => {}, // Raw SVG doesn't have attributes
        }
    }

    pub fn set_fill(&mut self, color: String) {
        let attributes = match self {
            SvgNode::Group { attributes, .. } => attributes,
            SvgNode::Rect { attributes, .. } => attributes,
            SvgNode::Path { attributes, .. } => attributes,
            SvgNode::Text { attributes, .. } => attributes,
            SvgNode::RawSvg(_) => return, // Raw SVG doesn't have attributes
        };
        attributes.retain(|attr| !matches!(attr, SvgAttribute::Fill(_)));
        attributes.push(SvgAttribute::Fill(color));
    }
}

impl fmt::Display for SvgNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SvgNode::Group { attributes, children, transform } => {
                write!(f, "<g")?;
                // Emit ontological transform if present
                if let Some(xform) = transform {
                    let xform_str = xform.to_svg_string();
                    if !xform_str.is_empty() {
                        write!(f, " transform=\"{}\"", xform_str)?;
                    }
                }
                for attr in attributes {
                    write!(f, " {}", attr)?;
                }
                write!(f, ">")?;
                for child in children {
                    write!(f, "{}", child)?;
                }
                write!(f, "</g>")
            }
            SvgNode::Rect { attributes, x, y, width, height } => {
                write!(f, "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"", x, y, width, height)?;
                for attr in attributes {
                    write!(f, " {}", attr)?;
                }
                write!(f, " />")
            }
            SvgNode::Path { attributes, d } => {
                write!(f, "<path d=\"{}\"", d)?;
                for attr in attributes {
                    write!(f, " {}", attr)?;
                }
                write!(f, " />")
            }
            SvgNode::Text { attributes, x, y, content } => {
                write!(f, "<text x=\"{}\" y=\"{}\"", x, y)?;
                for attr in attributes {
                    write!(f, " {}", attr)?;
                }
                write!(f, ">{}</text>", content)
            }
            SvgNode::RawSvg(svg) => {
                // Output raw SVG as-is
                write!(f, "{}", svg)
            }
        }
    }
}

/// Parse RGB color from string format (supports "rgb(r,g,b)" and "#RRGGBB")
/// Returns (r, g, b) as floats in 0.0-1.0 range, or None if parsing fails
pub fn parse_rgb_color(color: &str) -> Option<(f32, f32, f32)> {
    let color = color.trim();
    
    // Parse rgb(r,g,b) format
    if color.starts_with("rgb(") && color.ends_with(")") {
        let inner = &color[4..color.len()-1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].trim().parse::<u8>(),
                parts[1].trim().parse::<u8>(),
                parts[2].trim().parse::<u8>(),
            ) {
                return Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0));
            }
        }
    }
    
    // Parse #RRGGBB format
    if color.starts_with("#") && color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&color[1..3], 16),
            u8::from_str_radix(&color[3..5], 16),
            u8::from_str_radix(&color[5..7], 16),
        ) {
            return Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0));
        }
    }
    
    None
}

/// Convert RGB floats (0.0-1.0) to "rgb(r,g,b)" string
pub fn rgb_to_string(r: f32, g: f32, b: f32) -> String {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u8;
    format!("rgb({},{},{})", r, g, b)
}

/// Convert a NormalizedPath from morphism to SVG path data string
pub fn path_to_svg_d(points: &[(f32, f32)], closed: bool) -> String {
    if points.is_empty() {
        return String::new();
    }
    
    let mut d = String::new();
    
    // Start with moveto
    d.push_str(&format!("M {:.2} {:.2}", points[0].0, points[0].1));
    
    // Add lineto commands for remaining points
    for point in &points[1..] {
        d.push_str(&format!(" L {:.2} {:.2}", point.0, point.1));
    }
    
    // Close path if needed
    if closed {
        d.push_str(" Z");
    }
    
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_generation() {
        let rect = SvgNode::Rect {
            attributes: vec![SvgAttribute::Fill("red".into()), SvgAttribute::Opacity(0.5)],
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };

        let output = rect.to_string();
        assert!(output.contains("<rect"));
        assert!(output.contains("fill=\"red\""));
        assert!(output.contains("opacity=\"0.5\""));
    }
}
