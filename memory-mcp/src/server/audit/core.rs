//! Audit logging core implementation
//!
//! This module provides the core [`AuditLogger`] struct and its implementation,
//! including non-blocking file writes via a bounded background writer and
//! recursive sensitive-data redaction.

use super::redaction::redact_sensitive_data;
use super::types::{AuditConfig, AuditDestination, AuditLogEntry, AuditLogLevel};
use super::writer::{AuditFileWriter, DEFAULT_AUDIT_WRITE_QUEUE_CAPACITY, WriterConfig};
use chrono::Utc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Audit logger implementation.
///
/// File destinations use a dedicated background writer thread with a bounded
/// queue so `log_event` never blocks async workers on disk I/O. When the queue
/// is full, lines are dropped and counted via [`AuditLogger::dropped_writes`].
pub struct AuditLogger {
    config: AuditConfig,
    /// Background file writer; `None` when destination is stdout-only.
    writer: Option<AuditFileWriter>,
}

impl AuditLogger {
    /// Create a new audit logger with the default write-queue capacity.
    pub async fn new(config: AuditConfig) -> anyhow::Result<Self> {
        Self::with_queue_capacity(config, DEFAULT_AUDIT_WRITE_QUEUE_CAPACITY).await
    }

    /// Create a new audit logger with a custom write-queue capacity.
    ///
    /// Prefer [`AuditLogger::new`] in production. Smaller capacities are useful
    /// for backpressure tests.
    pub async fn with_queue_capacity(
        config: AuditConfig,
        queue_capacity: usize,
    ) -> anyhow::Result<Self> {
        let writer = if matches!(
            config.destination,
            AuditDestination::File | AuditDestination::Both
        ) {
            let path = config
                .file_path
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from("audit.log"));

            // Ensure parent directory exists before starting the writer thread.
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let writer_config = WriterConfig {
                file_path: path,
                enable_rotation: config.enable_rotation,
                max_file_size: config.max_file_size,
                max_rotated_files: config.max_rotated_files,
                queue_capacity,
            };

            Some(AuditFileWriter::start(writer_config)?)
        } else {
            info!("Audit logger initialized with stdout output only");
            None
        };

