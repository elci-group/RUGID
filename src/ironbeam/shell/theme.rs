//! IronBeam Theme Engine
//! 
//! Defines the visual language and design system for the IronBeam GUI.

#[derive(Clone, Debug)]
pub struct Theme {
    pub colors: ColorPalette,
    pub spacing: Spacing,
    pub typography: Typography,
}

#[derive(Clone, Debug)]
pub struct ColorPalette {
    pub background: String,
    pub surface: String,
    pub surface_highlight: String,
    pub border: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub accent: String,
    pub accent_hover: String,
    pub success: String,
    pub warning: String,
    pub error: String,
}

#[derive(Clone, Debug)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Debug)]
pub struct Typography {
    pub font_family: String,
    pub size_xs: f32,
    pub size_sm: f32,
    pub size_md: f32,
    pub size_lg: f32,
    pub size_xl: f32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            colors: ColorPalette {
                background: "#1e1e1e".to_string(),
                surface: "#252526".to_string(),
                surface_highlight: "#37373d".to_string(),
                border: "#454545".to_string(),
                text_primary: "#cccccc".to_string(),
                text_secondary: "#858585".to_string(),
                accent: "#007acc".to_string(),
                accent_hover: "#0098ff".to_string(),
                success: "#89d185".to_string(),
                warning: "#cca700".to_string(),
                error: "#f48771".to_string(),
            },
            spacing: Spacing {
                xs: 2.0,
                sm: 4.0,
                md: 8.0,
                lg: 16.0,
                xl: 24.0,
            },
            typography: Typography {
                font_family: "Inter, system-ui, sans-serif".to_string(),
                size_xs: 10.0,
                size_sm: 12.0,
                size_md: 14.0,
                size_lg: 18.0,
                size_xl: 24.0,
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
