# Quality Assessment Guide

This guide explains how to use the Quality Assessment module for pre-storage episode filtering.

## Quick Start

```rust
use memory_core::pre_storage::{QualityAssessor, QualityConfig};
use memory_core::{Episode, TaskContext, TaskType};

// Create assessor with default configuration (threshold: 0.7)
let assessor = QualityAssessor::new(QualityConfig::default());

// Assess an episode
let episode = Episode::new(
    "Implement authentication".to_string(),
    TaskContext::default(),
    TaskType::CodeGeneration,
);

let quality_score = assessor.assess_episode(&episode);
println!("Quality score: {:.2}", quality_score);

// Check if episode should be stored
if assessor.should_store(&episode) {
    println!("Episode passes quality threshold - storing...");
} else {
    println!("Episode rejected - quality too low");
}
```

## Configuration

### Default Configuration

The default configuration uses:
- **Quality Threshold**: 0.7 (episodes scoring below this are rejected)
- **Feature Weights**:
  - Task Complexity: 0.25
  - Step Diversity: 0.20
  - Error Rate: 0.20
  - Reflection Depth: 0.20
  - Pattern Novelty: 0.15

### Custom Threshold

```rust
use memory_core::pre_storage::QualityConfig;

// Higher threshold (more selective)
let high_quality_config = QualityConfig::new(0.85);

// Lower threshold (more permissive)
let low_quality_config = QualityConfig::new(0.5);
```

### Custom Feature Weights

```rust
use memory_core::pre_storage::{QualityConfig, QualityFeature};

let mut config = QualityConfig::default();

// Prioritize task complexity
config.set_weight(QualityFeature::TaskComplexity, 0.4);
config.set_weight(QualityFeature::StepDiversity, 0.15);
config.set_weight(QualityFeature::ErrorRate, 0.15);
config.set_weight(QualityFeature::ReflectionDepth, 0.15);
config.set_weight(QualityFeature::PatternNovelty, 0.15);

let assessor = QualityAssessor::new(config);
```

## Quality Features

### 1. Task Complexity

Evaluates:
- Number of execution steps
- Diversity of tools used

**Scoring**:
- Simple tasks (1-2 steps, 1 tool): 0.0-0.3
- Moderate tasks (3-10 steps, 2-3 tools): 0.3-0.7
- Complex tasks (10+ steps, 4+ tools): 0.7-1.0

### 2. Step Diversity

Evaluates:
- Variety of actions performed
- Mix of success/error results

**Scoring**:
- Repetitive actions: 0.0-0.3
- Some variation: 0.3-0.7
- High diversity: 0.7-1.0

### 3. Error Rate

Evaluates:
- Error occurrence
- Error recovery patterns

**Scoring**:
- High error rate (>60%): 0.0-0.3
- Moderate errors (20-60%): 0.3-0.7
- Low errors or good recovery (<20%): 0.7-1.0

### 4. Reflection Depth

Evaluates:
- Number of successes identified
- Number of improvements suggested
- Number of insights captured

**Scoring**:
- No or minimal reflection (0-2 items): 0.0-0.3
- Basic reflection (3-5 items): 0.3-0.7
- Comprehensive reflection (6+ items): 0.7-1.0

### 5. Pattern Novelty

Evaluates:
- Number of patterns extracted
- Number of heuristics discovered

**Scoring**:
- No patterns: 0.2
- Few patterns (1-2): 0.3-0.5
- Moderate patterns (3-5): 0.5-0.75
- Many patterns (6+): 0.75-1.0

## Tuning Guidelines

### When to Increase Threshold

Increase threshold (0.8-0.9) when:
- Storage space is limited
- You want only highest-quality episodes
- Domain has abundant episode data

### When to Decrease Threshold

Decrease threshold (0.5-0.6) when:
- Storage space is ample
- Domain has sparse episode data
- You need to capture edge cases

### When to Adjust Feature Weights

**Prioritize Complexity** when:
- Learning from sophisticated workflows
- Complex tasks are more valuable

```rust
config.set_weight(QualityFeature::TaskComplexity, 0.4);
```

**Prioritize Reflection** when:
- Insights are critical
- Learning from mistakes is important

```rust
config.set_weight(QualityFeature::ReflectionDepth, 0.35);
```

**Prioritize Error Recovery** when:
- Debugging workflows are common
- Error handling patterns are valuable

```rust
config.set_weight(QualityFeature::ErrorRate, 0.3);
```

## Example Scenarios

### Scenario 1: Storage Space Limited

```rust
// High threshold, prioritize complexity
let mut config = QualityConfig::new(0.85);
config.set_weight(QualityFeature::TaskComplexity, 0.35);
config.set_weight(QualityFeature::StepDiversity, 0.25);
config.set_weight(QualityFeature::ReflectionDepth, 0.25);
config.set_weight(QualityFeature::ErrorRate, 0.10);
config.set_weight(QualityFeature::PatternNovelty, 0.05);
```

