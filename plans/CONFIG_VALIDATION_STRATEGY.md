# Configuration Validation Strategy

**Goal**: Ensure simplified architecture maintains functionality while improving quality

---

## Validation Framework Design

### 1. Core Validation Principles

**1.1 Layered Validation**
- **Syntax Validation**: File format, structure, required fields
- **Semantic Validation**: Value ranges, dependencies, consistency
- **Business Logic Validation**: Performance implications, security considerations
- **Integration Validation**: Storage connectivity, feature compatibility

**1.2 Contextual Error Messages**
- **What**: Clear description of the problem
- **Where**: Specific location in configuration
- **Why**: Impact and consequences
- **How**: Actionable fix suggestions

**1.3 Progressive Validation**
- **Basic**: Essential checks for system startup
- **Recommended**: Performance and best practice checks
- **Advanced**: Security and optimization suggestions

### 2. Validation Rule Categories

#### 2.1 Database Configuration Validation

```rust
// Required field validation
if config.database.turso_url.is_none() && config.database.redb_path.is_none() {
    errors.push(ValidationError {
        category: "database".to_string(),
        message: "No storage backend configured".to_string(),
        severity: ErrorSeverity::Critical,
        location: "database".to_string(),
        suggestion: Some("Configure at least one storage backend:
  • Set database.turso_url for cloud storage
  • Set database.redb_path for local storage  
  • Use Config::simple() for guided setup".to_string()),
    });
}

// URL format validation
if let Some(url) = &config.database.turso_url {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        errors.push(ValidationError {
            category: "database".to_string(),
            message: "Invalid Turso URL format".to_string(),
            severity: ErrorSeverity::High,
            location: "database.turso_url".to_string(),
            suggestion: Some("URL must start with https:// or http://".to_string()),
        });
    }
}

// Token validation
if config.database.turso_url.is_some() && config.database.turso_token.is_none() {
    warnings.push(ValidationWarning {
        category: "security".to_string(),
        message: "Turso URL configured without authentication token".to_string(),
        location: "database.turso_token".to_string(),
        suggestion: Some("Add turso_token for secure access:
  • Generate token from Turso dashboard
  • Set environment variable: TURSO_TOKEN
  • Add to configuration file".to_string()),
    });
}
```

#### 2.2 Storage Configuration Validation

```rust
// Performance validation
if config.storage.max_episodes_cache < 100 {
    warnings.push(ValidationWarning {
        category: "performance".to_string(),
        message: "Cache size may be too small for optimal performance".to_string(),
        location: "storage.max_episodes_cache".to_string(),
        suggestion: Some("Consider increasing cache size:
  • Minimal: 100 episodes (< 100MB memory)
  • Standard: 1000 episodes (< 1GB memory)
  • High: 10000 episodes (< 4GB memory)".to_string()),
    });
}

if config.storage.max_episodes_cache > 50000 {
    warnings.push(ValidationWarning {
        category: "performance".to_string(),
        message: "Cache size may be too large for available memory".to_string(),
        location: "storage.max_episodes_cache".to_string(),
        suggestion: Some("Consider reducing cache size to prevent memory issues".to_string()),
    });
}

// Pool size validation
if config.storage.pool_size == 0 {
    errors.push(ValidationError {
        category: "storage".to_string(),
        message: "Connection pool size cannot be zero".to_string(),
        severity: ErrorSeverity::Critical,
        location: "storage.pool_size".to_string(),
        suggestion: Some("Set pool_size to at least 1 for basic functionality".to_string()),
    });
}

if config.storage.pool_size > 100 {
    warnings.push(ValidationWarning {
        category: "performance".to_string(),
        message: "Connection pool size is very large".to_string(),
        location: "storage.pool_size".to_string(),
        suggestion: Some("Large pools may not improve performance:
  • Consider 10-20 for most use cases
  • Monitor actual connection usage".to_string()),
    });
}
```

#### 2.3 CLI Configuration Validation

```rust
// Format validation
match config.cli.default_format.as_str() {
    "human" | "json" | "yaml" => {}
    _ => errors.push(ValidationError {
        category: "cli".to_string(),
        message: "Invalid default output format".to_string(),
        severity: ErrorSeverity::Medium,
        location: "cli.default_format".to_string(),
        suggestion: Some("Valid formats: 'human', 'json', 'yaml'".to_string()),
    }),
}

// Batch size validation
if config.cli.batch_size == 0 {
    errors.push(ValidationError {
        category: "cli".to_string(),
        message: "Batch size cannot be zero".to_string(),
        severity: ErrorSeverity::High,
        location: "cli.batch_size".to_string(),
        suggestion: Some("Set batch_size to at least 1".to_string()),
    });
}

if config.cli.batch_size > 10000 {
    warnings.push(ValidationWarning {
        category: "performance".to_string(),
        message: "Very large batch size may cause memory issues".to_string(),
        location: "cli.batch_size".to_string(),
        suggestion: Some("Consider smaller batches (100-1000) for better memory management".to_string()),
    });
}
```

### 3. Validation Implementation

#### 3.1 Validation Engine

```rust
pub struct ConfigValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ConfigValidator {
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(DatabaseValidationRule),
                Box::new(StorageValidationRule),
                Box::new(CliValidationRule),
                Box::new(SecurityValidationRule),
                Box::new(PerformanceValidationRule),
            ],
        }
    }
    
    pub fn validate(&self, config: &Config) -> Result<ValidationReport, ConfigError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        
        for rule in &self.rules {
            let result = rule.validate(config)?;
            errors.extend(result.errors);
            warnings.extend(result.warnings);
            suggestions.extend(result.suggestions);
        }
        
        Ok(ValidationReport {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggestions,
        })
    }
}

pub trait ValidationRule {
    fn validate(&self, config: &Config) -> Result<RuleResult, ConfigError>;
}

pub struct RuleResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}
```

