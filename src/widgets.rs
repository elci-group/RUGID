use crate::cell::CellId;
use crate::diffuser::{BehaviourFlag, Layer, Projection};
use crate::geometry::VectorRegion;
use crate::input::InputState;
use crate::projector::Projector;
use crate::signal::Signal;
use crate::temporal::{DeltaFrame, DeltaProgram, DeltaType};

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

        let hit = state.pointer.x >= x
            && state.pointer.x <= x + w
            && state.pointer.y >= y
            && state.pointer.y <= y + h;

        let pressed = hit && state.pointer.is_down;
        let focused = state.focused == Some(self.target);

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
        } else if focused {
            // Subtle highlight so keyboard focus is visible without a pointer hover
            let mut frame = DeltaFrame::zero(0, 1);
            frame.delta_b = 0.2;
            schedule.push(frame);
        }

        let program = DeltaProgram::new(self.target, schedule, DeltaType::Greasy);

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

        let hit = state.pointer.x >= x
            && state.pointer.x <= x + w
            && state.pointer.y >= y
            && state.pointer.y <= y + h;

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

        let program = DeltaProgram::new(self.target, schedule, DeltaType::Greasy);

        Projection {
            delta: program,
            layer: Layer::Content,
            behaviour: BehaviourFlag::None,
        }
    }
}

/// Reveals a tooltip cell while the pointer hovers `owner_geometry`.
///
/// The tooltip cell itself must be registered hidden (see
/// `Runtime::register_tooltip`, which applies a persistent Adhesive delta
/// to start it at opacity 0). This projector only handles the temporary
/// reveal-on-hover: it emits a single-tick Greasy `delta_a` pulse while
/// hovering, exactly like `ButtonProjector`'s hover highlight.
///
/// Note on duration: `TemporalMemory` accumulates a Greasy frame's delta
/// once per tick it's active and reverts the *full* accumulated sum in one
/// shot when it finishes, so a single long-duration frame overshoots badly
/// on reversion (confirmed empirically: a 90-tick, delta_a=1.0 frame
/// reverted to opacity -89, not 0). A single-tick pulse sidesteps that
/// entirely. Because `project()` only reruns when the input signal changes,
/// this means the tooltip only stays visually revealed for as long as *some*
/// input event (even incidental pointer jitter) keeps arriving while
/// hovering - the same characteristic `ButtonProjector`'s hover highlight
/// already has in production. A pointer held perfectly motionless with zero
/// further events will revert almost immediately; this is a limitation of
/// the stateless Projector model (see [`Projector`]), not specific to
/// tooltips.
pub struct TooltipProjector {
    /// The tooltip's own rendered cell.
    pub target: CellId,
    /// Hit-test geometry of the widget that owns this tooltip (hovering
    /// this reveals the tooltip, not the tooltip's own geometry).
    pub owner_geometry: VectorRegion,
}

impl Projector<InputState> for TooltipProjector {
    fn project(&self, signal: &Signal<InputState>) -> Projection {
        let state = &signal.payload;

        let x = self.owner_geometry.origin_s * 100.0;
        let y = self.owner_geometry.origin_p * 100.0;
        let w = self.owner_geometry.extent_s * 100.0;
        let h = self.owner_geometry.extent_p * 100.0;

        let hovering = state.pointer.x >= x
            && state.pointer.x <= x + w
            && state.pointer.y >= y
            && state.pointer.y <= y + h;

        let mut schedule = Vec::new();

        if hovering {
            let mut frame = DeltaFrame::zero(0, 1);
            frame.delta_a = 1.0;
            schedule.push(frame);
        }
        // While not hovering: emit nothing. The persistent hidden baseline
        // (set once at registration time) keeps it hidden with no
        // re-triggering needed.

        let program = DeltaProgram::new(self.target, schedule, DeltaType::Greasy);

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
    use crate::input::{InputDispatcher, InputEvent};

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
        let proj = button.project(signal);

        // Should have 1 frame (hover effect)
        assert_eq!(proj.delta.schedule.len(), 1);
        assert!(proj.delta.schedule[0].delta_r > 0.0);
    }

    #[test]
    fn test_tooltip_reveals_only_while_hovering_owner() {
        let owner_geo = VectorRegion::new(0.0, 0.0, 1.0, 1.0); // 0-100, 0-100
        let tooltip = TooltipProjector {
            target: CellId(1),
            owner_geometry: owner_geo,
        };

        let mut dispatcher = InputDispatcher::new();

        // Pointer outside the owner: no reveal.
        dispatcher.dispatch(InputEvent::PointerMove(500.0, 500.0));
        let proj = tooltip.project(dispatcher.signal());
        assert!(proj.delta.schedule.is_empty());

        // Pointer over the owner: reveal pulse present.
        dispatcher.dispatch(InputEvent::PointerMove(50.0, 50.0));
        let proj = tooltip.project(dispatcher.signal());
        assert_eq!(proj.delta.schedule.len(), 1);
        assert_eq!(proj.delta.schedule[0].delta_a, 1.0);
        assert_eq!(proj.delta.schedule[0].t_end, 1);
    }
}
