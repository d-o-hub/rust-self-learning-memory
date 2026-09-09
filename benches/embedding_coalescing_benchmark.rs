//! Benchmark for coalesced embedding provider batching and request coalescing performance.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use criterion::{Criterion, criterion_group, criterion_main};
use do_memory_core::embeddings::{BatchingConfig, CoalescedEmbeddingProvider, EmbeddingProvider};
use tokio::runtime::Runtime;

struct BenchProvider {
    batch_calls: Arc<AtomicUsize>,
    latency: Duration,
}

#[async_trait]
impl EmbeddingProvider for BenchProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if self.latency > Duration::ZERO {
            tokio::time::sleep(self.latency).await;
        }
        let val = text.len() as f32;
        Ok(vec![val, val])
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.batch_calls.fetch_add(1, Ordering::SeqCst);
        if self.latency > Duration::ZERO {
            tokio::time::sleep(self.latency).await;
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
        "bench-model"
    }
}

fn bench_coalescing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_coalesced_embeddings", |b| {
        b.to_async(&rt).iter(|| async {
            let batch_calls = Arc::new(AtomicUsize::new(0));
            let inner = BenchProvider {
                batch_calls: batch_calls.clone(),
                latency: Duration::from_millis(5),
            };
            let config = BatchingConfig {
                enabled: true,
                max_batch_size: 64,
                max_wait_ms: 5,
                max_in_flight: 8,
                coalesce_in_flight: true,
            };
            let provider = Arc::new(CoalescedEmbeddingProvider::new(
                Arc::new(inner),
                config,
                true,
            ));

            let mut handles = Vec::new();
            // 20 concurrent requests with 5 unique texts
            for i in 0..20 {
                let p = provider.clone();
                let text = format!("query_{}", i % 5);
                handles.push(tokio::spawn(async move {
                    let _ = p.embed_text(&text).await;
                }));
            }

            for h in handles {
                let _ = h.await;
            }
        });
    });
}

criterion_group!(benches, bench_coalescing);
criterion_main!(benches);
