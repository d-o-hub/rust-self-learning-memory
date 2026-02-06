# MCP Token Reduction Phase 1 Plan

**Document Version**: 1.0
**Created**: 2026-01-31
**Document Type**: Detailed Implementation Plan
**Status**: ✅ Planning Complete - Ready for Implementation
**Priority**: P0 (Critical - 90-96% input + 20-60% output reduction)
**Dependencies**: [MCP_OPTIMIZATION_IMPLEMENTATION_ROADMAP.md](./MCP_OPTIMIZATION_IMPLEMENTATION_ROADMAP.md)

---

## Executive Summary

This document provides a detailed implementation plan for **Phase 1 (P0)** optimizations: Dynamic Tool Loading and Field Selection/Projection. These two optimizations can achieve **90-96% input token reduction and 20-60% output token reduction** with just 8-12 hours of implementation effort.

### Phase 1 Scope
- **Target**: 90-96% input reduction + 20-60% output reduction
- **Timeline**: 1-2 weeks (8-12 hours)
- **Risk**: Low (backwards compatible changes)
- **Impact**: Highest ROI of all optimization phases

### Prerequisites
- ✅ Memory-MCP server (v0.1.14) stable
- ✅ Test infrastructure available
- ✅ Storage backends (Turso, redb) operational
- ✅ Development environment configured

---

## Part 1: Dynamic Tool Loading

### Overview

**Token Savings**: 90-96% input reduction
**Implementation Effort**: 2-3 days (16-24 hours)
**Impact**: Critical - Highest ROI
**Complexity**: Medium
**Risk**: Low (backwards compatible)

### Current State Analysis

#### Problem: Eager Tool Schema Loading

The current MCP server implementation loads all tool schemas at startup:

```rust
// Current implementation (simplified)
async fn start_mcp_server() -> Result<(), Error> {
    // Load ALL tool schemas at startup
    let all_tools = load_all_tool_schemas().await?; // 20 tools

    // Register with MCP server
    server.register_tools(all_tools).await?;

    // Problem: Every client receives full schemas
}
```

**Current Behavior**:
```typescript
// Client connects
const tools = await mcp.list_tools();
// Response: 20 tool schemas × ~600 tokens each = ~12,000 tokens

// Client may only use 2-3 tools
// Wasted: 10,000+ tokens for unused tool descriptions
```

**Token Cost Breakdown**:
- Tool names (20 × 10 chars): ~200 tokens
- Tool descriptions (20 × 100 chars): ~2,000 tokens
- Input schemas (20 × 400 chars): ~8,000 tokens
- Output schemas (20 × 100 chars): ~2,000 tokens
- **Total**: ~12,000 tokens per session

#### Target State: Lazy Loading

Only load tool schemas when explicitly requested:

```typescript
// Optimized workflow
const toolNames = await mcp.list_tools();
// Response: 20 tool names × 10 chars = ~200 tokens

const querySchema = await mcp.describe_tool("query_memory");
// Response: 1 tool schema = ~600 tokens

// Total: 200 + 600 = 800 tokens (93% reduction)
```

**Token Cost Breakdown**:
- Tool names (20 × 10 chars): ~200 tokens
- Requested tool schema (1 × 600 chars): ~600 tokens
- **Total**: ~800 tokens for typical client using 1-3 tools

### Implementation Plan

#### Step 1.1: Architecture Design (Day 1, 4 hours)

**Objective**: Design the ToolRegistry architecture for lazy loading.

**File Structure**:
```
memory-mcp/src/server/tools/
├── mod.rs                    # Module exports
├── registry.rs               # NEW: ToolRegistry implementation
├── loader.rs                 # NEW: ToolLoader trait
└── cache.rs                  # NEW: Schema cache with TTL
```

**ToolRegistry Design**:

```rust
// memory-mcp/src/server/tools/registry.rs

use std::sync::OnceLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::server::tools::loader::ToolLoader;
use crate::server::tools::cache::SchemaCache;

/// Tool registry with lazy loading
pub struct ToolRegistry {
    /// Lazy-loaded tool registry (initialized on first access)
    tools: OnceLock<HashMap<String, Tool>>,
    /// Tool loader implementation
    loader: Arc<dyn ToolLoader>,
    /// Schema cache with 5-minute TTL
    cache: Arc<SchemaCache>,
}

impl ToolRegistry {
    /// Create new tool registry
    pub fn new(loader: Arc<dyn ToolLoader>) -> Self {
        Self {
            tools: OnceLock::new(),
            loader,
            cache: Arc::new(SchemaCache::new(Duration::from_secs(300))),
        }
    }

    /// List all tool names only (lightweight, no schemas)
    pub fn list_tool_names(&self) -> Vec<ToolStub> {
        vec![
            ToolStub {
                name: "create_episode".to_string(),
                description: "Create a new learning episode".to_string(),
            },
            ToolStub {
                name: "query_memory".to_string(),
                description: "Query episodic memory by filters".to_string(),
            },
            ToolStub {
                name: "analyze_patterns".to_string(),
                description: "Extract and analyze patterns from episodes".to_string(),
            },
            // ... all 20 tool stubs (~200 tokens)
        ]
    }

    /// Get tool by name, loading registry on first access
    pub async fn get_tool(&self, name: &str) -> Option<Arc<Tool>> {
        // Load registry on first access (lazy initialization)
        let registry = self.tools.get_or_init(|| {
            // Block on async load (use tokio runtime)
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(self.loader.load_tools())
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to load tools: {}", e);
                        HashMap::new()
                    })
            })
        });

        registry.get(name).cloned().map(Arc::new)
    }

    /// Get full tool schema (expensive, on-demand)
    pub async fn describe_tool(&self, name: &str) -> Result<ToolSchema, Error> {
        // Check cache first (5-minute TTL)
        if let Some(schema) = self.cache.get(name).await {
            tracing::debug!("Cache hit for tool: {}", name);
            return Ok(schema);
        }

        tracing::debug!("Cache miss for tool: {}, loading from registry", name);

        // Load from registry
        let tool = self.get_tool(name).await?
            .ok_or_else(|| Error::ToolNotFound(name.to_string()))?;

        let schema = tool.schema();

        // Cache for 5 minutes
        self.cache.insert(name.to_string(), schema.clone()).await;

        Ok(schema)
    }

    /// Check if registry is loaded
    pub fn is_loaded(&self) -> bool {
        self.tools.get().is_some()
    }
}

/// Lightweight tool stub (name + brief description only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStub {
    pub name: String,
    pub description: String,
}
```

**ToolLoader Trait**:

```rust
// memory-mcp/src/server/tools/loader.rs

use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;

use crate::server::tools::Tool;

/// Tool loader trait for lazy initialization
#[async_trait]
pub trait ToolLoader: Send + Sync {
    /// Load all tools (called once on first access)
    async fn load_tools(&self) -> Result<HashMap<String, Tool>, Error>;
}

/// Default tool loader implementation
pub struct DefaultToolLoader {
    /// Tool definitions
    tool_definitions: Vec<ToolDefinition>,
}

impl DefaultToolLoader {
    /// Create new tool loader from definitions
    pub fn new(tool_definitions: Vec<ToolDefinition>) -> Self {
        Self {
            tool_definitions,
        }
    }
}

#[async_trait]
impl ToolLoader for DefaultToolLoader {
    async fn load_tools(&self) -> Result<HashMap<String, Tool>, Error> {
        let mut tools = HashMap::new();

        for definition in &self.tool_definitions {
            let tool = Tool::from_definition(definition.clone()).await?;
            tools.insert(tool.name().to_string(), tool);
        }

        Ok(tools)
    }
}
```

**Schema Cache**:

