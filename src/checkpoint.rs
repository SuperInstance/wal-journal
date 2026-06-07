//! Checkpoint management for the WAL.

use crate::entry::Entry;

/// A checkpoint marks a point in the WAL up to which all entries have been persisted.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Sequence number of the checkpoint.
    pub sequence: u64,
    /// Number of entries up to and including this checkpoint.
    pub entry_count: usize,
}

impl Checkpoint {
    /// Create a new checkpoint.
    pub fn new(sequence: u64, entry_count: usize) -> Self {
        Checkpoint {
            sequence,
            entry_count,
        }
    }

    /// Get the checkpoint sequence number.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Get the entry count at checkpoint time.
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }
}

/// Manages checkpoints for the WAL.
#[derive(Debug, Clone)]
pub struct CheckpointManager {
    /// The last successful checkpoint.
    last_checkpoint: Option<Checkpoint>,
    /// All checkpoints (for history).
    checkpoints: Vec<Checkpoint>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager.
    pub fn new() -> Self {
        CheckpointManager {
            last_checkpoint: None,
            checkpoints: Vec::new(),
        }
    }

    /// Create a checkpoint at the given sequence number.
    pub fn create(&mut self, sequence: u64, entry_count: usize) -> Checkpoint {
        let cp = Checkpoint::new(sequence, entry_count);
        self.last_checkpoint = Some(cp.clone());
        self.checkpoints.push(cp.clone());
        cp
    }

    /// Get the last checkpoint.
    pub fn last(&self) -> Option<&Checkpoint> {
        self.last_checkpoint.as_ref()
    }

    /// Get all checkpoints.
    pub fn all(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    /// Number of checkpoints created.
    pub fn count(&self) -> usize {
        self.checkpoints.len()
    }

    /// Determine which entries can be truncated after a checkpoint.
    /// Returns the index up to which entries can be safely removed.
    pub fn truncation_point(&self) -> usize {
        self.last_checkpoint.as_ref().map_or(0, |cp| cp.entry_count)
    }

    /// Check if an entry is before the last checkpoint (can be truncated).
    pub fn is_before_checkpoint(&self, entry: &Entry) -> bool {
        self.last_checkpoint
            .as_ref()
            .is_some_and(|cp| entry.sequence < cp.sequence)
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_checkpoint() {
        let mut mgr = CheckpointManager::new();
        let cp = mgr.create(10, 5);
        assert_eq!(cp.sequence(), 10);
        assert_eq!(cp.entry_count(), 5);
    }

    #[test]
    fn test_last_checkpoint() {
        let mut mgr = CheckpointManager::new();
        assert!(mgr.last().is_none());
        mgr.create(5, 3);
        assert_eq!(mgr.last().unwrap().sequence(), 5);
        mgr.create(10, 6);
        assert_eq!(mgr.last().unwrap().sequence(), 10);
    }

    #[test]
    fn test_checkpoint_count() {
        let mut mgr = CheckpointManager::new();
        mgr.create(1, 1);
        mgr.create(2, 2);
        mgr.create(3, 3);
        assert_eq!(mgr.count(), 3);
    }

    #[test]
    fn test_truncation_point() {
        let mut mgr = CheckpointManager::new();
        assert_eq!(mgr.truncation_point(), 0);
        mgr.create(10, 5);
        assert_eq!(mgr.truncation_point(), 5);
    }

    #[test]
    fn test_is_before_checkpoint() {
        let mut mgr = CheckpointManager::new();
        mgr.create(10, 5);
        let entry = crate::entry::Entry::new_data(5, vec![]);
        assert!(mgr.is_before_checkpoint(&entry));
        let entry2 = crate::entry::Entry::new_data(15, vec![]);
        assert!(!mgr.is_before_checkpoint(&entry2));
    }
}
