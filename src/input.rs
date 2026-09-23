use crate::cell::CellId;
use crate::signal::{Signal, SignalId};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PointerState {
    pub x: f32,
    pub y: f32,
    pub is_down: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InputState {
    pub pointer: PointerState,
    pub keys: HashSet<u32>, // Simple u32 keycodes for now
    /// The cell currently holding keyboard focus, if any.
    pub focused: Option<CellId>,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    PointerMove(f32, f32),
    PointerDown,
    PointerUp,
    KeyDown(u32),
    KeyUp(u32),
    /// Move focus to the next registered focusable cell (Tab).
    FocusNext,
    /// Move focus to the previous registered focusable cell (Shift+Tab).
    FocusPrev,
    /// Move focus directly to a specific cell (e.g. click-to-focus).
    SetFocus(CellId),
    /// Clear keyboard focus entirely.
    ClearFocus,
}

pub struct InputDispatcher {
    // We need interior mutability to update the signal from dispatch
    // In a real app, this might be handled differently (e.g. channels),
    // but for this architecture, we update the signal's payload.
    // However, Signal<T> is immutable. We create a NEW Signal<T> on update?
    // Or Signal<T> is a handle to versioned state?
    // Looking at signal.rs: Signal<T> has `payload: Arc<T>`.
    // To "update" a signal, we usually replace the Signal instance or have a mutable source.
    // But Signal is designed to be a snapshot.
    // The *source* of truth needs to be mutable and produce new Signals.
    // Let's make InputDispatcher hold the *current* Signal and allow replacing it?
    // Or better: InputDispatcher IS the source. It maintains the master state and vends Signals.
    // But BeaconWatcher tracks SignalIds. If we generate a new SignalId every frame, that's fine.
    // BUT, if we want *stable* SignalId but changing version...
    // signal.rs: `pub fn update(&mut self, payload: T)` updates version and payload.
    // So we can just hold a Signal and update it.
    signal: Signal<InputState>,
    // Tab order for keyboard focus navigation. Populated via `register_focusable`.
    focus_order: Vec<CellId>,
}

impl Default for InputDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl InputDispatcher {
    pub fn new() -> Self {
        Self {
            signal: Signal::new(InputState::default()),
            focus_order: Vec::new(),
        }
    }

    /// Register a cell as reachable via Tab/Shift+Tab, in registration order.
    /// Registering the same cell twice is a no-op.
    pub fn register_focusable(&mut self, id: CellId) {
        if !self.focus_order.contains(&id) {
            self.focus_order.push(id);
        }
    }

    /// Remove a cell from the tab order (e.g. when it's destroyed or disabled).
    pub fn unregister_focusable(&mut self, id: CellId) {
        self.focus_order.retain(|&cell| cell != id);
    }

    pub fn dispatch(&mut self, event: InputEvent) {
        // Clone current state to modify it (Copy-on-Write logic effectively)
        let mut new_state = (*self.signal.payload).clone();

        match event {
            InputEvent::PointerMove(x, y) => {
                new_state.pointer.x = x;
                new_state.pointer.y = y;
            }
            InputEvent::PointerDown => {
                new_state.pointer.is_down = true;
            }
            InputEvent::PointerUp => {
                new_state.pointer.is_down = false;
            }
            InputEvent::KeyDown(code) => {
                new_state.keys.insert(code);
            }
            InputEvent::KeyUp(code) => {
                new_state.keys.remove(&code);
            }
            InputEvent::FocusNext => {
                new_state.focused = self.step_focus(new_state.focused, 1);
            }
            InputEvent::FocusPrev => {
                new_state.focused = self.step_focus(new_state.focused, -1);
            }
            InputEvent::SetFocus(id) => {
                new_state.focused = Some(id);
            }
            InputEvent::ClearFocus => {
                new_state.focused = None;
            }
        }

        self.signal = self.signal.update(new_state);
    }

    /// Compute the next focus target `delta` steps away from `current` in the
    /// registered tab order, wrapping around. Returns `None` if nothing is
    /// registered. If `current` isn't registered (or is `None`), starts from
    /// the beginning of the order.
    fn step_focus(&self, current: Option<CellId>, delta: i32) -> Option<CellId> {
        if self.focus_order.is_empty() {
            return None;
        }

        let len = self.focus_order.len() as i32;
        let current_idx = current
            .and_then(|id| self.focus_order.iter().position(|&cell| cell == id))
            .map(|idx| idx as i32);

        let next_idx = match current_idx {
            Some(idx) => (idx + delta).rem_euclid(len),
            // Nothing focused yet: Tab starts at the first item, Shift+Tab at the last.
            None => {
                if delta >= 0 {
                    0
                } else {
                    len - 1
                }
            }
        };

        Some(self.focus_order[next_idx as usize])
    }

    pub fn signal(&self) -> &Signal<InputState> {
        &self.signal
    }

    pub fn signal_id(&self) -> SignalId {
        self.signal.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_dispatch() {
        let mut dispatcher = InputDispatcher::new();
        let initial_version = dispatcher.signal().version;

        // Move pointer
        dispatcher.dispatch(InputEvent::PointerMove(10.0, 20.0));
        assert_eq!(dispatcher.signal().payload.pointer.x, 10.0);
        assert_eq!(dispatcher.signal().payload.pointer.y, 20.0);
        assert!(dispatcher.signal().version > initial_version);

        // Press key
        dispatcher.dispatch(InputEvent::KeyDown(42));
        assert!(dispatcher.signal().payload.keys.contains(&42));
    }

    #[test]
    fn test_focus_tab_cycles_forward_and_wraps() {
        let mut dispatcher = InputDispatcher::new();
        dispatcher.register_focusable(CellId(1));
        dispatcher.register_focusable(CellId(2));
        dispatcher.register_focusable(CellId(3));

        dispatcher.dispatch(InputEvent::FocusNext);
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(1)));

        dispatcher.dispatch(InputEvent::FocusNext);
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(2)));

        dispatcher.dispatch(InputEvent::FocusNext);
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(3)));

        // Wraps back to the first cell.
        dispatcher.dispatch(InputEvent::FocusNext);
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(1)));
    }

    #[test]
    fn test_focus_shift_tab_cycles_backward_and_wraps() {
        let mut dispatcher = InputDispatcher::new();
        dispatcher.register_focusable(CellId(1));
        dispatcher.register_focusable(CellId(2));

        // Shift+Tab with nothing focused starts from the last item.
        dispatcher.dispatch(InputEvent::FocusPrev);
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(2)));

        dispatcher.dispatch(InputEvent::FocusPrev);
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(1)));

        // Wraps back to the last cell.
        dispatcher.dispatch(InputEvent::FocusPrev);
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(2)));
    }

    #[test]
    fn test_set_and_clear_focus() {
        let mut dispatcher = InputDispatcher::new();
        dispatcher.register_focusable(CellId(1));

        dispatcher.dispatch(InputEvent::SetFocus(CellId(1)));
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(1)));

        dispatcher.dispatch(InputEvent::ClearFocus);
        assert_eq!(dispatcher.signal().payload.focused, None);
    }

    #[test]
    fn test_unregister_focusable_removes_from_tab_order() {
        let mut dispatcher = InputDispatcher::new();
        dispatcher.register_focusable(CellId(1));
        dispatcher.register_focusable(CellId(2));
        dispatcher.unregister_focusable(CellId(1));

        dispatcher.dispatch(InputEvent::FocusNext);
        assert_eq!(dispatcher.signal().payload.focused, Some(CellId(2)));
    }

    #[test]
    fn test_focus_next_with_no_registered_cells_is_none() {
        let mut dispatcher = InputDispatcher::new();
        dispatcher.dispatch(InputEvent::FocusNext);
        assert_eq!(dispatcher.signal().payload.focused, None);
    }
}
