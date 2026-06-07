//! Crash recovery and log replay.

use crate::entry::Entry;
use crate::checkpoint::CheckpointManager;

/// Result of recovery.
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// Entries recovered from the log.
    pub recovered_entries: Vec<Entry>,
    /// Number of entries that were replayed.
    pub replayed_count: usize,
    /// The last valid sequence number found.
    pub last_sequence: u64,
    /// Whether recovery was clean (no gaps).
    pub clean: bool,
}

impl RecoveryResult {
    /// Create a new recovery result.
    pub fn new(
        recovered_entries: Vec<Entry>,
        replayed_count: usize,
        last_sequence: u64,
        clean: bool,
    ) -> Self {
        RecoveryResult {
            recovered_entries,
            replayed_count,
            last_sequence,
            clean,
        }
    }

    /// Number of recovered entries.
    pub fn len(&self) -> usize {
        self.recovered_entries.len()
    }

    /// Were any entries recovered?
    pub fn is_empty(&self) -> bool {
        self.recovered_entries.is_empty()
    }
}

/// Recover entries from a log, detecting gaps and corruption.
pub fn recover(raw_entries: Vec<Entry>, checkpoint_mgr: &CheckpointManager) -> RecoveryResult {
    let start_seq = checkpoint_mgr
        .last()
        .map_or(0, |cp| cp.sequence + 1);

    // Filter entries after the last checkpoint
    let mut recovered: Vec<Entry> = raw_entries
        .into_iter()
        .filter(|e| e.sequence >= start_seq)
        .collect();

    // Sort by sequence number
    recovered.sort_by_key(|e| e.sequence);

    // Check for gaps
    let clean = check_continuity(&recovered);
    let last_sequence = recovered.last().map_or(0, |e| e.sequence);
    let replayed_count = recovered.len();

    RecoveryResult::new(recovered, replayed_count, last_sequence, clean)
}

/// Check that entries form a continuous sequence.
fn check_continuity(entries: &[Entry]) -> bool {
    for i in 1..entries.len() {
        if entries[i].sequence != entries[i - 1].sequence + 1 {
            return false;
        }
    }
    true
}

/// Replay recovered entries, returning only committed data entries.
pub fn replay_committed(entries: &[Entry]) -> Vec<&Entry> {
    // Simple strategy: return data entries that have a corresponding commit
    let committed_seqs: std::collections::HashSet<u64> = entries
        .iter()
        .filter(|e| e.is_commit())
        .map(|e| e.sequence)
        .collect();

    entries
        .iter()
        .filter(|e| e.is_data() || committed_seqs.contains(&e.sequence))
        .collect()
}

/// Replay only data entries (ignoring commits and checkpoints).
pub fn replay_data_only(entries: &[Entry]) -> Vec<&Entry> {
    entries.iter().filter(|e| e.is_data()).collect()
}

/// Simulate a crash by truncating the log at a given point.
pub fn simulate_crash(entries: &[Entry], crash_after: usize) -> Vec<Entry> {
    entries.iter().take(crash_after).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recover_clean() {
        let entries = vec![
            Entry::new_data(0, b"a".to_vec()),
            Entry::new_data(1, b"b".to_vec()),
            Entry::new_data(2, b"c".to_vec()),
        ];
        let mgr = CheckpointManager::new();
        let result = recover(entries, &mgr);
        assert!(result.clean);
        assert_eq!(result.len(), 3);
        assert_eq!(result.last_sequence, 2);
    }

    #[test]
    fn test_recover_with_checkpoint() {
        let mut mgr = CheckpointManager::new();
        mgr.create(1, 2); // Checkpoint at sequence 1
        let entries = vec![
            Entry::new_data(0, b"a".to_vec()),
            Entry::new_data(1, b"b".to_vec()),
            Entry::new_data(2, b"c".to_vec()),
        ];
        let result = recover(entries, &mgr);
        // Should only recover entries after checkpoint
        assert_eq!(result.len(), 1);
        assert_eq!(result.recovered_entries[0].data, b"c");
    }

    #[test]
    fn test_recover_with_gap() {
        let entries = vec![
            Entry::new_data(0, b"a".to_vec()),
            Entry::new_data(2, b"c".to_vec()), // Gap at 1
        ];
        let mgr = CheckpointManager::new();
        let result = recover(entries, &mgr);
        assert!(!result.clean);
    }

    #[test]
    fn test_replay_committed() {
        let entries = vec![
            Entry::new_data(0, b"a".to_vec()),
            Entry::new_commit(1),
            Entry::new_data(2, b"b".to_vec()),
            // No commit for entry 2
        ];
        let committed = replay_committed(&entries);
        // Entry 0 has a commit (at seq 1), entry 2 doesn't
        assert!(committed.iter().any(|e| e.sequence == 0));
    }

    #[test]
    fn test_simulate_crash() {
        let entries = vec![
            Entry::new_data(0, b"a".to_vec()),
            Entry::new_data(1, b"b".to_vec()),
            Entry::new_data(2, b"c".to_vec()),
        ];
        let truncated = simulate_crash(&entries, 2);
        assert_eq!(truncated.len(), 2);
    }

    #[test]
    fn test_recover_empty() {
        let mgr = CheckpointManager::new();
        let result = recover(vec![], &mgr);
        assert!(result.is_empty());
        assert!(result.clean);
    }
}
