# Local Turso Database Setup & CI Integration Implementation Plan

## Executive Summary

This document provides a comprehensive implementation plan for setting up local Turso database usage and CI integration. The plan leverages existing infrastructure while adding new capabilities for seamless local development, testing, and production deployment workflows.

**Current State Analysis:**
- ✅ `memory-storage-turso` crate with full libSQL/Turso integration
- ✅ Existing `setup-local-db.sh` script for basic local setup
- ✅ Comprehensive CI workflows in `.github/workflows/`
- ✅ Configuration files (`memory-cli.toml`, `.env`)
- ✅ Documentation for local database setup
- ✅ Test infrastructure with temporary databases

**Target State:**
- 🚀 Enhanced local development with isolated Turso instances
- 🚀 CI/CD integration with local Turso testing
- 🚀 Environment-specific database management
- 🚀 Automated migration and deployment workflows
- 🚀 Comprehensive monitoring and maintenance procedures

---

## 1. Local Development Setup

### 1.1 Environment Configuration Enhancement

**Current State:** Basic `.env` file with empty Turso variables
**Target State:** Sophisticated environment management with multiple profiles

#### Implementation Steps:

1. **Create environment profile templates**
```bash
# Create new files:
.env.development    # Local development with file-based databases
.env.test           # CI/testing with ephemeral databases  
.env.staging        # Staging environment configuration
.env.production     # Production environment (secure)
```

2. **Enhance memory-cli.toml with environment profiles**
```toml
# Add profile-specific configurations
[profiles.development]
database_url = "file:./data/memory-dev.db"
pool_size = 5
cache_size = 500

[profiles.testing] 
database_url = ":memory:"
pool_size = 2
cache_size = 100

[profiles.ci]
database_url = "file:./ci-test-data/memory-ci.db"
pool_size = 10
cache_size = 200
```

3. **Create environment selection script**
```bash
# scripts/select-environment.sh
#!/bin/bash
ENV=${1:-development}
cp ".env.$ENV" .env
echo "Switched to $ENV environment"
```

### 1.2 Local Turso Database Management

**Current State:** Basic file-based SQLite setup
**Target State:** Full local Turso server with multiple database instances

#### Implementation Steps:

1. **Install and configure local Turso server**
```bash
# scripts/setup-local-turso.sh
#!/bin/bash
set -euo pipefail

# Install libsql server
if ! command -v libsql-server &> /dev/null; then
    echo "Installing libsql server..."
    curl -L https://github.com/tursodatabase/libsql/releases/latest/download/libsql-server-x86_64-linux-musl.tar.gz | tar xz
    sudo mv libsql-server /usr/local/bin/
fi

# Create database directory structure
mkdir -p ./turso-data/{dev,test,staging}

# Start development server on port 8080
nohup libsql-server \
    --port 8080 \
    --database ./turso-data/dev/memory.db \
    --enable-wal \
    --max-connections 100 \
    > ./logs/turso-dev.log 2>&1 &

echo "Local Turso server started on port 8080"
```

2. **Create database initialization scripts**
```bash
# scripts/init-turso-database.sh
#!/bin/bash
set -euo pipefail

ENV=${1:-development}
PORT=${2:-8080}
DB_PATH="./turso-data/$ENV/memory.db"

# Create environment-specific database
sqlite3 "$DB_PATH" << 'EOF'
-- Initialize with schema from memory-storage-turso
-- Tables: episodes, patterns, heuristics, execution_records, agent_metrics
EOF

echo "Database initialized for $ENV environment"
```

3. **Environment isolation system**
```bash
# scripts/manage-environments.sh
#!/bin/bash

case "${1:-list}" in
    list)
        echo "Available environments:"
        ls -la ./turso-data/
        ;;
    start)
        ENV=${2:-development}
        echo "Starting $ENV environment..."
        # Start server with specific database
        ;;
    stop)
        # Stop specific environment server
        ;;
    reset)
        ENV=${2:-development}
        echo "Resetting $ENV environment..."
        rm -f ./turso-data/$ENV/memory.db*
        # Reinitialize
        ;;
esac
```

### 1.3 Development vs Production Database Separation

**Current State:** Single configuration file
**Target State:** Clear separation with automatic environment detection

#### Implementation Steps:

1. **Environment auto-detection**
```rust
// memory-core/src/config/environment.rs
pub enum Environment {
    Development,
    Testing, 
    Staging,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match std::env::var("MEMORY_ENVIRONMENT").as_deref() {
            Ok("test") => Environment::Testing,
            Ok("staging") => Environment::Staging, 
            Ok("prod") => Environment::Production,
            _ => Environment::Development,
        }
    }
}
```

2. **Configuration loading with environment fallbacks**
```rust
// Enhanced configuration loading
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub max_retries: u32,
    pub enable_pooling: bool,
}

impl DatabaseConfig {
    pub fn for_environment(env: &Environment) -> Self {
        match env {
            Environment::Development => DatabaseConfig {
                url: "file:./data/memory-dev.db".to_string(),
                pool_size: 5,
                max_retries: 2,
                enable_pooling: false,
            },
            Environment::Testing => DatabaseConfig {
                url: ":memory:".to_string(),
                pool_size: 2,
                max_retries: 1,
                enable_pooling: false,
            },
            Environment::CI => DatabaseConfig {
                url: "file:./ci-test-data/memory-ci.db".to_string(),
                pool_size: 10,
                max_retries: 3,
                enable_pooling: true,
            },
            // Production configuration...
        }
    }
}
```

---

## 2. CI/CD Integration

### 2.1 GitHub Actions Workflow Modifications

**Current State:** Basic CI workflow with format, clippy, test, and coverage
**Target State:** Enhanced CI with local Turso testing matrix

#### Implementation Steps:

1. **Add Turso-specific CI jobs**

Add to `.github/workflows/ci.yml`:

