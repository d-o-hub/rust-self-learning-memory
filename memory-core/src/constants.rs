//! Global constants for memory-core
//!
//! This module centralizes all magic numbers and string constants used throughout
//! the memory system, making them easier to maintain and configure.

/// Default configuration values
pub mod defaults {
    use std::time::Duration;

    // Cache and storage
    pub const DEFAULT_CACHE_SIZE: usize = 1000;
    pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 3600; // 1 hour
    pub const DEFAULT_POOL_SIZE: usize = 10;
    pub const MAX_EPISODES_CACHE: usize = 10000;

    // Batch processing
    pub const DEFAULT_BATCH_SIZE: usize = 100;
    pub const MAX_BATCH_SIZE: usize = 1000;
    pub const MIN_BATCH_SIZE: usize = 10;

    // Performance tuning
    pub const DEFAULT_BUFFER_SIZE: usize = 1024;
    pub const DEFAULT_QUEUE_SIZE: usize = 100;
    pub const MAX_CONCURRENT_OPERATIONS: usize = 100;

    // Timeouts
    pub const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

    // Retry configuration
    pub const DEFAULT_MAX_RETRIES: usize = 3;
    pub const DEFAULT_RETRY_DELAY_MS: u64 = 100;
    pub const DEFAULT_RETRY_BACKOFF_MULTIPLIER: f64 = 2.0;

    // Pattern extraction
    pub const MIN_PATTERN_OCCURRENCES: usize = 2;
    pub const MIN_PATTERN_SUCCESS_RATE: f32 = 0.6;
    pub const MAX_PATTERNS_PER_EPISODE: usize = 10;

    // Embedding dimensions
    pub const EMBEDDING_DIMENSION_384: usize = 384;
    pub const EMBEDDING_DIMENSION_768: usize = 768;
    pub const EMBEDDING_DIMENSION_1536: usize = 1536;

    // Similarity thresholds
    pub const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.7;
    pub const MIN_SIMILARITY_THRESHOLD: f32 = 0.5;
    pub const MAX_SIMILARITY_THRESHOLD: f32 = 1.0;

    // Monitoring and metrics
    pub const DEFAULT_HEALTH_CHECK_INTERVAL_SECONDS: u64 = 30;
    pub const DEFAULT_METRICS_COLLECTION_INTERVAL_SECONDS: u64 = 60;

    // Memory management
    pub const DEFAULT_MAX_MEMORY_MB: usize = 512;
    pub const WARNING_MEMORY_THRESHOLD: f32 = 0.8; // 80%
}

/// Error messages
pub mod errors {
    pub const EPISODE_NOT_FOUND: &str = "Episode not found";
    pub const PATTERN_NOT_FOUND: &str = "Pattern not found";
    pub const HEURISTIC_NOT_FOUND: &str = "Heuristic not found";
    pub const INVALID_EPISODE_ID: &str = "Invalid episode ID format";
    pub const INVALID_PATTERN_ID: &str = "Invalid pattern ID format";
    pub const STORAGE_CONNECTION_FAILED: &str = "Failed to connect to storage backend";
    pub const SERIALIZATION_FAILED: &str = "Failed to serialize data";
    pub const DESERIALIZATION_FAILED: &str = "Failed to deserialize data";
    pub const EMBEDDING_GENERATION_FAILED: &str = "Failed to generate embedding";
    pub const CACHE_OPERATION_FAILED: &str = "Cache operation failed";
}

/// Log messages and prefixes
pub mod logging {
    pub const LOG_PREFIX_EPISODE: &str = "[EPISODE]";
    pub const LOG_PREFIX_PATTERN: &str = "[PATTERN]";
    pub const LOG_PREFIX_STORAGE: &str = "[STORAGE]";
    pub const LOG_PREFIX_CACHE: &str = "[CACHE]";
    pub const LOG_PREFIX_EMBEDDING: &str = "[EMBEDDING]";
    pub const LOG_PREFIX_MONITOR: &str = "[MONITOR]";
}