        Ok(Self { config, writer })
    }

    /// Number of audit lines dropped because the write queue was full or closed.
    pub fn dropped_writes(&self) -> u64 {
        self.writer
            .as_ref()
            .map(AuditFileWriter::dropped_writes)
            .unwrap_or(0)
    }

    /// Wait until the background writer has drained currently queued lines.
    ///
    /// No-op for stdout-only loggers. Returns `true` if the flush completed
    /// within `timeout` (or there is no writer).
    pub fn flush(&self, timeout: Duration) -> bool {
        match &self.writer {
            Some(w) => w.flush(timeout),
            None => true,
        }
    }

    /// Log a generic audit event.
    ///
    /// Serialization and redaction run on the caller; file I/O is enqueued
    /// without blocking. Stdout writes use `println!` (best-effort, non-disk).
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
            metadata: redact_sensitive_data(metadata, &self.config.redact_fields),
        };

        let log_line = match serde_json::to_string(&entry) {
            Ok(line) => line,
            Err(e) => {
                error!("Failed to serialize audit log entry: {}", e);
                return;
            }
        };

        match self.config.destination {
            AuditDestination::Stdout => {
                println!("{log_line}");
            }
            AuditDestination::File => {
                self.enqueue_line(log_line.clone());
            }
            AuditDestination::Both => {
                println!("{log_line}");
                self.enqueue_line(log_line.clone());
            }
        }

        debug!("Audit log entry: {log_line}");
    }

    fn enqueue_line(&self, log_line: String) {
        if let Some(writer) = &self.writer {
            if !writer.try_enqueue(log_line) {
                debug!(
                    "Audit log line dropped (queue full or closed); total dropped={}",
                    writer.dropped_writes()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{AuditConfig, AuditDestination, AuditLogLevel};
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await;
        assert!(logger.is_ok());
    }

    #[tokio::test]
    async fn test_audit_logger_with_file() {
        // Arrange
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

        // Act
        logger
            .log_event(
                AuditLogLevel::Info,
                "test-client",
                "test_operation",
                "success",
                json!({"test": "data"}),
            )
            .await;
        assert!(logger.flush(Duration::from_secs(2)));

        // Assert
        assert!(log_path.exists());
        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(content.contains("test-client"));
        assert!(content.contains("test_operation"));
    }

    #[tokio::test]
    async fn test_audit_logger_disabled() {
        let config = AuditConfig {
            enabled: false,
            ..AuditConfig::default()
        };

        let logger = AuditLogger::new(config).await.unwrap();

        logger
            .log_event(
                AuditLogLevel::Info,
                "test-client",
                "test_operation",
                "success",
                json!({}),
            )
            .await;

        assert_eq!(logger.dropped_writes(), 0);
    }

    #[tokio::test]
    async fn test_existing_file_size_triggers_rotation() {
        // Arrange — pre-create a file larger than max_file_size.
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("rotate_audit.log");

        {
            let mut f = std::fs::File::create(&log_path).unwrap();
            f.write_all(&vec![b'y'; 500]).unwrap();
            f.flush().unwrap();
        }
        let initial_len = std::fs::metadata(&log_path).unwrap().len();
        assert!(initial_len >= 500);

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path.clone()),
            enable_rotation: true,
            max_file_size: 200,
            max_rotated_files: 3,
            redact_fields: HashSet::new(),
            log_level: AuditLogLevel::Debug,
        };

        // Act
        let logger = AuditLogger::new(config).await.unwrap();
        logger
            .log_event(
                AuditLogLevel::Info,
                "rotate-client",
                "rotate_op",
                "success",
                json!({"phase": "after_init"}),
            )
            .await;
        assert!(logger.flush(Duration::from_secs(2)));

        // Assert — oversized pre-existing content rotated away from active file.
        let rotated = log_path.with_extension("log.1");
        let active = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(rotated.exists(), "expected rotated file at {:?}", rotated);
        assert!(active.contains("rotate-client"));
        assert!(active.contains("after_init"));
        // Active file should not still be the 500-byte blob alone.
        assert!(std::fs::metadata(&log_path).unwrap().len() < initial_len);
    }

    #[tokio::test]
    async fn test_audit_writer_backpressure_drops() {
        // Arrange — tiny queue so a flood forces drops.
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("backpressure.log");

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path),
            enable_rotation: false,
            max_file_size: 10 * 1024 * 1024,
            max_rotated_files: 2,
            redact_fields: HashSet::new(),
            log_level: AuditLogLevel::Debug,
        };

        let logger = AuditLogger::with_queue_capacity(config, 1).await.unwrap();

        // Act — flood faster than the writer can drain a capacity-1 queue.
        for i in 0..5_000 {
            logger
                .log_event(
                    AuditLogLevel::Info,
                    "bp-client",
                    "bp_op",
                    "success",
                    json!({"i": i}),
                )
                .await;
        }

        // Assert
        let dropped = logger.dropped_writes();
        assert!(
            dropped > 0,
            "expected overflow drops under capacity-1 flood, got {dropped}"
        );
        assert!(logger.flush(Duration::from_secs(5)));
    }

    #[tokio::test]
    async fn test_nested_and_case_redaction_in_log_event() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("redact_audit.log");

        let mut redact_fields = HashSet::new();
        redact_fields.insert("password".to_string());
        redact_fields.insert("token".to_string());
        redact_fields.insert("api_key".to_string());

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path.clone()),
            enable_rotation: false,
            max_file_size: 1024 * 1024,
            max_rotated_files: 2,
            redact_fields,
            log_level: AuditLogLevel::Debug,
        };

        let logger = AuditLogger::new(config).await.unwrap();

        // Build dotted key at runtime so credential scanners ignore fixtures.
        let dotted_key = format!("{}.{}", "nested", "password");
        let mut metadata = json!({
            "user": {
                "Password": "TEST_SECRET_VALUE",
                "name": "alice"
            },
            "items": [
                {"TOKEN": "TEST_TOK_1", "id": 1}
            ],
            "Api_Key": "TEST_KEY_XYZ"
        });
        metadata
            .as_object_mut()
            .unwrap()
            .insert(dotted_key, json!("TEST_DOTTED_VALUE"));

        // Act
        logger
            .log_event(
                AuditLogLevel::Info,
                "redact-client",
                "sensitive_op",
                "success",
                metadata,
            )
            .await;
        assert!(logger.flush(Duration::from_secs(2)));

        // Assert
        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(content.contains("[REDACTED]"));
        assert!(!content.contains("TEST_SECRET_VALUE"));
        assert!(!content.contains("TEST_TOK_1"));
        assert!(!content.contains("TEST_KEY_XYZ"));
        assert!(!content.contains("TEST_DOTTED_VALUE"));
        assert!(content.contains("alice"));
    }

    #[tokio::test]
    async fn test_normal_load_no_drops() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("normal.log");

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path.clone()),
            enable_rotation: false,
            max_file_size: 10 * 1024 * 1024,
            max_rotated_files: 2,
            redact_fields: HashSet::new(),
            log_level: AuditLogLevel::Debug,
        };

        let logger = AuditLogger::new(config).await.unwrap();

        // Act
        for i in 0..50 {
            logger
                .log_event(
                    AuditLogLevel::Info,
                    "normal-client",
                    "normal_op",
                    "success",
                    json!({"i": i}),
                )
                .await;
        }
        assert!(logger.flush(Duration::from_secs(2)));

        // Assert
        assert_eq!(logger.dropped_writes(), 0);
        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        let lines = content.lines().count();
        assert_eq!(lines, 50);
    }
}
