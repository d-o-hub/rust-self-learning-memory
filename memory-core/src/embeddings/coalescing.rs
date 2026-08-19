//! In-flight embedding request coalescing, batching, and atomic caching.

use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use lru::LruCache;
use parking_lot::Mutex;
use tokio::sync::{Semaphore, mpsc, watch};

use super::config::BatchingConfig;
use super::provider::{EmbeddingHealth, EmbeddingProvider};

const DEFAULT_CACHE_CAPACITY: usize = 10000;

#[derive(Debug)]
struct BatchItem {
    in_flight_key: String,
    cache_key: String,
    text: String,
}

struct InFlightGuard {
    keys: Vec<String>,
    in_flight: Arc<Mutex<HashMap<String, watch::Sender<Option<Result<Vec<f32>, String>>>>>>,
    completed: bool,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        if !self.completed {
            let mut map = self.in_flight.lock();
            for key in &self.keys {
                if let Some(tx) = map.remove(key) {
                    let _ = tx.send(Some(Err("Batch execution failed unexpectedly".to_string())));
                }
            }
        }
    }
}

/// Embedding provider wrapper that coalesces concurrent requests,
/// batches requests within time/size bounds, and atomically caches results.
pub struct CoalescedEmbeddingProvider {
    inner: Arc<dyn EmbeddingProvider>,
    config: BatchingConfig,
    cache_enabled: bool,
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    in_flight: Arc<Mutex<HashMap<String, watch::Sender<Option<Result<Vec<f32>, String>>>>>>,
    batch_tx: mpsc::Sender<BatchItem>,
}

impl CoalescedEmbeddingProvider {
    /// Create a new coalesced embedding provider wrapping an inner provider.
    #[must_use]
    pub fn new(
        inner: Arc<dyn EmbeddingProvider>,
        config: BatchingConfig,
        cache_enabled: bool,
    ) -> Self {
        let cache_capacity =
            NonZeroUsize::new(DEFAULT_CACHE_CAPACITY).expect("DEFAULT_CACHE_CAPACITY is non-zero");
        let cache = Arc::new(Mutex::new(LruCache::new(cache_capacity)));
        let in_flight = Arc::new(Mutex::new(HashMap::new()));

        let (batch_tx, batch_rx) = mpsc::channel(config.max_batch_size.max(1) * 16);

        if config.enabled {
            let inner_clone = inner.clone();
            let cache_clone = cache.clone();
            let in_flight_clone = in_flight.clone();
            let config_clone = config.clone();
            let cache_enabled_flag = cache_enabled;

            tokio::spawn(async move {
                Self::run_batch_worker(
                    inner_clone,
                    cache_clone,
                    in_flight_clone,
                    config_clone,
                    cache_enabled_flag,
                    batch_rx,
                )
                .await;
            });
        }

        Self {
            inner,
            config,
            cache_enabled,
            cache,
            in_flight,
            batch_tx,
        }
    }

    fn cache_key(&self, text: &str) -> String {
        format!("{}:{}", self.inner.model_name(), text.trim())
    }

