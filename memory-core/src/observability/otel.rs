//! Opt-in OpenTelemetry OTLP tracing (R-F2).
//!
//! The `otel` feature is intentionally inert until an OTLP endpoint is
//! configured. With no `OTEL_EXPORTER_OTLP_ENDPOINT`,
//! [`init_otel_from_env`] returns `Ok(None)` and builds no exporter, so a
//! build that enables the feature still pays no network or batch-processor
//! cost.
//!
//! # Configuration
//!
//! | Variable | Meaning |
//! |----------|---------|
//! | `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP/HTTP endpoint (for example `http://localhost:4318`) |
//! | `OTEL_SERVICE_NAME` | Resource service name (default: `do-memory-core`) |
//! | `OTEL_TRACES_SAMPLER_ARG` | Trace-id ratio sampler value in `[0, 1]` (default: `1.0`) |
//!
//! Credentials in the endpoint's userinfo are never rendered: use
//! [`OtelConfig::redacted_endpoint`] in logs and diagnostics. OTLP headers are
//! deliberately left to the exporter's environment configuration and are never
//! copied into span attributes.
//!
//! # Example
//!
//! ```no_run
//! use do_memory_core::observability::otel::init_otel_from_env;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let _guard = init_otel_from_env(Some("info"))?;
//! # Ok(())
//! # }
//! ```
//!
//! The returned guard owns the tracer provider; drop it (or call
//! [`OtelGuard::shutdown`]) to flush and shut the exporter down.

use std::time::Duration;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

/// Environment variable that enables the OTLP exporter when set.
pub const ENDPOINT_ENV: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Environment variable for the resource service name.
pub const SERVICE_NAME_ENV: &str = "OTEL_SERVICE_NAME";
/// Environment variable for the trace-id-ratio sampler value.
pub const SAMPLER_ARG_ENV: &str = "OTEL_TRACES_SAMPLER_ARG";

const DEFAULT_SERVICE_NAME: &str = "do-memory-core";
const EXPORT_TIMEOUT: Duration = Duration::from_secs(10);

/// Error returned while building the exporter or installing the subscriber.
#[derive(Debug, thiserror::Error)]
pub enum OtelInitError {
    /// The OTLP exporter could not be built from the supplied configuration.
    #[error("failed to build the OTLP span exporter: {0}")]
    Exporter(String),
    /// A tracing subscriber was already installed in this process.
    #[error("tracing subscriber is already initialized: {0}")]
    Subscriber(String),
}

/// Resolved OpenTelemetry exporter configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct OtelConfig {
    /// OTLP/HTTP endpoint, for example `http://localhost:4318`.
    pub endpoint: String,
    /// Resource service name.
    pub service_name: String,
    /// Trace-id-ratio sampler value in `[0, 1]`.
    pub sample_ratio: f64,
}

impl OtelConfig {
    /// Read the configuration from the process environment.
    ///
    /// Returns `None` when `OTEL_EXPORTER_OTLP_ENDPOINT` is unset or empty,
    /// which is the zero-overhead path.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        Self::from_env_with(|key| std::env::var(key).ok())
    }

    /// Read the configuration through an injectable environment accessor.
    #[must_use]
    pub fn from_env_with<F>(get: F) -> Option<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        let endpoint = get(ENDPOINT_ENV)?;
        if endpoint.trim().is_empty() {
            return None;
        }
        let service_name = get(SERVICE_NAME_ENV)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SERVICE_NAME.to_string());
        let sample_ratio = get(SAMPLER_ARG_ENV)
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .map_or(1.0, |value| value.clamp(0.0, 1.0));

        Some(Self {
            endpoint,
            service_name,
            sample_ratio,
        })
    }

    /// Render the endpoint with userinfo and query/fragment components removed.
    #[must_use]
    pub fn redacted_endpoint(&self) -> String {
        redact_endpoint(&self.endpoint)
    }
}

/// Guard that owns the tracer provider and shuts it down on drop.
#[derive(Debug)]
pub struct OtelGuard {
    provider: Option<SdkTracerProvider>,
}

impl OtelGuard {
    /// Force-flush pending spans.
    ///
    /// # Errors
    ///
    /// Returns the SDK error rendered as a string when the exporter cannot
    /// flush.
    pub fn force_flush(&self) -> Result<(), String> {
        self.provider.as_ref().map_or(Ok(()), |provider| {
            provider.force_flush().map_err(|error| error.to_string())
        })
    }

    /// Flush and shut the provider down explicitly.
    pub fn shutdown(mut self) {
        if let Some(provider) = self.provider.take() {
            let _ = provider.shutdown();
        }
    }
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.provider.take() {
            let _ = provider.shutdown();
        }
    }
}

