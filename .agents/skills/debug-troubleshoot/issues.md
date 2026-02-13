# Common Issues

## Async Deadlock

**Symptom**: Application hangs, no progress

**Cause**: Holding a lock across an await point

**Fix**:
```rust
// Bad
let data = guard.lock().await;
some_async_operation().await;  // Blocks others
drop(guard);

// Good
let data = {
    let guard = guard.lock().await;
    (*guard).clone()  // Copy data
};
some_async_operation().await;  // Lock released
```

## Database Connection Exhaustion

**Symptom**: "Too many connections" errors

**Cause**: Connections not returned to pool

**Fix**: Use proper pool management
```rust
// Use try_with or with_available
pool.try_with(|conn| async move {
    // Use connection
    Ok::<_, Error>(())
}).await?;
```

## Panic in Tokio Task

**Symptom**: Silent failures, missing logs

**Fix**: Add panic handler
```rust
std::panic::set_hook(Box::new(|panic| {
    error!("Panic: {:?}", panic);
}));
```

## Memory Leaks

**Symptom**: Increasing memory usage

**Check**: Arc<Mutex> cycles or growing collections

## Performance Issues

**Check**:
- Spawn blocking in async context
- Large data in memory
- Missing indexes in queries
