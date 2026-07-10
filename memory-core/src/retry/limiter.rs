use std::sync::Arc;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

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
    pub fn new(max_concurrent: usize) -> Arc<Self> {
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
}