/// Initialize the OTLP pipeline when `OTEL_EXPORTER_OTLP_ENDPOINT` is set.
///
/// Returns `Ok(None)` without building an exporter when the endpoint is unset.
///
/// # Errors
///
/// Returns [`OtelInitError`] when the exporter cannot be built or a tracing
/// subscriber is already installed.
pub fn init_otel_from_env(filter: Option<&str>) -> Result<Option<OtelGuard>, OtelInitError> {
    match OtelConfig::from_env() {
        Some(config) => init_otel(config, filter).map(Some),
        None => Ok(None),
    }
}

/// Build the OTLP/HTTP exporter and install a tracing subscriber that includes
/// both the human-readable console layer and the OpenTelemetry layer.
///
/// # Errors
///
/// Returns [`OtelInitError`] when the exporter cannot be built or a tracing
/// subscriber is already installed.
pub fn init_otel(config: OtelConfig, filter: Option<&str>) -> Result<OtelGuard, OtelInitError> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(config.endpoint.clone())
        .with_timeout(EXPORT_TIMEOUT)
        .build()
        .map_err(|error| OtelInitError::Exporter(error.to_string()))?;

    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(config.service_name.clone())
                .build(),
        )
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            config.sample_ratio,
        ))))
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer("do-memory-core");
    let filter = EnvFilter::new(filter.unwrap_or("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).with_thread_ids(true))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()
        .map_err(|error| OtelInitError::Subscriber(error.to_string()))?;

    Ok(OtelGuard {
        provider: Some(provider),
    })
}

/// Remove URL userinfo and query/fragment data from an OTLP endpoint.
fn redact_endpoint(endpoint: &str) -> String {
    let (scheme, rest) = endpoint
        .split_once("://")
        .map_or((None, endpoint), |(scheme, rest)| (Some(scheme), rest));
    let rest = rest.split(['?', '#']).next().unwrap_or(rest);
    let authority_end = rest.find('/').unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let path = &rest[authority_end..];
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_userinfo, host)| host);

    match scheme {
        Some(scheme) => format!("{scheme}://{host}{path}"),
        None => format!("{host}{path}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_sdk::testing::trace::new_test_exporter;
    use std::collections::HashMap;

    fn config(endpoint: &str) -> OtelConfig {
        OtelConfig {
            endpoint: endpoint.to_string(),
            service_name: DEFAULT_SERVICE_NAME.to_string(),
            sample_ratio: 1.0,
        }
    }

    #[test]
    fn endpoint_redaction_removes_credentials_and_query() {
        assert_eq!(
            config("https://user:pass@collector.example:4318/v1/traces?token=secret#frag")
                .redacted_endpoint(),
            "https://collector.example:4318/v1/traces"
        );
        assert_eq!(
            config("http://collector.example:4318").redacted_endpoint(),
            "http://collector.example:4318"
        );
    }

    #[test]
    fn unset_or_empty_endpoint_disables_the_exporter() {
        assert!(OtelConfig::from_env_with(|_| None).is_none());
        let empty = HashMap::from([(ENDPOINT_ENV, "   ".to_string())]);
        assert!(OtelConfig::from_env_with(|key| empty.get(key).cloned()).is_none());
    }

    #[test]
    fn sampler_is_clamped_and_defaults_to_one() {
        let configured = OtelConfig::from_env_with(|key| match key {
            ENDPOINT_ENV => Some("http://localhost:4318".to_string()),
            SAMPLER_ARG_ENV => Some("4.0".to_string()),
            _ => None,
        })
        .expect("endpoint is set");
        assert_eq!(configured.sample_ratio, 1.0);

        let defaulted = OtelConfig::from_env_with(|key| {
            (key == ENDPOINT_ENV).then(|| "http://localhost:4318".to_string())
        })
        .expect("endpoint is set");
        assert_eq!(defaulted.sample_ratio, 1.0);
    }

    #[cfg(feature = "otel-testing")]
    #[tokio::test]
    async fn otel_layer_exports_a_span_without_credentials() {
        let (exporter, mut exported, _shutdown) = new_test_exporter();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
        let tracer = provider.tracer("otel-test");
        let subscriber =
            tracing_subscriber::registry().with(tracing_opentelemetry::layer().with_tracer(tracer));

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("otel_test_span", episode_id = "abc");
            let _entered = span.enter();
        });

        provider.force_flush().expect("flush test spans");
        let span = exported.try_recv().expect("one exported span");
        assert_eq!(span.name, "otel_test_span");
        assert!(
            !span
                .attributes
                .iter()
                .any(|attribute| attribute.key.as_str().contains("token")),
            "the exporter must not invent credential attributes"
        );
    }
}
