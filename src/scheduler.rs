use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MegaCellId(pub u64);

impl fmt::Debug for MegaCellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MegaCell({})", self.0)
    }
}

pub struct Scheduler {
    // Map of MegaCellId -> Age (ticks since last scheduled)
    // Higher age = higher priority
    active_megacells: HashMap<MegaCellId, u64>,
    max_megacells_per_tick: usize,
}

impl Scheduler {
    pub fn new(max_megacells_per_tick: usize) -> Self {
        Self {
            active_megacells: HashMap::new(),
            max_megacells_per_tick,
        }
    }

    /// Register a MegaCell as needing work.
    /// If already registered, it keeps its current age (priority).
    pub fn schedule(&mut self, id: MegaCellId) {
        self.active_megacells.entry(id).or_insert(0);
    }

    /// Remove a MegaCell from the scheduler (e.g., if it becomes idle).
    pub fn unschedule(&mut self, id: MegaCellId) {
        self.active_megacells.remove(&id);
    }

    /// Get the next batch of MegaCells to process.
    /// Updates the age of deferred MegaCells.
    pub fn next_batch(&mut self) -> Vec<MegaCellId> {
        if self.active_megacells.is_empty() {
            return Vec::new();
        }

        // Sort by age (descending)
        let mut entries: Vec<(MegaCellId, u64)> = self.active_megacells.iter().map(|(k, v)| (*k, *v)).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        let count = self.max_megacells_per_tick.min(entries.len());
        let (scheduled, deferred) = entries.split_at(count);

        let scheduled_ids: Vec<MegaCellId> = scheduled.iter().map(|(id, _)| *id).collect();

        // Reset age for scheduled
        for id in &scheduled_ids {
            if let Some(age) = self.active_megacells.get_mut(id) {
                *age = 0;
            }
        }

        // Increment age for deferred
        for (id, _) in deferred {
            if let Some(age) = self.active_megacells.get_mut(id) {
                *age += 1;
            }
        }

        scheduled_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_budget() {
        let mut scheduler = Scheduler::new(2);
        
        scheduler.schedule(MegaCellId(1));
        scheduler.schedule(MegaCellId(2));
        scheduler.schedule(MegaCellId(3));

        let batch = scheduler.next_batch();
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_scheduler_fairness() {
        let mut scheduler = Scheduler::new(1);
        
        scheduler.schedule(MegaCellId(1));
        scheduler.schedule(MegaCellId(2));

        // Both have age 0. Tie-break might be arbitrary (stable sort?).
        // Let's see who gets picked.
        let batch1 = scheduler.next_batch();
        assert_eq!(batch1.len(), 1);
        let picked1 = batch1[0];

        // The other one should have age 1 now.
        // The picked one has age 0.
        
        let batch2 = scheduler.next_batch();
        assert_eq!(batch2.len(), 1);
        let picked2 = batch2[0];

        assert_ne!(picked1, picked2);
    }
}
