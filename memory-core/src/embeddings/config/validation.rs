//! Validation utilities for embedding configuration.

impl OptimizationConfig {
    /// Optimized configuration for `OpenAI`
    #[must_use]
    pub fn openai() -> Self {
        Self {
            timeout_seconds: Some(60),
            max_retries: 3,
            retry_delay_ms: 1000,
            max_batch_size: Some(2048),
            rate_limit_rpm: Some(3000),
            rate_limit_tpm: Some(1_000_000),
            compression_enabled: true,
            compression_threshold_bytes: 1024,
            connection_pool_size: 20,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(super::super::circuit_breaker::CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 2,
                timeout_seconds: 30,
                half_open_max_attempts: 3,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for Mistral AI
    #[must_use]
    pub fn mistral() -> Self {
        Self {
            timeout_seconds: Some(30),
            max_retries: 3,
            retry_delay_ms: 500,
            max_batch_size: Some(128),
            rate_limit_rpm: Some(100),
            rate_limit_tpm: Some(100_000),
            compression_enabled: true,
            compression_threshold_bytes: 512,
            connection_pool_size: 10,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(super::super::circuit_breaker::CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 2,
                timeout_seconds: 20,
                half_open_max_attempts: 2,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for Azure `OpenAI`
    #[must_use]
    pub fn azure() -> Self {
        Self {
            timeout_seconds: Some(90),
            max_retries: 4,
            retry_delay_ms: 2000,
            max_batch_size: Some(2048),
            rate_limit_rpm: Some(300),
            rate_limit_tpm: Some(300_000),
            compression_enabled: true,
            compression_threshold_bytes: 1024,
            connection_pool_size: 15,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(super::super::circuit_breaker::CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout_seconds: 60,
                half_open_max_attempts: 3,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for local/custom providers
    #[must_use]
    pub fn local() -> Self {
        Self {
            timeout_seconds: Some(10),
            max_retries: 2,
            retry_delay_ms: 100,
            max_batch_size: Some(32),
            rate_limit_rpm: None,
            rate_limit_tpm: None,
            compression_enabled: false,
            compression_threshold_bytes: 2048,
            connection_pool_size: 5,
            enable_circuit_breaker: false,
            circuit_breaker_config: None,
            enable_metrics: true,
        }
    }

    /// Get effective timeout (returns default if not specified)
    #[must_use]
    pub fn get_timeout_seconds(&self) -> u64 {
        self.timeout_seconds.unwrap_or(60)
    }

    /// Get effective batch size (returns default if not specified)
    #[must_use]
    pub fn get_max_batch_size(&self) -> usize {
        self.max_batch_size.unwrap_or(100)
    }
}
