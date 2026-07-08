// ============================================================================
// Configuration
// ============================================================================

use crate::memory::step_buffer::BatchConfig;
use crate::security::audit::AuditConfig;

/// Configuration for storage backend behavior.
///
/// Controls caching limits, synchronization timing, and optimization features
/// for the storage layer.
///
/// # Examples
///
/// ```
/// use do_memory_core::StorageConfig;
///
/// // Default configuration
/// let config = StorageConfig::default();
///
/// // Custom configuration
/// let custom_config = StorageConfig {
///     max_episodes_cache: 5000,      // Cache up to 5000 episodes
///     sync_interval_secs: 60,        // Sync every minute
///     enable_compression: true,      // Enable compression for storage
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Maximum number of episodes to keep in cache
    pub max_episodes_cache: usize,
    /// Interval in seconds between cache-to-durable syncs
    pub sync_interval_secs: u64,
    /// Whether to compress data when storing
    pub enable_compression: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_episodes_cache: 1000,
            sync_interval_secs: 300, // 5 minutes
            enable_compression: true,
        }
    }
}

/// Configuration for concurrency control to prevent cache contention.
///
/// Limits concurrent cache operations to prevent blocking the async runtime
/// when many operations occur simultaneously (e.g., from MCP server).
///
/// # Examples
///
/// ```
/// use do_memory_core::ConcurrencyConfig;
///
/// // Default configuration (moderate concurrency)
/// let config = ConcurrencyConfig::default();
///
/// // High concurrency for busy systems
/// let high_concurrency = ConcurrencyConfig {
///     max_concurrent_cache_ops: 20,
/// };
///
/// // Low concurrency for resource-constrained environments
/// let low_concurrency = ConcurrencyConfig {
///     max_concurrent_cache_ops: 5,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    /// Maximum number of concurrent cache operations allowed.
    ///
    /// Limits how many redb operations can run simultaneously to prevent
    /// overwhelming the async runtime with blocking tasks. Default is 10.
    pub max_concurrent_cache_ops: usize,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_cache_ops: 10,
        }
    }
}

