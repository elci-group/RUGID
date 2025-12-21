/// SVG Filter Engine
///
/// Generates resolution-independent post-processing effects using SVG Filters.

#[derive(Clone, Debug)]
pub enum FilterEffect {
    /// Bloom (Glow) Effect
    Bloom {
        intensity: f32, // 0.0 - 1.0
        radius: f32,    // Blur radius
        threshold: f32, // Brightness threshold
    },
    /// Chromatic Aberration (RGB Split)
    ChromaticAberration {
        offset: f32, // Pixel offset
    },
    /// Color Grading
    ColorGrade {
        saturation: f32, // 1.0 = Normal
        contrast: f32,   // 1.0 = Normal
        brightness: f32, // 0.0 = Normal
    },
    /// Analog Noise (Film Grain)
    Noise {
        intensity: f32, // 0.0 - 1.0
    },
}

#[derive(Clone, Debug)]
pub struct FilterStack {
    pub effects: Vec<FilterEffect>,
    pub id: String,
}

impl Default for FilterStack {
    fn default() -> Self {
        Self {
            effects: Vec::new(),
            id: "pp_stack".to_string(),
        }
    }
}

impl FilterStack {
    pub fn new(id: &str) -> Self {
        Self {
            effects: Vec::new(),
            id: id.to_string(),
        }
    }

    pub fn add_effect(&mut self, effect: FilterEffect) {
        self.effects.push(effect);
    }

