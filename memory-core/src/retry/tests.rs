use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::timeout;

use super::{RetryConfig, RetryPolicy};

#[derive(Debug, PartialEq, Eq)]
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

    assert_eq!(result.expect("first attempt success"), "success");
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

    assert_eq!(result.expect("success after retries"), "success");
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

    assert!(matches!(
        result,
        Err(super::RetryError::Operation(TestError(false)))
    ));
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

    assert!(matches!(result, Err(super::RetryError::Operation(_))));
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
}

fn make_failing_closure(
    cc: Arc<AtomicUsize>,
) -> impl Fn() -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<&'static str, TestError>> + Send>,
> {
    move || {
        cc.fetch_add(1, Ordering::SeqCst);
        let cc2 = cc.clone();
        Box::pin(async move {
            if cc2.load(Ordering::SeqCst) < 3 {
                Err(TestError(true))
            } else {
                Ok("success")
            }
        })
    }
}

#[tokio::test]
async fn test_retry_with_budget() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let mut policy =
        RetryPolicy::with_config(RetryConfig::new().with_max_retries(10)).with_retry_budget(2);

    let result = policy
        .execute(make_failing_closure(call_count.clone()))
        .await;

    // Budget of 2 allows initial + 2 retries = 3 attempts
    // Closure fails on attempts 1-2, succeeds on attempt 3
    assert_eq!(result.expect("budget allows success"), "success");
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
        .execute(make_failing_closure(call_count.clone()))
        .await;

    let elapsed = start.elapsed();
    assert_eq!(result.expect("jittered success"), "success");
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

#[test]
fn retry_error_display_and_source() {
    use super::RetryError;
    use std::error::Error as StdError;

    let op = RetryError::Operation(TestError(true));
    assert!(op.to_string().contains("TestError"));
    assert!(op.source().is_some());

    let qt: RetryError<TestError> = RetryError::QueueTimeout;
    assert!(qt.to_string().contains("retry_queue_timeout"));
    assert!(qt.source().is_none());
}

#[test]
fn retry_error_into_crate_error_and_from() {
    use super::RetryError;
    use crate::error::Error;

    let qt: RetryError<Error> = RetryError::QueueTimeout;
    assert!(matches!(qt.into_crate_error(), Error::RetryQueueTimeout));

    let op = RetryError::Operation(Error::Storage("x".into()));
    assert!(matches!(Error::from(op), Error::Storage(_)));

    let qt2: RetryError<Error> = RetryError::QueueTimeout;
    assert!(matches!(Error::from(qt2), Error::RetryQueueTimeout));
    assert!(!Error::RetryQueueTimeout.is_recoverable());
}

#[test]
fn queue_timeout_display() {
    use super::QueueTimeout;
    let msg = QueueTimeout.to_string();
    assert!(msg.contains("retry_queue_timeout") || msg.contains("permit"));
}

#[tokio::test]
async fn first_attempt_does_not_consume_retry_permit() {
    use super::ConcurrencyLimiter;
    use std::sync::Arc;

    let limiter = ConcurrencyLimiter::new(1);
    // Saturate the only permit — first attempts must still proceed (S1.6).
    let held = limiter.acquire().await;

    let call_count = AtomicUsize::new(0);
    let mut policy = RetryPolicy::with_config(
        RetryConfig::new()
            .with_max_retries(0)
            .with_base_delay(Duration::from_millis(1)),
    )
    .with_limiter(Arc::clone(&limiter));

    let result = policy
        .execute(|| {
            call_count.fetch_add(1, Ordering::SeqCst);
            async move { Ok::<&str, TestError>("ok") }
        })
        .await;

    assert_eq!(result.expect("first attempt free"), "ok");
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
    drop(held);
}

#[tokio::test]
async fn retry_queue_timeout_when_permits_saturated() {
    use super::{ConcurrencyLimiter, RetryError};
    use std::sync::Arc;

    let limiter = ConcurrencyLimiter::new(1);
    let _held = limiter.acquire().await;

    let call_count = AtomicUsize::new(0);
    let mut policy = RetryPolicy::with_config(
        RetryConfig::new()
            .with_max_retries(2)
            .with_base_delay(Duration::from_millis(1))
            .with_retry_queue_timeout(Duration::from_millis(30)),
    )
    .with_limiter(Arc::clone(&limiter));

    let result = policy
        .execute(|| {
            let n = call_count.fetch_add(1, Ordering::SeqCst);
            async move {
                // First attempt fails so a retry (which needs a permit) is scheduled.
                if n == 0 {
                    Err(TestError(true))
                } else {
                    Ok("should-not-reach")
                }
            }
        })
        .await;

    assert!(matches!(result, Err(RetryError::QueueTimeout)));
    // Only the free first attempt should have run.
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
#[should_panic(expected = "max_concurrent_retries must be greater than zero")]
async fn rejects_zero_max_concurrent_retries_in_config() {
    let _ = RetryConfig::new().with_max_concurrent_retries(0);
}

fn fail_then_ok(
    calls: Arc<AtomicUsize>,
) -> impl Fn() -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<&'static str, TestError>> + Send>,
> {
    move || {
        let n = calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            if n == 0 {
                Err(TestError(true))
            } else {
                Ok("a")
            }
        })
    }
}

#[tokio::test]
async fn permits_released_during_backoff_allow_other_first_attempts() {
    use super::ConcurrencyLimiter;
    use std::sync::Arc;

    let limiter = ConcurrencyLimiter::new(1);

    // Task A: fails first attempt (no permit), then sleeps during backoff without a permit.
    let limiter_a = Arc::clone(&limiter);
    let a = tokio::spawn(async move {
        let mut policy = RetryPolicy::with_config(
            RetryConfig::new()
                .with_max_retries(1)
                .with_base_delay(Duration::from_millis(100)),
        )
        .with_limiter(limiter_a);

        let calls = Arc::new(AtomicUsize::new(0));
        let result = policy.execute(fail_then_ok(Arc::clone(&calls))).await;
        (result, calls.load(Ordering::SeqCst))
    });

    // While A is in backoff (permit free), B's first attempt succeeds immediately.
    tokio::time::sleep(Duration::from_millis(20)).await;
    let mut policy_b = RetryPolicy::with_config(
        RetryConfig::new()
            .with_max_retries(0)
            .with_base_delay(Duration::from_millis(1)),
    )
    .with_limiter(Arc::clone(&limiter));

    let b_result = policy_b
        .execute(|| async move { Ok::<&str, TestError>("b") })
        .await;
    assert_eq!(b_result.expect("B first attempt free"), "b");

    let (a_result, a_calls) = a.await.unwrap();
    assert_eq!(a_result.expect("A retry succeeds"), "a");
    assert_eq!(a_calls, 2);
}