/// Main configuration for the self-learning memory system.
///
/// Controls all aspects of memory behavior including storage, pattern
/// extraction, concurrency control, and optional features like embeddings.
///
/// # Examples
///
/// ```
/// use do_memory_core::{MemoryConfig, StorageConfig, BatchConfig, ConcurrencyConfig, EvictionPolicy, EventEmitterMode};
/// use do_memory_core::types::RetrievalMode;
/// use do_memory_core::security::audit::AuditConfig;
///
/// // Default configuration
/// let config = MemoryConfig::default();
///
/// // Custom configuration with embeddings and capacity management
/// let custom_config = MemoryConfig {
///     batch_config: None,
///     storage: StorageConfig::default(),
///     enable_embeddings: true,
///     pattern_extraction_threshold: 0.8,  // More selective pattern extraction
///     concurrency: ConcurrencyConfig {
///         max_concurrent_cache_ops: 15,  // Allow more concurrent cache ops
///     },
///     max_episodes: Some(10000),  // Limit to 10k episodes
///     eviction_policy: Some(EvictionPolicy::RelevanceWeighted),
///     enable_summarization: true,
///     summary_min_length: 100,
///     summary_max_length: 200,
///     quality_threshold: 0.7,
///     enable_diversity_maximization: true,
///     diversity_lambda: 0.7,
///     enable_spatiotemporal_indexing: true,
///     temporal_bias_weight: 0.3,
///     max_clusters_to_search: 5,
///     retrieval_mode: RetrievalMode::Keyword,
///     semantic_search_mode: "hybrid".to_string(),
///     enable_query_embedding_cache: true,
///     semantic_similarity_threshold: 0.6,
///     semantic_weight: 0.5,
///     recency_weight: 0.25,
///     reward_weight: 0.15,
///     context_overlap_weight: 0.10,
///     ann_index_path: None,
///     audit_config: AuditConfig::default(),
///     event_emitter_mode: EventEmitterMode::NoOp,
/// };
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct MemoryConfig {
    // Core behavior
    /// Storage backend configuration
    pub storage: StorageConfig,
    /// Whether to enable semantic embeddings
    pub enable_embeddings: bool,
    /// Minimum similarity score to extract a pattern (0.0-1.0)
    pub pattern_extraction_threshold: f32,
    /// Minimum quality score for an episode to be used for learning (0.0-1.0)
    pub quality_threshold: f32,
    /// Batch processing configuration for background tasks
    pub batch_config: Option<BatchConfig>,
    /// Concurrency control configuration
    pub concurrency: ConcurrencyConfig,

    // Phase 2 (GENESIS) - Capacity management
    /// Maximum number of episodes to store (None = unlimited)
    pub max_episodes: Option<usize>,
    /// Eviction policy when capacity is reached (None = no eviction)
    pub eviction_policy: Option<crate::episode::EvictionPolicy>,

    // Phase 2 (GENESIS) - Semantic summarization
    /// Whether to generate semantic summaries for episodes
    pub enable_summarization: bool,
    /// Minimum summary length in words
    pub summary_min_length: usize,
    /// Maximum summary length in words
    pub summary_max_length: usize,

    // Phase 3 (Spatiotemporal Memory Organization)
    /// Enable spatiotemporal hierarchical indexing (default: true)
    pub enable_spatiotemporal_indexing: bool,
    /// Enable diversity maximization via MMR (default: true)
    pub enable_diversity_maximization: bool,
    /// Lambda parameter for MMR diversity (0.0-1.0, default: 0.7)
    /// 1.0 = pure relevance, 0.0 = pure diversity
    pub diversity_lambda: f32,
    /// Temporal bias weight in retrieval scoring (default: 0.3)
    /// Higher values favor more recent episodes
    pub temporal_bias_weight: f32,
    /// Maximum temporal clusters to search (default: 5)
    pub max_clusters_to_search: usize,

    // Phase 3 (Enhanced) - Semantic Search Configuration
    /// Retrieval mode for episodic memory (keyword, semantic, hybrid)
    pub retrieval_mode: crate::types::RetrievalMode,
    /// Semantic search mode: "hybrid" (default), "semantic-only", or "keyword-only"
    /// - hybrid: Combine semantic embeddings with temporal/domain filtering
    /// - semantic-only: Use only semantic similarity for ranking
    /// - keyword-only: Use traditional keyword matching only
    pub semantic_search_mode: String,
    /// Enable query embedding caching (default: true)
    /// Caches query embeddings to avoid regenerating for identical queries
    pub enable_query_embedding_cache: bool,
    /// Similarity threshold for semantic search (0.0-1.0, default: 0.6)
    /// Episodes with similarity below this threshold are filtered out
    pub semantic_similarity_threshold: f32,

    // Hybrid Ranking Weights
    /// Weight for semantic similarity (default: 0.5)
    pub semantic_weight: f32,
    /// Weight for recency (default: 0.25)
    pub recency_weight: f32,
    /// Weight for reward score (default: 0.15)
    pub reward_weight: f32,
    /// Weight for context overlap (default: 0.10)
    pub context_overlap_weight: f32,
    /// Path to the ANN index snapshot
    pub ann_index_path: Option<std::path::PathBuf>,

    // Security - Audit logging
    /// Audit logging configuration
    pub audit_config: AuditConfig,

    // External eventing (WG-149)
    /// CloudEvents emitter mode (default: NoOp for zero overhead)
    pub event_emitter_mode: crate::types::emitter::EventEmitterMode,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            enable_embeddings: false,
            pattern_extraction_threshold: 0.7,
            quality_threshold: 0.7,
            batch_config: Some(BatchConfig::default()),
            concurrency: ConcurrencyConfig::default(),

            // Phase 2 (GENESIS) - Capacity management defaults
            max_episodes: None, // Unlimited by default
            eviction_policy: Some(crate::episode::EvictionPolicy::RelevanceWeighted),

            // Phase 2 (GENESIS) - Semantic summarization defaults
            enable_summarization: true,
            summary_min_length: 100,
            summary_max_length: 200,

            // Phase 3 (Spatiotemporal) - Defaults
            enable_spatiotemporal_indexing: true,
            enable_diversity_maximization: true,
            diversity_lambda: 0.7,
            temporal_bias_weight: 0.3,
            max_clusters_to_search: 5,

            // Phase 3 (Enhanced) - Semantic search defaults
            retrieval_mode: crate::types::RetrievalMode::Keyword,
            semantic_search_mode: "hybrid".to_string(),
            enable_query_embedding_cache: true,
            semantic_similarity_threshold: 0.6,
            semantic_weight: 0.5,
            recency_weight: 0.25,
            reward_weight: 0.15,
            context_overlap_weight: 0.10,
            ann_index_path: None,

            // Security - Audit logging (disabled by default for development)
            audit_config: AuditConfig::default(),

            // External eventing (WG-149) - NoOp by default
            event_emitter_mode: crate::types::emitter::EventEmitterMode::NoOp,
        }
    }
}

