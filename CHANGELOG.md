# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2025-11-14

### Added

#### Production Deployment Guide
- Comprehensive DEPLOYMENT.md with production deployment procedures
- Performance tuning guidelines for connection pools, caching, and batching
- Monitoring setup with key metrics and health checks
- Disaster recovery procedures for Turso and redb restoration
- Systemd service configuration and Docker deployment examples
- Backup strategies and operational troubleshooting guides

#### Bincode Deserialization Security
- Added size limits for bincode deserialization to prevent OOM attacks:
  - Episode deserialization: 10MB limit
  - Pattern deserialization: 1MB limit
  - Heuristic deserialization: 100KB limit
  - Embedding deserialization: 1MB limit
- Comprehensive bincode security tests for redb storage layer
- Input validation bounds documentation in SECURITY.md
- Quota management guidance added to AGENTS.md

## [0.1.0] - 2025-11-13

### Added

#### Core Episode Management
- Episode lifecycle management system with `start_episode()`, `log_step()`, and `complete_episode()` operations
- Support for storing episodes with comprehensive metadata including task type, domain, language, and context tags
- Episodic memory retrieval with metadata filtering and relevance ranking
- Episode artifact storage and management with structured JSON serialization
- Task outcome tracking with execution metrics (duration, tokens, success/failure status)

#### Pattern Extraction System
- Hybrid pattern extraction engine with 4 specialized extractor types:
  - **Tool Sequence Extractor**: Identifies recurring tool usage sequences and call patterns
  - **Decision Point Extractor**: Captures decision boundaries, branching logic, and choice contexts
  - **Error Recovery Extractor**: Learns error handling patterns and recovery strategies
  - **Context Pattern Extractor**: Recognizes contextual patterns and domain-specific behaviors
- Parallel pattern extraction with concurrent processing across multiple extractors
- Intelligent clustering of similar patterns to reduce redundancy and identify meta-patterns
- Pattern confidence scoring with multi-dimensional relevance metrics
- Learning queue for asynchronous pattern extraction from completed episodes

#### Reward Calculation System
- Multi-dimensional scoring system for episodes:
  - Success/failure verdict scoring
  - Efficiency metrics (execution time, token usage)
  - Quality assessment based on error recovery patterns
  - Context appropriateness evaluation
- Cumulative learning metrics with pattern coverage analysis
- Reward threshold configuration with sensible defaults

#### Reflection and Analysis
- Automated reflection generation for episode analysis
- Key insight extraction from execution sequences
- Pattern-based recommendations for future similar tasks
- Confidence-weighted reflection summaries

#### Dual Storage Backend
- **Turso/libSQL Integration**:
  - Durable relational storage with ACID guarantees
  - Support for episodes, patterns, heuristics, and metadata tables
  - Parameterized queries preventing SQL injection
  - Connection pooling for efficient database access
  - Automatic schema initialization and migrations
- **redb Integration**:
  - High-performance in-memory caching layer
  - Fast key-value storage for hot data
  - Transaction support for consistent reads/writes
  - Minimal latency for retrieval operations

#### Storage Synchronization
- Bidirectional sync between Turso and redb with configurable update intervals
- Conflict resolution strategies (last-write-wins, merge) for concurrent updates
- Cache invalidation and refresh mechanisms
- Consistency verification with diagnostic reporting

#### MCP (Model Context Protocol) Server
- Full MCP server implementation for integration with Claude and other AI clients
- Standardized tools for:
  - Starting and managing episodes
  - Logging execution steps
  - Completing episodes with outcomes
  - Retrieving relevant historical context
  - Pattern analysis and visualization
- Real-time tool call handling with streaming support
- Protocol version 2024-11 compliance

#### Code Execution Sandbox
- 6-layer defense-in-depth security architecture:
  - **Filesystem Isolation**: Restricted file access with whitelist/blacklist
  - **Network Isolation**: Disabled network access by default
  - **Process Isolation**: Subprocess spawning restrictions
  - **Memory Limits**: Configurable memory constraints per execution
  - **CPU Limits**: Timeout-based execution termination
  - **System Call Filtering**: Restricted syscall whitelist
- Execution environment configuration with granular permission controls
- Safe script execution with artifact capture (stdout, stderr, exit code)
- Isolated artifact storage preventing execution escape
- Configurable sandbox strictness levels (strict, moderate, permissive)

