# Memory CLI User Guide

## Overview

The Memory CLI is a comprehensive command-line interface for managing the Self-Learning Memory System. It provides direct access to episode management, pattern analysis, storage operations, health monitoring, backup/restore, log analysis, evaluation tools, and system administration.

## Quick Start

```bash
# Install the CLI
cargo install --path memory-cli --features full

# Configure database connection (or use interactive wizard)
echo '[database]
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"' > memory-cli.toml

# Or run the interactive wizard
memory-cli config wizard

# Create your first episode
memory-cli episode create --task "Implement user authentication"

# View recent episodes
memory-cli episode list

# Check system health
memory-cli health check
```

## Command Aliases

The CLI provides convenient shortcuts for frequently used commands:

| Alias | Full Command |
|-------|--------------|
| `ep` | `episode` |
| `pat` | `pattern` |
| `st` | `storage` |
| `cfg` | `config` |
| `hp` | `health` |
| `bak` | `backup` |
| `mon` | `monitor` |
| `log` | `logs` |
| `comp` | `completion` |
| `ev` | `eval` |

Example:
```bash
# Long form
memory-cli episode list --limit 10

# Short form
memory-cli ep list --limit 10
```

## Quick Start

```bash
# Install the CLI
cargo install --path memory-cli --features full

# Configure database connection
echo '[database]
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"' > memory-cli.toml

# Create your first episode
memory-cli episode create --task "Implement user authentication"

# View recent episodes
memory-cli episode list

# Check system health
memory-cli storage health
```

## Command Reference

### Global Options

All commands support these global options:

- `--config <FILE>`: Path to configuration file
- `--format <FORMAT>`: Output format (human/json/yaml)
- `--verbose`: Enable verbose logging
- `--dry-run`: Preview operations without executing
- `--help`: Show help information

### Episode Commands

#### `memory-cli episode create`

Create a new episode to track a task execution.

**Options:**
- `--task <TASK>`: Task description (required)
- `--context <FILE>`: Path to context file (JSON/YAML)

**Examples:**
```bash
# Simple episode creation
memory-cli episode create --task "Implement user authentication"

# With context file
memory-cli episode create --task "Refactor database layer" --context db-context.json

# Dry run to preview
memory-cli --dry-run episode create --task "Test task"
```

**Context File Format (JSON):**
```json
{
  "language": "rust",
  "domain": "web-development",
  "tags": ["authentication", "security"],
  "complexity": "moderate",
  "estimated_duration": "4 hours"
}
```

#### `memory-cli episode list`

List episodes with optional filtering.

**Options:**
- `--task-type <TYPE>`: Filter by task type (code_generation, debugging, testing, analysis, documentation, refactoring, other)
- `--limit <NUM>`: Maximum episodes to return (default: 10)
- `--status <STATUS>`: Filter by status (in_progress, completed)

**Examples:**
```bash
# List recent episodes
memory-cli episode list

# Show only completed episodes
memory-cli episode list --status completed

# Filter by task type with limit
memory-cli episode list --task-type debugging --limit 20

# JSON output for scripting
memory-cli episode list --format json
```

#### `memory-cli episode view`

Display detailed information about a specific episode.

**Arguments:**
- `EPISODE_ID`: Episode UUID

**Examples:**
```bash
# View episode details
memory-cli episode view 12345678-1234-1234-1234-123456789abc

# JSON output for processing
memory-cli episode view 12345678-1234-1234-1234-123456789abc --format json
```

#### `memory-cli episode complete`

Mark an episode as completed with an outcome.

**Arguments:**
- `EPISODE_ID`: Episode UUID

**Options:**
- `--outcome <OUTCOME>`: Task outcome (success, partial_success, failure) (required)

**Examples:**
```bash
# Mark as successful
memory-cli episode complete 12345678-1234-1234-1234-123456789abc --outcome success

# Mark as partial success
memory-cli episode complete 12345678-1234-1234-1234-123456789abc --outcome partial_success

# Dry run first
memory-cli --dry-run episode complete 12345678-1234-1234-1234-123456789abc --outcome success
```

