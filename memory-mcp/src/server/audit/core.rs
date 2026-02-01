//! Audit logging core implementation
//!
//! This module provides the core AuditLogger struct and its implementation,
//! including file handling, rotation, and event logging.

use super::types::{AuditConfig, AuditDestination, AuditLogEntry, AuditLogLevel};
use chrono::Utc;
use serde_json::json;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Audit logger implementation
pub struct AuditLogger {
    config: AuditConfig,
    file_handle: Arc<Mutex<Option<File>>>,
    current_file_size: Arc<Mutex<u64>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub async fn new(config: AuditConfig) -> anyhow::Result<Self> {
        let file_handle = if config.destination == AuditDestination::File
            || config.destination == AuditDestination::Both
        {
            let path = config
                .file_path
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from("audit.log"));

            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let file = OpenOptions::new().create(true).append(true).open(&path)?;

            let metadata = file.metadata()?;
            let current_size = metadata.len();

            info!(
                "Audit logger initialized with file: {:?} (current size: {} bytes)",
                path, current_size
            );

            Some(file)
        } else {
            info!("Audit logger initialized with stdout output only");
            None
        };

        Ok(Self {
            config,
            file_handle: Arc::new(Mutex::new(file_handle)),
            current_file_size: Arc::new(Mutex::new(0)),
        })
    }

    /// Log a generic audit event
    pub async fn log_event(
        &self,
        level: AuditLogLevel,
        client_id: &str,
        operation: &str,
        result: &str,
        metadata: serde_json::Value,
    ) {
        if !self.config.enabled || !self.config.log_level.should_log(level) {
            return;
        }

        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: format!("{:?}", level).to_lowercase(),
            client_id: client_id.to_string(),
            operation: operation.to_string(),
            result: result.to_string(),
            metadata: self.redact_sensitive_data(metadata),
        };

        let log_line = match serde_json::to_string(&entry) {
            Ok(line) => line,
            Err(e) => {
                error!("Failed to serialize audit log entry: {}", e);
                return;
            }
        };

        // Write to appropriate destinations
        match self.config.destination {
            AuditDestination::Stdout => {
                println!("{}", log_line);
            }
            AuditDestination::File => {
                self.write_to_file(&log_line).await;
            }
            AuditDestination::Both => {
                println!("{}", log_line);
                self.write_to_file(&log_line).await;
            }
        }

        debug!("Audit log entry: {}", log_line);
    }

    /// Write log line to file with rotation support
    fn write_to_file<'a>(
        &'a self,
        log_line: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let mut file_guard = self.file_handle.lock().await;
            let mut size_guard = self.current_file_size.lock().await;

            if let Some(ref mut file) = *file_guard {
                // Check if rotation is needed
                if self.config.enable_rotation && *size_guard >= self.config.max_file_size {
                    drop(file_guard);
                    drop(size_guard);
                    self.rotate_logs().await;
                    return self.write_to_file(log_line).await;
                }

                if let Err(e) = writeln!(file, "{}", log_line) {
                    error!("Failed to write audit log to file: {}", e);
                } else if let Err(e) = file.flush() {
                    error!("Failed to flush audit log file: {}", e);
                } else {
                    *size_guard += log_line.len() as u64 + 1; // +1 for newline
                }
            }
        })
    }

    /// Rotate log files
    async fn rotate_logs(&self) {
        if let Some(ref base_path) = self.config.file_path {
            let base_path = base_path.clone();

            // Close current file
            {
                let mut file_guard = self.file_handle.lock().await;
                *file_guard = None;
            }

            // Rotate existing files
            for i in (1..self.config.max_rotated_files).rev() {
                let old_path = if i == 1 {
                    base_path.clone()
                } else {
                    base_path.with_extension(format!("log.{}", i - 1))
                };

                let new_path = base_path.with_extension(format!("log.{}", i));

                if old_path.exists() {
                    if let Err(e) = tokio::fs::rename(&old_path, &new_path).await {
                        warn!(
                            "Failed to rotate log file {:?} to {:?}: {}",
                            old_path, new_path, e
                        );
                    }
                }
            }

            // Remove oldest file if it exists
            let oldest_path =
                base_path.with_extension(format!("log.{}", self.config.max_rotated_files));
            if oldest_path.exists() {
                if let Err(e) = tokio::fs::remove_file(&oldest_path).await {
                    warn!("Failed to remove oldest log file {:?}: {}", oldest_path, e);
                }
            }

            // Reopen file for writing
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&base_path)
            {
                Ok(file) => {
                    let mut file_guard = self.file_handle.lock().await;
                    let mut size_guard = self.current_file_size.lock().await;
                    *file_guard = Some(file);
                    *size_guard = 0;
                    info!("Log files rotated successfully");
                }
                Err(e) => {
                    error!("Failed to reopen audit log file after rotation: {}", e);
                }
            }
        }
    }

    /// Redact sensitive data from metadata
    fn redact_sensitive_data(&self, mut metadata: serde_json::Value) -> serde_json::Value {
        if let Some(obj) = metadata.as_object_mut() {
            for (key, value) in obj.iter_mut() {
                if self
                    .config
                    .redact_fields
                    .iter()
                    .any(|f| key.to_lowercase().contains(f))
                {
                    *value = json!("[REDACTED]");
                }
            }
        }
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{AuditConfig, AuditDestination, AuditLogLevel};
    use super::*;
    use std::collections::HashSet;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await;
        assert!(logger.is_ok());
    }

    #[tokio::test]
    async fn test_audit_logger_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit.log");

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path.clone()),
            enable_rotation: false,
            max_file_size: 1024,
            max_rotated_files: 5,
            redact_fields: HashSet::new(),
            log_level: AuditLogLevel::Debug,
        };

        let logger = AuditLogger::new(config).await.unwrap();

        // Log an event
        logger
            .log_event(
                AuditLogLevel::Info,
                "test-client",
                "test_operation",
                "success",
                json!({"test": "data"}),
            )
            .await;

        // Give a moment for the file to be written
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check that file was created and contains the log
        assert!(log_path.exists());
        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(content.contains("test-client"));
        assert!(content.contains("test_operation"));
    }

    #[test]
    fn test_redact_sensitive_data() {
        let mut config = AuditConfig::default();
        config.redact_fields.insert("secret".to_string());

        let logger = AuditLogger {
            config: config.clone(),
            file_handle: Arc::new(Mutex::new(None)),
            current_file_size: Arc::new(Mutex::new(0)),
        };

        let metadata = json!({
            "public_field": "visible",
            "secret_key": "should_be_hidden",
            "nested_secret": "also_hidden"
        });

        let redacted = logger.redact_sensitive_data(metadata);
        let obj = redacted.as_object().unwrap();

        assert_eq!(obj["public_field"], "visible");
        assert_eq!(obj["secret_key"], "[REDACTED]");
        assert_eq!(obj["nested_secret"], "[REDACTED]");
    }

    #[tokio::test]
    async fn test_audit_logger_disabled() {
        let config = AuditConfig {
            enabled: false,
            ..AuditConfig::default()
        };

        let logger = AuditLogger::new(config).await.unwrap();

        // This should not panic or error when disabled
        logger
            .log_event(
                AuditLogLevel::Info,
                "test-client",
                "test_operation",
                "success",
                json!({}),
            )
            .await;
    }
}
