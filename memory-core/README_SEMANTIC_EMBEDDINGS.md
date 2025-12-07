# Semantic Embeddings for Memory-Core

This document describes the semantic embedding capabilities added to the memory-core system, enabling semantic similarity search and enhanced context retrieval.

## Overview

The semantic embeddings feature enhances the memory system with vector-based similarity search, allowing agents to find relevant past experiences based on semantic meaning rather than just keyword matching.

## Features

### 🧠 Semantic Search
- **Vector embeddings** for episodes and patterns
- **Semantic similarity search** beyond keyword matching
- **Hybrid search** combining semantic and traditional approaches
- **Multiple embedding providers** (local, OpenAI, custom)

### 🚀 Embedding Providers
- **Local Provider**: Offline sentence transformers (384/768 dims)
- **OpenAI Provider**: Cloud-based embeddings (1536/3072 dims)
- **Custom Providers**: Extensible architecture for new providers

### 📊 Smart Storage
- **Dual storage**: Turso for durability, redb for caching
- **Automatic backfilling** for existing episodes
- **Configurable similarity thresholds**
- **Efficient vector operations**

## Quick Start

### Basic Usage

```rust
use memory_core::{
    SelfLearningMemory,
    embeddings::{EmbeddingConfig, EmbeddingProvider},
    types::{TaskContext, TaskType, TaskOutcome}
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create memory system with semantic capabilities
    let memory = SelfLearningMemory::new();
    
    let config = EmbeddingConfig {
        provider: EmbeddingProvider::Local,
        similarity_threshold: 0.7,
        ..Default::default()
    };
    
    let semantic_memory = memory.with_semantic_search(config).await?;
    
    // Create episodes with rich context
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        domain: "web-api".to_string(),
        tags: vec!["authentication".to_string(), "security".to_string()],
        ..Default::default()
    };
    
    let episode_id = semantic_memory.start_episode(
        "Implement OAuth2 authentication flow".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    ).await;
    
    // ... log steps and complete episode ...
    
    // Semantic search for similar episodes
    let similar_episodes = semantic_memory.semantic_search_episodes(
        "How to implement user login with tokens?",
        &context,
        5
    ).await?;
    
    for result in similar_episodes {
        println!("Found: {} (similarity: {:.2})", 
            result.item.task_description, result.similarity);
    }
    
    Ok(())
}
```

### Advanced Usage

```rust
// Hybrid search combining semantic and traditional
let hybrid_results = semantic_memory.hybrid_search_episodes(
    "implement REST API authentication",
    &context,
    10,
    0.7 // 70% weight to semantic, 30% to traditional
).await?;

// Text similarity comparison
let similarity = semantic_memory.text_similarity(
    "implement user authentication",
    "build login system"
).await?;

// Pattern-based semantic search
let relevant_patterns = semantic_memory.semantic_search_patterns(
    &context,
    5
).await?;

// Backfill embeddings for existing data
let backfilled_episodes = semantic_memory.backfill_episode_embeddings().await?;
let backfilled_patterns = semantic_memory.backfill_pattern_embeddings().await?;
```

## Configuration

### Local Provider (Default)

```rust
let config = EmbeddingConfig {
    provider: EmbeddingProvider::Local,
    model: ModelConfig::local_sentence_transformer(
        "sentence-transformers/all-MiniLM-L6-v2",
        384
    ),
    similarity_threshold: 0.7,
    batch_size: 32,
    cache_embeddings: true,
    ..Default::default()
};
```

### OpenAI Provider

```rust
// Set OPENAI_API_KEY environment variable
let config = EmbeddingConfig {
    provider: EmbeddingProvider::OpenAI,
    model: ModelConfig::openai_3_small(), // or openai_3_large()
    similarity_threshold: 0.8,
    ..Default::default()
};
```

## How It Works

### Episode Embedding

Episodes are converted to searchable text by combining:
- **Task description**: The main task being performed
- **Context**: Domain, language, framework, tags
- **Execution steps**: Tools used and actions taken
- **Outcome**: Success/failure details and artifacts

