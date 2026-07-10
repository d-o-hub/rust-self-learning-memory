use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use parking_lot::Mutex;

/// Process-wide retry budget with a sliding window.
///
/// Tokens refill at the start of each window. Multiple callers share
/// the same [`Arc<RetryBudget>`] to coordinate retry pressure across
/// the entire process.
pub struct RetryBudget {
    max_tokens: u32,
    window: Duration,
    tokens: AtomicU32,
    window_start: Mutex<Instant>,
}

impl RetryBudget {
    /// Create a shared retry budget.
    ///
    /// - `max_tokens`: how many retries are allowed per window.
    /// - `window`: the sliding window duration.
    pub fn new(max_tokens: u32, window: Duration) -> Arc<Self> {
        Arc::new(Self {
            max_tokens,
            window,
            tokens: AtomicU32::new(max_tokens),
            window_start: Mutex::new(Instant::now()),
        })
    }

    /// Try to acquire one retry token.
    ///
    /// Returns `true` if a token was available (and consumed), `false` if
    /// the budget is exhausted for the current window.
    pub fn acquire(&self) -> bool {
        self.maybe_refill();
        let prev = self.tokens.fetch_sub(1, Ordering::SeqCst);
        if prev > 0 {
            true
        } else {
            // We went below zero — put it back (saturating).
            self.tokens.fetch_add(1, Ordering::SeqCst);
            false
        }
    }

    /// Number of tokens remaining in the current window.
    pub fn available(&self) -> u32 {
        self.maybe_refill();
        self.tokens.load(Ordering::SeqCst)
    }

    fn maybe_refill(&self) {
        let mut start = self.window_start.lock();
        let elapsed = start.elapsed();
        if elapsed >= self.window {
            self.tokens.store(self.max_tokens, Ordering::SeqCst);
            *start = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_exhaustion() {
        let budget = RetryBudget::new(2, Duration::from_secs(10));
        assert!(budget.acquire());
        assert!(budget.acquire());
        assert!(!budget.acquire());
        assert_eq!(budget.available(), 0);
    }

    #[test]
    fn window_refill() {
        let budget = RetryBudget::new(1, Duration::from_millis(50));
        assert!(budget.acquire());
        assert!(!budget.acquire());

        std::thread::sleep(Duration::from_millis(60));
        assert!(budget.acquire());
        assert!(!budget.acquire());
    }

    #[test]
    fn concurrent_acquisition() {
        use std::sync::Arc;
        use std::thread;

        let budget = RetryBudget::new(5, Duration::from_secs(10));
        let budget = Arc::new(budget);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let b = Arc::clone(&budget);
                thread::spawn(move || b.acquire())
            })
            .collect();

        let mut acquired = 0;
        for h in handles {
            if h.join().unwrap() {
                acquired += 1;
            }
        }
        assert_eq!(acquired, 5);
        assert_eq!(budget.available(), 0);
    }
}