#### 3.2 Individual Validation Rules

```rust
struct DatabaseValidationRule;

impl ValidationRule for DatabaseValidationRule {
    fn validate(&self, config: &Config) -> Result<RuleResult, ConfigError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        
        // Check for at least one storage backend
        if config.database.turso_url.is_none() && config.database.redb_path.is_none() {
            errors.push(ValidationError {
                category: "database".to_string(),
                message: "No storage backend configured".to_string(),
                severity: ErrorSeverity::Critical,
                location: "database".to_string(),
                suggestion: Some("Configure at least one storage backend:
  • Set database.turso_url for cloud storage
  • Set database.redb_path for local storage
  • Use Config::simple() for guided setup".to_string()),
            });
        }
        
        // Validate Turso configuration
        if let Some(url) = &config.database.turso_url {
            if !url.starts_with("https://") && !url.starts_with("http://") {
                errors.push(ValidationError {
                    category: "database".to_string(),
                    message: "Invalid Turso URL format".to_string(),
                    severity: ErrorSeverity::High,
                    location: "database.turso_url".to_string(),
                    suggestion: Some("URL must start with https:// or http://".to_string()),
                });
            }
            
            if url.contains("localhost") || url.contains("127.0.0.1") {
                warnings.push(ValidationWarning {
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
    
    #[test]
    fn test_extreme_values() {
        let mut config = Config::default();
        config.storage.max_episodes_cache = u32::MAX as usize;
        config.storage.pool_size = u32::MAX as usize;
        
        let result = ConfigValidator::new().validate(&config).unwrap();
        
        // Should have warnings about extreme values
        assert!(result.warnings.len() > 0);
    }
}
```

### 5. Performance Validation

#### 5.1 Benchmarking Tests

```rust
#[cfg(test)]
mod performance_validation {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_simple_config_creation(c: &Criterion) {
        c.bench_function("simple_config_creation", |b| {
            b.iter(|| {
                let config = Config::simple(
                    black_box(DatabaseType::Local),
                    black_box(PerformanceLevel::Standard)
                ).unwrap();
                black_box(config);
            })
        });
    }
    
    fn bench_validation_performance(c: &Criterion) {
        let config = Config::default();
        let validator = ConfigValidator::new();
        
        c.bench_function("validation_performance", |b| {
            b.iter(|| {
                let result = validator.validate(black_box(&config)).unwrap();
                black_box(result);
            })
        });
    }
    
    fn bench_config_loading(c: &Criterion) {
        c.bench_function("config_loading", |b| {
            b.iter(|| {
                let config = Config::load(black_box(None)).unwrap();
                black_box(config);
            })
        });
    }
}
```

### 6. User Experience Validation

#### 6.1 Usability Testing

```rust
#[cfg(test)]
mod usability_tests {
    use super::*;
    
    #[test]
    fn test_simple_mode_usability() {
        // Test that simple mode provides reasonable defaults
        let minimal_config = Config::simple(DatabaseType::Local, PerformanceLevel::Minimal).unwrap();
        let standard_config = Config::simple(DatabaseType::Local, PerformanceLevel::Standard).unwrap();
        let high_config = Config::simple(DatabaseType::Local, PerformanceLevel::High).unwrap();
        
        // Verify progression of values
        assert!(minimal_config.storage.max_episodes_cache < standard_config.storage.max_episodes_cache);
        assert!(standard_config.storage.max_episodes_cache < high_config.storage.max_episodes_cache);
        
        assert!(minimal_config.storage.pool_size < standard_config.storage.pool_size);
        assert!(standard_config.storage.pool_size < high_config.storage.pool_size);
    }
    
    #[test]
    fn test_error_message_helpfulness() {
        let mut config = Config::default();
        config.storage.max_episodes_cache = 0;
        
        let result = ConfigValidator::new().validate(&config).unwrap();
        let error = result.errors.first().unwrap();
        
        // Error messages should be helpful
        assert!(error.message.len() > 10);
        assert!(error.suggestion.is_some());
        assert!(error.suggestion.as_ref().unwrap().len() > 10);
    }
    
    #[test]
    fn test_wizard_suggestions() {
        // Test that wizard provides reasonable configuration suggestions
        let temp_dir = TempDir::new().unwrap();
        
        // This would test the wizard logic if we can mock stdin/stdout
        // For now, test the underlying configuration generation
    }
}
```

### 7. Success Criteria

#### 7.1 Functional Criteria
- [ ] All validation rules work correctly
- [ ] Error messages are helpful and actionable
- [ ] Simple Mode configurations are always valid
- [ ] Wizard generates valid configurations
- [ ] No regression in existing functionality

#### 7.2 Performance Criteria
- [ ] Configuration loading < 100ms
- [ ] Validation < 50ms for typical configurations
- [ ] Simple Mode setup < 10ms
- [ ] Memory usage not increased significantly

#### 7.3 Quality Criteria
- [ ] Test coverage > 90%
- [ ] All tests pass consistently
- [ ] No flaky tests
- [ ] Clear, documented APIs

#### 7.4 User Experience Criteria
- [ ] Error messages understandable to non-technical users
- [ ] Simple Mode covers 80% of use cases
- [ ] Wizard completes setup in < 5 minutes
- [ ] Migration from old API is seamless

---

**Validation Status**: Framework designed and ready for implementation
**Testing Approach**: Comprehensive unit, integration, and performance testing
**Quality Assurance**: Multiple validation layers with user experience focus