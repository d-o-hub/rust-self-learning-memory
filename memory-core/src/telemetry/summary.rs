use crate::telemetry::HistogramStats;
use std::collections::HashMap;

/// Summary of all telemetry metrics
#[derive(Debug, Clone)]
pub struct TelemetrySummary {
    /// Counter metrics
    pub counters: HashMap<String, u64>,
    /// Histogram metrics
    pub histograms: HashMap<String, HistogramStats>,
    /// Gauge metrics
    pub gauges: HashMap<String, f64>,
}

impl TelemetrySummary {
    /// Create an empty summary
    #[must_use]
    pub fn empty() -> Self {
        Self {
            counters: HashMap::new(),
            histograms: HashMap::new(),
            gauges: HashMap::new(),
        }
    }

    /// Check if summary is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.counters.is_empty() && self.histograms.is_empty() && self.gauges.is_empty()
    }

    /// Get total number of metrics
    #[must_use]
    pub fn total_metrics(&self) -> usize {
        self.counters.len() + self.histograms.len() + self.gauges.len()
    }

    /// Get formatted string representation
    #[must_use]
    pub fn format(&self) -> String {
        let mut output = String::from("Telemetry Summary:\n");

        // Counters
        output.push_str("  Counters:\n");
        for (name, value) in &self.counters {
            output.push_str(&format!("    {}: {}\n", name, value));
        }

        // Histograms
        output.push_str("  Histograms:\n");
        for (name, stats) in &self.histograms {
            output.push_str(&format!(
                "    {}: count={}, p50={:.2}, p95={:.2}, p99={:.2}\n",
                name, stats.count, stats.p50, stats.p95, stats.p99
            ));
        }

        // Gauges
        output.push_str("  Gauges:\n");
        for (name, value) in &self.gauges {
            output.push_str(&format!("    {}: {:.2}\n", name, value));
        }

        output
    }
}

impl Default for TelemetrySummary {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let summary = TelemetrySummary::empty();
        assert!(summary.is_empty());
        assert_eq!(summary.total_metrics(), 0);
    }

    #[test]
    fn test_format() {
        let mut summary = TelemetrySummary::empty();
        summary.counters.insert("test_counter".to_string(), 42);
        summary.gauges.insert("test_gauge".to_string(), 3.14);

        let formatted = summary.format();
        assert!(formatted.contains("Telemetry Summary"));
        assert!(formatted.contains("test_counter: 42"));
        assert!(formatted.contains("test_gauge: 3.14"));
    }
}