/// File paths and extensions
pub mod paths {
    pub const DEFAULT_DATA_DIR: &str = "./data";
    pub const DEFAULT_BACKUP_DIR: &str = "./backups";
    pub const DEFAULT_LOG_DIR: &str = "./logs";
    pub const DEFAULT_CACHE_DIR: &str = "./cache";

    pub const DB_FILE_EXTENSION: &str = ".db";
    pub const REDB_FILE_EXTENSION: &str = ".redb";
    pub const LOG_FILE_EXTENSION: &str = ".log";
    pub const BACKUP_FILE_EXTENSION: &str = ".backup";
}

/// Database table and column names
pub mod db {
    // Table names
    pub const TABLE_EPISODES: &str = "episodes";
    pub const TABLE_PATTERNS: &str = "patterns";
    pub const TABLE_HEURISTICS: &str = "heuristics";
    pub const TABLE_EMBEDDINGS: &str = "embeddings";
    pub const TABLE_SUMMARIES: &str = "episode_summaries";
    pub const TABLE_METRICS: &str = "agent_metrics";
    pub const TABLE_EXECUTION_RECORDS: &str = "execution_records";
    pub const TABLE_TASK_METRICS: &str = "task_metrics";

    // Common column names
    pub const COL_ID: &str = "id";
    pub const COL_EPISODE_ID: &str = "episode_id";
    pub const COL_PATTERN_ID: &str = "pattern_id";
    pub const COL_CREATED_AT: &str = "created_at";
    pub const COL_UPDATED_AT: &str = "updated_at";
    pub const COL_DELETED_AT: &str = "deleted_at";
}

/// HTTP and API constants
pub mod api {
    pub const DEFAULT_API_TIMEOUT_SECONDS: u64 = 30;
    pub const DEFAULT_MAX_RETRIES: usize = 3;
    pub const DEFAULT_RATE_LIMIT_PER_MINUTE: usize = 60;

    // User agent
    pub const USER_AGENT: &str = concat!("memory-core/", env!("CARGO_PKG_VERSION"),);
}

/// Feature flags (for conditional compilation)
pub mod features {
    #[cfg(feature = "openai-embeddings")]
    pub const OPENAI_ENABLED: bool = true;
    #[cfg(not(feature = "openai-embeddings"))]
    pub const OPENAI_ENABLED: bool = false;

    #[cfg(feature = "cohere-embeddings")]
    pub const COHERE_ENABLED: bool = true;
    #[cfg(not(feature = "cohere-embeddings"))]
    pub const COHERE_ENABLED: bool = false;

    #[cfg(feature = "local-embeddings")]
    pub const LOCAL_EMBEDDINGS_ENABLED: bool = true;
    #[cfg(not(feature = "local-embeddings"))]
    pub const LOCAL_EMBEDDINGS_ENABLED: bool = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert!(defaults::DEFAULT_CACHE_SIZE > 0);
        assert!(defaults::DEFAULT_BATCH_SIZE > 0);
        assert!(defaults::MAX_BATCH_SIZE >= defaults::DEFAULT_BATCH_SIZE);
        assert!(defaults::MIN_BATCH_SIZE <= defaults::DEFAULT_BATCH_SIZE);
    }

    #[test]
    fn test_similarity_thresholds() {
        assert!(defaults::MIN_SIMILARITY_THRESHOLD >= 0.0);
        assert!(defaults::MAX_SIMILARITY_THRESHOLD <= 1.0);
        assert!(defaults::DEFAULT_SIMILARITY_THRESHOLD >= defaults::MIN_SIMILARITY_THRESHOLD);
        assert!(defaults::DEFAULT_SIMILARITY_THRESHOLD <= defaults::MAX_SIMILARITY_THRESHOLD);
    }

    #[test]
    fn test_error_messages_not_empty() {
        assert!(!errors::EPISODE_NOT_FOUND.is_empty());
        assert!(!errors::PATTERN_NOT_FOUND.is_empty());
        assert!(!errors::STORAGE_CONNECTION_FAILED.is_empty());
    }
}