```yaml
turso-integration:
  name: Turso Integration Tests
  runs-on: ubuntu-latest
  needs: [ci-guard]
  if: needs.ci-guard.outputs.should-run == 'true'
  strategy:
    matrix:
      database: ["local-file", "local-turso", "memory-db"]
  env:
    CARGO_TARGET_DIR: /tmp/target
  steps:
    - uses: actions/checkout@v6
    
    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    
    - name: Install libsql-server
      run: |
        curl -L https://github.com/tursodatabase/libsql/releases/latest/download/libsql-server-x86_64-linux-musl.tar.gz | tar xz
        sudo mv libsql-server /usr/local/bin/
    
    - name: Setup test database
      run: |
        case "${{ matrix.database }}" in
          "local-file")
            echo "DATABASE_URL=file:./test-data/memory.db" >> $GITHUB_ENV
            mkdir -p test-data
            ;;
          "local-turso")
            echo "DATABASE_URL=libsql://localhost:8080/test" >> $GITHUB_ENV
            echo "TURSO_AUTH_TOKEN=" >> $GITHUB_ENV
            # Start local Turso server
            nohup libsql-server --port 8080 --database ./test-data/turso.db --enable-wal > /tmp/turso.log 2>&1 &
            sleep 5
            ;;
          "memory-db")
            # Use in-memory database for fast tests
            echo "DATABASE_URL=:memory:" >> $GITHUB_ENV
            ;;
        esac
    
    - name: Run Turso integration tests
      run: |
        cargo test -p memory-storage-turso --features integration-tests
      env:
        RUST_LOG: debug
        DATABASE_URL: ${{ env.DATABASE_URL }}
    
    - name: Test database connectivity
      run: |
        cargo run --bin memory-cli -- config show
        cargo run --bin memory-cli -- episode test-connection
```

2. **Enhanced test matrix with database backends**

```yaml
test-matrix:
  name: Test Matrix (${{ matrix.os }}, ${{ matrix.rust }}, ${{ matrix.database }})
  runs-on: ${{ matrix.os }}
  needs: [ci-guard]
  if: needs.ci-guard.outputs.should-run == 'true'
  strategy:
    fail-fast: false
    matrix:
      os: [ubuntu-latest, macos-latest]
      rust: [stable, beta]
      database: [file-sqlite, turso-local, memory-db]
      include:
        - os: ubuntu-latest
          rust: stable
          database: turso-cloud
          turso_url: ${{ secrets.TURSO_TEST_DATABASE_URL }}
          turso_token: ${{ secrets.TURSO_TEST_AUTH_TOKEN }}
  env:
    CARGO_TARGET_DIR: /tmp/target
  steps:
    # Matrix-specific setup and testing
```

### 2.2 Local Turso DB Setup in CI Pipelines

**Current State:** Basic test execution without database context
**Target State:** Sophisticated database setup with multiple backends

#### Implementation Steps:

1. **Database setup matrix job**

```yaml
setup-databases:
  name: Setup Test Databases
  runs-on: ubuntu-latest
  needs: [ci-guard]
  if: needs.ci-guard.outputs.should-run == 'true'
  outputs:
    database-urls: ${{ steps.setup.outputs.database-urls }}
  steps:
    - uses: actions/checkout@v6
    
    - name: Install dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y sqlite3 curl
    
    - name: Setup local Turso server
      run: |
        curl -L https://github.com/tursodatabase/libsql/releases/latest/download/libsql-server-x86_64-linux-musl.tar.gz | tar xz
        sudo mv libsql-server /usr/local/bin/
        
        # Start multiple instances for different test scenarios
        for port in 8080 8081 8082; do
          nohup libsql-server \
            --port $port \
            --database ./test-data/db-$port.db \
            --enable-wal \
            > /tmp/turso-$port.log 2>&1 &
        done
        
        # Wait for servers to be ready
        for port in 8080 8081 8082; do
          timeout 30 bash -c "until nc -z localhost $port; do sleep 1; done"
        done
    
    - name: Initialize test databases
      run: |
        mkdir -p test-data
        
        # Create file-based databases
        sqlite3 test-data/file-db.db "VACUUM;"
        
        # Create Turso databases via HTTP API
        for port in 8080 8081 8082; do
          curl -X POST "http://localhost:$port" -d "CREATE TABLE test (id INTEGER PRIMARY KEY);"
        done
        
        echo "Database URLs:"
        echo "file-sqlite: file:./test-data/file-db.db"
        echo "turso-8080: libsql://localhost:8080"
        echo "turso-8081: libsql://localhost:8081" 
        echo "turso-8082: libsql://localhost:8082"
        echo "memory-db: :memory:"
    
    - name: Output database URLs
      id: setup
      run: |
        echo "database-urls<<EOF" >> $GITHUB_OUTPUT
        echo "{\"file\": \"file:./test-data/file-db.db\", \"turso\": \"libsql://localhost:8080\", \"memory\": \":memory:\"}" >> $GITHUB_OUTPUT
        echo "EOF" >> $GITHUB_OUTPUT
```

2. **Test execution with database context**

```yaml
execute-tests:
  name: Execute Tests (${{ matrix.test-type }})
  runs-on: ubuntu-latest
  needs: [setup-databases]
  strategy:
    matrix:
      test-type: [unit, integration, performance, security]
      database: [file, turso, memory]
  steps:
    - uses: actions/checkout@v6
    
    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Run ${{ matrix.test-type }} tests
      run: |
        case "${{ matrix.test-type }}" in
          "unit")
            cargo test --lib -- --test-threads=1
            ;;
          "integration") 
            cargo test --test integration -- --test-threads=1
            ;;
          "performance")
            cargo test --bench performance -- --bench
            ;;
          "security")
            cargo test --test security -- --test-threads=1
            ;;
        esac
      env:
        DATABASE_URL: ${{ needs.setup-databases.outputs.database-urls[${{ matrix.database }}] }}
```

### 2.3 Database Migration Handling in CI

**Current State:** Manual schema initialization
**Target State:** Automated migration system

#### Implementation Steps:

1. **Migration script for CI**

```yaml
# Add to CI workflow
migrations:
  name: Database Migrations
  runs-on: ubuntu-latest
  needs: [ci-guard]
  if: needs.ci-guard.outputs.should-run == 'true'
  steps:
    - uses: actions/checkout@v6
    
    - name: Setup migration environment
      run: |
        cargo install sqlx-cli --no-default-features --features rustls,sqlite
        mkdir -p migrations
    
    - name: Run database migrations
      run: |
        # Generate migration if schema changed
        if [ -n "$(git diff --name-only HEAD~1 HEAD | grep -E '\.(sql|rs)$')" ]; then
          echo "Schema changes detected, generating migration..."
          cargo sqlx migrate add -r init
        fi
        
        # Apply migrations to test database
        DATABASE_URL="file:./test-data/migration-test.db" cargo sqlx migrate run
```

2. **Migration validation in CI**

```yaml
migration-validation:
  name: Migration Validation
  runs-on: ubuntu-latest
  needs: [migrations]
  steps:
    - uses: actions/checkout@v6
    
    - name: Validate schema compatibility
      run: |
        # Check if migration is reversible
        cargo run --bin validate-migrations
        
        # Test schema upgrade/downgrade
        DATABASE_URL="file:./test-data/validation.db" cargo run --bin test-migrations
        
        # Verify data integrity after migration
        cargo run --bin verify-data-integrity
```

### 2.4 Test Database Provisioning

**Current State:** Basic test database creation
**Target State:** Sophisticated test database lifecycle management

#### Implementation Steps:

