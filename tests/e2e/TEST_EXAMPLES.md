# E2E Test Examples

This document provides examples of running the E2E tests.

## Quick Start

### 1. Verify All Files Created
```bash
cd /workspaces/feat-phase3
bash tests/e2e/verify_tests.sh
```

Expected output:
```
✅ embeddings_openai_test.rs
   Lines: 573
   Tests: 13
✅ embeddings_local_test.rs
   Lines: 640
   Tests: 17
...
All E2E test files created successfully!
```

### 2. Run a Single Test Suite
```bash
# Local provider tests (fastest, no API key needed)
cargo test --test embeddings_local -- --nocapture
```

Expected output:
```
Running 17 tests
test local_provider_initialization ... ok
test local_single_embedding_generation ... ok
test_local_batch_embedding_generation ... ok
...
test result: ok. 17 passed; 0 failed
```

### 3. Run Specific Test
```bash
# Run only performance tests
cargo test --test embeddings_performance test_performance_single_embedding_latency -- --nocapture
```

### 4. Run with Logging
```bash
# Enable debug logging
RUST_LOG=debug cargo test --test embeddings_quality -- --nocapture
```

## Test Examples by Category

### Day 1: Provider Tests

#### OpenAI Provider (with API key)
```bash
export OPENAI_API_KEY="sk-..."
cargo test --test embeddings_openai test_openai_single_embedding_generation -- --nocapture
```

Output:
```
OpenAI embedding generation time: 1.234s
test result: ok. 1 passed
```

#### OpenAI Provider (without API key - mock)
```bash
unset OPENAI_API_KEY
cargo test --test embeddings_openai test_openai_mock_provider_creation -- --nocapture
```

#### Local Provider
```bash
cargo test --test embeddings_local test_local_provider_initialization -- --nocapture
```

### Day 2: Integration Tests

#### CLI Commands
```bash
cargo test --test embeddings_cli test_cli_embedding_list_providers -- --nocapture
```

#### MCP Tools
```bash
cargo test --test embeddings_mcp test_mcp_embedding_tools_registered -- --nocapture
```

### Day 3: Quality & Performance

#### Quality Tests
```bash
cargo test --test embeddings_quality test_quality_search_accuracy_known_queries -- --nocapture
```

#### Performance Benchmarks
```bash
cargo test --test embeddings_performance test_performance_single_embedding_latency -- --nocapture
```

Output:
```
============================================================
Performance Metrics: Single Embedding Generation
============================================================
  Operations: 100
  Total time: 2.345s
  Average: 23.45ms
  Min: 18.2ms
  Max: 45.1ms
  Throughput: 42.65 ops/sec
============================================================
test result: ok. 1 passed
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      
      - name: Run E2E tests
        run: |
          cargo test --test embeddings_local
          cargo test --test embeddings_cli
          cargo test --test embeddings_mcp
          cargo test --test embeddings_quality
          cargo test --test embeddings_performance
```

### GitLab CI Example

```yaml
test:
  script:
    - cargo test --test embeddings_local
    - cargo test --test embeddings_cli
    - cargo test --test embeddings_mcp
    - cargo test --test embeddings_quality
    - cargo test --test embeddings_performance
```

## Expected Results

### All Tests Should Pass
```bash
cargo test --test embeddings_local
```

Expected:
```
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### With OpenAI API Key
```bash
export OPENAI_API_KEY="sk-..."
cargo test --test embeddings_openai
```

Expected:
```
test result: ok. 13 passed; 0 failed; 0 ignored
```

### Without OpenAI API Key (Mock Mode)
```bash
unset OPENAI_API_KEY
cargo test --test embeddings_openai
```

Expected: Some tests may be skipped or use mock configurations
```
test result: ok. 8 passed; 5 skipped; 0 failed
```

## Performance Baselines

When running performance tests, you should see results similar to:

```
Single Embedding Generation:
  Average: 20-50ms (local provider)
  Throughput: 20-50 ops/sec

Batch Embedding (100 items):
  Total: ~500ms
  Average: ~5ms per item
  Throughput: ~200 ops/sec

Search Performance (1000 episodes):
  Average search: < 50ms
  Memory usage: ~6MB
```

## Troubleshooting

### Test Fails to Compile
```bash
# Clean and rebuild
cargo clean
cargo build --package e2e-tests
```

### Test Times Out
```bash
# Increase timeout
timeout 600 cargo test --test embeddings_performance
```

### OpenAI Tests Fail
```bash
# Check API key
echo $OPENAI_API_KEY

# Test API connection
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"
```

### CLI Tests Fail
```bash
# Build CLI first
cargo build --release --package memory-cli

# Run test with full path
CARGO_BIN_EXE_memory-cli=./target/release/memory-cli \
  cargo test --test embeddings_cli
```

## Test Coverage Summary

| Test Suite | Tests | API Key | Time | Coverage |
|------------|-------|---------|------|----------|
| OpenAI     | 13    | Optional | ~2min | Provider |
| Local      | 17    | None     | ~1min | Provider |
| CLI        | 20+   | None     | ~1min | Commands |
| MCP        | 15+   | None     | ~1min | Tools |
| Quality    | 10+   | None     | ~1min | Accuracy |
| Performance| 10+   | None     | ~3min | Speed |

**Total Runtime (without API key)**: ~7 minutes  
**Total Runtime (with API key)**: ~9 minutes

## Next Steps

After verifying tests pass:

1. **Review Results**: Check COMPLETION_REPORT.md for details
2. **Integrate CI/CD**: Add to your pipeline
3. **Monitor Performance**: Track benchmarks over time
4. **Extend Tests**: Add domain-specific tests as needed

Happy testing! 🧪
