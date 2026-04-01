# Reward Integration

How to merge external signals with internal reward calculations.

## Signal Merging Architecture

```
Internal Reward (70%)
├─ Outcome analysis
├─ Efficiency metrics  
└─ Quality assessment
         │
         ▼
    ┌─────────────┐
    │   Merger    │
    └─────────────┘
         ▲
         │
External Signals (30%)
├─ AgentFS toolcall stats
└─ Other providers
```

## SignalMerger Implementation

```rust
pub struct SignalMerger {
    /// Weight for internal calculation (default: 0.7)
    internal_weight: f32,
    /// Weight for external signals (default: 0.3)
    external_weight: f32,
    /// Minimum confidence for external signals
    min_confidence: f32,
}

impl SignalMerger {
    pub fn with_weights(internal: f32, external: f32) -> Self {
        assert!((internal + external - 1.0).abs() < 0.001,
            "Weights must sum to 1.0");
            
        Self {
            internal_weight: internal,
            external_weight: external,
            min_confidence: 0.5,
        }
    }
    
    /// Merge internal reward with external signals
    pub fn merge(
        &self,
        internal: &RewardScore,
        external_sets: &[ExternalSignalSet],
    ) -> MergedReward {
        // Filter low-confidence signals
        let valid_signals: Vec<_> = external_sets
            .iter()
            .filter(|s| s.confidence >= self.min_confidence)
            .collect();
        
        // Calculate external quality score
        let external_quality = if valid_signals.is_empty() {
            1.0 // No signals = neutral
        } else {
            valid_signals
                .iter()
                .filter_map(|s| s.episode_quality)
                .sum::<f32>() / valid_signals.len() as f32
        };
        
        // Calculate external success rate from tool signals
        let external_success = if valid_signals.is_empty() {
            internal.base // No external data = use internal
        } else {
            let total_samples: usize = valid_signals
                .iter()
                .flat_map(|s| &s.tool_signals)
                .map(|t| t.sample_count)
                .sum();
                
            let weighted_success: f32 = valid_signals
                .iter()
                .flat_map(|s| &s.tool_signals)
                .map(|t| t.success_rate * t.sample_count as f32)
                .sum();
                
            weighted_success / total_samples as f32
        };
        
        // Weighted merge
        let merged_base = internal.base * self.internal_weight 
            + external_success * self.external_weight;
        
        let merged_efficiency = internal.efficiency * self.internal_weight
            + external_quality * self.external_weight;
        
        MergedReward {
            base: merged_base,
            efficiency: merged_efficiency,
            internal_score: internal.total,
            external_score: external_success,
            confidence: if valid_signals.is_empty() { 0.5 } else { 0.8 },
        }
    }
}
```

## Integration Points

### Episode Completion

Update `complete_episode()` in `do-memory-core/src/memory/completion.rs`:

```rust
impl SelfLearningMemory {
    pub async fn complete_episode(
        &self,
        episode_id: Uuid,
        outcome: TaskOutcome,
    ) -> Result<()> {
        // ... existing code ...
        
        // 1. Calculate internal reward (existing)
        let internal_reward = self.reward_calculator.calculate(&episode);
        
        // 2. Fetch external signals (new)
        let external_signals = if let Some(registry) = &self.signal_registry {
            registry.aggregate_signals(&episode).await
        } else {
            vec![]
        };
        
        // 3. Merge signals (new)
        let merger = SignalMerger::with_weights(0.7, 0.3);
        let merged_reward = merger.merge(&internal_reward, &external_signals);
        
        // 4. Store merged reward (update existing)
        episode.reward = Some(RewardScore {
            total: merged_reward.calculate_total(),
            base: merged_reward.base,
            efficiency: merged_reward.efficiency,
            // ... other fields
        });
        
        // ... rest of completion logic ...
    }
}
```

### Pattern Effectiveness

Update pattern effectiveness tracking to include external validation:

```rust
impl PatternEffectiveness {
    /// Update with external signal correlation
    pub fn update_with_external_signal(
        &mut self,
        tool_signal: &ToolSignal,
    ) {
        // Weight internal effectiveness with external success rate
        let external_weight = 0.3;
        let internal_weight = 0.7;
        
        let blended_success_rate = 
            self.success_rate * internal_weight 
            + tool_signal.success_rate * external_weight;
        
        // Update effectiveness score
        self.effectiveness_score = blended_success_rate;
        
        // Track external validation
        self.external_validations += 1;
        self.last_external_validation = Some(Utc::now());
    }
}
```

## Weighting Strategies

