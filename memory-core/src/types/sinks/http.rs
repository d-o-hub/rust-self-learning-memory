//! HttpEmitter — Delivers CloudEvents to an HTTP webhook endpoint.
//!
//! Sends CloudEvents in binary content mode (CloudEvents 1.0 HTTP binding)
//! to a configured URL. Useful for production environments where events
//! should be routed to external services, message brokers, or monitoring.

use crate::types::emitter::{CloudEvent, EventEmitter};
use async_trait::async_trait;
use std::time::Duration;

/// An `EventEmitter` that sends CloudEvents to an HTTP endpoint.
///
/// Uses CloudEvents 1.0 HTTP Protocol Binding (binary content mode):
/// - `ce-specversion`, `ce-type`, `ce-source`, `ce-id`, `ce-time`, `ce-subject`
///   are sent as HTTP headers
/// - The event `data` payload is sent as the HTTP body (JSON)
///
/// # Examples
///
/// ```no_run
/// use do_memory_core::types::emitter::CloudEvent;
/// use do_memory_core::types::sinks::HttpEmitter;
/// use do_memory_core::types::emitter::EventEmitter;
///
/// # async fn example() {
/// let emitter = HttpEmitter::new("https://webhook.example.com/events");
/// let event = CloudEvent::new(
///     "com.test".to_string(),
///     "test://src".to_string(),
///     serde_json::json!({"key": "value"}),
/// );
/// emitter.emit(event).await.unwrap();
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct HttpEmitter {
    /// Webhook endpoint URL
    url: String,
    /// HTTP client for sending events
    client: reqwest::Client,
}

impl HttpEmitter {
    /// Create a new `HttpEmitter` that sends events to the given URL.
    ///
    /// The underlying HTTP client is configured with a 30-second timeout
    /// to prevent hanging connections.
    #[must_use]
    pub fn new(url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client with timeout");
        Self {
            url: url.to_string(),
            client,
        }
    }

    /// Get the webhook URL this emitter sends to.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[async_trait]
impl EventEmitter for HttpEmitter {
    async fn emit(&self, event: CloudEvent) -> anyhow::Result<()> {
        let mut request = self
            .client
            .post(&self.url)
            .header("ce-specversion", &event.specversion)
            .header("ce-type", &event.event_type)
            .header("ce-source", &event.source)
            .header("ce-id", &event.id)
            .header("ce-time", &event.time)
            .header("content-type", "application/json");

        if let Some(ref subject) = event.subject {
            request = request.header("ce-subject", subject);
        }

        let response = request.json(&event.data).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "HTTP emitter returned {} for event {}: {}",
                response.status().as_u16(),
                event.id,
                response.text().await.unwrap_or_default(),
            );
        }

        tracing::debug!(
            event_type = %event.event_type,
            event_id = %event.id,
            status = %response.status().as_u16(),
            "CloudEvent delivered via HTTP",
        );
        Ok(())
    }

    async fn emit_batch(&self, events: Vec<CloudEvent>) -> anyhow::Result<()> {
        // Send events concurrently using join_all
        let futures: Vec<_> = events.into_iter().map(|e| self.emit(e)).collect();
        let results = futures::future::join_all(futures).await;
        let mut errors = Vec::new();
        for result in results {
            if let Err(e) = result {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            let error_details: Vec<String> = errors
                .iter()
                .enumerate()
                .map(|(i, e)| format!("  [{}] {}", i, e))
                .collect();
            anyhow::bail!(
                "HTTP emitter batch had {} failure(s):\n{}",
                errors.len(),
                error_details.join("\n")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_emitter_creation() {
        let emitter = HttpEmitter::new("https://example.com/events");
        assert_eq!(emitter.url(), "https://example.com/events");
    }

    #[tokio::test]
    async fn test_http_emitter_emit_to_invalid_url() {
        let emitter = HttpEmitter::new("http://127.0.0.1:1/events");
        let event = CloudEvent::new(
            "com.test.event".to_string(),
            "test://source".to_string(),
            serde_json::json!({"test": true}),
        );

        // Should fail to connect (no server on port 1)
        let result = emitter.emit(event).await;
        assert!(result.is_err());
    }
}