1. **Test database lifecycle management**

```bash
# scripts/ci-test-db.sh
#!/bin/bash
set -euo pipefail

ENVIRONMENT=${1:-ci}
ACTION=${2:-create}

case "$ACTION" in
    create)
        echo "Creating test database for $ENVIRONMENT..."
        
        case "$ENVIRONMENT" in
            "ci")
                # Create CI-specific test database
                mkdir -p ci-test-data
                DATABASE_URL="file:./ci-test-data/memory-ci.db" \
                cargo run --bin setup-test-db
                ;;
            "integration")
                # Create integration test database
                DATABASE_URL="file:./test-data/integration.db" \
                cargo run --bin setup-test-db --integration
                ;;
            "performance")
                # Create performance test database with optimized settings
                DATABASE_URL="file:./test-data/performance.db" \
                cargo run --bin setup-test-db --performance
                ;;
        esac
        ;;
        
    cleanup)
        echo "Cleaning up test databases..."
        rm -rf ci-test-data test-data/*.db
        ;;
        
    seed)
        echo "Seeding test database with sample data..."
        DATABASE_URL="file:./test-data/seeded.db" \
        cargo run --bin seed-test-db --count 1000
        ;;
esac
```

2. **Database seeding for tests**

```yaml
# Add to CI workflow
seed-test-data:
  name: Seed Test Data
  runs-on: ubuntu-latest
  needs: [setup-databases]
  steps:
    - uses: actions/checkout@v6
    
    - name: Generate test data
      run: |
        # Generate realistic test episodes
        cargo run --bin generate-test-data \
          --episodes 1000 \
          --patterns 100 \
          --output ./test-data/sample-data.json
        
        # Load test data into databases
        for db_type in file turso memory; do
          DATABASE_URL="${{ needs.setup-databases.outputs.database-urls[$db_type] }}" \
          cargo run --bin load-test-data --input ./test-data/sample-data.json
        done
```

---

## 3. Configuration Management

### 3.1 Turso Connection Configuration

**Current State:** Basic URL and token configuration
**Target State:** Comprehensive connection management with multiple strategies

#### Implementation Steps:

1. **Enhanced connection configuration**

```rust
// memory-storage-turso/src/config.rs
#[derive(Debug, Clone)]
pub struct TursoConnectionConfig {
    pub url: String,
    pub auth_token: Option<String>,
    pub connection_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub max_retries: u32,
    pub retry_backoff: Duration,
    pub enable_tls: bool,
    pub verify_ssl: bool,
}

impl TursoConnectionConfig {
    pub fn from_environment() -> Result<Self> {
        let url = std::env::var("TURSO_DATABASE_URL")
            .map_err(|_| Error::Configuration("TURSO_DATABASE_URL not set".into()))?;
            
        let auth_token = std::env::var("TURSO_AUTH_TOKEN").ok();
        
        Ok(Self {
            url,
            auth_token,
            connection_timeout: Duration::from_secs(
                std::env::var("TURSO_CONNECTION_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?
            ),
            read_timeout: Duration::from_secs(60),
            write_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_backoff: Duration::from_millis(100),
            enable_tls: true,
            verify_ssl: true,
        })
    }
}
```

2. **Connection validation and health checks**

```rust
// Enhanced health check system
impl TursoStorage {
    pub async fn validate_connection(&self) -> Result<ConnectionValidation> {
        let start = Instant::now();
        
        // Basic connectivity test
        let health = self.health_check().await?;
        
        // Test read operation
        let read_test = self.test_read_operation().await?;
        
        // Test write operation  
        let write_test = self.test_write_operation().await?;
        
        let duration = start.elapsed();
        
        Ok(ConnectionValidation {
            is_healthy: health && read_test && write_test,
            latency: duration,
            timestamp: chrono::Utc::now(),
            details: ConnectionDetails {
                read_operational: read_test,
                write_operational: write_test,
                connection_pool_status: self.pool_status().await,
            },
        })
    }
}
```

### 3.2 Environment-Specific Settings

**Current State:** Single configuration file
**Target State:** Hierarchical configuration with environment overrides

#### Implementation Steps:

1. **Hierarchical configuration loading**

```rust
// memory-core/src/config/hierarchical.rs
pub struct HierarchicalConfig {
    base_config: BaseConfig,
    environment_overrides: HashMap<String, BaseConfig>,
    current_env: Environment,
}

impl HierarchicalConfig {
    pub fn load() -> Result<Self> {
        let current_env = Environment::from_env();
        
        // Load base configuration
        let base_config = BaseConfig::load_from_file("memory-cli.toml")?;
        
        // Load environment-specific overrides
        let mut environment_overrides = HashMap::new();
        for env in [Environment::Development, Environment::Testing, 
                   Environment::Staging, Environment::Production] {
            let override_file = format!("memory-cli.{}.toml", env.as_str());
            if let Ok(override_config) = BaseConfig::load_from_file(&override_file) {
                environment_overrides.insert(env.as_str().to_string(), override_config);
            }
        }
        
        Ok(Self {
            base_config,
            environment_overrides,
            current_env,
        })
    }
    
    pub fn get_database_config(&self) -> DatabaseConfig {
        let mut config = self.base_config.database.clone();
        
        // Apply environment-specific overrides
        if let Some(override_config) = self.environment_overrides.get(self.current_env.as_str()) {
            config.merge_with(&override_config.database);
        }
        
        config
    }
}
```

2. **Environment-specific configuration files**

Create environment-specific configuration files:

```toml
# memory-cli.development.toml
[database]
url = "file:./data/memory-dev.db"
pool_size = 5
max_retries = 2

[logging]
level = "debug"
max_log_files = 3

[monitoring]
enabled = true
health_check_interval = 10

[backup]
backup_dir = "./backups/dev"
max_backup_age_days = 7
```

```toml
# memory-cli.testing.toml
[database]
url = ":memory:"
pool_size = 2
max_retries = 1

[logging]
level = "warn"
max_log_files = 1

[monitoring]
enabled = false

[backup]
enabled = false
```

### 3.3 Secret Management for CI

**Current State:** Basic environment variables
**Target State:** Comprehensive secret management with GitHub Secrets integration

#### Implementation Steps:

1. **GitHub Secrets integration**

