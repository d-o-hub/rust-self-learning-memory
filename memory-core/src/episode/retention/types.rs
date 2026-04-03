//! Retention types, enums, and result structures.

use serde::{Deserialize, Serialize};

/// Criteria for episode retention decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionCriterion {
    /// Keep all episodes (no cleanup)
    KeepAll,
    /// Delete episodes older than a threshold
    AgeBased,
    /// Delete episodes with low reward scores
    RewardBased,
    /// Delete unreferenced episodes (no patterns/heuristics derived)
    Unreferenced,
    /// Delete failed episodes with no successful patterns
    FailedOnly,
    /// Combined criteria (all must pass to keep)
    Combined,
}

/// Trigger for running cleanup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetentionTrigger {
    /// Scheduled cleanup interval
    Scheduled,
    /// Storage quota exceeded
    StorageExceeded {
        /// Current episode count
        current: usize,
        /// Maximum allowed
        max: usize,
    },
    /// Memory pressure threshold reached
    MemoryPressure {
        /// Pressure level (0.0 - 1.0)
        level: f64,
    },
    /// Manual cleanup requested
    Manual,
}

/// Errors in retention policy configuration
#[derive(Debug, Clone, PartialEq)]
pub enum RetentionPolicyError {
    /// Invalid maximum age (must be >= 0)
    InvalidMaxAge(i64),
    /// Invalid reward threshold (must be 0.0 - 1.0)
    InvalidThreshold(f64),
    /// Invalid batch size (must be > 0)
    InvalidBatchSize,
}

impl std::fmt::Display for RetentionPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMaxAge(days) => write!(f, "Invalid max age: {days} (must be >= 0)"),
            Self::InvalidThreshold(threshold) => {
                write!(f, "Invalid threshold: {threshold} (must be 0.0 - 1.0)")
            }
            Self::InvalidBatchSize => write!(f, "Invalid batch size: must be > 0"),
        }
    }
}

impl std::error::Error for RetentionPolicyError {}

/// Result of a cleanup operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Number of episodes evaluated
    pub evaluated: usize,

    /// Number of episodes deleted
    pub deleted: usize,

    /// Number of episodes kept
    pub kept: usize,

    /// IDs of deleted episodes
    pub deleted_ids: Vec<uuid::Uuid>,

    /// IDs of kept episodes (when dry run)
    pub kept_ids: Vec<uuid::Uuid>,

    /// Trigger that caused cleanup
    pub trigger: Option<RetentionTrigger>,

    /// Duration of cleanup in milliseconds
    pub duration_ms: u64,

    /// Error messages (if any)
    pub errors: Vec<String>,
}

impl CleanupResult {
    /// Create a new cleanup result
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a deleted episode
    pub fn add_deleted(&mut self, id: uuid::Uuid) {
        self.deleted += 1;
        self.deleted_ids.push(id);
    }

    /// Add a kept episode
    pub fn add_kept(&mut self, id: uuid::Uuid) {
        self.kept += 1;
        self.kept_ids.push(id);
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Check if cleanup had any errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get cleanup success rate
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.evaluated == 0 {
            return 1.0;
        }
        let successful = self.evaluated - self.errors.len();
        successful as f64 / self.evaluated as f64
    }
}