### Scenario 2: Learning from Errors

```rust
// Moderate threshold, prioritize error recovery
let mut config = QualityConfig::new(0.65);
config.set_weight(QualityFeature::ErrorRate, 0.35);
config.set_weight(QualityFeature::StepDiversity, 0.25);
config.set_weight(QualityFeature::ReflectionDepth, 0.20);
config.set_weight(QualityFeature::TaskComplexity, 0.15);
config.set_weight(QualityFeature::PatternNovelty, 0.05);
```

### Scenario 3: Insight Capture

```rust
// Moderate threshold, prioritize reflection
let mut config = QualityConfig::new(0.70);
config.set_weight(QualityFeature::ReflectionDepth, 0.40);
config.set_weight(QualityFeature::PatternNovelty, 0.25);
config.set_weight(QualityFeature::TaskComplexity, 0.15);
config.set_weight(QualityFeature::StepDiversity, 0.10);
config.set_weight(QualityFeature::ErrorRate, 0.10);
```

## Integration Example

Integration with `SelfLearningMemory`:

```rust
use memory_core::pre_storage::{QualityAssessor, QualityConfig};
use memory_core::memory::SelfLearningMemory;

impl SelfLearningMemory {
    pub async fn complete_episode_with_quality_check(
        &self,
        episode_id: Uuid,
        outcome: TaskOutcome,
    ) -> Result<()> {
        // Get episode
        let mut episode = self.get_episode(episode_id).await?;

        // Complete episode
        episode.complete(outcome);

        // Assess quality
        let quality_score = self.quality_assessor.assess_episode(&episode);

        // Log quality score
        log::info!(
            "Episode {} quality score: {:.2}",
            episode_id,
            quality_score
        );

        // Check threshold
        if !self.quality_assessor.should_store(&episode) {
            log::info!(
                "Episode {} rejected: quality score {:.2} below threshold {:.2}",
                episode_id,
                quality_score,
                self.quality_config.quality_threshold
            );
            return Ok(());
        }

        // Store high-quality episode
        self.storage.store_episode(&episode).await?;

        Ok(())
    }
}
```

## Metrics and Monitoring

Track quality assessment metrics:

```rust
// Track quality scores over time
let mut quality_scores = Vec::new();

for episode in episodes {
    let score = assessor.assess_episode(&episode);
    quality_scores.push(score);
}

// Calculate statistics
let avg_quality = quality_scores.iter().sum::<f32>() / quality_scores.len() as f32;
let accepted_count = quality_scores.iter().filter(|&&s| s >= 0.7).count();
let rejection_rate = 1.0 - (accepted_count as f32 / quality_scores.len() as f32);

println!("Average quality: {:.2}", avg_quality);
println!("Rejection rate: {:.1}%", rejection_rate * 100.0);
```

## Best Practices

1. **Start with Defaults**: Use default configuration initially
2. **Monitor Metrics**: Track quality scores and rejection rates
3. **Tune Gradually**: Adjust threshold and weights based on data
4. **Domain-Specific**: Different domains may need different configurations
5. **Validate Changes**: Test threshold changes on historical data
6. **Document Rationale**: Keep notes on why you chose specific weights

## Troubleshooting

### Too Many Episodes Rejected

**Symptoms**: >50% rejection rate

**Solutions**:
- Lower threshold (e.g., from 0.7 to 0.6)
- Adjust feature weights to better match your domain
- Check if reflection generation is working properly

### All Episodes Accepted

**Symptoms**: <10% rejection rate

**Solutions**:
- Increase threshold (e.g., from 0.7 to 0.8)
- Verify quality assessment is running
- Check that episodes have diverse quality levels

### Quality Scores Clustered

**Symptoms**: Most scores in narrow range (e.g., 0.6-0.7)

**Solutions**:
- Adjust feature weights for more differentiation
- Check if episodes truly are similar quality
- Consider domain-specific feature weights

## Reference

### API Documentation

Full API documentation is available via:

```bash
cargo doc --package memory-core --open
```

Look for `memory_core::pre_storage` module.

### Configuration Fields

```rust
pub struct QualityConfig {
    /// Minimum quality score (0.0-1.0)
    pub quality_threshold: f32,

    /// Feature weights (should sum to ~1.0)
    feature_weights: HashMap<QualityFeature, f32>,
}
```

### Quality Feature Enum

```rust
pub enum QualityFeature {
    TaskComplexity,   // Steps + tool diversity
    StepDiversity,    // Action variety
    ErrorRate,        // Error handling
    ReflectionDepth,  // Reflection quality
    PatternNovelty,   // Pattern discovery
}
```

---

**Version**: 1.0
**Last Updated**: 2025-12-25
**Module**: `memory_core::pre_storage`
