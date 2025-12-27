# Quality Metrics MCP Tool

The Quality Metrics tool provides comprehensive quality monitoring and noise reduction statistics for the PREMem (Pre-Storage Reasoning for Episodic Memory) system.

## Overview

This tool enables users to:
- Track average quality scores over time
- Calculate noise reduction rates (percentage of rejected episodes)
- Analyze quality trends (improving/stable/declining)
- View quality score distribution
- Receive actionable recommendations for quality improvement

## Purpose

Phase 1 of the PREMem integration (see `plans/RESEARCH_INTEGRATION_EXECUTION_PLAN.md`) focuses on quality assessment and noise reduction. This tool provides visibility into how well the system is filtering low-quality episodes and improving memory efficiency.

## Tool Definition

### Name
`quality_metrics`

### Description
Retrieve memory quality metrics and noise reduction statistics from the PREMem system

### Input Schema

```json
{
  "time_range": {
    "type": "string",
    "enum": ["24h", "7d", "30d", "90d", "all"],
    "default": "7d",
    "description": "Time range for metrics calculation"
  },
  "include_trends": {
    "type": "boolean",
    "default": true,
    "description": "Include quality trend analysis over time"
  },
  "quality_threshold": {
    "type": "number",
    "minimum": 0.0,
    "maximum": 1.0,
    "description": "Quality threshold to use (default: 0.7)"
  }
}
```

## Output Format

The tool returns a JSON object with the following structure:

```json
{
  "average_quality_score": 0.75,
  "quality_score_distribution": {
    "0.0-0.3 (Low)": 2,
    "0.3-0.5 (Below Average)": 5,
    "0.5-0.7 (Average)": 15,
    "0.7-0.9 (Good)": 45,
    "0.9-1.0 (Excellent)": 33
  },
  "total_episodes_attempted": 100,
  "episodes_accepted": 75,
  "episodes_rejected": 25,
  "noise_reduction_rate": 25.0,
  "quality_trend": {
    "direction": "improving",
    "recent_scores": [0.70, 0.72, 0.75, 0.78, 0.80],
    "moving_average": 0.75,
    "confidence": 0.85,
    "change_rate": 2.5
  },
  "time_period": "7d",
  "recommendations": [
    "Good average quality score. System is capturing valuable episodes.",
    "Healthy noise reduction rate (25.0%). System is filtering effectively.",
    "Quality trend is improving! Current practices are working well.",
    "PREMem system is actively filtering low-quality episodes. This improves memory efficiency."
  ],
  "quality_threshold": 0.7
}
```

## Usage Examples

### Example 1: Basic Quality Check (Last 7 Days)

```typescript
// Query quality metrics for the past week
const result = await mcp.call_tool("quality_metrics", {
  time_range: "7d",
  include_trends: true
});

console.log(`Average Quality: ${result.average_quality_score}`);
console.log(`Noise Reduction: ${result.noise_reduction_rate}%`);
console.log(`Trend: ${result.quality_trend.direction}`);
```

### Example 2: Monitor Recent Quality (24 Hours)

```typescript
// Check quality for recent episodes
const result = await mcp.call_tool("quality_metrics", {
  time_range: "24h",
  include_trends: false,
  quality_threshold: 0.8
});

if (result.average_quality_score < 0.5) {
  console.warn("Quality has dropped! Check recommendations:");
  result.recommendations.forEach(rec => console.log("- " + rec));
}
```

### Example 3: Long-Term Quality Analysis

```typescript
// Analyze all historical data
const result = await mcp.call_tool("quality_metrics", {
  time_range: "all",
  include_trends: true,
  quality_threshold: 0.7
});

// Visualize distribution
console.log("Quality Distribution:");
for (const [bucket, count] of Object.entries(result.quality_score_distribution)) {
  const bar = "█".repeat(count / 2);
  console.log(`${bucket.padEnd(25)} ${bar} (${count})`);
}
```

### Example 4: Custom Threshold Analysis

```typescript
// Compare different quality thresholds
const thresholds = [0.6, 0.7, 0.8];
for (const threshold of thresholds) {
  const result = await mcp.call_tool("quality_metrics", {
    time_range: "30d",
    quality_threshold: threshold
  });

  console.log(`Threshold ${threshold}:`);
  console.log(`  Accepted: ${result.episodes_accepted}`);
  console.log(`  Rejected: ${result.episodes_rejected}`);
  console.log(`  Noise Reduction: ${result.noise_reduction_rate}%`);
}
```

