# Nextest Configuration Reference

## Profile Examples

### Development Profile (Fast Feedback)

```toml
[profile.dev]
retries = 0
fail-fast = true
slow-timeout = { period = "10s", terminate-after = 2 }
test-threads = 8
status-level = "skip"
output = "never"
```

### CI Profile (Comprehensive)

```toml
[profile.ci]
retries = 2
fail-fast = false
slow-timeout = { period = "60s", terminate-after = 5 }
test-threads = 4
status-level = "all"
output = "final"
```

### Benchmark Profile

```toml
[profile.bench]
retries = 3
fail-fast = false
slow-timeout = { period = "300s", terminate-after = 2 }
test-threads = 1
```

## Filter Expressions

| Filter | Description | Example |
|--------|-------------|---------|
| `-E 'kind(lib)'` | Library tests only | `kind(lib) and not test(integration)` |
| `-E 'package(memory-core)'` | Specific package | `package(memory-core)` |
| `-E 'test(test_name)'` | Specific test | `test(test_episode_creation)` |
| `-E 'binary(binary_name)'` | Binary tests | `binary(memory-cli)` |

## Running Tests

```bash
# All tests
cargo nextest run

# With specific profile
cargo nextest run --profile ci

# With filter
cargo nextest run -E 'kind(lib)'

# With reporter
cargo nextest run --status-level slow
cargo nextest run --output final

# With retries
cargo nextest run --retries 3
```

## Test Selection Examples

```bash
# Only unit tests
cargo nextest run -E 'kind(lib) and not test(integration)'

# Only integration tests
cargo nextest run -E 'kind(test)'

# Only tests in specific module
cargo nextest run -E 'mod(episode)' -- episode::

# Exclude slow tests
cargo nextest run -E 'not slow'
```
