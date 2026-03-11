# Circuit Breaker Configuration Guide

**Version**: 1.0
**Date**: 2025-12-28
**Status**: Production Ready

---

## Overview

The circuit breaker pattern prevents cascading failures when embedding providers become unavailable. It automatically detects provider failures and "opens" to fail requests fast instead of waiting for timeouts.

**Key Benefits**:
- Prevents resource exhaustion from hanging requests
- Faster failure detection and recovery
- Automatic retry with exponential backoff
- Configurable thresholds for different environments

---

## Quick Start

### Default Behavior (Recommended)

By default, the circuit breaker is **ENABLED** with sensible defaults:

```toml
# No configuration needed - circuit breaker enabled automatically
[embedding]
provider = "OpenAI"
model = { model_name = "text-embedding-ada-002", embedding_dimension = 1536 }
```

Default settings:
- Failure threshold: 5 consecutive failures
- Success threshold: 2 consecutive successes
- Timeout: 30 seconds before retry
- Half-open attempts: 3 test requests

### Disable Circuit Breaker (Not Recommended)

For testing or debugging, you can disable the circuit breaker:

```toml
[embedding.model.optimization]
enable_circuit_breaker = false
```

**⚠️ WARNING**: Disabling removes fail-fast protection and can lead to:
- Resource exhaustion during provider outages
- Slower error detection
- Cascading failures to dependent services

---

## Configuration Options

### Feature Flag

```toml
[embedding.model.optimization]
# Enable or disable circuit breaker entirely
enable_circuit_breaker = true  # Default: true
```

### Circuit Breaker Parameters

```toml
[embedding.model.optimization.circuit_breaker_config]
# Number of consecutive failures before opening circuit
failure_threshold = 5           # Default: 5

# Number of consecutive successes to close circuit
success_threshold = 2           # Default: 2

# How long to wait before testing recovery (seconds)
timeout_seconds = 30            # Default: 30

# Maximum test requests in half-open state
half_open_max_attempts = 3      # Default: 3
```

### Full Example Configuration

```toml
[embedding]
provider = "OpenAI"
similarity_threshold = 0.7
batch_size = 32
cache_embeddings = true
timeout_seconds = 30

[embedding.model]
model_name = "text-embedding-ada-002"
embedding_dimension = 1536
base_url = "https://api.openai.com/v1"

[embedding.model.optimization]
timeout_seconds = 60
max_retries = 3
retry_delay_ms = 1000
enable_circuit_breaker = true

[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 5
success_threshold = 2
timeout_seconds = 30
half_open_max_attempts = 3
```

---

## Environment-Specific Configurations

### Development

Fast feedback, tolerant of transient issues:

```toml
[embedding.model.optimization]
enable_circuit_breaker = true

[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 3      # Fail fast for quick feedback
success_threshold = 1      # Recover quickly
timeout_seconds = 10       # Short retry interval
half_open_max_attempts = 2
```

**Use when**:
- Local development
- Rapid iteration
- Testing circuit breaker behavior

### Staging

Balanced configuration for testing production-like behavior:

```toml
[embedding.model.optimization]
enable_circuit_breaker = true

[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 5      # Standard sensitivity
success_threshold = 2      # Validate recovery
timeout_seconds = 30       # Standard retry
half_open_max_attempts = 3
```

**Use when**:
- Pre-production validation
- Integration testing
- Performance testing

### Production

Conservative settings for stability:

```toml
[embedding.model.optimization]
enable_circuit_breaker = true

[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 10     # More tolerant of transient issues
success_threshold = 3      # Ensure stable recovery
timeout_seconds = 60       # Longer recovery time
half_open_max_attempts = 5 # More validation attempts
```

**Use when**:
- Production deployment
- High-traffic environments
- Cost-sensitive operations

---

## Disabling Circuit Breaker

### When to Disable

**⚠️ Use extreme caution. Only disable if**:

1. **Debugging provider integration**
   - Need to see full error details from provider
   - Investigating timeout vs connection issues
   - Testing provider behavior under load

2. **Emergency fallback**
   - Circuit breaker bugs causing issues (see incident runbook)
   - False positives blocking valid requests
   - Provider SLA requires attempting all requests

3. **Local/offline providers**
   - Using local embedding models (no network)
   - Mock providers for testing
   - Custom in-process providers

### How to Disable

#### Method 1: Configuration File (Recommended)

```toml
[embedding.model.optimization]
enable_circuit_breaker = false
```

Restart the service:
```bash
systemctl restart memory-service
```

#### Method 2: Environment Variable

```bash
export MEMORY_DISABLE_CIRCUIT_BREAKER=true
memory-mcp-server
```

#### Method 3: Runtime (If Implemented)

```bash
curl -X POST http://localhost:8080/admin/circuit-breaker/disable
```

---

## Tuning Guidelines

### Failure Threshold

**What it does**: Number of consecutive failures before circuit opens

**Increase if**:
- Seeing false positives (circuit opening unnecessarily)
- Provider has occasional transient errors
- Network is unstable
- Cost of false positive > cost of slow failure

