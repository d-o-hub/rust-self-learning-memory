# ProviderConfig Migration Guide

This guide documents the migration from the old `ModelConfig` API to the new `ProviderConfig` API for embedding configuration.

## Overview

The `ProviderConfig` enum provides a type-safe, unified interface for configuring different embedding providers (OpenAI, Mistral, Local, Azure, Custom). It replaces the previous `ModelConfig` struct with provider-specific configuration structs wrapped in an enum.

## Key Changes

| Aspect | Old (ModelConfig) | New (ProviderConfig) |
|--------|-------------------|---------------------|
| **Type** | Single struct | Enum with variants |
| **Provider-specific fields** | Optional fields, runtime checked | Strongly typed per variant |
| **Serialization** | Flat structure | Tagged enum with provider field |
| **Extensibility** | Add fields to struct | Add variants to enum |

## Migration Examples

### 1. Basic Configuration

#### Old (ModelConfig)
```rust
use memory_core::embeddings::ModelConfig;

// OpenAI
let config = ModelConfig::openai_3_small();

// Mistral
let config = ModelConfig::mistral_embed();

// Local
let config = ModelConfig::local_sentence_transformer(
    "sentence-transformers/all-MiniLM-L6-v2",
    384
);
```

#### New (ProviderConfig)
```rust
use memory_core::embeddings::ProviderConfig;

// OpenAI - Convenience constructor
let config = ProviderConfig::openai_3_small();

// Mistral - Convenience constructor
let config = ProviderConfig::mistral_embed();

// Local - Convenience constructor
let config = ProviderConfig::local_default();
// Or with custom model
let config = ProviderConfig::local_sentence_transformer(
    "sentence-transformers/all-MiniLM-L6-v2",
    384
);
```

### 2. Azure OpenAI Configuration

#### Old
```rust
use memory_core::embeddings::ModelConfig;

let config = ModelConfig::azure_openai(
    "my-deployment",
    "my-resource",
    "2023-05-15",
    1536
);
```

#### New
```rust
use memory_core::embeddings::{ProviderConfig, AzureOpenAIConfig};

let config = ProviderConfig::AzureOpenAI(AzureOpenAIConfig::new(
    "my-deployment",
    "my-resource",
    "2023-05-15",
    1536,
));
```

### 3. Custom Provider Configuration

#### Old
```rust
use memory_core::embeddings::ModelConfig;

let config = ModelConfig::custom(
    "custom-model",
    768,
    "http://localhost:1234/v1"
);
```

#### New
```rust
use memory_core::embeddings::{ProviderConfig, CustomConfig};

let config = ProviderConfig::Custom(CustomConfig::new(
    "custom-model",
    768,
    "http://localhost:1234/v1",
));

// With custom endpoint
let config = ProviderConfig::Custom(
    CustomConfig::new("custom-model", 768, "http://localhost:1234/v1")
        .with_endpoint("/custom-embeddings")
);
```

### 4. OpenAI with Custom Dimensions

#### Old
```rust
use memory_core::embeddings::ModelConfig;

let config = ModelConfig::openai_3_small();
// No direct way to customize dimensions
```

#### New
```rust
use memory_core::embeddings::{ProviderConfig, OpenAIConfig};

// With 512 dimensions (instead of default 1536)
let openai_cfg = OpenAIConfig::text_embedding_3_small()
    .with_dimensions(512);
let config = ProviderConfig::OpenAI(openai_cfg);

// Or use convenience constructor for default
let config = ProviderConfig::openai_3_small();
```

### 5. Mistral with Quantization

#### Old
```rust
use memory_core::embeddings::ModelConfig;

let config = ModelConfig::mistral_embed();
// No direct support for quantization settings
```

#### New
```rust
use memory_core::embeddings::{
    ProviderConfig,
    config::mistral::{MistralConfig, OutputDtype}
};

// With Int8 quantization
let mistral_cfg = MistralConfig::codestral_embed()
    .with_output_dimension(512)
    .with_output_dtype(OutputDtype::Int8);
let config = ProviderConfig::Mistral(mistral_cfg);

// Binary embeddings
let binary_cfg = MistralConfig::codestral_binary();
let config = ProviderConfig::Mistral(binary_cfg);
```

### 6. Complete EmbeddingConfig

#### Old
```rust
use memory_core::embeddings::{EmbeddingConfig, ModelConfig};

let config = EmbeddingConfig {
    provider: ModelConfig::openai_3_small(),
    similarity_threshold: 0.7,
    cache_embeddings: true,
    batch_size: 100,
    timeout_seconds: 30,
};
```