#### `memory-cli episode search`

Search episodes by content.

**Arguments:**
- `QUERY`: Search query string

**Options:**
- `--limit <NUM>`: Maximum results to return (default: 10)

**Examples:**
```bash
# Search for authentication-related episodes
memory-cli episode search "authentication"

# Limit results
memory-cli episode search "database" --limit 5
```

#### `memory-cli episode log-step`

Log an execution step within an episode.

**Arguments:**
- `EPISODE_ID`: Episode UUID

**Options:**
- `--tool <TOOL>`: Tool name (required)
- `--action <ACTION>`: Action description (required)
- `--success <BOOL>`: Whether step succeeded (required)
- `--latency-ms <NUM>`: Latency in milliseconds
- `--tokens <NUM>`: Token count
- `--observation <TEXT>`: Step observation

**Examples:**
```bash
# Log a successful step
memory-cli episode log-step 12345678-1234-1234-1234-123456789abc \
  --tool "grep" \
  --action "Search for authentication patterns" \
  --success true \
  --latency-ms 150 \
  --tokens 25 \
  --observation "Found 3 relevant patterns"

# Log a failed step
memory-cli episode log-step 12345678-1234-1234-1234-123456789abc \
  --tool "cargo" \
  --action "Run tests" \
  --success false \
  --observation "Compilation failed due to missing dependency"
```

### Pattern Commands

#### `memory-cli pattern list`

List patterns with optional filtering.

**Options:**
- `--min-confidence <FLOAT>`: Minimum confidence threshold (default: 0.0)
- `--pattern-type <TYPE>`: Filter by pattern type (ToolSequence, DecisionPoint, ErrorRecovery, ContextPattern)
- `--limit <NUM>`: Maximum patterns to return (default: 20)

**Examples:**
```bash
# List all patterns
memory-cli pattern list

# High-confidence patterns only
memory-cli pattern list --min-confidence 0.8

# Tool sequences with limit
memory-cli pattern list --pattern-type ToolSequence --limit 10
```

#### `memory-cli pattern view`

Display detailed information about a specific pattern.

**Arguments:**
- `PATTERN_ID`: Pattern identifier

**Examples:**
```bash
# View pattern details
memory-cli pattern view pattern-123

# JSON output
memory-cli pattern view pattern-123 --format json
```

#### `memory-cli pattern analyze`

Analyze pattern effectiveness across episodes.

**Arguments:**
- `PATTERN_ID`: Pattern identifier

**Options:**
- `--episodes <NUM>`: Number of episodes to analyze (default: 100)

**Examples:**
```bash
# Analyze pattern effectiveness
memory-cli pattern analyze pattern-123

# Analyze with more episodes
memory-cli pattern analyze pattern-123 --episodes 500
```

#### `memory-cli pattern effectiveness`

Show pattern effectiveness rankings.

**Options:**
- `--top <NUM>`: Show top N patterns (default: 10)
- `--min-uses <NUM>`: Minimum number of uses (default: 1)

**Examples:**
```bash
# Top 10 most effective patterns
memory-cli pattern effectiveness

# Top 5 patterns with at least 3 uses
memory-cli pattern effectiveness --top 5 --min-uses 3
```

#### `memory-cli pattern decay`

Apply pattern decay to remove ineffective patterns.

**Options:**
- `--dry-run`: Preview what would be decayed
- `--force`: Apply decay without confirmation

**Examples:**
```bash
# Preview decay operation
memory-cli pattern decay --dry-run

# Apply decay (requires confirmation)
memory-cli pattern decay --force
```

### Storage Commands

#### `memory-cli storage stats`

Display storage statistics and usage information.

**Examples:**
```bash
# View storage statistics
memory-cli storage stats

# JSON output for monitoring
memory-cli storage stats --format json
```

#### `memory-cli storage sync`

