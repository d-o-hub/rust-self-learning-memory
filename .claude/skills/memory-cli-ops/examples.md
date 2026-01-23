# Memory CLI Examples

## Episode Workflow

### Start a New Task

```bash
# Create episode
./target/release/memory-cli ep create -t "Implement authentication system"

# Output
Created episode: ep_abc123xyz
```

### Log Progress

```bash
# Log successful step
./target/release/memory-cli ep log-step ep_abc123xyz \
  -t "compiler" \
  -a "build project" \
  --success \
  --latency-ms 1250 \
  -o "Build completed with 0 warnings"

# Log failed step
./target/release/memory-cli ep log-step ep_abc123xyz \
  -t "test" \
  -a "run unit tests" \
  --success=false \
  --latency-ms 500 \
  -o "3 tests failed: auth_test, user_test, token_test"
```

### Complete Episode

```bash
# Successful completion
./target/release/memory-cli ep complete ep_abc123xyz success

# Partial completion
./target/release/memory-cli ep complete ep_abc123xyz partial

# Failed
./target/release/memory-cli ep complete ep_abc123xyz failed
```

## Pattern Analysis

### View Extracted Patterns

```bash
# List patterns
./target/release/memory-cli pat list --limit 20

# View specific pattern
./target/release/memory-cli pat view pat_xyz789

# Filter by confidence
./target/release/memory-cli pat list --min-confidence 0.8
```

## Storage Operations

### Check Status

```bash
./target/release/memory-cli st status

# Output
Cache: 150 episodes, 45 patterns
Sync: Up to date
Last sync: 2024-01-15 10:30:00 UTC
```

### Force Sync

```bash
./target/release/memory-cli st sync --force
```

## Configuration

### Show Current Config

```bash
./target/release/memory-cli cfg show

# JSON output
./target/release/memory-cli cfg show -f json
```

### Validate Config

```bash
./target/release/memory-cli cfg validate

# Output
Configuration valid: 3 issues, 0 critical
- Warning: Cache size below recommended
- Info: Telemetry enabled
```

## Monitoring

### Health Check

```bash
./target/release/memory-cli hp

# Detailed output
./target/release/memory-cli hp --detailed

# Output
Status: Healthy
Database: Connected
Cache: 150 entries
Memory: 45MB
Uptime: 7d 3h 42m
```
