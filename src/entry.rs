//! Log entry structure with sequence numbers.

/// Type of log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    /// Normal data entry.
    Data,
    /// Checkpoint marker.
    Checkpoint,
    /// Transaction commit marker.
    Commit,
}

/// A single entry in the write-ahead log.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Monotonically increasing sequence number.
    pub sequence: u64,
    /// Type of this entry.
    pub entry_type: EntryType,
    /// Payload data.
    pub data: Vec<u8>,
}

impl Entry {
    /// Create a new data entry.
    pub fn new_data(sequence: u64, data: Vec<u8>) -> Self {
        Entry {
            sequence,
            entry_type: EntryType::Data,
            data,
        }
    }

    /// Create a new checkpoint entry.
    pub fn new_checkpoint(sequence: u64) -> Self {
        Entry {
            sequence,
            entry_type: EntryType::Checkpoint,
            data: Vec::new(),
        }
    }

    /// Create a new commit entry.
    pub fn new_commit(sequence: u64) -> Self {
        Entry {
            sequence,
            entry_type: EntryType::Commit,
            data: Vec::new(),
        }
    }

    /// Is this a data entry?
    pub fn is_data(&self) -> bool {
        self.entry_type == EntryType::Data
    }

    /// Is this a checkpoint entry?
    pub fn is_checkpoint(&self) -> bool {
        self.entry_type == EntryType::Checkpoint
    }

    /// Is this a commit entry?
    pub fn is_commit(&self) -> bool {
        self.entry_type == EntryType::Commit
    }

    /// Get the sequence number.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Get the data payload.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Size of this entry in bytes.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_entry() {
        let entry = Entry::new_data(1, b"hello".to_vec());
        assert!(entry.is_data());
        assert!(!entry.is_checkpoint());
        assert_eq!(entry.sequence(), 1);
        assert_eq!(entry.data(), b"hello");
    }

    #[test]
    fn test_checkpoint_entry() {
        let entry = Entry::new_checkpoint(5);
        assert!(entry.is_checkpoint());
        assert!(!entry.is_data());
        assert_eq!(entry.size(), 0);
    }

    #[test]
    fn test_commit_entry() {
        let entry = Entry::new_commit(3);
        assert!(entry.is_commit());
        assert_eq!(entry.sequence(), 3);
    }

    #[test]
    fn test_entry_size() {
        let entry = Entry::new_data(1, vec![0u8; 100]);
        assert_eq!(entry.size(), 100);
    }
}