#### Learning and Heuristics
- Heuristic rule extraction from patterns and episode outcomes
- Condition-action rule generation for decision support
- Heuristic application during pattern matching
- Automatic heuristic scoring and pruning of low-confidence rules
- Continuous learning loop with feedback integration

#### Memory Retrieval System
- Semantic and metadata-based search capabilities
- Relevance ranking algorithm with adjustable weights
- Support for filtering by task type, domain, language, and custom tags
- Configurable result limits and ranking strategies
- Recent episode prioritization for temporal relevance

#### Testing Infrastructure
- Comprehensive test suite with 192+ tests
- Unit tests for all major components
- Integration tests with ephemeral test databases
- End-to-end workflow tests
- >90% code coverage across core modules
- Test utilities and helpers for reproducible testing

#### Security Test Suite
- 51 penetration tests covering:
  - SQL injection attack prevention
  - Sandbox escape attempts
  - Resource exhaustion attacks
  - Privilege escalation scenarios
  - Input validation bypass attempts
  - Cryptographic vulnerability checks
  - Access control violations

#### Performance Benchmarks
- Benchmarks for episode lifecycle operations (start, log, complete)
- Pattern extraction performance metrics across all extractor types
- Storage operation benchmarks (read, write, sync)
- Memory usage profiling for cache operations
- Scalability testing with large episode batches

#### CI/CD Pipelines
- Automated test execution (all platforms)
- Code formatting validation with `cargo fmt`
- Linting with `cargo clippy` (no warnings policy)
- Security audit with `cargo audit`
- License compliance checking with `cargo deny`
- Code coverage reporting and threshold enforcement
- Automated pre-commit hooks for local validation

#### Documentation
- Comprehensive API documentation with examples
- Architecture guide explaining system design
- Security hardening guide for deployment
- Contributing guidelines and development workflow
- Troubleshooting guide for common issues

### Security

#### SQL Injection Prevention
- Parameterized queries for all database operations
- Input validation before query construction
- Type-safe query builders preventing string concatenation
- Regular security audits of SQL construction patterns

#### Sandbox Isolation
- Filesystem access restricted to designated directories
- Network requests disabled by default in sandbox
- Process creation blocked or restricted
- Memory and CPU resource limits enforced
- Configurable timeout protection against infinite loops
- System call filtering with conservative whitelist approach

#### Data Protection
- TLS enforcement for all remote database connections
- Parameterized credentials handling via environment variables
- Artifact sanitization before storage
- Sensitive data redaction in logs and outputs
- Encryption support for at-rest data in configuration

#### Access Control
- Episode ownership and access metadata tracking
- Permission checks for episode retrieval and modification
- RBAC support for remote Turso database
- Token-based authentication for MCP protocol

#### Input Validation
- Validation of episode metadata (language, domain, tags)
- ExecutionStep validation for log operations
- Pattern data type validation
- Sandbox configuration sanitization
- Script content validation before execution

#### Error Handling
- Descriptive error messages without sensitive information leakage
- Proper error propagation with context preservation
- Audit logging of security-relevant events
- Graceful degradation under failure conditions

#### Continuous Security
- Automated security vulnerability scanning in CI/CD
- Regular dependency audits and updates
- Security test suite execution on every commit
- Penetration testing against common attack vectors
- Security best practices enforcement through linting rules

### Technical Details

#### Architecture
- Async/await-based architecture using Tokio runtime
- Modular design with independent components
- Trait-based extensibility for storage and pattern extraction
- Factory patterns for configurable component creation

#### Dependencies
- Core async runtime: `tokio` with full feature set
- Storage: `turso` and `redb` with transaction support
- Error handling: `anyhow` and `thiserror`
- Serialization: `serde` with JSON support
- Cryptography: `sha2` for hashing and verification
- Testing: `tokio-test` and `tempfile` for test infrastructure

#### Performance Characteristics
- Episode creation: <10ms average latency
- Pattern extraction: <100ms for typical episodes
- Memory cache hits: <1ms retrieval latency
- Parallel pattern extraction: up to 4x speedup on quad-core systems
- Storage sync: configurable intervals (default: 5 seconds)

#### Scalability
- Configurable connection pooling for Turso (default: 10 connections)
- Cache size limits with LRU eviction policy
- Batch pattern extraction with configurable batch sizes
- Concurrent semaphore limiting for resource-intensive operations
- Episode retention policies with archival support

[0.1.0]: https://github.com/d-o-hub/rust-self-learning-memory/releases/tag/v0.1.0