Synchronize data between storage backends.

**Options:**
- `--force`: Force full synchronization
- `--dry-run`: Preview sync operation

**Examples:**
```bash
# Incremental sync
memory-cli storage sync

# Full sync
memory-cli storage sync --force

# Preview sync
memory-cli storage sync --dry-run
```

#### `memory-cli storage vacuum`

Optimize and clean storage.

**Options:**
- `--dry-run`: Preview vacuum operation

**Examples:**
```bash
# Preview vacuum
memory-cli storage vacuum --dry-run

# Execute vacuum
memory-cli storage vacuum
```

#### `memory-cli storage health`

Check storage backend health.

**Examples:**
```bash
# Health check
memory-cli storage health

# JSON output for monitoring
memory-cli storage health --format json
```

#### `memory-cli storage connections`

Show connection status and pool information.

**Examples:**
```bash
# Connection status
memory-cli storage connections
```

### Eval Commands (alias: `ev`)

#### `memory-cli eval calibration`

View domain calibration statistics and effectiveness.

**Options:**
- `--domain <DOMAIN>`: Filter by specific domain
- `--all`: Show all domains (including those with few episodes)
- `--min-episodes <NUM>`: Minimum episodes required to show domain (default: 5)

**Examples:**
```bash
# View all domains
memory-cli eval calibration --all

# View specific domain
memory-cli eval calibration --domain web-development

# View reliable domains only
memory-cli eval calibration --min-episodes 10
```

#### `memory-cli eval stats`

View detailed statistics for a specific domain.

**Arguments:**
- `DOMAIN`: Domain to analyze

**Examples:**
```bash
# View web-development domain stats
memory-cli eval stats web-development

# JSON output for automation
memory-cli eval stats web-development --format json
```

#### `memory-cli eval set-threshold`

Set custom duration and step count thresholds for a domain.

**Options:**
- `--domain <DOMAIN>`: Domain to configure (required)
- `--duration <SECONDS>`: Duration threshold in seconds
- `--steps <NUM>`: Step count threshold

**Examples:**
```bash
# Set duration threshold
memory-cli eval set-threshold --domain web-development --duration 300

# Set step count threshold
memory-cli eval set-threshold --domain web-development --steps 15

# Set both thresholds
memory-cli eval set-threshold --domain web-development --duration 300 --steps 15
```

### Meta Commands

#### `memory-cli completion`

Generate shell completion scripts.

**Arguments:**
- `SHELL`: Shell type (bash, zsh, fish, etc.)

**Examples:**
```bash
# Generate Bash completions
memory-cli completion bash > memory-cli.bash

# Generate Zsh completions
memory-cli completion zsh > _memory-cli

# Generate Fish completions
memory-cli completion fish > memory-cli.fish
```

#### `memory-cli config`

Validate configuration file.

**Examples:**
```bash
# Validate current configuration
memory-cli config

# Validate specific config file
memory-cli --config custom.toml config
```

## Configuration

### Configuration File Locations

The CLI searches for configuration files in this order:

1. Explicit path via `--config`
2. `memory-cli.toml`
3. `memory-cli.json`
4. `memory-cli.yaml`
5. `.memory-cli.toml`
6. `.memory-cli.json`
7. `.memory-cli.yaml`

### Configuration Schema

```toml
[database]
# Turso database configuration
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"

# redb cache configuration
redb_path = "memory.redb"

[storage]
# Cache settings
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
# CLI behavior
default_format = "human"
progress_bars = true
batch_size = 100
```

### Environment Variables

You can override configuration using environment variables:

- `MEMORY_TURSO_URL`: Turso database URL
- `MEMORY_TURSO_TOKEN`: Turso authentication token
- `MEMORY_REDB_PATH`: redb database path

## Output Formats

### Human Format (Default)

Human-readable output with colors and formatting:

```
Episode Created
ID: 12345678-1234-1234-1234-123456789abc
Task: Implement user authentication
Status: created
```

### JSON Format

