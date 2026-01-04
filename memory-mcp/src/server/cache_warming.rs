// Cache warming configuration and methods
//!
//! This module contains cache warming configuration and methods for pre-loading
//! episodes, patterns, and query patterns to improve initial query performance.

use anyhow::Result;
use memory_core::SelfLearningMemory;
use std::sync::Arc;
use tracing::{debug, info};

/// Configuration for cache warming process
#[derive(Debug, Clone)]
pub struct CacheWarmingConfig {
    /// Number of recent episodes to pre-load
    pub recent_episodes_limit: usize,
    /// Number of patterns to pre-load per domain
    pub patterns_per_domain: usize,
    /// Sample queries to execute for warming
    pub sample_queries: Vec<SampleQuery>,
}

/// Sample query for cache warming
#[derive(Debug, Clone)]
pub struct SampleQuery {
    pub description: String,
    pub domain: String,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub tags: Vec<String>,
}

impl CacheWarmingConfig {
    /// Create cache warming config from environment variables
    pub fn from_env() -> Self {
        Self {
            recent_episodes_limit: std::env::var("MCP_CACHE_WARMING_EPISODES")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
            patterns_per_domain: std::env::var("MCP_CACHE_WARMING_PATTERNS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            sample_queries: Self::default_sample_queries(),
        }
    }

    /// Default sample queries for cache warming
    pub fn default_sample_queries() -> Vec<SampleQuery> {
        vec![
            SampleQuery {
                description: "implement api endpoint".to_string(),
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                framework: Some("axum".to_string()),
                tags: vec!["rest".to_string(), "api".to_string()],
            },
            SampleQuery {
                description: "parse json data".to_string(),
                domain: "data-processing".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                tags: vec!["json".to_string(), "parsing".to_string()],
            },
            SampleQuery {
                description: "write unit tests".to_string(),
                domain: "testing".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                tags: vec!["unit-tests".to_string(), "testing".to_string()],
            },
            SampleQuery {
                description: "debug performance issue".to_string(),
                domain: "debugging".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                tags: vec!["performance".to_string(), "debugging".to_string()],
            },
            SampleQuery {
                description: "refactor code for maintainability".to_string(),
                domain: "refactoring".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                tags: vec!["refactoring".to_string(), "maintainability".to_string()],
            },
        ]
    }
}

/// Warm the cache by pre-loading recent episodes and common query patterns
///
/// This method performs cache warming to improve initial query performance by:
/// 1. Pre-loading recent episodes into cache
/// 2. Pre-computing common query patterns
/// 3. Warming up pattern extraction and retrieval systems
pub async fn warm_cache(
    memory: &Arc<SelfLearningMemory>,
    config: &CacheWarmingConfig,
) -> Result<()> {
    info!("Starting cache warming process");

    let start_time = std::time::Instant::now();

    // Warm episodes cache
    warm_episodes_cache(memory, config).await?;

    // Warm patterns cache
    warm_patterns_cache(memory, config).await?;

    // Warm common query patterns
    warm_query_patterns(memory, config).await?;

    let duration = start_time.elapsed();
    info!("Cache warming completed in {:.2}s", duration.as_secs_f64());

    Ok(())
}

/// Warm the episodes cache by loading recent episodes
async fn warm_episodes_cache(
    memory: &Arc<SelfLearningMemory>,
    config: &CacheWarmingConfig,
) -> Result<()> {
    info!(
        "Warming episodes cache with {} recent episodes",
        config.recent_episodes_limit
    );

    // Create a generic context to retrieve diverse episodes
    let context = memory_core::TaskContext {
        domain: "general".to_string(),
        language: None,
        framework: None,
        complexity: memory_core::ComplexityLevel::Moderate,
        tags: vec![],
    };

    // Retrieve recent episodes using a broad query
    let episodes = memory
        .retrieve_relevant_context(
            "recent tasks".to_string(),
            context,
            config.recent_episodes_limit,
        )
        .await;

    info!("Pre-loaded {} episodes into cache", episodes.len());

    Ok(())
}

/// Warm the patterns cache by loading relevant patterns
async fn warm_patterns_cache(
    memory: &Arc<SelfLearningMemory>,
    config: &CacheWarmingConfig,
) -> Result<()> {
    info!(
        "Warming patterns cache with {} patterns per domain",
        config.patterns_per_domain
    );

    // Warm patterns for common domains
    let common_domains = vec![
        "web-api",
        "data-processing",
        "code-generation",
        "debugging",
        "refactoring",
        "testing",
        "analysis",
        "documentation",
    ];

    for domain in &common_domains {
        let context = memory_core::TaskContext {
            domain: domain.to_string(),
            language: None,
            framework: None,
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: vec![domain.to_string()],
        };

        let patterns = memory
            .retrieve_relevant_patterns(&context, config.patterns_per_domain)
            .await;

        debug!(
            "Pre-loaded {} patterns for domain '{}'",
            patterns.len(),
            domain
        );
    }

    info!("Patterns cache warming completed");
    Ok(())
}

/// Warm common query patterns by executing typical queries
async fn warm_query_patterns(
    memory: &Arc<SelfLearningMemory>,
    config: &CacheWarmingConfig,
) -> Result<()> {
    info!(
        "Warming query patterns with {} sample queries",
        config.sample_queries.len()
    );

    // Execute sample queries to warm up retrieval systems
    for query in &config.sample_queries {
        let context = memory_core::TaskContext {
            domain: query.domain.clone(),
            language: query.language.clone(),
            framework: query.framework.clone(),
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: query.tags.clone(),
        };

        // Query memory (this will populate caches)
        let _episodes = memory
            .retrieve_relevant_context(query.description.clone(), context.clone(), 5)
            .await;

        // Query patterns
        let _patterns = memory.retrieve_relevant_patterns(&context, 5).await;

        debug!("Warmed query pattern: '{}'", query.description);
    }

    info!("Query patterns warming completed");
    Ok(())
}

/// Check if cache warming is enabled via environment variable
pub fn is_cache_warming_enabled() -> bool {
    std::env::var("MCP_CACHE_WARMING_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true"
}