**Decrease if**:
- Need faster failure detection
- Provider failures are consistently severe
- Want to protect system more aggressively
- Cost of slow failure > cost of false positive

**Examples**:
```toml
# Very sensitive (dev/testing)
failure_threshold = 2

# Standard (staging)
failure_threshold = 5

# Tolerant (production)
failure_threshold = 10

# Very tolerant (unreliable network)
failure_threshold = 20
```

### Success Threshold

**What it does**: Number of consecutive successes to close circuit

**Increase if**:
- Provider recovery is often unstable
- Need high confidence before reopening
- Cost of premature reopening is high

**Decrease if**:
- Provider recovers reliably
- Need faster recovery time
- Cost of delayed recovery is high

**Examples**:
```toml
# Fast recovery (dev)
success_threshold = 1

# Balanced (staging)
success_threshold = 2

# Conservative (production)
success_threshold = 3

# Very conservative (critical systems)
success_threshold = 5
```

### Timeout Seconds

**What it does**: How long to wait before testing recovery

**Increase if**:
- Provider recovery takes time (e.g., rate limits reset after 60s)
- Want to reduce retry storm impact
- Circuit is flapping (rapid open/close cycles)

**Decrease if**:
- Need faster recovery attempts
- Provider recovers quickly
- Want more responsive system

**Examples**:
```toml
# Fast retry (dev)
timeout_seconds = 10

# Standard (staging)
timeout_seconds = 30

# Conservative (production)
timeout_seconds = 60

# Rate limit aware (OpenAI rate limits reset after 60s)
timeout_seconds = 90
```

### Half-Open Max Attempts

**What it does**: Maximum test requests allowed in half-open state

**Increase if**:
- Need higher confidence in recovery
- Can afford more test requests
- Provider recovery validation requires multiple requests