```rust
// memory-mcp/src/server/tools/cache.rs

use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::server::tools::ToolSchema;

/// Schema cache with TTL
pub struct SchemaCache {
    /// Cached schemas with timestamps
    cache: Arc<RwLock<HashMap<String, CachedSchema>>>,
    /// Time-to-live for cache entries
    ttl: Duration,
}

struct CachedSchema {
    schema: ToolSchema,
    inserted_at: DateTime<Utc>,
}

impl SchemaCache {
    /// Create new schema cache
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Get schema from cache (if not expired)
    pub async fn get(&self, key: &str) -> Option<ToolSchema> {
        let cache = self.cache.read().await;
        let cached = cache.get(key)?;

        // Check if expired
        if Utc::now() - cached.inserted_at > self.ttl {
            return None;
        }

        Some(cached.schema.clone())
    }

    /// Insert schema into cache
    pub async fn insert(&self, key: String, schema: ToolSchema) {
        let mut cache = self.cache.write().await;
        cache.insert(key, CachedSchema {
            schema,
            inserted_at: Utc::now(),
        });
    }

    /// Clear expired entries
    pub async fn clear_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = Utc::now();

        cache.retain(|_, cached| now - cached.inserted_at <= self.ttl);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            total_entries: cache.len(),
            ttl_seconds: self.ttl.as_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub ttl_seconds: u64,
}
```

**Testing**: Write unit tests for ToolRegistry, ToolLoader, and SchemaCache

**Acceptance Criteria**:
- [ ] ToolRegistry compiles without errors
- [ ] ToolLoader trait defined
- [ ] SchemaCache with TTL implemented
- [ ] Unit tests passing

---

#### Step 1.2: Handler Integration (Day 1-2, 4 hours)

**Objective**: Update MCP handlers to use ToolRegistry.

**Current Handler**:
```rust
// Current: tools/list returns full schemas
async fn handle_list_tools(&self) -> Result<ListToolsResult, Error> {
    let tools = self.tool_registry.get_all_tools()?;
    Ok(ListToolsResult {
        tools: tools.into_iter().map(|t| t.schema()).collect()
    })
}
```

**Updated Handlers**:

```rust
// memory-mcp/src/server/handlers.rs

use crate::server::tools::registry::{ToolRegistry, ToolStub};

impl McpServer {
    /// Lightweight: Return tool names only (not full schemas)
    async fn handle_list_tools(&self) -> Result<ListToolsResult, Error> {
        let tool_stubs = self.tool_registry.list_tool_names();

        tracing::info!("Returning {} tool stubs (lightweight)", tool_stubs.len());

        Ok(ListToolsResult {
            tools: tool_stubs.into_iter()
                .map(|stub| ToolMetadata {
                    name: stub.name,
                    description: stub.description,
                })
                .collect()
        })
    }

    /// On-demand: Return full tool schema
    async fn handle_describe_tool(&self, request: DescribeToolRequest) -> Result<DescribeToolResult, Error> {
        let tool_name = request.tool_name;

        tracing::info!("Describing tool: {}", tool_name);

        let schema = self.tool_registry.describe_tool(&tool_name).await?;

        Ok(DescribeToolResult {
            tool_name,
            schema,
        })
    }

    /// Batch describe: Get multiple tool schemas (for clients that want more than 1)
    async fn handle_describe_tools(&self, request: DescribeToolsRequest) -> Result<DescribeToolsResult, Error> {
        let tool_names = request.tool_names;

        tracing::info!("Describing {} tools", tool_names.len());

        let mut schemas = Vec::new();

        for name in &tool_names {
            match self.tool_registry.describe_tool(name).await {
                Ok(schema) => {
                    schemas.push(ToolSchemaDescription {
                        tool_name: name.clone(),
                        schema,
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to describe tool {}: {}", name, e);
                    // Continue with other tools (don't fail entire request)
                }
            }
        }

        Ok(DescribeToolsResult {
            tools: schemas,
        })
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct DescribeToolRequest {
    pub tool_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DescribeToolsRequest {
    pub tool_names: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DescribeToolResult {
    pub tool_name: String,
    pub schema: ToolSchema,
}

#[derive(Debug, Serialize)]
pub struct DescribeToolsResult {
    pub tools: Vec<ToolSchemaDescription>,
}

#[derive(Debug, Serialize)]
pub struct ToolSchemaDescription {
    pub tool_name: String,
    pub schema: ToolSchema,
}
```

**MCP Server Integration**:

```rust
// memory-mcp/src/server.rs

use crate::server::tools::registry::ToolRegistry;
use crate::server::tools::loader::DefaultToolLoader;

pub struct McpServer {
    /// Tool registry with lazy loading
    tool_registry: Arc<ToolRegistry>,
    /// Other server components...
}

impl McpServer {
    /// Create new MCP server
    pub async fn new(config: ServerConfig) -> Result<Self, Error> {
        // Create tool loader with all tool definitions
        let loader = Arc::new(DefaultToolLoader::new(
            Self::load_tool_definitions()?
        ));

        // Create tool registry (not loaded yet!)
        let tool_registry = Arc::new(ToolRegistry::new(loader));

        Ok(Self {
            tool_registry,
            // ... other components
        })
    }

    /// Load tool definitions (not schemas, just metadata)
    fn load_tool_definitions() -> Result<Vec<ToolDefinition>, Error> {
        vec![
            ToolDefinition {
                name: "create_episode".to_string(),
                handler: Arc::new(handle_create_episode),
                // ... metadata (no full schema)
            },
            ToolDefinition {
                name: "query_memory".to_string(),
                handler: Arc::new(handle_query_memory),
                // ... metadata
            },
            // ... all 20 tool definitions
        ]
    }

    /// Start MCP server
    pub async fn start(&self) -> Result<(), Error> {
        // Register handlers
        self.router.register("tools/list", self.handle_list_tools());
        self.router.register("tools/describe", self.handle_describe_tool());
        self.router.register("tools/describe_batch", self.handle_describe_tools());

        // ... other routes

        Ok(())
    }
}
```

**Testing**: Update integration tests for new handler behavior

**Acceptance Criteria**:
- [ ] `tools/list` returns only tool stubs (~200 tokens)
- [ ] `tools/describe` returns full schema (~600 tokens)
- [ ] `tools/describe_batch` returns multiple schemas
- [ ] Integration tests passing

---

#### Step 1.3: Testing & Validation (Day 2-3, 8 hours)

**Objective**: Comprehensive testing to validate correctness and token reduction.

**Unit Tests**:

```rust
// memory-mcp/src/server/tools/registry/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::tools::loader::MockToolLoader;

    #[tokio::test]
    async fn test_lazy_loading() {
        let loader = Arc::new(MockToolLoader::new());
        let registry = ToolRegistry::new(loader);

        // Registry not loaded yet
        assert!(!registry.is_loaded());

        // First access triggers load
        let tool = registry.get_tool("query_memory").await;
        assert!(tool.is_some());
        assert!(registry.is_loaded());

        // Subsequent access uses cache
        let tool2 = registry.get_tool("query_memory").await;
        assert!(tool2.is_some());
    }

    #[tokio::test]
    async fn test_list_tool_names() {
        let loader = Arc::new(MockToolLoader::new());
        let registry = ToolRegistry::new(loader);

        let names = registry.list_tool_names();

        // Should return tool stubs (not full tools)
        assert!(!names.is_empty());
        assert_eq!(names.len(), 20); // All tools
        assert!(names.iter().all(|n| !n.name.is_empty()));
    }

    #[tokio::test]
    async fn test_describe_tool() {
        let loader = Arc::new(MockToolLoader::new());
        let registry = ToolRegistry::new(loader);

        let schema = registry.describe_tool("query_memory").await;

        assert!(schema.is_ok());
        let schema = schema.unwrap();

        // Should have full schema
        assert_eq!(schema.name, "query_memory");
        assert!(!schema.description.is_empty());
        assert!(schema.input_schema.is_object());
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let loader = Arc::new(MockToolLoader::new());
        let registry = ToolRegistry::new(loader);

        // First call (cache miss)
        let schema1 = registry.describe_tool("query_memory").await.unwrap();

        // Second call (cache hit)
        let schema2 = registry.describe_tool("query_memory").await.unwrap();

        // Should return same schema from cache
        assert_eq!(schema1.name, schema2.name);
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let loader = Arc::new(MockToolLoader::new());
        let registry = ToolRegistry::new(loader);

        let result = registry.describe_tool("nonexistent_tool").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ToolNotFound(_)));
    }
}
```

