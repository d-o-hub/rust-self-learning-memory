//! Configuration for metrics export

use std::time::Duration;

/// Export format for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    /// Prometheus exposition format (text)
    #[default]
    Prometheus,
    /// JSON format
    Json,
    /// OpenTelemetry format
    OpenTelemetry,
}

/// Export target for metrics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportTarget {
    /// HTTP endpoint for scraping
    Http {
        /// Bind address
        bind_address: String,
        /// Port to listen on
        port: u16,
        /// Path for metrics endpoint
        path: String,
    },
    /// File-based export
    File {
        /// Path to output file
        path: String,
        /// Rotation interval
        rotation_interval: Duration,
    },
    /// Stdout export (for debugging)
    Stdout,
    /// No export (collection only)
    None,
}

impl Default for ExportTarget {
    fn default() -> Self {
        Self::Http {
            bind_address: "127.0.0.1".to_string(),
            port: 9090,
            path: "/metrics".to_string(),
        }
    }
}

/// Configuration for metrics export
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Export format
    pub format: ExportFormat,
    /// Export target
    pub target: ExportTarget,
    /// Export interval (for file/stdout targets)
    pub export_interval: Duration,
    /// Include operation-level metrics
    pub include_operations: bool,
    /// Include cache metrics
    pub include_cache: bool,
    /// Include pool metrics
    pub include_pool: bool,
    /// Include error metrics
    pub include_errors: bool,
    /// Maximum number of operation types to track
    pub max_operation_types: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::default(),
            target: ExportTarget::default(),
            export_interval: Duration::from_secs(60),
            include_operations: true,
            include_cache: true,
            include_pool: true,
            include_errors: true,
            max_operation_types: 100,
        }
    }
}

impl ExportConfig {
    /// Create a new configuration with HTTP endpoint
    pub fn http(bind_address: &str, port: u16) -> Self {
        Self {
            target: ExportTarget::Http {
                bind_address: bind_address.to_string(),
                port,
                path: "/metrics".to_string(),
            },
            ..Default::default()
        }
    }

    /// Create a new configuration with file output
    pub fn file(path: &str, rotation_interval: Duration) -> Self {
        Self {
            target: ExportTarget::File {
                path: path.to_string(),
                rotation_interval,
            },
            ..Default::default()
        }
    }

    /// Create a stdout-only configuration
    pub fn stdout() -> Self {
        Self {
            target: ExportTarget::Stdout,
            ..Default::default()
        }
    }

    /// Create a collection-only configuration (no export)
    pub fn collection_only() -> Self {
        Self {
            target: ExportTarget::None,
            ..Default::default()
        }
    }

    /// Set export format
    pub fn with_format(mut self, format: ExportFormat) -> Self {
        self.format = format;
        self
    }

    /// Set export interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.export_interval = interval;
        self
    }

    /// Disable operation metrics
    pub fn without_operations(mut self) -> Self {
        self.include_operations = false;
        self
    }

    /// Disable cache metrics
    pub fn without_cache(mut self) -> Self {
        self.include_cache = false;
        self
    }

    /// Disable pool metrics
    pub fn without_pool(mut self) -> Self {
        self.include_pool = false;
        self
    }

    /// Disable error metrics
    pub fn without_errors(mut self) -> Self {
        self.include_errors = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ExportConfig::default();
        assert_eq!(config.format, ExportFormat::Prometheus);
        assert_eq!(config.export_interval, Duration::from_secs(60));
        assert!(config.include_operations);
        assert!(config.include_cache);
        assert!(config.include_pool);
        assert!(config.include_errors);
    }

    #[test]
    fn test_http_config() {
        let config = ExportConfig::http("0.0.0.0", 8080);
        match config.target {
            ExportTarget::Http {
                bind_address,
                port,
                path,
            } => {
                assert_eq!(bind_address, "0.0.0.0");
                assert_eq!(port, 8080);
                assert_eq!(path, "/metrics");
            }
            _ => panic!("Expected Http target"),
        }
    }

    #[test]
    fn test_file_config() {
        let config = ExportConfig::file("/tmp/metrics.prom", Duration::from_secs(3600));
        match config.target {
            ExportTarget::File {
                path,
                rotation_interval,
            } => {
                assert_eq!(path, "/tmp/metrics.prom");
                assert_eq!(rotation_interval, Duration::from_secs(3600));
            }
            _ => panic!("Expected File target"),
        }
    }

    #[test]
    fn test_builder_methods() {
        let config = ExportConfig::default()
            .with_format(ExportFormat::Json)
            .with_interval(Duration::from_secs(30))
            .without_cache()
            .without_errors();

        assert_eq!(config.format, ExportFormat::Json);
        assert_eq!(config.export_interval, Duration::from_secs(30));
        assert!(!config.include_cache);
        assert!(!config.include_errors);
        assert!(config.include_operations);
        assert!(config.include_pool);
    }
}