```yaml
# .github/workflows/secrets-management.yml
name: Manage Database Secrets
on:
  workflow_dispatch:
    inputs:
      environment:
        description: 'Environment to manage'
        required: true
        type: choice
        options:
        - development
        - staging
        - production

jobs:
  generate-secrets:
    runs-on: ubuntu-latest
    steps:
      - name: Generate Turso database credentials
        run: |
          # Generate secure database URL and token
          DB_NAME="memory-${{ github.event.inputs.environment }}-$(date +%s)"
          echo "Generated database name: $DB_NAME"
          
          # Store in GitHub Secrets (would need actual API calls)
          echo "DB_URL=libsql://$DB_NAME.turso.io" >> $GITHUB_ENV
          echo "DB_TOKEN=$(echo $RANDOM | base64)" >> $GITHUB_ENV
          
      - name: Update environment secrets
        run: |
          # Update repository secrets
          gh secret set TURSO_DATABASE_URL --body="$DB_URL"
          gh secret set TURSO_AUTH_TOKEN --body="$DB_TOKEN"
```

2. **Secret validation in CI**

```yaml
validate-secrets:
  name: Validate CI Secrets
  runs-on: ubuntu-latest
  steps:
    - name: Check required secrets
      run: |
        REQUIRED_SECRETS=(
          "TURSO_DATABASE_URL"
          "TURSO_AUTH_TOKEN"
        )
        
        for secret in "${REQUIRED_SECRETS[@]}"; do
          if [ -z "${!secret:-}" ]; then
            echo "::error::Required secret $secret is not set"
            exit 1
          fi
        done
        
    - name: Validate database connectivity
      run: |
        # Test connection with provided credentials
        DATABASE_URL="$TURSO_DATABASE_URL" \
        TURSO_AUTH_TOKEN="$TURSO_AUTH_TOKEN" \
        cargo test -p memory-storage-turso --test connectivity
```

### 3.4 Database Connection Pooling

**Current State:** Basic connection pooling in memory-storage-turso
**Target State:** Advanced connection management with monitoring and optimization

#### Implementation Steps:

1. **Enhanced connection pool configuration**

```rust
// memory-storage-turso/src/pool/enhanced.rs
#[derive(Debug, Clone)]
pub struct EnhancedPoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub health_check_interval: Duration,
    pub enable_metrics: bool,
    pub enable_circuit_breaker: bool,
}

impl EnhancedPoolConfig {
    pub fn for_environment(env: &Environment) -> Self {
        match env {
            Environment::Development => Self {
                max_connections: 5,
                min_connections: 1,
                connection_timeout: Duration::from_secs(10),
                idle_timeout: Duration::from_secs(300),
                max_lifetime: Duration::from_secs(3600),
                health_check_interval: Duration::from_secs(30),
                enable_metrics: false,
                enable_circuit_breaker: false,
            },
            Environment::CI => Self {
                max_connections: 10,
                min_connections: 2,
                connection_timeout: Duration::from_secs(5),
                idle_timeout: Duration::from_secs(60),
                max_lifetime: Duration::from_secs(1800),
                health_check_interval: Duration::from_secs(10),
                enable_metrics: true,
                enable_circuit_breaker: true,
            },
            Environment::Production => Self {
                max_connections: 50,
                min_connections: 10,
                connection_timeout: Duration::from_secs(30),
                idle_timeout: Duration::from_secs(1800),
                max_lifetime: Duration::from_secs(7200),
                health_check_interval: Duration::from_secs(60),
                enable_metrics: true,
                enable_circuit_breaker: true,
            },
        }
    }
}
```

2. **Pool monitoring and metrics**

```rust
// memory-storage-turso/src/pool/monitoring.rs
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_connections: u32,
    pub waiting_requests: u32,
    pub connection_errors: u64,
    pub average_wait_time: Duration,
    pub health_status: HealthStatus,
}

impl PoolMetrics {
    pub async fn collect(&self) -> PoolMetricsSnapshot {
        PoolMetricsSnapshot {
            timestamp: chrono::Utc::now(),
            metrics: self.clone(),
            performance: self.calculate_performance_metrics(),
        }
    }
}
```

---

## 4. Testing Strategy

### 4.1 Test Database Setup and Teardown

**Current State:** Basic test database creation
**Target State:** Comprehensive test database lifecycle management

#### Implementation Steps:

1. **Test database lifecycle management**

```rust
// memory-storage-turso/src/testing/database_lifecycle.rs
pub struct TestDatabaseManager {
    temp_dir: TempDir,
    database_urls: HashMap<String, String>,
}

impl TestDatabaseManager {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        
        // Create different database types for comprehensive testing
        let databases = vec![
            ("file_memory", Self::create_file_database(&temp_dir)?),
            ("turso_memory", Self::create_turso_memory_database()?),
            ("in_memory", Self::create_in_memory_database()?),
        ];
        
        let mut database_urls = HashMap::new();
        for (name, url) in databases {
            database_urls.insert(name.to_string(), url);
        }
        
        Ok(Self {
            temp_dir,
            database_urls,
        })
    }
    
    pub async fn get_storage_for_test(&self, test_name: &str) -> Result<TursoStorage> {
        let db_url = self.database_urls.get("file_memory").unwrap();
        let storage = TursoStorage::new(db_url, "").await?;
        storage.initialize_schema().await?;
        Ok(storage)
    }
    
    pub async fn cleanup(&self) -> Result<()> {
        // Cleanup all test databases
        for (_name, url) in &self.database_urls {
            if url.starts_with("file:") {
                let path = url.strip_prefix("file:").unwrap();
                let _ = std::fs::remove_file(path);
            }
        }
        Ok(())
    }
}
```

2. **Automated test database provisioning**

```rust
// memory-core/src/testing/auto_provision.rs
#[tokio::test]
async fn test_with_provisioned_database() {
    let config = TestDatabaseConfig::new()
        .with_schema("standard")
        .with_seed_data("minimal")
        .with_timeout(Duration::from_secs(30));
    
    let storage = config.provision().await.unwrap();
    
    // Test logic here
    
    // Automatic cleanup when config is dropped
}
```

### 4.2 Integration Test Configuration

**Current State:** Basic integration tests
**Target State:** Comprehensive integration test suite with multiple database backends

#### Implementation Steps:

1. **Multi-backend integration tests**

```rust
// memory-storage-turso/tests/multi_backend_integration.rs
#[tokio::test]
#[test_case(":memory:" ; "memory")]
#[test_case("file:./test-data/integration.db" ; "file_sqlite")]
async fn test_cross_backend_compatibility(database_url: &str) {
    let storage = TursoStorage::new(database_url, "").await.unwrap();
    storage.initialize_schema().await.unwrap();
    
    // Test basic CRUD operations
    test_episode_crud(&storage).await;
    test_pattern_operations(&storage).await;
    test_heuristic_storage(&storage).await;
    
    // Test complex queries
    test_complex_queries(&storage).await;
    test_concurrent_operations(&storage).await;
}
```

2. **Performance regression testing**