**Integration Tests**:

```rust
// memory-mcp/tests/integration/tool_loading.rs

use memory_mcp::test_utils::TestServer;

#[tokio::test]
async fn test_list_tools_lightweight() {
    let server = TestServer::new().await;

    // Call tools/list
    let response = server.call("tools/list", json!({})).await;

    // Should return tool stubs (not full schemas)
    assert!(response["tools"].is_array());

    let tools = response["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 20);

    // Count tokens in response
    let token_count = count_tokens(&response);
    assert!(token_count < 500, "Expected <500 tokens, got {}", token_count);

    // Verify stubs have name and brief description only
    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        // Should NOT have input_schema (that's in full schema)
        assert!(!tool.get("inputSchema").is_some());
    }
}

#[tokio::test]
async fn test_describe_tool_full_schema() {
    let server = TestServer::new().await;

    // Call tools/describe
    let response = server.call("tools/describe", json!({
        "tool_name": "query_memory"
    })).await;

    // Should return full schema
    assert_eq!(response["tool_name"], "query_memory");
    assert!(response["schema"]["name"].is_string());
    assert!(response["schema"]["description"].is_string());
    assert!(response["schema"]["inputSchema"].is_object());
    assert!(response["schema"]["outputSchema"].is_object());

    // Count tokens
    let token_count = count_tokens(&response);
    assert!(token_count < 1000, "Expected <1000 tokens, got {}", token_count);
}

#[tokio::test]
async fn test_token_reduction() {
    let server = TestServer::new().await;

    // Baseline: Old approach (load all tools)
    // Note: We can't actually test old approach, but we can simulate
    let all_tool_names = [
        "create_episode", "query_memory", "analyze_patterns",
        // ... all 20 tools
    ];

    let mut baseline_tokens = 0;
    for name in &all_tool_names {
        let response = server.call("tools/describe", json!({
            "tool_name": name
        })).await;
        baseline_tokens += count_tokens(&response);
    }

    // Optimized: Load only 2 tools
    let tool_names_response = server.call("tools/list", json!({})).await;
    let tool_names_tokens = count_tokens(&tool_names_response);

    let query_schema_response = server.call("tools/describe", json!({
        "tool_name": "query_memory"
    })).await;
    let query_tokens = count_tokens(&query_schema_response);

    let create_schema_response = server.call("tools/describe", json!({
        "tool_name": "create_episode"
    })).await;
    let create_tokens = count_tokens(&create_schema_response);

    let optimized_tokens = tool_names_tokens + query_tokens + create_tokens;

    // Calculate reduction
    let reduction = 1.0 - (optimized_tokens as f64 / baseline_tokens as f64);

    println!("Baseline tokens: {}", baseline_tokens);
    println!("Optimized tokens: {}", optimized_tokens);
    println!("Reduction: {:.1}%", reduction * 100.0);

    // Target: >90% reduction for 2 tools
    assert!(reduction > 0.90, "Expected >90% reduction, got {:.1}%", reduction * 100.0);
}

#[tokio::test]
async fn test_cache_effectiveness() {
    let server = TestServer::new().await;

    // First call (cache miss, loads from registry)
    let start = std::time::Instant::now();
    let _ = server.call("tools/describe", json!({
        "tool_name": "query_memory"
    })).await;
    let first_call_duration = start.elapsed();

    // Second call (cache hit)
    let start = std::time::Instant::now();
    let _ = server.call("tools/describe", json!({
        "tool_name": "query_memory"
    })).await;
    let second_call_duration = start.elapsed();

    println!("First call: {:?}", first_call_duration);
    println!("Second call: {:?}", second_call_duration);

    // Second call should be faster (cached)
    assert!(second_call_duration < first_call_duration);
}
```

**Performance Tests**:

```rust
// memory-mcp/benches/tool_loading.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use memory_mcp::server::tools::registry::ToolRegistry;

fn bench_lazy_loading(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("tool_loading");

    // Benchmark first access (cache miss)
    group.bench_function("first_access", |b| {
        let loader = Arc::new(MockToolLoader::new());
        let registry = ToolRegistry::new(loader);

        b.iter(|| {
            rt.block_on(async {
                black_box(registry.get_tool("query_memory").await)
            })
        });
    });

    // Benchmark subsequent access (cache hit)
    group.bench_function("cached_access", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let loader = Arc::new(MockToolLoader::new());
        let registry = ToolRegistry::new(loader);

        // Prime cache
        rt.block_on(async {
            registry.get_tool("query_memory").await;
        });

        b.iter(|| {
            rt.block_on(async {
                black_box(registry.get_tool("query_memory").await)
            })
        });
    });

    group.finish();
}

criterion_group!(benches, bench_lazy_loading);
criterion_main!(benches);
```

**Acceptance Criteria**:
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Token reduction: ≥90%
- [ ] First access latency: <20ms
- [ ] Cached access latency: <1ms
- [ ] Cache hit rate: >80%

---

#### Step 1.4: Documentation (Day 3, 2 hours)

**Objective**: Update documentation for lazy loading behavior.

**MCP Tool Reference**:

```markdown
# MCP Tool Reference

## Tool Discovery

### list_tools

Lightweight tool listing (returns only names and brief descriptions).

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "name": "create_episode",
        "description": "Create a new learning episode"
      },
      {
        "name": "query_memory",
        "description": "Query episodic memory by filters"
      }
    ]
  },
  "id": 1
}
```

**Token Cost**: ~200 tokens

**Note**: This does NOT include full tool schemas. Use `tools/describe` to get full schema.

### describe_tool

Get full tool schema (expensive, on-demand).

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "tools/describe",
  "params": {
    "tool_name": "query_memory"
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tool_name": "query_memory",
    "schema": {
      "name": "query_memory",
      "description": "Query episodic memory by filters",
      "inputSchema": { ... },
      "outputSchema": { ... }
    }
  },
  "id": 1
}
```

**Token Cost**: ~600 tokens per tool

**Caching**: Schemas are cached for 5 minutes.

### describe_tools (Batch)

Get multiple tool schemas in one request (more efficient than multiple `describe_tool` calls).

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "tools/describe_batch",
  "params": {
    "tool_names": ["query_memory", "create_episode", "analyze_patterns"]
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "tool_name": "query_memory",
        "schema": { ... }
      },
      {
        "tool_name": "create_episode",
        "schema": { ... }
      },
      {
        "tool_name": "analyze_patterns",
        "schema": { ... }
      }
    ]
  },
  "id": 1
}
```

**Token Cost**: ~600 tokens per tool

**Recommended**: Use this when you need schemas for 2+ tools.

## Best Practices

### For Tool Discovery

