//! Table definitions and constants for redb storage

// Template constants for future use (cache limits, metadata keys, schema versioning)
// These are not currently used but provide configuration for future enhancements

/// Maximum number of episodes to cache (configurable via metadata)
#[allow(dead_code)]
pub const DEFAULT_MAX_EPISODES: usize = 1000;

/// Maximum number of patterns to cache
#[allow(dead_code)]
pub const DEFAULT_MAX_PATTERNS: usize = 500;

/// Metadata keys
#[allow(dead_code)]
pub const METADATA_MAX_EPISODES: &str = "max_episodes";
#[allow(dead_code)]
pub const METADATA_LAST_SYNC: &str = "last_sync_timestamp";
#[allow(dead_code)]
pub const METADATA_VERSION: &str = "schema_version";

/// Current schema version
#[allow(dead_code)]
pub const SCHEMA_VERSION: &str = "1.0.0";
