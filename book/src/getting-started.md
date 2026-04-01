# Getting Started

## Installation

### From crates.io

```bash
cargo install do-memory-cli
cargo install do-memory-mcp-server
```

### From source

```bash
git clone https://github.com/d-o-hub/rust-self-learning-memory
cd rust-self-learning-memory
cargo build --release
```

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `TURSO_DATABASE_URL` | Turso database URL | Yes (or local) |
| `TURSO_AUTH_TOKEN` | Turso auth token | No (local dev) |
| `OPENAI_API_KEY` | OpenAI API key | For embeddings |

### Local Development

```bash
# Start local Turso
turso dev --db-file ./data/memory.db --port 8080

# Set environment
export TURSO_DATABASE_URL="http://127.0.0.1:8080"
export TURSO_AUTH_TOKEN=""
```

## First Episode

```bash
# Create an episode
do-memory-cli episode create --task "Implement feature X"

# Log steps
do-memory-cli episode log-step --episode-id <ID> --step "Read requirements"

# Complete the episode
do-memory-cli episode complete --episode-id <ID> --outcome success

# Extract patterns
do-memory-cli pattern extract --episode-id <ID>
```

## MCP Server

```bash
# Start the server
do-memory-mcp-server

# The server communicates via JSON-RPC over stdin/stdout
```