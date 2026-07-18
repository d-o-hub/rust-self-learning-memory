//! Non-blocking bounded audit file writer.
//!
//! Owns the log file handle on a dedicated OS thread so request-path async
//! workers never block on disk I/O. Uses a bounded `sync_channel`; when the
//! queue is full, lines are dropped and an atomic counter is incremented.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

/// Default capacity for the audit write queue.
pub const DEFAULT_AUDIT_WRITE_QUEUE_CAPACITY: usize = 1024;

/// Commands processed by the background writer thread.
enum WriterCommand {
    /// Append a single log line (without trailing newline; writer adds it).
    Line(String),
    /// Drain-complete acknowledgment for tests / graceful flush.
    Flush(mpsc::Sender<()>),
    /// Stop the writer thread after processing prior commands.
    Shutdown(mpsc::Sender<()>),
}

/// Configuration for the file writer (rotation + path).
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Path to the active audit log file.
    pub file_path: PathBuf,
    /// Whether to rotate when `max_file_size` is exceeded.
    pub enable_rotation: bool,
    /// Maximum log file size in bytes before rotation.
    pub max_file_size: u64,
    /// Maximum number of rotated log files to keep.
    pub max_rotated_files: usize,
    /// Bounded queue capacity for non-blocking enqueue.
    pub queue_capacity: usize,
}

/// Bounded, non-blocking audit file writer with overflow metrics.
///
/// The background thread is detached (no `JoinHandle` stored) so this type
/// remains `Send + Sync` for use inside `Arc<AuditLogger>`.
pub struct AuditFileWriter {
    tx: SyncSender<WriterCommand>,
    dropped: Arc<AtomicU64>,
}

impl AuditFileWriter {
    /// Start a background writer thread for `config`.
    ///
    /// Initializes size tracking from the existing file metadata so oversized
    /// logs rotate on the first write.
    pub fn start(config: WriterConfig) -> anyhow::Result<Self> {
        let capacity = config.queue_capacity.max(1);
        let (tx, rx) = mpsc::sync_channel::<WriterCommand>(capacity);
        let dropped = Arc::new(AtomicU64::new(0));

        // Ensure parent directory exists (sync; only at startup).
        if let Some(parent) = config.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.file_path)?;

        let initial_size = file.metadata()?.len();
        info!(
            "Audit file writer started: {:?} (current size: {} bytes, queue capacity: {})",
            config.file_path, initial_size, capacity
        );

        // Dropping the JoinHandle detaches the thread; shutdown is coordinated
        // via the channel in Drop so AuditFileWriter stays Sync.
        let handle = thread::Builder::new()
            .name("audit-log-writer".into())
            .spawn(move || {
                run_writer_loop(rx, file, initial_size, config);
            })?;
        drop(handle);

