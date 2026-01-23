# Debugging Techniques

## Panic Backtraces

```bash
# Enable backtraces
RUST_BACKTRACE=1 cargo run

# Full backtrace
RUST_BACKTRACE=full cargo run
```

## Profiling

```bash
# CPU profiling
cargo flamegraph --release

# Memory profiling
heaptrack ./target/release/app
```

## Database Debugging

```bash
# Enable SQL logging
RUST_LOG=memory_storage_turso=debug cargo run

# Check slow queries
# Configure Turso to log slow queries
```

## Network Issues

```bash
# Check connections
netstat -tulpn | grep port

# Monitor traffic
tcpdump -i any port 8080
```

## Quick Diagnosis Flow

1. Check logs for errors
2. Enable debug logging
3. Use Tokio Console
4. Check resource usage
5. Review recent changes
