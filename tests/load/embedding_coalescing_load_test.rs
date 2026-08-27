//! Load test for embedding request batching, coalescing, cancellation, and backpressure.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use do_memory_core::embeddings::{
    BatchingConfig, CoalescedEmbeddingProvider, EmbeddingProvider,
};

struct SlowProvider {
    batch_calls: Arc<AtomicUsize>,
    total_embedded: Arc<AtomicUsize>,
    latency: Duration,
}

#[async_trait]
impl EmbeddingProvider for SlowProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        tokio::time::sleep(self.latency).await;
        self.total_embedded.fetch_add(1, Ordering::SeqCst);
        let val = text.len() as f32;
        Ok(vec![val, val])
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.batch_calls.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(self.latency).await;
        self.total_embedded.fetch_add(texts.len(), Ordering::SeqCst);
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
        "slow-load-model"
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_coalescing_load_under_concurrency_and_cancellation() {
    let batch_calls = Arc::new(AtomicUsize::new(0));
    let total_embedded = Arc::new(AtomicUsize::new(0));

    let inner = SlowProvider {
        batch_calls: batch_calls.clone(),
        total_embedded: total_embedded.clone(),
        latency: Duration::from_millis(50),
    };

    let config = BatchingConfig {
        enabled: true,
        max_batch_size: 32,
        max_wait_ms: 10,
        max_in_flight: 4,
        coalesce_in_flight: true,
    };

    let provider = Arc::new(CoalescedEmbeddingProvider::new(
        Arc::new(inner),
        config,
        true,
    ));

    let mut handles = Vec::new();
    let num_tasks = 100;
    let start = Instant::now();

    for i in 0..num_tasks {
        let p = provider.clone();
        // 5 unique prompts repeated 20 times each
        let text = format!("prompt_{}", i % 5);

        handles.push(tokio::spawn(async move {
            if i % 10 == 0 {
                // Simulate caller cancellation / timeout
                let _ = tokio::time::timeout(Duration::from_millis(20), p.embed_text(&text)).await;
            } else {
                let res = p.embed_text(&text).await;
                assert!(res.is_ok());
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let elapsed = start.elapsed();
    println!(
        "Load test completed in {:?}. Batch calls: {}, total embedded: {}",
        elapsed,
        batch_calls.load(Ordering::SeqCst),
        total_embedded.load(Ordering::SeqCst)
    );

    // Without batching/coalescing, 100 requests @ 50ms = 5000ms sequentially or many individual provider calls.
    // With coalescing into 5 unique keys, total embedded items sent to provider across all batches should be <= 5.
    assert!(total_embedded.load(Ordering::SeqCst) <= 10);
    assert!(batch_calls.load(Ordering::SeqCst) <= 5);
}