**Decrease if**:
- Want faster decision (reopen or close)
- Test requests are expensive
- Provider status is binary (works or doesn't)

**Examples**:
```toml
# Fast decision (dev)
half_open_max_attempts = 2

# Balanced (staging)
half_open_max_attempts = 3

# Conservative (production)
half_open_max_attempts = 5

# Expensive requests
half_open_max_attempts = 1
```

---

## Provider-Specific Recommendations

### OpenAI

```toml
[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 5       # OpenAI is generally reliable
success_threshold = 2       # Quick recovery
timeout_seconds = 90        # Rate limits reset after 60s
half_open_max_attempts = 3
```

**Rationale**:
- OpenAI has excellent uptime (>99.9%)
- Rate limits are primary failure mode
- Rate limits reset after 60 seconds
- Longer timeout prevents hitting limits during recovery

### Mistral AI

```toml
[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 3       # Smaller service, faster detection
success_threshold = 2
timeout_seconds = 20        # Faster recovery attempts
half_open_max_attempts = 2
```

**Rationale**:
- Smaller service may have more variability
- Faster recovery attempts to minimize downtime
- Lower rate limits require more conservative approach

### Azure OpenAI

```toml
[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 10      # Enterprise SLA, be tolerant
success_threshold = 3       # Ensure stable recovery
timeout_seconds = 60
half_open_max_attempts = 5
```

**Rationale**:
- Enterprise service with SLAs
- More tolerant of transient issues
- Conservative recovery to avoid flapping

### Local/Self-Hosted

```toml
[embedding.model.optimization]
enable_circuit_breaker = false  # Not needed for local providers
```

**Rationale**:
- No network latency or external failures
- Failures are typically immediate (out of memory, crash)
- Circuit breaker adds unnecessary overhead

---

## Monitoring

### Log Patterns

#### Circuit Opening (Action Required)

```
WARN Circuit breaker state transition: Closed -> Open (threshold reached: 5/5 failures)
```

**What to do**: Check provider health, review recent deployments

#### Recovery Success (Informational)

```
INFO Circuit breaker state transition: HalfOpen -> Closed (recovery successful after 2 successes)
```

**What to do**: Review root cause, validate recovery

#### Request Blocked (Expected During Outage)

```
DEBUG Circuit breaker: request blocked (open for 15s / 30s)
```

**What to do**: Wait for recovery or investigate if prolonged

### Metrics to Track

```
circuit_breaker_state{state="open|closed|half_open"}
circuit_breaker_state_transitions_total
circuit_breaker_blocked_requests_total
circuit_breaker_consecutive_failures
```

### Alerts

```yaml
# Alert if circuit open > 5 minutes
- alert: CircuitBreakerOpen
  expr: circuit_breaker_state{state="open"} == 1
  for: 5m
  severity: warning

# Alert if circuit flapping
- alert: CircuitBreakerFlapping
  expr: rate(circuit_breaker_state_transitions_total[1h]) > 5
  for: 5m
  severity: warning
```

---

## Troubleshooting

### Problem: Circuit Opens Immediately

**Symptoms**:
- Circuit opens after just a few requests
- Provider is actually healthy

**Possible Causes**:
1. `failure_threshold` too low
2. Timeouts too aggressive
3. Network instability

**Solution**:
```toml
[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 10  # Increase threshold
timeout_seconds = 60    # Increase timeout

[embedding.model.optimization]
timeout_seconds = 90    # Increase request timeout
```

### Problem: Circuit Never Opens

**Symptoms**:
- Provider failing but circuit stays closed
- Slow failures accumulating

**Possible Causes**:
1. Circuit breaker disabled
2. `failure_threshold` too high
3. Failures not consecutive (successes interspersed)

**Solution**:
```toml
[embedding.model.optimization]
enable_circuit_breaker = true  # Ensure enabled

[embedding.model.optimization.circuit_breaker_config]
failure_threshold = 3  # Lower threshold
```

### Problem: Circuit Flapping

**Symptoms**:
- Circuit repeatedly opening and closing
- Unstable behavior

**Possible Causes**:
1. Provider intermittent issues
2. Timeout too short
3. Success threshold too low

**Solution**:
```toml
[embedding.model.optimization.circuit_breaker_config]
timeout_seconds = 90    # Wait longer before retry
success_threshold = 3   # Require more successes
```

### Problem: Slow Recovery

**Symptoms**:
- Circuit stays open too long
- Provider recovered but circuit doesn't close

**Possible Causes**:
1. `timeout_seconds` too long
2. `success_threshold` too high

**Solution**:
```toml
[embedding.model.optimization.circuit_breaker_config]
timeout_seconds = 20    # Faster retry
success_threshold = 2   # Easier to close
```

---

## Best Practices

### DO:

✓ **Use default settings initially** - Optimized for most use cases
✓ **Enable in production** - Protection against provider outages
✓ **Monitor circuit state** - Track openings and recoveries
✓ **Tune based on data** - Adjust thresholds using real metrics
✓ **Document changes** - Note why thresholds were adjusted
✓ **Test circuit behavior** - Verify configuration in staging

### DON'T:

✗ **Disable without reason** - Removes important protection
✗ **Set threshold too low** - Causes false positives
✗ **Set threshold too high** - Delays failure detection
✗ **Ignore flapping** - Indicates configuration or provider issues
✗ **Skip monitoring** - Won't know if circuit is working
✗ **Copy-paste configs** - Each provider needs tuning

---

## Migration Guide

### Upgrading from Pre-Circuit-Breaker Version

**Before (v0.1.6 and earlier)**:
```toml
[embedding.model.optimization]
timeout_seconds = 60
max_retries = 3
```

**After (v0.1.7+)**:
```toml
[embedding.model.optimization]
timeout_seconds = 60
max_retries = 3
enable_circuit_breaker = true  # NEW: Enabled by default

[embedding.model.optimization.circuit_breaker_config]  # NEW
failure_threshold = 5
success_threshold = 2
timeout_seconds = 30
half_open_max_attempts = 3
```

**Action Required**: None - circuit breaker enabled automatically with safe defaults

### Opting Out

If you want to maintain pre-v0.1.7 behavior (no circuit breaker):

```toml
[embedding.model.optimization]
enable_circuit_breaker = false  # Explicitly disable
```

---

## FAQ

### Q: Will this break my existing config?

**A**: No. Circuit breaker is enabled by default with safe defaults that won't affect normal operation.

### Q: Does this add latency?

**A**: Negligible. Circuit breaker checks are in-memory and take microseconds. The fail-fast behavior actually *reduces* latency during provider outages.

### Q: Can I have different settings per provider?

**A**: Yes. Each embedding configuration can have its own circuit breaker settings.

```toml
[embedding_openai]
# OpenAI-specific config

[embedding_mistral]
# Mistral-specific config with different thresholds
```

### Q: What happens to requests during circuit open?

**A**: They fail immediately with `CircuitOpenError` instead of waiting for timeout. This allows graceful degradation.

### Q: How do I know if the circuit breaker is working?

**A**: Monitor logs for state transitions. You should see:
- `Closed -> Open` when provider fails
- `Open -> HalfOpen` when testing recovery
- `HalfOpen -> Closed` when recovered

### Q: Can I manually reset the circuit?

**A**: Yes (if implemented):
```bash
curl -X POST http://localhost:8080/admin/circuit-breaker/reset
```

Or restart the service (circuit resets to closed state).

### Q: Does this work with all providers?

**A**: Yes. Circuit breaker is provider-agnostic and works with OpenAI, Mistral, Azure, and custom providers.

### Q: What if I have multiple instances?

**A**: Each instance has its own circuit breaker state. Consider using a distributed circuit breaker (Redis, etc.) for multi-instance deployments.

---

## Additional Resources

- **Code**: `memory-core/src/embeddings/circuit_breaker.rs`
- **Tests**: `memory-core/src/embeddings/circuit_breaker.rs::tests`
- **Incident Runbook**: `plans/CIRCUIT_BREAKER_INCIDENT_RUNBOOK.md`
- **Verification Report**: `plans/ANALYSIS_SWARM_VERIFICATION_REPORT.md`

**Version**: 1.0
**Last Updated**: 2025-12-28
**Next Review**: 2026-01-28 (or after production deployment)
