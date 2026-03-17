# v0.1.22 Feature Documentation

This document describes the high-impact features implemented in v0.1.22 (ADR-044).

## 1. Actionable Recommendation Playbooks

Actionable playbooks bridge the gap between raw patterns and agent actions. They synthesize historical success patterns, reflections, and summaries into step-by-step guidance.

### Key Capabilities
- **Ordered Steps**: Generates logical sequences of tools and actions based on past success.
- **Applicability Rules**: Explicitly states when to apply (and when not to apply) a strategy.
- **Expected Outcomes**: Predicts the result of following the playbook.
- **Confidence Scoring**: Combines multiple signals to provide a reliability metric.

## 2. Recommendation Attribution & Feedback Loop

This feature closes the learning loop by tracking how agents interact with recommendations.

### Key Capabilities
- **Session Tracking**: Records exactly which patterns and playbooks were suggested to an agent.
- **Feedback Collection**: Captures which suggestions were actually applied and what the resulting task outcome was.
- **Effectiveness Ranking**: Uses real-world adoption and success data to improve future rankings.
- **Agent Ratings**: Allows agents to provide qualitative feedback on recommendation quality.

## 3. Episode Checkpoints & Handoff Packs

Enables mid-task progress sharing and seamless context transfer between agents or sessions.

### Key Capabilities
- **Manual Checkpoints**: Create snapshots of in-progress episodes at any step.
- **Handoff Packs**: Synthesized bundles containing lessons learned, salient facts, and suggested next steps.
- **Seamless Resumption**: Start new episodes pre-populated with context from a previous checkpoint.
- **Multi-Agent Flow**: Supports planner-to-executor and executor-to-verifier transitions.

## 4. Storage Consistency Check

A new diagnostic capability for the dual-layer storage system.

### Key Capabilities
- **`memory-cli storage check`**: Verifies that episodes exist and are consistent between the primary database (Turso/SQLite) and the cache (redb).
- **Consistency Reporting**: Identifies missing entries or field mismatches.
