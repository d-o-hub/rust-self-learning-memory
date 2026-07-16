use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;

/// Error returned when acquiring a concurrency permit times out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueTimeout;

impl std::fmt::Display for QueueTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "timed out waiting for a retry concurrency permit (retry_queue_timeout)"
        )
    }
}

impl std::error::Error for QueueTimeout {}

/// Concurrency limiter for retries using tokio semaphore permits.
///
/// Wraps a [`Semaphore`] so that at most `max_concurrent` retry
/// operations run simultaneously. Callers acquire a permit before
/// starting the retryable work and the permit is automatically released
/// when the returned guard is dropped.
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl ConcurrencyLimiter {
    /// Create a new limiter.
    ///
    /// # Panics
    ///
    /// Panics if `max_concurrent` is zero (S1.6: reject zero concurrency).
    pub fn new(max_concurrent: usize) -> Arc<Self> {
        assert!(
            max_concurrent > 0,
            "max_concurrent_retries must be greater than zero"
        );
        Arc::new(Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        })
    }

    /// Acquire a concurrency permit.
    ///
    /// Returns an owned permit that releases when dropped. This method
    /// waits (async) until a permit is available.
    pub async fn acquire(&self) -> OwnedSemaphorePermit {
        Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .expect("semaphore closed unexpectedly")
    }

    /// Acquire a permit, failing with [`QueueTimeout`] if `timeout_dur` elapses first.
    pub async fn acquire_timeout(
        &self,
        timeout_dur: Duration,
    ) -> Result<OwnedSemaphorePermit, QueueTimeout> {
        match timeout(timeout_dur, self.acquire()).await {
            Ok(permit) => Ok(permit),
            Err(_elapsed) => Err(QueueTimeout),
        }
    }

    /// Number of permits currently available (not held).
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Maximum concurrent retries this limiter allows.
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn limits_concurrency() {
        let limiter = ConcurrencyLimiter::new(2);
        let running = Arc::new(AtomicUsize::new(0));
        let max_observed = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..5 {
            let limiter = Arc::clone(&limiter);
            let running = Arc::clone(&running);
            let max_observed = Arc::clone(&max_observed);
            handles.push(tokio::spawn(async move {
                let _permit = limiter.acquire().await;
                let cur = running.fetch_add(1, Ordering::SeqCst) + 1;
                max_observed.fetch_max(cur, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(10)).await;
                running.fetch_sub(1, Ordering::SeqCst);
            }));
        }

        for h in handles {
            h.await.unwrap();
        }
        assert!(
            max_observed.load(Ordering::SeqCst) <= 2,
            "max observed concurrency was {}",
            max_observed.load(Ordering::SeqCst)
        );
    }

    #[tokio::test]
    async fn available_permits_decrease() {
        let limiter = ConcurrencyLimiter::new(3);
        assert_eq!(limiter.available_permits(), 3);

        let p1 = limiter.acquire().await;
        assert_eq!(limiter.available_permits(), 2);

        let p2 = limiter.acquire().await;
        assert_eq!(limiter.available_permits(), 1);

        drop(p1);
        assert_eq!(limiter.available_permits(), 2);

        drop(p2);
        assert_eq!(limiter.available_permits(), 3);
    }

    #[tokio::test]
    #[should_panic(expected = "max_concurrent_retries must be greater than zero")]
    async fn rejects_zero_concurrency() {
        let _ = ConcurrencyLimiter::new(0);
    }

    #[tokio::test]
    async fn acquire_timeout_returns_error_when_saturated() {
        let limiter = ConcurrencyLimiter::new(1);
        let _held = limiter.acquire().await;

        let result = limiter.acquire_timeout(Duration::from_millis(20)).await;
        assert!(result.is_err());
    }
}
