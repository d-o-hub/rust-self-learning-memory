//! Metric provenance types for truthful storage telemetry (PTA-A2).
//!
//! Every metric reported by the storage commands carries an explicit
//! [`MetricProvenance`]: either it was measured through a backend interface,
//! it is an explicitly labeled estimate, or it is unavailable. This prevents
//! fabricated values (e.g. a zero cache-hit rate or a fixed connection-pool
//! size) from being presented as observed telemetry.

use serde::Serialize;

/// Provenance of a reported metric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricProvenance {
    /// Directly measured through a backend interface.
    Measured,
    /// Derived estimate; not a direct measurement.
    Estimated,
    /// Not available through current backend interfaces.
    Unavailable,
}

/// A metric value with explicit provenance.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetricValue<T> {
    /// The value, or `None` when unavailable.
    pub value: Option<T>,
    /// How the value was obtained.
    pub provenance: MetricProvenance,
}

impl<T> MetricValue<T> {
    /// Build a directly measured metric.
    pub fn measured(value: T) -> Self {
        Self {
            value: Some(value),
            provenance: MetricProvenance::Measured,
        }
    }

    /// Build an explicitly estimated metric.
    pub fn estimated(value: T) -> Self {
        Self {
            value: Some(value),
            provenance: MetricProvenance::Estimated,
        }
    }

    /// Build a metric that is not available through current interfaces.
    pub fn unavailable() -> Self {
        Self {
            value: None,
            provenance: MetricProvenance::Unavailable,
        }
    }

    /// Whether no real value is reported (unavailable provenance or no value).
    pub fn is_unavailable(&self) -> bool {
        matches!(self.provenance, MetricProvenance::Unavailable) || self.value.is_none()
    }
}

impl<T: std::fmt::Display> MetricValue<T> {
    /// Render for human output: value, `value (estimate)`, or `unavailable`.
    pub fn render(&self) -> String {
        self.render_with_suffix("")
    }

    /// Render for human output with a unit suffix, e.g. `2048 bytes (estimate)`.
    pub fn render_with_suffix(&self, suffix: &str) -> String {
        match (self.provenance, self.value.as_ref()) {
            (MetricProvenance::Measured, Some(v)) => format!("{v}{suffix}"),
            (MetricProvenance::Estimated, Some(v)) => format!("{v}{suffix} (estimate)"),
            _ => "unavailable".to_string(),
        }
    }
}

impl MetricValue<u64> {
    /// Render as megabytes, labeling estimates and unavailable metrics.
    pub fn render_mb(&self) -> String {
        match (self.provenance, self.value) {
            (MetricProvenance::Measured, Some(v)) => {
                format!("{:.2} MB", v as f32 / 1_000_000.0)
            }
            (MetricProvenance::Estimated, Some(v)) => {
                format!("{:.2} MB (estimate)", v as f32 / 1_000_000.0)
            }
            _ => "unavailable".to_string(),
        }
    }
}

impl MetricValue<f32> {
    /// Render as a percentage, labeling estimates and unavailable metrics.
    pub fn render_percent(&self) -> String {
        match (self.provenance, self.value) {
            (MetricProvenance::Measured, Some(v)) => format!("{:.1}%", v * 100.0),
            (MetricProvenance::Estimated, Some(v)) => format!("{:.1}% (estimate)", v * 100.0),
            _ => "unavailable".to_string(),
        }
    }
}
