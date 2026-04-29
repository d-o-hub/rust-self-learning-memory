# Memory CLI Troubleshooting

## Common Issues

### Invalid Arguments (Exit code 2)

**Common mistakes**:
- `--type` for episode create → NO `--type` argument exists. Task type is inferred.
- `--format json` for episode list → No format option for list/view commands
- `-v` for verbose → No global verbose flag exists

**Solution**: Use `--help` to verify correct arguments:
```bash
do-memory-cli episode create --help
do-memory-cli episode log-step --help
do-memory-cli pattern list --help
```

### Command Not Found

```bash
# Ensure binary exists
ls ./target/release/do-memory-cli

# Add to PATH
export PATH="./target/release:$PATH"
```

### Database Errors

```bash
# Check database exists
ls -la data/

# Repair database
do-memory-cli storage repair

# Reinitialize
rm data/memory.db
do-memory-cli init
```

### Authentication Errors

```bash
# Check environment variables
echo $TURSO_TOKEN
echo $TURSO_URL

# Set credentials
export TURSO_TOKEN="your-token"
export TURSO_URL="libsql://your-db.turso.io"
```

### Connection Issues

```bash
# Test connectivity
curl -v $TURSO_URL

# Check network
ping your-db.turso.io

# Verify token
curl -H "Authorization: Bearer $TURSO_TOKEN" $TURSO_URL
```

### Performance Issues

```bash
# Check cache status
do-memory-cli storage status

# Clear cache
rm -rf data/cache.redb

# Rebuild cache
do-memory-cli storage sync
```

### Steps Not Persisting (Episode shows "Steps: 0")

**Root Cause**: CLI uses step batching (50 steps or 5s interval). Each CLI command is a separate process, so buffered steps are lost between invocations.

**Solutions**:
1. Complete the episode immediately after logging steps (completion flushes buffered steps)
2. Or use MCP server for long-running workflows (steps persist across commands)
3. Or log ≥50 steps in one session to trigger batch flush

**Note**: Pattern extraction requires steps. Episodes with 0 steps will not extract patterns.

### Patterns Not Found (Pattern list shows 0)

**Root Cause**: Pattern extraction requires execution steps. If episode has 0 steps, no patterns are extracted.

**Verification**:
```bash
# Check episode steps
do-memory-cli episode view <EPISODE_ID>
# Should show Steps: N (not 0)

# Check storage stats
do-memory-cli storage stats
# Patterns: Total should be > 0
```

## Debug Mode

```bash
# Enable verbose output
do-memory-cli -v <command>

# Enable debug logging
RUST_LOG=debug do-memory-cli <command>
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Database error |
| 4 | Network error |
| 5 | Authentication error |

## Log Analysis

```bash
# View recent logs
do-memory-cli logs --lines 100

# Filter by level
do-memory-cli logs --level error

# Search for pattern
do-memory-cli logs --grep "episode"
```
