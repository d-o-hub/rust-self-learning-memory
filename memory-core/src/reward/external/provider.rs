//! External signal provider trait and implementations

use super::{ExternalSignalSet, Result};
use async_trait::async_trait;

/// Abstraction for external signal sources
///
/// Implement this trait to integrate a new external signal provider
/// into the reward system.
#[async_trait]
pub trait ExternalSignalProvider: Send + Sync {
    /// Unique provider identifier
    ///
    /// # Examples
    ///
    /// ```ignore
    /// "agentfs"
    /// "github-copilot"
    /// "my-audit-system"
    /// ```
    fn name(&self) -> &str;

    /// Fetch signals for a specific episode
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to fetch signals for
    ///
    /// # Returns
    ///
    /// A set of normalized external signals, or an error if the provider
    /// is unavailable or misconfigured.
    async fn get_signals(&self, episode: &crate::episode::Episode) -> Result<ExternalSignalSet>;

    /// Get provider health/status
    ///
    /// Used to determine if the provider is operational before
    /// attempting to fetch signals.
    async fn health_check(&self) -> ProviderHealth;

    /// Validate provider configuration
    ///
    /// Called during initialization to ensure the provider is
    /// properly configured.
    fn validate_config(&self) -> Result<()>;
}

/// Provider health status
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderHealth {
    /// Provider is operational
    Healthy,
    /// Provider is experiencing issues
    Degraded(String),
    /// Provider is not operational
    Unhealthy(String),
}

impl ProviderHealth {
    /// Check if the provider is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, ProviderHealth::Healthy)
    }

    /// Check if the provider is operational (healthy or degraded)
    pub fn is_operational(&self) -> bool {
        matches!(self, ProviderHealth::Healthy | ProviderHealth::Degraded(_))
    }
}

/// Mock provider for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;

    /// Test fixture provider that returns pre-configured signals
    pub struct MockExternalSignalProvider {
        canned_signals: HashMap<String, super::super::ExternalSignalSet>,
        health_status: ProviderHealth,
    }

    impl MockExternalSignalProvider {
        /// Create a new mock provider with canned signals
        pub fn with_signals(signals: Vec<(String, super::super::ExternalSignalSet)>) -> Self {
            Self {
                canned_signals: signals.into_iter().collect(),
                health_status: ProviderHealth::Healthy,
            }
        }

        /// Set the health status for testing
        #[must_use]
        pub fn with_health(mut self, health: ProviderHealth) -> Self {
            self.health_status = health;
            self
        }
    }

    #[async_trait]
    impl ExternalSignalProvider for MockExternalSignalProvider {
        fn name(&self) -> &'static str {
            "mock"
        }

        async fn get_signals(
            &self,
            episode: &crate::episode::Episode,
        ) -> Result<ExternalSignalSet> {
            let key = episode.episode_id.to_string();
            Ok(self
                .canned_signals
                .get(&key)
                .cloned()
                .unwrap_or_else(|| ExternalSignalSet::empty("mock")))
        }

        async fn health_check(&self) -> ProviderHealth {
            self.health_status.clone()
        }

        fn validate_config(&self) -> Result<()> {
            Ok(())
        }
    }
}