    async fn run_batch_worker(
        inner: Arc<dyn EmbeddingProvider>,
        cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
        in_flight: Arc<Mutex<HashMap<String, watch::Sender<Option<Result<Vec<f32>, String>>>>>>,
        config: BatchingConfig,
        cache_enabled: bool,
        mut rx: mpsc::Receiver<BatchItem>,
    ) {
        let semaphore = Arc::new(Semaphore::new(config.max_in_flight.max(1)));
        let max_batch_size = config.max_batch_size.max(1);
        let max_wait = Duration::from_millis(config.max_wait_ms);

        while let Some(first) = rx.recv().await {
            let mut batch = vec![first];
            let deadline = tokio::time::Instant::now() + max_wait;

            while batch.len() < max_batch_size {
                let now = tokio::time::Instant::now();
                if now >= deadline {
                    break;
                }
                let timeout = deadline - now;
                match tokio::time::timeout(timeout, rx.recv()).await {
                    Ok(Some(item)) => {
                        batch.push(item);
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }

            let Ok(permit) = semaphore.clone().acquire_owned().await else {
                break;
            };

            let inner = inner.clone();
            let cache = cache.clone();
            let in_flight = in_flight.clone();

            tokio::spawn(async move {
                let _permit = permit;
                Self::execute_batch(inner, cache, in_flight, batch, cache_enabled).await;
            });
        }
    }

    async fn execute_batch(
        inner: Arc<dyn EmbeddingProvider>,
        cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
        in_flight: Arc<Mutex<HashMap<String, watch::Sender<Option<Result<Vec<f32>, String>>>>>>,
        batch: Vec<BatchItem>,
        cache_enabled: bool,
    ) {
        let mut unique_texts = Vec::new();
        let mut unique_keys = Vec::new();
        let mut seen = HashSet::new();

        for item in &batch {
            if seen.insert(item.in_flight_key.clone()) {
                unique_keys.push(item.in_flight_key.clone());
                unique_texts.push(item.text.clone());
            }
        }

        let mut cache_keys = Vec::new();
        let mut seen_cache = HashSet::new();

        for item in &batch {
            if seen_cache.insert(item.cache_key.clone()) {
                cache_keys.push(item.cache_key.clone());
            }
        }

        let mut guard = InFlightGuard {
            keys: unique_keys.clone(),
            in_flight: in_flight.clone(),
            completed: false,
        };

        let result = inner.embed_batch(&unique_texts).await;

        let result_to_fanout = match result {
            Ok(embeddings) if embeddings.len() == unique_keys.len() => {
                if cache_enabled {
                    let mut c = cache.lock();
                    for (key, vec) in cache_keys.iter().zip(embeddings.iter()) {
                        c.put(key.clone(), vec.clone());
                    }
                }
                Ok(embeddings)
            }
            Ok(embeddings) => Err(format!(
                "Provider embed_batch returned {} embeddings for {} texts",
                embeddings.len(),
                unique_keys.len()
            )),
            Err(err) => Err(err.to_string()),
        };

        Self::fanout_batch_result(in_flight, unique_keys, result_to_fanout);

        guard.completed = true;
    }

    fn fanout_batch_result(
        in_flight: Arc<Mutex<HashMap<String, watch::Sender<Option<Result<Vec<f32>, String>>>>>>,
        unique_keys: Vec<String>,
        result: Result<Vec<Vec<f32>>, String>,
    ) {
        let mut map = in_flight.lock();
        match result {
            Ok(embeddings) => {
                for (key, vec) in unique_keys.into_iter().zip(embeddings) {
                    if let Some(tx) = map.remove(&key) {
                        let _ = tx.send(Some(Ok(vec)));
                    }
                }
            }
            Err(err_msg) => {
                for key in unique_keys {
                    if let Some(tx) = map.remove(&key) {
                        let _ = tx.send(Some(Err(err_msg.clone())));
                    }
                }
            }
        }
    }
}

#[async_trait]
impl EmbeddingProvider for CoalescedEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if !self.config.enabled {
            if self.cache_enabled {
                let key = self.cache_key(text);
                if let Some(vec) = self.cache.lock().get(&key) {
                    return Ok(vec.clone());
                }
                let vec = self.inner.embed_text(text).await?;
                self.cache.lock().put(key, vec.clone());
                return Ok(vec);
            }
            return self.inner.embed_text(text).await;
        }

        let cache_key = self.cache_key(text);

        if self.cache_enabled {
            if let Some(vec) = self.cache.lock().get(&cache_key) {
                return Ok(vec.clone());
            }
        }

        let in_flight_key = if self.config.coalesce_in_flight {
            cache_key.clone()
        } else {
            format!("{}:{}", cache_key, uuid::Uuid::new_v4())
        };

        let (mut rx, is_first) = {
            let mut map = self.in_flight.lock();
            if let Some(tx) = map.get(&in_flight_key) {
                (tx.subscribe(), false)
            } else {
                let (tx, rx) = watch::channel(None);
                map.insert(in_flight_key.clone(), tx);
                (rx, true)
            }
        };

        if is_first {
            struct SendGuard<'a> {
                key: &'a str,
                map: &'a Arc<
                    Mutex<HashMap<String, watch::Sender<Option<Result<Vec<f32>, String>>>>>,
                >,
                sent: bool,
            }

            impl Drop for SendGuard<'_> {
                fn drop(&mut self) {
                    if !self.sent {
                        self.map.lock().remove(self.key);
                    }
                }
            }

            let mut send_guard = SendGuard {
                key: &in_flight_key,
                map: &self.in_flight,
                sent: false,
            };

            let send_res = self
                .batch_tx
                .send(BatchItem {
                    in_flight_key: in_flight_key.clone(),
                    cache_key: cache_key.clone(),
                    text: text.to_string(),
                })
                .await;

            if send_res.is_err() {
                anyhow::bail!("Embedding batch worker channel closed");
            }
            send_guard.sent = true;
        }

        if let Some(res) = rx.borrow().clone() {
            return res.map_err(|e| anyhow::anyhow!(e));
        }

        if rx.changed().await.is_err() {
            return Err(anyhow::anyhow!("In-flight request cancelled"));
        }

        match rx.borrow().clone() {
            Some(res) => res.map_err(|e| anyhow::anyhow!(e)),
            None => Err(anyhow::anyhow!("In-flight request closed without result")),
        }
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        if !self.config.enabled {
            return self.inner.embed_batch(texts).await;
        }

        let futures: Vec<_> = texts.iter().map(|t| self.embed_text(t)).collect();
        futures::future::try_join_all(futures).await
    }

    fn embedding_dimension(&self) -> usize {
        self.inner.embedding_dimension()
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }

    async fn is_available(&self) -> bool {
        self.inner.is_available().await
    }

    async fn health(&self) -> EmbeddingHealth {
        self.inner.health().await
    }

    async fn warmup(&self) -> Result<()> {
        self.inner.warmup().await
    }

    fn metadata(&self) -> serde_json::Value {
        let mut meta = self.inner.metadata();
        if let serde_json::Value::Object(ref mut obj) = meta {
            obj.insert("coalesced".to_string(), serde_json::json!(true));
            obj.insert(
                "batch_config".to_string(),
                serde_json::to_value(&self.config).unwrap_or_default(),
            );
        }
        meta
    }
}
