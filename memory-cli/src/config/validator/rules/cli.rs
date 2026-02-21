//! CLI configuration validation rules.

use crate::config::ValidationError;
use crate::config::types::CliConfig;

/// Validate CLI configuration (errors only).
pub fn validate_cli_config_errors(config: &CliConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate output format
    match config.default_format.as_str() {
        "human" | "json" | "yaml" => {}
        _ => {
            errors.push(ValidationError {
                field: "cli.default_format".to_string(),
                message: format!("Invalid output format: {}", config.default_format),
                suggestion: Some("Use 'human', 'json', or 'yaml'".to_string()),
                context: Some("Output formatting".to_string()),
            });
        }
    }

    // Validate batch size
    if config.batch_size == 0 {
        errors.push(ValidationError {
            field: "cli.batch_size".to_string(),
            message: "batch_size must be greater than 0".to_string(),
            suggestion: Some("Use a value between 1 and 1000".to_string()),
            context: Some("Bulk operations".to_string()),
        });
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cli_config_valid() {
        let config = CliConfig {
            default_format: "json".to_string(),
            progress_bars: false,
            batch_size: 100,
        };
        let errors = validate_cli_config_errors(&config);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_cli_config_invalid_format() {
        let config = CliConfig {
            default_format: "invalid".to_string(),
            progress_bars: false,
            batch_size: 100,
        };
        let errors = validate_cli_config_errors(&config);
        assert!(!errors.is_empty());
    }
}
