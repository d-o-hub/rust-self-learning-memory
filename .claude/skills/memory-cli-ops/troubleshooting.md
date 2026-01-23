# Memory CLI Troubleshooting

## Common Issues

### Command Not Found

```bash
# Ensure binary exists
ls ./target/release/memory-cli

# Add to PATH
export PATH="./target/release:$PATH"
```

### Database Errors

```bash
# Check database exists
ls -la data/

# Repair database
memory-cli storage repair

# Reinitialize
rm data/memory.db
memory-cli init
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
memory-cli storage status

# Clear cache
rm -rf data/cache.redb

# Rebuild cache
memory-cli storage sync
```

## Debug Mode

```bash
# Enable verbose output
memory-cli -v <command>

# Enable debug logging
RUST_LOG=debug memory-cli <command>
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
memory-cli logs --lines 100

# Filter by level
memory-cli logs --level error

# Search for pattern
memory-cli logs --grep "episode"
```
