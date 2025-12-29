# Turso AI Extension Compatibility Test Matrix

**Date**: 2025-12-29
**Agent**: testing-qa
**Purpose**: Compatibility matrix and test plan for SQLite extensions

## Overview

Turso preloads several SQLite extensions that can enhance functionality:
1. **JSON Functions**: `json_extract`, `json_group_array`, etc.
2. **SQLean Stats**: Statistical functions (`mean`, `median`, `stddev`)
3. **SQLean Crypto**: Cryptographic functions (`sha256`, `hmac`)
4. **SQLean UUID**: UUID generation and validation

This document defines the compatibility matrix and test requirements.

## Extension Availability Matrix

| Extension | Function | Turso Version | Required | Fallback |
|-----------|----------|---------------|----------|----------|
| JSON | `json_extract`, `json_group_array` | All | Yes | Rust serde_json |
| JSON | `json_each`, `json_tree` | All | Optional | Rust iteration |
| Stats | `mean`, `median`, `stddev` | All | Optional | Rust calculations |
| Stats | `percentile`, `variance` | All | Optional | Rust calculations |
| Crypto | `sha256`, `hmac` | All | Optional | Rust ring/sha2 |
| Crypto | `aes_encrypt`, `aes_decrypt` | All | Optional | Rust aes-gcm |
| UUID | `uuid()`, `uuid_str()` | All | Optional | Rust uuid crate |
| UUID | `uuid_blob()` | All | Optional | Rust uuid crate |

**Required**: Extension must be available in all Turso deployments
**Optional**: Extension may not be available, require graceful fallback

## Test Categories

### 1. JSON Functions Compatibility Tests

**Test File**: `memory-storage-turso/tests/json_extension_tests.rs`

**Test Cases**:
- `test_json_extract_basic`: Extract scalar values from JSON metadata
- `test_json_extract_nested`: Extract nested objects and arrays
- `test_json_each_iteration`: Iterate over JSON arrays using `json_each`
- `test_json_group_array_aggregation`: Aggregate values into JSON array
- `test_json_malformed_handling`: Graceful handling of invalid JSON
- `test_json_performance_vs_rust`: Compare SQL JSON vs Rust deserialization

**Implementation Details**:
```rust
// Example test using json_extract
async fn test_json_extract_basic() {
    let (storage, _dir) = create_test_storage().await.unwrap();
    
    // Store episode with metadata
    let mut episode = create_test_episode("Test");
    episode.metadata = serde_json::json!({
        "priority": "high",
        "tags": ["urgent", "important"]
    });
    
    storage.store_episode(&episode).await.unwrap();
    
    // Query using json_extract
    let conn = storage.get_connection().await.unwrap();
    let sql = "SELECT json_extract(metadata, '$.priority') FROM episodes WHERE episode_id = ?";
    let mut rows = conn.query(sql, params![episode.episode_id.to_string()]).await.unwrap();
    
    // Verify result
    if let Some(row) = rows.next().await.unwrap() {
        let priority: String = row.get(0).unwrap();
        assert_eq!(priority, "high");
    } else {
        panic!("No rows returned");
    }
}
```

### 2. SQLean Stats Extension Tests

**Test File**: `memory-storage-turso/tests/stats_extension_tests.rs`

**Test Cases**:
- `test_stats_mean_median`: Calculate mean and median of success rates
- `test_stats_stddev_variance`: Standard deviation and variance of rewards
- `test_stats_percentile`: Percentile calculations for latencies
- `test_stats_group_by_domain`: Statistical analysis by domain
- `test_stats_fallback_when_missing`: Graceful fallback to Rust calculations

**Implementation Details**:
Use feature flag `sqlite_extensions` to conditionally enable stats tests.

### 3. Crypto/UUID Extension Tests

**Test File**: `memory-storage-turso/tests/crypto_uuid_tests.rs`

**Test Cases**:
- `test_crypto_sha256_hashing`: SHA256 hash generation
- `test_crypto_hmac_authentication`: HMAC generation and verification
- `test_uuid_generation`: Generate UUIDs in SQL
- `test_uuid_validation`: Validate UUID format
- `test_crypto_fallback_when_missing`: Fallback to Rust implementations

### 4. Feature Flag Integration Tests

**Test File**: `memory-storage-turso/tests/feature_flag_tests.rs`

**Test Cases**:
- `test_feature_flag_json_disabled`: JSON functions unavailable when feature disabled
- `test_feature_flag_stats_disabled`: Stats functions unavailable
- `test_feature_flag_graceful_fallback`: Code falls back to Rust implementations
- `test_feature_flag_compilation`: Verify all feature combinations compile

## Feature Flag Design

### Cargo Features
```toml
[features]
default = []
turso_json = []  # Enable JSON extension usage
turso_stats = [] # Enable Stats extension usage  
turso_crypto = [] # Enable Crypto extension usage
turso_uuid = []  # Enable UUID extension usage
sqlite_extensions = ["turso_json", "turso_stats", "turso_crypto", "turso_uuid"]
```

