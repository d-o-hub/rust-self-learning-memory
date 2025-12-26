                    category: "security".to_string(),
                    message: "Localhost URL may not be accessible from other systems".to_string(),
                    location: "database.turso_url".to_string(),
                    suggestion: Some("Use a publicly accessible URL for production".to_string()),
                });
            }
        }
        
        // Validate redb path
        if let Some(path) = &config.database.redb_path {
            if path.starts_with("/tmp") {
                warnings.push(ValidationWarning {
                    category: "storage".to_string(),
                    message: "Temporary path may cause data loss".to_string(),
                    location: "database.redb_path".to_string(),
                    suggestion: Some("Use a persistent path for important data".to_string()),
                });
            }
        }
        
        Ok(RuleResult {
            errors,
            warnings,
            suggestions,
        })
    }
}

struct PerformanceValidationRule;

impl ValidationRule for PerformanceValidationRule {
    fn validate(&self, config: &Config) -> Result<RuleResult, ConfigError> {
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        
        // Cache size recommendations
        if config.storage.max_episodes_cache < 100 {
            warnings.push(ValidationWarning {
                category: "performance".to_string(),
                message: "Cache size may be too small".to_string(),
                location: "storage.max_episodes_cache".to_string(),
                suggestion: Some("Consider increasing to at least 1000 for better performance".to_string()),
            });
        } else if config.storage.max_episodes_cache > 10000 {
            warnings.push(ValidationWarning {
                category: "performance".to_string(),
                message: "Cache size may be too large".to_string(),
                location: "storage.max_episodes_cache".to_string(),
                suggestion: Some("Monitor memory usage with large cache sizes".to_string()),
            });
        }
        
        // Pool size recommendations
        if config.storage.pool_size < 5 {
            suggestions.push("Consider increasing pool_size to at least 10 for better concurrency".to_string());
        } else if config.storage.pool_size > 50 {
            suggestions.push("Large pool sizes may not improve performance. Consider 10-20.".to_string());
        }
        
        Ok(RuleResult {
            errors: Vec::new(),
            warnings,
            suggestions,
        })
    }
}
```

### 4. Testing Strategy

#### 4.1 Unit Tests for Validation Rules

```rust
#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::config::{Config, DatabaseType, PerformanceLevel};
    
    #[test]
    fn test_validation_no_storage_backend() {
        let mut config = Config::default();
        config.database.turso_url = None;
        config.database.redb_path = None;
        
        let result = ConfigValidator::new().validate(&config).unwrap();
        
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| 
            e.category == "database" && 
            e.message.contains("No storage backend configured")
        ));
    }
    
    #[test]
    fn test_validation_turso_url_format() {
        let mut config = Config::default();
        config.database.turso_url = Some("invalid-url".to_string());
        
        let result = ConfigValidator::new().validate(&config).unwrap();
        
        assert!(result.errors.iter().any(|e| 
            e.category == "database" && 
            e.message.contains("Invalid Turso URL format")
        ));
    }
    
    #[test]
    fn test_validation_zero_values() {
        let mut config = Config::default();
        config.storage.max_episodes_cache = 0;
        config.storage.pool_size = 0;
        config.cli.batch_size = 0;
        
        let result = ConfigValidator::new().validate(&config).unwrap();
        
        assert_eq!(result.errors.len(), 3);
        assert!(result.errors.iter().any(|e| e.message.contains("cannot be zero")));
    }
    
    #[test]
    fn test_validation_performance_warnings() {
        let mut config = Config::default();
        config.storage.max_episodes_cache = 50; // Too small
        config.storage.pool_size = 100; // Too large
        
        let result = ConfigValidator::new().validate(&config).unwrap();
        
        assert!(result.warnings.len() >= 2);
        assert!(result.warnings.iter().any(|w| 
            w.category == "performance" && 
            w.message.contains("Cache size may be too small")
        ));
    }
    
    #[test]
    fn test_simple_mode_validation() {
        let config = Config::simple(DatabaseType::Local, PerformanceLevel::Minimal).unwrap();
        let result = ConfigValidator::new().validate(&config).unwrap();
        
        // Simple mode configurations should be valid
        assert!(result.is_valid, "Simple mode configuration should be valid: {:?}", result.errors);
    }
    
    #[test]
    fn test_validation_error_context() {
        let mut config = Config::default();
        config.storage.max_episodes_cache = 0;
        
        let result = ConfigValidator::new().validate(&config).unwrap();
        
        let error = result.errors.first().unwrap();
        assert!(error.location.contains("storage.max_episodes_cache"));
        assert!(error.suggestion.is_some());
    }
}
```

#### 4.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_config_lifecycle_with_validation() {
        // Test complete config lifecycle with validation
        let temp_dir = TempDir::new().unwrap();
        
        // Create configuration
        let config = Config::simple(DatabaseType::Local, PerformanceLevel::Standard).unwrap();
        
        // Validate
        let validation = ConfigValidator::new().validate(&config).unwrap();
        assert!(validation.is_valid);
        
        // Save to file
        let config_path = temp_dir.path().join("test-config.toml");
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, content).unwrap();
        
        // Load from file
        let loaded_config = Config::load(Some(&config_path)).unwrap();
        let re_validation = ConfigValidator::new().validate(&loaded_config).unwrap();
        assert!(re_validation.is_valid);
    }
    
    #[tokio::test]
    async fn test_wizard_validation_integration() {
        // Test wizard with validation feedback
        // This would require mocking user input, which is complex
        // Instead, test the validation logic that wizard uses
    }
    
    #[tokio::test]
    async fn test_storage_creation_with_validation() {
        let config = Config::simple(DatabaseType::Memory, PerformanceLevel::Minimal).unwrap();
        let validation = ConfigValidator::new().validate(&config).unwrap();
        
        if validation.is_valid {
            // Should be able to create memory instance
            let memory = config.create_memory().await;
            assert!(memory.is_ok());
        }
    }
}
```

#### 4.3 Error Scenario Tests

```rust
#[cfg(test)]
mod error_scenario_tests {
    use super::*;
    
    #[test]
    fn test_malformed_config_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test invalid TOML
        let invalid_toml = temp_dir.path().join("invalid.toml");
        std::fs::write(&invalid_toml, "[database\ninvalid syntax").unwrap();
        
        let result = Config::load(Some(&invalid_toml));
        assert!(result.is_err());
        
        // Test invalid JSON
        let invalid_json = temp_dir.path().join("invalid.json");
        std::fs::write(&invalid_json, "{ \"database\": { \"turso_url\": invalid }").unwrap();
        
        let result = Config::load(Some(&invalid_json));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validation_with_missing_fields() {
        let mut config = Config::default();
        config.database.turso_url = Some("https://test.turso.io".to_string());
        // Missing turso_token and redb_path
        
        let result = ConfigValidator::new().validate(&config).unwrap();
        
        // Should have warnings about missing token
        assert!(result.warnings.iter().any(|w| 
            w.category == "security" && 
            w.message.contains("without authentication token")
        ));
    }
    
