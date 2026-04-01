//! External signal registry

use super::{ExternalSignalProvider, ExternalSignalSet};
use std::collections::HashMap;

/// Registry of available signal providers
///
/// Maintains a collection of external signal providers and
/// coordinates signal aggregation.
pub struct ExternalSignalRegistry {
    providers: HashMap<String, Box<dyn ExternalSignalProvider>>,
}

impl ExternalSignalRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a new provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider to register
    ///
    /// # Example
    ///
    /// ```ignore
    /// let registry = ExternalSignalRegistry::new();
    /// registry.register(Box::new(AgentFsProvider::new(config)));
    /// ```
    pub fn register(&mut self, provider: Box<dyn ExternalSignalProvider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }

    /// Get signals from all registered providers
    ///
    /// Iterates through all providers and aggregates their signals.
    /// Providers that fail are skipped and logged.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to fetch signals for
    pub async fn aggregate_signals(
        &self,
        episode: &crate::episode::Episode,
    ) -> Vec<ExternalSignalSet> {
        let mut signals = Vec::new();

        for (name, provider) in &self.providers {
            // Check health first
            let health = provider.health_check().await;
            if !health.is_operational() {
                tracing::warn!(provider = %name, "Provider not operational, skipping");
                continue;
            }

            // Fetch signals
            match provider.get_signals(episode).await {
                Ok(signal_set) => {
                    tracing::debug!(
                        provider = %name,
                        tool_count = signal_set.tool_signals.len(),
                        confidence = signal_set.confidence,
                        "Retrieved external signals"
                    );
                    signals.push(signal_set);
                }
                Err(e) => {
                    tracing::warn!(provider = %name, error = %e, "Failed to get signals");
                }
            }
        }

        signals
    }

    /// Get a specific provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn ExternalSignalProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Get list of registered provider names
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get count of registered providers
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    /// Remove a provider from the registry
    pub fn unregister(&mut self, name: &str) -> bool {
        self.providers.remove(name).is_some()
    }

    /// Clear all providers
    pub fn clear(&mut self) {
        self.providers.clear();
    }
}

impl Default for ExternalSignalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reward::external::MockExternalSignalProvider;

    #[test]
    fn test_registry_register() {
        let mut registry = ExternalSignalRegistry::new();
        let mock = MockExternalSignalProvider::with_signals(vec![]);

        registry.register(Box::new(mock));

        assert_eq!(registry.provider_count(), 1);
        assert!(registry.has_provider("mock"));
    }

    #[tokio::test]
    async fn test_registry_aggregate() {
        let mut registry = ExternalSignalRegistry::new();

        // Create mock with signals
        let mock = MockExternalSignalProvider::with_signals(vec![]);
        registry.register(Box::new(mock));

        // Create a dummy episode
        let episode = crate::episode::Episode::new(
            "test".to_string(),
            crate::types::TaskContext::default(),
            crate::types::TaskType::Testing,
        );

        // Aggregate (will return empty since no canned signals)
        let signals = registry.aggregate_signals(&episode).await;

        assert_eq!(signals.len(), 1); // One provider
        assert!(signals[0].tool_signals.is_empty());
    }
}