**DO**:
✅ Start with `tools/list` to get available tools
✅ Use `tools/describe` only for tools you actually use
✅ Use `tools/describe_batch` for 2+ tools
✅ Cache tool schemas client-side (don't re-request)

**DON'T**:
❌ Call `tools/describe` for all tools upfront
❌ Re-request schemas for every tool call
❌ Assume `tools/list` includes full schemas

### Example Workflow

```typescript
// Step 1: List available tools (lightweight)
const tools = await mcp.call_tool("tools/list", {});
// tokens: ~200

// Step 2: Find tools you need
const memoryTools = tools.filter(t => t.name.includes("memory"));

// Step 3: Get schemas only for tools you'll use
const schemas = await mcp.call_tool("tools/describe_batch", {
  tool_names: memoryTools.map(t => t.name)
});
// tokens: ~600 per tool

// Step 4: Use tools with cached schemas
const result = await mcp.call_tool("query_memory", params);
```

**Total tokens**: 200 + (600 × N tools)
**vs Old approach**: 12,000 tokens
**Reduction**: 90%+ for typical clients using 1-3 tools
```

**Migration Guide**:

```markdown
# Migration Guide: Tool Discovery Changes

## What Changed?

The MCP server now uses lazy loading for tool schemas:
- `tools/list` returns only tool names and brief descriptions (~200 tokens)
- `tools/describe` returns full tool schema on-demand (~600 tokens)

## Why?

To reduce token usage for clients that only use a subset of tools:
- **Before**: 12,000 tokens for all tool schemas
- **After**: 200 + (600 × tools used) = ~800 tokens for 1-3 tools
- **Reduction**: 90-96%

## How to Migrate?

### Before (Old Client)
```typescript
// Old: tools/list returned all schemas
const tools = await mcp.list_tools();
// tools had full schemas embedded

// Use tool directly
const result = await mcp.call_tool("query_memory", params);
```

### After (New Client)
```typescript
// New: tools/list returns names only
const tools = await mcp.call_tool("tools/list", {});

// Get schema for tool you want to use
const schema = await mcp.call_tool("tools/describe", {
  tool_name: "query_memory"
});

// Use tool
const result = await mcp.call_tool("query_memory", params);
```

### Optimized Approach (Recommended)
```typescript
// Batch fetch schemas for all tools you'll use
const schemas = await mcp.call_tool("tools/describe_batch", {
  tool_names: ["query_memory", "create_episode", "analyze_patterns"]
});

// Cache schemas client-side
const schemaCache = new Map(schemas.tools.map(t => [t.tool_name, t.schema]));

// Use tools with cached schemas
const result = await mcp.call_tool("query_memory", params);
```

## Backwards Compatibility

✅ **Existing clients continue to work**: If your client doesn't use `tools/describe`, it will still function (just with higher token usage).

✅ **Optional optimization**: Clients can adopt lazy loading gradually.

✅ **No breaking changes**: All existing APIs work unchanged.

## Questions?

- See [MCP Tool Reference](./MCP_TOOL_REFERENCE.md) for detailed API docs
- See [Performance Guide](./PERFORMANCE_GUIDE.md) for optimization tips
```

**Acceptance Criteria**:
- [ ] MCP tool reference updated
- [ ] Migration guide created
- [ ] Best practices documented
- [ ] Examples provided

---

## Part 2: Field Selection/Projection

### Overview

**Token Savings**: 20-60% output reduction
**Implementation Effort**: 1-2 days (8-16 hours)
**Impact**: High - Applicable to all tools
**Complexity**: Low
**Risk**: Low (backwards compatible)

### Current State Analysis

#### Problem: Complete Object Responses

Current MCP tools return complete objects with all fields:

```json
// Current: query_memory returns full episode
{
  "episode": {
    "id": "uuid-123",
    "task_description": "Implement feature X",
    "domain": "web-api",
    "task_type": "code_generation",
    "complexity": "moderate",
    "language": "rust",
    "framework": "actix-web",
    "tags": ["feature", "backend"],
    "status": "completed",
    "outcome_type": "success",
    "created_at": "2026-01-31T10:00:00Z",
    "completed_at": "2026-01-31T14:30:00Z",
    "duration_ms": 16200000,
    "reward_score": 0.95,
    "steps": [/* 50 steps */],
    "patterns": [/* 5 patterns */],
    "reflection": {/* ... */},
    "artifacts": [/* ... */],
    "relationships": [/* ... */]
  }
}
// Total: ~2,500 tokens
```

**Use Cases**:
- Client may only need `id`, `task_description`, `outcome_type` (150 tokens)
- Client may only need learning metrics: `reward_score`, `patterns`, `reflection` (600 tokens)
- Client may only need execution status: `status`, `outcome_type`, `created_at` (200 tokens)

**Wasted tokens**: 94% overhead for minimal queries

#### Target State: Field Projection

Add optional `include_fields` parameter:

```json
// Optimized: Request only needed fields
{
  "episode_id": "uuid-123",
  "include_fields": ["id", "task_description", "outcome_type"]
}

// Response:
{
  "episode": {
    "id": "uuid-123",
    "task_description": "Implement feature X",
    "outcome_type": "success"
  }
}
// Total: ~150 tokens (94% reduction)
```

### Implementation Plan

#### Step 2.1: Core Implementation (Day 1, 4 hours)

**Objective**: Implement field projection helper.

**File Structure**:
```
memory-mcp/src/common/
├── mod.rs              # Module exports
└── projection.rs       # NEW: Field projection helper
```

**Field Projection Helper**:

```rust
// memory-mcp/src/common/projection.rs

use serde::Serialize;
use serde_json::Value;
use anyhow::{Result, Error};

pub struct FieldProjection;

impl FieldProjection {
    /// Project specific fields from a serializable value
    ///
    /// # Arguments
    /// * `value` - The value to project fields from
    /// * `fields` - List of field names to include (empty = all fields)
    ///
    /// # Returns
    /// JSON value with only requested fields
    ///
    /// # Example
    /// ```rust
    /// let episode = Episode { ... };
    /// let fields = vec!["id".to_string(), "outcome_type".to_string()];
    /// let projected = FieldProjection::project(&episode, &fields)?;
    /// ```
    pub fn project<T: Serialize>(
        value: &T,
        fields: &[String],
    ) -> Result<Value, Error> {
        let full = serde_json::to_value(value)?;

        // If no fields specified, return all (backwards compatible)
        if fields.is_empty() {
            return Ok(full);
        }

        // Ensure value is an object
        let obj = full.as_object()
            .ok_or_else(|| Error::msg("Cannot project fields from non-object value"))?;

        // Filter to only requested fields
        let filtered: serde_json::Map<String, Value> = obj.iter()
            .filter(|(key, _)| fields.contains(key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();

        Ok(Value::Object(filtered))
    }

    /// Validate field names against available fields
    ///
    /// # Arguments
    /// * `fields` - Field names to validate
    /// * `available_fields` - All available field names
    ///
    /// # Returns
    /// Ok(()) if all fields valid, Err otherwise
    pub fn validate_fields(
        fields: &[String],
        available_fields: &[&str],
    ) -> Result<(), Error> {
        for field in fields {
            if !available_fields.contains(&field.as_str()) {
                return Err(Error::msg(format!("Invalid field: {}", field)));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestEpisode {
        id: String,
        task_description: String,
        domain: String,
        outcome_type: String,
        reward_score: f64,
    }

    #[test]
    fn test_project_no_fields() {
        let episode = TestEpisode {
            id: "123".to_string(),
            task_description: "Test task".to_string(),
            domain: "test".to_string(),
            outcome_type: "success".to_string(),
            reward_score: 0.95,
        };

        // No fields = all fields
        let result = FieldProjection::project(&episode, &[]).unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 5); // All fields
    }

    #[test]
    fn test_project_some_fields() {
        let episode = TestEpisode {
            id: "123".to_string(),
            task_description: "Test task".to_string(),
            domain: "test".to_string(),
            outcome_type: "success".to_string(),
            reward_score: 0.95,
        };

        // Project only 2 fields
        let fields = vec!["id".to_string(), "outcome_type".to_string()];
        let result = FieldProjection::project(&episode, &fields).unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("outcome_type"));
        assert!(!obj.contains_key("task_description"));
    }

    #[test]
    fn test_project_non_object() {
        let value = serde_json::json!(42);

        let result = FieldProjection::project(&value, &["id".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_fields_valid() {
        let fields = vec!["id".to_string(), "outcome_type".to_string()];
        let available = vec!["id", "task_description", "outcome_type"];

        let result = FieldProjection::validate_fields(&fields, &available);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fields_invalid() {
        let fields = vec!["id".to_string(), "invalid_field".to_string()];
        let available = vec!["id", "task_description", "outcome_type"];

        let result = FieldProjection::validate_fields(&fields, &available);
        assert!(result.is_err());
    }
}
```

**Testing**: Comprehensive unit tests for projection logic

**Acceptance Criteria**:
- [ ] `FieldProjection::project()` implemented
- [ ] `FieldProjection::validate_fields()` implemented
- [ ] Unit tests passing
- [ ] Edge cases handled (empty fields, invalid fields, non-objects)

---

#### Step 2.2: Tool Handler Updates (Day 1-2, 4 hours)

**Objective**: Add `include_fields` parameter to all tool handlers.

**Example: query_memory Tool**:

```rust
// memory-mcp/src/server/tools/episode/query.rs

use crate::common::projection::FieldProjection;

/// Query memory request
#[derive(Debug, Deserialize)]
pub struct QueryMemoryRequest {
    pub episode_id: String,
    /// Optional: Return only specified fields
    pub include_fields: Option<Vec<String>>,
}

/// Query memory response
#[derive(Debug, Serialize)]
pub struct QueryMemoryResponse {
    pub episode: serde_json::Value, // Projected or full
}

/// Available fields for query_memory
pub const QUERY_MEMORY_FIELDS: &[&str] = &[
    // Episode metadata
    "id",
    "task_description",
    "domain",
    "task_type",
    "complexity",
    "language",
    "framework",
    "tags",

    // Execution status
    "outcome_type",
    "status",
    "created_at",
    "completed_at",
    "duration_ms",

    // Learning metrics
    "reward_score",
    "patterns",
    "reflection",

    // Execution details
    "steps",
    "artifacts",
    "relationships",
];

/// Handle query_memory
pub async fn handle_query_memory(
    req: QueryMemoryRequest,
    ctx: &Context,
) -> Result<QueryMemoryResponse, Error> {
    // Validate fields if provided
    if let Some(ref fields) = req.include_fields {
        FieldProjection::validate_fields(fields, QUERY_MEMORY_FIELDS)?;
    }

    // Fetch episode from storage
    let episode = ctx.storage.get_episode(&req.episode_id).await?;

    // Apply field projection if requested
    let episode_value = if let Some(fields) = req.include_fields {
        FieldProjection::project(&episode, &fields)?
    } else {
        // No fields specified = all fields (backwards compatible)
        serde_json::to_value(&episode)?
    };

    Ok(QueryMemoryResponse {
        episode: episode_value,
    })
}
```

**Update All Tools**:

```rust
// memory-mcp/src/server/tools/mod.rs

// Episode tools
pub mod episode;
use episode::query::{QUERY_MEMORY_FIELDS, GET_EPISODE_FIELDS};
use episode::create::{CREATE_EPISODE_FIELDS};
use episode::delete::DELETE_EPISODE_FIELDS;

// Pattern tools
pub mod patterns;
use patterns::analyze::{ANALYZE_PATTERNS_FIELDS};
use patterns::search::{SEARCH_PATTERNS_FIELDS};
use patterns::recommend::{RECOMMEND_PATTERNS_FIELDS};

// Batch tools
pub mod batch;
use batch::bulk::{BULK_EPISODES_FIELDS};

// Macro to add include_fields to all tools
macro_rules! add_include_fields {
    ($request_type:ty, $fields:expr) => {
        impl $request_type {
            pub fn validate_fields(&self) -> Result<(), Error> {
                if let Some(ref fields) = self.include_fields {
                    FieldProjection::validate_fields(fields, $fields)?;
                }
                Ok(())
            }
        }
    };
}

// Apply to all tools
add_include_fields!(QueryMemoryRequest, QUERY_MEMORY_FIELDS);
add_include_fields!(GetEpisodeRequest, GET_EPISODE_FIELDS);
add_include_fields!(AnalyzePatternsRequest, ANALYZE_PATTERNS_FIELDS);
// ... etc for all 20 tools
```

**Tool Handler Template**:

```rust
// Generic template for all tools
pub async fn handle_tool<T>(
    req: ToolRequest<T>,
    ctx: &Context,
    handler: impl FnOnce(T, &Context) -> Result<ToolResponse, Error>,
) -> Result<ToolResponse, Error>
where
    T: IncludeFields + ValidateFields,
{
    // Validate fields
    req.validate_fields()?;

    // Execute handler
    let response = handler(req.into_inner(), ctx)?;

    // Apply field projection
    let projected = if let Some(fields) = req.include_fields() {
        FieldProjection::project(&response, &fields)?
    } else {
        serde_json::to_value(&response)?
    };

    Ok(ToolResponse::new(projected))
}
```

**Acceptance Criteria**:
- [ ] All 20 tools updated with `include_fields` parameter
- [ ] Field validation implemented
- [ ] Field projection applied in all handlers
- [ ] Backwards compatible (empty fields = all fields)

---

#### Step 2.3: Field Documentation (Day 2, 4 hours)

**Objective**: Document available fields for each tool.

**Field Documentation Template**:

```markdown
## query_memory - Available Fields

### Episode Metadata

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `id` | string | Episode UUID | "uuid-123" |
| `task_description` | string | Human-readable task description | "Implement feature X" |
| `domain` | string | Task domain | "web-api", "cli", "data-processing" |
| `task_type` | string | Type of task | "code_generation", "debugging", "testing" |
| `complexity` | string | Task complexity | "simple", "moderate", "complex" |
| `language` | string | Programming language | "rust", "python", "typescript" |
| `framework` | string | Framework used | "actix-web", "tokio", "async" |
| `tags` | array<string> | Context tags | ["feature", "backend", "api"] |

### Execution Status

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `outcome_type` | string | Episode outcome | "success", "failure", "partial_success" |
| `status` | string | Current status | "completed", "in_progress", "failed" |
| `created_at` | string | ISO 8601 creation timestamp | "2026-01-31T10:00:00Z" |
| `completed_at` | string | ISO 8601 completion timestamp (optional) | "2026-01-31T14:30:00Z" |
| `duration_ms` | number | Duration in milliseconds | 16200000 |

### Learning Metrics

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `reward_score` | number | Learning reward (0.0-1.0) | 0.95 |
| `patterns` | array | Extracted patterns | [Pattern objects] |
| `reflection` | object | Episode reflection analysis | {insights, lessons_learned} |

### Execution Details

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `steps` | array | Execution steps (50+ steps) | [Step objects] |
| `artifacts` | array | Produced artifacts | [Artifact objects] |
| `relationships` | array | Related episodes | [Relationship objects] |

### Usage Examples

#### Minimal Query (Outcome Only)
```json
{
  "episode_id": "uuid-123",
  "include_fields": ["id", "task_description", "outcome_type"]
}
```
**Response**: ~150 tokens (94% reduction)

#### Learning Metrics Only
```json
{
  "episode_id": "uuid-123",
  "include_fields": ["reward_score", "patterns", "reflection"]
}
```
**Response**: ~600 tokens (76% reduction)

#### Execution Status
```json
{
  "episode_id": "uuid-123",
  "include_fields": ["status", "outcome_type", "created_at", "completed_at"]
}
```
**Response**: ~200 tokens (92% reduction)

#### All Fields (Backwards Compatible)
```json
{
  "episode_id": "uuid-123"
}
```
**Response**: ~2,500 tokens (same as before)

### Token Reduction Examples

| Scenario | Fields Selected | Before | After | Reduction |
|----------|----------------|--------|-------|-----------|
| Minimal | 3 fields | 2,500 | 150 | 94% |
| Learning | 3 fields | 2,500 | 600 | 76% |
| Status | 4 fields | 2,500 | 200 | 92% |
| All | 20 fields | 2,500 | 2,500 | 0% |

**Average**: 20-60% reduction across typical use cases
```

**Create Field Reference Document**:

```markdown
# MCP Tool Field Reference

Complete reference for all available fields in MCP tool responses.

## Episode Tools

### create_episode

**Input fields**: See tool schema
**Output fields**:
- `id` (string): Created episode UUID

### get_episode

**Available fields**: Same as `query_memory` (see above)

### query_memory

**Available fields**: [See detailed documentation above](#query_memory---available-fields)

### delete_episode

**Output fields**:
- `deleted` (boolean): Whether episode was deleted
- `id` (string): Deleted episode ID

### bulk_episodes

**Available fields**: Array of episodes (same fields as `query_memory`)

## Pattern Tools

### analyze_patterns

**Output fields**:
- `patterns` (array): Extracted patterns
  - `id` (string): Pattern UUID
  - `pattern_type` (string): Pattern type
  - `description` (string): Pattern description
  - `success_rate` (number): Success rate (0.0-1.0)
  - `usage_count` (number): Number of times used
  - `domain` (string): Domain
  - `task_type` (string): Task type
  - `created_at` (string): Creation timestamp
  - `last_used` (string): Last usage timestamp
- `analysis_metadata` (object): Analysis metadata
  - `total_episodes` (number): Episodes analyzed
  - `duration_ms` (number): Analysis duration

### search_patterns

**Output fields**: Same as `analyze_patterns` (filtered patterns)

### recommend_patterns

**Output fields**: Same as `analyze_patterns` (recommended patterns)

## Batch Tools

### batch_create_episodes

**Output fields**:
- `episodes` (array): Created episodes
  - `id` (string): Episode UUID
  - `status` (string): Creation status
- `summary` (object):
  - `total` (number): Total requested
  - `succeeded` (number): Successfully created
  - `failed` (number): Failed to create

### batch_add_steps

**Output fields**:
- `steps` (array): Added steps
  - `episode_id` (string): Episode ID
  - `step_number` (number): Step number
  - `status` (string): Addition status
- `summary` (object):
  - `total_steps` (number): Total steps added
  - `episodes_affected` (number): Episodes updated

### batch_complete_episodes

**Output fields**:
- `episodes` (array): Completed episodes
  - `id` (string): Episode UUID
  - `outcome_type` (string): Episode outcome
  - `reward_score` (number): Learning reward
- `summary` (object):
  - `total` (number): Total requested
  - `succeeded` (number): Successfully completed
  - `failed` (number): Failed to complete

## Advanced Tools

### advanced_pattern_analysis

**Output fields**:
- `statistical_analysis` (object):
  - `mean_success_rate` (number): Mean success rate
  - `median_success_rate` (number): Median success rate
  - `std_dev` (number): Standard deviation
  - `confidence_interval` (array): 95% CI
- `trend_analysis` (object):
  - `trend` (string): "improving", "stable", "declining"
  - `slope` (number): Trend slope
- `correlations` (array): Feature correlations

### quality_metrics

**Output fields**:
- `overall_quality` (number): Overall quality score (0.0-1.0)
- `dimension_scores` (object):
  - `completeness` (number): Completeness score
  - `accuracy` (number): Accuracy score
  - `consistency` (number): Consistency score
  - `relevance` (number): Relevance score
- `trends` (object):
  - `quality_trend` (string): Quality trend
  - `improvement_rate` (number): Improvement rate

### health_check

**Output fields**:
- `status` (string): System health status
- `components` (object):
  - `storage` (object): Storage status
  - `cache` (object): Cache status
  - `embeddings` (object): Embeddings status
- `metrics` (object):
  - `uptime_ms` (number): Server uptime
  - `request_count` (number): Total requests
  - `error_rate` (number): Error rate

## Common Patterns

### Episode Fields (Used by Multiple Tools)

The following fields are available for tools that return episode data:

**Metadata**: `id`, `task_description`, `domain`, `task_type`, `complexity`, `language`, `framework`, `tags`
**Status**: `outcome_type`, `status`, `created_at`, `completed_at`, `duration_ms`
**Learning**: `reward_score`, `patterns`, `reflection`
**Details**: `steps`, `artifacts`, `relationships`

### Pattern Fields (Used by Multiple Tools)

The following fields are available for tools that return pattern data:

**Basic**: `id`, `pattern_type`, `description`, `success_rate`, `usage_count`
**Context**: `domain`, `task_type`, `created_at`, `last_used`

## Best Practices

### Field Selection

**DO**:
✅ Request only fields you need
✅ Use field selection for list views (id, description, status)
✅ Use field selection for analytics (reward_score, patterns)
✅ Test token reduction with different field combinations

**DON'T**:
❌ Request all fields if you only need a few
❌ Request nested objects if you only need top-level fields
❌ Assume all tools have the same fields (check documentation)

### Example Queries

```typescript
// Minimal: Get only outcome
const minimal = await mcp.call_tool("query_memory", {
  episode_id: "uuid-123",
  include_fields: ["id", "task_description", "outcome_type"]
});

// Learning-focused: Get metrics only
const learning = await mcp.call_tool("query_memory", {
  episode_id: "uuid-123",
  include_fields: ["reward_score", "patterns", "reflection"]
});

// Status-focused: Get execution status
const status = await mcp.call_tool("query_memory", {
  episode_id: "uuid-123",
  include_fields: ["status", "outcome_type", "created_at", "completed_at"]
});

// Full: Get all fields (backwards compatible)
const full = await mcp.call_tool("query_memory", {
  episode_id: "uuid-123"
  // include_fields omitted = all fields
});
```

## Token Reduction Calculator

Use this calculator to estimate token reduction for field selection:

| Scenario | Fields | Before | After | Reduction |
|----------|--------|--------|-------|-----------|
| Minimal outcome | 3 | 2,500 | 150 | 94% |
| Learning metrics | 3 | 2,500 | 600 | 76% |
| Execution status | 4 | 2,500 | 200 | 92% |
| Metadata only | 8 | 2,500 | 800 | 68% |
| All fields | 20 | 2,500 | 2,500 | 0% |

**Formula**: `Reduction % = (1 - After/Before) × 100`

**Typical savings**: 20-60% across common use cases
```

**Acceptance Criteria**:
- [ ] All 20 tools have field documentation
- [ ] Field reference guide created
- [ ] Usage examples provided
- [ ] Token reduction calculator included

---

#### Step 2.4: Testing & Validation (Day 2, 4 hours)

**Objective**: Validate field projection implementation.

**Unit Tests**:

```rust
// memory-mcp/src/common/projection/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    // ... (already included in Step 2.1)
}
```

**Integration Tests**:

```rust
// memory-mcp/tests/integration/field_projection.rs

use memory_mcp::test_utils::TestServer;

#[tokio::test]
async fn test_field_projection_minimal() {
    let server = TestServer::new().await;

    // Request minimal fields
    let response = server.call("query_memory", json!({
        "episode_id": "test-episode-1",
        "include_fields": ["id", "task_description", "outcome_type"]
    })).await;

    // Verify only requested fields returned
    let episode = &response["episode"];
    assert!(episode["id"].is_string());
    assert!(episode["task_description"].is_string());
    assert!(episode["outcome_type"].is_string());

    // Verify other fields NOT returned
    assert!(!episode.get("domain").is_some());
    assert!(!episode.get("patterns").is_some());
    assert!(!episode.get("steps").is_some());

    // Count tokens
    let token_count = count_tokens(&response);
    assert!(token_count < 300, "Expected <300 tokens, got {}", token_count);
}

#[tokio::test]
async fn test_field_projection_learning_metrics() {
    let server = TestServer::new().await;

    // Request learning metrics
    let response = server.call("query_memory", json!({
        "episode_id": "test-episode-1",
        "include_fields": ["reward_score", "patterns", "reflection"]
    })).await;

    // Verify learning fields returned
    let episode = &response["episode"];
    assert!(episode["reward_score"].is_number());
    assert!(episode["patterns"].is_array());
    assert!(episode["reflection"].is_object());

    // Verify other fields NOT returned
    assert!(!episode.get("id").is_some());
    assert!(!episode.get("steps").is_some());

    // Count tokens
    let token_count = count_tokens(&response);
    assert!(token_count < 800, "Expected <800 tokens, got {}", token_count);
}

#[tokio::test]
async fn test_field_projection_all_fields() {
    let server = TestServer::new().await;

    // No include_fields = all fields (backwards compatible)
    let response = server.call("query_memory", json!({
        "episode_id": "test-episode-1"
    })).await;

    // Verify all fields present
    let episode = &response["episode"];
    assert!(episode["id"].is_string());
    assert!(episode["task_description"].is_string());
    assert!(episode["domain"].is_string());
    assert!(episode["outcome_type"].is_string());
    assert!(episode["reward_score"].is_number());
    assert!(episode["patterns"].is_array());
    assert!(episode["steps"].is_array());

    // Should have all 20 fields
    let field_count = episode.as_object().unwrap().len();
    assert_eq!(field_count, 20);
}

#[tokio::test]
async fn test_field_projection_invalid_field() {
    let server = TestServer::new().await;

    // Request invalid field
    let response = server.call("query_memory", json!({
        "episode_id": "test-episode-1",
        "include_fields": ["id", "invalid_field"]
    })).await;

    // Should return error
    assert!(response["error"].is_object());
    assert!(response["error"]["message"].as_str().unwrap().contains("Invalid field"));
}

#[tokio::test]
async fn test_field_projection_multiple_tools() {
    let server = TestServer::new().await;

    // Test field projection works for all tools
    let tools_with_field_projection = vec![
        ("query_memory", json!({"episode_id": "test-1"})),
        ("get_episode", json!({"episode_id": "test-1"})),
        ("analyze_patterns", json!({"filter": {"domain": "web-api"}})),
        ("bulk_episodes", json!({"limit": 10})),
    ];

    for (tool, params) in tools_with_field_projection {
        let mut params_with_fields = params.clone();
        params_with_fields["include_fields"] = json!(["id", "description"]);

        let response = server.call(tool, params_with_fields).await;

        // Should succeed
        assert!(!response.get("error").is_some(), "Tool {} failed", tool);

        // Should have only requested fields
        // (exact validation depends on tool)
    }
}

#[tokio::test]
async fn test_token_reduction_measurement() {
    let server = TestServer::new().await;

    // Baseline: All fields
    let full_response = server.call("query_memory", json!({
        "episode_id": "test-episode-1"
    })).await;
    let full_tokens = count_tokens(&full_response);

    // Optimized: 3 fields
    let minimal_response = server.call("query_memory", json!({
        "episode_id": "test-episode-1",
        "include_fields": ["id", "task_description", "outcome_type"]
    })).await;
    let minimal_tokens = count_tokens(&minimal_response);

    // Calculate reduction
    let reduction = 1.0 - (minimal_tokens as f64 / full_tokens as f64);

    println!("Full response: {} tokens", full_tokens);
    println!("Minimal response: {} tokens", minimal_tokens);
    println!("Reduction: {:.1}%", reduction * 100.0);

    // Target: >90% reduction for 3 fields
    assert!(reduction > 0.90, "Expected >90% reduction, got {:.1}%", reduction * 100.0);
}
```

**Performance Tests**:

```rust
// memory-mcp/benches/field_projection.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memory_mcp::common::projection::FieldProjection;

#[derive(serde::Serialize)]
struct TestEpisode {
    id: String,
    task_description: String,
    domain: String,
    outcome_type: String,
    reward_score: f64,
    // ... more fields
}

fn bench_field_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("field_projection");

    let episode = TestEpisode {
        id: "test-id".to_string(),
        task_description: "Test task".to_string(),
        domain: "test".to_string(),
        outcome_type: "success".to_string(),
        reward_score: 0.95,
    };

    // Benchmark: No projection (all fields)
    group.bench_function("no_projection", |b| {
        b.iter(|| {
            black_box(FieldProjection::project(&episode, &[]).unwrap())
        })
    });

    // Benchmark: 3 fields
    group.bench_function("project_3_fields", |b| {
        let fields = vec!["id".to_string(), "task_description".to_string(), "outcome_type".to_string()];
        b.iter(|| {
            black_box(FieldProjection::project(&episode, &fields).unwrap())
        })
    });

    // Benchmark: 10 fields
    group.bench_function("project_10_fields", |b| {
        let fields = vec![
            "id".to_string(),
            "task_description".to_string(),
            "domain".to_string(),
            "task_type".to_string(),
            "outcome_type".to_string(),
            "status".to_string(),
            "created_at".to_string(),
            "reward_score".to_string(),
            "patterns".to_string(),
            "reflection".to_string(),
        ];
        b.iter(|| {
            black_box(FieldProjection::project(&episode, &fields).unwrap())
        })
    });

    group.finish();
}

criterion_group!(benches, bench_field_projection);
criterion_main!(benches);
```

**Acceptance Criteria**:
- [ ] All integration tests passing
- [ ] Token reduction: ≥20% (measured across use cases)
- [ ] Field projection overhead: <1ms
- [ ] All tools validated with field projection
- [ ] Error handling for invalid fields

---

## Integration Considerations

### Backwards Compatibility

**Principle**: All changes are backwards compatible. Existing clients work without modification.

**Implementation**:
- `include_fields` is optional parameter
- Default (empty or omitted) = all fields (current behavior)
- No breaking changes to existing APIs

**Examples**:
```typescript
// Old client (no changes)
const result = await mcp.call_tool("query_memory", {
  episode_id: "uuid-123"
});
// Works: Returns all fields (same as before)

// New client (optimized)
const result = await mcp.call_tool("query_memory", {
  episode_id: "uuid-123",
  include_fields: ["id", "task_description", "outcome_type"]
});
// Optimized: Returns only requested fields
```

### Performance Impact

**Overhead Analysis**:

| Operation | Overhead | Acceptable |
|-----------|----------|------------|
| Field projection | <1ms | ✅ Yes |
| Field validation | <0.5ms | ✅ Yes |
| JSON serialization | <2ms | ✅ Yes |
| **Total** | **<3.5ms** | ✅ Yes |

**Mitigation**:
- Optimized projection implementation (O(n) where n = fields)
- Cache field validation results
- Use efficient JSON library (serde_json)

### Client Impact

**Adoption Strategy**:
1. **Optional**: Clients can adopt gradually
2. **Transparent**: No changes required for basic usage
3. **Opt-in**: Advanced clients can use field selection
4. **Documented**: Clear examples and best practices

**Client Migration Path**:
```typescript
// Phase 1: Existing clients work unchanged
const result = await mcp.call_tool("query_memory", { episode_id: "123" });

// Phase 2: Add field selection where beneficial
const minimal = await mcp.call_tool("query_memory", {
  episode_id: "123",
  include_fields: ["id", "outcome_type"]
});

// Phase 3: Adopt across all tool calls
const optimized = await mcp.call_tool("query_memory", {
  episode_id: "123",
  include_fields: ["id", "task_description", "outcome_type"]
});
```

---

## Success Criteria

### Phase 1 Complete Checklist

#### Dynamic Tool Loading
- [ ] ToolRegistry implemented with lazy loading
- [ ] `tools/list` returns only tool stubs (~200 tokens)
- [ ] `tools/describe` returns full schema on-demand (~600 tokens)
- [ ] `tools/describe_batch` for efficient multi-tool schema loading
- [ ] Schema cache with 5-minute TTL
- [ ] Unit tests for lazy loading
- [ ] Integration tests for token reduction
- [ ] Performance tests (<20ms first, <1ms cached)
- [ ] Documentation updated
- [ ] Migration guide created

**Token Reduction Target**: ≥90% (12,000 → <500 tokens for typical clients)
**Performance Target**: <20ms first access, <1ms subsequent
**Cache Hit Rate**: >80%

#### Field Selection/Projection
- [ ] `FieldProjection` helper implemented
- [ ] `include_fields` parameter added to all 20 tools
- [ ] Field validation for each tool
- [ ] Unit tests for projection logic
- [ ] Integration tests for all tools
- [ ] Field documentation for all tools
- [ ] Field reference guide created
- [ ] Usage examples provided
- [ ] Token reduction calculator

**Token Reduction Target**: ≥20% (varies by field selection)
**Performance Target**: <1ms overhead per response
**Backwards Compatibility**: 100% (no breaking changes)

#### Overall Phase 1
- [ ] All tests passing (unit + integration)
- [ ] Token reduction measured and validated
- [ ] Performance targets met
- [ ] Documentation complete
- [ ] Migration guide published
- [ ] Client examples provided
- [ ] Quality gates passed

**Combined Token Reduction**: 90-96% input + 20-60% output
**Overall Timeline**: 1-2 weeks (8-12 hours)
**Risk**: Low (backwards compatible)

---

## Testing Strategy

### Unit Testing

**Coverage**: >90% for new code

**Scope**:
- ToolRegistry lazy initialization
- SchemaCache TTL behavior
- FieldProjection logic
- Field validation

**Tools**: Rust built-in test framework

### Integration Testing

**Coverage**: All 20 tools

**Scope**:
- `tools/list` behavior
- `tools/describe` behavior
- Field projection for each tool
- Token reduction measurement
- Performance benchmarks

**Tools**: Integration test suite with TestServer

### Performance Testing

**Metrics**:
- Latency (first access, cached access)
- Token count (before/after)
- Throughput (requests/second)

**Tools**: Criterion benchmark suite

### A/B Testing

**Method**: Compare before/after implementations

**Metrics**:
- Token count: `count_tokens(response)`
- Latency: `std::time::Instant::now()`
- Quality: Functional correctness

**Validation**: Ensure optimizations don't break functionality

---

## Documentation Plan

### API Documentation

**Updates Required**:
- MCP tool reference (all 20 tools)
- Field reference guide
- JSON schemas for `include_fields` parameter

**Format**: Markdown + inline Rust documentation

### User Guides

**Documents to Create**:
- Migration guide (tool discovery changes)
- Field selection best practices
- Token optimization guide
- Performance tuning guide

**Format**: Markdown with code examples

### Developer Documentation

**Documents to Update**:
- Architecture documentation (ToolRegistry, FieldProjection)
- Code examples (implementation patterns)
- Testing guide (how to test optimizations)

**Format**: Markdown + Rust documentation comments

---

## Rollout Plan

### Phase 1A: Development (Week 1)
- Day 1-3: Implement dynamic tool loading
- Day 4-5: Implement field selection/projection
- Day 6-7: Testing and documentation

### Phase 1B: Staging (Week 2)
- Deploy to staging environment
- Internal testing with production-like workload
- Measure actual token reduction
- Performance validation

### Phase 1C: Production (Week 2-3)
- Gradual rollout (10% → 50% → 100%)
- Monitor metrics (tokens, latency, errors)
- Collect user feedback
- Rollback plan if issues

### Success Metrics

**Token Reduction**:
- Input: ≥90% reduction measured
- Output: ≥20% reduction measured

**Performance**:
- No latency regression (>10ms)
- Cache hit rate >80%

**Quality**:
- All tests passing
- Zero breaking changes
- Client adoption smooth

---

## Conclusion

This detailed implementation plan provides step-by-step guidance for implementing Phase 1 (P0) optimizations: Dynamic Tool Loading and Field Selection/Projection.

### Key Deliverables

1. **Dynamic Tool Loading**: ToolRegistry with lazy loading, schema cache
2. **Field Selection/Projection**: Field projection helper, all tools updated
3. **Comprehensive Testing**: Unit, integration, performance tests
4. **Documentation**: API docs, migration guides, examples

### Expected Impact

- **Input Token Reduction**: 90-96% (12,000 → <500 tokens)
- **Output Token Reduction**: 20-60% (varies by usage)
- **Combined Impact**: 95% overall reduction for typical workflows
- **Performance**: <20ms overhead acceptable
- **Risk**: Low (backwards compatible)

### Next Steps

1. ✅ Begin implementation following this plan
2. ✅ Measure and validate each optimization
3. ✅ Iterate based on testing results
4. ✅ Deploy to staging for validation
5. ✅ Rollout to production gradually

### Success Criteria

**Phase 1 Complete** when:
- [ ] Both optimizations implemented and tested
- [ ] Token reduction targets met (≥90% input, ≥20% output)
- [ ] Performance targets met (<20ms overhead)
- [ ] All documentation complete
- [ ] Client migration guide published
- [ ] Staging validation successful

**Overall Timeline**: 1-2 weeks (8-12 hours)
**Risk Level**: Low
**Impact**: Critical (highest ROI)

---

**Document Status**: ✅ Planning Complete
**Next Action**: Begin Implementation (Follow this plan)
**Priority**: P0 (Critical)
**Dependencies**: [MCP_OPTIMIZATION_IMPLEMENTATION_ROADMAP.md](./MCP_OPTIMIZATION_IMPLEMENTATION_ROADMAP.md)
**Review Date**: 2026-01-31

---

## References

### Related Documents
- [MCP_OPTIMIZATION_IMPLEMENTATION_ROADMAP.md](./MCP_OPTIMIZATION_IMPLEMENTATION_ROADMAP.md) - Overall roadmap
- [MCP_TOKEN_OPTIMIZATION_RESEARCH.md](./research/MCP_TOKEN_OPTIMIZATION_RESEARCH.md) - Research details
- [MCP_OPTIMIZATION_DOCUMENTATION_STATUS.md](./MCP_OPTIMIZATION_DOCUMENTATION_STATUS.md) - Progress tracking
- [MCP_OPTIMIZATION_STATUS.md](./MCP_OPTIMIZATION_STATUS.md) - Status tracking (Task 3.3)

### Implementation References
- `memory-mcp/src/server/tools/` - Tool implementation
- `memory-mcp/src/common/` - Common utilities
- `memory-core/src/embeddings/` - Semantic service
- `plans/ARCHITECTURE/ARCHITECTURE_CORE.md` - Architecture documentation
