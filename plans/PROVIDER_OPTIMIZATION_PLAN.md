# Provider-Specific Optimization Plan

## Provider Characteristics Analysis

### OpenAI
- **Rate Limits**: 3,000 RPM (requests per minute), 1,000,000 TPM (tokens per minute)
- **Optimal Batch Size**: 2048 items per request (API limit)
- **Timeout**: 60s standard, 120s for large batches
- **Retry Strategy**: Exponential backoff for 429/503 errors
- **Best Practice**: Use batch API for >10 texts, connection pooling

### Mistral AI
- **Rate Limits**: Varies by plan, typically 100-500 RPM
- **Optimal Batch Size**: 128 items per request (recommended)
- **Timeout**: 30s standard, 60s for batches
- **Retry Strategy**: Simple retry with backoff
- **Best Practice**: Smaller batches than OpenAI, faster response times

### Azure OpenAI
- **Rate Limits**: Configurable per deployment (10-300 RPM typical)
- **Optimal Batch Size**: Matches OpenAI (2048 items)
- **Timeout**: 90s standard (higher latency than OpenAI)
- **Retry Strategy**: Handle Azure-specific error codes
- **Best Practice**: Regional deployment for latency, managed identity auth

### Local/Custom
- **Rate Limits**: Depends on hardware (typically no limits)
- **Optimal Batch Size**: Varies by model (8-64 typical)
- **Timeout**: 10s standard (much faster)
- **Retry Strategy**: Simple retry, fail fast
- **Best Practice**: Smaller batches for memory, no auth overhead

## Optimization Strategies

### 1. Provider-Specific Configuration
Add to `ModelConfig`:
- `timeout_seconds: Option<u64>` - Custom timeout per provider
- `max_retries: u32` - Retry attempts
- `retry_delay_ms: u64` - Base retry delay
- `max_batch_size: Option<usize>` - Provider-specific batch limit
- `rate_limit_rpm: Option<u32>` - Requests per minute
- `rate_limit_tpm: Option<u64>` - Tokens per minute

### 2. Adaptive Batch Sizing
- OpenAI: Up to 2048 items
- Mistral: Up to 128 items  
- Azure: Up to 2048 items (but may need tuning per deployment)
- Local: Up to 32 items (avoid OOM)

### 3. Retry Logic with Backoff
- Exponential backoff: 1s, 2s, 4s, 8s, 16s
- Handle provider-specific error codes
- Respect Retry-After headers
- Circuit breaker pattern for repeated failures

### 4. Rate Limiting
- Token bucket algorithm
- Per-provider rate limits
- Automatic throttling on 429 errors
- Predictive rate limiting based on historical data

### 5. Connection Pooling
- Reuse HTTP connections
- Keep-alive for better performance
- Provider-specific pool sizes

### 6. Request Optimization
- Compression for large batches (gzip)
- Streaming responses for large results
- Parallel requests with semaphore limiting
- Request coalescing for duplicate texts

## Implementation Priority

### Phase 1: Core Optimizations (High Priority)
1. Add provider-specific configuration fields
2. Implement retry logic with exponential backoff
3. Add adaptive batch sizing based on provider
4. Provider-specific timeout handling

### Phase 2: Advanced Features (Medium Priority)
5. Rate limiting with token bucket
6. Connection pooling optimization
7. Circuit breaker pattern
8. Request compression

### Phase 3: Performance Tuning (Lower Priority)
9. Streaming responses
10. Request coalescing
11. Predictive rate limiting
12. Performance metrics collection
