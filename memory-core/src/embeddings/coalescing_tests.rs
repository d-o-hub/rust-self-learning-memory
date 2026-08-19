//! Tests for coalesced embedding provider

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use anyhow::Result;
    use async_trait::async_trait;

    use crate::embeddings::coalescing::CoalescedEmbeddingProvider;
    use crate::embeddings::config::BatchingConfig;
    use crate::embeddings::provider::EmbeddingProvider;

    struct TrackingProvider {
        call_count: Arc<AtomicUsize>,
        batch_call_count: Arc<AtomicUsize>,
        delay: Duration,
        should_fail: bool,
    }

    impl TrackingProvider {
        fn new(delay: Duration, should_fail: bool) -> (Self, Arc<AtomicUsize>, Arc<AtomicUsize>) {
            let call_count = Arc::new(AtomicUsize::new(0));
            let batch_call_count = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    call_count: call_count.clone(),
                    batch_call_count: batch_call_count.clone(),
                    delay,
                    should_fail,
                },
                call_count,
                batch_call_count,
            )
        }
    }

    #[async_trait]
    impl EmbeddingProvider for TrackingProvider {
        async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if self.delay > Duration::ZERO {
                tokio::time::sleep(self.delay).await;
            }
            if self.should_fail {
                anyhow::bail!("Provider simulated error");
            }
            let val = text.len() as f32;
            Ok(vec![val, val])
        }

        async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            self.batch_call_count.fetch_add(1, Ordering::SeqCst);
            if self.delay > Duration::ZERO {
                tokio::time::sleep(self.delay).await;
            }
            if self.should_fail {
                anyhow::bail!("Provider simulated batch error");
            }
            let mut results = Vec::new();
            for text in texts {
                let val = text.len() as f32;
                results.push(vec![val, val]);
            }
            Ok(results)
        }

        fn embedding_dimension(&self) -> usize {
            2
        }

        fn model_name(&self) -> &'static str {
            "tracking-model"
        }
    }

    #[tokio::test]
    async fn test_request_coalescing_and_batching() {
        let (inner, _call_cnt, batch_cnt) =
            TrackingProvider::new(Duration::from_millis(50), false);
        let config = BatchingConfig {
            enabled: true,
            max_batch_size: 10,
            max_wait_ms: 20,
            max_in_flight: 4,
            coalesce_in_flight: true,
        };

        let provider = Arc::new(CoalescedEmbeddingProvider::new(
            Arc::new(inner),
            config,
            true,
        ));

        // 5 concurrent identical requests, 5 distinct requests
        let mut handles = Vec::new();
        for _ in 0..5 {
            let p = provider.clone();
            handles.push(tokio::spawn(async move { p.embed_text("same_text").await }));
        }
        for i in 0..5 {
            let p = provider.clone();
            let text = format!("text_{i}");
            handles.push(tokio::spawn(async move { p.embed_text(&text).await }));
        }

        let results = futures::future::join_all(handles).await;
        for res in results {
            assert!(res.unwrap().is_ok());
        }

        // Only 1 batch call should have happened for all requests
        assert_eq!(batch_cnt.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_cache_hits_bypass_provider() {
        let (inner, _call_cnt, batch_cnt) =
            TrackingProvider::new(Duration::from_millis(10), false);
        let config = BatchingConfig {
            enabled: true,
            max_batch_size: 10,
            max_wait_ms: 10,
            max_in_flight: 4,
            coalesce_in_flight: true,
        };

        let provider = Arc::new(CoalescedEmbeddingProvider::new(
            Arc::new(inner),
            config,
            true,
        ));

        // Initial request
        let res1 = provider.embed_text("cached_query").await;
        assert!(res1.is_ok());
        assert_eq!(batch_cnt.load(Ordering::SeqCst), 1);

        // Subsequent identical request should hit cache
        let res2 = provider.embed_text("cached_query").await;
        assert!(res2.is_ok());
        assert_eq!(batch_cnt.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_error_fanout_no_cache_poisoning() {
        let (inner, _call_cnt, batch_cnt) =
            TrackingProvider::new(Duration::from_millis(10), true);
        let config = BatchingConfig {
            enabled: true,
            max_batch_size: 10,
            max_wait_ms: 10,
            max_in_flight: 4,
            coalesce_in_flight: true,
        };

        let provider = Arc::new(CoalescedEmbeddingProvider::new(
            Arc::new(inner),
            config,
            true,
        ));

        let res1 = provider.embed_text("error_query").await;
        assert!(res1.is_err());
        assert_eq!(batch_cnt.load(Ordering::SeqCst), 1);

        // Next request should try again and not return cached error
        let res2 = provider.embed_text("error_query").await;
        assert!(res2.is_err());
        assert_eq!(batch_cnt.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_cancellation_safety() {
        let (inner, _call_cnt, batch_cnt) =
            TrackingProvider::new(Duration::from_millis(100), false);
        let config = BatchingConfig {
            enabled: true,
            max_batch_size: 10,
            max_wait_ms: 20,
            max_in_flight: 4,
            coalesce_in_flight: true,
        };

        let provider = Arc::new(CoalescedEmbeddingProvider::new(
            Arc::new(inner),
            config,
            true,
        ));

        let p1 = provider.clone();
        let p2 = provider.clone();

        let h1 = tokio::spawn(async move {
            let _ = tokio::time::timeout(Duration::from_millis(30), p1.embed_text("slow_text"))
                .await;
        });

        let h2 = tokio::spawn(async move { p2.embed_text("slow_text").await });

        h1.await.unwrap();
        let res2 = h2.await.unwrap();

        assert!(res2.is_ok());
        assert_eq!(batch_cnt.load(Ordering::SeqCst), 1);
    }
}
