pub mod budget;
pub mod limiter;

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

pub use budget::RetryBudget;
pub use limiter::ConcurrencyLimiter;

pub trait Retryable {
    fn is_recoverable(&self) -> bool;
}

impl Retryable for crate::error::Error {
    fn is_recoverable(&self) -> bool {
        self.is_recoverable()
    }
}

#[derive(Debug, Default)]
pub struct RetryMetrics {
    total: AtomicU64,
    success: AtomicU64,
    failure: AtomicU64,
}

impl RetryMetrics {
    #[must_use]
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            success: AtomicU64::new(0),
            failure: AtomicU64::new(0),
        }
    }

    pub fn record_retry(&self, succeeded: bool) {
        self.total.fetch_add(1, Ordering::SeqCst);
        if succeeded {
            self.success.fetch_add(1, Ordering::SeqCst);
        } else {
            self.failure.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[must_use]
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::SeqCst)
    }

    #[must_use]
    pub fn success_count(&self) -> u64 {
        self.success.load(Ordering::SeqCst)
    }

    #[must_use]
    pub fn failure_count(&self) -> u64 {
        self.failure.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter_factor: f64,
    pub max_retry_budget: Option<u32>,
    pub max_concurrent_retries: Option<usize>,
    pub budget_window: Option<Duration>,
    pub retry_queue_timeout: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            jitter_factor: 0.25,
            max_retry_budget: None,
            max_concurrent_retries: None,
            budget_window: None,
            retry_queue_timeout: None,
        }
    }
}

impl RetryConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    #[must_use]
    pub fn with_base_delay(mut self, base_delay: Duration) -> Self {
        self.base_delay = base_delay;
        self
    }

    #[must_use]
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    #[must_use]
    pub fn with_jitter(mut self, factor: f64) -> Self {
        self.jitter_factor = factor;
        self
    }

    #[must_use]
    pub fn with_retry_budget(mut self, budget: u32) -> Self {
        self.max_retry_budget = Some(budget);
        self
    }

    /// Set the maximum number of concurrent retries process-wide.
    #[must_use]
    pub fn with_max_concurrent_retries(mut self, max: usize) -> Self {
        self.max_concurrent_retries = Some(max);
        self
    }

    /// Set the sliding window duration for the shared retry budget.
    #[must_use]
    pub fn with_budget_window(mut self, window: Duration) -> Self {
        self.budget_window = Some(window);
        self
    }

    /// Set a timeout for the retry queue (prevents unbounded queuing).
    #[must_use]
    pub fn with_retry_queue_timeout(mut self, timeout: Duration) -> Self {
        self.retry_queue_timeout = Some(timeout);
        self
    }
}

pub struct RetryPolicy {
    config: RetryConfig,
    metrics: Option<RetryMetrics>,
    retry_budget: Option<NonZeroBudget>,
    shared_budget: Option<Arc<RetryBudget>>,
    limiter: Option<Arc<ConcurrencyLimiter>>,
}

struct NonZeroBudget {
    remaining: u32,
}

