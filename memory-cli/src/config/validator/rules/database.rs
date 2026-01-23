//! Database configuration validation rules.

use crate::config::types::DatabaseConfig;
use crate::config::ValidationError;

/// Validate database configuration.
pub fn validate_database_config(config: &DatabaseConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check if at least one storage option is configured
    if config.turso_url.is_none() && config.redb_path.is_none() {
        errors.push(ValidationError {
            field: "database".to_string(),
            message: "At least one database configuration is required".to_string(),
            suggestion: Some("Configure either turso_url or redb_path".to_string()),
            context: Some("No durable storage backend configured".to_string()),
        });
    }

    // Validate Turso URL if provided
    if let Some(turso_url) = &config.turso_url {
        if turso_url.trim().is_empty() {
            errors.push(ValidationError {
                field: "database.turso_url".to_string(),
                message: "Turso URL cannot be empty".to_string(),
                suggestion: Some("Provide a valid Turso database URL".to_string()),
                context: Some("Remote database access".to_string()),
            });
        } else if !is_valid_turso_url(turso_url) {
            // Warning, not error - handled elsewhere
        } else {
            // Security check for file: URLs
            if let Err(security_error) = validate_file_url_security(turso_url) {
                errors.push(security_error);
            }
        }
    }

    // Validate redb path if provided
    if let Some(redb_path) = &config.redb_path {
        if redb_path.trim().is_empty() {
            errors.push(ValidationError {
                field: "database.redb_path".to_string(),
                message: "redb path cannot be empty".to_string(),
                suggestion: Some(
                    "Provide a valid file path or use ':memory:' for in-memory storage".to_string(),
                ),
                context: Some("Local cache storage".to_string()),
            });
        }
    }

    errors
}

/// Check if Turso URL format is valid.
pub fn is_valid_turso_url(url: &str) -> bool {
    url.starts_with("libsql://") || url.starts_with("file:")
}

/// Validate that a file URL doesn't contain path traversal or access sensitive paths.
pub fn validate_file_url_security(url: &str) -> Result<(), ValidationError> {
    if !url.starts_with("file:") {
        return Ok(());
    }

    let path = url.strip_prefix("file:").unwrap_or(url);

    // Check for path traversal attempts
    if path.contains("..") {
        return Err(ValidationError {
            field: "database.turso_url".to_string(),
            message: "Storage error: Path traversal detected in database URL".to_string(),
            suggestion: Some("Use an absolute path without '..' components".to_string()),
            context: Some("Security: Path traversal attacks are blocked".to_string()),
        });
    }

    // Check for access to sensitive system paths
    let sensitive_paths = [
        "/etc/",
        "/bin/",
        "/sbin/",
        "/usr/bin/",
        "/usr/sbin/",
        "/sys/",
        "/proc/",
        "/dev/",
        "/boot/",
        "/root/",
        "/var/log/",
        "/var/run/",
    ];

    for sensitive_path in &sensitive_paths {
        if path.starts_with(sensitive_path) {
            return Err(ValidationError {
                field: "database.turso_url".to_string(),
                message: format!(
                    "Storage error: Access to sensitive system path is not allowed: {}",
                    path
                ),
                suggestion: Some(
                    "Use a path in your home directory or project directory".to_string(),
                ),
                context: Some("Security: Access to system paths is blocked".to_string()),
            });
        }
    }

    // Check for specific sensitive files
    let sensitive_files = ["/etc/passwd", "/etc/shadow", "/etc/hosts", "/etc/sudoers"];

    for sensitive_file in &sensitive_files {
        if path == *sensitive_file || path.ends_with(sensitive_file) {
            return Err(ValidationError {
                field: "database.turso_url".to_string(),
                message: format!(
                    "Storage error: Access to sensitive file is not allowed: {}",
                    sensitive_file
                ),
                suggestion: Some("Use a database file in your project directory".to_string()),
                context: Some("Security: Access to sensitive files is blocked".to_string()),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_database_config_no_storage() {
        let config = DatabaseConfig {
            turso_url: None,
            turso_token: None,
            redb_path: None,
        };
        let errors = validate_database_config(&config);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_database_config_with_redb() {
        let config = DatabaseConfig {
            turso_url: None,
            turso_token: None,
            redb_path: Some("/tmp/test.redb".to_string()),
        };
        let errors = validate_database_config(&config);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_is_valid_turso_url() {
        assert!(is_valid_turso_url("libsql://test.db"));
        assert!(is_valid_turso_url("file:/path/to/db"));
        assert!(!is_valid_turso_url("https://example.com"));
    }

    #[test]
    fn test_validate_file_url_security_valid() {
        let result = validate_file_url_security("file:/tmp/test.db");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_url_security_path_traversal() {
        let result = validate_file_url_security("file:/tmp/../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_url_security_sensitive_path() {
        let result = validate_file_url_security("file:/etc/passwd");
        assert!(result.is_err());
    }
}