        Ok(Self { tx, dropped })
    }

    /// Try to enqueue a log line without blocking.
    ///
    /// Returns `true` if enqueued, `false` if dropped (queue full or closed).
    pub fn try_enqueue(&self, line: String) -> bool {
        match self.tx.try_send(WriterCommand::Line(line)) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                false
            }
            Err(TrySendError::Disconnected(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    /// Number of log lines dropped due to full/closed queue.
    pub fn dropped_writes(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Block until the writer has processed all currently queued lines.
    ///
    /// Used by tests and graceful shutdown paths. Waits up to `timeout`.
    pub fn flush(&self, timeout: Duration) -> bool {
        let (ack_tx, ack_rx) = mpsc::channel();
        // Blocking send so Flush is not lost when the queue is full.
        if self.tx.send(WriterCommand::Flush(ack_tx)).is_err() {
            return false;
        }
        ack_rx.recv_timeout(timeout).is_ok()
    }
}

impl Drop for AuditFileWriter {
    fn drop(&mut self) {
        let (ack_tx, ack_rx) = mpsc::channel();
        // Best-effort shutdown: ignore send failures (thread already gone).
        let _ = self.tx.send(WriterCommand::Shutdown(ack_tx));
        let _ = ack_rx.recv_timeout(Duration::from_secs(2));
    }
}

fn run_writer_loop(
    rx: mpsc::Receiver<WriterCommand>,
    mut file: File,
    mut current_size: u64,
    config: WriterConfig,
) {
    loop {
        match rx.recv() {
            Ok(WriterCommand::Line(line)) => {
                if config.enable_rotation && current_size >= config.max_file_size {
                    match rotate_logs(&config) {
                        Ok(new_file) => {
                            file = new_file;
                            current_size = 0;
                            info!("Audit log files rotated successfully");
                        }
                        Err(e) => {
                            error!("Failed to rotate audit log: {}", e);
                            // Still attempt to write to the current file.
                        }
                    }
                }

                if let Err(e) = writeln!(file, "{}", line) {
                    error!("Failed to write audit log to file: {}", e);
                } else if let Err(e) = file.flush() {
                    error!("Failed to flush audit log file: {}", e);
                } else {
                    current_size = current_size.saturating_add(line.len() as u64 + 1);
                }
            }
            Ok(WriterCommand::Flush(ack)) => {
                let _ = file.flush();
                let _ = ack.send(());
            }
            Ok(WriterCommand::Shutdown(ack)) => {
                let _ = file.flush();
                let _ = ack.send(());
                break;
            }
            Err(_) => {
                // All senders dropped.
                let _ = file.flush();
                break;
            }
        }
    }
}

fn rotate_logs(config: &WriterConfig) -> anyhow::Result<File> {
    let base_path = &config.file_path;

    // Rotate existing files: base -> .log.1, .log.1 -> .log.2, ...
    for i in (1..config.max_rotated_files).rev() {
        let old_path = if i == 1 {
            base_path.clone()
        } else {
            base_path.with_extension(format!("log.{}", i - 1))
        };

        let new_path = base_path.with_extension(format!("log.{}", i));

        if old_path.exists() {
            if let Err(e) = std::fs::rename(&old_path, &new_path) {
                warn!(
                    "Failed to rotate log file {:?} to {:?}: {}",
                    old_path, new_path, e
                );
            }
        }
    }

    let oldest_path = base_path.with_extension(format!("log.{}", config.max_rotated_files));
    if oldest_path.exists() {
        if let Err(e) = std::fs::remove_file(&oldest_path) {
            warn!("Failed to remove oldest log file {:?}: {}", oldest_path, e);
        }
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(base_path)?;

    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn test_config(path: PathBuf, capacity: usize, max_size: u64) -> WriterConfig {
        WriterConfig {
            file_path: path,
            enable_rotation: true,
            max_file_size: max_size,
            max_rotated_files: 3,
            queue_capacity: capacity,
        }
    }

    #[test]
    fn test_writer_enqueues_and_writes() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("audit.log");
        let writer = AuditFileWriter::start(test_config(path.clone(), 64, 1024 * 1024)).unwrap();

        // Act
        assert!(writer.try_enqueue(r#"{"op":"test"}"#.to_string()));
        assert!(writer.flush(Duration::from_secs(2)));

        // Assert
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("test"));
        assert_eq!(writer.dropped_writes(), 0);
    }

    #[test]
    fn test_writer_initializes_size_from_existing_file_and_rotates() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("audit.log");

        {
            let mut f = File::create(&path).unwrap();
            let payload = vec![b'x'; 200];
            f.write_all(&payload).unwrap();
            f.flush().unwrap();
        }
        assert!(std::fs::metadata(&path).unwrap().len() >= 200);

        // Act
        let writer = AuditFileWriter::start(test_config(path.clone(), 64, 100)).unwrap();
        assert!(writer.try_enqueue(r#"{"after":"rotation"}"#.to_string()));
        assert!(writer.flush(Duration::from_secs(2)));

        // Assert — rotation should have moved the oversized file aside.
        let active_len = std::fs::metadata(&path).unwrap().len();
        let rotated = path.with_extension("log.1");
        assert!(
            rotated.exists() || active_len < 200,
            "expected rotation: rotated_exists={}, active_len={}",
            rotated.exists(),
            active_len
        );

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("after"));
    }

    #[test]
    fn test_writer_drops_when_queue_full() {
        // Arrange — capacity 1 so a flood forces overflow drops.
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("audit.log");
        let writer = AuditFileWriter::start(test_config(path, 1, 1024 * 1024)).unwrap();

        // Act
        let mut enqueued = 0u64;
        for i in 0..10_000 {
            if writer.try_enqueue(format!(r#"{{"i":{}}}"#, i)) {
                enqueued += 1;
            }
        }

        // Assert
        let dropped = writer.dropped_writes();
        assert!(
            dropped > 0,
            "expected drops under capacity-1 flood; enqueued={enqueued}, dropped={dropped}"
        );
        assert!(writer.flush(Duration::from_secs(5)));
        assert!(enqueued > 0);
    }
}
