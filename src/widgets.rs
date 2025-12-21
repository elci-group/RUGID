use crate::cell::CellId;
use crate::projector::Projector;
use crate::input::InputState;
use crate::signal::Signal;
use crate::diffuser::{Projection, Layer, BehaviourFlag};
use crate::temporal::{DeltaProgram, DeltaFrame, DeltaType};
use crate::geometry::VectorRegion;

pub struct ButtonProjector {
    pub target: CellId,
    pub geometry: VectorRegion, // Needed for hit testing
    pub label: String,
}

impl Projector<InputState> for ButtonProjector {
    fn project(&self, signal: &Signal<InputState>) -> Projection {
        let state = &signal.payload;
        
        // Hit test (Assuming absolute coordinates for now, or relative if we had transform)
        // For this MVP, we assume geometry is in screen space (which it is after LayoutSolver).
        // But wait, Projector doesn't know the resolved layout unless we pass it.
        // The `geometry` field here must be updated by the runtime/layout system.
        // For simplicity, we'll assume it's set correctly on creation or update.
        
        // Mock hit test: check if pointer is within geometry (scaled by 100 as per runtime)
        let x = self.geometry.origin_s * 100.0;
        let y = self.geometry.origin_p * 100.0;
        let w = self.geometry.extent_s * 100.0;
        let h = self.geometry.extent_p * 100.0;
        
        let hit = state.pointer.x >= x && state.pointer.x <= x + w &&
                  state.pointer.y >= y && state.pointer.y <= y + h;
        
        let pressed = hit && state.pointer.is_down;

        let mut schedule = Vec::new();
        
        if pressed {
            // Darken significantly
            let mut frame = DeltaFrame::zero(0, 1);
            frame.delta_r = -0.2;
            frame.delta_g = -0.2;
            frame.delta_b = -0.2;
            schedule.push(frame);
        } else if hit {
            // Lighten slightly
            let mut frame = DeltaFrame::zero(0, 1);
            frame.delta_r = 0.1;
            frame.delta_g = 0.1;
            frame.delta_b = 0.1;
            schedule.push(frame);
        }

        let program = DeltaProgram::new(
            self.target,
            schedule,
            DeltaType::Greasy,
        );

        Projection {
            delta: program,
            layer: Layer::Content,
            behaviour: BehaviourFlag::None,
        }
    }
}

pub struct SliderProjector {
    pub target: CellId,
    pub geometry: VectorRegion,
    pub value: f32, // 0.0 to 1.0
}

impl Projector<InputState> for SliderProjector {
    fn project(&self, signal: &Signal<InputState>) -> Projection {
        let state = &signal.payload;
        
        let x = self.geometry.origin_s * 100.0;
        let y = self.geometry.origin_p * 100.0;
        let w = self.geometry.extent_s * 100.0;
        let h = self.geometry.extent_p * 100.0;
        
        let hit = state.pointer.x >= x && state.pointer.x <= x + w &&
                  state.pointer.y >= y && state.pointer.y <= y + h;

        let mut schedule = Vec::new();
        
        // Visual feedback for thumb (simplified as color change for now)
        // In a real renderer, we'd emit geometry deltas (rect x/y).
        // RUGID MVP Renderer only supports color deltas.
        // So we'll simulate "active" state by changing color.
        
        if hit {
             let mut frame = DeltaFrame::zero(0, 1);
             frame.delta_b = 0.3; // Blue tint when hovering
             schedule.push(frame);
        }

        let program = DeltaProgram::new(
            self.target,
            schedule,
            DeltaType::Greasy,
        );

        Projection {
            delta: program,
            layer: Layer::Content,
            behaviour: BehaviourFlag::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{InputEvent, InputDispatcher};

    #[test]
    fn test_button_hover() {
        let geo = VectorRegion::new(0.0, 0.0, 1.0, 1.0); // 0-100, 0-100
        let button = ButtonProjector {
            target: CellId(1),
            geometry: geo,
            label: "Test".to_string(),
        };

        let mut dispatcher = InputDispatcher::new();
        dispatcher.dispatch(InputEvent::PointerMove(50.0, 50.0)); // Center
        
        let signal = dispatcher.signal();
        let proj = button.project(&signal);
        
        // Should have 1 frame (hover effect)
        assert_eq!(proj.delta.schedule.len(), 1);
        assert!(proj.delta.schedule[0].delta_r > 0.0);
    }
}