```rust
// memory-storage-turso/tests/performance_regression.rs
#[tokio::test]
async fn test_performance_regression() {
    let storage = TursoStorage::new("file:./perf-test.db", "").await.unwrap();
    storage.initialize_schema().await.unwrap();
    
    // Baseline performance measurements
    let baseline = PerformanceBaseline::load().unwrap_or_default();
    
    // Run performance tests
    let results = PerformanceTestRunner::new(&storage)
        .add_test("episode_store", EpisodeStoreTest::new(1000))
        .add_test("episode_query", EpisodeQueryTest::new(100))
        .add_test("pattern_extraction", PatternExtractionTest::new(100))
        .run()
        .await
        .unwrap();
    
    // Check for regressions
    for (test_name, result) in results {
        if let Some(baseline_result) = baseline.get(&test_name) {
            assert!(
                result.duration <= baseline_result.duration * 1.1, // 10% tolerance
                "Performance regression detected in {}: {}ms > {}ms",
                test_name,
                result.duration.as_millis(),
                (baseline_result.duration * 1.1).as_millis()
            );
        }
    }
    
    // Update baseline if tests pass
    baseline.update(&results);
    baseline.save().unwrap();
}
```

### 4.3 Data Seeding for Tests

**Current State:** No systematic data seeding
**Target State:** Comprehensive data seeding system for consistent testing

#### Implementation Steps:

1. **Structured test data generation**

```rust
// memory-core/src/testing/data_generators.rs
pub struct TestDataGenerator {
    rng: StdRng,
}

impl TestDataGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
    
    pub fn generate_episodes(&mut self, count: usize) -> Vec<Episode> {
        (0..count)
            .map(|_| self.generate_single_episode())
            .collect()
    }
    
    pub fn generate_episode_with_patterns(
        &mut self, 
        pattern_count: usize
    ) -> (Episode, Vec<Pattern>) {
        let episode = self.generate_single_episode();
        let patterns = (0..pattern_count)
            .map(|_| self.generate_pattern_for_episode(&episode))
            .collect();
        
        (episode, patterns)
    }
    
    fn generate_single_episode(&mut self) -> Episode {
        let task_types = vec![
            TaskType::CodeGeneration,
            TaskType::Debugging,
            TaskType::Refactoring,
            TaskType::Testing,
            TaskType::Documentation,
        ];
        
        let task_type = task_types[self.rng.gen_range(0..task_types.len())];
        let domain = self.generate_domain();
        let language = self.generate_language();
        
        Episode::new(
            self.generate_task_description(&task_type),
            TaskContext {
                domain,
                language: Some(language),
                tags: self.generate_tags(3),
                metadata: serde_json::Value::Object(serde_json::Map::new()),
            },
            task_type,
        )
    }
}
```

2. **Database seeding utilities**

```bash
# scripts/seed-test-database.sh
#!/bin/bash
set -euo pipefail

DATABASE_URL=${1:-"file:./test-data/seeded.db"}
EPISODES=${2:-100}
PATTERNS=${3:-50}

echo "Seeding database: $DATABASE_URL"
echo "Episodes: $EPISODES, Patterns: $PATTERNS"

# Generate and load test data
cargo run --bin generate-test-data \
  --output ./test-data/generated-data.json \
  --episodes $EPISODES \
  --patterns $PATTERNS \
  --seed 42

# Load data into database
DATABASE_URL="$DATABASE_URL" \
cargo run --bin load-test-data \
  --input ./test-data/generated-data.json \
  --validate

echo "Database seeded successfully"
```

---

## 5. Migration and Deployment

### 5.1 Database Schema Management

**Current State:** Manual schema initialization
**Target State:** Comprehensive migration system with version control

#### Implementation Steps:

1. **Migration system implementation**

```rust
// memory-core/src/migration/mod.rs
pub struct MigrationManager {
    storage: TursoStorage,
    migration_lock: Arc<Mutex<()>>,
}

impl MigrationManager {
    pub async fn new(storage: TursoStorage) -> Result<Self> {
        // Ensure migration tracking table exists
        storage.ensure_migration_table().await?;
        
        Ok(Self {
            storage,
            migration_lock: Arc::new(Mutex::new(())),
        })
    }
    
    pub async fn run_pending_migrations(&self) -> Result<Vec<MigrationResult>> {
        let _lock = self.migration_lock.lock().await;
        
        let current_version = self.get_current_schema_version().await?;
        let available_migrations = self.get_available_migrations().await?;
        
        let mut results = Vec::new();
        
        for migration in available_migrations {
            if migration.version > current_version {
                let result = self.apply_migration(&migration).await?;
                results.push(result);
            }
        }
        
        Ok(results)
    }
    
    pub async fn create_migration(&self, name: &str, sql: &str) -> Result<Migration> {
        let version = self.generate_migration_version();
        let migration = Migration {
            version,
            name: name.to_string(),
            sql: sql.to_string(),
            checksum: self.calculate_checksum(sql),
            created_at: chrono::Utc::now(),
        };
        
        self.save_migration(&migration).await?;
        Ok(migration)
    }
}
```

2. **Migration scripts and versioning**

```bash
# scripts/manage-migrations.sh
#!/bin/bash
set -euo pipefail

ACTION=${1:-status}
MIGRATION_NAME=${2:-""}

case "$ACTION" in
    "status")
        echo "Checking migration status..."
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin migration-manager --status
        ;;
        
    "create")
        if [ -z "$MIGRATION_NAME" ]; then
            echo "Usage: $0 create <migration_name>"
            exit 1
        fi
        
        echo "Creating migration: $MIGRATION_NAME"
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin migration-manager --create "$MIGRATION_NAME"
        ;;
        
    "run")
        echo "Running pending migrations..."
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin migration-manager --run
        ;;
        
    "rollback")
        echo "Rolling back last migration..."
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin migration-manager --rollback
        ;;
        
    "verify")
        echo "Verifying migration integrity..."
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin migration-manager --verify
        ;;
esac
```

### 5.2 Migration Scripts

**Current State:** Basic schema creation
**Target State:** Versioned migration system with rollback capabilities

#### Implementation Steps:

1. **Migration script generation**

```bash
# scripts/generate-migration.sh
#!/bin/bash
set -euo pipefail

MIGRATION_NAME=${1:-"$(date +%Y%m%d_%H%M%S)"}
MIGRATION_DIR="./migrations"

# Create migrations directory
mkdir -p "$MIGRATION_DIR"

# Generate migration files
UP_SQL="$MIGRATION_DIR/${MIGRATION_NAME}_up.sql"
DOWN_SQL="$MIGRATION_DIR/${MIGRATION_NAME}_down.sql"

cat > "$UP_SQL" << 'EOF'
-- Migration: MIGRATION_NAME
-- Created: $(date)
-- Description: TODO: Add description

-- Add your SQL here
EOF

cat > "$DOWN_SQL" << 'EOF'
-- Rollback for migration: MIGRATION_NAME
-- Created: $(date)

-- Add rollback SQL here
EOF

echo "Migration files created:"
echo "  Up: $UP_SQL"
echo "  Down: $DOWN_SQL"
echo ""
echo "Edit the files and then run:"
echo "  DATABASE_URL=file:./data/memory.db cargo run --bin migration-manager --apply $MIGRATION_NAME"
```

