---
name: recommendation-feedback
description: Record and analyze recommendation sessions and feedback to drive the self-learning loop
---

# Recommendation Feedback

## When to Use

- Recording which patterns/playbooks were recommended to an agent
- Capturing feedback on whether recommendations were useful
- Analyzing recommendation effectiveness statistics
- Closing the self-learning feedback loop

## The Self-Learning Loop

```
1. Agent starts task → query_memory / recommend_patterns
2. System recommends patterns/playbooks → record_recommendation_session
3. Agent completes task → record_recommendation_feedback
4. System adjusts future recommendations based on outcomes
```

## CLI Commands

| Command | Purpose |
|---------|---------|
| `do-memory-cli feedback record-session -e <ID> -p <patterns> -P <playbooks>` | Record what was recommended |
| `do-memory-cli feedback record-feedback -s <SESSION> -a <applied> -o <outcome> -m <msg>` | Record what worked |
| `do-memory-cli feedback stats` | View recommendation statistics |

## MCP Tools

| Tool | Parameters | Purpose |
|------|-----------|---------|
| `record_recommendation_session` | episode_id, patterns, playbooks | Log recommended items |
| `record_recommendation_feedback` | session_id, applied, consulted, outcome, rating | Log what was used |
| `get_recommendation_stats` | - | Aggregate effectiveness stats |

## Recording a Session

When patterns are recommended to an agent:

```bash
do-memory-cli feedback record-session \
  --episode-id "abc-123" \
  --patterns "pat-1,pat-2,pat-3" \
  --playbooks "pb-1"
```

## Recording Feedback

After the episode completes:

```bash
do-memory-cli feedback record-feedback \
  --session "session-uuid" \
  --applied "pat-1,pat-3" \
  --consulted "ep-old-1" \
  --outcome "success" \
  --message "Pattern 1 was directly applicable, pat-2 was irrelevant" \
  --rating 0.8
```

### Outcome Values

| Value | Meaning |
|-------|---------|
| `success` | Task completed successfully using recommendations |
| `partial` | Some recommendations helped, others didn't |
| `failure` | Recommendations were not useful |

## Viewing Stats

```bash
do-memory-cli feedback stats
```

Returns: total sessions, average rating, most-applied patterns, patterns with highest success correlation.

## Integration with Pattern Ranking

Feedback data directly influences `rank_patterns()` in future queries:
- Patterns with high apply+success rates get boosted
- Patterns frequently recommended but never applied get demoted
- This is the core mechanism that makes the memory system self-improving