impl MemoryConfig {
    /// Create a `MemoryConfig` from environment variables.
    ///
    /// Reads configuration from environment variables, falling back to defaults
    /// for any missing values.
    ///
    /// # Environment Variables
    ///
    /// ## Phase 2 (GENESIS) - Capacity Management
    /// * `MEMORY_MAX_EPISODES` - Maximum number of episodes to store (default: `None`/unlimited)
    /// * `MEMORY_EVICTION_POLICY` - Eviction policy: `"LRU"` or `"RelevanceWeighted"` (default: `RelevanceWeighted`)
    ///
    /// ## Phase 2 (GENESIS) - Semantic Summarization
    /// * `MEMORY_ENABLE_SUMMARIZATION` - Enable summarization: `"true"` or `"false"` (default: `true`)
    ///
    /// ## Phase 3 (Spatiotemporal) - Hierarchical Retrieval
    /// * `MEMORY_ENABLE_SPATIOTEMPORAL` - Enable spatiotemporal indexing: `"true"` or `"false"` (default: `true`)
    /// * `MEMORY_ENABLE_DIVERSITY` - Enable diversity maximization: `"true"` or `"false"` (default: `true`)
    /// * `MEMORY_DIVERSITY_LAMBDA` - MMR lambda parameter (0.0-1.0, default: `0.7`)
    /// * `MEMORY_TEMPORAL_BIAS` - Temporal bias weight (0.0-1.0, default: `0.3`)
    /// * `MEMORY_MAX_CLUSTERS` - Maximum temporal clusters to search (default: `5`)
    ///
    /// ## External Eventing (WG-149)
    /// * `MEMORY_EVENT_EMITTER` - CloudEvents emitter mode: `"noop"`, `"log"`, or `"http"`
    ///   (default: `noop`; `http` requires the `http-emitter` feature)
    /// * `MEMORY_EVENT_EMITTER_URL` - Webhook URL when using `http` mode
    ///   (default: `http://localhost:8080/events`)
    ///
    /// # Examples
    ///
    /// ```
    /// use do_memory_core::MemoryConfig;
    ///
    /// // With environment variables set:
    /// // MEMORY_MAX_EPISODES=10000
    /// // MEMORY_EVICTION_POLICY=RelevanceWeighted
    /// // MEMORY_ENABLE_SUMMARIZATION=true
    ///
    /// let config = MemoryConfig::from_env();
    /// ```
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Phase 2 (GENESIS) - Capacity management
        if let Ok(max_episodes) = std::env::var("MEMORY_MAX_EPISODES") {
            config.max_episodes = max_episodes.parse().ok();
        }

        if let Ok(policy) = std::env::var("MEMORY_EVICTION_POLICY") {
            config.eviction_policy = match policy.to_lowercase().as_str() {
                "lru" => Some(crate::episode::EvictionPolicy::LRU),
                "relevanceweighted" | "relevance_weighted" | "relevance-weighted" => {
                    Some(crate::episode::EvictionPolicy::RelevanceWeighted)
                }
                _ => {
                    tracing::warn!(
                        "Invalid MEMORY_EVICTION_POLICY '{}', using default RelevanceWeighted",
                        policy
                    );
                    Some(crate::episode::EvictionPolicy::RelevanceWeighted)
                }
            };
        }

        // Phase 2 (GENESIS) - Semantic summarization
        if let Ok(enable_summarization) = std::env::var("MEMORY_ENABLE_SUMMARIZATION") {
            config.enable_summarization = matches!(
                enable_summarization.to_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            );
        }

        // Phase 3 (Spatiotemporal Memory Organization)
        if let Ok(enable_spatiotemporal) = std::env::var("MEMORY_ENABLE_SPATIOTEMPORAL") {
            config.enable_spatiotemporal_indexing = matches!(
                enable_spatiotemporal.to_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            );
        }

        if let Ok(enable_diversity) = std::env::var("MEMORY_ENABLE_DIVERSITY") {
            config.enable_diversity_maximization = matches!(
                enable_diversity.to_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            );
        }