2. **Automated migration in CI**

```yaml
# Add to .github/workflows/ci.yml
migrations:
  name: Database Migrations
  runs-on: ubuntu-latest
  needs: [ci-guard]
  if: needs.ci-guard.outputs.should-run == 'true'
  steps:
    - uses: actions/checkout@v6
    
    - name: Setup migration environment
      run: |
        cargo install sqlx-cli --no-default-features --features rustls,sqlite
    
    - name: Run migration tests
      run: |
        # Test migration on clean database
        TEST_DB_URL="file:./test-data/migration-test.db" \
        rm -f "$TEST_DB_URL" \
        DATABASE_URL="$TEST_DB_URL" cargo sqlx migrate run --source ./migrations
        
        # Verify migration integrity
        DATABASE_URL="$TEST_DB_URL" cargo run --bin verify-migrations
    
    - name: Check for pending migrations
      run: |
        # Generate migration if schema changed
        if git diff --name-only HEAD~1 HEAD | grep -qE '\.(sql|rs)$'; then
          echo "Schema changes detected, generating migration..."
          ./scripts/generate-migration.sh "auto_$(date +%s)"
        fi
```

### 5.3 Deployment Automation

**Current State:** Manual deployment process
**Target State:** Automated deployment with environment promotion

#### Implementation Steps:

1. **Deployment script system**

```bash
# scripts/deploy.sh
#!/bin/bash
set -euo pipefail

ENVIRONMENT=${1:-"staging"}
VERSION=${2:-"$(git rev-parse HEAD)"}
DRY_RUN=${3:-false}

echo "Deploying to $ENVIRONMENT (version: $VERSION)"
echo "Dry run: $DRY_RUN"

case "$ENVIRONMENT" in
    "development")
        deploy_to_development "$VERSION" "$DRY_RUN"
        ;;
    "staging")
        deploy_to_staging "$VERSION" "$DRY_RUN"
        ;;
    "production")
        deploy_to_production "$VERSION" "$DRY_RUN"
        ;;
    *)
        echo "Unknown environment: $ENVIRONMENT"
        exit 1
        ;;
esac

deploy_to_deployment() {
    local version=$1
    local dry_run=$2
    
    echo "Setting up development environment..."
    
    # Setup local database
    ./scripts/setup-local-db.sh --clean
    DATABASE_URL="file:./data/memory-dev.db" \
    ./scripts/run-migrations.sh
    
    # Run deployment tests
    DATABASE_URL="file:./data/memory-dev.db" \
    cargo test -p memory-storage-turso --test integration
    
    if [ "$dry_run" = "false" ]; then
        echo "Development deployment completed"
    else
        echo "Development deployment would complete (dry run)"
    fi
}
```

2. **Environment promotion workflow**

```yaml
# .github/workflows/deploy.yml
name: Deploy Application
on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      environment:
        description: 'Target environment'
        required: true
        type: choice
        options: [staging, production]

jobs:
  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main' || github.event.inputs.environment == 'staging'
    environment: staging
    steps:
      - uses: actions/checkout@v6
      
      - name: Setup staging database
        run: |
          # Use GitHub Secrets for staging credentials
          DATABASE_URL="${{ secrets.STAGING_DATABASE_URL }}" \
          ./scripts/setup-staging-db.sh
          
      - name: Run staging migrations
        run: |
          DATABASE_URL="${{ secrets.STAGING_DATABASE_URL }}" \
          ./scripts/run-migrations.sh --staging
          
      - name: Deploy application
        run: |
          # Deploy to staging environment
          echo "Deploying to staging..."
          
      - name: Run staging tests
        run: |
          DATABASE_URL="${{ secrets.STAGING_DATABASE_URL }}" \
          cargo test -p memory-storage-turso --test staging-integration
          
      - name: Smoke tests
        run: |
          DATABASE_URL="${{ secrets.STAGING_DATABASE_URL }}" \
          cargo run --bin smoke-test
```

---

## 6. Monitoring and Maintenance

### 6.1 Database Health Checks

**Current State:** Basic health check method
**Target State:** Comprehensive monitoring system with alerting

#### Implementation Steps:

1. **Enhanced health monitoring**

```rust
// memory-storage-turso/src/monitoring/health.rs
pub struct DatabaseHealthMonitor {
    storage: Arc<TursoStorage>,
    config: HealthCheckConfig,
    metrics: Arc<Mutex<HealthMetrics>>,
}

impl DatabaseHealthMonitor {
    pub async fn perform_comprehensive_check(&self) -> HealthReport {
        let start = Instant::now();
        
        // Perform multiple health checks
        let connectivity = self.check_connectivity().await;
        let performance = self.check_performance().await;
        let integrity = self.check_data_integrity().await;
        let capacity = self.check_capacity().await;
        let replication = self.check_replication_status().await;
        
        let duration = start.elapsed();
        
        HealthReport {
            timestamp: chrono::Utc::now(),
            overall_status: self.determine_overall_status(&[
                &connectivity, &performance, &integrity, &capacity, &replication
            ]),
            checks: vec![
                HealthCheck::Connectivity(connectivity),
                HealthCheck::Performance(performance),
                HealthCheck::Integrity(integrity),
                HealthCheck::Capacity(capacity),
                HealthCheck::Replication(replication),
            ],
            duration,
            recommendations: self.generate_recommendations(&[
                &connectivity, &performance, &integrity, &capacity, &replication
            ]),
        }
    }
    
    async fn check_performance(&self) -> PerformanceCheck {
        let query_start = Instant::now();
        
        // Test basic query performance
        let storage = Arc::clone(&self.storage);
        let query_result = tokio::time::timeout(
            Duration::from_secs(5),
            async {
                storage.get_statistics().await
            }
        ).await;
        
        match query_result {
            Ok(Ok(_)) => PerformanceCheck {
                status: HealthStatus::Healthy,
                latency: query_start.elapsed(),
                threshold: Duration::from_millis(100),
                message: "Query performance is optimal".to_string(),
            },
            Ok(Err(e)) => PerformanceCheck {
                status: HealthStatus::Unhealthy,
                latency: query_start.elapsed(),
                threshold: Duration::from_millis(100),
                message: format!("Query failed: {}", e),
            },
            Err(_) => PerformanceCheck {
                status: HealthStatus::Timeout,
                latency: Duration::from_secs(5),
                threshold: Duration::from_millis(100),
                message: "Query timeout".to_string(),
            },
        }
    }
}
```

