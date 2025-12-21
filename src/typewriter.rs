//! Typewriter Effect System
//!
//! Provides text content morphing capabilities, including:
//! - Character-by-character reveal (typewriter effect)
//! - Cross-fade transitions
//! - Scramble/unscramble effects

/// Typewriter effect: reveals text character by character
pub struct TypewriterEffect {
    pub target_text: String,
    pub current_position: usize,
}

impl TypewriterEffect {
    pub fn new(target_text: String) -> Self {
        Self {
            target_text,
            current_position: 0,
        }
    }
    
    /// Get the visible portion of the text based on progress (0.0 - 1.0)
    pub fn get_visible_text(&self, progress: f32) -> String {
        let progress = progress.clamp(0.0, 1.0);
        let visible_chars = (self.target_text.len() as f32 * progress).floor() as usize;
        self.target_text.chars().take(visible_chars).collect()
    }
    
    /// Advance by one character, returns current visible text
    pub fn tick(&mut self) -> String {
        self.current_position = (self.current_position + 1).min(self.target_text.len());
        self.target_text.chars().take(self.current_position).collect()
    }
    
    /// Check if typewriter effect is complete
    pub fn is_complete(&self) -> bool {
        self.current_position >= self.target_text.len()
    }
    
    /// Reset to beginning
    pub fn reset(&mut self) {
        self.current_position = 0;
    }
}

/// Cross-fade between two text strings
pub fn cross_fade_text(from: &str, to: &str, progress: f32) -> String {
    let progress = progress.clamp(0.0, 1.0);
    
    // Simple approach: switch at midpoint
    if progress < 0.5 {
        from.to_string()
    } else {
        to.to_string()
    }
}

/// Scramble effect: randomly replace characters while transitioning
pub fn scramble_text(from: &str, to: &str, progress: f32, seed: u64) -> String {
    let progress = progress.clamp(0.0, 1.0);
    let max_len = from.len().max(to.len());
    
    // Simple deterministic "randomness" based on seed
    let mut result = String::new();
    
    for i in 0..max_len {
        let char_progress = (i as f32 / max_len as f32 - progress).abs();
        
        if char_progress < 0.1 {
            // In transition zone: show scrambled character
            let scrambled = ((seed as usize + i) % 26 + 65) as u8 as char;
            result.push(scrambled);
        } else if progress < 0.5 {
            // Show 'from' text
            if let Some(c) = from.chars().nth(i) {
                result.push(c);
            }
        } else {
            // Show 'to' text
            if let Some(c) = to.chars().nth(i) {
                result.push(c);
            }
        }
    }
    
    result
}

/// Text content morph mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextMorphMode {
    /// Character-by-character reveal
    Typewriter,
    /// Simple cross-fade
    CrossFade,
    /// Scramble transition
    Scramble,
}

/// Apply text morphing based on mode
pub fn morph_text_content(
    from: &str,
    to: &str,
    progress: f32,
    mode: TextMorphMode,
    seed: Option<u64>,
) -> String {
    match mode {
        TextMorphMode::Typewriter => {
            let effect = TypewriterEffect::new(to.to_string());
            effect.get_visible_text(progress)
        }
        TextMorphMode::CrossFade => cross_fade_text(from, to, progress),
        TextMorphMode::Scramble => scramble_text(from, to, progress, seed.unwrap_or(0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typewriter_progression() {
        let mut effect = TypewriterEffect::new("Hello".to_string());
        assert_eq!(effect.tick(), "H");
        assert_eq!(effect.tick(), "He");
        assert_eq!(effect.tick(), "Hel");
        assert_eq!(effect.tick(), "Hell");
        assert_eq!(effect.tick(), "Hello");
        assert!(effect.is_complete());
    }

    #[test]
    fn test_typewriter_progress() {
        let effect = TypewriterEffect::new("Test".to_string());
        assert_eq!(effect.get_visible_text(0.0), "");
        assert_eq!(effect.get_visible_text(0.25), "T");
        assert_eq!(effect.get_visible_text(0.5), "Te");
        assert_eq!(effect.get_visible_text(0.75), "Tes");
        assert_eq!(effect.get_visible_text(1.0), "Test");
    }

    #[test]
    fn test_cross_fade() {
        assert_eq!(cross_fade_text("Old", "New", 0.0), "Old");
        assert_eq!(cross_fade_text("Old", "New", 0.4), "Old");
        assert_eq!(cross_fade_text("Old", "New", 0.6), "New");
        assert_eq!(cross_fade_text("Old", "New", 1.0), "New");
    }

    #[test]
    fn test_morph_text_modes() {
        let result = morph_text_content("Old", "New", 0.5, TextMorphMode::Typewriter, None);
        assert_eq!(result, "N"); // 50% of "New" = 1 char
        
        let result = morph_text_content("Old", "New", 0.3, TextMorphMode::CrossFade, None);
        assert_eq!(result, "Old");
        
        let result = morph_text_content("Old", "New", 0.7, TextMorphMode::CrossFade, None);
        assert_eq!(result, "New");
    }
}
