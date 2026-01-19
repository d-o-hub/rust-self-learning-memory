//! Enhanced error handling module for memory-cli
//!
//! Provides helpful error messages with context and suggestions for common issues.

use anyhow::Result;
use colored::Colorize;

/// Trait for adding helpful context and suggestions to errors
#[allow(dead_code)]
pub trait EnhancedError<T> {
    /// Add context with helpful suggestions
    fn context_with_help(self, msg: &str, help: &[&str]) -> Result<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> EnhancedError<T> for Result<T, E> {
    fn context_with_help(self, msg: &str, help: &[&str]) -> Result<T> {
        self.map_err(|e| {
            let mut error_msg = format!("{}\n\n{}", msg.red().bold(), e);

            if !help.is_empty() {
                error_msg.push_str(&format!("\n\n{}", "Possible solutions:".yellow().bold()));
                for (i, h) in help.iter().enumerate() {
                    error_msg.push_str(&format!("\n  {}. {}", i + 1, h));
                }
            }

            anyhow::anyhow!(error_msg)
        })
    }
}

/// Common error messages and help text
#[allow(dead_code)]
pub mod helpers {
    /// Episode not found error help
    #[allow(dead_code)]
    pub const EPISODE_NOT_FOUND_HELP: &[&str] = &[
        "Check that the episode ID is correct (use 'memory-cli episode list' or 'memory-cli ep list')",
        "Verify the episode hasn't been deleted",
        "Check storage connection: 'memory-cli storage health' or 'memory-cli st health'",
    ];

    /// Pattern not found error help
    #[allow(dead_code)]
    pub const PATTERN_NOT_FOUND_HELP: &[&str] = &[
        "Check that the pattern ID is correct (use 'memory-cli pattern list' or 'memory-cli pat list')",
        "Patterns may have decayed due to low confidence",
        "Verify storage connection: 'memory-cli storage health' or 'memory-cli st health'",
    ];

    /// Storage connection error help
    #[allow(dead_code)]
    pub const STORAGE_CONNECTION_HELP: &[&str] = &[
        "Verify Turso database URL is correct in configuration",
        "Check network connectivity if using cloud database",
        "Try using local database: set TURSO_URL=file:./memory.db",
        "Validate configuration: 'memory-cli config validate'",
    ];

    /// Configuration error help
    #[allow(dead_code)]
    pub const CONFIG_ERROR_HELP: &[&str] = &[
        "Check configuration file at ~/.config/memory-cli/config.toml",
        "Verify all required fields are present",
        "Use 'memory-cli config check' to validate configuration",
        "See documentation for configuration examples",
    ];

    /// Invalid input error help
    #[allow(dead_code)]
    pub const INVALID_INPUT_HELP: &[&str] = &[
        "Check input format and data types",
        "Verify JSON syntax if providing context",
        "Use --help to see valid options",
    ];

    /// Database operation error help
    #[allow(dead_code)]
    pub const DATABASE_OPERATION_HELP: &[&str] = &[
        "Check storage health: 'memory-cli storage health' or 'memory-cli st health'",
        "Try synchronizing storage: 'memory-cli storage sync' or 'memory-cli st sync'",
        "Consider vacuuming database: 'memory-cli storage vacuum' or 'memory-cli st vacuum'",
        "Check database file permissions",
    ];

    /// Storage operation error help
    #[allow(dead_code)]
    pub const STORAGE_ERROR_HELP: &[&str] = &[
        "Verify database connection is healthy: 'memory-cli storage health'",
        "Check database file exists and has correct permissions",
        "Try restarting the application to reset connection pool",
        "Consider recreating the database if corruption is suspected",
    ];

    /// Format enhanced error message
    #[allow(dead_code)]
    pub fn format_error_message(error: &str, context: &str, help: &[&str]) -> String {
        use colored::Colorize;

        let mut msg = format!("{}\n\n{}", context.red().bold(), error);

        if !help.is_empty() {
            msg.push_str(&format!("\n\n{}", "Possible solutions:".yellow().bold()));
            for (i, h) in help.iter().enumerate() {
                msg.push_str(&format!("\n  {}. {}", i + 1, h));
            }
        }

        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_error_contains_help() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let enhanced = result.context_with_help("Operation failed", &["Solution 1", "Solution 2"]);

        assert!(enhanced.is_err());
        let error_str = enhanced.unwrap_err().to_string();
        assert!(error_str.contains("Possible solutions:"));
        assert!(error_str.contains("Solution 1"));
        assert!(error_str.contains("Solution 2"));
    }

    #[test]
    fn test_enhanced_error_without_help() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let enhanced = result.context_with_help("Operation failed", &[]);

        assert!(enhanced.is_err());
        let error_str = enhanced.unwrap_err().to_string();
        assert!(error_str.contains("Operation failed"));
        assert!(!error_str.contains("Possible solutions:"));
    }

    #[test]
    fn test_format_error_message() {
        let formatted = helpers::format_error_message(
            "Test error",
            "Operation failed",
            &["Solution 1", "Solution 2"],
        );

        assert!(formatted.contains("Operation failed"));
        assert!(formatted.contains("Test error"));
        assert!(formatted.contains("Possible solutions:"));
        assert!(formatted.contains("1. Solution 1"));
        assert!(formatted.contains("2. Solution 2"));
    }
}
