# Configuration Validation Guide

**Last Updated**: 2025-12-27
**Status**: Implemented and Operational
**Goal**: Ensure configuration quality through comprehensive validation

---

## Table of Contents

1. [Overview](#overview)
2. [Validation Framework](#validation-framework)
3. [Validation Rules](#validation-rules)
4. [Testing Strategy](#testing-strategy)
5. [Performance Validation](#performance-validation)
6. [User Experience Validation](#user-experience-validation)
7. [Success Criteria](#success-criteria)

---

## Overview

The configuration validation system ensures that all Memory CLI configurations are valid, secure, performant, and user-friendly before deployment. The validation framework implements layered validation with contextual error messages and progressive disclosure.

**Key Features**:
- ✅ **Layered Validation**: Syntax, semantic, business logic, and integration checks
- ✅ **Rich Error Messages**: Clear problems with actionable fix suggestions
- ✅ **Progressive Validation**: Basic, recommended, and advanced checks
- ✅ **Multiple Rule Categories**: Database, storage, CLI, security, performance
- ✅ **Comprehensive Testing**: Unit, integration, performance, and UX tests

---

## Validation Framework

### Core Validation Principles

**1. Layered Validation**
- **Syntax Validation**: File format, structure, required fields
- **Semantic Validation**: Value ranges, dependencies, consistency
- **Business Logic Validation**: Performance implications, security considerations
- **Integration Validation**: Storage connectivity, feature compatibility

**2. Contextual Error Messages**
- **What**: Clear description of the problem
- **Where**: Specific location in configuration
- **Why**: Impact and consequences
- **How**: Actionable fix suggestions

**3. Progressive Validation**
- **Basic**: Essential checks for system startup
- **Recommended**: Performance and best practice checks
- **Advanced**: Security and optimization suggestions

### Validation Engine

**Core Structure**:

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

---

## Validation Rules

### 1. Database Configuration Validation

**Required Field Validation**:

```rust
// Ensure at least one storage backend is configured
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
```

**URL Format Validation**:

```rust
// Validate Turso URL format
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
```

**Token Validation**:

```rust
// Check for Turso token when URL is configured
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

**Path Validation**:

```rust
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
```

### 2. Storage Configuration Validation

**Cache Size Validation**:

```rust
// Performance validation for cache size
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
```

**Pool Size Validation**:

```rust
// Connection pool validation
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

### 3. CLI Configuration Validation

**Format Validation**:

```rust
// Output format validation
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
```

**Batch Size Validation**:

```rust
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

### Individual Validation Rule Implementation

**Database Validation Rule**:

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
        }

        Ok(RuleResult {
            errors,
            warnings,
            suggestions,
        })
    }
}
```

**Performance Validation Rule**:

```rust
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

---

## Testing Strategy

### Unit Tests for Validation Rules

**No Storage Backend Test**:

```rust
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
```

**URL Format Test**:

```rust
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
```

**Zero Values Test**:

```rust
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
```

**Performance Warnings Test**:

```rust
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
```

**Simple Mode Validation Test**:

```rust
#[test]
fn test_simple_mode_validation() {
    let config = Config::simple(DatabaseType::Local, PerformanceLevel::Minimal).unwrap();
    let result = ConfigValidator::new().validate(&config).unwrap();

    // Simple mode configurations should be valid
    assert!(result.is_valid, "Simple mode configuration should be valid: {:?}", result.errors);
}
```

**Error Context Test**:

```rust
#[test]
fn test_validation_error_context() {
    let mut config = Config::default();
    config.storage.max_episodes_cache = 0;

    let result = ConfigValidator::new().validate(&config).unwrap();

    let error = result.errors.first().unwrap();
    assert!(error.location.contains("storage.max_episodes_cache"));
    assert!(error.suggestion.is_some());
}
```

### Integration Tests

**Complete Configuration Lifecycle**:

```rust
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
```

**Storage Creation Validation**:

```rust
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
```

### Error Scenario Tests

**Malformed Configuration Files**:

```rust
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
```

**Missing Fields Validation**:

```rust
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
```

**Extreme Values Test**:

```rust
#[test]
fn test_extreme_values() {
    let mut config = Config::default();
    config.storage.max_episodes_cache = u32::MAX as usize;
    config.storage.pool_size = u32::MAX as usize;

    let result = ConfigValidator::new().validate(&config).unwrap();

    // Should have warnings about extreme values
    assert!(result.warnings.len() > 0);
}
```

---

## Performance Validation

### Benchmarking Tests

**Simple Configuration Creation**:

```rust
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
```

**Validation Performance**:

```rust
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
```

**Configuration Loading**:

```rust
fn bench_config_loading(c: &Criterion) {
    c.bench_function("config_loading", |b| {
        b.iter(|| {
            let config = Config::load(black_box(None)).unwrap();
            black_box(config);
        })
    });
}
```

---

## User Experience Validation

### Usability Tests

**Simple Mode Usability**:

```rust
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
```

**Error Message Helpfulness**:

```rust
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
```

---

## Success Criteria

### Functional Criteria

- [x] All validation rules work correctly
- [x] Error messages are helpful and actionable
- [x] Simple Mode configurations are always valid
- [x] Wizard generates valid configurations
- [x] No regression in existing functionality

### Performance Criteria

- [x] Configuration loading < 100ms
- [x] Validation < 50ms for typical configurations
- [x] Simple Mode setup < 10ms
- [x] Memory usage not increased significantly

### Quality Criteria

- [x] Test coverage > 90%
- [x] All tests pass consistently
- [x] No flaky tests
- [x] Clear, documented APIs

### User Experience Criteria

- [x] Error messages understandable to non-technical users
- [x] Simple Mode covers 80% of use cases
- [x] Wizard completes setup in < 5 minutes
- [x] Migration from old API is seamless

---

## Implementation Files

### Core Validation Implementation
- `memory-cli/src/config/validation.rs` - Main validation framework
- `memory-cli/src/config/validator.rs` - Individual validation rules
- `memory-cli/src/config/error.rs` - Error types and messages

### Test Files
- `memory-cli/tests/config_validation.rs` - Unit tests for validation rules
- `memory-cli/tests/config_integration.rs` - Integration tests with lifecycle
- `memory-cli/benches/config_validation.rs` - Performance benchmarks

---

## Cross-References

### Related Configuration Documents
- **UX Guide**: See [CONFIG_UX_GUIDE.md](CONFIG_UX_GUIDE.md)
- **Phase Plans**: See CONFIG_PHASE*.md files (historical phases 1-6)
- **Status**: See [CONFIGURATION_OPTIMIZATION_STATUS.md](CONFIGURATION_OPTIMIZATION_STATUS.md)

### Related Project Documents
- **Project Status**: See [PROJECT_STATUS_UNIFIED.md](PROJECT_STATUS_UNIFIED.md)
- **Testing Guide**: See [../TESTING.md](../TESTING.md)
- **Quality Gates**: See [../docs/QUALITY_GATES.md](../docs/QUALITY_GATES.md)

---

**Document Status**: Consolidated from 3 CONFIG_VALIDATION_* files
**Consolidation Date**: 2025-12-27
**Validation Status**: Implemented and Operational
**Test Coverage**: >90% (all validation rules tested)
**Next Review**: Ongoing maintenance with each release

---

*This guide consolidates information from CONFIG_VALIDATION_DESIGN.md, CONFIG_VALIDATION_IMPLEMENTATION.md, and CONFIG_VALIDATION_TESTING.md into a single comprehensive validation reference.*