Machine-readable JSON output:

```json
{
  "episode_id": "12345678-1234-1234-1234-123456789abc",
  "task": "Implement user authentication",
  "status": "created"
}
```

### YAML Format

Configuration-friendly YAML output:

```yaml
episode_id: 12345678-1234-1234-1234-123456789abc
task: Implement user authentication
status: created
```

## Error Handling

### Exit Codes

- `0`: Success
- `1`: General error
- `2`: Configuration error
- `3`: Validation error
- `4`: Authentication error
- `5`: Connection error

### Common Errors

**Configuration Errors:**
```
Error: Failed to read config file: memory-cli.toml
Solution: Check file permissions and path
```

**Database Errors:**
```
Error: Connection refused
Solution: Verify database URL and credentials
```

**Validation Errors:**
```
Error: Invalid episode ID format
Solution: Use a valid UUID format
```

## Advanced Usage

### Scripting Examples

#### Batch Episode Processing
```bash
#!/bin/bash
# Export recent episodes to JSON files

memory-cli episode list --limit 50 --format json | \
  jq -r '.episodes[].episode_id' | \
  while read episode_id; do
    memory-cli episode view "$episode_id" --format json > "episode_$episode_id.json"
  done
```

#### Pattern Effectiveness Monitoring
```bash
#!/bin/bash
# Alert on low-effectiveness patterns

threshold=0.7
memory-cli pattern effectiveness --format json | \
  jq --arg threshold "$threshold" '.rankings[] | select(.effectiveness_score < ($threshold | tonumber))' | \
  while read pattern; do
    echo "Low effectiveness pattern detected:"
    echo "$pattern" | jq .
  done
```

#### Health Monitoring
```bash
#!/bin/bash
# Check system health for monitoring

if ! memory-cli storage health --format json | jq -e '.overall == "healthy"' > /dev/null; then
  echo "Storage health check failed!" >&2
  memory-cli storage health
  exit 1
fi

echo "All systems healthy"
```

### Integration with CI/CD

#### Pre-commit Hook
```bash
#!/bin/bash
# Validate configuration before commit

if ! memory-cli config; then
  echo "Configuration validation failed"
  exit 1
fi
```

#### Deployment Health Check
```bash
#!/bin/bash
# Health check for deployment verification

echo "Running memory system health checks..."

# Check configuration
memory-cli config || exit 1

# Check storage health
memory-cli storage health --format json | jq -e '.overall == "healthy"' || exit 1

# Check recent episodes
episode_count=$(memory-cli episode list --limit 1 --format json | jq '.total_count')
if [ "$episode_count" -lt 0 ]; then
  echo "Episode count check failed"
  exit 1
fi

echo "All health checks passed"
```

## Troubleshooting

### Debug Mode

Enable verbose logging for detailed diagnostics:

```bash
memory-cli --verbose episode list
```

### Dry Run Mode

Preview operations without making changes:

```bash
memory-cli --dry-run episode complete <episode-id> --outcome success
```

### Common Issues

1. **"Turso storage feature not enabled"**
   - Build with `--features turso` or use `--features full`

2. **"Connection refused"**
   - Check database URL and credentials
   - Verify network connectivity

3. **"Permission denied"**
   - Check file permissions for redb database
   - Ensure write access to configuration directory

4. **"Invalid episode ID format"**
   - Use valid UUID format (e.g., `12345678-1234-1234-1234-123456789abc`)

### Performance Tuning

- Use `--limit` to control result set size
- Enable caching with redb for better performance
- Use `--dry-run` to test operations before execution
- Monitor storage health regularly

## Best Practices

1. **Configuration Management**
   - Use version-controlled configuration files
   - Separate development and production configs
   - Validate configuration before deployment

2. **Error Handling**
   - Always check exit codes in scripts
   - Use `--verbose` for debugging
   - Implement proper error recovery

3. **Performance**
   - Use appropriate limits for large datasets
   - Enable caching for frequent operations
   - Monitor storage health regularly

