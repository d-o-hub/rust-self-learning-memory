# Code Patterns

## Data Structures

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureData {
    pub id: String,
    pub created_at: i64,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub enabled: bool,
    pub max_items: usize,
}
```

## Core Logic

```rust
use anyhow::Result;

pub struct Feature {
    config: FeatureConfig,
}

impl Feature {
    pub fn new(config: FeatureConfig) -> Self {
        Self { config }
    }

    pub async fn process(&self, input: FeatureData) -> Result<FeatureData> {
        // Implementation
        Ok(input)
    }
}
```

## Async Storage

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

pub async fn store(
    pool: &Arc<Mutex<Database>>,
    data: &FeatureData,
) -> Result<()> {
    // Storage implementation
    Ok(())
}
```

## Error Handling

```rust
use anyhow::{Context, Result};

pub async fn risky_operation() -> Result<Value> {
    operation()
        .await
        .context("Failed to perform operation")?
        .validate()
        .context("Validation failed")?
}
```

## Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_feature_process() {
        let config = FeatureConfig { enabled: true, max_items: 100 };
        let feature = Feature::new(config);
        let input = FeatureData { /* ... */ };

        let result = feature.process(input).await;
        assert!(result.is_ok());
    }
}
```
