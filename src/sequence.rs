//! Monotonic sequence number generation.

/// A monotonically increasing sequence number generator.
#[derive(Debug, Clone)]
pub struct SequenceGenerator {
    current: u64,
}

impl SequenceGenerator {
    /// Create a new sequence generator starting at 0.
    pub fn new() -> Self {
        SequenceGenerator { current: 0 }
    }

    /// Create a generator starting at a specific value.
    pub fn starting_at(start: u64) -> Self {
        SequenceGenerator { current: start }
    }

    /// Generate the next sequence number.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        let seq = self.current;
        self.current += 1;
        seq
    }

    /// Get the current sequence number (last generated).
    pub fn current(&self) -> u64 {
        self.current
    }

    /// Reset to a specific value (e.g., after recovery).
    pub fn reset_to(&mut self, value: u64) {
        self.current = value;
    }

    /// Peek at the next value without advancing.
    pub fn peek(&self) -> u64 {
        self.current
    }
}

impl Default for SequenceGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential() {
        let mut gen = SequenceGenerator::new();
        assert_eq!(gen.next(), 0);
        assert_eq!(gen.next(), 1);
        assert_eq!(gen.next(), 2);
    }

    #[test]
    fn test_starting_at() {
        let mut gen = SequenceGenerator::starting_at(100);
        assert_eq!(gen.next(), 100);
        assert_eq!(gen.next(), 101);
    }

    #[test]
    fn test_current() {
        let mut gen = SequenceGenerator::new();
        gen.next();
        gen.next();
        assert_eq!(gen.current(), 2);
    }

    #[test]
    fn test_reset() {
        let mut gen = SequenceGenerator::new();
        gen.next();
        gen.next();
        gen.reset_to(50);
        assert_eq!(gen.next(), 50);
    }

    #[test]
    fn test_peek() {
        let mut gen = SequenceGenerator::new();
        assert_eq!(gen.peek(), 0);
        gen.next();
        assert_eq!(gen.peek(), 1);
        assert_eq!(gen.next(), 1);
    }

    #[test]
    fn test_default() {
        let gen = SequenceGenerator::default();
        assert_eq!(gen.current(), 0);
    }
}