4. **Security**
   - Store tokens securely (environment variables or secure files)
   - Use least-privilege database permissions
   - Regularly rotate authentication tokens

## Support

For issues and questions:

1. Check this documentation first
2. Use `--help` for command-specific guidance
3. Enable `--verbose` for detailed error information
4. Check the main project documentation for architecture details
---

## Semantic Search with Embeddings

Memory CLI supports semantic similarity search using embeddings, allowing you to find relevant episodes based on meaning rather than just keywords.

### Quick Start

1. **Enable embeddings in your config file:**

```toml
[embeddings]
enabled = true
provider = "openai"  # or "local", "mistral", "azure", "custom"
model = "text-embedding-3-small"
dimension = 1536
api_key_env = "OPENAI_API_KEY"
```

2. **Set your API key (if using OpenAI/Mistral/Azure):**

```bash
export OPENAI_API_KEY="sk-your-key-here"
```

3. **Test your configuration:**

```bash
memory-cli embedding test
```

### Embedding Commands

#### `memory-cli embedding test`

Test your embedding provider configuration and connectivity.

**Example:**
```bash
$ memory-cli embedding test

🧪 Testing Embedding Provider Configuration
============================================================

📋 Configuration:
   Provider: openai
   Model: text-embedding-3-small
   Dimension: 1536
   Similarity Threshold: 0.7

🔌 Connecting to provider...
✅ Provider initialized: text-embedding-3-small
   Dimension: 1536

🧠 Testing single embedding generation...
✅ Embedding generated successfully
   Text: 'Implement REST API authentication with JWT tokens'
   Dimensions: 1536
   Time: 245ms

⚡ Testing batch embedding generation...
✅ Batch embeddings generated successfully
   Count: 3
   Time: 412ms
   Avg per text: 137ms

✨ All tests passed!
```

#### `memory-cli embedding config`

Show current embedding configuration.

**Example:**
```bash
$ memory-cli embedding config

⚙️  Embedding Configuration
============================================================

Status: ✅ Enabled

Provider Settings:
  provider: openai
  model: text-embedding-3-small
  dimension: 1536
  api_key_env: OPENAI_API_KEY (✅ Set)

Search Settings:
  similarity_threshold: 0.7
  batch_size: 32
  cache_embeddings: true
  timeout_seconds: 30
```

#### `memory-cli embedding list-providers`

List all available embedding providers with details.

**Example:**
```bash
$ memory-cli embedding list-providers

📚 Available Embedding Providers
============================================================

🏠 Local Provider
   • Model: sentence-transformers/all-MiniLM-L6-v2
   • Dimension: 384
   • Cost: Free (runs on your CPU)
   • Speed: Fast for small batches
   • Setup: Requires 'local-embeddings' feature

🌐 OpenAI Provider
   • Model: text-embedding-3-small (default)
   • Dimension: 1536
   • Cost: $0.02 per 1M tokens
   • Speed: Very fast (API-based)
   • Setup: Requires OPENAI_API_KEY
```

#### `memory-cli embedding benchmark`

Benchmark your embedding provider's performance.

**Example:**
```bash
$ memory-cli embedding benchmark

⚡ Benchmarking Embedding Provider
============================================================

Provider: openai (text-embedding-3-small)

📊 Single Embedding Benchmark
  Iterations: 10
  Average: 234ms
  Min: 198ms
  Max: 287ms

📊 Batch Embedding Benchmark
  Batch size 5: 512ms (102ms per item)
  Batch size 10: 891ms (89ms per item)
  Batch size 20: 1.54s (77ms per item)

✅ Benchmark complete!
```

### Provider Configuration Examples

#### Local Provider (Free, CPU-based)

```toml
[embeddings]
enabled = true
provider = "local"
model = "sentence-transformers/all-MiniLM-L6-v2"
dimension = 384
```

**Pros:** Free, no API key needed, works offline  
**Cons:** Slower than cloud providers, requires local-embeddings feature