Example embedding text:
```
"Implement REST API authentication. domain: web-api. language: rust. 
framework: axum. tags: security, authentication. tools used: validator, 
hasher, jwt. actions: validate credentials, hash password, generate token. 
outcome: success - Authentication system implemented"
```

### Pattern Embedding

Patterns are embedded based on:
- **Pattern type and description**: Tool sequences, decision points, etc.
- **Context**: Where the pattern applies
- **Effectiveness metrics**: Success rates and usage statistics

### Similarity Search

1. **Query embedding**: Convert search query + context to vector
2. **Vector similarity**: Calculate cosine similarity with stored embeddings
3. **Filtering**: Apply similarity threshold and relevance filters
4. **Ranking**: Sort by similarity score and return top results

## Performance

| Operation | Local Provider | OpenAI Provider | Storage |
|-----------|---------------|----------------|---------|
| Episode embedding | ~5ms | ~100ms | ~2ms |
| Search (1000 episodes) | ~10ms | N/A | ~5ms |
| Batch processing | ~50ms/100 | ~2s/100 | ~20ms |

## Architecture

```
SemanticMemory
├── EmbeddingProvider (Local/OpenAI/Custom)
├── EmbeddingStorage (Turso/redb/InMemory)
└── SelfLearningMemory (Core system)
```

### Components

- **SemanticService**: Coordinates embedding generation and search
- **EmbeddingProvider**: Converts text to vectors
- **EmbeddingStorage**: Stores and retrieves embeddings
- **SimilaritySearch**: Finds similar vectors efficiently

## Use Cases

### 🔍 Intelligent Context Retrieval
Find relevant past experiences for new tasks:
```rust
// Query: "Need to add rate limiting to API"
// Finds episodes about: rate limiting, API protection, middleware, throttling
```

### 🎯 Pattern Discovery
Discover applicable patterns for current context:
```rust
// Context: Rust + web-api + performance
// Finds patterns: async processing, caching strategies, connection pooling
```

### 🧩 Cross-Domain Learning
Learn from similar tasks in different domains:
```rust
// Query: "Handle large file uploads"
// Finds episodes from: web APIs, mobile apps, data processing pipelines
```

### 📈 Continuous Improvement
Improve recommendations over time:
- Embeddings capture semantic relationships
- Feedback improves future retrievals
- Patterns evolve with new evidence

## Best Practices

### Context Design
Provide rich context for better semantic matching:
```rust
TaskContext {
    language: Some("rust".to_string()),
    framework: Some("axum".to_string()),
    domain: "microservices".to_string(),
    tags: vec![
        "authentication".to_string(),
        "jwt".to_string(),
        "middleware".to_string(),
        "async".to_string()
    ],
    complexity: ComplexityLevel::Moderate,
}
```

### Episode Descriptions
Write descriptive task descriptions:
- ✅ "Implement JWT-based authentication middleware for REST API"
- ❌ "Fix auth"

### Step Documentation
Document execution steps clearly:
```rust
ExecutionStep::new(
    1, 
    "jwt_validator".to_string(), 
    "Validate JWT token signature using HMAC-SHA256 and check expiration".to_string()
);
```

## Future Enhancements

- **Multi-modal embeddings**: Code, documentation, and error messages
- **Dynamic embeddings**: Update embeddings as patterns evolve
- **Federated search**: Search across multiple agent memories
- **Embedding compression**: Reduce storage requirements
- **Custom similarity metrics**: Domain-specific similarity functions

## Contributing

The semantic embeddings system is designed to be extensible. You can:

1. **Add new providers**: Implement `EmbeddingProvider` trait
2. **Custom storage**: Implement `EmbeddingStorageBackend` trait
3. **Similarity functions**: Add new similarity calculations
4. **Model integration**: Support additional embedding models

See the examples in `memory-core/examples/semantic_search.rs` for detailed usage patterns.