# MCP Tools Reference

Please see `docs/API_REFERENCE.md` for the single source of truth on available tools and their schemas.

Below are the primary tools to be aware of:

## Core & Pattern Discovery
- **query_memory**: Search past episodes to inform current tasks
- **search_patterns**: Semantic search across learned patterns
- **recommend_patterns**: Get task-specific pattern recommendations
- **analyze_patterns**: Identify successful strategies based on task type

## Playbooks & Agents
- **recommend_playbook**: Actionable step-by-step guidance derived from successful episodes
- **checkpoint_episode**: Save state for long-running workflows or agent handoffs

## Feedback
- **record_recommendation_session**: Initiate a tracking session for pattern/playbook usage
- **record_recommendation_feedback**: Rate the effectiveness of recommended patterns

## Code Execution
- **execute_agent_code**: Unavailable / fail-closed. This tool will reject calls. Use external execution environments for actual code running.

## Episode Lifecycle
Create, get, complete, log steps, and manage tags/relationships. Use the standard CRUD tools (`create_episode`, `add_episode_step`, `complete_episode`, etc.) documented in `docs/API_REFERENCE.md`.