        if let Ok(lambda) = std::env::var("MEMORY_DIVERSITY_LAMBDA") {
            if let Ok(value) = lambda.parse::<f32>() {
                config.diversity_lambda = value.clamp(0.0, 1.0);
            }
        }

        if let Ok(bias) = std::env::var("MEMORY_TEMPORAL_BIAS") {
            if let Ok(value) = bias.parse::<f32>() {
                config.temporal_bias_weight = value.clamp(0.0, 1.0);
            }
        }

        if let Ok(clusters) = std::env::var("MEMORY_MAX_CLUSTERS") {
            if let Ok(value) = clusters.parse::<usize>() {
                config.max_clusters_to_search = value;
            }
        }

        // Phase 3 (Enhanced) - Semantic search configuration
        if let Ok(mode) = std::env::var("MEMORY_SEMANTIC_MODE") {
            config.semantic_search_mode = match mode.to_lowercase().as_str() {
                "semantic-only" | "semantic_only" | "semanticonly" => "semantic-only".to_string(),
                "keyword-only" | "keyword_only" | "keywordonly" => "keyword-only".to_string(),
                "hybrid" => "hybrid".to_string(),
                _ => {
                    tracing::warn!(
                        "Invalid MEMORY_SEMANTIC_MODE '{}', using default 'hybrid'",
                        mode
                    );
                    "hybrid".to_string()
                }
            };
        }

        if let Ok(enable_cache) = std::env::var("MEMORY_QUERY_CACHE") {
            config.enable_query_embedding_cache = matches!(
                enable_cache.to_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            );
        }

        if let Ok(threshold) = std::env::var("MEMORY_SIMILARITY_THRESHOLD") {
            if let Ok(value) = threshold.parse::<f32>() {
                config.semantic_similarity_threshold = value.clamp(0.0, 1.0);
            }
        }

        // Hybrid Retrieval Mode and Weights
        if let Ok(mode) = std::env::var("MEMORY_RETRIEVAL_MODE") {
            if let Ok(retrieval_mode) = mode.parse() {
                config.retrieval_mode = retrieval_mode;
            }
        }

        if let Ok(w) = std::env::var("MEMORY_SEMANTIC_WEIGHT") {
            if let Ok(v) = w.parse() {
                config.semantic_weight = v;
            }
        }
        if let Ok(w) = std::env::var("MEMORY_RECENCY_WEIGHT") {
            if let Ok(v) = w.parse() {
                config.recency_weight = v;
            }
        }
        if let Ok(w) = std::env::var("MEMORY_REWARD_WEIGHT") {
            if let Ok(v) = w.parse() {
                config.reward_weight = v;
            }
        }
        if let Ok(w) = std::env::var("MEMORY_CONTEXT_WEIGHT") {
            if let Ok(v) = w.parse() {
                config.context_overlap_weight = v;
            }
        }

        if let Ok(path) = std::env::var("MEMORY_ANN_INDEX_PATH") {
            config.ann_index_path = Some(std::path::PathBuf::from(path));
        }

        // Security - Audit logging configuration from environment
        config.audit_config = AuditConfig::from_env();

        // External eventing (WG-149) - EventEmitter mode from environment
        if let Ok(mode) = std::env::var("MEMORY_EVENT_EMITTER") {
            config.event_emitter_mode = match mode.to_lowercase().as_str() {
                "log" | "logging" => crate::types::emitter::EventEmitterMode::Log,
                "noop" | "none" => crate::types::emitter::EventEmitterMode::NoOp,
                #[cfg(feature = "http-emitter")]
                "http" | "webhook" => {
                    let url = std::env::var("MEMORY_EVENT_EMITTER_URL").unwrap_or_else(|_| {
                        tracing::warn!(
                            "MEMORY_EVENT_EMITTER_URL not set, using default http://localhost:8080/events"
                        );
                        "http://localhost:8080/events".to_string()
                    });
                    crate::types::emitter::EventEmitterMode::Http { url }
                }
                #[cfg(not(feature = "http-emitter"))]
                "http" | "webhook" => {
                    tracing::warn!(
                        "MEMORY_EVENT_EMITTER=http requires the `http-emitter` feature. Using default NoOp."
                    );
                    crate::types::emitter::EventEmitterMode::NoOp
                }
                _ => {
                    tracing::warn!(
                        "Invalid MEMORY_EVENT_EMITTER '{}', using default NoOp",
                        mode
                    );
                    crate::types::emitter::EventEmitterMode::NoOp
                }
            };
        }

        config
    }
}
