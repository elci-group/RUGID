use std::collections::{HashMap, HashSet};
use crate::signal::SignalId;
use crate::cell::CellId;

/// The BeaconWatcher:
/// * Tracks signal versions
/// * Maintains signal → cell dependency mappings
/// * Emits targeted invalidation events
///
/// Responsibilities:
/// * No rendering
/// * No geometry
/// * No policy decisions
pub struct BeaconWatcher {
    versions: HashMap<SignalId, u64>,
    dependencies: HashMap<SignalId, HashSet<CellId>>,
}

impl BeaconWatcher {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Register a signal or update its tracked version.
    /// Returns true if the version changed (meaning invalidation might be needed).
    pub fn track(&mut self, id: SignalId, version: u64) -> bool {
        let entry = self.versions.entry(id).or_insert(0);
        if *entry != version {
            *entry = version;
            true
        } else {
            false
        }
    }

    pub fn get_version(&self, id: SignalId) -> Option<u64> {
        self.versions.get(&id).copied()
    }

    /// Subscribe a cell to a signal.
    pub fn subscribe(&mut self, cell: CellId, signal: SignalId) {
        self.dependencies.entry(signal).or_default().insert(cell);
    }

    /// Identify cells that need to be re-evaluated due to a signal change.
    /// This should be called after `track` returns true.
    pub fn invalidate(&self, signal: SignalId) -> Vec<CellId> {
        self.dependencies
            .get(&signal)
            .map(|cells| cells.iter().copied().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::Signal;

    use crate::cell::Cell;
    use crate::geometry::VectorRegion;

    #[test]
    fn test_beacon_tracking() {
        let mut beacon = BeaconWatcher::new();
        let sig = Signal::new(100);

        // First track
        assert!(beacon.track(sig.id, sig.version));
        assert_eq!(beacon.get_version(sig.id), Some(1));

        // Same version, no change
        assert!(!beacon.track(sig.id, sig.version));

        // Update signal
        let sig2 = sig.update(200);
        assert!(beacon.track(sig2.id, sig2.version));
        assert_eq!(beacon.get_version(sig.id), Some(2));
    }

    #[test]
    fn test_beacon_dependencies() {
        let mut beacon = BeaconWatcher::new();
        let sig1 = Signal::new(10);
        let sig2 = Signal::new(20);
        let cell1 = Cell::new(VectorRegion::new(0.0, 0.0, 1.0, 1.0));
        let cell2 = Cell::new(VectorRegion::new(0.0, 0.0, 1.0, 1.0));

        // Subscribe cells
        beacon.subscribe(cell1.id, sig1.id);
        beacon.subscribe(cell2.id, sig1.id);
        beacon.subscribe(cell2.id, sig2.id);

        // Invalidate sig1 -> should affect cell1 and cell2
        let invalid1 = beacon.invalidate(sig1.id);
        assert_eq!(invalid1.len(), 2);
        assert!(invalid1.contains(&cell1.id));
        assert!(invalid1.contains(&cell2.id));

        // Invalidate sig2 -> should affect only cell2
        let invalid2 = beacon.invalidate(sig2.id);
        assert_eq!(invalid2.len(), 1);
        assert!(invalid2.contains(&cell2.id));

        // Invalidate unknown signal -> empty
        let sig_unknown = Signal::new(99);
        assert!(beacon.invalidate(sig_unknown.id).is_empty());
    }
}