#### New
```rust
use memory_core::embeddings::{EmbeddingConfig, ProviderConfig, OpenAIConfig};

let config = EmbeddingConfig {
    provider: ProviderConfig::OpenAI(
        OpenAIConfig::text_embedding_3_small().with_dimensions(512)
    ),
    similarity_threshold: 0.7,
    cache_embeddings: true,
    batch_size: 100,
    timeout_seconds: 30,
};
```

## API Reference

### ProviderConfig Convenience Constructors

```rust
// OpenAI
ProviderConfig::openai_3_small()      // 1536 dims
ProviderConfig::openai_3_large()      // 3072 dims
ProviderConfig::openai_ada_002()      // 1536 dims

// Mistral
ProviderConfig::mistral_embed()       // 1024 dims
ProviderConfig::codestral_embed()     // 1536 dims
ProviderConfig::codestral_binary()    // Binary embeddings

// Local
ProviderConfig::local_default()       // 384 dims (MiniLM)
ProviderConfig::local_sentence_transformer(name, dim)
```

### Enum Variants

```rust
pub enum ProviderConfig {
    Local(LocalConfig),
    OpenAI(OpenAIConfig),
    Mistral(MistralConfig),
    AzureOpenAI(AzureOpenAIConfig),
    Custom(CustomConfig),
}
```

### Common Methods

```rust
// Get effective dimension
let dim = config.effective_dimension();

// Get model name
let name = config.model_name();

// Get optimization config
let opt = config.optimization();

// Validate configuration
config.validate()?;
```

## Serialization

### JSON Format

#### Old (ModelConfig)
```json
{
  "provider": "openai",
  "model_name": "text-embedding-3-small",
  "embedding_dimension": 1536
}
```

#### New (ProviderConfig)
```json
{
  "provider": "openai",
  "model": {
    "model_name": "text-embedding-3-small",
    "dimensions": 1536,
    "encoding_format": "float"
  },
  "optimization": {
    "timeout_seconds": 60,
    "max_retries": 3
  }
}
```

### Code Example

```rust
use memory_core::embeddings::ProviderConfig;

// Serialize
let config = ProviderConfig::openai_3_small();
let json = serde_json::to_string(&config)?;

// Deserialize
let config: ProviderConfig = serde_json::from_str(&json)?;
```

## Pattern Matching

Access provider-specific fields using pattern matching:

```rust
use memory_core::embeddings::ProviderConfig;

fn get_config_details(config: &ProviderConfig) -> String {
    match config {
        ProviderConfig::OpenAI(cfg) => {
            format!("OpenAI: {} dims", cfg.effective_dimension())
        }
        ProviderConfig::Mistral(cfg) => {
            format!("Mistral: {:?} dtype", cfg.output_dtype)
        }
        ProviderConfig::Local(cfg) => {
            format!("Local: {}", cfg.model_name)
        }
        ProviderConfig::AzureOpenAI(cfg) => {
            format!("Azure: {}", cfg.endpoint_url())
        }
        ProviderConfig::Custom(cfg) => {
            format!("Custom: {}", cfg.embeddings_url())
        }
    }
}
```

## Benefits of the New API

1. **Type Safety**: Provider-specific fields are strongly typed
2. **Better Serialization**: Tagged enums provide clear JSON structure
3. **Extensibility**: Easy to add new providers as enum variants
4. **Validation**: Provider-specific validation at compile time
5. **Optimization**: Each provider has tailored optimization settings
6. **Flexibility**: Fine-grained control over provider-specific options

## Migration Checklist

- [ ] Replace `ModelConfig` imports with `ProviderConfig`
- [ ] Update configuration construction calls
- [ ] Add provider-specific config imports if needed
- [ ] Update serialization/deserialization code
- [ ] Test provider-specific functionality
- [ ] Update documentation and examples

## Troubleshooting

### "No variant named 'openai_3_small' found for enum 'ProviderConfig'"

Use the convenience constructor: `ProviderConfig::openai_3_small()` not `ProviderConfig::openai_3_small`.

### "Cannot access field on enum"

Use pattern matching or common methods like `effective_dimension()` and `model_name()`.

### "Missing field 'provider' in JSON"

Ensure JSON includes the provider tag: `{"provider": "openai", ...}`.

## Further Reading

- See `memory-core/examples/embedding_config_refactor.rs` for complete examples
- See `memory-core/src/embeddings/config/provider_config.rs` for implementation details
- See `memory-core/src/embeddings/config/` for provider-specific configurations