### Strategy 1: Balanced (Default)

```rust
SignalMerger::with_weights(0.7, 0.3)
```
- 70% internal reward (outcome, efficiency, quality)
- 30% external signals (AgentFS, other providers)
- Use case: General production usage

### Strategy 2: External Heavy

```rust
SignalMerger::with_weights(0.5, 0.5)
```
- Equal weighting
- Use case: High-confidence external provider with large sample size

### Strategy 3: External Only (Testing)

```rust
SignalMerger::with_weights(0.0, 1.0)
```
- 100% external signals
- Use case: Comparing internal vs external rankings

### Strategy 4: Internal Only (Fallback)

```rust
SignalMerger::with_weights(1.0, 0.0)
```
- 100% internal reward
- Use case: External provider offline or misconfigured

## Confidence-Based Weighting

Dynamic weights based on external signal confidence:

```rust
impl SignalMerger {
    pub fn merge_with_confidence(
        &self,
        internal: &RewardScore,
        external_sets: &[ExternalSignalSet],
    ) -> MergedReward {
        let avg_confidence: f32 = if external_sets.is_empty() {
            0.0
        } else {
            external_sets.iter().map(|s| s.confidence).sum::<f32>() 
                / external_sets.len() as f32
        };
        
        // Dynamic weight: higher confidence = more external weight
        let external_weight = avg_confidence * self.external_weight;
        let internal_weight = 1.0 - external_weight;
        
        // ... merge with dynamic weights
    }
}
```

## Conflict Resolution

When internal and external signals disagree:

```rust
pub enum ConflictResolution {
    /// Prefer external signals (ground truth)
    PreferExternal,
    /// Prefer internal calculation
    PreferInternal,
    /// Average both signals
    Average,
    /// Use threshold to decide
    Threshold { threshold: f32 },
}

impl SignalMerger {
    pub fn resolve_conflict(
        &self,
        internal: f32,
        external: f32,
        strategy: ConflictResolution,
    ) -> f32 {
        let diff = (internal - external).abs();
        
        match strategy {
            ConflictResolution::PreferExternal => external,
            ConflictResolution::PreferInternal => internal,
            ConflictResolution::Average => (internal + external) / 2.0,
            ConflictResolution::Threshold { threshold } => {
                if diff > threshold {
                    // Significant disagreement - prefer external
                    external
                } else {
                    // Minor disagreement - average
                    (internal + external) / 2.0
                }
            }
        }
    }
}
```

## Configuration

### Runtime Configuration

```rust
pub struct ExternalSignalRuntimeConfig {
    /// Globally enable/disable external signals
    pub enabled: bool,
    /// Default internal weight
    pub internal_weight: f32,
    /// Default external weight
    pub external_weight: f32,
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
}

impl Default for ExternalSignalRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            internal_weight: 0.7,
            external_weight: 0.3,
            min_confidence: 0.5,
            conflict_resolution: ConflictResolution::Average,
        }
    }
}
```

## Testing

### Unit Test: Merging Logic

```rust
#[test]
fn test_signal_merging() {
    let merger = SignalMerger::with_weights(0.7, 0.3);
    
    let internal = RewardScore {
        base: 0.8,
        efficiency: 1.2,
        // ...
    };
    
    let external = ExternalSignalSet {
        provider: "agentfs".to_string(),
        tool_signals: vec![ToolSignal {
            tool_name: "web_search".to_string(),
            success_rate: 0.9,
            avg_latency_ms: 150.0,
            sample_count: 100,
            metadata: HashMap::new(),
        }],
        confidence: 0.8,
        // ...
    };
    
    let merged = merger.merge(&internal, &[external]);
    
    // Verify weighted calculation
    assert!(merged.base > 0.8 && merged.base < 0.9);
}
```

### Integration Test: End-to-End

```rust
#[tokio::test]
async fn test_external_signal_integration() {
    // Setup mock provider
    let mock_provider = MockExternalSignalProvider::with_signals(vec![
        ExternalSignalSet {
            provider: "test".to_string(),
            tool_signals: vec![/* ... */],
            confidence: 0.9,
        }
    ]);
    
    // Create memory with provider
    let memory = SelfLearningMemory::new()
        .with_external_signal(Box::new(mock_provider));
    
    // Create and complete episode
    let episode_id = memory.start_episode(/* ... */).await;
    // ... log steps ...
    memory.complete_episode(episode_id, outcome).await.unwrap();
    
    // Verify reward includes external signal
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.reward.is_some());
    // Check external signal influence
}
```