### Conditional Compilation
```rust
#[cfg(feature = "turso_json")]
pub async fn query_with_json_extract(&self, episode_id: Uuid, path: &str) -> Result<String> {
    // Use json_extract in SQL
}

#[cfg(not(feature = "turso_json"))]
pub async fn query_with_json_extract(&self, episode_id: Uuid, path: &str) -> Result<String> {
    // Deserialize JSON in Rust and extract
}
```

## Test Utilities

### Extension Availability Detector
```rust
pub struct ExtensionTester {
    conn: libsql::Connection,
}

impl ExtensionTester {
    pub async fn new(storage: &TursoStorage) -> Result<Self> {
        let conn = storage.get_connection().await?;
        Ok(Self { conn })
    }
    
    pub async fn test_json_functions(&self) -> bool {
        let result = self.conn.query("SELECT json_extract('{\"a\":1}', '$.a')", ()).await;
        result.is_ok()
    }
    
    pub async fn test_stats_functions(&self) -> bool {
        let result = self.conn.query("SELECT mean(1,2,3)", ()).await;
        result.is_ok()
    }
    
    pub async fn test_crypto_functions(&self) -> bool {
        let result = self.conn.query("SELECT sha256('test')", ()).await;
        result.is_ok()
    }
    
    pub async fn test_uuid_functions(&self) -> bool {
        let result = self.conn.query("SELECT uuid()", ()).await;
        result.is_ok()
    }
}
```

### Test Data Generator
```rust
pub struct ExtensionTestData {
    episodes_with_metadata: Vec<Episode>,
    episodes_with_rewards: Vec<Episode>,
}

impl ExtensionTestData {
    pub fn generate_for_json_tests(count: usize) -> Self {
        // Generate episodes with complex metadata
    }
    
    pub fn generate_for_stats_tests(count: usize) -> Self {
        // Generate episodes with varying success rates and rewards
    }
}
```

## Compatibility Validation Procedure

### 1. Pre-implementation Validation
- Check Turso documentation for extension availability
- Verify extension functions with test queries
- Document any version-specific limitations

### 2. Implementation Validation
- Write tests for each extension function
- Verify both success and failure paths
- Test edge cases and error handling

### 3. Post-implementation Validation
- Run tests with feature flags enabled/disabled
- Verify graceful fallback when extensions unavailable
- Performance comparison against Rust implementations

## Risk Matrix

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Extension unavailable | Low | High | Feature flags, graceful fallback |
| Function signature changes | Low | Medium | Version detection, compatibility layer |
| Performance regression | Medium | Low | Benchmark comparison, optional usage |
| Security vulnerabilities | Low | High | Input validation, parameterized queries |

## Test Execution Matrix

| Test Category | Feature Flag | CI Run | Local Development |
|---------------|--------------|--------|-------------------|
| JSON Functions | `turso_json` | Always | Optional |
| Stats Functions | `turso_stats` | Weekly | Optional |
| Crypto Functions | `turso_crypto` | Weekly | Optional |
| UUID Functions | `turso_uuid` | Always | Optional |
| Feature Flags | All combinations | PRs | Always |

**CI Run**: Frequency in continuous integration
**Local Development**: Recommended for developers

## Integration with Quality Gates

### New Quality Gates
Add to `tests/quality_gates.rs`:
- `quality_gate_extension_compatibility`: Verify extensions work correctly
- `quality_gate_feature_flag_compilation`: All feature combinations compile
- `quality_gate_extension_fallback`: Graceful fallback when extensions unavailable

### Quality Gate Configuration
```bash
# Enable extension testing in CI
export QUALITY_GATE_EXTENSION_TESTING=true

# Set extension availability expectations
export EXPECT_JSON_EXTENSIONS=true
export EXPECT_STATS_EXTENSIONS=false  # Optional
```

## Implementation Timeline

### Phase 0 (Preparation)
- [x] Research extension availability
- [ ] Design compatibility matrix
- [ ] Create test plan

### Phase 1 (JSON Functions)
- [ ] Implement JSON function tests
- [ ] Add feature flag `turso_json`
- [ ] Integrate with quality gates

### Phase 2 (Stats Functions)
- [ ] Implement stats function tests
- [ ] Add feature flag `turso_stats`
- [ ] Performance comparison

### Phase 3 (Crypto/UUID Functions)
- [ ] Implement crypto/uuid tests
- [ ] Add feature flags `turso_crypto`, `turso_uuid`
- [ ] Security validation

### Phase 4 (Integration)
- [ ] Combine all extensions
- [ ] Test feature flag combinations
- [ ] Update documentation

## Deliverables

1. **Test Files**: Complete test suite for all extensions
2. **Feature Flags**: Cargo feature configuration
3. **Quality Gates**: Automated extension validation
4. **Documentation**: Extension usage guide
5. **Compatibility Matrix**: This document

## Success Criteria

### Quantitative
- [ ] 100% test coverage for extension functionality
- [ ] 0 false positives in compatibility detection
- [ ] <10% performance overhead for fallback implementations
- [ ] All feature flag combinations compile successfully

### Qualitative
- [ ] Clear error messages when extensions unavailable
- [ ] Intuitive feature flag configuration
- [ ] Comprehensive documentation for extension usage
- [ ] Easy addition of new extension tests

---

*Extension Compatibility Test Matrix v1.0*
*Created by testing-qa agent on 2025-12-29*