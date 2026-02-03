//! Types for metrics export

/// Type of metric
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Counter (monotonically increasing)
    Counter,
    /// Gauge (can go up or down)
    Gauge,
    /// Histogram (distribution of values)
    Histogram,
    /// Summary (calculated percentiles)
    Summary,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Counter => write!(f, "counter"),
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Histogram => write!(f, "histogram"),
            MetricType::Summary => write!(f, "summary"),
        }
    }
}

/// Value of a metric
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricValue {
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Duration in microseconds
    DurationMicros(u64),
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::Integer(v) => write!(f, "{}", v),
            MetricValue::Float(v) => write!(f, "{:.6}", v),
            MetricValue::DurationMicros(v) => write!(f, "{}", v),
        }
    }
}

/// A single exported metric
#[derive(Debug, Clone)]
pub struct ExportedMetric {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: MetricValue,
    /// Labels (key-value pairs)
    pub labels: Vec<(String, String)>,
    /// Help text
    pub help: String,
    /// Timestamp (optional)
    pub timestamp: Option<i64>,
}

impl ExportedMetric {
    /// Create a new metric
    pub fn new(name: &str, metric_type: MetricType, value: MetricValue) -> Self {
        Self {
            name: name.to_string(),
            metric_type,
            value,
            labels: Vec::new(),
            help: String::new(),
            timestamp: None,
        }
    }

    /// Add a label
    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.push((key.to_string(), value.to_string()));
        self
    }

    /// Set help text
    pub fn with_help(mut self, help: &str) -> Self {
        self.help = help.to_string();
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Format as Prometheus exposition format
    pub fn to_prometheus(&self) -> String {
        let mut output = String::new();

        // Add help line if present
        if !self.help.is_empty() {
            output.push_str(&format!("# HELP {} {}\n", self.name, self.help));
        }

        // Add type line
        output.push_str(&format!("# TYPE {} {}\n", self.name, self.metric_type));

        // Format labels
        let label_str = if self.labels.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = self
                .labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            format!("{{{}}}", pairs.join(","))
        };

        // Add value line
        if let Some(ts) = self.timestamp {
            output.push_str(&format!(
                "{}{} {} {}\n",
                self.name, label_str, self.value, ts
            ));
        } else {
            output.push_str(&format!("{}{} {}\n", self.name, label_str, self.value));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_type_display() {
        assert_eq!(MetricType::Counter.to_string(), "counter");
        assert_eq!(MetricType::Gauge.to_string(), "gauge");
        assert_eq!(MetricType::Histogram.to_string(), "histogram");
        assert_eq!(MetricType::Summary.to_string(), "summary");
    }

    #[test]
    fn test_metric_value_display() {
        assert_eq!(MetricValue::Integer(42).to_string(), "42");
        assert_eq!(MetricValue::Float(3.14159).to_string(), "3.141590");
        assert_eq!(MetricValue::DurationMicros(1000).to_string(), "1000");
    }

    #[test]
    fn test_exported_metric_creation() {
        let metric = ExportedMetric::new(
            "test_metric",
            MetricType::Counter,
            MetricValue::Integer(100),
        );

        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert_eq!(metric.value, MetricValue::Integer(100));
        assert!(metric.labels.is_empty());
    }

    #[test]
    fn test_exported_metric_with_label() {
        let metric = ExportedMetric::new("test_metric", MetricType::Gauge, MetricValue::Float(1.5))
            .with_label("operation", "get")
            .with_label("status", "success");

        assert_eq!(metric.labels.len(), 2);
        assert_eq!(
            metric.labels[0],
            ("operation".to_string(), "get".to_string())
        );
        assert_eq!(
            metric.labels[1],
            ("status".to_string(), "success".to_string())
        );
    }

    #[test]
    fn test_exported_metric_prometheus_format() {
        let metric = ExportedMetric::new(
            "test_counter",
            MetricType::Counter,
            MetricValue::Integer(100),
        )
        .with_help("A test counter")
        .with_label("env", "test");

        let output = metric.to_prometheus();
        assert!(output.contains("# HELP test_counter A test counter"));
        assert!(output.contains("# TYPE test_counter counter"));
        assert!(output.contains("test_counter{env=\"test\"} 100"));
    }

    #[test]
    fn test_exported_metric_with_timestamp() {
        let metric = ExportedMetric::new("test_gauge", MetricType::Gauge, MetricValue::Float(2.5))
            .with_timestamp(1234567890);

        let output = metric.to_prometheus();
        assert!(output.contains("test_gauge 2.500000 1234567890"));
    }
}
