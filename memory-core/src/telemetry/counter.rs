use std::sync::atomic::{AtomicU64, Ordering};

/// Counter metric for monotonically increasing values
///
/// Counters track values that only increase over time, such as
/// total number of operations executed.
#[derive(Debug)]
pub struct Counter {
    /// Inner atomic counter
    value: AtomicU64,
}

impl Counter {
    /// Create a new counter with value 0
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Increment the counter by 1
    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the counter by a specific value
    pub fn increment_by(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }

    /// Get the current counter value
    #[must_use]
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let counter = Counter::new();
        counter.increment();
        counter.increment();

        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_increment_by() {
        let counter = Counter::new();
        counter.increment_by(5);
        counter.increment_by(3);

        assert_eq!(counter.get(), 8);
    }

    #[test]
    fn test_default() {
        let counter = Counter::default();
        assert_eq!(counter.get(), 0);
    }
}
