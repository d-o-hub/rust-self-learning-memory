//! Validation messages module
//!
//! This module provides types for validation results, errors, and warnings,
//! as well as formatting functions for displaying validation results.

use std::path::Path;

use crate::config::Config;

/// Validation result with enhanced error context
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// List of validation errors with context
    pub errors: Vec<ValidationError>,
    /// List of warnings that don't prevent usage
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result with errors
    pub fn with_errors(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add warnings to a validation result
    pub fn with_warnings(mut self, warnings: Vec<ValidationWarning>) -> Self {
        self.warnings.extend(warnings);
        self
    }
}

/// Enhanced validation error with context and suggestions
#[derive(Debug)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Human-readable error message
    pub message: String,
    /// Suggested value or fix
    pub suggestion: Option<String>,
    /// Context for the error
    pub context: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " Suggestion: {}", suggestion)?;
        }
        if let Some(context) = &self.context {
            write!(f, " Context: {}", context)?;
        }
        Ok(())
    }
}

/// Non-blocking validation warnings
#[derive(Debug)]
pub struct ValidationWarning {
    /// Field that generated the warning
    pub field: String,
    /// Warning message
    pub message: String,
    /// Suggested improvement
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

/// Validate the entire configuration
pub fn validate_config(config: &Config) -> ValidationResult {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // Validate database configuration
    let db_validation = super::rules::validate_database_config(&config.database);
    errors.extend(db_validation.errors);
    warnings.extend(db_validation.warnings);

    // Validate storage configuration
    let storage_validation = super::rules::validate_storage_config(&config.storage);
    errors.extend(storage_validation.errors);
    warnings.extend(storage_validation.warnings);

    // Validate CLI configuration
    let cli_validation = super::rules::validate_cli_config(&config.cli);
    errors.extend(cli_validation.errors);
    warnings.extend(cli_validation.warnings);

    // Cross-configuration validation
    let cross_warnings = super::rules::validate_cross_config(config);
    warnings.extend(cross_warnings);

    let is_valid = errors.is_empty();
    let mut result = if is_valid {
        ValidationResult::ok()
    } else {
        ValidationResult::with_errors(errors)
    };

    if !warnings.is_empty() {
        result = result.with_warnings(warnings);
    }

    result
}

/// Validate configuration file path
pub fn validate_config_path(path: &Path) -> Result<(), ValidationError> {
    if !path.exists() {
        return Err(ValidationError {
            field: "config_path".to_string(),
            message: format!("Configuration file does not exist: {}", path.display()),
            suggestion: Some("Check file path or create configuration file".to_string()),
            context: Some("File system validation".to_string()),
        });
    }

    if !path.is_file() {
        return Err(ValidationError {
            field: "config_path".to_string(),
            message: format!("Configuration path is not a file: {}", path.display()),
            suggestion: Some("Provide a valid file path".to_string()),
            context: Some("File system validation".to_string()),
        });
    }

    // Check file extension
    match path.extension().and_then(|s| s.to_str()) {
        Some("toml") | Some("json") | Some("yaml") | Some("yml") => Ok(()),
        Some(ext) => Err(ValidationError {
            field: "config_path".to_string(),
            message: format!("Unsupported configuration file format: .{}", ext),
            suggestion: Some("Use .toml, .json, .yaml, or .yml".to_string()),
            context: Some("File format validation".to_string()),
        }),
        None => Err(ValidationError {
            field: "config_path".to_string(),
            message: "Configuration file has no extension".to_string(),
            suggestion: Some("Add .toml, .json, .yaml, or .yml extension".to_string()),
            context: Some("File format validation".to_string()),
        }),
    }
}

/// Helper function to format validation results
pub fn format_validation_result(result: &ValidationResult) -> String {
    let mut output = String::new();

    if result.is_valid {
        output.push_str("Configuration is valid\n");
    } else {
        output.push_str(&format!(
            "Configuration has {} error(s):\n",
            result.errors.len()
        ));
        for (i, error) in result.errors.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, error));
        }
    }

    if !result.warnings.is_empty() {
        output.push_str(&format!(
            "\nConfiguration has {} warning(s):\n",
            result.warnings.len()
        ));
        for (i, warning) in result.warnings.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, warning));
        }
    }

    output
}

#[cfg(test)]
mod messages_tests {
    use super::*;

    #[test]
    fn test_validation_result_ok() {
        let result = ValidationResult::ok();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_with_errors() {
        let errors = vec![ValidationError {
            field: "test".to_string(),
            message: "Test error".to_string(),
            suggestion: None,
            context: None,
        }];
        let result = ValidationResult::with_errors(errors);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validation_result_with_warnings() {
        let result = ValidationResult::ok().with_warnings(vec![ValidationWarning {
            field: "test".to_string(),
            message: "Test warning".to_string(),
            suggestion: None,
        }]);
        assert!(result.is_valid);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            field: "test".to_string(),
            message: "Error message".to_string(),
            suggestion: Some("Fix it".to_string()),
            context: Some("Test context".to_string()),
        };
        let display = error.to_string();
        assert!(display.contains("Error message"));
        assert!(display.contains("Fix it"));
        assert!(display.contains("Test context"));
    }

    #[test]
    fn test_validation_warning_display() {
        let warning = ValidationWarning {
            field: "test".to_string(),
            message: "Warning message".to_string(),
            suggestion: Some("Improve it".to_string()),
        };
        let display = warning.to_string();
        assert!(display.contains("Warning message"));
        assert!(display.contains("Improve it"));
    }

    #[test]
    fn test_validate_config_path_exists() {
        let path = Path::new("/tmp");
        let result = validate_config_path(path);
        // /tmp should exist but is a directory, not a file
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_path_not_exists() {
        let path = Path::new("/nonexistent/path.toml");
        let result = validate_config_path(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_validation_result_valid() {
        let result = ValidationResult::ok();
        let output = format_validation_result(&result);
        assert!(output.contains("valid"));
    }

    #[test]
    fn test_format_validation_result_with_errors() {
        let errors = vec![ValidationError {
            field: "test".to_string(),
            message: "Test error".to_string(),
            suggestion: None,
            context: None,
        }];
        let result = ValidationResult::with_errors(errors);
        let output = format_validation_result(&result);
        assert!(output.contains("error"));
    }

    #[test]
    fn test_format_validation_result_with_warnings() {
        let result = ValidationResult::ok().with_warnings(vec![ValidationWarning {
            field: "test".to_string(),
            message: "Test warning".to_string(),
            suggestion: None,
        }]);
        let output = format_validation_result(&result);
        assert!(output.contains("warning"));
    }
}