    /// Generate the <filter> definition string
    pub fn generate_svg_defs(&self) -> String {
        if self.effects.is_empty() {
            return String::new();
        }

        let mut xml = format!(r#"<filter id="{}" x="-20%" y="-20%" width="140%" height="140%">"#, self.id);
        
        // Chain inputs: Start with SourceGraphic
        let mut last_result = "SourceGraphic".to_string();
        
        for (i, effect) in self.effects.iter().enumerate() {
            let result_name = format!("effect_{}", i);
            
            match effect {
                FilterEffect::Bloom { intensity, radius, threshold } => {
                    // 1. Extract bright areas (Threshold)
                    // Using feColorMatrix to subtract threshold
                    // value = R*1 + G*1 + B*1 - threshold? No, that's luminance.
                    // Let's use a luminance approximation: 0.21 R + 0.72 G + 0.07 B
                    // And subtract threshold.
                    let thresh_res = format!("{}_thresh", result_name);
                    // Matrix:
                    // R G B A Offset
                    // 1 0 0 0 -thresh
                    // 0 1 0 0 -thresh
                    // 0 0 1 0 -thresh
                    // 0 0 0 1 0
                    // This is a harsh threshold.
                    
                    xml.push_str(&format!(
                        r#"<feColorMatrix in="{}" type="matrix" values="1 0 0 0 -{}  0 1 0 0 -{}  0 0 1 0 -{}  0 0 0 1 0" result="{}" />"#,
                        last_result, threshold, threshold, threshold, thresh_res
                    ));
                    
                    // 2. Blur
                    let blur_res = format!("{}_blur", result_name);
                    xml.push_str(&format!(
                        r#"<feGaussianBlur in="{}" stdDeviation="{}" result="{}" />"#,
                        thresh_res, radius, blur_res
                    ));
                    
                    // 3. Composite (Add)
                    // We can use feMerge or feComposite.
                    // If we use feComposite operator="arithmetic" k2=1 k3=intensity?
                    // Or just simple "screen" blend.
                    xml.push_str(&format!(
                        r#"<feComposite in="{}" in2="{}" operator="arithmetic" k2="1" k3="{}" result="{}" />"#,
                        last_result, blur_res, intensity, result_name
                    ));
                }
                
                FilterEffect::ChromaticAberration { offset } => {
                    let r_res = format!("{}_r", result_name);
                    let b_res = format!("{}_b", result_name);
                    let g_res = format!("{}_g", result_name); // G is usually anchor
                    
                    // Split channels?
                    // Actually, simpler:
                    // 1. Offset Source for Red
                    // 2. Offset Source for Blue (opposite)
                    // 3. Merge R from 1, G from Source, B from 2.
                    
                    // Offset Red
                    xml.push_str(&format!(
                        r#"<feOffset in="{}" dx="{}" dy="0" result="{}" />"#,
                        last_result, offset, r_res
                    ));
                    
                    // Offset Blue
                    xml.push_str(&format!(
                        r#"<feOffset in="{}" dx="-{}" dy="0" result="{}" />"#,
                        last_result, offset, b_res
                    ));
                    
                    // Merge channels using feMerge is tricky because we need to isolate channels first.
                    // Better approach:
                    // Use feColorMatrix to isolate R, G, B
                    // Then feComposite 'lighten' or 'screen'? No.
                    
                    // Standard SVG Chromatic Aberration:
                    // 1. Take Red channel of Offset R
                    // 2. Take Blue channel of Offset B
                    // 3. Take Green channel of Source
                    // 4. Combine.
                    
                    // Isolate Red from r_res
                    let r_only = format!("{}_r_only", result_name);
                    xml.push_str(&format!(
                        r#"<feColorMatrix in="{}" type="matrix" values="1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0" result="{}" />"#,
                        r_res, r_only
                    ));
                    
                    // Isolate Blue from b_res
                    let b_only = format!("{}_b_only", result_name);
                    xml.push_str(&format!(
                        r#"<feColorMatrix in="{}" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0" result="{}" />"#,
                        b_res, b_only
                    ));
                    
                    // Isolate Green from Source
                    let g_only = format!("{}_g_only", result_name);
                    xml.push_str(&format!(
                        r#"<feColorMatrix in="{}" type="matrix" values="0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0" result="{}" />"#,
                        last_result, g_only
                    ));
                    
                    // Composite (Add)
                    xml.push_str(&format!(
                        r#"<feComposite in="{}" in2="{}" operator="add" result="{}_rg" />"#,
                        r_only, g_only, result_name
                    ));
                    xml.push_str(&format!(
                        r#"<feComposite in="{}_rg" in2="{}" operator="add" result="{}" />"#,
                        result_name, b_only, result_name
                    ));
                }
                
                FilterEffect::ColorGrade { saturation, contrast, brightness } => {
                    // Saturation Matrix
                    let s = saturation;
                    let lum_r = 0.2126;
                    let lum_g = 0.7152;
                    let lum_b = 0.0722;
                    let one_minus_s = 1.0 - s;
                    
                    let r1 = lum_r * one_minus_s + s;
                    let r2 = lum_g * one_minus_s;
                    let r3 = lum_b * one_minus_s;
                    
                    let g1 = lum_r * one_minus_s;
                    let g2 = lum_g * one_minus_s + s;
                    let g3 = lum_b * one_minus_s;
                    
                    let b1 = lum_r * one_minus_s;
                    let b2 = lum_g * one_minus_s;
                    let b3 = lum_b * one_minus_s + s;
                    
                    let sat_res = format!("{}_sat", result_name);
                    xml.push_str(&format!(
                        r#"<feColorMatrix in="{}" type="matrix" values="{:.2} {:.2} {:.2} 0 0  {:.2} {:.2} {:.2} 0 0  {:.2} {:.2} {:.2} 0 0  0 0 0 1 0" result="{}" />"#,
                        last_result, r1, r2, r3, g1, g2, g3, b1, b2, b3, sat_res
                    ));
                    
                    // Contrast/Brightness (Slope/Intercept)
                    // Slope = contrast
                    // Intercept = brightness + 0.5 * (1 - contrast) (to pivot around grey)
                    let slope = contrast;
                    let intercept = brightness + 0.5 * (1.0 - contrast);
                    
                    xml.push_str(&format!(
                        r#"<feComponentTransfer in="{}" result="{}">
                            <feFuncR type="linear" slope="{}" intercept="{}" />
                            <feFuncG type="linear" slope="{}" intercept="{}" />
                            <feFuncB type="linear" slope="{}" intercept="{}" />
                        </feComponentTransfer>"#,
                        sat_res, result_name, slope, intercept, slope, intercept, slope, intercept
                    ));
                }
                
                FilterEffect::Noise { intensity } => {
                    let noise_res = format!("{}_noise", result_name);
                    // Generate noise
                    xml.push_str(&format!(
                        r#"<feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="3" stitchTiles="stitch" result="{}" />"#,
                        noise_res
                    ));
                    
                    // Desaturate noise (make it grey)
                    let grey_noise = format!("{}_grey", result_name);
                    xml.push_str(&format!(
                        r#"<feColorMatrix in="{}" type="saturate" values="0" result="{}" />"#,
                        noise_res, grey_noise
                    ));
                    
                    // Blend with source (Overlay or Multiply)
                    // We want to add grain.
                    // Use arithmetic composite?
                    // out = k1*in1*in2 + k2*in1 + k3*in2 + k4
                    // Let's use simple blend mode="overlay" or "multiply" with opacity?
                    // SVG filters don't have opacity on primitives easily.
                    // We can use feComposite arithmetic to blend.
                    // result = source + (noise - 0.5) * intensity
                    
                    // Let's just overlay.
                    xml.push_str(&format!(
                        r#"<feBlend in="{}" in2="{}" mode="overlay" result="{}" />"#,
                        grey_noise, last_result, result_name
                    ));
                }
            }
            
            last_result = result_name;
        }
        
        xml.push_str("</filter>");
        xml
    }
}