2. **Health check automation**

```bash
# scripts/health-check.sh
#!/bin/bash
set -euo pipefail

ENVIRONMENT=${1:-"development"}
OUTPUT_FORMAT=${2:-"json"}

echo "Running health check for $ENVIRONMENT environment..."

# Run comprehensive health check
DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
cargo run --bin health-monitor --environment "$ENVIRONMENT" --format "$OUTPUT_FORMAT"

# Check specific metrics
echo "Database size:"
DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
sqlite3 "$(echo "$DATABASE_URL" | sed 's|file:||')" "SELECT name, COUNT(*) FROM sqlite_master WHERE type='table' GROUP BY name;"

echo "Recent activity:"
DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
sqlite3 "$(echo "$DATABASE_URL" | sed 's|file:||')" "SELECT COUNT(*) as episodes_today FROM episodes WHERE date(created_at, 'unixepoch') = date('now');"
```

### 6.2 Performance Monitoring

**Current State:** Basic pool statistics
**Target State:** Comprehensive performance monitoring with alerting

#### Implementation Steps:

1. **Performance metrics collection**

```rust
// memory-storage-turso/src/monitoring/performance.rs
pub struct PerformanceMonitor {
    storage: Arc<TursoStorage>,
    metrics: Arc<Mutex<PerformanceMetrics>>,
    collection_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub query_metrics: QueryMetrics,
    pub connection_metrics: ConnectionMetrics,
    pub system_metrics: SystemMetrics,
}

impl PerformanceMonitor {
    pub async fn start_collection(&self) -> Result<()> {
        let monitor = Arc::clone(&self);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitor.collection_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = monitor.collect_metrics().await {
                    tracing::error!("Failed to collect performance metrics: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    async fn collect_metrics(&self) -> Result<()> {
        let storage = Arc::clone(&self.storage);
        
        // Collect query performance metrics
        let query_metrics = self.collect_query_metrics(&storage).await?;
        
        // Collect connection pool metrics
        let connection_metrics = self.collect_connection_metrics(&storage).await?;
        
        // Collect system metrics
        let system_metrics = self.collect_system_metrics().await?;
        
        let metrics = PerformanceMetrics {
            timestamp: chrono::Utc::now(),
            query_metrics,
            connection_metrics,
            system_metrics,
        };
        
        let mut metrics_guard = self.metrics.lock().await;
        *metrics_guard = metrics;
        
        Ok(())
    }
    
    pub async fn generate_performance_report(&self) -> Result<PerformanceReport> {
        let metrics = self.metrics.lock().await;
        
        Ok(PerformanceReport {
            current_metrics: metrics.clone(),
            trends: self.calculate_trends().await?,
            alerts: self.check_performance_alerts(&metrics).await?,
            recommendations: self.generate_performance_recommendations(&metrics),
        })
    }
}
```

2. **Performance alerting system**

```rust
// memory-storage-turso/src/monitoring/alerts.rs
pub struct PerformanceAlerts {
    thresholds: AlertThresholds,
    alert_handlers: Vec<Box<dyn AlertHandler>>,
}

impl PerformanceAlerts {
    pub async fn check_and_alert(&self, metrics: &PerformanceMetrics) -> Result<Vec<Alert>> {
        let mut alerts = Vec::new();
        
        // Check query latency
        if metrics.query_metrics.average_latency > self.thresholds.max_query_latency {
            alerts.push(Alert::new(
                AlertLevel::Warning,
                "High query latency detected",
                format!("Average query latency: {:?} (threshold: {:?})", 
                    metrics.query_metrics.average_latency,
                    self.thresholds.max_query_latency
                ),
            ));
        }
        
        // Check connection pool utilization
        let pool_utilization = metrics.connection_metrics.utilization_percentage();
        if pool_utilization > self.thresholds.max_pool_utilization {
            alerts.push(Alert::new(
                AlertLevel::Critical,
                "Connection pool saturation",
                format!("Pool utilization: {:.1}% (threshold: {:.1}%)", 
                    pool_utilization,
                    self.thresholds.max_pool_utilization
                ),
            ));
        }
        
        // Send alerts
        for alert in &alerts {
            for handler in &self.alert_handlers {
                if let Err(e) = handler.send_alert(alert).await {
                    tracing::error!("Failed to send alert via {}: {}", 
                        handler.name(), e);
                }
            }
        }
        
        Ok(alerts)
    }
}
```

### 6.3 Backup Strategies

**Current State:** Basic backup configuration
**Target State:** Comprehensive backup and recovery system

#### Implementation Steps:

1. **Automated backup system**

```rust
// memory-storage-turso/src/backup/manager.rs
pub struct BackupManager {
    storage: Arc<TursoStorage>,
    config: BackupConfig,
    backup_storage: Arc<dyn BackupStorage>,
}

impl BackupManager {
    pub async fn create_scheduled_backup(&self) -> Result<BackupResult> {
        let backup_id = format!("backup_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        
        // Create backup metadata
        let metadata = BackupMetadata {
            id: backup_id.clone(),
            created_at: chrono::Utc::now(),
            database_version: self.get_schema_version().await?,
            record_counts: self.get_record_counts().await?,
            checksum: None,
        };
        
        // Perform backup
        let backup_data = self.create_backup_data().await?;
        let checksum = self.calculate_checksum(&backup_data);
        
        let mut metadata = metadata;
        metadata.checksum = Some(checksum);
        
        // Store backup
        self.backup_storage.store(&backup_id, backup_data, metadata).await?;
        
        Ok(BackupResult {
            backup_id,
            size_bytes: backup_data.len(),
            checksum,
            created_at: metadata.created_at,
        })
    }
    
    pub async fn restore_from_backup(&self, backup_id: &str) -> Result<RestoreResult> {
        // Verify backup integrity
        let backup_data = self.backup_storage.retrieve(backup_id).await?;
        let metadata = self.backup_storage.get_metadata(backup_id).await?;
        
        if let Some(expected_checksum) = metadata.checksum {
            let actual_checksum = self.calculate_checksum(&backup_data);
            if expected_checksum != actual_checksum {
                return Err(Error::Backup("Backup checksum verification failed".into()));
            }
        }
        
        // Create backup of current state before restore
        let emergency_backup = self.create_scheduled_backup().await?;
        
        // Restore data
        self.restore_backup_data(&backup_data).await?;
        
        Ok(RestoreResult {
            restored_backup_id: backup_id.to_string(),
            emergency_backup_id: emergency_backup.backup_id,
            restored_at: chrono::Utc::now(),
        })
    }
}
```

