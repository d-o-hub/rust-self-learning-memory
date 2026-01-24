#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::time::timeout;

    use crate::retry::{RetryConfig, RetryMetrics, RetryPolicy};

    #[derive(Debug)]
    struct TestError(bool);

    impl std::error::Error for TestError {}

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError({})", self.0)
        }
    }

    impl crate::retry::Retryable for TestError {
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
        let mut policy = RetryPolicy::new().with_config(
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
        let mut policy = RetryPolicy::new().with_config(RetryConfig::new().with_max_retries(3));

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
        let mut policy = RetryPolicy::new().with_config(
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
    async fn test_retry_with_metrics() {
        let metrics = RetryMetrics::new();
        let call_count = AtomicUsize::new(0);
        let mut policy = RetryPolicy::new()
            .with_config(
                RetryConfig::new()
                    .with_max_retries(3)
                    .with_base_delay(Duration::from_millis(5)),
            )
            .with_metrics(metrics.clone());

        let _ = policy
            .execute(|| {
                call_count.fetch_add(1, Ordering::SeqCst);
                async move {
                    if call_count.load(Ordering::SeqCst) < 3 {
                        Err(TestError(true))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert_eq!(metrics.total(), 2);
        assert_eq!(metrics.success_count(), 1);
        assert_eq!(metrics.failure_count(), 1);
    }

    #[tokio::test]
    async fn test_retry_with_budget() {
        let call_count = AtomicUsize::new(0);
        let mut policy = RetryPolicy::new()
            .with_config(RetryConfig::new().with_max_retries(10))
            .with_retry_budget(2);

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
    async fn test_retry_sync() {
        let call_count = AtomicUsize::new(0);
        let policy = RetryPolicy::new().with_config(
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
        let call_count = AtomicUsize::new(0);
        let mut policy = RetryPolicy::new().with_config(
            RetryConfig::new()
                .with_max_retries(3)
                .with_base_delay(Duration::from_millis(100))
                .with_jitter(0.5),
        );

        let start = std::time::Instant::now();
        let result = policy
            .execute(|| {
                call_count.fetch_add(1, Ordering::SeqCst);
                async move {
                    if call_count.load(Ordering::SeqCst) < 3 {
                        Err(TestError(true))
                    } else {
                        Ok("success")
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
        let mut policy = RetryPolicy::new().with_config(
            RetryConfig::new()
                .with_max_retries(10)
                .with_base_delay(Duration::from_secs(10)),
        );

        let result = timeout(
            Duration::from_millis(100),
            policy.execute(|| async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok("success")
            }),
        )
        .await;

        assert!(result.is_err());
    }
}
