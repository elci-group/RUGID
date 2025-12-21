use std::collections::HashSet;
use crate::signal::{Signal, SignalId};

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
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    PointerMove(f32, f32),
    PointerDown,
    PointerUp,
    KeyDown(u32),
    KeyUp(u32),
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
}

impl InputDispatcher {
    pub fn new() -> Self {
        Self {
            signal: Signal::new(InputState::default()),
        }
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
        }

        self.signal = self.signal.update(new_state);
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
}
