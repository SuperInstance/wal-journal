//! Core WAL log operations.

use crate::checkpoint::CheckpointManager;
use crate::entry::{Entry, EntryType};
use crate::recovery::{self, RecoveryResult};
use crate::sequence::SequenceGenerator;

/// Write-ahead log.
#[derive(Debug, Clone)]
pub struct Wal {
    entries: Vec<Entry>,
    seq_gen: SequenceGenerator,
    checkpoint_mgr: CheckpointManager,
}

impl Wal {
    /// Create a new empty WAL.
    pub fn new() -> Self {
        Wal {
            entries: Vec::new(),
            seq_gen: SequenceGenerator::new(),
            checkpoint_mgr: CheckpointManager::new(),
        }
    }

    /// Append a data entry to the log.
    pub fn append(&mut self, data: &[u8]) -> u64 {
        let seq = self.seq_gen.next();
        let entry = Entry::new_data(seq, data.to_vec());
        self.entries.push(entry);
        seq
    }

    /// Append a commit marker.
    pub fn append_commit(&mut self) -> u64 {
        let seq = self.seq_gen.next();
        let entry = Entry::new_commit(seq);
        self.entries.push(entry);
        seq
    }

    /// Create a checkpoint at the current position.
    pub fn checkpoint(&mut self) -> u64 {
        let seq = self.seq_gen.next();
        let entry = Entry::new_checkpoint(seq);
        self.entries.push(entry);
        self.checkpoint_mgr.create(seq, self.entries.len());
        seq
    }

    /// Recover entries from the log (simulate replay).
    pub fn recover(&self) -> RecoveryResult {
        recovery::recover(self.entries.clone(), &self.checkpoint_mgr)
    }

    /// Get all entries.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Is the log empty?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the current sequence number.
    pub fn current_sequence(&self) -> u64 {
        self.seq_gen.current()
    }

    /// Truncate entries before the last checkpoint.
    pub fn truncate_before_checkpoint(&mut self) {
        let point = self.checkpoint_mgr.truncation_point();
        if point > 0 && point <= self.entries.len() {
            self.entries.drain(0..point);
        }
    }

    /// Get entries of a specific type.
    pub fn entries_of_type(&self, entry_type: EntryType) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|e| e.entry_type == entry_type)
            .collect()
    }

    /// Get a range of entries by sequence number.
    pub fn range(&self, start_seq: u64, end_seq: u64) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|e| e.sequence >= start_seq && e.sequence <= end_seq)
            .collect()
    }

    /// Total size of all entry data.
    pub fn total_data_size(&self) -> usize {
        self.entries.iter().map(|e| e.data.len()).sum()
    }

    /// Reset the WAL (for testing or after full recovery).
    pub fn reset(&mut self) {
        self.entries.clear();
        self.seq_gen = SequenceGenerator::new();
        self.checkpoint_mgr = CheckpointManager::new();
    }
}

impl Default for Wal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append() {
        let mut wal = Wal::new();
        let seq = wal.append(b"hello");
        assert_eq!(seq, 0);
        assert_eq!(wal.len(), 1);
    }

    #[test]
    fn test_multiple_appends() {
        let mut wal = Wal::new();
        wal.append(b"a");
        wal.append(b"b");
        wal.append(b"c");
        assert_eq!(wal.len(), 3);
        assert_eq!(wal.entries()[0].sequence, 0);
        assert_eq!(wal.entries()[2].sequence, 2);
    }

    #[test]
    fn test_checkpoint_and_truncate() {
        let mut wal = Wal::new();
        wal.append(b"data1");
        wal.append(b"data2");
        wal.checkpoint();
        wal.append(b"data3");
        assert_eq!(wal.len(), 4); // 2 data + 1 checkpoint + 1 data
        wal.truncate_before_checkpoint();
        assert!(wal.len() <= 2);
    }

    #[test]
    fn test_recovery() {
        let mut wal = Wal::new();
        wal.append(b"tx1");
        wal.append(b"tx2");
        let result = wal.recover();
        assert_eq!(result.len(), 2);
        assert!(result.clean);
    }

    #[test]
    fn test_recovery_after_checkpoint() {
        let mut wal = Wal::new();
        wal.append(b"old_data");
        wal.checkpoint();
        wal.append(b"new_data");
        let result = wal.recover();
        // Only entries after checkpoint should be recovered
        assert!(result.len() <= 1);
    }

    #[test]
    fn test_sequence_ordering() {
        let mut wal = Wal::new();
        let s1 = wal.append(b"a");
        let s2 = wal.append(b"b");
        let s3 = wal.checkpoint();
        let s4 = wal.append(b"c");
        assert!(s1 < s2);
        assert!(s2 < s3);
        assert!(s3 < s4);
    }

    #[test]
    fn test_entries_of_type() {
        let mut wal = Wal::new();
        wal.append(b"data");
        wal.append_commit();
        wal.checkpoint();
        let data_entries = wal.entries_of_type(EntryType::Data);
        assert_eq!(data_entries.len(), 1);
        let commits = wal.entries_of_type(EntryType::Commit);
        assert_eq!(commits.len(), 1);
    }

    #[test]
    fn test_range_query() {
        let mut wal = Wal::new();
        for i in 0..10 {
            wal.append(format!("data_{}", i).as_bytes());
        }
        let range = wal.range(3, 6);
        assert_eq!(range.len(), 4);
    }

    #[test]
    fn test_total_data_size() {
        let mut wal = Wal::new();
        wal.append(b"hello"); // 5 bytes
        wal.append(b"world"); // 5 bytes
        assert_eq!(wal.total_data_size(), 10);
    }

    #[test]
    fn test_reset() {
        let mut wal = Wal::new();
        wal.append(b"data");
        wal.checkpoint();
        wal.reset();
        assert!(wal.is_empty());
        assert_eq!(wal.current_sequence(), 0);
    }

    #[test]
    fn test_empty_wal() {
        let wal = Wal::new();
        assert!(wal.is_empty());
        assert_eq!(wal.current_sequence(), 0);
    }
}
