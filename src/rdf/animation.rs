//! Animation parsing utilities
//!
//! Handles parsing of movement, rotation, and fade animations from RDF syntax.

use super::ast::{AttributeValue, AnimationModifier};
use crate::temporal::{DeltaProgram, DeltaFrame, DeltaType};
use crate::cell::CellId;

/// Animation type
#[derive(Debug, Clone)]
pub enum AnimationType {
    Move,
    RotateX,
    RotateY,
    RotateZ,
    Fade,
}

/// Parsed animation specification
#[derive(Debug, Clone)]
pub struct AnimationSpec {
    pub anim_type: AnimationType,
    pub modifier: AnimationModifier,
    pub from: Vec<f32>,
    pub to: Vec<f32>,
    pub duration_frames: u64,
}

impl AnimationSpec {
    /// Convert to a DeltaProgram for the temporal memory system
    pub fn to_delta_program(&self, cell_id: CellId) -> DeltaProgram {
        let mut schedule = Vec::new();
        
        let duration = self.duration_frames.max(1);
        
        match self.modifier {
            AnimationModifier::Singular => {
                // Linear interpolation from start to end
                schedule = self.generate_linear_frames(duration);
            }
            AnimationModifier::Rebound => {
                // Forward then backward (yoyo)
                let forward = self.generate_linear_frames(duration / 2);
                let backward = self.generate_reverse_frames(duration / 2);
                schedule.extend(forward);
                schedule.extend(backward);
            }
            AnimationModifier::Sawtooth => {
                // Linear then instant reset
                schedule = self.generate_linear_frames(duration);
                // The reset happens automatically when the program loops
            }
            AnimationModifier::Continuous => {
                // Constant delta per frame
                schedule = self.generate_continuous_frames(duration);
            }
        }
        
        let delta_type = match self.modifier {
            AnimationModifier::Singular => DeltaType::Adhesive { 
                inverse: Box::new(self.generate_inverse_frame())
            },
            _ => DeltaType::Greasy,
        };
        
        DeltaProgram::new(cell_id, schedule, delta_type)
    }
    
    fn generate_linear_frames(&self, duration: u64) -> Vec<DeltaFrame> {
        let mut frames = Vec::new();
        
        for t in 0..duration {
            let progress = t as f32 / duration as f32;
            let mut frame = DeltaFrame::zero(t, t + 1);
            
            self.apply_deltas(&mut frame, progress, 1.0 / duration as f32);
            frames.push(frame);
        }
        
        frames
    }
    
    fn generate_reverse_frames(&self, duration: u64) -> Vec<DeltaFrame> {
        let mut frames = Vec::new();
        let start_t = duration;
        
        for t in 0..duration {
            let progress = 1.0 - (t as f32 / duration as f32);
            let mut frame = DeltaFrame::zero(start_t + t, start_t + t + 1);
            
            self.apply_deltas(&mut frame, progress, -1.0 / duration as f32);
            frames.push(frame);
        }
        
        frames
    }
    
    fn generate_continuous_frames(&self, duration: u64) -> Vec<DeltaFrame> {
        let mut frames = Vec::new();
        
        // For continuous rotation, 'to' contains the speed per frame
        let speed = self.to.first().copied().unwrap_or(1.0);
        
        for t in 0..duration {
            let mut frame = DeltaFrame::zero(t, t + 1);
            
            match self.anim_type {
                AnimationType::RotateX => {
                    // Use delta_origin_p for pitch simulation
                    frame.delta_origin_p = speed * 0.01;
                }
                AnimationType::RotateY => {
                    frame.delta_origin_s = speed * 0.01;
                }
                AnimationType::RotateZ => {
                    // Z rotation would need special handling
                    frame.delta_origin_s = speed * 0.005;
                    frame.delta_origin_p = speed * 0.005;
                }
                _ => {}
            }
            
            frames.push(frame);
        }
        
        frames
    }
    
    fn apply_deltas(&self, frame: &mut DeltaFrame, _progress: f32, delta_per_frame: f32) {
        match self.anim_type {
            AnimationType::Move => {
                if self.from.len() >= 2 && self.to.len() >= 2 {
                    let dx = (self.to[0] - self.from[0]) * delta_per_frame;
                    let dy = (self.to[1] - self.from[1]) * delta_per_frame;
                    frame.delta_origin_s = dx;
                    frame.delta_origin_p = dy;
                }
            }
            AnimationType::Fade => {
                if !self.to.is_empty() && !self.from.is_empty() {
                    let da = (self.to[0] - self.from[0]) * delta_per_frame;
                    frame.delta_a = da;
                }
            }
            AnimationType::RotateX | AnimationType::RotateY | AnimationType::RotateZ => {
                // Rotation deltas - these would need 3D rotation support in renderer
                // For now, simulate with position changes
            }
        }
    }
    
    fn generate_inverse_frame(&self) -> DeltaFrame {
        let mut frame = DeltaFrame::zero(0, 1);
        
        match self.anim_type {
            AnimationType::Move => {
                if self.from.len() >= 2 && self.to.len() >= 2 {
                    frame.delta_origin_s = self.from[0] - self.to[0];
                    frame.delta_origin_p = self.from[1] - self.to[1];
                }
            }
            AnimationType::Fade => {
                if !self.to.is_empty() && !self.from.is_empty() {
                    frame.delta_a = self.from[0] - self.to[0];
                }
            }
            _ => {}
        }
        
        frame
    }
}

/// Parse animation from attribute value
pub fn parse_animation(name: &str, value: &AttributeValue) -> Option<AnimationSpec> {
    let anim_type = match name {
        "move" => AnimationType::Move,
        "rotatex" => AnimationType::RotateX,
        "rotatey" => AnimationType::RotateY,
        "rotatez" => AnimationType::RotateZ,
        "fade" => AnimationType::Fade,
        _ => return None,
    };
    
    match value {
        AttributeValue::Animation(anim) => {
            Some(AnimationSpec {
                anim_type,
                modifier: anim.anim_type,
                from: anim.from.clone(),
                to: anim.to.clone(),
                duration_frames: anim.duration.unwrap_or(60),
            })
        }
        AttributeValue::Float(f) => {
            // Continuous rotation at speed f
            Some(AnimationSpec {
                anim_type,
                modifier: AnimationModifier::Continuous,
                from: vec![0.0],
                to: vec![*f],
                duration_frames: 3600, // 1 minute at 60fps
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ast::AnimationModifier;
    
    #[test]
    fn test_move_animation() {
        let spec = AnimationSpec {
            anim_type: AnimationType::Move,
            modifier: AnimationModifier::Singular,
            from: vec![0.0, 0.0],
            to: vec![1.0, 0.5],
            duration_frames: 60,
        };
        
        let program = spec.to_delta_program(CellId::next());
        assert_eq!(program.schedule.len(), 60);
    }
    
    #[test]
    fn test_rebound_animation() {
        let spec = AnimationSpec {
            anim_type: AnimationType::Move,
            modifier: AnimationModifier::Rebound,
            from: vec![0.0, 0.0],
            to: vec![1.0, 0.0],
            duration_frames: 60,
        };
        
        let program = spec.to_delta_program(CellId::next());
        // Forward 30 + backward 30 = 60
        assert_eq!(program.schedule.len(), 60);
    }
}
