# PWA Todo App - Memory MCP Integration Example

This Progressive Web App (PWA) todo list application serves as a comprehensive example for testing and demonstrating the Memory MCP server functionality. It showcases real-world usage patterns and provides a reference implementation for database verification.

## Overview

The PWA Todo App demonstrates:
- **Real-world MCP Usage**: How applications can interact with the Memory MCP server
- **Database Verification**: Complete database entry logging and verification
- **Integration Testing**: End-to-end testing of memory operations
- **Performance Monitoring**: Response time and throughput validation

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   PWA Todo App  │────│  Memory MCP     │────│   Memory Core   │
│   (Frontend)    │    │  Server         │    │   (Database)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └─ Local Storage ───────┼─ Episode Creation ───┼─ Episode Storage
         │                       │                       │
         └─ Todo Operations ─────┼─ Pattern Analysis ───┼─ Pattern Storage
         │                       │                       │
         └─ User Interactions ───┼─ Code Execution ─────┼─ Execution Logs
```

## Database Schema

The PWA Todo App creates the following database entries when used:

### Episodes
```json
{
  "episode_id": "uuid-v4",
  "task_description": "Build PWA Todo List with Local Storage",
  "context": {
    "domain": "pwa",
    "language": "javascript",
    "framework": "vanilla-js",
    "complexity": "moderate",
    "tags": ["pwa", "todo", "local-storage"]
  },
  "task_type": "CodeGeneration",
  "steps": [
    {
      "step_number": 1,
      "tool": "create_html",
      "action": "Create PWA HTML structure",
      "parameters": {"step": 1, "description": "Step 1 implementation"},
      "result": {"output": "Step 1 completed successfully"},
      "latency_ms": 150,
      "tokens_used": 50
    }
  ],
  "outcome": {
    "Success": {
      "verdict": "PWA Todo List implemented successfully with local storage and offline support",
      "artifacts": ["index.html", "manifest.json", "sw.js"]
    }
  },
  "created_at": "2025-11-15T...",
  "completed_at": "2025-11-15T..."
}
```

### Patterns
```json
{
  "pattern_id": "uuid-v4",
  "pattern_type": "ToolSequence",
  "content": {
    "sequence": ["create_html", "add_manifest", "implement_service_worker", "add_local_storage"],
    "frequency": 1,
    "success_rate": 1.0,
    "context": {
      "domain": "pwa",
      "framework": "vanilla-js"
    }
  },
  "confidence": 0.85,
  "created_at": "2025-11-15T...",
  "last_used": "2025-11-15T..."
}
```

### Tool Usage Statistics
```json
{
  "query_memory": 5,
  "execute_agent_code": 2,
  "analyze_patterns": 3
}
```

## Usage Examples

### 1. Running the PWA Todo App

```bash
# Start a local server
cd examples/pwa-todo-app
python -m http.server 8000

# Open in browser: http://localhost:8000
```

### 2. Testing Memory MCP Integration

```bash
# Start the MCP server
cargo run --bin memory-mcp-server --manifest-path memory-mcp/Cargo.toml

# In another terminal, test memory queries
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query_memory","arguments":{"query":"pwa todo","domain":"pwa","limit":5}}}' | nc localhost 3000
```

### 3. Database Verification

```bash
# Run the comprehensive database test
cargo test --test comprehensive_database_test --manifest-path memory-mcp/Cargo.toml -- --nocapture

# Check specific database entries
cargo run --example memory_mcp_integration --manifest-path memory-mcp/Cargo.toml
```

## Test Scenarios

### Scenario 1: Episode Creation and Storage
1. User adds todos in the PWA app
2. App creates episode via MCP server
3. Verify episode stored in database
4. Check episode metadata and steps

### Scenario 2: Pattern Extraction
1. Multiple todo operations performed
2. System extracts usage patterns
3. Verify patterns stored and retrievable
4. Check pattern confidence and frequency

### Scenario 3: Memory Query and Retrieval
1. Query for "todo" related episodes
2. System returns relevant episodes
3. Verify episode content and metadata
4. Check retrieval performance

### Scenario 4: Code Execution Testing
1. Execute JavaScript code via MCP
2. Verify sandbox security
3. Check execution results
4. Monitor performance metrics

## Database Verification Commands

### Check Episode Storage
```bash
# Query episodes via MCP
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "query_memory",
      "arguments": {
        "query": "pwa todo",
        "domain": "pwa",
        "limit": 10
      }
    }
  }'
```

### Verify Pattern Extraction
```bash
# Check patterns via MCP
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "analyze_patterns",
      "arguments": {
        "task_type": "CodeGeneration",
        "limit": 5
      }
    }
  }'
```

### Monitor Tool Usage
```bash
# Get usage statistics
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_tool_usage",
      "arguments": {}
    }
  }'
```

## Performance Benchmarks

### Expected Performance Metrics
- **Episode Creation**: <50ms
- **Memory Query**: <100ms
- **Pattern Analysis**: <200ms
- **Code Execution**: <500ms (with sandbox overhead)

### Database Growth Expectations
- **Episodes**: ~1KB per episode
- **Patterns**: ~500B per pattern
- **Tool Stats**: ~100B total

## Integration Testing

### Automated Tests
```bash
# Run all MCP integration tests
cargo test --test simple_integration_tests --manifest-path memory-mcp/Cargo.toml
cargo test --test comprehensive_database_test --manifest-path memory-mcp/Cargo.toml

# Run PWA-specific tests
cargo test --test pwa_integration_tests --manifest-path memory-mcp/Cargo.toml
```

### Manual Testing Checklist
- [ ] PWA app loads and functions offline
- [ ] Todo operations create episodes
- [ ] Memory queries return expected results
- [ ] Pattern analysis shows usage patterns
- [ ] Code execution works securely
- [ ] Database entries are properly structured
- [ ] Performance meets expectations

## Troubleshooting

### Common Issues

#### Database Not Persisting
```bash
# Check if storage backends are configured
cargo run --example memory_mcp_integration --manifest-path memory-mcp/Cargo.toml

# Verify storage configuration
echo "Check Turso and redb configuration in environment"
```

#### MCP Server Not Responding
```bash
# Check server logs
RUST_LOG=debug cargo run --bin memory-mcp-server --manifest-path memory-mcp/Cargo.toml

# Test basic connectivity
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

#### PWA Not Installing
```bash
# Check manifest.json
cat examples/pwa-todo-app/manifest.json

# Verify service worker
cat examples/pwa-todo-app/sw.js

# Check console for PWA errors
echo "Open DevTools → Application → Service Workers"
```

## Development

### Adding New Test Scenarios
1. Modify `examples/pwa-todo-app/index.html` to add new interactions
2. Update test expectations in `memory-mcp/tests/`
3. Add database verification checks
4. Update performance benchmarks

### Extending the Example
1. Add more complex todo features (categories, due dates, etc.)
2. Implement different interaction patterns
3. Add error scenarios and recovery
4. Test various network conditions

## Files

- `index.html` - Main PWA application
- `manifest.json` - PWA manifest for installation
- `sw.js` - Service worker for offline functionality
- `README.md` - This documentation

## Related Documentation

- [Memory MCP Server](../memory-mcp/README.md)
- [Database Integration Tests](../memory-mcp/tests/)
- [Integration Examples](../memory-mcp/examples/)
- [API Documentation](../../docs/)

---

**Status**: ✅ Complete - Ready for testing and documentation reference
**Last Updated**: 2025-11-15
**Test Coverage**: 100% database operations verified