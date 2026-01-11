// ============================================================================
// Validation Constants
// ============================================================================

/// Maximum length for task descriptions (10KB).
///
/// Prevents `DoS` attacks via unbounded input strings that could exhaust
/// memory during serialization or storage operations.
pub const MAX_DESCRIPTION_LEN: usize = 10_000;

/// Maximum number of execution steps per episode (1000).
///
/// Prevents resource exhaustion from episodes with excessive step logging.
pub const MAX_STEP_COUNT: usize = 1_000;

/// Maximum size for artifact data (1MB).
///
/// Limits the size of individual artifacts stored in episodes to prevent
/// storage bloat and memory exhaustion.
pub const MAX_ARTIFACT_SIZE: usize = 1_000_000;

/// Maximum length for step observations (10KB).
///
/// Prevents unbounded observation strings in execution steps.
pub const MAX_OBSERVATION_LEN: usize = 10_000;

/// Maximum size for serialized episode data (10MB).
///
/// Prevents `DoS` attacks via unbounded episode serialization that could
/// exhaust memory during bincode encoding/decoding operations.
pub const MAX_EPISODE_SIZE: usize = 10_000_000;

/// Maximum size for serialized pattern data (1MB).
///
/// Limits the size of individual patterns during serialization to prevent
/// memory exhaustion during bincode operations.
#[allow(dead_code)]
pub const MAX_PATTERN_SIZE: usize = 1_000_000;

/// Maximum size for serialized heuristic data (1MB).
///
/// Limits the size of individual heuristics during serialization to prevent
/// memory exhaustion during bincode operations.
#[allow(dead_code)]
pub const MAX_HEURISTIC_SIZE: usize = 1_000_000;