impl RetryPolicy {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: RetryConfig::default(),
            metrics: None,
            retry_budget: None,
            shared_budget: None,
            limiter: None,
        }
    }

    #[must_use]
    pub fn with_config(config: RetryConfig) -> Self {
        let retry_budget = config
            .max_retry_budget
            .map(|b| NonZeroBudget { remaining: b });

        let shared_budget = config
            .budget_window
            .zip(config.max_retry_budget)
            .map(|(window, tokens)| RetryBudget::new(tokens, window));

        let limiter = config.max_concurrent_retries.map(ConcurrencyLimiter::new);

        Self {
            config,
            metrics: None,
            retry_budget,
            shared_budget,
            limiter,
        }
    }

    #[must_use]
    pub fn with_metrics(mut self, metrics: RetryMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    #[must_use]
    pub fn with_retry_budget(mut self, budget: u32) -> Self {
        self.retry_budget = Some(NonZeroBudget { remaining: budget });
        self
    }

    /// Attach a shared process-wide retry budget.
    #[must_use]
    pub fn with_shared_budget(mut self, budget: Arc<RetryBudget>) -> Self {
        self.shared_budget = Some(budget);
        self
    }

    /// Attach a concurrency limiter for retries.
    #[must_use]
    pub fn with_limiter(mut self, limiter: Arc<ConcurrencyLimiter>) -> Self {
        self.limiter = Some(limiter);
        self
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let exp_delay = self.config.base_delay * (2u32.pow(attempt.saturating_sub(1)));
        let delay = std::cmp::min(exp_delay, self.config.max_delay);

        if self.config.jitter_factor > 0.0 {
            let jitter_range = delay.as_millis() as f64 * self.config.jitter_factor;
            let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
            let adjusted_ms = (delay.as_millis() as f64 + jitter).max(0.0);
            Duration::from_millis(adjusted_ms as u64)
        } else {
            delay
        }
    }

    fn can_retry(&mut self) -> bool {
        if let Some(ref mut budget) = self.retry_budget {
            if budget.remaining == 0 {
                return false;
            }
            budget.remaining = budget.remaining.saturating_sub(1);
        }
        if let Some(ref shared) = self.shared_budget {
            if !shared.acquire() {
                return false;
            }
        }
        true
    }

    fn record_success(&self, attempt: u32) {
        if attempt > 0 {
            if let Some(ref metrics) = self.metrics {
                metrics.record_retry(true);
            }
        }
    }

    fn record_failure(&self, attempt: u32) {
        if attempt > 0 {
            if let Some(ref metrics) = self.metrics {
                metrics.record_retry(false);
            }
        }
    }

    pub async fn execute<F, T, E, Fut>(&mut self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Retryable + std::error::Error + Send + Sync + 'static,
        E: std::fmt::Debug,
    {
        let _permit = if let Some(ref limiter) = self.limiter {
            Some(limiter.acquire().await)
        } else {
            None
        };

        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => {
                    self.record_success(attempt);
                    return Ok(result);
                }
                Err(e) => {
                    let error_ref = &e;
                    let is_recoverable = error_ref.is_recoverable();

                    if !is_recoverable || !self.can_retry() || attempt >= self.config.max_retries {
                        return Err(e);
                    }

                    attempt += 1;
                    let delay = self.calculate_delay(attempt);

                    self.record_failure(attempt);

                    warn!(
                        "Retry attempt {}/{} failed: {:?}, retrying in {:?}",
                        attempt, self.config.max_retries, error_ref, delay
                    );

                    sleep(delay).await;
                }
            }
        }
    }

    pub fn execute_sync<F, T, E>(mut self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Result<T, E>,
        E: Retryable + std::error::Error + Send + Sync + 'static,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;

        loop {
            match operation() {
                Ok(result) => {
                    self.record_success(attempt);
                    return Ok(result);
                }
                Err(e) => {
                    let is_recoverable = e.is_recoverable();

                    if !is_recoverable || !self.can_retry() || attempt >= self.config.max_retries {
                        return Err(e);
                    }

                    attempt += 1;
                    let delay = self.calculate_delay(attempt);

                    self.record_failure(attempt);

                    warn!(
                        "Retry attempt {}/{} failed: {:?}, retrying in {:?}",
                        attempt, self.config.max_retries, e, delay
                    );

                    std::thread::sleep(delay);
                }
            }
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::time::timeout;

    use super::{RetryConfig, RetryPolicy};

    #[derive(Debug)]
    struct TestError(bool);

    impl std::error::Error for TestError {}

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError({})", self.0)
        }
    }

    impl super::Retryable for TestError {
        fn is_recoverable(&self) -> bool {
            self.0
        }
    }

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let call_count = AtomicUsize::new(0);
        let mut policy = RetryPolicy::new();

        let result = policy
            .execute(|| {
                let count = call_count.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count == 0 {
                        Ok("success")
                    } else {
                        Err(TestError(true))
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let call_count = AtomicUsize::new(0);
        let mut policy = RetryPolicy::with_config(
            RetryConfig::new()
                .with_max_retries(3)
                .with_base_delay(Duration::from_millis(10)),
        );

        let result = policy
            .execute(|| {
                let count = call_count.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count < 2 {
                        Err(TestError(true))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_non_recoverable_error() {
        let call_count = AtomicUsize::new(0);
        let mut policy = RetryPolicy::with_config(RetryConfig::new().with_max_retries(3));

        let result = policy
            .execute(|| {
                let count = call_count.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count < 5 {
                        Err(TestError(false))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_max_retries_exceeded() {
        let call_count = AtomicUsize::new(0);
        let mut policy = RetryPolicy::with_config(
            RetryConfig::new()
                .with_max_retries(2)
                .with_base_delay(Duration::from_millis(5)),
        );

        let result = policy
            .execute(|| {
                call_count.fetch_add(1, Ordering::SeqCst);
                async move { Err::<(), _>(TestError(true)) }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_budget() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mut policy =
            RetryPolicy::with_config(RetryConfig::new().with_max_retries(10)).with_retry_budget(2);

        let result = policy
            .execute({
                let cc = call_count.clone();
                move || {
                    cc.fetch_add(1, Ordering::SeqCst);
                    async move { Err::<(), _>(TestError(true)) }
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_sync() {
        let call_count = AtomicUsize::new(0);
        let policy = RetryPolicy::with_config(
            RetryConfig::new()
                .with_max_retries(3)
                .with_base_delay(Duration::from_millis(10)),
        );

        let result = policy.execute_sync(|| {
            let count = call_count.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(TestError(true))
            } else {
                Ok("success")
            }
        });

        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_jitter() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mut policy = RetryPolicy::with_config(
            RetryConfig::new()
                .with_max_retries(3)
                .with_base_delay(Duration::from_millis(100))
                .with_jitter(0.5),
        );

        let start = std::time::Instant::now();
        let result = policy
            .execute({
                let cc = call_count.clone();
                move || {
                    cc.fetch_add(1, Ordering::SeqCst);
                    let cc2 = cc.clone();
                    async move {
                        if cc2.load(Ordering::SeqCst) < 3 {
                            Err(TestError(true))
                        } else {
                            Ok("success")
                        }
                    }
                }
            })
            .await;

        let elapsed = start.elapsed();
        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
        assert!(elapsed >= Duration::from_millis(100));
        assert!(elapsed < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_retry_timeout() {
        let mut policy = RetryPolicy::with_config(
            RetryConfig::new()
                .with_max_retries(10)
                .with_base_delay(Duration::from_secs(10)),
        );

        let result = timeout(
            Duration::from_millis(100),
            policy.execute(|| async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok::<&str, TestError>("success")
            }),
        )
        .await;

        assert!(result.is_err());
    }
}
