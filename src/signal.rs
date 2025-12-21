use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt;

static SIGNAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(u64);

impl fmt::Debug for SignalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sig({})", self.0)
    }
}

impl SignalId {
    fn next() -> Self {
        SignalId(SIGNAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// A signal represents a discrete, immutable snapshot of state.
///
/// Constraints:
/// * Signals are immutable after creation
/// * Signals are never borrowed
/// * Signals contain no UI or layout knowledge
#[derive(Clone)]
pub struct Signal<T> {
    pub id: SignalId,
    pub version: u64,
    pub payload: Arc<T>,
}

impl<T> Signal<T> {
    pub fn new(payload: T) -> Self {
        Self {
            id: SignalId::next(),
            version: 1,
            payload: Arc::new(payload),
        }
    }

    /// Create a new version of this signal with a new payload.
    /// The ID remains the same, but the version increments.
    pub fn update(&self, new_payload: T) -> Self {
        Self {
            id: self.id,
            version: self.version + 1,
            payload: Arc::new(new_payload),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signal")
            .field("id", &self.id)
            .field("version", &self.version)
            .field("payload", &self.payload)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let sig = Signal::new(42);
        assert_eq!(sig.version, 1);
        assert_eq!(*sig.payload, 42);
    }

    #[test]
    fn test_signal_update() {
        let sig1 = Signal::new(10);
        let sig2 = sig1.update(20);

        assert_eq!(sig1.id, sig2.id);
        assert_eq!(sig1.version, 1);
        assert_eq!(sig2.version, 2);
        assert_eq!(*sig1.payload, 10);
        assert_eq!(*sig2.payload, 20);
    }

    #[test]
    fn test_signal_immutability() {
        let sig = Signal::new(vec![1, 2, 3]);
        let _sig_clone = sig.clone();
        // Cannot modify payload directly as it is in an Arc and we don't expose mutable access
    }
}