## Integration with Memory System

The quality metrics tool integrates with the PREMem quality assessment system:

```rust
use memory_core::pre_storage::{QualityAssessor, QualityConfig};
use memory_mcp::mcp::tools::quality_metrics::{QualityMetricsTool, QualityMetricsInput};

// Create quality assessor
let config = QualityConfig::new(0.7);
let assessor = QualityAssessor::new(config);

// Assess an episode
let quality_score = assessor.assess_episode(&episode);

// Query metrics via MCP tool
let tool = QualityMetricsTool::new(memory);
let input = QualityMetricsInput {
    time_range: "7d".to_string(),
    include_trends: true,
    quality_threshold: Some(0.7),
};
let metrics = tool.execute(input).await?;
```

## Understanding the Metrics

### Average Quality Score
- **Range**: 0.0 to 1.0
- **Interpretation**:
  - < 0.5: Low quality, review task execution
  - 0.5-0.7: Average quality
  - 0.7-0.85: Good quality
  - ≥ 0.85: Excellent quality

### Noise Reduction Rate
- **Formula**: `(episodes_rejected / total_episodes) × 100`
- **Interpretation**:
  - < 10%: Very selective, most episodes are high quality
  - 10-30%: Healthy filtering
  - 30-50%: Moderate filtering
  - > 50%: Aggressive filtering, may need to lower threshold

### Quality Trend
- **Direction**:
  - `improving`: Quality scores are increasing over time
  - `stable`: Quality is consistent
  - `declining`: Quality is decreasing (investigate!)
  - `unknown`: Insufficient data
- **Confidence**: 0.0-1.0 (higher = more reliable trend)
- **Change Rate**: Percentage change per time period

### Quality Score Distribution
Shows how episodes are distributed across quality buckets:
- **0.0-0.3 (Low)**: Poor quality, should be rejected
- **0.3-0.5 (Below Average)**: Marginal quality
- **0.5-0.7 (Average)**: Acceptable quality
- **0.7-0.9 (Good)**: High quality episodes
- **0.9-1.0 (Excellent)**: Exceptional episodes

## Recommendations

The tool provides actionable recommendations based on metrics:

### Data Volume Recommendations
- If < 10 episodes: Collect more data for reliable metrics
- If < 50 episodes: Continue building episode history

### Quality Recommendations
- Low average: Review task complexity and execution patterns
- Good average: System is working well
- Excellent average: Memory system is highly effective

### Noise Reduction Recommendations
- Very low (<10%): Consider raising threshold
- Healthy (10-50%): System filtering effectively
- High (>50%): Consider lowering threshold or improving task quality

### Trend Recommendations
- Improving trend: Current practices working well
- Declining trend: Review recent changes
- Stable trend: Maintain current standards

## Best Practices

1. **Regular Monitoring**: Check quality metrics weekly to catch issues early
2. **Threshold Tuning**: Start with 0.7, adjust based on noise reduction rate
3. **Trend Analysis**: Enable trends to understand long-term patterns
4. **Time Range Selection**:
   - `24h`: For immediate feedback
   - `7d`: For weekly reviews
   - `30d`: For monthly analysis
   - `all`: For historical perspective

## Performance Considerations

- Query time scales with episode count
- Trend analysis requires ≥3 episodes
- Large time ranges (90d, all) may take longer
- Consider using shorter time ranges for frequent checks

## Error Handling

The tool handles various error cases gracefully:

- **Invalid time range**: Returns error with valid options
- **No episodes found**: Returns empty metrics with recommendations
- **Insufficient data for trends**: Sets trend to "unknown"

## Related Documentation

- `plans/RESEARCH_INTEGRATION_EXECUTION_PLAN.md` - PREMem integration plan
- `memory-core/src/pre_storage/quality.rs` - Quality assessment implementation
- `TESTING.md` - Testing guidelines for quality metrics

## Future Enhancements

Planned improvements for Phase 2 and beyond:
- Domain-specific quality metrics
- Quality score prediction
- Automated threshold recommendation
- Quality score explanations (why an episode scored X)
- Comparison metrics (current vs. historical baseline)