#### OpenAI Provider (Recommended)

```toml
[embeddings]
enabled = true
provider = "openai"
model = "text-embedding-3-small"
dimension = 1536
api_key_env = "OPENAI_API_KEY"
```

```bash
export OPENAI_API_KEY="sk-your-key-here"
```

**Pros:** Fast, reliable, best quality  
**Cons:** Costs $0.02 per 1M tokens

#### Mistral Provider

```toml
[embeddings]
enabled = true
provider = "mistral"
model = "mistral-embed"
dimension = 1024
api_key_env = "MISTRAL_API_KEY"
```

```bash
export MISTRAL_API_KEY="your-mistral-key"
```

#### Azure OpenAI Provider

```toml
[embeddings]
enabled = true
provider = "azure"
model = "your-deployment-name"
dimension = 1536
api_key_env = "AZURE_OPENAI_API_KEY"
```

```bash
export AZURE_OPENAI_API_KEY="your-azure-key"
export AZURE_DEPLOYMENT="your-deployment"
export AZURE_RESOURCE="your-resource"
export AZURE_API_VERSION="2023-05-15"
```

#### Custom Provider (LM Studio, Ollama, etc.)

```toml
[embeddings]
enabled = true
provider = "custom"
model = "text-embedding-model"
dimension = 768
base_url = "http://localhost:1234/v1"
```

**Works with:** LM Studio, Ollama, LocalAI, or any OpenAI-compatible API

### Using Semantic Search

Once embeddings are enabled, episode search automatically uses semantic similarity:

```bash
# Search episodes by meaning, not just keywords
memory-cli episode search "user authentication" --limit 5

# The system will find episodes about:
# - "OAuth2 login flow"
# - "JWT token implementation"
# - "Session management"
# Even if they don't contain the exact words "user authentication"
```

### Troubleshooting

#### "Embeddings are disabled"
- Check your config file: `[embeddings] enabled = true`
- Or enable for current session: `memory-cli embedding enable`

#### "API error 401: Unauthorized"
- Verify your API key is set: `echo $OPENAI_API_KEY`
- Check the key is valid and not expired
- Ensure `api_key_env` matches your environment variable name

#### "API error 429: Rate limit exceeded"
- Reduce `batch_size` in config
- Add delays between requests
- Upgrade your API plan

#### "Failed to create HTTP client"
- Check internet connection
- Verify firewall allows HTTPS connections
- Check proxy settings if applicable

#### "Local embeddings not available"
- Compile with: `cargo build --features local-embeddings`
- Or switch to a cloud provider (OpenAI, Mistral)

### Performance Tips

1. **Enable caching:** `cache_embeddings = true` (default)
2. **Use batch operations:** Higher `batch_size` for bulk operations
3. **Choose the right provider:**
   - Local: Free but slower
   - OpenAI: Fast and reliable (recommended)
   - Custom: Depends on your setup

4. **Optimize similarity threshold:**
   - Lower (0.5-0.6): More results, less precise
   - Medium (0.7): Balanced (default)
   - Higher (0.8-0.9): Fewer results, more precise

### Cost Estimation (OpenAI)

- **Price:** $0.02 per 1 million tokens
- **Average episode:** ~100 tokens
- **1000 episodes:** ~$0.002 (less than a penny)
- **Caching:** Reduces costs by ~90% for repeated queries

### Security Best Practices

1. **Never commit API keys:** Use environment variables
2. **Use .env files:** Add `.env` to `.gitignore`
3. **Rotate keys regularly:** Especially if exposed
4. **Use read-only keys:** When possible
5. **Monitor usage:** Check your API provider dashboard

### Next Steps

- See `memory-core/EMBEDDING_PROVIDERS.md` for detailed provider docs
- See `memory-core/QUICK_START_EMBEDDINGS.md` for code examples
- Run `memory-cli embedding test` to verify your setup
- Try semantic search: `memory-cli episode search "your query"`

