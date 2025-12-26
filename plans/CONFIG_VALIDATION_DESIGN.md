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
