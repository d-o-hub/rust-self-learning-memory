//! Table definitions and constants for redb storage

/// Maximum number of episodes to cache (configurable via metadata)
pub const DEFAULT_MAX_EPISODES: usize = 1000;

/// Maximum number of patterns to cache
pub const DEFAULT_MAX_PATTERNS: usize = 500;

/// Metadata keys
pub const METADATA_MAX_EPISODES: &str = "max_episodes";
pub const METADATA_LAST_SYNC: &str = "last_sync_timestamp";
pub const METADATA_VERSION: &str = "schema_version";

/// Current schema version
pub const SCHEMA_VERSION: &str = "1.0.0";