2. **Backup automation scripts**

```bash
# scripts/backup-manager.sh
#!/bin/bash
set -euo pipefail

ACTION=${1:-"status"}
BACKUP_ID=${2:-""}
RETENTION_DAYS=${BACKUP_RETENTION_DAYS:-30}

case "$ACTION" in
    "create")
        echo "Creating scheduled backup..."
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin backup-manager --create
        
        # Upload to cloud storage if configured
        if [ -n "${CLOUD_BACKUP_URL:-}" ]; then
            echo "Uploading backup to cloud storage..."
            ./scripts/upload-backup-to-cloud.sh
        fi
        ;;
        
    "list")
        echo "Available backups:"
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin backup-manager --list
        ;;
        
    "restore")
        if [ -z "$BACKUP_ID" ]; then
            echo "Usage: $0 restore <backup_id>"
            exit 1
        fi
        
        echo "Restoring from backup: $BACKUP_ID"
        read -p "This will replace current data. Continue? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
            cargo run --bin backup-manager --restore "$BACKUP_ID"
        else
            echo "Restore cancelled"
        fi
        ;;
        
    "cleanup")
        echo "Cleaning up old backups (older than $RETENTION_DAYS days)..."
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin backup-manager --cleanup --retention-days "$RETENTION_DAYS"
        ;;
        
    "verify")
        if [ -z "$BACKUP_ID" ]; then
            echo "Usage: $0 verify <backup_id>"
            exit 1
        fi
        
        echo "Verifying backup: $BACKUP_ID"
        DATABASE_URL="${DATABASE_URL:-file:./data/memory.db}" \
        cargo run --bin backup-manager --verify "$BACKUP_ID"
        ;;
esac
```

---

## Implementation Timeline

### Phase 1: Local Development Enhancement (Week 1-2)
- [ ] Create environment profile templates (.env.development, .env.test, etc.)
- [ ] Enhance setup-local-db.sh with Turso server support
- [ ] Implement environment auto-detection system
- [ ] Create database lifecycle management scripts

### Phase 2: CI Integration (Week 3-4)
- [ ] Add Turso-specific CI jobs to .github/workflows/ci.yml
- [ ] Implement test database provisioning matrix
- [ ] Create migration validation in CI
- [ ] Add performance regression testing

### Phase 3: Configuration Management (Week 5-6)
- [ ] Implement hierarchical configuration system
- [ ] Create environment-specific configuration files
- [ ] Add secret management for CI with GitHub Secrets
- [ ] Enhance connection pooling with monitoring

### Phase 4: Testing Strategy (Week 7-8)
- [ ] Implement comprehensive test database lifecycle
- [ ] Create multi-backend integration tests
- [ ] Add structured test data generation
- [ ] Implement performance regression testing

### Phase 5: Migration & Deployment (Week 9-10)
- [ ] Build migration management system
- [ ] Create deployment automation scripts
- [ ] Implement environment promotion workflow
- [ ] Add deployment validation

### Phase 6: Monitoring & Maintenance (Week 11-12)
- [ ] Implement comprehensive health monitoring
- [ ] Add performance metrics collection and alerting
- [ ] Create automated backup and recovery system
- [ ] Build maintenance automation scripts

---

## Dependencies and Prerequisites

### Required Tools
- **Rust**: Latest stable version with Cargo
- **libsql-server**: Local Turso server for testing
- **SQLite3**: For file-based database testing
- **Git**: For version control and deployment

### Required Crates (Additional)
- `sqlx-cli`: For migration management
- `tempfile`: For temporary test databases
- `serde_json`: For configuration management
- `chrono`: For timestamp handling
- `tracing`: For logging and monitoring

### Infrastructure Requirements
- **GitHub Actions**: CI/CD pipeline execution
- **GitHub Secrets**: Secure credential management
- **Storage**: For backup and test data
- **Network**: For Turso cloud connections

### Environment Variables
```bash
# Core configuration
MEMORY_ENVIRONMENT=development|testing|staging|production
DATABASE_URL=libsql://... or file:...
TURSO_AUTH_TOKEN=...

# Optional configuration
TURSO_CONNECTION_TIMEOUT=30
BACKUP_RETENTION_DAYS=30
CLOUD_BACKUP_URL=...
```

---

## Success Metrics

### Development Efficiency
- **Setup Time**: Reduce local development setup from 30 minutes to 5 minutes
- **Database Provisioning**: Create isolated test databases in under 10 seconds
- **Environment Switching**: Switch between environments in under 30 seconds

### CI/CD Performance
- **Test Execution**: Reduce CI test execution time by 20% through parallel database testing
- **Migration Reliability**: Achieve 99.9% migration success rate
- **Deployment Success**: Reduce failed deployments by 50%

### System Reliability
- **Health Monitoring**: Detect database issues within 60 seconds
- **Backup Success**: Achieve 99.5% automated backup success rate
- **Data Integrity**: Zero data corruption incidents

### Developer Experience
- **Local Development**: 100% offline development capability
- **Testing**: Comprehensive test coverage with multiple database backends
- **Documentation**: Complete setup and troubleshooting guides

---

## Risk Mitigation

### Technical Risks
- **Database Corruption**: Implement robust backup and recovery systems
- **Migration Failures**: Add rollback mechanisms and testing
- **Performance Degradation**: Implement monitoring and alerting

### Operational Risks
- **Secret Exposure**: Use GitHub Secrets and proper secret management
- **Environment Confusion**: Clear environment separation and documentation
- **Data Loss**: Automated backup with multiple retention periods

### Security Risks
- **SQL Injection**: Continue using parameterized queries
- **Unauthorized Access**: Implement proper authentication and authorization
- **Data Breach**: Encrypt sensitive data at rest and in transit

---

## Conclusion

This comprehensive implementation plan provides a roadmap for establishing robust local Turso database usage and CI integration. The plan builds upon existing infrastructure while adding new capabilities for enhanced development workflows, comprehensive testing, and reliable deployment processes.

The phased approach ensures manageable implementation while delivering immediate value at each stage. Success will be measured by improved development efficiency, reduced deployment risks, and enhanced system reliability.

**Key Benefits:**
- 🚀 Faster local development setup and iteration
- 🧪 Comprehensive testing with multiple database backends
- 📈 Improved CI/CD pipeline performance and reliability
- 🛡️ Enhanced monitoring and maintenance capabilities
- 🔒 Robust security and data protection measures

**Next Steps:**
1. Review and approve the implementation plan
2. Begin Phase 1 development with local environment enhancement
3. Establish success metrics and monitoring
4. Begin CI integration development in parallel

This plan will transform the current basic setup into a production-ready, scalable database management system that supports the entire development lifecycle from local development through production deployment.
